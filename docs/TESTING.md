# Testes e evidências

## Verificações reproduzíveis

```sh
scripts/check.sh
scripts/test-protocols.sh
xvfb-run -a dbus-run-session -- cargo test -p cctv-ui -- --ignored --test-threads=1
scripts/test-native-services.sh
```

Dependências adicionais: ffmpeg, openssl, python3, curl, Xvfb, dbus-run-session, gnome-keyring e pulseaudio. O script de serviços cria chaveiro e sink nulo descartáveis; nunca deve apontar para os alto-falantes/chaveiro real. PULSEAUDIO_BIN e PULSEAUDIO_MODULE_DIR permitem usar pacotes extraídos. Testes de protocolo escutam somente 127.0.0.1 nas portas 18080/18443/18554; precisam estar livres.

`cargo test --workspace`: modelos/URLs/layouts, sincronização de destino e endpoint ONVIF, reflow sem perda, CRUD/migração/exclusão/ordenação atômica, sessões compartilhadas/liberação, exclusividade lógica de áudio, quatro decodes H.264/AAC reais simultâneos por HTTP, JPEG, reconexão, descarte e encerramento; XML limitado, digest, SOAP movimento/Stop e descoberta UDP correlacionada. Os três testes de GUI/chaveiro/dispositivo de áudio são ignorados nesse comando e executados separadamente acima.

`test-protocols.sh`: gera H.264 e HEVC com lavfi e executa o simulador. Recebe frames reais por TS HTTP, JPEG, MJPEG, HLS, HEVC e RTSP TCP autenticado. Verifica falha com senha incorreta, porta fechada e certificado TLS autoassinado não confiável. O probe ONVIF valida informações, capacidades, perfil, URI, movimento/Stop e consulta/acionamento/criação de presets, com conferência dos eventos do servidor.

`native_crud_editor_and_focus`: abre GTK real e salva endpoint ONVIF sem consultar, verifica host/porta e invalidação ao trocar dispositivo; insere câmeras por lista/+ e drop; confirma reflow/undo, SQLite e cancelamento; exercita Enter, menu de foco, retorno e Escape; recebe frame real e verifica recolhimento/restauração da sidebar sem alterar preferências. Valida erro de sobreposição dentro do diálogo do painel. A consulta SOAP com dois perfis confirma reabilitação do seletor e troca da URL. Favoritar outra View preserva seleção; duplicação descarta foco antigo; ordenação e exclusão da inicial mantêm dados e seleção. Um servidor SOAP local recebe ContinuousMove seguido de Stop ao ativar o botão PTZ. Confirma liberação do estado no fechamento. `CCTV_UI_SCREENSHOTS=/tmp/cctv-ui` salva PNGs opcionais de monitor, cadastro e editor em tamanhos diferentes e do erro local. Foi executado no Xvfb com render Cairo. Não equivale a testes humanos de acessibilidade ou DnD.

`secret_service_roundtrip_and_delete`: grava credencial fictícia, lê, compara e apaga no Secret Service isolado. `real_pcm_output_and_exclusive_mute`: envia PCM ao servidor PulseAudio silencioso, verifica que a outra câmera não produz áudio e que a troca silencia a anterior.

As fixtures sample.ts e sample.jpg são geradas com fontes sintéticas lavfi, sem dados privados. Para regenerar: testsrc2 320×180, 10 fps por dois segundos, H.264 ultrafast/zerolatency/GOP 10 + seno AAC 48 kHz; JPEG é um frame testsrc2. FFmpeg é subprocesso somente no preparo de testes, nunca no player.

## Aceitação física pendente

Registrar modelo/firmware, URL sanitizada, transporte, codecs/perfis, driver, CPU/RSS/descartes e resultado para 4×1080p, 9×720p e 16 painéis. Avaliar recuperação de rede, drift A/V, VA-API, Stop ao perder foco, presets reais, Wayland, HiDPI e leitor de tela. AppImage deve ser executado em VM/computador separado compatível com Ubuntu 24.04. Tudo isso permanece NOT VERIFIED até haver evidência; não generalizar os resultados sintéticos de 320×180 para essas cargas.
