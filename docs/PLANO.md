# Plano — PixelPulse
Versão: 0.7 (2026-11-15)
Repositório: github.com/renahangil/pixelpulse

## Decisões
- Nome do app: PixelPulse. Rust + Tauri 2, UI web leve, Windows 11 Pro.
- v1: visual pixel art com cores e sombreamento discretos sobre glass escuro; barras em blocos de pixel, cada métrica com uma cor.
- Expansão: hover (200 ms) mais clique para fixar.
- SSD na v1 = percentual de espaço usado do disco do sistema. Atividade de leitura e escrita fica para depois.
- IA na v1 = tokens do dia lidos de logs locais (Claude Code e Codex). Copilot e APIs ficam para depois.
- Repositório privado na conta @renahangil. A main sempre compila; trabalho em branches fase-N-nome com Pull Request.
- Licença: GPL-3.0 (software livre com copyleft: quem distribuir versões modificadas deve manter o código aberto sob a mesma licença). Arquivo LICENSE na raiz; dependências devem ser compatíveis.

## Fases
| # | Fase | Status |
| --- | --- | --- |
| 0 | Ambiente, repositório e estrutura | concluída |
| 1 | Coletor de métricas (Rust) | concluída |
| 2 | Cache e ponte com a UI | concluída |
| 3 | Janela estilo ilha | concluída |
| 4 | Visual pixel art com glass discreto | pendente |
| 5 | Medição e otimização | pendente |
| 6 | IA local (Claude Code, Codex) | pendente |
| 7 | Acabamento (bandeja, autostart, config) | pendente |
| 8 | README e publicação | pendente |

## Fase 0 — critérios de aceite e validação

- [x] Exemplo greet removido do Rust e da UI; página estática com o texto PixelPulse.
- [x] productName, título da janela e título HTML definidos como PixelPulse.
- [x] Perfil release configurado e comentado: opt-level = 's', lto = true, codegen-units = 1, panic = 'abort' e strip = true.
- [x] Oito pastas solicitadas preservadas com `.gitkeep`, sem implementar fases futuras.
- [x] `.gitignore` cobre target, node_modules, dist e arquivos de sistema.
- [x] README provisório, template de PR e mapa em `docs/ESTRUTURA.md` criados.
- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passou.
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passou sem warnings.
- [x] `npm run tauri build`: passou, incluindo TypeScript, Vite, Rust release e instaladores MSI/NSIS.
- [x] Tamanho do executável release medido após o build.

Executável: `src-tauri/target/release/pixelpulse.exe` — 3278848 bytes (3.127 MiB).
A versão do aplicativo permanece 0.1.0; 0.4 é a versão deste plano.
CPU e RAM em execução não foram medidos nesta fase; a medição e otimização estão previstas na Fase 5.
Nenhuma dependência foi adicionada ou atualizada.

## Fase 1 — critérios de aceite e validação

- [x] Módulo collectors com cpu.rs, memory.rs, disk.rs, gpu.rs e mod.rs, sem alterações na UI.
- [x] Snapshot, DiskInfo e GpuInfo serializáveis com serde; unidades documentadas.
- [x] sysinfo 0.40.3 com somente system/disk e atualizações seletivas, sem refresh_all.
- [x] CPU usa uma leitura de base e respeita MINIMUM_CPU_UPDATE_INTERVAL antes da seguinte.
- [x] Volume do sistema escolhido por SystemDrive; só o espaço desse volume é atualizado.
- [x] nvml-wrapper 0.13.0 inicializado uma vez; GPU ausente retorna None e sensores são independentes/opcionais.
- [x] coletor_cli imprime JSON a cada 2 s, incluindo CPU normalizada e RAM do próprio processo, sem WebView2.
- [x] Três testes unitários passaram: RAM/unidades/total zero, disco/capacidade inválida e GPU ausente/sensores parciais.
- [x] cargo fmt -- --check e cargo clippy --all-targets -- -D warnings passaram.
- [x] cargo test passou: 3 testes, sem falhas; cargo build --release --example coletor_cli passou.
- [x] cargo-license verificou os 11 pacotes novos; licenças registradas em docs/ESTRUTURA.md.

Validação local em 2026-11-15, com o CLI release por 15 amostras (32,92 s incluindo
inicialização), sem UI/WebView2. Valores de memória em MB decimais; CPU do processo
normalizada pelo número de CPUs lógicas:

