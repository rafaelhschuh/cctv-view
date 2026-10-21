# CCTV View — Especificação de Desenvolvimento

## 1. Objetivo

Desenvolver um aplicativo desktop Linux completo, nativo, moderno e de alto desempenho para monitoramento de câmeras CCTV/IP.

**Identidade do projeto**

- Nome: CCTV View
- Application ID: `br.dev.schuh.cctv-view`
- Executável: `cctv-view`
- Linguagem: Rust
- Interface gráfica: GTK 4 (gtk4-rs)
- Motor multimídia: FFmpeg/libav
- Plataforma prioritária: Zorin OS
- Distribuição: AppImage
- Idioma principal: Português do Brasil

O produto deve ser um aplicativo de produção, não uma demonstração, mockup ou protótipo visual.

Todos os requisitos principais, incluindo ONVIF, PTZ, áudio, zoom, editor de Views e reprodução de múltiplas câmeras, pertencem ao escopo obrigatório da primeira versão completa.

É permitido organizar o desenvolvimento em etapas para manter a qualidade técnica, mas não remover silenciosamente funcionalidades do escopo.

---

## 2. Papel e comportamento da IA

Atue como uma equipe de engenharia de software sênior, com experiência em:

- Rust e aplicações desktop Linux.
- GTK 4, GLib, GIO e GDK.
- FFmpeg, codecs e aceleração por hardware.
- RTSP, HTTP, HTTPS e ONVIF.
- Arquitetura assíncrona e sistemas concorrentes.
- Interfaces desktop de alto desempenho.
- Segurança e gerenciamento de credenciais.
- Testes, CI/CD e distribuição AppImage.
- Manutenção de projetos desenvolvidos com agentes de IA.

O objetivo não é produzir a maior quantidade de código possível.

O objetivo é produzir **software correto, funcional, modular, testável, documentado e fácil de evoluir**.

### Princípios obrigatórios

1. Compreender antes de modificar.
2. Planejar antes de implementar.
3. Implementar funcionalidades completas, não apenas componentes visuais.
4. Fazer alterações pequenas e verificáveis.
5. Compilar e testar regularmente.
6. Corrigir a causa dos problemas, não apenas seus sintomas.
7. Documentar decisões e limitações.
8. Nunca declarar como funcional algo não implementado.
9. Não remover funcionalidades existentes para facilitar uma alteração.
10. Manter o projeto compreensível para outros desenvolvedores e agentes.

---

## 3. Boas práticas de vibe coding e engenharia com IA

O repositório deve ser preparado desde o início para desenvolvimento assistido por agentes.

### 3.1 Arquivo AGENTS.md

Criar `AGENTS.md` na raiz do projeto como documento canônico de instruções para agentes de programação.

Criar também `agent.md`, caso seja necessário para compatibilidade com ferramentas específicas, apontando para `AGENTS.md` sem duplicar instruções.

O `AGENTS.md` deve conter:

- Descrição e objetivos do produto.
- Stack tecnológica obrigatória.
- Estrutura e responsabilidades dos módulos.
- Convenções de código Rust.
- Convenções de interface GTK.
- Regras de concorrência.
- Regras de integração FFmpeg.
- Regras de segurança.
- Procedimentos de build.
- Comandos de teste.
- Critérios de aceitação.
- Regras para alterações arquiteturais.
- Política de documentação.
- Instruções de continuidade entre sessões.

### Regras dos agentes

Antes de qualquer tarefa relevante, o agente deve:

1. Ler `AGENTS.md`.
2. Consultar a documentação relacionada.
3. Inspecionar o código e seus pontos de integração.
4. Verificar se a tarefa já está implementada parcial ou totalmente.
5. Elaborar um plano objetivo.
6. Identificar riscos e dependências.
7. Implementar seguindo os padrões existentes.
8. Executar as validações apropriadas.
9. Atualizar documentação e status.
10. Apresentar um resumo verificável.

**Proibição:** não começar a reescrever módulos inteiros sem antes entender a implementação existente.

### 3.2 Planejamento persistente

Manter os arquivos:

`docs/PLAN.md`

Plano geral de desenvolvimento, organizado em entregas, dependências e critérios de aceitação.

`docs/TASKS.md`

Backlog de tarefas, com estados:

- TODO
- IN PROGRESS
- BLOCKED
- DONE

Cada tarefa relevante deve registrar:

