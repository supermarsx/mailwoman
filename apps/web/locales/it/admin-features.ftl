# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Profili delle funzioni
admin-features-intro = Un profilo è un insieme denominato di interruttori di funzione. Lo assegni a un dominio, a un gruppo o a un utente. Le assegnazioni individuali supportano caselle native e identità proxy o SSO verificate; controllano l'accesso alle funzioni di Mailwoman.
admin-features-new = Nuovo profilo
admin-features-search = Cerca profili
admin-features-empty = Ancora nessun profilo delle funzioni.
admin-features-no-match = Nessun profilo corrisponde alla ricerca.
admin-features-load-error = Impossibile caricare il catalogo delle funzioni.
admin-features-not-loaded = Il catalogo delle funzioni non è ancora caricato. Ricarichi la pagina e riprovi.
admin-features-stale = Un altro amministratore ha modificato il catalogo delle funzioni. È stato ricaricato; controlli i valori e salvi di nuovo.
admin-features-not-found = Nessun profilo con l'id { $id }.
admin-features-col-name = Nome
admin-features-col-values = Interruttori
admin-features-col-assigned = Assegnato a
admin-features-values-count = { $count ->
    [one] { $count } interruttore impostato
   *[other] { $count } interruttori impostati
 }
admin-features-assigned-count = { $count ->
    [0] Non assegnato
    [one] { $count } posizione
   *[other] { $count } posizioni
 }
admin-features-editor = Impostazioni del profilo
admin-features-name = Nome del profilo
admin-features-name-help = Mostrato solo agli amministratori, fino a 128 caratteri.
admin-features-name-invalid = Inserisca un nome da 1 a 128 caratteri.
admin-features-switches = Interruttori di funzione
admin-features-switches-help = "Non impostato" lascia la funzione al profilo successivo, al dominio o al valore predefinito del server. Un valore bloccato non può essere modificato dall'utente né da un livello inferiore.
admin-features-col-feature = Funzione
admin-features-col-value = Valore
admin-features-col-lock = Blocco
admin-features-value-for = Valore per { $feature }
admin-features-unset = Non impostato
admin-features-on = Consentita
admin-features-off = Non consentita
admin-features-lock = Bloccata
admin-features-save = Salva il profilo
admin-features-create = Crea il profilo
admin-features-delete = Elimina il profilo
admin-features-delete-title = Eliminare il profilo { $name }?
admin-features-delete-body = { $count ->
    [0] Il profilo non è assegnato da nessuna parte. I suoi interruttori smettono subito di applicarsi.
    [one] Il profilo viene rimosso dall'unica posizione ({ $count }) a cui è assegnato e i suoi interruttori smettono subito di applicarsi.
   *[other] Il profilo viene rimosso dalle { $count } posizioni a cui è assegnato e i suoi interruttori smettono subito di applicarsi.
 }
admin-features-saved = Profilo { $name } salvato.
admin-features-deleted = Profilo { $name } eliminato.
admin-features-assigned = Profilo { $name } assegnato.
admin-features-unassigned = Assegnazione di { $name } rimossa.
admin-features-assignments = Assegnato a
admin-features-assignments-help = I profili si applicano in ordine di priorità, dal più alto. A parità di priorità, un'assegnazione a un utente si applica prima di un gruppo e un gruppo prima di un dominio.
admin-features-assignments-empty = Questo profilo non è ancora assegnato.
admin-features-assign-form = Assegna questo profilo
admin-features-col-scope = Si applica a
admin-features-col-target = Nome
admin-features-col-priority = Priorità
admin-features-col-actions = Azioni
admin-features-scope-domain = Dominio
admin-features-scope-group = Gruppo
admin-features-scope-user = Utente
admin-features-assign = Assegna
admin-features-unassign = Rimuovi
admin-features-no-domain = Nessun dominio configurato.
admin-features-no-group = Non esistono gruppi di utenti.
admin-features-no-user = Nessun account di casella ha ancora effettuato l'accesso.
admin-features-tools = Strumenti del profilo
admin-features-preset = Inizia da un preset
admin-features-preset-choose = Scelga un preset
admin-features-preset-apply = Applica il preset
admin-features-preset-applied = Preset { $name } applicato. Controlli i valori e salvi.
admin-features-preset-full = Completo
admin-features-preset-standard = Standard
admin-features-preset-mail-only = Solo posta
admin-features-preset-minimal = Minimo / chiosco
admin-features-import = Importa JSON
admin-features-export = Esporta JSON
admin-features-exported = Profilo esportato come file JSON.
admin-features-imported = { $count ->
    [one] Importato { $count } valore. Lo controlli e salvi.
   *[other] Importati { $count } valori. Li controlli e salvi.
 }
