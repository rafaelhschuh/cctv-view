# Build

Ambiente validado: Zorin OS 18.1 / Ubuntu 24.04, Linux x86_64, Rust 1.96, GTK 4.14.5, FFmpeg 6.1.1. A interface usa APIs GTK ≥ 4.8; o pacote local foi ligado às bibliotecas do Zorin 18. Não foi validado em versões anteriores do Zorin.

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config cmake libgtk-4-dev libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libswresample-dev libsecret-1-dev libpulse-dev
cargo build --locked -p cctv-view
cargo build --release --locked -p cctv-view
scripts/dev.sh
```

Instale Rust stable por rustup. rust-toolchain.toml define stable, rustfmt e clippy; Cargo.lock fixa as dependências Rust. Bibliotecas nativas devem corresponder ao ABI usado na compilação. Não remova headers nem altere bindings para contornar dependências ausentes.

Nesta sessão, os pacotes de desenvolvimento foram extraídos em `/tmp/cctv-deps/root`, sem instalação no sistema. O ambiente temporário `/tmp/cctv-env.sh` fornece PKG_CONFIG_PATH, C_INCLUDE_PATH e LIBRARY_PATH. Essa alternativa é local e não faz parte da configuração permanente do repositório.

`cargo check` valida o workspace. `scripts/check.sh` inclui fmt, Clippy, testes e release. A execução dos testes de rede requer sockets loopback permitidos. Para distribuição, consulte APPIMAGE.md.