- Identificador.
- Objetivo.
- Contexto.
- Arquivos ou módulos afetados.
- Dependências.
- Critérios de aceitação.
- Testes necessários.
- Estado atual.

`docs/PROGRESS.md`

Diário técnico resumido, contendo:

- Funcionalidades concluídas.
- Mudanças arquiteturais.
- Problemas identificados.
- Soluções aplicadas.
- Testes executados.
- Limitações pendentes.
- Próxima ação recomendada.

Esses documentos devem permitir que outro agente retome o desenvolvimento sem depender do histórico completo de uma conversa.

### 3.3 Documentação viva

Criar e manter:

```text
docs/
├── README.md
├── ARCHITECTURE.md
├── DEVELOPMENT.md
├── BUILD.md
├── APPIMAGE.md
├── PLAN.md
├── TASKS.md
├── PROGRESS.md
├── FEATURES.md
├── DATABASE.md
├── SECURITY.md
├── TESTING.md
├── TROUBLESHOOTING.md
├── UI_GUIDELINES.md
├── CAMERA_PROTOCOLS.md
├── FFMPEG_PIPELINE.md
├── ONVIF.md
├── PTZ.md
└── adr/
    └── 0001-initial-architecture.md
```

Não criar documentação fictícia apenas para preencher arquivos.

Cada documento deve conter informações úteis, reais e consistentes com a implementação.

Quando modificar uma funcionalidade, atualizar a documentação correspondente na mesma tarefa.

### 3.4 Registros de decisões — ADR

Utilizar Architecture Decision Records para decisões técnicas importantes.

Exemplos:

- Escolha do backend multimídia.
- Estratégia de renderização GTK.
- Gerenciamento de sessões RTSP.
- Integração ONVIF.
- Modelo de concorrência.
- Estratégia de armazenamento.
- Empacotamento AppImage.

Cada ADR deve explicar:

- Contexto.
- Alternativas consideradas.
- Decisão tomada.
- Justificativa.
- Consequências.
- Limitações conhecidas.

Evitar mudanças arquiteturais arbitrárias entre sessões de desenvolvimento.

### 3.5 Desenvolvimento incremental

Trabalhar com entregas verticais verificáveis.

Uma funcionalidade somente deve ser considerada concluída quando seus componentes necessários estiverem integrados:

- Modelo de dados.
- Lógica de negócio.
- Integração com serviços.
- Interface.
- Tratamento de erros.
- Persistência, quando aplicável.
- Testes.
- Documentação.

Não implementar toda a interface primeiro para somente depois descobrir se os protocolos e o player funcionam.

### 3.6 Definição de pronto

Uma tarefa somente recebe status DONE quando:

- Seu código está implementado.
- Está integrado ao restante do aplicativo.
- Não possui placeholders disfarçados de implementação.
- Não introduz erros de compilação.
- Passa nos testes disponíveis.
- Possui tratamento apropriado de erros.
- Não provoca bloqueios conhecidos na UI.
- Está documentada.
- Seus critérios de aceitação foram atendidos.

Se a execução de testes não for possível, registrar explicitamente `NOT VERIFIED`, com a justificativa.

### 3.7 Controle de contexto

Para preservar a continuidade entre sessões de IA:

- Atualizar `docs/PROGRESS.md` ao concluir uma etapa.
- Registrar decisões importantes nos ADRs.
- Manter tarefas pequenas e específicas.
- Referenciar arquivos e módulos envolvidos.
- Evitar depender de instruções presentes apenas no chat.
- Nunca assumir que outro agente conhece decisões não documentadas.

O repositório deve ser a fonte de verdade do projeto.

### 3.8 Regras contra código de baixa qualidade

Evitar:

- Arquivos gigantes com responsabilidades diferentes.
- Duplicação desnecessária de lógica.
- Workarounds sem justificativa.
- `unsafe` sem documentação.
- `unwrap()` indiscriminado.
- Estados globais mutáveis.
- Dependências adicionadas sem necessidade.
- Interfaces públicas desnecessariamente complexas.
- Código morto.
- Funções ou módulos falsos.
- Comentários que descrevem um comportamento diferente do código.
- Testes que não exercitam a funcionalidade real.
- Reescrever componentes funcionando por preferência estética do agente.

Não esconder falhas de compilação ou erros importantes.

---

## 4. Stack tecnológica

### Backend e aplicação

