# PixelPulse roadmap

Plan revision 0.12 · First production version 1.0.0

## Release decisions

- Target current, supported Windows 11 x64 editions with WebView2. Keep Tauri 2 and the vanilla TypeScript frontend.
- Use an opaque, rounded island by default. Keep Acrylic and RGBA as opt-in compositor diagnostics. Keep the island topmost through Windows events without taking input focus.
- Display CPU, RAM, GPU and the Windows system volume. Extra volumes are opt-in. Storage values represent occupied capacity. Prefer discrete graphics when identified and use NVML for an unambiguous NVIDIA match or Windows counters when available.
- Keep diagnostics local, startup opt-in, and preferences in the user profile. Ship English (US), Brazilian Portuguese and Spanish.
- Use GPL-3.0 and retain the Silkscreen OFL notice. Merge by pull request and validate the exact main revision before publishing. See the [release checklist](RELEASING.md).

## Progress

| Phase | Outcome |
| --- | --- |
| 0 | Cleaned the template, named the app, defined the release profile and organized the repository. |
| 1 | Built selective Rust collectors, optional NVML, a CLI and initial unit tests. |
| 2 | Added the snapshot cache, 60-sample history, atomic mode, sampler and Tauri event bridge. |
| 3 | Implemented native DPI-aware sizes, hover delays and pin/unpin behavior. |
| 4 | Added the art grid, font, palette, segmented bars and optional compositor material. |
| 5 | Implemented hidden-window suspension, display-aware event deduplication, selective GPU reads and DWM diagnostics. The Rust memory goal remains open. |
| 6 | Closed the visual design, added winning-metric labeling, event-driven topmost recovery, native exit and original icon. Local AI usage was deferred. |
| 7 | Added local inventory, discrete GPU discovery, extra volumes, locale selection and startup options. Validated diagnostics and preference changes on the GT 1030 test PC. Store certification, signed-package update/sign-in and broader GPU coverage remain open. |
| 8 | Prepared the 1.0.0 release branch and CI, moved island help text down 6 px, enlarged settings typography, and updated the public documentation. Repository visibility, merge, final release tag and Store submission are owner-operated steps. |

The English Microsoft Store submission procedure is retained on the phase 7 branch. Git history preserves earlier acceptance reports and release measurements.

## Release candidate validation

Windows x64 release build, Rust formatting, warning-free Clippy and all eight Rust tests passed. WebView2 verified footer spacing, settings typography in all three locales, native context-menu actions, and island resizing with an optional volume. MakeAppx built a 1.0.0.0 local-test package containing the same release executable and license. Signed installation and Store certification remain pending.

## Performance record

Use release builds and report the measurement duration, CPU normalization, counters and units. Targets are under 1% average CPU and under 25 MB resident private memory for the Rust process, and under 120 MB resident private memory including WebView2.

| Historical release measurement | Before optimization | After optimization |
| --- | ---: | ---: |
| Main process normalized average CPU | 0.041% | 0.008% |
| Whole group normalized average CPU | 0.088% | 0.064% |
| Main process resident private memory | 25.39 MB | 25.32 MB |
| Whole group resident private memory | 91.47 MB | 91.33 MB |
| Main process working set | 53.28 MB | 51.37 MB |
| Sum of working sets, including shared pages | 371.52 MB | 372.05 MB |

In the later 35.91-second diagnostics release run with settings closed, normalized average CPU was 0.005% for the main process and 0.065% for the group. Resident private memory was 25.51 MB and 95.75 MB respectively. The owner reported WebView readings of 62 collapsed and 68 expanded and PixelPulse readings of 24.2 and 24.4. The memory counter and units were unspecified. The main process exceeded the literal 25 MB target in controlled runs.

Keep Tauri for this release. If repeat release measurements with the same counter show the strict memory limit is still required, measure the CLI and NVML baseline before deciding whether to prototype Win32/Direct2D. Measure idle, collapsed and expanded on representative systems and verify behavior across monitors, DPI and drivers.

## Future work

- Validate AMD and Intel graphics, hybrid systems, linked adapters and sensor degradation. Confirm signed MSIX installation, update, uninstall and startup on a clean Windows VM. Recheck overlapped-window behavior with real apps and mixed DPI.
- Prototype Linux on Ubuntu 26.04 LTS, then check compatibility on Ubuntu 26.10. Reuse the frontend, cache and portable sysinfo collectors. Introduce platform implementations for mount discovery and labels, GPU telemetry, system inventory, preferences and XDG autostart. NVML is an option on supported NVIDIA Linux drivers; other adapters need Linux-specific accounting.
- Validate native window semantics on GNOME Wayland early. Wayland delegates placement and stacking to the compositor, so exact top-center and always-on-top behavior may require a supported compositor integration or a regular-window fallback. Test scale, pointer interaction and context menus. Build with WebKitGTK and validate a Linux package and CI on the supported baseline. [Ubuntu releases](https://wiki.ubuntu.com/Releases) · [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux)
- Explore local AI usage counters for Claude Code and Codex. Read only token counts and timestamps. Keep prompt and reply content out of the feature.

## Version history

| Plan revision | Date | Record |
| --- | --- | --- |
| 0.12 | 2026-11-20 | Release candidate 1.0.0, Windows CI, documentation in English, release checklist and final settings typography. |
| 0.11 | 2026-11-20 | Diagnostics, startup, volumes, GPU fallback, locales and MSIX preparation. |
| 0.10 | 2026-11-20 | Opaque island, metric identification, z-order, exit and icon. |
| 0.9 | 2026-11-15 | Optimization audit and implementation, compositor trade-offs and owner measurements. |
| 0.8 | 2026-11-15 | Pixel-art graphics, font, optional Acrylic and reduced motion. |
| 0.7 | 2026-11-15 | DPI-aware island, hover and pin interaction. |
| 0.6 | 2026-11-15 | Cache, scheduler and Tauri events. |
| 0.5 | 2026-11-15 | Rust hardware collectors and sample CLI. |
| 0.4 | 2026-11-15 | Template cleanup, folders and release profile. |
| 0.3 | 2026-11-15 | GPL-3.0 adopted. |
| 0.2 | 2026-11-15 | Project name, pixel-art direction and GitHub workflow. |
| 0.1 | 2026-11-15 | Initial project plan. |
