# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Abbrechen
common-close = Schließen
common-save = Speichern
common-delete = Löschen
common-remove = Entfernen
common-edit = Bearbeiten
common-add = Hinzufügen
common-back = Zurück
common-next = Weiter
common-done = Fertig
common-apply = Anwenden
common-confirm = Bestätigen
common-retry = Erneut versuchen
common-contact-administrator = Wenn dies weiterhin auftritt, wenden Sie sich an Ihren Administrator.
common-shell-not-configured = Diese App wurde nicht für einen Server eingerichtet. Wenden Sie sich an Ihren Administrator.
common-copy = Kopieren
common-search = Suchen
common-more = Mehr
common-yes = Ja
common-no = Nein

# -- Generic states ----------------------------------------------------------
common-loading = Wird geladen…
common-saving = Wird gespeichert…
common-empty = Noch nichts vorhanden
common-error = Etwas ist schiefgelaufen
common-error-network = Der Server ist nicht erreichbar. Prüfen Sie Ihre Verbindung und versuchen Sie es erneut.
common-offline = Sie sind offline

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Verbindung wird hergestellt…
common-conn-degraded = Echtzeit-Aktualisierungen pausiert — Verbindung wird wiederhergestellt
common-conn-offline = Sie sind offline — Änderungen werden synchronisiert, sobald Sie wieder verbunden sind
common-conn-auth-expired = Ihre Sitzung ist abgelaufen — bitte melden Sie sich erneut an
common-conn-reconnect = Neu verbinden
common-conn-sign-in-again = Erneut anmelden
common-conn-reconnected = Wieder verbunden

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Neue Nachricht
common-notify-no-subject = (kein Betreff)

# -- Generic labels ----------------------------------------------------------
common-required = Erforderlich
common-optional = Optional

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Ordner und Apps öffnen
common-nav-close = Ordner und Apps schließen

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Anhänge
common-attach-search = Anhänge durchsuchen
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Nach Typ filtern
common-attach-loading = Anhänge werden geladen…
common-attach-empty = Keine passenden Anhänge.
common-attach-cat-all = alle
common-attach-cat-image = Bild
common-attach-cat-pdf = PDF
common-attach-cat-text = Text
common-attach-cat-audio = Audio
common-attach-cat-video = Video
common-attach-cat-other = Sonstige

