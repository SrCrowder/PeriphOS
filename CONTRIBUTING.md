# Contributing to PeriphOS

Thanks for helping build an open alternative to OEM peripheral software.

## Ground rules

- **Never fake support.** A feature is only shown as working when it really works on the
  hardware. Planned features are disabled and labeled as such.
- **Keep the core hardware-agnostic.** OS-specific and vendor-specific code lives in
  drivers, never in `periphos-core`.
- **Think cross-platform.** Windows is the first target, but Linux must stay possible.
  Isolate platform code behind `#[cfg(target_os = "...")]` and explain any limitation.
- **Small, focused pull requests** are easier to review than large ones.

## Development setup

See the [README](README.md#getting-started). Before opening a pull request, run:

```bash
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Code style

- **TypeScript:** strict mode, no `any`. Shared types live in `src/types` and must mirror
  the Rust types they represent. Feature code goes in `src/features/<feature>`, reusable
  UI in `src/components/ui`.
- **Rust:** `cargo fmt` and clippy clean. Business rules belong in `periphos-core`,
  persistence in `periphos-storage`; Tauri commands stay thin.
- **Database:** never edit a released migration; append a new one.
- **UI text:** sentence case, plain verbs, buttons say exactly what they do.

## Commits

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(devices): enumerate USB HID devices on Windows
fix(profiles): reject duplicate names regardless of case
docs: explain support levels
```

Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`, `style`, `perf`.

## Adding device support

Device support is added as a driver implementing `periphos_core::device::DeviceDriver`.
See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#device-drivers). When contributing a
protocol, document where the knowledge comes from (your own captures, public
documentation, or an open-source project with a compatible license). Do not contribute
code copied from proprietary software.

### Adding a model to the device database

Known models live in `crates/periphos-hid/devices.json`. To add one:

1. Run `cargo run -p periphos-hid --example scan` with the device connected to get its USB ID.
2. Add an entry with its name, type, capabilities and the `sources` you used (manufacturer
   specification pages are best). Leave out details you cannot source.
3. Use `"support": "vendorProtocol"` for hardware features: the generic driver cannot change
   them. Only a driver that really implements a feature may report `native` or `software`.
4. Run `cargo test -p periphos-hid`.

If you do not write code, opening a **Device support request** issue with your device's
details is still very helpful.
