# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Amministrazione completa di ogni funzionalità, compresi amministratori, ruoli, sicurezza, instradamento della posta e configurazione dei servizi. Riserva questo ruolo a operatori fidati responsabili dell'intera installazione.
admin-role-purpose-domain_admin = Amministrazione quotidiana di utenti e domini: creare, aggiornare, disattivare o eliminare utenti; gestire quote, funzionalità degli utenti, regole di accesso del dominio, impostazioni e personalizzazione; revocare sessioni e gestire i blocchi. Include anche la diagnostica della posta e le esportazioni di utilizzo e dati. Non concede la gestione dei ruoli di amministratore né modifiche ai server upstream.
admin-role-purpose-helpdesk = Supporto agli account e smistamento degli incidenti: esaminare utenti, attività di audit e utilizzo; modificare i flag degli utenti, disattivare account e revocare sessioni; vedere i blocchi ed eseguire simulazioni di accesso. Non concede la creazione o l'eliminazione di utenti, le modifiche alle quote né la scrittura delle impostazioni generali.
admin-role-purpose-auditor = Ispezione e reportistica: concede ogni funzionalità che termina con .read, più l'esportazione di audit e utilizzo. Non concede modifiche alla configurazione né agli account. I report esportati possono contenere informazioni amministrative sensibili.
admin-role-purpose-security_officer = Operazioni di sicurezza: gestire i criteri di sicurezza e MFA, il criterio di accesso e i blocchi; disattivare utenti, revocare sessioni o chiavi API; esaminare ed esportare l'attività di audit; leggere i log di autenticazione ed eseguire diagnostica. La configurazione SSO è leggibile, ma questo ruolo non concede la modifica dell'SSO né la gestione dei ruoli di amministratore.
admin-role-purpose-custom = Un ruolo personalizzato con { $count } permessi selezionati. La sua autorità deriva dall'elenco esatto dei permessi qui sotto, non dal nome. Controlla ogni azione prima di assegnarlo.
admin-role-purpose-scope = L'ambito di dominio viene assegnato quando questo ruolo è concesso a un amministratore. Le eccezioni consenti/nega a livello di account possono cambiare l'accesso effettivo. Una descrizione non concede permessi aggiuntivi.
admin-role-purpose-editor = Scegli le attività che questo amministratore deve svolgere, poi seleziona i permessi corrispondenti. Accesso in lettura, esportazione dei dati e modifica dei record sono azioni separate. Controlla gli identificatori esatti delle funzionalità prima di salvare.
admin-role-purpose-selection = Queste descrizioni spiegano le azioni selezionate. Non ampliano i permessi inviati al server.
admin-role-purpose-read = Visualizzare le informazioni di quest'area.
admin-role-purpose-export = Esportare le informazioni di quest'area.
admin-role-purpose-revoke = Revocare le sessioni o le credenziali indicate da questo permesso.
admin-role-purpose-delete = Eliminare i record coperti da questo permesso.
admin-role-purpose-disable = Disattivare gli account coperti da questo permesso.
admin-role-purpose-create = Creare i record coperti da questo permesso.
admin-role-purpose-specific = Eseguire l'azione specifica indicata da questa funzionalità; controlla il suo identificatore prima di concederla.
admin-role-purpose-description = Scopo e responsabilità
admin-role-purpose-description-help = Spiega chi dovrebbe ricevere questo ruolo personalizzato e quali attività svolge (fino a 1024 caratteri). Questo testo non modifica i suoi permessi.
