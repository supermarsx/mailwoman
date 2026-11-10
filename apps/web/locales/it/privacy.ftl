# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: privacy. Source: ../en/privacy.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-title = Privacy
privacy-help = Le immagini remote possono rivelare quando si apre un messaggio. Scegliere quando vengono caricate e se il server le recupera per conto dell'utente.
privacy-remote-mode = Contenuto remoto
privacy-mode-block = Blocca il contenuto remoto
privacy-mode-ask = Chiedi prima di caricare tramite il server
privacy-mode-proxy = Carica tramite il server
privacy-mode-direct = Carica direttamente dai server del mittente
privacy-trackers = Blocca immagini e beacon di tracciamento
privacy-clean-links = Rimuovi i parametri di tracciamento dai link
privacy-direct-unavailable = Il caricamento diretto richiede l'autorizzazione dell'amministratore. Espone l'indirizzo di rete all'host dell'immagine.
privacy-direct-native = Il caricamento diretto non è disponibile in questa app nativa. Le immagini possono essere caricate tramite il server.
privacy-inherit = Usa l'impostazione ereditata
privacy-content-failed = Non è stato possibile autorizzare alcune immagini. Il messaggio resta leggibile, con quelle immagini bloccate.
privacy-frame-failed = Impossibile caricare il visualizzatore isolato delle immagini. Ricaricare il messaggio per riprovare.
privacy-download-file = Scarica tramite il server
privacy-file-failed = Impossibile scaricare questo file tramite il server.
privacy-admin-title = Privacy e contenuto remoto
privacy-admin-help = Impostare valori predefiniti, blocchi e limiti tramite la policy delle impostazioni. I conteggi diagnostici riguardano solo questo processo del server.
privacy-cache-entries = Immagini in cache
privacy-cache-bytes = Dimensione della cache (byte)
privacy-replaced-trackers = Pixel di tracciamento sostituiti
privacy-edit-setting = Modifica policy
privacy-outbound-title = Connessioni in uscita
privacy-fail-closed = Un percorso configurato che non riesce blocca la richiesta. Non ricade mai su una connessione diretta.
privacy-linked-files = File collegati in questo messaggio
privacy-allow-direct = Consenti il caricamento diretto delle immagini
privacy-max-bytes = Dimensione massima dell'immagine (byte)
privacy-cache-limit = Dimensione massima della cache (byte)
privacy-rate = Richieste al minuto
privacy-on = Attivo
privacy-off = Disattivato
privacy-locked = Bloccato dalla policy dell'amministratore
privacy-process-stats = Dall'avvio di questo processo del server
privacy-refresh = Aggiorna
privacy-traffic-class = Classe di traffico
privacy-allowed = Consentito
privacy-refused = Rifiutato dalla policy
privacy-failed = Non riuscito
privacy-policy-values = Policy sul contenuto remoto
privacy-cache-summary = Cache e protezione dal tracciamento
privacy-traffic-summary = Elaborazione del traffico in uscita
privacy-metric = Metrica
privacy-value = Valore
privacy-total-duration = Tempo totale
privacy-no-traffic = Dall'avvio di questo processo non è stato registrato alcun traffico in uscita.
privacy-analytics-title = Conteggi di utilizzo facoltativi
privacy-analytics-help = Disattivato per impostazione predefinita. Se l'amministratore abilita un collector, è possibile condividere nomi fissi delle schermate come Posta o Calendario. Messaggi, indirizzi email, ricerche, dettagli dei percorsi e identificatori del browser non vengono mai inviati. Sono conteggi di eventi, non statistiche sui visitatori unici.
privacy-analytics-consent = Consenti i conteggi facoltativi di utilizzo delle schermate per questo account
privacy-analytics-signal = Il segnale di privacy del browser impedisce l'analisi, anche se il consenso è salvato.
privacy-load-time-title = Tempi di caricamento
privacy-load-time-help = Aiuta l'amministratore a capire quanto sia veloce Mailwoman. Vengono comunicati solo il numero di millisecondi e se i dati provengono dalla cache o dalla rete, in forma anonima e aggregata. Non sono inclusi cartelle, messaggi, mittenti né oggetti.
privacy-load-time-folder = Misura il tempo di apertura delle cartelle
privacy-load-time-folder-help = Attivo per impostazione predefinita. Comunica il tempo dalla scelta di una cartella alla visualizzazione del suo elenco di messaggi.
privacy-load-time-message = Misura il tempo di apertura dei messaggi
privacy-load-time-message-help = Attivo per impostazione predefinita. Comunica il tempo dall'apertura di un messaggio alla visualizzazione di intestazioni e testo.
privacy-analytics-save-error = Impossibile sincronizzare il consenso. Questo browser sospende le nuove richieste di analisi finché la modifica non viene salvata.
privacy-analytics-saving = Salvataggio del consenso…
privacy-back-policies = Torna alle policy sulla privacy
privacy-image-toast = Mostra una notifica durante il caricamento delle immagini remote
privacy-image-downloads = Download manuali delle immagini remote
privacy-image-download-help = Se un'immagine non viene visualizzata, scaricarne una copia verificata tramite questo server. Il server controlla che appartenga a questo messaggio e applica comunque le proprie regole di privacy e sicurezza. Le immagini decifrate dal client non sono disponibili qui.
privacy-image-download-ready = L'immagine è pronta e il download è iniziato.
privacy-image-download-failed = Impossibile scaricare l'immagine. Potrebbe non essere disponibile o essere bloccata dalla policy sulla privacy.
privacy-strip-mailer-headers = Rimuovi le intestazioni del programma di posta in uscita
privacy-strip-mailer-help = Gli invii tramite SMTP nativo ed EWS rimuovono le intestazioni di impronta del client di primo livello. I corpi dei messaggi, i messaggi allegati e i dati dei rapporti restano intatti. Questo non copre gli invii tramite proxy JMAP upstream né le intestazioni aggiunte in seguito da un relay. I messaggi firmati in precedenza che coprono un'intestazione rimossa richiedono la disattivazione di questa impostazione prima dell'invio.
privacy-frame-compatible = Visualizzatore compatibile: le immagini caricate direttamente da altri siti restano bloccate per privacy. Sono supportate le immagini incorporate e quelle caricate tramite il server.
privacy-analytics-tag-custom = uno script di analisi personalizzato
privacy-analytics-tag-title = Analisi nel browser: { $provider }
privacy-analytics-tag-help = L'amministratore offre anche { $provider }, che viene eseguito in questa pagina nel browser. A differenza dei conteggi sopra, il suo script può vedere le pagine aperte in questa app, compresi messaggi e indirizzi mostrati a schermo, il browser e l'indirizzo IP, e può impostare propri identificatori. Il fornitore può aggiornare il proprio script senza chiedere di nuovo il consenso. Disattivato a meno che lo si attivi qui; disattivarlo ricarica la pagina senza di esso, in ogni scheda.
privacy-analytics-tag-consent = Consenti a { $provider } di essere eseguito in questa app per questo account
privacy-analytics-tag-changed = Il fornitore di analisi, o il modo in cui questo server lo carica, è cambiato dall'ultimo consenso. Resta disattivato finché non si acconsente di nuovo.
privacy-trusted-links-title = Domini dei link attendibili
privacy-trusted-links-help = I link a questi domini si aprono direttamente da Controlla i link, senza il passaggio della destinazione. Gli altri link, compresi i domini simili, vengono comunque controllati. I link che usano HTTP semplice o un indirizzo IP vengono sempre controllati.
privacy-trusted-links-domains = Domini
privacy-trusted-links-domains-help = Un dominio per riga, ad esempio example.com. La corrispondenza è esatta: example.com non rende attendibile examples.com né example.com.other.net. Usare la forma ASCII (xn--) per i nomi internazionali.
privacy-trusted-links-invalid = Inserire fino a 100 nomi di dominio con almeno due etichette e senza schema, percorso, porta o carattere jolly. Non accettati: { $entries }
privacy-trusted-links-save = Salva domini attendibili
privacy-trusted-links-subdomains = Considera attendibili anche i sottodomini (mail.example.com per example.com)
privacy-trusted-links-subdomains-help = Disattivato: è attendibile solo il dominio esatto. Attivo: è attendibile anche qualsiasi nome che termina con .example.com.
privacy-linked-files-count = { $count ->
    [one] Un file collegato in questo messaggio
   *[other] { $count } file collegati in questo messaggio
}
privacy-linked-links = Link in questo messaggio
privacy-linked-files-show = Mostra i file
privacy-linked-links-show = Scarica un link
privacy-linked-files-help = I download passano da questo server, quindi l'host del file non vede l'indirizzo. La dimensione di un file è nota solo dopo il download.
privacy-linked-links-help = Un link di questo messaggio può essere scaricato come file tramite questo server, così il suo host non vede l'indirizzo.
privacy-linked-file-item = Scarica { $name } da { $host }
privacy-linked-other-item = Scarica { $link }
privacy-links-enable = Abilita i link
privacy-links-options = Opzioni dei link
privacy-links-enable-once = Abilita una volta
privacy-links-trust-sender = Consenti sempre i link di questo mittente
privacy-links-revoke-sender = Non consentire più sempre i link di questo mittente
privacy-links-trust-failed = Impossibile salvare le preferenze dei link.
privacy-trusted-link-senders-title = Mittenti di link attendibili
privacy-trusted-link-senders-help = Consente automaticamente i link dei messaggi per questi indirizzi mittente esatti. Le destinazioni dei link vengono comunque convalidate.
privacy-trusted-link-senders-list = Indirizzi dei mittenti
privacy-trusted-link-senders-list-help = Un indirizzo email completo per riga. La parte prima di @ viene confrontata esattamente, maiuscole comprese. Rimuovere un indirizzo per revocare l'attendibilità.
privacy-trusted-link-senders-save = Salva mittenti attendibili
privacy-trusted-link-senders-remove = Rimuovi { $sender }
privacy-trusted-link-senders-invalid = Inserire indirizzi email individuali validi: { $entries }
privacy-links-disable = Disabilita i link
privacy-links-enable-help = Consente ai link di questo messaggio di aprirsi in una nuova scheda. Si possono disabilitare di nuovo.
privacy-links-loading = Preparazione dei link…
privacy-body-blocked = HTML del messaggio bloccato per sicurezza
privacy-body-blocked-detail = Questo corpo contiene contenuti potenzialmente non sicuri o non è stato possibile controllarlo. Il contenuto attivo resta bloccato. Il messaggio originale non è modificato.
privacy-body-show-sanitized = Mostra HTML ripulito
privacy-body-hide-sanitized = Mostra testo sicuro

