# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Consenti l'invio
admin-settings-help-mail-allow-send = Consente agli utenti di inviare messaggi. Se disattivato, l'editor disabilita Invia e la sua scorciatoia e il server rifiuta gli invii; le bozze possono comunque essere salvate.
admin-settings-key-mail-allow-reply = Consenti la risposta
admin-settings-help-mail-allow-reply = Consente agli utenti di rispondere e rispondere a tutti dal lettore e con le scorciatoie. Se disattivato, queste azioni non vengono proposte.
admin-settings-key-mail-allow-forward = Consenti l'inoltro
admin-settings-help-mail-allow-forward = Consente agli utenti di inoltrare i messaggi nel corpo o come allegato. Se disattivato, queste azioni non vengono proposte.
admin-settings-key-mail-allow-download = Consenti i download
admin-settings-help-mail-allow-download = Consente agli utenti di scaricare messaggi e allegati. Se disattivato, l'app web non propone alcun download e il server rifiuta l'esportazione dei messaggi.
admin-settings-key-mail-allow-print = Consenti la stampa
admin-settings-help-mail-allow-print = Consente agli utenti di stampare un messaggio dal lettore. Se disattivato, l'azione Stampa non viene proposta.
admin-settings-key-mail-show-spam = Mostra la cartella Spam
admin-settings-help-mail-show-spam = Mostra la cartella Spam o Junk e i messaggi contrassegnati come spam. Se disattivato, sono nascosti dall'elenco delle cartelle e dai risultati di ricerca; non viene eliminato nulla.
admin-settings-key-mail-show-auth-status = Mostra i risultati di autenticazione del mittente
admin-settings-help-mail-show-auth-status = Mostra nel lettore i risultati SPF, DKIM e DMARC dell'intestazione Authentication-Results.
admin-settings-key-mail-close-on-delete = Chiudi un messaggio quando viene eliminato
admin-settings-help-mail-close-on-delete = Chiude il messaggio, la scheda o la finestra separata aperti quando quel messaggio viene eliminato o spostato nel Cestino. Se disattivato, il lettore mantiene il comportamento precedente.
admin-settings-key-links-trusted-senders = Mittenti di link attendibili
admin-settings-help-links-trusted-senders = Caselle dei mittenti esatti i cui link nei messaggi verificati possono aprirsi senza consenso per singolo messaggio. Un indirizzo per voce.
admin-settings-key-appearance-font-size = Dimensione del carattere dell'interfaccia
admin-settings-help-appearance-font-size = Dimensione del carattere dell'interfaccia in pixel, da 12 a 24.
admin-settings-key-appearance-font-weight = Spessore del carattere dell'interfaccia
admin-settings-help-appearance-font-weight = Spessore di base del testo dell'interfaccia: 300, 400, 500 o 600. Il testo enfatizzato mantiene la propria gerarchia.
admin-settings-key-appearance-font-color = Colore del testo dell'interfaccia
admin-settings-help-appearance-font-color = Colore del testo dell'interfaccia come #RRGGBB. Lascia vuoto per seguire il tema.
admin-settings-key-appearance-low-contrast = Attenua il contrasto del testo
admin-settings-help-appearance-low-contrast = Attenua il contrasto del testo dell'interfaccia.
admin-settings-key-mail-message-list-order = Ordine dell'elenco dei messaggi
admin-settings-help-mail-message-list-order = Ordine dell'elenco dei messaggi: più recenti, meno recenti, mittente od oggetto.
admin-settings-key-mail-message-list-style = Stile dell'elenco dei messaggi
admin-settings-help-mail-message-list-style = Mostra l'elenco dei messaggi come schede o come righe con intestazioni di colonna.
admin-settings-key-mail-message-list-preview = Anteprime nell'elenco dei messaggi
admin-settings-help-mail-message-list-preview = Mostra estratti del corpo nell'elenco dei messaggi.
admin-settings-key-mail-after-delete = Dopo l'eliminazione di un messaggio
admin-settings-help-mail-after-delete = Quale messaggio mostra il lettore dopo l'eliminazione del messaggio aperto: il successivo, il precedente o nessuno.
admin-settings-key-reader-attachments-at-end = Allegati dopo il corpo
admin-settings-help-reader-attachments-at-end = Mostra gli allegati dopo il corpo del messaggio anziché prima.
admin-settings-key-notifications-ask-permission = Proponi l'autorizzazione per le notifiche
admin-settings-help-notifications-ask-permission = Proponi la richiesta di autorizzazione alle notifiche del browser agli utenti sui dispositivi che non hanno consentito né bloccato le notifiche.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Impostazioni
admin-settings-description = Valori per l'intero server, per dominio e per utente. Un valore bloccato non può essere modificato dagli utenti né da ambiti più specifici.
admin-settings-list-caption = Impostazioni
admin-settings-search = Cerca impostazioni
admin-settings-empty = Nessuna impostazione corrisponde alla ricerca.
admin-settings-unavailable = Questo server non offre ancora il modello delle impostazioni.
admin-settings-col-setting = Impostazione
admin-settings-col-value = Valore
admin-settings-col-source = Origine
admin-settings-col-overrides = Valori più specifici
admin-settings-open = Apri
admin-settings-overrides-none = Nessuno
admin-settings-overrides = { $domains } domini, { $users } utenti
admin-settings-not-found = Non esiste alcuna impostazione denominata { $key }.
admin-settings-value-on = Attivo
admin-settings-value-off = Disattivato
admin-settings-value-empty = Non impostato
admin-settings-value-list = { $count ->
    [one] { $count } elemento
   *[other] { $count } elementi
 }
admin-settings-detail-key = Chiave: { $key }
admin-settings-global = Per l'intero server
admin-settings-global-help = Questo valore si applica a ogni utente, a meno che un dominio o un utente abbia un valore più specifico, o che l'utente scelga il proprio dove consentito.
admin-settings-value-label = Valore
admin-settings-json-hint = JSON. Il server controlla il valore e indica la parte che rifiuta.
admin-settings-json-invalid = Questo JSON non è valido.
admin-settings-user-override = Gli utenti possono modificarlo
admin-settings-impact-user = { $count ->
    [one] { $count } scelta personale di un utente viene ignorata
   *[other] { $count } scelte personali degli utenti vengono ignorate
 }
admin-settings-impact-admin-user = { $count ->
    [one] { $count } valore per utente viene ignorato
   *[other] { $count } valori per utente vengono ignorati
 }
admin-settings-impact-profile = { $count ->
    [one] { $count } valore di profilo funzionalità viene ignorato
   *[other] { $count } valori di profilo funzionalità vengono ignorati
 }
admin-settings-impact-group = { $count ->
    [one] { $count } valore di gruppo viene ignorato
   *[other] { $count } valori di gruppo vengono ignorati
 }
admin-settings-domains = Valori di dominio
admin-settings-domains-help = Un valore di dominio si applica agli utenti di quel dominio al posto del valore per l'intero server.
admin-settings-domains-caption = Valori di dominio di { $setting }
admin-settings-domains-empty = Nessun dominio ha un valore proprio.
admin-settings-domain = Dominio
admin-settings-domain-value = Valore per questo dominio
admin-settings-domain-locked = Bloccato per gli utenti del dominio
admin-settings-domain-add = Aggiungi un valore di dominio
admin-settings-domain-save = Salva valore di dominio
admin-settings-domain-remove = Rimuovi
admin-settings-domain-remove-title = Rimuovere il valore di { $domain }?
admin-settings-domain-remove-body = Gli utenti di { $domain } tornano a ricevere il valore per l'intero server.
admin-settings-domain-required = Inserisca un nome di dominio.
admin-settings-locked-yes = Bloccato
admin-settings-locked-no = Non bloccato
admin-settings-users = Valori per utente
admin-settings-users-help = Un valore impostato da un amministratore per un singolo utente. Per i limiti l'utente è indicato dall'indirizzo.
admin-settings-users-caption = Valori per utente di { $setting }
admin-settings-users-empty = Nessun utente ha un valore impostato da un amministratore.
admin-settings-user = Utente
admin-settings-user-hint = ID dell'account oppure, per i limiti, l'indirizzo.
admin-settings-user-value = Valore per questo utente
admin-settings-user-add = Aggiungi un valore per utente
admin-settings-user-save = Salva valore utente
admin-settings-user-required = Inserisca un utente.
admin-settings-user-remove-title = Rimuovere il valore di { $user }?
admin-settings-user-remove-body = L'utente riceve il valore del suo dominio o il valore per l'intero server.
admin-settings-source-default = Predefinito integrato
admin-settings-source-envFallback = Variabile d'ambiente (nessun valore memorizzato)
admin-settings-source-envPin = Variabile d'ambiente
admin-settings-source-global = Memorizzato per l'intero server
admin-settings-cat-interface = Interfaccia
admin-settings-cat-appearance = Aspetto
admin-settings-cat-mail = Posta
admin-settings-cat-folders = Disposizione delle cartelle
admin-settings-cat-notifications = Notifiche
admin-settings-cat-offline = Uso offline
admin-settings-cat-assistant = Assistente IA
admin-settings-cat-upstream = Server di posta
admin-settings-cat-signin = Accesso
admin-settings-cat-admin = Console di amministrazione
admin-settings-cat-passwords = Password
admin-settings-cat-limits = Limiti e utilizzo
admin-settings-cat-plugins = Plugin
admin-settings-cat-features = Autorizzazioni delle funzionalità
admin-settings-cat-integrations = Autorizzazioni delle integrazioni
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Consente file, condivisione e operazioni su calendario, contatti e attività di Nextcloud collegati all'account.
admin-settings-key-cap-integration-google = Bridge Google
admin-settings-help-cap-integration-google = Consente le operazioni del bridge Google, inclusi l'ottenimento e il rinnovo dei token.
admin-settings-key-cap-integration-microsoft = Bridge Microsoft
admin-settings-help-cap-integration-microsoft = Consente le operazioni del bridge Microsoft, inclusi l'ottenimento e il rinnovo dei token.
admin-settings-key-cap-integration-family = Integrazione singola
admin-settings-help-cap-integration-family = Autorizzazione per l'identificatore di una singola integrazione; la copertura a runtime è elencata nel catalogo delle funzionalità.
admin-settings-key-cap-ui-plugin-family = Plugin UI singolo
admin-settings-help-cap-ui-plugin-family = Autorizzazione per un plugin UI. Usi l'esatta chiave di criterio del catalogo delle integrazioni.
admin-settings-key-cap-ai-assistant = Assistente IA
admin-settings-help-cap-ai-assistant = Consente la configurazione e l'invocazione dell'assistente.
admin-settings-key-cap-plugins = Plugin
admin-settings-help-cap-plugins = Consente le operazioni dei plugin UI e del motore.
admin-settings-key-cap-pim-calendar = Calendario
admin-settings-help-cap-pim-calendar = Consente i metodi del calendario e le sottoscrizioni.
admin-settings-key-cap-pim-contacts = Contatti
admin-settings-help-cap-pim-contacts = Consente i metodi di contatti e rubrica.
admin-settings-key-cap-pim-tasks = Attività
admin-settings-help-cap-pim-tasks = Consente i metodi delle attività.
admin-settings-key-cap-pim-notes = Note
admin-settings-help-cap-pim-notes = Consente i metodi delle note.
admin-settings-key-cap-export = Esportazione
admin-settings-help-cap-export = Consente le esportazioni di posta e PIM.
admin-settings-key-cap-import = Importazione
admin-settings-help-cap-import = Consente le importazioni di posta e PIM.
admin-settings-key-cap-api-keys = Chiavi API
admin-settings-help-cap-api-keys = Criterio delle chiavi API. Consulti il catalogo delle funzionalità per la copertura dell'applicazione.
admin-settings-key-cap-oauth-apps = Applicazioni OAuth
admin-settings-help-cap-oauth-apps = Criterio delle applicazioni OAuth. Consulti il catalogo delle funzionalità per la copertura dell'applicazione.
admin-settings-key-cap-rules = Regole di posta
admin-settings-help-cap-rules = Consente i metodi gestiti per le regole di posta e la risposta automatica.
admin-settings-key-cap-rules-forward = Inoltro tramite regole
admin-settings-help-cap-rules-forward = Criterio di inoltro. Consulti il catalogo delle funzionalità per la copertura dell'applicazione.
admin-settings-key-cap-remote-images = Immagini remote
admin-settings-help-cap-remote-images = Funzionalità delle immagini remote. Le impostazioni di privacy esistenti continuano a regolare le richieste; controlli la copertura della funzionalità.
admin-settings-key-cap-user-background = Sfondo personalizzato
admin-settings-help-cap-user-background = Funzionalità dello sfondo personalizzato, disattivata per impostazione predefinita. Controlli la copertura della funzionalità.
admin-settings-key-cap-user-css = CSS personalizzato
admin-settings-help-cap-user-css = Funzionalità del CSS personalizzato, disattivata per impostazione predefinita. Controlli la copertura della funzionalità.
admin-settings-key-limit-attachment-bytes = Limite di dimensione degli allegati
admin-settings-help-limit-attachment-bytes = Criterio degli allegati in byte, predefinito 50000000. L'applicazione a runtime è ancora in sospeso.
admin-settings-key-limit-send-per-hour = Limite di invio orario
admin-settings-help-limit-send-per-hour = Criterio di invio all'ora; zero significa illimitato. L'applicazione a runtime è ancora in sospeso.
admin-settings-key-limit-recipients-per-message = Destinatari per messaggio
admin-settings-help-limit-recipients-per-message = Criterio sul numero di destinatari; zero significa illimitato. L'applicazione a runtime è ancora in sospeso.
admin-settings-cat-import = Importazione
admin-settings-cat-webhooks = Webhook
admin-settings-cat-realtime = Aggiornamenti in tempo reale
admin-settings-cat-privacy = Privacy
admin-settings-cat-observability = Osservabilità
admin-settings-cat-egress = Connessioni in uscita
admin-settings-cat-updates = Aggiornamenti
admin-settings-cat-backups = Backup
admin-settings-cat-about = Informazioni
admin-settings-cat-config = Configurazione
admin-settings-cat-jobs = Processi
admin-settings-cat-maintenance = Manutenzione
admin-settings-cat-health = Controlli di integrità
admin-settings-cat-diagnostics = Diagnostica
admin-settings-cat-users = Utenti
admin-settings-cat-server-text = Testo del server
admin-settings-key-ui-language = Lingua
admin-settings-help-ui-language = La lingua del client di posta per gli utenti che non ne hanno scelta una. "Come il browser" segue le lingue di ciascun browser.
admin-settings-key-ui-notify-new-version = Avviso di nuova versione
admin-settings-help-ui-notify-new-version = Attivo (predefinito) mostra agli utenti connessi un avviso con "Ricarica ora" e "Più tardi" quando, dopo il caricamento della loro pagina, è stata distribuita una versione più recente dell'app web. Disattivato non lo mostra mai; gli utenti ricevono il nuovo codice al successivo ricaricamento.
admin-settings-key-ui-undo-history-size = Dimensione della cronologia di annulla e ripeti
admin-settings-help-ui-undo-history-size = Quante azioni recenti sulla posta (archivia, elimina, sposta, segna come letto, contrassegna, etichetta e simili) i pulsanti Annulla e Ripeti ricordano, da 0 a 50. 0 disattiva i pulsanti e la scorciatoia Ctrl+Z. Il valore predefinito è 50. Gli utenti possono scegliere una dimensione propria, mai superiore al massimo indicato sotto, a meno che un amministratore la blocchi.
admin-settings-key-folders-reorder-enabled = Trascinamento delle cartelle
admin-settings-help-folders-reorder-enabled = Consente di riordinare le cartelle trascinandole nella barra laterale e in Impostazioni. Disattivato nasconde le maniglie di trascinamento e i comandi Sposta. Gli utenti possono disattivarlo per sé, a meno che sia bloccato.
admin-settings-key-folders-reorder-allow-nesting = Annidamento delle cartelle
admin-settings-help-folders-reorder-allow-nesting = Consente di rilasciare una cartella su un'altra per annidarla, o di riportarla al livello superiore. Disattivato mantiene ogni cartella sotto il suo genitore attuale e consente di riordinare solo tra cartelle sorelle.
admin-settings-key-folders-reorder-allow-server-move = Sposta le cartelle sul server
admin-settings-help-folders-reorder-allow-server-move = Consente che annidare o separare una cartella cambi il suo genitore reale sul server di posta, quindi la modifica vale ovunque. Disattivato mantiene questi spostamenti solo come disposizione in questa app.
admin-settings-key-folders-reorder-allow-system-folders = Sposta le cartelle di sistema
admin-settings-help-folders-reorder-allow-system-folders = Consente di trascinare Posta in arrivo, Bozze, Inviata, Posta indesiderata, Cestino e Archivio per riordinarle. Disattivato le mantiene fisse in alto. Vengono riordinate solo in questa app, mai rinominate né spostate sul server.
admin-settings-key-folders-reorder-max-depth = Profondità di annidamento delle cartelle
admin-settings-help-folders-reorder-max-depth = Il livello di annidamento più profondo che un trascinamento può creare, da 1 a 16. Uno spostamento che porterebbe una cartella o una sua sottocartella più in profondità viene rifiutato.
admin-settings-key-folders-order-locked = Blocca l'ordine delle cartelle
admin-settings-help-folders-order-locked = Blocca l'ordine delle cartelle per tutti nell'ambito. Il trascinamento e i comandi Sposta sono disattivati e gli utenti vedono un suggerimento di blocco. Le cartelle fissate e l'ordine impostato dall'amministratore restano.
admin-settings-key-folders-pinned-admin = Cartelle fissate d'ufficio
admin-settings-help-folders-pinned-admin = Cartelle mostrate in un gruppo Fissate in cima alla barra laterale di ogni utente. Una voce per riga, come role:inbox o un percorso di cartella come Shared/Support. Gli utenti non possono sbloccarle né spostarle. Una voce senza cartella corrispondente viene saltata.
admin-settings-key-folders-order-default = Ordine predefinito delle cartelle
admin-settings-help-folders-order-default = L'ordine iniziale delle cartelle per gli utenti che non le hanno organizzate, dalla prima all'ultima. Una voce per riga, come role:<role> o un percorso di cartella. Le cartelle non elencate seguono nell'ordine del server. L'ordine personale di un utente sostituisce questo.
admin-settings-key-folders-new-folder-placement = Posizione delle nuove cartelle
admin-settings-help-folders-new-folder-placement = Dove viene collocata, tra le cartelle sorelle, una cartella che compare sul server: alla fine o in ordine alfabetico.
admin-settings-key-ui-undo-history-max = Massimo della cronologia di annulla e ripeti
admin-settings-help-ui-undo-history-max = Il massimo di azioni che un utente può conservare nella cronologia di Annulla e Ripeti, da 0 a 50. La dimensione personale di un utente viene ridotta a questo valore. 0 disattiva la cronologia per tutti. La cronologia risiede nella scheda del browser e viene cancellata alla disconnessione.
admin-settings-key-ui-bar-mail-top-bar = Barra degli strumenti della posta
admin-settings-help-ui-bar-mail-top-bar = "pinned" mantiene sempre visibile la barra degli strumenti della posta con i controlli di disposizione. "auto" la nasconde dietro una maniglia sottile che la apre quando l'utente ci punta, la tocca o ci arriva con Tab.
admin-settings-key-ui-bar-workspace-tabs = Schede dell'area di lavoro
admin-settings-help-ui-bar-workspace-tabs = "pinned" mantiene sempre visibile la barra delle schede dell'area di lavoro. "auto" mostra solo una maniglia sottile con il nome della scheda attiva finché l'utente non la apre.
admin-settings-key-ui-bar-mail-ribbon = Barra multifunzione della posta
admin-settings-help-ui-bar-mail-ribbon = Per la disposizione con barra multifunzione. "pinned" la mantiene aperta. "auto" mostra solo i nomi delle schede; una scheda apre la barra sopra l'elenco dei messaggi.
admin-settings-key-ui-bar-composer-ribbon = Barra multifunzione dell'editor
admin-settings-help-ui-bar-composer-ribbon = Per ogni editor di messaggi. "pinned" mantiene aperta la barra dell'editor. "auto" mostra solo i nomi delle schede; aprire una scheda spinge in basso il corpo del messaggio invece di coprirlo.
admin-settings-key-appearance-theme = Tema
admin-settings-help-appearance-theme = Il tema di colori che gli utenti vedono finché non ne scelgono uno proprio. Bloccarlo vincola tutti gli utenti a questo tema.
admin-settings-key-appearance-accent = Colore di risalto
admin-settings-help-appearance-accent = Il colore di risalto del tema. Vuoto significa il colore di risalto proprio del tema.
admin-settings-key-appearance-badge-color = Colore del badge dei non letti
admin-settings-help-appearance-badge-color = Il colore dei badge con il conteggio dei non letti. Default mantiene il colore del tema; Accent segue il colore di risalto; accent-lighter e accent-darker usano il risalto più chiaro o più scuro del 20%, con testo nero o bianco. Un blocco prevale sulla scelta dell'account.
admin-settings-key-appearance-density = Densità
admin-settings-help-appearance-density = Quanto spazio occupano elenchi e controlli: compact, cozy o relaxed.
admin-settings-key-appearance-font = Carattere dell'interfaccia
admin-settings-key-packages-allowed = Tracciamento dei pacchi consentito
admin-settings-help-packages-allowed = Consente il tracciamento dei pacchi per gli account interessati. Il provider deve essere abilitato anche in Tracciamento dei pacchi e ogni utente deve aderire. Disattivare questa impostazione impedisce subito nuove chiamate al provider.
admin-settings-help-appearance-font = Scelga un carattere incluso per gli account senza preferenza di carattere, oppure blocchi la scelta per tutti gli account interessati. I caratteri vengono caricati da questo server; non viene fatta alcuna richiesta a Google Fonts. Default usa il carattere del tema.
admin-settings-key-appearance-brand-name = Nome del prodotto
admin-settings-help-appearance-brand-name = Il nome mostrato nella pagina di accesso e nella scheda del browser.
admin-settings-key-login-notice = Avviso di accesso
admin-settings-help-login-notice = Un breve testo mostrato nella pagina di accesso, per esempio un contatto di assistenza. Lasci vuoto per nessuno.
admin-settings-key-mail-read-receipts = Conferme di lettura
admin-settings-key-receipts-dsn-allowed = Richieste di stato di consegna
admin-settings-help-receipts-dsn-allowed = Consente agli account di richiedere le notifiche di stato di consegna SMTP. Ciò non richiede né dimostra che un messaggio sia stato letto.
admin-settings-help-mail-read-receipts = Che cosa accade quando un messaggio aperto chiede una conferma di lettura: chiedere all'utente, non inviare mai o inviare sempre.
admin-settings-key-notifications-enabled = Notifiche di nuova posta
admin-settings-help-notifications-enabled = Se gli utenti ricevono notifiche per la nuova posta. Non modifica le loro regole di notifica.
admin-settings-key-offline-retention-days = Durata della cache offline
admin-settings-help-offline-retention-days = Per quanti giorni un dispositivo conserva i messaggi per l'uso offline. 0 significa nessun limite di durata.
admin-settings-key-offline-budget-mb = Dimensione della cache offline
admin-settings-help-offline-budget-mb = Quanti megabyte un dispositivo può usare per la posta offline. 0 significa nessuna cache offline.
admin-settings-key-assistant-enabled = Assistente IA
admin-settings-help-assistant-enabled = Se gli utenti vedono l'assistente IA. L'interruttore di distribuzione in Assistente IA continua a valere sopra questo.
admin-settings-key-assist-profile = Profilo del provider IA
admin-settings-help-assist-profile = L'ID del profilo provider che serve questi utenti (Assistente IA > Profili provider). Vuoto usa l'endpoint della distribuzione. Un ID di profilo inesistente disattiva l'assistente IA per questi utenti.
admin-settings-key-upstream-default = Server di posta predefinito
admin-settings-help-upstream-default = Il server di posta a cui accede un indirizzo quando nessun dominio ne indica uno. La tabella dei domini prevale su questo.
admin-settings-key-upstream-proxy-timeout-secs = Timeout del server di posta
admin-settings-help-upstream-proxy-timeout-secs = Quanto può durare una richiesta al server di posta, in secondi.
admin-settings-key-signin-new-users = Nuovi utenti
admin-settings-help-signin-new-users = Chi può creare un account al primo accesso: chiunque (open), solo gli indirizzi aggiunti in anticipo (preprovisioned), questi più gli utenti importati (provisioned), o nessun nuovo utente (closed). Gli account esistenti non sono mai interessati. Si modifica in Criteri di accesso.
admin-settings-key-signin-new-users-mail = Accesso con password: nuovi utenti
admin-settings-help-signin-new-users-mail = Regola per un nuovo utente di casella di posta che accede con una password (IMAP, POP3, Exchange). Inherit usa Nuovi utenti.
admin-settings-key-signin-new-users-sso = Single sign-on: nuovi utenti
admin-settings-help-signin-new-users-sso = Regola per un nuovo utente che accede tramite single sign-on o un'intestazione attendibile. Inherit usa Nuovi utenti.
admin-settings-key-signin-new-users-linked = Caselle collegate: nuovi utenti
admin-settings-help-signin-new-users-linked = Regola per il collegamento di una casella che qui non ha ancora un account, valutata in base al dominio della casella collegata. Inherit usa Nuovi utenti.
admin-settings-key-signin-new-users-linked-imap = Caselle IMAP collegate: nuovi utenti
admin-settings-help-signin-new-users-linked-imap = Regola solo per le caselle IMAP collegate. Inherit usa Caselle collegate.
admin-settings-key-signin-new-users-linked-pop3 = Caselle POP3 collegate: nuovi utenti
admin-settings-help-signin-new-users-linked-pop3 = Regola solo per le caselle POP3 collegate. Inherit usa Caselle collegate.
admin-settings-key-signin-new-users-linked-ews = Caselle Exchange collegate: nuovi utenti
admin-settings-help-signin-new-users-linked-ews = Regola solo per le caselle Exchange (EWS) collegate. Inherit usa Caselle collegate.
admin-settings-key-signin-preprovision-ttl-days = Durata dell'utente pre-aggiunto (giorni)
admin-settings-help-signin-preprovision-ttl-days = Per quanto tempo resta valido un utente in sospeso aggiunto in anticipo quando non è indicata un'altra durata.
admin-settings-key-signin-preprovision-max-ttl-days = Durata massima dell'utente pre-aggiunto (giorni)
admin-settings-help-signin-preprovision-max-ttl-days = La durata massima che un amministratore può assegnare a un utente in sospeso aggiunto in anticipo.
admin-settings-key-signin-preprovision-domain-wide = Consenti voci pre-aggiunte per un intero dominio
admin-settings-help-signin-preprovision-domain-wide = Consente voci che ammettono chiunque di un intero dominio, fino a un limite di utilizzi. Disattivato per impostazione predefinita.
admin-settings-key-signin-preprovision-retention-days = Conserva gli utenti in sospeso scaduti (giorni)
admin-settings-help-signin-preprovision-retention-days = Per quanto tempo un utente in sospeso scaduto viene conservato prima di essere rimosso.
admin-settings-key-signin-domain-mode = Domini di accesso
admin-settings-help-signin-domain-mode = Se qualsiasi dominio può accedere o solo i domini elencati. Si modifica in Criteri di accesso.
admin-settings-key-signin-refusal-detail = Messaggi di rifiuto
admin-settings-help-signin-refusal-detail = Se un accesso rifiutato ne indica il motivo (account disattivato, indirizzo non consentito) o dà la stessa risposta di una password errata. Si modifica in Criteri di accesso.
admin-settings-key-signin-ldap-bind-precheck = Verifica LDAP prima dell'accesso
admin-settings-help-signin-ldap-bind-precheck = Verifica la password nella directory prima del server di posta. Si modifica in Integrazioni, dopo un test di bind.
admin-settings-key-twofa-pending-ttl-secs = Limite di tempo del secondo fattore
admin-settings-help-twofa-pending-ttl-secs = Quanto tempo ha l'utente per inserire il secondo fattore dopo la password, in secondi.
admin-settings-key-twofa-max-attempts = Tentativi del secondo fattore
admin-settings-help-twofa-max-attempts = Quanti codici errati del secondo fattore sono consentiti prima che l'accesso ricominci.
admin-settings-key-admin-session-idle-secs = Timeout di inattività della console
admin-settings-help-admin-session-idle-secs = Quanto dura una sessione della console di amministrazione senza richieste, in secondi.
admin-settings-key-admin-session-absolute-secs = Limite di sessione della console
admin-settings-help-admin-session-absolute-secs = La durata massima di una sessione della console di amministrazione, in secondi, attiva o no.
admin-settings-key-admin-session-rotate-secs = Rotazione dell'ID di sessione della console
admin-settings-help-admin-session-rotate-secs = Ogni quanto, in secondi, la console di amministrazione sostituisce l'ID di sessione durante l'uso (0 = solo dopo una conferma di riautenticazione). La rotazione non estende mai i limiti di inattività o assoluto.
admin-settings-key-passwd-policy-min-length = Lunghezza minima della password
admin-settings-help-passwd-policy-min-length = Il numero minimo di caratteri di una nuova password.
admin-settings-key-passwd-policy-require-upper = Richiedi una lettera maiuscola
admin-settings-help-passwd-policy-require-upper = Una nuova password deve contenere una lettera maiuscola.
admin-settings-key-passwd-policy-require-lower = Richiedi una lettera minuscola
admin-settings-help-passwd-policy-require-lower = Una nuova password deve contenere una lettera minuscola.
admin-settings-key-passwd-policy-require-digit = Richiedi una cifra
admin-settings-help-passwd-policy-require-digit = Una nuova password deve contenere una cifra.
admin-settings-key-passwd-policy-require-symbol = Richiedi un simbolo
admin-settings-help-passwd-policy-require-symbol = Una nuova password deve contenere un carattere che non sia una lettera né una cifra.
admin-settings-key-limit-storage-bytes = Limite di archiviazione
admin-settings-help-limit-storage-bytes = Il massimo di byte che un utente può archiviare su questo server. 0 significa nessun limite.
admin-settings-key-limit-send-per-day = Messaggi inviati al giorno
admin-settings-help-limit-send-per-day = Il massimo di messaggi che un utente può inviare in un giorno. 0 significa nessun limite.
admin-settings-key-limit-ai-calls-per-day = Chiamate all'assistente IA al giorno
admin-settings-help-limit-ai-calls-per-day = Il massimo di richieste all'assistente IA che un utente può fare in un giorno. 0 significa nessun limite.
admin-settings-key-quota-warn-thresholds = Livelli di avviso di utilizzo
admin-settings-help-quota-warn-thresholds = Le percentuali di un limite a cui gli utenti vengono avvisati, dalla più bassa.
admin-settings-key-quota-warn-enabled = Avvisi di utilizzo
admin-settings-help-quota-warn-enabled = Se gli utenti vengono avvisati quando si avvicinano a un limite.
admin-settings-key-usage-retention-days = Cronologia di utilizzo
admin-settings-help-usage-retention-days = Per quanti giorni vengono conservati i conteggi giornalieri di utilizzo.
admin-settings-key-usage-meter-api = Conta l'uso delle API
admin-settings-help-usage-meter-api = Se le richieste API vengono contate nei dati di utilizzo.
admin-settings-key-usage-measure-interval-hours = Intervallo di misurazione dell'archiviazione
admin-settings-help-usage-measure-interval-hours = Ogni quanto vengono misurati i dati archiviati di ogni utente, in ore.
admin-settings-key-assistant-rate-limit-per-min = Limite di frequenza dell'assistente IA
admin-settings-help-assistant-rate-limit-per-min = Il massimo di richieste all'assistente IA al minuto per l'intero server. 0 significa nessun limite.
admin-settings-key-plugins-kv-max-key-bytes = Dimensione della chiave del plugin
admin-settings-help-plugins-kv-max-key-bytes = La chiave più lunga che un plugin può memorizzare, in byte.
admin-settings-key-plugins-kv-max-value-bytes = Dimensione del valore del plugin
admin-settings-help-plugins-kv-max-value-bytes = Il valore più grande che un plugin può memorizzare sotto una chiave, in byte.
admin-settings-key-plugins-kv-max-total-bytes = Archiviazione del plugin
admin-settings-help-plugins-kv-max-total-bytes = Il massimo di byte che un plugin può memorizzare in totale.
admin-settings-key-plugins-kv-max-keys = Chiavi del plugin
admin-settings-help-plugins-kv-max-keys = Il massimo di chiavi che un plugin può memorizzare.
admin-settings-key-import-max-messages = Numero di messaggi dell'importazione
admin-settings-help-import-max-messages = Il massimo di messaggi che un'importazione può portare.
admin-settings-key-import-max-message-bytes = Dimensione dei messaggi dell'importazione
admin-settings-help-import-max-message-bytes = Il messaggio più grande che un'importazione accetta, in byte.
admin-settings-key-webhooks-max-attempts = Tentativi del webhook
admin-settings-help-webhooks-max-attempts = Quante volte viene tentata una consegna webhook prima di essere abbandonata.
admin-settings-key-webhooks-timeout-secs = Timeout del webhook
admin-settings-help-webhooks-timeout-secs = Quanto può durare una consegna webhook, in secondi.
admin-settings-key-outbox-max-attempts = Tentativi di invio
admin-settings-help-outbox-max-attempts = Quante volte un messaggio viene consegnato al server di posta prima di essere segnalato come non inviato.
admin-settings-key-realtime-recheck-secs = Nuova verifica degli aggiornamenti in tempo reale
admin-settings-help-realtime-recheck-secs = Ogni quanto una connessione aperta di aggiornamenti in tempo reale verifica che la sessione sia ancora valida, in secondi.
admin-settings-key-privacy-watermark-enabled = Filigrana del lettore
admin-settings-help-privacy-watermark-enabled = Mostra una filigrana con l'indirizzo del lettore sopra i messaggi aperti.
admin-settings-key-privacy-watermark-opacity = Intensità della filigrana
admin-settings-help-privacy-watermark-opacity = Quanto è visibile la filigrana, da 0,02 (tenue) a 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Cronologia del tracciamento delle aperture
admin-settings-help-privacy-open-tracking-retention-days = Per quanti giorni vengono conservati i record di tracciamento della lettura.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Token di tracciamento non inviati
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Per quante ore un token di tracciamento attende l'invio del suo messaggio prima di essere eliminato.
admin-settings-key-masked-email-domain = Dominio dell'indirizzo mascherato

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-user-override-help = Se attivo, gli utenti possono scegliere un proprio valore nelle loro Impostazioni. Se disattivato, per loro vale il valore impostato qui o per il loro dominio.
admin-settings-lock-help = Se bloccato, i valori di dominio e le scelte degli utenti vengono ignorati e questo valore vale per tutti.
admin-settings-save = Salva
admin-settings-saved = Salvato. Si applica entro 30 secondi su ogni processo del server.
admin-settings-saved-restart = Salvato. Si applica dopo un riavvio.
admin-settings-dedicated = Questa impostazione si modifica in { $section }, che ne verifica prima l'effetto.
admin-settings-dedicated-link = Apri { $section }
admin-settings-pinned = Impostato dalla variabile d'ambiente { $name }. Il valore memorizzato qui si applica quando la variabile non è impostata.
admin-settings-lock-title = Bloccare { $setting }?
admin-settings-lock-body = Il blocco ignora i valori più specifici elencati sotto. Vengono conservati e tornano a valere quando sblocca.
admin-settings-lock-confirm = Blocca
admin-settings-impact-domain = { $count ->
    [one] { $count } valore di dominio viene ignorato
   *[other] { $count } valori di dominio vengono ignorati
 }
