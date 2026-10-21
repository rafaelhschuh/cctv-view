# Pipeline libav

`crates/media/src/decoder.c` integra libavformat, libavcodec, libavutil, libswscale, libswresample e libpulse-simple. Não executa FFmpeg por câmera. cc/pkg-config compila o shim contra headers e ABI nativos.

A sessão abre a entrada com transporte/timeout, interrupt_callback e TLS, encontra vídeo/áudio e alimenta decodificadores separados por tipo de pacote. Os recursos são propriedade exclusiva do worker e liberados no único caminho de limpeza. O callback de vídeo copia BGRA para Rust antes que libav reutilize seus buffers.

VA-API é tentado se o codec anunciar suporte e o dispositivo puder ser criado; frames de hardware são transferidos à memória de sistema. Na falha, uma nova tentativa usa software. Não há suporte NVDEC/VDPAU explícito. Cada decoder usa duas threads. Resolução máxima de entrada é 8192 por eixo e saída de vídeo é limitada a 1920 de largura. swscale converte para BGRA; GdkMemoryTexture é criado somente na thread GTK.

A sessão mantém somente o frame mais recente, em Arc sob Mutex. Consumidores GTK consultam a cada 33 ms. Frames substituídos antes de consumo incrementam o contador de descarte. Múltiplos painéis compatíveis compartilham uma conexão. Ainda há cópias para textura; isso pode consumir CPU e banda de memória em mosaicos grandes.

Timestamps de vídeo regulam a reprodução de entradas HTTP/HLS para evitar consumo acelerado. Saltos maiores que dois segundos reiniciam o relógio local. Áudio é convertido a S16LE estéreo 48 kHz e enviado ao PulseAudio somente para a câmera selecionada. Buffers de áudio são limitados, com flush no mute. A sincronização é básica por ordem de demux/pacing; sincronização A/V rigorosa e compensação adaptativa de drift estão pendentes.

Reconexão usa backoff limitado de 2 a 16 segundos. Callback de interrupção limita I/O e observa cancelamento/Weak sem consumidores. Status e métricas são consultáveis sem GTK no worker. Diagnóstico exibe decoder efetivamente usado, frames, descartes, idade local do frame e PCM aceito pelo servidor de áudio; CPU e RSS vêm de /proc. Idade local não mede a latência desde o sensor.

O contrato Rust/C inclui input_kind: 0 detecta a entrada, 1 força mpjpeg, 2 força mjpeg e 3 força jpeg_pipe. No modo JPEG, o shim retorna sucesso após copiar a primeira imagem; o worker agenda a próxima requisição pelo intervalo configurado, mantendo o último frame e observando cancelamento em intervalos de 25 ms. JPEG não tenta VA-API. Falhas mantêm backoff com reset após sucesso. Os formatos explícitos e a atualização periódica são cobertos por fixtures HTTP locais.

O contrato Rust/C também recebe tls_verify, aplicado à opção TLS do libav; o CA file só é configurado no modo validado. O modo no_credentials impede composição de usuário/senha na URL mesmo se o chamador fornecer Credentials. As flags fazem parte da chave da sessão compartilhada. A UI pula Vault::get no modo anônimo antes de adquirir a sessão.
