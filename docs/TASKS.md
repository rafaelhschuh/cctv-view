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
| UX01 | Corrigir persistência ONVIF, conexão e cadastro / core,ui | revisão crítica | endpoint salvo/invalidação, derivação de host/porta, campos agrupados e feedback fixo | DONE |
| UX02 | Editor acessível e não destrutivo / core,ui | UX01 | adicionar por clique/teclado/DnD, reflow/undo, erro local, nome legível | DONE |
| UX03 | Navegação, foco, Views e PTZ / ui | revisão crítica | ações próximas, vazio acionável, sidebar adaptativa, controles acessíveis e pulso limitado | DONE |
| UX04 | Validar e empacotar revisão / tests,docs,packaging | UX01–UX03 | fmt/Clippy/testes/release, GTK pequeno/grande, AppImage e commits locais | DONE |
| UX05 | Compactar rodapé, remover Pronto e alinhar Views / ui | UX04 | screenshot e regressão GTK com nomes longos, badges e erro | DONE |
| M02 | Tipos RTSP/HTTP(S)/MJPEG/JPEG e opções / core,storage,media,ui | UX05 | formatos explícitos, JPEG periódico/cancelamento, JSON legado, GTK e AppImage | DONE |
| UX06 | Corrigir regressão de densidade da lista / ui | UX05 | uma linha nativa, indicadores inline, seleção/foco, GTK claro/escuro e AppImage | DONE |
| A01 | Streams públicas e certificado configurável / core,storage,media,onvif,ui | M02/UX06 | defaults legados, GTK/SQLite sem chaveiro, TLS default/exceção em vídeo/SOAP e AppImage | DONE |
| U03 | Foco, apresentação e clipping / ui | UX06 | duplo clique alterna, X sobreposto, fullscreen sem chrome, bordas com frame | DONE |
| U04 | Dashboard 12×12 / core,ui | U02 | escala legada, presets sem perda, desenho/move/resize e undo/SQLite | DONE |
| A02 | Senha do aplicativo / storage,ui | confirmação do usuário | hash/salt, senha errada, alteração/remoção, startup e bloqueio imediato | DONE |
| C01 | CLI de scripts / app,ui,storage | A02/U03 | help/version/list sem display, abertura View/fullscreen, biblioteca protegida/stdin | DONE |
| U05 | Biblioteca, pastas e clonagem / core,storage,ui | U01 | filtro/busca, linhas alinhadas, persistência, rascunho sem senha e preservação de Views | DONE |
| U06 | Preferências e transferência de câmeras / storage,ui | U05/A02 | abas, revisão de URL, JSON sem chaveiro, limites, importação atômica/aditiva e AppImage | DONE |
| A03 | Reautenticar ações de dados e criptografar transferência / storage,ui | U05/U06/A02 | credenciais/URLs completas, senha por ação, AEAD, rollback, chaveiro GTK isolado e AppImage | DONE |
| R03 | Workflow GitHub AppImage e GPLv3 / workflows,packaging,docs | R01 | actionlint, crates GPL-3.0-only, build/pacote/smoke local e artifact configurado | DONE |