# -- Machine translation (2026-11): added to match en ---------------------
privacy-body-sanitized-shown = HTML ripulito mostrato
privacy-unsafe-html = HTML del messaggio potenzialmente non sicuro
privacy-unsafe-html-ask = Chiedi prima di mostrare l'HTML ripulito
privacy-unsafe-html-strict = Mostra sempre il testo sicuro
privacy-unsafe-html-sanitized = Mostra sempre l'HTML ripulito
privacy-unsafe-html-help = L'HTML ripulito rimuove il contenuto attivo. Risorse remote, link e conferme automatiche restano bloccati per questi messaggi. Gli originali non vengono modificati.
privacy-original-html-allow = Consenti l'HTML originale per messaggio
privacy-original-html-help = Offre una visualizzazione esplicita dell'HTML originale per ogni messaggio. Script e moduli restano disattivati e immagini e link seguono comunque i controlli sulla privacy. L'HTML ripulito resta quello predefinito.
privacy-original-html = HTML originale
privacy-original-html-show = Mostra HTML originale
privacy-original-html-hide = Torna alla vista ripulita
privacy-original-html-active = HTML originale mostrato — script disattivati, risorse controllate
privacy-original-html-offer = Conserva il layout originale del messaggio solo per questo messaggio.
privacy-tracker-choice = Consenti una scelta dei tracker per messaggio
privacy-tracker-choice-help = Offre un'azione esplicita per caricare i tracker bloccati per un solo messaggio. I tracker restano bloccati finché non si sceglie e un amministratore può imporre la protezione.
privacy-image-retry-attempts = Tentativi automatici per le immagini
privacy-image-retry-help = Riprova solo gli errori temporanei delle immagini dopo che quelle attuali hanno finito di caricarsi. Zero disattiva i tentativi automatici; è comunque possibile riprovare manualmente. I blocchi dell'amministratore hanno la precedenza.
privacy-image-retry-delay = Ritardo del nuovo tentativo per le immagini (secondi)
privacy-concurrency-title = Capacità adattiva per le immagini
privacy-concurrency-mode-auto = Automatico
privacy-concurrency-mode-conservative = Poche risorse
privacy-concurrency-mode-balanced = Bilanciato
privacy-concurrency-mode-throughput = Alto throughput
privacy-concurrency-mode-manual = Manuale
privacy-concurrency-apply = Applica profilo
privacy-concurrency-saved = Profilo di concorrenza delle immagini salvato.
privacy-concurrency-unavailable = Non disponibile
privacy-concurrency-sizing = Core CPU: { $cpu } · Limite di memoria: { $limit } · Memoria disponibile: { $available }
privacy-concurrency-running = Profilo attivo: { $mode } · Dimensionamento adattivo: { $state }
privacy-concurrency-cap-perMessage = Per messaggio
privacy-concurrency-cap-perAccount = Per account
privacy-concurrency-cap-global = Rete globale
privacy-concurrency-cap-pipeline = Pipeline di elaborazione
privacy-concurrency-cap-decoders = Decodificatori
privacy-concurrency-effective-ceiling = { $effective } effettivo / { $ceiling } massimo
privacy-concurrency-message-bound = Il valore per messaggio è un limite superiore dell'istanza. Ogni messaggio usa al massimo tale limite o il numero di immagini approvate, se inferiore.
privacy-concurrency-active = Attivi: { $network } rete · { $pipeline } pipeline · { $decoders } decodificatori · { $queued } in coda
privacy-image-rate-readout = Quota di recupero per account ({ $mode }): burst { $burst } (configurato { $configuredBurst }), ripristino { $perMin } al minuto (configurato { $configuredPerMin })
privacy-image-cache-readout = Cache delle immagini ({ $mode }): fino a { $bytes } e { $entries } voci, durata { $ttl } s, percentuale di successi { $hit }
privacy-image-hit-unknown = non ci sono ancora abbastanza ricerche
privacy-tab-overview = Panoramica
privacy-tab-remote = Contenuto remoto e tracker
privacy-tab-image-proxy = Proxy delle immagini
privacy-tab-fetch-limits = Limiti di recupero
privacy-tab-traffic = Traffico
privacy-tab-cookies = Cookie
privacy-tab-denied = Non si ha accesso a questa scheda.
privacy-tab-remote-help = Valori predefiniti, blocchi e valori per dominio per contenuto remoto, protezione dal tracciamento e pulizia dei link. Ogni impostazione viene salvata separatamente.
privacy-settings-denied = La lettura delle impostazioni richiede il permesso settings.read, che non è assegnato. Le statistiche nelle altre schede restano disponibili.
privacy-overview-status = Stato
privacy-overview-figures = Dati principali
privacy-status-ok = Il proxy delle immagini sta riportando dati.
privacy-status-unavailable = Impossibile leggere le statistiche del proxy delle immagini.
privacy-history-title = Proxy delle immagini nel tempo
privacy-history-worker-title = Cronologia dei worker
privacy-history-cache-title = Cache e limiti
privacy-history-traffic-title = Traffico in uscita
privacy-history-empty = Ancora nessun campione in questo intervallo.
privacy-history-unavailable = La cronologia non è disponibile su questo server.
privacy-history-summary = { $series }: ultimo { $latest }, massimo { $max }.
privacy-history-range = Intervallo di tempo
privacy-history-range-15m = 15 minuti
privacy-history-range-1h = 1 ora
privacy-history-range-6h = 6 ore
privacy-history-range-24h = 24 ore
privacy-history-updated = Aggiornato { $time }
privacy-history-not-updated = Non ancora aggiornato
privacy-history-memory-note = Il server conserva un campione ogni { $interval } secondi per 24 ore, solo in memoria. La cronologia riparte vuota dopo un riavvio e non è condivisa tra processi del server.
privacy-chart-requests = Immagini servite
privacy-chart-requests-unit = Richieste al secondo
privacy-chart-bytes = Dati serviti
privacy-chart-bytes-unit = Byte al secondo
privacy-chart-hit = Percentuale di successi della cache
privacy-chart-hit-unit = Percentuale delle ricerche
privacy-chart-ram = Memoria
privacy-chart-disk = Disco
privacy-chart-queue = Recuperi in corso e in attesa
privacy-chart-queue-unit = Numero
privacy-chart-active = In corso
privacy-chart-queued = In attesa
privacy-chart-busy = Risposte di occupato
privacy-chart-busy-unit = Risposte per intervallo
privacy-chart-trackers = Pixel di tracciamento sostituiti
privacy-chart-trackers-unit = Numero per intervallo
privacy-chart-size = Dimensione della cache
privacy-chart-size-unit = Byte conservati
privacy-chart-entries = Voci della cache
privacy-chart-entries-unit = Numero
privacy-chart-wait = Attesa del limite di frequenza
privacy-chart-wait-unit = Millisecondi medi per richiesta in attesa
privacy-chart-rate = Budget del limite di frequenza
privacy-chart-rate-unit = Quota di burst e account attivi
privacy-chart-burst = Quota di burst
privacy-chart-accounts = Account attivi
privacy-chart-allowed = Richieste consentite
privacy-chart-refused = Richieste rifiutate e non riuscite
privacy-chart-traffic-unit = Numero per intervallo
privacy-image-group-rate = Frequenza delle richieste
privacy-image-group-cache = Cache
privacy-image-group-ram = Livello di memoria
privacy-image-group-disk = Livello su disco
privacy-image-group-tiering = Livelli e manutenzione

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-error-reports-help = Quando una parte di Mailwoman fallisce, il browser può inviare all'amministratore un breve rapporto tecnico in modo che il problema possa essere risolto. Contiene il tipo di errore, il punto in cui si è verificato nell'app, la versione dell'app e il tipo di browser. Non conserva mai il contenuto dei messaggi, gli indirizzi, i nomi degli allegati o ciò che hai digitato.

