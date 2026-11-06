# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Pagine di diagnostica
admin-diagx-tab-overview = Panoramica
admin-diagx-tab-server = Server
admin-diagx-tab-connectivity = Connettività
admin-diagx-tab-backends = Backend di posta
admin-diagx-tab-posture = Postura di sicurezza
admin-diagx-tab-tools = Strumenti
admin-diagx-tab-history = Cronologia
admin-diagx-tab-report = Report
admin-diagx-tab-server-intro = Controlli delle parti del server stesso: il database, le sue migrazioni e la chiave del server. Vengono eseguiti insieme ai controlli del server.
admin-diagx-tab-connectivity-intro = Se questo server è raggiungibile con il suo nome pubblico, può raggiungere i backend salvati e tiene fuori portata gli indirizzi privati. Gli strumenti di dominio qui sotto consultano i record DNS, verificano il TLS e individuano le impostazioni di un dominio.
admin-diagx-tab-backends-intro = Una connessione e un handshake di sicurezza con ogni endpoint salvato di posta, invio, regole e DAV. Non vengono inviate credenziali e non viene recapitata posta.
admin-diagx-tab-posture-intro = Impostazioni che proteggono accessi e sessioni, e come fail2ban vedrebbe l'indirizzo di una richiesta.
admin-diagx-tab-tools-intro = Altri punti in cui controllare il servizio e i limiti che si applicano alle esecuzioni di diagnostica.
admin-diagx-tab-history-intro = Esecuzioni salvate, dalla più recente. Aprire un'esecuzione per vederne i risultati e cosa è cambiato rispetto all'esecuzione precedente.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Avviso
admin-diagx-status-fail = Non riuscito
admin-diagx-status-skipped = Saltato
admin-diagx-status-not-run = Non eseguito
admin-diagx-status-running = In esecuzione
admin-diagx-run = Esegui
admin-diagx-run-again = Esegui di nuovo
admin-diagx-running = In esecuzione…
admin-diagx-run-server-note = Esegue di nuovo i controlli del server. Tutte le schede di quell'esecuzione vengono aggiornate.
admin-diagx-run-backends-note = Controlla di nuovo tutti i backend salvati. Tutte le schede dei backend vengono aggiornate.
admin-diagx-how-to-fix = Come risolvere
admin-diagx-details = Dettagli
admin-diagx-copy-details = Copia i dettagli
admin-diagx-copy-value = Copia { $label }
admin-diagx-copied = Copiato.
admin-diagx-copy-failed = Il browser non ha consentito la copia. Selezionare il testo e copiarlo manualmente.
admin-diagx-not-run-yet = Non ancora eseguito.
admin-diagx-not-reported = L'ultima esecuzione del server non ha riportato questo controllo; non si applica a questa distribuzione.
admin-diagx-posture-met = Soddisfatto
admin-diagx-posture-not-met = Non soddisfatto
admin-diagx-loading = Caricamento…
admin-diagx-none = Nessuno
admin-diagx-yes = Sì
admin-diagx-no = No
admin-diagx-open = Apri
admin-diagx-row-result = Risultato
admin-diagx-row-duration = Durata
admin-diagx-row-checked = Controllato
admin-diagx-row-id = ID del controllo
admin-diagx-row-domain = Dominio
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protocollo
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Funzionalità
admin-diagx-row-credentials = Credenziali inviate
admin-diagx-row-source = Trovato in
admin-diagx-row-oauth = Richiede OAuth
admin-diagx-overview-intro = Il risultato più recente di ogni controllo, per gruppo. Esegui tutto esegue i controlli del server e poi controlla ogni backend salvato.
admin-diagx-run-all = Esegui tutti i controlli
admin-diagx-run-all-step = Passaggio { $step } di { $steps }: { $suite ->
    [backends] backend salvati
   *[server] controlli del server
}.
admin-diagx-run-all-done = Tutti i controlli sono terminati. I risultati sono salvati in Cronologia.
admin-diagx-backends-progress = { $completed } di { $total } backend controllati.
admin-diagx-counts = { $ok } OK · { $warn } avvisi · { $fail } non riusciti · { $notRun } non eseguiti
admin-diagx-attention = Richiede attenzione
admin-diagx-attention-none = Nessun controllo ha segnalato un avviso o un errore nella sua ultima esecuzione.
admin-diagx-never-run = Nessun controllo è stato ancora eseguito. Eseguire tutti i controlli per ottenere un primo risultato.
admin-diagx-last-server = Ultimi controlli del server
admin-diagx-last-backends = Ultimi controlli dei backend
admin-diagx-never = Mai
admin-diagx-open-health = Stato del servizio
admin-diagx-check-database-connectivity = Connessione al database
admin-diagx-check-database-connectivity-purpose = Il database risponde a una query semplice.
admin-diagx-check-database-connectivity-fix = Verificare che il database sia in esecuzione e che MW_DB_PATH o MW_DATABASE_URL punti a esso. Il log del server indica l'errore di connessione.
admin-diagx-check-database-write = Scritture nel database
admin-diagx-check-database-write-purpose = Una riga di prova può essere scritta e poi annullata.
admin-diagx-check-database-write-fix = Verificare lo spazio libero su disco e che il file o l'utente del database possa scrivere. Un volume in sola lettura o un disco pieno impedisce le scritture.
admin-diagx-check-database-migrations = Migrazioni del database
admin-diagx-check-database-migrations-purpose = Il database ha tutte le migrazioni previste da questa versione del server.
admin-diagx-check-database-migrations-fix = Riavviare il server perché applichi le migrazioni in sospeso e controllarne il log per una migrazione non riuscita. Non eseguire un server più vecchio su un database più recente.
admin-diagx-check-server-key-roundtrip = Chiave del server
admin-diagx-check-server-key-roundtrip-purpose = La chiave del server sigilla e apre un valore di prova.
admin-diagx-check-server-key-roundtrip-fix = Verificare che MW_SERVER_KEY (o il file della chiave) sia la chiave con cui è stato creato questo database. I segreti memorizzati non possono essere aperti con un'altra chiave.
admin-diagx-check-other-purpose = Un controllo riportato da questo server.
admin-diagx-check-other-fix = Aprire Dettagli per il risultato grezzo e controllare il log del server al momento dell'esecuzione.
admin-diagx-check-network-public-dns = Nome pubblico
admin-diagx-check-network-public-dns-purpose = Il nome host in MW_PUBLIC_URL viene risolto da questo server.
admin-diagx-check-network-public-dns-fix = Verificare il record DNS del nome pubblico, oppure impostare MW_PUBLIC_URL sull'indirizzo che le persone usano per raggiungere questo server.
admin-diagx-check-network-public-port = Porta pubblica
admin-diagx-check-network-public-port-purpose = La porta dell'URL pubblico accetta una connessione da questo server.
admin-diagx-check-network-public-port-fix = Verificare il firewall, il reverse proxy e l'inoltro delle porte. Alcuni router non riescono a connettersi al proprio indirizzo pubblico (NAT hairpinning); provare anche dall'esterno della rete.
admin-diagx-check-network-backend-ports = Porte dei backend
admin-diagx-check-network-backend-ports-purpose = L'host e la porta di ogni backend salvato accettano una connessione diretta.
admin-diagx-check-network-backend-ports-fix = Verificare che gli host dei backend siano attivi e che un firewall consenta a questo server di raggiungerli. Backend di posta mostra quale endpoint non supera l'handshake.
admin-diagx-check-egress-private-policy = Protezione degli indirizzi privati
admin-diagx-check-egress-private-policy-purpose = Per impostazione predefinita, le connessioni in uscita verso indirizzi di loopback e privati vengono rifiutate.
admin-diagx-check-egress-private-policy-fix = Rivedere le impostazioni di uscita in Privacy. Consentire gli indirizzi privati solo per i backend indicati che ne hanno bisogno.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Le richieste raggiungono questo server tramite HTTPS.
admin-diagx-check-security-https-fix = Servire la console tramite HTTPS, oppure impostare MW_PUBLIC_URL su https e MW_TRUSTED_PROXIES quando un reverse proxy termina il TLS.
admin-diagx-check-security-secure-cookies = Cookie sicuri
admin-diagx-check-security-secure-cookies-purpose = I cookie di sessione vengono inviati solo tramite HTTPS.
admin-diagx-check-security-secure-cookies-fix = Impostare MW_COOKIE_SECURE su true quando il server è raggiunto tramite HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Ai browser viene indicato di usare HTTPS per questo sito.
admin-diagx-check-security-hsts-fix = Attivare Strict-Transport-Security nelle impostazioni di sicurezza, oppure inviarlo dal reverse proxy.
admin-diagx-check-security-csrf = Protezione CSRF
admin-diagx-check-security-csrf-purpose = Le richieste che modificano dati devono includere il token CSRF.
admin-diagx-check-security-csrf-fix = Impostare MW_CSRF_STRICT su true.
admin-diagx-check-security-admin-factors = Secondo fattore degli amministratori
admin-diagx-check-security-admin-factors-purpose = Ogni amministratore ha un secondo fattore di accesso.
admin-diagx-check-security-admin-factors-fix = Chiedere a ogni amministratore che non ne ha uno di aggiungere una passkey o un'app di autenticazione in Le mie preferenze, oppure richiederlo nel Centro sicurezza.
admin-diagx-check-security-signin-pacing = Ritardo tra i tentativi di accesso
admin-diagx-check-security-signin-pacing-purpose = Dopo un accesso fallito segue una breve attesa.
admin-diagx-check-security-signin-pacing-fix = Attivare il rallentamento degli accessi nel criterio di sicurezza.
admin-diagx-check-security-credential-locks = Blocchi delle credenziali
admin-diagx-check-security-credential-locks-purpose = Accessi falliti ripetuti bloccano la credenziale per un po'.
admin-diagx-check-security-credential-locks-fix = Attivare i blocchi automatici nel Centro sicurezza.
admin-diagx-check-security-geography = Dati di geografia degli accessi
admin-diagx-check-security-geography-purpose = Il criterio di geografia degli accessi dispone dei dati sui paesi di cui ha bisogno.
admin-diagx-check-security-geography-fix = Installare o aggiornare il database dei paesi, oppure disattivare il criterio di geografia.
admin-diagx-check-security-render-isolation = Rendering isolato dei messaggi
admin-diagx-check-security-render-isolation-purpose = I messaggi vengono renderizzati nel worker separato e isolato.
admin-diagx-check-security-render-isolation-fix = Verificare che MW_RENDER_BIN punti al programma mw-render e che si avvii. Lo stato del servizio nella Dashboard mostra lo stato del worker.
admin-diagx-check-security-deployment = Impostazioni della distribuzione
admin-diagx-check-security-deployment-purpose = Le impostazioni di sicurezza della distribuzione possono essere lette.
admin-diagx-check-security-deployment-fix = Verificare la connessione al database, poi aprire la valutazione di sicurezza della distribuzione.
admin-diagx-check-security-plaintext-backends = Password su connessioni non cifrate
admin-diagx-check-security-plaintext-backends-purpose = Nessun backend salvato accetta password senza TLS.
admin-diagx-check-security-plaintext-backends-fix = Modificare le connessioni del dominio e scegliere TLS o STARTTLS. Consentire connessioni non cifrate solo su una rete privata sotto il proprio controllo.
admin-diagx-check-fail2ban-client-address = Indirizzo del client per fail2ban
admin-diagx-check-fail2ban-client-address-purpose = L'indirizzo che una riga di log di fail2ban indicherebbe è l'indirizzo reale del client.
admin-diagx-check-fail2ban-client-address-fix = Dietro un reverse proxy, impostare MW_TRUSTED_PROXIES sull'indirizzo del proxy, altrimenti fail2ban banna il proxy anziché il client. Un indirizzo di loopback è normale quando si apre la console sul server stesso.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Endpoint { $protocol }: connessione e handshake di sicurezza, senza accedere.
admin-diagx-backend-other-purpose = Una nota dall'esecuzione dei backend.
admin-diagx-backend-fix = Verificare il nome host, la porta e la modalità TLS salvati per questo backend, che il DNS venga risolto e che il certificato sia valido per il nome host. Poi eseguire di nuovo.
admin-diagx-backend-config-fix = Aprire le connessioni del dominio e salvarle di nuovo; l'elenco dei backend salvati non ha potuto essere letto.
admin-diagx-backend-results = Controlli dei backend
admin-diagx-backends-not-run = I backend salvati non sono ancora stati controllati.
admin-diagx-backends-run-at = Ultimo controllo { $date }: { $count } risultati.
admin-diagx-failover-title = Failover
admin-diagx-failover-purpose = Se un dominio può passare a un altro backend di posta quando la prima scelta non è raggiungibile.
admin-diagx-failover-state = Failover { $state ->
    [on] attivo
   *[off] disattivo
} · { $backends } backend di posta{ $enabled ->
    [no] {" "}· dominio disattivato
   *[yes] {""}
 }
