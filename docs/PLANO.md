# Plano — PixelPulse
Versão: 0.10 (2026-11-20)
Repositório: github.com/renahangil/pixelpulse

## Decisões
- Nome do app: PixelPulse. Rust + Tauri 2, UI web leve, Windows 11 Pro.
- v1: visual pixel art com cores e sombreamento discretos sobre fundo escuro opaco; barras em blocos de pixel, cada métrica com uma cor.
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
| 4 | Visual pixel art com glass discreto | concluída |
| 5 | Medição e otimização | implementada; aguardando medição do usuário e inspeção visual nativa |
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

## Fase 4 — critérios de aceite e validação

- [x] API oficial de window-vibrancy 0.8.1 conferida; acrylic e fallback rgba(20,20,26,0.6) sem blur. Correção na Fase 5: a tinta (20,20,26,72) só é utilizada no caminho legado do crate.
- [x] Cantos CSS de 4 px e recorte do conteúdo; geometria nativa da Fase 3 preservada.
- [x] tokens.css com grade --px:3px, textos solicitados e três tons discretos de CPU/RAM/GPU/SSD/IA.
- [x] Quatro barras de 16 spans persistentes; sombreamento superior/inferior de 3 px e trilho com xadrez sutil.
- [x] Silkscreen Regular latina WOFF2 local de 8.404 bytes; OFL 1.1 confirmada, copiada e incluída no bundle.
- [x] Resumo com quatro pontos de 6×6 px e maior percentual disponível; expandida com quatro SVGs inline 8×8, rótulos, barras e percentuais.
- [x] GPU N/D mostra traço e barra apagada; percentuais com largura fixa de 42 px (60 px no resumo).
- [x] Animação de entrada só com transform/opacity em steps(3), desativada por prefers-reduced-motion.
- [x] DESIGN.md documenta grade, paleta, tamanhos, fonte/licença, atualizações e explicação do CSS.
- [x] cargo fmt, cargo clippy --all-targets -- -D warnings e cargo test passaram (6 testes Rust).
- [x] npm run tauri build -- --no-bundle passou, incluindo TypeScript/Vite e release Rust.
- [x] Licença MIT OR Apache-2.0 do crate confirmada com cargo-license; nenhum pacote transitivo novo.

Validação automatizada do executável release em 2026-11-15, com perfis isolados:

- Acrylic ativou sem erro; 12 redimensionamentos após aquecimento: média 21,02 ms, máximo 23,89 ms.
- Fallback RGBA: 12 redimensionamentos após aquecimento: média 21,13 ms, máximo 25,64 ms.
- As medições incluem a chamada IPC e a espera pelo viewport correto; são uma comparação curta local, não uma medição de CPU/RAM da Fase 5.
- Fixtures de 0/50/100/25% produziram 0/8/16/4 blocos acesos, respectivamente.
- De 8 para 10 blocos, somente duas classes mudaram; payload repetido produziu zero mutações do DOM.
- Todos os 64 elementos foram preservados; GPU indisponível apagou seus 16 blocos e mostrou traço.
- Fonte local carregada e licença acessível no bundle; uma única face; nenhuma URL de fonte externa em runtime.
- Pontos de 6×6 px, valores de 42 px, raio de 4 px e texto #d8d8de confirmados.
- Movimento reduzido resultou em animation:none; modo normal usou steps(3); sem overflow ou erros de JavaScript.
- Capturas da UI recolhida e expandida foram inspecionadas após concluir a animação, com acrylic e fallback.

A captura/automação nativa do Windows estava indisponível. Os testes e PNGs verificam
a UI no WebView2, não o resultado final do compositor DWM por trás dela; essa inspeção
nativa permanece registrada nas Pendências. As instâncias de teste foram encerradas.
Scripts, perfis, relatórios e capturas ficam em target/fase4-*, ignorados pelo Git.
A versão do aplicativo permanece 0.1.0; 0.8 é a versão deste plano.

## Fase 5 — implementação, medição e aceite pendente