privacy-error-reports-send = Invia segnalazioni di errori da questo browser

privacy-error-reports-send-help = Attivo per impostazione predefinita. Disattiva per mantenere questi report sul tuo dispositivo. Il pannello di errore mostra ancora un riferimento che puoi copiare.

privacy-error-reports-title = Segnalazioni di errori

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-links-revoke-domain = Smetti di consentire sempre i collegamenti da questo dominio del mittente

privacy-links-trust-domain = Consenti sempre i collegamenti da questo dominio mittente

privacy-trusted-link-sender-domains = Domini mittente

privacy-trusted-link-sender-domains-help = Consenti collegamenti esaminati nei messaggi provenienti da questi domini esatti del mittente. Un dominio per linea; i sottodomini sono separati. Questo non si fida dei siti Web di destinazione né del caricamento delle immagini. Un elenco vuoto richiede ciascun messaggio.

privacy-trusted-link-sender-domains-save = Salva i domini dei mittenti attendibili

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-feedback-image-host = Preferenza host immagine salvata.

privacy-feedback-images-off = Autorizzazione immagine rimossa.

privacy-feedback-images-on = Autorizzazione immagine salvata. Ora è possibile caricare le immagini consentite.

privacy-feedback-link-domain-off = Trust del collegamento di dominio rimosso per { $address }.

privacy-feedback-link-domain-on = I collegamenti saranno consentiti nei messaggi provenienti dal dominio { $address }.

privacy-feedback-link-sender-off = Attendibilità del collegamento del mittente rimossa per { $address }.

privacy-feedback-link-sender-on = I collegamenti saranno consentiti nei messaggi provenienti da { $address }.

privacy-feedback-links-off = Collegamenti disabilitati per questo messaggio.

privacy-feedback-links-on = Collegamenti abilitati per questo messaggio.
