# Diagnóstico de problemas

**Build não encontra libav/libsecret/PulseAudio:** instale os pacotes -dev de BUILD.md; verifique pkg-config. A presença do executável ffmpeg não fornece headers. Não use versões de headers diferentes das bibliotecas de runtime.

**Chaveiro bloqueado:** desbloqueie pelo gerenciador de senhas do desktop e repita a operação. Não copie senhas para SQLite. A mensagem impede salvar credenciais quando Secret Service não está disponível.

**Sem vídeo:** use o teste de conexão. Confirme URL e porta na própria URL, rede, credenciais e transporte. Tente TCP em redes com restrições UDP. Para erro após VA-API, desabilite aceleração na câmera; o fallback também é automático. O status mostra tentativa/código libav sem expor URL.

**ONVIF não descobre:** redes/VLANs/firewalls podem bloquear UDP 3702 ou multicast. Informe o endpoint manualmente. Sincronize relógios. Media2 exclusivo e autenticação HTTP Digest ainda não são suportados.

**Sem áudio:** habilite áudio na câmera, entre em foco, ative o botão de som e confira volume nas preferências e no servidor PulseAudio/PipeWire. O diagnóstico informa bytes PCM entregues; zero indica que o servidor não recebeu áudio ou não existe track compatível. A troca de View silencia o áudio.

**PTZ indisponível:** consulte perfis ONVIF, escolha um perfil PTZ e habilite autorização. SOAP Fault pode indicar operação não suportada. Solte/feche para solicitar Stop. Não envie comandos repetidos para contornar falhas físicas.

**AppImage não monta:** use APPIMAGE_EXTRACT_AND_RUN=1. Pacote gerado em Ubuntu 24.04 não foi validado em glibc mais antiga. Não copie drivers GPU de outra máquina. Para bugs, informe versão, protocolo sem segredos, mensagens sanitizadas, passos e métricas; não publique imagens privadas.
