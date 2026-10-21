# Changelog

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
