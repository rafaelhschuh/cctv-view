# Segurança

Não há telemetria, serviços locais de produção, privilégios de root nem shell para reproduzir streams. O simulador é ferramenta de desenvolvimento e escuta apenas loopback.

Segredos são guardados via libsecret/Secret Service; falha de chaveiro é exibida, sem fallback no banco. O tipo Credentials não implementa Debug/Serialize. No modo **Sem credenciais**, reprodução, consulta e PTZ ignoram usuário/senha e não leem o chaveiro. A referência ao segredo é preservada para reativação; a opção separada de remoção apaga o segredo ao salvar.

As URLs cadastradas rejeitam userinfo; o player compõe a autenticação somente na memória do worker. URLs e SOAP não são registrados. Parâmetros de query podem conter tokens de fabricante e devem ser tratados como sensíveis pelo operador.

HTTPS é validado por padrão em reqwest/rustls e libav; cadastros antigos mantêm esse padrão. libav usa as autoridades em `/etc/ssl/certs/ca-certificates.crt`. A opção por câmera **Avançado → Segurança HTTPS → Validar certificado HTTPS** pode ser desmarcada explicitamente para aceitar certificado autoassinado/inválido no vídeo, teste de conexão, ONVIF e PTZ. A exceção dispensa a verificação da identidade do servidor somente nessa configuração, sem alterar o trust store ou o padrão de outras câmeras. Redirecionamentos SOAP são desativados e endpoints anunciados precisam usar HTTP(S), sem userinfo e no mesmo host cadastrado.

SOAP tem limite de 2 MiB, profundidade 64, 20 mil nós e 256 atributos por elemento; DTD e entidades não predefinidas são recusadas. Descoberta deduplica endpoints, limita quantidade e duração e correlaciona RelatesTo. Perfis e URIs externas são validados. libav permite somente protocolos de streaming explícitos; não habilita file nem acesso arbitrário a arquivos por playlists.

PTZ depende de autorização do usuário e perfil com configuração PTZ. Movimentos contínuos pedem timeout de 500 ms, renovado enquanto pressionado. Soltar, cancelar gesto, perder foco, fechar ou falhar solicita Stop. Uma câmera defeituosa que ignore timeout/Stop não pode receber garantia de parada por software cliente.

Limitações: credenciais permanecem em memória enquanto em uso; não existe garantia de zeroização de todas as cópias nativas. RTSP/HTTP sem TLS não protege conteúdo na rede. O código usa bibliotecas de mídia nativas e deve acompanhar correções de segurança. AppImage inclui versões do ambiente de build; reconstruir após atualizações. Não há auditoria de segurança externa nem certificação.

## Auditoria de dependências desta entrega

Em 2026-10-21, cargo-audit 0.22.2 encontrou RUSTSEC-2026-0194 e RUSTSEC-2026-0195 na versão inicial de quick-xml. A dependência foi atualizada para 0.42.0, o parser foi adaptado e ganhou limite de 256 atributos por elemento, raiz única e checagem da operação SOAP de resposta. A nova auditoria do Cargo.lock (255 dependências) retornou **zero vulnerabilidades e zero avisos**, usando a base RustSec obtida na sessão. As regressões SOAP/XML e de integração foram repetidas.

Esse resultado cobre dependências Rust conhecidas pela base na data; não audita binários GTK/FFmpeg do sistema nem demonstra ausência de vulnerabilidades desconhecidas. CI repete cargo audit nas execuções futuras.

## Bloqueio do aplicativo (0.2.0)

A senha de abertura é separada das credenciais de câmera: estas continuam em Secret Service, enquanto settings.app_lock guarda somente salt/hash versionados. Usa PBKDF2-HMAC-SHA256 com 600.000 iterações, salt criptográfico aleatório de 128 bits e saída de 256 bits; derivação/verificação pela biblioteca ring já presente no lockfile. Fontes: [ring PBKDF2](https://docs.rs/ring/0.17.14/ring/pbkdf2/index.html), [OWASP Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html).

Senha nova requer ao menos 8 caracteres e no máximo 1024 bytes; alteração/remoção exige a atual. KDF e SQLite ficam em workers na GUI. Tentativa errada tem atraso adicional de 750 ms. Bloquear encerra sessões e fecha diálogos; renderização e atalhos não contornam a tela. Startup espera desbloqueio antes de criar sessões. Listagens CLI exigem senha quando a biblioteca está protegida; --unlock-stdin recebe por pipe, nunca por argumentos. A CLI não exporta URLs/referências de segredo.

Esse bloqueio protege a interface do programa, não criptografa banco/frames/credenciais nem impede que o dono da conta Linux modifique os arquivos ou execute outro software. Não substitui bloqueio de sessão/permissões do sistema. Não existe recuperação de senha pela aplicação. Nenhuma senha/hash é escrita em logs.
