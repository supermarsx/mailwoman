# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Contatti
contacts-search = Cerca contatti
contacts-new = Nuovo contatto
contacts-find-duplicates = Trova duplicati
contacts-loading = Caricamento…
contacts-none = Nessun contatto.
contacts-list = Elenco contatti
contacts-favorite = Aggiungi { $name } ai preferiti
contacts-export-vcard = Esporta vCard
contacts-export-csv = Esporta CSV
contacts-select-hint = Selezionare un contatto per vederne la scheda.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Rubriche e gruppi
contacts-address-books = Rubriche
contacts-all = Tutti i contatti
contacts-favorites = Preferiti
contacts-groups = Gruppi
contacts-new-group = Nuovo gruppo
contacts-new-group-name = Nome del nuovo gruppo
contacts-group-name-placeholder = Nome del gruppo
contacts-create = Crea
contacts-data = Dati
contacts-import = Importa…

# -- Business card -----------------------------------------------------------
contacts-card = Contatto { $name }
contacts-favorited = Nei preferiti
contacts-favorite-action = Preferito
contacts-email = Email
contacts-phone = Telefono
contacts-dates = Date
contacts-groups-label = Gruppi
contacts-notes = Note
contacts-security-key = Chiave di sicurezza
contacts-key-on-file = È presente una chiave/un certificato (solo visualizzazione finché non arrivano PGP/S-MIME).
contacts-no-key = Nessuna chiave presente.
contacts-directory-security = Sicurezza della directory
contacts-member-of = Membro di
contacts-group-membership = Appartenenza a { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Nuovo contatto
contacts-edit-contact = Modifica contatto
contacts-full-name = Nome completo
contacts-given = Nome
contacts-given-placeholder = Nome
contacts-surname = Cognome
contacts-surname-placeholder = Cognome
contacts-organization = Organizzazione
contacts-job-title = Qualifica
contacts-job-title-placeholder = Qualifica
contacts-email-n = Email { $n }
contacts-email-label = Etichetta email { $n }
contacts-email-label-placeholder = lavoro
contacts-remove-email = Rimuovi email { $n }
contacts-add-email = Aggiungi email
contacts-phone-n = Telefono { $n }
contacts-remove-phone = Rimuovi telefono { $n }
contacts-add-phone = Aggiungi telefono
contacts-security-key-label = Chiave di sicurezza (segnaposto opaco)
contacts-security-key-placeholder = Chiave PGP / certificato S-MIME (solo visualizzazione)
contacts-favorite-word = Preferito

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Importa contatti
contacts-imported = { $count ->
    [one] { $count } contatto importato
   *[other] { $count } contatti importati
}.
contacts-import-hint = Incollare vCard (.vcf) o CSV, oppure scegliere un file. Il formato viene rilevato automaticamente.
contacts-import-file = Importa file
contacts-paste = Incolla vCard o CSV
contacts-preview = Anteprima
contacts-map-columns = Associa colonne
contacts-col = Colonna
contacts-maps-to = Corrisponde a
contacts-sample = Esempio
contacts-map-column = Associa colonna { $header }
contacts-preview-count = Anteprima ({ $count })
contacts-import-preview = Anteprima dell'importazione
contacts-no-name = (nessun nome)
contacts-no-email = nessuna email
contacts-import-n = Importa { $count ->
    [one] { $count } contatto
   *[other] { $count } contatti
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Unisci duplicati
contacts-merge-review-hint = Controllare la scheda unita. Le altre schede vengono contrassegnate come eliminate (reversibile).
contacts-merged-preview = Anteprima dell'unione
contacts-emails = Email
contacts-phones = Telefoni
contacts-merge-confirm = Unisci contatti
contacts-no-duplicates = Nessun duplicato trovato.
contacts-possible-duplicates = { $count } possibili duplicati
contacts-review-merge = Controlla unione

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Dettagli del contatto
contacts-address-recipients = Mostra destinatari
contacts-address-save = Salva contatto
contacts-address-compose = Invia email
contacts-address-unsaved = Questo indirizzo non è salvato nei suoi contatti.
contacts-address-book = Rubrica
contacts-address-lookup-failed = Impossibile caricare le informazioni del contatto. Riprova.
contacts-address-save-failed = Impossibile salvare il contatto. Riprova.
contacts-address-saved = Contatto salvato.
contacts-address-loading = Caricamento del contatto…
contacts-editor-person = Nome e lavoro
contacts-editor-reach = Email e telefono
contacts-editor-other = Note e preferenze
contacts-editor-save-error = Impossibile salvare il contatto. Le modifiche sono ancora qui. Riprova.
contacts-group-editor-help = Crea un gruppo in una rubrica, poi aggiungi i membri dalle loro schede contatto.
contacts-group-details = Dettagli del gruppo
contacts-group-save-error = Impossibile salvare il gruppo. Le modifiche sono ancora qui. Riprova.
contacts-edit-group = Modifica gruppo di contatti
contacts-edit-named-group = Modifica gruppo { $name }
contacts-group-name = Nome del gruppo
contacts-group-edit-help = Aggiorna il nome di questo gruppo. La rubrica e i membri restano invariati.
contacts-back-to-list = Torna ai contatti
contacts-membership-title = Appartenenza al gruppo
contacts-membership-help = Scegli un gruppo, poi aggiungi o rimuovi i suoi membri. Le modifiche vengono salvate subito.
contacts-membership-no-groups = Nessun gruppo ancora. Creane uno per iniziare ad aggiungere membri.
contacts-membership-pick = Seleziona un gruppo per vederne i membri.
contacts-membership-of = Membri di { $name }
contacts-membership-members = Membri
contacts-membership-add = Digita un nome o un indirizzo per aggiungere un membro
contacts-membership-add-group = Digita il nome di un gruppo per aggiungere questo contatto
contacts-membership-count = { $count ->
    [one] 1 membro
   *[other] { $count } membri
}
contacts-membership-failed = Alcune modifiche all'appartenenza non sono state salvate. L'elenco mostra ciò che è memorizzato.
contacts-membership-manage = Gestisci appartenenza al gruppo
contacts-import-source = Origine
contacts-import-book = Rubrica
contacts-import-review = Revisione
contacts-import-empty = Scegli un file o incolla testo vCard o CSV, poi seleziona Anteprima. I contatti trovati sono elencati qui e può escluderne qualcuno.
contacts-merge-sets = Possibili duplicati
contacts-merge-set = Duplicati di { $name }
contacts-merge-keep = Scheda da mantenere
contacts-merge-done = Unito in { $name }.
contacts-address-actions = Azioni del contatto
contacts-address-save-to = Salva nei contatti
contacts-address-goto = Vai al contatto
contacts-address-copy = Copia informazioni del contatto
contacts-address-copied = Informazioni del contatto copiate.
contacts-address-copy-failed = Impossibile copiare le informazioni del contatto.
contacts-share = Condividi contatto
contacts-share-email = Invia contatto per email
contacts-share-copy = Copia come vCard
contacts-share-download = Scarica .vcf
contacts-share-native = Condividi…
contacts-share-copied = vCard copiata.
contacts-share-copy-failed = Impossibile copiare la vCard.
contacts-share-downloaded = vCard scaricata.
contacts-share-failed = Impossibile condividere il contatto.
contacts-share-attach-failed = Impossibile allegare il file .vcf, quindi la vCard è nel testo del messaggio.
contacts-address-saved-favorite = Contatto salvato e aggiunto ai preferiti.
contacts-favorite-add = Aggiungi ai preferiti
contacts-favorite-remove = Rimuovi dai preferiti
contacts-favorite-save = Salva e aggiungi ai preferiti
contacts-favorite-added = { $name } aggiunto ai preferiti.
contacts-favorite-removed = { $name } rimosso dai preferiti.
contacts-favorite-failed = Impossibile aggiornare i preferiti.
contacts-select-row = Seleziona { $name }
contacts-selected-count = { $count } selezionati
contacts-select-clear = Cancella selezione
contacts-bulk-actions = Azioni per i contatti selezionati
contacts-bulk-favorites-added = { $count } aggiunti ai preferiti.
contacts-bulk-favorites-removed = { $count } rimossi dai preferiti.
contacts-row-menu = Azioni per { $name }
contacts-favorites-empty = Nessun preferito ancora. Usa la stella su un contatto per aggiungerne uno.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } è già nei tuoi contatti.

contacts-save-already-badge = Già salvato

contacts-save-already-many = Tutti i contatti { $count } sono già salvati.

contacts-save-automated = Sembra un mittente automatico (nessuna risposta, notifiche).

contacts-save-book = Rubrica

contacts-save-book-readonly = { $name } (sola lettura)

contacts-save-cancel = Annulla

contacts-save-choose = Scegli i contatti...

contacts-save-contacts = Salva i contatti da questo messaggio

contacts-save-dialog-title = Salva i contatti da questo messaggio

contacts-save-email = E-mail

contacts-save-failed = Impossibile salvare il contatto. Riprova.

contacts-save-group = Gruppo

contacts-save-group-none = Nessun gruppo

contacts-save-loading = Caricamento indirizzi…

contacts-save-name = Nome

contacts-save-new = Salva nuovi contatti

contacts-save-no-book = Nessuna rubrica può accettare nuovi contatti.

contacts-save-no-sender = Questo messaggio non ha indirizzo del mittente.

contacts-save-note = Salvato da un messaggio su { $date }

contacts-save-nothing = Nessuno da salvare: tutti sono già nei tuoi contatti, sei tu o sembrano automatizzati.

contacts-save-open = Aperto

contacts-save-open-contact = Contatto aperto

contacts-save-open-link = Contatto aperto

contacts-save-own = Questo è uno dei tuoi indirizzi.

contacts-save-partial = Salvati { $saved } di { $total } contatti. Gli altri non potevano essere salvati.

contacts-save-pick = Risparmia { $email }

contacts-save-recipients = Salva tutti i destinatari

contacts-save-removed = Contatti salvati rimossi.

contacts-save-role-bcc = Ccn

contacts-save-role-cc = CC

contacts-save-role-from = Da

contacts-save-role-replyTo = Rispondi a

contacts-save-role-sender = Mittente

contacts-save-role-to = A

contacts-save-saved-heading = Già nei tuoi contatti

contacts-save-saved-many = Contatti { $count } salvati in { $book }

contacts-save-saved-one = Salvato da { $name } a { $book }

contacts-save-sender = Salva mittente come contatto

contacts-save-settings-book = Rubrica

contacts-save-settings-book-default = Rubrica predefinita

contacts-save-settings-enabled = Offerta Salva i contatti da questo messaggio

contacts-save-settings-enabled-help = Aggiunge l'elemento ai menu dei messaggi e al menu Altro del lettore.

contacts-save-settings-note = Aggiungi una nota con la data

contacts-save-settings-note-help = La nota dice "Salvato da un messaggio il" seguito dalla data.

contacts-save-settings-reply-to = Utilizza Rispondi a per elenchi e mittenti di notifiche

contacts-save-settings-reply-to-help = Salva l'indirizzo per le risposte anziché il mittente per tali mittenti.

contacts-save-settings-scope = Selezionato per primo nella finestra di dialogo

contacts-save-settings-scope-all = Tutti sul messaggio

contacts-save-settings-scope-sender = Solo mittente

contacts-save-settings-sender = Offerta Salva mittente come contatto

contacts-save-settings-sender-help = Un elemento con un clic nei menu dei messaggi.

contacts-save-settings-skip = Salta i mittenti automatici

contacts-save-settings-skip-help = Tralascia gli indirizzi di mancata risposta, di notifica e di mancato recapito a meno che non li selezioni.

contacts-save-settings-title = Salva i contatti dai messaggi

contacts-save-show-automated = Mostra indirizzi automatizzati e senza risposta

contacts-save-submit = { $count ->
        [one] Salva { $count } contatto
        [many] Salva { $count } contatti
       *[other] Salva { $count } contatti
    }

contacts-save-undo = Annulla
