# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Dashboard
admin-shell-group-overview = Panoramica
admin-shell-group-people = Persone
admin-shell-group-security = Sicurezza
admin-shell-group-configuration = Configurazione
admin-shell-group-operations = Operazioni
admin-shell-menu = Menu
admin-shell-close-menu = Chiudi menu
admin-shell-search = Cerca
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Accesso effettuato come { $name }
admin-shell-language-label = Lingua
admin-shell-breadcrumb = Percorso di navigazione
admin-shell-unknown-section = Questa sezione non esiste oppure il proprio ruolo non la include.
admin-shell-back-to-dashboard = Torna alla dashboard
admin-shell-cap-missing = Il proprio ruolo non include { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Caricamento…
admin-shell-retry = Riprova
admin-shell-cancel = Annulla
admin-shell-save = Salva
admin-shell-not-available = Non disponibile

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Lasciare questa pagina?
admin-shell-leave-body = Le modifiche in questa pagina non sono state salvate. Uscendo andranno perse.
admin-shell-leave-confirm = Esci e scarta le modifiche

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Digitare { $name } per confermare

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Cerca nella console di amministrazione
admin-shell-palette-label = Sezione, utente, dominio o impostazione
admin-shell-palette-results = Risultati
admin-shell-palette-none = Nessun risultato.
admin-shell-palette-searching = Ricerca di utenti, domini e impostazioni…
admin-shell-palette-sections-only = Questo server non può ancora cercare utenti, domini e impostazioni, quindi sono elencate solo le sezioni.
admin-shell-palette-error = La ricerca di utenti, domini e impostazioni non è riuscita; sono elencate solo le sezioni.
admin-shell-palette-kind-section = Sezione
admin-shell-palette-kind-user = Utente
admin-shell-palette-kind-domain = Dominio
admin-shell-palette-kind-setting = Impostazione

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filtri
admin-shell-table-actions = Azioni
admin-shell-table-error = Impossibile caricare l'elenco.
admin-shell-table-paging = Pagine di { $table }
admin-shell-table-prev = Pagina precedente
admin-shell-table-next = Pagina successiva

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Ultimo { $latest }, massimo { $max }
admin-shell-chart-col-time = Ora
admin-shell-chart-col-item = Voce
admin-shell-chart-col-value = Valore

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Impostato da
admin-shell-pinned-sr = Questo valore proviene dall'ambiente del server e non può essere modificato qui.
admin-shell-lock-text = Blocca per gli utenti
admin-shell-lock-setting = { $setting }
admin-shell-restore = Ripristina il valore precedente
admin-shell-restore-note = Modificato da { $actor } il { $time }. Valore precedente: { $value }
admin-shell-restore-done = Il valore precedente è stato ripristinato.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Passaggi
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Passaggio { $n } di { $total }: { $title }
admin-shell-wizard-review = Riepilogo
admin-shell-wizard-back = Indietro
admin-shell-wizard-next = Avanti
admin-shell-wizard-saved = Salvato.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Impossibile raggiungere il server. Controllare la connessione e riprovare.
admin-shell-err-session = La sessione di amministrazione è terminata. Accedere di nuovo.
admin-shell-err-forbidden = Il proprio ruolo non consente questa azione. Rivolgersi a un amministratore con il ruolo adeguato.
admin-shell-err-out-of-scope = Questo oggetto appartiene a un dominio non coperto dal proprio ruolo.
admin-shell-err-not-found = Non esiste più. Ricaricare l'elenco.
admin-shell-err-conflict = Qualcuno lo ha modificato nel frattempo. Ricaricare e riprovare.
admin-shell-err-validation = Il server ha rifiutato un valore. Controllare i campi e riprovare.
admin-shell-err-rate-limited = Troppe richieste. Attendere un minuto e riprovare.
admin-shell-err-server = Il server ha risposto con un errore ({ $status }). Riprovare; se si ripete, controllare il log del server.
admin-shell-err-upstream-dns = Impossibile risolvere il nome host. Controllare l'ortografia dell'host.
admin-shell-err-upstream-connect = Nessuna risposta su quell'host e porta. Controllare la porta ed eventuali firewall intermedi.
admin-shell-err-upstream-tls = Il server ha risposto ma l'handshake TLS non è riuscito. Controllare la porta e l'impostazione di sicurezza della connessione.
admin-shell-err-upstream-auth = Il server ha rifiutato le credenziali. Controllare nome utente e password.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Troppi tentativi di accesso. Attendere qualche minuto e riprovare.
admin-shell-totp-form = Accesso amministratore, secondo passaggio
admin-shell-totp-note = Inserire il codice a 6 cifre dell'app di autenticazione configurata per { $name }.
admin-shell-totp-code = Codice di autenticazione
admin-shell-totp-verify = Verifica
admin-shell-totp-start-over = Ricomincia
admin-shell-totp-invalid = Il codice non è valido. Inserire il codice attuale dell'app di autenticazione.
admin-shell-totp-expired = L'accesso ha richiesto troppo tempo. Ricominciare e inserire la password.
