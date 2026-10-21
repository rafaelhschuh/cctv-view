# Tarefas

Estados: TODO, IN PROGRESS, BLOCKED, DONE. DONE identifica o critério local descrito; não substitui a homologação física em H01/H02. Evidências e limitações em PROGRESS.md e FEATURES.md.

| ID | Objetivo / contexto / módulos | Dependências | Aceitação / testes | Estado |
|---|---|---|---|---|
| F01 | Criar workspace e convenções / raiz,app | Rust/GTK | fmt, Clippy, workspace e release | DONE |
| D01 | Modelos, CRUD, migrações, chaveiro / core,storage | SQLite/libsecret | roundtrip, validação, exclusão e Secret Service isolado | DONE |
| M01 | Decode, áudio, sessões / media | libav/PulseAudio | frames reais, quatro streams, reconexão, descarte, shutdown e PCM exclusivo | DONE |
| U01 | Dashboard, biblioteca e Views / ui | D01/M01 | cadastro e View persistentes no GTK real, foco/Escape | DONE |
| U02 | Editor e zoom / ui,core | U01 | sinal drop GTK, resize numérico, layout válido, cancelamento, GDK snapshot | DONE |
| O01 | Descoberta e SOAP / onvif | rede/Tokio | UDP correlacionado; servidor autenticado com informações, serviços/perfis/URI | DONE |
| P01 | PTZ/presets / onvif,ui | O01 | XML e operações reais no mock; stop e timeout, controles integrados | DONE |
| R01 | Release, CI e AppImage / scripts,packaging | todas | release, pacote gerado e smoke por extração no host | DONE |
| H01 | Homologar câmeras e desempenho / testes de campo | hardware físico | 4×1080p,9×720p,16 painéis, RTSP UDP, A/V, VA-API e PTZ real | TODO |
| H02 | Homologar desktop e distribuição | VM/segunda máquina | Wayland, HiDPI, temas, leitor de tela, FUSE e AppImage separado | TODO |
| R02 | Preparar distribuição pública | R01/H01/H02 | licenças e fontes correspondentes, inventário de bibliotecas e matriz final | TODO |
| O02 | Ampliar compatibilidade de fabricantes | dispositivos que exijam suporte | Media2, HTTP Digest SOAP e desvio de relógio com fixtures dedicadas | TODO |
