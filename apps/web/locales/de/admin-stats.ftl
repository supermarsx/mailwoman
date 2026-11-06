# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Kennzahlen
admin-stats-charts = Diagramme
admin-stats-tables = Tabellen
admin-stats-group-traffic = Datenverkehr in diesem Zeitraum
admin-stats-group-storage = Speicher und Zustellung, aktuell
admin-stats-refresh = Statistiken aktualisieren
admin-stats-loading = Statistiken werden geladen…
admin-stats-requests-total = Anfragen
admin-stats-errors-total = Serverfehler (5xx)
admin-stats-p95-peak = Langsamstes 95. Perzentil
admin-stats-requests = Anfragen
admin-stats-errors = Serverfehler (5xx)
admin-stats-not-found = Nicht gefunden (404)
admin-stats-latency = Antwortzeit
admin-stats-series-p50 = Median
admin-stats-series-p95 = 95. Perzentil
admin-stats-unit-count = Anzahl pro { $bucket }
admin-stats-unit-duration = Zeit pro { $bucket }, in der angezeigten Einheit
admin-stats-unit-accounts = Gesperrte Konten pro Domain
admin-stats-chart-about = Liniendiagramm von { $from } bis { $to }.
admin-stats-bars-about = { $count ->
    [one] Balkendiagramm mit einer Domain.
   *[other] Balkendiagramm mit { $count } Domains.
}
admin-stats-view-table = Als Tabelle anzeigen
admin-stats-legend = Legende
admin-stats-empty-chart = In diesem Zeitraum wurde noch nichts gemessen. Wählen Sie einen längeren Zeitraum oder schauen Sie später wieder vorbei, wenn der Server einige Anfragen verarbeitet hat.
admin-stats-empty-blocked = Es sind keine Konten gesperrt.
admin-stats-delta-vs = gegenüber dem vorherigen Zeitraum
admin-stats-delta-same = Wie im vorherigen Zeitraum
admin-stats-delta-sr-up = { $change } höher als im vorherigen Zeitraum.
admin-stats-delta-sr-down = { $change } niedriger als im vorherigen Zeitraum.
admin-stats-delta-sr-same = Gleich wie im vorherigen Zeitraum.
admin-ct-group = Ladezeiten im Client
admin-ct-metric-timeToUsable = Zeit bis zur Nutzbarkeit
admin-ct-metric-ttfb = Erstes Byte
admin-ct-metric-dcl = DOM bereit
admin-ct-metric-load = Seitenladezeit
admin-ct-metric-fcp = Erste inhaltliche Darstellung
admin-ct-metric-lcp = Größte inhaltliche Darstellung
admin-ct-metric-folderOpen = Ordner öffnen
admin-ct-metric-folderLoadCold = Ordner laden, aus dem Netzwerk
admin-ct-metric-folderLoadCached = Ordner laden, aus dem Cache
admin-ct-metric-messageLoadCold = Nachricht laden, aus dem Netzwerk
admin-ct-metric-messageLoadCached = Nachricht laden, aus dem Cache oder per Vorabladen
admin-ct-device-desktop = Desktop
admin-ct-device-mobile = Mobil
admin-ct-device-unknown = Unbekannt
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } Stichprobe
       *[other] { $count } Stichproben
    }
admin-ct-no-samples = Keine Stichproben in diesem Zeitraum.
admin-ct-few-samples = Nur { $count } Stichproben; mindestens { $min } sind erforderlich.
admin-ct-chart-usable = Zeit bis zur Nutzbarkeit
admin-ct-group-folder = Ladezeiten von Ordnern
admin-ct-group-message = Ladezeiten von Nachrichten
admin-ct-chart-folder-cold = Ordner aus dem Netzwerk laden
admin-ct-chart-folder-cached = Ordner aus dem Cache laden
admin-ct-chart-message-cold = Nachricht aus dem Netzwerk laden
admin-ct-chart-message-cached = Nachricht aus dem Cache oder per Vorabladen laden
admin-ct-folder-off = Ladezeiten von Ordnern sind standardmäßig ausgeschaltet. Gemessen werden nur Personen, die sie eingeschaltet haben.
admin-ct-message-off = Ladezeiten von Nachrichten sind standardmäßig ausgeschaltet. Gemessen werden nur Personen, die sie eingeschaltet haben.
admin-ct-chart-note = Perzentile werden nur für Intervalle mit mindestens { $min } Stichproben angezeigt.
admin-ct-empty-chart = Kein Intervall hat die { $min } Stichproben, die für die Anzeige eines Medians nötig sind. Wählen Sie einen längeren Zeitraum.
admin-ct-col-samples = Stichproben
admin-ct-col-metric = Messgröße
admin-ct-col-device = Gerät
admin-ct-table-title = Ladezeiten im Client nach Gerät
admin-ct-table-caption = Stichproben, Median und 95. Perzentil für jede Messgröße und Geräteklasse.
admin-ct-empty-table = In diesem Zeitraum wurden keine Ladezeiten von Browsern gemeldet.
admin-ct-failed = Die Ladezeiten im Client konnten nicht geladen werden.
admin-ct-disabled = Die Erfassung der Ladezeiten im Client ist ausgeschaltet. Die folgenden Zahlen stammen aus der Zeit davor.
admin-ct-note = Nur von angemeldeten Browsern, { $rate } % der Sitzungen als Stichprobe, anonym und aggregiert. Die Zeit bis zur Nutzbarkeit läuft vom Navigationsbeginn, bis die Nachrichtenliste ihre ersten Zeilen zeigt. Die Zahlen sind Schätzungen mit einer Abweichung von etwa { $error } % vom wahren Wert.
