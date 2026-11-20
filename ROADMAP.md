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
- ⏳ Switch onboard memory mode on and off
- ⏳ Devices connected by cable and over Bluetooth
- ⏳ Linux: avoid listing a receiver-connected device twice (the kernel also exposes it)

## Wooting driver 🟡

- ✅ Wooting keyboards listed by their own driver (`wooting`)
- ✅ Switch between the four profiles stored on the keyboard; the choice is saved per PeriphOS
  profile, so each game can use a different keyboard profile (actuation, rapid trigger, key
  bindings and lighting are set up in Wootility)
- ✅ In-app guide and a warning while Wootility is running
- ⏳ Basic lighting control (a fixed colour per profile)
- ⏳ Reading analog key values

## Phase 3: Profiles and game detection 🟡

- ✅ Device changes are saved straight into the active profile
- ✅ Pick the active profile from the sidebar; activating one by hand applies its settings to
  connected devices
- ✅ When a device connects (or comes back after being switched off and losing its settings),
  the active profile's settings are applied to it

- ✅ Link games and apps to profiles: pick an open application (Microsoft Store / Game Pass
  games are linked by package, no need to browse protected `WindowsApps` folders) or choose an .exe
- ✅ Automatic switching on Windows: the profile linked to the app in front activates, the default
  profile otherwise
- ✅ Manual override: activating a profile pauses automatic switching until it is resumed
- ✅ Pick installed Microsoft Store / Game Pass games without starting them (games first, all
  apps on request)
- ✅ Warn when other software that also controls a device is running (Logitech G HUB, Wootility)
- ⏳ Automatic switching on Linux (X11 first; Wayland needs desktop-specific support)
- ⏳ Profiles shared by several devices
- ✅ Keep running in the system tray when the window is closed (single instance)
- ✅ Start when the user signs in, straight into the tray
- ✅ Tray tooltip and menu show the active profile and battery levels; switch profiles or
  return to automatic switching from the tray menu

## Phase 4: Input remapping and macros ⏳

- ✅ Action model: mouse button, key with modifiers, media key, device function (DPI, wheel
  tilt, onboard profile), disabled; assignments PeriphOS cannot edit yet are kept
- ✅ Read-only technical report on device pages (HID++ features, controls, onboard memory),
  used to add support for new devices from real data
- ✅ Logitech G button remapping stored in the mouse's onboard profile (HID++ 0x8100): read,
  modify one assignment, CRC, write, read back, undo on mismatch; never removes the last
  left click. Works without PeriphOS running. Confirmed on a G502 X PLUS
- ✅ Button diagram for the G502 X family (original drawing, a label per button), list view
  for other mice; restore factory assignments (read from the mouse's built-in profile) or
  the user's own saved defaults per device; restore a single button from the dialog
- ⏳ Logitech onboard profile slots per PeriphOS profile (switch slots instead of rewriting
  memory, which wears flash), G-shift layer, macros stored on the mouse
- ⏳ Macros through PeriphOS for devices without onboard macros. The G502 X family has no
  control diversion (HID++ 0x1B04), so its software macros need the mouse button spy
  (0x8110) or a low-level hook
- ⏳ Software remapping layer on Windows (low-level hooks), with Linux (evdev/uinput) in mind
- ⏳ Macro editor and recorder, delays, repeat and toggle modes, per-profile enablement

## Phase 5: Advanced features ⏳

- ⏳ RGB Studio and lighting adapters
- ⏳ First manufacturer-specific drivers (onboard DPI, polling rate, onboard profiles):
  Logitech HID++ (documented by the Solaar and libratbag projects) and Wooting (open SDKs)
- 🟡 **Wireless battery monitor**: battery level for Logitech wireless devices, low-battery
  notifications with two configurable levels (sent once per discharge), levels in the tray
  tooltip and menu; other manufacturers are planned
- ✅ **Battery on the taskbar** (default, Windows): an icon and the charge of each wireless device
  as text next to the tray, in the spirit of TraySense; hidden during full-screen games.
  Alternatives: one tray icon per device, or off
- ⏳ More taskbar readout styles and items (for example the active profile)
- ⏳ Profile import/export and full backup/restore
- ⏳ Languages: choose the app language in Settings (English first, Spanish next; translations
  contributed by the community)
- ⏳ Appearance selector: interface styles (Graphite, Stage, Mono) and logo marks besides the default Tactile style and Dial logo
- ⏳ Community driver/extension model
- ⏳ **Game controllers**: battery level of wireless controllers (e.g. Xbox), and button
  remapping and macros for controllers where possible
- ⏳ **Report a device**: a button for devices that are unknown or not well supported, which
  prepares a report with what is needed to support the model (IDs, interfaces, descriptors),
  shown to the user before anything is sent
- ⏳ **Custom themes**: create and switch user-made themes (colours, type, shapes and more), with
  a theme guide written so it can be handed to an AI agent to build a theme
- ⏳ Linux builds and packaging as a first-class target
