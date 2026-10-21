# Plano persistente
1. Fundação: seis crates, modelos validados, migrações, chaveiro, janela GTK e CI.
2. Mídia: libav nativa em worker, frames limitados, áudio PulseAudio, reconexão e sessões compartilhadas.
3. Dashboard: CRUD, Views persistentes, foco e zoom.
4. Editor: grade com snap, DnD, posição/tamanho, presets de layout, salvar/cancelar.
5. ONVIF: descoberta, autenticação, serviços/perfis, stream, PTZ e presets.
6. Estabilização: testes locais, análise estática, release, pacote AppImage e revisão.
Todas as etapas integram o escopo 1.0. Hardware e AppImage em ambiente separado exigem validação específica. Nenhuma dependência ausente justifica simular uma funcionalidade.

## Revisão de UI/UX (2026-10-21)
Corrigir os 12 achados da revisão independente: endpoint ONVIF e fonte única de URL; cadastro em seções com feedback fixo; editor por clique/teclado, reflow não destrutivo e desfazer; erros locais; navegação/foco/Views adaptáveis; PTZ acessível com parada segura. Validar modelo/persistência, testes GTK reais com janelas pequenas/grandes e regressões de mídia/ONVIF; reconstruir AppImage. Somente commits locais.

Revisão implementada na prévia 0.1.1. Evidências e limitações em UI_REVIEW.md, TESTING.md e PROGRESS.md; homologação H01/H02 continua pendente.

## Ajustes de formulário e conexões (0.1.2)
Compactar feedback vazio, remover mensagem Pronto e alinhar Views. Adicionar tipo de fonte persistido com validação e opções específicas, demuxers MJPEG explícitos e JPEG periódico cancelável. Validar compatibilidade de JSON legado, decodificação nativa, GTK com opções/erro e alinhamento; atualizar AppImage e commits locais.

## Correção de densidade da lista (0.1.3)
Remover a altura fixa e o subtítulo vazio introduzidos em 0.1.2; integrar nome/menu em uma linha GTK selecionável e preservar foco. Validar uma/várias Views com nomes longos e indicadores, temas claro/escuro, seleção/ativação e AppImage; somente commit local.

## Acesso e certificado (0.1.4)
Adicionar Sem credenciais em Conexão e validação de certificado configurável em Avançado. Persistir com defaults compatíveis, aplicar no player/ONVIF/PTZ, preservar segredo sem acessá-lo no modo público, validar HTTPS por padrão e com exceção explícita. Regressões GTK, JSON legado e simulador HTTPS/SOAP; atualizar AppImage e commits locais.

## Dashboard, apresentação, CLI e senha (0.2.0)
- Alternar foco por duplo clique, saída X flutuante e tela cheia somente com streams; corrigir clipping das bordas.
- Editor 12×12 com painéis personalizados, seleção por retângulo, DnD, resize, contraste e presets proporcionais; preservar Views antigas e undo/cancelamento.
- CLI para abrir View/tela cheia/biblioteca específica e listar dados sem GUI.
- Senha do aplicativo confirmada pelo usuário: hash com salt, configuração/alteração/remoção, bloqueio inicial e imediato, CLI protegida e senha via stdin.
- Validar modelos, persistência e autenticação, GTK real/escuro/pequeno, CLI headless/erro e AppImage. Somente commits locais.

Entrega 0.2.0 implementada e verificada localmente; evidências em PROGRESS.md, TESTING.md e UI_REVIEW.md. H01/H02 continuam pendentes.
