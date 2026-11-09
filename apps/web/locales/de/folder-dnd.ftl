# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Ordner verschoben: { $name }
folder-dnd-moved-many = { $count } Ordner verschoben
folder-dnd-undo = Rückgängig
folder-dnd-cancel = Abbrechen
folder-dnd-handle = { $name } ziehen, um es zu verschieben
folder-dnd-locked-hint = Die Reihenfolge der Ordner wird von Ihrem Administrator festgelegt.
folder-dnd-forced-hint = Von Ihrem Administrator angeheftet. Er kann nicht verschoben oder gelöst werden.
folder-dnd-disabled-hint = Das Ziehen und Ablegen von Ordnern ist ausgeschaltet.
folder-dnd-system-hint = Systemordner können nicht verschoben werden.
folder-dnd-pinned-group = Angeheftet
folder-dnd-top-level = In die oberste Ebene verschieben
folder-dnd-problem-disabled = Das Verschieben von Ordnern ist ausgeschaltet.
folder-dnd-problem-locked = Die Reihenfolge der Ordner wird von Ihrem Administrator festgelegt.
folder-dnd-problem-self = Ein Ordner kann nicht in sich selbst verschoben werden.
folder-dnd-problem-descendant = Ein Ordner kann nicht in einen seiner eigenen Unterordner verschoben werden.
folder-dnd-problem-system = Systemordner können nicht hierher verschoben werden.
folder-dnd-problem-no-children = Dieser Ordner kann keine Unterordner enthalten.
folder-dnd-problem-nesting-off = Ihr Administrator erlaubt keine verschachtelten Ordner.
folder-dnd-problem-depth = Dadurch würden Ordner zu tief verschachtelt.
folder-dnd-problem-forced = Dieser Ordner wurde von Ihrem Administrator angeheftet.
folder-dnd-problem-unknown = Dieser Ordner existiert nicht mehr.
folder-dnd-problem-noop = Der Ordner befindet sich bereits dort.
folder-dnd-problem-collision = Dort gibt es bereits einen Ordner namens { $name }. Benennen Sie ihn zuerst um.
folder-dnd-problem-server = Der Server hat { $name } nicht verschoben. Es wurde nichts geändert.
folder-dnd-announce-up = { $name } nach oben verschoben
folder-dnd-announce-down = { $name } nach unten verschoben
folder-dnd-announce-indent = { $name } in den Ordner darüber verschoben
folder-dnd-announce-outdent = { $name } eine Ebene nach oben verschoben
folder-dnd-nest-title = { $name } hierher verschieben?
folder-dnd-nest-title-many = { $count } Ordner hierher verschieben?
folder-dnd-nest-body = Wählen Sie, ob dies den Ordner auf dem Mailserver ändert oder nur, wie diese App ihn anzeigt.
folder-dnd-nest-legend = Wie verschoben werden soll
folder-dnd-nest-local = Nur in dieser App anordnen
folder-dnd-nest-local-help = Nur eine Ansicht. Der Ordner bleibt auf dem Server und in anderen Mail-Apps an seiner Stelle.
folder-dnd-nest-server = Auf dem Server verschieben
folder-dnd-nest-server-help = Ändert den Ordner überall, in jeder Mail-App, die dieses Konto verwendet.
folder-dnd-nest-remember = Meine Auswahl merken
folder-dnd-nest-confirm = Verschieben
folder-dnd-arranged-only = Nur in dieser App angeordnet
folder-dnd-arranged-only-help = Seine Position auf dem Server ist eine andere: { $path }
folder-dnd-apply-server = Auf dem Server anwenden
folder-dnd-reset = Reihenfolge auf Standard zurücksetzen
folder-dnd-reset-done = Ordnerreihenfolge zurückgesetzt
folder-dnd-move-to = { $name } verschieben nach …
folder-dnd-layout-link = Ordnerlayout
folder-dnd-reorder-enable = Ordner per Ziehen neu anordnen
folder-dnd-nest-mode = Wenn ich einen Ordner verschachtele
folder-dnd-nest-mode-ask = Nachfragen
folder-dnd-nest-mode-server = Auf dem Server verschieben
folder-dnd-nest-mode-local = Nur in dieser App anordnen
folder-dnd-new-placement = Neue Ordner erscheinen
folder-dnd-new-placement-end = Nach den Ordnern derselben Ebene
folder-dnd-new-placement-alphabetical = In alphabetischer Reihenfolge
folder-dnd-handles = Ziehgriffe anzeigen
folder-dnd-handles-hover = Beim Darüberfahren
folder-dnd-handles-always = Immer
folder-dnd-arrange-title = Ordner anordnen
folder-dnd-arrange-help = Ziehen Sie einen Ordner, um ihn neu zu ordnen, legen Sie ihn auf einem anderen Ordner ab, um ihn zu verschachteln, oder verwenden Sie Alt und die Pfeiltasten bei einem fokussierten Ordner.
folder-dnd-admin-locked = Von Ihrem Administrator festgelegt
folder-dnd-arrange-menu = Verschieben nach …
folder-star-add = Ordner mit Stern markieren
folder-star-remove = Stern vom Ordner entfernen
folder-star-starred = Mit Stern
folder-star-toggle = { $name } mit Stern markieren
folder-star-only = Nur Ordner mit Stern anzeigen
folder-star-only-help = Zeigt nur Ordner mit Stern und die Ordner, die sie enthalten.
folder-dnd-indent = { $name } in den Ordner darüber verschieben
folder-dnd-outdent = { $name } eine Ebene nach oben verschieben
folder-dnd-more-actions = Anordnen
folder-dnd-selected-move = Auswahl verschieben nach …
folder-dnd-locked-row = Gesperrt
folder-dnd-problem-pin = Angeheftete Ordner bleiben über den anderen. Lösen Sie ihn zuerst, um ihn dorthin zu verschieben.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Leer…

