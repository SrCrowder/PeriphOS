<p align="center">
  <img src="public/periphos.svg" width="72" alt="" />
</p>

<h1 align="center">PeriphOS</h1>

<p align="center">
  Forget about the OEM software and start managing your peripherals with ease.<br/>
  An open-source, universal control center for mice, keyboards and other PC peripherals.
</p>

---

> **Status: early development.** PeriphOS detects mice, keyboards and controllers, reads the
> battery of wireless Logitech devices, changes DPI and report rate on Logitech mice, switches
> Wooting keyboard profiles, and switches profiles automatically per game. Logitech G mouse
> buttons can be reassigned. Macros and lighting are not available yet. See the [roadmap](ROADMAP.md).

## Supported devices

What PeriphOS can do today, per device. **Tested** means confirmed on real hardware;
**expected** means the device uses the same protocol but nobody has confirmed it yet. Reports
for untested devices are very welcome (see [Contributing](#contributing)).

| Device | Connection | Battery | DPI | Report rate | Buttons | Onboard profiles | Status |
| ------ | ---------- | :-----: | :-: | :---------: | :-----: | :--------------: | ------ |
| Logitech G502 X PLUS | LIGHTSPEED receiver | ✅ | ✅ | ✅ | ✅ ² | Detected, not switched | Tested |
| Logitech G502 X LIGHTSPEED | LIGHTSPEED receiver | ✅ | ✅ | ✅ | ✅ ² | Detected, not switched | Expected |
| Other Logitech mice and keyboards (HID++ 2.0) | Unifying, LIGHTSPEED, Nano or Bolt receiver | ✅ | ✅ ¹ | ✅ ¹ | ✅ ² | Detected, not switched | Expected |
| Logitech devices on a USB cable | USB | — | — | — | — | — | Detected only |
| Wooting 80HE | USB | n/a | n/a | — | In Wootility | ✅ Switches the 4 profiles stored on the keyboard | Tested |
| Other Wooting keyboards (Two HE, 60HE family; older One and Two) | USB | n/a | n/a | — | In Wootility | ✅ Same as the 80HE | Expected |
| Any other mouse, keyboard or controller | USB, Bluetooth | — | — | — | — | — | Detected only |

¹ When the device reports the setting; PeriphOS only offers values the device accepts.<br/>
² Mice that keep their buttons in onboard profiles (Logitech G series): mouse buttons, keys and
shortcuts, media keys and DPI functions, stored on the mouse, with factory and personal
defaults to go back to. Macros are not supported yet.

Every device works with PeriphOS profiles, which switch automatically when a game or app
comes to the front. On Logitech devices, close G HUB while using PeriphOS; on Wooting
keyboards, close Wootility: both apps take over the device. Actuation points, rapid trigger
and lighting for Wooting keyboards are still set in Wootility and stored in each onboard
profile.

Battery levels can be shown as text on the Windows taskbar, as tray icons, and as
low-battery notifications.

## Why

Every manufacturer ships its own heavy, account-driven control panel: Logitech G HUB,
Razer Synapse, Corsair iCUE, SteelSeries GG... PeriphOS aims to replace them with one
lightweight application that:

- manages devices from different manufacturers in a single interface;
- remaps buttons and keys, runs macros, and controls DPI, polling rate and lighting
  **where the hardware actually allows it**;
- switches profiles automatically when a game or application starts, including
  Microsoft Store / Game Pass titles;
- is honest about support: every feature is labeled as stored on the device, applied in
  software, needing a vendor protocol, or unavailable.

## Platforms

| Platform          | Status                                                    |
| ----------------- | --------------------------------------------------------- |
| Windows 10 / 11   | Primary target                                            |
| Linux             | High priority, the architecture is built to support it    |
| macOS             | Not planned for now; contributions welcome                |

## Download

- **Development builds** (every change, for testing): the
  [`nightly` prerelease](https://github.com/SrCrowder/PeriphOS/releases/tag/nightly).
- **Stable releases**: [Releases](https://github.com/SrCrowder/PeriphOS/releases/latest)
  (none published yet).

Install once; after that PeriphOS updates itself from Settings > Updates. The installers are not
code-signed yet, so Windows SmartScreen asks for confirmation: choose **More info**, then
**Run anyway**.

## Getting started

### Prerequisites

- [Node.js](https://nodejs.org/) 20 or newer
- [Rust](https://rustup.rs/) (stable)
- The Tauri 2 system prerequisites for your OS:
  [tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/)
  - **Windows:** Microsoft C++ Build Tools and WebView2 (preinstalled on Windows 11).
  - **Debian/Ubuntu:** `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`

### Run

```bash
npm install
npm run tauri dev      # desktop app with hot reload
```

### Other commands

```bash
npm run dev            # UI only, in the browser (in-memory preview backend, nothing is saved)
npm run build          # type-check and build the frontend
npm run tauri build    # build the installable desktop app
cargo test --workspace # run the Rust test suite
cargo clippy --workspace --all-targets
```

## Project layout

```
crates/periphos-core     Hardware-agnostic domain: devices, capabilities, drivers, profiles, settings
crates/periphos-storage  SQLite persistence with versioned migrations
crates/periphos-hid      Generic HID device detection and the known-device database
crates/periphos-logitech Logitech HID++ driver (receivers, battery, DPI, report rate)
crates/periphos-wooting  Wooting keyboard driver (onboard profile switching)
crates/periphos-apps     Foreground app and installed game detection for automatic profiles
crates/periphos-taskbar  Battery text readout on the Windows taskbar
src-tauri/               Desktop app: Tauri setup and IPC commands (thin layer)
src/                     React + TypeScript frontend
docs/                    Architecture, design notes and release process
```

Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design, technical decisions
and known hardware/OS limitations.

## Contributing

Contributions are welcome, especially device protocol knowledge. Start with
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)

PeriphOS is an independent project and is not affiliated with or endorsed by any
peripheral manufacturer. Product names are trademarks of their respective owners.
