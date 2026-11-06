# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Betriebstelemetrie
admin-telemetry-help = Betriebs-Traces und -Metriken an einen OpenTelemetry-kompatiblen Collector exportieren, etwa einen selbst gehosteten OpenTelemetry Collector. Der Export ist standardmäßig ausgeschaltet. Gespeicherte Änderungen gelten nach einem Neustart des Servers.
admin-telemetry-back = Zurück zur Observability
admin-telemetry-saved = Telemetriekonfiguration gespeichert. Starten Sie den Server neu, um die Änderungen anzuwenden.
admin-telemetry-restart = Die gespeicherte Konfiguration weicht vom laufenden Server ab. Ein Neustart ist erforderlich.
admin-telemetry-reload = Konfiguration neu laden
admin-telemetry-save = Telemetriekonfiguration speichern
admin-telemetry-collector = Collector und Signale
admin-telemetry-collector-help = Nur ein Administrator kann dieses serverweite Ziel wählen. Verwenden Sie einen privaten Collector oder HTTPS; HTTP überträgt Telemetrie unverschlüsselt.
admin-telemetry-enabled = OTLP-Export aktivieren
admin-telemetry-traces = Traces exportieren
admin-telemetry-metrics = Metriken exportieren
admin-telemetry-endpoint = OTLP-gRPC-Endpunkt
admin-telemetry-endpoint-help = Absolute HTTP- oder HTTPS-Adresse des Collectors, meist Port 4317. Geben Sie keine Zugangsdaten, keine Query und kein Fragment an. Dies ist ein gRPC-Endpunkt, nicht OTLP-HTTP-Port 4318.
admin-telemetry-serviceName = Dienstname
admin-telemetry-serviceName-help = Stabile Dienstbezeichnung, mit der diese Mailwoman-Bereitstellung in Ihrem Collector identifiziert wird. Verwenden Sie keine E-Mail-Adresse und keine anderen personenbezogenen Angaben.
admin-telemetry-environment = Umgebung
admin-telemetry-environment-help = Optionale Bezeichnung der Bereitstellung, etwa production oder staging. Sie wird mit der exportierten Telemetrie gesendet.
admin-telemetry-pinned = Deaktivierte Felder werden durch Umgebungsvariablen der Bereitstellung gesteuert. Ändern Sie diese Variablen, um ihre Werte zu überschreiben.
admin-telemetry-limits = Sampling und Exportgrenzen
admin-telemetry-limits-help = Begrenzen Arbeitsspeicher und Collector-Datenverkehr. Die Batchgröße darf die Warteschlangengröße nicht überschreiten. Volle Warteschlangen können Spans verwerfen, statt E-Mail-Anfragen zu blockieren.
admin-telemetry-sampleRatio = Trace-Sampling-Verhältnis
admin-telemetry-sampleRatio-help = Anteil neuer Root-Traces, die behalten werden, von 0 bis 1. Untergeordnete Spans folgen der Sampling-Entscheidung des übergeordneten.
admin-telemetry-exportTimeoutMs = Export-Zeitlimit (Millisekunden)
admin-telemetry-exportTimeoutMs-help = Maximale Dauer einer Collector-Exportanfrage, von 100 bis 30000 Millisekunden.
admin-telemetry-metricsIntervalSecs = Metrikintervall (Sekunden)
admin-telemetry-metricsIntervalSecs-help = Zeit zwischen Metrikexporten, von 10 bis 3600 Sekunden.
admin-telemetry-spanQueueSize = Größe der Span-Warteschlange
admin-telemetry-spanQueueSize-help = Maximale Anzahl der auf den Export wartenden Spans, von 128 bis 32768.
admin-telemetry-spanBatchSize = Span-Batchgröße
admin-telemetry-spanBatchSize-help = Maximale Anzahl Spans in einem Export, von 1 bis 4096 und nicht größer als die Warteschlangengröße.
admin-telemetry-spanExportDelayMs = Span-Batchverzögerung (Millisekunden)
admin-telemetry-spanExportDelayMs-help = Normale Wartezeit zwischen Span-Batches, von 100 bis 60000 Millisekunden.
admin-telemetry-prometheus-help = Ein separater authentifizierter Endpunkt für einen Prometheus-kompatiblen Scraper. Der Zugriff erfordert immer das Bearer-Token MW_METRICS_TOKEN der Bereitstellung; sein Wert wird nie an diese Seite zurückgegeben.
admin-telemetry-prometheusEnabled = Prometheus-Scraping aktivieren
admin-telemetry-token-missing = Es ist kein Scrape-Token konfiguriert. Der Metrik-Endpunkt bleibt unzugänglich, bis der Betreiber MW_METRICS_TOKEN setzt.
