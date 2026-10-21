# Testes e evidências

## Verificações reproduzíveis

```sh
scripts/check.sh
scripts/test-protocols.sh
xvfb-run -a scripts/test-ui.sh
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

## Tipos de conexão e acabamento 0.1.2

A suíte padrão inclui validação de esquema/intervalo, roundtrip dos defaults de câmera legada e `explicit_mjpeg_formats_and_snapshot_refresh`. Este último recebe frames reais dos demuxers multipart e MJPEG contínuo e verifica que JPEG faz três requisições no intervalo, sem backoff no sucesso, permanecendo Playing e cancelando novas requisições após shutdown.

O teste GTK mede a distância entre área de abas e Salvar com feedback vazio, confirma que o rodapé reaparece no erro de timeout, alterna as opções MJPEG/JPEG, salva tipo/intervalo e verifica linhas de Views com mesma altura (badges e nomes longos). Captura opcional inclui conexão, opções de cada formato, erro de rodapé e lista alinhada. A captura aguarda um render node de WidgetPaintable antes de exportar, evitando uma corrida com o frame GTK. As capturas não equivalem a validação de leitor de tela ou hardware físico.

## Lista nativa 0.1.3

O teste GTK cobre uma View sem indicadores e várias com favorito/nome longo, verifica alturas uniformes de até 48 px, seleção da View atual, ativação da segunda linha e preservação do foco nessa linha. `CCTV_TEST_DARK=1` ativa a preferência GTK de tema escuro para repetir o teste e as capturas. A captura 14-single-view reproduz o caso da imagem reportada. Não substitui teste humano de leitor de tela/Wayland/HiDPI.

## Credenciais e certificado 0.1.4

A regressão GTK marca Sem credenciais com usuário preenchido, verifica os campos desabilitados, reativa/desativa a autenticação, desmarca validação TLS e confere flags no SQLite. Salva e reproduz uma câmera pública com referência fictícia de segredo sem acessar Secret Service; PTZ também abre/aciona o mock com referência fictícia ignorada. O roundtrip legado confirma no_credentials=false e tls_verify=true quando campos estão ausentes.

O simulador de protocolos testa certificado autoassinado recusado por padrão, vídeo TS/HLS/JPEG aceito com --no-verify-tls e nova recusa sem a flag; ONVIF HTTPS também é recusado por padrão e aceito explicitamente. A probe pública usa --no-credentials com Credentials não vazio fornecido pelo chamador. Esses flags pertencem somente aos executáveis de teste; no aplicativo, as opções são armazenadas por câmera. Capturas 02-camera-connection e 15-tls-options registram os controles no GTK real.

`anonymous_mode_never_answers_auth_challenge` recebe Credentials não vazio e verifica que um desafio HTTP 401/Basic não recebe Authorization no modo anônimo. Como controle, as mesmas credenciais autenticam e recebem um frame ao desativar o modo. Suíte padrão atual: 18 testes.

## Dashboard, apresentação, CLI e senha 0.2.0

Core verifica escala proporcional 2×2 → 12×12, preservação de IDs/proporção, preset principal 8×8 + menores e rejeição de perda/colisão. Storage verifica ausência de plaintext, salt diferente na mesma senha, senha errada, alteração e remoção com atual. CLI tem parser e integração de processo sem DISPLAY/WAYLAND_DISPLAY: help/listagem, bloqueio, senha via pipe e recusa da incorreta.

GTK: duplo clique entra/sai de foco, X flutuante, apresentação oculta sidebar/header/barras; captura real com textura e clipping. No editor, gesto de desenho cria painel 4×3, alça aumenta para 5×4, undo restaura, colisão é recusada e cancelamento preserva SQLite. Configuração de senha, bloqueio com streams parados e tentativa incorreta/correta são exercitados; captura da tela bloqueada. Testes injetam sinais GTK reais, sem comprovar arrasto humano/Wayland/HiDPI ou homologação de hardware.

Logs efêmeros desta etapa usam /tmp/cctv-dashboard-*. A versão final repete scripts/check.sh, GTK escuro, smoke de CLI/GUI no AppImage e checksum. A suíte padrão totaliza 22 testes; GUI executada separadamente, serviços de áudio/chaveiro mantêm a evidência anterior.

## Biblioteca e preferências 0.3.0
Regressões de storage cobrem pastas, renomeação/exclusão preservando Views e segredos originais; preparação de clone sem chaveiro; JSON sem referências/perfis/query/fragmento; importação aditiva com novos IDs; mesclagem de pastas por caixa; lote inválido sem gravações; userinfo/campos extras/versão/tamanho inválidos e defaults legados. Teste de exportação verifica modo 0600 e recusa de sobrescrita.

A regressão GTK existente foi ampliada: criar pasta, mover, clonar por rascunho, busca sem resultados, renomear com filtro preservado, três abas de preferências, revisão editável de exportação, arquivo real em worker, leitura/prévia, cancelar antes de gravar e rejeitar cancelamento durante importação. Usa banco e credenciais fictícias em diretório temporário. Diálogo do seletor nativo de arquivos por interação física/portal, Wayland, Orca e HiDPI: NOT VERIFIED.

Verificação final: scripts/check.sh com 26 testes padrão, fmt/Clippy/release; Clippy repetido após ajuste dos rodapés; regressão GTK em ambos os temas, com capturas inspecionadas. AppImage 0.3.0 validou --version, --list-cameras sem display e abertura/encerramento com biblioteca fictícia em /tmp/cctv-library-package, por extração/Xvfb; SHA256SUMS conferido. Auditoria RustSec com base local: zero vulnerabilidades/avisos. Logs em PROGRESS.md.

## Credenciais e criptografia 0.4.0
Storage testa ciphertext sem nomes/URLs/usuário/senha legíveis, salt/nonce diferentes a cada exportação, igualdade completa após descriptografar, senha errada, ciphertext adulterado, custo KDF/formato/tamanho inválido e senha vazia. Importação com backend de segredos controlado verifica senha restaurada sob nova referência, ausência de plaintext no SQLite e compensação de segredos quando o segundo registro falha.

A regressão GTK agora requer **scripts/test-ui.sh**, que inicia D-Bus/Secret Service privados com XDG temporário e encerra/apaga tudo ao terminar. Não execute o teste diretamente contra o chaveiro do usuário. O display deve ser Xvfb. O fluxo verifica clone com senha real fictícia, exportação criptografada/arquivo, senha de arquivo incorreta/correta, importação, senha do app incorreta/correta/cancelada, senha salva visível no campo após autenticação e novo prompt a cada exportação.

Verificação final 0.4.0: 27 testes padrão, fmt/Clippy/release, regressão GTK em claro/escuro com chaveiro privado, auditoria RustSec local sem achados e AppImage com --version/--list-cameras headless, GUI isolada e checksum. Teste adicional de clonagem altera a câmera por uma segunda conexão durante a leitura de credenciais e confirma que a cópia mantém o mesmo conjunto de metadados original. O teste de autorização cobre dados de uma lista desatualizada e verifica nova confirmação após cada ação. Logs/capturas em PROGRESS.md e UI_REVIEW.md.
