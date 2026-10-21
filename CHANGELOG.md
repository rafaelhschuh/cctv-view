# Changelog

## 0.1.0 — 2026-10-21

- Workspace Rust com aplicação GTK nativa, biblioteca de câmeras e armazenamento SQLite versionado.
- Parser XML 0.42 com limites e respostas SOAP verificadas; auditoria RustSec sem vulnerabilidades conhecidas no lockfile.
- Credenciais em Secret Service; validação de URLs e TLS por padrão.
- Player libav em workers, sessões compartilhadas, último frame limitado, reconexão e fallback de VA-API para software.
- Áudio via libswresample/PulseAudio, volume e seleção exclusiva de câmera.
- Views, favoritos, ordem, inicialização persistente, foco, zoom e editor de grade com DnD e redimensionamento.
- ONVIF WS-Discovery/Media1, autenticação UsernameToken digest, controle PTZ e presets.
- Métricas locais de sessões, CPU, memória, frames descartados e áudio entregue.
- Simuladores e testes nativos, CI, release e empacotamento AppImage.

Ainda requer validação física de câmeras, drivers, acessibilidade com leitor de tela, Wayland/HiDPI e distribuição em um sistema separado. Não representa certificação ONVIF nem garantia de desempenho por quantidade de câmeras.