admin-diagx-no-domains = Nessun dominio salvato.
admin-diagx-domain-tools = Strumenti di dominio
admin-diagx-domain-tools-purpose = Controlli una tantum per un dominio salvato. Non eseguono l'accesso né inviano posta.
admin-diagx-domain = Dominio
admin-diagx-tool-dns = Record DNS
admin-diagx-tool-dns-purpose = Consulta i record MX, SPF, DMARC e di servizio (SRV) del dominio.
admin-diagx-tool-dns-fix = Aggiungere i record mancanti presso il provider DNS. Senza MX il dominio non riceve posta; senza SPF e DMARC altri server possono rifiutarne la posta.
admin-diagx-srv-found = { $found } di { $total } record di servizio trovati
admin-diagx-tool-tls = TLS e handshake
admin-diagx-tool-tls-purpose = Si connette a ogni endpoint salvato del dominio e riporta versione TLS, scadenza del certificato e tempo impiegato.
admin-diagx-tool-tls-fix = Per un endpoint non raggiungibile, verificare host, porta e firewall. Rinnovare un certificato che scade entro 14 giorni.
admin-diagx-probe-none = Il dominio non ha endpoint salvati da provare.
admin-diagx-probe-line = { $host } · raggiungibile: { $reachable ->
    [yes] sì
   *[no] no
} · TLS { $tls } · certificato { $days ->
    [none] non riportato
   *[other] scade tra { $days } giorni
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
 }
