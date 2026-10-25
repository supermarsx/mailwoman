# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Cambia password
passwd-heading = Cambia password

# -- Forced change -----------------------------------------------------------
passwd-force-change = L'amministratore richiede di cambiare la password prima di continuare.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Password attuale
passwd-new-label = Nuova password
passwd-confirm-label = Conferma nuova password

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Le nuove password corrispondono.
passwd-match-no = Le nuove password non corrispondono ancora.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = almeno { $count } caratteri
passwd-rule-uppercase = una lettera maiuscola
passwd-rule-lowercase = una lettera minuscola
passwd-rule-digit = una cifra
passwd-rule-symbol = un simbolo

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Questo account è zero-access. Prima che la modifica venga applicata verrà mostrata una frase di recupero — salvarla per poter accedere ai dati anche se qualcosa va storto.

# -- Actions -----------------------------------------------------------------
passwd-continue = Continua
passwd-submit = Cambia password

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Salvare la frase di recupero
passwd-recovery-prose = Scrivere questa frase e conservarla in un luogo sicuro. Viene mostrata prima del cambio di password per poter recuperare i dati anche se la nuova password va persa. Non è archiviata sul server.
passwd-recovery-ack-label = Ho salvato la frase di recupero
passwd-recovery-ack-text = Ho salvato la frase di recupero in un luogo sicuro.

# -- Done phase --------------------------------------------------------------
passwd-done = La password è stata cambiata.
passwd-done-reencrypt = Le credenziali del server archiviate sono state cifrate di nuovo con la nuova password.
passwd-done-rewrap = Le chiavi zero-access sono state protette di nuovo con la nuova password.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = inserire la password attuale
passwd-error-mismatch = la nuova password e la sua conferma non corrispondono
passwd-error-policy = la nuova password richiede { $rules }
passwd-error-ack-first = confermare prima di aver salvato la frase di recupero
passwd-error-prepare-recovery = impossibile preparare la frase di recupero
passwd-error-change = impossibile cambiare la password
passwd-error-session-ended = La sessione è terminata. Accedere di nuovo.
passwd-error-not-configured = Questo server non è configurato per cambiare le password. Contattare l'amministratore.
passwd-error-contact-admin = Il server non ha potuto completare la modifica. Contattare l'amministratore.

# -- Forced change screen ----------------------------------------------------
passwd-forced-title = Cambiare la password per continuare
passwd-forced-signed-in-as = Accesso effettuato come { $username }.
passwd-forced-explain = Un amministratore ha richiesto il cambio della password per questo account. Posta, calendario e contatti non sono disponibili finché la modifica non va a buon fine.
passwd-forced-backend-note = Se la modifica viene rifiutata anche se la password attuale è corretta, questo server potrebbe non essere in grado di cambiare la password. Contattare l'amministratore: può configurare il cambio delle password o rimuovere il requisito.
passwd-forced-sign-out = Esci
passwd-forced-still-required = Il server ha accettato la modifica ma richiede ancora un cambio di password per questo account. Contattare l'amministratore.
passwd-forced-reload-failed = La password è stata cambiata, ma non è stato possibile caricare l'account. Controllare la connessione e riprovare.
passwd-forced-retry = Riprova
