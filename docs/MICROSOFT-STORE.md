# Distribuição do PixelPulse pela Microsoft Store

O lançamento da loja usa **MSIX x64** para Windows 11 com WebView2 Runtime.
O Windows gerencia a instalação e as atualizações. A localização usual é
`C:\Program Files\WindowsApps\<identidade_do_pacote>` no volume padrão de
aplicativos escolhido pelo sistema. Para consultar uma instalação, use
`Get-AppxPackage -Name "<Package/Identity/Name>" | Select-Object InstallLocation`.

As preferências ficam em `<perfil do usuário>\.pixelpulse\settings.json`.
Esse arquivo pode permanecer após desinstalar. Inicialização é por usuário e
começa desativada. A variante MSIX usa `StartupTask`. No executável avulso,
o checkbox acompanha a entrada PixelPulse no Registro. Desmarcar remove a
entrada, e marcar grava o caminho da cópia em execução. Se mover o executável,
desmarque e marque novamente. Uma política ou opção de inicialização do Windows
ainda pode impedir a execução. Desative a inicialização da variante avulsa antes
de habilitar a versão da loja para evitar duas ilhas no login.

## 1. Prepare a conta e reserve o produto

1. Entre no [Partner Center](https://partner.microsoft.com/dashboard) com a conta
   individual inscrita no programa Windows e confirme que a verificação da conta e o
   contrato de desenvolvedor estão concluídos. Use o nome público que deseja exibir.
   Revise os dados de contato. Para produto pago, configure também os perfis fiscal e
   de pagamento conforme as instruções do portal.
2. No workspace **Apps and games**, crie um produto do tipo **MSIX ou PWA** e reserve
   **PixelPulse** se estiver disponível. Use exatamente o nome reservado no pacote
   e na ficha. Caso o nome esteja indisponível, ajuste o nome nos manifestos,
   executável e materiais antes de submeter.
3. Em **Product management → Product identity**, copie três valores da ficha do
   produto: **Package/Identity/Name**, **Package/Identity/Publisher** e
   **Package/Properties/PublisherDisplayName**. O identificador Tauri
   `com.renahangil.pixelpulse` continua sendo interno. Use a identidade fornecida
   pela Store no MSIX e mantenha a mesma em futuras atualizações.

Referência: [Primeiros passos na Store](https://learn.microsoft.com/en-us/windows/apps/publish/get-started).

## 2. Gere e valide o pacote

1. Instale o **Windows SDK** com as ferramentas MakeAppx e SignTool. Use a máquina
   x64 com as dependências de build do projeto.
2. A partir da raiz, gere o executável e use os três valores do portal:

   ```powershell
   npm ci
   npm run tauri build -- --no-bundle

   .\src-tauri\packaging\build-msix.ps1 `
     -IdentityName '<Package/Identity/Name>' `
     -Publisher '<Package/Identity/Publisher>' `
     -PublisherDisplayName '<Package/Properties/PublisherDisplayName>' `
     -Version '0.2.0.0'
   ```

3. Encontre o pacote em `src-tauri/target/msix/`. Para cada atualização, aumente
   a versão do pacote e deixe seu quarto componente em `0` para uso da Store.
   No manifesto, `MinVersion` estabelece Windows 11 como piso inicial e
   `MaxVersionTested` registra a família usada como referência. Revise com os
   resultados da matriz de teste antes da submissão. Mantenha `TaskId` e o ID do
   aplicativo estáveis para as atualizações.
4. Antes da submissão, instale uma cópia de teste assinada com certificado de
   desenvolvimento cujo assunto corresponde ao Publisher de teste, em uma máquina
   ou VM de teste que confie expressamente nesse certificado. Use uma identidade
   de teste separada, por exemplo `PixelPulse.LocalTest` e
   `CN=PixelPulse Local Test`. Execute o Windows App Certification Kit (WACK)
   quando disponível e teste a instalação limpa, atualização, desinstalação,
   inicialização desativada, opt-in no login, fechamento da ilha e persistência.
   Verifique também que o WebView2 Runtime está disponível na instalação suportada.
5. Envie o **MSIX com a identidade real** na seção **Packages**. A Store cuida
   da assinatura de distribuição após a certificação. O MSIX gerado localmente é
   sem assinatura de distribuição. Assinatura local para sideload é um passo
   distinto e a Store não assina o executável avulso.

Referências: [MakeAppx](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool),
[assinatura](https://learn.microsoft.com/en-us/windows/msix/package/signing-package-overview),
[armazenamento no MSIX](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes).

## 3. Preencha a ficha no Partner Center

| Seção | Dados propostos ou a preparar |
| --- | --- |
| Pricing and availability | Gratuito como proposta inicial. Escolha mercados, audiência pública ou privada, visibilidade e data de lançamento. |
| Properties | Categoria de utilitários/ferramentas, conforme o catálogo exibido. Informe desenvolvedor, copyright, termos, suporte e URL de privacidade quando exigida. Responda aos campos do portal conforme as funcionalidades reais. |
| Age ratings | Complete o questionário IARC. Responda sobre conteúdo, rede, dados, compras e interação com as características do build submetido. |
| Packages | MSIX Windows Desktop x64. Confira nome, Publisher, versão e arquitetura antes de enviar. |
| Store listings | Prepare nome, descrição, lista de recursos, novidades, capturas reais, logotipo e termos de busca em inglês, português brasileiro e espanhol. Use as dimensões e formatos indicados no portal. |
| Submission options | Explique `runFullTrust` para executar o aplicativo desktop Tauri, consultar métricas do Windows e manter a janela flutuante. Documente a tarefa de inicialização desativada por padrão e o teste do menu. |

**Descrição sugerida em inglês:** PixelPulse is a compact pixel-art system
monitor for Windows. Keep CPU, memory, graphics and storage usage in sight with a
floating panel, and choose which local drives to monitor.

**Descrição sugerida em português:** PixelPulse é um monitor de sistema em pixel
art para Windows. Acompanhe CPU, memória, GPU e espaço ocupado numa ilha flutuante
e escolha quais unidades locais exibir.

**Descrição sugerida em espanhol:** PixelPulse es un monitor de sistema con arte
pixelado para Windows. Consulta CPU, memoria, GPU y espacio ocupado en un panel
flotante y elige las unidades locales que quieres mostrar.

**Notas para a certificação:** ao abrir, a janela pequena aparece centralizada
no topo do monitor principal. Passe o mouse para expandir e clique para fixar.
No botão direito, abra **System Diagnostics** ou use **Exit PixelPulse** para
encerrar. Em **Devices**, selecione um volume extra. **Startup** é opt-in.
**Language** permite alternar entre inglês americano, português brasileiro e espanhol.
GPU, tipo de disco e métricas de hardware dependem das informações que o Windows
e os drivers oferecem. Dados indisponíveis têm indicação na tela.

**Privacidade e licença:** descreva o inventário e as leituras locais, as
preferências gravadas e a ausência de envio desses dados pelo PixelPulse. Antes de
distribuir sob GPL-3.0, providencie acesso público ao código-fonte correspondente
ao binário e às instruções de build, inclusive as partes necessárias para
gerar o MSIX. Uma branch em repositório privado não dá esse acesso aos usuários.
O pacote inclui `LICENSE` e o executável inclui a licença OFL da fonte.
Revise os termos de distribuição e a ficha conforme o pacote final.

Com todos os campos concluídos, selecione **Submit for certification**. Se a
certificação pedir ajustes, consulte o relatório, corrija o problema e aumente a
versão do pacote antes de reenviar. O processo de publicação e a conta do
Partner Center são operados pelo titular.

Referências: [Submissão MSIX e campos](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/create-app-submission),
[erros de submissão](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/resolve-submission-errors),
[GPL-3.0](https://www.gnu.org/licenses/gpl-3.0.html) e
[checklist da Store](https://learn.microsoft.com/en-us/windows/apps/publish/store-readiness-checklist).
