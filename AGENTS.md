# CCTV View — instruções canônicas
Leia este arquivo, docs/PROGRESS.md, docs/TASKS.md e a documentação do módulo antes de alterar código.
Produto: monitor de câmeras Linux nativo, em português, GTK4/gtk4-rs, Rust stable, FFmpeg/libav, SQLite e Secret Service. Não usar frontend web, subprocessos para o player nem credenciais no banco/logs.

## Módulos
- core: modelos, validação, layouts; sem GTK/rede.
- storage: migrações SQLite, XDG e chaveiro do sistema.
- media: FFI documentada para libav e PulseAudio, sessões compartilhadas e cancelamento.
- onvif: descoberta WS-Discovery e SOAP autenticado, perfis e PTZ.
- ui: widgets GTK, editor, biblioteca e apresentação de eventos.
- app: inicialização, logging e runtime.
Dependências seguem app → ui → serviços → core; evitar ciclos.

## Convenções
Rust formatado com rustfmt, Clippy sem warnings. Erros retornam Result; não usar unwrap em fluxos de produção. FFI exige comentário de segurança e propriedade explícita. Alterações arquiteturais exigem ADR. Atualizar documentação e testes na mesma entrega.
GTK somente na thread principal; rede, SQLite, chaveiro e decode em workers. Filas limitadas, último frame substitui obsoleto. Cancelamento e timeout obrigatórios. Não registrar URLs autenticadas, senhas nem SOAP completo. TLS validado por padrão. PTZ exige capacidade, autorização e parada segura.
Usar widgets e tema nativos, nomes acessíveis, teclado, estados vazio/erro. CSS apenas local.

## Verificação
Dependências e ambiente: docs/BUILD.md. Comandos: scripts/check.sh; cargo fmt --all -- --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace; cargo build --release.
Aceitar somente funcionalidades integradas, compiladas e verificadas. Registrar NOT VERIFIED para hardware, GUI ou distribuição não testados. Não declarar primeira versão completa enquanto houver critérios pendentes em docs/FEATURES.md.

## Continuidade
Manter docs/PLAN.md, TASKS.md, PROGRESS.md, FEATURES.md e CHANGELOG.md sincronizados. Antes de encerrar, registrar verificações, limitações e próxima ação. Commits pequenos autorizados pelo usuário; neste projeto somente commits locais, sem push/PR.
