# Progresso técnico — 2026-10-21

## Entrega implementada

Repositório vazio transformado em workspace com seis crates. Core valida câmeras/URLs, painéis, sobreposição e layouts. Storage tem migração SQLite 1, CRUD, exclusão transacional de referências, preferências e integração libsecret. Status de conexão é persistido separadamente para não sobrescrever alterações concorrentes da configuração.

Mídia usa libav nativa e workers por sessão, com compartilhamento por Weak/Arc, uma posição de frame, backoff, interrupção de I/O, decodificação VA-API opcional/fallback e áudio PCM via PulseAudio. ONVIF tem descoberta UDP, autenticação WS-Security digest, informações, serviços, Media1, perfis, URIs, PTZ e presets.

A interface GTK oferece biblioteca, teste de conexão, Views/favoritos/ordem/inicial, editor com DnD/resize/preview/cancelamento, foco, zoom/pan, áudio exclusivo, PTZ e diagnóstico. Configurações e janela persistem. Usa CSS somente para contraste das mensagens sobre vídeo; não usa frontend web, telemetria nem subprocessos de reprodução.

## Decisões e correções

ADR 0001 registra a fronteira Rust/C das bibliotecas nativas, GDK MemoryTexture e concorrência. A revisão corrigiu reutilização de credenciais/sessões após editar câmera, ordem de gravação de preferências, gerações de comandos PTZ, correlação de descoberta, contagem de frames descartados e preservação de células vazias. O HeaderBar é criado antes da janela inicial ser apresentada e reutilizado após o carregamento, evitando substituição de titlebar em janela já realizada. Fechar a janela desfaz ciclos de sinais GTK, cancela sessões e drena preferências; uma regressão verifica a liberação do State.

## Evidências realizadas

- Auditoria cargo-audit 0.22.2: quick-xml atualizado para 0.42.0 após dois avisos; auditoria final com zero vulnerabilidades/avisos em 255 dependências. Parser reforçado e regressões repetidas.
- `cargo fmt --all -- --check`, Clippy com `-D warnings`, suíte workspace e release passaram via `scripts/check.sh`.
- Suíte padrão: 11 testes passam; três testes dependentes de serviços/display ficam ignorados no comando padrão e foram executados separadamente.
- Mídia: quatro sessões H.264/AAC HTTP simultâneas, JPEG, frame BGRA real, descarte, reconexão após EOF e shutdown.
- `scripts/test-protocols.sh`: HTTP TS, JPEG, MJPEG, HLS, HEVC, RTSP TCP com autenticação; senha errada, porta fechada e TLS não confiável rejeitados. SOAP autenticado validou informações, capacidades, perfil/URI, movimento/Stop e presets.
- Descoberta: teste UDP local valida resposta correlacionada, deduplicação e recusa de endereço inválido/resposta não relacionada.
- GTK em Xvfb: cadastro e persistência, editor, foco por duplo clique, Escape, drop GTK, redimensionamento numérico e cancelamento; vídeo real e captura de janela inspecionada. A liberação do estado após fechar é testada.
- Secret Service: criação/leitura/exclusão de credencial fictícia em chaveiro isolado. Áudio: PCM real aceito por sink virtual silencioso, exclusividade e mute da câmera anterior; `scripts/test-native-services.sh` passou.
- AppImage local gerado com linuxdeploy/appimagetool/runtime oficiais; inicializou e encerrou por extração no display virtual, com banco temporário. Pacote final em dist/ e checksum em dist/SHA256SUMS, ambos ignorados pelo Git.

A validação de empacotamento revelou disputa ao sobrescrever um AppImage aberto; o script foi corrigido para publicar o artefato por renomeação atômica de arquivo temporário.

Logs desta sessão estão em /tmp/cctv-check.log, /tmp/cctv-protocols.log, /tmp/cctv-gtk.log, /tmp/cctv-native-services.log, /tmp/cctv-appimage-build.log e /tmp/cctv-appimage-smoke.log. São efêmeros; os comandos e fixtures para reproduzir estão no repositório. Resultados adicionais de auditoria de dependências são registrados em SECURITY.md quando disponíveis.

## Limitações e próxima ação

Consultar FEATURES.md: não há homologação de fabricante, hardware PTZ/VA-API, A/V prolongado, Wayland/HiDPI/leitor de tela, cargas de referência nem AppImage em outra máquina. Media2 exclusivo e HTTP Digest SOAP não estão implementados; erros são explícitos. Não declarar 1.0 completa.

Próxima ação: executar H01 na máquina/câmeras de referência, registrar matriz por modelo e métricas; validar o AppImage em VM Zorin 18 separada e revisar licenças/fontes correspondentes antes de distribuição pública. Todos os recursos centrais permanecem no escopo da primeira versão homologada. Somente commits locais são autorizados; não houve push nem PR.
