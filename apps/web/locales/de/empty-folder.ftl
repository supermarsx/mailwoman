# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Ordner leeren …
empty-folder-title = „{ $name }“ leeren?
empty-folder-body-trash = { $count ->
    [one] Die Nachricht in { $name } wird in den Papierkorb verschoben, wo Sie sie wiederherstellen können.
   *[other] Alle { $count } Nachrichten in { $name } werden in den Papierkorb verschoben, wo Sie sie wiederherstellen können.
    }
empty-folder-body-destroy-trash = { $count ->
    [one] Die Nachricht im Papierkorb wird endgültig gelöscht. Dies kann nicht rückgängig gemacht werden.
   *[other] Alle { $count } Nachrichten im Papierkorb werden endgültig gelöscht. Dies kann nicht rückgängig gemacht werden.
    }
empty-folder-body-destroy-junk = { $count ->
    [one] Die Nachricht in { $name } wird endgültig gelöscht. Dies kann nicht rückgängig gemacht werden.
   *[other] Alle { $count } Nachrichten in { $name } werden endgültig gelöscht. Dies kann nicht rückgängig gemacht werden.
    }
empty-folder-body-destroy-no-trash = Dieses Konto hat keinen Papierkorb, daher { $count ->
    [one] wird die Nachricht in { $name } endgültig gelöscht.
   *[other] werden alle { $count } Nachrichten in { $name } endgültig gelöscht.
    }
empty-folder-confirm-trash = In den Papierkorb verschieben
empty-folder-confirm-destroy = Endgültig löschen
empty-folder-cancel = Abbrechen
empty-folder-already-empty = „{ $name }“ ist bereits leer.
empty-folder-prepare-failed = „{ $name }“ konnte nicht gezählt werden, daher wurde nichts geändert. Prüfen Sie Ihre Verbindung und versuchen Sie es erneut.
empty-folder-syncing = „{ $name }“ wird noch synchronisiert und kann daher noch nicht gezählt werden. Es wurde nichts geändert; versuchen Sie es gleich noch einmal.
empty-folder-running = Es wird bereits ein Ordner geleert. Stoppen Sie dies über die Benachrichtigung, bevor Sie einen weiteren starten.
empty-folder-progress-trash = Nachrichten aus { $name } werden in den Papierkorb verschoben: { $done } von { $total }.
empty-folder-progress-destroy = Nachrichten aus { $name } werden gelöscht: { $done } von { $total }.
empty-folder-stop = Stoppen
empty-folder-stopping = Wird nach dem aktuellen Stapel gestoppt. Bisher { $done } von { $total } erledigt.
empty-folder-done-trash = { $done ->
    [one] 1 Nachricht aus { $name } in den Papierkorb verschoben.
   *[other] { $done } Nachrichten aus { $name } in den Papierkorb verschoben.
    }
empty-folder-done-destroy = { $done ->
    [one] 1 Nachricht aus { $name } endgültig gelöscht.
   *[other] { $done } Nachrichten aus { $name } endgültig gelöscht.
    }
empty-folder-stopped = Gestoppt. { $done } von { $total } Nachrichten erledigt; { $remaining } noch in { $name }.
empty-folder-refused = Gestoppt. { $done } von { $total } Nachrichten erledigt; der Mailserver hat { $refused } abgelehnt. { $remaining } noch in { $name }.
empty-folder-failed = Das Leeren von { $name } wurde nach { $done } Nachrichten mit einem Fehler beendet. Der letzte Stapel wurde möglicherweise abgeschlossen, möglicherweise nicht; prüfen Sie den Ordner, bevor Sie es erneut versuchen.