A auditoria foi entregue antes das alterações; o usuário aprovou a implementação
mantendo Acrylic. Medida inicial corrigida pelo usuário: aproximadamente 24 MB de
RAM e CPU em poucos décimos (exibição arredondada de 0%), em `npm run tauri dev`.
O relato anterior de 20% estava incorreto e não é usado como referência de ganho.
Esse número de desenvolvimento não é diretamente comparável ao release.

- [x] Acrylic mantido por padrão; alternativa RGBA preservada, sem flags de WebView2.
- [x] Conferida a implementação instalada: no Windows 11 recente, window-vibrancy usa DWMSBT_TRANSIENTWINDOW e ignora o argumento RGBA; escurecimento CSS mantido e documentação corrigida.
- [x] Solicitado DWMWCP_ROUNDSMALL e criado diagnóstico dos atributos apenas do próprio HWND, sem listar outras janelas nem capturar a tela.
- [x] AppState recebe visibilidade do documento por atômico/canal; ocultação pausa sensores e eventos, sem confundir recolhimento ou falta de foco com ocultação.
- [x] Flags nativas de ocultação/minimização verificadas sem bloquear o event loop; quando só a janela está oculta, a verificação de flags continua a cada 5 s sem sensores.
- [x] Retomada reinicializa a base da CPU; uma amostra já em andamento pode concluir durante a transição, sem emitir se o documento já foi marcado oculto.
- [x] Cache continua registrando todas as leituras visíveis até 60; deduplicação de evento considera inteiro/segmentos expandidos ou maior inteiro recolhido.
- [x] Expandir/restaurar consulta o cache, com proteção contra resposta mais antiga que um evento; timers de hover são cancelados ao ocultar.
- [x] App consulta apenas uso GPU; CLI mantém VRAM e temperatura opcionais. Snapshot completo preservado, sem micro-otimizações fora do escopo.
- [x] Contador de eventos e aviso de modo limitados a debug; erros e diagnóstico inicial permanecem em release.
- [x] cargo fmt, cargo clippy --all-targets -- -D warnings e cargo test aprovados (8 testes, incluindo deduplicação por segmento e interrupção da espera).
- [x] npm run tauri build -- --no-bundle aprovado; TypeScript, Vite e executável release.
- [x] windows 0.62.2 já transitivo tornado direto, somente no Windows; Apache-2.0 OR MIT confirmado com cargo-license; nenhum pacote/versão transitiva novo.
- [x] Release real: get_glass_diagnostics retornou backdrop_type=3, corner_preference=3, errors=[]; UI GPU não consultou sensores extras.
- [x] Teste de visibilitychange no WebView2: cache congelado e zero eventos durante 6,3 s após estabilizar a ocultação; retomada confirmada, 64 blocos preservados e nenhum erro JS.
- [x] Regressão WebView2 com Acrylic e RGBA: 0/8/16/4 segmentos, só duas classes na transição 8→10, payload igual sem mutações, GPU ausente apagada, fonte local, movimento reduzido e nenhum overflow/erro JS. Fixtures isoladas com o coletor pausado para não serem substituídas por amostras reais.
- [x] Doze redimensionamentos aquecidos por modo: Acrylic média 17,81 ms/máximo 22,72 ms; RGBA média 18,62 ms/máximo 23,51 ms. Medida IPC/viewport, não uma inspeção de pixels do compositor.
- [ ] Usuário medir novamente o executável release, recolhido e expandido, usando a mesma coluna/unidade de memória para antes e depois.
- [ ] Inspeção visual final do compositor: cantos, artefatos, expandir/recolher e diferentes DPI/drivers. A ferramenta nativa ainda não conectou ao helper nesta sessão.

A leitura do atributo DWM confirma a configuração aceita, não a aparência dos pixels.
A preferência de cantos pequenos é uma indicação ao sistema, sem garantia de raio
físico igual ao CSS de 4 px. Não declaramos uma inspeção visual concluída por consultar
DwmIsCompositionEnabled ou enumerar janelas; essas operações não validam Acrylic.

### Antes e depois em release