- CPU média do CLI: 0,0781%; máximo observado: 0,1953%.
- RAM média do CLI: 33,94 MB; faixa observada: 33,94–33,98 MB, acima da meta de 25 MB.
- Intervalo médio observado entre linhas: 1,996 s; faixa 1,939–2,004 s.
- Percentuais de CPU, RAM e disco dentro de 0–100; RAM usada/total consistente com ram_percent.
- Volume C: validado com System.IO.DriveInfo; diferença de 8.192 bytes na última leitura de espaço usado.
- GPU local respondeu aos três sensores: uso 5%, VRAM total 4 GiB (4.294,97 MB) e temperatura 44 °C na última amostra.
- Ausência de GPU e sensores não suportados foram verificados pelo teste unitário, sem depender do hardware.

Comando a partir da raiz:
`cargo run --manifest-path src-tauri/Cargo.toml --release --example coletor_cli -- 15`.
Sem o argumento 15, o CLI continua até Ctrl+C. As amostras e o resumo locais ficam
em target/fase-1-amostras.jsonl e target/fase-1-resumo.json dentro de src-tauri,
ignorados pelo Git. Esta observação curta não substitui a avaliação prolongada da Fase 5.
A versão do aplicativo permanece 0.1.0; 0.5 é a versão deste plano.

## Fase 2 — critérios de aceite e validação

- [x] AppState com último Snapshot e histórico VecDeque de até 60 amostras, sob Arc/RwLock.
- [x] Modo em AtomicU8 compartilhado: Collapsed = 5 s e Expanded = 2 s, inicialmente Collapsed.
- [x] Uma thread iniciada no setup; espera bloqueante interrompida por mudança de modo ou parada.
- [x] Comandos Tauri set_mode(mode) e get_snapshot(); leitura da UI não consulta sensores.
- [x] Evento snapshot após gravar o cache e liberar o lock; contador temporário no stderr.
- [x] UI de texto puro, sem CSS, com listener e dois botões de teste; GPU/sensor ausente mostra N/D.
- [x] UI ignora payload repetido e adia a atualização do texto enquanto document.hidden for verdadeiro.
- [x] Fluxo thread/cache/evento/UI explicado em docs/ESTRUTURA.md.
- [x] cargo fmt e cargo clippy --all-targets -- -D warnings passaram.
- [x] cargo test passou: 5 testes (3 dos coletores e 2 novos para histórico e modo/avisos).
- [x] npm run tauri build -- --no-bundle passou, incluindo TypeScript/Vite e executável release.
- [x] UI e comandos testados no WebView2 real do executável release pela conexão CDP local.

Validação em 2026-11-15, com perfil WebView2 isolado dentro de target:

- Recolhido: intervalos de 5,003 s e 5,007 s entre eventos recebidos pela UI.
- Expandido: intervalos de 2,008 s, 2,014 s e 2,005 s entre eventos recebidos pela UI.
- Contador Rust confirmado no stderr, com sequências estáveis de aproximadamente 2 s e 5 s.
- get_snapshot retornou o Snapshot em cache; métricas reais exibidas e nenhuma folha de estilo carregada.
- Payload repetido gerou zero mutações no DOM das métricas.
- Visibilidade simulada como oculta manteve o texto; visibilitychange após a restauração exibiu o último payload.
- Nenhum erro de JavaScript observado durante a validação.
- Teste unitário confirmou descarte das amostras antigas ao inserir 65 e limite final de 60, com leitura independente do cache.
- Teste unitário confirmou os intervalos, o aviso para acordar a thread, a coalescência de avisos e o sinal de parada.

A automação nativa de Windows não conectou ao helper; a validação funcional foi feita
pelo protocolo local do WebView2, conforme o fluxo documentado do Playwright.
A porta e o perfil de teste foram habilitados somente no processo de validação;
nenhuma configuração de depuração foi adicionada ao aplicativo. A instância de teste
foi encerrada após a validação. Logs, perfil e screenshot ficam ignorados em target.
Nenhuma dependência do aplicativo foi adicionada ou alterada nesta fase.
A versão do aplicativo permanece 0.1.0; 0.6 é a versão deste plano.

## Fase 3 — critérios de aceite e validação

