# Instruções para o Copilot — PixelPulse
Siga o AGENTS.md na raiz do repositório. Resumo:
- Projeto: app para Windows 11 Pro em Rust + Tauri 2, UI em TypeScript puro, visual pixel art discreto sobre glass escuro. Licença GPL-3.0.
- Leveza é a prioridade: sem loops ocupados, sem dependências pesadas, UI só atualiza quando visível, animações só em transform e opacity.
- Rust: cargo fmt e cargo clippy sem warnings, Result em vez de unwrap, GPU ausente não pode derrubar o app.
- Privacidade: nunca ler nem guardar o conteúdo de prompts ou respostas de ferramentas de IA, só contagens e datas.
- Licença: não sugira dependências nem trechos de código com licença incompatível com a GPL-3.0.
- Ao revisar: aponte bugs, consumo de CPU e RAM e clareza para iniciante, com arquivo e linha, e sugira a correção mínima.
- Responda em português, em linguagem simples.