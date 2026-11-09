# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: settings. Source: ../en/settings.ftl

settings-title = Impostazioni
settings-appearance = Aspetto
settings-close = Chiudi impostazioni

settings-theme = Tema
settings-density = Densità
settings-accent = Colore di accento
settings-font = Carattere dell'interfaccia
settings-layout = Layout

settings-density-compact = Compatta
settings-density-cozy = Comoda
settings-density-relaxed = Ampia

settings-font-default = Predefinito
settings-font-system = Sistema
settings-font-serif = Serif
settings-font-mono = Mono

settings-layout-default = Predefinito
settings-layout-ribbon = Barra multifunzione

settings-cancel = Annulla
settings-edit = Modifica
settings-delete = Elimina
settings-save = Salva
settings-saved = Salvato

settings-2fa-title = Autenticazione a due fattori
settings-2fa-intro = Aggiungere un secondo fattore per richiedere più di una password all'accesso.
settings-2fa-policy-required = L'organizzazione richiede un secondo fattore su questo account.
settings-2fa-enabled = Attivata
settings-2fa-recovery-title = Codici di recupero
settings-2fa-recovery-once = Salvarli adesso. Vengono mostrati una sola volta e non possono essere recuperati. Ogni codice funziona una sola volta.
settings-2fa-recovery-ack = Ho salvato questi codici
settings-2fa-totp-title = App di autenticazione
settings-2fa-totp-enrol = Configura un'app di autenticazione
settings-2fa-totp-scan = Aggiungere questo segreto all'app di autenticazione, quindi inserire il codice che mostra.
settings-2fa-totp-uri-link = Apri in un'app di autenticazione
settings-2fa-totp-code-label = Codice di autenticazione
settings-2fa-totp-confirm = Conferma
settings-2fa-totp-disable = Rimuovi app di autenticazione
settings-2fa-passkey-title = Passkey
settings-2fa-passkey-remove = Rimuovi
settings-2fa-passkey-unsupported = Questo browser non supporta le passkey.
settings-2fa-passkey-label-placeholder = Nome della passkey (facoltativo)
settings-2fa-passkey-add = Aggiungi una passkey
settings-2fa-recovery-remaining = { $count ->
        [one] { $count } codice di recupero rimanente
       *[other] { $count } codici di recupero rimanenti
    }
settings-2fa-recovery-regenerate = Genera nuovi codici di recupero
settings-2fa-recovery-code-label = Codice di recupero
settings-2fa-error-generic = Operazione non riuscita. Riprovare.
settings-2fa-error-code = Il codice non è stato verificato.
settings-2fa-error-passkey = Impossibile registrare la passkey.

settings-2fa-challenge-title = Autenticazione a due fattori
settings-2fa-challenge-intro = Confermare il secondo fattore per completare l'accesso.
settings-2fa-challenge-failed = Verifica non riuscita. Riprovare.
settings-2fa-challenge-use-passkey = Usa una passkey
settings-2fa-challenge-method = Scegli un metodo
settings-2fa-challenge-totp-tab = Codice di autenticazione
settings-2fa-challenge-recovery-tab = Codice di recupero
settings-2fa-challenge-verify = Verifica

settings-sessions-title = Sessioni attive
settings-sessions-intro = Sessioni attualmente collegate a questo account.
settings-sessions-last-seen = Ultima attività { $when }
settings-sessions-revoke = Disconnetti
settings-sessions-current = Questa sessione
settings-sessions-revoke-others = { $count ->
        [one] Disconnetti { $count } altra sessione
       *[other] Disconnetti { $count } altre sessioni
    }
settings-sessions-error = Impossibile modificare la sessione.

settings-sig-title = Firme
settings-sig-intro = Testo aggiunto ai messaggi inviati. Una firma può essere quella predefinita.
settings-sig-default = Predefinita
settings-sig-new = Nuova firma
settings-sig-name-label = Nome
settings-sig-body-label = Firma
settings-sig-default-label = Usa come predefinita
settings-sig-error-name = Assegnare un nome alla firma.
settings-sig-error-generic = Impossibile salvare la firma.

settings-ident-title = Identità
settings-ident-intro = Il nome e l'indirizzo con cui si invia. Ogni identità può usare una propria firma.
settings-ident-new = Nuova identità
settings-ident-name-label = Nome visualizzato
settings-ident-email-label = Indirizzo email
settings-ident-replyto-label = Indirizzo di risposta (facoltativo)
settings-ident-signature-label = Firma predefinita
settings-ident-signature-none = Nessuna
settings-ident-error-fields = Inserire un nome e un indirizzo email valido.
settings-ident-error-generic = Impossibile salvare l'identità.

settings-notif-title = Notifiche
settings-notif-intro = Scegliere quali nuovi messaggi generano una notifica e quando non disturbare.
settings-notif-enabled-label = Mostra notifiche per la nuova posta
settings-notif-quiet-title = Ore di silenzio
settings-notif-quiet-enabled-label = Silenzia le notifiche in un orario stabilito
settings-notif-quiet-start = Da
settings-notif-quiet-end = A
settings-notif-rules-title = Regole
settings-notif-rule-match-placeholder = Mittente, casella o oggetto contiene…
settings-notif-rule-action-label = Azione
settings-notif-rule-notify = Notifica
settings-notif-rule-mute = Silenzia
settings-notif-rule-add = Aggiungi una regola
settings-notif-error = Impossibile salvare le impostazioni delle notifiche.

settings-search-title = Ricerche salvate
settings-search-intro = Mostrare una ricerca salvata come cartella nell'elenco delle caselle.
settings-search-empty = Non ci sono ancora ricerche salvate.
settings-search-as-folder = Mostra { $name } come cartella
settings-search-as-folder-label = Mostra come cartella
settings-search-error = Impossibile modificare la ricerca salvata.

settings-kbd-title = Scorciatoie da tastiera
settings-kbd-intro = Scegliere un set di scorciatoie. Le modifiche si applicano a questo dispositivo.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Disattivate (nessuna scorciatoia)
settings-kbd-action-compose = Scrivi
settings-kbd-action-archive = Archivia
settings-kbd-action-reply = Rispondi
settings-kbd-action-next = Messaggio successivo
settings-kbd-action-previous = Messaggio precedente
settings-kbd-action-search = Cerca

settings-offline-title = Cache offline
settings-offline-intro = Quanta posta conservare su questo dispositivo per l'uso offline e come recuperare spazio.
settings-offline-budget-label = Spazio per la cache (MB)
settings-offline-retention-label = Conserva per (giorni)
settings-offline-strategy-label = Quando lo spazio è esaurito
settings-offline-lru = Rimuovi i meno usati di recente
settings-offline-oldest = Rimuovi prima i più vecchi
settings-offline-manual = Solo quando la svuoto io
settings-offline-purge = Svuota ora la cache offline