- [x] Janela sem decorações, transparente, sempre no topo, sem barra de tarefas, sem resize manual e sem foco inicial.
- [x] Janela inicialmente oculta, posicionada no setup e só depois mostrada, sem chamada de set_focus.
- [x] window.rs usa o monitor primário, sua origem física e o DPI; margem superior de 8 pixels lógicos.
- [x] Tamanhos lógicos: Collapsed 180×36; Expanded 340×230; a janela nativa é redimensionada e recentralizada.
- [x] set_mode ajusta geometria antes do modo de amostragem; mudanças de DPI também recalculam o posicionamento.
- [x] Hover expande após 200 ms e recolhe após 400 ms; entrada/saída cancela o timer oposto.
- [x] Clique fixa; novo clique ou Esc libera; configuração hover-click/click-only e tempos no topo de main.ts.
- [x] UI serializa comandos e renderiza métricas apenas quando expandidas/visíveis e com texto diferente.
- [x] Fundo colorido e fonte do sistema provisórios; nenhum visual final da Fase 4 implementado.
- [x] cargo fmt e cargo clippy --all-targets -- -D warnings passaram.
- [x] cargo test passou: 6 testes, incluindo geometria a 100%, 150%, 200% e origem negativa do monitor.
- [x] npm run tauri build -- --no-bundle passou, incluindo TypeScript/Vite e release Rust.

Validação em 2026-11-15, no WebView2 do executável release com perfil isolado:

- Monitor primário local: 2560×1440, escala 100%.
- Recolhida: innerSize e outerSize 180×36, posição física (1190, 8).
- Expandida: innerSize e outerSize 340×230, posição física (1110, 8).
- A largura nativa recolhida confirma que não há uma janela transparente de 340×230 bloqueando cliques fora da ilha.
- Consultas nativas confirmaram foco inicial false, always-on-top true, decorada false e redimensionável false.
- Eventos de mouse controlados no WebView2: expansão após aproximadamente 224 ms e recolhimento após 419 ms (timers mais execução do comando).
- Entrada de 80 ms não expandiu; reentrada cancelou recolhimento; fixada permaneceu aberta após sair.
- Segundo clique liberou a fixação, e Esc fora da área recolheu; estado final e tamanho nativo voltaram a Collapsed/180×36.
- Click-only validado em iframe isolado com cópia do TypeScript e comandos simulados: hover não expande; clique fixa; novo clique/Esc recolhe.
- Conteúdo expandido sem overflow; screenshots provisórios inspecionados; nenhum erro de JavaScript observado.

A automação usou a ferramenta Playwright existente em target e uma porta local
habilitada apenas na instância de teste. Eventos controlados isolaram a validação
de movimentos reais do ponteiro. Os testes de DPI a 150% e 200% foram unitários,
sem alterar a escala do Windows. A instância de teste foi encerrada após validar.
Esc libera quando a ilha recebe o teclado; em hover-click, enquanto o ponteiro
permanece dentro, o hover continua mantendo a janela aberta após liberar a fixação.
Nenhuma dependência do aplicativo foi adicionada ou alterada.
A versão do aplicativo permanece 0.1.0; 0.7 é a versão deste plano.

## Pendências
- Fase 5: investigar e reduzir a RAM do CLI Rust com NVML ativa (média de 33,94 MB em release na Fase 1; meta abaixo de 25 MB). A CPU média observada ficou abaixo de 1%.

## Registro de versões
- 0.7 (2026-11-15): Fase 3 concluída; janela nativa estilo ilha com DPI, tamanhos 180×36/340×230, hover temporizado e fixação por clique/Esc; fundo provisório, seis testes, fmt/Clippy e build release aprovados; geometria e interações validadas no WebView2.
- 0.6 (2026-11-15): Fase 2 concluída; cache compartilhado de 60 amostras, modo atômico, thread com espera de 2/5 s, comandos/eventos Tauri e UI textual; fmt/Clippy, cinco testes e build release aprovados; cadência e atualização seletiva validadas no WebView2.
- 0.5 (2026-11-15): Fase 1 concluída; coletores Rust, Snapshot serializável, NVML opcional, CLI de 2 s, três testes e checks aprovados; licenças verificadas e consumo release registrado, com RAM acima da meta como pendência da Fase 5.
- 0.4 (2026-11-15): Fase 0 concluída; template limpo, nome PixelPulse, perfil release para tamanho, estrutura e documentação inicial, fmt/Clippy/build aprovados e executável medido.
- 0.3 (2026-11-15): licença GPL-3.0 adotada.
- 0.2 (2026-11-15): nome PixelPulse, pixel art já na v1, organização no GitHub (@renahangil), Fase 8 (README).
- 0.1 (2026-11-15): plano inicial.