# -- Password fields (components/PasswordInput.tsx): the button at the end of
common-password-show = Passwort anzeigen
common-password-hide = Passwort verbergen

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
common-module-mail = E-Mail
common-module-calendar = Kalender
common-module-tasks = Aufgaben
common-module-notes = Notizen
common-module-contacts = Kontakte
common-module-keys = Schlüssel

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
common-viewer-loading = Anzeige wird geladen…
common-viewer-loading-audio = Audio wird geladen…
common-viewer-loading-image = Bild wird geladen…
common-viewer-loading-video = Video wird geladen…
common-viewer-rendering-pdf = PDF wird gerendert…
common-viewer-pdf-failed = Dieses PDF konnte nicht dargestellt werden.
common-viewer-no-preview = Für diesen Dateityp gibt es keine Vorschau.
common-viewer-unknown-type = unbekannter Typ
common-viewer-download = Herunterladen
common-viewer-attachments = Anhänge

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Neuer Versuch …
common-waiting-online = Warten auf eine Verbindung …
common-failure-offline = Sie sind offline. Stellen Sie die Verbindung wieder her und versuchen Sie es erneut.
common-failure-network = Der Server war nicht erreichbar.
common-failure-server = Der Server hat nicht rechtzeitig geantwortet.
common-failure-rate = Zu viele Anfragen in kurzer Zeit. Warten Sie einen Moment und versuchen Sie es erneut.
common-failure-conflict = Dies wurde gleichzeitig an anderer Stelle geändert. Laden Sie neu und versuchen Sie es erneut.
common-failure-account = Das angemeldete Konto hat sich geändert. Laden Sie die Seite neu.
common-failure-linked = Dies ist nicht verfügbar, solange ein verknüpftes Postfach geöffnet ist.
common-failure-refused = Der Server hat die Änderung abgelehnt.
common-new-version = Eine neue Version von Mailwoman ist verfügbar. Wenn Sie dies schließen, erinnern wir Sie in einer Stunde erneut.
common-new-version-reload = Jetzt neu laden
common-toast-close = Benachrichtigung schließen
common-toast-seconds = { $seconds } s
common-attach-refresh = Anhänge aktualisieren
common-attach-open-message = Die Nachricht mit diesem Anhang öffnen
common-attach-page-filter-note = Filter gelten für diese Seite. Mit „Weiter“ blättern Sie zu älteren Nachrichten mit Anhängen.
common-attach-pages = Anhangsseiten
common-attach-previous = Zurück
common-attach-next = Weiter
common-attach-page = Seite { $page }
common-attach-column-name = Datei
common-attach-column-type = Typ
common-attach-column-size = Größe
common-attach-column-sender = Absender
common-attach-column-subject = Betreff
common-attach-column-date = Empfangen
common-pdf-preview = PDF-Vorschau
common-pdf-link-title = Link aus diesem PDF öffnen?
common-pdf-link-help = Das Ziel wird in einem neuen Tab außerhalb von Mailwoman geöffnet.
common-pdf-link-open = Link öffnen
common-open-event = Termin öffnen
common-message-preview-too-large = Vorschauen von Nachrichtenanhängen sind auf 4 MiB begrenzt. Den ursprünglichen Anhang können Sie weiterhin herunterladen.
common-message-preview-failed = Dieser Nachrichtenanhang konnte nicht in der Vorschau angezeigt werden. Die Datei wird möglicherweise nicht unterstützt oder ist beschädigt, oder der Vorschau-Dienst ist nicht verfügbar. Das Original können Sie weiterhin herunterladen.
common-message-preview-attachments = Dateien in dieser Nachricht
common-message-preview-download = Datei herunterladen
common-message-preview-size-bytes = { $count } Bytes
common-attach-cat-message = Nachrichten / Outlook-Anhänge
common-error-retry-help = Dieser Bereich ist vorübergehend nicht verfügbar. Versuchen Sie es erneut, um ihn neu zu laden.
common-error-network-help = Prüfen Sie Ihre Verbindung und versuchen Sie es erneut. Der Server ist möglicherweise vorübergehend nicht verfügbar.
common-error-page = Diese Seite konnte nicht geladen werden
common-error-page-help = Eine Ressource der Seite konnte nicht heruntergeladen werden. Versuchen Sie es erneut. Besteht das Problem weiter, laden Sie die Anwendung neu, um die neueste Version zu erhalten.
common-reload-app = Anwendung neu laden
common-new-version-reloading = Eine neue Version ist verfügbar, Seite wird neu geladen …
common-attach-account-changed = Das Konto hat sich beim Laden der Anhänge geändert. Öffnen Sie die Anhänge für das aktuelle Konto erneut.
common-attach-search-account = Gesamtes Konto durchsuchen
common-attach-scanning = Suche läuft … { $pages } Nachrichtenseiten geprüft · { $count } Dateien gefunden
common-attach-scan-cancelled = Suche abgebrochen · { $count } Dateien auf { $pages } Seiten gefunden. Die Ergebnisse sind unvollständig.
common-attach-scan-incomplete = Suche unterbrochen · { $count } Dateien gefunden. Wiederholen Sie die Suche, um das gesamte Konto zu durchsuchen.
common-attach-scan-limited = Es werden die ersten 1.000 passenden Dateien angezeigt. Grenzen Sie die Filter ein, um gezielter zu suchen.
common-attach-scan-complete = Suche abgeschlossen · { $count } Dateien auf { $pages } Nachrichtenseiten gefunden
common-attach-browse-pages = Seiten durchblättern
common-attach-cancel-search = Suche abbrechen
multiselect-selected = Ausgewählt: { $label }
multiselect-remove = { $label } entfernen
multiselect-no-match = Keine passenden Optionen
common-notification-permission-title = Benachrichtigungen erlauben?
common-notification-permission-help = Lassen Sie sich auf diesem Gerät über neue Nachrichten benachrichtigen. Sie können dies in den Einstellungen ändern.
common-notification-permission-allow = Benachrichtigungen erlauben
common-notification-permission-later = Jetzt nicht
common-notification-permission-error = Benachrichtigungen konnten nicht aktiviert werden. Prüfen Sie die Berechtigungen und Benachrichtigungseinstellungen und versuchen Sie es erneut.
common-notifications-enabled = Benachrichtigungen auf diesem Gerät aktiviert.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [one] 1 ausgewählt
   *[other] { $count } ausgewählt
    }