settings-dir-title = Direzione dell'interfaccia
settings-dir-intro = Seguire la lingua o forzare una direzione. Da destra a sinistra rispecchia il layout.
settings-dir-auto = Automatica
settings-dir-ltr = Da sinistra a destra
settings-dir-rtl = Da destra a sinistra
settings-dir-preview-title = Anteprima
settings-dir-preview-body = Questo testo si rispecchia quando la direzione è da destra a sinistra.
settings-mail-features = Funzioni di posta
settings-mail-feature-allowSend = Consenti l'invio dei messaggi
settings-mail-feature-allowSend-help = Se disattivato, Invia e la sua scorciatoia sono disabilitati. Le bozze possono comunque essere salvate.
settings-mail-feature-allowReply = Consenti Rispondi e Rispondi a tutti
settings-mail-feature-allowReply-help = Se disattivato, le azioni Rispondi e Rispondi a tutti e le relative scorciatoie non vengono proposte.
settings-mail-feature-allowForward = Consenti l'inoltro
settings-mail-feature-allowForward-help = Se disattivato, le azioni Inoltra e la relativa scorciatoia non vengono proposte.
settings-mail-feature-allowDownload = Consenti il download di messaggi e allegati
settings-mail-feature-allowDownload-help = Se disattivato, il download di messaggi e allegati non viene proposto.
settings-mail-feature-allowPrint = Consenti la stampa
settings-mail-feature-allowPrint-help = Se disattivato, l'azione Stampa non viene proposta.
settings-mail-feature-showSpam = Mostra la cartella Spam
settings-mail-feature-showSpam-help = Se disattivato, la cartella Spam e i messaggi contrassegnati come spam sono nascosti dall'elenco delle cartelle e dai risultati di ricerca. Non viene eliminato nulla.
settings-mail-feature-showAuthStatus = Mostra i risultati di autenticazione del mittente
settings-mail-feature-showAuthStatus-help = Mostra nel lettore i risultati SPF, DKIM e DMARC di un messaggio.
settings-mail-feature-closeOnDelete = Chiudi un messaggio quando viene eliminato
settings-mail-feature-closeOnDelete-help = Chiude il messaggio, la scheda o la finestra aperti quando quel messaggio viene eliminato o spostato nel Cestino.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Percorso delle impostazioni
settings-crumb-loading = Caricamento del nome
settings-crumb-new = Nuovo
settings-crumb-edit = Modifica
settings-badge-color = Colore del badge dei non letti
settings-badge-color-accent = Uguale all'accento
settings-badge-color-accent-lighter = Accento più chiaro
settings-badge-color-accent-darker = Accento più scuro
settings-badge-color-option = Badge { $color }
settings-appearance-theme-card = Tema e cambio
settings-appearance-theme-card-help = Scegliere un tema e se segue il sistema o l'ora del giorno.
settings-appearance-colours-card = Colori
settings-appearance-colours-card-help = L'accento usato per azioni e selezione e il colore dei contatori dei non letti.
settings-appearance-text-card = Testo e layout
settings-appearance-text-card-help = Quanto è densa l'interfaccia, quale carattere usa e come viene disposta la posta.
settings-sig-create-variant = Crea variante
settings-sig-error-existing = Esiste già una firma con questo nome. Scegliere un nome diverso.
settings-sig-save-failed = Impossibile salvare la firma. { $reason }
settings-ident-signature-new = Nuovi messaggi
settings-ident-signature-account = Usa la firma dell'account
settings-ident-signature-replies = Risposte
settings-ident-signature-inherit = Usa la firma dei nuovi messaggi
settings-ident-signature-unavailable = Firma non disponibile: { $name }
settings-ident-signature-help = Creare le varianti di firma in Firme, poi assegnarne una qui. La stessa variante può essere usata per entrambi.
settings-ident-save-failed = Impossibile salvare l'identità. { $reason }
settings-search-save-failed = Impossibile salvare la ricerca salvata. { $reason }
settings-kbd-preset-title = Set di scorciatoie
settings-editor-discard-title = Scartare le modifiche non salvate?
settings-editor-discard-detail = Le modifiche non sono state salvate. Continuare a modificare oppure scartare questa bozza per uscire dall'editor.
settings-editor-keep = Continua a modificare
settings-editor-discard = Scarta le modifiche
settings-editor-missing = Questo elemento non esiste più. Tornare all'elenco e scegliere un altro elemento.
settings-editor-back = Torna all'elenco
settings-sig-edit-title = Modifica firma
settings-ident-edit-title = Modifica identità
settings-sections = Sezioni delle impostazioni
settings-category-account = Account
settings-category-mail = Posta
settings-category-interface = Interfaccia
settings-category-integrations = Integrazioni
settings-category-about = Informazioni
settings-nav-preferences = Tastiera e visualizzazione
settings-nav-rules = Regole di posta
settings-nav-access = Accesso API
settings-nav-metadata = Informazioni sul server
settings-folders-title = Cartelle e barra laterale
settings-folders-help = Fissare le cartelle sopra le altre, scegliere un'icona e trascinare o usare i pulsanti di spostamento per ordinare le cartelle all'interno di ciascun gruppo. Queste scelte si sincronizzano con questo account e non spostano messaggi né rinominano cartelle.
settings-folders-pin = Fissa { $name }
settings-folders-icon = Icona per { $name }
settings-folders-automatic = Ruolo automatico della cartella
settings-folders-drag = Trascinare { $name } per riordinare
settings-folders-up = Sposta { $name } in alto
settings-folders-down = Sposta { $name } in basso
settings-folders-sync-idle = Accedere per sincronizzare le preferenze delle cartelle.
settings-folders-sync-loading = Caricamento delle preferenze dell'account…
settings-folders-sync-saving = Salvataggio delle preferenze dell'account…
settings-folders-sync-synced = Preferenze sincronizzate con questo account.
settings-folders-sync-error = Impossibile sincronizzare le preferenze. Le modifiche restano in questa sessione e verranno riprovate al ritorno della connessione.
settings-folder-icon-folder = Cartella
settings-folder-icon-star = Stella
settings-folder-icon-flag = Bandiera
settings-folder-icon-tag = Etichetta
settings-folder-icon-clock = Orologio
settings-folder-icon-calendar = Calendario
settings-folder-icon-file = Documento
settings-folder-icon-image = Immagine
settings-folder-icon-lock = Lucchetto
settings-folder-icon-users = Persone
settings-folder-icon-archive = Archivio
settings-folder-icon-mail = Busta
settings-folder-icon-attachment = Allegato
settings-folder-icon-check = Segno di spunta
settings-folder-icon-alert = Avviso
settings-search-new = Nuova cartella di ricerca
settings-search-editor = Cartella di ricerca
settings-search-editor-help = I messaggi devono soddisfare tutte le condizioni. Lasciare vuota una condizione per ignorarla. Le cartelle di ricerca mostrano i messaggi corrispondenti senza spostarli né copiarli.
settings-search-name = Nome della cartella
settings-search-name-required = Inserire un nome per questa cartella di ricerca.
settings-search-invalid = Questa ricerca salvata ha un filtro non valido. Non può essere modificata in sicurezza.
settings-search-saved = Cartella di ricerca “{ $name }” salvata.
settings-search-field-text = Cerca in tutto il testo del messaggio
settings-search-field-from = Il mittente contiene
settings-search-field-to = Il destinatario contiene
settings-search-field-subject = L'oggetto contiene
settings-search-field-body = Il corpo del messaggio contiene
settings-search-field-filename = Il nome file dell'allegato contiene
settings-search-field-inMailbox = Limita all'identificatore della casella (facoltativo)
settings-search-field-after = Ricevuto dopo (data e ora UTC)
settings-search-field-before = Ricevuto prima di (data e ora UTC)
settings-search-attachment = Allegati
settings-search-any = Qualsiasi messaggio
settings-search-with-attachments = Con allegati
settings-search-without-attachments = Senza allegati
settings-search-unread = Solo messaggi non letti
settings-search-flagged = Solo messaggi contrassegnati
settings-sessions-search = Cerca account, dispositivo o IP
settings-sessions-account = Account
settings-sessions-device = Dispositivo e client
settings-sessions-location = IP e ultimo paese noto
settings-sessions-location-unavailable = Paese non disponibile
settings-sessions-activity = Attività
settings-sessions-actions = Azioni
settings-sessions-revoke-device = Disconnetti { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Scrittura e risposte
settings-quote-style = Stile dell'intestazione di risposta e inoltro
settings-quote-classic = Risposta citata classica
settings-quote-outlook = Intestazione in stile Outlook
settings-quote-help = Lo stile Outlook aggiunge Da, Inviato, A, Cc e Oggetto sopra il messaggio originale, sia in testo formattato sia in testo semplice. Le bozze esistenti non cambiano.
settings-external-warning = Avvisi per destinatari esterni
settings-external-off = Disattivato
settings-external-warn = Mostra un avviso
settings-external-confirm = Richiedi conferma prima dell'invio
settings-external-help = Confronta A, Cc e Ccn con il dominio dell'identità selezionata e con l'elenco dei domini attendibili. Protegge gli invii da questo editor; non è una regola di consegna del server.
settings-trusted-domains = Domini dei destinatari attendibili
settings-trusted-domains-help = Inserire fino a 100 domini esatti, uno per riga. I sottodomini vanno elencati separatamente. Il dominio dell'identità mittente è sempre attendibile. URL, indirizzi email e caratteri jolly non sono accettati.
settings-trusted-domains-invalid = Inserire fino a 100 nomi di dominio validi, senza URL, indirizzi email né caratteri jolly.
settings-trusted-domains-save = Salva domini attendibili
settings-shortcuts-enabled = Abilita le scorciatoie dell'applicazione
settings-shortcuts-action = Azione
settings-shortcuts-binding = Assegnazione
settings-shortcuts-edit = Modifica scorciatoia
settings-shortcuts-edit-action = Modifica la scorciatoia di { $action }
settings-shortcuts-help = Usare inherit per il set selezionato, disabled per nessuna scorciatoia, oppure una combinazione come Ctrl+Shift+K. Ctrl corrisponde a Cmd su macOS. Le assegnazioni in conflitto sono inattive. I campi di testo e le finestre di dialogo mantengono i tasti consueti; le combinazioni riservate al browser potrebbero non essere disponibili.
settings-shortcuts-invalid = Inserire inherit, disabled oppure un singolo tasto valido con i modificatori facoltativi Ctrl, Alt e Maiusc.
settings-shortcuts-conflict = Due azioni usano la stessa scorciatoia. Modificare un'assegnazione; le scorciatoie in conflitto non verranno eseguite.
settings-shortcuts-discard = Scartare la modifica non salvata della scorciatoia?
settings-shortcuts-save = Salva scorciatoia
settings-shortcuts-back = Torna alle scorciatoie
settings-shortcuts-off = Il set Disattivato disabilita ogni scorciatoia, comprese quelle personalizzate. Scegliere un altro set in Preferenze per usare queste assegnazioni.
settings-kbd-action-replyAll = Rispondi a tutti
settings-kbd-action-forward = Inoltra
settings-kbd-action-refresh = Aggiorna i messaggi
settings-kbd-action-settings = Apri le impostazioni
settings-kbd-action-trash = Sposta nel Cestino
settings-kbd-action-markRead = Segna come letto

# -- Machine translation (2026-11): added to match en ---------------------
settings-kbd-action-markUnread = Segna come non letto
settings-kbd-action-spam = Segna come spam
settings-kbd-action-zoomIn = Ingrandisci il messaggio
settings-kbd-action-zoomOut = Riduci il messaggio
settings-kbd-action-zoomReset = Reimposta lo zoom del messaggio
settings-kbd-action-renameFolder = Rinomina cartella
settings-kbd-action-openInTab = Apri il messaggio in una scheda a sé
settings-notif-device-title = Autorizzazione su questo dispositivo
settings-notif-device-help = Le preferenze di notifica seguono l'account. L'autorizzazione del browser appartiene a questo dispositivo; Mailwoman la richiede solo quando si sceglie di consentirla.
settings-notif-ask-permission = Proponi l'autorizzazione alle notifiche sui nuovi dispositivi
settings-notif-ask-permission-help = Mostra una richiesta di autorizzazione quando questo dispositivo non ha ancora consentito né bloccato le notifiche. Attivo per impostazione predefinita; questa preferenza viene salvata subito.
settings-notif-permission-default = L'autorizzazione del browser non è stata ancora richiesta.
settings-notif-permission-granted = Le notifiche del browser sono consentite su questo dispositivo.
settings-notif-permission-denied = Le notifiche sono bloccate. Modificare l'autorizzazione di questo sito nelle impostazioni del browser per consentirle.
settings-notif-permission-unsupported = Questo browser non supporta qui le notifiche di sistema.
settings-notif-permission-request = Consenti le notifiche su questo dispositivo
settings-notif-permission-error = Il browser non ha potuto richiedere l'autorizzazione alle notifiche.
settings-notif-timezone = Fuso orario delle ore di silenzio (IANA)
settings-notif-timezone-device = Usa il fuso orario di questo dispositivo
settings-permissions-background-title = Notifiche in background in questo browser
settings-permissions-background-help = Salvare prima le preferenze di notifica e consentire le notifiche del browser. Registrare questo browser per ricevere l'attività dell'account quando l'app è chiusa.
settings-permissions-push-unsupported = Il Web Push in background non è disponibile in questo browser.
settings-permissions-push-registered = Questo browser ha una sottoscrizione push. Registrarla per associarla all'account corrente.
settings-permissions-push-unregistered = Questo browser non ha una sottoscrizione push.
settings-permissions-push-register = Registra le notifiche in background
settings-permissions-push-remove = Rimuovi la sottoscrizione del browser
settings-permissions-push-policy = L'amministratore non consente le notifiche push in background.
settings-permissions-storage-title = Archiviazione del browser
settings-permissions-storage-help = L'archiviazione persistente protegge i dati locali di questo sito dalla rimozione automatica quando il browser la concede. Riguarda solo questo dispositivo e non modifica la conservazione sul server. I dati del sito possono essere cancellati nelle impostazioni del browser; le modifiche offline non inviate potrebbero andare perse.
settings-permissions-storage-unsupported = Le richieste di archiviazione persistente non sono disponibili in questo browser.
settings-permissions-storage-persistent = Il browser ha concesso l'archiviazione persistente per questo sito.
settings-permissions-storage-evictable = Il browser può rimuovere automaticamente i dati locali di questo sito.
settings-permissions-storage-usage = Archiviazione attuale del sito: { $mib } MiB.
settings-permissions-storage-request = Richiedi l'archiviazione persistente
settings-permissions-storage-policy = L'amministratore non consente le richieste di archiviazione persistente.
settings-permissions-error = Impossibile completare l'azione del browser. Controllare le impostazioni salvate dell'account e le autorizzazioni del browser per questo sito.
settings-notif-background-rules = Le regole sui contenuti vengono valutate mentre Mailwoman è aperto. Le notifiche di sistema in background sono soppresse finché sono configurate regole sui contenuti, perché un risveglio opaco in background non contiene i dettagli del messaggio.
settings-time-title = Fusi orari
settings-time-help = Scegliere come vengono mostrati gli orari dei messaggi e, facoltativamente, un fuso orario di confronto per la pianificazione. Le località non vengono mai dedotte dagli indirizzi email.
settings-time-displayZone = Fuso orario di visualizzazione
settings-time-displayZone-help = Usare local per questo dispositivo oppure un nome IANA come Europe/Lisbon.
settings-time-comparisonZone = Fuso orario di confronto
settings-time-comparisonZone-help = Inserire un nome IANA come America/New_York, oppure lasciare vuoto per nascondere i confronti.
settings-time-helpers = Mostra gli aiuti per i fusi orari
settings-time-save = Salva fuso orario
settings-time-invalid = Inserire un fuso orario IANA valido. Usare local solo per il fuso di visualizzazione, oppure lasciare vuoto il fuso di confronto.
settings-time-discard = Scartare le modifiche non salvate al fuso orario?
settings-folders-name = Cartella
settings-folders-actions = Azioni
settings-folders-edit = Modifica l'aspetto di { $name }
settings-folders-icon-help = Scegliere una tra cinquanta icone a tema, oppure mantenere la selezione automatica in base allo scopo della cartella. Segue l'account e non rinomina la casella.
settings-folders-save = Salva l'aspetto della cartella
settings-folders-back = Torna alle cartelle
settings-folders-discard = Scartare le modifiche non salvate all'aspetto della cartella?
settings-folder-labels = Etichette della cartella
settings-folder-labels-help = Organizzare questa cartella con un massimo di 32 etichette dell'account. Non etichetta né sposta i messaggi al suo interno. Gestire nomi e colori delle etichette in Etichette.
settings-folders-leave-pending = Queste modifiche sono ancora in coda per la sincronizzazione. Uscire da questo editor?
settings-folders-retry = Riprova a salvare
settings-folder-role-title = Cartelle speciali
settings-folder-role-help = Scegliere la cartella reale usata per ogni ruolo di posta. Automatico usa il rilevamento del server. Usa il predefinito dell'amministratore rimuove la sostituzione. Destinazioni in conflitto o mancanti disattivano quel ruolo invece di scegliere un'altra cartella.
settings-folder-role-auto = Rilevamento automatico
settings-folder-role-inherit = Usa il predefinito dell'amministratore
settings-folder-role-path = Percorso configurato: { $path }
settings-folder-role-missing = La cartella configurata non è disponibile
settings-folder-role-conflict = Questa destinazione manca o è in conflitto con un altro ruolo. Quel ruolo non è disponibile finché l'associazione non viene corretta.
settings-folder-role-refresh-error = Salvato, ma non è stato possibile aggiornare l'elenco delle cartelle. Aggiornare le cartelle prima di usare la nuova associazione.
settings-folder-role-unsupported = Queste sostituzioni richiedono cartelle di posta native. Gli account JMAP upstream mantengono i ruoli delle cartelle del proprio server.
settings-folder-role-inbox = Posta in arrivo
settings-folder-role-drafts = Bozze
settings-folder-role-sent = Posta inviata
settings-folder-role-trash = Cestino
settings-folder-role-junk = Posta indesiderata e spam
settings-folder-role-archive = Archivio
settings-sessions-select = Seleziona
settings-sessions-select-device = Seleziona la sessione di { $device }
settings-sessions-select-visible = Seleziona tutte le altre sessioni visibili
settings-sessions-current-protected = Questa sessione resta connessa e non può essere selezionata qui. Usare Esci per terminarla.
settings-sessions-selected-title = Disconnetti le sessioni selezionate ({ $count })
settings-sessions-selected-confirm = Termina solo le altre sessioni selezionate. Questa sessione resta connessa. Le richieste non riuscite o saltate verranno segnalate, senza nuovi tentativi automatici.
settings-sessions-selected-action = Disconnetti le sessioni selezionate
settings-sessions-cancel = Annulla
settings-sessions-selected-result = Disconnesse: { $revoked }. Già terminate: { $unchanged }. Non è stato possibile confermare: { $failed }. Non tentate: { $skipped }.
settings-sessions-refresh-failed = Non è stato possibile aggiornare l'elenco delle sessioni. Ricaricarlo prima di selezionare altre sessioni.
settings-sig-format = Formato della firma
settings-sig-format-plain = Testo semplice
settings-sig-format-html = Testo formattato (HTML)
settings-sig-html-help = L'HTML è limitato a 2 MiB e viene ripulito prima del salvataggio e dell'invio. Le immagini remote e i contenuti attivi vengono rimossi.
settings-sig-html-source = Modifica il sorgente HTML
settings-sig-editor-loading = Caricamento dell'editor delle firme…
settings-sig-rich-body = Firma formattata
settings-sig-plain-alternative = Alternativa in testo semplice
settings-sig-plain-help = I messaggi in testo semplice usano questo testo. Le modifiche al testo formattato lo aggiornano finché non viene personalizzato; le modifiche al sorgente HTML conservano l'alternativa.
settings-sig-account-changed = L'account connesso è cambiato. Riaprire questa firma nell'account corretto.
settings-sig-image-type = { $name } non è un'immagine PNG, JPEG, GIF o WebP.
settings-sig-image-size = { $name } non ci sta: l'HTML di una firma, immagini incluse, è limitato a 2 MiB.
settings-sig-too-large = Questa firma supera i 2 MiB di HTML. Rimuovere un'immagine o parte della formattazione.
settings-sig-images-help = Le immagini sono memorizzate nella firma e inviate come immagini inline di ogni messaggio.
settings-folder-icon-inbox = Posta in arrivo
settings-folder-icon-sent = Inviati
settings-folder-icon-edit = Modifica
settings-folder-icon-delete = Elimina
settings-folder-icon-spam = Spam
settings-folder-icon-search = Cerca
settings-folder-icon-pin = Puntina
settings-folder-icon-link = Link
settings-folder-icon-shield = Scudo
settings-folder-icon-unlock = Lucchetto aperto
settings-folder-icon-fileAudio = Audio
settings-folder-icon-fileVideo = Video
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Codice
settings-folder-icon-print = Stampa
settings-folder-icon-download = Download
settings-folder-icon-preferences = Preferenze
settings-folder-icon-dashboard = Dashboard
settings-folder-icon-analytics = Analisi
settings-folder-icon-nextcloud = File nel cloud
settings-folder-icon-domains = Domini
settings-folder-icon-twofactor = Autenticazione
settings-folder-icon-assistant = Assistente
settings-folder-icon-integrations = Integrazioni
settings-folder-icon-logs = Log
settings-folder-icon-home = Home
settings-folder-icon-briefcase = Valigetta
settings-folder-icon-heart = Cuore
settings-folder-icon-shopping = Acquisti
settings-folder-icon-education = Istruzione
settings-folder-icon-travel = Viaggi
settings-folder-icon-coffee = Caffè
settings-folder-icon-gift = Regalo
settings-folder-icon-bank = Banca
settings-folder-icon-book = Libro
settings-nav-keyboard-display = Tastiera e visualizzazione
settings-2fa-passkey-policy-disabled = L'amministratore ha disattivato l'accesso e la registrazione con passkey per questo account. Le credenziali esistenti restano elencate per poterle gestire o rimuovere.
settings-mail-interactions = Selezione e spostamento dei messaggi
settings-mail-modifier-selection = Seleziona i messaggi con i tasti modificatori
settings-mail-modifier-selection-help = Ctrl o Command attiva o disattiva singoli messaggi. Maiusc seleziona un intervallo e Maiusc con i tasti freccia lo estende. Il pulsante di selezione visibile resta disponibile quando questa opzione è disattivata.
settings-mail-drag-folder = Trascina i messaggi nelle cartelle
settings-mail-drag-folder-help = Trascinare un messaggio selezionato per spostare la selezione, oppure un messaggio non selezionato per spostare solo quello. Sono accettate solo le cartelle di questo account. Chi usa il tocco o la tastiera può usare la barra degli strumenti di selezione.
settings-search-sections = Cerca nelle impostazioni
settings-search-no-sections = Nessuna sezione delle impostazioni corrispondente.
settings-time-local = Fuso orario locale del dispositivo
settings-time-none = Nessun fuso orario di confronto
settings-folder-counts = Conteggi delle cartelle
settings-folder-counts-none = Nascondi i conteggi
settings-folder-counts-unread = Messaggi non letti
settings-folder-counts-total = Messaggi totali
settings-folder-counts-both = Messaggi non letti e totali
settings-folder-counts-unreadIfAny = Messaggi non letti, solo se presenti
settings-folder-count-scope = Cartelle con conteggi
settings-folder-count-scope-all = Tutte le cartelle
settings-folder-count-scope-main = Solo le cartelle principali
settings-folder-count-scope-help = Le cartelle principali sono Posta in arrivo, Bozze, Inviati, Archivio, Posta indesiderata, Cestino, Posta in uscita e altre cartelle di uso speciale. Le cartelle personalizzate e le relative sottocartelle non mostrano alcun conteggio.
settings-account-unread-badge = Mostra il numero di non letti accanto al nome della casella
settings-folder-counts-compact = Badge più piccoli fuori dalla Posta in arrivo
settings-focused-inbox = Posta in arrivo con priorità
settings-focused-inbox-help = Divide la Posta in arrivo nelle schede Priorità e Altro sopra l'elenco. I messaggi di newsletter e mittenti automatici vanno in Altro. Le altre cartelle e i risultati di ricerca mostrano tutti i messaggi.
settings-focused-senders-title = Gestisci i mittenti di Priorità
settings-focused-senders-help = Mittenti che si è scelto di spostare sempre in Priorità o in Altro. Rimuoverne uno per lasciare di nuovo decidere l'ordinamento automatico.
settings-focused-senders-empty = Nessun mittente per ora. Usare Sposta sempre in Priorità o Sposta sempre in Altro su un messaggio.

# -- Machine translation (2026-11): added to match en ---------------------
settings-focused-senders-to-focused = Sempre in Priorità
settings-focused-senders-to-other = Sempre in Altro
settings-focused-senders-remove = Rimuovi
settings-focused-senders-remove-label = Rimuovi { $sender }
settings-focused-senders-remove-failed = Impossibile rimuovere il mittente. Riprovare.
settings-folder-counts-reset = Eredita i conteggi delle cartelle
settings-folder-counts-reset-size = Eredita la dimensione dei badge
settings-mail-resume-tabs = Riprendi le schede aperte
settings-mail-resume-tabs-help = Ripristina i riferimenti alle schede dell'account alla riapertura di Mailwoman. I corpi dei messaggi e gli URL degli allegati non vengono salvati nel record dell'area di lavoro.
settings-row-actions-mode = Azioni rapide dell'elenco dei messaggi
settings-row-actions-mode-open = Sempre visibili
settings-row-actions-mode-hover = Mostra al passaggio del mouse
settings-row-actions-mode-click = Mostra al clic
settings-row-actions-mode-off = Disattivato
settings-row-actions-mode-help = Come compaiono le azioni rapide su ogni riga di messaggio. Al passaggio del mouse mostra tre controlli e un pulsante Altro; al clic mostra un pulsante che li apre; sugli schermi touch il passaggio del mouse si comporta come il clic. Disattivato le rimuove dalle righe (il menu contestuale e la barra degli strumenti hanno comunque tutte le azioni).
settings-mail-feature-showGoToTop = Mostra un pulsante Vai in alto nell'elenco dei messaggi
settings-mail-feature-showGoToTop-help = Aggiunge un pulsante che riporta l'elenco dei messaggi al primo messaggio. Disattivato per impostazione predefinita.
settings-mail-feature-showCreateFolderButton = Mostra un pulsante Crea cartella accanto a Scrivi
settings-mail-feature-showCreateFolderButton-help = Aggiunge nella barra laterale un pulsante con sola icona che crea una cartella. Disattivato per impostazione predefinita.
settings-draft-save = Bozze dell'account
settings-draft-save-help = Salva nella cartella Bozze dell'account per accedervi da un altro dispositivo. Il salvataggio è silenzioso; solo gli errori mostrano una notifica. Gli account non supportati e i contenuti decifrati protetti non vengono mai caricati in silenzio.
settings-draft-save-mode = Salva automaticamente
settings-draft-realtime = Dopo una pausa di digitazione
settings-draft-periodic = A intervalli regolari
settings-draft-manual = Solo quando si sceglie Salva
settings-draft-debounce = Pausa di digitazione (millisecondi)
settings-draft-interval = Intervallo di salvataggio (secondi)
settings-permissions-push-other-registration = Questo browser ha una sottoscrizione non registrata per questo account. Rimuoverla prima di registrare di nuovo.
settings-show-from = Mostra Da durante la scrittura
settings-show-from-help = Iniziare con il campo Da visibile. È possibile mostrarlo anche per un solo messaggio dalla barra multifunzione. Sono accettati solo gli indirizzi di invio autorizzati.
settings-sessions-table = Tabella delle sessioni attive
settings-folders-table = Elenco delle cartelle
settings-kbd-reference = Visualizza le assegnazioni del set
settings-kbd-reference-help = Solo riferimento del set. Le assegnazioni personalizzate e la policy dell'amministratore possono modificare le scorciatoie attive.
settings-sessions-client-details = Dettagli del client
settings-2fa-passkey-rules = L'organizzazione accetta solo passkey che:
settings-2fa-passkey-rule-platform = sono memorizzate su questo dispositivo (Windows Hello, Touch ID, blocco schermo di Android)
settings-2fa-passkey-rule-cross-platform = si trovano su una chiave di sicurezza o su un telefono
settings-2fa-passkey-rule-user-verification = verificano PIN, impronta digitale o volto
settings-2fa-passkey-rule-resident-key = sono memorizzate sull'autenticatore (individuabili)
settings-2fa-passkey-gap-allowed = Aggiunta prima della policy attuale e non la soddisfa. Funziona ancora; aggiungere una nuova passkey e rimuovere questa.
settings-2fa-passkey-gap-refused = Non soddisfa la policy attuale e non è più accettata per l'accesso. Aggiungere una nuova passkey e rimuovere questa.
settings-2fa-passkey-browser-refused-policy = Il browser non ha trovato alcun autenticatore conforme alla policy sulle passkey dell'organizzazione, oppure la richiesta è stata annullata.
settings-2fa-policy-user-verification = L'organizzazione richiede una passkey che verifichi PIN, impronta digitale o volto. Questo autenticatore non ha verificato l'identità.
settings-2fa-policy-platform = L'organizzazione accetta solo passkey memorizzate su questo dispositivo (come Windows Hello, Touch ID o blocco schermo di Android). Chiavi di sicurezza e telefoni non sono accettati.
settings-2fa-policy-cross-platform = L'organizzazione accetta solo chiavi di sicurezza o passkey su telefono. Le passkey memorizzate su questo dispositivo non sono accettate.
settings-2fa-policy-attachment-unknown = Il browser non ha indicato di che tipo di autenticatore si tratti, quindi non è possibile verificarlo rispetto alla policy sulle passkey dell'organizzazione. Provare con un altro browser.
settings-2fa-policy-resident-key = L'organizzazione richiede una passkey memorizzata sull'autenticatore. Questo autenticatore non ne ha memorizzata una.
settings-2fa-policy-legacy = Questa passkey è stata aggiunta prima dell'attuale policy sulle passkey dell'organizzazione e non la soddisfa più. Accedere con un altro fattore o con un codice di ripristino, poi aggiungere una nuova passkey.
settings-2fa-policy-last-factor-blocked = L'organizzazione non consente di rimuovere l'ultimo secondo fattore. Aggiungere prima un altro metodo di accesso.
settings-2fa-policy-last-factor-required = L'organizzazione richiede l'autenticazione a due fattori. Aggiungere un altro metodo di accesso prima di rimuovere questo.
settings-2fa-last-factor-hint = Aggiungere prima un altro metodo di accesso.
settings-2fa-last-factor-tooltip = Aggiungere prima un altro metodo di accesso. L'organizzazione richiede almeno una passkey o un'app di autenticazione sull'account.
settings-2fa-challenge-passkey-not-used = La passkey non è stata usata: la richiesta è stata annullata o è scaduta, oppure questo autenticatore non può fare ciò che l'organizzazione richiede (come la verifica di PIN, impronta digitale o volto). Riprovare o usare un altro metodo.
settings-folders-hidden-help = Le cartelle nascoste e le relative sottocartelle restano fuori dalla barra laterale. Non vengono eliminate né soggette a restrizioni di accesso. Mostrare qui una sottocartella per mostrare anche le cartelle superiori nascoste. La Posta in arrivo resta sempre disponibile.
settings-folders-collapse-user = Comprimi le cartelle utente nella barra laterale
settings-folders-show = Mostra { $name } nella barra laterale
settings-folders-hide = Nascondi { $name } dalla barra laterale
settings-folders-show-short = Mostra
settings-folders-hide-short = Nascondi
settings-folders-search = Cerca cartelle
settings-folders-select = Seleziona cartella: { $name }
settings-folders-selected = Cartelle selezionate: { $count }
settings-folders-select-visible = Seleziona visibili
settings-folders-select-all = Seleziona tutte le cartelle
settings-folders-clear = Cancella selezione
settings-folders-pin-selected = Fissa selezionate
settings-folders-unpin-selected = Sblocca selezionate
settings-folders-show-selected = Mostra selezionate
settings-folders-hide-selected = Nascondi selezionate
settings-folders-visible = Visibili
settings-folders-hidden = Nascoste
settings-folders-pinned = Fissate
settings-folders-location = Posizione
settings-folders-counts = { $unread } non lette / { $total } totali
settings-folders-more = Altre azioni per { $name }
settings-folders-empty = Nessuna cartella corrisponde alla ricerca.
settings-folders-root = Livello superiore
settings-folders-order-help = Trascinare le cartelle per riordinarle nella barra laterale. Le cartelle fissate restano insieme.
cache-title = Cache della posta e precaricamento
cache-help = Mantiene le cartelle e i messaggi visitati di recente nella memoria di questa scheda. Il contenuto in cache viene aggiornato in background. Chiudere la scheda o uscire lo cancella. Il precaricamento non segna mai la posta come letta né carica immagini remote.
cache-enabled = Abilita la cache di sessione
cache-mailboxes = Memorizza in cache gli elenchi dei messaggi delle cartelle
cache-messages = Memorizza in cache i messaggi aperti
cache-freshSeconds = Intervallo di aggiornamento (secondi)
cache-maxAgeSeconds = Età massima in cache (secondi)
cache-maxEntries = Numero massimo di voci in cache
cache-maxBytes = Limite di memoria (byte)
cache-prefetchEnabled = Abilita il precaricamento speculativo
cache-hoverEnabled = Precarica quando il puntatore è su una cartella
cache-focusEnabled = Precarica quando una cartella riceve il focus della tastiera
cache-dwellMs = Ritardo di puntatore e focus (millisecondi)
cache-idleEnabled = Precarica le cartelle in inattività
cache-idleDelayMs = Ritardo di inattività (millisecondi)
cache-maxConcurrency = Numero massimo di precaricamenti simultanei
cache-maxQueued = Numero massimo di precaricamenti in coda
cache-preloadMessages = Consenti corpi dei messaggi speculativi
cache-excludedFolders = Cartelle escluse dal precaricamento
cache-exclusions-help = Una regola per riga: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive, oppure id: seguito dall'ID esatto di una cartella.
cache-refreshOnOpen = Aggiorna il contenuto in cache all'apertura
cache-clear = Cancella ora la posta in cache
settings-compose-window = Apri i nuovi messaggi in
settings-compose-window-tab = Una scheda dell'area di lavoro
settings-compose-window-new = Una nuova finestra
settings-compose-window-help = Se il browser blocca una nuova finestra, l'editor si apre in una scheda dell'area di lavoro.
settings-compose-read-receipt = Richiedi sempre la conferma di lettura
settings-compose-delivery-receipt = Richiedi sempre la conferma di consegna
settings-compose-reply-copy = Salva una copia delle risposte nella cartella originale
settings-compose-receipts-help = Le conferme sono richieste. Il destinatario o il suo server di posta può rifiutarle o ignorarle.
settings-compose-forward = Formato predefinito di inoltro
settings-compose-forward-inline = Testo del messaggio nel corpo
settings-compose-forward-attachment = Messaggio originale come allegato .eml
settings-compose-saving = Salvataggio della preferenza di scrittura…
settings-compose-saved = Preferenza di scrittura salvata
settings-ident-organization = Organizzazione
settings-ident-auto-bcc = Ccn automatica
settings-ident-auto-bcc-help = Aggiunge fino a 20 indirizzi in Ccn quando si invia da questa identità. Separare gli indirizzi con virgole.
settings-ident-pgp-key = Chiave OpenPGP
settings-ident-pgp-none = Usa la chiave predefinita
settings-ident-pgp-unavailable = La chiave salvata non è disponibile — scegliere un'altra chiave
settings-ident-pgp-help = Scegliere una delle chiavi di firma OpenPGP esistenti. Il materiale della chiave privata resta nel deposito delle chiavi.
settings-ident-error-optional = Controllare gli indirizzi di Rispondi a e di Ccn automatica, l'organizzazione e la chiave selezionata.
settings-compose-open-tracking = Registra per impostazione predefinita le aperture dei messaggi
settings-compose-open-tracking-help = Aggiunge un'immagine di tracciamento ai messaggi formattati idonei. Le aperture dipendono dal caricamento delle immagini da parte del destinatario e non dimostrano che il messaggio sia stato letto. L'amministratore può disattivare il tracciamento o bloccare questa impostazione predefinita.
settings-compose-open-tracking-inherit = Usa il tracciamento predefinito ereditato
settings-reply-images = Immagini in risposte e inoltri
settings-reply-images-ask = Chiedi ogni volta
settings-reply-images-allow = Includi sempre
settings-reply-images-trusted = Includi per i domini mittente attendibili, altrimenti chiedi
settings-reply-images-deny = Non includere mai
settings-reply-images-help = Scegliere se una risposta o un inoltro porta con sé le immagini del messaggio originale. Le immagini incluse vengono copiate nella bozza come allegati.
settings-reply-image-domains = Domini mittente attendibili per le immagini
settings-reply-image-domains-help = Inserire fino a 200 domini mittente esatti, uno per riga. Usato quando la scelta precedente è domini attendibili.
settings-reply-image-domains-save = Salva i domini attendibili per le immagini
settings-ident-group-basics = Dati di base
settings-ident-group-replies = Gestione delle risposte
settings-ident-group-signature = Firma
settings-ident-group-sending = Opzioni di invio
settings-ident-name-required = Inserire un nome visualizzato.
settings-ident-email-invalid = Inserire un indirizzo email valido, come nome@esempio.org.
settings-ident-email-duplicate = Esiste già un'identità con questo indirizzo.
settings-ident-replyto-invalid = Inserire un indirizzo email valido oppure lasciare vuoto.
settings-ident-replyto-help = Le risposte ai messaggi inviati da questa identità vanno a questo indirizzo invece che all'indirizzo del mittente.
settings-ident-bcc-invalid = Controllare l'elenco Ccn: usare indirizzi validi separati da virgole, al massimo 20.
settings-ident-delete-title = Eliminare l'identità { $email }?
settings-ident-delete-body = Non sarà più possibile inviare da questa identità. Bozze e messaggi inviati non sono interessati.
settings-ident-empty = Non ci sono ancora identità. Aggiungerne una per scegliere nome, indirizzo e firma durante la scrittura.
settings-sig-delete-title = Eliminare la firma { $name }?
settings-sig-delete-body = Le identità che la usano tornano alla firma dell'account. I messaggi già inviati non sono interessati.
settings-sig-deleted = Firma eliminata.
settings-reply-from = Rispondi da
settings-reply-from-match = L'indirizzo a cui è stato inviato il messaggio
settings-reply-from-default = Sempre l'identità predefinita
settings-reply-from-ask = Chiedi quando corrispondono più identità
settings-reply-from-keep = Mantieni l'identità già selezionata
settings-reply-from-help = Quando si risponde o si inoltra, scegliere quale identità usare. Se nessuna identità corrisponde all'indirizzo a cui è stato inviato il messaggio, viene usata l'identità predefinita. Gli indirizzi con segno più come nome+tag@esempio.org corrispondono a nome@esempio.org.
settings-ident-default-badge = Predefinita
settings-ident-make-default = Imposta come predefinita
settings-ident-default-set = { $email } è ora l'identità predefinita.
settings-ident-create-variant = Crea variante

# -- Machine translation (2026-11): added to match en ---------------------
settings-ident-variant-count = { $count } { $count ->
        [one] variante
       *[other] varianti
}
settings-ident-inherits = Usa { $base } per: { $fields }
settings-ident-field-replyTo = rispondi a
settings-ident-field-autoBcc = Ccn automatica
settings-ident-field-organization = organizzazione
settings-ident-field-pgpKeyId = chiave OpenPGP
settings-ident-field-signatureName = firma
settings-ident-delete-default-body = Questa è l'identità predefinita. L'identità rimanente più vecchia diventa quella predefinita. Non sarà più possibile inviare da questa identità. Bozze e messaggi inviati non sono interessati.
settings-ident-variants-choice-title = Eliminare { $email }?
settings-ident-variants-choice-body = Questa identità ha { $count } { $count ->
        [one] variante
       *[other] varianti
}. Eliminarle anche, oppure conservarle come identità separate. Una variante conservata copia le impostazioni ereditate da questa identità.
settings-ident-delete-with-variants = Elimina l'identità e le varianti
settings-ident-delete-keep-variants = Elimina l'identità, conserva le varianti
settings-ident-new-variant = Nuova variante
settings-ident-edit-variant-title = Modifica variante
settings-ident-variant-intro = Una variante di { $base }. Lasciare vuoto un campo per usare il valore di quell'identità.
settings-ident-label-label = Etichetta della variante
settings-ident-label-required = Inserire un'etichetta per questa variante, ad esempio Newsletter.
settings-ident-inherit-hint = Vuoto: usa { $value } dell'identità di base.
settings-ident-inherit-none = nulla
settings-check-policy-allow = Invia senza chiedere
settings-check-policy-warn = Chiedi ogni volta
settings-check-policy-warn-remember = Chiedi, e consenti di non chiedere più
settings-check-policy-deny = Blocca l'invio
settings-check-invalid = Indirizzi dei destinatari non validi
settings-check-attachment = Allegato citato ma mancante
settings-check-subject = Oggetto vuoto
settings-check-help = Cosa fare quando il controllo trova un problema. Chiedi offre sempre Invia comunque; solo Blocca l'invio ferma un messaggio.
settings-external-warn-remember = Chiedi, e consenti di non chiedere più
settings-external-deny = Blocca l'invio
settings-auto-chip = Completa un destinatario quando si digita uno spazio, una virgola o un punto e virgola
settings-auto-chip-help = Un indirizzo valido diventa un destinatario completo e il separatore viene aggiunto automaticamente.
settings-recipient-separator = Separatore dei destinatari
settings-recipient-separator-comma = Virgola
settings-recipient-separator-semicolon = Punto e virgola
settings-header-collapse = Intestazione del messaggio (A, Cc, Oggetto)
settings-header-collapse-expanded = Mostra sempre
settings-header-collapse-collapsed = Comprimi in una riga di riepilogo
settings-header-collapse-auto = Comprimi mentre si scrive il corpo
settings-header-collapse-help = Compressa, l'intestazione mostra una riga come A: Ana +2 e si apre al clic oppure quando un campo richiede attenzione.
settings-retention-title = Conservazione
settings-folder-icon-camera = Fotocamera
settings-folder-icon-music = Musica
settings-folder-icon-film = Film
settings-folder-icon-lightbulb = Idea
settings-folder-icon-rocket = Razzo
settings-folder-icon-trophy = Trofeo
settings-folder-icon-leaf = Foglia
settings-folder-icon-globe = Globo
settings-folder-icon-cloud = Nuvola
settings-folder-icon-bookmark = Segnalibro
settings-folder-icon-wallet = Portafoglio
settings-folder-icon-medical = Medico
settings-folder-icon-bell = Campana
settings-folder-icon-key = Chiave
settings-folder-icon-receipt = Ricevuta
settings-folder-icon-cart = Carrello
settings-folder-icon-package = Pacco
settings-folder-icon-wrench = Strumenti
settings-folder-icon-chart = Grafico
settings-folder-icon-team = Team
settings-folder-icon-store = Negozio
settings-folder-icon-office = Ufficio

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-icon-laptop = Laptop
settings-folder-icon-crown = Corona
settings-folder-icon-family = Famiglia
settings-folder-icon-group-general = Generale
settings-folder-icon-group-mail = Posta
settings-folder-icon-group-work = Lavoro
settings-folder-icon-group-finance = Finanza e acquisti
settings-folder-icon-group-travel = Viaggi e casa
settings-folder-icon-group-personal = Personale e svago
settings-folder-icon-group-tech = Tecnologia e strumenti
settings-folder-icon-search-label = Cerca icone
settings-folder-icon-no-match = Nessuna icona corrisponde

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Autenticazione (DMARC)

settings-badge-auth-help = Il risultato DMARC; apre tutti i dettagli di sicurezza.

settings-badge-density = Dimensioni del distintivo

settings-badge-density-comfortable = Confortevole

settings-badge-density-compact = Compatto

settings-badge-density-help = Altezza e spaziatura compatta o comoda per ogni badge.

settings-badge-folder = Cartella

settings-badge-folder-help = Dove è archiviato il messaggio. Solo quando è notevole lo mostra per Junk and Trash.

settings-badge-labels = Etichette per badge

settings-badge-labels-help = Icona con testo o solo icona (il testo completo rimane nella descrizione comando e per le utilità per la lettura dello schermo).

settings-badge-labels-icon = Solo icona

settings-badge-labels-text = Icona e testo

settings-badge-list = Lista di posta

settings-badge-list-help = La mailing list da cui proviene il messaggio; apre i dettagli e le azioni dell'abbonamento. Solo quando notevole lo mostra quando la posta continua ad arrivare dopo aver annullato l'iscrizione.

settings-badge-mode-always = Sempre

settings-badge-mode-never = Mai

settings-badge-mode-notable = Solo quando notevole

settings-badge-sanitized = Sanificato HTML

settings-badge-sanitized-help = Che cosa ha fatto il disinfettante HTML al messaggio. Solo quando notevole lo mostra quando qualcosa è stato bloccato.

settings-badge-spam = Punteggio spam

settings-badge-spam-help = Il punteggio che il server ricevente ha scritto nelle intestazioni dei messaggi.

settings-folders-hide-empty = Nascondi le cartelle vuote nella barra laterale

settings-folders-hide-empty-help = Le cartelle riappaiono quando contengono messaggi. La posta in arrivo, le cartelle di ricerca, le cartelle con conteggi sconosciuti e le cartelle principali delle cartelle visibili rimangono visualizzate. Sostituisci le singole cartelle in Proprietà, Visualizzazione; ogni cartella rimane in questo elenco.

settings-folders-list-card = Elenco e ordine delle cartelle

settings-header-auto-collapse = Piega l'intestazione dopo aver scritto (secondi)

settings-header-auto-collapse-help = Quando il corpo è stato concentrato per questo numero di secondi e l'intestazione ha un destinatario e un oggetto, si ripiega sulla riga di riepilogo. 0 lo disattiva.

settings-message-badges = Badge dei messaggi

settings-message-badges-help = Scegli quali badge visualizzare nell'intestazione di un messaggio aperto. I segni di priorità sono impostati in Cartelle e barra laterale. Puoi anche fare clic con il pulsante destro del mouse su un badge per nasconderlo.

settings-reply-from-headers = Intestazioni che scelgono l'identità

settings-reply-from-headers-help = Un nome di intestazione per riga, provato in questo ordine: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc. Un'intestazione tralasciata non viene letta.

settings-reply-from-headers-invalid = Utilizza solo questi nomi, uno per riga: Consegnato-A, X-Originale-A, Busta-A, X-Inoltrato-A, Risentito-A, A, Cc.

settings-reply-from-headers-save = Salva l'ordine delle intestazioni

settings-sessions-linked-created = Collegamento creato

settings-sessions-linked-details = Dettagli del collegamento

settings-sessions-linked-kind = Tipo di collegamento

settings-sessions-linked-kind-linked = Casella di posta collegata

settings-sessions-linked-last-used = Ultimo usato

settings-sessions-linked-line = Accesso alla casella di posta collegata · da { $origin }

settings-sessions-linked-line-hidden = Accesso alla casella di posta collegata

settings-sessions-linked-origin = Viene da

settings-sessions-linked-removed = Sconosciuto (account rimosso)

settings-sessions-linked-revoke = Revoca l'accesso al collegamento

settings-sessions-linked-revoke-label = Revoca l'accesso al collegamento per { $mailbox }

settings-sessions-linked-target = Cassetta postale aperta

settings-sessions-origin-full = Indirizzo completo

settings-sessions-origin-help = Controlla il modo in cui questa tabella denomina l'account da cui proviene una sessione di cassetta postale collegata. Solo tu lo vedi, nelle tue sessioni.

settings-sessions-origin-hidden = Nascosto

settings-sessions-origin-masked-local-part = Indirizzo con la prima parte mascherata

settings-sessions-origin-setting = Mostra da dove proviene l'accesso alla casella di posta collegata

settings-shortcuts-search = Cerca scorciatoie
