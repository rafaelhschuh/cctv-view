# Changelog

## 0.4.1 — 2026-11-03

- Atualização automática do AppImage: informação `gh-releases-zsync` embutida e `CCTV-View-x86_64.AppImage.zsync` publicado em cada Release. Gear Lever e AppImageUpdate detectam a origem e baixam só os blocos alterados. Quem está na 0.4.0 precisa instalar a 0.4.1 manualmente uma vez.
- Ferramentas de empacotamento fixadas em releases com tag e tokio 1.54.1 (RUSTSEC-2026-0384).
- Workflow AppImage dedicado para push, pull request e execução manual: runner Ubuntu 24.04, ferramentas conferidas por SHA256, build release, smoke headless/GTK e artifact com AppImage, checksum, fontes próprios e informações do build.
- CI usa o chaveiro descartável em `scripts/test-ui.sh`; empacotamento removido do CI geral para evitar duplicação.
- Push na main publica automaticamente a GitHub Release `vX.Y.Z` quando a versão do workspace ainda não foi lançada: tag criada pelo workflow, AppImage, checksum, fontes e informações do build, notas extraídas do CHANGELOG.
- Licença alterada para GPL-3.0-only, com texto integral GPLv3, copyright, herança explícita nos seis crates e metadados AppStream. O pacote inclui LICENSE e COPYRIGHT; dependências preservam suas próprias licenças.

## 0.4.0 — 2026-10-21

- Editar/visualizar dados, clonar e exportar câmeras pedem a senha do aplicativo a cada ação, quando configurada. Cancelar ou errar a senha não libera a operação; desbloqueio inicial não guarda essa autorização.
- Clone completo em uma ação: preserva URLs, parâmetros, fragmentos, configurações/perfis e credenciais em nova entrada independente no Secret Service. Edição permite visualizar a senha salva após autorização.
- Removida a revisão obrigatória de URLs e a exclusão de credenciais da transferência. Exportação sempre cria JSON criptografado com senha escolhida na hora e confirmação; importação exige a senha do arquivo.
- Criptografia autenticada AES-256-GCM, salt/nonce aleatórios, chave derivada por PBKDF2-HMAC-SHA256; formato versionado com limites e rejeição de senha incorreta/arquivo alterado antes de importar. Senhas de câmera continuam fora do SQLite e logs.
- Restauração de credenciais em novos IDs do chaveiro, transação SQLite e limpeza compensatória em falha; teste GTK usa chaveiro e D-Bus descartáveis.
- Verificado com 27 testes padrão, GTK claro/escuro, fmt/Clippy/release, auditoria local e smoke/checksum do AppImage 0.4.0.

## 0.3.0 — 2026-10-21

- Biblioteca com busca por nome/endereço/fabricante, filtro por pasta, contador e linhas nativas com nome, pasta/endereço, tipo e ações alinhadas.
- Pastas persistentes: criar, renomear, mover câmeras e excluir sem apagar câmeras, credenciais ou painéis de Views. Renomear preserva a seleção.
- Clonar abre um rascunho editável com novo identificador, sem credenciais/perfis/autorização PTZ. Cancelar não grava a cópia.
- Preferências organizadas em Geral, Biblioteca e Segurança, com áudio, atalhos, diagnóstico e bloqueio em seções próprias.
- Exportação/importação JSON versionada de câmeras e pastas, sem ler o chaveiro ou transferir senha/referências. Parâmetros e fragmentos de URL omitidos; revisão editável antes de exportar para remover tokens do caminho. Importação validada com prévia, novos IDs e transação aditiva; não altera Views nem senha do aplicativo.
- Exportação cria arquivo privado sem sobrescrever arquivos existentes. Importação tem limites de tamanho/quantidade, recusa campos desconhecidos e impede cancelamento depois de iniciar a gravação.
- Verificação: 26 testes padrão, GTK em claro/escuro, fmt/Clippy/release, auditoria local e smoke do AppImage 0.3.0 com checksum.

## 0.2.0 — 2026-10-21

- Duplo clique alterna foco; X flutuante substitui a faixa de retorno. Tela cheia oculta cabeçalho, sidebar, barras e textos sob os vídeos; remove margens externas vazias da área ocupada pelos painéis. Canvas mantém clipping arredondado antes/depois do primeiro frame.
- Editor 12×12 com desenho de retângulos, painéis de tamanhos diferentes, DnD, resize e presets proporcionais. Fundos/bordas contrastantes, tamanho inicial configurável e undo. Views antigas são convertidas somente no rascunho; conversões inválidas preservam a grade original.
- CLI para View por nome/ID, fullscreen, biblioteca específica, listagens JSON sem display, help/version e desbloqueio por stdin.
- Senha do aplicativo em Preferências: configurar/alterar/remover, bloqueio inicial/imediato e parada das sessões. Apenas hash PBKDF2/salt no SQLite; senha atual obrigatória para mudança/remoção. Listagens protegidas e nenhum argumento de senha.
- Testes de geometria, hash/salt/mudança, CLI headless e GTK com desenho/resize, foco/apresentação e senha incorreta/correta; AppImage atualizado.