Comparação sequencial local em 2026-11-15: cópia do release da Fase 4 (commit
9598338) e release otimizado, ambos com Acrylic, modo Collapsed fixado em 180×36.
Perfis isolados; 10 s de aquecimento, oito CPUs lógicas, sete processos por árvore
com PIDs estáveis e 12 leituras de memória por versão. Ensaios anteriores em que
hover mudou o modo foram descartados para esta comparação.

A porta CDP foi habilitada temporariamente somente nessas instâncias para fixar o
modo e impedir hover acidental. A conexão foi encerrada antes de coletar os
contadores. Isso não é uma alteração dos argumentos de produção e pode acrescentar
algum custo de diagnóstico; repetir sem depuração na medição do usuário. A opção
permite controle local do WebView enquanto ativa, por isso fica fora do app final.

| Métrica | Antes | Depois |
| --- | ---: | ---: |
| Duração dos contadores | 124,44 s | 125,75 s |
| CPU média Rust normalizada | 0,041% | 0,008% |
| CPU média grupo Rust + WebView2 | 0,088% | 0,064% |
| RAM privada residente Rust (média) | 25,39 MB | 25,32 MB |
| RAM privada residente grupo (média) | 91,47 MB | 91,33 MB |
| Working set total Rust (média) | 53,28 MB | 51,37 MB |
| Soma dos working sets do grupo (média) | 371,52 MB | 372,05 MB |

Memória em MB decimais (1.000.000 bytes), CPU pela diferença de tempo acumulado de
cada processo dividida por seu intervalo medido e pelas oito CPUs lógicas. O grupo
inclui somente os descendentes da instância testada; não inclui o DWM compartilhado
do desktop nem outros WebViews. Somar working sets pode contar páginas compartilhadas
mais de uma vez; não é o mesmo indicador de memória privada do Gerenciador de Tarefas.
A CPU do DWM/GPU não foi isolada, portanto estes números não medem todo o custo do glass.

O Rust caiu aproximadamente 0,033 ponto percentual de CPU nesta rodada; o grupo caiu
0,024 ponto. Não atribuímos um percentual de ganho garantido a uma única comparação.
RAM privada permaneceu praticamente igual: não houve redução relevante demonstrada.
Executável otimizado: 3.452.928 bytes, contra 3.447.808 bytes do release anterior
(+5.120 bytes). Logs/cópia anterior/JSON/scripts locais: target/fase5-*, ignorados.

### Metas e recomendação

A CPU do Rust ficou abaixo de 1%. A memória privada residente do grupo ficou abaixo
de 120 MB neste ensaio. A do Rust, 25,32 MB decimais (aproximadamente 24,15 MiB),
ainda excede o teto literal de 25 MB. Pelo working set total do Rust, 51,37 MB,
a meta também não é atingida. Não alteramos a meta nem declaramos a RAM aprovada por
trocar de coluna/unidade; a nova medição deve identificar qual coluna está sendo usada.

Recomendação: manter Tauri + Acrylic durante esta validação, pois a CPU já é baixa
e trocar o material não demonstrou vantagem. Se o teto de 25 MB for obrigatório e
a nova medição confirmar o excesso, recomendo prototipar uma UI nativa Win32/Direct2D,
preservando coletores/cache Rust, antes de continuar a expandir a UI web. Medir também
o CLI com a mesma coluna para separar o piso do Rust/NVML: a troca de UI não garante
sozinha eliminar memória do driver. Nenhuma migração foi implementada nesta fase.

### Retorno do usuário após a implementação

Valores informados pelo usuário, preservando a unidade escrita ("Mb", sem confirmar
se a coluna do sistema usa MB decimais ou MiB). CPU desta rodada e agregação dos
processos WebView2 não foram informadas; não usar esses dados para aprovar as metas
por uma conversão presumida de unidade ou soma incompleta.

| Estado | WebView informado | PixelPulse informado | Soma se forem consumos separados |
| --- | ---: | ---: | ---: |
| Recolhida | 62 | 24,2 | 86,2 |
| Expandida | 68 | 24,4 | 92,4 |

