# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Conteúdo remoto

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Não foi possível atualizar as permissões de imagens remotas. { $reason }
remote-banner-blocked = Imagens remotas bloqueadas
remote-banner-blocked-policy = Imagens remotas bloqueadas pelas suas configurações de privacidade
remote-banner-shown = Imagens exibidas
remote-banner-proxied = Imagens carregadas pelo servidor
remote-banner-direct = Imagens carregadas dos servidores do remetente
remote-banner-trackers = { $count ->
    [one] { $count } rastreador removido
   *[other] { $count } rastreadores removidos
}
remote-banner-trackers-limited = Pelo menos { $count } rastreadores removidos
remote-banner-show = Mostrar imagens
remote-banner-show-help = Carregar as imagens desta mensagem pelo servidor, para que o remetente não veja seu endereço IP
remote-banner-more = Mais opções de imagens
remote-banner-always-sender = Sempre mostrar imagens de { $sender }
remote-banner-always-domain = Sempre mostrar imagens de { $domain }
remote-banner-always-all = Sempre mostrar imagens em todos os e-mails
remote-banner-settings = Configurações de privacidade
remote-banner-undo = Desfazer
remote-banner-undo-help-single = Bloquear novamente as imagens desta mensagem
remote-banner-undo-help-per-sender = Parar de mostrar automaticamente as imagens de { $sender }
remote-banner-undo-help-per-domain = Parar de mostrar automaticamente as imagens de { $domain }
remote-banner-undo-help-all = Parar de mostrar automaticamente as imagens em todos os e-mails
remote-banner-blocked-help = Imagens armazenadas em outros servidores podem informar ao remetente quando e onde você abriu esta mensagem. Elas permanecem bloqueadas até você optar por mostrá-las.
remote-banner-help-block = Suas configurações de privacidade bloqueiam todas as imagens remotas. Altere isso nas configurações de Privacidade.
remote-banner-help-ask = As imagens são carregadas por este servidor, que oculta seu endereço IP do remetente.
remote-banner-help-proxy = As imagens são carregadas por este servidor, que oculta seu endereço IP do remetente.
remote-banner-help-direct = As imagens são carregadas diretamente dos servidores do remetente, que podem ver seu endereço IP.
remote-banner-hosts = Hosts bloqueados: { $hosts }
remote-banner-trackers-help = As imagens de rastreamento são removidas antes de a mensagem ser exibida. As imagens que o servidor só reconhece como rastreadores depois de baixá-las são contadas por tempo limitado.
remote-images-toast-loading = Carregando imagens…
remote-images-toast-done = Imagens carregadas
remote-images-toast-failed = { $count ->
    [one] { $count } imagem não pôde ser carregada
   *[other] { $count } imagens não puderam ser carregadas
}
remote-images-toast-retry = Tentar novamente
remote-images-toast-progress = Carregando imagens… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Carregando imagens… { $loaded }/{ $total } · Tentativa { $attempt }/{ $max }
remote-images-toast-cancel = Cancelar
remote-images-toast-cancelled = Carregamento de imagens cancelado
remote-images-toast-load-again = Carregar novamente
remote-banner-hosts-manage = Origens das imagens…
image-hosts-title = Imagens por origem
image-hosts-help = Escolha o que acontece com as imagens de cada origem. Nunca mostrar prevalece sobre Sempre mostrar e vale para todas as mensagens.
image-hosts-none = Esta mensagem não tem imagens remotas.
image-hosts-for = Imagens de { $host }
image-hosts-allow = Sempre mostrar
image-hosts-ask = Perguntar
image-hosts-block = Nunca mostrar
image-hosts-failed = Não foi possível salvar a configuração da origem das imagens.
image-tip-blocked = Imagem bloqueada. Escolha Mostrar imagens para carregá-la.
image-tip-loading = Carregando imagem…
image-tip-error = Imagem indisponível