## 0.1.4 — 2026-10-21

- Opção Validar certificado HTTPS em Avançado, marcada por padrão e configurável por câmera. Aplicada ao player, teste de vídeo, consulta ONVIF e PTZ; certificado autoassinado aceito ao desmarcar, sem alterar o padrão das outras câmeras.
- Opção Sem credenciais em Conexão para streams públicas: oculta usuário/senha, não envia autenticação nem acessa o chaveiro durante reprodução/consulta/PTZ. Credenciais salvas são preservadas, com remoção explícita separada; reativar autenticação carrega o chaveiro em worker.
- Campos persistidos com defaults compatíveis para cadastros antigos. Regressões de persistência/GTK e HTTPS TS/HLS/JPEG/SOAP, incluindo recusa padrão após usar a exceção; AppImage atualizado.

## 0.1.3 — 2026-10-21

- Corrigida a regressão de densidade da lista de Views: removidos altura fixa de 60 px, subtítulo vazio e botão de seleção separado.
- Uma linha nativa selecionável reúne nome, indicadores opcionais e menu sem moldura, preservando foco de teclado ao ativar a View.
- Conferência GTK com uma/várias Views nos temas claro e escuro e AppImage atualizado.

## 0.1.2 — 2026-10-21

- Rodapé do cadastro compacto: feedback vazio não reserva altura; erros e resultados aparecem quando necessários. Removida a mensagem permanente “Pronto” da janela principal.
- Seletor RTSP/HTTP(S)/MJPEG/JPEG, instruções e opções avançadas específicas. RTSP configura TCP/UDP; MJPEG seleciona multipart ou fluxo contínuo; JPEG configura intervalo real de novas requisições, mantendo a imagem entre atualizações.
- Tipo de fonte validado/persistido independentemente do serviço ONVIF. Cadastros antigos preservam detecção automática, sem migração destrutiva.
- Linhas de Views com alturas, subtítulos e menus alinhados; nomes longos permanecem contidos na sidebar.
- Regressões com decodificação nativa dos dois modos MJPEG, atualização/cancelamento JPEG e opções/rodapé/alinhamento no GTK real; AppImage atualizado.

## 0.1.1 — 2026-10-21

- Cadastro em abas Conexão/ONVIF/Avançado, feedback fora da rolagem e foco no campo inválido. URL do vídeo define host/porta; endpoint ONVIF é salvo sem depender de consulta e sua alteração invalida perfis/capacidades anteriores.
- Editor com nome legível, seleção de câmera por clique/teclado além de DnD, preenchimento explícito, layouts que preservam painéis e histórico de desfazer. Erros aparecem no diálogo de painel; coordenadas visíveis começam em 1.
- Ações junto de cada View, indicadores de favorita/inicial, ordenação transacional e sidebar adaptativa. Estados vazios oferecem ações; foco por Enter/menu e botão visível de retorno à grade.
- Rótulos acessíveis nos controles e PTZ com ativação por teclado, pulso limitado e atalhos de diagonais/zoom sem interceptar edição de campos.
- Regressões de modelos, persistência, GTK e movimento/Stop no simulador; AppImage atualizado. Homologação física e leitor de tela continuam pendentes.

## 0.1.0 — 2026-10-21

- Workspace Rust com aplicação GTK nativa, biblioteca de câmeras e armazenamento SQLite versionado.
- Parser XML 0.42 com limites e respostas SOAP verificadas; auditoria RustSec sem vulnerabilidades conhecidas no lockfile.
- Credenciais em Secret Service; validação de URLs e TLS por padrão.
- Player libav em workers, sessões compartilhadas, último frame limitado, reconexão e fallback de VA-API para software.
- Áudio via libswresample/PulseAudio, volume e seleção exclusiva de câmera.
- Views, favoritos, ordem, inicialização persistente, foco, zoom e editor de grade com DnD e redimensionamento.
- ONVIF WS-Discovery/Media1, autenticação UsernameToken digest, controle PTZ e presets.
- Métricas locais de sessões, CPU, memória, frames descartados e áudio entregue.
- Simuladores e testes nativos, CI, release e empacotamento AppImage.

Ainda requer validação física de câmeras, drivers, acessibilidade com leitor de tela, Wayland/HiDPI e distribuição em um sistema separado. Não representa certificação ONVIF nem garantia de desempenho por quantidade de câmeras.
