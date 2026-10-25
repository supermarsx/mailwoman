# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Dashboard
admin-shell-group-overview = Übersicht
admin-shell-group-people = Personen
admin-shell-group-security = Sicherheit
admin-shell-group-configuration = Konfiguration
admin-shell-group-operations = Betrieb
admin-shell-menu = Menü
admin-shell-close-menu = Menü schließen
admin-shell-search = Suchen
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Angemeldet als { $name }
admin-shell-language-label = Sprache
admin-shell-breadcrumb = Navigationspfad
admin-shell-unknown-section = Diesen Bereich gibt es nicht, oder Ihre Rolle umfasst ihn nicht.
admin-shell-back-to-dashboard = Zurück zum Dashboard
admin-shell-cap-missing = Ihre Rolle umfasst { $capability } nicht.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Wird geladen…
admin-shell-retry = Erneut versuchen
admin-shell-cancel = Abbrechen
admin-shell-save = Speichern
admin-shell-not-available = Nicht verfügbar

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Diese Seite verlassen?
admin-shell-leave-body = Ihre Änderungen auf dieser Seite wurden nicht gespeichert. Beim Verlassen werden sie verworfen.
admin-shell-leave-confirm = Verlassen und Änderungen verwerfen

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Geben Sie { $name } zur Bestätigung ein

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Admin-Konsole durchsuchen
admin-shell-palette-label = Bereich, Benutzer, Domain oder Einstellung
admin-shell-palette-results = Ergebnisse
admin-shell-palette-none = Keine Treffer.
admin-shell-palette-searching = Benutzer, Domains und Einstellungen werden durchsucht…
admin-shell-palette-sections-only = Dieser Server kann Benutzer, Domains und Einstellungen noch nicht durchsuchen, daher werden nur Bereiche aufgeführt.
admin-shell-palette-error = Die Suche nach Benutzern, Domains und Einstellungen ist fehlgeschlagen; nur Bereiche werden aufgeführt.
admin-shell-palette-kind-section = Bereich
admin-shell-palette-kind-user = Benutzer
admin-shell-palette-kind-domain = Domain
admin-shell-palette-kind-setting = Einstellung

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filter
admin-shell-table-actions = Aktionen
admin-shell-table-error = Die Liste konnte nicht geladen werden.
admin-shell-table-paging = Seiten von { $table }
admin-shell-table-prev = Vorherige Seite
admin-shell-table-next = Nächste Seite

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Aktuell { $latest }, Höchstwert { $max }
admin-shell-chart-col-time = Zeit
admin-shell-chart-col-item = Element
admin-shell-chart-col-value = Wert

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Festgelegt durch
admin-shell-pinned-sr = Dieser Wert stammt aus der Umgebung des Servers und kann hier nicht geändert werden.
admin-shell-lock-text = Für Benutzer sperren
admin-shell-lock-setting = { $setting }
admin-shell-restore = Vorherigen Wert wiederherstellen
admin-shell-restore-note = Geändert von { $actor } am { $time }. Vorheriger Wert: { $value }
admin-shell-restore-done = Der vorherige Wert ist wiederhergestellt.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Schritte
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Schritt { $n } von { $total }: { $title }
admin-shell-wizard-review = Überprüfen
admin-shell-wizard-back = Zurück
admin-shell-wizard-next = Weiter
admin-shell-wizard-saved = Gespeichert.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Der Server war nicht erreichbar. Prüfen Sie die Verbindung und versuchen Sie es erneut.
admin-shell-err-session = Ihre Admin-Sitzung ist beendet. Melden Sie sich erneut an.
admin-shell-err-forbidden = Ihre Rolle erlaubt diese Aktion nicht. Wenden Sie sich an einen Admin mit der passenden Rolle.
admin-shell-err-out-of-scope = Dieses Objekt gehört zu einer Domain, die Ihre Rolle nicht abdeckt.
admin-shell-err-not-found = Es existiert nicht mehr. Laden Sie die Liste neu.
admin-shell-err-conflict = Jemand hat es inzwischen geändert. Laden Sie neu und versuchen Sie es erneut.
admin-shell-err-validation = Der Server hat einen Wert abgewiesen. Prüfen Sie die Felder und versuchen Sie es erneut.
admin-shell-err-rate-limited = Zu viele Anfragen. Warten Sie eine Minute und versuchen Sie es erneut.
admin-shell-err-server = Der Server antwortete mit einem Fehler ({ $status }). Versuchen Sie es erneut; wenn es sich wiederholt, prüfen Sie das Serverprotokoll.
admin-shell-err-upstream-dns = Der Hostname konnte nicht aufgelöst werden. Prüfen Sie die Schreibweise des Hosts.
admin-shell-err-upstream-connect = Auf diesem Host und Port hat nichts geantwortet. Prüfen Sie den Port und eine eventuell dazwischenliegende Firewall.
admin-shell-err-upstream-tls = Der Server hat geantwortet, aber der TLS-Handshake ist fehlgeschlagen. Prüfen Sie den Port und die Einstellung zur Verbindungssicherheit.
admin-shell-err-upstream-auth = Der Server hat die Zugangsdaten abgewiesen. Prüfen Sie Benutzername und Passwort.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Zu viele Anmeldeversuche. Warten Sie einige Minuten und versuchen Sie es erneut.
admin-shell-totp-form = Admin-Anmeldung, zweiter Schritt
admin-shell-totp-note = Geben Sie den 6-stelligen Code aus der Authenticator-App ein, die für { $name } eingerichtet ist.
admin-shell-totp-code = Authentifizierungscode
admin-shell-totp-verify = Bestätigen
admin-shell-totp-start-over = Neu beginnen
admin-shell-totp-invalid = Dieser Code ist ungültig. Geben Sie den aktuellen Code aus Ihrer Authenticator-App ein.
admin-shell-totp-expired = Die Anmeldung hat zu lange gedauert. Beginnen Sie neu und geben Sie Ihr Passwort ein.
