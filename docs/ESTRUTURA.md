# Estrutura — PixelPulse

Mapa da Fase 0; módulos funcionais serão criados somente nas fases correspondentes.
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
- `src/index.html`: página inicial estática com o texto PixelPulse, antes na raiz do template.
- `src/main.ts`: ponto de entrada TypeScript, ainda sem comportamento dinâmico.
- `src/styles.css`: estilo mínimo provisório da página; não antecipa o visual da Fase 4.
- `src/styles/`: pasta reservada aos estilos e tokens visuais das próximas fases.
- `src/styles/.gitkeep`: preserva a pasta de estilos ainda vazia.
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
- `src-tauri/tauri.conf.json`: nome PixelPulse, janela, caminhos da UI e empacotamento.
- `src-tauri/src/`: código Rust do aplicativo.
- `src-tauri/src/main.rs`: entrada do executável Windows que chama a biblioteca do app.
- `src-tauri/src/lib.rs`: inicialização do Tauri, sem o comando de exemplo greet.
- `src-tauri/src/collectors/`: pasta reservada aos coletores de métricas da Fase 1.
- `src-tauri/src/collectors/.gitkeep`: preserva a pasta de coletores ainda vazia.
- `src-tauri/src/cache/`: pasta reservada ao cache da Fase 2.
- `src-tauri/src/cache/.gitkeep`: preserva a pasta de cache ainda vazia.
- `src-tauri/src/ai/`: pasta reservada à leitura local de contagens de IA, apenas na Fase 6.
- `src-tauri/src/ai/.gitkeep`: preserva a pasta de IA sem antecipar sua implementação.
- `src-tauri/examples/`: pasta reservada aos programas de teste em linha de comando.
- `src-tauri/examples/.gitkeep`: preserva a pasta de exemplos ainda vazia.
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

`commands.rs`, `window.rs`, os arquivos dos coletores, `styles/tokens.css`,
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
