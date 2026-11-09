# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Cartella spostata: { $name }
folder-dnd-moved-many = { $count } cartelle spostate
folder-dnd-undo = Annulla
folder-dnd-cancel = Annulla
folder-dnd-handle = Trascinare { $name } per spostarla
folder-dnd-locked-hint = L'ordine delle cartelle è fissato dall'amministratore.
folder-dnd-forced-hint = Fissata dall'amministratore. Non può essere spostata né sbloccata.
folder-dnd-disabled-hint = Il trascinamento delle cartelle è disattivato.
folder-dnd-system-hint = Le cartelle di sistema non possono essere spostate.
folder-dnd-pinned-group = Fissate
folder-dnd-top-level = Sposta al livello superiore
folder-dnd-problem-disabled = Lo spostamento delle cartelle è disattivato.
folder-dnd-problem-locked = L'ordine delle cartelle è fissato dall'amministratore.
folder-dnd-problem-self = Una cartella non può essere spostata in se stessa.
folder-dnd-problem-descendant = Una cartella non può essere spostata in una delle proprie sottocartelle.
folder-dnd-problem-system = Le cartelle di sistema non possono essere spostate qui.
folder-dnd-problem-no-children = Quella cartella non può contenere sottocartelle.
folder-dnd-problem-nesting-off = L'amministratore non consente di annidare le cartelle.
folder-dnd-problem-depth = Questo anniderebbe le cartelle troppo in profondità.
folder-dnd-problem-forced = Questa cartella è fissata dall'amministratore.
folder-dnd-problem-unknown = Quella cartella non esiste più.
folder-dnd-problem-noop = La cartella è già lì.
folder-dnd-problem-collision = Lì esiste già una cartella denominata { $name }. Rinominarla prima.
folder-dnd-problem-server = Il server non ha spostato { $name }. Non è stato modificato nulla.
folder-dnd-announce-up = { $name } spostata in su
folder-dnd-announce-down = { $name } spostata in giù
folder-dnd-announce-indent = { $name } spostata nella cartella sopra
folder-dnd-announce-outdent = { $name } spostata di un livello in su
folder-dnd-nest-title = Spostare { $name } qui?
folder-dnd-nest-title-many = Spostare { $count } cartelle qui?
folder-dnd-nest-body = Scegliere se questo modifica la cartella sul server di posta o solo il modo in cui questa app la mostra.
folder-dnd-nest-legend = Come spostarla
folder-dnd-nest-local = Disponi solo in questa app
folder-dnd-nest-local-help = Solo una vista. La cartella resta dov'è sul server e nelle altre app di posta.
folder-dnd-nest-server = Sposta sul server
folder-dnd-nest-server-help = Modifica la cartella ovunque, in ogni app di posta che usa questo account.
folder-dnd-nest-remember = Ricorda la scelta
folder-dnd-nest-confirm = Sposta
folder-dnd-arranged-only = Disposta solo in questa app
folder-dnd-arranged-only-help = La sua posizione sul server è diversa: { $path }
folder-dnd-apply-server = Applica sul server
folder-dnd-reset = Ripristina l'ordine predefinito
folder-dnd-reset-done = Ordine delle cartelle ripristinato
folder-dnd-move-to = Sposta { $name } in...
folder-dnd-layout-link = Layout delle cartelle
folder-dnd-reorder-enable = Riordina le cartelle trascinandole
folder-dnd-nest-mode = Quando annido una cartella
folder-dnd-nest-mode-ask = Chiedi
folder-dnd-nest-mode-server = Sposta sul server
folder-dnd-nest-mode-local = Disponi solo in questa app
folder-dnd-new-placement = Le nuove cartelle compaiono
folder-dnd-new-placement-end = Dopo le cartelle dello stesso livello
folder-dnd-new-placement-alphabetical = In ordine alfabetico
folder-dnd-handles = Mostra le maniglie di trascinamento
folder-dnd-handles-hover = Al passaggio del mouse
folder-dnd-handles-always = Sempre
folder-dnd-arrange-title = Disponi le cartelle
folder-dnd-arrange-help = Trascinare una cartella per riordinarla, rilasciarla su un'altra cartella per annidarla, oppure usare Alt e i tasti freccia su una cartella con il focus.
folder-dnd-admin-locked = Impostato dall'amministratore
folder-dnd-arrange-menu = Sposta in…
folder-star-add = Contrassegna la cartella con una stella
folder-star-remove = Rimuovi la stella dalla cartella
folder-star-starred = Con stella
folder-star-toggle = Contrassegna { $name } con una stella
folder-star-only = Mostra solo quelle con stella
folder-star-only-help = Mostra solo le cartelle con stella e le cartelle che le contengono.
folder-dnd-indent = Sposta { $name } nella cartella sopra
folder-dnd-outdent = Sposta { $name } di un livello in su
folder-dnd-more-actions = Disponi
folder-dnd-selected-move = Sposta le selezionate in…
folder-dnd-locked-row = Bloccata
folder-dnd-problem-pin = Le cartelle fissate restano sopra le altre. Sbloccarla prima per spostarla lì.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Vuoto…

folder-bar-label = Azioni per le cartelle selezionate

folder-bar-mark-read = Segna tutto letto

folder-bar-pin = Perno

folder-bar-star = Stella

folder-bar-unpin = Sblocca

folder-bar-unstar = Elimina stella

folder-dnd-handle-hint = Trascina per spostare questa cartella

folder-pinned-group = Appuntato

folder-pinned-unpin-zone = Rilascia qui per sbloccare

