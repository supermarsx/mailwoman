# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Contenu distant

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } image distante bloquée
   *[other] { $count } images distantes bloquées
}
remote-blocked-trackers = { $count ->
    [one] { $count } traceur bloqué
   *[other] { $count } traceurs bloqués
} sur { $blocked ->
    [one] { $blocked } image distante
   *[other] { $blocked } images distantes
}
remote-blocked-hosts = Hôtes bloqués

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Charger les images
remote-grant-sender = Toujours charger depuis { $sender }
remote-grant-domain = Toujours charger depuis { $domain }
remote-grant-all = Toujours charger toutes les images distantes

remote-grant-done-single = Images distantes chargées pour ce message.
remote-grant-done-per-sender = Les images distantes de cet expéditeur seront chargées.
remote-grant-done-per-domain = Les images distantes de ce domaine seront chargées.
remote-grant-done-all = Les images distantes seront chargées pour tout le courrier.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Les images distantes sont chargées pour ce message.
remote-revoke = Désactiver
remote-revoke-done = Images distantes désactivées. Elles seront de nouveau bloquées à la prochaine ouverture.
