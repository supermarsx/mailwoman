# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Externe inhoud

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } externe afbeelding geblokkeerd
   *[other] { $count } externe afbeeldingen geblokkeerd
}
remote-blocked-trackers = { $count ->
    [one] { $count } tracker geblokkeerd
   *[other] { $count } trackers geblokkeerd
} van { $blocked ->
    [one] { $blocked } externe afbeelding
   *[other] { $blocked } externe afbeeldingen
}
remote-blocked-hosts = Geblokkeerde hosts

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Afbeeldingen laden
remote-grant-sender = Altijd laden van { $sender }
remote-grant-domain = Altijd laden van { $domain }
remote-grant-all = Altijd alle externe afbeeldingen laden

remote-grant-done-single = Externe afbeeldingen geladen voor dit bericht.
remote-grant-done-per-sender = Externe afbeeldingen van deze afzender worden voortaan geladen.
remote-grant-done-per-domain = Externe afbeeldingen van dit domein worden voortaan geladen.
remote-grant-done-all = Externe afbeeldingen worden voortaan voor alle mail geladen.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Externe afbeeldingen worden geladen voor dit bericht.
remote-revoke = Uitzetten
remote-revoke-done = Externe afbeeldingen uitgezet. Ze worden weer geblokkeerd wanneer je het bericht opnieuw opent.