admin-diagx-tool-detect = Individua le impostazioni
admin-diagx-tool-detect-purpose = Consulta le impostazioni di posta pubblicate (autoconfig, autodiscover, SRV) per un nome di dominio. Nulla viene salvato.
admin-diagx-tool-detect-fix = Pubblicare record autoconfig o SRV per il dominio, oppure inserire le impostazioni a mano in Domini.
admin-diagx-detect-name = Nome di dominio da consultare
admin-diagx-tool-health = Stato del servizio
admin-diagx-tool-health-purpose = Sincronizzazione della posta, la posta in uscita, l'indice di ricerca, il worker di rendering e i plugin, nella Dashboard.
admin-diagx-tool-deployment-purpose = Ogni impostazione di sicurezza della distribuzione che questo server può osservare, con l'impostazione che la controlla.
admin-diagx-tool-ssh = Test dei tunnel SSH
admin-diagx-tool-ssh-purpose = Prova i tunnel SSH usati per raggiungere backend su un'altra rete.
admin-diagx-tool-geo = Geografia degli accessi
admin-diagx-tool-geo-purpose = Da dove provengono gli accessi e il database dei paesi dietro il criterio di geografia.
admin-diagx-catalogue-purpose = Cosa copre ogni suite di diagnostica e i limiti che ogni esecuzione rispetta.
admin-diagx-suite-unsupported = Non disponibile: { $reason }
admin-diagx-suite-server = Controlli del server
admin-diagx-suite-backends = Backend salvati
admin-diagx-run-title = { $suite ->
    [backends] Backend salvati
   *[server] Controlli del server
}, { $date }
admin-diagx-changes = Modifiche
admin-diagx-changes-first = Nessuna esecuzione precedente di questo tipo con cui confrontare.
admin-diagx-changes-none = Nulla è cambiato rispetto all'esecuzione del { $date }.
admin-diagx-changes-since = Cambiato rispetto all'esecuzione del { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Report di risoluzione dei problemi
admin-diagx-report-purpose = Un report JSON per l'assistenza: flag di runtime, gli ultimi risultati e l'esecuzione scelta. Password, token, cookie, chiavi, configurazione grezza e posta non sono mai inclusi.
admin-diagx-report-run = Esecuzione da includere
admin-diagx-report-preview = Anteprima
admin-crumb-server = Server
admin-crumb-connectivity = Connettività
admin-crumb-posture = Postura di sicurezza
admin-crumb-tools = Strumenti
admin-crumb-history = Cronologia
admin-crumb-report = Report
admin-diagx-table-actions = Azioni
admin-diagx-check-pdfjs-viewer-assets = File del visualizzatore PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = I file del visualizzatore PDF sono tutti presenti, del tipo corretto e corrispondono a questa build del server.
admin-diagx-check-pdfjs-viewer-assets-fix = Ridistribuire la build web, oppure rimuovere la cartella pdfjs da MW_WEB_DIR così da usare la copia integrata nel server. Se questo fallisce anziché avvisare soltanto, ricostruire l'immagine del server.
