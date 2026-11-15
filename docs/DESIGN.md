# Design — PixelPulse

Especificação visual da Fase 4. Os dados continuam sendo coletados a cada 5 s
recolhido e 2 s expandido; hover de 200 ms, saída de 400 ms e fixação por clique/Esc
permanecem como na Fase 3.

## Grade e dimensões

- Unidade de arte: `--px: 3px`; um quadrado de arte ocupa 3×3 pixels CSS/lógicos.
- Janela recolhida: 180×36 lógicos; expandida: 340×230, posicionadas pelo Rust/DPI.
- A margem do monitor permanece 8 pixels lógicos; é geometria nativa da Fase 3.
- Cantos: raio de 4 px, a exceção solicitada à grade para um contorno quase reto.
- Ícones: SVG inline próprio com viewBox 0 0 8 8, desenhado só em coordenadas inteiras,
  exibido em 24×24 px, com shape-rendering: crispEdges e image-rendering: pixelated.
- Pontos recolhidos: 2×2 pixels de arte (6×6 px), separados por 3 pixels de arte (9 px).
- Barras: 16 blocos; cada bloco mede 3×4 pixels de arte (9×12 px), com intervalo de 3 px.
- Linha expandida: 30 px de altura; ícone 24, rótulo 30, barra 189 e valor 42 px,
  com três intervalos de 9 px. Recuo esquerdo de 12 px; conteúdo ocupa 312 px.
- Os 340×230 nativos não são múltiplos de 3: preservamos a dimensão pedida e alinhamos
  o conteúdo à grade a partir da origem, deixando a sobra na margem direita/inferior.

## Paleta

As cores vivem em src/styles/tokens.css. Cada métrica tem um tom claro para a linha
superior, base para o corpo e escuro para a linha inferior de cada bloco aceso.
Não há neon, glow nem sombras externas.

| Métrica | Claro | Base | Escuro |
| --- | --- | --- | --- |
| CPU — azul | #91b0cc | #6c8da9 | #48637a |
| RAM — violeta | #b0a0c8 | #8e7ba8 | #645574 |
| GPU — verde | #9eb8a0 | #7c9b80 | #536c56 |
| SSD — âmbar | #c5b08a | #a58d63 | #746044 |
| IA — coral | #d1a393 | #b28373 | #7e594e |

IA tem somente tokens reservados: o coletor e sua apresentação ficam para a Fase 6.
Texto principal: #d8d8de. Texto secundário: #8f8fa0.
O trilho usa xadrez CSS de #24242e e #282832, em células de 3 px, quase iguais entre si;
a borda superior é #30303a e a inferior #191920.

## Glass

A assinatura oficial consultada de window-vibrancy 0.8.1 é
`apply_acrylic(window: impl HasWindowHandle, color: Option<Color>) -> Result<(), Error>`;
Color é uma tupla (u8, u8, u8, u8), RGBA. A tentativa usa (20,20,26,72): tinta escura
com alpha de aproximadamente 28%. Sobre ela, a superfície CSS usa rgba(20,20,26,0.3).

O plano B é exatamente rgba(20,20,26,0.6), sem blur/backdrop-filter CSS nem desfoque
do sistema. Falha ao aplicar acrylic seleciona esse plano automaticamente. A decisão
da comparação local é registrada em Pendências de docs/PLANO.md. A variável de
ambiente PIXELPULSE_GLASS=acrylic ou fallback permite comparar as alternativas sem
introduzir uma tela de configurações. O modo efetivo chega à UI por get_glass_mode.

O raio CSS de 4 px e overflow:hidden recortam a superfície da UI. A documentação
oficial alerta para atraso do acrylic em determinados builds do Windows; a validação
nativa do compositor em outros drivers/DPI continua sendo assunto da Fase 5.

## Fonte local e licença

