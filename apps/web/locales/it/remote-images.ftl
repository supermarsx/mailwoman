# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Contenuto remoto

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } immagine remota bloccata
   *[other] { $count } immagini remote bloccate
}
remote-blocked-trackers = { $count ->
    [one] { $count } tracker bloccato
   *[other] { $count } tracker bloccati
} su { $blocked ->
    [one] { $blocked } immagine remota
   *[other] { $blocked } immagini remote
}
remote-blocked-hosts = Host bloccati

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Carica immagini
remote-grant-sender = Carica sempre da { $sender }
remote-grant-domain = Carica sempre da { $domain }
remote-grant-all = Carica sempre tutte le immagini remote

remote-grant-done-single = Immagini remote caricate per questo messaggio.
remote-grant-done-per-sender = Le immagini remote verranno caricate da questo mittente.
remote-grant-done-per-domain = Le immagini remote verranno caricate da questo dominio.
remote-grant-done-all = Le immagini remote verranno caricate per tutta la posta.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Le immagini remote vengono caricate per questo messaggio.
remote-revoke = Disattiva
remote-revoke-done = Immagini remote disattivate. Verranno bloccate di nuovo alla prossima apertura.
