# Protocolos de câmera

O cadastro aceita URLs rtsp://, http:// e https://. Endereço/porta são derivados do cadastro por URL; a URL de stream é a fonte efetiva da conexão. Prefira informar a porta explicitamente na URL quando necessária. ONVIF não é um formato de vídeo: ele retorna perfis e URIs para o player.

RTSP usa TCP por padrão e permite UDP por câmera. HTTP/HTTPS é demultiplexado pelo libav; H.264, HEVC, MJPEG, JPEG e HLS dependem dos codecs compilados no FFmpeg. Substream é opcional e usado no mosaico; o foco usa a URL principal. A seleção manual de perfil ONVIF define o principal e sugere um perfil menor para substream.

Testado localmente: RTSP TCP com Basic Auth e H.264, HTTP TS H.264/AAC, HTTP HEVC, JPEG, multipart MJPEG e HLS. Falhas verificadas: senha RTSP incorreta, porta fechada e HTTPS com certificado não confiável. TLS bem-sucedido com CA privada, RTSP UDP e compatibilidade por fabricante permanecem na matriz de validação física.

Fabricantes podem exigir caminhos específicos, HTTP Digest, Media2, tokens de sessão ou relógio sincronizado. A implementação ONVIF atual usa Media1 e WS-Security UsernameToken digest. Recursos não reconhecidos geram mensagens de erro; não há promessa de compatibilidade universal.

## Seleção explícita de fonte (0.1.2)

ONVIF continua sendo um serviço de controle independente. O seletor persiste `stream_kind` e valida o esquema da URL principal e do substream. Colar uma URL RTSP ou HTTP(S) ajusta a seleção quando o esquema muda; escolher MJPEG/JPEG mantém esse formato ao colar HTTP(S).

| Tipo | URL e opções | Reprodução |
|---|---|---|
| RTSP | rtsp://; TCP/UDP, timeout, substream, áudio e aceleração | Entrada RTSP libav |
| HTTP(S) | http(s)://; timeout, substream, áudio e aceleração | Detecção automática de vídeo/HLS; TLS validado por padrão, configurável por câmera |
| MJPEG | http(s)://; multipart ou contínuo, timeout | Demuxer mpjpeg ou mjpeg; sem áudio/substream |
| JPEG | http(s)://; atualização de 200–60000 ms, timeout | Demuxer jpeg_pipe, uma imagem por requisição; intervalo após recebimento, sem áudio/substream |

JPEG mantém a última imagem enquanto aguarda o intervalo, libera a conexão após cada imagem e cancela espera/I/O ao encerrar. Falhas usam backoff; após sucesso, retorna ao intervalo configurado. A taxa efetiva inclui tempo de rede e decodificação. A configuração participa da chave da sessão compartilhada. Cadastros anteriores usam Auto com detecção existente até serem editados; `snapshot_interval_ms` tem padrão 1000 e `mjpeg_multipart` tem padrão true.

Testes nativos recebem múltiplos frames reais de multipart/contínuo e confirmam requisições JPEG espaçadas, estado Playing entre atualizações e ausência de novas requisições após shutdown. Não constituem homologação de fabricante.

## Acesso público e certificado (0.1.4)

**Conexão → Sem credenciais** oculta os campos de usuário/senha e usa acesso anônimo, incluindo teste de vídeo, ONVIF e PTZ. Nenhum segredo salvo é enviado ou lido enquanto esse modo está ativo. Os dados anteriores permanecem no chaveiro para reativação; Remover credenciais do chaveiro ao salvar continua sendo uma ação explícita separada. Streams públicas podem usar RTSP, HTTP(S), MJPEG ou JPEG.

**Avançado → Segurança HTTPS → Validar certificado HTTPS** começa marcado. Desmarcar aceita certificados autoassinados/inválidos para aquela câmera tanto no player quanto nos clientes ONVIF/PTZ. A flag integra a chave das sessões, assim como o modo sem credenciais; configurações diferentes não compartilham sessão. Testes no simulador confirmam recusa padrão, aceitação explícita em TS/HLS/JPEG e SOAP e nova recusa em uma configuração padrão.
