# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Aggiorna
admin-dash-unavailable = I dati della dashboard non sono ancora disponibili.
admin-dash-unavailable-detail = Questo server non fornisce i dati della dashboard (GET /admin/dashboard risponde 404). Arriveranno con un aggiornamento successivo del server. Le altre sezioni funzionano come prima.
admin-dash-error-network = Impossibile caricare la dashboard: il server non ha risposto. Riprovare.
admin-dash-error-status = Impossibile caricare la dashboard: il server ha risposto { $status }. Riprovare; se si ripete, controllare il log del server.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Richiede attenzione
admin-dash-warn-fix = Apri la sezione che risolve il problema
admin-dash-warn-blocked-accounts = { $count ->
    [one] Un account non può accedere perché il suo dominio non ha una configurazione.
   *[other] { $count } account non possono accedere perché i loro domini non hanno una configurazione.
}
admin-dash-warn-no-trusted-proxy = Le regole IP sono attive ma non è configurato alcun proxy attendibile, quindi ogni client potrebbe sembrare provenire dall'indirizzo del proxy.
admin-dash-warn-allowlist-bypass = Una variabile d'ambiente disattiva l'elenco IP consentiti.
admin-dash-warn-metrics-token = L'endpoint delle metriche non ha un token impostato.
admin-dash-warn-pending-migrations = { $count ->
    [one] Una migrazione del database non è stata applicata. Riavviare il server per applicarla.
   *[other] { $count } migrazioni del database non sono state applicate. Riavviare il server per applicarle.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Un certificato upstream scade a breve (dall'ultima verifica di connessione).
   *[other] { $count } certificati upstream scadono a breve (dall'ultima verifica di connessione).
}
admin-dash-warn-unknown = Avviso del server { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] Un account bloccato: nessuna configurazione del dominio
   *[other] { $count } account bloccati: nessuna configurazione del dominio
}
admin-dash-blocked-note = Questi account accedevano prima di questo aggiornamento, ma il loro dominio non ha una configurazione nella console, quindi ora non possono accedere. Configurando il dominio potranno accedere di nuovo.
admin-dash-blocked-chart = Account bloccati per dominio
admin-dash-blocked-row = { $count ->
    [one] un account
   *[other] { $count } account
}
admin-dash-blocked-seen = (ultimo server di posta usato { $host })
admin-dash-configure-domain = Configura { $domain }
admin-dash-blocked-no-access = Il proprio ruolo non consente di configurare i domini.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Elenco di configurazione: nulla in sospeso
    [one] Elenco di configurazione: una voce in sospeso
   *[other] Elenco di configurazione: { $open } voci in sospeso
}
admin-dash-check-done = Fatto
admin-dash-check-open = In sospeso
admin-dash-check-optional = (facoltativo)
admin-dash-check-go = Apri
admin-dash-check-hide = Nascondi
admin-dash-check-unhide = Mostra di nuovo
admin-dash-check-hidden = { $count ->
    [one] Una voce è nascosta. Nasconderla non la segna come fatta.
   *[other] { $count } voci sono nascoste. Nasconderle non le segna come fatte.
}
admin-dash-check-show-hidden = Mostra le voci nascoste
admin-dash-check-hide-hidden = Escludi le voci nascoste
admin-dash-check-domain-exists = Aggiungere un dominio di posta
admin-dash-check-domain-upstream = Assegnare un server di posta al dominio o impostare un upstream predefinito
admin-dash-check-test-connection = Eseguire una verifica di connessione riuscita (negli ultimi 7 giorni)
admin-dash-check-preferred-domain = Scegliere un dominio preferito, così gli utenti possono accedere senza digitarlo
admin-dash-check-second-admin = Creare un secondo account amministratore o disattivare l'accesso tramite ambiente
admin-dash-check-trusted-proxies = Configurare i proxy attendibili davanti a questo server
admin-dash-check-security-policy = Salvare il criterio di sicurezza
admin-dash-check-metrics-token = Impostare un token per l'endpoint delle metriche
admin-dash-check-branding = Verificare il branding

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Account e sessioni
admin-dash-users = Utenti creati
admin-dash-signed-in-24h = Account che hanno effettuato l'accesso, ultime 24 ore
admin-dash-signed-in-7d = Account che hanno effettuato l'accesso, ultimi 7 giorni
admin-dash-domains = Domini
admin-dash-domains-detail = { $upstream } con un server di posta, { $disabled } disattivati
admin-dash-sessions = Sessioni webmail attive
admin-dash-admin-sessions = Sessioni di amministrazione attive
admin-dash-security = Sicurezza degli accessi
admin-dash-failed-webmail = Accessi webmail falliti, ultime 24 ore
admin-dash-failed-admin = Accessi amministratore falliti, ultime 24 ore
admin-dash-active-blocks = Indirizzi bloccati
admin-dash-allowlists = Elenchi IP consentiti
admin-dash-allowlists-value = Amministrazione { $admin }, webmail { $webmail }
admin-dash-on = attivo
admin-dash-off = disattivato
admin-dash-trusted-proxies = Proxy attendibili
admin-dash-configured = Configurati
admin-dash-not-configured = Non configurati
admin-dash-untrusted-forwarded = { $count ->
    [one] Una richiesta nelle ultime 24 ore conteneva intestazioni di inoltro da un peer non attendibile
   *[other] { $count } richieste nelle ultime 24 ore contenevano intestazioni di inoltro da un peer non attendibile
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Traffico, ultimi { $mins } minuti
admin-dash-traffic-note = Conteggiato da questo processo del server dal suo avvio. Altri processi o repliche non sono inclusi.
admin-dash-traffic-empty = Nessuna richiesta conteggiata in questo intervallo.
admin-dash-requests = Richieste al minuto
admin-dash-errors = Errori del server (5xx) al minuto
admin-dash-p95 = Tempo di risposta, 95° percentile

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Stato
admin-dash-h-ok = OK
admin-dash-h-attention = Richiede attenzione
admin-dash-h-db = Database
admin-dash-h-migrations = Migrazioni
admin-dash-h-engine = Sincronizzazione posta
admin-dash-h-outbox = Posta in uscita
admin-dash-h-index = Indice di ricerca
admin-dash-h-sandbox = Sandbox di rendering
admin-dash-h-plugins = Runtime dei plugin

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Archiviazione
admin-dash-db-size = File del database
admin-dash-wal-size = Write-ahead log
admin-dash-index-size = Indice di ricerca
admin-dash-volume-free = Spazio libero sul volume dei dati
admin-dash-volume-of = di { $total }
admin-dash-na-wal = Questo database non usa un file write-ahead log, oppure il server non lo ha riportato.
admin-dash-na-volume = Il server non può leggere lo spazio libero su questa piattaforma.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Azioni di amministrazione recenti
admin-dash-audit-empty = Nessuna azione di amministrazione registrata.
admin-dash-audit-when = Quando
admin-dash-audit-actor = Chi
admin-dash-audit-action = Azione
admin-dash-audit-target = Oggetto
admin-dash-audit-all = Apri il registro di audit completo
admin-dash-server = Server
admin-dash-version = Versione
admin-dash-build = Build
admin-dash-mode = Modalità
admin-dash-mode-engine = Motore: questo server sincronizza e archivia la posta
admin-dash-mode-proxy = Proxy: questo server inoltra le richieste al server di posta
admin-dash-uptime = In esecuzione da
admin-dash-base-path = Servito su
admin-dash-base-root = la radice del sito
admin-dash-duration-dh = { $d } g { $h } h
admin-dash-duration-hm = { $h } h { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = Un criterio per paese è attivo ma non è caricato alcun database dei paesi utilizzabile, quindi gli accessi che copre vengono rifiutati.
admin-dash-warn-geoip-stale = Un criterio per paese è attivo e il database dei paesi non è aggiornato.
admin-dash-h-status-ok = OK
admin-dash-h-status-warn = Avviso
admin-dash-h-status-fail = In errore
admin-dash-h-status-disabled = Disabilitato
admin-dash-h-checked = Verificato alle { $time }
admin-dash-h-refresh = Verifica di nuovo
admin-dash-h-refresh-failed = Impossibile ripetere il controllo di integrità (stato { $status }). Viene mostrato il risultato precedente.
admin-dash-h-refresh-network = Impossibile ripetere il controllo di integrità: il server non ha risposto. Viene mostrato il risultato precedente.
admin-dash-h-enable = Per attivarlo
admin-dash-h-probe-time = La sonda ha richiesto { $time }
admin-dash-h-outbox-probe-time = L'aggregazione dello stato ha richiesto { $time }; i tempi dell'handshake di invio sono mostrati separatamente.
admin-dash-h-history = Ultimi { $n } controlli
admin-dash-h-open-diagnostics = Diagnostica
admin-dash-h-open-logs = Log
admin-dash-h-open-plugins = Plugin
admin-dash-h-open-maintenance = Manutenzione
admin-dash-h-not-tracked = Non monitorato
admin-dash-h-unlimited = nessun limite
admin-dash-h-m-none = nessuno
admin-dash-h-m-ago = { $age } fa
admin-dash-h-m-ping = Andata e ritorno
admin-dash-h-m-pool = Pool di connessioni
admin-dash-h-m-pool-value = { $inUse } in uso, { $idle } inattive ({ $size } aperte)
admin-dash-h-m-engine = Server di database
admin-dash-h-m-location = Posizione
admin-dash-h-m-db-size = Dimensione del database
admin-dash-h-m-wal-size = Write-ahead log
admin-dash-h-m-journal = Modalità journal
admin-dash-h-m-slow = Query lente (ultima ora)
admin-dash-h-m-applied = Applicate
admin-dash-h-m-applied-value = { $applied } di { $expected } in questa build
admin-dash-h-m-latest = Ultima applicata
admin-dash-h-m-pending = Non ancora applicate
admin-dash-h-m-failed = Avviate, non completate
admin-dash-h-m-unknown = Da una build più recente
admin-dash-h-m-accounts = Caselle di posta
admin-dash-h-m-accounts-value = { $total } registrate: { $syncing } in sincronizzazione, { $idle } inattive, { $erroring } in errore
admin-dash-h-m-last-sync = Dall'ultima sincronizzazione completata
admin-dash-h-m-last-sync-value = mediana { $p50 }, massimo { $max }
admin-dash-h-m-never-synced = Non sincronizzate dall'avvio del server
admin-dash-h-m-queue = Sincronizzazioni in corso
admin-dash-h-m-errors-hour = Errori di sincronizzazione (ultima ora)
admin-dash-h-m-by-kind = Connessioni per backend
admin-dash-h-m-kind-value = { $kind }: { $accounts } caselle, { $watch } connessioni di monitoraggio
admin-dash-h-m-recent-errors = Errori recenti
admin-dash-h-m-queued = In attesa di invio
admin-dash-h-m-queued-value = { $queued } da inviare, { $deferred } in nuovo tentativo, { $sending } in invio
admin-dash-h-m-oldest = Più vecchio in attesa
admin-dash-h-m-scheduled = Pianificati o in sospeso
admin-dash-h-m-scheduled-value = { $scheduled } pianificati, { $held } in sospeso
admin-dash-h-m-failed-total = Non riusciti (ancora in una Posta in uscita)
admin-dash-h-m-failed-total-value = { $failed } non riusciti, { $uncertain } con consegna sconosciuta
admin-dash-h-m-sends-hour = Invii (ultima ora)
admin-dash-h-m-sends-hour-value = { $sent } inviati, { $deferred } rinviati, { $failed } non riusciti
admin-dash-h-m-last-send = Ultimo invio
admin-dash-h-m-last-send-value = { $outcome }, { $age } fa
admin-dash-h-m-last-send-none = Nessuno dall'avvio del server
admin-dash-h-outcome-sent = Inviato
admin-dash-h-outcome-deferred = Rinviato
admin-dash-h-outcome-failed = Non riuscito
admin-dash-h-outcome-uncertain = Consegna sconosciuta
admin-dash-h-m-submission = Server di invio
admin-dash-h-m-submission-value = { $reachable } di { $checked } hanno risposto (solo handshake, senza accesso; il più lento { $slowest }), verificato { $age } fa
admin-dash-h-m-submission-none = Nessun server di invio configurato.
admin-dash-h-m-submission-pending = Verifica della connessione in corso.
admin-dash-h-m-submission-unmeasured = { $count } server richiedono un controllo autenticato presso il provider; non sono state inviate credenziali.
admin-dash-h-m-documents = Documenti
admin-dash-h-m-documents-value = { $documents } di { $messages } messaggi archiviati
admin-dash-h-m-index-pending = Non ancora indicizzati
admin-dash-h-m-index-size = Dimensione su disco
admin-dash-h-m-index-memory = In memoria
admin-dash-h-m-last-commit = Ultimo commit
admin-dash-h-m-rebuild = Ricostruzione
admin-dash-h-m-rebuild-running = In corso: { $pct } ({ $done } di { $total })
admin-dash-h-m-rebuild-idle = Non in corso
admin-dash-h-m-version = Versione del worker
admin-dash-h-m-worker-missing = Nessun binario del worker
admin-dash-h-m-render-ping = Ping (avvio, jail, risposta)
admin-dash-h-m-render-jobs = Job in esecuzione ora
admin-dash-h-m-render-failures = Problemi (ultima ora)
admin-dash-h-m-render-failures-value = { $failed } non riusciti, { $timeouts } scaduti, { $busy } rifiutati perché occupato
admin-dash-h-m-jail = Jail del kernel
admin-dash-h-m-jail-required = Richiesto
admin-dash-h-m-jail-optional = Non richiesto su questa piattaforma
admin-dash-h-m-jail-degraded = Ridotto: { $why }
admin-dash-h-m-plugins = Plugin
admin-dash-h-m-plugins-value = { $installed } installati, { $enabled } abilitati, { $loaded } caricati
admin-dash-h-m-plugin-list = Plugin abilitati
admin-dash-h-m-plugin-loaded = caricato
admin-dash-h-m-plugin-not-loaded = non caricato ({ $reason })
admin-dash-h-m-plugin-limits = memoria { $memory }, scadenza { $deadline }, fuel { $fuel }
admin-dash-h-m-plugin-calls = Chiamate non riuscite (ultima ora)
admin-dash-h-m-plugin-calls-value = { $traps } trap, { $limits } superamenti di limite, { $errors } altri errori
admin-dash-h-m-host-ping = Lock dell'host dei plugin
admin-dash-h-t-ping = Avviso oltre { $warn }, errore oltre { $fail }
admin-dash-h-t-oldest = Avviso quando un messaggio attende oltre { $warn }, errore oltre { $fail }
admin-dash-h-t-pending = Avviso oltre { $n } messaggi non indicizzati
admin-dash-h-t-render = Avviso oltre { $warn }; errore se non c'è risposta entro { $timeout }
admin-dash-h-t-sync = Errore quando { $pct } o più delle caselle non si sincronizzano
admin-dash-h-chart-db = Andata e ritorno del database
admin-dash-h-chart-render = Ping del worker di rendering
admin-dash-h-chart-outbox = Messaggi in attesa di invio
admin-dash-h-chart-search = Messaggi non indicizzati
admin-dash-na-index-proxy = Modalità proxy: questo server non mantiene un indice di ricerca.
admin-dash-na-index-memory = L'indice di ricerca è tenuto in memoria, quindi non ha una dimensione su disco.
admin-dash-customize = Personalizza dashboard
admin-dash-limitations = Disponibilità dei dati
