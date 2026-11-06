# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Dati principali
admin-stats-charts = Grafici
admin-stats-tables = Tabelle
admin-stats-group-traffic = Traffico in questo periodo
admin-stats-group-storage = Archiviazione e consegna, adesso
admin-stats-refresh = Aggiorna le statistiche
admin-stats-loading = Caricamento delle statistiche…
admin-stats-requests-total = Richieste
admin-stats-errors-total = Errori del server (5xx)
admin-stats-p95-peak = 95º percentile più lento
admin-stats-requests = Richieste
admin-stats-errors = Errori del server (5xx)
admin-stats-not-found = Non trovato (404)
admin-stats-latency = Tempo di risposta
admin-stats-series-p50 = Mediana
admin-stats-series-p95 = 95º percentile
admin-stats-unit-count = Conteggio per { $bucket }
admin-stats-unit-duration = Tempo per { $bucket }, nell'unità mostrata
admin-stats-unit-accounts = Account bloccati per dominio
admin-stats-chart-about = Grafico a linee da { $from } a { $to }.
admin-stats-bars-about = { $count ->
    [one] Grafico a barre con un dominio.
   *[other] Grafico a barre con { $count } domini.
}
admin-stats-view-table = Visualizza come tabella
admin-stats-legend = Legenda
admin-stats-empty-chart = In questo periodo non è ancora stato misurato nulla. Scegli un periodo più lungo, oppure ricontrolla dopo che il server avrà gestito qualche richiesta.
admin-stats-empty-blocked = Nessun account è bloccato.
admin-stats-delta-vs = rispetto al periodo precedente
admin-stats-delta-same = Uguale al periodo precedente
admin-stats-delta-sr-up = In aumento di { $change } rispetto al periodo precedente.
admin-stats-delta-sr-down = In calo di { $change } rispetto al periodo precedente.
admin-stats-delta-sr-same = Uguale al periodo precedente.
admin-ct-group = Tempi di caricamento del client
admin-ct-metric-timeToUsable = Tempo all'utilizzabilità
admin-ct-metric-ttfb = Primo byte
admin-ct-metric-dcl = DOM pronto
admin-ct-metric-load = Caricamento della pagina
admin-ct-metric-fcp = First contentful paint
admin-ct-metric-lcp = Largest contentful paint
admin-ct-metric-folderOpen = Apertura cartella
admin-ct-metric-folderLoadCold = Caricamento cartella, dalla rete
admin-ct-metric-folderLoadCached = Caricamento cartella, dalla cache
admin-ct-metric-messageLoadCold = Caricamento messaggio, dalla rete
admin-ct-metric-messageLoadCached = Caricamento messaggio, da cache o prefetch
admin-ct-device-desktop = Desktop
admin-ct-device-mobile = Dispositivo mobile
admin-ct-device-unknown = Sconosciuto
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } campione
       *[other] { $count } campioni
    }
admin-ct-no-samples = Nessun campione in questo periodo.
admin-ct-few-samples = Solo { $count } campioni; ne servono almeno { $min }.
admin-ct-chart-usable = Tempo all'utilizzabilità
admin-ct-group-folder = Tempi di caricamento delle cartelle
admin-ct-group-message = Tempi di caricamento dei messaggi
admin-ct-chart-folder-cold = Caricamento cartella dalla rete
admin-ct-chart-folder-cached = Caricamento cartella dalla cache
admin-ct-chart-message-cold = Caricamento messaggio dalla rete
admin-ct-chart-message-cached = Caricamento messaggio da cache o prefetch
admin-ct-folder-off = I tempi di caricamento delle cartelle sono disattivati per impostazione predefinita. Vengono misurate solo le persone che li hanno attivati.
admin-ct-message-off = I tempi di caricamento dei messaggi sono disattivati per impostazione predefinita. Vengono misurate solo le persone che li hanno attivati.
admin-ct-chart-note = I percentili vengono mostrati solo per gli intervalli con almeno { $min } campioni.
admin-ct-empty-chart = Nessun intervallo ha i { $min } campioni necessari per mostrare una mediana. Scegli un periodo più lungo.
admin-ct-col-samples = Campioni
admin-ct-col-metric = Misura
admin-ct-col-device = Dispositivo
admin-ct-table-title = Tempi di caricamento del client per dispositivo
admin-ct-table-caption = Campioni, mediana e 95º percentile per ogni misura e classe di dispositivo.
admin-ct-empty-table = In questo periodo non sono stati segnalati tempi di caricamento dai browser.
admin-ct-failed = Impossibile caricare i tempi di caricamento del client.
admin-ct-disabled = La raccolta dei tempi di caricamento del client è disattivata. I dati qui sotto risalgono a prima della disattivazione.
admin-ct-note = Solo da browser con accesso effettuato, { $rate }% delle sessioni campionate, anonime e aggregate. Il tempo all'utilizzabilità va dall'inizio della navigazione fino a quando l'elenco dei messaggi mostra le prime righe. I dati sono stime, entro circa il { $error }% del valore reale.
