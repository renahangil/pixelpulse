# Plano — PixelPulse
Versão: 0.3 (2026-11-15)
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
| 0 | Ambiente, repositório e estrutura | pendente |
| 1 | Coletor de métricas (Rust) | pendente |
| 2 | Cache e ponte com a UI | pendente |
| 3 | Janela estilo ilha | pendente |
| 4 | Visual pixel art com glass discreto | pendente |
| 5 | Medição e otimização | pendente |
| 6 | IA local (Claude Code, Codex) | pendente |
| 7 | Acabamento (bandeja, autostart, config) | pendente |
| 8 | README e publicação | pendente |

## Pendências
(nenhuma)

## Registro de versões
- 0.3 (2026-11-15): licença GPL-3.0 adotada.
- 0.2 (2026-11-15): nome PixelPulse, pixel art já na v1, organização no GitHub (@renahangil), Fase 8 (README).
- 0.1 (2026-11-15): plano inicial.