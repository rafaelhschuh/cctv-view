# ADR 0001 — GTK4, serviços Rust e FFI nativa pequena

Status: adotado em 2026-10-21.

## Contexto
É necessário um aplicativo GTK nativo com FFmpeg real, áudio, ONVIF e armazenamento seguro, sem player baseado em processos ou frontend web. O ambiente é Zorin 18/Ubuntu 24.04, FFmpeg 6.1 e GTK 4.14.

## Alternativas
Bindings ffmpeg-next gerados; bindings diretos de grandes estruturas libav; GStreamer; subprocessos FFmpeg; um shim C pequeno com fronteira de dados explícita. Para segredos, D-Bus manual ou libsecret. Para render, Cairo, memória BGRA em GDK ou integração GPU zero-copy.

## Decisão e justificativa
Workspace separa core/storage/media/onvif/ui/app. Usar cc/pkg-config para shims pequenos que mantêm estruturas libav e libsecret no lado nativo. Rust controla modelos, sessões e callbacks documentados. Essa fronteira evita depender da representação de structs libav em Rust e aproveita os headers do sistema. O player não cria subprocessos.

GDK MemoryTexture BGRA com snapshot para proporção/zoom/pan foi escolhido por compatibilidade GTK/X11/Wayland. Rede/decode/áudio/chaveiro/SQLite ficam em workers. ONVIF usa Tokio/reqwest e parser XML limitado. SQLite mantém metadados/Views; segredos somente no Secret Service.

## Consequências e limitações
Há C além de Rust, explicitamente limitado às integrações nativas; mudanças exigem revisão de memória e ABI. Requer toolchain C e headers no build. A cópia CPU/BGRA não é zero-copy e limita eficiência em cargas altas. VA-API é a única aceleração explícita inicial. A/V tem pacing básico e exige validação física. A versão 1.0 continua condicionada à matriz de aceitação, incluindo hardware e ambiente AppImage separado. Uma mudança nesses contratos requer novo ADR.