A diferença informada ao expandir é de 6 no WebView e 0,2 no PixelPulse. Essas
leituras são compatíveis com consumo moderado; confirmar unidade/coluna, todos os
processos do grupo e CPU antes de encerrar o aceite quantitativo da fase.

O usuário confirmou cantos arredondados e forneceu capturas da janela expandida e
recolhida. O fundo aparece uniforme/cinza e a transparência/glass não está perceptível,
segundo o relato. A configuração DWM aceita não comprova transparência efetiva.
Não há confirmação de teste em outros DPI/drivers nem de ausência de tremulação.

Avaliação: não há indicação nesses números para migrar de Tauri por RAM neste
momento. Recomendo manter a aparência opaca por legibilidade/previsibilidade e,
se essa for a escolha do usuário, desativar o Acrylic em uma alteração posterior
para não solicitar um efeito sem benefício visual observado. Não estimar economia
de CPU/RAM sem nova comparação. Se transparência passar a ser prioridade estética,
o primeiro experimento deve usar o fallback RGBA já existente, apenas no fundo,
sem blur e preservando a opacidade do texto/ícones; evitar dependências/flags novas.
Nenhuma dessas mudanças visuais foi aplicada nesta avaliação.

A documentação Microsoft informa que Acrylic pode ficar sólido com transparência
desativada no Windows, economia de bateria, determinados hardwares e, no caso do
background acrylic, desativação da janela. A ilha evita tomar foco, portanto essa
última condição é uma hipótese relevante, não um diagnóstico confirmado. Não alterar
preferências globais do Windows nem forçar foco como parte desta avaliação.
Fonte: [Acrylic: adaptação e uso de GPU](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic).

### Nova medição solicitada ao usuário

Executar `src-tauri/target/release/pixelpulse.exe`, com `npm run tauri dev` fechado.
Aguardar 30 s, observar 2 min recolhido e 2 min expandido/fixado. Informar CPU média
ou faixa e RAM do processo PixelPulse, RAM do grupo incluindo os processos WebView2,
nome da coluna e unidade (MB/MiB). Conferir os cantos sobre fundos claros/escuros,
expansão repetida e escala habitual; informar qualquer tremulação ou lentidão.
Se houver artefato, comparar com PIXELPULSE_GLASS=fallback sem alterar outras opções.

A fase permanece em validação: o registro do depois do usuário e a inspeção visual
são necessários para encerrar o aceite. A tag de fase concluída será criada somente
nesse encerramento; 0.9 identifica esta atualização do plano, não uma conclusão falsa.

## Fechamento visual — branch fase-6-ui-melhorias

