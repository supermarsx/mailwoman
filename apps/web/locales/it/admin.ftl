# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Amministrazione Mailwoman
admin-nav = Sezioni di amministrazione
admin-sign-out = Esci
admin-nav-domains = Domini
admin-nav-users = Utenti
admin-nav-integrations = Integrazioni
admin-nav-observability = Osservabilità
admin-nav-plugins = Plugin
admin-nav-assist = Assistente IA
admin-nav-sso = Single sign-on
admin-nav-servermeta = Metadati del server
admin-nav-rethread = Manutenzione
admin-nav-2fa = Richiedi due fattori

admin-delete = Elimina
admin-revoke = Revoca
admin-remove = Rimuovi
admin-saved = Salvato.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Accesso amministratore
admin-login-note = Questo pannello usa una sessione di amministrazione separata. La sessione termina dopo 30 minuti di inattività e al più tardi 12 ore dopo l'accesso.
admin-login-session-ended = La sessione di amministrazione è terminata. Accedere di nuovo.
admin-login-username = Nome utente amministratore
admin-login-password = Password
admin-login-sign-in = Accedi
admin-login-signing-in = Accesso in corso…
admin-login-invalid = Credenziali di amministratore non valide
admin-login-unreachable = Impossibile raggiungere il server

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domini
admin-domains-load-error = Impossibile caricare i domini
admin-domains-save-error = Impossibile salvare il dominio
admin-domains-delete-error = Impossibile eliminare il dominio
admin-domains-add = Aggiungi dominio
admin-domains-name = Nome del dominio
admin-domains-name-placeholder = example.com
admin-domains-note = Un dominio viene registrato per nome. I nomi registrati sono proposti nella schermata Richiedi due fattori quando una regola si applica a un solo dominio. La registrazione di un dominio non instrada né filtra la posta.
admin-domains-save = Salva dominio
admin-domains-empty = Nessun dominio.
admin-domains-delete-for = Elimina { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Utenti
admin-users-load-error = Impossibile caricare gli utenti
admin-users-provision-error = Impossibile creare l'utente
admin-users-flag-error = Impossibile aggiornare l'opzione
admin-users-revoke-error = Impossibile revocare le sessioni
admin-users-provision = Crea utente
admin-users-username = Nome utente
admin-users-domain = Dominio
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Quota in byte (0 = illimitata)
admin-users-quota-msgs = Quota in messaggi (0 = illimitata)
admin-users-empty = Nessun utente.
admin-users-col-account = Account
admin-users-col-quota = Quota (byte/msg)
admin-users-col-zeroaccess = Accesso zero
admin-users-col-flags = Opzioni
admin-users-col-sessions = Sessioni
admin-users-zeroaccess-for = Accesso zero per { $account }
admin-users-zeroaccess-help = Solo un'annotazione. L'archiviazione ad accesso zero viene attivata e disattivata dall'utente, nelle proprie impostazioni, con la propria chiave. Selezionare questa casella non cifra la posta archiviata dell'account e deselezionarla non la decifra.
admin-users-disable-for = Disattiva { $account }
admin-users-disabled = disattivato
admin-users-disabled-help = Blocca l'accesso e chiude tutte le sessioni aperte di questo account. Le sue chiavi API e i suoi token vengono rifiutati. Non disattiva la casella sul server di posta: altri client di posta possono ancora raggiungerla finché non viene disattivata anche lì.
admin-users-force-change-for = Forza il cambio password per { $account }
admin-users-force-change = forza cambio
admin-users-force-change-help = L'utente viene trattenuto su una schermata di cambio password, al prossimo accesso e nelle sessioni già aperte, e non può accedere alla posta finché il cambio non riesce. Serve un backend di cambio password funzionante (MW_PASSWD_BACKEND). Senza di esso il cambio non può riuscire e l'utente resta su quella schermata finché non si deseleziona questa casella.
admin-users-revoke-for = Revoca le sessioni di { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integrazioni
admin-integrations-load-error = Impossibile caricare le integrazioni
admin-integrations-revoke-error = Impossibile revocare la chiave
admin-integrations-ldap = Directory LDAP / GAL
admin-integrations-nextcloud = Bridge Nextcloud
admin-integrations-active = Attivo
admin-integrations-configured = Configurato
admin-integrations-not-configured = Non configurato
admin-integrations-unknown = Stato sconosciuto
admin-integrations-config-note = "Configurato" significa che questa installazione ha impostazioni per l'integrazione: una voce di directory LDAP attiva nel database oppure le tre variabili d'ambiente MW_NEXTCLOUD_*. Entrambe vengono lette all'avvio del server. Non significa che il servizio remoto sia stato contattato o sia raggiungibile.
admin-integrations-webhooks = Webhook
admin-integrations-webhooks-empty = Nessun webhook registrato.
admin-integrations-keys = Chiavi API e MCP
admin-integrations-keys-empty = Nessuna chiave emessa.
admin-integrations-col-prefix = Prefisso
admin-integrations-col-account = Account
admin-integrations-col-scopes = Ambiti
admin-integrations-col-status = Stato
admin-integrations-status-revoked = revocata
admin-integrations-status-active = attiva
admin-integrations-revoke-key = Revoca la chiave { $prefix }
admin-integrations-col-unattended = Invio non presidiato
admin-integrations-unattended-not-requested = Non richiesto
admin-integrations-unattended-requested = Richiesto, non approvato
admin-integrations-unattended-approved = Approvato
admin-integrations-unattended-approve = Approva
admin-integrations-unattended-withdraw = Ritira
admin-integrations-unattended-approve-key = Approva l'invio non presidiato per la chiave { $prefix }
admin-integrations-unattended-withdraw-key = Ritira l'approvazione dell'invio non presidiato per la chiave { $prefix }
admin-integrations-unattended-approve-title = Approvare l'invio non presidiato per questa chiave?
admin-integrations-unattended-withdraw-title = Ritirare l'approvazione per questa chiave?
admin-integrations-unattended-owner = Proprietario: { $account }
admin-integrations-unattended-key = Chiave: { $prefix }
admin-integrations-unattended-scope = Permessi: { $permissions }. Strumenti MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = Impossibile leggere l'ambito di questa chiave.
admin-integrations-scope-read = lettura
admin-integrations-scope-send = invio
admin-integrations-scope-delete = eliminazione
admin-integrations-scope-mail = posta
admin-integrations-scope-pim = calendario, attività, note e contatti
admin-integrations-scope-none = nessuno
admin-integrations-unattended-approve-meaning = I messaggi inviati con questa chiave tramite MCP vengono trasmessi senza che una persona li rilasci.
admin-integrations-unattended-withdraw-meaning = Un invio effettuato con questa chiave tramite MCP viene rifiutato finché la chiave non viene approvata di nuovo. Il messaggio non viene trasmesso e non viene messo nella Posta in uscita del proprietario.
admin-integrations-unattended-approve-effect = L'approvazione ha effetto immediato, dal prossimo invio della chiave. Non serve un riavvio.
admin-integrations-unattended-withdraw-effect = Il ritiro ha effetto immediato: la chiave smette di inviare senza presidio dal prossimo invio. Non serve un riavvio.
admin-integrations-unattended-cancel = Annulla
admin-integrations-unattended-saving = Salvataggio…
admin-integrations-unattended-approved-done = Invio non presidiato approvato per la chiave { $prefix }. È già in vigore.
admin-integrations-unattended-withdrawn-done = Approvazione ritirata per la chiave { $prefix }. Questa chiave non invia più senza presidio.
admin-integrations-unattended-error-gone = La chiave { $prefix } non esiste più o è stata revocata. Non è stato modificato nulla.
admin-integrations-unattended-error-conflict = La chiave { $prefix } è revocata oppure il suo proprietario non ha richiesto l'invio non presidiato. Non è stato approvato nulla.
admin-integrations-unattended-error = Impossibile modificare l'approvazione per la chiave { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Osservabilità
admin-obs-load-error = Impossibile caricare i dati di osservabilità
admin-obs-export-error = Impossibile esportare il registro di audit
admin-obs-ban-add-error = Impossibile aggiungere il blocco
admin-obs-unban-error = Impossibile rimuovere il blocco
admin-obs-telemetry-note = Log e telemetria non si impostano qui. Il filtro dei log (MW_LOG), il collector OTLP (MW_OTLP_ENDPOINT) e l'endpoint delle metriche (MW_METRICS_TOKEN) vengono letti dall'ambiente del server al suo avvio.
admin-obs-audit = Registro di audit
admin-obs-export = Esporta JSONL
admin-obs-audit-empty = Nessuna voce di audit.
admin-obs-col-time = Ora
admin-obs-col-actor = Autore
admin-obs-col-action = Azione
admin-obs-col-target = Oggetto
admin-obs-bans = Monitor accessi / elenco blocchi
admin-obs-bans-note = Questo elenco è un'annotazione. Mailwoman non rifiuta le connessioni dagli indirizzi elencati e una voce non ha effetto sull'accesso. Ogni accesso amministratore fallito scrive una riga nel log del server su cui una jail di fail2ban può agire, e cinque errori da uno stesso indirizzo lo aggiungono qui. Il blocco è effettuato da quella jail o dal firewall.
admin-obs-ban-add = Aggiungi blocco
admin-obs-ban-ip = Indirizzo IP
admin-obs-ban-reason = Motivo
admin-obs-ban-btn = Blocca IP
admin-obs-bans-empty = Nessun blocco attivo.
admin-obs-unban-for = Sblocca { $ip }
admin-obs-unban-btn = Sblocca

# -- Plugins (§22) -----------------------------------------------------------
admin-plugins-title = Plugin
admin-plugins-intro = I plugin del motore vengono eseguiti in una sandbox WebAssembly con capacità controllate. Approvare un plugin prima di poterlo attivare e concedere solo le capacità di cui ha bisogno.
admin-plugins-empty = Nessun plugin registrato.
admin-plugins-signed = Firmato
admin-plugins-unsigned = Non firmato
admin-plugins-approved = Approvato
admin-plugins-enabled = Attivo
admin-plugins-approve = Approva
admin-plugins-enable = Attiva
admin-plugins-disable = Disattiva
admin-plugins-net = rete: { $hosts }
admin-plugins-limits = limiti: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = limiti: { $memory } MiB · { $deadline } ms · { $fuel } fuel
admin-plugins-allow-unsigned-for = Consenti il plugin non firmato { $name }
admin-plugins-allow-unsigned = Consenti l'esecuzione di questo plugin non firmato
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
admin-allowlist-title = Elenco consentiti dei plugin di terze parti
admin-allowlist-intro = Un componente di terze parti viene caricato solo dopo che un amministratore ne ha approvato l'esatto digest SHA-256. Il digest qui sotto è calcolato dal server sui byte del componente su disco; approvarlo vincola esattamente quei byte. I componenti propri sono vincolati nella build e non si gestiscono qui.
admin-allowlist-load-error = Impossibile caricare l'elenco consentiti dei plugin.
admin-allowlist-present-heading = Componenti su disco
admin-allowlist-present-empty = Non è presente alcun componente di terze parti. Inserire un componente nella directory dei plugin di terze parti perché compaia qui.
admin-allowlist-digest-label = SHA-256 calcolato
admin-allowlist-status-approved = Approvato
admin-allowlist-status-pending = Non approvato
admin-allowlist-status-firstparty = Proprio
admin-allowlist-unsigned-note = Ammesso tramite vincolo del digest. Questo componente non ha firma; l'approvazione considera attendibili esattamente i byte il cui digest è mostrato, come previsto per un componente approvato in questo modo.
admin-allowlist-highpower-note = Le capacità ad alto privilegio ({ $caps }) non possono essere concesse a un plugin di terze parti. Il server le rifiuta indipendentemente dall'approvazione; sono disponibili solo per i componenti propri.
admin-allowlist-firstparty-note = Questo id corrisponde a un componente proprio. Il vincolo del componente proprio ha sempre la precedenza, quindi questo id non può essere approvato come terza parte.
admin-allowlist-approve = Approva digest
admin-allowlist-approve-for = Approva il digest per { $id }
admin-allowlist-revoke = Revoca
admin-allowlist-revoke-for = Revoca il vincolo per { $id }
admin-allowlist-uninstall = Disinstalla
admin-allowlist-uninstall-for = Disinstalla { $id }
admin-allowlist-pins-heading = Vincoli approvati e revocati
admin-allowlist-pins-empty = Nessun vincolo registrato.
admin-allowlist-pin-approved-by = Approvato da { $by } il { $at }
admin-allowlist-pin-revoked = Revocato
admin-allowlist-cancel = Annulla

admin-allowlist-approve-title = Approvare il caricamento di questo componente?
admin-allowlist-approve-detail = L'approvazione vincola esattamente i byte il cui SHA-256 è mostrato qui sotto. Dopo l'approvazione, per questo id verranno caricati solo byte corrispondenti a questo digest; qualsiasi modifica al componente produce un digest diverso e non verrà caricata finché non viene approvata di nuovo. L'approvazione da sola non concede alcuna capacità.
admin-allowlist-approve-confirm = Approva digest

admin-allowlist-revoke-title = Revocare questo vincolo?
admin-allowlist-revoke-detail = La revoca rimuove l'approvazione di questo digest e disattiva il plugin. Ha effetto al caricamento successivo; un'istanza già in esecuzione non viene fermata fino ad allora.
admin-allowlist-revoke-confirm = Revoca vincolo

admin-allowlist-uninstall-title = Disinstallare questo plugin?
admin-allowlist-uninstall-detail = La disinstallazione elimina i dati chiave/valore memorizzati dal plugin per tutti gli account, rimuove i suoi vincoli dall'elenco consentiti e lo disattiva. Il file del componente su disco non viene eliminato; può essere approvato di nuovo in seguito.
admin-allowlist-uninstall-confirm = Disinstalla plugin

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Single sign-on
admin-sso-intro = Configurare i backend di accesso OIDC e SAML 2.0. I backend attivi compaiono come pulsanti "Accedi con…" nella pagina di accesso alla posta, per l'intera installazione o per un solo dominio.
admin-sso-add = Aggiungi un backend di accesso
admin-sso-edit = Modifica
admin-sso-create = Aggiungi backend
admin-sso-update = Salva modifiche
admin-sso-cancel = Annulla
admin-sso-empty = Nessun backend SSO configurato.
admin-sso-load-error = Impossibile caricare i backend SSO.
admin-sso-save-error = Impossibile salvare il backend.
admin-sso-delete-error = Impossibile eliminare il backend.

admin-sso-id = ID del backend
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Nome visualizzato
admin-sso-display-name-placeholder = Accedi con Acme SSO
admin-sso-kind = Protocollo
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Ambito
admin-sso-scope-deployment = Intera installazione
admin-sso-scope-domain = Un dominio
admin-sso-domain = Dominio
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Attivo
admin-sso-first-login = Criterio al primo accesso
admin-sso-policy-allowlist = Elenco consentiti (rifiuta utenti sconosciuti)
admin-sso-policy-autocreate = Crea automaticamente gli account al primo accesso

admin-sso-issuer = URL dell'emittente
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = ID client
admin-sso-client-secret = Segreto client
admin-sso-secret-unchanged = Lasciare vuoto per mantenere il segreto memorizzato
admin-sso-redirect = URL di reindirizzamento
admin-sso-scopes = Scope
admin-sso-metadata = Metadati SP

admin-sso-sp-entity-id = Entity ID SP
admin-sso-acs-url = URL ACS
admin-sso-idp-metadata-url = URL dei metadati IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML dei metadati IdP
admin-sso-idp-metadata-xml-placeholder = Incollare l'XML dei metadati IdP oppure usare l'URL qui sopra
admin-sso-idp-sso-url = URL SSO dell'IdP
admin-sso-idp-slo-url = URL di disconnessione (SLO) dell'IdP
admin-sso-idp-certs = Certificati di firma dell'IdP (PEM)
admin-sso-idp-certs-placeholder = Un certificato PEM per blocco, separati da una riga vuota
admin-sso-nameid-format = Formato NameID
admin-sso-want-signed = Richiedi asserzioni firmate
admin-sso-want-encrypted = Richiedi asserzioni cifrate

admin-sso-claims = Mappatura dei claim
admin-sso-claim-email = Claim email
admin-sso-claim-username = Claim nome utente
admin-sso-claim-display = Claim nome visualizzato
admin-sso-claim-groups = Claim gruppi

admin-sso-badge-enabled = Attivo
admin-sso-badge-disabled = Disattivato
admin-sso-enable = Attiva
admin-sso-disable = Disattiva
admin-sso-delete = Elimina
admin-sso-enable-for = Attiva { $name }
admin-sso-disable-for = Disattiva { $name }
admin-sso-delete-for = Elimina { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = Metadati del server
admin-servermeta-intro = Visualizzare e modificare le annotazioni server RFC 5464 di un account creato. Le modifiche vengono scritte direttamente sul server di posta, che decide se l'account può impostarle.
admin-servermeta-account = Account
admin-servermeta-select-option = Selezionare un account…
admin-servermeta-select-prompt = Selezionare un account per visualizzarne e modificarne le annotazioni server.
admin-servermeta-load-error = Impossibile caricare l'elenco degli account.
admin-servermeta-no-accounts = Nessun account creato.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = Ricostruisci le conversazioni
admin-rethread-intro = Riesegue il raggruppamento in conversazioni (JWZ) sulla posta archiviata di un account creato e ricalcola le chiavi delle conversazioni. È un'operazione di manutenzione una tantum, non viene eseguita automaticamente.
admin-rethread-account = Account
admin-rethread-select-option = Selezionare un account…
admin-rethread-no-accounts = Nessun account creato.
admin-rethread-load-error = Impossibile caricare l'elenco degli account.
admin-rethread-run = Ricostruisci le conversazioni
admin-rethread-confirm-title = Ricostruire le conversazioni di questa casella?
admin-rethread-confirm-warning = La ricostruzione ricalcola il raggruppamento in conversazioni di questo account. Le conversazioni esistenti possono unirsi o dividersi e gli utenti possono vederle raggruppate in modo diverso.
admin-rethread-confirm-detail = È un'operazione di manutenzione una tantum. Viene eseguita una volta ora; può essere ripetuta senza rischi e non elimina alcun messaggio.
admin-rethread-confirm = Ricostruisci le conversazioni
admin-rethread-running = Ricostruzione in corso…
admin-rethread-cancel = Annulla
admin-rethread-summary = { $messages } messaggi raggruppati in { $threads } conversazioni su { $accounts } account; { $reassigned } messaggi spostati in un'altra conversazione.
admin-rethread-error = La ricostruzione delle conversazioni non è riuscita. Se il server ha rifiutato la richiesta, il raggruppamento non è stato modificato; controllare i log del server e riprovare.

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = Indice di ricerca
admin-searchindex-intro = La ricerca legge un indice costruito dalla posta archiviata. Il server lo ricostruisce all'avvio quando non corrisponde alla posta archiviata. La posta degli account ad accesso zero non viene indicizzata, quindi la ricerca non trova nulla per quegli account.
admin-searchindex-count = { $documents } di { $messages } messaggi archiviati sono indicizzati.
admin-searchindex-on-disk = L'indice è conservato su disco.
admin-searchindex-in-memory = L'indice è conservato in memoria e viene ricostruito a ogni avvio del server.
admin-searchindex-progress = Ricostruzione in corso: { $done } di { $total } messaggi completati.
admin-searchindex-run = Ricostruisci l'indice di ricerca
admin-searchindex-running = Ricostruzione in corso…
admin-searchindex-summary = Letti { $messages } messaggi archiviati in { $accounts } account: { $indexed } indicizzati, { $removed } rimossi dall'indice, { $failed } non leggibili. { $zeroAccess } account ad accesso zero non indicizzati.
admin-searchindex-busy = È già in corso una ricostruzione. Attendere che finisca.
admin-searchindex-error = La ricostruzione non è riuscita. L'indice mantiene ciò che conteneva; controllare i log del server e riprovare.
admin-searchindex-status-error = Impossibile leggere lo stato dell'indice di ricerca.
admin-searchindex-unavailable = Questo server funziona in modalità proxy e non ha un indice di ricerca.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = Richiedi due fattori
admin-2fa-intro = Richiedere un secondo fattore (passkey o app di autenticazione) per l'accesso. Ogni utente può registrare un fattore autonomamente; richiederlo qui obbliga gli account interessati a registrarlo al prossimo accesso.
admin-2fa-load-error = Impossibile caricare il criterio dei due fattori.
admin-2fa-save-error = Impossibile salvare il criterio dei due fattori.
admin-2fa-global = Richiedi due fattori per l'intera installazione
admin-2fa-global-label = Richiedi due fattori per l'intera installazione
admin-2fa-global-note = Se attivo, ogni account deve avere un secondo fattore. Una regola per dominio qui sotto può anche richiederlo per un solo dominio senza richiederlo ovunque.
admin-2fa-domains-heading = Requisiti per dominio
admin-2fa-domains-empty = Nessun requisito per dominio impostato.
admin-2fa-col-domain = Dominio
admin-2fa-col-require = Richiesto
admin-2fa-require-for = Richiedi due fattori per { $domain }
admin-2fa-add-domain = Dominio
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Richiedi due fattori per questo dominio
admin-2fa-add-require-label = Richiedi due fattori per questo dominio
admin-2fa-add-save = Aggiungi regola di dominio

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Uscita
admin-egress-heading = Instradamenti proxy in uscita
admin-egress-intro = Instradare le richieste in uscita del server attraverso un proxy HTTP CONNECT o SOCKS5. Mailwoman applica comunque il proprio criterio sugli indirizzi: al proxy non viene mai chiesto di risolvere un nome.
admin-egress-empty = Nessun instradamento configurato. Le richieste in uscita sono dirette.
admin-egress-none-active = Nessun instradamento in uso. Le richieste in uscita sono dirette. Un instradamento salvato trasporta traffico solo dopo aver scelto "Usa questo instradamento".
admin-egress-in-use = Le richieste in uscita usano l'instradamento "{ $id }".
admin-egress-deactivate = Smetti di usare un proxy
admin-egress-activate = Usa questo instradamento
admin-egress-activate-for = Usa l'instradamento { $id }
admin-egress-activate-error = Impossibile passare all'instradamento
admin-egress-deactivate-error = Impossibile smettere di usare l'instradamento
admin-egress-col-state = Stato
admin-egress-state-active = In uso
admin-egress-state-inactive = Non in uso
admin-egress-test-scope = Si può verificare solo l'instradamento in uso. Una verifica scarica un URL fisso attraverso l'instradamento e indica fin dove è arrivata; non modifica nulla.
admin-egress-probe-url = Scaricato: { $url }
admin-egress-load-error = Impossibile caricare gli instradamenti in uscita
admin-egress-save-error = Impossibile salvare l'instradamento
admin-egress-delete-error = Impossibile eliminare l'instradamento
admin-egress-invalid = Sono richiesti un id, un host e una porta maggiore di zero.
admin-egress-username-required = Una password richiede un nome utente; senza di esso l'instradamento si autenticherebbe come nessuno.
admin-egress-col-id = Id
admin-egress-col-route = Instradamento
admin-egress-col-username = Nome utente
admin-egress-col-credentials = Password
admin-egress-col-plaintext = In chiaro
admin-egress-col-actions = Azioni
admin-egress-cred-set = Impostata
admin-egress-cred-none = Nessuna
admin-egress-plaintext-on = Consentito
admin-egress-plaintext-off = Rifiutato
admin-egress-add-heading = Aggiungi un instradamento
admin-egress-edit-heading = Modifica instradamento
admin-egress-id = Id
admin-egress-scheme = Schema
admin-egress-host = Host
admin-egress-port = Porta
admin-egress-username = Nome utente
admin-egress-password = Password
admin-egress-password-unchanged = Lasciare vuoto per mantenere la password memorizzata
admin-egress-allow-plaintext = Consenti origini http in chiaro attraverso questo instradamento
admin-egress-allow-plaintext-note = Disattivato per impostazione predefinita. Un'origine https impedisce al proxy di leggere il traffico che trasporta; consentire http significa che il proxy lo vede.
admin-egress-test = Verifica
admin-egress-testing = Verifica in corso…
admin-egress-test-failed = Non è stato possibile eseguire la verifica, quindi non si sa nulla di questo instradamento.
admin-egress-outcome-connected = Connesso
admin-egress-outcome-auth-rejected = Autenticazione rifiutata
admin-egress-outcome-refused-by-policy = Rifiutato dal criterio
admin-egress-outcome-dns-failed = Nome non risolto
admin-egress-outcome-unreachable = Irraggiungibile
admin-egress-outcome-origin-tls-failed = TLS dell'origine non riuscito
admin-egress-outcome-route-invalid = Instradamento non valido
admin-egress-outcome-unknown = Risultato non riconosciuto
admin-egress-stage-dns = fermato alla risoluzione del nome
admin-egress-stage-connect = fermato durante la connessione al proxy
admin-egress-stage-tunnel = fermato durante l'apertura del tunnel
admin-egress-stage-origin = fermato all'origine
admin-egress-stage-unknown = fermato in una fase non riconosciuta
admin-egress-proxied-yes = Passato attraverso il proxy
admin-egress-proxied-no = Non passato attraverso il proxy