settings-folders-bulk-delete-body = { $folders ->
        [one] { $folders } cartella e i messaggi { $messages } in essa contenuti verranno eliminati. Questa operazione non può essere annullata.
        [many] { $folders } cartelle (con le relative sottocartelle) e i messaggi { $messages } in esse contenuti verranno eliminati. Questa operazione non può essere annullata.
       *[other] { $folders } cartelle (con le relative sottocartelle) e i messaggi { $messages } in esse contenuti verranno eliminati. Questa operazione non può essere annullata.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [one] Elimina { $count } cartella
        [many] Elimina { $count } cartelle
       *[other] Elimina { $count } cartelle
    }

settings-folders-bulk-delete-done = { $count ->
        [one] Eliminata { $count } cartella
        [many] Eliminate { $count } cartelle
       *[other] Eliminate { $count } cartelle
    }

settings-folders-bulk-delete-failed = Niente è stato cancellato. Non cancellato: { $names }

settings-folders-bulk-delete-list = Cartelle da eliminare

settings-folders-bulk-delete-none = Nessuna delle cartelle selezionate può essere eliminata (le cartelle di sistema e le cartelle che non è possibile eliminare vengono saltate).

settings-folders-bulk-delete-others = Inoltre non cancellato: { $names }

settings-folders-bulk-delete-partial = Cartelle { $done } eliminate. Non cancellato: { $names }

settings-folders-bulk-delete-partial-done = Cartelle { $done } eliminate.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [one] Non eliminato perché l'account è bloccato a fini giudiziari: { $names }
        [many] Non eliminato perché l'account è soggetto a blocco legale ({ $count } cartelle): { $names }
       *[other] Non eliminato perché l'account è soggetto a blocco legale ({ $count } cartelle): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [one] Non cancellato perché l'elaborazione dell'account è limitata: { $names }
        [many] Non cancellato perché l'elaborazione dell'account è limitata ({ $count } cartelle): { $names }
       *[other] Non cancellato perché l'elaborazione dell'account è limitata ({ $count } cartelle): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } le cartelle selezionate vengono saltate perché non possono essere cancellate: { $names }

settings-folders-bulk-delete-subfolders = Elimina anche le sottocartelle di ciascuna cartella

settings-folders-bulk-delete-subfolders-off = Vengono eliminate solo le cartelle selezionate. Viene mantenuta una cartella che contiene ancora una sottocartella esterna alla selezione.

settings-folders-bulk-delete-title = { $count ->
        [one] Eliminare { $count } cartella?
        [many] Eliminare { $count } cartelle?
       *[other] Eliminare { $count } cartelle?
    }

settings-folders-bulk-empty-body = I messaggi { $total } nelle cartelle { $folders } verranno rimossi: { $toTrash } verranno spostati nel Cestino e potranno essere annullati, { $permanent } verranno eliminati definitivamente e non potranno essere ripristinati. Le sottocartelle non vengono toccate.

settings-folders-bulk-empty-confirm = Svuota le cartelle

settings-folders-bulk-empty-counting = Conteggio dei messaggi in ciascuna cartella...

settings-folders-bulk-empty-done = Cartelle { $count } svuotate, messaggi { $messages }

settings-folders-bulk-empty-fail-refused = il server ne ha rifiutato una parte; Rimangono { $remaining } messaggi

settings-folders-bulk-empty-fail-stopped = fermato; Rimangono { $remaining } messaggi

settings-folders-bulk-empty-fail-uncertain = il risultato dell'ultimo lotto è sconosciuto; controlla la cartella

settings-folders-bulk-empty-list = Cartelle da svuotare

settings-folders-bulk-empty-partial = Cartelle { $done } svuotate; { $failed } fallito. Vedi l'elenco.

settings-folders-bulk-empty-permanent = { $count } i messaggi vengono cancellati definitivamente

settings-folders-bulk-empty-progress = { $done } di { $of } cartelle completate, { $messages } messaggi elaborati. Adesso: { $name }

settings-folders-bulk-empty-skip-count-failed = non poteva essere contato, saltato

settings-folders-bulk-empty-skip-empty = già vuoto, saltato

settings-folders-bulk-empty-skip-permanent-denied = verrebbe eliminato definitivamente, cosa che l'amministratore ha disattivato, saltato

settings-folders-bulk-empty-skip-search = una ricerca salvata, saltata

settings-folders-bulk-empty-skip-unsupported = questo account non può svuotarlo, saltato

settings-folders-bulk-empty-title = { $count ->
        [one] Svuotare { $count } cartella?
        [many] Svuotare { $count } cartelle?
       *[other] Svuotare { $count } cartelle?
    }

settings-folders-bulk-empty-to-trash = { $count } i messaggi vengono spostati nel Cestino

settings-folders-bulk-empty-undo-label = Cartelle { $count } svuotate (messaggi { $messages })

settings-folders-delete-selected = Elimina…

settings-folders-select-user = Seleziona tutte le cartelle utente

settings-folders-subscribe-failed = Non è stato possibile modificare gli abbonamenti.

settings-folders-subscribe-partial = Cartelle { $done } modificate; { $failed } non può essere modificato.

settings-folders-subscribe-selected = Iscriviti

settings-folders-subscribed = { $count ->
        [one] Iscritto a { $count } cartella
        [many] Sottoscritto a { $count } cartelle
       *[other] Sottoscritto a { $count } cartelle
    }

settings-folders-unsubscribe-selected = Annulla l'iscrizione

settings-folders-unsubscribed = { $count ->
        [one] Iscrizione annullata da { $count } cartella
        [many] Annullata l'iscrizione da { $count } cartelle
       *[other] Annullata l'iscrizione da { $count } cartelle
    }
