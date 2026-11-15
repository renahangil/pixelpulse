# AGENTS.md — PixelPulse

## Projeto
PixelPulse é um app para Windows 11 Pro: uma «ilha» flutuante no topo central da tela que mostra CPU, RAM, GPU (NVIDIA), SSD e, depois, uso de IA (Claude Code, Codex). Visual pixel art com cores e sombreamentos discretos, sobre um fundo glass escuro. Repositório: github.com/renahangil/pixelpulse. Licença: GPL-3.0.

## Quem usa este repositório
Iniciante em Rust e Tauri. Explique em português, em linguagem simples. Comente o código de forma educativa (o porquê, não o óbvio). Ao fim de cada tarefa, liste cada arquivo criado ou alterado e para que serve.

## Stack
- Rust estável + Tauri 2. UI em HTML, CSS e TypeScript puro (Vite). Sem React, Vue ou similares.
- Métricas: crate sysinfo (CPU, RAM, disco) e nvml-wrapper (GPU NVIDIA, opcional com fallback).
- Efeito glass: crate window-vibrancy (acrylic).
- Fonte pixel: uma fonte bitmap de licença livre (OFL), embutida localmente no projeto.
- Use versões estáveis atuais e confira a documentação oficial antes de usar uma API. Nunca invente APIs, crates ou campos de arquivos.

## Prioridade número 1: leveza
- Meta: processo Rust abaixo de 1% de CPU médio e abaixo de 25 MB de RAM; com WebView2 abaixo de 120 MB no total.
- Amostragem a cada 2 s com o painel expandido e a cada 5 s recolhido. Nada de loops ocupados: use sleep ou timers.
- Só atualize a UI quando ela estiver visível e o valor tiver mudado.
- Antes de adicionar um crate, justifique em uma frase. Evite dependências pesadas.
- Meça sempre em build release.

## Estilo visual (pixel art)
- Grade: 1 pixel de arte = 3 px de tela (constante CSS --px). Tudo alinhado à grade, com image-rendering: pixelated.
- Paleta discreta em src/styles/tokens.css, documentada em docs/DESIGN.md. Cada cor tem 3 tons (claro, base, escuro) para o sombreamento. Sem neon e sem glow.
- Barras segmentadas em blocos de pixel. Animação só com transform e opacity, em passos discretos (steps()).
- Fonte pixel embutida localmente, abaixo de 30 KB. Nenhum recurso baixado em tempo de execução.
- Nada de canvas pesado, GIFs ou imagens grandes.

## Estrutura de pastas
- src-tauri/src/: main.rs, lib.rs, commands.rs, window.rs e as pastas collectors/ (cpu.rs, memory.rs, disk.rs, gpu.rs, mod.rs), cache/ e ai/ (a partir da Fase 6).
- src-tauri/examples/: programas de teste em linha de comando.
- src/: index.html, main.ts, styles/ (tokens.css e demais), ui/ (componentes) e assets/fonts/.
- docs/: PLANO.md, ESTRUTURA.md, DESIGN.md, formatos-logs.md e img/.
- .github/: copilot-instructions.md e pull_request_template.md.
- Raiz: README.md, LICENSE, AGENTS.md.
- Não crie pastas ou arquivos fora dessa estrutura sem registrar em docs/ESTRUTURA.md.

## Regras de código
- Rode cargo fmt e cargo clippy sem warnings antes de commitar.
- Use Result para erros e evite unwrap fora de testes.
- Se a GPU ou o NVML não estiverem disponíveis, o app continua e mostra N/D.
- Privacidade: ao ler dados de uso de IA, leia só contagens e datas (tokens, timestamps). Nunca leia, guarde ou envie o conteúdo de prompts e respostas. Nenhuma chamada de rede sem pedido explícito.
- Nenhum segredo no código.

## Licença
- O projeto é GPL-3.0 (arquivo LICENSE na raiz).
- Ao adicionar uma dependência (crate ou pacote npm), confira se a licença é compatível com a GPL-3.0 (MIT, Apache-2.0 e BSD são; verifique com cargo-license ou cargo deny) e registre em docs/ESTRUTURA.md.
- Só copie código de outra fonte se a licença for compatível, anotando a origem.

## Organização no GitHub
- A branch main sempre compila. Trabalhe na branch fase-N-nome e entre por Pull Request, usando o template.
- Commits em português com prefixo feat:, fix:, docs: ou chore:.
- Tags plano-v0.X a cada fase concluída.

## Fluxo de trabalho
- Uma fase do docs/PLANO.md por vez. Não adiante fases.
- Antes de codar, mostre um plano de 5 linhas. Depois implemente em passos pequenos.
- Ao concluir: rode os checks, atualize docs/PLANO.md (status da fase, versão +0.1 e registro de versões) e faça commit.
- Não refatore fora do escopo. Se achar um problema, registre em Pendências no PLANO.

## Definição de pronto (toda fase)
Compila, fmt e clippy limpos, critérios de aceite atendidos, PLANO atualizado e explicação dos arquivos entregue.