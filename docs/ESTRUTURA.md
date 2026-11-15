# Estrutura — PixelPulse

Mapa até a Fase 3; módulos posteriores serão criados somente nas fases correspondentes.
Cada `.gitkeep` é vazio e permite ao Git preservar uma pasta ainda sem implementação.

## Raiz e configuração

- `AGENTS.md`: regras do projeto, da colaboração e da organização em fases.
- `README.md`: apresentação provisória; a documentação completa fica para a Fase 8.
- `LICENSE`: texto da licença GPL-3.0 do projeto.
- `.gitignore`: exclusões de builds, dependências, logs e arquivos locais do sistema/editor.
- `package.json`: metadados, dependências e comandos npm da interface e do Tauri.
- `package-lock.json`: versões exatas das dependências npm para instalações reproduzíveis.
- `tsconfig.json`: configuração da checagem estática do TypeScript.
- `vite.config.ts`: usa `src/` como raiz da UI e gera `dist/` para o Tauri.
- `.github/`: instruções e modelos usados na colaboração pelo GitHub.
- `.github/copilot-instructions.md`: orientações do projeto para o GitHub Copilot.
- `.github/pull_request_template.md`: roteiro de mudanças, testes, aceite e impacto em CPU/RAM.
- `.vscode/`: configuração local de editor herdada do template e registrada neste mapa.
- `.vscode/extensions.json`: recomendações de extensões para Tauri e Rust.

## Interface

- `src/`: código e recursos locais da interface, sem frameworks de UI.
- `src/index.html`: conteúdo da ilha, cabeçalho, indicador de fixação e métricas expandidas.
- `src/main.ts`: escuta snapshot e controla hover com atrasos, fixação, Esc e trocas de modo serializadas.
- `src/styles/`: estilos da UI; os tokens e o visual pixel art ficam para a Fase 4.
- `src/styles/window.css`: fundo colorido e layout provisórios, limitados ao tamanho nativo da ilha.
- `src/ui/`: pasta reservada aos componentes da interface.
- `src/ui/.gitkeep`: preserva a pasta de componentes ainda vazia.
- `src/assets/`: recursos locais da interface, sem downloads durante a execução.
- `src/assets/fonts/`: pasta reservada à fonte pixel com licença livre.
- `src/assets/fonts/.gitkeep`: preserva a pasta de fontes ainda vazia.

## Aplicativo Rust e Tauri

- `src-tauri/`: projeto nativo e configurações do aplicativo desktop.
- `src-tauri/.gitignore`: ignora o build Rust e os esquemas gerados pelo Tauri.
- `src-tauri/Cargo.toml`: metadados, dependências Rust e perfil release voltado ao tamanho.
- `src-tauri/Cargo.lock`: versões exatas das dependências Rust usadas na compilação.
- `src-tauri/build.rs`: aciona a preparação de recursos nativos do Tauri durante o build.
- `src-tauri/tauri.conf.json`: janela transparente sem bordas, no topo, sem barra de tarefas ou foco inicial, além dos caminhos e empacotamento.
- `src-tauri/src/`: código Rust do aplicativo.
- `src-tauri/src/main.rs`: entrada do executável Windows que chama a biblioteca do app.
- `src-tauri/src/lib.rs`: registra estado/comandos, posiciona/mostra a ilha no setup e controla a thread de coleta.
- `src-tauri/src/collectors/`: coleta local de CPU, RAM, disco e GPU, sem integração com a UI.
- `src-tauri/src/collectors/mod.rs`: structs serializáveis, unidades e orquestração das amostras.
- `src-tauri/src/collectors/cpu.rs`: leitura seletiva de uso de CPU com aquecimento entre duas amostras.
- `src-tauri/src/collectors/memory.rs`: leitura de RAM física, conversões e teste de capacidade zero.
- `src-tauri/src/collectors/disk.rs`: seleção do disco do sistema, leitura de espaço e teste do cálculo.
- `src-tauri/src/collectors/gpu.rs`: NVML persistente, sensores opcionais e teste do fallback.
- `src-tauri/src/cache/`: armazenamento compartilhado e ciclo de vida da thread de amostragem.
- `src-tauri/src/cache/mod.rs`: AppState com Arc/RwLock, histórico de 60, modo atômico e avisos de mudança.
- `src-tauri/src/cache/sampler.rs`: coleta/espera/emissão em uma thread, contador de eventos e encerramento.
- `src-tauri/src/window.rs`: tamanhos lógicos, conversão DPI, posição no monitor primário e resposta a mudanças de escala.
- `src-tauri/src/commands.rs`: set_mode ajusta janela e amostragem; get_snapshot lê somente o cache.
- `src-tauri/src/ai/`: pasta reservada à leitura local de contagens de IA, apenas na Fase 6.
- `src-tauri/src/ai/.gitkeep`: preserva a pasta de IA sem antecipar sua implementação.
- `src-tauri/examples/`: programas de teste em linha de comando, sem abrir janela ou WebView2.
- `src-tauri/examples/coletor_cli.rs`: imprime JSON a cada 2 s com o Snapshot e CPU/RAM do próprio processo.
- `src-tauri/capabilities/`: permissões da interface perante o Tauri, herdadas do template.
- `src-tauri/capabilities/default.json`: permissões padrão da janela principal.
- `src-tauri/icons/`: ícones locais de empacotamento herdados do template, sem nova arte nesta fase.
- `src-tauri/icons/32x32.png`: ícone PNG de 32 px.
- `src-tauri/icons/128x128.png`: ícone PNG de 128 px.
- `src-tauri/icons/128x128@2x.png`: ícone PNG de 256 px para telas de maior densidade.
- `src-tauri/icons/icon.ico`: ícone nativo do aplicativo e instaladores Windows.
- `src-tauri/icons/icon.icns`: ícone do template destinado ao empacotamento macOS.
- `src-tauri/icons/icon.png`: ícone PNG base do template.
- `src-tauri/icons/StoreLogo.png`: logo do template para a loja Windows.
- `src-tauri/icons/Square30x30Logo.png`: ícone quadrado Windows de 30 px.
- `src-tauri/icons/Square44x44Logo.png`: ícone quadrado Windows de 44 px.
- `src-tauri/icons/Square71x71Logo.png`: ícone quadrado Windows de 71 px.
- `src-tauri/icons/Square89x89Logo.png`: ícone quadrado Windows de 89 px.
- `src-tauri/icons/Square107x107Logo.png`: ícone quadrado Windows de 107 px.
- `src-tauri/icons/Square142x142Logo.png`: ícone quadrado Windows de 142 px.
- `src-tauri/icons/Square150x150Logo.png`: ícone quadrado Windows de 150 px.
- `src-tauri/icons/Square284x284Logo.png`: ícone quadrado Windows de 284 px.
- `src-tauri/icons/Square310x310Logo.png`: ícone quadrado Windows de 310 px.

