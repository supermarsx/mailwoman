# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Annulla
common-close = Chiudi
common-save = Salva
common-delete = Elimina
common-remove = Rimuovi
common-edit = Modifica
common-add = Aggiungi
common-back = Indietro
common-next = Avanti
common-done = Fatto
common-apply = Applica
common-confirm = Conferma
common-retry = Riprova
common-contact-administrator = Se il problema persiste, contattare l'amministratore.
common-shell-not-configured = Questa app non è stata configurata per un server. Contattare l'amministratore.
common-copy = Copia
common-search = Cerca
common-more = Altro
common-yes = Sì
common-no = No

# -- Generic states ----------------------------------------------------------
common-loading = Caricamento…
common-saving = Salvataggio…
common-empty = Ancora nessun elemento
common-error = Si è verificato un errore
common-error-network = Impossibile raggiungere il server. Verificare la connessione e riprovare.
common-offline = Offline

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
common-conn-connecting = Connessione…
common-conn-degraded = Aggiornamenti in tempo reale sospesi — riconnessione in corso
common-conn-offline = Offline — le modifiche verranno sincronizzate alla riconnessione
common-conn-auth-expired = Sessione scaduta — accedere di nuovo
common-conn-reconnect = Riconnetti
common-conn-sign-in-again = Accedi di nuovo
common-conn-reconnected = Riconnesso

# -- New-mail notification (App.tsx). The native desktop or mobile notification
common-notify-new-message = Nuovo messaggio
common-notify-no-subject = (nessun oggetto)

# -- Generic labels ----------------------------------------------------------
common-required = Obbligatorio
common-optional = Facoltativo

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
common-nav-open = Apri cartelle e app
common-nav-close = Chiudi cartelle e app

# -- Attachments (global, cross-account view; rides the entry bundle so the
common-attach-title = Allegati
common-attach-search = Cerca allegati
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Filtra per tipo
common-attach-loading = Caricamento degli allegati…
common-attach-empty = Nessun allegato corrispondente.
common-attach-cat-all = tutti
common-attach-cat-image = immagine
common-attach-cat-pdf = pdf
common-attach-cat-text = testo
common-attach-cat-audio = audio
common-attach-cat-video = video
common-attach-cat-other = altro

