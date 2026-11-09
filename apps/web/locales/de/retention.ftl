# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Aufbewahrung
retention-intro = Regeln, die Ihre eigenen E-Mails nach einer bestimmten Zahl von Tagen verschieben oder löschen. Eine Regel kann einen Ordner, ein Label, einen Absender, eine Größe, den Lesestatus oder das Vorhandensein eines Anhangs berücksichtigen. Markierte E-Mails und E-Mails mit dem Label keep bleiben unberührt, sofern Sie das nicht ändern. Gelöschte E-Mails gelangen zuerst in den Papierkorb und werden nach Ablauf der Rückgängig-Frist endgültig entfernt.
retention-disabled = Aufbewahrungsregeln sind auf diesem Server ausgeschaltet. Ihre vorhandenen Regeln werden nicht ausgeführt.
retention-never = nie
retention-error = Die Aufbewahrungseinstellungen konnten nicht geladen oder gespeichert werden. Versuchen Sie es erneut.
retention-last-run = Letzte Ausführung: { $when }
retention-next-run = nächste Ausführung in etwa { $days } Tag(en)
retention-next-run-unknown = nächste Ausführung noch nicht geplant
retention-enabled = Meine Aufbewahrungsregeln ausführen
retention-trash-first = Gelöschte E-Mails zuerst in den Papierkorb verschieben
retention-trash-forced = Dieser Server verschiebt E-Mails immer zuerst in den Papierkorb und entfernt sie nach Ablauf der Rückgängig-Frist endgültig.
retention-undo = Rückgängig-Frist in Tagen
retention-undo-help = E-Mails, die eine Regel in den Papierkorb verschoben hat, bleiben dort so lange, bevor sie endgültig entfernt werden. 0 bis { $max } Tage.
retention-pause = Alle Regeln für 30 Tage pausieren
retention-paused-until = Pausiert bis { $when }
retention-resume = Fortsetzen
retention-policy = Was dieser Server erlaubt
retention-limits-both = Eine Regel, die E-Mails löscht, wartet mindestens { $min } Tage und höchstens { $max } Tage.
retention-limits-min = Eine Regel, die E-Mails löscht, wartet mindestens { $min } Tage.
retention-max-rules = Sie können bis zu { $count } Regeln behalten.
retention-permanent-allowed = Eine Regel darf E-Mails endgültig löschen, wenn Sie den Papierkorb-Schritt ausschalten.
retention-permanent-denied = Regeln dürfen E-Mails nicht endgültig löschen. E-Mails gelangen in den Papierkorb und werden nach { $days } Tagen endgültig entfernt.
retention-protected = Ordner, die Regeln nie anfassen: { $folders }.
retention-mandatory = Von der Administration festgelegt: E-Mails in { $folder }, die älter als { $days } Tage sind, werden gelöscht.
retention-rules = Ihre Regeln
retention-add = Neue Regel
retention-no-rules = Sie haben noch keine Regeln. Übernehmen Sie unten eine Vorlage oder fügen Sie eine hinzu.
retention-rule-paused = pausiert
retention-rule-on = An
retention-rule-enabled = Regel { $name } ist an
retention-edit-short = Bearbeiten
retention-delete = Löschen
retention-new = Neue Regel
retention-edit = Regel bearbeiten
retention-name = Name
retention-action = Was geschehen soll
retention-action-keep = Dauerhaft behalten
retention-action-delete = Löschen
retention-action-trash = In den Papierkorb verschieben
retention-action-archive = Ins Archiv verschieben
retention-action-move = In einen Ordner verschieben
retention-action-markRead = Als gelesen markieren
retention-days = Nach Tagen
retention-target = Ordner
retention-choose-folder = Ordner auswählen
retention-scope = Welche E-Mails
retention-folder = Ordner
retention-any-folder = Beliebiger Ordner
retention-subfolders = Unterordner einbeziehen
retention-labels = Labels (durch Kommas getrennt)
retention-senders = Absenderadressen (durch Kommas getrennt)
retention-domains = Absenderdomains (durch Kommas getrennt)
retention-min-size = Kleinste Größe in KB
retention-max-size = Größte Größe in KB
retention-read = Lesestatus
retention-read-any = Gelesen oder ungelesen
retention-read-read = Nur gelesen
retention-read-unread = Nur ungelesen
retention-attachment = Anhänge
retention-attachment-any = Mit oder ohne
retention-attachment-with = Mit Anhang
retention-attachment-without = Ohne Anhang
retention-exclude = Unberührt lassen
retention-exclude-flagged = Markierte (mit Stern versehene) E-Mails
retention-exclude-unread = Ungelesene E-Mails
retention-exclude-tags = E-Mails mit diesen Labels (durch Kommas getrennt)
retention-exclude-note = E-Mails unter einer Rechtssperre werden nie angefasst, unabhängig von dieser Einstellung.
retention-role-inbox = Posteingang
retention-role-sent = Gesendet
retention-role-archive = Archiv
retention-role-junk = Spam
retention-role-trash = Papierkorb
retention-role-drafts = Entwürfe
retention-btn-preview = Vorschau
retention-save = Speichern
retention-cancel = Abbrechen
retention-preview-none = Keine Nachricht würde sich ändern.
retention-preview-count = Dies würde { $count } Nachricht(en) ändern.
retention-preview-truncated = Für diese Vorschau wurde nur ein Teil des Postfachs gelesen.
retention-preview-sample = Beispiele
retention-no-subject = (kein Betreff)
retention-do-trash = in den Papierkorb
retention-do-move = verschoben
retention-do-delete = gelöscht
retention-do-markRead = als gelesen markiert
retention-skipped-legalHold = Ihr Konto unterliegt einer Rechtssperre. Es wird nichts geändert.
retention-skipped-restricted = Die Verarbeitung Ihres Kontos ist eingeschränkt. Es wird nichts geändert.
retention-skipped-noRules = Sie haben keine aktiven Regeln, und die Administration hat keine festgelegt.
retention-error-name = Geben Sie der Regel einen Namen.
retention-error-days = Die Tage müssen eine ganze Zahl von 0 bis 3650 sein.
retention-error-min = Eine Regel, die E-Mails löscht, muss mindestens { $days } Tage warten.
retention-error-max = Eine Regel, die E-Mails löscht, darf höchstens { $days } Tage warten.
retention-error-target = Wählen Sie den Ordner, in den E-Mails verschoben werden sollen.
retention-summary-keep = dauerhaft behalten
retention-summary-delete = nach { $days } Tagen in den Papierkorb, dann endgültig entfernt
retention-summary-delete-permanent = nach { $days } Tagen endgültig gelöscht
retention-summary-trash = nach { $days } Tagen in den Papierkorb
retention-summary-archive = nach { $days } Tagen ins Archiv
retention-summary-move = nach { $days } Tagen in einen Ordner
retention-summary-markRead = nach { $days } Tagen als gelesen markiert
retention-templates = Vorlagen
retention-templates-help = Eine Vorlage fügt eine vorgefertigte Regel hinzu. Prüfen Sie ihre Labels und Tage, bevor Sie sich darauf verlassen.
retention-adopt = Diese Regel hinzufügen
retention-run-heading = Jetzt ausführen
retention-preview-all = Vorschau aller Regeln
retention-btn-run = Meine Regeln jetzt ausführen
retention-run-result = { $count } Nachricht(en) geändert.
retention-history = Letzte Ausführungen
retention-history-when = Wann
retention-history-matched = Treffer
retention-history-changed = Geändert
retention-history-errors = Fehler
retention-context-action = Aufbewahrung …

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Kalender

