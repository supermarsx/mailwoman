# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Externe Inhalte

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Die Berechtigungen für externe Bilder konnten nicht aktualisiert werden. { $reason }
remote-banner-blocked = Externe Bilder blockiert
remote-banner-blocked-policy = Externe Bilder durch Ihre Datenschutzeinstellungen blockiert
remote-banner-shown = Bilder angezeigt
remote-banner-proxied = Bilder über den Server geladen
remote-banner-direct = Bilder von den Servern des Absenders geladen
remote-banner-trackers = { $count ->
    [one] { $count } Tracker entfernt
   *[other] { $count } Tracker entfernt
    }
remote-banner-trackers-limited = Mindestens { $count } Tracker entfernt
remote-banner-show = Bilder anzeigen
remote-banner-show-help = Die Bilder dieser Nachricht über den Server laden, sodass der Absender Ihre IP-Adresse nicht sieht
remote-banner-more = Weitere Bildoptionen
remote-banner-always-sender = Bilder von { $sender } immer anzeigen
remote-banner-always-domain = Bilder von { $domain } immer anzeigen
remote-banner-always-all = Bilder in allen E-Mails immer anzeigen
remote-banner-settings = Datenschutzeinstellungen
remote-banner-undo = Rückgängig
remote-banner-undo-help-single = Die Bilder dieser Nachricht wieder blockieren
remote-banner-undo-help-per-sender = Bilder von { $sender } nicht mehr automatisch anzeigen
remote-banner-undo-help-per-domain = Bilder von { $domain } nicht mehr automatisch anzeigen
remote-banner-undo-help-all = Bilder in allen E-Mails nicht mehr automatisch anzeigen
remote-banner-blocked-help = Auf anderen Servern gespeicherte Bilder können dem Absender mitteilen, wann und wo Sie diese Nachricht geöffnet haben. Sie bleiben blockiert, bis Sie sich für die Anzeige entscheiden.
remote-banner-help-block = Ihre Datenschutzeinstellungen blockieren alle externen Bilder. Ändern Sie dies in den Datenschutzeinstellungen.
remote-banner-help-ask = Bilder werden über diesen Server geladen, der Ihre IP-Adresse vor dem Absender verbirgt.
remote-banner-help-proxy = Bilder werden über diesen Server geladen, der Ihre IP-Adresse vor dem Absender verbirgt.
remote-banner-help-direct = Bilder werden direkt von den Servern des Absenders geladen, die Ihre IP-Adresse sehen können.
remote-banner-hosts = Blockierte Hosts: { $hosts }
remote-banner-trackers-help = Tracking-Bilder werden entfernt, bevor die Nachricht angezeigt wird. Bilder, die der Server erst nach dem Herunterladen als Tracker erkennt, werden für begrenzte Zeit mitgezählt.
remote-images-toast-loading = Bilder werden geladen …
remote-images-toast-done = Bilder geladen
remote-images-toast-failed = { $count ->
    [one] { $count } Bild konnte nicht geladen werden
   *[other] { $count } Bilder konnten nicht geladen werden
    }
remote-images-toast-retry = Erneut versuchen
remote-images-toast-progress = Bilder werden geladen … { $loaded }/{ $total }
remote-images-toast-retrying-progress = Bilder werden geladen … { $loaded }/{ $total } · Versuch { $attempt }/{ $max }
remote-images-toast-cancel = Abbrechen
remote-images-toast-cancelled = Laden der Bilder abgebrochen
remote-images-toast-load-again = Erneut laden
remote-banner-hosts-manage = Bildquellen …
image-hosts-title = Bilder nach Quelle
image-hosts-help = Legen Sie fest, was mit Bildern aus jeder Quelle geschieht. „Nie anzeigen“ hat Vorrang vor „Immer anzeigen“ und gilt für jede Nachricht.
image-hosts-none = Diese Nachricht enthält keine externen Bilder.
image-hosts-for = Bilder von { $host }
image-hosts-allow = Immer anzeigen
image-hosts-ask = Nachfragen
image-hosts-block = Nie anzeigen
image-hosts-failed = Die Einstellung für die Bildquelle konnte nicht gespeichert werden.
image-tip-blocked = Bild blockiert. Wählen Sie „Bilder anzeigen“, um es zu laden.
image-tip-loading = Bild wird geladen …
image-tip-error = Bild nicht verfügbar
