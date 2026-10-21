# ONVIF

O serviço usa WS-Discovery 2005/04 por UDP multicast 239.255.255.250:3702, TTL 1, janela de quatro segundos na interface. O socket é temporário e os endpoints são deduplicados/correlacionados pelo MessageID. Equipamentos em outra VLAN ou interfaces sem rota multicast podem exigir cadastro manual.

O endpoint manual costuma ser `http://HOST/onvif/device_service`. Client executa GetDeviceInformation, GetCapabilities, GetProfiles e GetStreamUri (RTP-Unicast/RTSP), preservando fabricante, modelo, serviços, tokens e resoluções. A UI permite selecionar perfil e sugerir substream menor. Só aceita URIs sem credenciais embutidas; equipamentos que retornem userinfo exigem ajuste do endpoint/firmware ou uma evolução explícita do fluxo seguro.

Autenticação usa WS-Security UsernameToken PasswordDigest: Base64(SHA1(nonce || Created || senha)), nonce UUID aleatório por chamada e data UTC. Relógio incorreto pode causar falha de autenticação. SOAP 1.2 tem timeout, limite de tamanho e parser com nomes locais para tolerar diferentes prefixos de namespace.

Media1 está implementado. Media2 exclusivo, HTTP Digest da camada HTTP e correção automática de desvio de relógio não estão implementados. Serviços anunciados em host diferente são recusados; cadastrar o host explicitamente é a política atual. Não é uma implementação completa de toda a especificação ONVIF.

Evidências: servidor SOAP local validou digest, informações, capacidades, perfis, URI e operações PTZ/presets; teste UDP valida correlação e deduplicação. Nenhum fabricante/câmera física foi homologado. Fontes: https://www.onvif.org/profiles/specifications/ e https://www.onvif.org/ver20/ptz/wsdl/.

Em 0.1.4, Client::new mantém TLS verificado e Client::new_with_tls aceita a configuração explícita da câmera. A consulta e o PTZ usam essa configuração. Sem credenciais fornece Credentials vazio (sem UsernameToken) e não acessa o chaveiro; restrições de host, esquema, redirecionamento e tamanho continuam aplicadas.