folder-bar-label = Aktionen für die ausgewählten Ordner

folder-bar-mark-read = Alles als gelesen markieren

folder-bar-pin = Pin

folder-bar-star = Stern

folder-bar-unpin = Lösen

folder-bar-unstar = Unstar

folder-dnd-handle-hint = Ziehen Sie, um diesen Ordner zu verschieben

folder-pinned-group = Angepinnt

folder-pinned-unpin-zone = Zum Lösen hierher fallen lassen

settings-folders-bulk-delete-body = { $folders ->
        [one] { $folders } Ordner und die darin enthaltenen { $messages } Nachrichten werden gelöscht. Dies kann nicht rückgängig gemacht werden.
       *[other] { $folders }-Ordner (mit ihren Unterordnern) und die darin enthaltenen { $messages }-Nachrichten werden gelöscht. Dies kann nicht rückgängig gemacht werden.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [one] { $count } Ordner löschen
       *[other] { $count } Ordner löschen
    }

settings-folders-bulk-delete-done = { $count ->
        [one] { $count } Ordner gelöscht
       *[other] { $count } Ordner gelöscht
    }

settings-folders-bulk-delete-failed = Es wurde nichts gelöscht. Nicht gelöscht: { $names }

settings-folders-bulk-delete-list = Zu löschende Ordner

settings-folders-bulk-delete-none = Keiner der ausgewählten Ordner kann gelöscht werden (Systemordner und Ordner, die Sie nicht löschen dürfen, werden übersprungen).

settings-folders-bulk-delete-others = Auch nicht gelöscht: { $names }

settings-folders-bulk-delete-partial = Gelöschte { $done }-Ordner. Nicht gelöscht: { $names }

