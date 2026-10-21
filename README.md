# CCTV View

Aplicativo Linux nativo para monitoramento de câmeras IP, em Rust, GTK 4 e FFmpeg/libav. Interface em português e tema do sistema, com prioridade para Zorin OS 18 x86_64.

A versão **0.1.3** integra cadastro de câmeras, reprodução multistream, Views persistentes, editor visual, foco, zoom digital, áudio exclusivo, descoberta ONVIF, perfis, PTZ e presets. É uma versão inicial: validação em câmeras físicas, drivers VA-API e em outro computador permanece pendente. Consulte a [matriz de validação](docs/FEATURES.md) antes de usar em operação crítica.

## Executar

O pacote local é gerado em `dist/CCTV-View-x86_64.AppImage`:

```sh
chmod +x dist/CCTV-View-x86_64.AppImage
./dist/CCTV-View-x86_64.AppImage
```

Sem FUSE: `APPIMAGE_EXTRACT_AND_RUN=1 ./dist/CCTV-View-x86_64.AppImage`.

Para desenvolvimento, instale as [dependências nativas](docs/BUILD.md) e execute `scripts/dev.sh`. Verificações: `scripts/check.sh`.

## Usar

1. Abra **Câmeras** na barra superior; adicione uma URL RTSP/HTTP/HTTPS ou descubra um dispositivo ONVIF na rede.
2. Na aba **Conexão**, escolha **RTSP**, **HTTP(S)**, **MJPEG** ou **JPEG**, e informe a URL completa e as credenciais. Host e porta são derivados da URL. Na aba **ONVIF**, salve o endpoint mesmo sem consultar ou consulte os perfis e autorize PTZ. A aba **Avançado** mostra as opções do tipo escolhido: transporte TCP/UDP no RTSP, formato multipart/contínuo no MJPEG e intervalo de atualização no JPEG (200–60000 ms). HTTP(S) detecta vídeo/HLS automaticamente e mantém a validação TLS. Timeout é comum; substream, áudio e aceleração aparecem para RTSP/HTTP(S). Use **Testar conexão de vídeo** antes de salvar.
3. Crie uma **Nova View**, escolha um layout e adicione câmeras pelo botão **+**, pela lista ou arrastando para a grade. Trocar o layout preserva os painéis; **Desfazer layout** recupera a alteração anterior. Arraste painéis para reposicionar; use a alça ↘ para redimensionar. Clique para configurar câmera, posição, tamanho e proporção.
4. Salve a View. Enter no painel, **Ampliar câmera** no menu ou duplo clique ativa foco; roda do mouse faz zoom e arrasto faz pan. **Voltar à grade** ou Escape retorna; F11 alterna tela cheia.
5. No foco, habilite áudio ou abra o PTZ, quando disponível. Segure os controles PTZ; solte para parar. Ativar um botão por teclado produz um pulso curto. Setas, diagonais (Home/Page Up/End/Page Down) e +/− também controlam o movimento; Escape para.

O menu ao lado de cada View reúne edição, duplicação, favoritos, inicialização, ordem e exclusão. Em janelas estreitas, a lista lateral recolhe automaticamente; o botão da barra permite reabri-la.

Dados seguem XDG e credenciais ficam no chaveiro Secret Service. Não existe telemetria, servidor de aplicação nem execução como root.

[Documentação](docs/README.md) · [Arquitetura](docs/ARCHITECTURE.md) · [Testes](docs/TESTING.md) · [Progresso](docs/PROGRESS.md)
