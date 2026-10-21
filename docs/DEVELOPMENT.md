# Desenvolvimento

Leia AGENTS.md, TASKS.md e PROGRESS.md. Preserve o fluxo de dependências e trabalhe em entregas verticais: modelo, serviço, UI, persistência, erros, testes e documentação.

`scripts/dev.sh` executa `cargo run -p cctv-view`; `scripts/build.sh` produz release; `scripts/check.sh` centraliza verificações. `CCTV_VIEW_DATA_DIR=/tmp/cctv-dev` usa uma biblioteca isolada. RUST_LOG controla tracing; não habilite logs de protocolo contendo segredos.

Para mídia e SOAP, use tests/camera_simulator.py e scripts/test-protocols.sh. Os exemplos probe são ferramentas de testes: recebem URL no argumento e credenciais pelas variáveis CCTV_TEST_USER e CCTV_TEST_PASSWORD. Nunca cole credenciais reais nos comandos, logs ou arquivos de testes.

A UI está organizada por dashboard, camera, editor, player, ptz, diagnostics e jobs. Mudanças no protocolo devem começar pelos serviços e simuladores. Código FFI precisa de comentários de segurança e limpeza em todos os caminhos. Atualize docs/FEATURES.md quando houver novas evidências; mantenha hardware não testado explícito.
