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
