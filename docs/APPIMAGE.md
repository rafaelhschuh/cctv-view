# AppImage

`scripts/fetch-appimage-tools.sh` baixa linuxdeploy, appimagetool e type2-runtime oficiais e confere os hashes em packaging/appimage/tools.sha256. As ferramentas vêm de releases com tag (ver GitHub Actions abaixo); se um hash não conferir, o script falha. As ferramentas são ignoradas pelo Git.

```sh
scripts/fetch-appimage-tools.sh
scripts/build-appimage.sh
./dist/CCTV-View-x86_64.AppImage
```

Dependências de empacotamento: `desktop-file-utils` e `appstream` (além das ferramentas baixadas).

O script compila release com Cargo.lock, cria um AppDir limpo, coleta dependências com linuxdeploy e adiciona desktop, SVG, metadados, schemas GTK e ícones. AppRun define caminhos somente no processo e executa usr/bin/cctv-view. O AppImage é gerado em arquivo temporário e publicado por renomeação atômica, permitindo reconstruir enquanto a versão anterior está aberta. AppImage e SHA256SUMS ficam em dist/, fora do Git. Os metadados são validados com appstreamcli, tratando somente url-homepage-missing como informação, pois não há homepage pública neste projeto local. O empacotador não repete essa validação. Nenhum outro erro é ignorado.

O runtime é fornecido localmente para impedir downloads implícitos durante a etapa de empacotamento.

Baseline do pacote desta sessão: Zorin 18.1 / Ubuntu 24.04 x86_64, glibc 2.39. Necessita sessão gráfica X11/Wayland compatível, drivers gráficos do host, D-Bus/Secret Service e servidor PulseAudio ou PipeWire-Pulse. As autoridades TLS são do host. Chaveiro, dados e configurações não ficam dentro do pacote. Sem FUSE, usar APPIMAGE_EXTRACT_AND_RUN=1.

Bibliotecas FFmpeg, GTK e dependências têm licenças próprias. O código do aplicativo é GPL-3.0-only; isso não muda as licenças do pacote agregado. linuxdeploy coleta avisos nativos disponíveis em usr/share/doc; o script collect-rust-licenses.py inclui os avisos distribuídos nos crates e um inventário SPDX em usr/share/licenses/cctv-view/RUST-DEPENDENCIES.tsv; verificar o inventário e as obrigações LGPL/GPL, incluindo fontes correspondentes das versões embarcadas, antes de redistribuição pública. O FFmpeg do Ubuntu pode habilitar componentes GPL. Não foi produzido um pacote universal de fontes nesta sessão.

Foi executado smoke do AppImage por extração no display Xvfb, com banco temporário e encerramento automático. Isso valida inicialização no host de build; não substitui teste em outra VM/computador. Execução sem extração/FUSE, Wayland, drivers e sistema separado permanecem NOT VERIFIED. O pacote é uma prévia 0.4.0, não release 1.0 homologada.

## GitHub Actions

O workflow dedicado **AppImage**, em `.github/workflows/appimage.yml`, usa runner `ubuntu-24.04` x86_64. Ele instala as dependências de BUILD.md, confere os hashes das ferramentas, compila com Cargo.lock e executa smoke de CLI sem display e de GTK com Xvfb/D-Bus e banco temporário. As actions do workflow estão fixadas por commit. `contents: read` é suficiente; não requer PAT, secrets personalizados, FUSE nem permissões de escrita no repositório.

Executa em todo push (incluindo tags), pull request e por **Actions → AppImage → Run workflow**. O botão manual depende de o arquivo estar no branch padrão do repositório. Depois da execução, baixe o artifact **CCTV-View-x86_64**, retido por 30 dias. A execução não cria nem publica GitHub Releases.

O artifact contém:

- `CCTV-View-x86_64.AppImage`;
- `SHA256SUMS`, com nomes relativos à pasta extraída (`sha256sum --check SHA256SUMS`);
- `CCTV-View-source.tar.gz`, fontes próprios do commit compilado, incluindo Cargo.lock, scripts e licença;
- `BUILD-INFO.txt`, commit, toolchain e versões dos pacotes nativos no runner.

O arquivo de fontes próprio não inclui um pacote completo das fontes de todas as bibliotecas embarcadas; o inventário e a homologação de redistribuição de R02 continuam separados. As ferramentas são baixadas de releases com tag (linuxdeploy `1-alpha-20251107-1`, appimagetool `1.9.1`, type2-runtime `20251108`) e conferidas pelos SHA256 de `packaging/appimage/tools.sha256`, que coincidem com os digests publicados pelo GitHub. Os builds `continuous` mudam sem aviso e quebravam a conferência; para atualizar, trocar a tag e o hash juntos, sem desativar a verificação.

O CI geral (`ci.yml`) mantém fmt, Clippy, testes de modelo, protocolos, serviços, GTK com chaveiro descartável e auditoria. O empacotamento foi movido ao workflow dedicado para evitar gerar o mesmo AppImage duas vezes por evento. O próprio AppImage inclui LICENSE GPLv3 e COPYRIGHT em `usr/share/licenses/cctv-view/`; metadados AppStream declaram GPL-3.0-only, mantendo CC0-1.0 para os metadados.
