# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Telemetria operativa
admin-telemetry-help = Esporta tracce e metriche operative verso un collector compatibile con OpenTelemetry, come un OpenTelemetry Collector self-hosted. L'esportazione è disattivata per impostazione predefinita. Le modifiche salvate si applicano dopo il riavvio del server.
admin-telemetry-back = Torna all'osservabilità
admin-telemetry-saved = Configurazione della telemetria salvata. Riavvia il server per applicare le modifiche.
admin-telemetry-restart = La configurazione salvata differisce dal server in esecuzione. È necessario un riavvio.
admin-telemetry-reload = Ricarica la configurazione
admin-telemetry-save = Salva la configurazione della telemetria
admin-telemetry-collector = Collector e segnali
admin-telemetry-collector-help = Solo un amministratore può scegliere questa destinazione valida per tutto il server. Usa un collector privato o HTTPS; HTTP trasmette la telemetria senza cifratura.
admin-telemetry-enabled = Attiva l'esportazione OTLP
admin-telemetry-traces = Esporta le tracce
admin-telemetry-metrics = Esporta le metriche
admin-telemetry-endpoint = Endpoint OTLP gRPC
admin-telemetry-endpoint-help = Indirizzo assoluto HTTP o HTTPS del collector, di solito porta 4317. Non includere credenziali, query o frammenti. È un endpoint gRPC, non la porta OTLP HTTP 4318.
admin-telemetry-serviceName = Nome del servizio
admin-telemetry-serviceName-help = Etichetta di servizio stabile usata per identificare questa installazione di Mailwoman nel suo collector. Non usare un indirizzo email né altre informazioni personali.
admin-telemetry-environment = Ambiente
admin-telemetry-environment-help = Etichetta facoltativa dell'installazione, come production o staging. Viene inviata insieme alla telemetria esportata.
admin-telemetry-pinned = I campi disattivati sono controllati dalle variabili d'ambiente dell'installazione. Modifica quelle variabili per sovrascriverne i valori.
admin-telemetry-limits = Campionamento e limiti di esportazione
admin-telemetry-limits-help = Limita la memoria e il traffico verso il collector. La dimensione del batch non può superare la dimensione della coda. Le code piene possono scartare span invece di bloccare le richieste di posta.
admin-telemetry-sampleRatio = Rapporto di campionamento delle tracce
admin-telemetry-sampleRatio-help = Frazione delle nuove tracce radice da conservare, da 0 a 1. Gli span figli seguono la decisione di campionamento del genitore.
admin-telemetry-exportTimeoutMs = Timeout di esportazione (millisecondi)
admin-telemetry-exportTimeoutMs-help = Durata massima di una richiesta di esportazione al collector, da 100 a 30000 millisecondi.
admin-telemetry-metricsIntervalSecs = Intervallo delle metriche (secondi)
admin-telemetry-metricsIntervalSecs-help = Tempo tra le esportazioni delle metriche, da 10 a 3600 secondi.
admin-telemetry-spanQueueSize = Dimensione della coda di span
admin-telemetry-spanQueueSize-help = Numero massimo di span in attesa di esportazione, da 128 a 32768.
admin-telemetry-spanBatchSize = Dimensione del batch di span
admin-telemetry-spanBatchSize-help = Numero massimo di span in una esportazione, da 1 a 4096 e non superiore alla dimensione della coda.
admin-telemetry-spanExportDelayMs = Ritardo del batch di span (millisecondi)
admin-telemetry-spanExportDelayMs-help = Attesa normale tra i batch di span, da 100 a 60000 millisecondi.
admin-telemetry-prometheus-help = Un endpoint autenticato separato per uno scraper compatibile con Prometheus. L'accesso richiede sempre il bearer token MW_METRICS_TOKEN dell'installazione; il suo valore non viene mai restituito a questa pagina.
admin-telemetry-prometheusEnabled = Attiva lo scraping Prometheus
admin-telemetry-token-missing = Nessun token di scraping configurato. L'endpoint delle metriche resta inaccessibile finché l'operatore non imposta MW_METRICS_TOKEN.
