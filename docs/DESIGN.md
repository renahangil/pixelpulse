# PixelPulse design specification

## Window and interaction

The island is centered on the primary monitor, 8 logical pixels below its top edge. The native window follows the monitor DPI and resizes to the island bounds. Collapsed size is 180 × 36 logical px. Expanded size is 340 × 230, plus 36 px per optional volume. The settings window opens centered at 710 × 520 logical px.

Hover expands after 200 ms and leaving collapses after 400 ms. Click to pin, click again or press ESC with the island focused to unpin. Enter and Space also toggle pinning when the island has focus. The native context menu opens System Diagnostics and exits the process.

## Surfaces and type

- The default surface is opaque `#25252d`, with 4 px CSS corners and a small native corner preference where supported. Opaque rendering gives consistent legibility across desktop backgrounds.
- `PIXELPULSE_GLASS=acrylic` requests Desktop Acrylic for comparison. The tint supplied to window-vibrancy applies to its legacy path. Windows controls its current compositor material and may render it solid. The CSS tint is `rgba(20, 20, 26, 0.3)`.
- `PIXELPULSE_GLASS=fallback` uses `rgba(20, 20, 26, 0.6)` without a blur. The same surface is used if an explicit Acrylic request fails.
- Main text is `#d8d8de`; supporting text is `#8f8fa0`. Use the local Silkscreen face for the wordmark, numbers, metric labels and settings navigation and headings. Supporting copy uses the system font. Pin state and instructions are 10 px. Settings navigation is 13 px and pane titles are 18 px.
- Silkscreen Regular Latin is a single 8,404-byte WOFF2, licensed under SIL Open Font License 1.1. Copyright 2001 The Silkscreen Project Authors. Its local license accompanies the app bundle. SHA-256: `E6C72EA4702249202BCDD79D3343057E4E25EF1F04E3FCFFD8602AB53B40B4CC`. [Upstream font](https://github.com/google/fonts/tree/main/ofl/silkscreen) · [Bundled license](../src/assets/fonts/OFL.txt)

## Pixel grid

One art pixel equals 3 CSS px (`--px`). Use crisp inline SVGs for metric and navigation icons. Rounded corners follow the surface radius.

| Element | Size and layout |
| --- | --- |
| Collapsed dots | Four 6 px dots on fixed 9 px cells separated by 3 px. The winner grows to 9 px and gains a light top edge without shifting its neighbors. |
| Collapsed summary | A 72 px left-aligned label/value field beside the dots. Both groups center as one unit. |
| Expanded rows | 30 px tall with 6 px gaps. Columns are 24 px icon, 30 px label, 189 px bar and 42 px value, with 9 px gaps. |
| Segments | 16 blocks of 9 × 12 px with 3 px gaps. Lit blocks use one art pixel of light shading on top and dark shading on the bottom. |
| Footer | 18 px top margin, leaving the measurement rows as the visual focus. |
| Settings | Two-column layout with 18 px navigation icons and a scrollable content pane. The island expands to fit its selected metrics. |

The expanded bar rounds `percentage × 16 / 100` and clamps it to 0–16. The disabled track uses a subtle `#24242e` / `#282832` checkerboard with top `#30303a` and bottom `#191920`. Each bar reuses its spans and changes only affected classes. Fixed widths keep values aligned. Missing GPU utilization is a dash with an accessible unavailable state. Real zero displays as `0%`. The collapsed winner is chosen from the four main metrics, with ties resolved in display order.

| Metric | Light | Base | Dark |
| --- | --- | --- | --- |
| CPU blue | `#91b0cc` | `#6c8da9` | `#48637a` |
| RAM violet | `#b0a0c8` | `#8e7ba8` | `#645574` |
| GPU green | `#9eb8a0` | `#7c9b80` | `#536c56` |
| Storage amber | `#c5b08a` | `#a58d63` | `#746044` |
| AI coral (reserved) | `#d1a393` | `#b28373` | `#7e594e` |

## Motion and assets

Content uses brief, discrete opacity and transform animations with reduced-motion support. Native resizing fits the actual click area. Avoid glow, heavy shadows, external fonts and runtime asset requests. The [master icon](../src-tauri/icons/pixelpulse.svg) combines a stepped pulse with the four metric colors. The [README banner](img/pixelpulse-banner.svg) reuses the mark and palette as a standalone vector illustration.
