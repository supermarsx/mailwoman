# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Conteúdo remoto

remote-change-failed = Não foi possível atualizar as permissões de imagens remotas. { $reason }

# -- Single-line banner (t55) ---------------------------------------------------
remote-banner-blocked = Imagens remotas bloqueadas
remote-banner-blocked-policy = Imagens remotas bloqueadas pelas suas definições de privacidade
remote-banner-shown = Imagens apresentadas
remote-banner-proxied = Imagens carregadas através do servidor
remote-banner-direct = Imagens carregadas a partir dos servidores do remetente
remote-banner-trackers = { $count ->
    [one] { $count } rastreador removido
   *[other] { $count } rastreadores removidos
}
remote-banner-trackers-limited = Pelo menos { $count } rastreadores removidos
remote-banner-show = Mostrar imagens
remote-banner-show-help = Carregar as imagens desta mensagem através do servidor, para que o remetente não veja o seu endereço IP
remote-banner-more = Mais opções de imagens
remote-banner-always-sender = Mostrar sempre imagens de { $sender }
remote-banner-always-domain = Mostrar sempre imagens de { $domain }
remote-banner-always-all = Mostrar sempre imagens em todo o correio
remote-banner-settings = Definições de privacidade
remote-banner-undo = Anular
remote-banner-undo-help-single = Voltar a bloquear as imagens desta mensagem
remote-banner-undo-help-per-sender = Deixar de mostrar automaticamente imagens de { $sender }
remote-banner-undo-help-per-domain = Deixar de mostrar automaticamente imagens de { $domain }
remote-banner-undo-help-all = Deixar de mostrar automaticamente imagens em todo o correio
remote-banner-blocked-help = As imagens guardadas noutros servidores podem indicar ao remetente quando e onde abriu esta mensagem. Ficam bloqueadas até optar por mostrá-las.
remote-banner-help-block = As suas definições de privacidade bloqueiam todas as imagens remotas. Pode alterar isto nas definições de privacidade.
remote-banner-help-ask = As imagens são carregadas através deste servidor, que oculta o seu endereço IP ao remetente.
remote-banner-help-proxy = As imagens são carregadas através deste servidor, que oculta o seu endereço IP ao remetente.
remote-banner-help-direct = As imagens são carregadas diretamente dos servidores do remetente, que podem ver o seu endereço IP.
remote-banner-hosts = Servidores bloqueados: { $hosts }
remote-banner-trackers-help = As imagens de rastreio são removidas antes de a mensagem ser apresentada. As imagens que o servidor só reconhece como rastreadores depois de as transferir são contadas durante um período limitado.

# -- Loading-images toast (t55) ---------------------------------------------------
remote-images-toast-loading = A carregar imagens…
remote-images-toast-done = Imagens carregadas
remote-images-toast-failed = { $count ->
    [one] Não foi possível carregar { $count } imagem
   *[other] Não foi possível carregar { $count } imagens
}
remote-images-toast-retry = Tentar novamente