- Rust stable.
- Cargo workspace.
- GTK 4 via gtk4-rs.
- GLib, GIO e GDK.
- Tokio para tarefas assíncronas apropriadas.
- SQLite para persistência.
- Serde para serialização.
- tracing para diagnóstico.
- Secret Service/libsecret para credenciais.

### Multimídia

Utilizar FFmpeg através de bibliotecas nativas e bindings Rust apropriados:

- libavformat.
- libavcodec.
- libavutil.
- libswscale.
- libswresample.

Suportar:

- H.264.
- H.265/HEVC.
- MJPEG.
- JPEG estático.
- HLS compatível.
- Áudio de streams suportados pelo FFmpeg.
- Decodificação por hardware quando disponível.
- Fallback para software.

Não utilizar Electron, Chromium, React ou um frontend web.

Não utilizar uma coleção de subprocessos FFmpeg como implementação principal do player.

### Integração visual com Zorin OS

A aplicação deve utilizar widgets GTK nativos e respeitar o tema instalado no computador.

Requisitos:

- Seguir preferências de tema claro/escuro.
- Respeitar cores, tipografia e ícones.
- Suportar Wayland e X11.
- Suportar HiDPI.
- Evitar estilos globais que substituam a aparência do tema GTK.
- Não impor libadwaita como tema obrigatório.
- Utilizar CSS GTK somente para componentes específicos.
- Respeitar acessibilidade e preferências de movimento reduzido quando aplicável.

---

## 5. Funcionalidades obrigatórias

### 5.1 Cadastro de câmeras

Implementar uma biblioteca de câmeras com operações completas de cadastro, edição, exclusão e consulta.

Cada câmera deve possuir:

- Nome.
- Identificador interno.
- Endereço.
- Porta.
- Protocolo.
- URL de streaming.
- Credenciais protegidas.
- Fabricante e modelo, se identificáveis.
- Perfis de mídia.
- Preferência de RTSP TCP/UDP.
- Configurações de timeout.
- Configurações de áudio.
- Capacidades ONVIF.
- Permissão de controle PTZ.
- Status da última conexão.

Protocolos iniciais obrigatórios:

- HTTP.
- HTTPS.
- RTSP.
- ONVIF para descoberta, configuração e controle.

Criar interface de teste de conexão com mensagens claras.

### 5.2 Descoberta ONVIF

Implementar:

- Descoberta de dispositivos na rede com WS-Discovery.
- Cadastro manual ONVIF.
- Autenticação.
- Consulta de informações do dispositivo.
- Identificação de serviços disponíveis.
- Consulta de perfis de mídia.
- Seleção de streams.
- Obtenção de URLs RTSP.
- Consulta de capacidades PTZ.
- Identificação de recursos não suportados.

Não presumir compatibilidade universal entre fabricantes.

Tratar ONVIF como serviço de descoberta e controle integrado ao mecanismo de vídeo, e não como um formato de vídeo independente.

### 5.3 Dashboard principal

Criar interface de monitoramento com:

- Headerbar GTK.
- Sidebar recolhível.
- Seletor de Views.
- Área de exibição de câmeras.
- Acesso ao cadastro de câmeras.
- Acesso ao editor de Views.
- Configurações.
- Indicadores de conexão.
- Estados vazios bem projetados.

A última View utilizada deve poder ser restaurada na inicialização.

### 5.4 Sistema de Views

Uma View representa um layout de monitoramento de uma ou várias câmeras.

Permitir:

- Criar.
- Editar.
- Renomear.
- Duplicar.
- Excluir.
- Reordenar.
- Marcar como favorita.
- Definir como inicial.
- Adicionar e remover câmeras.

Uma câmera pode aparecer em diversas Views sem duplicação de cadastro.

### 5.5 Editor visual de Views

Desenvolver um verdadeiro editor de dashboards.

Funcionalidades:

- Arrastar e soltar câmeras.
- Reposicionar painéis.
- Redimensionar painéis.
- Trocar a câmera exibida.
- Ajustar proporção.
- Organizar layouts personalizados.
- Salvar e restaurar configurações.
- Cancelar alterações.
- Visualizar o resultado antes de salvar.

Layouts predefinidos:

- 1 câmera.
- 2 câmeras.
- 2×2.
- 3×3.
- 4×4.
- Câmera principal com secundárias.
- Layout personalizado.

Utilizar um sistema de grade flexível com snap-to-grid, permitindo que um painel ocupe várias células.

