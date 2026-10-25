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
mail-inbox-focused-enable = Posta in arrivo evidenziata
mail-inbox-filter = Filtro posta in arrivo
mail-inbox-focused = Evidenziata
mail-inbox-other = Altro
mail-inbox-turn-off = Disattiva

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
mail-toast-pinned = Fissato
mail-toast-unpinned = Sbloccato
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
