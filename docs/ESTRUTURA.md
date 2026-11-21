# Repository structure

Generated dependencies, test captures, binaries and package staging are excluded from Git. This map covers source-controlled components.

| Path | Purpose |
| --- | --- |
| `AGENTS.md` | Engineering, accessibility, performance and validation rules. |
| `README.md` | Public overview and Windows local test build instructions. |
| `LICENSE` | GPL-3.0 license text. |
| `.gitignore` | Excludes local files, credentials, packages and build output. |
| `.github/copilot-instructions.md` | Repository guidance for Copilot contributions. |
| `.github/pull_request_template.md` | PR summary, validation, acceptance and resource impact prompts. |
| `.github/workflows/ci.yml` | Windows checks for pull requests to and pushes on main. |
| `.vscode/extensions.json` | Recommended editor extensions. |
| `package.json`, `package-lock.json` | npm scripts, dependencies and pinned dependency tree. |
| `tsconfig.json`, `vite.config.ts` | TypeScript and multi-entry Vite build configuration. |
| `src/index.html`, `src/main.ts` | Island markup and interaction, snapshot and visibility logic. |
| `src/settings.html`, `src/settings.ts` | Settings navigation, hardware inventory, preferences and startup UI. |
| `src/styles/tokens.css` | Shared colors, grid spacing and corner radius. |
| `src/styles/window.css`, `src/styles/settings.css` | Island and settings layouts and type. |
| `src/ui/metrics.ts` | Metric rows, segments and collapsed summary rendering. |
| `src/ui/configuration.ts` | TypeScript contracts for the settings commands and data. |
| `src/ui/i18n.ts`, `src/ui/locales.json` | Locale selection and the shared English, Portuguese and Spanish catalog. |
| `src/assets/fonts/` | Local Silkscreen Regular WOFF2 and its OFL notice. |
| `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` | Rust dependencies, locked versions and release profile. |
| `src-tauri/.gitignore` | Excludes Cargo output and generated Tauri schemas. |
| `src-tauri/build.rs`, `src-tauri/tauri.conf.json` | Tauri build entry, app metadata, island settings and bundle configuration. |
| `src-tauri/capabilities/default.json` | Tauri permissions for local app windows. |
| `src-tauri/src/main.rs`, `src-tauri/src/lib.rs` | Native entry point, setup, commands and orderly shutdown. |
| `src-tauri/src/commands.rs` | IPC between Rust and the two webviews. |
| `src-tauri/src/window.rs` | DPI-aware window geometry, centering, compositor modes and native context menus. |
| `src-tauri/src/z_order.rs` | Event-driven Win32 topmost recovery without taking keyboard focus. |
| `src-tauri/src/settings.rs`, `src-tauri/src/startup.rs` | Local diagnostics and preferences, Win32 registry startup and MSIX startup. |
| `src-tauri/src/cache/mod.rs`, `src-tauri/src/cache/sampler.rs` | Shared 60-sample cache, mode, visibility, cadence and deduplicated events. |
| `src-tauri/src/collectors/mod.rs` | Snapshot schema and orchestration of local collectors. |
| `src-tauri/src/collectors/cpu.rs`, `memory.rs`, `disk.rs` | Selective sysinfo refreshes, CPU baseline and storage volume classification. |
| `src-tauri/src/collectors/gpu.rs` | DXCore inventory, adapter selection, NVML and PDH readings. |
| `src-tauri/src/ai/` | Placeholder for a future local usage collector. |
| `src-tauri/examples/coletor_cli.rs` | Standalone sample and process resource diagnostic. |
| `src-tauri/icons/` | Master SVG and generated PNG, ICO and ICNS assets used by Tauri and MSIX. |
| `src-tauri/packaging/AppxManifest.xml` | MSIX identity template, supported Windows family, visual assets and opt-in startup task. |
| `src-tauri/packaging/build-msix.ps1` | Validates the x64 binary and stages the EXE, manifest, logos and license for MakeAppx. |
| `docs/DESIGN.md`, `docs/PLANO.md` | Visual specification, project history, measurements and future work. |
| `docs/RELEASING.md` | GitHub rules, release and public visibility procedure. |
| `docs/img/pixelpulse-banner.svg` | README artwork based on the original PixelPulse mark. |

sysinfo is MIT. window-vibrancy, nvml-wrapper and windows are available under MIT or Apache-2.0. The embedded font retains its separate OFL notice. Confirm licenses for newly introduced dependencies and preserve required notices.
