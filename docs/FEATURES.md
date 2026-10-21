# Matriz de funcionalidades e validação

**0.4.0, prévia funcional. Não é uma declaração de homologação 1.0.** Código integrado e verificação local são separados de testes físicos. A especificação integral permanece em SPECIFICATION.md.

| Requisito | Implementação e evidência | Limite / estado de homologação |
|---|---|---|
| GTK4 nativo e tema | Janela real em Zorin 18, teste GTK/Xvfb e PNG inspecionado; lista conferida em claro/escuro | Wayland, HiDPI, leitor de tela e revisão completa de temas: NOT VERIFIED |
| Câmeras CRUD | Cadastro em abas com rodapé compacto, tipos RTSP/HTTP(S)/MJPEG/JPEG, opções específicas, Sem credenciais/validação TLS configuráveis, destino derivado da URL, endpoint ONVIF salvo sem consulta/invalidação, transporte/timeout/áudio/hardware/chaveiro; SQLite e GTK testados | Hardware de câmera: NOT VERIFIED |
| Biblioteca/pastas/transferência | Busca, pastas, clone completo com credenciais independentes, reautenticação por ação e JSON criptografado com senha do arquivo; URLs preservadas e importação aditiva/atômica | Senha do arquivo necessária para importar; Views/senha do aplicativo não são transferidas |
| RTSP | H.264 TCP autenticado no simulador; senha errada e indisponibilidade testadas | UDP e fabricantes: NOT VERIFIED |
| HTTP/JPEG/MJPEG/HLS/HEVC | Frames reais locais; modos explícitos MJPEG multipart/contínuo e JPEG periódico/cancelável testados | Variantes de fabricante: NOT VERIFIED |
| HTTPS | TLS validado por padrão; exceção explícita por câmera testada com TS/HLS/JPEG e SOAP autoassinados | Conexão com CA privada em hardware: NOT VERIFIED |
| ONVIF descoberta | Probe multicast e correlação; integração UDP local valida endpoints e deduplicação | Multicast em rede física/VLAN: NOT VERIFIED |
| ONVIF perfis/URI | GetDeviceInformation, GetCapabilities, GetProfiles, GetStreamUri em servidor autenticado | Media1; Media2 exclusivo e HTTP Digest não implementados |
| Multistream/sessões | Quatro decodes reais simultâneos; compartilhamento, descarte, reconexão e encerramento testados | Referências 4×1080p/9×720p/16 painéis: NOT VERIFIED |
| Views | Criar/editar/renomear/duplicar/excluir/ordenar/favoritar/inicial; linha nativa compacta com seleção, indicadores/menu inline, foco preservado e ordem transacional; CRUD e layout persistente testados | Revisão humana de todos os atalhos: NOT VERIFIED |
| Editor visual | Dashboard 12×12, desenho por retângulo, painéis personalizados e contraste; inserção por clique/teclado e DnD GTK, troca/reposição, alça de resize, dimensões/proporção, presets sem perda/undo, erro local, preview, salvar/cancelar | Teste GTK injeta drop e redimensiona no diálogo; arrasto físico da alça: NOT VERIFIED |
| Foco/zoom/pan/tela cheia | GDK snapshot, roda, arrasto e controles; duplo clique alternável, Enter/menu/X/Escape e apresentação sem chrome exercitados no GTK | Tela cheia com diferentes compositores: NOT VERIFIED |
| Áudio | libav + resample + PulseAudio, mute, volume, exclusividade; PCM real em sink nulo testado | Audição física e drift A/V longo: NOT VERIFIED |
| PTZ/presets | Direções/diagonais/zoom/velocidade; ContinuousMove/Stop/GetPresets/GotoPreset/SetPreset validados no simulador; ativação de botão GTK gera movimento/Stop limitado | Motor real, firmware e parada física: NOT VERIFIED |
| Reconexão e cancelamento | Interrupção por timeout/cancel, backoff, Weak de consumidores e testes de liberação | Stress prolongado e câmeras instáveis: NOT VERIFIED |
| Aceleração | VA-API opcional, detecção e fallback software; diagnóstico mostra backend usado | Driver/dispositivo VA-API: NOT VERIFIED |
| Senha do aplicativo/CLI | Gate inicial e imediato, hash/salt, alteração/remoção e CLI headless/stdin protegida testados | Não criptografa arquivos nem substitui a conta Linux |
| Segurança | Secret Service roundtrip/delete em sessão isolada, URLs sem userinfo, XML limitado e TLS | Revisão independente: NOT VERIFIED |
| Métricas | CPU/RSS, frames/descartes, idade local do frame, decoder, PCM entregue | Não mede latência desde o sensor |
| Documentação/agentes/CI | AGENTS, ADR, plano, tarefas, progresso, scripts e workflow presentes | CI remota não executada: apenas commits locais |
| Release/AppImage | Release compilado; pacote local gerado; smoke com extração/Xvfb e dados temporários | Outro sistema e execução FUSE: NOT VERIFIED |

Para considerar a primeira versão completa, concluir as verificações físicas da especificação, compatibilidade de dispositivos escolhidos, desempenho e AppImage em sistema separado. As restrições explícitas acima não foram removidas do backlog.
