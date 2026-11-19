# Design directions

PeriphOS uses the **Tactile** direction (D) as its default interface style: rounded
hardware-like modules, the active profile as a solid block of signal color, a wide display
typeface (Archivo) and indicator LEDs in the navigation.

Other explored directions are kept here as static mocks. The plan is to offer them later as
selectable interface styles in Settings, on top of the existing theme (dark / light / system)
and accent color options.

| Style | File | Summary |
| ----- | ---- | ------- |
| A · Graphite | (first Phase 1 UI, in git history) | Classic text sidebar, quiet bordered panels, Inter. Closest to Linear / Discord. |
| B · Stage | [mocks/b-stage.html](mocks/b-stage.html) | Icon rail plus top tabs, a large central stage for the connected device, electric cyan, Sora. |
| C · Mono | [mocks/c-mono.html](mocks/c-mono.html) | Almost monochrome, typographic, Ctrl+K command bar and numeric shortcuts, IBM Plex. |
| D · Tactile | [mocks/d-tactile.html](mocks/d-tactile.html) | **Current default.** |

Open a mock directly in a browser to view it.

## Logo

The current mark is **Dial** ([logos/dial.svg](logos/dial.svg)): a control knob on a signal
tile. Inside the app it is drawn with theme tokens, so it follows the selected accent color;
the app icon (`public/periphos.svg`, `src-tauri/icons/`) uses the default orange.

Alternative marks are kept for a future appearance selector, next to the interface styles:

| Mark | File |
| ---- | ---- |
| Dial (current) | [logos/dial.svg](logos/dial.svg) |
| P-dot (first logo) | [logos/p-dot.svg](logos/p-dot.svg) |
| Mouse | [logos/mouse.svg](logos/mouse.svg) |
| Keycap | [logos/keycap.svg](logos/keycap.svg) |
| Profile slots | [logos/profile-slots.svg](logos/profile-slots.svg) |
| Signal ring | [logos/signal-ring.svg](logos/signal-ring.svg) |

Regenerate the app icons after changing the mark: render `public/periphos.svg` to a
1024 px PNG and run `npx tauri icon <png> -o src-tauri/icons`.
