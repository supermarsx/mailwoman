# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-email = Indirizzo email
auth-password = Password
auth-sign-in = Accedi
auth-signing-in = Accesso in corso…
auth-invalid-credentials = Credenziali non valide
auth-refused-note = Se l'indirizzo email e la password sono corretti, l'account potrebbe essere stato disattivato da un amministratore. Un account disattivato non può accedere finché un amministratore non lo riattiva.
auth-unreachable = Impossibile raggiungere il server
auth-address-not-configured = Questo indirizzo non è configurato su questo server. Contattare l'amministratore.

# -- Single sign-on (t9) -----------------------------------------------------
auth-sso-heading = Single sign-on
auth-sso-divider = oppure continua con
auth-sso-button = Accedi con { $name }
auth-sso-error = Il single sign-on non è stato completato. Riprovare o accedere con la password.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Autorizza applicazione
auth-consent-title = Autorizza accesso
auth-consent-loading = Caricamento della richiesta…
auth-consent-intro = vuole accedere al suo account.
auth-consent-approved = Client approvato dall'amministratore
auth-consent-unapproved = Client non riconosciuto — non approvato dall'amministratore
auth-consent-requesting = Richiede
auth-consent-redirects-to = Reindirizza a
auth-consent-for-resource = Per la risorsa
auth-consent-deny = Nega
auth-consent-allow = Consenti
auth-consent-error = impossibile registrare la decisione

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Troppi tentativi di accesso. Attendi prima di riprovare.
auth-retry-countdown = Riprova tra { $seconds } s
auth-upstream-protocol = Accesso interrotto: il server di posta ha inviato una risposta che questo server non è riuscito a leggere. Riprova più tardi o contatta l'amministratore.
auth-account-disabled = Questo account è stato disattivato da un amministratore. Non potrà accedere finché un amministratore non lo riattiva.
auth-signin-not-allowed = Questo indirizzo non può accedere a questo server. Contatta l'amministratore.
auth-signin-not-provisioned = Non esiste ancora un account per questo indirizzo e i nuovi account non vengono creati automaticamente. Chiedi all'amministratore di aggiungerti.
auth-reference = Riferimento: { $id }
auth-domain-hint = Per { $domain } puoi accedere con il solo nome utente.
auth-passkey-sign-in = Accedi con una passkey
auth-passkey-failed = Non è stato possibile accedere con la passkey. Riprova o usa la password.
auth-remembered = Accessi precedenti
auth-remembered-help = Indirizzi usati con successo su questo dispositivo. Selezionandone uno si compila solo l'indirizzo.
auth-forget-address = Dimentica { $address } su questo dispositivo
auth-login-timeout = Accesso scaduto. Non è stato fatto alcun nuovo tentativo automatico. Controlla la connessione e riprova quando vuoi.
auth-verification-title = Verifica dell'accesso
auth-verification-loading = Caricamento della verifica dell'accesso…
auth-verification-error = Non è stato possibile completare la verifica. Ripeti la verifica prima di accedere.
auth-verification-required = Completa una nuova verifica dell'accesso prima di riprovare.
auth-identity-title = Condividi la tua identità
auth-identity-sign-in = Accedi di nuovo per continuare questa richiesta di identità.
auth-identity-sign-out = Esci da Mailwoman
auth-identity-expired = Questa richiesta di identità è scaduta o è cambiata. Torna all'applicazione che l'ha richiesta e ricomincia.
auth-identity-retry = Riprova
auth-identity-disclosure = Questa applicazione riceve le informazioni di identità riportate di seguito. Questa autorizzazione non concede l'accesso alla tua posta.
auth-identity-scope-openid = Un identificatore di identità stabile
auth-identity-scope-profile = Le informazioni di profilo disponibili
auth-identity-scope-email = L'indirizzo email con cui hai effettuato l'accesso, senza attestare che sia stato verificato in modo indipendente
auth-identity-scope-offline_access = Rinnovare l'accesso all'identità finché questa sessione di Mailwoman resta valida, fino a 15 giorni dall'accesso
auth-identity-deny = Annulla
auth-identity-approve = Consenti accesso all'identità
auth-identity-grants = Applicazioni che usano la tua identità
auth-identity-grants-help = Revoca l'accesso all'identità per un'applicazione. Le autorizzazioni API di posta separate non vengono rimosse.
auth-identity-no-grants = Nessuna applicazione ha attualmente il consenso all'identità.
auth-identity-revoke = Revoca accesso all'identità
auth-maintenance = La webmail è temporaneamente in pausa per manutenzione. Riprova più tardi.
