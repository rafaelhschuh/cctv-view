# Arquitetura

`app → ui → core/storage/media/onvif`. Core não depende de GTK, SQLite ou rede. Os serviços não dependem de UI; não há ciclos.

A aplicação registra `br.dev.schuh.cctv-view` em GApplication. A thread GTK contém somente widgets e estado de apresentação em Rc/RefCell. Um runtime Tokio executa SOAP e descoberta. `ui::jobs` entrega resultados de workers por canais de capacidade um, consumidos em timers GLib. SQLite e chaveiro são acessados em workers. Preferências usam uma fila FIFO e barreira de encerramento.

Cada sessão libav possui um worker dedicado. SessionManager guarda Weak e compartilha a mesma sessão para identidade/URL/transporte/timeout/hardware compatíveis. Um consumidor possui Arc; ao sair da View, os timers com widgets Weak liberam as sessões. O callback de interrupção observa cancelamento ou perda de todos os consumidores. Editar câmera invalida sua sessão anterior.

O frame corrente tem uma única posição substituível por sessão. O GTK cria GdkMemoryTexture BGRA e faz snapshot com recorte e transformação. Isso mantém o backlog limitado, mas ainda envolve cópias CPU; não é zero-copy GPU.

ONVIF é controle/descoberta; a URI obtida alimenta libav. PTZ usa comandos serializados por janela, geração de movimentos e timeout no dispositivo. Não há servidor de produção, telemetria nem processo FFmpeg por câmera.

O shim C é restrito a APIs nativas libav/PulseAudio e libsecret, compilado por cc/pkg-config. Rust mantém modelos, sessões, cancelamento, segurança de fluxos e interface. Ver ADR 0001 para justificativa e limitações.

A CLI em app é analisada antes de inicializar GTK. Listagens passam pelo bloqueio da biblioteca e retornam somente metadados mínimos; abertura configura LaunchOptions. Instâncias com View/tela cheia/biblioteca explícita são independentes. O gate GTK antecede build/playback; bloqueio de sessão encerra mídia, fecha diálogos e substitui a raiz por autenticação. Ver ADR 0002.
