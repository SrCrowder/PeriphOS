# Roadmap

PeriphOS is built in small, working milestones. This file is the source of truth for
what exists today. Legend: ✅ implemented, 🟡 partially implemented, ⏳ planned.

## Phase 1: Foundation (v0.1) ✅

- ✅ Tauri 2 + React + TypeScript + Rust workspace
- ✅ Hardware-agnostic core crate (`periphos-core`) with device, capability and driver abstractions
- ✅ SQLite persistence with versioned migrations (`periphos-storage`)
- ✅ Application shell, sidebar navigation, reusable design system ("Tactile" style, see [docs/design](docs/design/README.md))
- ✅ Dark theme by default, light and system themes, accent colors, reduced motion
- ✅ Dashboard with active profile, recently edited profiles, devices and system status
- ✅ Devices page with honest empty state and support-level legend
- ✅ Profiles: create, rename, describe, delete, choose the default profile, saved locally
- ✅ Settings page (appearance settings persisted; other options shown as planned)
- 🟡 "Active profile" is always the default profile until automatic switching exists

## Updates ✅

- ✅ In-app update check and install, with Stable and Development channels ([docs/RELEASING.md](docs/RELEASING.md))
- ✅ Signed Windows installers built by CI on GitHub Releases
- ⏳ Windows code signing (removes the SmartScreen warning)
- ⏳ Linux AppImage updates

## Phase 2: Device management ✅

- ✅ Generic HID driver (`hid-generic`) for Windows and Linux: identifies mice, keyboards,
  controllers and known receivers without changing any device setting
- ✅ Manufacturer, model, USB ID, device type and connection type (USB, Bluetooth, receiver)
- ✅ Connection and disconnection pushed to the UI (2-second polling)
- ✅ Device detail page with each feature and its support level
- ✅ Device database with sources (`crates/periphos-hid/devices.json`): Logitech G502 X family,
  LIGHTSPEED and Unifying receivers, Wooting keyboards (80HE with full feature list), Corsair M75
- ✅ Virtual devices created by other software (e.g. the G HUB virtual keyboard) are recognised
  and hidden by default
- 🟡 Two identical devices without a serial number are shown as one
- ⏳ Native hotplug notifications (WM_DEVICECHANGE on Windows, udev monitor on Linux)

## Logitech HID++ driver 🟡

- ✅ Devices connected through Logitech receivers (Unifying, LIGHTSPEED, Nano, Bolt) are
  identified by name and type
- ✅ Battery level and charging state (unified battery, battery status, voltage and ADC features)
- ✅ Feature list of the identified model from the database (matched by the name the device reports)
- ✅ Read and change DPI and report rate; every change is validated against the values the
  device accepts and read back to confirm it took effect
- ✅ Devices that stop answering for a moment (asleep to save battery) stay listed for 60 seconds
- ⏳ Save DPI and report rate in PeriphOS profiles and apply them on profile switch
- ⏳ Switch onboard memory mode on and off
- ⏳ Devices connected by cable and over Bluetooth
- ⏳ Linux: avoid listing a receiver-connected device twice (the kernel also exposes it)

## Phase 3: Profiles and game detection ⏳

- ⏳ Link profiles to applications: by executable, by picking a running process, and by
  installed Microsoft Store / Game Pass package (no need to browse protected `WindowsApps` folders)
- ⏳ Foreground application watcher and automatic profile switching
- ⏳ Manual profile override
- ⏳ Profiles shared by several devices
- ⏳ Launch at startup and minimize to tray

## Phase 4: Input remapping and macros ⏳

- ⏳ Action model (key, mouse button, media, shortcut, launch app, macro, disabled)
- ⏳ Software remapping layer on Windows (low-level hooks), with Linux (evdev/uinput) in mind
- ⏳ Macro editor and recorder, delays, repeat and toggle modes, per-profile enablement

## Phase 5: Advanced features ⏳

- ⏳ RGB Studio and lighting adapters
- ⏳ First manufacturer-specific drivers (onboard DPI, polling rate, onboard profiles):
  Logitech HID++ (documented by the Solaar and libratbag projects) and Wooting (open SDKs)
- 🟡 **Wireless battery monitor**: battery level is shown for Logitech wireless devices;
  low-battery notifications, a tray indicator and other manufacturers are planned
- ⏳ Profile import/export and full backup/restore
- ⏳ Appearance selector: interface styles (Graphite, Stage, Mono) and logo marks besides the default Tactile style and Dial logo
- ⏳ Community driver/extension model
- ⏳ Linux builds and packaging as a first-class target
