# Revisão UI/UX — 0.1.1

Revisão crítica independente pelo subagente Schrodinger, seguida de implementação e conferência pontual. A interface permanece GTK4 nativa e usa o tema do sistema. Prioridades: organização por tarefa, ações junto de seu alvo, alternativas ao arrasto, preservação dos dados e recuperação de erros.

| Problema identificado | Correção |
|---|---|
| Endpoint ONVIF não persistia ao salvar sem consultar | Coletor valida/salva explicitamente e invalida perfis, capacidades e autorização ao mudar dispositivo |
| Botões + e biblioteca aceitavam somente arrasto | Seletor de câmera por clique/ativação de teclado e inserção pela lista; DnD preservado |
| Nome da View ilegível em janela pequena | Nome em linha própria; controles de grade agrupados abaixo |
| URL, endereço e porta competiam como fontes da conexão | URL completa define host/porta e destino informativo |
| Cadastro plano com opções avançadas antes das credenciais | Abas Conexão, ONVIF e Avançado; seções e rolagem própria |
| Feedback fora da área visível ou atrás de modal | Rodapé fixo no cadastro; erro local no diálogo do painel; foco no campo inválido |
| Preset substituía painéis e câmeras | Reflow preserva identidade/câmera/proporção, rejeita perda e oferece até 20 passos de desfazer; preencher é explícito |
| Estado vazio sem ação e comandos sem alvo habilitados | Adicionar câmera/Criar View/Escolher câmeras e Editar View desabilitado sem seleção |
| Foco dependia de duplo clique e retorno não era visível | Enter, menu Ampliar câmera e botão Voltar à grade; Escape preservado |
| PTZ dependia de gestos do mouse | Ativação padrão de botão gera pulso limitado com Stop; atalhos de diagonais/zoom; campos preservam teclas de edição |
| Rótulos sem associação acessível explícita | Relação rótulo/controle e nomes acessíveis GTK nos comandos |
| Ações de View distantes, inicial sem indicador e sidebar ocupava janela pequena | Menu por linha, badges Favorita/Inicial, ordenação transacional e recolhimento adaptativo |

A segunda revisão encontrou três regressões, também corrigidas: consulta ONVIF reabilita o seletor e permite escolher outro perfil; favoritar outra View preserva seleção e reprodução; duplicação limpa foco e renderização descarta referência a painel removido.

## Evidência

- 14 testes padrão de core/storage/media/ONVIF passaram, além da regressão GTK ampliada no Xvfb; fmt, Clippy com warnings negados e build release passaram.
- GTK real: endpoint salvo sem consulta/invalidação; mock ONVIF com dois perfis selecionáveis e URLs distintas; inserção por lista/+ e drop; preset/undo e cancelamento; erro de sobreposição local; Enter/menu/retorno/Escape; sidebar pequena/grande; favoritos de outra View, duplicação em foco, ordenação e exclusão da inicial; movimento/Stop PTZ e liberação do estado.
- AppImage 0.1.1 gerado e smoke por extração/Xvfb com banco temporário passou; checksum conferido. A conferência pontual final do revisor não encontrou novos problemas nos três ajustes adicionais.
- Capturas de monitor, cadastro, editor e erro local inspecionadas. Exemplos pequenos abaixo; vídeo é uma fixture sintética, sem câmeras ou credenciais privadas.

![Monitor em janela pequena](screenshots/monitor-small.png)

![Cadastro com rolagem e rodapé fixo](screenshots/camera-small.png)

![Editor com nome legível e ações de inserção](screenshots/editor-small.png)

Capturas mostram widgets no tema instalado; não substituem testes humanos. Leitor de tela/Orca, arrasto físico, Wayland/HiDPI, motor PTZ, VA-API e câmeras reais continuam NOT VERIFIED. Ver TESTING.md e FEATURES.md para reprodução e limites. AppImage e checksum ficam em dist/; somente commits locais.

## Ajustes solicitados — 0.1.2

- Rodapé: a área vazia de feedback foi removida do cálculo de altura; Cancelar/Salvar ficam junto das abas. A regressão GTK mede o intervalo e confirma erro de timeout visível sem fechar o formulário.
- Tela principal: removido o texto permanente “Pronto”. A área de status fica oculta sem erros reais.
- Conexões: seletor RTSP/HTTP(S)/MJPEG/JPEG com opções avançadas e instruções específicas. Persistência e validação por tipo, demuxers explícitos para MJPEG e renovação periódica de JPEG no player nativo; cadastros antigos mantêm Auto.
- Views: linhas com altura uniforme, subtítulos reservados e menus centralizados; sem rolagem horizontal, nomes longos truncados e fundo do tema nativo.
- Evidência: 17 testes padrão, incluindo formatos MJPEG, intervalo/cancelamento JPEG e JSON legado; regressão GTK com seleção/opções, persistência, rodapé normal/erro e linhas uniformes. Capturas inspecionadas. Hardware e tecnologias assistivas seguem os limites de FEATURES.md.

![Cadastro compacto e tipo de conexão](screenshots/connection-types.png)

![Opções específicas de JPEG](screenshots/jpeg-options.png)

![Lista de Views alinhada](screenshots/views-aligned.png)

Pacote 0.1.2: checks, matriz de protocolos e smoke de inicialização/encerramento do AppImage passaram; checksum conferido. Sem push.

## Correção da regressão da lista — 0.1.3

A captura do usuário revelou que a reserva de subtítulo e a altura de 60 px da versão 0.1.2 tornaram um item simples excessivamente alto. Esse tratamento foi removido. Agora há uma única ListBoxRow nativa selecionável com nome, indicadores opcionais e menu sem moldura; favorita/inicial não criam linha extra. Ativar uma View preserva o foco no item. A seleção abrange também a área do menu, sem botão colorido separado.

A regressão GTK cobre uma View simples, várias Views, nome longo, favorita, seleção e ativação da segunda linha com foco preservado; confere altura uniforme de até 48 px. Capturas em temas claro e escuro inspecionadas. Isso substitui a proposta visual anterior para a lista; não muda os modelos ou serviços de reprodução.

![Uma View no tema escuro, sem altura vazia](screenshots/view-single-dark.png)

![Lista compacta com menu e favorita na mesma linha](screenshots/views-compact-dark.png)