admin-features-import-json = Il file non è un JSON valido.
admin-features-import-format = Il file non è un profilo delle funzioni di Mailwoman.
admin-features-import-version = Questo file di profilo delle funzioni proviene da una versione più recente e non può essere importato.
admin-features-clone = Duplica
admin-features-clone-name = Copia di { $name }
admin-features-filter = Trova una funzione
admin-features-filter-placeholder = Nome, impostazione o chiave
admin-features-filter-none = Nessuna funzione corrisponde alla ricerca.
admin-features-diff-summary = { $differs ->
    [one] { $differs } valore differisce dal predefinito del server
   *[other] { $differs } valori differiscono dal predefinito del server
} · { $same } lo ripetono
admin-features-differs = Differisce dal predefinito
admin-features-default-is = Predefinito: { $value }
admin-features-fix-values = Corregga i valori contrassegnati come non validi prima di salvare.
admin-features-allow-all = Consenti tutto
admin-features-deny-all = Non consentire nulla
admin-features-clear-all = Cancella
admin-features-group-allow = Consenti ogni funzione in { $group }
admin-features-group-deny = Non consentire alcuna funzione in { $group }
admin-features-group-clear = Cancella ogni valore in { $group }
admin-features-enforcement-ui = Nascosta solo nell'app
admin-features-enforcement-partial = Alcuni limiti non sono ancora conteggiati
admin-features-other = Altre impostazioni che un profilo può contenere
admin-features-other-help = Ogni altra impostazione che un amministratore può definire per profilo. Lasci vuoto un campo per mantenere il valore del server.
admin-features-cat-compose = Posta e composizione
admin-features-cat-reading = Lettura e privacy
admin-features-cat-views = Viste e ricerca
admin-features-cat-organise = Organizzazione
admin-features-cat-pim = Calendario, contatti, attività, note e chiavi
admin-features-cat-data = Dati
admin-features-cat-accounts = Account e accesso
admin-features-cat-integrations = Integrazioni
admin-features-cat-developer = Accesso per sviluppatori
admin-features-cat-personal = Personalizzazione
admin-features-feature-compose-sending = Limiti di invio
admin-features-feature-compose-sending-help = I destinatari per messaggio vengono rifiutati dal server oltre il limite. I messaggi all'ora e al giorno sono memorizzati per il profilo ma non ancora conteggiati.
admin-features-feature-storage = Quota di archiviazione
admin-features-feature-storage-help = Il limite di archiviazione è memorizzato per il profilo; la quota propria del server di posta continua ad applicarsi.
admin-features-feature-access-protocols = Protocolli di accesso alla posta
admin-features-feature-access-protocols-help = Con quali protocolli del server di posta questi utenti possono accedere, separati da virgole: imap, pop3, ews.
admin-features-feature-accounts-linked = Caselle collegate
admin-features-feature-calendar = Calendario
admin-features-feature-contacts = Contatti
admin-features-feature-tasks = Attività
admin-features-feature-notes = Note
admin-features-preview = Valori effettivi
admin-features-preview-help = A cosa si risolve ogni funzione per un utente o per un utente di un dominio, dopo ogni profilo, gruppo, valore di dominio e blocco.
admin-features-preview-kind = Anteprima per
admin-features-preview-target = Chi
admin-features-preview-show = Mostra
admin-features-preview-loading = Caricamento…
admin-features-preview-state = Effettivo
admin-features-preview-from = Deciso da
admin-features-preview-profiles = Profili che si applicano, vince il primo: { $names }
admin-features-preview-no-profiles = Nessun profilo si applica; decidono i valori del dominio e del server.
admin-features-source-default = Predefinito integrato
admin-features-source-env-fallback = Predefinito dell'ambiente
admin-features-source-env-pin = Ambiente (fissato)
admin-features-source-global = Valore del server
admin-features-source-domain = Valore del dominio
admin-features-source-profile = Profilo
admin-features-source-profile-named = Profilo { $name }
admin-features-source-group = Valore del gruppo
admin-features-source-admin-user = Valore impostato per questo utente
admin-features-source-user = Scelta dell'utente stesso
admin-features-source-locked = { $source } (bloccato)
admin-features-feature-compose-open-tracking = Tracciamento delle aperture dei messaggi
admin-features-feature-compose-open-tracking-help = Consente di registrare le aperture dei messaggi formattati idonei tramite l'immagine di tracciamento del server. Richiede un URL pubblico configurato.
admin-features-feature-reading-original-html = Vista HTML originale
admin-features-feature-reading-original-html-help = Consente agli utenti di scegliere l'HTML originale per singoli messaggi, all'interno del frame di lettura isolato. L'HTML ripulito resta il predefinito.
admin-features-inherit = Eredita
admin-features-override = Imposta un valore
admin-features-setting-mode = Modalità del valore per { $setting }
admin-features-inherit-help = Eredita rimuove il valore e il blocco di questo profilo. Decide il successivo profilo, dominio o valore del server applicabile; l'anteprima effettiva mostra il risultato e la sua origine.
admin-features-domain-alias = { $alias } (alias di { $domain })
admin-features-domain-aliases = { $domain } (alias: { $aliases })