Separar Modo Edição e Modo Visualização.

### 5.6 Player de câmera

O player deve ser um componente reutilizável e eficiente.

Implementar:

- Reprodução de vídeo real.
- Estados de conexão.
- Reconexão automática.
- Controles contextuais.
- Exibição de nome e status.
- Manutenção de proporção de imagem.
- Tratamento de perda de sinal.
- Indicadores de erro.
- Gerenciamento de sessões.

Nunca bloquear a thread principal GTK com operações de rede, decodificação ou áudio.

### 5.7 Modo Foco

Ao dar duplo clique em uma câmera:

- Maximizar o painel dentro do aplicativo.
- Ocultar temporariamente os outros painéis.
- Exibir controles adicionais.
- Permitir tela cheia.
- Permitir zoom e pan.
- Permitir áudio.
- Exibir PTZ quando disponível e habilitado.
- Retornar à View anterior com Escape.

Não perder o layout original ao entrar ou sair do Modo Foco.

### 5.8 Zoom digital

Implementar:

- Zoom pela roda do mouse.
- Botões de zoom.
- Pan por arrasto.
- Restauração do zoom.
- Limites configurados.

O zoom digital deve ser independente do zoom óptico PTZ.

### 5.9 Áudio

Implementar:

- Reprodução de áudio de câmeras compatíveis.
- Mute e unmute.
- Controle de volume.
- Indicador de áudio.
- Sincronização audiovisual quando viável.
- Política segura para múltiplas câmeras.

Por padrão, permitir apenas uma câmera com áudio ativo de cada vez, com possibilidade de evolução da política.

### 5.10 PTZ

O controle PTZ faz parte do escopo obrigatório.

Implementar através de ONVIF, de acordo com os recursos suportados pelo dispositivo.

Funcionalidades:

- Cima.
- Baixo.
- Esquerda.
- Direita.
- Movimentos diagonais suportados.
- Parada imediata de movimento.
- Zoom óptico.
- Velocidade de movimento.
- Consulta e acionamento de presets.
- Criação de presets quando suportada.
- Controles por interface e atalhos adequados.

**Regra:** PTZ somente deve aparecer quando a câmera indicar compatibilidade e o usuário habilitar esse recurso.

Implementar parada segura de movimento ao soltar controles, perder foco ou ocorrer falha.

---

## 6. Arquitetura de desempenho

O aplicativo deve suportar múltiplos streams simultâneos sem bloquear a interface.

Separar claramente:

1. Conexões de rede.
2. Demultiplexação.
3. Decodificação.
4. Buffering.
5. Renderização.
6. Áudio.
7. Eventos de câmera.
8. Controle ONVIF/PTZ.

### Gerenciador de sessões

Implementar gerenciamento centralizado para:

- Criar sessões.
- Reutilizar sessões compatíveis.
- Compartilhar frames entre consumidores quando aplicável.
- Encerrar sessões não utilizadas.
- Gerenciar reconexões.
- Controlar recursos.
- Evitar duplicação desnecessária de conexões.

### Pipeline multimídia

Priorizar:

- Baixa latência.
- Descarte de frames obsoletos.
- Filas limitadas.
- Backpressure.
- Uso eficiente de CPU e memória.
- Aceleração por hardware.
- Recuperação de falhas.
- Shutdown confiável.

### Renderização GTK

Usar um mecanismo compatível com GTK 4, como GDK Paintable/texturas ou outra abordagem tecnicamente adequada.

Documentar a estratégia escolhida em `docs/FFMPEG_PIPELINE.md`.

Minimizar cópias de frames e conversões de formatos, sobretudo em cenários multistream.

### Perfis de vídeo

Quando a câmera disponibilizar múltiplos perfis:

- Preferir substream em painéis pequenos.
- Preferir stream principal no Modo Foco.
- Permitir escolha manual.
- Evitar alternâncias excessivas.
- Permitir ajustar o comportamento por câmera.

### Metas de testes

Validar configurações de referência como:

- 4 câmeras 1080p.
- 9 câmeras 720p.
- 16 painéis em mosaico.

Não prometer quantidade fixa de streams em qualquer hardware.

Criar métricas de CPU, memória, uso dos decodificadores, perda de frames e latência para apoiar otimizações.

---

## 7. Interface e experiência do usuário

Desenvolver uma interface moderna, fluida, organizada e visualmente integrada ao Zorin OS.