## Documentação

- `docs/`: planejamento e documentação técnica do projeto.
- `docs/PLANO.md`: decisões, fases, critérios de aceite, pendências e registro de versões.
- `docs/ESTRUTURA.md`: este mapa dos arquivos e pastas do projeto.
- `docs/img/`: pasta reservada às imagens da documentação.
- `docs/img/.gitkeep`: preserva a pasta de imagens ainda vazia.

`styles/tokens.css`,
`docs/DESIGN.md` e `docs/formatos-logs.md` serão criados nas fases que os utilizarem.

## Arquivos locais gerados (fora do controle de versão)

- `.git/`: histórico, branches, tags e metadados internos do Git.
- `node_modules/`: pacotes npm instalados e respectivos caches locais.
- `dist/`: interface compilada pelo Vite e incorporada ao aplicativo release.
- `src-tauri/target/`: cache Rust, executável release e instaladores em `release/bundle/`.
- `src-tauri/gen/schemas/`: esquemas de configuração e permissões gerados pelo Tauri.

## Dependências e referências

Nenhum crate ou pacote npm foi adicionado ou atualizado na Fase 0; os manifestos e
lockfiles do template foram preservados quanto às dependências. A verificação de
licença com cargo-license ou cargo deny deverá acompanhar qualquer nova dependência.

