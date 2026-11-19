<p align="center">
  <img src="public/periphos.svg" width="72" alt="" />
</p>

<h1 align="center">PeriphOS</h1>

<p align="center">
  Forget about the OEM software and start managing your peripherals with ease.<br/>
  An open-source, universal control center for mice, keyboards and other PC peripherals.
</p>

---

> **Status: early development.** PeriphOS detects mice, keyboards and controllers, identifies
> known models and shows what each one can do, finds devices connected through Logitech
> receivers and shows their battery, and manages profiles and settings. It **does not change
> any device setting yet**. See the [roadmap](ROADMAP.md).

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
crates/periphos-logitech Logitech HID++ driver (receivers, battery)
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
