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