# -- Machine translation (2026-11): added to match en ---------------------
admin-retention-col-category = Categoria
admin-retention-col-kept = Conservato
admin-retention-col-change = Modifica
admin-retention-cat-default = Predefinita
admin-retention-cat-admin = Azioni degli amministratori
admin-retention-cat-security = Eventi di sicurezza
admin-retention-cat-alert = Avvisi per gli utenti
admin-retention-forever = Ogni voce
admin-retention-days = { $days ->
    [one] { $days } giorno
   *[other] { $days } giorni
 }
admin-retention-follows-default = Predefinito: { $kept }
admin-retention-hold-on = Conservazione legale
admin-retention-read-only = La modifica della conservazione richiede il permesso settings.write.
admin-retention-edit-for = Conservazione per { $category }
admin-retention-keep = Conserva
admin-retention-mode-inherit = Segui il predefinito
admin-retention-mode-forever = Ogni voce
admin-retention-mode-days = Per un numero di giorni
admin-retention-days-label = Giorni
admin-retention-hold = Conservazione legale
admin-retention-save = Salva
admin-retention-bounds = Inserisca un numero intero da { $min } a { $max }.
admin-retention-saved = Conservazione per { $category } salvata.
admin-retention-task-none = Nessuna attività di pulizia ancora. Viene creata, ogni giorno alle 03:15 UTC, quando viene salvato per la prima volta un numero di giorni.
admin-retention-task-disabled = L'attività di pulizia pianificata è disattivata; le voci vengono eliminate solo con Pulisci ora.
admin-retention-task-next = Prossima pulizia pianificata: { $when }.
admin-retention-task-daily = Pulizia ogni giorno alle 03:15 UTC.
admin-retention-last = Ultima pulizia che ha rimosso voci: { $when }.
admin-retention-prune = Pulisci ora
admin-retention-prune-title = Pulire ora il registro di audit?
admin-retention-prune-body = Le voci più vecchie della conservazione della loro categoria vengono eliminate. Le categorie che conservano ogni voce o sono sotto conservazione legale non vengono modificate. L'operazione non può essere annullata.
admin-retention-pruned = { $count ->
    [0] Nessuna voce era abbastanza vecchia da essere rimossa.
    [one] { $count } voce rimossa.
   *[other] { $count } voci rimosse.
 }
admin-auth-attempted = Tentato come { $name }
