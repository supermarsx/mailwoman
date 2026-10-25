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
admin-dash-ms = { $ms } ms

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Stato
admin-dash-h-ok = OK
admin-dash-h-attention = Richiede attenzione
admin-dash-h-db = Database
admin-dash-h-db-ok = Ha risposto in { $ms }
admin-dash-h-migrations = Migrazioni
admin-dash-h-migrations-ok = Aggiornate ({ $n } applicate)
admin-dash-h-migrations-behind = { $applied } di { $expected } applicate
admin-dash-h-engine = Sincronizzazione posta
admin-dash-h-engine-value = { $accounts } account registrati, { $errors } con un errore di sincronizzazione
admin-dash-h-outbox = Posta in uscita
admin-dash-h-outbox-value = { $pending } in attesa (il più vecchio { $oldest }), { $failed } non riusciti nelle ultime 24 ore
admin-dash-h-outbox-none = nessuno
admin-dash-h-index = Indice di ricerca
admin-dash-h-index-value = { $n } documenti
admin-dash-h-index-rebuilding = Ricostruzione in corso ({ $n } documenti finora)
admin-dash-h-sandbox = Sandbox di rendering
admin-dash-h-sandbox-ok = Disponibile: i documenti non attendibili vengono visualizzati in un processo separato.
admin-dash-h-sandbox-missing = Assente: i documenti non attendibili vengono visualizzati all'interno del processo del server.
admin-dash-h-sandbox-required = Assente, e questo server la richiede: la visualizzazione dei documenti non attendibili non riesce.
admin-dash-h-plugins = Runtime dei plugin
admin-dash-h-plugins-ok = Disponibile
admin-dash-h-plugins-off = Disattivato
admin-dash-na-proxy = questo server funziona in modalità proxy e non sincronizza direttamente la posta.
admin-dash-na-unreported = il server non lo ha riportato.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Archiviazione
admin-dash-db-size = File del database
admin-dash-wal-size = Write-ahead log
admin-dash-index-size = Indice di ricerca
admin-dash-volume-free = Spazio libero sul volume dei dati
admin-dash-volume-of = di { $total }
admin-dash-na-wal = Questo database non usa un file write-ahead log, oppure il server non lo ha riportato.
admin-dash-na-index = Questo server non ha un indice di ricerca.
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
