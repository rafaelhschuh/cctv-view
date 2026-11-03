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
- Suíte padrão: 22 testes passam; três testes dependentes de serviços/display ficam ignorados no comando padrão e foram executados separadamente.
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

## Revisão UI/UX 0.1.1

Os 12 achados iniciais e três regressões apontadas na segunda revisão crítica foram corrigidos. Cadastro em abas e feedback fixo; endpoint ONVIF independente da consulta, invalidando dispositivo antigo; destino derivado da URL; editor com inserção por clique/teclado, reflow sem perda, undo e erro local; menus por View, indicadores e sidebar adaptativa; foco por Enter/menu e retorno visível; rótulos acessíveis e PTZ com ativação limitada. Favoritar outra View preserva monitoramento; duplicar limpa foco; referências de foco inválidas são descartadas.

A regressão GTK ampliada passou com consulta SOAP de dois perfis, persistência, editor, janelas pequenas/grandes, favoritos/ordem/exclusão, movimento seguido de Stop e liberação do estado. Capturas e rastreabilidade em UI_REVIEW.md. Checks padrão passaram e o AppImage foi atualizado para 0.1.1. O pacote final iniciou/encerrou por extração no Xvfb, sem Gtk-WARNING/CRITICAL/panic; SHA256SUMS conferido. A revisão final somente leitura confirmou os três ajustes adicionais sem novos achados nesses pontos. Os testes de Secret Service e sink PulseAudio da entrega anterior continuam registrados; não foram repetidos porque os adaptadores não mudaram. Logs adicionais efêmeros: /tmp/cctv-ux-check.log, /tmp/cctv-ux-tests.log, /tmp/cctv-ux-gtk.log, /tmp/cctv-ux-appimage-build.log e /tmp/cctv-ux-appimage-smoke.log. As limitações de homologação física permanecem.

## Conexões e acabamento 0.1.2

Rodapé do cadastro reduzido ao conteúdo: feedback vazio fica oculto e erros/resultados reaparecem. A mensagem permanente Pronto foi removida. Views têm linhas/subtítulos de altura consistente, menus centralizados e nomes longos contidos, usando o fundo do tema nativo.

O seletor RTSP/HTTP(S)/MJPEG/JPEG persiste o tipo separado de ONVIF e controla os grupos de opções avançadas. RTSP expõe TCP/UDP; HTTP(S) mantém autodetecção e TLS validado; MJPEG escolhe multipart ou contínuo; JPEG configura atualização de 200–60000 ms. O shim libav força os demuxers de imagem, finaliza cada snapshot e o worker agenda novas requisições sem backoff no sucesso, preservando o frame e cancelando a espera ao encerrar. Novos campos serde têm defaults compatíveis; roundtrip de JSON legado testado, sem migração SQL destrutiva.

Passaram os 17 testes padrão e a regressão GTK ampliada. A captura agora aguarda um frame GTK válido, corrigindo uma corrida ocasional de WidgetPaintable. Capturas de conexão, opções JPEG, rodapé de erro e lista com badges/nomes longos foram inspecionadas; veja UI_REVIEW.md. Logs efêmeros desta etapa: /tmp/cctv-types-tests.log, /tmp/cctv-types-gtk.log, /tmp/cctv-types-check.log, /tmp/cctv-types-protocols.log, /tmp/cctv-types-appimage-build.log e /tmp/cctv-types-appimage-smoke.log. Os testes de chaveiro/PulseAudio isolados da primeira entrega não foram repetidos nesta etapa; homologação física permanece em H01/H02.

Validação final 0.1.2: scripts/check.sh (fmt/Clippy/testes/release) e scripts/test-protocols.sh passaram. AppImage 0.1.2 gerado, inicializado/encerrado por extração no Xvfb com banco temporário, sem Gtk-WARNING/CRITICAL/panic; SHA256SUMS conferido. Somente commits locais, sem push/PR. Próxima ação de homologação permanece H01/H02.

## Correção da lista 0.1.3

A reserva de subtítulo e a altura de 60 px introduzidas em 0.1.2 pioraram a lista de uma única View, conforme captura do usuário. Removidas. ListBoxRow nativa passa a reunir nome, indicadores opcionais e menu sem moldura na mesma linha, com seleção nativa e foco preservado ao ativar. Sem CSS adicional ou mudança em backend. Capturas claro/escuro inspecionadas em UI_REVIEW.md. Teste cobre altura compacta (até 48 px), nome longo, indicadores, seleção e ativação com foco.

