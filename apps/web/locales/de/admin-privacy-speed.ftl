# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Geschwindigkeit
privacy-speed-unavailable = Geschwindigkeitswerte sind von diesem Server nicht verfügbar.
privacy-speed-empty = In diesem Zeitraum wurden noch keine Bilder ausgeliefert.
privacy-speed-off = Die Geschwindigkeitserfassung ist ausgeschaltet (Einstellung: Geschwindigkeitserfassung des Bild-Proxys). Es wird nichts Neues aufgezeichnet.
privacy-speed-verdict-healthy = In Ordnung
privacy-speed-verdict-slow = Langsam
privacy-speed-verdict-degraded = Beeinträchtigt
privacy-speed-verdict-noData = Zu wenige Daten
privacy-speed-reason-ok = Abgerufene Bilder und Cache-Treffer liegen innerhalb der unten genannten Grenzen, und nur wenige Anfragen sind fehlgeschlagen.
privacy-speed-reason-failureRate = { $failurePct } % der Anfragen endeten mit „ausgelastet“, einer Zeitüberschreitung oder einem Fehler beim Upstream.
privacy-speed-reason-missP95 = Bilder, die abgerufen werden mussten, sind langsam: Ihr 95. Perzentil liegt über der unten genannten Grenze.
privacy-speed-reason-hitP95 = Zwischengespeicherte Bilder antworten langsam: Ihr 95. Perzentil liegt über der unten genannten Grenze.
privacy-speed-reason-fewRequests = Weniger als { $min } Anfragen in diesem Zeitraum, zu wenige für eine Beurteilung.
privacy-speed-thresholds = Langsam ab: 95. Perzentil abgerufener Bilder { $slowMiss }, 95. Perzentil zwischengespeicherter Bilder { $slowHit } oder { $slowFail } % ausgelastet, Zeitüberschreitung oder Fehler. Beeinträchtigt ab: 95. Perzentil abgerufener Bilder { $degradedMiss } oder { $degradedFail } % fehlgeschlagen. Feste Grenzen, keine Einstellungen.
privacy-speed-median = Median
privacy-speed-p95 = 95. Perzentil
privacy-speed-p99 = 99. Perzentil
privacy-speed-max = Langsamste
privacy-speed-requests = Gemessene Anfragen
privacy-speed-busy = Ausgelastet-Antworten
privacy-speed-timeouts = Zeitüberschreitungen
privacy-speed-upstream-errors = Upstream-Fehler
privacy-speed-hit = Cache-Treffer
privacy-speed-miss = Abgerufene Bilder (Cache-Fehlzugriffe)
privacy-speed-split-count = { $count } Anfragen
privacy-speed-split-times = Median { $p50 }, 95. Perzentil { $p95 }, langsamste { $max }.
privacy-speed-chart = Zeit bis zur Auslieferung eines Bildes
privacy-speed-chart-miss = Zeit bis zur Auslieferung eines abgerufenen Bildes
privacy-speed-chart-unit = Von der Anfrage bis zur fertigen Antwort
privacy-speed-phases = Wohin die Zeit geht
privacy-speed-phases-help = Für Anfragen, die keine Cache-Treffer waren: wie lange jeder Schritt dauerte und welchen Anteil er an der Gesamtzeit hatte.
privacy-speed-phase = Schritt
privacy-speed-share = Zeitanteil
privacy-speed-phase-queue = Warten auf Kapazität
privacy-speed-phase-cache = Cache-Abfrage
privacy-speed-phase-upstream = Upstream-Abruf
privacy-speed-phase-decode = Dekodieren und erneut kodieren
privacy-speed-phase-other = Alles andere
privacy-speed-note = Nur auf diesem Serverprozess gemessen, im Arbeitsspeicher gehalten und nach einem Neustart leer. Perzentile sind auf { $error } % genau. Es werden keine URLs, Hosts oder Konten gespeichert.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = Verbundene Abrufe und Prioritätsanfragen

privacy-work-coalesced-series = Fetches verbanden sich im Flug zu einem

privacy-work-handle = Behandeln Sie die Cache-Trefferquote

privacy-work-handle-series = Behandeln Sie Cache-Treffer

privacy-work-per-interval = Pro Abtastintervall

privacy-work-percent = Anteil der Handle-Suchen

privacy-work-priority-series = Vorrangige Anfragen

privacy-work-resized = Bilder verkleinert

privacy-work-resized-series = Größe geändert

privacy-work-revalidated-series = Neu validiert

privacy-work-saved = Durch Herunterskalierung eingesparte Bytes

privacy-work-saved-series = Bytes gespart

privacy-work-stale = Veraltete Bilder werden bereitgestellt und erneut validiert

privacy-work-stale-series = Abgestanden serviert

privacy-work-title = Arbeit vermieden und umgestaltet
