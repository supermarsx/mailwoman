# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: rules. Source: ../en/rules.ftl

rules-title = Regeln & Filter
rules-intro = Verwalten Sie Filter-Skripte und visuelle Regeln. Verfügbare Funktionen hängen vom Konto und den Administratoreinstellungen ab.
rules-empty = Noch keine Regeln.
rules-unavailable = Dieser Mailserver unterstützt keine E-Mail-Regeln.
rules-load-failed = Die Regeln konnten nicht geladen werden.
rules-new = Neue Regel
rules-list-label = E-Mail-Regeln
rules-enabled = Aktiviert
rules-delete-named = Regel { $name } löschen

# Builder
rules-builder-title = Regel-Editor
rules-builder-hint = Wählen Sie eine Regel zum Bearbeiten aus oder erstellen Sie eine neue.
rules-name-label = Name der Regel
rules-match-legend = Wann ausführen
rules-match-all = Alle Bedingungen erfüllen
rules-match-any = Eine beliebige Bedingung erfüllen
rules-conditions-legend = Bedingungen
rules-actions-legend = Aktionen
rules-add-condition = Bedingung hinzufügen
rules-add-action = Aktion hinzufügen
rules-remove-condition = Bedingung entfernen
rules-remove-action = Aktion entfernen
rules-cond-type-label = Feld
rules-cond-op-label = Operator
rules-cond-value-label = Vergleichswert
rules-act-type-label = Aktion
rules-act-value-label = Aktionswert
rules-save = Regel speichern
rules-editing = { $name } wird bearbeitet
rules-save-refused = Die Regel wurde nicht gespeichert: { $reason }

# Condition fields and operators
rules-cond-from = Von
rules-cond-to = An
rules-cond-subject = Betreff
rules-cond-thread-unsupported = Konversation { $op } { $value }: Konversationsbedingungen werden nicht unterstützt, daher kann die Regel erst gespeichert werden, wenn diese Bedingung entfernt wurde.
rules-op-contains = enthält
rules-op-is = ist

# Actions
rules-act-move = In Postfach verschieben
rules-act-tag = Schlagwort hinzufügen
rules-act-archive = Archivieren
rules-act-suppress = Benachrichtigung unterdrücken
rules-act-stop = Verarbeitung beenden

# Where it runs
rules-runs-server = Server (Sieve)
rules-runs-server-detail = Wird auf den Mailserver hochgeladen und dort angewendet.
rules-runs-engine = Engine
rules-runs-engine-detail = Wird beim Synchronisieren neuer Posteingangsnachrichten angewendet; Antworten und Verschiebungen warten auf die nächste Synchronisierung.

# Panes
rules-panes-label = Regelwerkzeuge
rules-tab-builder = Editor
rules-tab-raw = Sieve-Quelltext
rules-tab-dryrun = Testlauf

# Raw editor
rules-raw-label = Sieve-Quelltext
rules-raw-reset = Auf generierten Stand zurücksetzen
rules-lint-clean = Keine Probleme gefunden.

# Dry run
rules-dryrun-title = Testlauf
rules-dryrun-help = Geben Sie eine Beispielnachricht ein, um zu sehen, welche Regeln greifen würden.
rules-dryrun-results = Ergebnisse des Testlaufs
rules-dryrun-none = Keine aktivierten Regeln zum Auswerten.
rules-dryrun-nomatch = Kein Treffer
rules-dryrun-stopped = Nicht erreicht (eine vorherige Regel hat die Verarbeitung beendet)

rules-scripts-label = Sieve-Skripte
rules-script-active = Aktiv
rules-script-new = Neues Skript
rules-script-export = Skript exportieren
rules-script-import = Skript importieren
rules-script-too-large = Das Skript überschreitet die Größenbegrenzung.
rules-script-read-only = Die direkte Skriptbearbeitung wurde administrativ deaktiviert.
rules-script-validate = Skript prüfen
rules-script-deactivate = Deaktivieren
rules-script-activate = Aktivieren
rules-script-saved = Skript gespeichert.

rules-vacation-title = Abwesenheitsnotiz
rules-vacation-enabled = Automatische Antworten aktivieren
rules-vacation-from = Beginn (UTC)
rules-vacation-to = Ende (UTC)
rules-vacation-body = Antworttext
rules-vacation-saved = Abwesenheitsnotiz gespeichert.
rules-script-template = Skriptvorlage
rules-forward-title = Weiterleitung
rules-forward-address = Weiterleiten an
rules-forward-copy = Kopie in diesem Konto behalten
rules-forward-refused = Die Weiterleitung an diese Domain ist nicht erlaubt.