Logs desta etapa: /tmp/cctv-list-check.log, /tmp/cctv-list-light.log, /tmp/cctv-list-dark.log, /tmp/cctv-list-appimage-build.log e /tmp/cctv-list-appimage-smoke.log. Próxima homologação permanece H01/H02; somente commits locais.

Validação 0.1.3: scripts/check.sh passou (17 testes padrão, fmt, Clippy e release); regressão GTK repetida em claro e escuro passou, incluindo foco preservado. AppImage gerado e smoke por extração/Xvfb com dados temporários passou; checksum conferido.

## Certificado e streams públicas 0.1.4

Sem credenciais na aba Conexão oculta/desabilita os campos e pula o chaveiro na reprodução, consulta ONVIF e PTZ; o player também ignora Credentials recebidos do chamador. A referência ao segredo é preservada sem leitura e pode ser removida pela ação explícita existente. Reativar autenticação carrega o segredo em worker. Validar certificado HTTPS na aba Avançado fica marcado por padrão, mas pode ser desmarcado por câmera e vale para vídeo, teste de conexão, ONVIF e PTZ. Campos serde preservam defaults seguros em JSON legado; as flags integram a chave das sessões.

Passaram fmt/Clippy, release, 18 testes padrão e regressão GTK. O teste nativo de desafio HTTP confirma ausência de Authorization no modo anônimo e sucesso ao reativar autenticação. O simulador confirma HTTPS autoassinado recusado por padrão e aceito com exceção explícita em TS/HLS/JPEG e SOAP, seguido de nova recusa padrão. GTK confirma persistência e ignora referências fictícias de chaveiro no modo público. Capturas inspecionadas em UI_REVIEW.md.

AppImage 0.1.4 gerado e smoke por extração/Xvfb com banco temporário passou; checksum conferido. Logs efêmeros: /tmp/cctv-access-check.log, /tmp/cctv-access-tests.log, /tmp/cctv-access-clippy.log, /tmp/cctv-access-gtk.log, /tmp/cctv-access-protocols.log, /tmp/cctv-access-appimage-build.log e /tmp/cctv-access-appimage-smoke.log. Os testes de chaveiro/sink reais isolados da primeira entrega não foram repetidos; homologação física continua H01/H02. Somente commits locais, sem push/PR.

## Dashboard, apresentação, CLI e senha 0.2.0

Duplo clique alterna foco e X flutuante substitui a faixa extra. Apresentação oculta cabeçalho/lateral/barras e remove margem exterior vazia dos painéis. Snapshot aplica clipping arredondado antes e depois do primeiro frame. Editor 12×12 adiciona regiões por desenho, tamanho inicial, presets proporcionais, DnD, resize e contraste; undo/cancelamento e colisão preservam os dados. Conversão de View antiga ocorre só no rascunho; erros mantêm a grade original.

Senha do aplicativo foi confirmada pelo usuário como bloqueio de acesso: hash/salt PBKDF2 em SQLite, configuração/alteração/remoção com senha atual, gate inicial e Bloquear agora com sessões encerradas. Verificação em worker e guarda contra callback tardio de janela fechada. CLI analisa antes do GTK, lista metadados mínimos sem display e abre biblioteca/View/fullscreen; listagens protegidas, senha por stdin e erro de senha antes de inicializar GUI. ADR 0002 registra decisão e limite de proteção da interface sem criptografia de arquivos.

Passaram scripts/check.sh (fmt, Clippy, 22 testes padrão e release), regressão GTK escura com desenho 4×3/resize 5×4/undo/colisão, foco/X/fullscreen e senha errada/correta. Testes de storage confirmam salts diferentes, ausência de plaintext e alteração/remoção; integração CLI confirma headless e autenticação. AppImage 0.2.0 foi gerado e validou help/version/listagem sem display, recusa sem senha, desbloqueio por pipe, recusa de senha errada e abertura GUI da View Script Inicial em fullscreen com encerramento automático. Checksum conferido; sem Gtk-WARNING/CRITICAL/panic. Auditoria cargo-audit da base local: zero vulnerabilidades/avisos.

