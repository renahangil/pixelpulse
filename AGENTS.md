# AGENTS.md — PixelPulse

## Project

PixelPulse is a Windows 11 floating system monitor with a pixel-art island. It displays CPU, RAM, GPU and storage usage. The repository is [renahangil/pixelpulse](https://github.com/renahangil/pixelpulse) and the project is licensed under GPL-3.0.

## Working with maintainers

Explain changes clearly in the user's language and describe each file created or changed. Write source comments and repository documentation in English. Explain decisions and invariants in code comments.

## Stack

- Stable Rust and Tauri 2 with plain HTML, CSS and TypeScript built by Vite.
- sysinfo for CPU, RAM and storage, NVML for NVIDIA and Windows DXCore/PDH for GPU discovery and fallback.
- WebView2 with an opaque default surface. Optional compositor diagnostics use window-vibrancy.
- One locally embedded OFL font. Check official documentation for APIs and use stable versions.

## Performance

- Release targets: average main-process CPU below 1%, main-process resident private memory below 25 MB, and combined resident private memory below 120 MB including WebView2. Record the counter and units.
- Sample every 2 seconds expanded and every 5 seconds collapsed. Use blocking waits or timers.
- Pause sensor sampling when hidden. Update visible UI only when a rendered value changes.
- Justify any new dependency and check its GPL-3.0 compatibility. Measure changes in a release build.

## Design and structure

Use the [design specification](docs/DESIGN.md) for the 3 px art grid, colors, segment shading, local font, inline SVG icons and reduced-motion rules. Keep assets small and local. Keep the [repository map](docs/STRUCTURE.md) current when adding files or folders.

## Code and privacy

- Use Result for recoverable errors and avoid unwrap outside tests.
- Keep unavailable readings optional. Hardware or driver limitations must leave the rest of the monitor usable.
- Store only intended preferences. Keep diagnostics on the device. Future local AI usage collection must only access counts and timestamps and must skip prompt/response content.
- Keep secrets out of source and build outputs. Make external calls only within the user's authorized scope.
- Check new dependency licenses with cargo-license or a comparable tool. Record source and licensing for any reused assets or code.

## Git and validation

- Keep main buildable. Work on a focused phase branch and merge through a pull request using the template.
- Write commit messages in Portuguese with feat:, fix:, docs: or chore: prefixes. Use the release procedure for production tags.
- Before coding, share a five-line plan. Keep changes within scope and record future work in the roadmap.
- Run cargo fmt, cargo clippy without warnings, cargo test and a Windows release build. Update the roadmap and document validation gaps before committing.
