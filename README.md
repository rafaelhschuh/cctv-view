# CCTV View

Aplicativo Linux nativo para monitoramento de câmeras IP, em Rust, GTK 4 e FFmpeg/libav. Interface em português e tema do sistema, com prioridade para Zorin OS 18 x86_64.

A versão **0.4.0** integra cadastro de câmeras, reprodução multistream, Views persistentes, editor visual, foco, zoom digital, áudio exclusivo, descoberta ONVIF, perfis, PTZ e presets. É uma versão inicial: validação em câmeras físicas, drivers VA-API e em outro computador permanece pendente. Consulte a [matriz de validação](docs/FEATURES.md) antes de usar em operação crítica.

## Executar

O pacote local é gerado em `dist/CCTV-View-x86_64.AppImage`:

```sh
chmod +x dist/CCTV-View-x86_64.AppImage
./dist/CCTV-View-x86_64.AppImage
```

Sem FUSE: `APPIMAGE_EXTRACT_AND_RUN=1 ./dist/CCTV-View-x86_64.AppImage`.

Para desenvolvimento, instale as [dependências nativas](docs/BUILD.md) e execute `scripts/dev.sh`. Verificações: `scripts/check.sh`.

## Usar

1. Abra **Câmeras** na barra superior; adicione uma URL RTSP/HTTP/HTTPS ou descubra um dispositivo ONVIF na rede.
2. Na aba **Conexão**, escolha **RTSP**, **HTTP(S)**, **MJPEG** ou **JPEG**, e informe a URL completa. Para streams públicas, marque **Sem credenciais**; para streams autenticadas, informe usuário/senha. Host e porta são derivados da URL. Na aba **ONVIF**, salve o endpoint mesmo sem consultar ou consulte os perfis e autorize PTZ. A aba **Avançado** mostra as opções do tipo escolhido: transporte TCP/UDP no RTSP, formato multipart/contínuo no MJPEG e intervalo de atualização no JPEG (200–60000 ms). HTTP(S) detecta vídeo/HLS automaticamente e valida TLS por padrão. Em **Avançado → Segurança HTTPS**, desmarque **Validar certificado HTTPS** para aceitar certificados autoassinados ou inválidos nessa câmera, inclusive no ONVIF. Timeout é comum; substream, áudio e aceleração aparecem para RTSP/HTTP(S). Use **Testar conexão de vídeo** antes de salvar.
3. Crie uma **Nova View**, use a grade **12×12** e escolha um layout. Adicione pelo **+**, pela lista ou desenhe um retângulo numa região vazia; depois escolha a câmera. Painéis podem ter larguras/alturas diferentes. Trocar o layout preserva os painéis; **Desfazer layout** recupera a alteração anterior. Arraste painéis para reposicionar; use a alça ↘ para redimensionar. Clique para configurar câmera, posição, tamanho e proporção.
4. Salve a View. Enter no painel, **Ampliar câmera** no menu ou duplo clique ativa foco; roda do mouse faz zoom e arrasto faz pan. Outro duplo clique, **X flutuante** no canto superior direito ou Escape retorna; F11 alterna tela cheia somente com streams: cabeçalho, lista lateral e barras das câmeras ficam ocultos.
5. No foco, habilite áudio ou abra o PTZ, quando disponível. Segure os controles PTZ; solte para parar. Ativar um botão por teclado produz um pulso curto. Setas, diagonais (Home/Page Up/End/Page Down) e +/− também controlam o movimento; Escape para.

O menu ao lado de cada View reúne edição, duplicação, favoritos, inicialização, ordem e exclusão. Em janelas estreitas, a lista lateral recolhe automaticamente; o botão da barra permite reabri-la.

Dados seguem XDG e credenciais ficam no chaveiro Secret Service. Não existe telemetria, servidor de aplicação nem execução como root.

[Documentação](docs/README.md) · [Arquitetura](docs/ARCHITECTURE.md) · [Testes](docs/TESTING.md) · [Progresso](docs/PROGRESS.md)

## Organizar e transferir câmeras

Na **Biblioteca de câmeras**, use a busca e o filtro de pasta. **Nova pasta** cria uma pasta vazia; os ícones ao lado do filtro renomeiam ou excluem a pasta selecionada. Excluir uma pasta mantém as câmeras em **Sem pasta**. Em cada linha: editar, clonar, mover para pasta e excluir câmera. Clonar cria uma cópia completa, incluindo URLs, configurações e credenciais, com nova referência independente no chaveiro.

Se houver senha no aplicativo, **editar/visualizar os dados, clonar e exportar** sempre pedem essa senha em um diálogo, mesmo com o aplicativo desbloqueado. Sem senha configurada, essas ações abrem diretamente.

Em **Preferências → Biblioteca**, exporte/importe um JSON **criptografado** com câmeras, pastas e credenciais. A exportação pede uma senha para o arquivo, separada da senha do aplicativo; a importação pede essa senha para descriptografar e mostra a prévia antes de adicionar novos cadastros. URLs completas, inclusive parâmetros e fragmentos, são preservadas. Views e a senha de bloqueio do aplicativo não são transferidas. Exportar exige um nome de arquivo novo.

## Senha do aplicativo

Em **Preferências → Segurança → Senha do aplicativo**, informe nova senha e confirmação (mínimo de 8 caracteres). Senha atual é necessária para alterar/remover uma existente. **Bloquear agora** encerra as sessões de vídeo e mostra a tela de desbloqueio; a senha também é exigida na abertura. O bloqueio protege a interface e as listagens CLI; não criptografa o SQLite nem substitui as permissões da conta Linux.

## CLI para scripts

```sh
./dist/CCTV-View-x86_64.AppImage --help
./dist/CCTV-View-x86_64.AppImage --list-views
./dist/CCTV-View-x86_64.AppImage --view "Portaria" --fullscreen
./dist/CCTV-View-x86_64.AppImage --data-dir /caminho/biblioteca --list-cameras
```

As listagens JSON funcionam sem display e omitem URLs/credenciais. Bibliotecas com senha exigem desbloqueio. `--unlock-stdin` recebe a senha por pipe de um gerenciador de segredos, sem colocá-la em argumentos; sem essa opção, a abertura gráfica pede a senha. Consulte [CLI](docs/CLI.md).
