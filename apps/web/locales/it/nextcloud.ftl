# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Su
nextcloud-loading = Caricamento…
nextcloud-list-error = Impossibile elencare il contenuto di questa cartella.
nextcloud-empty = Questa cartella è vuota.
nextcloud-file-list = File di Nextcloud
nextcloud-open-folder = Apri la cartella { $name }
nextcloud-select-file = Seleziona { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Allega da Nextcloud
nextcloud-attach-action = { $count ->
    [0] Allega
    [one] Allega { $count } file
   *[other] Allega { $count } file
  }
nextcloud-error-select-file = selezionare almeno un file da allegare
nextcloud-error-attach-failed = impossibile allegare i file selezionati

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Salva su Nextcloud
nextcloud-save-title = Salva “{ $name }” su Nextcloud
nextcloud-save-action = Salva qui ({ $dir })
nextcloud-saved = Salvato in { $path }
nextcloud-error-save-failed = impossibile salvare su Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Crea link di condivisione
nextcloud-share-title = Link di condivisione
nextcloud-share-intro = Creare un link pubblico a { $path } invece di allegare il file.
nextcloud-protect-password = Proteggi con password
nextcloud-password-label = Password
nextcloud-share-password = Password di condivisione
nextcloud-set-expiry = Imposta una data di scadenza
nextcloud-expires-on = Scade il
nextcloud-expiry-date = Data di scadenza
nextcloud-create-link = Crea link
nextcloud-password-protected = Protetto da password
nextcloud-no-password = Nessuna password
nextcloud-expires = scade il { $date }
nextcloud-no-expiry = nessuna scadenza
nextcloud-error-need-password = inserire una password o disattivare la protezione con password
nextcloud-error-need-expiry = scegliere una data di scadenza o disattivare la scadenza
nextcloud-error-share-failed = impossibile creare il link di condivisione

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } supera il limite degli allegati. Caricalo nella cartella “Mail attachments” del suo Nextcloud e inserisci nel messaggio un link di condivisione.
nextcloud-upload-and-link = Carica e inserisci il link
nextcloud-share-not-now = Non ora
nextcloud-upload-progress = Caricamento: { $percent }%
nextcloud-upload-progress-label = Avanzamento del caricamento
nextcloud-share-inserted = Link di condivisione per { $name } inserito nel messaggio.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = Il link funziona fino al { $date }.
nextcloud-share-body-password = Il link è protetto da password.
nextcloud-account-title = Account Nextcloud
nextcloud-account-help = Collega il suo account a un'istanza approvata dall'amministratore. Le credenziali sono memorizzate cifrate solo per questo account di posta.
nextcloud-account-load-error = Impossibile caricare i collegamenti Nextcloud. Ricarica per riprovare.
nextcloud-account-unlink-error = Impossibile rimuovere il collegamento. Ricarica prima di riprovare.
nextcloud-account-reload = Ricarica
nextcloud-account-no-instances = Nessuna istanza Nextcloud attiva disponibile. Chiedi all'amministratore di configurarne una.
nextcloud-account-ready = Collegato e pronto
nextcloud-account-stale = La configurazione dell'istanza è cambiata. Verifica di nuovo il collegamento.
nextcloud-account-unlinked = Non collegato
nextcloud-account-relink = Verifica di nuovo il collegamento
nextcloud-account-link = Collega account
nextcloud-account-unlink = Scollega account
nextcloud-account-password-help = Nelle impostazioni di sicurezza personali di Nextcloud, crea una password per app e copia qui il suo nome utente e la password. La verifica effettua una sola richiesta limitata; le credenziali rifiutate non vengono riprovate.
nextcloud-account-username = Nome utente della password per app di Nextcloud
nextcloud-account-password = Password per app di Nextcloud
nextcloud-account-verify = Verifica e collega
nextcloud-account-verify-error = Verifica o salvataggio non riusciti. Controlla l'istanza, il nome utente e la password per app. Il campo password è stato svuotato; un collegamento funzionante esistente non è stato sostituito.
nextcloud-admin-title = Istanze Nextcloud
nextcloud-admin-help = Approva le istanze HTTPS che gli utenti di posta possono collegare singolarmente. Agli account di posta non viene fornita alcuna password di servizio condivisa. Le operazioni sui file sono limitate a 8 MiB. Gli utenti del motore nativo, dopo il collegamento, individuano e connettono esplicitamente le raccolte di calendari, attività e rubriche.
nextcloud-admin-add = Aggiungi istanza Nextcloud
nextcloud-admin-edit = Modifica istanza Nextcloud
nextcloud-admin-enabled = Attiva
nextcloud-admin-disabled = Disattivata
nextcloud-admin-missing = Questa istanza non esiste più. Torna all'elenco delle istanze.
nextcloud-admin-private-help = Gli indirizzi privati richiedono un permesso di egress Nextcloud esplicito. I percorsi proxy configurati restano determinanti; una richiesta rifiutata dal proxy non ripiega mai su una connessione diretta.
nextcloud-admin-revalidation = Salvare qualsiasi modifica a un'istanza richiede agli utenti di verificare di nuovo i loro collegamenti. Disattivare un'istanza interrompe subito le nuove operazioni.
nextcloud-admin-authority = Autorità dell'istanza
nextcloud-admin-name = Nome visualizzato
nextcloud-admin-name-help = Un nome breve mostrato agli utenti quando scelgono un'istanza.
nextcloud-admin-url = URL di installazione HTTPS
nextcloud-admin-url-help = Usa la radice dell'installazione, come https://cloud.example.org/ o https://example.org/nextcloud/. Il certificato deve essere attendibile e corrispondere al nome host. Credenziali, parametri di query e frammenti vengono rifiutati.
nextcloud-admin-enable = Consenti agli utenti di collegare questa istanza
nextcloud-admin-ca = Autorità di certificazione aggiuntive (PEM)
nextcloud-admin-ca-help = Certificati CA privati facoltativi per questo server Nextcloud. L'attendibilità pubblica resta disponibile e la verifica del nome host resta obbligatoria. Le modifiche richiedono agli utenti di verificare di nuovo i loro collegamenti; l'attendibilità dei certificati del proxy è separata.
nextcloud-instance-select = Istanza Nextcloud
nextcloud-instance-placeholder = Scegli un'istanza collegata
nextcloud-instances-empty = Nessun collegamento Nextcloud verificato. Aggiungi o verifica un collegamento in Impostazioni utente → Nextcloud.
nextcloud-instances-error = Impossibile caricare le istanze collegate. Riapri il selettore per riprovare.
nextcloud-save-warning = L'amministratore richiede una conferma prima che questo allegato possa uscire dalla casella.
nextcloud-save-acknowledge = Ho compreso l'avviso sull'allegato e voglio salvare questo file nell'istanza Nextcloud selezionata.
nextcloud-save-attachment = Salva { $name } in Nextcloud
nextcloud-save-close = Chiudi la destinazione Nextcloud
nextcloud-save-options = Salva gli allegati in Nextcloud
nextcloud-pim-title = Contatti e calendari
nextcloud-pim-help = Individua le raccolte di questo account Nextcloud verificato, poi connetti esplicitamente quelle che desidera. Sincronizza per recuperare i dati attuali; le modifiche alle raccolte connesse usano la raccolta remota e falliscono se il collegamento o i permessi vengono revocati.
nextcloud-pim-category = Categoria della raccolta
nextcloud-pim-calendar = Calendari
nextcloud-pim-contacts = Rubriche
nextcloud-pim-tasks = Elenchi di attività
nextcloud-pim-discover = Individua raccolte
nextcloud-pim-discovered = Raccolte disponibili
nextcloud-pim-empty = Non è stata restituita nessuna raccolta corrispondente.
nextcloud-pim-connect = Connetti
nextcloud-pim-connected = Raccolte connesse
nextcloud-pim-sync = Sincronizza ora
nextcloud-pim-sync-needed = Raccolta connessa. Sincronizza ora per caricarne il contenuto.
nextcloud-pim-synced = Sincronizzazione completata.
nextcloud-pim-error = Operazione non riuscita. Controlla il collegamento dell'account e i permessi, poi riprova esplicitamente. I dati nella cache esistenti non sono stati scartati.
nextcloud-pim-native-required = La sincronizzazione di contatti e calendario richiede il motore di posta nativo. L'accesso ai file resta disponibile tramite il collegamento dell'account.
nextcloud-pim-disconnect = Disconnetti
nextcloud-pim-disconnect-help = Disconnettere mantiene i dati nella cache e interrompe le scritture remote. Gli account ricollegati richiedono di nuovo l'individuazione e la connessione esplicita.
nextcloud-pim-ready = Pronta
nextcloud-pim-disconnected = Disconnessa o collegamento modificato
nextcloud-pim-unnamed = Raccolta senza nome
nextcloud-pim-back = Torna agli account Nextcloud
nextcloud-pim-create-title = Nuova raccolta in Nextcloud
nextcloud-pim-create-help = Crea un calendario o un elenco di attività vuoto in questo account Nextcloud e lo connette qui.
nextcloud-pim-create-name = Nome della nuova raccolta
nextcloud-pim-create = Crea in Nextcloud
nextcloud-pim-created = { $name } è stata creata in Nextcloud e connessa. Sincronizza ora per caricarla.
nextcloud-pim-interval = Intervallo di aggiornamento automatico (secondi)
nextcloud-pim-interval-help = Zero usa solo l'aggiornamento manuale. Gli intervalli positivi hanno un minimo di 60 secondi. Gli account connessi si aggiornano in background; in caso di errori l'attesa aumenta e le credenziali rifiutate bloccano ulteriori richieste finché non ricolleghi.
nextcloud-pim-interval-reset = Usa il valore predefinito dell'amministratore
nextcloud-admin-empty = Nessuna istanza approvata per ora. Aggiungi un'istanza per renderla disponibile al collegamento degli account.
nextcloud-admin-status = Stato
nextcloud-admin-open = Apri { $name }
