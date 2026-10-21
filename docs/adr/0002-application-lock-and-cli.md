# ADR 0002 — Bloqueio da aplicação e CLI

Status: adotado em 2026-10-21.

O usuário solicitou CLI para inicialização e confirmou que a nova senha deve bloquear o acesso ao aplicativo. Credenciais de câmera permanecem separadas.

App analisa opções antes do GTK; listagens JSON sem display retornam metadados mínimos e exigem autenticação quando houver app_lock. Senha só entra por GUI ou stdin, sem argumento. LaunchOptions configura View, tela cheia e biblioteca, com instâncias independentes para abertura explícita. SQLite/KDF da GUI ficam em workers.

Storage mantém registro app_lock versionado em settings, PBKDF2-HMAC-SHA256 com 600.000 iterações, salt randômico de 16 bytes e hash de 32. Usa ring 0.17.14 já transitivo no projeto, sem novo shim C ou algoritmo criptográfico artesanal. Senha atual é obrigatória para alterar/remover; transação IMMEDIATE evita troca concorrente entre verificação e gravação. Não há senha legível no banco. APIs de core/media/ONVIF não dependem dessa política de apresentação.

Gate antecede build/playback. Bloqueio imediato fecha diálogos, encerra sessões e impede render/atalhos de expor conteúdo; sucesso restaura o root e cria sessões novas. Callbacks tardios não restauram janela fechada. KDF usa verificação da biblioteca ring e atraso extra no erro da GUI.

Limite: protege a interface/listagens do programa; não é criptografia de dados nem uma fronteira contra o próprio dono da conta Linux, que pode editar/copiar os arquivos. Não há recuperação embutida. Os testes cobrem mudanças/remoção, salt, ausência de plaintext, CLI e GTK; nenhuma certificação externa é alegada.
