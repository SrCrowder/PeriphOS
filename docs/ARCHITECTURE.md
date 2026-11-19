# Architecture

This document describes how PeriphOS is organized, why, and which hardware and
operating-system limitations shape the design.

## Technology choices

| Concern            | Choice                         | Why |
| ------------------ | ------------------------------ | --- |
| Desktop shell      | **Tauri 2**                    | Uses the OS webview (WebView2 on Windows, WebKitGTK on Linux): installers of a few MB and far lower memory use than Electron. The backend is Rust, which is where device I/O must live anyway. |
| Backend            | **Rust**                       | Safe low-level access to HID, Win32 and Linux APIs, no garbage-collector pauses for input handling, one language for app and drivers. |
| Frontend           | **React + TypeScript (strict)**| Large contributor pool, mature tooling. |
| Styling            | **Tailwind CSS 4** + own components | Design tokens as CSS variables, no runtime CSS-in-JS. |
| State              | **Zustand**                    | Tiny, no boilerplate, works outside React (e.g. toasts from any module). |
| Persistence        | **SQLite via `rusqlite` (bundled)** | Owned by the Rust side, so the future background profile switcher can read profiles without the UI. Bundled SQLite means no system dependency. |
| IPC                | Tauri commands (events later)  | Typed wrappers in `src/lib/ipc.ts`. |

Alternatives considered: **Electron** (heavier, would still need a native module for HID),
**`tauri-plugin-sql`** (puts SQL in the frontend; rejected because the backend must own
profiles for automatic switching), **Qt/C++** (smaller contributor base, slower UI work).

## Layers

```
┌──────────────────────────── src/ (React) ────────────────────────────┐
│ features/*  pages per domain     components/ui  design system        │
│ stores/*    Zustand state        lib/ipc.ts     typed command bridge │
└───────────────────────────────┬───────────────────────────────────────┘
                                │ Tauri IPC (invoke)
┌───────────────────────── src-tauri/ (thin) ──────────────────────────┐
│ commands/*  one module per domain, errors → { code, message }        │
│ state.rs    AppState: Storage (Mutex) + DeviceManager                │
└──────────────┬───────────────────────────────┬────────────────────────┘
               │                               │
┌──────────────▼─────────────┐   ┌─────────────▼──────────────────────┐
│ crates/periphos-storage    │   │ crates/periphos-core               │
│ SQLite, migrations,        │──▶│ Device, Capability, SupportLevel,  │
│ repositories               │   │ DeviceDriver, DeviceManager,       │
└────────────────────────────┘   │ Profile, AppSettings, CoreError    │
                                 └─────────────▲──────────────────────┘
                                               │ implements DeviceDriver
                                 ┌─────────────┴──────────────────────┐
                                 │ drivers (Phase 2+): hid-generic,   │
                                 │ vendor protocols, ...              │
                                 └────────────────────────────────────┘
```

`periphos-core` has no dependency on Tauri, SQLite or any OS API. This keeps it unit-testable
and lets a future background service (needed so profiles keep switching with the window
closed) reuse it unchanged.

## Device model

A **driver** discovers devices and reports each one as a `DeviceInfo` with a list of
`Capability` values. Each capability has a `SupportLevel`:

| Level            | Meaning |
| ---------------- | ------- |
| `native`         | Written to the device itself through a protocol PeriphOS implements. Survives without PeriphOS running. |
| `software`       | Emulated by PeriphOS at the OS input layer. Only works while PeriphOS runs. |
| `vendorProtocol` | The hardware has it, but it needs a manufacturer protocol PeriphOS does not implement yet. |
| `unsupported`    | Not available. |

Rules:

- Detecting a HID device **does not** mean its onboard settings can be changed. A generic
  driver can identify a device but must report vendor features as `vendorProtocol`.
- The same device may be reported by more than one driver (e.g. generic HID plus a vendor
  driver). Merging those reports by VID/PID/serial is a Phase 2 task.

### Device drivers

```rust
pub trait DeviceDriver: Send + Sync {
    fn info(&self) -> DriverInfo;                      // id, name, supported platforms
    fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>>; // must not change device state
}
```

The trait is intentionally minimal. Methods for reading and writing settings will be added
when the first real driver needs them, rather than guessed in advance. Drivers declare the
platforms they support and `DeviceManager::register` ignores the others, so Windows-only
and Linux-only drivers can coexist. A failing driver never hides the results of the others.

### `hid-generic` (crates/periphos-hid)

The first driver identifies devices through the `hidapi` crate (Windows HID API, Linux
hidraw via udev). It only reads descriptors and never opens a device, so it needs no special
permissions on either platform.

- HID interfaces are grouped into one device by vendor ID, product ID and serial number.
- A group is reported when it exposes a mouse, keyboard, joystick or gamepad top-level
  collection (Generic Desktop usage page), or when it is a known model. Vendor-only devices
  such as RGB controllers are skipped.
- The main type comes from the lowest-numbered input interface (mice often add a keyboard
  interface for macros).
- Devices on a bus `hidapi` does not recognise (not USB, Bluetooth, I2C or SPI) are treated as
  virtual: manufacturer apps create them (Logitech G HUB's virtual keyboard, Corsair iCUE's
  virtual input device). They may reuse a real model's ID, so on an unknown bus a catalogued
  model name is not trusted unless the database marks the entry as virtual. The UI hides
  virtual devices by default.
- `devices.json` maps USB IDs to model names and capabilities. Every entry cites its sources.
  `productIdMask` handles devices whose product ID changes with a mode (Wooting keyboards
  change the last hex digit with the controller emulation mode).
