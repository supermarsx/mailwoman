# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Conteúdo remoto

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } imagem remota bloqueada
   *[other] { $count } imagens remotas bloqueadas
}
remote-blocked-trackers = { $count ->
    [one] { $count } rastreador bloqueado
   *[other] { $count } rastreadores bloqueados
} de { $blocked ->
    [one] { $blocked } imagem remota
   *[other] { $blocked } imagens remotas
}
remote-blocked-hosts = Hosts bloqueados

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Carregar imagens
remote-grant-sender = Sempre carregar de { $sender }
remote-grant-domain = Sempre carregar de { $domain }
remote-grant-all = Sempre carregar todas as imagens remotas

remote-grant-done-single = Imagens remotas carregadas para esta mensagem.
remote-grant-done-per-sender = As imagens remotas deste remetente serão carregadas.
remote-grant-done-per-domain = As imagens remotas deste domínio serão carregadas.
remote-grant-done-all = As imagens remotas serão carregadas em todas as mensagens.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = As imagens remotas estão sendo carregadas para esta mensagem.
remote-revoke = Desativar
remote-revoke-done = Imagens remotas desativadas. Elas voltam a ser bloqueadas na próxima abertura.