pim-cal-days = Tage nach Ende einer Veranstaltung

pim-cal-enabled = Beendete Ereignisse löschen

pim-cal-help = Löschen Sie Ereignisse in den von Ihnen ausgewählten Kalendern eine festgelegte Anzahl von Tagen nach ihrem Ende. Wiederkehrende Ereignisse werden niemals gelöscht.

pim-cal-title = Kalenderereignisse

pim-days-range = Wählen Sie zwischen { $min } und { $max } Tagen.

pim-held-title = Entfernte Notizen, die Sie wiederherstellen können

pim-held-until = Bis { $when } aufbewahrt

pim-limits = Zwischen { $min } und { $max } Tagen.

pim-notes-days = Tage seit der letzten Änderung einer Notiz

pim-notes-enabled = Entfernen Sie unberührte Notizen

pim-notes-help = Entfernen Sie Notizen, die Sie eine bestimmte Anzahl von Tagen lang nicht geändert haben. Angeheftete Notizen bleiben erhalten. Eine entfernte Notiz kann { $days } Tage lang wiederhergestellt werden.

pim-notes-title = Notizen

pim-off = Von Ihrem Administrator deaktiviert.

pim-preview = Vorschau

pim-restore = Wiederherstellen

pim-result = { $events } Ereignis(e) und { $notes } Notiz(en).