admin-settings-help-masked-email-domain = Il dominio sotto cui vengono creati gli indirizzi mascherati. Lasci vuoto per disattivare gli indirizzi mascherati.
admin-settings-key-abuse-report-address = Indirizzo per le segnalazioni di abuso
admin-settings-help-abuse-report-address = Dove vengono inviate le segnalazioni di spam e abuso.
admin-settings-key-dlp-admin-address = Avvisi di perdita di dati
admin-settings-help-dlp-admin-address = Dove vengono inviati gli avvisi di prevenzione della perdita di dati.
admin-settings-key-observability-error-forward-url = Indirizzo di inoltro degli errori
admin-settings-help-observability-error-forward-url = Un indirizzo https a cui vengono inoltrati i report di errore del client. Lasci vuoto per nessuno.
admin-settings-key-egress-probe-url = Indirizzo di prova in uscita
admin-settings-help-egress-probe-url = L'indirizzo https usato per provare un proxy in uscita.
admin-settings-key-push-vapid-contact = Contatto push
admin-settings-help-push-vapid-contact = Il contatto (mailto: o https:) fornito ai servizi push con ogni notifica.
admin-settings-key-push-quiet-hours = Ore di silenzio push
admin-settings-help-push-quiet-hours = Ore (UTC) in cui non viene inviata alcuna notifica push. Lasci vuoto per nessuna.
admin-settings-key-updates-check = Controlla gli aggiornamenti
admin-settings-help-updates-check = Se il server controlla una volta al giorno la presenza di una versione più recente e la mostra nella dashboard.
admin-settings-key-updates-feed-url = Feed delle versioni
admin-settings-help-updates-feed-url = L'indirizzo https letto dal controllo degli aggiornamenti.
admin-settings-key-updates-apply = Applica gli aggiornamenti
admin-settings-help-updates-apply = Se un aggiornamento può essere avviato dalla console tramite l'agente di aggiornamento dell'host.
admin-settings-key-backup-retention-count = Backup conservati
admin-settings-help-backup-retention-count = Quanti backup del database vengono conservati.
admin-settings-key-about-security-contact = Contatto per la sicurezza
admin-settings-help-about-security-contact = Il contatto in security.txt (mailto: o https:).
admin-settings-key-about-security-policy-url = Criterio di sicurezza
admin-settings-help-about-security-policy-url = L'indirizzo del criterio di sicurezza, elencato in security.txt.
admin-settings-key-about-project-links = Link di Informazioni
admin-settings-help-about-project-links = Link mostrati nella pagina Informazioni, ciascuno con un'etichetta e un indirizzo https.
admin-settings-key-config-snapshots-kept = Snapshot della configurazione
admin-settings-help-config-snapshots-kept = Quanti snapshot della configurazione vengono conservati.
admin-settings-key-jobs-history-days = Cronologia dei processi
admin-settings-help-jobs-history-days = Quanti giorni di esecuzioni dei processi vengono conservati.
admin-settings-key-maintenance-announce-minutes-default = Anticipo dell'avviso di manutenzione
admin-settings-help-maintenance-announce-minutes-default = Quanti minuti prima di una finestra di manutenzione gli utenti ne vengono informati.
admin-settings-key-maintenance-default-message = Messaggio di manutenzione
admin-settings-help-maintenance-default-message = Il testo che gli utenti vedono durante la manutenzione, per lingua.
admin-settings-key-health-disk-free-min-percent = Soglia minima di spazio libero su disco
admin-settings-help-health-disk-free-min-percent = Il controllo di prontezza fallisce quando lo spazio libero su disco scende sotto questa percentuale.
admin-settings-key-observability-otlp-metrics = Invia metriche via OTLP
admin-settings-help-observability-otlp-metrics = Invia le metriche del server al collector OTLP impostato all'avvio.
admin-settings-key-metrics-drilldown-top-n = Righe di dettaglio delle metriche
admin-settings-help-metrics-drilldown-top-n = Quante righe mostra un dettaglio delle metriche.
admin-settings-key-metrics-rollup-days = Cronologia delle metriche
admin-settings-help-metrics-rollup-days = Quanti giorni di riepiloghi orari delle metriche vengono conservati.
admin-settings-key-observability-client-timing-enabled = Tempi di caricamento del browser
admin-settings-help-observability-client-timing-enabled = Raccoglie i tempi di caricamento delle pagine dai browser connessi come istogrammi aggregati anonimi. Nulla identifica una persona, una sessione o una pagina. Disattivi per interrompere la raccolta.
admin-settings-key-observability-client-timing-sample-rate-percent = Frequenza di campionamento dei tempi di caricamento del browser
admin-settings-help-observability-client-timing-sample-rate-percent = Percentuale di sessioni del browser che segnalano i tempi di caricamento (da 1 a 100).
admin-settings-key-observability-client-timing-min-samples = Campioni minimi dei tempi di caricamento
admin-settings-help-observability-client-timing-min-samples = La mediana e il 95° percentile dei tempi di caricamento restano nascosti finché un periodo non ha almeno questo numero di campioni.
admin-settings-key-observability-client-timing-folder-loads = Tempi di caricamento delle cartelle
admin-settings-help-observability-client-timing-folder-loads = Misura quanto tempo impiega una cartella a mostrare l'elenco dei messaggi, distinguendo se proviene dalla cache del browser o dalla rete. Attivo per impostazione predefinita. Gli utenti possono disattivarlo per sé, a meno che questa impostazione sia bloccata. Richiede che i tempi di caricamento del browser siano attivi.
admin-settings-key-observability-client-timing-message-loads = Tempi di caricamento dei messaggi
admin-settings-help-observability-client-timing-message-loads = Misura quanto tempo impiega un messaggio a mostrare le intestazioni e il corpo sanificato, distinguendo se era in cache o precaricato, o proviene dalla rete. Attivo per impostazione predefinita. Gli utenti possono disattivarlo per sé, a meno che questa impostazione sia bloccata. Richiede che i tempi di caricamento del browser siano attivi.
admin-settings-key-diagnostics-retention-days = Cronologia della diagnostica
admin-settings-help-diagnostics-retention-days = Quanti giorni di esecuzioni diagnostiche vengono conservati.
admin-settings-key-data-export-link-hours = Finestra di download dell'esportazione dei dati
admin-settings-help-data-export-link-hours = Per quante ore un'esportazione dei dati può essere scaricata prima di essere eliminata.
admin-settings-key-dsr-enabled = Richieste sui dati
admin-settings-help-dsr-enabled = Se le persone possono presentare richieste sui dati (accesso, download, rettifica, cancellazione, limitazione, opposizione) da Impostazioni.
admin-settings-key-dsr-type-access = Accetta richieste di accesso
admin-settings-help-dsr-type-access = Se il tipo di richiesta di accesso viene offerto agli utenti.
admin-settings-key-dsr-type-portability = Accetta richieste di download (copia portabile)
admin-settings-help-dsr-type-portability = Se il tipo di richiesta di download (copia portabile) viene offerto agli utenti.
admin-settings-key-dsr-type-rectification = Accetta richieste di rettifica
admin-settings-help-dsr-type-rectification = Se il tipo di richiesta di rettifica viene offerto agli utenti.
admin-settings-key-dsr-type-erasure = Accetta richieste di cancellazione
admin-settings-help-dsr-type-erasure = Se il tipo di richiesta di cancellazione viene offerto agli utenti.
admin-settings-key-dsr-type-restriction = Accetta richieste di limitazione
admin-settings-help-dsr-type-restriction = Se il tipo di richiesta di limitazione viene offerto agli utenti.
admin-settings-key-dsr-type-objection = Accetta richieste di opposizione
admin-settings-help-dsr-type-objection = Se il tipo di richiesta di opposizione viene offerto agli utenti.
admin-settings-key-dsr-type-other = Accetta altre richieste
admin-settings-help-dsr-type-other = Se il tipo di richiesta generica viene offerto agli utenti.
admin-settings-key-dsr-approval-export = Approvazione delle richieste di accesso e download
admin-settings-help-dsr-approval-export = auto avvia l'esportazione appena la richiesta è confermata. admin attende che un amministratore la approvi.
admin-settings-key-dsr-approval-erasure = Approvazione delle richieste di cancellazione
admin-settings-help-dsr-approval-erasure = auto pianifica la cancellazione appena la richiesta è confermata. admin attende che un amministratore la approvi.
admin-settings-key-dsr-approval-other = Gestione delle altre richieste
admin-settings-help-dsr-approval-other = auto contrassegna la richiesta come in corso appena confermata. admin attende che un amministratore la approvi.
admin-settings-key-dsr-reauth-required = Nuova autenticazione per le richieste
admin-settings-help-dsr-reauth-required = Richiede una password o un secondo fattore recenti per presentare una richiesta di esportazione o cancellazione e per creare un link di download.
admin-settings-key-dsr-confirmation-required = Messaggio di conferma
admin-settings-help-dsr-confirmation-required = Invia un messaggio di conferma con un codice alla posta in arrivo del richiedente; la richiesta non procede finché il codice non viene inserito.
admin-settings-key-dsr-identity-verification-required = Fase di verifica dell'identità
admin-settings-help-dsr-identity-verification-required = Un amministratore deve registrare che l'identità del richiedente è stata verificata prima che la richiesta sia approvata.
admin-settings-key-dsr-erasure-grace-days = Periodo di tolleranza della cancellazione
admin-settings-help-dsr-erasure-grace-days = Giorni tra l'approvazione e la cancellazione. In questo periodo il richiedente può annullare. 0 esegue la cancellazione al prossimo passaggio del worker.
admin-settings-key-dsr-sla-days = Giorni per rispondere a una richiesta
admin-settings-help-dsr-sla-days = Il numero di giorni dalla presentazione alla scadenza mostrata a utenti e amministratori.
admin-settings-key-dsr-sla-max-extension-days = Proroga massima
admin-settings-help-dsr-sla-max-extension-days = Il massimo di giorni che un amministratore può aggiungere alla scadenza di una richiesta, con un motivo registrato.
admin-settings-key-dsr-sla-reminder-days = Segnalazioni di scadenza
admin-settings-help-dsr-sla-reminder-days = Giorni prima della scadenza in cui una richiesta viene segnalata in coda come in scadenza.
admin-settings-key-dsr-export-mail-allowed = Posta nelle esportazioni
admin-settings-help-dsr-export-mail-allowed = Consente al richiedente di includere messaggi in un'esportazione. Sono inclusi solo i messaggi presenti nella cache di questo server; la posta che resta sul server di posta a monte no.
admin-settings-key-dsr-export-mail-format = Formato della posta nelle esportazioni
admin-settings-help-dsr-export-mail-format = mbox scrive un file per cartella; eml scrive un file per messaggio.
admin-settings-key-dsr-export-include-settings = Esportazione: impostazioni
admin-settings-help-dsr-export-include-settings = Include le impostazioni in un'esportazione.
admin-settings-key-dsr-export-include-identities = Esportazione: identità
admin-settings-help-dsr-export-include-identities = Include le identità in un'esportazione.
admin-settings-key-dsr-export-include-contacts = Esportazione: contatti
admin-settings-help-dsr-export-include-contacts = Include i contatti in un'esportazione.
admin-settings-key-dsr-export-include-calendars = Esportazione: calendari
admin-settings-help-dsr-export-include-calendars = Include i calendari in un'esportazione.
admin-settings-key-dsr-export-include-notes = Esportazione: note
admin-settings-help-dsr-export-include-notes = Include le note in un'esportazione.
admin-settings-key-dsr-export-include-rules = Esportazione: filtri e regole
admin-settings-help-dsr-export-include-rules = Include filtri e regole in un'esportazione.
admin-settings-key-dsr-export-include-sessions = Esportazione: sessioni, dispositivi e cronologia degli accessi
admin-settings-help-dsr-export-include-sessions = Include sessioni, dispositivi e cronologia degli accessi in un'esportazione.
admin-settings-key-dsr-export-include-consent = Esportazione: registri dei consensi
admin-settings-help-dsr-export-include-consent = Include i registri dei consensi in un'esportazione.
admin-settings-key-dsr-export-include-audit = Esportazione: voci di audit sulla persona
admin-settings-help-dsr-export-include-audit = Include in un'esportazione le voci di audit sulla persona.
admin-settings-key-dsr-export-max-megabytes = Limite di dimensione dell'esportazione
admin-settings-help-dsr-export-max-megabytes = Esportazione più grande in megabyte. Le sezioni che lo supererebbero vengono escluse ed elencate nel manifesto.
admin-settings-key-dsr-export-format = Formati di esportazione
admin-settings-help-dsr-export-format = json scrive file JSON. jsonCsv scrive anche file CSV per i dati in elenco.
admin-settings-key-dsr-export-encrypt = Sigilla le esportazioni a riposo
admin-settings-help-dsr-export-encrypt = Memorizza le esportazioni create cifrate con la chiave del server finché non vengono scaricate o scadono.
admin-settings-key-dsr-download-link-minutes = Validità del link di download
admin-settings-help-dsr-download-link-minutes = Minuti in cui un link di download funziona dopo la creazione. L'esportazione stessa viene conservata per la finestra di download dell'esportazione dei dati.
admin-settings-key-dsr-download-single-use = Link di download monouso
admin-settings-help-dsr-download-single-use = Un link di download smette di funzionare dopo essere stato usato una volta.
admin-settings-key-dsr-retention-completed-days = Conservazione delle richieste concluse
admin-settings-help-dsr-retention-completed-days = Giorni in cui una richiesta conclusa e la sua traccia vengono conservate prima di essere eliminate.
admin-settings-key-dsr-notify-admins = Registra le nuove richieste per gli amministratori
admin-settings-help-dsr-notify-admins = Registra ogni nuova richiesta come evento di sicurezza nel registro di audit, dove la vedono gli amministratori e i destinatari degli avvisi.
admin-settings-key-dsr-notify-requester = Messaggi di avanzamento al richiedente
admin-settings-help-dsr-notify-requester = Inserisce un messaggio nella posta in arrivo del richiedente quando una richiesta cambia stato.
admin-settings-key-dsr-contact-name = Nome del contatto per la protezione dei dati
admin-settings-help-dsr-contact-name = Nome mostrato agli utenti nella pagina delle richieste sui dati.
admin-settings-key-dsr-contact-email = Indirizzo del contatto per la protezione dei dati
admin-settings-help-dsr-contact-email = Indirizzo e-mail mostrato agli utenti nella pagina delle richieste sui dati.
admin-settings-key-dsr-contact-url = Pagina del contatto per la protezione dei dati
admin-settings-help-dsr-contact-url = Link mostrato agli utenti nella pagina delle richieste sui dati.
admin-settings-key-dsr-on-behalf-allowed = Richieste per conto di un utente
admin-settings-help-dsr-on-behalf-allowed = Consente a un amministratore di presentare una richiesta per un utente, per esempio una ricevuta per posta.
admin-settings-key-dsr-rate-limit-per-day = Richieste per persona al giorno
admin-settings-help-dsr-rate-limit-per-day = Il massimo di richieste che una persona può presentare in 24 ore.
admin-settings-key-dsr-legal-hold-subjects = Conservazione legale
admin-settings-help-dsr-legal-hold-subjects = Indirizzi, o voci @dominio, la cui cancellazione è bloccata. Una richiesta di cancellazione che li riguarda resta in coda come bloccata finché la voce non viene rimossa.
admin-settings-key-scrub-enabled = Pulizia della persona
admin-settings-help-scrub-enabled = Consente l'azione Pulisci persona e la pulizia che può seguire una cancellazione. Se disattivato, nessuna delle due viene eseguita.
admin-settings-key-scrub-pseudonym-algorithm = Lunghezza dello pseudonimo
admin-settings-help-scrub-pseudonym-algorithm = Come viene creato lo pseudonimo con chiave che sostituisce un identificatore: HMAC-SHA256 con la generazione di chiave attuale, troncato a 16 o 32 caratteri esadecimali.
admin-settings-key-scrub-include-logs = Pulisci i file di log
admin-settings-help-scrub-include-logs = Una pulizia riscrive l'identificatore anche nei file di log ruotati, compresi quelli compressi.
admin-settings-key-scrub-auto-run-on-erasure = Pulizia dopo la cancellazione
admin-settings-help-scrub-auto-run-on-erasure = Una cancellazione completata sostituisce anche gli identificatori della persona nella cronologia di audit e nei log con lo pseudonimo. Disattivato per impostazione predefinita.
admin-settings-key-scrub-key-retention-days = Conservazione della chiave dello pseudonimo
admin-settings-help-scrub-key-retention-days = Giorni in cui una chiave dello pseudonimo viene conservata dopo la rotazione. 0 la conserva. Dopo l'eliminazione, i vecchi pseudonimi non possono più essere ricalcolati e le lapidi create con la chiave non possono più essere riapplicate.
admin-settings-key-backup-propagate-erasure = Riapplica le cancellazioni al ripristino
admin-settings-help-backup-propagate-erasure = Un ripristino riapplica le lapidi di cancellazione più recenti del backup, così le persone cancellate da allora vengono rimosse di nuovo.
admin-settings-key-backup-tombstone-retention-days = Conservazione delle lapidi di cancellazione
admin-settings-help-backup-tombstone-retention-days = Giorni in cui una lapide di cancellazione viene conservata. 0 le conserva per sempre.
admin-settings-key-dsr-text-intro = Introduzione alle richieste sui dati
admin-settings-help-dsr-text-intro = Testo mostrato sopra il modulo di richiesta. Un valore per lingua.
admin-settings-key-dsr-text-confirmation = Testo del messaggio di conferma
admin-settings-help-dsr-text-confirmation = Testo aggiunto al messaggio di conferma. Un valore per lingua.
admin-settings-key-dsr-text-completion = Testo del messaggio di completamento
admin-settings-help-dsr-text-completion = Testo aggiunto al messaggio inviato quando una richiesta è completata. Un valore per lingua.
admin-settings-key-server-text-family = Testo del server
admin-settings-help-server-text-family = Testo che il server scrive in e-mail e notifiche, per lingua e dominio.
admin-settings-cat-sieve = Filtri di posta
admin-settings-key-sieve-max-script-bytes = Dimensione massima dello script di filtro
admin-settings-help-sieve-max-script-bytes = Numero massimo di byte in un singolo script di filtro Sieve.
admin-settings-key-sieve-max-scripts = Numero massimo di script di filtro
admin-settings-help-sieve-max-scripts = Numero massimo di script salvati per account.
admin-settings-key-sieve-max-redirects = Numero massimo di reindirizzamenti
admin-settings-help-sieve-max-redirects = Numero massimo di comandi di reindirizzamento in uno script.
admin-settings-key-sieve-vacation-allowed = Azioni di filtro per le ferie
admin-settings-help-sieve-vacation-allowed = Consente agli script di inviare risposte automatiche di assenza quando il backend le supporta.
admin-settings-key-sieve-allowed-extensions = Estensioni di filtro consentite
admin-settings-help-sieve-allowed-extensions = Un elenco facoltativo di nomi di estensioni Sieve in minuscolo. Anche il backend deve supportare ogni estensione.
admin-settings-key-cap-sieve-raw-edit = Modifica il sorgente dello script di filtro
admin-settings-help-cap-sieve-raw-edit = Consente la modifica diretta degli script Sieve per gli account selezionati.
admin-settings-key-cap-sieve-forward-external = Inoltra fuori dal dominio dell'account
admin-settings-help-cap-sieve-forward-external = Consente l'inoltro esterno negli script. Il salvataggio di questi script richiede anche una verifica recente.
admin-settings-key-session-max-concurrent = Sessioni simultanee
admin-settings-help-session-max-concurrent = Numero massimo di sessioni per account. Le sessioni più vecchie vengono revocate quando un nuovo accesso supera questo limite.
admin-settings-key-session-idle-secs = Timeout di inattività della sessione
admin-settings-help-session-idle-secs = Termina le sessioni inattive dopo questo numero di secondi, fino al limite della distribuzione.
admin-settings-key-session-absolute-secs = Durata della sessione
admin-settings-help-session-absolute-secs = Termina le sessioni dopo questo numero di secondi, fino al limite della distribuzione.
admin-settings-key-twofa-remember-days = Durata del dispositivo ricordato
admin-settings-help-twofa-remember-days = Giorni dopo i quali un dispositivo ricordato deve verificare di nuovo il secondo fattore. Zero disattiva questa funzione.
admin-settings-key-twofa-passkeys-enabled = Consenti le passkey
admin-settings-help-twofa-passkeys-enabled = Consente le passkey come fattore di autenticazione.
admin-settings-key-twofa-required = Richiedi l'autenticazione a due fattori
admin-settings-help-twofa-required = Richiede un secondo fattore registrato prima di emettere una sessione.
admin-settings-key-twofa-passkey-attachment = Tipo di autenticatore della passkey
admin-settings-help-twofa-passkey-attachment = any (predefinito): qualsiasi autenticatore. platform: passkey memorizzate sul dispositivo (Windows Hello, Touch ID, blocco schermo di Android). cross-platform: chiavi di sicurezza e telefoni. Lo segnala il browser; non è firmato dall'autenticatore.
admin-settings-key-twofa-passkey-user-verification = Verifica dell'utente della passkey
admin-settings-help-twofa-passkey-user-verification = required: la passkey deve verificare un PIN o un dato biometrico all'aggiunta e a ogni accesso; una procedura senza verifica viene rifiutata. preferred (predefinito) e discouraged cambiano solo ciò che viene chiesto al browser.
admin-settings-key-twofa-passkey-resident-key = Passkey individuabili
admin-settings-help-twofa-passkey-resident-key = required: la credenziale deve essere memorizzata sull'autenticatore. discouraged (predefinito) o preferred. In modalità engine le passkey sono sempre individuabili, perché consentono anche l'accesso senza password.
admin-settings-key-twofa-passkey-legacy = Passkey aggiunte prima del criterio attuale
admin-settings-help-twofa-passkey-legacy = allow (predefinito): una passkey più vecchia che non rispetta il criterio continua a funzionare e all'utente viene chiesto di sostituirla. refuse: viene rifiutata all'accesso con un messaggio; l'utente accede con un altro fattore o un codice di recupero.
admin-settings-key-twofa-last-factor-removal = Rimozione dell'ultimo secondo fattore
admin-settings-help-twofa-last-factor-removal = Quando rimuovere una passkey o un'app di autenticazione lascerebbe l'account senza alcun fattore. allow (predefinito): viene rimosso e l'accesso richiede solo la password. block: l'utente deve prima aggiungere un altro metodo di accesso (i codici di recupero non contano). Dove l'autenticazione a due fattori è obbligatoria, l'ultimo viene sempre mantenuto.
admin-settings-key-privacy-remote-content = Contenuti remoti
admin-settings-help-privacy-remote-content = Blocca le immagini, chiedi prima, usa il proxy o carica direttamente quando consentito.
admin-settings-key-privacy-trackers = Blocca i tracker
admin-settings-help-privacy-trackers = Blocca le immagini di tracciamento delle e-mail note.
admin-settings-key-privacy-clean-links = Pulisci i link
admin-settings-help-privacy-clean-links = Rimuove i parametri di tracciamento dai link dei messaggi.
admin-settings-key-privacy-max-bytes = Limite delle immagini remote
admin-settings-help-privacy-max-bytes = Numero massimo di byte recuperati per una singola immagine remota.
admin-settings-key-privacy-cache-bytes = Cache delle immagini remote
admin-settings-help-privacy-cache-bytes = Numero massimo di byte mantenuti nella cache delle immagini remote.
admin-settings-key-privacy-rate-per-min = Frequenza delle richieste di immagini remote
admin-settings-help-privacy-rate-per-min = Numero massimo di recuperi di immagini remote per account al minuto.
admin-settings-privacy-fetch-title = Recupero delle immagini remote
admin-settings-privacy-fetch-help = Limiti di tempo, reindirizzamenti e download paralleli per le immagini che il server recupera per il lettore. Un'immagine oltre un limite mostra un segnaposto.
admin-settings-key-privacy-fetch-connect-timeout-ms = Timeout di connessione delle immagini remote
admin-settings-help-privacy-fetch-connect-timeout-ms = Millisecondi consentiti per aprire la connessione all'host di un'immagine, compreso l'handshake TLS. Predefinito 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Timeout di risposta delle immagini remote
admin-settings-help-privacy-fetch-headers-timeout-ms = Millisecondi consentiti dall'inizio di una richiesta finché l'host dell'immagine risponde con le intestazioni. Predefinito 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Timeout di stallo delle immagini remote
admin-settings-help-privacy-fetch-idle-timeout-ms = La pausa più lunga, in millisecondi, tra due parti del download di un'immagine prima che venga interrotto. Predefinito 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Timeout totale delle immagini remote
admin-settings-help-privacy-fetch-total-timeout-ms = Millisecondi consentiti per recuperare un'immagine, compresi i reindirizzamenti. L'attesa in coda ha un budget separato, così un'immagine in attesa non consuma il proprio tempo di rete prima di iniziare. Predefinito 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Timeout della coda delle immagini remote
admin-settings-help-privacy-fetch-queue-timeout-ms = Numero massimo di millisecondi in cui le immagini possono attendere capacità oltre al loro budget di recupero. La coda limitata sostiene grandi picchi autenticati mentre restano applicati i limiti di recupero e decodifica attivi. Predefinito 60000.
admin-settings-key-privacy-max-redirects = Reindirizzamenti delle immagini remote
admin-settings-help-privacy-max-redirects = Reindirizzamenti seguiti per una singola immagine. Ciascuno viene controllato con le stesse regole di indirizzo della prima richiesta. Predefinito 3.
admin-settings-key-privacy-max-concurrent-per-message = Download di immagini remote per messaggio
admin-settings-help-privacy-max-concurrent-per-message = Immagini di un messaggio recuperate contemporaneamente. Le altre attendono uno slot libero. Intervallo 1–128; predefinito 30.
admin-settings-key-privacy-max-concurrent-per-account = Download di immagini remote per account
admin-settings-help-privacy-max-concurrent-per-account = Immagini recuperate contemporaneamente per un account su tutti i suoi messaggi aperti. Intervallo 1–256; predefinito 128.
admin-settings-key-privacy-allow-direct = Consenti le immagini dirette
admin-settings-key-cap-remote-content-direct = Autorizzazione per le immagini dirette
admin-settings-help-cap-remote-content-direct = Consente a questo account di selezionare le immagini dirette quando il server lo permette.
admin-settings-help-privacy-allow-direct = Consente il caricamento diretto delle immagini per gli account con la funzionalità richiesta.
admin-settings-key-logs-auth-retention-days = Conservazione della cronologia di autenticazione
admin-settings-help-logs-auth-retention-days = Giorni di conservazione dei metadati di autenticazione, indipendentemente dall'identità durevole dell'account.
admin-settings-key-logs-auth-max-entries = Limite di voci della cronologia di autenticazione
admin-settings-help-logs-auth-max-entries = Numero massimo di voci conservate. I fallimenti ripetuti corrispondenti entro un minuto condividono una voce.
admin-settings-key-reader-color-mode = Colori dei messaggi
admin-settings-cat-reader = Visualizzazione dei messaggi
admin-settings-key-mail-row-actions-mode = Azioni rapide dell'elenco messaggi
admin-settings-help-mail-row-actions-mode = Come compaiono le azioni rapide su ogni riga di messaggio: open (sempre disponibili, mostrate al passaggio del mouse o al focus), hover (tre controlli e un pulsante Altro), click (un pulsante che le apre) oppure off (rimosse dalle righe; il menu contestuale e la barra degli strumenti mantengono ogni azione). Hover si comporta come click sui touch screen. Gli utenti possono scegliere la propria opzione, a meno che un amministratore la blocchi.
admin-settings-key-mail-quick-reply = Risposta rapida

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-quick-reply = Offre una scheda di risposta compatta con i destinatari già compilati. Disattivato per impostazione predefinita; gli utenti possono attivarlo, a meno che un amministratore lo blocchi. Le normali protezioni di invio restano valide.
admin-settings-key-mail-request-read-receipt = Richiedi le conferme di lettura per impostazione predefinita
admin-settings-help-mail-request-read-receipt = Avvia i messaggi in uscita supportati con la richiesta di conferma selezionata. È un'impostazione predefinita di composizione, non una garanzia che il destinatario invierà la conferma.
admin-settings-help-reader-color-mode = Automatico segue i temi scuri. Scelga il rendering originale, adattivo o invertito all'interno del riquadro isolato del messaggio.
admin-settings-key-reader-font-size = Dimensione del carattere del messaggio
admin-settings-help-reader-font-size = Dimensione di lettura predefinita in pixel, da 12 a 24.
admin-settings-key-reader-line-height = Altezza di riga del messaggio
admin-settings-help-reader-line-height = Interlinea di lettura da 1,2 a 2,2.
admin-settings-key-reader-content-width = Larghezza di lettura del messaggio
admin-settings-help-reader-content-width = Contenuto del messaggio a larghezza comoda, ampia o a tutto riquadro.
admin-settings-key-reader-loading = Aspetto del caricamento del messaggio
admin-settings-help-reader-loading = Righe segnaposto, un indicatore di testo o nessun indicatore visivo. Gli annunci per l'accessibilità restano disponibili.
admin-settings-key-reader-loading-delay-ms = Ritardo del caricamento del messaggio
admin-settings-help-reader-loading-delay-ms = Ritarda solo l'indicatore di caricamento, tra 0 e 1000 millisecondi. La consegna del messaggio non viene mai ritardata.
admin-settings-key-reader-zoom = Zoom del messaggio
admin-settings-help-reader-zoom = Zoom predefinito del corpo del messaggio in percentuale, da 50 a 200; il lettore usa il passo di zoom più vicino. Solo il messaggio viene ridimensionato e la stampa usa il 100%.
admin-settings-key-reader-attachments-collapsed = Barra degli allegati compressa
admin-settings-help-reader-attachments-collapsed = Se la barra degli allegati del lettore parte compressa, mostrando solo il numero e la dimensione totale. L'ultima apertura o chiusura dell'utente viene salvata qui. Disattivato per impostazione predefinita: gli allegati sono mostrati.
admin-settings-key-login-show-domain-hint = Mostra il dominio preferito all'accesso
admin-settings-help-login-show-domain-hint = Mostra quale dominio accetta un nome utente senza suffisso e-mail. Nascosto per impostazione predefinita. Cambia solo il suggerimento; l'accesso con un nome utente continua a funzionare.
admin-settings-key-signin-retry-cooldown-secs = Attesa tra i tentativi di accesso
admin-settings-help-signin-retry-cooldown-secs = Secondi minimi tra i tentativi di accesso con password dalla stessa rete client. Predefinito 3; 0 disattiva l'attesa. I blocchi esistenti per errori restano validi.
admin-settings-key-mail-quote-style = Stile dell'intestazione di risposta e inoltro
admin-settings-key-mail-external-recipient-warning = Avviso per destinatario esterno
admin-settings-help-mail-external-recipient-warning = Disattivato, avvisa o richiede conferma nell'editor. Riguarda A, Cc e Ccn. È una salvaguardia dell'editor, non una restrizione di consegna del server.
admin-settings-key-mail-trusted-recipient-domains = Domini di destinatari attendibili
admin-settings-help-mail-trusted-recipient-domains = Fino a 100 domini di destinatari esatti. Anche il dominio dell'identità di invio selezionata è attendibile. Le voci non valide non concedono mai fiducia. Blocchi questa impostazione per impedire modifiche da parte degli utenti.
admin-settings-help-mail-quote-style = Usa risposte citate classiche o intestazioni di messaggio in stile Outlook. Imposta il valore predefinito dell'utente; lo blocchi per imporre lo stile. Le bozze esistenti non cambiano.
admin-settings-key-signin-anti-bot-mode = Protezione anti-bot all'accesso
admin-settings-help-signin-anti-bot-mode = Verifica proof-of-work self-hosted: off, mailbox, admin o all. Nessun servizio di verifica esterno viene contattato.
admin-settings-key-signin-anti-bot-difficulty = Impegno della verifica di accesso
admin-settings-help-signin-anti-bot-difficulty = Contatore massimo del risolutore, 100–20000. Valori più alti richiedono più lavoro al browser; predefinito 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Validità della verifica di accesso
admin-settings-help-signin-anti-bot-expiry-secs = Secondi in cui una verifica monouso resta valida, 60–600; predefinito 180. Dopo ogni tentativo con credenziali serve una nuova verifica.
admin-settings-key-attachments-preview-allowed = Consenti le anteprime degli allegati
admin-settings-help-attachments-preview-allowed = Disattivi per impedire il rendering delle anteprime degli allegati; i download originali hanno un controllo proprio.
admin-settings-key-attachments-download-allowed = Consenti la consegna dell'allegato originale
admin-settings-help-attachments-download-allowed = Disattivare impedisce anche le anteprime che richiedono i byte originali. Le anteprime del browser non sono una protezione dalla copia.
admin-settings-key-attachments-max-bytes = Byte massimi dell'allegato
admin-settings-help-attachments-max-bytes = Zero significa illimitato. Si applica ai byte originali consegnati, comprese le richieste di intervallo.
admin-settings-key-attachments-allowed-extensions = Estensioni di nome file consentite
admin-settings-help-attachments-allowed-extensions = Array JSON di estensioni minuscole senza punto, come ["pdf","tar.gz"]. Vuoto consente qualsiasi estensione; le regole di blocco prevalgono.
admin-settings-key-attachments-blocked-extensions = Estensioni di nome file bloccate
admin-settings-help-attachments-blocked-extensions = Array JSON di estensioni minuscole. Valutato sui nomi di file MIME memorizzati, mai sul nome nell'URL di download.
admin-settings-key-attachments-warn-extensions = Avvisa per le estensioni di nome file
admin-settings-help-attachments-warn-extensions = Array JSON di estensioni che richiedono una conferma esplicita prima della consegna dell'originale.
admin-settings-key-attachments-allowed-mime-types = Tipi MIME consentiti
admin-settings-help-attachments-allowed-mime-types = Array JSON di tipi minuscoli o caratteri jolly di primo livello, come ["image/*","application/pdf"]. Vuoto consente qualsiasi tipo.
admin-settings-key-attachments-blocked-mime-types = Tipi MIME bloccati
admin-settings-help-attachments-blocked-mime-types = Array JSON di tipi minuscoli o caratteri jolly di primo livello. Le regole di blocco hanno la precedenza.
admin-settings-key-attachments-warn-mime-types = Avvisa per i tipi MIME
admin-settings-help-attachments-warn-mime-types = Array JSON di tipi minuscoli o caratteri jolly di primo livello che richiedono conferma. I metadati MIME non sono un rilevamento di malware.
admin-settings-cat-shortcuts = Scorciatoie da tastiera
admin-settings-cat-time = Fusi orari
admin-settings-key-time-display-zone = Fuso orario di visualizzazione
admin-settings-help-time-display-zone = Fuso orario IANA come Europe/Lisbon, oppure local per seguire il dispositivo attuale. Gli orari dei messaggi restano memorizzati come istanti originali.
admin-settings-key-time-comparison-zone = Fuso orario di confronto
admin-settings-help-time-comparison-zone = Secondo fuso orario IANA facoltativo per gli aiuti dei messaggi e del calendario. Lasci vuoto per nascondere il confronto; nessuna posizione viene dedotta da un indirizzo e-mail.
admin-settings-key-time-show-helpers = Mostra gli aiuti sui fusi orari
admin-settings-help-time-show-helpers = Mostra i confronti tra ora locale e fuso selezionato per gli orari dei messaggi e gli eventi del calendario. I blocchi dell'amministratore possono imporre questa scelta.
admin-settings-key-shortcuts-enabled = Attiva le scorciatoie da tastiera
admin-settings-help-shortcuts-enabled = Consente le scorciatoie dell'applicazione. I blocchi possono disattivarle globalmente o per un dominio. I campi di digitazione e le finestre di dialogo modali mantengono la normale gestione della tastiera.
admin-settings-key-shortcuts-compose = Scorciatoia per scrivere
admin-settings-key-shortcuts-archive = Scorciatoia per archiviare
admin-settings-key-shortcuts-reply = Scorciatoia per rispondere
admin-settings-key-shortcuts-reply-all = Scorciatoia per rispondere a tutti
admin-settings-key-shortcuts-forward = Scorciatoia per inoltrare
admin-settings-key-shortcuts-next = Scorciatoia per il messaggio successivo
admin-settings-key-shortcuts-previous = Scorciatoia per il messaggio precedente
admin-settings-key-shortcuts-search = Scorciatoia per attivare la ricerca
admin-settings-key-shortcuts-refresh = Scorciatoia per aggiornare i messaggi
admin-settings-key-shortcuts-settings = Scorciatoia per aprire le impostazioni
admin-settings-key-shortcuts-trash = Scorciatoia per spostare nel cestino
admin-settings-key-shortcuts-mark-read = Scorciatoia per segnare come letto
admin-settings-key-shortcuts-mark-unread = Scorciatoia per segnare come non letto
admin-settings-key-shortcuts-spam = Scorciatoia per segnare come spam
admin-settings-key-shortcuts-zoom-in = Scorciatoia per ingrandire il messaggio
admin-settings-key-shortcuts-zoom-out = Scorciatoia per ridurre il messaggio
admin-settings-key-shortcuts-zoom-reset = Scorciatoia per reimpostare lo zoom del messaggio
admin-settings-key-shortcuts-open-in-tab = Scorciatoia per aprire il messaggio in una scheda propria
admin-settings-key-notifications-push-allowed = Notifiche push in background consentite
admin-settings-help-notifications-push-allowed = Consente alle notifiche attive di questo account di registrarsi e ricevere push in background. È comunque richiesta l'autorizzazione del browser.
admin-settings-key-offline-persistent-storage-allowed = Richieste di archiviazione persistente del browser consentite
admin-settings-help-offline-persistent-storage-allowed = Consente una richiesta esplicita di proteggere i dati in cache locali dall'eliminazione automatica del browser. Il browser può rifiutare; ciò non concede automaticamente l'autorizzazione.
admin-settings-key-security-signin-mailbox-failure-limit = Limite di credenziali errate della casella
admin-settings-help-security-signin-mailbox-failure-limit = Le credenziali rifiutate da uno stesso IP di origine entro la finestra di conteggio attivano un blocco temporaneo dell'accesso alla casella. Gli errori di rete e di configurazione non contano.
admin-settings-key-security-signin-admin-failure-limit = Limite di credenziali errate dell'amministratore
admin-settings-help-security-signin-admin-failure-limit = Le credenziali di amministratore rifiutate da uno stesso IP di origine attivano un blocco temporaneo separato dell'accesso come amministratore. La sessione breve dell'amministratore e i requisiti di prova recente restano invariati.
admin-settings-key-security-signin-failure-window-secs = Finestra di conteggio delle credenziali errate
admin-settings-help-security-signin-failure-window-secs = Finestra mobile in secondi per i rifiuti di credenziali e MFA. Un accesso completato con successo azzera quell'origine e quella superficie.
admin-settings-key-security-signin-lock-secs = Durata del blocco temporaneo dell'accesso
admin-settings-help-security-signin-lock-secs = Durata del blocco in secondi. I tentativi ripetuti durante il blocco non ne estendono la scadenza originale. I blocchi IP globali manuali sono separati.
admin-settings-key-security-signin-geo-scope = Ambito del criterio per paese all'accesso
admin-settings-help-security-signin-geo-scope = Disattivato per impostazione predefinita. Si applica solo ai nuovi accessi di casella, di amministratore o a tutti. Le sessioni esistenti non vengono revocate. Controlli la diagnostica dell'origine attuale prima di attivarlo; un database GeoIP locale non disponibile rifiuta l'accesso.
admin-settings-key-security-signin-geo-mode = Elenco di paesi consentiti o negati
admin-settings-help-security-signin-geo-mode = Allow consente solo i paesi elencati; deny rifiuta i paesi elencati. Gli indirizzi sconosciuti usano il proprio criterio esplicito.
admin-settings-key-security-signin-geo-countries = Codici paese per l'accesso
admin-settings-help-security-signin-geo-countries = Array JSON di codici paese di due lettere maiuscole, per esempio ["PT", "GB"]. Usa solo il database locale dei paesi (Sicurezza > Database dei paesi, oppure MW_GEOIP_DB); gli indirizzi non vengono mai inviati a un servizio di geolocalizzazione.
admin-settings-key-security-signin-geo-unknown = Criterio per indirizzo di origine non mappato
admin-settings-help-security-signin-geo-unknown = Consente o nega gli indirizzi assenti da un database locale leggibile, compresi gli indirizzi privati. Database mancanti o illeggibili rifiutano sempre gli accessi con criterio attivo, anche quando gli indirizzi sconosciuti sono consentiti.
admin-settings-key-contacts-avatar-mode = Foto dei contatti e avatar generati
admin-settings-help-contacts-avatar-mode = Genera le iniziali in locale, nasconde gli avatar o usa le foto incorporate dalle rubriche di ciascun account. Non viene contattato alcun URL di foto remota né servizio esterno di avatar. Il criterio per utente e per dominio può essere gestito da un amministratore.
admin-settings-cat-contacts = Foto dei contatti
admin-settings-key-reader-action-style = Barra multifunzione delle azioni del messaggio
admin-settings-help-reader-action-style = Le icone compatte sono l'impostazione predefinita. Scelga le etichette per icone con testo visibile; entrambe mantengono i nomi delle azioni e i suggerimenti a tema. Supporta blocchi di criterio globali, di dominio e di utente.
admin-settings-key-nextcloud-sync-interval-secs = Intervallo di aggiornamento automatico di Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Secondi tra gli aggiornamenti in background delle raccolte per gli account connessi del motore nativo. Zero significa solo manuale; i valori positivi hanno un minimo di sessanta secondi. Gli errori allungano l'intervallo. Non viene tentato alcun accesso in background.
admin-settings-key-mail-folder-role-inbox = Mappatura della cartella Posta in arrivo
admin-settings-key-mail-folder-role-drafts = Mappatura della cartella Bozze
admin-settings-key-mail-folder-role-sent = Mappatura della cartella Inviata
admin-settings-key-mail-folder-role-trash = Mappatura della cartella Cestino
admin-settings-key-mail-folder-role-junk = Mappatura della cartella Posta indesiderata
admin-settings-key-mail-folder-role-archive = Mappatura della cartella Archivio
admin-settings-key-mail-request-delivery-receipt = Richiedi le notifiche di consegna per impostazione predefinita
admin-settings-help-mail-request-delivery-receipt = Seleziona le notifiche di consegna per i nuovi messaggi quando l'account supporta SMTP DSN. Il blocco controlla questo valore predefinito salvato; receipts.dsnAllowed stabilisce separatamente se le richieste sono consentite. La consegna non dimostra la lettura.
admin-settings-key-security-signin-geo-enforce-sessions = Applica il criterio per paese alle sessioni esistenti
admin-settings-help-security-signin-geo-enforce-sessions = Disattivato per impostazione predefinita. Applica l'ambito dei paesi di accesso configurato alle richieste autenticate, all'accesso bearer nativo e al contenuto firmato del lettore. I client bloccati possono comunque disconnettersi. I flussi già aperti vengono controllati alla riconnessione; ciò non termina le connessioni esistenti.
admin-settings-key-security-protection-rate-mode = Budget aggregati di richieste
admin-settings-help-security-protection-rate-mode = Disattivato per impostazione predefinita. Observe registra l'esaurimento del budget senza rifiutare; enforce restituisce un 429 temporaneo. I limiti di concorrenza di protocollo esistenti restano indipendenti.
admin-settings-key-security-protection-adaptive-mode = Protezione adattiva delle richieste
admin-settings-help-security-protection-adaptive-mode = Disattivato per impostazione predefinita. Observe registra anomalie tipizzate spiegabili e azioni proposte; enforce applica limitazioni o blocchi temporanei circoscritti. È un criterio deterministico a contatori, non un classificatore IA.
admin-settings-key-security-protection-client-requests = Richieste del client per finestra
admin-settings-help-security-protection-client-requests = Richieste per finestra da un indirizzo non connesso, e ogni richiesta di accesso. Una richiesta con una sessione valida di casella o di amministratore usa invece una quota propria, più ampia. Zero disattiva questo budget.
admin-settings-key-security-protection-account-requests = Richieste dell'account per finestra
admin-settings-help-security-protection-account-requests = Richieste per finestra di una casella connessa, tra le sue sessioni, dispositivi e indirizzi. Mantenuta sopra la quota anonima così l'uso con accesso ha più margine. Zero disattiva questo budget.
admin-settings-key-rate-limit-diagnostics-burst = Esecuzioni diagnostiche contemporanee
admin-settings-help-rate-limit-diagnostics-burst = Esecuzioni diagnostiche e controlli del backend che una sessione di amministratore può avviare di seguito. Avviare un controllo già in corso restituisce quel controllo invece di contare di nuovo.
admin-settings-key-rate-limit-diagnostics-per-minute = Esecuzioni diagnostiche al minuto
admin-settings-help-rate-limit-diagnostics-per-minute = Esecuzioni diagnostiche e controlli del backend al minuto che una sessione di amministratore riacquista.
admin-settings-key-security-protection-admin-requests = Richieste dell'amministratore per finestra
admin-settings-help-security-protection-admin-requests = Richieste per finestra di un amministratore connesso. Mantenuta sopra la quota anonima. Zero disattiva questo budget.
admin-settings-key-security-protection-rate-window-secs = Finestra del budget di richieste (secondi)
admin-settings-help-security-protection-rate-window-secs = Intervallo fisso del budget di richieste. Lo scarto di carico per capacità e le normali risposte not-found non sono prova adattiva.
admin-settings-key-security-protection-adaptive-clients = Osserva i client per la protezione adattiva
admin-settings-key-security-protection-adaptive-accounts = Osserva gli account per la protezione adattiva
admin-settings-key-security-protection-adaptive-admins = Osserva gli amministratori per la protezione adattiva
admin-settings-key-security-protection-short-window-secs = Finestra breve delle anomalie (secondi)
admin-settings-help-security-protection-short-window-secs = Finestra mobile di osservazione. Le finestre lunghe supportano da ore a trenta giorni; l'arrotondamento dei bucket può trattenere al massimo un sessantaquattresimo di finestra in più.
admin-settings-key-security-protection-short-minimum-samples = Campioni minimi del rapporto breve
admin-settings-help-security-protection-short-minimum-samples = Minimo di richieste normali più anomale osservate prima che scatti il trigger del rapporto. La soglia assoluta delle anomalie è indipendente da questo minimo.
admin-settings-key-security-protection-short-absolute-anomalies = Budget assoluto di anomalie breve
admin-settings-help-security-protection-short-absolute-anomalies = Scatta quando si accumulano tante anomalie selezionate, anche se le richieste normali diluiscono il rapporto. Zero disattiva questo trigger.
admin-settings-key-security-protection-short-ratio-percent = Rapporto di anomalie breve (percentuale)
admin-settings-help-security-protection-short-ratio-percent = Scatta quando le anomalie selezionate raggiungono questa percentuale dopo il numero minimo di campioni. Basta il trigger assoluto o quello del rapporto. Zero disattiva questo trigger.
admin-settings-key-security-protection-long-window-secs = Finestra lunga delle anomalie (secondi)
admin-settings-help-security-protection-long-window-secs = Finestra mobile di osservazione. Le finestre lunghe supportano da ore a trenta giorni; l'arrotondamento dei bucket può trattenere al massimo un sessantaquattresimo di finestra in più.
admin-settings-key-security-protection-long-minimum-samples = Campioni minimi del rapporto lungo
admin-settings-help-security-protection-long-minimum-samples = Minimo di richieste normali più anomale osservate prima che scatti il trigger del rapporto. La soglia assoluta delle anomalie è indipendente da questo minimo.
admin-settings-key-security-protection-long-absolute-anomalies = Budget assoluto di anomalie lungo
admin-settings-help-security-protection-long-absolute-anomalies = Scatta quando si accumulano tante anomalie selezionate, anche se le richieste normali diluiscono il rapporto. Zero disattiva questo trigger.
admin-settings-key-security-protection-long-ratio-percent = Rapporto di anomalie lungo (percentuale)
admin-settings-help-security-protection-long-ratio-percent = Scatta quando le anomalie selezionate raggiungono questa percentuale dopo il numero minimo di campioni. Basta il trigger assoluto o quello del rapporto. Zero disattiva questo trigger.
admin-settings-key-security-protection-normal-cap-per-bucket = Limite della linea di base normale per bucket
admin-settings-help-security-protection-normal-cap-per-bucket = Limita la linea di base normale appresa in ogni bucket della finestra mobile. Budget assoluti di anomalie indipendenti impediscono che un riempimento illimitato di richieste valide nasconda l'abuso.
admin-settings-key-security-protection-signals = Segnali di prova adattiva
admin-settings-help-security-protection-signals = Selezioni osservazioni autorevoli di malformato, troppo grande, credenziali rifiutate, permesso negato, scadenza del corpo o, su richiesta, mancata corrispondenza di una route di amministrazione. Il sondaggio delle route di amministrazione esclude le risposte di entità non trovata e i file statici; codici di stato generici, rifiuti per capacità e stringhe user-agent non stabiliscono un abuso.
admin-settings-key-security-protection-throttle-secs = Limitazione temporanea iniziale (secondi)
admin-settings-help-security-protection-throttle-secs = Il primo superamento della soglia avvia una scadenza fissa di limitazione. Le richieste durante quella scadenza non la estendono.
admin-settings-key-security-protection-block-secs = Blocco temporaneo intensificato (secondi)
admin-settings-help-security-protection-block-secs = Un superamento successivo della soglia prima del recupero può intensificarsi in questo blocco circoscritto. Non viene creato alcun blocco permanente dell'account.
admin-settings-key-security-protection-decay-secs = Intervallo di recupero tranquillo (secondi)
admin-settings-help-security-protection-decay-secs = Dopo questo intervallo senza anomalie e la scadenza dell'azione corrente, l'intensificazione torna alla fase iniziale. Le prove mobili scadono secondo le finestre configurate.
admin-settings-key-security-protection-exempt-clients = Indirizzi e reti client esentati
admin-settings-help-security-protection-exempt-clients = Indirizzi IP esatti o reti CIDR esentano solo la dimensione client. I limiti degli account verificati restano validi. Usi con attenzione per reti NAT condivise; gli indirizzi inoltrati sono attendibili solo tramite i proxy configurati.
admin-settings-key-security-protection-exempt-accounts = ID di account verificati esentati
admin-settings-help-security-protection-exempt-accounts = ID esatti di account casella verificati dal server esentano solo la loro dimensione account, mai quella del client di origine. Indirizzi e-mail e parametri di account forniti dal chiamante non sono prova di identità.
admin-settings-key-security-protection-exempt-admins = ID di amministratori esentati
admin-settings-help-security-protection-exempt-admins = ID esatti di amministratori autenticati esentano solo la loro dimensione amministratore. I blocchi esistenti per errori di accesso e le scadenze rigorose della sessione di amministratore restano invariati.
admin-settings-key-security-protection-storage-failure = Criterio per errore di archiviazione dell'applicazione
admin-settings-help-security-protection-storage-failure = Closed rifiuta il lavoro protetto quando un budget di applicazione attivo non può essere verificato. Open lo consente temporaneamente con un avviso operativo circoscritto. Gli errori in sola osservazione non impongono mai un'azione.
admin-settings-key-security-admin-probe-enabled = Protezione dal sondaggio dell'amministrazione
admin-settings-help-security-admin-probe-enabled = Conta le richieste che cercano la console di amministrazione per rete client (indirizzo IPv4, IPv6 /64), rallenta un'origine che insiste e poi la blocca. Errori di battitura e segnalibri obsoleti restano entro la tolleranza. Si modifica in Sicurezza > Protezione adattiva.
admin-settings-key-security-admin-probe-grace-allowance = Tolleranza (errori per finestra)
admin-settings-help-security-admin-probe-grace-allowance = Errori ignorati per finestra. Il punteggio cala continuamente di altrettanti punti per finestra.
admin-settings-key-security-admin-probe-window-secs = Finestra di tolleranza (secondi)
admin-settings-help-security-admin-probe-window-secs = La finestra a cui si applica la tolleranza.
admin-settings-key-security-admin-probe-block-after = Errori oltre la tolleranza prima di un blocco
admin-settings-help-security-admin-probe-block-after = Errori oltre la tolleranza che avviano un blocco. Gli errori intermedi ricevono risposta dopo il ritardo tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Ritardo tarpit (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Ritardo aggiunto a ogni errore contato oltre la tolleranza. 0 disattiva il tarpit. Le altre richieste non vengono mai ritardate.
admin-settings-key-security-admin-probe-block-secs = Primo blocco (secondi)
admin-settings-help-security-admin-probe-block-secs = Durata del primo blocco. Un'origine bloccata riceve la stessa risposta per ogni percorso.
admin-settings-key-security-admin-probe-max-block-secs = Blocco più lungo (secondi)
admin-settings-help-security-admin-probe-max-block-secs = Un blocco ripetuto raddoppia la durata precedente, fino a questo limite.
admin-settings-key-security-admin-probe-repeat-window-secs = Finestra di ripetizione (secondi)
admin-settings-help-security-admin-probe-repeat-window-secs = Un blocco che inizia entro questo tempo dalla fine del precedente conta come ripetizione.
admin-settings-key-security-admin-probe-block-status = Stato della risposta di blocco
admin-settings-help-security-admin-probe-block-status = Stato HTTP inviato a un'origine bloccata, con Retry-After: 429 o 403.
admin-settings-key-security-admin-probe-signals = Errori contati
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (un 404 alla profondità di percorso della console), admin-signin-failed e admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Percorsi di sondaggio
admin-settings-help-security-admin-probe-probe-paths = Percorsi che gli scanner richiedono, come /wp-admin o /.env. Un percorso che ne contiene uno come segmenti interi conta come errore. Il percorso della console stessa non conta mai.
admin-settings-key-security-admin-probe-allowlist = Reti mai bloccate
admin-settings-help-security-admin-probe-allowlist = Indirizzi IP o reti CIDR (ufficio, VPN) che non vengono mai contati né bloccati.
admin-settings-key-security-admin-probe-audit-ip = Indirizzo di origine nel registro di audit
admin-settings-help-security-admin-probe-audit-ip = masked mostra IPv4 /24 e IPv6 /48; full mostra l'indirizzo. La riga di fail2ban contiene sempre l'indirizzo completo.
admin-settings-key-security-admin-probe-fail2ban-log = Riga di log fail2ban al blocco
admin-settings-help-security-admin-probe-fail2ban-log = Scrive una riga mailwoman[admin-probe] con rhost=<address> quando un'origine viene bloccata.
admin-settings-key-appearance-custom-css-enabled = Consenti CSS di aspetto personale
admin-settings-help-appearance-custom-css-enabled = Consente stili visivi limitati nello spazio di lavoro del proprietario. Disattivato per impostazione predefinita; i blocchi di aspetto applicati alla casella sospendono questi stili.
admin-settings-key-appearance-custom-js-enabled = Consenti script di aspetto personale
admin-settings-help-appearance-custom-js-enabled = Consente script isolati che possono solo richiedere modifiche di aspetto. Disattivato per impostazione predefinita; gli script non possono accedere a posta, dati dell'account né al DOM dell'applicazione.
admin-settings-key-mail-modifier-selection = Selezione dei messaggi con i tasti modificatori
admin-settings-help-shortcuts-compose = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-archive = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-reply = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-reply-all = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-forward = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-next = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-previous = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-search = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-refresh = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-settings = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-trash = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-mark-read = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-mark-unread = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-spam = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-zoom-in = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-zoom-out = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-zoom-reset = Usi inherit per il predefinito selezionato, disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-shortcuts-open-in-tab = Usi inherit per il predefinito selezionato (Shift+Invio), disabled per rimuovere questa azione, o una combinazione come Ctrl+Shift+K. Le combinazioni in conflitto o non valide non vengono eseguite.
admin-settings-help-mail-folder-role-inbox = Null usa il rilevamento automatico. Usi un oggetto con percorso esatto per i valori predefiniti condivisi o un oggetto con l'ID di una casella propria per un account. Destinazioni mancanti o in conflitto disattivano questo ruolo; si applicano i blocchi dell'amministratore.
admin-settings-help-mail-folder-role-drafts = Null usa il rilevamento automatico. Usi un oggetto con percorso esatto per i valori predefiniti condivisi o un oggetto con l'ID di una casella propria per un account. Destinazioni mancanti o in conflitto disattivano questo ruolo; si applicano i blocchi dell'amministratore.
admin-settings-help-mail-folder-role-sent = Null usa il rilevamento automatico. Usi un oggetto con percorso esatto per i valori predefiniti condivisi o un oggetto con l'ID di una casella propria per un account. Destinazioni mancanti o in conflitto disattivano questo ruolo; si applicano i blocchi dell'amministratore.
admin-settings-help-mail-folder-role-trash = Null usa il rilevamento automatico. Usi un oggetto con percorso esatto per i valori predefiniti condivisi o un oggetto con l'ID di una casella propria per un account. Destinazioni mancanti o in conflitto disattivano questo ruolo; si applicano i blocchi dell'amministratore.
admin-settings-help-mail-folder-role-junk = Null usa il rilevamento automatico. Usi un oggetto con percorso esatto per i valori predefiniti condivisi o un oggetto con l'ID di una casella propria per un account. Destinazioni mancanti o in conflitto disattivano questo ruolo; si applicano i blocchi dell'amministratore.
admin-settings-help-mail-folder-role-archive = Null usa il rilevamento automatico. Usi un oggetto con percorso esatto per i valori predefiniti condivisi o un oggetto con l'ID di una casella propria per un account. Destinazioni mancanti o in conflitto disattivano questo ruolo; si applicano i blocchi dell'amministratore.
admin-settings-help-security-protection-adaptive-clients = Attiva questa dimensione del soggetto quando la modalità adattiva è attiva. Account e amministratori vengono contati solo dopo la verifica della loro identità.
admin-settings-help-security-protection-adaptive-accounts = Attiva questa dimensione del soggetto quando la modalità adattiva è attiva. Account e amministratori vengono contati solo dopo la verifica della loro identità.
admin-settings-help-security-protection-adaptive-admins = Attiva questa dimensione del soggetto quando la modalità adattiva è attiva. Account e amministratori vengono contati solo dopo la verifica della loro identità.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-modifier-selection = Consente la selezione di intervalli con Ctrl, Command e Maiusc. Blocchi questo valore per imporlo; i controlli di selezione visibili restano disponibili.
admin-settings-key-mail-drag-to-folder = Trascina i messaggi nelle cartelle
admin-settings-help-mail-drag-to-folder = Consente di trascinare i messaggi dell'account nelle cartelle reali. Ciò non concede il permesso di scrittura sulla casella; il server autorizza comunque ogni spostamento.
admin-settings-key-admin-allow-last-superadmin-loss = Consenti la perdita dell'ultimo Super admin
admin-settings-help-admin-allow-last-superadmin-loss = Disattivato per impostazione predefinita. Attivarlo consente che modifiche successive rimuovano l'ultimo accesso Super admin. Un Super admin completo deve confermare la conseguenza con un'autenticazione recente.
admin-settings-last-superadmin-title = Consentire la perdita di tutto l'accesso Super admin?
admin-settings-last-superadmin-body = Modifiche successive agli account potrebbero lasciare nessuno in grado di amministrare questa distribuzione. Il ripristino richiederebbe l'amministrazione locale della distribuzione. Digiti ALLOW LAST SUPERADMIN LOSS per attivare questa eccezione.
admin-settings-last-superadmin-confirm = Consenti la perdita di accesso
admin-settings-key-mail-folder-counts = Contatori delle cartelle
admin-settings-help-mail-folder-counts = Scelga nascosti, non letti, non letti solo finché c'è posta non letta, totale o entrambi i contatori accanto alle cartelle. Imposti valori predefiniti globali, di dominio o di utente e blocchi la scelta quando serve. Si applica a ogni casella collegata di un account.
admin-settings-key-mail-folder-count-scope = Cartelle con contatori
admin-settings-help-mail-folder-count-scope = Tutte le cartelle, oppure solo le cartelle principali (Posta in arrivo, Bozze, Inviata, Archivio, Posta indesiderata, Cestino, Posta in uscita e altre cartelle a uso speciale). Con le sole cartelle principali, le cartelle personalizzate e le loro sottocartelle non mostrano il contatore. Si applica a ogni casella collegata di un account.
admin-settings-key-mail-compact-folder-badges = Badge secondari compatti delle cartelle
admin-settings-help-mail-compact-folder-badges = Mostra badge più piccoli fuori da Posta in arrivo. Gli utenti possono scegliere la propria dimensione, a meno che questa impostazione sia bloccata.
admin-settings-key-mail-focused-inbox = Posta in arrivo con priorità
admin-settings-help-mail-focused-inbox = Divide Posta in arrivo nelle schede Con priorità e Altro; newsletter e mittenti automatici vanno in Altro. Disattivato per impostazione predefinita. Gli utenti possono attivarlo o disattivarlo in Impostazioni > Cartelle e barra laterale, a meno che questa impostazione sia bloccata.
admin-settings-key-mail-account-unread-badge = Contatore dei non letti accanto al nome della casella
admin-settings-help-mail-account-unread-badge = Mostra il contatore dei non letti di Posta in arrivo accanto all'indirizzo o al nome visualizzato connesso nella barra laterale. Disattivato per impostazione predefinita. I contatori delle altre caselle nel menu delle caselle sono sempre mostrati. Si applica a ogni casella collegata di un account.
admin-settings-key-privacy-remote-image-downloads = Download manuali di immagini remote
admin-settings-key-privacy-remote-images-toast = Notifica di caricamento delle immagini
admin-settings-help-privacy-remote-images-toast = Mostra una notifica nel lettore mentre le immagini remote di un messaggio vengono caricate tramite il proxy di privacy, e indica quando sono state caricate o quante non è stato possibile caricare. Attivo per impostazione predefinita. Cambia solo ciò che il lettore mostra; quali immagini vengono caricate è ancora deciso dal criterio sui contenuti remoti.
admin-settings-help-privacy-remote-image-downloads = Consente il download esplicito di un'immagine referenziata dal messaggio dell'utente tramite il proxy di privacy protetto. Disattivato per impostazione predefinita; non apre mai direttamente l'URL esterno.
admin-settings-key-mail-resume-tabs = Riprendi le schede dell'area di lavoro
admin-settings-help-mail-resume-tabs = Ripristina i riferimenti alle schede dell'account dopo il ricaricamento. Disattivarlo cancella il registro locale dell'area di lavoro senza eliminare le bozze.
admin-settings-key-metrics-default-range-seconds = Intervallo predefinito della cronologia delle richieste
admin-settings-help-metrics-default-range-seconds = Secondi mostrati quando una richiesta di cronologia non specifica un intervallo. Limitato dalla conservazione e dall'intervallo massimo.
admin-settings-key-metrics-max-range-seconds = Intervallo massimo della cronologia delle richieste
admin-settings-help-metrics-max-range-seconds = Massimo di secondi per query di cronologia. La conservazione resta un limite aggiuntivo.
admin-settings-key-contacts-favorites-first = Preferiti per primi nei suggerimenti
admin-settings-help-contacts-favorites-first = Elenca i contatti preferiti prima delle altre corrispondenze nei suggerimenti dei destinatari. Cambia solo l'ordine; nessun suggerimento viene nascosto. Le sostituzioni per utente e dominio possono essere bloccate.
admin-settings-key-contacts-sender-icons = Icone dei mittenti incluse
admin-settings-help-contacts-sender-icons = Mostra icone incluse localmente per i domini dei mittenti riconosciuti, con iniziali generate per gli altri. Le foto dei contatti hanno la priorità. Le icone non verificano l'identità di un mittente. Nessun servizio esterno di loghi viene contattato; le sostituzioni per utente e dominio possono essere bloccate.
admin-settings-key-mail-brand-avatars = Loghi dei marchi accanto ai mittenti verificati
admin-settings-help-mail-brand-avatars = Mostra un logo del marchio incluso localmente accanto al mittente nel lettore, solo quando il messaggio ha superato DKIM e un controllo DMARC allineato per il dominio del marchio stesso. I messaggi che non superano o non hanno questi controlli mantengono l'avatar normale. Nessun logo viene recuperato dal mittente o da un servizio di loghi.
admin-settings-key-mail-draft-save-mode = Modalità di salvataggio delle bozze dell'account
admin-settings-help-mail-draft-save-mode = Salva dopo una pausa di digitazione, periodicamente o solo su richiesta. Il salvataggio richiede una cartella Bozze a monte supportata; le scritture incerte non vengono mai ripetute automaticamente.
admin-settings-key-mail-draft-debounce-ms = Pausa di digitazione della bozza (millisecondi)
admin-settings-help-mail-draft-debounce-ms = Attende da 500 a 10000 millisecondi dopo la modifica prima di salvare in modalità in tempo reale.
admin-settings-key-mail-draft-interval-secs = Intervallo di salvataggio della bozza (secondi)
admin-settings-help-mail-draft-interval-secs = Salva le bozze modificate ogni 5-3600 secondi in modalità periodica. Le bozze non modificate non vengono riscritte.
admin-settings-key-login-remember-signins = Ricorda gli indirizzi di accesso riusciti
admin-settings-help-login-remember-signins = Conserva su questo dispositivo fino a dieci nomi utente o indirizzi e-mail usati con successo per il menu a tendina dell'accesso. Non salva mai password né sessioni. Disattivarlo rimuove gli indirizzi salvati di questo server la prossima volta che un dispositivo carica la pagina di accesso o completa un accesso; non controlla il riempimento automatico del browser stesso.
admin-settings-key-login-show-remembered-hint = Mostra la spiegazione degli indirizzi salvati
admin-settings-help-login-show-remembered-hint = Mostra un testo esplicativo nel menu a tendina degli accessi precedenti. Nascosto per impostazione predefinita. Si applica solo quando è attivo il ricordo degli accessi riusciti.
admin-settings-key-mail-show-from = Mostra il campo Da dell'editor
admin-settings-help-mail-show-from = Mostra Da per impostazione predefinita nei nuovi editor. Gli utenti possono mostrarlo per un messaggio; solo gli indirizzi autorizzati possono inviare.
admin-settings-key-accounts-linking-enabled = Consenti account di caselle collegate
admin-settings-help-accounts-linking-enabled = Disattivato per impostazione predefinita. La disattivazione globale blocca ogni accesso collegato; entrambi gli account devono inoltre rispettare il proprio criterio di dominio e di account. Il collegamento richiede l'effettiva autenticazione dell'account figlio e una prova recente di quello principale.
admin-settings-key-accounts-use-linked = Usa le caselle collegate
admin-settings-help-accounts-use-linked = Esclusione dell'utente dall'accesso collegato. Non può annullare una disattivazione amministrativa. I dati della casella vengono conservati quando l'accesso è disattivato.
admin-settings-key-accounts-max-linked = Numero massimo di caselle collegate
admin-settings-help-accounts-max-linked = Numero massimo di caselle figlie verificate per identità principale, da 1 a 16. Predefinito 5.
admin-settings-key-accounts-store-linked-credentials = Mantieni l'accesso alle caselle collegate
admin-settings-help-accounts-store-linked-credentials = Conserva sigillata sul server la password di ogni casella collegata, così l'accesso si rinnova da solo, tra sessioni e riavvii. Non viene mai conservata per account ad accesso zero né per caselle con verifica in due passaggi. Disattivarlo cancella le password memorizzate; i collegamenti si rinnovano poi accedendo di nuovo. Attivo per impostazione predefinita.
admin-settings-key-privacy-strip-mailer-headers = Rimuovi le intestazioni del programma di posta dai messaggi inviati
admin-settings-help-privacy-strip-mailer-headers = Rimuove X-Mailer, User-Agent e X-MimeOLE di primo livello dalle e-mail SMTP ed EWS native. I messaggi allegati e i payload dei report restano intatti. Gli invii tramite il proxy JMAP a monte non sono coperti.
admin-settings-key-images-trusted-hosts = Host di immagini attendibili
admin-settings-help-images-trusted-hosts = Fino a 100 host di immagini, e i loro sottodomini, le cui immagini remote vengono caricate senza un'autorizzazione per messaggio quando i contenuti remoti sono su chiedi. Non annulla mai la modalità di blocco; un host bloccato resta bloccato.
admin-settings-key-images-blocked-hosts = Host di immagini bloccati
admin-settings-help-images-blocked-hosts = Fino a 100 host di immagini, e i loro sottodomini, le cui immagini remote non vengono mai caricate in nessuna modalità. Prevale sugli host attendibili e su qualsiasi autorizzazione.
admin-settings-key-links-trusted-domains = Domini di link attendibili
admin-settings-help-links-trusted-domains = Fino a 100 domini esatti i cui link si aprono da Rivedi i link senza il passaggio della destinazione. Nessun carattere jolly, schema, porta o indirizzo IP. I link HTTP semplici vengono sempre rivisti. Blocchi questa impostazione per impedire agli utenti di modificare l'elenco.
admin-settings-key-links-trusted-subdomains = Considera attendibili i sottodomini dei domini di link attendibili
admin-settings-help-links-trusted-subdomains = Disattivato: è attendibile solo il dominio esatto. Attivo: sono attendibili anche i nomi che terminano con .<domain>. Il confronto avviene su etichette intere, quindi example.com non rende mai attendibile evilexample.com.
admin-settings-key-cap-compose-rich-text = Scrittura in testo formattato
admin-settings-help-cap-compose-rich-text = Se gli utenti possono scrivere messaggi formattati (HTML). Disattivato invia solo testo normale.
admin-settings-key-cap-compose-attachments = Allegati nei nuovi messaggi
admin-settings-help-cap-compose-attachments = Se gli utenti possono allegare file ai messaggi che scrivono. Le immagini in linea nel corpo non contano.
admin-settings-key-cap-compose-share-links = Link di condivisione per file di grandi dimensioni
admin-settings-help-cap-compose-share-links = Se gli utenti possono inviare un file grande come link di condivisione Nextcloud invece che come allegato.
admin-settings-key-cap-compose-scheduled-send = Invio programmato
admin-settings-help-cap-compose-scheduled-send = Se gli utenti possono programmare l'invio di un messaggio a un momento successivo.
admin-settings-key-cap-compose-read-receipts = Richiedi le conferme di lettura
admin-settings-help-cap-compose-read-receipts = Se gli utenti possono chiedere ai destinatari una conferma di lettura.
admin-settings-key-cap-signatures = Firme
admin-settings-help-cap-signatures = Se gli utenti possono creare e modificare le proprie firme.
admin-settings-key-cap-identities = Identità di invio
admin-settings-help-cap-identities = Se gli utenti possono aggiungere e modificare le identità con cui inviano.
admin-settings-key-cap-keys = Chiavi OpenPGP e S/MIME
admin-settings-help-cap-keys = Se gli utenti dispongono del modulo Chiavi e possono firmare e cifrare i messaggi.
admin-settings-key-cap-links-trusted = Modifica i domini di link attendibili
admin-settings-help-cap-links-trusted = Se gli utenti possono modificare il proprio elenco di domini di link attendibili. L'elenco dell'amministratore continua ad applicarsi.
admin-settings-key-cap-zoom = Zoom del lettore
admin-settings-help-cap-zoom = Se gli utenti possono ingrandire il lettore di messaggi.
admin-settings-key-cap-mail-focused-inbox = Posta in arrivo con priorità
admin-settings-help-cap-mail-focused-inbox = Se gli utenti possono attivare le schede Con priorità e Altro della posta in arrivo.
admin-settings-key-cap-views-events = Vista Eventi
admin-settings-help-cap-views-events = Se gli utenti dispongono della vista Eventi della posta di calendario.
admin-settings-key-cap-views-attachments = Vista Allegati
admin-settings-help-cap-views-attachments = Se gli utenti dispongono della vista Allegati. I messaggi restano raggiungibili tramite la ricerca.
admin-settings-key-cap-search-operators = Operatori di ricerca
admin-settings-help-cap-search-operators = Se gli utenti possono cercare con operatori come from:, subject: o has:attachment.
admin-settings-key-cap-saved-searches = Ricerche salvate
admin-settings-help-cap-saved-searches = Se gli utenti possono salvare le ricerche.
admin-settings-key-cap-vacation = Risposta automatica di assenza
admin-settings-help-cap-vacation = Se gli utenti possono impostare una risposta automatica di assenza.
admin-settings-key-cap-labels = Etichette
admin-settings-help-cap-labels = Se gli utenti possono applicare etichette ai messaggi.
admin-settings-key-cap-snooze = Posticipa
admin-settings-help-cap-snooze = Se gli utenti possono posticipare i messaggi. Terminare un posticipo resta possibile.
admin-settings-key-cap-follow-up = Promemoria di follow-up
admin-settings-help-cap-follow-up = Se gli utenti possono impostare promemoria di follow-up sui messaggi. Rimuoverne uno resta possibile.
admin-settings-key-cap-masked = Indirizzi mascherati
admin-settings-help-cap-masked = Se gli utenti possono creare e gestire indirizzi mascherati.
admin-settings-key-cap-directory = Ricerca nella directory
admin-settings-help-cap-directory = Se gli utenti possono cercare nella directory dell'organizzazione.
admin-settings-key-cap-mcp = Accesso MCP
admin-settings-help-cap-mcp = Se le chiavi e i token dell'utente possono usare l'endpoint MCP.
admin-settings-key-cap-appearance-custom = Personalizzazione dell'aspetto
admin-settings-help-cap-appearance-custom = Se gli utenti possono modificare tema, colore di risalto, colori personalizzati e dei badge, densità e carattere. Disattivato mantiene l'aspetto dell'amministratore.
admin-settings-key-cap-appearance-custom-code = CSS e script personalizzati
admin-settings-help-cap-appearance-custom-code = Se gli utenti possono salvare il proprio CSS e gli script di aspetto. Gli interruttori di distribuzione in Aspetto continuano a valere sopra questo.
admin-settings-key-cap-avatars = Avatar delle caselle
admin-settings-help-cap-avatars = Se gli utenti possono modificare gli avatar delle caselle.
admin-settings-key-cap-shortcuts = Scorciatoie da tastiera personalizzate
admin-settings-help-cap-shortcuts = Se gli utenti possono modificare le scorciatoie da tastiera. Le scorciatoie predefinite continuano a funzionare.
admin-settings-key-cap-analytics = Analisi del prodotto
admin-settings-help-cap-analytics = Se agli utenti viene chiesto il consenso all'analisi e possono inviare eventi di utilizzo.
admin-settings-key-cap-apps-native = App desktop e mobili
admin-settings-help-cap-apps-native = Se gli utenti possono usare le app desktop e mobili.
admin-settings-key-access-protocols = Protocolli di accesso alla posta
admin-settings-help-access-protocols = Con quali protocolli del server di posta questi utenti possono accedere: imap, pop3, ews. Gli accessi tramite un server JMAP seguono quel server.
admin-settings-key-limit-linked-mailboxes = Caselle collegate per utente
admin-settings-help-limit-linked-mailboxes = Quante caselle un utente può collegare, al di sotto del massimo del server. 0 significa che si applica solo il massimo del server.
admin-settings-key-login-show-account-help-notice = Mostra l'avviso di aiuto per l'account all'accesso
admin-settings-help-login-show-account-help-notice = Mostra una breve spiegazione dopo un accesso non riuscito. Nascosto per impostazione predefinita; l'avviso non rivela se un account esiste o è disattivato.
admin-settings-key-appearance-leetspeak = Stile di testo leetspeak
admin-settings-help-appearance-leetspeak = Mostra le etichette dell'interfaccia e il testo dei messaggi in sola lettura in leetspeak. Il messaggio originale e il testo modificabile restano invariati. Gli utenti possono scegliere il testo normale, a meno che questa impostazione sia bloccata.
admin-settings-key-mail-cache-enabled = Attiva la cache di sessione
admin-settings-key-mail-cache-mailboxes = Memorizza nella cache gli elenchi di messaggi delle cartelle
admin-settings-key-mail-cache-messages = Memorizza nella cache i messaggi aperti
admin-settings-key-mail-cache-fresh-seconds = Intervallo di freschezza (secondi)
admin-settings-key-mail-cache-max-age-seconds = Età massima in cache (secondi)
admin-settings-key-mail-cache-max-entries = Voci massime in cache
admin-settings-key-mail-cache-max-bytes = Budget di memoria (byte)
admin-settings-key-mail-cache-prefetch-enabled = Attiva il precaricamento speculativo
admin-settings-key-mail-cache-hover-enabled = Precarica quando si punta una cartella
admin-settings-key-mail-cache-focus-enabled = Precarica quando una cartella riceve il focus da tastiera
admin-settings-key-mail-cache-dwell-ms = Ritardo di puntatore e focus (millisecondi)
admin-settings-key-mail-cache-idle-enabled = Precarica le cartelle quando inattivo
admin-settings-key-mail-cache-idle-delay-ms = Ritardo di inattività (millisecondi)
admin-settings-key-mail-cache-max-concurrency = Precaricamenti simultanei massimi
admin-settings-key-mail-cache-max-queued = Precaricamenti in coda massimi
admin-settings-key-mail-cache-preload-messages = Consenti corpi di messaggi speculativi
admin-settings-key-mail-cache-excluded-folders = Cartelle escluse dal precaricamento
admin-settings-key-mail-cache-refresh-on-open = Aggiorna il contenuto in cache all'apertura
admin-settings-help-mail-cache-refresh-on-open = Aggiorna il contenuto in cache all'apertura
admin-settings-key-mail-templates-enabled = Modelli di risposta
admin-settings-help-mail-templates-enabled = Attiva nell'editor i modelli di risposta in testo normale, personali e condivisi. Gli utenti possono escluderli, a meno che questa impostazione sia bloccata.
admin-settings-key-mail-templates-allow-personal = Consenti modelli personali
admin-settings-help-mail-templates-allow-personal = Consente agli utenti di creare i propri modelli di risposta. I modelli condivisi sono gestiti separatamente dagli amministratori.
admin-settings-key-mail-templates-shared = Modelli di risposta condivisi
admin-settings-help-mail-templates-shared = Gestisca risposte riutilizzabili in testo normale per questo ambito. I modelli non impostano mai i destinatari né inviano messaggi.
admin-settings-key-mail-templates-personal = Modelli di risposta personali
admin-settings-help-mail-templates-personal = Risposte personali riutilizzabili in testo normale. Creare o modificare richiede che i modelli di risposta e i modelli personali siano attivi.
admin-settings-key-mail-compose-window = Finestra dell'editor
admin-settings-help-mail-compose-window = Apre le nuove bozze in una scheda dell'area di lavoro o in una finestra separata. I popup bloccati ripiegano su una scheda.
admin-settings-key-mail-reply-copy-to-original = Mantieni le risposte nella cartella originale
admin-settings-help-mail-reply-copy-to-original = Salva una copia della risposta inviata nella cartella del messaggio originale, dove supportato.
admin-settings-key-mail-compose-check-invalid-recipients = Controlla i destinatari non validi prima dell'invio
admin-settings-help-mail-compose-check-invalid-recipients = Che cosa accade quando un messaggio viene inviato a un indirizzo non valido: consentire, avvisare, avvisare e ricordare la scelta o rifiutare.
admin-settings-key-mail-compose-check-missing-attachment = Controlla l'allegato mancante prima dell'invio
admin-settings-help-mail-compose-check-missing-attachment = Che cosa accade quando il testo menziona un allegato ma nessuno è allegato: consentire, avvisare, avvisare e ricordare la scelta o rifiutare.
admin-settings-key-mail-compose-check-empty-subject = Controlla l'oggetto vuoto prima dell'invio
admin-settings-help-mail-compose-check-empty-subject = Che cosa accade quando un messaggio non ha oggetto: consentire, avvisare, avvisare e ricordare la scelta o rifiutare.
admin-settings-key-mail-compose-auto-chip-recipients = Completa automaticamente i destinatari digitati
admin-settings-help-mail-compose-auto-chip-recipients = Trasforma un indirizzo digitato in un chip destinatario quando è seguito da uno spazio, una virgola o un punto e virgola.
admin-settings-key-mail-compose-header-collapse = Intestazione dell'editor
admin-settings-help-mail-compose-header-collapse = Come si comportano le righe A, Cc, Ccn e Oggetto dell'editor: espanse, ridotte a un riepilogo di una riga, oppure auto (ridotte mentre si scrive il corpo, aperte quando un campo ha il focus). Gli utenti possono comprimerle o espanderle da soli, a meno che sia bloccato.
admin-settings-key-mail-compose-recipient-separator = Separatore dei destinatari
admin-settings-help-mail-compose-recipient-separator = Il separatore inserito dopo un destinatario completato: una virgola o un punto e virgola.
admin-settings-key-mail-reply-from-mode = Identità usata per risposte e inoltri
admin-settings-help-mail-reply-from-mode = Da quale identità viene inviata una risposta o un inoltro: quella a cui era indirizzato l'originale, quella predefinita, un selettore o quella attualmente selezionata.
admin-settings-key-mail-forward-mode = Formato di inoltro predefinito
admin-settings-help-mail-forward-mode = Inoltra il testo in linea o allega il messaggio originale come file .eml.
admin-settings-key-mail-show-go-to-top = Mostra il pulsante Vai in cima
admin-settings-help-mail-show-go-to-top = Mostra nell'elenco dei messaggi un pulsante Vai in cima che torna al primo messaggio. Disattivato per impostazione predefinita.
admin-settings-key-mail-show-create-folder-button = Mostra il pulsante Crea cartella
admin-settings-help-mail-show-create-folder-button = Mostra nella barra laterale un pulsante Crea cartella solo con icona accanto a Scrivi. Disattivato per impostazione predefinita.
admin-settings-key-security-protection-user-agent-mode = Modalità di osservazione dello user-agent
admin-settings-help-security-protection-user-agent-mode = Enforce per impostazione predefinita: le intestazioni di bot riconosciuti selezionate vengono bloccate subito quando è attivo il blocco dei bot alla prima richiesta. Observe registra le decisioni senza rifiutare; off disattiva questa dimensione. Le dichiarazioni delle intestazioni non sono identità verificate.
admin-settings-key-security-protection-user-agent-block-bots = Blocca i bot riconosciuti alla prima richiesta
admin-settings-help-security-protection-user-agent-block-bots = Vero per impostazione predefinita. Le intestazioni selezionate di bot di ricerca, IA, automazione e personalizzati attivano un blocco temporaneo HTTP 429 alla prima richiesta in modalità Enforce. Observe si limita a registrare; falso ripristina i budget di richieste precedenti. Intestazioni mancanti o insolite, ambito ed eccezioni mantengono il comportamento esistente.
admin-settings-key-security-protection-user-agent-categories = Categorie di user-agent
admin-settings-key-security-protection-user-agent-weak-requests = Soglia di richieste a segnale debole
admin-settings-key-security-protection-user-agent-crawler-requests = Soglia di richieste dei crawler
admin-settings-key-security-protection-user-agent-window-secs = Finestra di osservazione dello user-agent (secondi)
admin-settings-key-security-protection-user-agent-max-action = Risposta massima per user-agent
admin-settings-help-security-protection-user-agent-max-action = Intensificazione massima per prove deboli o di richieste ripetute, predefinita la limitazione. Questo tetto non limita i blocchi alla prima richiesta delle intestazioni di bot riconosciuti selezionate.
admin-settings-key-security-protection-user-agent-scope = Ambito del criterio per user-agent
admin-settings-key-security-protection-user-agent-adaptive = Osservazioni adattive dello user-agent
admin-settings-key-security-protection-user-agent-patterns = Pattern di user-agent
admin-settings-key-security-protection-user-agent-exceptions = Eccezioni di user-agent
admin-settings-key-mail-reader-unsafe-html-mode = HTML dei messaggi potenzialmente non sicuro
admin-settings-help-mail-reader-unsafe-html-mode = Ask offre per impostazione predefinita un pulsante di HTML sanificato per messaggio. Strict mostra solo testo sicuro; sanitized mostra automaticamente l'HTML ripulito. Il contenuto attivo e le risorse remote automatiche restano bloccati. Blocchi per imporre questo criterio.
admin-settings-key-reader-attachments-density = Densità degli allegati
admin-settings-help-reader-attachments-density = Pulsanti di file compatti per impostazione predefinita, oppure anteprime comode. Controlla la presentazione quando gli allegati di un messaggio sono espansi.
admin-settings-key-cap-compose-open-tracking = Consenti il tracciamento dell'apertura dei messaggi
admin-settings-help-cap-compose-open-tracking = Se questo account può registrare le aperture dei messaggi. Richiede un URL pubblico configurato; gli account disattivati non possono creare né associare token di tracciamento.
admin-settings-key-mail-open-tracking = Traccia le aperture per impostazione predefinita
admin-settings-help-mail-open-tracking = Scelta di tracciamento predefinita per messaggio per i messaggi formattati idonei. Gli utenti possono sostituirla, a meno che sia bloccata; disattivare la funzionalità di tracciamento prevale sempre.
admin-settings-key-cap-reading-original-html = Consenti il rendering dell'HTML originale
admin-settings-help-cap-reading-original-html = Se gli utenti possono scegliere la vista HTML originale per singoli messaggi. Il renderer sanificato predefinito e la sicurezza del riquadro isolato restano in vigore.
admin-settings-key-mail-reader-allow-original-html = Offri la vista HTML originale
admin-settings-help-mail-reader-allow-original-html = Mostra l'opzione per messaggio di usare l'HTML originale quando la funzionalità di lettura lo consente. Disattivato per impostazione predefinita; non cambia il renderer di messaggi predefinito.
admin-settings-key-privacy-tracker-override-allowed = Offri il consenso ai tracker per messaggio
admin-settings-help-privacy-tracker-override-allowed = Mostra l'azione Consenti i tracker nella barra di privacy del messaggio. Disponibile per impostazione predefinita, ma mai attiva senza un'autorizzazione esplicita. Un criterio di blocco dei tracker bloccato vieta comunque la sostituzione.
admin-settings-key-privacy-max-concurrent-global = Recuperi di immagini paralleli globali
admin-settings-help-privacy-max-concurrent-global = Numero massimo di richieste di rete di immagini attive in questo processo del server. Intervallo 1–256; predefinito 128. I limiti per messaggio e per account continuano ad applicarsi.
admin-settings-key-privacy-max-concurrent-pipeline = Pipeline di elaborazione immagini simultanee
admin-settings-help-privacy-max-concurrent-pipeline = Numero massimo di immagini ammesse insieme a recupero, decodifica e pubblicazione nella cache. Intervallo 1–256; predefinito 128. I limiti di rete e di decodificatore sono applicati separatamente.
admin-settings-key-privacy-max-concurrent-decodes = Decodificatori di immagini paralleli
admin-settings-help-privacy-max-concurrent-decodes = Numero massimo di decodifiche di immagini in esecuzione contemporaneamente. Intervallo 1–16; predefinito 10. Valori più alti usano più CPU e memoria.
admin-settings-key-privacy-max-queued-requests = Richieste di immagini in attesa
admin-settings-help-privacy-max-queued-requests = Numero massimo di richieste in attesa di capacità di rete o di elaborazione delle immagini. Intervallo 16–16384; predefinito 2048. L'overflow o la scadenza della coda restano ritentabili.
admin-settings-key-privacy-rate-burst = Margine di burst delle richieste di immagini
admin-settings-help-privacy-rate-burst = Token di recupero per account iniziali e massimi, reintegrati alla frequenza di richieste di immagini configurata. Intervallo 1–12000; predefinito 1024. I successi in cache non consumano token.
admin-settings-key-privacy-image-rate-mode = Dimensionamento della frequenza delle richieste di immagini
admin-settings-help-privacy-image-rate-mode = Automatic dimensiona burst e reintegro per account in base alla capacità delle repliche, al carico della coda e al numero di immagini del messaggio, entro i minimi e massimi sotto. Manual usa esattamente le impostazioni di margine di burst e frequenza delle richieste. Predefinito automatic. Le richieste oltre il margine attendono un token fino alla scadenza della coda di recupero; autenticazione e autorizzazioni vengono verificate prima.
admin-settings-help-mail-cache-enabled = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-mailboxes = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-messages = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-fresh-seconds = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-max-age-seconds = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-max-entries = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-max-bytes = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-prefetch-enabled = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-hover-enabled = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-focus-enabled = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-dwell-ms = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-idle-enabled = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-idle-delay-ms = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-max-concurrency = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-max-queued = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-preload-messages = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-mail-cache-excluded-folders = Conserva nella memoria di questa scheda le cartelle e i messaggi visitati di recente. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o disconnettersi lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
admin-settings-help-security-protection-user-agent-categories = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-weak-requests = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-crawler-requests = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-window-secs = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-scope = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-adaptive = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-patterns = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.
admin-settings-help-security-protection-user-agent-exceptions = Configura questa parte dell'osservazione facoltativa dello user-agent. Il testo dello user-agent fornito dal client è un segnale debole, non una prova di identità o di abuso.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-rate-burst-floor = Minimo del margine di burst automatico
admin-settings-help-privacy-image-rate-burst-floor = Margine di burst per account più basso nella modalità di frequenza automatica. Intervallo 1–12000; predefinito 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Massimo del margine di burst automatico
admin-settings-help-privacy-image-rate-burst-ceiling = Margine di burst per account più alto nella modalità di frequenza automatica. Intervallo 1–12000; predefinito 2048.
admin-settings-key-privacy-image-rate-refill-floor = Minimo del ripristino automatico
admin-settings-help-privacy-image-rate-refill-floor = Ripristino per account più basso, in token di recupero al minuto, nella modalità di frequenza automatica. Intervallo 10–60000; predefinito 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Massimo del ripristino automatico
admin-settings-help-privacy-image-rate-refill-ceiling = Ripristino per account più alto, in token di recupero al minuto, nella modalità di frequenza automatica. Intervallo 10–60000; predefinito 6000.
admin-settings-key-privacy-image-cache-mode = Dimensionamento della cache immagini
admin-settings-help-privacy-image-cache-mode = Automatico dimensiona la cache in base alla memoria rimasta dopo le riserve per decoder e pipeline e allunga la durata finché il tasso di hit è alto e c'è memoria libera. Manuale usa esattamente le impostazioni di dimensione della cache, numero di voci e durata. Predefinito automatico.
admin-settings-key-privacy-image-cache-bytes-floor = Minimo della dimensione automatica della cache
admin-settings-help-privacy-image-cache-bytes-floor = Cache immagini più piccola, in byte, in modalità automatica. Intervallo 0–1073741824; predefinito 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Massimo della dimensione automatica della cache
admin-settings-help-privacy-image-cache-bytes-ceiling = Cache immagini più grande, in byte, in modalità automatica. Intervallo 0–1073741824; predefinito 268435456.
admin-settings-key-privacy-cache-entries = Limite di voci della cache immagini
admin-settings-help-privacy-cache-entries = Numero massimo di immagini in cache; il conteggio fisso in modalità manuale e il massimo in modalità automatica. Intervallo 16–65536; predefinito 4096.
admin-settings-key-privacy-cache-entry-bytes = Immagine più grande in cache
admin-settings-help-privacy-cache-entry-bytes = Singola immagine ricodificata più grande mantenuta in cache, in byte. Le immagini più grandi vengono servite ma non conservate. Intervallo 65536–16777216; predefinito 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Minimo della durata automatica della cache
admin-settings-help-privacy-image-cache-ttl-floor-secs = Durata minima della cache in secondi in modalità automatica, usata con un tasso di hit basso o sotto pressione di memoria. Intervallo 0–86400; predefinito 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Massimo della durata automatica della cache
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Durata massima della cache in secondi in modalità automatica, usata con un tasso di hit alto e memoria libera. Intervallo 0–86400; predefinito 21600.
admin-settings-key-privacy-cache-ttl-secs = Durata della cache delle immagini riuscite
admin-settings-help-privacy-cache-ttl-secs = Secondi per cui le immagini decodificate restano riutilizzabili in questo processo. Intervallo 0–86400; predefinito 3600. Zero disattiva il riutilizzo delle immagini riuscite; l'autorizzazione viene sempre verificata prima.
admin-settings-key-privacy-image-latency-tracking = Monitoraggio della velocità del proxy immagini
admin-settings-help-privacy-image-latency-tracking = Registra quanto impiega ogni immagine passata dal proxy (mediana, p95, p99 e il tempo di attesa, recupero e decodifica) come conteggi in intervalli di tempo fissi in memoria. Non vengono memorizzati URL, host o account. Attivo per impostazione predefinita; la cronologia si azzera al riavvio ed è per processo del server. Disattivarlo interrompe le nuove registrazioni; la sezione Velocità continua a mostrare quanto registrato.
admin-settings-key-privacy-negative-cache-ttl-secs = Durata della cache delle immagini non riuscite
admin-settings-help-privacy-negative-cache-ttl-secs = Secondi per cui si ricordano i guasti idonei del server di origine, per evitare richieste ripetute. Intervallo 0–300; predefinito 10. Zero disattiva il riutilizzo dei guasti. Un nuovo tentativo esplicito cancella i guasti interessati.
admin-settings-key-privacy-image-retry-attempts = Nuovi tentativi automatici per le immagini
admin-settings-help-privacy-image-retry-attempts = Numero massimo di nuovi tentativi per errori temporanei delle immagini in un caricamento visibile di un messaggio. Intervallo 0–8; predefinito 3. Zero disattiva i tentativi automatici mantenendo quello manuale.
admin-settings-key-privacy-image-retry-delay-ms = Ritardo dei nuovi tentativi automatici per le immagini
admin-settings-help-privacy-image-retry-delay-ms = Millisecondi tra i nuovi tentativi automatici, dopo che il frame corrente si è stabilizzato. Intervallo 500–60000; predefinito 5000. I rifiuti permanenti non vengono ritentati automaticamente.
admin-settings-key-privacy-image-decode-timeout-ms = Timeout di decodifica delle immagini
admin-settings-help-privacy-image-decode-timeout-ms = Millisecondi di tempo reale consentiti per una decodifica di immagine in sandbox dopo il recupero. Intervallo 1000–60000; predefinito 10000. Aumentare per immagini grandi; i timeout di rete restano separati.
admin-settings-key-privacy-image-concurrency-mode = Profilo di concorrenza delle immagini
admin-settings-help-privacy-image-concurrency-mode = Automatico si adatta alla capacità della CPU, alla memoria del container e al numero di immagini approvate di ogni messaggio. Poche risorse, Bilanciato e Alta velocità di elaborazione si adattano anch'essi, con limiti di sicurezza diversi. Manuale usa direttamente i limiti numerici. Applicare un profilo modifica solo questa impostazione.
admin-settings-cat-image-proxy = Proxy immagini
admin-settings-key-privacy-image-ram-max-mi-b = Massimo del livello RAM
admin-settings-help-privacy-image-ram-max-mi-b = Limite superiore in MiB per il livello immagini in memoria, applicato in aggiunta alla dimensione della cache automatica o manuale. Zero non impone alcun limite. Mantenerlo piccolo e lasciare che il livello su disco conservi la maggior parte. Intervallo 0–1048576; predefinito 0.
admin-settings-key-privacy-image-disk-enabled = Livello su disco delle immagini
admin-settings-help-privacy-image-disk-enabled = Conserva su disco, dietro il livello RAM, le immagini già ricodificate. Disattivato per impostazione predefinita; se la directory manca, è piena o in sola lettura, la cache continua silenziosamente solo in RAM. Viene memorizzato solo l'output del jail, con nomi di file hash.
admin-settings-key-privacy-image-disk-path = Directory su disco delle immagini
admin-settings-help-privacy-image-disk-path = Directory assoluta usata per il livello su disco. Se vuota, usa una cartella image-cache accanto al file del database. Montare lì un volume scrivibile quando il file system radice è in sola lettura. I percorsi relativi e .. vengono rifiutati e lasciano il livello disattivato.
admin-settings-key-privacy-image-disk-mode = Dimensionamento del disco immagini
admin-settings-help-privacy-image-disk-mode = Automatico usa un decimo dello spazio che questa cache potrebbe usare (spazio libero più quello già occupato), tra il minimo e il massimo e mai più della metà di tale spazio. Manuale usa esattamente la dimensione massima indicata sotto. Predefinito automatico.
admin-settings-key-privacy-image-disk-max-mi-b = Dimensione del disco immagini (manuale)
admin-settings-help-privacy-image-disk-max-mi-b = Dimensione del livello su disco in MiB in modalità manuale; la lettura mostra in entrambi i casi la dimensione effettiva. Intervallo 1–1048576; predefinito 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Minimo della dimensione automatica del disco
admin-settings-help-privacy-image-disk-floor-mi-b = Livello su disco più piccolo in MiB in modalità automatica; usato anche finché lo spazio libero è sconosciuto. Intervallo 0–1048576; predefinito 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Massimo della dimensione automatica del disco
admin-settings-help-privacy-image-disk-ceiling-mi-b = Livello su disco più grande in MiB in modalità automatica. Intervallo 0–1048576; predefinito 4096.
admin-settings-key-privacy-image-disk-entries = Limite di voci su disco
admin-settings-help-privacy-image-disk-entries = Numero massimo di immagini conservate su disco. Ogni voce costa anche una quantità fissa di RAM nell'indice, conteggiata nel livello RAM. Intervallo 16–1048576; predefinito 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Immagine più grande su disco
admin-settings-help-privacy-image-disk-entry-bytes = Singola immagine ricodificata più grande scritta su disco, in byte. Le immagini oltre il limite di voce RAM ma entro questo vanno direttamente su disco. Intervallo 65536–67108864; predefinito 8388608.
admin-settings-key-privacy-image-disk-min-hits = Hit prima della scrittura su disco
admin-settings-help-privacy-image-disk-min-hits = Hit di cui una voce RAM ha bisogno prima che la sua rimozione la scriva su disco invece di scartarla. Zero scrive ogni voce rimossa il cui punteggio supera la soglia di declassamento. Intervallo 0–100; predefinito 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Coda di scrittura su disco
admin-settings-help-privacy-image-disk-write-queue-mi-b = Dati immagine in MiB che possono attendere il writer su disco in background. Contano nel livello RAM; oltre questo valore le immagini rimosse vengono scartate invece di essere accodate. Intervallo 1–256; predefinito 8.
admin-settings-key-privacy-image-ttl-step-secs = Incremento di durata per hit
admin-settings-help-privacy-image-ttl-step-secs = Secondi aggiunti alla durata residua di un'immagine a ogni hit, moltiplicati per 1 più il logaritmo del conteggio di hit con decadimento, così le immagini usate spesso durano più a lungo. Zero mantiene ogni immagine alla durata di base. Intervallo 0–86400; predefinito 600.
admin-settings-key-privacy-image-ttl-max-secs = Durata estesa massima
admin-settings-help-privacy-image-ttl-max-secs = Durata residua massima in secondi che gli hit possono accumulare. Intervallo 60–2592000; predefinito 86400.
admin-settings-key-privacy-image-max-age-secs = Età massima dell'immagine
admin-settings-help-privacy-image-max-age-secs = Età in secondi dopo la quale un'immagine viene recuperata di nuovo, indipendentemente dagli hit. Il percorso di recupero non conserva ETag né Last-Modified del server di origine, quindi non c'è rivalidazione condizionale; l'immagine viene semplicemente recuperata di nuovo. Intervallo 3600–7776000; predefinito 604800.
admin-settings-key-privacy-image-score-half-life-secs = Emivita del punteggio
admin-settings-help-privacy-image-score-half-life-secs = Secondi in cui il conteggio di hit e la recenza di un'immagine si dimezzano nel punteggio di priorità che decide RAM, disco e rimozione. Intervallo 60–604800; predefinito 3600.
admin-settings-key-privacy-image-score-demote-milli = Punteggio di declassamento
admin-settings-help-privacy-image-score-demote-milli = Punteggio, in millesimi, sotto il quale un'immagine in RAM viene declassata su disco o scartata. La soglia di promozione è questo valore più l'isteresi. Intervallo 0–10000; predefinito 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Isteresi di promozione
admin-settings-help-privacy-image-score-hysteresis-percent = Percentuale di cui la soglia di promozione sta sopra la soglia di declassamento. Un punteggio tra le due lascia l'immagine dov'è. Intervallo 0–500; predefinito 50.
admin-settings-key-privacy-image-context-open-percent = Peso del messaggio aperto
admin-settings-help-privacy-image-context-open-percent = Peso nel punteggio, in percentuale, per le immagini richieste da un messaggio. Le immagini di un messaggio contano come Recenti dopo dieci minuti di inattività. Intervallo 100–1000; predefinito 200.
admin-settings-key-privacy-image-context-recent-percent = Peso del messaggio recente
admin-settings-help-privacy-image-context-recent-percent = Peso nel punteggio, in percentuale, per le immagini usate di recente. Le richieste che non indicano alcun messaggio pesano 100. Intervallo 100–1000; predefinito 150.
admin-settings-key-privacy-image-account-share-percent = Quota di RAM per account
admin-settings-help-privacy-image-account-share-percent = Quota massima del livello RAM che gli inserimenti di un account possono riempire mentre altri account usano la cache; oltre di essa l'account rimuove prima le proprie immagini con il punteggio più basso. Un singolo account può usarla tutta. Intervallo 1–100; predefinito 50.
admin-settings-key-privacy-image-sweep-secs = Intervallo di pulizia della cache
admin-settings-help-privacy-image-sweep-secs = Secondi tra le scansioni in background che fanno scadere le immagini, ricalcolano il punteggio di un lotto limitato, declassano le immagini fredde in RAM sotto pressione e ricontrollano lo spazio libero su disco. Intervallo 5–3600; predefinito 30.
admin-settings-key-privacy-image-snapshot-secs = Intervallo dello snapshot dell'indice
admin-settings-help-privacy-image-snapshot-secs = Secondi tra gli snapshot atomici dell'indice su disco, che conservano conteggi di hit e durate estese tra i riavvii senza una scrittura per hit. Zero disattiva gli snapshot; dopo un riavvio viene letta l'intestazione di ogni file. Intervallo 0–86400; predefinito 300.
admin-settings-key-privacy-image-sweep-batch = Dimensione del lotto di scansione
admin-settings-help-privacy-image-sweep-batch = Voci che una scansione di manutenzione può far scadere, ricalcolare o rimuovere. Limita il lavoro svolto sotto il blocco della cache. Intervallo 16–8192; predefinito 256.
admin-image-proxy-title = Lettura della cache immagini
admin-image-proxy-unavailable = La lettura della cache immagini non è disponibile.
admin-image-proxy-unknown = sconosciuto
admin-image-proxy-ram = Livello RAM: { $used } usati su { $budget } (configurati { $configured }), { $entries } immagini, { $hot } calde, { $cold } fredde, tasso di hit { $hit }.
admin-image-proxy-disk-off = Livello su disco: disattivato.
admin-image-proxy-disk = Livello su disco: { $bytes } usati su { $budget } effettivi ({ $mode }; minimo { $floor }, massimo { $ceiling }, manuale { $manual }), { $entries } immagini, { $hot } calde, { $cold } fredde, tasso di hit { $hit }.
admin-image-proxy-disk-inactive = Livello su disco: attivo ma non operativo. { $reason }
admin-image-proxy-moves = Promozioni al minuto: { $perMin }. Promozioni totali { $promotions }, declassamenti { $demotions }, scarti { $drops }. Voci dell'indice: { $index }.
admin-image-proxy-refresh = Aggiorna lettura
admin-settings-key-security-trust-enabled = Livelli di fiducia
admin-settings-help-security-trust-enabled = Assegna a ogni richiesta un livello: autenticata, origine nota (un accesso riuscito recente da quell'indirizzo) o sconosciuta. Cambiano solo le quote delle richieste anonime; credenziali, secondi fattori e blocchi non vengono mai allentati.
admin-settings-key-security-trust-known-ttl-days = Finestra di origine nota (giorni)
admin-settings-help-security-trust-known-ttl-days = Per quanto tempo un accesso riuscito mantiene noto il suo indirizzo. I tentativi falliti non contano mai.
admin-settings-key-security-trust-known-capacity = Origini note conservate
admin-settings-help-security-trust-known-capacity = Numero massimo di origini note conservate in memoria (ciascuna con un hash salato, un paese e un conteggio di account). La più vecchia viene scartata per prima.
admin-settings-key-security-trust-persist = Salva le origini note tra i riavvii
admin-settings-help-security-trust-persist = Salva la tabella delle origini note (hash con chiave, paese, ultimo accesso, conteggio account) nell'archivio delle impostazioni e la carica all'avvio. Le voci più vecchie della finestra vengono scartate.
admin-settings-key-security-trust-persist-interval-secs = Intervallo di salvataggio delle origini note (secondi)
admin-settings-help-security-trust-persist-interval-secs = Secondi tra i salvataggi, da 30 a 3600. Il salvataggio avviene solo se la tabella è cambiata, e ancora una volta allo spegnimento ordinato.
admin-settings-key-security-trust-unknown-percent = Quota per origine sconosciuta (%)
admin-settings-help-security-trust-unknown-percent = Quota di richieste anonime di un'origine sconosciuta, come percentuale dei limiti configurati. 100 mantiene la base; un valore più basso è più restrittivo.
admin-settings-key-security-trust-known-percent = Quota per origine nota (%)
admin-settings-help-security-trust-known-percent = Quota di richieste anonime di un'origine nota. 200 raddoppia i budget di richieste e dimezza il tempo di attesa dell'accesso e la difficoltà della sfida. Non aggira mai credenziali, secondi fattori o blocchi.
admin-settings-key-security-trust-ua-missing = Bot: user-agent vuoto
admin-settings-help-security-trust-ua-missing = Azione per le richieste anonime con user-agent vuoto: consentire, limitare o bloccare. Le sessioni autenticate non sono mai interessate.
admin-settings-key-security-trust-ua-uncommon = Bot: user-agent non riconosciuto
admin-settings-help-security-trust-ua-uncommon = Azione per le richieste anonime con user-agent non riconosciuto: consentire, limitare o bloccare.
admin-settings-key-security-trust-ua-automation = Bot: script e automazione
admin-settings-help-security-trust-ua-automation = Azione per le richieste anonime che dichiarano di essere uno script, una libreria HTTP, un browser headless o un bot generico: consentire, limitare o bloccare.
admin-settings-key-security-trust-ua-search = Bot: crawler dei motori di ricerca
admin-settings-help-security-trust-ua-search = Azione per le richieste anonime che dichiarano di essere il crawler di un motore di ricerca: consentire, limitare o bloccare.
admin-settings-key-security-trust-ua-ai-training = Bot: crawler per l'addestramento di IA
admin-settings-help-security-trust-ua-ai-training = Azione per le richieste anonime che dichiarano di essere GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended o Applebot-Extended: consentire, limitare o bloccare. Una dichiarazione è un'intestazione, non un'identità verificata.
admin-settings-key-security-trust-ua-ai-search = Bot: crawler di ricerca IA
admin-settings-help-security-trust-ua-ai-search = Azione per le richieste anonime che dichiarano di essere OAI-SearchBot, PerplexityBot o Claude-SearchBot: consentire, limitare o bloccare.
admin-settings-key-security-trust-ua-ai-user = Bot: recuperi IA su richiesta dell'utente
admin-settings-help-security-trust-ua-ai-user = Azione per le richieste anonime che dichiarano di essere ChatGPT-User o Claude-User: consentire, limitare o bloccare.
admin-settings-key-security-trust-ua-custom = Bot: pattern personalizzati
admin-settings-help-security-trust-ua-custom = Azione per le richieste anonime che corrispondono ai pattern user-agent personalizzati: consentire, limitare o bloccare.
admin-settings-key-security-trust-admin-geo-enabled = Blocca la scoperta dell'admin da paesi sconosciuti
admin-settings-help-security-trust-admin-geo-enabled = Blocca un'origine che sonda la console di amministrazione da un paese senza accessi riusciti recenti. Richiede un database dei paesi leggibile e la protezione dalle sonde sull'admin. MW_ADMIN_GEO_ENABLED=false la disattiva per il ripristino.
admin-settings-key-security-trust-admin-geo-countries = Paesi sempre noti (blocco admin)
admin-settings-help-security-trust-admin-geo-countries = Codici paese ISO in maiuscolo sempre trattati come noti, oltre ai paesi degli accessi riusciti recenti.
admin-settings-key-security-trust-admin-geo-lookup-failure = Origine senza paese
admin-settings-help-security-trust-admin-geo-lookup-failure = allow non blocca e registra il caso; unknown tratta un'origine senza paese rilevato come un paese sconosciuto.
admin-settings-key-security-trust-admin-geo-threshold = Sonde prima di un blocco
admin-settings-help-security-trust-admin-geo-threshold = Sonde di scoperta dell'admin da un paese sconosciuto, entro la finestra, che avviano un blocco.
admin-settings-key-security-trust-admin-geo-window-secs = Finestra delle sonde (secondi)
admin-settings-help-security-trust-admin-geo-window-secs = Finestra per la soglia delle sonde.
admin-settings-key-security-trust-admin-geo-block-on-first = Blocca alla prima sonda
admin-settings-help-security-trust-admin-geo-block-on-first = Blocca alla prima sonda da un paese sconosciuto. Aumenta il rischio di bloccare un amministratore in viaggio.
admin-settings-key-security-trust-admin-geo-block-secs = Primo blocco (secondi)
admin-settings-help-security-trust-admin-geo-block-secs = Durata del primo blocco.
admin-settings-key-security-trust-admin-geo-second-block-secs = Secondo blocco (secondi)
admin-settings-help-security-trust-admin-geo-second-block-secs = Durata del secondo blocco della stessa origine nel periodo di recidiva.
admin-settings-key-security-trust-admin-geo-max-block-secs = Blocco per recidiva (secondi)
admin-settings-help-security-trust-admin-geo-max-block-secs = Durata del terzo blocco e di ogni blocco successivo nel periodo di recidiva.
admin-settings-key-security-trust-admin-geo-repeat-days = Periodo di recidiva (giorni)
admin-settings-help-security-trust-admin-geo-repeat-days = Un blocco entro questo numero di giorni dal precedente conta come recidiva e usa la durata successiva, più lunga.
admin-settings-key-security-signin-account-geo-mode = Criterio del paese di accesso per account
admin-settings-help-security-signin-account-geo-mode = Disattivato per impostazione predefinita. Blocca gli accessi da paesi che l'account non ha mai usato o da qualsiasi paese fuori dal suo elenco consentito. IP e paesi in lista consentita passano sempre. Si applica solo ai nuovi accessi al webmail, mai alle sessioni esistenti né ai client di protocollo. Il rilevamento del paese usa un database di geolocalizzazione IP che può essere errato o obsoleto; VPN, operatori mobili, proxy e viaggi possono bloccare un accesso legittimo o lasciarne passare uno estero. Aggiungere IP o paesi attendibili alla lista consentita.
admin-settings-key-security-signin-account-geo-allow-countries = Paesi consentiti dell'account
admin-settings-help-security-signin-account-geo-allow-countries = Codici paese ISO in maiuscolo sempre consentiti per l'account. Il rilevamento del paese usa un database di geolocalizzazione IP che può essere errato o obsoleto; VPN, operatori mobili, proxy e viaggi possono bloccare un accesso legittimo o lasciarne passare uno estero. Aggiungere IP o paesi attendibili alla lista consentita.
admin-settings-key-security-signin-account-geo-allow-ips = IP consentiti dell'account
admin-settings-help-security-signin-account-geo-allow-ips = Indirizzi IP o reti CIDR sempre consentiti per l'account. Un singolo indirizzo IPv6 equivale al suo /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Paesi consentiti per tutti gli account
admin-settings-help-security-signin-account-geo-global-allow-countries = Codici paese ISO in maiuscolo consentiti per tutti gli account, aggiunti all'elenco di ciascun account.
admin-settings-key-security-signin-account-geo-global-allow-ips = IP consentiti per tutti gli account
admin-settings-help-security-signin-account-geo-global-allow-ips = Indirizzi IP o reti CIDR (un ufficio o una VPN) consentiti per tutti gli account, aggiunti all'elenco di ciascun account.
admin-settings-key-security-signin-account-geo-lookup-failure = Origine senza paese (criterio account)
admin-settings-help-security-signin-account-geo-lookup-failure = Quando l'origine non ha un paese rilevato: consentire l'accesso e registrarlo, oppure negarlo.
admin-settings-key-security-signin-account-geo-history-days = Cronologia dei paesi dell'account (giorni)
admin-settings-help-security-signin-account-geo-history-days = Per quanti giorni un paese resta nella cronologia degli accessi dell'account per il criterio dei paesi insoliti. È conservata separatamente dal registro di autenticazione, quindi accorciare il registro non la accorcia.
admin-settings-key-security-geo-no-database = Nessun database dei paesi: comportamento predefinito
admin-settings-help-security-geo-no-database = Cosa fanno le funzioni basate sul paese quando nessun database è caricato: consentire, trattare l'origine come paese sconosciuto o bloccare (il blocco vale solo per gli accessi). Vale anche l'impostazione propria della funzione e prevale la più restrittiva.
admin-settings-key-security-geo-not-found = Indirizzo non presente nel database: comportamento predefinito
admin-settings-help-security-geo-not-found = Cosa fanno le funzioni basate sul paese quando un database è caricato ma non ha voci per l'indirizzo. Gli indirizzi privati, di loopback e link-local sono reti locali e non vengono mai interessati.
admin-settings-key-security-signin-geo-no-database = Criterio di accesso dell'istanza senza database
admin-settings-help-security-signin-geo-no-database = Quando il criterio del paese di accesso dell'istanza è attivo e nessun database è caricato: bloccare gli accessi (fail closed) o consentirli senza controlli. Le reti locali sono sempre consentite.
admin-settings-key-security-geo-session-lookup-failure = Avvisi di sessione senza paese
admin-settings-help-security-geo-session-lookup-failure = Quando il paese di una richiesta autenticata non può essere determinato: ignorarlo, oppure generare un avviso di sessione per sessione al giorno. Non blocca né revoca mai una sessione.
admin-settings-key-security-protocol-framing-mode = Protezione di protocollo: framing delle richieste
admin-settings-help-security-protocol-framing-mode = Classe request smuggling: Content-Length insieme a Transfer-Encoding, Transfer-Encoding diverso da esattamente chunked (o su HTTP/1.0 e HTTP/2), Content-Length non numerico o in conflitto. Applica risponde 400 e chiude la connessione; solo rilevamento si limita a contare. Un'esenzione per prefisso di route non si applica mai a questa classe.
admin-settings-key-security-protocol-host-mode = Protezione di protocollo: sintassi dell'Host
admin-settings-help-security-protocol-host-mode = Sintassi dell'intestazione Host: assente in HTTP/1.1, vuota, ripetuta, o con userinfo, spazi o caratteri di percorso. Letterali IPv6, porte e punto finale sono accettati. Applica risponde 400 e chiude la connessione.
admin-settings-key-security-protocol-host-pinning-mode = Protezione di protocollo: l'Host deve corrispondere al nome pubblico
admin-settings-help-security-protocol-host-pinning-mode = Host che non corrisponde all'host di MW_PUBLIC_URL né a uno degli allowedHosts. Non fa nulla finché nessuno dei due indica un host. Rilevamento per impostazione predefinita perché proxy e controlli di integrità possono inviare un nome interno.
admin-settings-key-security-protocol-path-control-mode = Protezione di protocollo: caratteri di controllo nel percorso
admin-settings-help-security-protocol-path-control-mode = NUL, caratteri di controllo o CR/LF codificati (%00, %0d, %0a) nel percorso della richiesta. Applica risponde 400. Mai esentato da un prefisso di route.
admin-settings-key-security-protocol-target-mode = Protezione di protocollo: trucchi nel target della richiesta
admin-settings-help-security-protocol-target-mode = Codifica percentuale doppia o eccessiva di punti e barre, punti codificati, segmenti punto, segmenti di percorso vuoti (//), barre rovesciate, byte grezzi non ASCII e un target in forma assoluta che indica un'autorità diversa da Host.
admin-settings-key-security-protocol-path-length-mode = Protezione di protocollo: lunghezza di percorso e query
admin-settings-help-security-protocol-path-length-mode = Percorso o query più lunghi di maxPathBytes o maxQueryBytes. Applica risponde 414.
admin-settings-key-security-protocol-method-mode = Protezione di protocollo: metodi HTTP
admin-settings-help-security-protocol-method-mode = Metodo fuori da allowedMethods (per impostazione predefinita TRACE, TRACK, CONNECT e metodi sconosciuti o in minuscolo). Applica risponde 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Protezione di protocollo: intestazioni a valore singolo ripetute
admin-settings-help-security-protocol-duplicate-headers-mode = Un'intestazione che dovrebbe comparire una sola volta e compare più volte (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, intestazioni condizionali e di intervallo), un Content-Length ripetuto con valori uguali, o intestazioni Cookie ripetute su HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Protezione di protocollo: intestazioni di inoltro da peer non attendibili
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* o X-Real-IP inviati da un peer fuori da MW_TRUSTED_PROXIES. Le intestazioni non sono mai considerate attendibili comunque; questo registra il tentativo. Conta per la protezione adattiva solo se sono configurati proxy attendibili.
admin-settings-key-security-protocol-header-hygiene-mode = Protezione di protocollo: igiene delle intestazioni
admin-settings-help-security-protocol-header-hygiene-mode = Numero di intestazioni, un singolo valore o l'intestazione Cookie oltre i limiti configurati; byte di intestazione non ASCII; Upgrade o Connection: upgrade fuori dalla route WebSocket; Connection che nomina intestazioni come Content-Length o Host. Applica risponde 431 per i rilievi di dimensione e numero e 400 per gli altri.
admin-settings-key-security-protocol-body-mode = Protezione di protocollo: framing del corpo della richiesta
admin-settings-help-security-protocol-body-mode = GET o HEAD con un corpo, Content-Length oltre maxBodyBytes (413), Expect diverso da 100-continue (417) o senza corpo.
admin-settings-key-security-protocol-max-path-bytes = Protezione di protocollo: percorso più lungo (byte)
admin-settings-help-security-protocol-max-path-bytes = Percorso di richiesta più lungo prima che la classe lunghezza del percorso lo segnali.
admin-settings-key-security-protocol-max-query-bytes = Protezione di protocollo: query più lunga (byte)
admin-settings-help-security-protocol-max-query-bytes = Stringa di query più lunga prima che la classe lunghezza del percorso la segnali.
admin-settings-key-security-protocol-max-header-count = Protezione di protocollo: massimo di intestazioni
admin-settings-help-security-protocol-max-header-count = Massimo di campi di intestazione in una richiesta prima che la classe igiene delle intestazioni lo segnali. Il limite rigido del listener (MW_HTTP_MAX_HEADERS) continua a valere.
admin-settings-key-security-protocol-max-header-value-bytes = Protezione di protocollo: valore di intestazione più lungo (byte)
admin-settings-help-security-protocol-max-header-value-bytes = Valore di una singola intestazione più lungo prima che la classe igiene delle intestazioni lo segnali.
admin-settings-key-security-protocol-max-cookie-bytes = Protezione di protocollo: intestazione Cookie più grande (byte)
admin-settings-help-security-protocol-max-cookie-bytes = Dimensione combinata delle intestazioni Cookie prima che la classe igiene delle intestazioni la segnali.
admin-settings-key-security-protocol-max-body-bytes = Protezione di protocollo: corpo dichiarato più grande (byte)
admin-settings-help-security-protocol-max-body-bytes = Content-Length più grande che la classe corpo accetta senza segnalarlo. Mantenerlo sopra il caricamento o l'importazione più grande che una route accetta; un'esenzione per prefisso di route può aumentarlo per una singola route.
admin-settings-key-security-protocol-allowed-methods = Protezione di protocollo: metodi consentiti
admin-settings-help-security-protocol-allowed-methods = Metodi che la classe metodi accetta. Il valore predefinito elenca i verbi HTTP ordinari e i verbi WebDAV, CalDAV e CardDAV; oggi il server non monta route DAV in ingresso, quindi questi ottengono un 405 al router.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protocol-allowed-hosts = Protezione di protocollo: host extra accettati
admin-settings-help-security-protocol-allowed-hosts = Nomi host accettati dalla classe di vincolo dell'host oltre all'host di MW_PUBLIC_URL, ad esempio un nome interno usato da un proxy o da un controllo di integrità. Porte e punto finale vengono ignorati.
admin-settings-key-security-protocol-exempt-prefixes = Protezione di protocollo: prefissi di route esenti
admin-settings-help-security-protocol-exempt-prefixes = Prefissi di route (ciascuno che inizia con / e più lungo di un carattere) per i quali le classi euristiche non vengono eseguite, per un client legittimo che ne ha bisogno, ad esempio una route per caricamenti grandi. Corrisponde a segmenti di percorso interi. I rilievi di framing e di controllo del percorso non sono mai esentati, e / viene ignorato.
admin-settings-key-security-health-mode = Accesso all'endpoint di integrità
admin-settings-help-security-health-mode = Chi può raggiungere /healthz e /readyz. internal-only: indirizzi client di loopback, privati, link-local e unique-local. allowlist: quelli più le reti sotto. open: chiunque. Chiunque altro riceve lo stesso 404 di un percorso sconosciuto. Una richiesta con il token MW_HEALTH_TOKEN è sempre ammessa. L'indirizzo del client è quello risolto dal proxy attendibile: dietro un reverse proxy senza MW_FORWARDED_MODE e MW_TRUSTED_PROXIES ogni richiesta sembra provenire dall'indirizzo privato del proxy, quindi l'intera internet viene ammessa.
admin-settings-key-security-health-networks = Reti dell'endpoint di integrità
admin-settings-help-security-health-networks = Indirizzi IP o reti CIDR che possono raggiungere /healthz e /readyz in modalità allowlist, oltre agli intervalli interni, ad esempio un monitor di uptime esterno.
admin-settings-key-security-spray-enabled = Rilevamento di password spraying
admin-settings-help-security-spray-enabled = Conta i nomi utente distinti con cui un'origine fallisce. Di ciascun nome utente viene conservato solo un hash con chiave, in memoria.
admin-settings-key-security-spray-window-secs = Finestra di spray (secondi)
admin-settings-help-security-spray-window-secs = Finestra in cui gli accessi falliti di un'origine vengono contati per nome utente distinto. Predefinito 900.
admin-settings-key-security-spray-challenge-at = Spray: soglia della sfida
admin-settings-help-security-spray-challenge-at = Nomi utente distinti falliti da una stessa origine, entro la finestra, che impongono la sfida di proof-of-work a quell'origine. Gli accessi riusciti non contano mai, quindi un indirizzo condiviso di ufficio o campus non è interessato. Predefinito 10.
admin-settings-key-security-spray-pace-at = Spray: soglia di rallentamento
admin-settings-help-security-spray-pace-at = Nomi utente distinti falliti che aggiungono un'attesa extra tra i tentativi dell'origine. Predefinito 15.
admin-settings-key-security-spray-lock-at = Spray: soglia di blocco
admin-settings-help-security-spray-lock-at = Nomi utente distinti falliti che bloccano l'origine per la durata del blocco, tramite il blocco di accesso esistente. Predefinito 25.
admin-settings-key-security-spray-pace-secs = Spray: attesa di rallentamento (secondi)
admin-settings-help-security-spray-pace-secs = Secondi tra i tentativi di un'origine pari o superiore alla soglia di rallentamento. Predefinito 10.
admin-settings-key-security-spray-lock-secs = Spray: durata del blocco (secondi)
admin-settings-help-security-spray-lock-secs = Durata del blocco temporaneo. Un amministratore che sblocca l'origine ne azzera anche il conteggio. Predefinito 900.
admin-settings-key-security-spray-known-multiplier = Spray: moltiplicatore per origine nota
admin-settings-help-security-spray-known-multiplier = Moltiplicatore delle tre soglie per un'origine nota, con un accesso riuscito recente. Predefinito 3.
admin-settings-key-security-spray-capacity = Spray: origini monitorate
admin-settings-help-security-spray-capacity = Numero massimo di origini monitorate in memoria; le meno recentemente attive vengono scartate per prime. Predefinito 20000.
admin-settings-key-security-spike-enabled = Rilevamento dei picchi sull'intera istanza
admin-settings-help-security-spike-enabled = Apprende una linea di base degli accessi falliti ed entra in modalità rafforzata quando l'intera istanza è molto sopra di essa. Non blocca mai l'istanza.
admin-settings-key-security-spike-multiplier-percent = Picco: moltiplicatore (%)
admin-settings-help-security-spike-multiplier-percent = Un segnale conta quando è almeno questa percentuale della sua linea di base appresa (400 = quattro volte) e sopra il suo minimo assoluto.
admin-settings-key-security-spike-min-failures-per-min = Picco: minimo di fallimenti al minuto
admin-settings-help-security-spike-min-failures-per-min = Minimo: accessi falliti al minuto sotto i quali il segnale del tasso di fallimento non conta mai. Predefinito 20.
admin-settings-key-security-spike-min-sources = Picco: minimo di origini che falliscono
admin-settings-help-security-spike-min-sources = Minimo: origini distinte che falliscono in un intervallo di cinque minuti sotto cui quel segnale non conta mai. Predefinito 10.
admin-settings-key-security-spike-min-usernames = Picco: minimo di nomi utente che falliscono
admin-settings-help-security-spike-min-usernames = Minimo: nomi utente distinti che falliscono in un intervallo di cinque minuti sotto cui quel segnale non conta mai. Predefinito 30.
admin-settings-key-security-spike-signals-required = Picco: segnali richiesti
admin-settings-help-security-spike-signals-required = Segnali che devono essere insieme sopra la linea di base e il minimo. La firma dello spray distribuito conta come due. Predefinito 2.
admin-settings-key-security-spike-warmup-hours = Picco: riscaldamento (ore)
admin-settings-help-security-spike-warmup-hours = Ore di traffico osservato prima che la modalità rafforzata possa iniziare. Fino ad allora la linea di base si limita ad apprendere. Predefinito 48.
admin-settings-key-security-spike-adapt-hours = Picco: adattamento della linea di base (ore)
admin-settings-help-security-spike-adapt-hours = Costante di tempo della linea di base. Più lunga assorbe la crescita più lentamente. Un picco non aggiorna mai la linea di base. Predefinito 168.
admin-settings-key-security-spike-duration-secs = Picco: durata della modalità rafforzata (secondi)
admin-settings-help-security-spike-duration-secs = Quanto dura la modalità rafforzata dopo l'ultima prova di un picco; viene prolungata finché i segnali restano sopra metà del loro livello di attivazione. Predefinito 1800.
admin-settings-key-security-spike-force-challenge = Picco: forza la sfida
admin-settings-help-security-spike-force-challenge = In modalità rafforzata, richiede la sfida di proof-of-work per gli accessi anonimi da origini sconosciute. Gli utenti autenticati e le origini note non sono interessati.
admin-settings-key-security-spike-pace-factor-percent = Picco: attesa di accesso (%)
admin-settings-help-security-spike-pace-factor-percent = In modalità rafforzata, l'attesa di accesso per le origini sconosciute come percentuale dell'attesa abituale. Predefinito 300.
admin-settings-key-security-session-binding-enabled = Vincolo della sessione
admin-settings-help-security-session-binding-enabled = Vincola ogni sessione del browser a hash con chiave degli attributi del suo client. I client nativi con token bearer, le chiavi API e i client di protocollo sono esenti.
admin-settings-key-security-session-binding-attributes = Vincolo della sessione: attributi
admin-settings-help-security-session-binding-attributes = Attributi vincolati: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Network è disattivato per impostazione predefinita perché le reti mobili cambiano. I nomi sconosciuti vengono ignorati.
admin-settings-key-security-session-binding-strictness = Vincolo della sessione: rigore
admin-settings-help-security-session-binding-strictness = Tollerante accetta gli aggiornamenti del browser e la modifica di un singolo attributo debole; rigoroso segnala qualsiasi differenza debole.
admin-settings-key-security-session-binding-webmail-action = Vincolo della sessione: azione nel webmail
admin-settings-help-security-session-binding-webmail-action = Cosa fa una discrepanza nella sessione del webmail: off, detect (solo registrare), step-up (richiedere di nuovo password e secondo fattore) o revocare la sessione. Predefinito step-up.
admin-settings-key-security-session-binding-admin-action = Vincolo della sessione: azione nella console di amministrazione
admin-settings-help-security-session-binding-admin-action = Cosa fa una discrepanza nella sessione della console di amministrazione: off, detect, step-up o revocare. Predefinito revocare.
admin-settings-key-security-theft-enabled = Rilevamento del furto di sessione
admin-settings-help-security-theft-enabled = Rileva il riutilizzo di un id di sessione ruotato, i viaggi impossibili e l'uso simultaneo da paesi diversi.
admin-settings-key-security-theft-rotation-grace-secs = Furto: tolleranza della rotazione (secondi)
admin-settings-help-security-theft-rotation-grace-secs = Dopo la rotazione di una sessione, il vecchio id può essere presentato di nuovo per questo tempo (una richiesta in corso) prima che conti come furto. Predefinito 30.
admin-settings-key-security-theft-min-travel-secs = Furto: tempo minimo di viaggio (secondi)
admin-settings-help-security-theft-min-travel-secs = Un cambio di paese più rapido di questo dopo la richiesta precedente viene trattato come viaggio impossibile. Il rilevamento del paese può essere impreciso; VPN e reti mobili possono causare falsi cambi. Predefinito 3600.
admin-settings-key-security-theft-concurrent-use = Furto: uso simultaneo
admin-settings-help-security-theft-concurrent-use = Segnala una sessione usata alternativamente da paesi (o reti) diversi entro cinque minuti: off, country o network. Il rilevamento del paese può essere impreciso; VPN e reti mobili possono causare falsi cambi.
admin-settings-key-security-theft-webmail-action = Furto: risposta nel webmail
admin-settings-help-security-theft-webmail-action = Risposta a un viaggio impossibile o all'uso simultaneo in una sessione del webmail: alert, step-up o revocare. Il riutilizzo di una sessione ruotata revoca sempre. Predefinito step-up.
admin-settings-key-security-theft-admin-action = Furto: risposta nella console di amministrazione
admin-settings-help-security-theft-admin-action = Risposta a un viaggio impossibile o all'uso simultaneo in una sessione della console di amministrazione: alert, step-up o revocare. Predefinito revocare.
admin-settings-key-security-theft-retention-days = Furto: conservazione dei record (giorni)
admin-settings-help-security-theft-retention-days = Per quanto tempo vengono conservati i marcatori delle sessioni ruotate e i record di sicurezza per sessione; impostarlo almeno pari alla durata massima di una sessione. Predefinito 15.
admin-settings-key-security-alerts-country-change = Avviso di cambio di paese
admin-settings-help-security-alerts-country-change = Avvisa quando una sessione autenticata viene usata da un paese diverso da quello in cui è iniziata. Il rilevamento del paese può essere impreciso; VPN e reti mobili possono causare falsi cambi. Gli utenti possono modificarlo a meno che non sia bloccato; la disattivazione da parte di un utente viene rispettata solo se il silenziamento è consentito.
admin-settings-key-security-alerts-country-change-enabled = Avviso di cambio di paese: interruttore globale
admin-settings-help-security-alerts-country-change-enabled = Disattivato, spegne l'avviso di cambio di paese per tutti. Il rilevamento del paese può essere impreciso; VPN e reti mobili possono causare falsi cambi.
admin-settings-key-security-alerts-country-change-cooldown-secs = Avviso di cambio di paese: intervallo minimo (secondi)
admin-settings-help-security-alerts-country-change-cooldown-secs = Tempo minimo tra gli avvisi di cambio di paese di una stessa sessione. Ogni nuovo paese avvisa una volta per sessione. Predefinito 300.
admin-settings-key-security-alerts-country-change-user-mute = Avviso di cambio di paese: consenti agli utenti di silenziarlo
admin-settings-help-security-alerts-country-change-user-mute = Se un utente può disattivare il proprio avviso di cambio di paese. Se disattivato, il server continua ad avvisare anche se l'impostazione dell'utente è disattivata.
admin-settings-key-security-alerts-channels = Canali degli avvisi di sicurezza
admin-settings-help-security-alerts-channels = Come gli avvisi di sicurezza della sessione raggiungono l'utente: nell'app, come messaggio nella propria casella di posta o entrambi. Gli utenti possono modificarlo a meno che non sia bloccato.
admin-settings-key-security-alerts-email-enabled = Avvisi di sicurezza via e-mail
admin-settings-help-security-alerts-email-enabled = Consente di recapitare gli avvisi di sicurezza della sessione nella casella di posta dell'utente. Nessun messaggio viene inviato tramite SMTP; senza una sessione di recapito l'avviso resta nell'app e nel registro di audit.
admin-settings-key-security-alerts-sensitivity = Sensibilità degli avvisi di sicurezza
admin-settings-help-security-alerts-sensitivity = Bassa avvisa solo sui rilievi di posizione e non eleva né revoca mai; normale segue le azioni dell'amministratore; alta raddoppia l'intervallo di viaggio e controlla anche le reti simultanee. Gli utenti possono modificarlo a meno che non sia bloccato.
admin-settings-key-security-alerts-notice-retention-days = Avvisi di sicurezza: conservazione (giorni)
admin-settings-help-security-alerts-notice-retention-days = Per quanto tempo gli avvisi di sicurezza nell'app vengono conservati per account (al massimo 20 per account). Predefinito 30.
admin-settings-inline-effective = In vigore:
admin-settings-inline-reset = Annulla le modifiche
admin-settings-inline-domains = Valori per dominio
admin-attachments-tab-lists = Elenchi di consenso e blocco
admin-attachments-tab-limits = Limiti
admin-attachments-tab-delivery = Download e anteprima
admin-attachments-tab-warnings = Avvisi
admin-attachments-tab-test = Prova un file
admin-attachments-tab-denied = Non ha accesso a questa scheda.
admin-attachments-group-lists = Elenchi di consenso e blocco
admin-attachments-group-limits = Limite di dimensione
admin-attachments-group-delivery = Download e anteprima
admin-attachments-group-warnings = Avvisi
admin-attachments-add-extension = Estensione, ad esempio pdf o .tar.gz
admin-attachments-add-mime = Tipo MIME, ad esempio image/png o image/*
admin-attachments-add = Aggiungi
admin-attachments-remove = Rimuovi { $rule }
admin-attachments-filter = Filtra l'elenco
admin-attachments-list-label = Regole in questo elenco
admin-attachments-count = { $count } di { $max } regole
admin-attachments-empty-list = Nessuna regola in questo elenco.
admin-attachments-none-match = Nessuna regola corrisponde al filtro.
admin-attachments-bulk = Incolla più voci
admin-attachments-bulk-add = Aggiungi tutte
admin-attachments-import = Importa un file di testo
admin-attachments-export = Esporta come testo
admin-attachments-err-extension = Non è un'estensione. Usi lettere, cifre, trattini e trattini bassi, fino a 32 caratteri; i punti possono separare le parti, come in tar.gz.
admin-attachments-err-mime = Non è un tipo MIME. Usi tipo/sottotipo, oppure tipo/* per tutti i sottotipi.
admin-attachments-err-long = Questa regola è troppo lunga.
admin-attachments-invalid = Non aggiunta, non è una regola valida: { $entries }
admin-attachments-duplicates = Già presente nell'elenco: { $rules }
admin-attachments-overflow = Un elenco contiene al massimo { $max } regole. Le altre non sono state aggiunte.
admin-attachments-conflict-blockedOverAllowed = { $rule } si sovrappone a { $other } nell'altro elenco. Le regole di blocco prevalgono, quindi viene bloccata.
admin-attachments-conflict-blockedOverWarn = { $rule } si sovrappone a { $other } nell'altro elenco. Un file bloccato viene rifiutato prima di qualsiasi avviso, quindi l'avviso non compare mai.
admin-attachments-conflict-allowedAndWarn = { $rule } si sovrappone a { $other } nell'altro elenco. Il file è consentito e mostra un avviso.
admin-attachments-bytes-unit = Unità
admin-attachments-unlimited = Nessun limite di dimensione
admin-attachments-bytes-equals = { $size } ({ $bytes } byte)
admin-attachments-list-allowedExtensions = estensioni consentite
admin-attachments-list-allowedMimeTypes = tipi MIME consentiti
admin-attachments-list-blockedExtensions = estensioni bloccate
admin-attachments-list-blockedMimeTypes = tipi MIME bloccati

# -- Machine translation (2026-11): added to match en ---------------------
admin-attachments-list-warnExtensions = estensioni con avviso
admin-attachments-list-warnMimeTypes = tipi MIME con avviso
admin-attachments-sim-title = Prova un file
admin-attachments-sim-help = Chiede al server come il criterio salvato tratta un file con questo nome, tipo e dimensione. Nulla viene modificato o memorizzato. I messaggi e gli allegati non vengono mai letti.
admin-attachments-sim-filename = Nome del file
admin-attachments-sim-mime = Tipo MIME
admin-attachments-sim-size = Dimensione
admin-attachments-sim-run = Prova
admin-attachments-sim-invalid = Inserisca un nome di file e una dimensione pari o superiore a zero.
admin-attachments-sim-allowed = Consentito
admin-attachments-sim-warn = Consentito con avviso
admin-attachments-sim-blocked = Non recapitato
admin-attachments-sim-preview-yes = L'anteprima è consentita.
admin-attachments-sim-preview-no = L'anteprima non è consentita.
admin-attachments-sim-checked = Dimensione verificata: { $size }.
admin-attachments-sim-matched = Regole corrispondenti in: { $lists }.
admin-attachments-sim-matched-none = Nessuna regola di consenso, blocco o avviso corrisponde.
admin-attachments-reason-downloadsDisabled = I download sono disattivati.
admin-attachments-reason-attachmentTooLarge = Il file supera il limite di dimensione.
admin-attachments-reason-attachmentBlocked = Corrisponde una regola di blocco.
admin-attachments-reason-attachmentNotAllowed = Esiste un elenco di consenso e nulla al suo interno corrisponde.
admin-attachments-reason-attachmentContainerRestricted = I messaggi allegati e i file TNEF vengono rifiutati finché è attiva una qualsiasi restrizione, perché il loro contenuto non è elencato.
admin-settings-key-mail-reply-images = Immagini nelle risposte e negli inoltri
admin-settings-help-mail-reply-images = Se una risposta o un inoltro include le immagini del messaggio originale: chiedere (predefinito), consentire sempre, consentire solo per i domini mittente attendibili e chiedere negli altri casi, oppure negare sempre. Bloccare questa impostazione per imporla. Le immagini vengono lette solo quando incluse.
admin-settings-key-mail-reply-image-trusted-domains = Domini mittente attendibili per le immagini
admin-settings-help-mail-reply-image-trusted-domains = Fino a 200 domini mittente esatti le cui immagini vengono incluse senza chiedere quando il criterio è domini attendibili. Gli utenti aggiungono alla propria lista dalla richiesta nella citazione, a meno che questa impostazione sia bloccata.
admin-settings-key-mail-send-recall-delay-seconds = Finestra di annullamento dell'invio (predefinita)
admin-settings-help-mail-send-recall-delay-seconds = Quanti secondi un messaggio inviato attende nella Posta in uscita, dove il mittente può richiamarlo, prima di essere recapitato. 0 invia subito. Predefinito 10, fino a 1800 (30 minuti). Gli utenti scelgono un proprio valore fino al limite sotto, a meno che non lo blocchi.
admin-settings-key-mail-send-recall-delay-max-seconds = Finestra di annullamento dell'invio (limite)
admin-settings-help-mail-send-recall-delay-max-seconds = La finestra di annullamento dell'invio più lunga che un utente può scegliere, in secondi. Il server riduce una richiesta più lunga a questo valore. Predefinito e massimo 1800.
admin-settings-key-mail-send-allow-recall = Consenti di richiamare i messaggi inviati
admin-settings-help-mail-send-allow-recall = Se disattivato, i messaggi partono subito e non possono essere richiamati dalla Posta in uscita. Gli invii programmati non sono interessati; possono ancora essere annullati fino all'ora stabilita.
admin-settings-key-mail-send-max-delayed-seconds = Orizzonte di programmazione
admin-settings-help-mail-send-max-delayed-seconds = Con quanto anticipo un messaggio può essere programmato, in secondi. Predefinito 7776000 (90 giorni), massimo 31536000 (1 anno). 0 disattiva l'invio programmato.
admin-settings-key-mail-send-max-queued = Messaggi in coda per casella di posta
admin-settings-help-mail-send-max-queued = Quanti messaggi una casella di posta può avere in attesa nella Posta in uscita (finestra di annullamento o programmati) contemporaneamente. Predefinito 200. Gli invii ulteriori vengono rifiutati con un messaggio chiaro e mai scartati.
admin-attachments-sim-pick = Compila da un file (facoltativo)
admin-attachments-mime-toggle = Mostra i tipi MIME comuni
admin-attachments-mime-hint = Scelga un tipo comune oppure digiti un altro tipo/sottotipo.
admin-attachments-mime-invalid = Questo non è un tipo/sottotipo come text/plain. Verrà inviato come digitato.
admin-attachments-mime-group-documents = Documenti
admin-attachments-mime-group-images = Immagini
admin-attachments-mime-group-archives = Archivi
admin-attachments-mime-group-executables = Eseguibili e script
admin-attachments-mime-group-media = Audio e video
admin-attachments-mime-group-email = E-mail e calendario
admin-attachments-mime-group-other = Altro
admin-settings-key-cookies-enabled = Controllo del consenso ai cookie
admin-settings-help-cookies-enabled = Mostra ai visitatori il controllo del consenso ai cookie. Disattivato, l'applicazione si comporta come senza il controllo.
admin-settings-key-cookies-mode = Modello di consenso
admin-settings-help-cookies-mode = optIn: nulla di facoltativo viene memorizzato finché il visitatore non accetta. optOut: valgono i valori predefiniti delle categorie finché il visitatore non rifiuta. noticeOnly: il visitatore viene informato, senza scelta.
admin-settings-key-cookies-region-mode = Modello di consenso nelle regioni rigorose
admin-settings-help-cookies-region-mode = Il modello usato per i visitatori delle regioni rigorose. same usa anche lì il modello di consenso indicato sopra.
admin-settings-key-cookies-strict-regions = Regioni rigorose
admin-settings-help-cookies-strict-regions = Codici paese ISO 3166-1 che usano il modello di consenso delle regioni rigorose. Per impostazione predefinita gli Stati UE e SEE, il Regno Unito e la Svizzera.
admin-settings-key-cookies-region-header = Intestazione del paese
admin-settings-help-cookies-region-header = Nome dell'intestazione della richiesta, impostata da un proxy o da una CDN, che contiene il codice paese del visitatore.
admin-settings-key-cookies-unknown-region = Paese sconosciuto
admin-settings-help-cookies-unknown-region = Quale modello si applica quando non è possibile determinare alcun paese: quello delle regioni rigorose o quello normale.
admin-settings-key-cookies-functional-enabled = Offri cookie funzionali
admin-settings-help-cookies-functional-enabled = Offre la categoria preferenze e funzionalità (tema, lingua, memoria del layout) nel controllo del consenso.
admin-settings-key-cookies-functional-default = Predefinito per i funzionali
admin-settings-help-cookies-functional-default = Stato iniziale della categoria funzionale quando il modello di consenso consente valori predefiniti.
admin-settings-key-cookies-analytics-enabled = Offri cookie di analisi
admin-settings-help-cookies-analytics-enabled = Offre la categoria analisi nel controllo del consenso.
admin-settings-key-cookies-analytics-default = Predefinito per l'analisi
admin-settings-help-cookies-analytics-default = Stato iniziale della categoria analisi quando il modello di consenso consente valori predefiniti.
admin-settings-key-cookies-marketing-enabled = Offri cookie di marketing
admin-settings-help-cookies-marketing-enabled = Offre la categoria marketing nel controllo del consenso.
admin-settings-key-cookies-marketing-default = Predefinito per il marketing
admin-settings-help-cookies-marketing-default = Stato iniziale della categoria marketing quando il modello di consenso consente valori predefiniti.
admin-settings-key-cookies-layout = Layout
admin-settings-help-cookies-layout = banner è una barra sull'intera pagina, card è una scheda flottante, modal è una finestra di dialogo che trattiene la pagina finché il visitatore non sceglie.
admin-settings-key-cookies-position = Posizione
admin-settings-help-cookies-position = Dove compare il controllo del consenso nella pagina.
admin-settings-key-cookies-reject-prominent = Rifiuta in evidenza quanto accetta
admin-settings-help-cookies-reject-prominent = Mostra Rifiuta tutto con le stesse dimensioni e lo stesso stile di Accetta tutto.
admin-settings-key-cookies-policy-url = URL dell'informativa sui cookie
admin-settings-help-cookies-policy-url = Link alla Sua informativa sui cookie, mostrato nel controllo del consenso.
admin-settings-key-cookies-privacy-url = URL dell'informativa sulla privacy
admin-settings-help-cookies-privacy-url = Link alla Sua informativa sulla privacy, mostrato nel controllo del consenso.
admin-settings-key-cookies-policy-version = Versione dell'informativa
admin-settings-help-cookies-policy-version = Aumenti questo numero quando cambia l'uso dei cookie, così che ai visitatori venga chiesto di nuovo.
admin-settings-key-cookies-lifetime-days = Validità del consenso
admin-settings-help-cookies-lifetime-days = Giorni in cui una decisione resta valida prima che venga chiesto di nuovo al visitatore.
admin-settings-key-cookies-reprompt-on-version = Chiedi di nuovo a una nuova versione dell'informativa
admin-settings-help-cookies-reprompt-on-version = Chiede di nuovo ai visitatori quando la versione dell'informativa è superiore a quella su cui hanno deciso.
admin-settings-key-cookies-honour-gpc = Rispetta il Global Privacy Control
admin-settings-help-cookies-honour-gpc = Tratta un segnale Global Privacy Control del browser come rifiuto di analisi e marketing.
admin-settings-key-cookies-honour-dnt = Rispetta il Do Not Track
admin-settings-help-cookies-honour-dnt = Tratta un segnale Do Not Track del browser come rifiuto di analisi e marketing.
admin-settings-key-cookies-log-consent = Conserva un registro dei consensi
admin-settings-help-cookies-log-consent = Conserva sul server una registrazione di ogni decisione di consenso, per le statistiche e per i Suoi archivi.
admin-settings-key-cookies-log-retention-days = Conservazione del registro dei consensi
admin-settings-help-cookies-log-retention-days = Giorni per cui le righe del registro dei consensi vengono conservate prima di essere eliminate.
admin-settings-key-cookies-scan-interval-hours = Intervallo di scansione dell'archiviazione
admin-settings-help-cookies-scan-interval-hours = Ore tra le scansioni automatiche dei cookie e dell'archiviazione del browser. 0 esegue la scansione solo su richiesta.
admin-settings-key-cookies-unclassified = Archiviazione non classificata
admin-settings-help-cookies-unclassified = report lascia da parte l'archiviazione che nulla classifica e la elenca per la revisione. Gli altri valori la trattano come quella categoria finché non la classifica.
admin-settings-key-cookies-text-title = Titolo del controllo del consenso
admin-settings-help-cookies-text-title = Titolo del controllo del consenso per lingua. Se vuoto, usa il testo integrato.
admin-settings-key-cookies-text-body = Testo del controllo del consenso
admin-settings-help-cookies-text-body = Testo esplicativo del controllo del consenso per lingua. Se vuoto, usa il testo integrato.
admin-settings-key-cookies-label-necessary = Nome della categoria necessari
admin-settings-help-cookies-label-necessary = Nome della categoria necessari per lingua. Se vuoto, usa il nome integrato.
admin-settings-key-cookies-description-necessary = Descrizione della categoria necessari
admin-settings-help-cookies-description-necessary = Descrizione della categoria necessari per lingua. Se vuota, usa la descrizione integrata.
admin-settings-key-cookies-label-functional = Nome della categoria funzionali
admin-settings-help-cookies-label-functional = Nome della categoria funzionali per lingua. Se vuoto, usa il nome integrato.
admin-settings-key-cookies-description-functional = Descrizione della categoria funzionali
admin-settings-help-cookies-description-functional = Descrizione della categoria funzionali per lingua. Se vuota, usa la descrizione integrata.
admin-settings-key-cookies-label-analytics = Nome della categoria analisi
admin-settings-help-cookies-label-analytics = Nome della categoria analisi per lingua. Se vuoto, usa il nome integrato.
admin-settings-key-cookies-description-analytics = Descrizione della categoria analisi
admin-settings-help-cookies-description-analytics = Descrizione della categoria analisi per lingua. Se vuota, usa la descrizione integrata.
admin-settings-key-cookies-label-marketing = Nome della categoria marketing
admin-settings-help-cookies-label-marketing = Nome della categoria marketing per lingua. Se vuoto, usa il nome integrato.
admin-settings-key-cookies-description-marketing = Descrizione della categoria marketing
admin-settings-help-cookies-description-marketing = Descrizione della categoria marketing per lingua. Se vuota, usa la descrizione integrata.
admin-settings-key-encryption-rotation-days = Intervallo di rotazione della chiave del database (giorni)
admin-settings-help-encryption-rotation-days = Giorni dopo l'ultimo cambio di chiave trascorsi i quali la chiave del database viene segnalata come da ruotare. 0 non imposta alcuna scadenza. La chiave stessa non viene mai memorizzata nelle impostazioni.
admin-settings-key-encryption-warnings-enabled = Segnala i rilievi sulla cifratura
admin-settings-help-encryption-warnings-enabled = Mostra gli archivi in chiaro e una rotazione della chiave scaduta come rilievo nella valutazione della distribuzione.
admin-settings-key-retention-sweeper-enabled = Pulizia della conservazione
admin-settings-help-retention-sweeper-enabled = Esegue la pulizia giornaliera che elimina le righe più vecchie dei limiti di questa pagina. Disattivata, conserva tutte le righe finché non viene riattivata.
admin-settings-key-retention-sweeper-batch-rows = Righe per lotto
admin-settings-help-retention-sweeper-batch-rows = Righe che una singola istruzione di eliminazione rimuove. Lotti più piccoli mantengono i blocchi del database per meno tempo.
admin-settings-key-retention-sweeper-max-batches = Lotti per archivio per esecuzione
admin-settings-help-retention-sweeper-max-batches = Numero massimo di istruzioni di eliminazione che un archivio riceve in un'esecuzione. Un archivio con più righe vecchie prosegue nell'esecuzione successiva.
admin-settings-key-retention-sweeper-jitter-minutes = Dispersione dell'orario di avvio (minuti)
admin-settings-help-retention-sweeper-jitter-minutes = L'esecuzione successiva parte un numero casuale di minuti, fino a questo valore, dopo 24 ore, così più server non eseguono la pulizia insieme.
admin-settings-key-retention-sweeper-vacuum-after-rows = Recupera spazio dopo (righe)
admin-settings-help-retention-sweeper-vacuum-after-rows = Quando un'esecuzione elimina almeno questo numero di righe, il database viene compattato (VACUUM). 0 non compatta mai dopo una pulizia; l'operazione settimanale di vacuum è separata.
admin-settings-key-retention-password-change-audit-days = Registrazioni dei cambi di password (giorni)
admin-settings-help-retention-password-change-audit-days = Giorni per cui viene conservata la registrazione di un cambio di password (account, metodo, esito). 0 le conserva per sempre.
admin-settings-key-retention-sso-login-audit-days = Registrazioni degli accessi con single sign-on (giorni)
admin-settings-help-retention-sso-login-audit-days = Giorni per cui viene conservata la registrazione di un accesso con single sign-on (provider, soggetto con hash, esito). 0 le conserva per sempre.
admin-settings-key-retention-dlp-audit-days = Registrazioni di prevenzione della perdita di dati (giorni)
admin-settings-help-retention-dlp-audit-days = Giorni per cui viene conservata la registrazione di una corrispondenza con una regola di prevenzione della perdita di dati. 0 le conserva per sempre.
admin-settings-key-retention-assist-audit-days = Registrazioni di accesso dell'assistente (giorni)

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-retention-assist-audit-days = Giorni per cui viene conservata la registrazione dell'uso di una funzione da parte di un assistente (attore, funzione, riepilogo dell'ambito). 0 le conserva per sempre.
admin-settings-key-retention-alert-events-days = Registrazioni degli avvisi generati (giorni)
admin-settings-help-retention-alert-events-days = Giorni per cui viene conservata la registrazione di un avviso amministratore generato. 0 le conserva per sempre.
admin-settings-key-retention-job-runs-days = Cronologia dei processi in background (giorni)
admin-settings-help-retention-job-runs-days = Giorni per cui viene conservata l'esecuzione conclusa di un processo in background. Un'esecuzione ancora in corso non viene mai rimossa. 0 conserva la cronologia per sempre.
admin-settings-key-retention-outbox-days = Registrazioni degli invii completati e annullati (giorni)
admin-settings-help-retention-outbox-days = Giorni per cui viene conservata la registrazione di invio di un messaggio recapitato o annullato. Un messaggio in attesa, in nuovo tentativo, fallito o incerto non viene mai rimosso. 0 le conserva per sempre.
admin-settings-key-retention-expired-credentials-grace-days = Credenziali scadute (giorni dopo la scadenza)
admin-settings-help-retention-expired-credentials-grace-days = Giorni per cui un dispositivo ricordato, una sfida di accesso, un token di accesso, un token OAuth, un nonce di webhook, un contatore di errori di origine o un handle di link immagine scaduti vengono conservati dopo la scadenza. 0 lo rimuove alla prossima esecuzione.
admin-settings-key-retention-ip-addresses-days = Riduci gli indirizzi IP memorizzati dopo (giorni)
admin-settings-help-retention-ip-addresses-days = Dopo questo numero di giorni l'indirizzo IP nella cronologia di autenticazione e nelle voci di audit viene troncato o sottoposto a hash e la riga viene conservata. 0 mantiene gli indirizzi come registrati.
admin-settings-key-retention-ip-addresses-mode = Metodo di riduzione dell'IP
admin-settings-help-retention-ip-addresses-mode = Tronca mantiene il prefisso IPv4 /24 o IPv6 /48. Hash sostituisce l'indirizzo con un hash breve con chiave che raggruppa comunque lo stesso indirizzo.
admin-settings-key-retention-user-enabled = Gli utenti possono impostare regole di conservazione
admin-settings-help-retention-user-enabled = Gli utenti possono impostare proprie regole di conservazione per cartelle e posta. Se disattivato, le regole utente esistenti smettono di essere eseguite; le regole valide per tutta l'organizzazione qui sotto restano attive.
admin-settings-key-retention-user-min-days = Giorni minimi prima che una regola utente elimini la posta
admin-settings-help-retention-user-min-days = Una regola utente che elimina la posta non può agire sulla posta più recente di questo valore. Un valore inferiore viene rifiutato al salvataggio della regola e innalzato all'esecuzione.
admin-settings-key-retention-user-max-days = Giorni massimi che una regola utente può attendere per eliminare la posta
admin-settings-help-retention-user-max-days = Una regola utente che elimina la posta non può attendere più di questo valore. 0 non imposta alcun limite superiore. Un valore maggiore viene rifiutato al salvataggio della regola e abbassato all'esecuzione.
admin-settings-key-retention-user-max-rules = Massimo di regole di conservazione per utente
admin-settings-help-retention-user-max-rules = Il massimo di regole di conservazione che un utente può mantenere.
admin-settings-key-retention-user-allow-permanent-delete = Le regole utente possono eliminare la posta definitivamente
admin-settings-help-retention-user-allow-permanent-delete = Se disattivato, una regola di eliminazione sposta sempre la posta nel Cestino e la finestra di annullamento vale prima dell'eliminazione definitiva.
admin-settings-key-retention-user-undo-days = Finestra di annullamento delle regole di conservazione (giorni)
admin-settings-help-retention-user-undo-days = Giorni per cui la posta spostata nel Cestino da una regola di conservazione vi resta prima di essere eliminata definitivamente. Un utente può scegliere una finestra più breve. 0 elimina alla prossima esecuzione.
admin-settings-key-retention-user-mandatory = Regole di posta valide per tutta l'organizzazione
admin-settings-help-retention-user-mandatory = Regole che ogni account riceve, come voci ruolo-cartella:giorni, ad esempio trash:30 o junk:30. La posta in quella cartella più vecchia dei giorni indicati viene eliminata definitivamente. Un utente non può modificarle, sospenderle né escludere posta da esse. La posta di un account sotto conservazione legale viene saltata.
admin-settings-key-retention-user-protected-folders = Cartelle che le regole di conservazione non toccano mai
admin-settings-help-retention-user-protected-folders = Cartelle su cui nessuna regola utente opera, per ruolo (drafts, sent, inbox, archive) o per nome della cartella. Una cartella protetta prevale su una regola valida per tutta l'organizzazione per la stessa cartella.
admin-settings-key-retention-user-templates = Modelli di conservazione offerti agli utenti
admin-settings-help-retention-user-templates = Modelli che gli utenti possono adottare nelle Impostazioni, per id: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Esegui le regole utente per gli account con restrizioni
admin-settings-help-retention-user-run-for-restricted = Se le regole di conservazione utente vengono eseguite anche per gli account sotto una limitazione del trattamento. Disattivato, quegli account vengono saltati.
admin-settings-key-retention-user-interval-hours = Ore tra le esecuzioni di conservazione utente
admin-settings-help-retention-user-interval-hours = Ore tra le esecuzioni pianificate del processo di conservazione utente per un account.
admin-settings-key-retention-user-batch-messages = Messaggi per chiamata di conservazione
admin-settings-help-retention-user-batch-messages = Messaggi che una chiamata di spostamento o eliminazione gestisce. Ne vengono usati al massimo 200.
admin-settings-key-retention-user-max-per-run = Massimo di messaggi modificati per account per esecuzione
admin-settings-help-retention-user-max-per-run = Massimo di messaggi che il processo di conservazione utente modifica per un account in un'esecuzione. Il resto attende l'esecuzione successiva.
admin-settings-key-retention-user-pause-ms = Pausa tra i lotti di conservazione (ms)
admin-settings-help-retention-user-pause-ms = Pausa tra i lotti, in millisecondi, affinché una pulizia estesa non carichi il server di posta.
admin-settings-key-retention-class-short-days = Limite della classe breve (giorni)
admin-settings-help-retention-class-short-days = Limite per i dati operativi: sessioni, token, limiti di frequenza, cronologia dei processi e metriche. Si applica quando il limite di classe è attivo.
admin-settings-key-retention-class-medium-days = Limite della classe media (giorni)
admin-settings-help-retention-class-medium-days = Limite per la cronologia degli accessi, la cronologia della protezione delle richieste e gli avvisi di sicurezza. Non deve essere inferiore alla classe breve; un valore più basso viene innalzato a essa.
admin-settings-key-retention-class-long-days = Conservazione della classe lunga (giorni)
admin-settings-help-retention-class-long-days = Giorni per cui la classe lunga (audit di sicurezza, azioni degli amministratori, richieste, consenso, tombstone) viene conservata, archivio incluso. 0 la conserva per sempre e la console avvisa. Non deve essere inferiore alla classe media.
admin-settings-key-retention-class-enforce = Mantieni ogni archivio entro il limite della sua classe
admin-settings-help-retention-class-enforce = Se attivo, il limite proprio di ogni archivio viene ricondotto al limite della sua classe e 0 (conserva) conta come il limite. Se disattivo, valgono i limiti per archivio.
admin-settings-key-retention-archive-enabled = Archivia le righe datate della classe lunga
admin-settings-help-retention-archive-enabled = Sposta le righe datate della classe lunga in file di archivio sigillati e compressi invece di eliminarle.
admin-settings-key-retention-archive-directory = Directory dell'archivio
admin-settings-help-retention-archive-directory = Directory per i file di archivio, assoluta o relativa alla directory di lavoro del server. I segmenti di directory superiori (..) vengono rifiutati.
admin-settings-key-retention-archive-batch-rows = Righe per file di archivio
admin-settings-help-retention-archive-batch-rows = Righe scritte in un file di archivio.
admin-settings-key-retention-archive-max-segments = File di archivio per archivio per esecuzione
admin-settings-help-retention-archive-max-segments = File di archivio che un'esecuzione può scrivere per ogni archivio. Il resto attende l'esecuzione successiva.
admin-settings-key-retention-archive-key-generation = Generazione della chiave di sigillo dell'archivio
admin-settings-help-retention-archive-key-generation = Generazione della chiave che sigilla i nuovi file di archivio. Aumentarla per ruotare la chiave; i file precedenti vengono comunque verificati con la propria generazione.
admin-settings-key-retention-archive-verify-days = Giorni tra i controlli dell'archivio
admin-settings-help-retention-archive-verify-days = Giorni tra i controlli di integrità dei file di archivio.
admin-settings-key-retention-archive-legal-hold = Conservazione legale dell'archivio
admin-settings-help-retention-archive-legal-hold = Conserva ogni file di archivio, qualunque sia il limite della classe lunga.
admin-settings-key-logging-file-enabled = Scrivi file di log
admin-settings-help-logging-file-enabled = Scrive anche il log del server in file a rotazione. Disattivato, il log resta solo sullo standard output, come di norma in un container.
admin-settings-key-logging-file-directory = Directory dei log
admin-settings-help-logging-file-directory = Directory per i file di log, assoluta o relativa alla directory di lavoro del server. I segmenti di directory superiori (..) vengono rifiutati.
admin-settings-key-logging-file-max-size-mb = Ruota per dimensione (MB)
admin-settings-help-logging-file-max-size-mb = Il file di log corrente viene chiuso e ne viene avviato uno nuovo quando raggiunge questa dimensione.
admin-settings-key-logging-file-rotate-hours = Ruota per età (ore)
admin-settings-help-logging-file-rotate-hours = Il file di log corrente viene ruotato quando ha questa età. 0 ruota solo per dimensione.
admin-settings-key-logging-file-max-age-days = Conserva i file ruotati (giorni)
admin-settings-help-logging-file-max-age-days = I file di log ruotati più vecchi di questo valore vengono eliminati.
admin-settings-key-logging-file-max-files = Massimo di file ruotati
admin-settings-help-logging-file-max-files = I file ruotati più vecchi oltre questo numero vengono eliminati.
admin-settings-key-logging-file-max-total-mb = Dimensione totale massima (MB)
admin-settings-help-logging-file-max-total-mb = I file ruotati più vecchi vengono eliminati finché la directory dei log non scende sotto questa dimensione.
admin-settings-key-logging-file-compress = Comprimi i file ruotati
admin-settings-help-logging-file-compress = I file di log ruotati vengono compressi con gzip.
admin-settings-key-logging-level = Livello di log
admin-settings-help-logging-level = Il livello per i componenti che non hanno un livello proprio qui sotto. La variabile d'ambiente MW_LOG imposta il livello all'avvio finché questa impostazione non viene letta.
admin-settings-key-logging-component-levels = Livelli di log per componente
admin-settings-help-logging-component-levels = Una voce componente=livello per riga, ad esempio mw_imap=debug. I livelli sono error, warn, info, debug e trace.
admin-settings-key-logging-redaction-addresses = Indirizzi e-mail nei log
admin-settings-help-logging-redaction-addresses = Completo lascia gli indirizzi come registrati. Troncato mantiene la prima lettera e il dominio. Hash sostituisce la parte locale con un hash breve. Nessuno rimuove l'indirizzo.
admin-settings-key-logging-redaction-ips = Indirizzi IP nei log
admin-settings-help-logging-redaction-ips = Completo lascia gli indirizzi come registrati. Troncato mantiene il prefisso IPv4 /24 o IPv6 /48. Hash sostituisce l'indirizzo con un hash breve. Nessuno lo rimuove.
admin-settings-key-logging-redaction-tokens = Segreti nei log
admin-settings-help-logging-redaction-tokens = Rimuove dalle righe di log token bearer, chiavi API, password negli URL e segreti simili.
admin-settings-key-retention-sessions-idle-days = Sessioni inutilizzate (giorni)
admin-settings-help-retention-sessions-idle-days = Giorni per cui una registrazione di sessione del browser o di un client nativo, con il suo indirizzo e user agent, può restare inutilizzata prima di essere eliminata. I timeout di sessione valgono per primi; questo rimuove solo le righe rimaste. 0 le conserva.
admin-settings-key-policy-public-pages = Pagine pubbliche delle informative
admin-settings-help-policy-public-pages = Serve l'informativa sulla privacy generata su /privacy e l'informativa sui cookie su /cookies-policy senza accesso.
admin-settings-key-policy-auto-version = Versioni automatiche dell'informativa
admin-settings-help-policy-auto-version = Registra una nuova versione dell'informativa quando cambia un'impostazione che l'informativa descrive. Disattivato: le modifiche attendono la pubblicazione.
admin-settings-key-policy-reconsent-on-change = Richiedi di nuovo il consenso ai cookie a una nuova versione dell'informativa
admin-settings-help-policy-reconsent-on-change = Aumenta la versione dell'informativa sui cookie quando viene registrata una nuova versione dell'informativa, così ai visitatori viene chiesto di nuovo (richiede cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Nome del titolare del trattamento
admin-settings-help-policy-controller-name = Nome dell'organizzazione che decide perché e come i dati vengono trattati, mostrato nell'informativa sulla privacy.
admin-settings-key-policy-controller-address = Indirizzo del titolare del trattamento
admin-settings-help-policy-controller-address = Indirizzo postale del titolare del trattamento, mostrato nell'informativa sulla privacy.
admin-settings-key-policy-legal-bases = Basi giuridiche
admin-settings-help-policy-legal-bases = Le basi giuridiche su cui si fonda, con parole Sue, mostrate nell'informativa sulla privacy.
admin-settings-key-policy-hosting-provider = Provider di hosting
admin-settings-help-policy-hosting-provider = Chi ospita i dati e dove, mostrato nell'informativa sulla privacy.
admin-settings-key-policy-hosting-transfers = Trasferimenti fuori dal paese di hosting
admin-settings-help-policy-hosting-transfers = Eventuali trasferimenti di dati verso altri paesi e le garanzie utilizzate, mostrati nell'informativa sulla privacy.
admin-settings-key-policy-supervisory-authority = Autorità di controllo
admin-settings-help-policy-supervisory-authority = L'autorità a cui gli utenti possono presentare reclamo, mostrata nell'informativa sulla privacy.
admin-settings-key-dpia-suggest = Suggerisci valutazioni d'impatto
admin-settings-help-dpia-suggest = Elenca una valutazione d'impatto suggerita quando l'assistente, il tracciamento delle aperture o l'analisi è attivo e non ne esiste alcuna.
admin-settings-key-dpia-review-months = Intervallo di revisione della valutazione d'impatto
admin-settings-help-dpia-review-months = Mesi fino alla data di revisione impostata quando una valutazione viene approvata.
admin-settings-key-dpia-reminder-days = Finestra di promemoria della valutazione d'impatto
admin-settings-help-dpia-reminder-days = Giorni prima della data di revisione in cui una valutazione viene elencata come in scadenza.
admin-settings-key-restriction-enabled = Limitazioni del trattamento
admin-settings-help-restriction-enabled = Se un amministratore o una richiesta approvata può limitare il trattamento dei dati di una persona.
admin-settings-key-restriction-auto-apply-on-approval = Limita quando una richiesta viene approvata
admin-settings-help-restriction-auto-apply-on-approval = Applica la limitazione non appena una richiesta di limitazione viene approvata. Disattivato: un amministratore la applica da Conformità > Limitazioni.
admin-settings-key-restriction-mail-delivery = Nuova posta durante la limitazione

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-restriction-mail-delivery = Cosa ha scelto l'amministratore per la posta che arriva mentre si applica una limitazione: conservarla, accodarla o rifiutarla. Questo server non riceve posta; il valore viene registrato e mostrato, e il server di posta agisce di conseguenza.
admin-settings-key-restriction-default-expiry-days = Durata predefinita della limitazione (giorni)
admin-settings-help-restriction-default-expiry-days = Giorni dopo cui una nuova limitazione termina da sola. 0 la mantiene finché qualcuno non la revoca.
admin-settings-key-restriction-notify-user = Avvisa la persona
admin-settings-help-restriction-notify-user = Invia alla persona un avviso quando una limitazione viene applicata o revocata.
admin-settings-key-restriction-scope-indexing = Sospendi l'indicizzazione della ricerca e gli embedding
admin-settings-help-restriction-scope-indexing = Se l'indicizzazione della ricerca e gli embedding vengono sospesi per impostazione predefinita in una nuova limitazione.
admin-settings-key-restriction-scope-assistant = Sospendi l'assistente
admin-settings-help-restriction-scope-assistant = Se l'assistente viene sospeso per impostazione predefinita in una nuova limitazione.
admin-settings-key-restriction-scope-analytics = Sospendi analisi e tracciamento delle aperture
admin-settings-help-restriction-scope-analytics = Se analisi e tracciamento delle aperture vengono sospesi per impostazione predefinita in una nuova limitazione.
admin-settings-key-restriction-scope-classification = Sospendi regole e archiviazione automatica
admin-settings-help-restriction-scope-classification = Se regole e archiviazione automatica vengono sospese per impostazione predefinita in una nuova limitazione.
admin-settings-key-restriction-scope-push = Sospendi le notifiche push
admin-settings-help-restriction-scope-push = Se le notifiche push vengono sospese per impostazione predefinita in una nuova limitazione.
admin-settings-key-restriction-scope-third-party = Sospendi servizi collegati e plugin
admin-settings-help-restriction-scope-third-party = Se servizi collegati e plugin vengono sospesi per impostazione predefinita in una nuova limitazione.
admin-settings-key-consent-enabled = Consenso per finalità
admin-settings-help-consent-enabled = Mostra agli utenti le finalità e le scelte di consenso e le fa rispettare a runtime.
admin-settings-key-consent-receipts-retention-days = Conservazione delle ricevute di consenso (giorni)
admin-settings-help-consent-receipts-retention-days = Giorni per cui le ricevute di consenso vengono conservate. 0 le conserva finché la persona non viene cancellata.
admin-settings-key-consent-art22-notice = Informativa sul trattamento automatizzato
admin-settings-help-consent-art22-notice = Mostra agli utenti l'informativa in linguaggio semplice sulle funzioni dell'assistente e della ricerca, con gli interruttori per funzione.
admin-settings-key-consent-analytics-basis = Base giuridica: conteggi di utilizzo
admin-settings-help-consent-analytics-basis = La base giuridica mostrata agli utenti per i conteggi di utilizzo. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-analytics-version = Versione del testo: conteggi di utilizzo
admin-settings-help-consent-analytics-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-analytics-text = Formulazione propria: conteggi di utilizzo
admin-settings-help-consent-analytics-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-analytics-tag-basis = Base giuridica: script del provider di analisi
admin-settings-help-consent-analytics-tag-basis = La base giuridica mostrata agli utenti per lo script del provider di analisi. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-analytics-tag-version = Versione del testo: script del provider di analisi
admin-settings-help-consent-analytics-tag-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-analytics-tag-text = Formulazione propria: script del provider di analisi
admin-settings-help-consent-analytics-tag-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-assistant-basis = Base giuridica: assistente
admin-settings-help-consent-assistant-basis = La base giuridica mostrata agli utenti per l'assistente. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-assistant-version = Versione del testo: assistente
admin-settings-help-consent-assistant-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-assistant-text = Formulazione propria: assistente
admin-settings-help-consent-assistant-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-push-basis = Base giuridica: notifiche push
admin-settings-help-consent-push-basis = La base giuridica mostrata agli utenti per le notifiche push. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-push-version = Versione del testo: notifiche push
admin-settings-help-consent-push-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-push-text = Formulazione propria: notifiche push
admin-settings-help-consent-push-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-open-tracking-basis = Base giuridica: tracciamento delle aperture
admin-settings-help-consent-open-tracking-basis = La base giuridica mostrata agli utenti per il tracciamento delle aperture. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-open-tracking-version = Versione del testo: tracciamento delle aperture
admin-settings-help-consent-open-tracking-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-open-tracking-text = Formulazione propria: tracciamento delle aperture
admin-settings-help-consent-open-tracking-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-remote-content-basis = Base giuridica: immagini remote
admin-settings-help-consent-remote-content-basis = La base giuridica mostrata agli utenti per le immagini remote. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-remote-content-version = Versione del testo: immagini remote
admin-settings-help-consent-remote-content-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-remote-content-text = Formulazione propria: immagini remote
admin-settings-help-consent-remote-content-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-integrations-basis = Base giuridica: servizi collegati
admin-settings-help-consent-integrations-basis = La base giuridica mostrata agli utenti per i servizi collegati. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-integrations-version = Versione del testo: servizi collegati
admin-settings-help-consent-integrations-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-integrations-text = Formulazione propria: servizi collegati
admin-settings-help-consent-integrations-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-consent-telemetry-basis = Base giuridica: misurazioni dei tempi di caricamento
admin-settings-help-consent-telemetry-basis = La base giuridica mostrata agli utenti per le misurazioni dei tempi di caricamento. Solo il consenso fa attendere alla funzione un sì; le altre basi valgono a meno che la persona non revochi. L'amministratore decide quale base sia corretta.
admin-settings-key-consent-telemetry-version = Versione del testo: misurazioni dei tempi di caricamento
admin-settings-help-consent-telemetry-version = Aumentare questo numero quando la formulazione o il trattamento cambia; a chi ha accettato una versione precedente viene chiesto di nuovo.
admin-settings-key-consent-telemetry-text = Formulazione propria: misurazioni dei tempi di caricamento
admin-settings-help-consent-telemetry-text = Il Suo testo per questa finalità, per lingua. Se vuoto, usa la formulazione integrata.
admin-settings-key-breach-enabled = Registro delle violazioni
admin-settings-help-breach-enabled = Attiva il registro e l'orologio di notifica.
admin-settings-key-breach-notice-hours = Ore per notificare l'autorità
admin-settings-help-breach-notice-hours = Ore dalla presa di conoscenza alla notifica all'autorità. Il termine abituale è 72.
admin-settings-key-breach-reminders-hours = Orari dei promemoria
admin-settings-help-breach-reminders-hours = Ore prima della scadenza che generano un promemoria, ad esempio 6, 24 e 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Ripetizione dopo la scadenza
admin-settings-help-breach-reminders-overdue-repeat-hours = Ore tra i promemoria una volta superata la scadenza. 0 ricorda una sola volta.
admin-settings-key-breach-authority-name = Autorità di controllo
admin-settings-help-breach-authority-name = Nome stampato sulla notifica all'autorità.
admin-settings-key-breach-authority-url = Pagina di notifica dell'autorità
admin-settings-help-breach-authority-url = Indirizzo della pagina di notifica dell'autorità.
admin-settings-key-breach-authority-contact = Contatto dell'autorità
admin-settings-help-breach-authority-contact = Dati di contatto dell'autorità.
admin-settings-key-breach-dpo-name = Nome del referente per la protezione dei dati
admin-settings-help-breach-dpo-name = Nome indicato come punto di contatto nelle notifiche.
admin-settings-key-breach-dpo-email = E-mail del referente per la protezione dei dati
admin-settings-help-breach-dpo-email = Indirizzo indicato come punto di contatto nelle notifiche.
admin-settings-key-breach-dpo-phone = Telefono del referente per la protezione dei dati
admin-settings-help-breach-dpo-phone = Numero di telefono indicato nelle notifiche.
admin-settings-key-breach-template-subject = Oggetto del messaggio
admin-settings-help-breach-template-subject = Oggetto del messaggio alle persone interessate, per lingua.
admin-settings-key-breach-template-body = Testo del messaggio
admin-settings-help-breach-template-body = Testo del messaggio alle persone interessate, per lingua.
admin-settings-key-breach-notify-audit = Registra le notifiche nel registro di audit
admin-settings-help-breach-notify-audit = Promemoria e rilevamenti diventano eventi di sicurezza.
admin-settings-key-breach-notify-webhook-urls = Indirizzi webhook
admin-settings-help-breach-notify-webhook-urls = Indirizzi informati di promemoria e messaggi. Trattarli come segreti; ogni chiamata è firmata.
admin-settings-key-breach-notify-operator-addresses = Caselle di posta degli operatori
admin-settings-help-breach-notify-operator-addresses = Indirizzi di caselle di posta locali che ricevono i promemoria nella propria posta in arrivo.
admin-settings-key-breach-subjects-in-app = Avviso nell'account della persona
admin-settings-help-breach-subjects-in-app = Mostra il messaggio all'interno dell'account della persona.
admin-settings-key-breach-subjects-inbox = Messaggio nella casella di posta della persona
admin-settings-help-breach-subjects-inbox = Recapita il messaggio nella casella di posta della persona.
admin-settings-key-breach-subjects-webhook = Invia anche ai webhook
admin-settings-help-breach-subjects-webhook = Invia il testo del messaggio anche agli indirizzi webhook.
admin-settings-key-breach-retention-years = Conservazione del registro in anni
admin-settings-help-breach-retention-years = Anni per cui un incidente chiuso resta nel registro.
admin-settings-key-breach-auto-enabled = Automazioni
admin-settings-help-breach-auto-enabled = Apre bozze di incidente a partire dai rilevamenti.
admin-settings-key-breach-auto-start-clock = Orologio della bozza
admin-settings-help-breach-auto-start-clock = Avvia l'orologio quando la bozza viene aperta, oppure solo quando viene confermata.
admin-settings-key-breach-auto-assignee = Assegna le bozze a
admin-settings-help-breach-auto-assignee = A chi viene assegnata una bozza.
admin-settings-key-breach-auto-escalate-after-hours = Escalation dopo ore
admin-settings-help-breach-auto-escalate-after-hours = Ore di attesa di una bozza prima dell'escalation. 0 non esegue mai l'escalation.
admin-settings-key-breach-auto-escalate-to = Escalation a
admin-settings-help-breach-auto-escalate-to = A chi viene assegnata una bozza dopo l'escalation.
admin-settings-key-breach-auto-window-minutes = Finestra di conteggio in minuti
admin-settings-help-breach-auto-window-minutes = Minuti entro cui una regola conta gli eventi.
admin-settings-key-breach-auto-theft-threshold = Avvisi di furto di sessione
admin-settings-help-breach-auto-theft-threshold = Avvisi nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spray o picco di accessi
admin-settings-help-breach-auto-credential-stuffing-threshold = Eventi nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-mass-forward-threshold = Regole di inoltro in massa
admin-settings-help-breach-auto-mass-forward-threshold = Eventi nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-role-escalation-threshold = Modifiche dei ruoli di amministratore
admin-settings-help-breach-auto-role-escalation-threshold = Modifiche nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-export-volume-threshold = Esportazioni insolite
admin-settings-help-breach-auto-export-volume-threshold = Esportazioni nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-audit-chain-threshold = Errori della catena di audit
admin-settings-help-breach-auto-audit-chain-threshold = Errori che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-backup-anomaly-threshold = Errori di backup o ripristino
admin-settings-help-breach-auto-backup-anomaly-threshold = Errori nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-breach-auto-dsr-export-threshold = Molte esportazioni di richieste sui dati
admin-settings-help-breach-auto-dsr-export-threshold = Esportazioni nella finestra che aprono una bozza. 0 disattiva.
admin-settings-key-ratelimit-queue-ms = Attesa massima per uno slot libero
admin-settings-help-ratelimit-queue-ms = Tempo massimo che una richiesta attende uno slot libero (concorrenza JMAP, decoder di immagini, worker di anteprima) prima di essere rifiutata con 429 e Retry-After. Intervallo 0-30000 ms; predefinito 3000. Zero rifiuta subito.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Richieste JMAP simultanee per account

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-ratelimit-jmap-per-account-concurrent = Richieste JMAP che un account autenticato può eseguire contemporaneamente, tra le sue schede e i suoi dispositivi. Non è mai condiviso con un altro account né con un altro utente dietro lo stesso indirizzo. Intervallo 4-64; predefinito 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Richieste JMAP simultanee, tutti gli account
admin-settings-help-ratelimit-jmap-global-concurrent = Richieste JMAP che l'intero processo del server esegue contemporaneamente, tutti gli account insieme. Una richiesta oltre il limite attende per il tempo di coda. Intervallo 4-4096; predefinito 256.
admin-settings-key-ratelimit-session-burst = Segnalazioni di tempi consecutive per sessione
admin-settings-help-ratelimit-session-burst = Segnalazioni di tempi del client che una sessione può inviare consecutivamente. L'endpoint usa come chiave la sessione, mai l'indirizzo. Intervallo 1-1000; predefinito 60.
admin-settings-key-ratelimit-session-per-minute = Segnalazioni di tempi ripristinate al minuto
admin-settings-help-ratelimit-session-per-minute = Segnalazioni di tempi del client al minuto che una sessione recupera dopo il burst. Intervallo 1-600; predefinito 30.
admin-settings-key-ratelimit-anon-burst = Richieste di consenso anonime consecutive
admin-settings-help-ratelimit-anon-burst = Richieste che un indirizzo può fare consecutivamente alle route anonime del consenso ai cookie. Dimensionato per un ufficio dietro un unico indirizzo. Intervallo 1-1000; predefinito 60.
admin-settings-key-ratelimit-anon-per-minute = Richieste di consenso anonime ripristinate al minuto
admin-settings-help-ratelimit-anon-per-minute = Richieste al minuto che un indirizzo recupera sulle route anonime del consenso ai cookie. Intervallo 1-600; predefinito 30.
admin-settings-key-ratelimit-trusted-multiplier = Quota dell'account da origine nota (%)
admin-settings-help-ratelimit-trusted-multiplier = Percentuale della quota di richieste dell'account che una casella di posta autenticata ottiene quando la richiesta proviene da un'origine con un accesso riuscito registrato. 100 mantiene la quota di base. Non allenta mai i budget anonimo, amministratore o user agent né un blocco attivo. Intervallo 100-1000; predefinito 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Facendo clic in un punto qualsiasi della barra degli allegati del lettore, la si espande o la si comprime. Quando è spento, funziona solo il pulsante freccia.

admin-settings-help-attachments-bar-download-all = Se la barra degli allegati offre un pulsante Scarica tutto che salva gli allegati consentiti come un unico file ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Quando ogni allegato nella barra mostra il relativo pulsante di download: al passaggio del mouse o sullo stato attivo (impostazione predefinita), sempre o mai. I download seguono ancora la politica degli allegati e mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Se le immagini mostrate all'interno del testo del messaggio sono incluse in Scarica tutto ZIP. Disattivato per impostazione predefinita.

admin-settings-help-cap-reading-delete-sender = Consenti agli utenti di abilitare l'azione del lettore che sposta tutti i messaggi del mittente corrispondenti nel Cestino dopo la conferma. Disabilitato per impostazione predefinita. Anche il profilo della funzione deve consentirlo.

admin-settings-help-compose-feature-family = Attiva o disattiva un controllo del compositore in base all'ID funzione. Impostazione predefinita attiva; bloccalo per impedire agli utenti di modificarlo.

admin-settings-help-compose-headers-allow-auto-submitted = Consente agli utenti di impostare l'invio automatico (RFC 3834) su un messaggio. Disattivato per impostazione predefinita: indica agli altri sistemi di non rispondere.

admin-settings-help-compose-headers-allow-custom = Consente agli utenti di aggiungere le proprie intestazioni (fino a 20, nomi controllati, intestazioni protette rifiutate).

admin-settings-help-compose-headers-allow-return-receipt-to = Consente agli utenti di richiedere ricevute con l'intestazione legacy Return-Receipt-To. Disattivato per impostazione predefinita.

admin-settings-help-compose-headers-allow-threading-edit = Consente agli utenti di modificare In-Reply-To e Riferimenti o di avviare un nuovo thread da una risposta.

admin-settings-help-compose-headers-forced = Una coppia Nome=valore per riga, aggiunta a ogni messaggio inviato dal dominio e in sostituzione del valore dell'utente. Le intestazioni protette vengono rifiutate.

admin-settings-help-compose-insert-image-resize-default = La dimensione iniziale della finestra di dialogo Immagine da file: originale, grande (1600 px) o media (1024 px).

admin-settings-help-compose-insert-max-inline-image-mb = Le immagini più grandi non vengono inserite nel corpo del messaggio; possono ancora essere attaccati. da 1 a 50.

admin-settings-help-compose-layout-default-font = Famiglia, dimensione e colore del carattere con cui inizia un nuovo messaggio, come famiglia|dimensione|#colore. Vuoto utilizza il carattere del tema.

admin-settings-help-compose-reply-to-allow-override = Quando è disattivata, la risposta dell'identità viene utilizzata così com'è e gli utenti non possono modificarla o rimuoverla.

admin-settings-help-compose-reply-to-show = Se il compositore mostra la riga Rispondi a: solo quando un'identità o il messaggio originale ne ha una, sempre o mai.

admin-settings-help-compose-sent-folder-fixed-id = Ruolo o ID della cartella utilizzato quando la regola è "cartella fissa".

admin-settings-help-compose-sent-folder-mode = Dove va a finire la copia di un messaggio inviato: la normale cartella Posta inviata, una cartella fissa, per dominio del destinatario, per identità o per regole oggetto.

admin-settings-help-compose-sent-folder-per-identity = Una coppia identità=cartella per riga. Utilizzato quando la regola è "per identità".

admin-settings-help-compose-sent-folder-rules = Una regola per riga, domain:example.com=cartella o subject:text=cartella. Vince la prima partita.

admin-settings-help-compose-sent-folder-skip-copy = Non conservare mai una copia di un messaggio inviato. Un amministratore può forzare questa attivazione o disattivazione.

admin-settings-help-compose-styles-custom = Gli stili del compositore di un utente come JSON, scritti dalla Galleria Stili. Gli utenti lo impostano; non c'è niente da configurare qui.

admin-settings-help-compose-styles-default-set = L'aspetto della galleria degli stili del compositore: predefinito, moderno, classico o compatto.

admin-settings-help-compose-styles-max-custom = La maggior parte degli stili personalizzati che un utente può mantenere nel compositore. 0 disattiva Crea stile.

admin-settings-help-contacts-save-from-message-add-note = Aggiungi la nota "Salvato da un messaggio il <data>" ai contatti salvati da un messaggio. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-contacts-save-from-message-default-book = L'ID della rubrica che riceve i contatti salvati da un messaggio; "default" utilizza la rubrica predefinita. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-contacts-save-from-message-default-scope = Quali partecipanti vengono selezionati per primi nella finestra di dialogo Salva contatti: solo il mittente, il mittente e Rispondi a o tutti. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-contacts-save-from-message-enabled = Offerta Salva i contatti da questo messaggio nei menu dei messaggi. Quando è spento l'elemento è nascosto. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-contacts-save-from-message-skip-automated = Lascia gli indirizzi di mancata risposta, di notifica e di mancato recapito fuori dalla finestra di dialogo Salva contatti a meno che non siano selezionati manualmente. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-contacts-save-sender-enabled = Offri la possibilità di salvare il mittente come contatto con un clic nei menu dei messaggi. Quando è spento l'elemento è nascosto. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-contacts-save-sender-use-reply-to = Salva l'indirizzo per le risposte anziché il mittente quando il mittente è una mailing list o un indirizzo di notifica. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-dsr-admin-step-up = Attivo (impostazione predefinita): un amministratore che approva una richiesta di dati, esegue una cancellazione o approva in blocco deve prima confermare di essere nuovamente lui (password e secondo fattore o passkey). La conferma dura cinque minuti. Off rimuove il controllo aggiuntivo; la decisione è ancora registrata nella traccia della richiesta.

admin-settings-help-export-default-format = Il formato con cui inizia la finestra di dialogo di esportazione.

admin-settings-help-export-default-scope = L'ambito con cui inizia la finestra di dialogo di esportazione: messaggio, conversazione, selezione o cartella.

admin-settings-help-export-dialog-enabled = L'esportazione nella visualizzazione dei messaggi e nei menu dei messaggi apre una finestra di dialogo. Quando sono disattivati, questi menu scaricano il messaggio come .eml.

admin-settings-help-export-folder-enabled = Consenti agli utenti di scaricare un'intera cartella come ZIP di messaggi. Ogni esportazione viene controllata.

admin-settings-help-export-folder-format = I formati offerti dall'esportazione di una cartella: un ZIP di file .eml, un ZIP contenente un file mbox o entrambi.

admin-settings-help-export-folder-include-subfolders = Se l'esportazione di una cartella include le sue sottocartelle per impostazione predefinita, mantenute come struttura di cartelle nel ZIP.

admin-settings-help-export-folder-max-bytes = Il numero massimo di byte di messaggi che una cartella esportata può contenere. Viene rifiutata una cartella più grande con le dimensioni indicate.

admin-settings-help-export-folder-max-messages = Il numero massimo di messaggi che una cartella esportata può contenere.

admin-settings-help-export-formats = I formati offerti dalla finestra di dialogo di esportazione: eml, mbox, txt, md, msg, docx, csv, json, pdf. Il server rifiuta un formato creato dal server elencato.

admin-settings-help-export-message-download-enabled = Consenti agli utenti di scaricare un messaggio come .eml, solo le sue intestazioni o come file mbox.

admin-settings-help-folders-hide-empty = Nascondi le cartelle vuote nella barra laterale. Gli utenti possono modificare l'impostazione predefinita o sovrascriverla per singole cartelle a meno che questa impostazione non sia bloccata. Nessuna posta viene eliminata.

admin-settings-help-folders-pinned-hide-from-tree = Le cartelle bloccate vengono visualizzate in un gruppo Bloccato sopra l'albero delle cartelle e mantengono la loro posizione nell'albero. Attiva nasconde una cartella bloccata che non ha sottocartelle dall'albero, quindi viene visualizzata solo nel gruppo Bloccati. Una cartella bloccata con sottocartelle rimane nell'albero.

admin-settings-help-mail-badge-auth = Quando nell'intestazione del lettore è presente il badge di autenticazione DMARC: sempre, solo se notabile, oppure mai.

admin-settings-help-mail-badge-density = Dimensione dei badge nella testata del lettore: compatta o comoda.

admin-settings-help-mail-badge-folder = Quando l'intestazione del lettore mostra il badge della cartella: sempre, solo quando notabile, oppure mai.

admin-settings-help-mail-badge-headers = Mostra un badge nel lettore per le intestazioni Expires, Reply-By, Sensitivity e Sostituisci di un messaggio.

admin-settings-help-mail-badge-labels = Se i badge dell'intestazione del lettore mostrano un'icona con testo o solo l'icona.

admin-settings-help-mail-badge-list = Quando l'intestazione del lettore mostra il badge della mailing list: sempre, solo quando notevole o mai. Ha bisogno di un'intestazione List-Id o List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Quando l'intestazione del lettore presenta il badge igienizzato HTML: sempre, solo se notabile, oppure mai.

admin-settings-help-mail-badge-spam = Quando l'intestazione del lettore mostra il badge del punteggio spam: sempre, solo quando notevole o mai.

admin-settings-help-mail-brand-avatars-require-auth = mai: mostra un logo ogni volta che il dominio corrisponde. errore noto (impostazione predefinita): mostralo a meno che il risultato DKIM o DMARC del messaggio memorizzato nella cache non sia un errore; i messaggi non selezionati mantengono il logo. sempre: mostralo solo dopo un passaggio DMARC allineato con DKIM per il dominio proprio del brand. Le sostituzioni di utenti e domini possono essere bloccate.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Pochi secondi in cui il corpo deve mantenere la concentrazione prima che l'intestazione completa del destinatario e dell'oggetto si ripieghi nel suo riepilogo. Scegli 0 per disabilitare la piegatura a tempo o 1–60 secondi. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-mail-compose-priority-default = La priorità con cui inizia un nuovo messaggio nel compositore.

admin-settings-help-mail-compose-priority-enabled = Mostra il selettore di priorità Normale, Alta e Bassa nel gruppo Opzioni del compositore. Quando è disattivata, la nuova posta viene inviata senza intestazioni prioritarie.

admin-settings-help-mail-expiry-hide-expired = Cosa fare con un messaggio la cui data di scadenza è passata: niente, mostrarlo in grigio o suggerire di archiviarlo.

admin-settings-help-mail-list-keep = Mittenti e mailing list sempre mantenuti, uno per riga come sender:address o list:List-Id. Pulisci questo mittente, annulla l'iscrizione ai suggerimenti e ignorali nelle regole di conservazione.

admin-settings-help-mail-list-page-size = Quanti messaggi recupera l'elenco ogni volta che un utente scorre fino alla fine. Predefinito 50, da 20 a 200. Gli utenti scelgono il proprio valore a meno che non lo blocchi.

admin-settings-help-mail-list-unsubscribe-mode = Come funziona l'annullamento dell'iscrizione quando una lista offre una richiesta con un clic: chiedi prima, inviala subito o apri solo il collegamento o la bozza della lista.

admin-settings-help-mail-message-list-group = Raggruppa conversazioni per data di ricezione o invio, mittente, oggetto, importanza o dimensione. Gli utenti possono ignorarlo a meno che non siano bloccati.

admin-settings-help-mail-priority-marks = Quale priorità contrassegna l'elenco dei messaggi, le conversazioni e la visualizzazione del lettore: sia alta che bassa, solo alta o nessuna. Gli utenti scelgono in Impostazioni > Cartelle e barra laterale se non bloccate.

admin-settings-help-mail-priority-marks-style = Come appare un segno di priorità: un'icona, un'icona con le parole Alta o Bassa o una barra sottile. Icona e testo mantengono il significato senza colore.

admin-settings-help-mail-priority-send-headers = Quali intestazioni riceve un messaggio con priorità alta o bassa quando viene inviato: una qualsiasi tra Importanza, X-Priority e X-MSMail-Priority.

admin-settings-help-mail-priority-trust-sender-headers = Indica se viene mostrata la priorità impostata dal mittente: sempre, solo quando il messaggio ha superato un controllo allineato DMARC o mai. Utilizza attendibile o mai contro i mittenti che contrassegnano ogni messaggio con priorità alta.

admin-settings-help-mail-reply-from-header-priority = Le intestazioni dei messaggi vengono lette, in questo ordine, per trovare l'indirizzo a cui è stato inviato un messaggio quando si sceglie l'identità per una risposta. Nomi: Consegnato-A, X-Originale-A, Busta-A, X-Inoltrato-A, Rispedito-A, A, Cc. Un'intestazione tralasciata non viene letta.

admin-settings-help-mail-search-default-scope = Avvia nuove normali ricerche di posta nella cartella corrente o in tutte le cartelle. Controlla l'impostazione predefinita iniziale, anche quando è bloccata; gli utenti possono comunque scegliere un ambito diverso per una ricerca particolare. Le query salvate e con ambito esplicito vengono conservate.

admin-settings-help-metrics-dimensions-enabled = Memorizza le statistiche salvate suddivise per tipologia di client (web) e classe di dispositivo (desktop, mobile). Disattiva per memorizzare una riga combinata per metrica e minuto.

admin-settings-help-metrics-persist-enabled = Salva le statistiche sulle prestazioni (velocità del proxy dell'immagine, tempi di caricamento del browser, limiti di velocità e conteggi di protezione delle richieste) nel database a ogni intervallo di scaricamento e quando il server si arresta, in modo che i grafici conservino la cronologia dopo un riavvio. Niente identifica una persona, un indirizzo o un messaggio. Disattivarli per mantenerli solo in memoria.

admin-settings-help-metrics-persist-flush-seconds = La frequenza con cui le statistiche sulle prestazioni vengono scritte nel database (da 5 a 3600 secondi). Un server che viene ucciso senza preavviso perde al massimo questa cifra.

admin-settings-help-observability-client-capture-console = Segnala anche le chiamate console.error, al massimo cinque al minuto per pagina. Disattivato per impostazione predefinita perché le librerie registrano le condizioni previste.

admin-settings-help-observability-client-capture-csp = Segnala le violazioni della politica di sicurezza dei contenuti rilevate dal browser, solo con la direttiva bloccata e l'origine. Attivo per impostazione predefinita.

admin-settings-help-observability-client-enabled = Raccogli i report sugli errori dall'app Web (limiti degli errori, errori non rilevati, blocchi non riusciti, errori del lavoratore, violazioni CSP) con analisi dello stack cancellate, visualizzate in Osservabilità > Errori del client. Attivo per impostazione predefinita. I report non contengono contenuti di messaggi, indirizzi, token, nomi di allegati o query URL. Spegni per interrompere la raccolta; i report esistenti scadono con la relativa conservazione.

admin-settings-help-observability-client-forward = Invia inoltre ogni report cancellato al sink di errore configurato con MW_ERROR_FORWARD_URL. Disattivato per impostazione predefinita. Senza un sink configurato non viene inviato nulla.

admin-settings-help-observability-client-include-account-hash = Aggiungi un hash unidirezionale con chiave dell'account ai report inviati da una sessione di accesso, in modo che i report di un account possano essere distinti senza nominarlo. Disattivato per impostazione predefinita. L'hash cambia quando la chiave privacy ruota.

admin-settings-help-observability-client-max-per-session = La maggior parte dei rapporti di errore può essere inviata con il caricamento di una pagina (da 1 a 1000). Le ripetizioni dello stesso errore entro 60 secondi contano una volta. Predefinito 20.

admin-settings-help-observability-client-sample-rate = Frazione di report di errore inviati da un browser, da 0 a 1. Predefinito 1 (tutti). Il caricamento di ogni pagina si ferma comunque al limite per sessione.

admin-settings-help-observability-client-source-maps = Risolvi le posizioni dei bundle nelle analisi dello stack nei file di origine quando la build ha fornito mappe di origine nascoste (create con MW_SOURCEMAP=hidden). Le mappe vengono lette solo dal server e non vengono mai fornite ai browser. Attivo per impostazione predefinita; senza mappe vengono mostrati i fotogrammi grezzi.

admin-settings-help-privacy-error-reports = Indica se un browser invia una segnalazione di errore quando una parte dell'app non funziona. Attivo per impostazione predefinita. Ambito globale, dominio e utente: imposta qui il valore predefinito o bloccalo; le persone possono disattivarlo autonomamente in Impostazioni > Privacy a meno che non sia bloccato.

admin-settings-help-privacy-image-cache-max-stale-secs = Pochi secondi dopo la sua freschezza, un'immagine memorizzata nella cache viene mantenuta per rispondere quando non è possibile raggiungere l'origine. Mai al di sotto della finestra di riconvalida. Intervallo 0–604800; predefinito 86400.

admin-settings-help-privacy-image-cache-stale-secs = Pochi secondi dopo la sua freschezza, un'immagine memorizzata nella cache viene ancora pubblicata immediatamente mentre viene riconvalidata in background. Zero lo spegne. Intervallo 0–86400; predefinito 3600.

admin-settings-help-privacy-image-resize-enabled = Ridimensiona le immagini remote di grandi dimensioni fino alla larghezza richiesta dal lettore (320, 640, 960, 1280 o 1600 pixel). Le immagini animate vengono fornite non ridimensionate. Disattivato per impostazione predefinita.

admin-settings-help-privacy-image-resize-format = Seleziona automaticamente il miglior codificatore nella build, che è PNG. Automatico predefinito.

admin-settings-help-privacy-image-resize-max-pixels = Le immagini con più pixel decodificati vengono fornite senza scala; la dimensione viene letta dall'intestazione prima di qualsiasi decodifica. Intervallo 100.000–64.000.000; predefinito 4000000.

admin-settings-help-privacy-image-resize-max-width = L'immagine più ampia prodotta dal ridimensionamento. Intervallo 320–1600; predefinito 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Il modo in cui Impostazioni > Sicurezza > Sessioni assegna un nome all'account da cui proviene una sessione di accesso alla casella di posta collegata: l'indirizzo completo, l'indirizzo con la prima parte del nome mascherata o nulla. Il valore viene mostrato solo al titolare delle sessioni, mai agli altri utenti. Predefinito: indirizzo completo.

admin-settings-help-properties-folder-enabled = Offerta Proprietà nei menu delle cartelle e nel collegamento. Quando è disattivata, la finestra di dialogo non è disponibile.

admin-settings-help-properties-folder-notes-enabled = Consenti una descrizione o una nota su ciascuna cartella.

admin-settings-help-properties-folder-notes-max-length = Il numero massimo di caratteri che una nota di cartella può contenere.

admin-settings-help-properties-folder-overrides-allowed = Quali impostazioni un utente può sovrascrivere per una singola cartella. Una sostituzione non elencata viene nascosta e qualsiasi valore memorizzato viene ignorato.

admin-settings-help-properties-folder-sections = Le sezioni della finestra di dialogo Proprietà della cartella: generale, remota, dimensione, offline, sostituzioni, note, azioni. Una sezione non elencata è nascosta.

admin-settings-help-properties-folder-size-compute = Quando viene calcolata la dimensione di una cartella sul server: mai, quando si apre la finestra di dialogo o in background. Il conteggio di una casella di posta di grandi dimensioni costa una scansione al server.

admin-settings-help-properties-headers-raw = Consentire alla finestra di dialogo Proprietà del messaggio di elencare le intestazioni non elaborate. Quando è disattivato, viene visualizzato solo il riepilogo analizzato.

admin-settings-help-properties-message-enabled = Proprietà dell'offerta nei menu dei messaggi e nel collegamento. Quando è disattivata, la finestra di dialogo non è disponibile.

admin-settings-help-properties-message-permanent-delete = Indica se la finestra di dialogo del messaggio può eliminare definitivamente un messaggio: consenti, chiedi due volte o nega. Un messaggio sottoposto a conservazione a fini giudiziari non viene mai eliminato. L'eliminazione in questo modo non può essere annullata.

admin-settings-help-properties-message-sections = Le sezioni della finestra di dialogo Proprietà del messaggio: riepilogo, punteggio, intestazioni, allegati, azioni. Una sezione non elencata è nascosta.

admin-settings-help-properties-offline-clear-enabled = Consenti agli utenti di svuotare la cache offline di una singola cartella dalla finestra di dialogo Proprietà.

admin-settings-help-properties-remote-location-visibility = Quanta parte della posizione lato server vedono gli utenti: completa (host e percorso), host mascherato (host parzialmente nascosto), solo percorso (percorso della cassetta postale senza host) o nascosta.

admin-settings-help-properties-scoring-show = Mostra il punteggio spam, i risultati dell'autenticazione e i segnali di sicurezza nella finestra di dialogo Proprietà del messaggio.

admin-settings-help-properties-shortcut-enabled = Apri la finestra di dialogo Proprietà dalla tastiera su una cartella focalizzata o su un messaggio selezionato.

admin-settings-help-properties-shortcut-key = La combinazione di tasti che apre Proprietà; Alt+Invio per impostazione predefinita.

admin-settings-help-reader-actions-archive = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-block-sender = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-colors = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-conversation = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-create-filter = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-delete = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-delete-sender = Nascosto per impostazione predefinita. Scegli la barra degli strumenti o il menu Altro per abilitarlo quando la funzionalità di eliminazione del mittente e il profilo funzione lo consentono. Gli utenti possono ignorare il posizionamento a meno che non siano bloccati.

admin-settings-help-reader-actions-edit-draft = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-export = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-finish-followup = Scegli la barra degli strumenti, il menu Altro o Nascosto per completare i messaggi contrassegnati e i promemoria di follow-up.

admin-settings-help-reader-actions-flag = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-focus-focused = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-focus-other = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-forward = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-headers = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-links = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-move = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-not-spam = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-open-draft = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-pin = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-print = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-properties = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-read = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-reply = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-reply-all = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-save-contacts = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-save-sender = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-search = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-security = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-spam = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-sweep = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-view-source = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-reader-actions-zoom = Scegli barra degli strumenti, menu Altro o nascosto. Si applicano ancora restrizioni sulle funzionalità, autorizzazioni e contesto del messaggio. Gli utenti possono scegliere a meno che non sia bloccato.

admin-settings-help-retention-client-errors-event-days = Giorni in cui viene conservato il verificarsi di un singolo errore del browser (la relativa traccia dello stack e la classe del browser). Impostazione predefinita 30. Un gruppo viene mantenuto più a lungo, vedere l'impostazione successiva.

admin-settings-help-retention-client-errors-group-days = Giorni in cui viene conservato un gruppo di errori del browser (nome, messaggio, conteggi) dopo l'ultima visualizzazione. Predefinito 180. Mai inferiore alla conservazione dell'occorrenza.

admin-settings-help-retention-metrics-day-buckets-days = Giorni di statistiche sulle prestazioni giornaliere conservate prima che vengano eliminate. 0 li mantiene illimitati, il che consente alla tabella di crescere lentamente finché il server è in funzione.

admin-settings-help-retention-metrics-hour-buckets-days = Giorni di statistiche sulle prestazioni orarie conservate prima di essere unite in righe giornaliere (almeno 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Ore di statistiche sulle prestazioni di un minuto conservate prima di essere unite in righe orarie (almeno 1).

admin-settings-help-retention-user-allow-strip-attachments = Indica se gli utenti possono creare regole per rimuovere allegati di grandi dimensioni dalla vecchia posta. Disattivato per impostazione predefinita. Quando è disattivata, l'azione è nascosta nell'editor delle regole, una regola che la utilizza viene rifiutata quando viene salvata e le regole salvate in precedenza vengono ignorate durante l'esecuzione del processo.

admin-settings-help-retention-user-calendar-enabled = Indica se gli utenti possono scegliere calendari i cui eventi vengono eliminati un determinato numero di giorni dopo la loro conclusione. Disattivato per impostazione predefinita. Quando è disattivata, l'opzione è nascosta in Impostazioni, una scelta viene rifiutata quando viene salvata e le scelte salvate in precedenza non vengono eseguite.

admin-settings-help-retention-user-calendar-max-days = La maggior parte dei giorni un utente può attendere prima che gli eventi terminati vengano eliminati. Un valore maggiore viene rifiutato quando viene salvato e abbassato durante l'esecuzione.

admin-settings-help-retention-user-calendar-min-days = Numero minimo di giorni dopo la fine di un evento prima che una regola utente possa eliminarlo. Un valore inferiore viene rifiutato quando viene salvato e aumentato durante l'esecuzione.

admin-settings-help-retention-user-notes-enabled = Indica se gli utenti possono scegliere di rimuovere le note non modificate per un determinato numero di giorni. Disattivato per impostazione predefinita. Le note rimosse possono essere ripristinate fino al termine della finestra di annullamento delle regole di conservazione.

admin-settings-help-retention-user-notes-max-days = La maggior parte dei giorni un utente può attendere prima che le note intatte vengano rimosse. Un valore maggiore viene rifiutato quando viene salvato e abbassato durante l'esecuzione.

admin-settings-help-retention-user-notes-min-days = Numero minimo di giorni in cui una nota deve rimanere intatta prima che una regola utente possa rimuoverla. Un valore inferiore viene rifiutato quando viene salvato e aumentato durante l'esecuzione.

admin-settings-help-retention-user-strip-keep-inline-images = Se le immagini mostrate all'interno del corpo del messaggio vengono mantenute quando gli allegati vengono rimossi.

admin-settings-help-retention-user-strip-min-age-days = Numero minimo di giorni in cui un messaggio deve essere vecchio prima che una regola possa rimuovere i suoi allegati. Un valore inferiore viene rifiutato quando la regola viene salvata e generato durante l'esecuzione.

admin-settings-help-retention-user-strip-min-attachment-kb = Verranno rimossi solo gli allegati di almeno queste dimensioni. Gli allegati più piccoli rimangono nel messaggio.

admin-settings-help-retention-user-strip-undo-days = Giorni in cui il messaggio originale rimane nel Cestino dopo la rimozione degli allegati. -1 segue la finestra di annullamento per le regole di conservazione. 0 cancella immediatamente l'originale.

admin-settings-help-server-shutdown-grace-seconds = Per quanto tempo un server in arresto lascia terminare le richieste in corso prima di chiudere le connessioni, in secondi. La variabile MW_HTTP_SHUTDOWN_TIMEOUT_MS la sovrascrive. Si applica al prossimo avvio.

admin-settings-help-session-last-activity-flush-seconds = L'ora dell'ultima visualizzazione di una sessione viene scritta al massimo con questa frequenza, in secondi. Zero lo scrive su ogni richiesta. Non modifica i limiti inattivi o assoluti. Si applica al prossimo avvio.

admin-settings-help-session-persist-across-restarts = Quando questa opzione è attivata, le sessioni della cassetta postale, dell'app e della console rimangono valide dopo il riavvio del server. Quando è disattivato, ogni sessione termina all'avvio del server. Si applica al prossimo avvio.

admin-settings-key-attachments-bar-click-to-toggle = Fare clic sulla barra degli allegati per attivare/disattivare

admin-settings-key-attachments-bar-download-all = Pulsante Scarica tutto

admin-settings-key-attachments-bar-show-download-buttons = Pulsanti per il download degli allegati

admin-settings-key-attachments-bar-zip-inline-images = Immagini in linea nel Scarica tutto ZIP

admin-settings-key-cap-reading-delete-sender = Consenti l'eliminazione di tutta la posta di un mittente

admin-settings-key-compose-feature-family = Caratteristiche del compositore

admin-settings-key-compose-headers-allow-auto-submitted = Consenti invio automatico

admin-settings-key-compose-headers-allow-custom = Consenti intestazioni personalizzate

admin-settings-key-compose-headers-allow-return-receipt-to = Consenti ricevuta di ritorno a

admin-settings-key-compose-headers-allow-threading-edit = Consenti la modifica delle intestazioni del threading

admin-settings-key-compose-headers-forced = Intestazioni forzate

admin-settings-key-compose-insert-image-resize-default = Preimpostazione delle dimensioni dell'immagine

admin-settings-key-compose-insert-max-inline-image-mb = Immagine in linea più grande (MB)

admin-settings-key-compose-layout-default-font = Carattere predefinito del messaggio

admin-settings-key-compose-reply-to-allow-override = Consenti la modifica delle risposte

admin-settings-key-compose-reply-to-show = Riga Rispondi a

admin-settings-key-compose-sent-folder-fixed-id = Copia inviata: cartella fissa

admin-settings-key-compose-sent-folder-mode = Copia inviata: regola cartella

admin-settings-key-compose-sent-folder-per-identity = Copia inviata: cartella per identità

admin-settings-key-compose-sent-folder-rules = Copia inviata: regole

admin-settings-key-compose-sent-folder-skip-copy = Copia inviata: non salvare

admin-settings-key-compose-styles-custom = Stili personalizzati

admin-settings-key-compose-styles-default-set = Insieme di stile

admin-settings-key-compose-styles-max-custom = Stili personalizzati per utente

admin-settings-key-contacts-save-from-message-add-note = Nota sui contatti salvati da un messaggio

admin-settings-key-contacts-save-from-message-default-book = Rubrica per i contatti salvati da un messaggio

admin-settings-key-contacts-save-from-message-default-scope = I contatti vengono selezionati per primi quando si salva da un messaggio

admin-settings-key-contacts-save-from-message-enabled = Salvare i contatti da un messaggio

admin-settings-key-contacts-save-from-message-skip-automated = Salta i mittenti automatici durante il salvataggio dei contatti

admin-settings-key-contacts-save-sender-enabled = Salva mittente come contatto

admin-settings-key-contacts-save-sender-use-reply-to = Salva Rispondi a per i mittenti dell'elenco

admin-settings-key-dsr-admin-step-up = Conferma nuovamente prima di approvare

admin-settings-key-export-default-format = Formato predefinito della finestra di dialogo Esporta

admin-settings-key-export-default-scope = Ambito predefinito della finestra di dialogo Esporta

admin-settings-key-export-dialog-enabled = Finestra di dialogo Esporta

admin-settings-key-export-folder-enabled = Esportazione di cartelle

admin-settings-key-export-folder-format = Formato di esportazione della cartella

admin-settings-key-export-folder-include-subfolders = Includere sottocartelle nell'esportazione

admin-settings-key-export-folder-max-bytes = Limite dimensione esportazione cartella

admin-settings-key-export-folder-max-messages = Limite dei messaggi di esportazione della cartella

admin-settings-key-export-formats = Formati di dialogo di esportazione

admin-settings-key-export-message-download-enabled = Scaricamento dei messaggi

admin-settings-key-folders-hide-empty = Nascondi le cartelle vuote

admin-settings-key-folders-pinned-hide-from-tree = Nascondi le cartelle appuntate dall'albero

admin-settings-key-mail-badge-auth = Distintivo di autenticazione

admin-settings-key-mail-badge-density = Dimensioni del badge del lettore

admin-settings-key-mail-badge-folder = Distintivo della cartella

admin-settings-key-mail-badge-headers = Badge di intestazione

admin-settings-key-mail-badge-labels = Etichette per badge lettore

admin-settings-key-mail-badge-list = Distintivo della mailing list

admin-settings-key-mail-badge-sanitized = Badge HTML igienizzato

admin-settings-key-mail-badge-spam = Badge del punteggio spam

admin-settings-key-mail-brand-avatars-require-auth = Autenticazione necessaria per i loghi del marchio

admin-settings-key-mail-compose-header-auto-collapse-seconds = Ritardo di compressione automatica dell'intestazione del compositore

admin-settings-key-mail-compose-priority-default = Priorità del messaggio predefinita

admin-settings-key-mail-compose-priority-enabled = Selettore di priorità nel compositore

admin-settings-key-mail-expiry-hide-expired = Messaggi scaduti

admin-settings-key-mail-list-keep = Tieni sempre

admin-settings-key-mail-list-page-size = Dimensioni della pagina dell'elenco dei messaggi

admin-settings-key-mail-list-unsubscribe-mode = Mailing list Annulla l'iscrizione

admin-settings-key-mail-message-list-group = Raggruppamento dei messaggi

admin-settings-key-mail-priority-marks = Segni di priorità

admin-settings-key-mail-priority-marks-style = Stile del contrassegno di priorità

admin-settings-key-mail-priority-send-headers = Intestazioni prioritarie scritte all'invio

admin-settings-key-mail-priority-trust-sender-headers = Considera attendibile la priorità impostata dal mittente

admin-settings-key-mail-reply-from-header-priority = Intestazioni che scelgono l'identità della risposta

admin-settings-key-mail-search-default-scope = Ambito di ricerca della posta predefinito

admin-settings-key-metrics-dimensions-enabled = Suddividi le statistiche per tipo di client e classe di dispositivo

admin-settings-key-metrics-persist-enabled = Mantieni le statistiche durante i riavvii

admin-settings-key-metrics-persist-flush-seconds = Intervallo di salvataggio delle statistiche (secondi)

admin-settings-key-observability-client-capture-console = Segnala errori della console

admin-settings-key-observability-client-capture-csp = Segnala violazioni CSP

admin-settings-key-observability-client-enabled = Rapporti sugli errori del browser

admin-settings-key-observability-client-forward = Inoltrare segnalazioni di errori

admin-settings-key-observability-client-include-account-hash = Aggiungi l'hash dell'account alle segnalazioni di errori

admin-settings-key-observability-client-max-per-session = Rapporti di errore per caricamento della pagina

admin-settings-key-observability-client-sample-rate = Frequenza di campionamento del rapporto errori

admin-settings-key-observability-client-source-maps = Risolvi le tracce dello stack con le mappe di origine

admin-settings-key-privacy-error-reports = Invia segnalazioni di errori

admin-settings-key-privacy-image-cache-max-stale-secs = Mantieni obsoleto quando l'origine fallisce (secondi)

admin-settings-key-privacy-image-cache-stale-secs = Pubblicazione obsoleta durante la riconvalida (secondi)

admin-settings-key-privacy-image-resize-enabled = Ridimensiona immagini di grandi dimensioni

admin-settings-key-privacy-image-resize-format = Formato immagine ridimensionato

admin-settings-key-privacy-image-resize-max-pixels = Immagine più grande da ridimensionare (pixel)

admin-settings-key-privacy-image-resize-max-width = Immagine ridimensionata più ampia (pixel)

admin-settings-key-privacy-sessions-show-linked-origin = Origine della casella di posta collegata nelle sessioni

admin-settings-key-properties-folder-enabled = Finestra di dialogo delle proprietà della cartella

admin-settings-key-properties-folder-notes-enabled = Note sulla cartella

admin-settings-key-properties-folder-notes-max-length = Lunghezza della nota della cartella

admin-settings-key-properties-folder-overrides-allowed = Sono consentite sostituzioni per cartella

admin-settings-key-properties-folder-sections = Sezioni della finestra di dialogo Cartella

admin-settings-key-properties-folder-size-compute = Calcolo della dimensione della cartella

admin-settings-key-properties-headers-raw = Intestazioni grezze

admin-settings-key-properties-message-enabled = Finestra di dialogo delle proprietà del messaggio

admin-settings-key-properties-message-permanent-delete = Eliminazione permanente dalle proprietà

admin-settings-key-properties-message-sections = Sezioni della finestra di dialogo dei messaggi

admin-settings-key-properties-offline-clear-enabled = Svuota la cache offline per cartella

admin-settings-key-properties-remote-location-visibility = Dettagli sulla posizione remota

admin-settings-key-properties-scoring-show = Punteggio e dettagli di autenticazione

admin-settings-key-properties-shortcut-enabled = Collegamento alle proprietà

admin-settings-key-properties-shortcut-key = Tasto di scelta rapida Proprietà

admin-settings-key-reader-actions-archive = Posizionamento dell'azione di archiviazione

admin-settings-key-reader-actions-block-sender = Blocca il posizionamento dell'azione del mittente

admin-settings-key-reader-actions-colors = Posizionamento dell'azione dei colori del messaggio

admin-settings-key-reader-actions-conversation = Visualizza il posizionamento dell'azione della conversazione

admin-settings-key-reader-actions-create-filter = Crea posizionamento azione filtro

admin-settings-key-reader-actions-delete = Elimina il posizionamento dell'azione

admin-settings-key-reader-actions-delete-sender = Elimina tutti i messaggi dall'azione del mittente

admin-settings-key-reader-actions-edit-draft = Modifica la bozza del posizionamento dell'azione

admin-settings-key-reader-actions-export = Esporta posizionamento dell'azione

admin-settings-key-reader-actions-finish-followup = Contrassegna il follow-up come azione completata

admin-settings-key-reader-actions-flag = Posizionamento dell'azione contrassegna/rimuovi contrassegno

admin-settings-key-reader-actions-focus-focused = Passa al posizionamento dell'azione mirata

admin-settings-key-reader-actions-focus-other = Passa al posizionamento Altra azione

admin-settings-key-reader-actions-forward = Posizionamento dell'azione in avanti

admin-settings-key-reader-actions-headers = Posizionamento delle azioni delle intestazioni non elaborate

admin-settings-key-reader-actions-links = Esamina il posizionamento delle azioni dei link

admin-settings-key-reader-actions-move = Sposta il posizionamento dell'azione

admin-settings-key-reader-actions-not-spam = Posizionamento di azioni non spam

admin-settings-key-reader-actions-open-draft = Apri il posizionamento dell'azione della bozza di risposta

admin-settings-key-reader-actions-pin = Posizionamento dell'azione Blocca/Sblocca

admin-settings-key-reader-actions-print = Stampa il posizionamento dell'azione

admin-settings-key-reader-actions-properties = Posizionamento dell'azione delle proprietà del messaggio

admin-settings-key-reader-actions-read = Posizionamento di azioni lette/non lette

admin-settings-key-reader-actions-reply = Posizionamento dell'azione di risposta

admin-settings-key-reader-actions-reply-all = Rispondi a tutti i posizionamenti di azioni

admin-settings-key-reader-actions-save-contacts = Salva il posizionamento delle azioni dei contatti dei messaggi

admin-settings-key-reader-actions-save-sender = Salva il posizionamento dell'azione del mittente

admin-settings-key-reader-actions-search = Posizionamento di azioni di ricerca correlate

admin-settings-key-reader-actions-security = Posizionamento dell'azione in modalità di sicurezza

admin-settings-key-reader-actions-spam = Contrassegna come posizionamento di azioni spam

admin-settings-key-reader-actions-sweep = Effettua la scansione del posizionamento dell'azione del mittente

admin-settings-key-reader-actions-view-source = Visualizza il posizionamento dell'azione dell'origine del messaggio

admin-settings-key-reader-actions-zoom = Posizionamento dell'azione zoom

admin-settings-key-retention-client-errors-event-days = Occorrenze di errori mantenute (giorni)

admin-settings-key-retention-client-errors-group-days = Gruppi di errori mantenuti (giorni)

admin-settings-key-retention-metrics-day-buckets-days = Statistiche giornaliere conservate (giorni)

admin-settings-key-retention-metrics-hour-buckets-days = Statistiche orarie conservate (giorni)

admin-settings-key-retention-metrics-minute-buckets-hours = Statistiche dei minuti conservati (ore)

admin-settings-key-retention-user-allow-strip-attachments = Gli utenti possono rimuovere gli allegati dalla vecchia posta

admin-settings-key-retention-user-calendar-enabled = Gli utenti possono eliminare gli eventi del calendario terminati

admin-settings-key-retention-user-calendar-max-days = La maggior parte dei giorni prima della fine degli eventi viene eliminata

admin-settings-key-retention-user-calendar-min-days = Il numero minimo di giorni prima della fine degli eventi viene eliminato

admin-settings-key-retention-user-notes-enabled = Gli utenti possono rimuovere le note intatte

admin-settings-key-retention-user-notes-max-days = Quasi tutti i giorni prima che le note intatte vengano rimosse

admin-settings-key-retention-user-notes-min-days = Meno giorni prima che le note intatte vengano rimosse

admin-settings-key-retention-user-strip-keep-inline-images = Mantieni le immagini in linea

admin-settings-key-retention-user-strip-min-age-days = Meno giorni prima che gli allegati vengano rimossi

admin-settings-key-retention-user-strip-min-attachment-kb = Allegato più piccolo rimosso (KB)

admin-settings-key-retention-user-strip-undo-days = Finestra di annullamento dopo la rimozione degli allegati (giorni)

admin-settings-key-server-shutdown-grace-seconds = Periodo di grazia per lo spegnimento

admin-settings-key-session-last-activity-flush-seconds = Intervallo di scrittura visualizzato per ultimo

admin-settings-key-session-persist-across-restarts = Mantieni le sessioni tra i riavvii

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = Disattivato per impostazione predefinita. Salva istantanee di posta crittografate nell'ambito dell'account durante i ricaricamenti del browser senza scadenza. Si applicano ancora i limiti di voci e byte esistenti. Gli utenti possono cancellare le copie salvate; disabilitando questa preferenza li rimuovi dal browser attivo. Ciò non concede l'autorizzazione all'archiviazione persistente del browser o al download degli allegati.

admin-settings-help-mail-conversations-allowed = Consenti agli utenti di raggruppare le risposte correlate in conversazioni. Disponibile per impostazione predefinita, ma inizia la preferenza dell'utente. La disabilitazione della disponibilità sovrascrive la scelta dell'utente senza modificare il raggruppamento di categorie.

admin-settings-help-mail-conversations-enabled = Disattivato per impostazione predefinita. Raggruppa le risposte caricate per thread, indipendentemente da Raggruppa per. Imposta il valore predefinito o blocca questa preferenza. Anche la disponibilità della conversazione deve essere abilitata.

admin-settings-key-mail-cache-keep-indefinitely = Mantieni la cache della posta del browser a tempo indeterminato

admin-settings-key-mail-conversations-allowed = Rendi disponibile il threading della conversazione

admin-settings-key-mail-conversations-enabled = Raggruppare i messaggi in conversazioni

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = Fino a 100 domini di mittenti esatti in minuscolo i cui collegamenti ai messaggi esaminati vengono abilitati automaticamente. I sottodomini sono separati. Questo non considera attendibili i siti Web di destinazione né consente immagini remote. Vuoto per impostazione predefinita; gli utenti possono modificarlo a meno che non siano bloccati.

admin-settings-key-links-trusted-sender-domains = Domini dei mittenti attendibili per i collegamenti ai messaggi

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Mostra i dettagli di download e file accanto al titolo dell'anteprima come icone (impostazione predefinita), etichette o icone ed etichette. Può essere impostato e bloccato a livello globale, per dominio o per utente.

admin-settings-help-reader-attachment-thumbnails = Disattivato per impostazione predefinita. Consenti agli allegati di immagini visibili di caricare le anteprime in miniatura. Si applicano ancora le regole di sicurezza degli allegati. Può essere impostato e bloccato a livello globale, per dominio o per utente.

admin-settings-key-reader-attachment-action-style = Azioni di anteprima dell'allegato

admin-settings-key-reader-attachment-thumbnails = Miniature delle immagini allegate

admin-settings-key-mail-compose-auto-add-mention-recipients = Aggiungi automaticamente le persone menzionate ai destinatari
admin-settings-help-mail-compose-auto-add-mention-recipients = Quando selezioni una persona dai suggerimenti delle @menzioni, aggiungila ad A se non è già presente in A, Cc o Ccn. Disattivato per impostazione predefinita.

admin-settings-linked-disable-title = Disattivare l’accesso salvato agli account collegati?
admin-settings-linked-disable-body = Salvare questa modifica rimuove le credenziali usate per rinnovare l’accesso agli account collegati. Gli utenti dovranno accedere di nuovo quando occorre rinnovare l’accesso. Riattivare questa opzione non ripristina le credenziali.
admin-settings-linked-disable-confirm = Disattiva e cancella l’accesso salvato

admin-settings-key-reader-auto-hide-header = Nascondi automaticamente le intestazioni dei messaggi
admin-settings-help-reader-auto-hide-header = Apri ogni messaggio con i dettagli di mittente, destinatari e metadati compressi. Oggetto e azioni restano visibili. Usa Mostra intestazione per vedere i dettagli. Disattivato per impostazione predefinita.

admin-settings-key-time-region = Regione
admin-settings-help-time-region = auto o un codice regione di due lettere, come GB, US o PT.

admin-settings-key-time-date-format = Formato data
admin-settings-help-time-date-format = auto, short, medium, long, full o un modello come yyyy-MM-dd o dd/MM/yyyy. Racchiudi il testo letterale tra 'apici'.

admin-settings-key-time-time-format = Formato ora
admin-settings-help-time-time-format = auto, short, medium, long, full o HH:mm:ss / h:mm a. H = 24 ore, h = 12 ore, a = AM/PM.

admin-settings-key-time-hour-cycle = Orologio
admin-settings-help-time-hour-cycle = I simboli H o h di un modello personalizzato hanno precedenza.

admin-settings-key-time-first-day-of-week = Primo giorno della settimana
admin-settings-help-time-first-day-of-week = Automatico segue le convenzioni regionali del calendario.

admin-settings-key-compliance-mail-enabled = Attiva email
admin-settings-help-compliance-mail-enabled = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-host = Host del relay SMTP
admin-settings-help-compliance-mail-host = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-port = Porta SMTP
admin-settings-help-compliance-mail-port = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-security = Modalità TLS
admin-settings-help-compliance-mail-security = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-from = Indirizzo del mittente
admin-settings-help-compliance-mail-from = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-username = Nome utente SMTP
admin-settings-help-compliance-mail-username = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-allowPrivate = Consenti relay privato
admin-settings-help-compliance-mail-allowPrivate = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-dsr-notify-emailAdmins = Notifica agli operatori le richieste di dati
admin-settings-help-dsr-notify-emailAdmins = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-dsr-notify-operatorAddresses = Destinatari operatori
admin-settings-help-dsr-notify-operatorAddresses = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-dsr-notify-emailRequester = Notifica il richiedente via email
admin-settings-help-dsr-notify-emailRequester = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-breach-notify-email = Notifica gli operatori delle violazioni
admin-settings-help-breach-notify-email = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-breach-subjects-email = Notifica le persone interessate via email
admin-settings-help-breach-subjects-email = Configura un relay SMTP con TLS. Le email usano il percorso proxy system_mail. Verifica le accettazioni incerte prima di reinviare.

admin-settings-key-compliance-mail-requireProxy = Richiedi percorso proxy
admin-settings-help-compliance-mail-requireProxy = Richiedi percorso proxy (system_mail).
admin-settings-key-retention-complianceMail-days = Conservazione delle email completate (giorni)
admin-settings-help-retention-complianceMail-days = Giorni di conservazione delle consegne completate. Quelle in attesa o incerte rimangono fino alla risoluzione.

admin-settings-key-incidents-auto-enabled = Rileva automaticamente potenziali incidenti
admin-settings-help-incidents-auto-enabled = I controlli falliti e gli avvisi di sessione creano potenziali incidenti. Valuta la rilevanza e verifica il ripristino. Le lacune di monitoraggio non provano un'interruzione.
admin-settings-key-incidents-auto-failureSamples = Controlli falliti prima del rilevamento
admin-settings-help-incidents-auto-failureSamples = I controlli falliti e gli avvisi di sessione creano potenziali incidenti. Valuta la rilevanza e verifica il ripristino. Le lacune di monitoraggio non provano un'interruzione.
admin-settings-key-incidents-nis2-trustService = Usa termine per servizi fiduciari
admin-settings-help-incidents-nis2-trustService = I controlli falliti e gli avvisi di sessione creano potenziali incidenti. Valuta la rilevanza e verifica il ripristino. Le lacune di monitoraggio non provano un'interruzione.
admin-settings-key-incidents-notify-email = Invia aggiornamenti e promemoria via email
admin-settings-help-incidents-notify-email = I controlli falliti e gli avvisi di sessione creano potenziali incidenti. Valuta la rilevanza e verifica il ripristino. Le lacune di monitoraggio non provano un'interruzione.
admin-settings-key-incidents-notify-operatorAddresses = Destinatari delle operazioni
admin-settings-help-incidents-notify-operatorAddresses = I controlli falliti e gli avvisi di sessione creano potenziali incidenti. Valuta la rilevanza e verifica il ripristino. Le lacune di monitoraggio non provano un'interruzione.
admin-settings-key-incidents-retention-days = Conservazione incidenti chiusi (giorni)
admin-settings-help-incidents-retention-days = I controlli falliti e gli avvisi di sessione creano potenziali incidenti. Valuta la rilevanza e verifica il ripristino. Le lacune di monitoraggio non provano un'interruzione.

admin-settings-key-mail-snooze-presets = Opzioni di rinvio
admin-settings-help-mail-snooze-presets = Aggiungi, rinomina, riordina, modifica o nascondi le opzioni. Il menu consente sempre di scegliere data e ora.

admin-settings-key-reader-actions-view-draft = Visualizza bozza di risposta
admin-settings-help-reader-actions-view-draft = Visualizza bozza di risposta

admin-settings-key-notifications-view-enabled = Abilita la vista Notifiche
admin-settings-help-notifications-view-enabled = Controlla il centro notifiche. Gli avvisi di sicurezza, di posta e del browser mantengono le proprie impostazioni.
admin-settings-key-notifications-view-show-in-sidebar = Mostra Notifiche nella barra laterale
admin-settings-help-notifications-view-show-in-sidebar = Controlla il centro notifiche. Gli avvisi di sicurezza, di posta e del browser mantengono le proprie impostazioni.

admin-settings-key-mail-unified-account = Account virtuale unificato
admin-settings-help-mail-unified-account = Aggiungi un account virtuale al selettore degli account, con cartelle e contatori combinati. Indipendente dalla vista unificata dei messaggi.

admin-settings-key-accounts-settings-inherit-groups = Eredita preferenze dell’account principale
admin-settings-help-accounts-settings-inherit-groups = I gruppi selezionati usano i valori dell’account principale senza una modifica locale. Scelte locali e criteri amministrativi hanno priorità. Temi, firme, credenziali e ID delle cartelle non vengono ereditati.

admin-settings-key-mail-badge-importance = Importanza
admin-settings-help-mail-badge-importance = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-follow-up = Completamento
admin-settings-help-mail-badge-follow-up = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-snooze = Posticipa
admin-settings-help-mail-badge-snooze = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-receipts = Conferme di lettura
admin-settings-help-mail-badge-receipts = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-delivery = Stato di consegna
admin-settings-help-mail-badge-delivery = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-dates = Date del messaggio
admin-settings-help-mail-badge-dates = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-sensitivity = Riservatezza
admin-settings-help-mail-badge-sensitivity = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-reply-by = Scadenza della risposta
admin-settings-help-mail-badge-reply-by = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-expires = Scadenza
admin-settings-help-mail-badge-expires = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.
admin-settings-key-mail-badge-supersedes = Messaggi sostituiti
admin-settings-help-mail-badge-supersedes = Scegli sempre, solo se rilevante o mai. Evidenzia valori insoliti, promemoria attivi, conferme, consegne o scadenze vicine. I dati del messaggio vengono conservati.

admin-settings-key-mail-follow-up-presets = Contrassegna per completamento
admin-settings-help-mail-follow-up-presets = Scegli i promemoria di completamento indipendentemente dal rinvio. I contrassegni mantengono visibili i messaggi; il rinvio li nasconde fino all’ora scelta. I valori locali possono sostituire quelli ereditati se consentito dall’amministratore. Solo Più tardi oggi, Domani e Settimana prossima sono attivi per impostazione predefinita. Le altre opzioni sono già disponibili da attivare o personalizzare.
