# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Impostazioni
mail-compose = Scrivi
mail-nav-mailboxes = Caselle
mail-nav-attachments = Allegati
mail-nav-outbox = Posta in uscita
mail-nav-sharing = Condividi cartella
mail-nav-apps = App
mail-logout = Esci
mail-offline = Offline
mail-module-loading = Caricamento di { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Cerca
mail-search-label = Cerca nella posta
mail-search-placeholder = Cerca nella posta — from:alice subject:fattura larger:1mb
mail-search-clear = Cancella

# -- Message list ------------------------------------------------------------
mail-list-label = Messaggi
mail-loading = Caricamento dei messaggi…
mail-empty = Nessun messaggio
mail-unknown-sender = (mittente sconosciuto)
mail-no-subject = (nessun oggetto)
mail-pinned = Fissato
mail-has-attachment = Con allegato
mail-answered = Risposto
mail-forwarded = Inoltrato
mail-unread = Non letto
mail-flagged = Contrassegnato
mail-selected = Selezionato
mail-row-position = Messaggio { $pos } di { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } messaggio
   *[other] { $count } messaggi
}
mail-thread-expand = Espandi conversazione, { $count ->
    [one] { $count } messaggio
   *[other] { $count } messaggi
}
mail-thread-collapse = Comprimi conversazione, { $count ->
    [one] { $count } messaggio
   *[other] { $count } messaggi
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Opzioni di visualizzazione
mail-reading-pane = Riquadro di lettura
mail-reading-pane-right = A destra
mail-reading-pane-bottom = In basso
mail-reading-pane-off = Disattivato

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Messaggi selezionati
mail-bulk-count = { $count } selezionati
mail-mark-read = Segna come letto
mail-mark-unread = Segna come non letto
mail-flag-message = Contrassegna
mail-unflag-message = Rimuovi contrassegno
mail-move-to = Sposta nella cartella
mail-move-to-placeholder = Sposta in…
mail-select-all = Seleziona tutto
mail-clear-selection = Annulla selezione

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = Filtro posta in arrivo
mail-inbox-focused = Evidenziata
mail-inbox-other = Altro

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Schede aperte
mail-subtab-pin = Fissa { $title }
mail-subtab-unpin = Sblocca { $title }
mail-subtab-tearoff = Apri { $title } in una nuova finestra
mail-subtab-close = Chiudi { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Fissa
mail-unpin = Sblocca
mail-snooze = Posticipa
mail-snooze-menu = Posticipa fino a
mail-snooze-later = Più tardi oggi
mail-snooze-tomorrow = Domani
mail-snooze-next-week = Settimana prossima
mail-unsnooze = Annulla posticipo
mail-label = Etichetta
mail-labels-menu = Etichette
mail-flag = Contrassegna per completamento
mail-clear-flag = Rimuovi completamento
mail-archive = Archivia
mail-delete = Elimina
mail-more-actions = Altre azioni

# -- Reader ------------------------------------------------------------------
mail-reader-label = Messaggio
mail-reader-empty = Selezionare un messaggio da leggere
mail-back = Indietro
mail-reader-from = Da: { $addr }
mail-reader-to = A: { $addr }
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Azioni sul messaggio
mail-reply = Rispondi
mail-reply-all = Rispondi a tutti
mail-forward = Inoltra
mail-reply-failed = Impossibile preparare il messaggio. Riprovare.
mail-reply-unthreaded = Impossibile leggere l'ID del messaggio originale. Questa risposta non sarà collegata alla sua conversazione.
mail-spam = Spam
mail-export = Esporta
mail-sweep-sender = Pulisci mittente
mail-message-body = Corpo del messaggio
mail-no-content = Nessun contenuto
mail-sanitizing = Sanificazione…
mail-attachments = Allegati
mail-attachment-unnamed = (senza nome)
mail-attachment-open = Allegato { $name }
mail-attachment-close = Chiudi allegato
mail-attachment-loading = Caricamento dell'allegato…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Messaggio cifrato
mail-encrypted-note = Questo messaggio è cifrato end-to-end. Sbloccarlo su questo dispositivo per leggerlo.
mail-key-passphrase = Passphrase della chiave
mail-decrypt = Decifra
mail-decrypting = Decifratura…
mail-decrypt-no-key = Nessuna chiave privata disponibile per decifrare questo messaggio.
mail-decrypt-failed = Decifratura non riuscita

# -- Reply and forward: text written into the outgoing message --------------
mail-quote-attribution = Il giorno { $date }, { $name } ha scritto:
mail-quote-attribution-undated = { $name } ha scritto:
mail-forward-heading = ---------- Messaggio inoltrato ----------
mail-forward-from = Da: { $addr }
mail-forward-date = Data: { $date }
mail-forward-subject = Oggetto: { $subject }
mail-forward-to = A: { $addr }
mail-forward-cc = Cc: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Scrivi messaggio
mail-compose-title = Nuovo messaggio
mail-compose-title-reply = Rispondi
mail-compose-title-reply-all = Rispondi a tutti
mail-compose-title-forward = Inoltra
mail-compose-close = Chiudi
mail-compose-from = Da
mail-compose-from-default = Predefinito
mail-compose-to = A
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Contatti suggeriti
mail-compose-cc = Cc
mail-compose-bcc = Ccn
mail-compose-show-cc-bcc = Aggiungi Cc o Ccn
mail-compose-bcc-encrypted = Un messaggio cifrato elenca la chiave di ogni destinatario, quindi un destinatario in Ccn sarebbe visibile agli altri. Rimuovere i destinatari in Ccn o disattivare la cifratura.
mail-compose-plaintext-confirm-title = Inviare il testo decifrato senza cifratura?
mail-compose-plaintext-confirm-body = Questo messaggio cita testo da un messaggio cifrato. Non è impostato per essere cifrato, quindi il testo citato verrebbe inviato senza protezione.
mail-compose-plaintext-confirm-send = Invia non cifrato
mail-compose-plaintext-confirm-cancel = Torna indietro
mail-compose-subject = Oggetto
mail-compose-body = Testo
mail-compose-attachments = Allegati
mail-compose-remove-attachment = Rimuovi { $name }
mail-compose-attach-nextcloud = Allega da Nextcloud
mail-compose-close-nextcloud = Chiudi Nextcloud
mail-compose-attach-file = Allega un file
mail-compose-uploading = Caricamento…
mail-compose-upload-unavailable = Il caricamento dei file non è disponibile al momento.
mail-compose-upload-failed = Impossibile caricare { $name }.
mail-compose-upload-too-large = { $name } è di { $size } MB. La dimensione massima di caricamento è { $max } MB.
mail-compose-send-later = Invia più tardi
mail-compose-cancel = Annulla
mail-compose-send = Invia
mail-compose-schedule = Pianifica
mail-compose-sending = Invio…
mail-compose-encrypted-subject = Messaggio cifrato
mail-compose-dlp-blocked = L'invio è bloccato da una regola di prevenzione della perdita di dati (vedere l'avviso sopra).
mail-compose-send-failed = Invio non riuscito
mail-compose-send-later-past = Scegliere un orario di invio nel futuro, oppure svuotare il campo per inviare subito.
mail-compose-options = Opzioni di invio
mail-compose-request-receipt = Richiedi una conferma di lettura
mail-compose-request-receipt-hint = Il programma di posta del destinatario può chiedere al destinatario, inviarne una senza chiedere oppure ignorare la richiesta.
mail-compose-track-opens = Registra quando questo messaggio viene aperto
mail-compose-track-opens-hint = Al messaggio viene aggiunta un'immagine 1×1 caricata da questo server. Molti programmi di posta bloccano le immagini o le caricano in anticipo, quindi un'apertura registrata non prova che il messaggio sia stato letto, e l'assenza di registrazione non prova che non lo sia stato. Il destinatario non viene avvisato.
mail-compose-image-type = Impossibile inserire { $name } nel testo. Usare un'immagine PNG, JPEG, GIF o WebP.
mail-send-not-signed-in = Accesso non effettuato.
mail-send-no-draft-folder = Questo account non ha una cartella in cui salvare la bozza.
mail-send-no-recipient = Aggiungere almeno un destinatario.
mail-send-bad-recipient = { $count ->
    [one] Questo non è un indirizzo email: { $addresses }
   *[other] Questi non sono indirizzi email: { $addresses }
}
mail-send-later-past = L'orario di invio non è nel futuro. Non è stato inviato nulla.
mail-send-tracking-failed = Al momento non è possibile registrare le aperture, quindi non è stato inviato nulla. Deselezionare "Registra quando questo messaggio viene aperto" per inviare senza.
mail-send-draft-refused = Il server ha rifiutato questo messaggio: { $reason }. Non è stato inviato nulla.
mail-send-submission-refused = Il server ha rifiutato di inviare questo messaggio: { $reason }. Non è stato inviato nulla.
mail-compose-sign-unlock-title = Sblocca chiave di firma
mail-compose-sign-unlock-note = Inserire la passphrase della chiave per firmare questo messaggio. Resta sbloccata solo per questa finestra di composizione.
mail-compose-sign-unlock = Sblocca
mail-compose-sign-unlocking = Sblocco…
mail-compose-sign-no-key = Nessuna chiave di firma disponibile su questo dispositivo.
mail-compose-sign-unlock-failed = Impossibile sbloccare la chiave di firma.
mail-compose-sign-unlock-required = Sbloccare la chiave di firma per inviare questo messaggio firmato.

mail-compose-format-plain = Testo semplice
mail-compose-format-rich = Testo formattato
mail-compose-rt-toolbar = Formattazione
mail-compose-rt-bold = Grassetto
mail-compose-rt-italic = Corsivo
mail-compose-rt-underline = Sottolineato
mail-compose-rt-strike = Barrato
mail-compose-rt-bullet = Elenco puntato
mail-compose-rt-ordered = Elenco numerato
mail-compose-rt-quote = Citazione
mail-compose-rt-link = Link
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Applica
mail-compose-rt-image = Inserisci un'immagine

mail-compose-signature = Firma
mail-compose-signature-none = Aggiungi una firma…

mail-compose-recall = Richiama
mail-compose-recall-action = Richiama
mail-compose-recall-holding = In attesa di invio
mail-compose-recall-scheduled = Pianificato per { $when }

mail-compose-drafts = Bozze
mail-compose-drafts-empty = Nessuna bozza salvata
mail-compose-drafts-delete = Elimina bozza
mail-compose-drafts-no-recipient = (nessun destinatario)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Posta in uscita
mail-outbox-refresh = Aggiorna
mail-outbox-empty = Nessun messaggio in attesa di invio
mail-outbox-send-now = Invia ora
mail-outbox-cancel = Annulla
mail-outbox-scheduled = Pianificato
mail-outbox-holding = Invio a breve
mail-outbox-sent = Inviato
mail-outbox-canceled = Annullato
mail-outbox-held = Trattenuto
mail-outbox-failed = Non inviato
mail-outbox-release = Rilascia
mail-outbox-discard = Scarta
mail-outbox-held-note = Trattenuto. Non viene inviato finché non è rilasciato.
mail-outbox-held-note-api-key = Trattenuto — creato dalla chiave API { $name }. Non viene inviato finché non è rilasciato.
mail-outbox-held-note-oauth = Trattenuto — creato dall'app collegata { $name }. Non viene inviato finché non è rilasciato.
mail-outbox-origin-api-key = Creato dalla chiave API { $name }
mail-outbox-origin-oauth = Creato dall'app collegata { $name }
mail-outbox-to = A { $recipients }
mail-outbox-no-subject = (nessun oggetto)
mail-outbox-attempts = Tentativi: { $count }. Ultimo errore: { $error }
mail-outbox-error = Errore: { $error }
mail-outbox-toast-canceled = Invio annullato
mail-outbox-toast-discarded = Scartato. Il messaggio non è stato inviato.
mail-outbox-toast-sent = Inviato
mail-outbox-toast-retrying = Non ancora inviato: { $error }. Verrà ritentato.
mail-outbox-toast-failed = Non inviato: { $error }
mail-outbox-toast-not-released = Impossibile inviare: { $error }
mail-outbox-toast-not-canceled = Impossibile annullare: { $error }

# -- Network status and the offline queue ------------------------------------
mail-net-lost = Connessione persa — nuovo tentativo…
mail-net-back = Di nuovo online
mail-queue-sent = { $count ->
    [one] Inviata { $count } modifica in coda
   *[other] Inviate { $count } modifiche in coda
}
mail-queue-failed = { $count ->
    [one] { $count } modifica in coda non riuscita
   *[other] { $count } modifiche in coda non riuscite
}
mail-queue-notice-label = Modifiche che non è stato possibile applicare
mail-queue-notice-title = { $count ->
    [one] Non è stato possibile applicare { $count } modifica fatta offline
   *[other] Non è stato possibile applicare { $count } modifiche fatte offline
}
mail-queue-kind-send = Invia “{ $subject }”
mail-queue-kind-send-untitled = Invia un messaggio senza oggetto
mail-queue-kind-move = Sposta un messaggio
mail-queue-kind-flag = Modifica un'etichetta o un contrassegno
mail-queue-kind-draft = Salva una bozza
mail-queue-kind-pim = Modifica a calendario, attività, nota o contatto
mail-queue-reason = Motivo: { $reason }
mail-queue-reason-unknown = Il server non ha indicato il motivo.
mail-queue-retry = Riprova
mail-queue-discard = Scarta
mail-queue-retry-item = Riprova: { $what }
mail-queue-discard-item = Scarta: { $what }
mail-queue-discarded = Scartata. La modifica non è stata applicata.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Pulisci messaggi
mail-sweep-title = Pulisci { $sender }
mail-sweep-all = Elimina tutti i messaggi di questo mittente
mail-sweep-keep-latest = Conserva il più recente, elimina gli altri
mail-sweep-older-than = Elimina quelli più vecchi di N giorni
mail-sweep-block = Elimina tutti e blocca questo mittente
mail-sweep-days = Giorni
mail-sweep-count = { $count ->
    [one] { $count } messaggio verrà spostato nel Cestino
   *[other] { $count } messaggi verranno spostati nel Cestino
}
mail-sweep-cancel = Annulla
mail-sweep-run = Pulisci { $count }
mail-sweeping = Pulizia…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Chiudi
mail-undo-action = Annulla
mail-undo-send-cancel = Annulla
mail-undo-failed = Impossibile annullare

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Etichetta aggiunta
mail-toast-label-removed = Etichetta rimossa
mail-toast-pinned = Messaggio “{ $target }” fissato
mail-toast-unpinned = Messaggio “{ $target }” sbloccato
mail-toast-snoozed = Posticipato
mail-toast-follow-up-set = Completamento impostato
mail-toast-follow-up-cleared = Completamento rimosso
mail-toast-archived = Archiviato
mail-toast-trashed = Spostato nel Cestino
mail-toast-marked-spam = Segnato come spam
mail-toast-moved-to = Spostato in { $folder }
mail-toast-moved = Spostato nella cartella
mail-toast-archived-count = { $count } messaggi archiviati
mail-toast-trashed-count = { $count } messaggi spostati nel Cestino
mail-toast-marked-spam-count = { $count } messaggi segnati come spam
mail-toast-moved-count = { $count } messaggi spostati
mail-toast-moved-to-count = { $count } messaggi spostati in { $folder }
mail-toast-marked-read = { $count ->
    [one] Segnato come letto
   *[other] { $count } messaggi segnati come letti
}
mail-toast-marked-unread = { $count ->
    [one] Segnato come non letto
   *[other] { $count } messaggi segnati come non letti
}
mail-toast-flagged = { $count ->
    [one] Contrassegnato
   *[other] { $count } messaggi contrassegnati
}
mail-toast-unflagged = { $count ->
    [one] Contrassegno rimosso
   *[other] Contrassegno rimosso da { $count } messaggi
}
mail-toast-move-failed = Impossibile spostare i messaggi. Non è stato spostato nulla.
mail-toast-update-failed = Impossibile modificare i messaggi. Non è stato modificato nulla.
mail-toast-update-refused = { $count ->
    [one] Il server ha rifiutato la modifica per un messaggio
   *[other] Il server ha rifiutato la modifica per { $count } messaggi
}
mail-toast-no-archive-folder = Nessuna cartella Archivio
mail-toast-no-trash-folder = Nessuna cartella Cestino
mail-toast-no-spam-folder = Nessuna cartella Spam
mail-toast-sweep-nothing = Niente da pulire
mail-toast-swept = { $count ->
    [one] { $count } messaggio rimosso
   *[other] { $count } messaggi rimossi
}
mail-toast-load-more-failed = Impossibile caricare altri messaggi
mail-toast-export-nothing = Niente da esportare
mail-toast-exported = .eml esportato
mail-toast-export-failed = Esportazione non riuscita
mail-toast-send-queued = In coda — verrà inviato al ritorno online
mail-toast-tracking-not-bound = Il messaggio è stato inviato, ma le sue aperture non verranno registrate.
mail-toast-send-scheduled = Invio pianificato
mail-toast-sent = Messaggio inviato
mail-toast-send-canceled = Invio annullato

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Barra multifunzione
mail-ribbon-tab-home = Home
mail-ribbon-tab-view = Visualizza
mail-ribbon-tab-folder = Cartella
mail-ribbon-expand = Espandi
mail-ribbon-collapse = Comprimi
mail-ribbon-group-new = Nuovo
mail-ribbon-compose = Scrivi
mail-ribbon-group-session = Sessione
mail-ribbon-logout = Esci
mail-ribbon-group-theme = Tema
mail-ribbon-group-density = Densità
mail-ribbon-group-settings = Impostazioni
mail-ribbon-more = Altro…
mail-ribbon-group-folders = Cartelle
mail-density-compact = compatta
mail-density-cozy = comoda
mail-density-relaxed = ampia

# -- Message security level: the three-position switch in the reader toolbar
mail-maxsec-label = Livello di sicurezza del messaggio
mail-maxsec-full = Completo
mail-maxsec-full-hint = HTML sanificato con immagini e stili.
mail-maxsec-no-media = Senza media
mail-maxsec-no-media-hint = HTML sanificato con tutte le immagini e i media rimossi.
mail-maxsec-plain-text = Testo semplice
mail-maxsec-plain-text-hint = Nessun HTML visualizzato — messaggio mostrato solo come testo semplice.
mail-feature-off = Disattivato dalle impostazioni o dall'amministratore
mail-send-disabled = L'invio è disattivato dalle impostazioni o dall'amministratore. Può comunque salvare una bozza.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Eventi
mail-events-upcoming = Solo prossimi
mail-events-filter = Filtro eventi
mail-loading-more = Caricamento dei messaggi meno recenti…
mail-event-state-invitation = Invito
mail-event-state-updated = Aggiornato
mail-event-state-cancelled = Annullato
mail-event-state-reply = Risposta
mail-event-state-counter = Nuovo orario proposto
mail-event-state-event = Evento
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Mostra altre azioni
mail-remote-moved = Questo messaggio è stato spostato in { $folder } da un altro dispositivo o app. Si sta ancora leggendo la copia aperta.
mail-remote-moved-unknown = Questo messaggio è stato spostato da un altro dispositivo o app. Si sta ancora leggendo la copia aperta.
mail-remote-deleted = Questo messaggio è stato eliminato da un altro dispositivo o app. È possibile continuare a leggere questa copia finché non viene chiusa.
mail-remote-dismiss = Ignora
mail-reply-draft = Risposta in corso
mail-reply-draft-open = Apri bozza
mail-draft-load-failed = Impossibile aprire la bozza. Riprovare.
mail-reply-images-omitted =
    { $count ->
        [one] Non è stato possibile includere un'immagine nella risposta.
       *[other] Non è stato possibile includere { $count } immagini nella risposta.
}
mail-export-forbidden = L'amministratore ha disattivato l'esportazione dei messaggi per questo account.
mail-export-open-failed = Impossibile aprire le opzioni di esportazione. Chiudere una scheda dell'area di lavoro e riprovare.
mail-headers-title = Intestazioni grezze del messaggio
mail-headers-action-help = Visualizza le intestazioni originali del messaggio, compresi i dettagli di consegna e autenticazione.
mail-headers-help = Intestazioni originali del messaggio. Le righe ripiegate vengono conservate; qui sono mostrate solo le intestazioni.
mail-headers-loading = Caricamento delle intestazioni del messaggio…
mail-headers-failed = Impossibile caricare le intestazioni del messaggio. Controllare la connessione e riprovare.
mail-headers-copy = Copia intestazioni
mail-headers-copied = Intestazioni copiate.
mail-headers-copy-failed = Impossibile copiare le intestazioni. Selezionare il testo per copiarlo manualmente.
mail-attachment-failed = Impossibile scaricare questo allegato. Riprovare quando la connessione sarà disponibile.
mail-compose-rt-rule = Linea orizzontale
mail-compose-signature-new = Nuovi messaggi
mail-compose-signature-replies = Risposte
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = Dimensione del corpo
mail-editor-body-size-help = Dimensione del corpo attuale in UTF-8, compresi il markup HTML e le immagini incorporate. Le intestazioni del messaggio e gli allegati separati non sono inclusi.
mail-outbox-load-failed = Impossibile controllare la Posta in uscita. Provare Aggiorna dopo aver controllato la connessione.
mail-outbox-unavailable = Posta in uscita non disponibile. Usare Aggiorna per riprovare.
mail-maxsec-view = Vista: { $mode }
mail-maxsec-menu = Modalità di visualizzazione
mail-toast-tab-pinned = Scheda “{ $target }” fissata
mail-toast-tab-unpinned = Scheda “{ $target }” sbloccata
mail-workspace-message = Messaggio
mail-opened-announcement = Aperto: { $subject }
mail-workspace-layout = Layout dell'area di lavoro
mail-workspace-tabs = Schede
mail-workspace-side-by-side = Affiancati
mail-workspace-stack = Impilati
mail-workspace-refresh = Aggiorna cartella
mail-workspace-load-error = Impossibile caricare questo messaggio. Chiudere la scheda e riprovare.
mail-resize-folders = Ridimensiona la barra laterale delle cartelle
mail-resize-list = Ridimensiona l'elenco dei messaggi
mail-resize-tiles = Ridimensiona i riquadri dell'area di lavoro
mail-show-folders = Mostra la barra laterale delle cartelle
mail-hide-folders = Nascondi la barra laterale delle cartelle
mail-compact-folders = Icone delle cartelle compatte
mail-toggle-list = Mostra o nascondi l'elenco dei messaggi
mail-toggle-reader = Mostra o nascondi il riquadro di lettura
mail-folder-counts = { $unread } non letti, { $total } messaggi
mail-print = Stampa / Salva come PDF
mail-go-to-top = Vai in alto
mail-print-help = Stampare questo messaggio o scegliere Salva come PDF nella finestra di stampa. Le immagini esterne restano bloccate.
mail-file-details = Dettagli e impronta del file
mail-file-type = Tipo di file
mail-file-extension = Estensione
mail-file-detected = Firma rilevata
mail-file-unknown = Nessuna firma riconosciuta
mail-file-size = Dimensione
mail-file-size-bytes = { $count } byte
mail-file-hash-unavailable = Impronta non disponibile (richiede un browser sicuro e un file fino a 64 MiB).
mail-file-fingerprint-help = Un'impronta identifica esattamente questi byte. Non dimostra che il file sia sicuro né chi lo abbia creato.
mail-file-warning-executable = Questo file può eseguire programmi o modificare le impostazioni di sistema. Scaricarlo solo se si ritiene attendibile la fonte.
mail-file-warning-macro = Questo documento può contenere macro. Non abilitare le macro senza aver verificato mittente e scopo.
mail-file-warning-disguisedName = Il nome del file può nascondere la sua vera estensione. Controllare i dettagli del file prima di scaricarlo.
mail-file-warning-typeMismatch = La firma rilevata del file non corrisponde al tipo dichiarato. L'anteprima automatica è stata bloccata.
mail-file-warning-activeDocument = Questo formato può contenere contenuti attivi. L'anteprima è isolata; aprire una copia scaricata può comportarsi in modo diverso.
mail-file-preview-blocked = L'anteprima automatica è disattivata per questo tipo di file o in caso di tipo non corrispondente.
mail-quote-header-from = Da:
mail-quote-header-sent = Inviato:
mail-quote-header-to = A:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Oggetto:
mail-spam-score = Analisi dello spam
mail-spam-score-loading = Controllo delle intestazioni di spam…
mail-spam-score-unavailable = Punteggio spam non disponibile
mail-spam-score-unverified = Riportato dalle intestazioni del messaggio, non verificato in modo indipendente. Le intestazioni fornite dal mittente possono essere contraffatte.
mail-spam-score-positive = L'intestazione segnala spam.
mail-spam-score-negative = L'intestazione segnala un risultato non spam.
mail-spam-score-unknown = Con questo punteggio non è stato fornito alcun verdetto.
mail-external-recipients = Destinatari esterni
mail-external-count = { $count } destinatario/i fuori dai domini attendibili
mail-external-help = Controllare i destinatari in A, Cc e Ccn prima di inviare informazioni riservate. L'attendibilità si basa sull'identità di invio selezionata e sulla policy dei domini effettiva.
mail-external-reviewed = Ho controllato questi destinatari esterni.
mail-external-confirm-required = Controllare e confermare i destinatari esterni prima dell'invio.
mail-quick-reply = Risposta rapida
mail-quick-reply-help = Offre una scheda di risposta compatta con i destinatari già compilati. Disattivato per impostazione predefinita. I normali controlli di sicurezza e le opzioni di invio restano validi.
mail-quick-reply-expand = Modifica destinatari e oggetto
mail-attachment-policy-loading = Controllo della policy sugli allegati…
mail-attachment-policy-unavailable = Impossibile controllare la policy sugli allegati. Riprovare prima di aprire questo file.
mail-attachment-policy-blocked = L'amministratore ha bloccato questo allegato o il download dell'originale.
mail-attachment-policy-warning = L'amministratore raccomanda cautela con questo allegato. Procedere solo se si ritiene attendibile la fonte.
mail-attachment-policy-acknowledge = Ho capito — continua
mail-attachment-policy-no-preview = Le anteprime degli allegati sono disattivate dall'amministratore. I download degli originali potrebbero comunque essere disponibili.
mail-time-sent = Inviato
mail-time-received = Ricevuto
mail-time-unknown = Non indicato
mail-time-comparison = Ora in { $zone }
mail-time-privacy = Il fuso orario di confronto è una scelta dell'utente. Le marche temporali dei messaggi non identificano la posizione di nessuno.
mail-time-event-comparison = Confronto dell'orario dell'evento
mail-time-ambiguous = Questo orario locale ricorre due volte quando cambia l'ora. Vengono mostrati entrambi gli istanti possibili; controllare lo scostamento previsto dell'evento prima di salvare.
mail-bulk-add-label = Aggiungi etichetta
mail-select-messages = Seleziona messaggi
mail-bulk-remove-label = Rimuovi etichetta
mail-mark-spam = Segna come spam
mail-bulk-forward = Inoltra i messaggi selezionati come allegati
mail-bulk-download = Scarica i messaggi selezionati come mbox
mail-bulk-forward-subject = Messaggi inoltrati ({ $count })
mail-bulk-transfer-failed = Impossibile preparare i messaggi selezionati. Selezionare al massimo 100 messaggi per un totale di non oltre 32 MiB e controllare i permessi di esportazione dell'account.
mail-toast-labels-added = Etichetta { $label } aggiunta a { $count } messaggi
mail-toast-labels-removed = Etichetta { $label } rimossa da { $count } messaggi
mail-action-help-reply = Rispondi al mittente in una nuova scheda di scrittura.
mail-action-help-reply-all = Rispondi al mittente e agli altri destinatari in una nuova scheda di scrittura.
mail-action-help-forward = Controlla e inoltra questo messaggio ai destinatari scelti.
mail-action-help-pin = Mantieni fissato questo messaggio per accedervi facilmente, oppure rimuovi la fissazione.
mail-action-help-read = Cambia se questo messaggio è segnato come letto. La modifica può essere annullata.

# -- Machine translation (2026-11): added to match en ---------------------
mail-action-help-flag = Imposta o rimuovi il contrassegno del messaggio per il seguito.
mail-action-help-archive = Sposta questo messaggio nella cartella di archivio configurata.
mail-action-help-delete = Sposta questo messaggio nel Cestino. La mossa può essere annullata.
mail-action-help-spam = Segna questo messaggio come posta indesiderata e spostalo nella cartella spam configurata.
mail-action-help-export = Apre la pagina di esportazione per scaricare questo messaggio nel formato scelto.
mail-action-help-sweep = Rivedi le azioni per gli altri messaggi di questo mittente prima di applicarle.
mail-empty-trash = Svuota il Cestino
mail-trash-help = Controllare ed eliminare definitivamente i messaggi attualmente nel Cestino. L'operazione non può essere annullata.
mail-trash-preparing = Preparazione di un'istantanea dei messaggi nel Cestino…
mail-trash-prepare-failed = Impossibile preparare l'anteprima del Cestino. Controllare la connessione e riaprirla. Nessun messaggio è stato eliminato.
mail-trash-confirm = Eliminare definitivamente { $count } messaggi dal Cestino? L'operazione non può essere annullata. I messaggi arrivati dopo questa anteprima sono esclusi; i messaggi spostati fuori dal Cestino vengono saltati.
mail-trash-limit = Questa operazione include fino a { $count } messaggi. Nel Cestino ne restano altri; controllare un'altra operazione in seguito per rimuoverli.
mail-trash-permanent = Elimina definitivamente
mail-trash-stop = Interrompi dopo la richiesta corrente
mail-trash-progress = Eliminati: { $deleted }; saltati: { $skipped } messaggi non più nel Cestino; ancora in questa anteprima: { $remaining }.
mail-trash-stopped = Eliminazione interrotta. Eliminazione confermata: { $deleted }; saltati: { $skipped }; rimanenti: { $remaining }; esito sconosciuto: { $uncertain }. Controllare il Cestino prima di avviare un'altra operazione.
mail-links-title = Link del messaggio
mail-links-review = Controlla i link ({ $count })
mail-links-help = I link del messaggio si aprono solo dopo aver controllato qui la loro destinazione. Nessuna destinazione viene recuperata automaticamente.
mail-links-inert = Usare Controlla i link sopra il messaggio per esaminare e aprire questa destinazione.
mail-links-limit = Sono elencate le prime 100 destinazioni supportate distinte. Gli altri link restano inattivi nel messaggio.
mail-links-destination = Destinazione da aprire
mail-links-external = Questa destinazione proviene dal mittente del messaggio. Aprirla porta fuori da Mailwoman; controllare l'indirizzo prima di continuare.
mail-links-open = Apri in una nuova scheda
mail-links-compose = Controlla un nuovo messaggio
mail-links-warning-http = Questo indirizzo usa HTTP non cifrato.
mail-links-warning-address = Questa destinazione usa un indirizzo IP invece di un nome di dominio.
mail-links-warning-international = Questo dominio contiene un nome internazionalizzato. Controllare con attenzione la grafia ASCII.
mail-links-warning-mismatch = L'indirizzo web visibile indica un host diverso dalla destinazione effettiva.
mail-links-warning-direction = Il testo del link contiene caratteri che cambiano la direzione del testo e possono nascondere l'indirizzo.
mail-outbox-uncertain = Stato di consegna sconosciuto
mail-outbox-uncertain-note = L'invio è iniziato, ma la consegna non ha potuto essere confermata. Questo messaggio non verrà reinviato automaticamente. Controllare Inviati o chiedere conferma al destinatario prima di inviare di nuovo.
mail-conversation-title = Visualizza conversazione
mail-conversation-help = Sfoglia i messaggi di conversazione disponibili di questo account in tutte le cartelle. L'apertura di un messaggio non cambia la cartella selezionata.
mail-conversation-count = Mostrati { $loaded } di { $total } messaggi della conversazione.
mail-conversation-other-folder = Altra cartella
mail-conversation-loading = Caricamento della conversazione…
mail-conversation-missing = { $count } messaggio/i non più disponibile/i.
mail-conversation-error = Impossibile caricare la conversazione. Riprovare.
mail-conversation-more = Carica altri messaggi
mail-conversation-limit = Questa vista è limitata a { $count } messaggi.
mail-compose-request-delivery = Richiedi le notifiche di consegna
mail-compose-request-delivery-hint = Chiede ai server di posta riceventi di segnalare consegna, errore o ritardo. Non richiede né dimostra che qualcuno abbia letto il messaggio. I rapporti restituiti includono le intestazioni, non il corpo completo.
mail-compose-delivery-unavailable = Le notifiche di consegna non sono disponibili o sono disattivate dall'amministratore per questo account. Una notifica richiesta esplicitamente non verrà rimossa in silenzio.
mail-compose-account-changed = Questa composizione appartiene a un altro accesso. Accedere all'account corretto e riaprire la bozza.
mail-context-open-message = Apri messaggio
mail-context-open-folder = Apri cartella
mail-context-download = Scarica messaggio
mail-context-refresh = Aggiorna cartella
mail-context-folder-settings = Impostazioni della cartella
mail-context-failed = Impossibile completare l'azione. Riprovare.
mail-sharing-unavailable = La condivisione delle cartelle non è disponibile oppure non si dispone più dell'autorizzazione per gestirla.
mail-trash-cancelling = Interruzione dopo la richiesta corrente. Eliminazione confermata: { $deleted }; saltati: { $skipped }; rimanenti: { $remaining }. Le eliminazioni già accettate non possono essere annullate.
mail-trash-already-running = L'eliminazione del Cestino è già in corso. Usare la notifica di avanzamento per interrompere i lotti futuri.
mail-context-search-sender = Cerca questo mittente
mail-context-search-subject = Cerca lo stesso oggetto
mail-drag-too-many = Trascinare al massimo { $count } messaggi alla volta. Selezionare un gruppo più piccolo prima di spostare.
mail-subtab-reorder-help = Trascinare per riordinare le schede, oppure premere Ctrl+Maiusc+Freccia sinistra o destra sulla scheda attiva.
mail-compose-ribbon = Barra multifunzione dell'editor
mail-compose-ribbon-format = Formatta testo
mail-compose-ribbon-insert = Inserisci
mail-compose-font = Carattere
mail-compose-font-size = Dimensione carattere
mail-compose-font-inherit = Predefinito
mail-compose-undo = Annulla
mail-compose-redo = Ripeti
mail-compose-undo-tip = Annulla ({ $shortcut })
mail-compose-redo-tip = Ripeti ({ $shortcut })
mail-workspace-limit = Chiudere una scheda prima di aprirne un'altra. Si possono mantenere fino a 32 schede.
mail-compose-private-storage-unavailable = Il ripristino locale delle bozze non è disponibile per questa sessione. Tenere aperto questo editor finché non si invia o si copia il messaggio.
mail-offline-storage-unavailable = Le modifiche offline non sono disponibili finché questa sessione non ha verificato l'autorità di archiviazione. Riconnettersi e accedere di nuovo.
mail-draft-status-idle = Non ancora salvata nell'account
mail-draft-status-pending = Modifiche non salvate
mail-draft-status-saving = Salvataggio in Bozze…
mail-draft-status-saved = Salvata nelle Bozze dell'account
mail-draft-status-error = La bozza non è stata salvata. La copia di ripristino locale resta disponibile.
mail-draft-status-uncertain = Manca la conferma del salvataggio. Controllare Bozze prima di creare un'altra copia; il salvataggio automatico è in pausa.
mail-draft-status-conflict = Questa bozza è cambiata su un altro dispositivo. Riaprire la copia dell'account prima di continuare.
mail-draft-status-unsupported = Al momento questo account non può confermare i salvataggi nella cartella Bozze.
mail-draft-protected = Il contenuto citato cifrato o decifrato non viene salvato come testo semplice. Un'eventuale bozza precedente dell'account resta finché non viene scartata esplicitamente.
mail-draft-save-now = Salva in Bozze
mail-draft-save-failed = Impossibile salvare questa bozza nell'account. Controllare l'editor prima di chiuderlo.
mail-draft-uncertain = La bozza potrebbe essere stata salvata. I tentativi automatici sono in pausa; controllare la cartella Bozze.
mail-draft-conflict = Un altro dispositivo ha modificato questa bozza. Le modifiche non hanno sovrascritto la sua copia.
mail-draft-cleanup-pending = La nuova bozza è stata salvata, ma non è stato possibile rimuovere la copia precedente. Controllare Bozze prima di salvare di nuovo.
mail-draft-account-list = Bozze dell'account
mail-draft-loading = Caricamento delle bozze dell'account…
mail-draft-close-recovery = Chiudi e mantieni solo il ripristino locale
mail-draft-discard-account = Scarta
mail-draft-discard-pending = Non è stato possibile confermare la rimozione della bozza. Il suo record nell'account resta; riprovare Scarta dopo aver controllato la cartella Bozze.
mail-draft-limit = Questo account ha già 100 bozze salvate. Scartare una bozza dell'account non desiderata prima di salvarne una nuova.
mail-draft-quota-help = Vengono mantenute fino a 100 bozze dell'account. Scarta rimuove la copia confermata sul server prima di liberare il posto.
mail-draft-discard-confirm = Conferma la rimozione della bozza
mail-draft-discard-confirm-detail = Rimuovere definitivamente “{ $subject }” dalle Bozze dell'account? Chiudere invece l'editor mantiene la bozza.
mail-draft-discard-title = Scartare questa bozza?
mail-draft-keep-editing = Continua a modificare
mail-draft-discard-action = Scarta
mail-draft-close-title = Scartare le modifiche non salvate?
mail-draft-close-protected = Questo messaggio contiene contenuto cifrato o decifrato, quindi non viene salvato come bozza. Chiuderlo scarta queste modifiche. Un'eventuale bozza precedente dell'account resta in Bozze.
mail-draft-close-unstored = In questa sessione non è possibile salvare le bozze, quindi chiudere questo messaggio lo scarta.
mail-workspace-view-mode = Disposizione delle viste
mail-workspace-tabs-help = Mostra una vista attiva mantenendo aperte le altre schede.
mail-workspace-side-by-side-help = Mostra due viste selezionate una accanto all'altra. Trascinare una scheda su una delle viste per sostituirla.
mail-workspace-stack-help = Mostra due viste selezionate una sopra l'altra. Trascinare una scheda su una delle viste per sostituirla.
mail-workspace-masonry-side-by-side = Mosaico affiancato
mail-workspace-masonry-side-by-side-help = Dispone tutte le viste aperte in colonne sfalsate. Gli schermi piccoli mostrano la scheda attiva.
mail-workspace-masonry-stack = Mosaico impilato
mail-workspace-masonry-stack-help = Dispone tutte le viste aperte in righe sfalsate. Gli schermi piccoli mostrano la scheda attiva.
mail-workspace-view-first = Prima vista
mail-workspace-view-second = Seconda vista
mail-workspace-view-empty = Trascinare qui una scheda aperta per scegliere questa vista.
mail-popup-blocked = Consentire le finestre popup per Mailwoman per staccare questa scheda. La scheda resta qui.
mail-popup-unavailable = Impossibile caricare la finestra staccata. La scheda e le modifiche non salvate restano nella finestra principale. Riprovare dopo aver aggiornato l'app.
mail-popup-reattach = Riaggancia alla finestra principale
mail-popup-action-reattached = Tornato alla finestra principale per completare questa azione.
mail-popup-detached = Questa scheda è aperta in una finestra separata. Chiudendo quella finestra, la scheda torna qui.
mail-compose-from-custom = Un altro indirizzo autorizzato…
mail-compose-from-alias = Indirizzo Da per questo messaggio
mail-compose-from-alias-help = Usare un alias da cui si è autorizzati a inviare. Cambia solo questa bozza; l'autorizzazione viene verificata prima dell'invio.
mail-compose-from-refused = Questo indirizzo non è autorizzato all'invio da questo account.
mail-compose-from-unavailable = Impossibile verificare questo indirizzo mittente. Il messaggio non è stato inviato.
mail-compose-from-invalid = Inserire un unico indirizzo email valido per il mittente.
mail-compose-from-toggle = Mostra o nascondi il mittente per questo messaggio
mail-compose-ribbon-security = Sicurezza
mail-compose-ribbon-delivery = Consegna
mail-compose-ribbon-recipients = Destinatari
mail-compose-ribbon-message-type = Tipo di messaggio
mail-compose-recall-help = Annulla i messaggi ancora trattenuti per l'annullamento o la consegna programmata. I messaggi già consegnati non possono essere richiamati.
mail-compose-recall-loading = Controllo dei messaggi in coda…
mail-compose-recall-empty = Nessun messaggio in coda è disponibile da annullare.

# -- Machine translation (2026-11): added to match en ---------------------
mail-compose-recall-failed = Impossibile confermare lo stato della coda. Chiudere e riaprire questo pannello per aggiornarlo prima di un'altra azione.
mail-compose-importance = Importanza
mail-compose-importance-high = Alta
mail-compose-importance-normal = Normale
mail-compose-importance-low = Bassa
mail-insert-account-title = Inserisci elemento dell'account
mail-insert-account-help = Allegare un'esportazione del calendario, un biglietto da visita di un contatto o un messaggio originale di questo account.
mail-insert-account-close = Chiudi il selettore degli elementi dell'account
mail-insert-account-kind = Tipo di elemento
mail-insert-account-event = Evento del calendario (.ics)
mail-insert-account-calendar = Calendario (.ics)
mail-insert-account-contact = Biglietto da visita del contatto (.vcf)
mail-insert-account-message = Messaggio originale (.eml)
mail-insert-account-search = Filtra gli elementi elencati
mail-insert-account-selection = Selezionare un elemento
mail-insert-account-scope = Sono elencati solo gli elementi dell'account corrente. I messaggi originali seguono la policy di esportazione del server.
mail-insert-account-limited = Vengono mostrati fino a 100 elementi. Il filtro cerca solo in questo elenco.
mail-insert-account-unavailable = Nessuna esportazione dell'account supportata è disponibile.
mail-insert-account-empty = Nessun elemento corrispondente.
mail-insert-account-attach = Allega l'elemento selezionato
mail-insert-account-loading = Caricamento dell'elemento dell'account…
mail-insert-account-error = Impossibile allegare l'elemento. Controllare l'accesso e la policy di esportazione, poi riprovare. I calendari di grandi dimensioni potrebbero richiedere l'esportazione dei singoli eventi.
mail-compose-html-source = Sorgente HTML
mail-editor-clear-format = Cancella formattazione
mail-compose-format-html = HTML
mail-compose-html-visual = Editor visuale
mail-compose-html-help = Modificare direttamente l'HTML. Vengono salvate e inviate solo le formattazioni email supportate; script, immagini esterne e markup non sicuro vengono rimossi.
mail-compose-html-processing = Preparazione di HTML sicuro…
mail-compose-html-failed = Impossibile preparare l'HTML. Modificarlo o riprovare prima di salvare o inviare.
mail-compose-check-names = Controlla i nomi
mail-compose-check-names-help = Risolve i nomi esatti dai contatti caricati e controlla la formattazione degli indirizzi. Non verifica la consegna alla casella.
mail-compose-names-unresolved = Scegliere un indirizzo email completo per questi nomi non risolti o ambigui: { $names }
mail-compose-names-checked = Formattazione dei destinatari controllata; { $count } nome/i di contatto risolto/i. La consegna alla casella non è stata verificata.
mail-compose-ribbon-table = Tabella
mail-compose-group-clipboard = Appunti
mail-compose-group-paragraph = Paragrafo
mail-compose-group-editing = Modifica
mail-compose-group-view = Visualizzazione
mail-compose-dark-background = Sfondo scuro
mail-compose-dark-background-help = Mostra il messaggio su sfondo scuro durante la scrittura. Cambia solo lo schermo: il messaggio viene inviato senza colore di sfondo.
mail-compose-group-assist = Assistente di scrittura
mail-editor-counts = Statistiche dell'editor
mail-editor-words = { $count ->
    [one] { $count } parola
   *[other] { $count } parole
}
mail-editor-paragraphs = { $count ->
    [one] { $count } paragrafo
   *[other] { $count } paragrafi
}
mail-editor-characters = { $count ->
    [one] { $count } carattere
   *[other] { $count } caratteri
}
mail-editor-selection-required = Selezionare un testo o celle di tabella adatti prima di usare questa azione.
mail-editor-clipboard-unavailable = L'accesso agli appunti non era disponibile. Usare le scorciatoie Copia, Taglia o Incolla del browser.
mail-editor-selection-changed = La selezione è cambiata. Selezionare il testo desiderato e riprovare.
mail-editor-clipboard = Appunti e formattazione
mail-editor-copy = Copia
mail-editor-cut = Taglia
mail-editor-paste = Incolla
mail-editor-copy-format = Copia formattazione
mail-editor-apply-format = Applica formattazione
mail-editor-style = Stile del paragrafo
mail-editor-text-color = Colore del carattere
mail-editor-clear-color = Colore predefinito
mail-editor-highlight = Evidenziazione
mail-editor-clear-highlight = Rimuovi evidenziazione
mail-editor-paragraph = Opzioni del paragrafo
mail-editor-alignment = Allineamento
mail-editor-line-height = Altezza della riga
mail-editor-space-before = Spazio prima
mail-editor-space-after = Spazio dopo
mail-editor-indent = Rientro
mail-editor-shading = Ombreggiatura del paragrafo
mail-editor-clear-shading = Rimuovi ombreggiatura
mail-editor-case = Cambia maiuscole/minuscole
mail-editor-sort = Ordina i paragrafi selezionati
mail-editor-sort-alpha = Alfabetico
mail-editor-sort-numeric = Ordine numerico
mail-editor-table-rows = Righe della tabella
mail-editor-table-columns = Colonne della tabella
mail-editor-table-insert = Inserisci tabella
mail-editor-table-limits = Fino a 20 righe, 10 colonne e 200 celle per tabella; al massimo 10 tabelle. Le tabelle annidate non sono supportate.
mail-editor-table-row-before = Riga sopra
mail-editor-table-row-after = Riga sotto
mail-editor-table-row-delete = Elimina riga
mail-editor-table-column-before = Colonna prima
mail-editor-table-column-after = Colonna dopo
mail-editor-table-column-delete = Elimina colonna
mail-editor-table-merge = Unisci celle
mail-editor-table-split = Dividi cella
mail-editor-table-header = Attiva/disattiva riga di intestazione
mail-editor-table-delete = Elimina tabella
mail-editor-table-shading = Ombreggiatura della cella
mail-editor-find-replace = Trova e sostituisci
mail-editor-find = Trova testo
mail-editor-replacement = Sostituisci con
mail-editor-match-case = Maiuscole/minuscole
mail-editor-previous = Corrispondenza precedente
mail-editor-next = Corrispondenza successiva
mail-editor-replace = Sostituisci la corrispondenza selezionata
mail-editor-replace-all = Sostituisci tutte le corrispondenze
mail-editor-matches = { $count } corrispondenze (fino a 1.000)
mail-editor-select-all = Seleziona tutto
mail-editor-clear-selection = Cancella selezione
mail-editor-history = Cronologia di annullamento
mail-editor-undo-groups = Annulla gli ultimi { $count } gruppi di modifiche
mail-editor-zoom = Zoom dell'editor
mail-editor-markers = Segni di paragrafo
mail-editor-style-normal = Normale
mail-editor-style-no-spacing = Nessuna spaziatura
mail-editor-style-title = Titolo
mail-editor-style-subtitle = Sottotitolo
mail-editor-style-heading1 = Titolo 1
mail-editor-style-heading2 = Titolo 2
mail-editor-style-heading3 = Titolo 3
mail-editor-style-heading4 = Titolo 4
mail-editor-style-heading5 = Titolo 5
mail-editor-style-heading6 = Titolo 6
mail-editor-align-left = Sinistra
mail-editor-align-center = Centro
mail-editor-align-right = Destra
mail-editor-align-justify = Giustificato
mail-editor-case-lower = minuscolo
mail-editor-case-upper = MAIUSCOLO
mail-editor-case-title = Iniziali Maiuscole
mail-editor-case-sentence = Maiuscola come in una frase
mail-editor-action-history = Cronologia di { $action }
mail-editor-history-empty = Nessuna azione disponibile
mail-editor-redo-groups = Ripeti { $count } modifiche
mail-compose-identities-failed = Impossibile caricare le identità di invio. Chiudere e riaprire questa bozza per riprovare.
mail-folders-loading = Caricamento delle cartelle…
mail-initial-sync-error = Non è stato ancora possibile sincronizzare i messaggi.
mail-initial-sync-error-help = La connessione al server di posta è stata interrotta. Riprovare, oppure chiedere all'amministratore di controllare i log di connessione se il problema persiste.
mail-links-trusted = Dominio attendibile: si apre senza controllo
mail-links-trust-domain = Considera attendibile { $domain }
mail-links-trust-failed = Impossibile salvare il dominio attendibile. Riprovare.
mail-focus-always-focused = Sposta sempre in Priorità
mail-focus-always-other = Sposta sempre in Altro

# -- Machine translation (2026-11): added to match en ---------------------
mail-focus-moved-focused = I messaggi di { $sender } ora vanno in Priorità.
mail-focus-moved-other = I messaggi di { $sender } ora vanno in Altro.
mail-focus-save-failed = Impossibile salvare la scelta della Posta in arrivo con priorità per questo mittente. Riprovare.
mail-context-search-domain = Cerca i messaggi di questo dominio
mail-loading-mailbox = Caricamento di { $name }…
mail-user-folders = Cartelle utente
mail-context-hide-folder = Nascondi dalla barra laterale
mail-folder-fallback = cartella
mail-toast-moving = { $count ->
    [one] Spostamento di un messaggio in { $folder }…
   *[other] Spostamento di { $count } messaggi in { $folder }…
}
mail-toast-move-queueing = { $count ->
    [one] Accodamento di un messaggio da spostare in { $folder }…
   *[other] Accodamento di { $count } messaggi da spostare in { $folder }…
}
mail-toast-move-queued = { $count ->
    [one] Un messaggio in coda per essere spostato in { $folder }
   *[other] { $count } messaggi in coda per essere spostati in { $folder }
}
mail-toast-move-partial = Spostati { $count } messaggi su { $total } in { $folder }; { $failed } non è stato possibile spostarli
mail-send-copy-folder-unavailable = La cartella selezionata per la copia inviata non è disponibile. Scegliere un'altra cartella prima di inviare.
mail-send-copy-failed = Il messaggio è stato accettato, ma non è stato possibile archiviare la copia inviata. Non inviarlo di nuovo.
mail-compose-attach-public-key = Allega la mia chiave pubblica o certificato
mail-compose-include-hashes = Includi gli hash SHA-256 dei file allegati
mail-compose-hashes-pending = Calcolo degli hash degli allegati. Inviare quando è pronto.
mail-compose-hashes-failed = Impossibile verificare i byte degli allegati. Disattivare gli hash o riprovare riattivandoli.
mail-compose-crypto-pending = La cifratura è in aggiornamento. Attendere prima di inviare.
mail-compose-public-key-unavailable = Nessuna chiave pubblica o certificato attuale corrisponde all'indirizzo di invio.
mail-compose-sent-copy = Salva il messaggio inviato in
mail-compose-sent-copy-default = Predefinito dell'account
folder-create = Crea cartella
folder-name = Nome della cartella
folder-parent = Cartella superiore
folder-top-level = Livello superiore
folder-created = Cartella creata.
folder-create-unavailable = La creazione di cartelle non è disponibile per questo account o per questa cartella superiore.
folder-create-invalid = Scegliere un nome di cartella valido e univoco, senza separatori di gerarchia né caratteri di controllo.
folder-create-failed = Impossibile creare la cartella. Aggiornare e riprovare.
folder-cleanup = Pulisci cartella…
folder-cleanup-title = Pulisci { $name }
folder-cleanup-help = Trova i messaggi più vecchi dell'età scelta. Controllare il conteggio prima di eliminare. Le sottocartelle sono escluse.
folder-cleanup-age = Più vecchi di
folder-age-days = Giorni
folder-age-weeks = Settimane
folder-age-months = Mesi
folder-age-years = Anni
folder-cleanup-preview = Anteprima della pulizia
folder-cleanup-confirm = Elimina i messaggi corrispondenti
folder-cleanup-permanent = Eliminare definitivamente { $count } messaggi corrispondenti. L'operazione non può essere annullata.
folder-cleanup-trash = Spostare { $count } messaggi corrispondenti nel Cestino.
folder-cleanup-before = Ricevuti prima del { $date }
folder-cleanup-done = { $count } messaggi puliti.
mail-from = Mittente
mail-subject = Oggetto
mail-date = Data
mail-cancel = Annulla
mail-bulk-selected = { $count } conversazioni selezionate
mail-bulk-select-folder = Seleziona tutto in { $folder }
mail-list-order = Ordine dei messaggi
mail-list-order-newest = Prima i più recenti
mail-list-order-oldest = Prima i più vecchi
mail-list-order-sender = Mittente
mail-list-order-subject = Oggetto
mail-list-order-error = Impossibile salvare l'ordine dei messaggi.
mail-bulk-selection-load-failed = Impossibile selezionare l'intera cartella. Controllare la connessione e riprovare.
mail-privacy-bar = Privacy del messaggio
mail-privacy-trackers-allow = Consenti i tracker per questo messaggio
mail-privacy-trackers-block = Blocca di nuovo i tracker
mail-privacy-images-loading = { $loaded }/{ $total } immagini mostrate · { $pending } in caricamento
mail-privacy-images-failed = { $loaded }/{ $total } immagini mostrate · { $failed } non riuscite
mail-privacy-images-loaded = { $loaded }/{ $total } immagini mostrate
mail-privacy-images-loading-failed = { $loaded }/{ $total } immagini mostrate · { $pending } in caricamento · { $failed } non riuscite
mail-reader-folder-title = Cartella: { $folder }
mail-reader-move-to = Sposta in…
mail-action-help-move = Sposta questo messaggio in un'altra cartella.
folder-ops-new-subfolder = Nuova sottocartella
folder-ops-mark-read = Segna tutto come letto
folder-ops-this-folder = Questa cartella
folder-ops-with-subfolders = Includi le sottocartelle
folder-ops-no-unread = Nessun messaggio non letto.
folder-ops-read-unavailable = Non è possibile modificare i messaggi in questa cartella.
folder-ops-more = Altro
folder-ops-system-folder = Una cartella di sistema non può essere modificata in questo modo.
folder-ops-sort-subfolders = Ordina le sottocartelle per
folder-ops-sort-folders = Ordina le cartelle per
folder-ops-sort-name = Nome
folder-ops-sort-unread = Numero di non letti
folder-ops-sort-total = Numero totale
folder-ops-sort-date = Ultima attività
folder-ops-sort-custom = Ordine personalizzato
folder-ops-sort-asc = Crescente
folder-ops-sort-desc = Decrescente
folder-ops-delete-with-subfolders = Elimina la cartella e le sottocartelle
folder-ops-delete-unavailable = Questa cartella non può essere eliminata.
folder-ops-delete-tree-body = Questa operazione elimina definitivamente { $folders } cartelle e i { $count } messaggi che contengono. L'operazione non può essere annullata.
folder-ops-copy-all-menu = Copia tutti gli elementi in…
folder-ops-move-all-menu = Sposta tutti gli elementi in…
folder-ops-move-folder-menu = Sposta cartella in…
folder-ops-move-up = Sposta in su
folder-ops-move-down = Sposta in giù
folder-ops-move-unavailable = Non è possibile spostare questa cartella né i suoi elementi.
folder-ops-move-duplicate = La destinazione ha già una cartella con questo nome.
folder-ops-move-folder-title = Sposta { $name }
folder-ops-move-folder-body = La cartella viene spostata con le sue sottocartelle e i suoi messaggi. Scegliere la cartella in cui verrà inserita, oppure il livello superiore.
folder-ops-move-here = Sposta qui
folder-ops-moved-folder = { $name } spostata.
folder-ops-choose-folder = Scegliere una cartella di destinazione.
folder-ops-destination = Cartella di destinazione
folder-ops-include-subfolders = Includi i messaggi di { $count } sottocartelle
folder-ops-copy-all-title = Copia tutti gli elementi di { $name }
folder-ops-move-all-title = Sposta tutti gli elementi di { $name }
folder-ops-copy-all-body = Aggiunge { $count } messaggi alla cartella di destinazione. Restano dove sono.
folder-ops-move-all-body = Sposta { $count } messaggi nella cartella di destinazione. Si potrà annullare in seguito.
folder-ops-copy-all = Copia elementi
folder-ops-move-all = Sposta elementi
folder-ops-undo = Annulla
folder-ops-stop = Interrompi
folder-ops-marking = Segnatura di { $name } come letta…
folder-ops-marking-progress = Segnatura di { $name } come letta: { $done } di { $total }
folder-ops-marked = { $count } messaggi in { $name } segnati come letti.
folder-ops-copying = Copia di { $name }…
folder-ops-copying-progress = Copia di { $name }: { $done } di { $total }
folder-ops-copied = Copiati { $count } messaggi da { $name }.
folder-ops-moving = Spostamento di { $name }…
folder-ops-moving-progress = Spostamento di { $name }: { $done } di { $total }
folder-ops-moved = Spostati { $count } messaggi da { $name }.
folder-ops-stopped = { $name } interrotta dopo { $count } messaggi su { $total }.
folder-ops-partial = { $count } messaggi su { $total } in { $name } sono stati elaborati; { $refused } sono stati rifiutati.
folder-ops-failed = Impossibile completare { $name }. Aggiornare e riprovare.
folder-ops-error-forbidden = Non si dispone dei diritti per questa modifica.
folder-ops-error-exists = Lì esiste già una cartella con questo nome.
folder-ops-error-mailboxHasEmail = La cartella contiene ancora messaggi.
folder-ops-error-notFound = La cartella non esiste più.
folder-ops-error-failed = Impossibile apportare la modifica. Aggiornare e riprovare.
folder-ops-expand = Mostra le sottocartelle di { $name }
folder-ops-collapse = Nascondi le sottocartelle di { $name }
mail-context-reply = Rispondi
mail-context-reply-all = Rispondi a tutti
mail-context-forward = Inoltra

# -- Machine translation (2026-11): added to match en ---------------------
mail-context-quick-print = Stampa rapida
mail-context-copy = Copia
mail-context-copy-to-folder = Copia il messaggio nella cartella…
mail-context-copy-sender = Copia l'indirizzo del mittente
mail-context-copy-subject = Copia l'oggetto
mail-context-copy-link = Copia il link del messaggio
mail-context-copied = Copiato.
mail-context-copy-failed = Impossibile copiare negli appunti.
mail-context-print-failed = Impossibile stampare il messaggio.
mail-reply-images-title = Immagini del messaggio originale
mail-reply-images-ask = Includere le immagini del messaggio originale in questa risposta?
mail-reply-images-include = Includi immagini
mail-reply-images-not-now = Non ora
mail-reply-images-trust = Considera sempre attendibile { $domain }
mail-reply-images-loading = Aggiunta delle immagini…
mail-reply-images-cancelled = Caricamento annullato. Le immagini non sono state aggiunte.
mail-reply-images-failed = Impossibile aggiungere le immagini.
mail-reply-images-retry = Carica di nuovo
mail-image-cancel = Annulla
mail-compose-from-why-match = Invio da { $identity } perché il messaggio è stato inviato a { $address }.
mail-compose-from-why-match-plus = Invio da { $identity } perché il messaggio è stato inviato a { $address }, un indirizzo con segno più di essa.
mail-compose-from-why-ambiguous = Più identità corrispondono a { $address }. Invio da { $identity }; sceglierne un'altra:
mail-compose-from-why-default = Invio dall'identità predefinita, { $identity }, perché nessuna corrisponde ai destinatari originali.
mail-compose-from-why-forced-default = Invio dall'identità predefinita, { $identity }, come impostato nelle impostazioni.
mail-compose-from-why-keep = Invio da { $identity }.
mail-compose-from-change = Cambia
reader-attachments-reason-encrypted = Questo messaggio è cifrato. Gli allegati compaiono dopo averlo sbloccato.
reader-attachments-reason-list-missing = Impossibile caricare l'elenco degli allegati di questo messaggio, quindi la barra degli allegati è nascosta.
reader-attachments-reason-inline-only = Gli allegati di questo messaggio ({ $count }) sono immagini inline mostrate nel messaggio stesso.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] Un allegato di questo messaggio non ha contenuto scaricabile, quindi la barra degli allegati è nascosta.
   *[other] { $count } allegati di questo messaggio non hanno contenuto scaricabile, quindi la barra degli allegati è nascosta.
}
reader-attachments-reason-policy-blocked = La policy sugli allegati blocca il download { $count ->
    [one] di questo allegato.
   *[other] di questi allegati.
}
reader-attachments-reason-policy-unavailable = Impossibile controllare la policy sugli allegati, quindi gli allegati non possono ancora essere aperti.
mail-drafts-popup-count = { $count ->
    [one] 1 bozza
   *[other] { $count } bozze
}
mail-drafts-popup-search = Cerca per oggetto o destinatario
mail-drafts-popup-tab-local = Bozze locali
mail-drafts-popup-local-hint = Le bozze locali sono conservate solo in questo browser.
mail-drafts-popup-badge-account = Account
mail-drafts-popup-badge-local = Locale
mail-drafts-popup-current = Aperta ora
mail-drafts-popup-select = Seleziona
mail-drafts-popup-select-all = Seleziona tutto
mail-drafts-popup-clear = Cancella
mail-drafts-popup-done = Fine
mail-drafts-popup-selected = { $count } selezionate
mail-drafts-popup-select-row = Seleziona { $subject }
mail-drafts-popup-open = Apri
mail-drafts-popup-discard = Scarta
mail-drafts-popup-discard-selected = Scarta selezionate
mail-drafts-popup-more = Altre azioni per { $subject }
mail-drafts-popup-retry = Riprova
mail-drafts-popup-load-failed = Impossibile caricare le bozze dell'account.
mail-drafts-popup-empty-account = Nessuna bozza dell'account
mail-drafts-popup-no-match = Nessuna bozza corrisponde a “{ $query }”
mail-drafts-popup-discarding = Scarto in corso…
mail-drafts-popup-row-failed = Impossibile scartarla. La copia dell'account resta.
mail-drafts-popup-confirm-title = { $count ->
    [one] Scartare questa bozza?
   *[other] Scartare { $count } bozze?
}
mail-drafts-popup-confirm-account = Le bozze dell'account vengono prima rimosse dalla cartella Bozze dell'account e il loro posto viene liberato solo dopo la conferma di tale rimozione.
mail-drafts-popup-confirm-local = Le bozze locali vengono rimosse da questo browser.
mail-drafts-popup-announce-done = { $count ->
    [one] 1 bozza scartata
   *[other] { $count } bozze scartate
}
mail-drafts-popup-announce-failed = { $done } scartate, { $failed } non è stato possibile scartarle
mail-check-title = Controlla prima di inviare
mail-check-intro = Questi elementi potrebbero richiedere un secondo sguardo:
mail-check-invalid = Alcuni indirizzi dei destinatari sembrano non validi: { $list }
mail-check-subject = L'oggetto è vuoto.
mail-check-attachment = Il messaggio menziona un allegato, ma non è allegato nulla.
mail-check-external = Destinatari fuori dai domini attendibili: { $list }
mail-check-send-anyway = Invia comunque
mail-check-review = Controlla
mail-check-remember = Non chiedere più per questi
mail-check-denied-invalid = L'invio è bloccato dalle impostazioni. Correggere gli indirizzi dei destinatari non validi: { $list }
mail-check-denied-subject = L'invio è bloccato dalle impostazioni. Aggiungere un oggetto.
mail-check-denied-attachment = L'invio è bloccato dalle impostazioni. Allegare il file citato oppure riformulare il messaggio.
mail-check-denied-external = L'invio è bloccato dalle impostazioni. Fuori dai domini attendibili: { $list }
mail-alerts-label = Problemi del messaggio
mail-alert-invalid = Indirizzo non valido: { $address }
mail-alert-subject = Nessun oggetto
mail-recipient-chip-remove = Rimuovi { $address }
mail-recipient-chips = Destinatari in { $field }
mail-recipient-invalid-title = Non è un indirizzo email valido
mail-recipient-chip-name = { $name }, { $address }. Invio per i dettagli del contatto, Backspace o F2 per modificare, Canc per rimuovere
mail-recipient-chip-name-invalid = { $address }, non è un indirizzo valido. Invio per modificare, Canc per rimuovere
mail-recipient-added = { $name } aggiunto
mail-recipient-removed = { $name } rimosso
mail-recipient-removed-many = { $count ->
    [one] { $count } destinatario rimosso
   *[other] { $count } destinatari rimossi
}
mail-recipient-editing = Modifica di { $name }
mail-recipient-all-selected = { $count ->
    [one] { $count } destinatario selezionato
   *[other] { $count } destinatari selezionati
}
mail-external-line = { $count } fuori dai domini attendibili: { $domains }
mail-external-dismiss = Ignora
mail-status-label = Stato dell'editor
mail-status-more = Altre { $count } notifiche
mail-status-more-label = Altre notifiche
mail-status-no-recipients = Nessun destinatario
mail-status-no-attachment = Nessun allegato
mail-status-external = Esterni: { $count }
mail-status-external-reviewed = Destinatari esterni controllati
mail-status-external-review = Segna come controllato
mail-status-from-match = Da { $identity }
mail-status-from-match-plus = Da { $identity }
mail-status-from-ambiguous = Da { $identity } (più corrispondenze)
mail-status-from-default = Dall'identità predefinita
mail-status-from-forced-default = Dall'identità predefinita
mail-status-from-keep = Da { $identity }
mail-status-held = { $count } in coda
mail-status-storage = Nessun ripristino locale
mail-status-protected = Non salvato come testo semplice
mail-status-saving = Salvataggio…
mail-status-saved = Salvato { $time }
mail-status-saved-full = Salvato nelle Bozze dell'account · { $time }
mail-status-not-saved = Bozza non salvata
mail-status-retry = Riprova
mail-status-conflict = Modificata su un altro dispositivo
mail-status-uncertain = Salvataggio non confermato
mail-status-close-recovery = Chiudi, mantieni il ripristino
mail-status-unsupported = Salvataggio della bozza non confermato
mail-compose-header = Intestazione del messaggio
mail-header-label = Intestazione del messaggio
mail-header-collapse = Comprimi l'intestazione del messaggio
mail-header-expand = Espandi l'intestazione del messaggio: { $summary }
mail-header-from-chip = Da { $address }. Cambia mittente
mail-compose-group-timing = Tempistica di invio
mail-compose-group-priority = Priorità
mail-compose-group-receipts = Conferme e tracciamento
mail-compose-group-keys-files = Chiavi e file
mail-compose-group-sent-copy = Copia inviata
mail-editor-style-quote = Citazione
mail-editor-style-code = Codice
mail-compose-ribbon-table-layout = Layout della tabella
mail-compose-ribbon-picture = Immagine
mail-editor-table-group-rows = Righe
mail-editor-table-group-columns = Colonne
mail-editor-table-group-cells = Celle
mail-editor-table-row-height = Altezza della riga
mail-editor-table-column-width = Larghezza della colonna
mail-editor-table-size-auto = Automatica
mail-editor-table-padding = Margine interno della cella
mail-editor-table-padding-compact = Compatto
mail-editor-table-padding-normal = Normale
mail-editor-table-padding-roomy = Ampio
mail-editor-table-align-left = Allinea la cella a sinistra

