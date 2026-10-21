# Interface GTK

Usar widgets GTK nativos e preferências do sistema. Não há libadwaita obrigatória nem tema próprio. CSS é limitado à classe cctv-player-status, para garantir contraste das mensagens sobre o vídeo. Headerbar, sidebar, diálogos, botões simbólicos e estilos semânticos seguem o Zorin instalado. Ícones têm tooltips e nomes acessíveis; rótulos são associados aos controles com metadados GTK.

O menu de cada câmera (também acessível pelo botão direito) permite configurar, reconectar e abrir o editor de View. Dashboard e editor são modos distintos. A grade de visualização preserva as células vazias. Enter no painel, menu Ampliar câmera e duplo clique focam sem modificar o modelo; Voltar à grade e Escape restauram a View. F11 alterna tela cheia e Ctrl+Q fecha janelas, cancela sessões e persiste preferências. No foco, roda controla zoom de 1× a 8×, arrasto faz pan limitado à imagem e os botões restauram o zoom. Proporção pode manter, preencher com recorte ou esticar.

O editor usa DnD GTK com identificadores internos. Soltar câmera em painel troca sua referência; soltar painel ocupado tenta trocar posições. Todas as operações passam pela validação de limites e sobreposição. Coluna/linha no diálogo começam em 1; o modelo persistido continua base zero. A alça ↘ redimensiona com snap; o diálogo permite precisão por teclado. Cancelar descarta o rascunho. A lista e os botões + adicionam por clique/ativação de teclado. Trocar preset reorganiza painéis sem remover câmeras ou proporções; se não couber, o estado anterior permanece. Preencher espaços é uma ação explícita. Desfazer layout mantém até 20 alterações. O diálogo do painel apresenta erros locais sem fechar.

O cadastro separa Conexão, ONVIF e Avançado em abas com rolagem própria e feedback fixo no rodapé. Salvar ONVIF não exige consultar; alterar endpoint invalida capacidades e autorização antigas. A URL do vídeo determina host/porta, exibidos como destino.

Ações da View ficam no menu de cada linha. Favorita e Inicial aparecem junto do nome; sem View, Editar View fica desabilitado. A sidebar recolhe abaixo de 900 px sem alterar a preferência persistida e restaura em largura maior. O usuário pode reabri-la.

Estados vazios oferecem Adicionar câmera e Criar View/Escolher câmeras; erros não apresentam credenciais. Rede, chaveiro e decode não bloqueiam o GTK. Não há animações próprias, portanto não existe movimento ornamental a desativar.

Validação realizada: teste GTK com vídeo real em Xvfb: cadastro/endpoint ONVIF sem consulta e invalidação, inserção por clique e drop, preservação/undo, feedback local, Enter/menu/retorno/Escape, sidebar adaptativa, cancelamento e pulso PTZ com movimento/Stop em servidor local. Capturas de janela grande/pequena inspecionadas. Testes humanos de DnD, leitor de tela, tema escuro, Wayland e HiDPI permanecem NOT VERIFIED.

Na revisão 0.1.2, o rodapé do cadastro não reserva altura para mensagens vazias. O seletor de conexão controla instruções, placeholder e grupos avançados (RTSP, HTTP(S), MJPEG e JPEG), com opções incompatíveis ocultas; a seleção é persistida e aplicada no player. Erros direcionam à aba/campo correspondente. A janela principal não exibe texto permanente de prontidão. Linhas de Views mantêm altura e espaço de subtítulo iguais; menus ficam centralizados, nomes longos são truncados e a rolagem horizontal é desativada.
