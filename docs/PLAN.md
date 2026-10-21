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