### Princípios

- GTK nativo.
- Navegação intuitiva.
- Hierarquia visual clara.
- Bom uso de espaçamento.
- Controles discretos.
- Animações leves, quando apropriadas.
- Estados de carregamento.
- Mensagens úteis.
- Menus contextuais.
- Ícones simbólicos.
- Drag and drop funcional.
- Suporte a teclado.
- Suporte a temas claro e escuro.

### Persistência de experiência

Salvar:

- Última View.
- Tamanho e estado da janela.
- Preferências do usuário.
- Layouts personalizados.
- Configurações das câmeras.
- Preferências de streaming.
- Preferências de áudio e PTZ.

Não armazenar senhas diretamente no banco de dados.

---

## 8. Segurança

Implementar segurança desde a arquitetura inicial.

Requisitos:

- Credenciais no Secret Service/libsecret.
- Proteção de dados sensíveis nos logs.
- Validação TLS por padrão.
- Configurações excepcionais de TLS somente com consentimento explícito.
- Controle de timeout de rede.
- Validação de dados externos.
- Tratamento de dispositivos malformados.
- Proteção contra comandos PTZ indevidos.
- Nenhuma execução como root.
- Nenhuma abertura desnecessária de serviços locais.
- Nenhuma telemetria não autorizada.

Documentar o modelo de segurança em `docs/SECURITY.md`.

---

## 9. Estrutura do repositório

Utilizar a seguinte estrutura inicial, adaptando-a apenas quando houver justificativa:

```text
cctv-view/
├── AGENTS.md
├── agent.md
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .gitignore
├── .github/
│   ├── workflows/
│   │   └── ci.yml
│   └── PULL_REQUEST_TEMPLATE.md
├── crates/
│   ├── app/
│   ├── core/
│   ├── media/
│   ├── onvif/
│   ├── storage/
│   └── ui/
├── resources/
│   ├── icons/
│   ├── styles/
│   └── ui/
├── packaging/
│   ├── appimage/
│   └── desktop/
├── scripts/
│   ├── dev.sh
│   ├── check.sh
│   ├── build.sh
│   └── build-appimage.sh
├── docs/
│   ├── README.md
│   ├── ARCHITECTURE.md
│   ├── DEVELOPMENT.md
│   ├── BUILD.md
│   ├── APPIMAGE.md
│   ├── PLAN.md
│   ├── TASKS.md
│   ├── PROGRESS.md
│   ├── FEATURES.md
│   ├── DATABASE.md
│   ├── SECURITY.md
│   ├── TESTING.md
│   ├── TROUBLESHOOTING.md
│   ├── UI_GUIDELINES.md
│   ├── CAMERA_PROTOCOLS.md
│   ├── FFMPEG_PIPELINE.md
│   ├── ONVIF.md
│   ├── PTZ.md
│   └── adr/
└── tests/
```

Separar corretamente as responsabilidades e evitar dependências circulares entre módulos.

---

## 10. Persistência e dados

Utilizar SQLite com migrações versionadas.

Seguir XDG Base Directory Specification.

Diretórios sugeridos:

- Dados: `~/.local/share/br.dev.schuh.cctv-view/`
- Configurações: `~/.config/br.dev.schuh.cctv-view/`
- Cache: diretório XDG de cache correspondente.

Não armazenar configurações dentro do AppImage.

O banco de dados deve preservar:

- Câmeras.
- Views.
- Layouts.
- Configurações.
- Preferências.
- Metadados de capacidades.

Guardar apenas referências a segredos no banco de dados, deixando as credenciais no chaveiro do sistema.

---

## 11. Ferramentas de qualidade

Configurar:

- rustfmt.
- Clippy.
- Testes unitários.
- Testes de integração.
- Testes de regressão.
- GitHub Actions ou CI equivalente.
- Verificação de dependências e vulnerabilidades.
- Build release.
- Validação AppImage.

Criar o script `scripts/check.sh` para centralizar verificações reproduzíveis.

Comandos mínimos:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

Utilizar ferramentas adicionais, como cargo-audit e cargo-deny, quando forem apropriadas e estiverem disponíveis.

### Testes com câmeras

Criar infraestrutura de testes usando streams locais e fixtures reprodutíveis.

Testar:

- RTSP funcional.
- RTSP indisponível.
- Credenciais incorretas.
- HTTP/MJPEG.
- HTTPS.
- Reconexão.
- Troca de Views.
- Múltiplos streams.
- Descarte de frames.
- Encerramento de sessões.
- Áudio.
- ONVIF.
- Comandos PTZ.

Para ONVIF e PTZ, utilizar mocks ou simuladores nos testes automatizados e validar em hardware real quando disponível.

Nunca declarar compatibilidade real com um dispositivo não testado.

---

## 12. Git e fluxo de trabalho

Utilizar Git desde a primeira implementação.

### Convenções

- Commits pequenos e objetivos.
- Mensagens descritivas.
- Separação entre refatorações e novas funcionalidades quando possível.
- Não incluir arquivos de build desnecessários no repositório.
- Não incluir credenciais.
- Não alterar arquivos não relacionados sem necessidade.

Exemplos de commits:

```text
feat(media): add RTSP session manager
feat(onvif): implement device discovery
feat(ui): add customizable view editor
fix(media): handle interrupted streams
refactor(core): separate camera state management
docs(architecture): document rendering pipeline
test(ptz): cover continuous movement stop
```

### Pull requests ou revisões

Toda alteração relevante deve ser revisada quanto a:

- Correção funcional.
- Segurança.
- Impacto de desempenho.
- Concorrência.
- Tratamento de erros.
- Testes.
- Documentação.
- Compatibilidade com o tema GTK.

O agente deve revisar seu próprio diff antes de finalizar uma tarefa.

Não realizar commits automaticamente quando a ferramenta ou o usuário não tiver autorizado essa ação.

---

## 13. AppImage

Produzir um AppImage funcional para Linux x86_64.

Nome de saída:

`CCTV-View-x86_64.AppImage`

Requisitos:

- Build Rust release.
- Arquivo `.desktop`.
- Ícones.
- Metadados.
- Bibliotecas necessárias.
- Recursos GTK.
- Dependências FFmpeg compatíveis.
- Funcionamento sem instalação.
- Execução sem root.
- Integração visual com o sistema sempre que tecnicamente possível.
- Scripts reproduzíveis de build.

Utilizar ferramentas adequadas como linuxdeploy e appimagetool.

Testar em ambiente compatível separado da máquina de desenvolvimento.

Documentar requisitos mínimos de sistema, dependências não empacotadas, limitações de drivers e licenciamento das bibliotecas utilizadas.

---

## 14. Plano de implementação

Organizar o trabalho em entregas incrementais, mantendo todos os recursos centrais no objetivo da primeira versão completa.

### Etapa 1 — Fundação

- Workspace Rust.
- GTK 4 funcional.
- Tema nativo.
- Navegação.
- Banco SQLite.
- Estrutura de módulos.
- AGENTS.md.
- Documentação inicial.
- CI e scripts de validação.

### Etapa 2 — Player real

- Integração FFmpeg.
- RTSP.
- HTTP/HTTPS.
- H.264/H.265.
- Reprodução GTK.
- Reconexão.
- Cadastro de câmeras.
- Testes de conexão.

### Etapa 3 — Views e dashboard

- Dashboard principal.
- Views persistentes.
- Múltiplas câmeras.
- Layouts predefinidos.
- Modo Foco.
- Gerenciamento de sessões.

### Etapa 4 — Editor de layouts

- Drag and drop.
- Redimensionamento.
- Reorganização.
- Layouts personalizados.
- Salvamento e restauração.
- Responsividade.

### Etapa 5 — ONVIF, PTZ e áudio

- Descoberta ONVIF.
- Perfis e capacidades.
- Integração com streams.
- PTZ completo conforme compatibilidade.
- Presets.
- Zoom digital.
- Reprodução de áudio.
- Controles integrados ao player.

### Etapa 6 — Integração e estabilização

- Aceleração por hardware.
- Otimização multistream.
- Testes de desempenho.
- Correção de falhas.
- Revisão de segurança.
- Validação de todos os requisitos.
- AppImage.
- Documentação final.

**Importante:** as etapas representam ordem de implementação, não autorização para eliminar requisitos. ONVIF, PTZ, áudio e personalização de Views devem ser entregues e validados antes de considerar a primeira versão completa.

---

## 15. Fluxo obrigatório ao executar tarefas

Para qualquer solicitação de desenvolvimento, adote o seguinte processo:

### PASSO 1 — Investigar

Leia as instruções do repositório e os documentos pertinentes.

Inspecione a implementação existente.

