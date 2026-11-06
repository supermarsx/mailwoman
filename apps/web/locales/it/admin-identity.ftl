# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Provider di identità
admin-identity-description = Consenti alle applicazioni approvate di accedere con Mailwoman. È separato dai provider di accesso esterni e dall'accesso all'API di posta. È richiesto il codice di autorizzazione con PKCE; gli altri grant non sono disponibili.
admin-identity-configure = Configura l'emittente
admin-identity-new = Aggiungi client di identità
admin-identity-edit = Modifica client
admin-identity-enabled = Attivo
admin-identity-disabled = Disattivato
admin-identity-all = Tutti
admin-identity-discovery = Apri il documento di discovery
admin-identity-search = Cerca client
admin-identity-status = Stato
admin-identity-clients = Client di identità
admin-identity-client = Identità del client
admin-identity-client-id = ID client
admin-identity-name = Nome
admin-identity-method = Autenticazione del client
admin-identity-public = Client pubblico (PKCE, nessun segreto)
admin-identity-pkce = Ogni client deve usare PKCE S256. I segreti dei client riservati vengono generati qui e mostrati una sola volta.
admin-identity-rotate-secret = Sostituisci il segreto del client esistente
admin-identity-redirects = Destinazioni registrate
admin-identity-redirect-help = Un URI HTTPS esatto per riga; HTTP su loopback numerico è consentito per le applicazioni locali. Caratteri jolly e frammenti vengono rifiutati.
admin-identity-callbacks = URI di callback di autorizzazione
admin-identity-logout-uris = URI di reindirizzamento dopo il logout
admin-identity-scopes = Ambiti di identità consentiti
admin-identity-actions = Azioni
admin-identity-delete = Elimina client
admin-identity-delete-confirm = Eliminare l'accesso di identità per { $name }? I grant di identità esistenti smettono di funzionare e la sua registrazione di client solo identità viene rimossa.
admin-identity-empty = Nessun client di identità corrispondente.
admin-identity-cancel = Annulla
admin-identity-save = Salva
admin-identity-done = Fatto
admin-identity-secret = Segreto del client generato
admin-identity-secret-once = Copia ora questo segreto nell'applicazione relying party. Non potrà essere mostrato di nuovo. Non inserire mai un segreto riservato nel codice del browser.
admin-identity-issuer = URL HTTPS pubblico dell'emittente
admin-identity-issuer-help = Usa l'origine HTTPS esatta di questa installazione e il percorso /oidc, incluso l'eventuale prefisso dell'installazione. Cambiare l'emittente o la configurazione revoca i grant di identità esistenti.
admin-identity-rotate-key = Ruota la chiave di firma
admin-identity-key-help = La rotazione richiede una verifica recente dell'amministratore. Le chiavi pubbliche precedenti restano disponibili per 15 minuti in modo che i token emessi da poco possano concludere la loro breve durata.
