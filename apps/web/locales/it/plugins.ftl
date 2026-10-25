# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Avviso plugin UI non firmati
plugins-unsigned-title = Sono in esecuzione plugin non firmati
plugins-unsigned-text = Questi plugin sono stati ammessi senza una firma verificata. Vengono eseguiti in sandbox, ma il loro codice non è stato verificato con una chiave attendibile.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plugin: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = I plugin del motore vengono eseguiti in una sandbox WebAssembly. Un plugin viene eseguito solo dopo essere stato registrato, approvato, aver ricevuto almeno una capacità ed essere stato attivato. Può usare solo le capacità concesse qui.
plugins-admin-unsigned-banner = È caricato un plugin di terze parti senza firma. Viene eseguito perché un amministratore lo ha consentito; il server verifica il suo file solo rispetto al digest approvato.
plugins-admin-load-error = Impossibile caricare i plugin.
plugins-admin-register-heading = Registra un plugin
plugins-admin-register-component = Componente
plugins-admin-register-third-party = Componente di terze parti
plugins-admin-register-id = Id del plugin
plugins-admin-register-id-hint = Il server legge il componente dal file con il nome di questo id nella directory dei plugin di terze parti. Approvare prima il digest di quel file nell'elenco consentiti qui sotto.
plugins-admin-register-name = Nome
plugins-admin-register-version = Versione
plugins-admin-register-capabilities = Capacità dichiarate dal plugin
plugins-admin-register-hosts = Host che il plugin può contattare
plugins-admin-register-hosts-hint-first-party = Nomi host separati da virgole. Lasciare vuoto per usare gli host predefiniti del componente.
plugins-admin-register-hosts-hint-third-party = Nomi host separati da virgole. Lasciare vuoto per nessun accesso alla rete.
plugins-admin-register-submit = Registra
plugins-admin-trust-first-party = Proprio, digest incorporato
plugins-admin-trust-third-party = Terze parti, digest approvato
plugins-admin-status-loaded = Caricato
plugins-admin-status-not-loaded = Non caricato
plugins-admin-status-restart = Riavvio necessario
plugins-admin-restart-to-load = Riavvio necessario per caricare questo plugin.
plugins-admin-restart-to-apply = Riavvio necessario per applicare queste modifiche. Alcuni account di questo bridge non vengono serviti fino ad allora.
plugins-admin-reason-not-approved = Non caricato: un amministratore non ha approvato questo plugin.
plugins-admin-reason-disabled = Non caricato: il plugin non è attivo.
plugins-admin-reason-unsigned-not-allowed = Non caricato: il plugin non ha firma e non è stato consentito eseguirlo senza.
plugins-admin-reason-no-grant = Non caricato: non è stata concessa alcuna capacità di cui ha bisogno.
plugins-admin-reason-component-unavailable = Non caricato: nessun file del componente ha superato la verifica del digest.
plugins-admin-reason-load-failed = Non caricato: il server non è riuscito a caricare il componente. Vedere il log del server.
plugins-admin-reason-proxy-mode = Non caricato: questo server funziona in modalità proxy e non elabora direttamente la posta.
plugins-admin-reason-no-account-binding = Non caricato: nessun account è associato a questo bridge.
plugins-admin-reason-no-host-caller = Non caricato: questa versione del server non chiama gli hook forniti da questo plugin.
plugins-admin-reason-another-classifier-active = Non caricato: è caricato un altro classificatore antispam. Ne viene eseguito uno solo alla volta.
plugins-admin-running-with = In esecuzione con: { $capabilities }
plugins-admin-hosts = Host: { $hosts }
plugins-admin-hosts-none = Host: nessuno
plugins-admin-grants-heading = Capacità concesse
plugins-admin-grants-hint = Il plugin può usare solo le capacità selezionate qui. Il salvataggio sostituisce la concessione precedente.
plugins-admin-grant-for = Concedi { $capability } a { $name }
plugins-admin-grants-save = Salva concessioni
plugins-admin-endpoint = Indirizzo del classificatore
plugins-admin-endpoint-for = Indirizzo del classificatore per { $name }
plugins-admin-endpoint-hint = Dove il plugin raggiunge il suo daemon. L'host deve essere anche nell'elenco degli host del plugin. Lasciare vuoto per usare il valore predefinito del componente.
plugins-admin-endpoint-save = Salva indirizzo
plugins-admin-test = Verifica classificatore
plugins-admin-test-verdict = Esito della verifica: { $verdict }
plugins-admin-uninstall = Disinstalla
plugins-admin-uninstall-confirm = Conferma disinstallazione
plugins-admin-uninstall-note = La disinstallazione rimuove la registrazione, le sue concessioni e le sue impostazioni memorizzate.
plugins-admin-uninstall-detail = La disinstallazione elimina i dati memorizzati dal plugin per tutti gli account, i suoi vincoli nell'elenco consentiti, la registrazione e le concessioni. Il file del componente su disco non viene eliminato.
plugins-admin-revoke-detail = La revoca rimuove l'approvazione di questo digest e disattiva il plugin. Se il plugin è in esecuzione, viene fermato subito.
plugins-admin-error-already-registered = Questo plugin è già registrato. Disinstallarlo prima di registrarlo di nuovo.
plugins-admin-error-component-unavailable = Nessun file del componente di questo plugin corrisponde al digest incorporato nel server.
plugins-admin-error-digest-not-approved = Non è stato trovato alcun file del componente con un digest approvato per questo id. Approvare prima il digest del file nell'elenco consentiti.
plugins-admin-error-not-approved = Approvare il plugin prima di attivarlo.
plugins-admin-error-unsigned-not-allowed = Questo plugin non ha firma. Consentirne l'esecuzione senza firma prima di attivarlo.
plugins-admin-error-not-loaded = Il plugin non è caricato, quindi non può essere verificato.
plugins-admin-error-classifier-error = La chiamata al classificatore non è riuscita: { $detail }
plugins-admin-error-other = Il server ha rifiutato la richiesta: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = Plugin UI
plugins-admin-ui-title = Plugin UI
plugins-admin-ui-intro = I plugin UI aggiungono controlli al client web. Ognuno viene eseguito in un frame in sandbox e può usare solo le capacità concesse qui. Un plugin viene offerto agli utenti connessi dopo essere stato approvato e attivato.
plugins-admin-ui-empty = Nessun plugin UI registrato.
plugins-admin-ui-load-error = Impossibile caricare i plugin UI.
plugins-admin-ui-register-heading = Registra un plugin UI
plugins-admin-ui-manifest = Manifest (JSON)
plugins-admin-ui-bundle = File del bundle
plugins-admin-ui-bundle-hint = Obbligatorio per un plugin firmato: la firma viene verificata su questo file.
plugins-admin-ui-allow-unsigned = Consenti la registrazione di questo plugin senza firma
plugins-admin-ui-register = Registra
plugins-admin-ui-manifest-invalid = Il manifest non è un JSON valido.
plugins-admin-ui-refused = Il server ha rifiutato la richiesta: { $detail }
plugins-admin-ui-approve = Approva e attiva
plugins-admin-ui-extension-points = Punti di estensione: { $points }
plugins-admin-ui-grants-heading = Concedi una capacità
plugins-admin-ui-grants-note = Il server non indica quali capacità sono già concesse. Concedere di nuovo una capacità sostituisce la concessione precedente.
plugins-admin-ui-grant = Concedi { $capability }
plugins-admin-ui-grant-for = Concedi { $capability } a { $name }
plugins-admin-ui-grant-hosts = Host per { $capability } su { $name }
plugins-admin-ui-grant-hosts-hint = Nomi host separati da virgole da cui il plugin può scaricare dati.
plugins-admin-ui-granted = { $capability } concessa.
plugins-admin-ui-delete = Elimina
plugins-admin-ui-delete-confirm = Conferma eliminazione