Não presuma que um módulo está ausente sem verificar.

### PASSO 2 — Planejar

Crie um plano curto e verificável.

Identifique:

- Objetivo.
- Arquivos afetados.
- Dependências.
- Riscos.
- Testes.
- Critérios de conclusão.

### PASSO 3 — Implementar

Escreva código real, compilável e integrado.

Priorize uma implementação funcional de ponta a ponta em vez de gerar muitos módulos incompletos.

### PASSO 4 — Validar

Execute formatadores, análise estática, compilação e testes relevantes.

Corrija falhas introduzidas.

Caso não consiga validar algo, documente a limitação sem inventar resultados.

### PASSO 5 — Revisar

Revise as alterações procurando:

- Código duplicado.
- Vazamentos de recursos.
- Race conditions.
- Bloqueios GTK.
- Falhas de segurança.
- Erros de integração.
- Problemas de manutenção.

### PASSO 6 — Documentar

Atualize os arquivos pertinentes:

- TASKS.md.
- PROGRESS.md.
- Documentação técnica.
- ADRs, quando necessário.
- CHANGELOG.md para alterações relevantes ao usuário.

### PASSO 7 — Reportar

Ao concluir, apresentar:

- O que foi implementado.
- Arquivos modificados.
- Resultados dos testes.
- Limitações.
- Pendências.
- Próximas tarefas recomendadas.

Não utilizar mensagens vagas como "implementado com sucesso" sem apresentar evidências verificáveis.

---

## 16. Critérios de aceitação da primeira versão completa

O CCTV View somente poderá ser considerado funcionalmente completo quando:

- [ ] Iniciar como aplicativo GTK 4 nativo.
- [ ] Respeitar o tema do Zorin OS dentro das possibilidades do GTK e do empacotamento.
- [ ] Permitir cadastro, edição e remoção de câmeras.
- [ ] Reproduzir RTSP.
- [ ] Reproduzir os formatos HTTP/HTTPS especificados.
- [ ] Realizar descoberta ONVIF.
- [ ] Obter perfis e URLs de mídia ONVIF.
- [ ] Exibir múltiplos streams simultaneamente.
- [ ] Permitir criar e gerenciar Views.
- [ ] Permitir editar layouts visualmente.
- [ ] Persistir layouts personalizados.
- [ ] Suportar Modo Foco.
- [ ] Suportar zoom digital.
- [ ] Suportar áudio.
- [ ] Suportar controles PTZ em câmeras compatíveis.
- [ ] Suportar presets PTZ quando disponíveis.
- [ ] Reconectar streams após falhas.
- [ ] Não bloquear a interface durante tarefas de mídia.
- [ ] Gerenciar corretamente memória e recursos.
- [ ] Utilizar armazenamento seguro de credenciais.
- [ ] Possuir testes automatizados significativos.
- [ ] Possuir documentação atualizada.
- [ ] Possuir AGENTS.md e planejamento persistente.
- [ ] Compilar em release.
- [ ] Gerar AppImage funcional.
- [ ] Executar o AppImage em ambiente compatível de teste.

Nenhuma caixa deve ser marcada apenas porque o código da funcionalidade foi criado. A validação correspondente precisa ter sido realizada.

---

## 17. Instrução de início

Inicie o trabalho pelo estado atual do repositório.

Se o repositório estiver vazio:

1. Crie o workspace Rust.
2. Configure a aplicação GTK 4.
3. Crie `AGENTS.md` e o arquivo de compatibilidade `agent.md`.
4. Estabeleça as convenções arquiteturais.
5. Crie a documentação fundamental.
6. Configure SQLite e logging.
7. Crie a navegação inicial.
8. Prepare a arquitetura multimídia.
9. Configure scripts e testes.
10. Compile e valide a fundação.

Depois continue implementando os requisitos do produto em entregas verificáveis até completar o escopo.

Se o projeto já existir, preserve as partes funcionais e evolua sobre elas.

**Não entregue somente planos ou documentação. O objetivo é desenvolver efetivamente o aplicativo.**

Mantenha o projeto compilável, a documentação sincronizada e a implementação alinhada à arquitetura definida.

O resultado final esperado é o **CCTV View**, um software Linux nativo de monitoramento de câmeras IP, com reprodução fluida, ONVIF, PTZ, áudio, zoom, dashboard customizável, editor de Views, integração com o tema GTK e distribuição AppImage.