# -- Machine translation (2026-11): added to match en ---------------------
mail-editor-table-align-center = Centra la cella
mail-editor-table-align-right = Allinea la cella a destra
mail-picture-group-size = Dimensione
mail-picture-group-arrange = Disponi
mail-picture-group-text = Testo
mail-picture-group-style = Stile
mail-picture-group-edit = Modifica
mail-picture-width = Larghezza
mail-picture-height = Altezza
mail-picture-lock = Mantieni le proporzioni
mail-picture-reset-size = Dimensione originale
mail-picture-align-inline = In linea con il testo
mail-picture-align-left = Allinea a sinistra
mail-picture-align-center = Centra
mail-picture-align-right = Allinea a destra
mail-picture-alt = Testo alternativo
mail-picture-title = Titolo
mail-picture-border = Bordo
mail-picture-radius = Angoli
mail-picture-none = Nessuno
mail-picture-round = Arrotondati
mail-picture-rotate-ccw = Ruota a sinistra
mail-picture-rotate-cw = Ruota a destra
mail-picture-flip-h = Capovolgi in orizzontale
mail-picture-flip-v = Capovolgi in verticale
mail-picture-crop = Ritaglia
mail-picture-crop-left = Sinistra (%)
mail-picture-crop-top = Alto (%)
mail-picture-crop-right = Destra (%)
mail-picture-crop-bottom = Basso (%)
mail-picture-crop-apply = Ritaglia
mail-picture-replace = Sostituisci
mail-picture-remove = Rimuovi
mail-picture-failed-no-image = Selezionare prima un'immagine.
mail-picture-failed-no-source = Questa immagine non ha una copia locale da modificare. Sostituirla.
mail-picture-failed-failed = Impossibile modificare l'immagine.
mail-context-download-attachments = Scarica allegati
mail-context-download-named = Scarica { $name }
mail-context-download-all = Scarica tutto come ZIP
mail-context-loading-attachments = Caricamento degli allegati…
mail-context-attachments-failed = Impossibile caricare gli allegati.
mail-context-searching = Ricerca del messaggio…
mail-context-go-reply = Vai alla risposta
mail-context-go-forward = Vai all'inoltro
mail-context-reply-not-found = Risposta non trovata
mail-context-forward-not-found = Inoltro non trovato
mail-no-recipients = (nessun destinatario)
mail-attachment-downloaded = { $name } scaricato
mail-attachment-download-failed = Download non riuscito.
mail-attachments-zip-done = Scaricati { $count } allegati come ZIP.
mail-attachments-zip-capped = Lo ZIP contiene i primi { $count } allegati; il limite è di { $max } file o { $size }.
mail-focus-not-found = Messaggio non trovato in { $folder }
mail-focus-go-to = Vai a { $folder }
mail-nav-views-queues = Viste e code
mail-badge-unread = { $unread ->
    [one] { $unread } messaggio non letto
   *[other] { $unread } messaggi non letti
}
mail-badge-total = { $total ->
    [one] { $total } messaggio in questa cartella
   *[other] { $total } messaggi in questa cartella
}
mail-badge-both = { $unread ->
    [one] { $unread } messaggio non letto
   *[other] { $unread } messaggi non letti
} su { $total ->
    [one] { $total } in questa cartella
   *[other] { $total } in questa cartella
}
mail-badge-drafts = { $total ->
    [one] Bozze: { $total } messaggio
   *[other] Bozze: { $total } messaggi
}
mail-badge-compact-hint = Le cartelle diverse dalla Posta in arrivo usano un badge più piccolo. Modificarlo in Impostazioni, Cartelle e barra laterale.
mail-badge-outbox = { $count ->
    [one] Posta in uscita: { $count } messaggio in attesa di invio
   *[other] Posta in uscita: { $count } messaggi in attesa di invio
}
mail-color-menu = Colori del messaggio
mail-color-button = Colori del messaggio: { $mode }
mail-color-use-default = Usa il mio predefinito ({ $mode })
mail-color-make-default = Imposta come mio predefinito
mail-color-locked = Bloccato dall'amministratore
mail-color-confirm-title = Cambiare i colori predefiniti dei messaggi?
mail-color-confirm-body = Ogni messaggio userà "{ $mode }", a meno che non lo si cambi per un singolo messaggio.
mail-color-confirm = Imposta come predefinito
mail-color-default-failed = Impossibile salvare i colori predefiniti dei messaggi.
