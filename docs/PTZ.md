# PTZ

A UI só oferece PTZ no foco quando o usuário autoriza, existe serviço PTZ e o perfil selecionado anuncia PTZConfiguration. O controle opera sobre o ProfileToken escolhido; zoom óptico é independente do zoom digital do canvas.

ContinuousMove envia PanTilt e Zoom com velocidades entre -1 e 1 e Timeout PT0.5S. Segurar renova o comando a cada aproximadamente 200 ms mais a duração da chamada. Stop é solicitado ao soltar/cancelar, perder foco, fechar a janela e ao falhar. A geração de comandos invalida movimentos antigos e o mutex serializa requisições para que um Stop não seja ultrapassado por um movimento antigo. Setas do teclado controlam direção; Escape solicita parada.

GetPresets, GotoPreset e SetPreset estão integrados à janela. Recursos recusados por SOAP Fault são informados ao usuário. Não é presumido que todo dispositivo PTZ aceite todos os comandos; diagonais, zoom e criação de presets dependem do equipamento.

Testes locais cobrem o XML de movimento/parada, autenticação, timeout do dispositivo, consulta e acionamento/criação de presets. A resposta física, latência do motor, soltura em Wayland e firmware que ignore Stop/Timeout permanecem NOT VERIFIED. Não há garantia de parada física em uma falha da câmera ou da rede.
