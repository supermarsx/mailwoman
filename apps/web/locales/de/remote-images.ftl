# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Externe Inhalte

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } externes Bild blockiert
   *[other] { $count } externe Bilder blockiert
}
remote-blocked-trackers = { $count ->
    [one] { $count } Tracker blockiert
   *[other] { $count } Tracker blockiert
} von { $blocked ->
    [one] { $blocked } externen Bild
   *[other] { $blocked } externen Bildern
}
remote-blocked-hosts = Blockierte Hosts

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Bilder laden
remote-grant-sender = Immer von { $sender } laden
remote-grant-domain = Immer von { $domain } laden
remote-grant-all = Immer alle externen Bilder laden

remote-grant-done-single = Externe Bilder für diese Nachricht geladen.
remote-grant-done-per-sender = Externe Bilder von diesem Absender werden geladen.
remote-grant-done-per-domain = Externe Bilder von dieser Domain werden geladen.
remote-grant-done-all = Externe Bilder werden für alle E-Mails geladen.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Externe Bilder werden für diese Nachricht geladen.
remote-revoke = Ausschalten
remote-revoke-done = Externe Bilder ausgeschaltet. Beim nächsten Öffnen werden sie wieder blockiert.