Capturas persistentes em UI_REVIEW.md. Logs efêmeros: /tmp/cctv-dashboard-check.log, /tmp/cctv-dashboard-tests.log, /tmp/cctv-dashboard-gtk.log, /tmp/cctv-dashboard-appimage-build.log, /tmp/cctv-dashboard-appimage-smoke.log e /tmp/cctv-dashboard-audit.json. O smoke de CLI usou somente biblioteca e senha fictícias em /tmp/cctv-dashboard-cli-package; não alterou a biblioteca do usuário. Os testes de chaveiro/áudio e protocolos da entrega anterior mantêm a evidência; adaptadores não mudaram. Próxima homologação permanece H01/H02. Somente commits locais, sem push/PR.

## Biblioteca e preferências 0.3.0
Implementação: Camera.folder opcional e settings.camera_folders compatíveis com bancos anteriores. Biblioteca com busca, filtro, contador, linhas nativas alinhadas e ações de pasta/câmera. Clonar prepara um novo cadastro sem segredo/perfis/PTZ e abre rascunho; somente Salvar persiste. Preferências em Geral/Biblioteca/Segurança. DTO de transferência explícito, sem Vault nem senha do aplicativo, URLs sem query/fragmento, revisão editável de caminhos, limites e prévia; importação aditiva em transação, mesclagem de pastas sem duplicação por caixa e arquivo de exportação 0600 sem sobrescrita.

Revisão crítica de Einstein encontrou e orientou correções de seleção ao renomear, transação de movimento, cancelamento durante gravação e revisão de tokens em caminhos de URL. Validação final: scripts/check.sh passou (fmt, Clippy sem warnings, 26 testes padrão e release); Clippy repetido após acabamento dos diálogos. Regressão GTK real em claro e escuro passou com nomes longos, tipos mistos, linhas de mesma altura, cancelamento de clone/prévia, pasta renomeada e revisão de exportação. Sem Gtk-WARNING/CRITICAL/panic nas execuções finais. AppImage 0.3.0 gerado, versão/listagem sem display e abertura/encerramento GTK com banco temporário passaram; checksum conferido. O display de teste antigo tinha sido encerrado antes do primeiro smoke; a repetição iniciou/encerrou seu próprio Xvfb. Auditoria da base RustSec local: zero vulnerabilidades e avisos.

Capturas em docs/screenshots e UI_REVIEW.md. Logs efêmeros: /tmp/cctv-library-check.log, /tmp/cctv-library-clippy.log, /tmp/cctv-library-gtk.log, /tmp/cctv-library-light-gtk.log, /tmp/cctv-library-appimage-build.log, /tmp/cctv-library-appimage-smoke.log e /tmp/cctv-library-audit.json. Não houve acesso à biblioteca real ou ao chaveiro para clonar/exportar/importar. Testes físicos de câmera/Secret Service/áudio não foram repetidos nesta entrega; adaptadores não mudaram. Somente commits locais, sem push/PR. Próxima homologação permanece H01/H02; seletor de arquivo por interação física/portal, Wayland, HiDPI e leitor de tela continuam NOT VERIFIED.

## Credenciais e transferência criptografada 0.4.0
Usuário corrigiu o escopo: deseja manter credenciais e URLs ao editar/clonar/exportar, com apenas confirmação da senha do app quando configurada e senha própria para criptografar o arquivo. A revisão de URLs e a sanitização/exclusão de dados de 0.3.0 foram substituídas. Implementados clone completo com segredo independente, reautenticação por ação, campo de edição com senha salva, envelope JSON AES-256-GCM/PBKDF2 e importação com senha e tag autenticada. Nenhuma senha é persistida em SQLite/logs; arquivos só recebem ciphertext. Validação final: scripts/check.sh passou (fmt, Clippy, 27 testes padrão e release), com Clippy/release repetidos após proteção de cadastros em listas desatualizadas. GTK escuro e claro com Secret Service descartável passou: clonar preserva a senha em referência nova; senha errada de arquivo não importa; senha correta restaura; reautenticação errada/cancelada não abre dados, e toda edição/clonagem/exportação protegida pede de novo. Capturas inspecionadas em UI_REVIEW.md. AppImage 0.4.0 validou versão/listagem sem display, abertura/encerramento GTK com banco temporário e checksum; nenhum Gtk-WARNING/CRITICAL/panic nos testes finais. Auditoria RustSec local: zero vulnerabilidades/avisos.

