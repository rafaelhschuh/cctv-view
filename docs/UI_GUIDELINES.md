# Interface GTK

Usar widgets GTK nativos e preferências do sistema. Não há libadwaita obrigatória nem tema próprio. CSS é limitado à classe cctv-player-status, para garantir contraste das mensagens sobre o vídeo. Headerbar, sidebar, diálogos, botões simbólicos e estilos semânticos seguem o Zorin instalado. Ícones têm tooltips; entradas identificam seu propósito.

O menu de cada câmera (também acessível pelo botão direito) permite configurar, reconectar e abrir o editor de View. Dashboard e editor são modos distintos. A grade de visualização preserva as células vazias. Duplo clique foca sem modificar o modelo; Escape restaura a View. F11 alterna tela cheia e Ctrl+Q fecha janelas, cancela sessões e persiste preferências. No foco, roda controla zoom de 1× a 8×, arrasto faz pan limitado à imagem e os botões restauram o zoom. Proporção pode manter, preencher com recorte ou esticar.

O editor usa DnD GTK com identificadores internos. Soltar câmera em painel troca sua referência; soltar painel ocupado tenta trocar posições. Todas as operações passam pela validação de limites e sobreposição. Coluna/linha são coordenadas base zero. A alça ↘ redimensiona com snap; o diálogo permite precisão por teclado. Cancelar descarta o rascunho.

Estados vazios explicam o próximo passo; erros não apresentam credenciais. Rede, chaveiro e decode não bloqueiam o GTK. Não há animações próprias, portanto não existe movimento ornamental a desativar.

Validação realizada: teste GTK com vídeo real em Xvfb, cadastro, editor, persistência, sinais de foco/Escape/drop, redimensionamento numérico e cancelamento; captura de janela inspecionada. Testes humanos de DnD, leitor de tela, tema escuro, Wayland e HiDPI permanecem NOT VERIFIED.
