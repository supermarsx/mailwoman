# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Velocità
privacy-speed-unavailable = I dati di velocità non sono disponibili da questo server.
privacy-speed-empty = In questo intervallo non è ancora stata servita alcuna immagine.
privacy-speed-off = Il monitoraggio della velocità è disattivato (impostazione: Monitoraggio della velocità del proxy immagini). Non viene registrato nulla di nuovo.
privacy-speed-verdict-healthy = Buono
privacy-speed-verdict-slow = Lento
privacy-speed-verdict-degraded = Degradato
privacy-speed-verdict-noData = Dati insufficienti
privacy-speed-reason-ok = Le immagini recuperate e gli hit della cache rientrano nei limiti sottostanti e poche richieste sono fallite.
privacy-speed-reason-failureRate = Il { $failurePct } % delle richieste è terminato con occupato, timeout o errore a monte.
privacy-speed-reason-missP95 = Le immagini che hanno dovuto essere recuperate sono lente: il loro 95º percentile supera il limite sottostante.
privacy-speed-reason-hitP95 = Le immagini in cache rispondono lentamente: il loro 95º percentile supera il limite sottostante.
privacy-speed-reason-fewRequests = Meno di { $min } richieste in questo intervallo, troppo poche per giudicare.
privacy-speed-thresholds = Lento da: 95º percentile dell'immagine recuperata { $slowMiss }, 95º percentile dell'immagine in cache { $slowHit }, oppure { $slowFail } % occupato, timeout o fallito. Degradato da: 95º percentile dell'immagine recuperata { $degradedMiss } oppure { $degradedFail } % fallito. Limiti fissi, non impostazioni.
privacy-speed-median = Mediana
privacy-speed-p95 = 95º percentile
privacy-speed-p99 = 99º percentile
privacy-speed-max = Più lenta
privacy-speed-requests = Richieste misurate
privacy-speed-busy = Risposte occupato
privacy-speed-timeouts = Timeout
privacy-speed-upstream-errors = Errori a monte
privacy-speed-hit = Hit della cache
privacy-speed-miss = Immagini recuperate (miss della cache)
privacy-speed-split-count = { $count } richieste
privacy-speed-split-times = Mediana { $p50 }, 95º percentile { $p95 }, più lenta { $max }.
privacy-speed-chart = Tempo per servire un'immagine
privacy-speed-chart-miss = Tempo per servire un'immagine recuperata
privacy-speed-chart-unit = Dalla richiesta ricevuta alla risposta pronta
privacy-speed-phases = Dove va il tempo
privacy-speed-phases-help = Per le richieste che non sono state un hit della cache: quanto è durato ogni passaggio e la sua quota del loro tempo totale.
privacy-speed-phase = Passaggio
privacy-speed-share = Quota del tempo
privacy-speed-phase-queue = Attesa di capacità
privacy-speed-phase-cache = Ricerca nella cache
privacy-speed-phase-upstream = Recupero a monte
privacy-speed-phase-decode = Decodifica e ricodifica
privacy-speed-phase-other = Tutto il resto
privacy-speed-note = Misurato solo su questo processo del server, tenuto in memoria e vuoto dopo un riavvio. I percentili sono precisi entro il { $error } %. Non vengono memorizzati URL, host o account.
