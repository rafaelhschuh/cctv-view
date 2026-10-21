# Segurança

Não há telemetria, serviços locais de produção, privilégios de root nem shell para reproduzir streams. O simulador é ferramenta de desenvolvimento e escuta apenas loopback.

Segredos são guardados via libsecret/Secret Service; falha de chaveiro é exibida, sem fallback no banco. O tipo Credentials não implementa Debug/Serialize. As URLs cadastradas rejeitam userinfo; o player compõe a autenticação somente na memória do worker. URLs e SOAP não são registrados. Parâmetros de query podem conter tokens de fabricante e devem ser tratados como sensíveis pelo operador.

HTTPS é validado em reqwest/rustls e libav. libav usa as autoridades em `/etc/ssl/certs/ca-certificates.crt`. Não há opção de ignorar TLS nesta versão. Certificados privados devem ser instalados no trust store do sistema. Redirecionamentos SOAP são desativados e endpoints anunciados precisam usar HTTP(S), sem userinfo e no mesmo host cadastrado.

SOAP tem limite de 2 MiB, profundidade 64, 20 mil nós e 256 atributos por elemento; DTD e entidades não predefinidas são recusadas. Descoberta deduplica endpoints, limita quantidade e duração e correlaciona RelatesTo. Perfis e URIs externas são validados. libav permite somente protocolos de streaming explícitos; não habilita file nem acesso arbitrário a arquivos por playlists.

PTZ depende de autorização do usuário e perfil com configuração PTZ. Movimentos contínuos pedem timeout de 500 ms, renovado enquanto pressionado. Soltar, cancelar gesto, perder foco, fechar ou falhar solicita Stop. Uma câmera defeituosa que ignore timeout/Stop não pode receber garantia de parada por software cliente.

Limitações: credenciais permanecem em memória enquanto em uso; não existe garantia de zeroização de todas as cópias nativas. RTSP/HTTP sem TLS não protege conteúdo na rede. O código usa bibliotecas de mídia nativas e deve acompanhar correções de segurança. AppImage inclui versões do ambiente de build; reconstruir após atualizações. Não há auditoria de segurança externa nem certificação.

## Auditoria de dependências desta entrega

Em 2026-10-21, cargo-audit 0.22.2 encontrou RUSTSEC-2026-0194 e RUSTSEC-2026-0195 na versão inicial de quick-xml. A dependência foi atualizada para 0.42.0, o parser foi adaptado e ganhou limite de 256 atributos por elemento, raiz única e checagem da operação SOAP de resposta. A nova auditoria do Cargo.lock (255 dependências) retornou **zero vulnerabilidades e zero avisos**, usando a base RustSec obtida na sessão. As regressões SOAP/XML e de integração foram repetidas.

Esse resultado cobre dependências Rust conhecidas pela base na data; não audita binários GTK/FFmpeg do sistema nem demonstra ausência de vulnerabilidades desconhecidas. CI repete cargo audit nas execuções futuras.
