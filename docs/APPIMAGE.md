# AppImage

`scripts/fetch-appimage-tools.sh` baixa linuxdeploy, appimagetool e type2-runtime oficiais e confere os hashes em packaging/appimage/tools.sha256. Assets continuous podem mudar: nesse caso, o script falha; revisar a atualização e os hashes antes de aceitá-los. As ferramentas são ignoradas pelo Git.

```sh
scripts/fetch-appimage-tools.sh
scripts/build-appimage.sh
./dist/CCTV-View-x86_64.AppImage
```

Dependências de empacotamento: `desktop-file-utils` e `appstream` (além das ferramentas baixadas).

O script compila release com Cargo.lock, cria um AppDir limpo, coleta dependências com linuxdeploy e adiciona desktop, SVG, metadados, schemas GTK e ícones. AppRun define caminhos somente no processo e executa usr/bin/cctv-view. O AppImage é gerado em arquivo temporário e publicado por renomeação atômica, permitindo reconstruir enquanto a versão anterior está aberta. AppImage e SHA256SUMS ficam em dist/, fora do Git. Os metadados são validados com appstreamcli, tratando somente url-homepage-missing como informação, pois não há homepage pública neste projeto local. O empacotador não repete essa validação. Nenhum outro erro é ignorado.

O runtime é fornecido localmente para impedir downloads implícitos durante a etapa de empacotamento.

Baseline do pacote desta sessão: Zorin 18.1 / Ubuntu 24.04 x86_64, glibc 2.39. Necessita sessão gráfica X11/Wayland compatível, drivers gráficos do host, D-Bus/Secret Service e servidor PulseAudio ou PipeWire-Pulse. As autoridades TLS são do host. Chaveiro, dados e configurações não ficam dentro do pacote. Sem FUSE, usar APPIMAGE_EXTRACT_AND_RUN=1.

Bibliotecas FFmpeg, GTK e dependências têm licenças próprias. O código do aplicativo é MIT; isso não muda as licenças do pacote agregado. linuxdeploy coleta avisos nativos disponíveis em usr/share/doc; o script collect-rust-licenses.py inclui os avisos distribuídos nos crates e um inventário SPDX em usr/share/licenses/cctv-view/RUST-DEPENDENCIES.tsv; verificar o inventário e as obrigações LGPL/GPL, incluindo fontes correspondentes das versões embarcadas, antes de redistribuição pública. O FFmpeg do Ubuntu pode habilitar componentes GPL. Não foi produzido um pacote universal de fontes nesta sessão.

Foi executado smoke do AppImage por extração no display Xvfb, com banco temporário e encerramento automático. Isso valida inicialização no host de build; não substitui teste em outra VM/computador. Execução sem extração/FUSE, Wayland, drivers e sistema separado permanecem NOT VERIFIED. O pacote é uma prévia 0.1.1, não release 1.0 homologada.
