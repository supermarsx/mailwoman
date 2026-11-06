# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Svuota cartella…
empty-folder-title = Svuotare “{ $name }”?
empty-folder-body-trash = { $count ->
    [one] Il messaggio in { $name } verrà spostato nel Cestino, dove potrai ripristinarlo.
   *[other] Tutti i { $count } messaggi in { $name } verranno spostati nel Cestino, dove potrai ripristinarli.
 }
empty-folder-body-destroy-trash = { $count ->
    [one] Il messaggio nel Cestino verrà eliminato definitivamente. L'operazione non può essere annullata.
   *[other] Tutti i { $count } messaggi nel Cestino verranno eliminati definitivamente. L'operazione non può essere annullata.
 }
empty-folder-body-destroy-junk = { $count ->
    [one] Il messaggio in { $name } verrà eliminato definitivamente. L'operazione non può essere annullata.
   *[other] Tutti i { $count } messaggi in { $name } verranno eliminati definitivamente. L'operazione non può essere annullata.
 }
empty-folder-body-destroy-no-trash = Questo account non ha una cartella Cestino, quindi { $count ->
    [one] il messaggio in { $name } verrà eliminato definitivamente.
   *[other] tutti i { $count } messaggi in { $name } verranno eliminati definitivamente.
 }
empty-folder-confirm-trash = Sposta nel Cestino
empty-folder-confirm-destroy = Elimina definitivamente
empty-folder-cancel = Annulla
empty-folder-already-empty = “{ $name }” è già vuota.
empty-folder-prepare-failed = Impossibile contare i messaggi di “{ $name }”, quindi non è stato modificato nulla. Controlla la connessione e riprova.
empty-folder-syncing = “{ $name }” è ancora in sincronizzazione, quindi non può essere ancora contata. Non è stato modificato nulla; riprova tra un momento.
empty-folder-running = Una cartella è già in fase di svuotamento. Interrompila dalla sua notifica prima di avviarne un'altra.
empty-folder-progress-trash = Spostamento dei messaggi da { $name } al Cestino: { $done } su { $total }.
empty-folder-progress-destroy = Eliminazione dei messaggi da { $name }: { $done } su { $total }.
empty-folder-stop = Interrompi
empty-folder-stopping = Interruzione dopo il lotto corrente. Finora { $done } su { $total }.
empty-folder-done-trash = { $done ->
    [one] Spostato 1 messaggio da { $name } al Cestino.
   *[other] Spostati { $done } messaggi da { $name } al Cestino.
 }
empty-folder-done-destroy = { $done ->
    [one] Eliminato definitivamente 1 messaggio da { $name }.
   *[other] Eliminati definitivamente { $done } messaggi da { $name }.
 }
empty-folder-stopped = Interrotto. { $done } messaggi su { $total } completati; { $remaining } ancora in { $name }.
empty-folder-refused = Interrotto. { $done } messaggi su { $total } completati; il server di posta ne ha rifiutati { $refused }. { $remaining } ancora in { $name }.
empty-folder-failed = Lo svuotamento di { $name } si è interrotto con un errore dopo { $done } messaggi. L'ultimo lotto potrebbe essere stato completato oppure no; controlla la cartella prima di riprovare.