Uma única face: Silkscreen Regular 400, de aparência bitmap/pixel, no arquivo
src/assets/fonts/silkscreen-regular-latin.woff2, com 8.404 bytes (menos de 30 KB).
Usada apenas nos rótulos, marca e números. Textos auxiliares usam a fonte do sistema.
Não há outra face, peso sintetizado, biblioteca de ícones ou recurso remoto em runtime.

Licença: SIL Open Font License 1.1, confirmada no repositório oficial de fontes Google.
Copyright: 2001 The Silkscreen Project Authors (https://github.com/googlefonts/silkscreen).
A fonte mantém sua licença OFL; o código do projeto permanece GPL-3.0.
A cópia integral está em src/assets/fonts/OFL.txt e também integra o bundle via
link rel=license, para acompanhar inclusive o executável distribuído sozinho.

- [Licença original](https://raw.githubusercontent.com/google/fonts/main/ofl/silkscreen/OFL.txt).
- [Família original](https://github.com/google/fonts/tree/main/ofl/silkscreen).
- [WOFF2 Regular latino original](https://fonts.gstatic.com/s/silkscreen/v6/m8JXjfVPf62XiF7kO-i9YLNlaw.woff2).
- SHA-256 WOFF2: E6C72EA4702249202BCDD79D3343057E4E25EF1F04E3FCFFD8602AB53B40B4CC.

O subconjunto latino inclui letras portuguesas e o traço usado para N/D. O arquivo
foi obtido do CDN oficial durante o desenvolvimento; o aplicativo carrega somente
os assets locais emitidos pelo Vite e incorporados pelo Tauri.

## Estados e atualizações

Recolhida: quatro pontos, na ordem CPU/RAM/GPU/SSD, e o maior percentual disponível.
A GPU indisponível não participa desse máximo. Expandida: quatro linhas com ícone,
rótulo, barra e percentual inteiro; o SSD representa espaço usado do disco do sistema.
GPU ou sensor de uso indisponível: 16 blocos apagados e traço (—), com N/D no texto
acessível. Zero real é mostrado como 0%, também com todos os blocos apagados.

Cada barra cria 16 spans uma única vez. A quantidade acesa é round(percentual/100×16),
limitada a 0–16. A UI altera somente as classes no intervalo entre a quantidade
anterior e a nova; rótulos, saídas e atributos acessíveis também evitam escritas iguais.
Valores reservam 42 px por linha e 60 px no resumo, impedindo o deslocamento das colunas.
O resumo oculto e as linhas ocultas não são atualizados, e document.hidden continua
suspendendo escritas visuais. Não há canvas, imagens grandes ou loops de animação.

## Movimento

A entrada do conteúdo expandido dura 120 ms em steps(3,end), usando somente opacity
e transform (deslocamento inicial de 3 px). As barras mudam instantaneamente.
Nenhuma animação usa tamanho, filtro, cor, sombra ou layout. prefers-reduced-motion:
reduce desativa essa entrada. O redimensionamento nativo continua imediato, sem
interpolação de largura/altura na UI.

## Como ler o CSS

- tokens.css é a caixa de tintas e medidas; mudar um token troca seu uso em toda a UI.
- window.css monta a grade e escolhe o fundo conforme o glass disponível.
- Cada linha define --tone-light/base/dark; os blocos herdam esses três tons.
- .block desenha o trilho; .is-lit troca a base e as duas linhas de sombreamento.
- ::before e ::after são as linhas superior e inferior, sem criar spans extras.
- O xadrez é um gradiente CSS estático minúsculo, sem arquivo de imagem.
- A largura fixa dos números mantém a barra e o rótulo no mesmo lugar ao passar de 9% para 100%.

Referências: [apply_acrylic](https://docs.rs/window-vibrancy/0.8.1/window_vibrancy/fn.apply_acrylic.html)
e [clear_acrylic](https://docs.rs/window-vibrancy/0.8.1/window_vibrancy/fn.clear_acrylic.html).