pim-run = Lauf jetzt

pim-run-title = Lauf jetzt

pim-save = Speichern

pim-skipped = { $count } Auswahl(en) wurden übersprungen, weil Ihr Administrator sie deaktiviert hat.

pim-untitled = Unbenannte Notiz

retention-action-stripAttachments = Entfernen Sie große Anhänge

retention-do-stripAttachments = Anhänge entfernt

retention-error-strip-disabled = Das Entfernen von Anhängen wurde von Ihrem Administrator deaktiviert.

retention-error-strip-min = Eine Regel, die Anhänge entfernt, muss mindestens { $days } Tage warten.

retention-fd-action = Was passiert mit der E-Mail-Adresse { $folder }?

retention-fd-days = Tage bevor { $folder } verarbeitet wird

retention-fd-help = Legen Sie fest, wie lange E-Mails in einem Ordner verbleiben, bevor sie verschoben oder gelöscht werden. Ein Ordner, der auf „Übernehmen“ eingestellt ist, hat keinen eigenen Standard; Es gelten weiterhin die oben genannten Regeln und die Beschränkungen des Administrators. Unterordner folgen ihrem Ordner.

retention-fd-inherit = Erben

retention-fd-mandatory = Der Administrator behält diesen Ordner { $days } Tage lang

retention-fd-protected = Geschützt durch den Administrator

retention-fd-save = Ordnerstandards speichern

retention-fd-title = Ordnerstandards

retention-rule-strip-off = Von Ihrem Administrator deaktiviert

retention-strip-allowed = Eine Regel kann Anhänge von mindestens { $kb } KB aus E-Mails entfernen, die mindestens { $days } Tage alt sind. Eine kurze Notiz ersetzt jeden Anhang.

retention-strip-denied = Das Entfernen von Anhängen wurde von Ihrem Administrator deaktiviert.

retention-strip-done = Anhänge aus { $count } Nachricht(en) entfernt, wodurch { $size } freigegeben wird.

retention-strip-inline-kept = Die in der Nachricht angezeigten Bilder bleiben erhalten.

retention-strip-inline-removed = Die in der Nachricht angezeigten Bilder werden ebenfalls entfernt.

retention-strip-limits = Entfernt Anhänge von mindestens { $kb } KB aus E-Mails, die mindestens { $days } Tage alt sind. Das Original bleibt { $undo } Tage im Papierkorb.

retention-strip-preview = { $count }-Nachrichten würden Anhänge verlieren und { $size } freigeben.

retention-strip-preview-estimate = Ungefähr { $count } Nachrichten würden Anhänge verlieren, wodurch bis zu { $size } frei würden.

retention-strip-skipped = { $count } Regel(n) zum Entfernen von Anhängen wurden übersprungen, da Ihr Administrator dies deaktiviert hat.

retention-summary-stripAttachments = Nach { $days } Tagen wurden große Anhänge entfernt
