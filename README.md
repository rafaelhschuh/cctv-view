# CCTV View

Aplicativo Linux nativo para monitoramento de câmeras IP, em Rust, GTK 4 e FFmpeg/libav. Interface em português e tema do sistema, com prioridade para Zorin OS 18 x86_64.

A versão **0.1.0** integra cadastro de câmeras, reprodução multistream, Views persistentes, editor visual, foco, zoom digital, áudio exclusivo, descoberta ONVIF, perfis, PTZ e presets. É uma versão inicial: validação em câmeras físicas, drivers VA-API e em outro computador permanece pendente. Consulte a [matriz de validação](docs/FEATURES.md) antes de usar em operação crítica.

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
2. Informe usuário e senha nos campos protegidos. Consulte os perfis ONVIF, selecione o stream e autorize PTZ se necessário. Use **Testar conexão de vídeo** antes de salvar.
3. Crie uma **Nova View**, escolha um layout ou arraste câmeras para a grade. Arraste painéis para reposicionar; use a alça ↘ para redimensionar. Clique para configurar câmera, posição, tamanho e proporção.
4. Salve a View. Duplo clique em um painel ativa foco; roda do mouse faz zoom e arrasto faz pan. Escape retorna; F11 alterna tela cheia.
5. No foco, habilite áudio ou abra o PTZ, quando disponível. Segure os controles PTZ; solte para parar. As setas do teclado também controlam o movimento na janela PTZ.

Dados seguem XDG e credenciais ficam no chaveiro Secret Service. Não existe telemetria, servidor de aplicação nem execução como root.

[Documentação](docs/README.md) · [Arquitetura](docs/ARCHITECTURE.md) · [Testes](docs/TESTING.md) · [Progresso](docs/PROGRESS.md)