- Windows 11 x64 25H2 ou 26H2 atualizado é o alvo recomendado. Windows 11 24H2 em edições ainda atendidas e Windows 10 22H2 com WebView2 atualizado são alvos secundários de verificação. A entrega desta branch é um executável Windows x64.
- Tauri 2, WebView2, HTML/CSS/TypeScript e a estética pixel art permanecem. Fundo fosco opaco por padrão, cantos CSS de 4 px e preferência nativa pequena onde suportada. PIXELPULSE_GLASS=acrylic ou fallback ficam disponíveis para comparar os efeitos manualmente.
- A ilha reafirma sua ordem ao mostrar/redimensionar. Eventos Win32 de primeiro plano, exibição e reordenação verificam se outra janela normal do desktop ocupa a mesma região e está acima. O ajuste usa SWP_NOACTIVATE e o hook é removido na saída. Superfícies protegidas e tela cheia exclusiva têm regras próprias do Windows.
- O resumo mostra o rótulo e o valor da maior métrica. O ponto vencedor cresce de 6 a 9 px em uma célula fixa. Empates seguem a ordem CPU, RAM, GPU, SSD. GPU indisponível é excluída. O cache diferencia vencedores mesmo se o percentual arredondado repetir.
- O menu de contexto nativo oferece Encerrar PixelPulse e aciona a saída normal do Tauri para remover o hook e parar a coleta.
- O ícone usa um pulso em degraus e quatro pixels que remetem ao painel. O SVG mestre gera os ícones de empacotamento, incluindo o recurso ICO do executável. Referências funcionais da pesquisa: [TrafficMonitor](https://github.com/zhongyang219/TrafficMonitor/wiki/Main-Window), [Rainmeter](https://www.rainmeter.net/) e [PowerToys](https://learn.microsoft.com/en-us/windows/powertoys/command-palette/overview).
- Validação: fmt, Clippy sem warnings, oito testes Rust e build release aprovados. No WebView2, troca de vencedor com o mesmo percentual, 100% sem corte, tamanho dos pontos, expansão/recolhimento e GPU N/D aprovados. Uma janela concorrente nativa confirmou recuperação do topo sem troca de foco. A ação Encerrar PixelPulse no menu encerrou o processo com código 0. O ICO foi confirmado no PE x64 nos seis tamanhos de 16 a 256 px. Executável: src-tauri/target/release/pixelpulse.exe, 3.438.592 bytes.

## Pendências
- Glass — decisão da Fase 4: manter acrylic escuro por padrão; a comparação release local não mostrou atraso relevante frente ao RGBA. Se a API falhar, usar automaticamente rgba(20,20,26,0.6), sem desfoque; PIXELPULSE_GLASS=fallback permite forçar esse plano B se houver artefatos ou lentidão na máquina.
- Fechamento visual: o relato de Acrylic aparentemente opaco orientou a adoção de fundo opaco. Conferir WhatsApp e diferentes DPIs no ambiente de uso. O teste local de sobreposição usou janela concorrente controlada.
- Fase 5: meta de RAM Rust ainda não aprovada (25,32 MB privados/51,37 MB working set no app otimizado). Medir novamente com coluna/unidade explícitas e comparar ao CLI/NVML (working set médio histórico de 33,94 MB); recomendação de UI nativa se o limite rígido continuar excedido está no relatório acima.

## Registro de versões
- 0.10 (2026-11-20): fechamento da UI com fundo opaco, identificação da maior métrica, correção de sobreposição por eventos, menu Encerrar PixelPulse e ícone próprio. A fase de IA local permanece no roteiro.
- 0.9 (2026-11-15): inclui retorno do usuário (WebView 62/68; PixelPulse 24,2/24,4; cantos arredondados e glass não perceptível), com unidade/CPU ainda pendentes. Fase 5 implementada após auditoria/aprovação; pausa oculta, eventos deduplicados, GPU seletiva, logs só em debug e diagnóstico/cantos DWM; release e oito testes aprovados. Aceite permanece pendente de nova medição do usuário e inspeção visual nativa.
- 0.8 (2026-11-15): Fase 4 concluída; glass com fallback, paleta em três tons, barras de 16 blocos, SVGs pixel, Silkscreen OFL local, resumo recolhido e movimento reduzido; checks/release e validação de DOM aprovados, decisão e limite de inspeção nativa registrados.
- 0.7 (2026-11-15): Fase 3 concluída; janela nativa estilo ilha com DPI, tamanhos 180×36/340×230, hover temporizado e fixação por clique/Esc; fundo provisório, seis testes, fmt/Clippy e build release aprovados; geometria e interações validadas no WebView2.
- 0.6 (2026-11-15): Fase 2 concluída; cache compartilhado de 60 amostras, modo atômico, thread com espera de 2/5 s, comandos/eventos Tauri e UI textual; fmt/Clippy, cinco testes e build release aprovados; cadência e atualização seletiva validadas no WebView2.
- 0.5 (2026-11-15): Fase 1 concluída; coletores Rust, Snapshot serializável, NVML opcional, CLI de 2 s, três testes e checks aprovados; licenças verificadas e consumo release registrado, com RAM acima da meta como pendência da Fase 5.
- 0.4 (2026-11-15): Fase 0 concluída; template limpo, nome PixelPulse, perfil release para tamanho, estrutura e documentação inicial, fmt/Clippy/build aprovados e executável medido.
- 0.3 (2026-11-15): licença GPL-3.0 adotada.
- 0.2 (2026-11-15): nome PixelPulse, pixel art já na v1, organização no GitHub (@renahangil), Fase 8 (README).
- 0.1 (2026-11-15): plano inicial.
