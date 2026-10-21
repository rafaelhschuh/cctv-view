# ADR 0003 — Reautenticação e transferência completa criptografada

Status: adotado em 2026-10-21; substitui a política de transferência/clone de 0.3.0.

O usuário solicitou preservar senhas e URLs nas operações de edição, clonagem e exportação, com confirmação da senha do aplicativo por ação quando configurada e criptografia do JSON com uma senha escolhida na exportação. A senha do arquivo é separada da senha do aplicativo e exigida para importar.

A UI centraliza a reautenticação antes de abrir cadastros existentes, clonar e exportar. A libsecret continua a ser o repositório de senhas locais: clone e importação geram referências independentes. A senha salva é carregada no campo PasswordEntry após autorização, permitindo seu ícone nativo de visualização. Reprodução/ONVIF/PTZ continuam a respeitar Sem credenciais.

Storage usa ring 0.17 já presente: AES-256-GCM com nonce de 96 bits, tag de 128 bits e chave derivada por PBKDF2-HMAC-SHA256/600.000 iterações com salt de 128 bits. Cada exportação gera salt e nonce novos. Metadados versionados e AAD constante delimitam o formato. Envelope JSON carrega somente ciphertext/salt/nonce Base64; o conteúdo interno versionado contém cadastros completos e credenciais, sem referências ao chaveiro do dispositivo de origem. Não há arquivo temporário em claro. Importação valida tamanho/metadados/tag/modelos antes de escrever.

O processo usa workers para KDF, AEAD, SQLite e chaveiro; GTK só apresenta o fluxo e descarta callbacks se a janela foi fechada/bloqueada. A gravação SQL é transacional; falhas apagam compensatoriamente segredos novos. Queda do processo ou falha do chaveiro na limpeza ainda pode deixar segredo órfão, pois não há transação distribuída. Não há fallback de senha legível no banco.

Arquivos de transferência em claro de 0.3.0 são incompatíveis com o novo fluxo. Banco, bloqueio do aplicativo, Views e CLI mantêm compatibilidade. Criptografia do arquivo não altera a proteção do banco local nem o limite de segurança da conta Linux. Não existe recuperação da senha do arquivo.
