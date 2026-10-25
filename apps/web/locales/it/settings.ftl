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