settings-folders-bulk-delete-partial-done = Gelöschte { $done }-Ordner.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [one] Nicht gelöscht, da das Konto einer gesetzlichen Sperre unterliegt: { $names }
       *[other] Nicht gelöscht, da das Konto einer rechtlichen Sperre unterliegt ({ $count } Ordner): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [one] Nicht gelöscht, da die Verarbeitung des Kontos eingeschränkt ist: { $names }
       *[other] Nicht gelöscht, da die Verarbeitung des Kontos eingeschränkt ist ({ $count } Ordner): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } ausgewählte Ordner werden übersprungen, da sie nicht gelöscht werden können: { $names }

settings-folders-bulk-delete-subfolders = Löschen Sie außerdem die Unterordner jedes Ordners

settings-folders-bulk-delete-subfolders-off = Nur die ausgewählten Ordner werden gelöscht. Ein Ordner, der außerhalb der Auswahl noch einen Unterordner hat, bleibt erhalten.

settings-folders-bulk-delete-title = { $count ->
        [one] { $count } Ordner löschen?
       *[other] { $count } Ordner löschen?
    }

settings-folders-bulk-empty-body = { $total } Nachrichten in { $folders } Ordnern werden entfernt: { $toTrash } werden in den Papierkorb verschoben und können rückgängig gemacht werden, { $permanent } werden endgültig gelöscht und können nicht wiederhergestellt werden. Unterordner werden nicht berührt.

settings-folders-bulk-empty-confirm = Leeren Sie die Ordner

settings-folders-bulk-empty-counting = Zählen der Nachrichten in jedem Ordner…

settings-folders-bulk-empty-done = { $count } Ordner, { $messages } Nachrichten geleert

settings-folders-bulk-empty-fail-refused = der Server lehnte einen Teil davon ab; Es bleiben { $remaining } Meldungen übrig

settings-folders-bulk-empty-fail-stopped = gestoppt; Es bleiben { $remaining } Meldungen übrig

settings-folders-bulk-empty-fail-uncertain = das Ergebnis der letzten Charge ist unbekannt; Überprüfen Sie den Ordner

settings-folders-bulk-empty-list = Ordner zum Leeren

settings-folders-bulk-empty-partial = { $done } Ordner geleert; { $failed } fehlgeschlagen. Siehe die Liste.

settings-folders-bulk-empty-permanent = { $count } Nachrichten werden endgültig gelöscht

settings-folders-bulk-empty-progress = { $done } von { $of } Ordnern erledigt, { $messages } Nachrichten verarbeitet. Jetzt: { $name }

settings-folders-bulk-empty-skip-count-failed = konnte nicht gezählt werden, übersprungen

settings-folders-bulk-empty-skip-empty = bereits leer, übersprungen

settings-folders-bulk-empty-skip-permanent-denied = würde endgültig gelöscht werden, was der Administrator deaktiviert hat, übersprungen

settings-folders-bulk-empty-skip-search = Eine gespeicherte Suche, übersprungen

settings-folders-bulk-empty-skip-unsupported = Dieses Konto kann es nicht leeren, übersprungen

settings-folders-bulk-empty-title = { $count ->
        [one] { $count } Ordner leeren?
       *[other] { $count } Ordner leeren?
    }

settings-folders-bulk-empty-to-trash = { $count } Nachrichten werden in den Papierkorb verschoben

settings-folders-bulk-empty-undo-label = Geleerte { $count } Ordner ({ $messages } Nachrichten)

settings-folders-delete-selected = Löschen…

settings-folders-select-user = Wählen Sie alle Benutzerordner aus

settings-folders-subscribe-failed = Die Abonnements konnten nicht geändert werden.

settings-folders-subscribe-partial = { $done }-Ordner geändert; { $failed } konnte nicht geändert werden.

settings-folders-subscribe-selected = Abonnieren

settings-folders-subscribed = { $count ->
        [one] { $count } Ordner abonniert
       *[other] { $count } Ordner abonniert
    }

settings-folders-unsubscribe-selected = Abmelden

settings-folders-unsubscribed = { $count ->
        [one] { $count } Ordner abgemeldet
       *[other] Von { $count } Ordnern abgemeldet
    }