- [Cargo: perfis e opções de release](https://doc.rust-lang.org/cargo/reference/profiles.html).
- [Tauri 2: nome do produto e configuração da janela](https://v2.tauri.app/reference/config/).
- [Vite: raiz do projeto](https://vite.dev/config/shared-options.html#root).
- [Vite: diretório de saída](https://vite.dev/config/build-options.html#build-outdir).

## Coletor da Fase 1

Em `src-tauri/`, execute `cargo run --release --example coletor_cli` e encerre com
Ctrl+C; para uma execução finita, use `cargo run --release --example coletor_cli -- 15`.
Da raiz, acrescente `--manifest-path src-tauri/Cargo.toml` logo depois de `cargo run`.
O número opcional deve ser maior que zero.

Cada linha é JSON com `snapshot` e `process`. RAM e VRAM usam MB decimais
(1.000.000 bytes), disco usa GB decimais (1.000.000.000 bytes), temperatura usa °C
e percentuais usam a escala 0–100. A CPU do próprio processo é dividida pelo número
de CPUs lógicas para representar uma fração da capacidade total da máquina.

O coletor inicia a base da CPU e espera `MINIMUM_CPU_UPDATE_INTERVAL` se a próxima
leitura chegar cedo demais; o CLI espera 2 s antes de publicar a primeira amostra.
A lista de volumes é obtida uma vez, e só o espaço do volume `SystemDrive` é
atualizado depois disso; falhas de disco são devolvidas como erro, sem inventar 0%.
Nenhuma leitura de atividade de disco, swap, frequência da CPU ou conteúdo de logs é feita.

`gpu: null` representa NVML ou placa indisponível; dentro de uma GPU disponível,
cada sensor pode ser `null` independentemente dos demais. Usa-se a primeira placa
NVIDIA (índice 0), sem fixar modelo nem capacidade da VRAM. Desde a Fase 2, o setup
do Tauri também inicia a coleta; o CLI continua disponível de forma independente.

Dependências diretas adicionadas nesta fase:

- `sysinfo 0.40.3` (MIT): consulta CPU, RAM, disco e processo; somente features `system` e `disk`, sem Rayon.
- `nvml-wrapper 0.13.0` (MIT OR Apache-2.0): consulta GPU NVIDIA com carregamento dinâmico e ausência tolerada.

A ferramenta cargo-license fica em `src-tauri/target/tools/` e serve apenas à
verificação de licenças; não integra as dependências nem o executável do aplicativo.

Referências oficiais consultadas:

- [sysinfo: System e atualizações seletivas](https://docs.rs/sysinfo/0.40.3/sysinfo/struct.System.html).
- [sysinfo: espaço do disco](https://docs.rs/sysinfo/0.40.3/sysinfo/struct.Disk.html).
- [sysinfo: features](https://docs.rs/crate/sysinfo/0.40.3/features).
- [nvml-wrapper: carregamento e reutilização da NVML](https://docs.rs/nvml-wrapper/0.13.0/nvml_wrapper/).

### Verificação de licenças da Fase 1

Verificação com cargo-license 0.7.0, em 2026-11-15:
`src-tauri/target/tools/bin/cargo-license.exe --manifest-path src-tauri/Cargo.toml --json --output src-tauri/target/fase-1-licencas.json`.
A comparação com o metadata anterior identificou os 11 pacotes abaixo; nenhuma
versão já existente foi atualizada. As licenças são compatíveis com GPL-3.0:
MIT/Apache-2.0 nas alternativas múltiplas e ISC para libloading, conforme a [lista de licenças da GNU](https://www.gnu.org/licenses/license-list.html#ISC).

| Pacote adicionado | Versão | Licença declarada |
| --- | --- | --- |
| darling | 0.20.11 | MIT |
| darling_core | 0.20.11 | MIT |
| darling_macro | 0.20.11 | MIT |
| libloading | 0.8.9 | ISC |
| ntapi | 0.4.3 | Apache-2.0 OR MIT |
| nvml-wrapper | 0.13.0 | Apache-2.0 OR MIT |
| nvml-wrapper-sys | 0.10.0 | Apache-2.0 OR MIT |
| objc2-io-kit | 0.3.2 | Apache-2.0 OR MIT OR Zlib |
| static_assertions | 1.1.0 | Apache-2.0 OR MIT |
| sysinfo | 0.40.3 | MIT |
| wrapcenum-derive | 0.4.1 | Apache-2.0 OR MIT |

A NVML é fornecida pelo driver NVIDIA, carregada como DLL do sistema e não
redistribuída no repositório. O aplicativo continua funcionando quando ela está ausente.

## Cache e ponte com a UI — Fase 2

O setup cria o AppState e um Collector, registra os comandos e inicia exatamente
uma thread de amostragem. O primeiro resultado é publicado assim que estiver pronto;
as próximas coletas começam a cada 5 s (Collapsed, padrão) ou 2 s (Expanded).

O AppState compartilha por Arc/RwLock o Snapshot atual (None antes da primeira
coleta) e um VecDeque com as últimas 60 amostras bem-sucedidas, incluindo a atual.
A entrada mais antiga sai antes de entrar a 61ª. O modo fica em Arc/AtomicU8.
Um canal limitado a um aviso acorda a thread ao mudar o modo; ela recalcula o prazo
em relação ao início da última coleta, sem filas crescentes nem espera ocupada.

Após coletar, a thread escreve no cache, libera o lock e emite snapshot com o
Snapshot como payload. Ela emite na cadência mesmo se valores se repetirem; a UI
compara o texto e só altera o DOM quando há mudança visível. Enquanto document.hidden
for verdadeiro, apenas guarda o último payload em memória; ao voltar, mostra esse valor.

A UI registra o listener antes de chamar get_snapshot, evitando perder o primeiro
resultado. A resposta inicial não sobrescreve um evento já recebido. get_snapshot
só lê o cache, e set_mode recebe "Collapsed" ou "Expanded"; nenhum comando consulta
os sensores. Os botões mudam somente a frequência, não o tamanho da janela nesta fase.
GPU/sensor null é mostrado como N/D. Não há CSS, polling JavaScript ou dependência nova.

O contador temporário escreve no stderr, por exemplo:
`[snapshot #4] modo=Expanded intervalo=2s delta=Some(2.0001)`.
Em `npm run tauri dev`, veja-o no terminal; ao clicar nos botões, confira os deltas
entre eventos sucessivos de um mesmo modo (aproximadamente 2 s ou 5 s). O primeiro
evento não tem delta; um evento de transição pode chegar logo após o clique se o
novo prazo já venceu. O contador reinicia ao abrir novamente o aplicativo.
Os logs não são persistidos automaticamente e não contêm prompts nem respostas.

Ao sair, o Tauri sinaliza a parada, acorda a thread e aguarda seu término. Falhas de
coleta são registradas sem substituir a última leitura boa por zeros; falhas de lock
retornam erro aos comandos. Histórico e modo vivem apenas na memória.

Referências oficiais:

- [Tauri: estado gerenciado](https://v2.tauri.app/develop/state-management/).
- [Tauri: eventos Rust para frontend](https://v2.tauri.app/develop/calling-frontend/).
- [Tauri: comandos do frontend](https://v2.tauri.app/develop/calling-rust/).
- [Rust: RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
- [Rust: espera com recv_timeout](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.recv_timeout).

A validação usa temporariamente playwright-core 1.65.1 (Apache-2.0, conforme seu
package.json) em `src-tauri/target/fase2-tools/`, sem adicionar dependências ao app.
O perfil WebView2 isolado, logs e screenshot de teste também ficam em target,
ignorados pelo Git; a porta de depuração é ativada apenas no processo de teste.
Referência: [Playwright com WebView2](https://playwright.dev/docs/webview2).

## Janela estilo ilha — Fase 3

A janela nasce oculta, sem decorações, sem sombra, transparente, sempre no topo,
sem item na barra de tarefas, sem redimensionamento manual e sem foco inicial.
O setup a posiciona antes de mostrá-la. Ela pode receber foco após um clique para
que Esc funcione; hover e troca de tamanho não chamam set_focus.

window.rs usa o monitor primário (não a área útil descontando a barra de tarefas),
sua origem física, resolução e escala. A margem superior é de 8 pixels lógicos:
8 físicos a 100%, 12 a 150% e 16 a 200%. Os tamanhos lógicos são 180×36 recolhida e
340×230 expandida; a conversão para pixels físicos usa o DPI do monitor primário.
Origens negativas de monitores também são consideradas. Mudanças de DPI refazem
esse cálculo por evento, sem polling. Cada set_mode consulta novamente o monitor.

set_mode ajusta posição e tamanho nativos e só então atualiza o modo da coleta.
Se a notificação da thread falhar, tenta restaurar a geometria anterior. Não existe
uma janela grande com uma pequena região desenhada: a área nativa recolhida é
apenas 180×36 lógicos, deixando livres os cliques fora desse retângulo.

No topo de src/main.ts ficam INTERACTION_MODE ("hover-click" ou "click-only"),
EXPAND_DELAY_MS (200) e COLLAPSE_DELAY_MS (400). Entrar agenda expansão; sair cancela
a expansão pendente e agenda recolhimento. Reentrar cancela o recolhimento pendente.
Não há timers periódicos na UI, apenas timeouts de uma interação em andamento.

Um clique fixa aberta; novo clique ou Esc libera a fixação. Em hover-click,
liberar enquanto o ponteiro permanece dentro mantém a janela aberta pelo hover;
liberar com o ponteiro fora recolhe. Em click-only, liberar recolhe imediatamente.
Esc funciona quando a ilha recebe o teclado, por exemplo após clicar nela; não há
atalho global nesta fase. Enter/espaço também alternam a fixação com a ilha focada.

A UI serializa comandos: se o modo desejado mudar enquanto um comando está em
andamento, aplica o próximo depois da resposta. O conteúdo expandido só aparece
após confirmar o modo, e métricas recolhidas não geram atualização de texto no DOM.
O fundo azul acinzentado e a fonte do sistema são provisórios; glass, fonte pixel,
paleta final e barras segmentadas continuam reservados à Fase 4.

Nenhuma dependência do aplicativo foi adicionada ou atualizada.
Referências oficiais consultadas:

- [Tauri: WindowConfig](https://v2.tauri.app/reference/config/#windowconfig).
- [Tauri: APIs da WebviewWindow](https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewWindow.html).
- [Tauri: coordenadas lógicas e físicas](https://v2.tauri.app/reference/javascript/api/namespacedpi/).
