# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = Nicht gefunden (404) pro Intervall
admin-history-nf-total = Nicht gefunden (404) in diesem Zeitraum
admin-history-nf-share = { $share } der Anfragen
admin-history-nf-not-tracked-window = 404-Antworten wurden in diesem Zeitraum nicht erfasst.
admin-history-nf-not-tracked-before = 404-Antworten wurden vor { $time } nicht erfasst. Frühere Zeiträume werden als nicht verfügbar angezeigt, nicht als null.
admin-history-nf-paths-title = Häufigste Pfade ohne Treffer
admin-history-nf-paths-help = Bis zu { $limit } Pfade pro Stunde, von { $from } bis { $to }, auf volle Stunden erweitert. Query-Strings werden entfernt, und E-Mail-Adressen, Zahlen, IDs, Token und lange Werte werden durch Platzhalter wie {"{"}token{"}"} ersetzt. Erwartete Fehlzugriffe der App selbst, etwa Anfragen nach Browsersymbolen, werden nicht gezählt.
admin-history-nf-paths-approximate = Es sind mehr unterschiedliche Pfade eingegangen, als gespeichert werden, daher sind manche Zahlen ein Bereich statt eines genauen Werts.
admin-history-nf-paths-empty = Keine Nicht-gefunden-Antworten in diesem Zeitraum.
admin-history-nf-paths-error = Die Pfade ohne Treffer konnten nicht geladen werden. Aktualisieren Sie, um es erneut zu versuchen.
admin-history-nf-paths-method = Methode
admin-history-nf-paths-path = Pfad
admin-history-nf-paths-count = Anzahl
admin-history-nf-paths-range = { $min } bis { $max }
admin-history-nf-paths-client = Von den eigenen Clients der App
admin-history-nf-paths-client-help = Pfade, die die Web-App, ihr Service Worker oder die Desktop- und Mobil-Apps anfordern: Build-Dateien, das Worker-Skript sowie API-, JMAP-, DAV- und Konsolenrouten. Ein Fehlzugriff hier bedeutet meist, dass ein Client veraltet oder eine Route falsch ist.
admin-history-nf-paths-external = Anderer Datenverkehr
admin-history-nf-paths-external-help = Pfade, die die App nie anfordert, etwa von Scannern, die nach PHP-Dateien, Dotfiles oder anderer Software suchen, sowie Links von anderswo.
admin-history-nf-paths-all = Pfade
admin-history-nf-paths-all-help = Dieser Server unterscheidet Client-Pfade nicht von anderem Datenverkehr.