Revisão crítica de Cicero encontrou uma segunda leitura de metadados na clonagem que poderia combinar URL nova com credenciais antigas; clone passou a usar uma única leitura, com regressão que altera o cadastro por outra conexão durante o acesso ao chaveiro. A regressão GTK identificou que fechar o diálogo descartava a ação autorizada antes de executá-la; o callback agora é retirado antes do fechamento. Cadastros preenchidos de uma lista antiga também reautenticam, mesmo se seu ID não estiver mais no estado atual.

Logs efêmeros: /tmp/cctv-crypto-check.log, /tmp/cctv-crypto-clippy.log, /tmp/cctv-crypto-gtk.log, /tmp/cctv-crypto-light-gtk.log, /tmp/cctv-crypto-appimage-build.log, /tmp/cctv-crypto-appimage-smoke.log e /tmp/cctv-crypto-audit.json. GUI usa scripts/test-ui.sh em D-Bus/XDG/chaveiro temporários, sem ler ou alterar biblioteca/credenciais reais. Reprodução de áudio, hardware, Wayland, HiDPI e leitor de tela permanecem em H01/H02; seletor de arquivo por interação física/portal não foi homologado. Somente commits locais, sem push/PR. Próxima ação externa ao pedido: homologação H01/H02.

## Workflow AppImage e GPLv3 — 2026-10-23
Usuário solicitou geração no GitHub e GPL 3.0. Workflow dedicado AppImage em push/PR/manual, Ubuntu 24.04, actions fixadas por commit e ferramentas conferidas por SHA256. Publica artifact com binário, checksum, fontes próprios e informações do build, sem criar Release ou exigir token customizado. Empacotamento removido do CI geral; GTK do CI corrigido para `xvfb-run -a scripts/test-ui.sh` com Secret Service descartável.

LICENSE contém o texto integral GPLv3; COPYRIGHT preserva autoria e identifica GPL-3.0-only. Workspace e seis crates, AppStream e documentação atualizados; empacotamento inclui os dois arquivos. Dependências mantêm suas licenças. Actionlint e metadados passaram na sessão Codex.

Retomada em 2026-11-03 (Claude Code): remote `origin` = github.com/rafaelhschuh/cctv-view, repositório vazio. Commits das actions fixadas conferidos via API do GitHub. Build local não repetido: os pacotes -dev de libav/libsecret/pulse não estão instalados neste host (o prefixo temporário /tmp/cctv-deps foi perdido) e o usuário optou por validar pelo GitHub. Usuário autorizou push explícito para este envio.

Primeira execução no GitHub: CI passou fmt, Clippy, testes, GTK com chaveiro descartável, protocolos, serviços e release, mas cargo-audit falhou por RUSTSEC-2026-0384 (tokio 1.54.0, também yanked); AppImage falhou porque os assets `continuous` de linuxdeploy/appimagetool mudaram e não conferiram com os hashes. Correção: tokio 1.54.1 e ferramentas fixadas em releases com tag, com SHA256 iguais aos digests publicados pelo GitHub.

Segunda execução (commit 50d4bdd): CI "Rust and native integration" passou por completo, incluindo cargo-audit, e o workflow AppImage passou na conferência das ferramentas, build, CLI sem display, smoke GTK sem Gtk-WARNING/CRITICAL/panic e `sha256sum --check`. Artifact CCTV-View-x86_64 (~73 MB) publicado com retenção até 2026-12-03. R03 concluída. Build local continua não repetido neste host (faltam pacotes -dev); AppImage em outro sistema e FUSE seguem em H02. Próxima ação: homologação H01/H02.

Publicação de Releases (2026-11-03): usuário perguntou por que não havia Release e pediu para corrigir o workflow. Adicionado job `release` em tags `v*`, dependente do build verificado, com `contents: write` só nele, checagem tag × versão do workspace, notas por `scripts/release-notes.sh` (testado localmente com 0.4.0 e versão inexistente) e `gh release create --verify-tag`. YAML conferido localmente; actions fixadas por commit. Nenhuma tag foi criada: a publicação real depende de o usuário pedir ou enviar `vX.Y.Z`.

Ajuste a pedido do usuário: em vez de tag manual, a Release sai automaticamente quando a versão muda. O job `release` roda em push na main, consulta `gh release view vX.Y.Z` e só publica (criando a tag no `GITHUB_SHA`) se ela não existir. Concorrência da main deixou de cancelar execuções. Como 0.4.0 ainda não tinha Release, este push publica v0.4.0.
