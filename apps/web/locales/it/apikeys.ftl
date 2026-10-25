# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Chiavi API
apikeys-heading = Chiavi API
apikeys-intro = Creare chiavi con ambito limitato per script e integrazioni. Ogni chiave è archiviata come hash, mostrata una sola volta e revocabile singolarmente. Concedere l'ambito minimo necessario.

apikeys-label = Etichetta
apikeys-label-aria = Etichetta della chiave
apikeys-label-placeholder = es. script di backup

apikeys-create = Crea chiave
apikeys-saved = L'ho salvata

apikeys-reveal-warning = Copiare subito questo segreto — viene mostrato una sola volta e non può essere recuperato.
apikeys-copy = Copia
apikeys-copy-aria = Copia il segreto negli appunti
apikeys-copied = Copiato negli appunti

apikeys-error-need-label = assegnare un'etichetta alla chiave per riconoscerla in seguito
apikeys-error-create = impossibile creare la chiave
apikeys-error-revoke = impossibile revocare la chiave

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Chiavi esistenti
apikeys-existing = Chiavi esistenti
apikeys-none = Ancora nessuna chiave.
apikeys-created = creata il { $date }
apikeys-last-used = ultimo uso { $date }
apikeys-never-used = mai usata
apikeys-revoked-at = revocata il { $date }
apikeys-revoke = Revoca
apikeys-status-active = Attiva
apikeys-status-revoked = Revocata

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Ambito
apikeys-capabilities = Permessi
apikeys-cap-read = Lettura
apikeys-cap-send = Invio
apikeys-cap-delete = Eliminazione
apikeys-surface = Area
apikeys-surface-mail-label = Posta
apikeys-surface-pim-label = PIM (calendario / attività / note / contatti)
apikeys-accounts = Account
apikeys-all-accounts = Tutti gli account
apikeys-folders = Cartelle
apikeys-all-folders = Tutte le cartelle
apikeys-folder-ids = ID delle cartelle (separati da virgole)
apikeys-folder-subset-aria = Sottoinsieme di cartelle
apikeys-constraints = Vincoli
apikeys-expiry = Scadenza (RFC 3339, vuoto = nessuna scadenza)
apikeys-expiry-aria = Scadenza
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Limite di frequenza (richieste/min, vuoto = illimitato)
apikeys-rate-limit-aria = Limite di frequenza
apikeys-ip-allowlist = IP consentiti (CIDR/IP, separati da virgole; vuoto = qualsiasi)
apikeys-ip-allowlist-aria = IP consentiti
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = lettura
apikeys-verb-send = invio
apikeys-verb-delete = eliminazione
apikeys-summary-mail = posta
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = nessuna operazione
apikeys-summary-nothing = niente
apikeys-summary-none = nessuno
apikeys-summary-verbs-on = { $verbs } su { $surfaces }
apikeys-summary-all-accounts = tutti gli account
apikeys-summary-accounts = account: { $ids }
apikeys-summary-all-folders = tutte le cartelle
apikeys-summary-folders = cartelle: { $ids }
apikeys-summary-expires = scade il { $date }
apikeys-summary-rate = { $n } rich./min
apikeys-summary-ips = IP: { $ips }
apikeys-summary-mcp-tools = Strumenti MCP: { $tools }
apikeys-summary-unattended = Invio NON PRESIDIATO

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Chiavi MCP
apikeys-mcp-heading = Chiavi MCP
apikeys-mcp-intro = Concedere a un agente IA (tramite il Model Context Protocol) esattamente gli strumenti scelti. Il corpo dei messaggi restituito all'agente è etichettato come input non attendibile. Ogni strumento viene concesso singolarmente.
apikeys-mcp-label-aria = Etichetta della chiave MCP
apikeys-mcp-label-placeholder = es. agente assistente
apikeys-mcp-create = Crea chiave MCP
apikeys-mcp-reveal-warning = Copiare subito questo segreto — viene mostrato una sola volta.
apikeys-mcp-error-need-label = assegnare un'etichetta alla chiave MCP
apikeys-mcp-error-need-tool = concedere almeno uno strumento
apikeys-mcp-error-create = impossibile creare la chiave MCP

apikeys-tools = Strumenti
apikeys-tools-group = Strumenti MCP
apikeys-outbox-suffix = (soggetto a Posta in uscita)

apikeys-unattended-aria = Invio non presidiato
apikeys-unattended-allow = Consenti l'invio non presidiato (salta la Posta in uscita — richiede la controfirma di un amministratore)
apikeys-unattended-pending = L'invio non presidiato resta inattivo finché un amministratore non controfirma questa chiave.
apikeys-unattended-disclosure = Per impostazione predefinita, un invio concesso finisce nella Posta in uscita e attende la conferma nell'app — l'automazione non può inviare da sola. L'invio non presidiato RIMUOVE questo passaggio di controllo umano, così questa chiave può inviare posta senza conferma. Richiede inoltre che un amministratore controfirmi la chiave. Concederlo solo ad automazioni di cui ci si fida pienamente; una chiave con invio non presidiato compromessa può inviare posta a proprio nome senza alcuna richiesta di conferma.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Cerca nella posta
apikeys-tool-mail-search-desc = Cerca messaggi. I risultati sono etichettati come input non attendibile.
apikeys-tool-mail-read-label = Leggi posta
apikeys-tool-mail-read-desc = Legge il corpo di un messaggio. I corpi sono etichettati come input non attendibile.
apikeys-tool-folders-list-label = Elenca cartelle
apikeys-tool-folders-list-desc = Elenca le cartelle della casella.
apikeys-tool-drafts-create-label = Crea bozze
apikeys-tool-drafts-create-desc = Crea una bozza (mai inviata automaticamente).
apikeys-tool-mail-send-label = Invia posta
apikeys-tool-mail-send-desc = Mette un messaggio in coda nella Posta in uscita per la conferma nell'app.
apikeys-tool-calendar-read-label = Leggi calendario
apikeys-tool-calendar-read-desc = Legge gli eventi del calendario.
apikeys-tool-calendar-propose-label = Proponi eventi
apikeys-tool-calendar-propose-desc = Propone (senza confermare) eventi del calendario.
apikeys-tool-tasks-read-label = Leggi attività
apikeys-tool-tasks-read-desc = Legge le attività.
apikeys-tool-tasks-write-label = Scrivi attività
apikeys-tool-tasks-write-desc = Crea o aggiorna attività.
apikeys-tool-contacts-read-label = Leggi contatti
apikeys-tool-contacts-read-desc = Legge i contatti.
