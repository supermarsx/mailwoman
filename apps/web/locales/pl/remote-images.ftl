# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Zawartość zdalna

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] Zablokowano { $count } zdalny obraz
    [few] Zablokowano { $count } zdalne obrazy
    [many] Zablokowano { $count } zdalnych obrazów
   *[other] Zablokowano { $count } zdalnego obrazu
}
remote-blocked-trackers = { $count ->
    [one] Zablokowano { $count } element śledzący
    [few] Zablokowano { $count } elementy śledzące
    [many] Zablokowano { $count } elementów śledzących
   *[other] Zablokowano { $count } elementu śledzącego
} spośród { $blocked ->
    [one] { $blocked } zdalnego obrazu
    [few] { $blocked } zdalnych obrazów
    [many] { $blocked } zdalnych obrazów
   *[other] { $blocked } zdalnego obrazu
}
remote-blocked-hosts = Zablokowane hosty

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Wczytaj obrazy
remote-grant-sender = Zawsze wczytuj od { $sender }
remote-grant-domain = Zawsze wczytuj z { $domain }
remote-grant-all = Zawsze wczytuj wszystkie zdalne obrazy

remote-grant-done-single = Zdalne obrazy zostały wczytane dla tej wiadomości.
remote-grant-done-per-sender = Zdalne obrazy od tego nadawcy będą wczytywane.
remote-grant-done-per-domain = Zdalne obrazy z tej domeny będą wczytywane.
remote-grant-done-all = Zdalne obrazy będą wczytywane dla całej poczty.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Zdalne obrazy są wczytywane dla tej wiadomości.
remote-revoke = Wyłącz
remote-revoke-done = Zdalne obrazy zostały wyłączone. Przy następnym otwarciu będą ponownie blokowane.
