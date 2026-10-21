# Banco e persistência

SQLite em `$XDG_DATA_HOME/br.dev.schuh.cctv-view/cctv.db`, com fallback `~/.local/share/…`. O diretório é criado com modo 0700. CCTV_VIEW_DATA_DIR permite isolamento para testes. Não se grava dentro do AppImage.

A migração 1 é transacional e registrada em PRAGMA user_version. Versões futuras desconhecidas são rejeitadas. Há WAL, foreign_keys ON e busy_timeout de três segundos.

- cameras: id primário e JSON validado com nome, protocolo, URLs sem userinfo, configurações, perfis, capacidades, last_status e referência ao chaveiro.
- views: id primário e JSON validado com ordem, favorito, dimensões da grade e painéis referenciando câmeras existentes.
- settings: chave/valor JSON; preferences contém View atual/inicial, janela, sidebar e volume.

Excluir câmera remove seus painéis de todas as Views na mesma transação. save_view valida existência das câmeras, limites e sobreposição. Como as relações ficam em JSON, a integridade entre painéis e câmeras é aplicada pela camada Store, não por chaves estrangeiras SQL.

O segredo nunca integra o JSON Camera. libsecret guarda uma tupla usuário/senha sob o atributo camera. Bancos e chaveiros são recursos distintos: alterações não são uma transação distribuída; uma falha entre os dois pode deixar um segredo órfão, mas não faz fallback para texto puro. Faça backup do banco fechado ou pela API SQLite; copiar apenas cctv.db com WAL ativo não é um backup seguro.

A partir de 0.1.2, Camera inclui stream_kind, snapshot_interval_ms e mjpeg_multipart. Os defaults serde preservam JSONs anteriores como Auto/1000/true; roundtrip legado testado. O esquema SQL continua na migração 1 e não apaga/regrava registros para introduzir esses campos.

Em 0.1.4, no_credentials (padrão false) e tls_verify (padrão true) são persistidos no JSON Camera. Defaults serde mantêm registros antigos autenticados conforme a referência existente e certificados validados. No modo anônimo, a referência ao chaveiro é preservada sem carregá-la; somente a opção explícita de remoção a apaga.

Em 0.2.0, settings.app_lock contém versão 1, salt aleatório de 16 bytes e hash de 32 bytes PBKDF2-HMAC-SHA256 (600.000 iterações). Não contém senha legível. Criar/alterar/remover usa transação IMMEDIATE com verificação da senha atual. Ausência do registro desativa o bloqueio; registro inválido falha fechado. Esquema SQL permanece versão 1. Grades 12×12 usam os campos de geometria existentes.

Em 0.3.0, Camera.folder é um nome de pasta opcional com default None para registros antigos. settings.camera_folders guarda os nomes (incluindo pastas vazias), ordenados sem distinção de caixa. Criar/renomear/excluir/mover pastas e importar usam transação IMMEDIATE; renomear/excluir atualiza somente o campo folder dos JSONs, sem regravar credenciais ou Views. Excluir pasta move suas câmeras para Sem pasta. Não há migração SQL destrutiva: user_version continua 1. Importar agrega câmeras com UUIDs novos e mescla pastas de nomes iguais sem distinção de caixa.

O formato de transferência é independente do JSON interno: format=cctv-view-cameras, version=1, folders e cameras (somente campos permitidos). Limites: 8 MiB, 2000 câmeras, 500 pastas. Todos os cadastros e referências de pasta são validados antes de iniciar a transação, sem alterações parciais em caso de erro. Não leva chaveiro, senha do aplicativo nem Views.

Em 0.4.0, a transferência externa passa a ser um envelope JSON criptografado (format=cctv-view-encrypted, version=1). O conteúdo interno é CameraArchive versão 2 com câmeras completas e credenciais; secret_ref é removido e novos UUIDs/referências são criados ao importar. Nenhuma senha de câmera vai ao SQLite: somente as referências novas, com os segredos no Secret Service. Query e fragmento de URLs permanecem intactos. O esquema SQL continua versão 1 e bibliotecas existentes não são alteradas para adotar a criptografia do arquivo.

Clone e importação validam antes de gravar; a transação SQLite IMMEDIATE abrange a criação dos cadastros. Credenciais são gravadas sob novas referências e apagadas compensatoriamente se a gravação SQL/chaveiro falhar. SQLite e Secret Service não têm transação distribuída: queda do processo ou falha na limpeza pode deixar segredo órfão; originais não são sobrescritos. A senha do aplicativo e Views não integram o arquivo. O JSON não criptografado de 0.3.0 é histórico e não é aceito neste novo fluxo.
