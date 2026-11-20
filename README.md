# PixelPulse

Monitor local de CPU, RAM, GPU e armazenamento para Windows 11. As métricas
aparecem em uma ilha flutuante com visual pixel art.

- Passe o ponteiro sobre a ilha para expandir. Clique para fixar e use **ESC**
  para liberar.
- No menu de contexto, abra **System Diagnostics** para consultar o hardware,
  escolher volumes adicionais, configurar o início com o Windows ou mudar o idioma.
- As preferências ficam em `%USERPROFILE%\.pixelpulse\settings.json`. O aplicativo
  consulta as métricas localmente.

## Compilar no Windows

Use Windows 11 x64 com WebView2, Node.js, Rust estável e as ferramentas C++ exigidas
pelo [Tauri](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm run tauri build -- --no-bundle
```

O executável fica em `src-tauri/target/release/pixelpulse.exe`.
Veja o [guia da Microsoft Store](docs/MICROSOFT-STORE.md) para gerar e validar o MSIX,
e o [mapa do projeto](docs/ESTRUTURA.md) para localizar os módulos.

Código sob [GPL-3.0](LICENSE). A fonte Silkscreen usa [OFL 1.1](src/assets/fonts/OFL.txt).
