# PixelPulse contributor instructions

Read [AGENTS.md](../AGENTS.md) and the [roadmap](../docs/ROADMAP.md) before editing. Keep changes focused and preserve the Windows 11 release behavior.

- Use stable Rust, Tauri 2 and plain TypeScript, HTML and CSS. Reuse the existing collectors and shared settings state.
- Keep the sample loop blocking between updates. Only refresh necessary sensors and update visible UI when its rendered values change.
- Follow the [design specification](../docs/DESIGN.md) for pixel-grid spacing, contrast, local assets and reduced motion. Preserve all shipped UI locales.
- Avoid secrets, telemetry and unnecessary dependencies. Confirm external APIs in official documentation and check license compatibility.
- Write comments and documentation in English. Explain why the code works this way, especially around native APIs and shared state.
- Verify a Windows release build, cargo fmt, Clippy with warnings denied and unit tests before submitting.