- Because it only identifies hardware, every feature it lists is `vendorProtocol` until a
  manufacturer driver implements it.
- `cargo run -p periphos-hid --example scan` prints what it detects.

The app rescans every two seconds on a background thread (`src-tauri/src/device_watcher.rs`)
and emits a `devices-changed` event when the result changes.

### `logitech-hidpp` (crates/periphos-logitech)

Talks Logitech's HID++ protocol to Logitech receivers to find the devices connected through
them and read their battery. It only sends read requests.

- It opens the receiver's vendor HID collections (usage page `0xFF00`, usages `0x0001` for
  short and `0x0002` for long reports). On Windows each is a separate handle; on Linux one
  hidraw node carries both. `transport.rs` hides the difference.
- Every slot (1 to 6) is pinged; online devices are asked for their name and type (feature
  `0x0005`) and battery (`0x1004`, `0x1000`, `0x1001` or `0x1F20`, whichever they implement).
  The wireless product ID comes from the receiver's pairing register.
- Requests carry software ID `0x09` so replies are not confused with those of other HID++
  software running at the same time (G HUB uses `0x0D`, Solaar `0x0B`, OpenRGB `0x07`).
- Probing takes up to a few seconds, so it runs on a background thread every 10 seconds and
  `enumerate` returns the latest result.
- Protocol logic is tested against simulated devices (`session.rs`, `probe.rs`).
- It was implemented from public descriptions of HID++ (Logitech's feature documentation and
  the Solaar and libratbag projects, whose code is GPL). No code from those projects is used;
  only protocol facts such as message layouts, feature numbers and receiver product IDs.

Known gap: on Linux the kernel's `hid-logitech-dj` driver also exposes devices connected
through a receiver as their own HID devices, so they may be listed twice.

Next drivers: Wooting (open SDKs) and more of HID++ (DPI, report rate).

## Persistence

- One database file, `periphos.db`, in the OS app-data directory
  (`%APPDATA%\io.github.srcrowder.periphos` on Windows, `~/.local/share/io.github.srcrowder.periphos` on Linux).
- Schema version stored in `PRAGMA user_version`; migrations in
  `crates/periphos-storage/src/migrations.rs` are append-only and run in a transaction.
  Opening a database created by a newer PeriphOS fails with a clear message instead of
  corrupting it.
- Invariant enforced in SQL and code: exactly one default profile exists. Profile names
  are unique, case-insensitive.
- Settings are one JSON document with per-field defaults, so older files always load.

## Frontend

- `src/components/ui`: design-system primitives (Button, Panel, Dialog, Field, Switch...).
- `src/components/layout`: application shell.
- `src/features/<domain>`: pages and feature components.
- `src/stores`: Zustand stores; they call `src/lib/ipc.ts`, never `invoke` directly.
- `src/types`: TypeScript mirrors of the Rust types. Keep them in sync by hand for now;
  generating them (e.g. with `ts-rs` or `specta`) is an option once the API grows.
- Themes are CSS variables switched by `data-theme` / `data-accent` attributes on `<html>`.
  The default visual style is "Tactile" ([docs/design](design/README.md)); alternative styles
  are planned as a `data-style` variant over the same tokens, so components never hard-code colors.
- Running `npm run dev` in a browser uses `src/lib/previewBackend.ts`, an in-memory
  implementation of the same commands, shown with a banner. It never reports hardware;
  adding `?fixtures` to the URL (dev server only) shows example devices for UI work.

## Platform and hardware limitations

These constraints are known up-front and drive the roadmap.

### Windows

- **Raw Input can observe but not block input.** Remapping requires a low-level keyboard/mouse
  hook (`WH_KEYBOARD_LL` / `WH_MOUSE_LL`) that swallows the original event and injects a new
  one with `SendInput`. This works per session, does not affect UAC prompts or elevated
  windows unless PeriphOS also runs elevated, and injected input is flagged as such.
- **Anti-cheat.** Several games block or flag injected input and hooks. Software remapping
  and macros may not work, and could be considered a violation, in some titles. PeriphOS will
  warn and never try to hide synthetic input. Onboard (`native`) settings are unaffected.
- **Exclusive HID access.** Windows owns mice and keyboards as system devices; their top-level
  input collections cannot be opened for writing. Vendor features are usually exposed on a
  separate vendor-defined HID interface, which can be opened. Some devices need a driver
  filter for full remapping; PeriphOS will not ship kernel drivers in early versions.
- **Microsoft Store / Game Pass games** live under `C:\Program Files\WindowsApps`, which users
  cannot browse. PeriphOS will identify them by package (Package Family Name / AUMID via the
  Windows packaging APIs) and by picking a running process, instead of asking for an `.exe`.
- **Vendor software conflicts.** Running G HUB, Synapse, iCUE, etc. at the same time may
  overwrite settings PeriphOS writes to the device.

### Linux

- **Device permissions.** Access to `/dev/hidraw*` requires udev rules; PeriphOS will ship them
  rather than run as root.
- **Remapping** goes through evdev (grab the device) and uinput (emit events), which also needs
  permissions on `/dev/uinput`.
- **Foreground window detection** works on X11, but Wayland deliberately hides it; automatic
  profile switching will need compositor-specific integrations (e.g. KDE/GNOME extensions) and
  may be unavailable on some desktops.
- Tauri uses WebKitGTK on Linux, so UI features must be tested on WebKit, not only Chromium.

### General

- Most advanced features (DPI, polling rate, onboard memory, lighting, battery level) are
  **proprietary protocols**, different per manufacturer and often per product line. Support
  will grow device family by device family.
- Bluetooth devices often expose fewer features than the same device over USB or a receiver.
