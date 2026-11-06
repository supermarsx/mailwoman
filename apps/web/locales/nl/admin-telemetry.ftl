# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Operationele telemetrie
admin-telemetry-help = Exporteer operationele traces en metrics naar een collector die compatibel is met OpenTelemetry, zoals een zelf gehoste OpenTelemetry Collector. Exporteren staat standaard uit. Opgeslagen wijzigingen worden toegepast na een herstart van de server.
admin-telemetry-back = Terug naar observability
admin-telemetry-saved = Telemetrieconfiguratie opgeslagen. Herstart de server om de wijzigingen toe te passen.
admin-telemetry-restart = De opgeslagen configuratie wijkt af van de draaiende server. Een herstart is vereist.
admin-telemetry-reload = Configuratie opnieuw laden
admin-telemetry-save = Telemetrieconfiguratie opslaan
admin-telemetry-collector = Collector en signalen
admin-telemetry-collector-help = Alleen een beheerder kan deze serverbrede bestemming kiezen. Gebruik een privécollector of HTTPS; HTTP verzendt telemetrie zonder versleuteling.
admin-telemetry-enabled = OTLP-export inschakelen
admin-telemetry-traces = Traces exporteren
admin-telemetry-metrics = Metrics exporteren
admin-telemetry-endpoint = OTLP-gRPC-endpoint
admin-telemetry-endpoint-help = Absoluut HTTP- of HTTPS-adres van de collector, meestal poort 4317. Neem geen inloggegevens, query of fragment op. Dit is een gRPC-endpoint, niet OTLP-HTTP-poort 4318.
admin-telemetry-serviceName = Servicenaam
admin-telemetry-serviceName-help = Stabiel servicelabel waarmee deze Mailwoman-installatie in uw collector wordt geïdentificeerd. Gebruik geen e-mailadres of andere persoonlijke gegevens.
admin-telemetry-environment = Omgeving
admin-telemetry-environment-help = Optioneel installatielabel zoals production of staging. Dit wordt met de geëxporteerde telemetrie meegestuurd.
admin-telemetry-pinned = Uitgeschakelde velden worden bepaald door omgevingsvariabelen van de installatie. Wijzig die variabelen om hun waarden te overschrijven.
admin-telemetry-limits = Sampling en exportlimieten
admin-telemetry-limits-help = Begrens het geheugen en het verkeer naar de collector. De batchgrootte mag de wachtrijgrootte niet overschrijden. Volle wachtrijen kunnen spans laten vallen in plaats van mailverzoeken te blokkeren.
admin-telemetry-sampleRatio = Samplingverhouding van traces
admin-telemetry-sampleRatio-help = Fractie van nieuwe roottraces die wordt bewaard, van 0 tot 1. Onderliggende spans volgen de samplingbeslissing van hun ouder.
admin-telemetry-exportTimeoutMs = Time-out voor export (milliseconden)
admin-telemetry-exportTimeoutMs-help = Maximale duur van een exportverzoek aan de collector, van 100 tot 30000 milliseconden.
admin-telemetry-metricsIntervalSecs = Metricsinterval (seconden)
admin-telemetry-metricsIntervalSecs-help = Tijd tussen metricsexports, van 10 tot 3600 seconden.
admin-telemetry-spanQueueSize = Grootte van de spanwachtrij
admin-telemetry-spanQueueSize-help = Maximaal aantal spans dat op export wacht, van 128 tot 32768.
admin-telemetry-spanBatchSize = Spanbatchgrootte
admin-telemetry-spanBatchSize-help = Maximaal aantal spans in één export, van 1 tot 4096 en niet groter dan de wachtrijgrootte.
admin-telemetry-spanExportDelayMs = Vertraging tussen spanbatches (milliseconden)
admin-telemetry-spanExportDelayMs-help = Normale wachttijd tussen spanbatches, van 100 tot 60000 milliseconden.
admin-telemetry-prometheus-help = Een apart geauthenticeerd endpoint voor een Prometheus-compatibele scraper. Toegang vereist altijd het bearer-token MW_METRICS_TOKEN van de installatie; de waarde ervan wordt nooit naar deze pagina teruggestuurd.
admin-telemetry-prometheusEnabled = Prometheus-scraping inschakelen
admin-telemetry-token-missing = Er is geen scrape-token ingesteld. Het metrics-endpoint blijft ontoegankelijk totdat de operator MW_METRICS_TOKEN instelt.
