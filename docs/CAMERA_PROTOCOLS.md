# Protocolos de câmera

O cadastro aceita URLs rtsp://, http:// e https://. Endereço/porta são metadados de cadastro; a URL de stream é a fonte efetiva da conexão. Prefira informar a porta explicitamente na URL quando necessária. ONVIF não é um formato de vídeo: ele retorna perfis e URIs para o player.

RTSP usa TCP por padrão e permite UDP por câmera. HTTP/HTTPS é demultiplexado pelo libav; H.264, HEVC, MJPEG, JPEG e HLS dependem dos codecs compilados no FFmpeg. Substream é opcional e usado no mosaico; o foco usa a URL principal. A seleção manual de perfil ONVIF define o principal e sugere um perfil menor para substream.

Testado localmente: RTSP TCP com Basic Auth e H.264, HTTP TS H.264/AAC, HTTP HEVC, JPEG, multipart MJPEG e HLS. Falhas verificadas: senha RTSP incorreta, porta fechada e HTTPS com certificado não confiável. TLS bem-sucedido com CA privada, RTSP UDP e compatibilidade por fabricante permanecem na matriz de validação física.

Fabricantes podem exigir caminhos específicos, HTTP Digest, Media2, tokens de sessão ou relógio sincronizado. A implementação ONVIF atual usa Media1 e WS-Security UsernameToken digest. Recursos não reconhecidos geram mensagens de erro; não há promessa de compatibilidade universal.
