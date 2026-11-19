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

## Phase 2: Device management ⏳

- ⏳ Windows-first USB HID enumeration driver (`hid-generic`), designed to also run on Linux (hidraw)
- ⏳ Identify manufacturer, model, VID/PID, interface and connection type
- ⏳ Device connection and disconnection events pushed to the UI
- ⏳ Device detail page with capabilities and their support level
- ⏳ Device database (known VID/PID to model and capabilities)

## Phase 3: Profiles and game detection ⏳

- ⏳ Link profiles to applications: by executable, by picking a running process, and by
  installed Microsoft Store / Game Pass package (no need to browse protected `WindowsApps` folders)
- ⏳ Foreground application watcher and automatic profile switching
- ⏳ Manual profile override
- ⏳ Profiles shared by several devices
- ⏳ Launch at startup and minimize to tray
- ⏳ Releases and updates: signed installers built by CI on GitHub Releases, in-app update check
  and install (Tauri updater). Linux: AppImage updates in-app; distro packages update through the package manager

## Phase 4: Input remapping and macros ⏳

- ⏳ Action model (key, mouse button, media, shortcut, launch app, macro, disabled)
- ⏳ Software remapping layer on Windows (low-level hooks), with Linux (evdev/uinput) in mind
- ⏳ Macro editor and recorder, delays, repeat and toggle modes, per-profile enablement

## Phase 5: Advanced features ⏳

- ⏳ RGB Studio and lighting adapters
- ⏳ First manufacturer-specific drivers (onboard DPI, polling rate, onboard profiles)
- ⏳ **Wireless battery monitor**: battery level of wireless mice, keyboards and headsets,
  with low-battery notifications and a tray indicator
- ⏳ Profile import/export and full backup/restore
- ⏳ Selectable interface styles (Graphite, Stage, Mono) besides the default Tactile style
- ⏳ Community driver/extension model
- ⏳ Linux builds and packaging as a first-class target