# -- Password fields (components/PasswordInput.tsx): the button at the end of
common-password-show = Mostra password
common-password-hide = Nascondi password

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
common-module-mail = Posta
common-module-calendar = Calendario
common-module-tasks = Attività
common-module-notes = Note
common-module-contacts = Contatti
common-module-keys = Chiavi

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
common-viewer-loading = Caricamento del visualizzatore…
common-viewer-loading-audio = Caricamento dell'audio…
common-viewer-loading-image = Caricamento dell'immagine…
common-viewer-loading-video = Caricamento del video…
common-viewer-rendering-pdf = Rendering del PDF…
common-viewer-pdf-failed = Impossibile visualizzare questo PDF.
common-viewer-no-preview = Nessuna anteprima disponibile per questo tipo di file.
common-viewer-unknown-type = tipo sconosciuto
common-viewer-download = Scarica
common-viewer-attachments = Allegati

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Nuovo tentativo…
common-waiting-online = In attesa di una connessione…
common-failure-offline = Risulta offline. Riconnettiti e riprova.
common-failure-network = Impossibile raggiungere il server.
common-failure-server = Il server non ha risposto in tempo.
common-failure-rate = Troppe richieste in poco tempo. Attendi un momento e riprova.
common-failure-conflict = Questo elemento è stato modificato altrove nello stesso momento. Ricarica e riprova.
common-failure-account = L'account con cui ha effettuato l'accesso è cambiato. Ricarica la pagina.
common-failure-linked = Non disponibile mentre è aperta una casella collegata.
common-failure-refused = Il server ha rifiutato la modifica.
common-new-version = È disponibile una nuova versione di Mailwoman. Chiudendo questo avviso, verrà ricordato di nuovo tra un'ora.
common-new-version-reload = Ricarica ora
common-toast-close = Chiudi notifica
common-toast-seconds = { $seconds } s
common-attach-refresh = Aggiorna allegati
common-attach-open-message = Apri il messaggio che contiene questo allegato
common-attach-page-filter-note = I filtri si applicano a questa pagina. Usa Avanti per sfogliare i messaggi più vecchi con allegati.
common-attach-pages = Pagine degli allegati
common-attach-previous = Precedente
common-attach-next = Avanti
common-attach-page = Pagina { $page }
common-attach-column-name = File
common-attach-column-type = Tipo
common-attach-column-size = Dimensione
common-attach-column-sender = Mittente
common-attach-column-subject = Oggetto
common-attach-column-date = Ricevuto
common-pdf-preview = Anteprima PDF
common-pdf-link-title = Aprire il link di questo PDF?
common-pdf-link-help = La destinazione verrà aperta in una nuova scheda, fuori da Mailwoman.
common-pdf-link-open = Apri link
common-open-event = Apri evento
common-message-preview-too-large = Le anteprime degli allegati dei messaggi sono limitate a 4 MiB. Può comunque scaricare l'allegato originale.
common-message-preview-failed = Non è stato possibile mostrare l'anteprima di questo allegato. Il file potrebbe non essere supportato o essere danneggiato, oppure il servizio di anteprima non è disponibile. Può comunque scaricare l'originale.
common-message-preview-attachments = File contenuti in questo messaggio
common-message-preview-download = Scarica file
common-message-preview-size-bytes = { $count } byte
common-attach-cat-message = Messaggi / allegati Outlook
common-error-retry-help = Questa sezione non è temporaneamente disponibile. Riprova per ricaricarla.
common-error-network-help = Controlla la connessione e riprova. Il server potrebbe essere temporaneamente non disponibile.
common-error-page = Impossibile caricare questa pagina
common-error-page-help = Non è stato possibile scaricare una risorsa della pagina. Riprova. Se il problema persiste, ricarica l'applicazione per ottenere l'ultima versione.
common-reload-app = Ricarica applicazione
common-new-version-reloading = È disponibile una nuova versione, ricaricamento in corso…
common-attach-account-changed = L'account è cambiato durante il caricamento degli allegati. Riapri Allegati per l'account corrente.
common-attach-search-account = Cerca in tutto l'account
common-attach-scanning = Ricerca in corso… { $pages } pagine di messaggi controllate · { $count } file trovati
common-attach-scan-cancelled = Ricerca annullata · { $count } file trovati in { $pages } pagine. I risultati sono incompleti.
common-attach-scan-incomplete = Ricerca interrotta · { $count } file trovati. Riprova per cercare in tutto l'account.
common-attach-scan-limited = Vengono mostrati i primi 1.000 file corrispondenti. Restringi i filtri per trovare file più specifici.
common-attach-scan-complete = Ricerca completata · { $count } file trovati in { $pages } pagine di messaggi
common-attach-browse-pages = Sfoglia pagine
common-attach-cancel-search = Annulla ricerca
multiselect-selected = Selezionato: { $label }
multiselect-remove = Rimuovi { $label }
multiselect-no-match = Nessuna opzione corrispondente
common-notification-permission-title = Consentire le notifiche?
common-notification-permission-help = Ricevi notifiche sui nuovi messaggi su questo dispositivo. Può modificarlo in Impostazioni.
common-notification-permission-allow = Consenti notifiche
common-notification-permission-later = Non ora
common-notification-permission-error = Non è stato possibile attivare le notifiche. Controlla i permessi e le impostazioni delle notifiche, quindi riprova.
common-notifications-enabled = Notifiche attivate su questo dispositivo.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [one] 1 selezionato
   *[other] { $count } selezionati
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = Il server è occupato. Attendere qualche secondo e riprovare.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = Nessuna corrispondenza

combobox-toggle = Mostra opzioni per { $label }

common-attach-cat-archive = Archivi

common-attach-cat-document = Documenti

common-attach-cat-spreadsheet = Fogli di calcolo

common-conn-reconnecting = Riconnessione…

common-viewer-pdf-details = Dettagli tecnici

common-viewer-pdf-details-help = Includi queste righe quando segnali il problema. Non contengono alcun contenuto di documenti.

timezone-group-other = Altro

timezone-use-browser = Utilizza il fuso orario del browser ({ $zone })
