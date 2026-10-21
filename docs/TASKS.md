# Tarefas
Cada linha contém objetivo, módulos, dependências e aceitação; verificações finais em PROGRESS.md.

| ID | Objetivo / módulos | Dependências | Aceitação / testes | Estado |
|---|---|---|---|---|
| F01 | Workspace, instruções, CI / raiz | Rust/GTK | build, fmt, clippy | IN PROGRESS |
| D01 | Modelos, CRUD e migrações / core,storage | SQLite, Secret Service | roundtrip, validação, exclusão | IN PROGRESS |
| M01 | Decode, áudio, sessões / media | libav, PulseAudio | stream local, cancelamento, frames | IN PROGRESS |
| U01 | Dashboard, biblioteca e Views / ui | D01,M01 | integração e teste GTK | TODO |
| U02 | Editor, foco e zoom / ui,core | U01 | layout válido e persistência | TODO |
| O01 | SOAP e descoberta / onvif | rede | fixtures, servidor local | TODO |
| P01 | PTZ e presets / onvif,ui | O01 | Stop, timeout, autorização | TODO |
| R01 | Release e AppImage / packaging | todas | release e smoke isolado | TODO |
| H01 | Matriz de câmeras e desempenho | hardware físico | 4x1080,9x720,16 painéis, A/V, PTZ | TODO |
