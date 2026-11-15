# Plano — PixelPulse
Versão: 0.4 (2026-11-15)
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
| 1 | Coletor de métricas (Rust) | pendente |
| 2 | Cache e ponte com a UI | pendente |
| 3 | Janela estilo ilha | pendente |
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

## Pendências
(nenhuma)

## Registro de versões
- 0.4 (2026-11-15): Fase 0 concluída; template limpo, nome PixelPulse, perfil release para tamanho, estrutura e documentação inicial, fmt/Clippy/build aprovados e executável medido.
- 0.3 (2026-11-15): licença GPL-3.0 adotada.
- 0.2 (2026-11-15): nome PixelPulse, pixel art já na v1, organização no GitHub (@renahangil), Fase 8 (README).
- 0.1 (2026-11-15): plano inicial.
