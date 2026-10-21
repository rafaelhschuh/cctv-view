# CLI

A mesma CLI está disponível no binário `target/release/cctv-view` e no AppImage. Não requer display para help/version/listagens. Use --help para todas as opções.

```sh
./dist/CCTV-View-x86_64.AppImage --version
./dist/CCTV-View-x86_64.AppImage --list-views
./dist/CCTV-View-x86_64.AppImage --list-cameras
./dist/CCTV-View-x86_64.AppImage --view "Portaria" --fullscreen
./dist/CCTV-View-x86_64.AppImage --data-dir /caminho/biblioteca --view "Portaria"
```

--view aceita nome exato ou ID existente; --fullscreen abre apresentação sem barras/sidebar. Invocações com opções explícitas de abertura usam instância própria. Sem opções, reutiliza a instância registrada quando disponível. F11/Escape sai da apresentação; duplo clique alterna foco.

Listagens JSON contêm apenas ID/nome e geometria/contagem de painéis da View. Não exportam URL, senha ou referência ao chaveiro. Retorno 0 indica sucesso; erros de parser/autenticação/listagem retornam 1. Na abertura gráfica, View inexistente gera mensagem na interface. --data-dir define uma biblioteca independente; por padrão usa XDG ou CCTV_VIEW_DATA_DIR.

Quando houver senha na biblioteca, a GUI pede desbloqueio antes de reproduzir e listagens sem autenticação são recusadas. Para automação, --unlock-stdin recebe a senha por pipe de um gerenciador de segredos; não aceita entrada de terminal com eco nem senha em argumento. Evite inserir senha real no histórico, repositório ou logs. A senha é configurada/alterada/removida em Preferências → Senha do aplicativo; Bloquear agora encerra as sessões e mostra o gate.

O AppImage precisa de FUSE ou APPIMAGE_EXTRACT_AND_RUN=1, inclusive em comandos headless. A CLI não inicia subprocessos de reprodução nem um servidor de controle. A proteção é por biblioteca e não criptografa arquivos; veja SECURITY.md.
