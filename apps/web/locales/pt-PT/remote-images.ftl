# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: remote-images. Source: ../en/remote-images.ftl

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
remote-blocked-hosts = Anfitriões bloqueados

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Carregar imagens
remote-grant-sender = Carregar sempre de { $sender }
remote-grant-domain = Carregar sempre de { $domain }
remote-grant-all = Carregar sempre todas as imagens remotas

remote-grant-done-single = As imagens remotas estão permitidas para esta mensagem e estão a ser carregadas.
remote-change-failed = Não foi possível atualizar as permissões de imagens remotas. Tente novamente.
remote-grant-done-per-sender = As imagens remotas serão carregadas a partir deste remetente.
remote-grant-done-per-domain = As imagens remotas serão carregadas a partir deste domínio.
remote-grant-done-all = As imagens remotas serão carregadas em todo o correio.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = As imagens remotas estão a ser carregadas para esta mensagem.
remote-revoke = Desativar
remote-revoke-done = Imagens remotas desativadas. Voltam a ser bloqueadas na próxima abertura.
