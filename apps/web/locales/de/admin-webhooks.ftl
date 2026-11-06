# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooks
admin-webhooks-intro = Ein ausgehender Webhook sendet eine signierte Benachrichtigung an eine URL, wenn in einem Konto etwas passiert. Ein eingehender Webhook erlaubt es einem anderen System, in einem Konto eine Aktion einer E-Mail-Regel auszuführen.
admin-webhooks-empty = Noch keine Webhooks.
admin-webhooks-unavailable = Dieser Server bietet keine Webhook-Verwaltung an.
admin-webhooks-load-error = Die Webhooks konnten nicht geladen werden.
admin-webhooks-direction-outbound = Ausgehend
admin-webhooks-direction-inbound = Eingehend
admin-webhooks-col-direction = Richtung
admin-webhooks-col-label = Name
admin-webhooks-col-account = Konto
admin-webhooks-col-target = Sendet an oder erlaubt
admin-webhooks-col-created = Erstellt
admin-webhooks-no-label = (kein Name)
admin-webhooks-created-by = von { $who }
admin-webhooks-last-used = Zuletzt verwendet { $when }
admin-webhooks-never-used = Noch nicht verwendet
admin-webhooks-delete = Löschen
admin-webhooks-delete-named = Webhook { $name } löschen
admin-webhooks-delete-title = Den Webhook { $name } löschen?
admin-webhooks-delete-body = Lieferungen an ihn werden eingestellt, oder mit seinem Geheimnis signierte Aufrufe werden abgelehnt. Dies kann nicht rückgängig gemacht werden; ein neuer Webhook erhält ein neues Geheimnis.
admin-webhooks-deleted = Webhook gelöscht.
admin-webhooks-add = Webhook hinzufügen
admin-webhooks-add-direction = Richtung
admin-webhooks-add-outbound-help = Mailwoman ruft Ihre URL auf.
admin-webhooks-add-inbound-help = Ihr System ruft Mailwoman auf.
admin-webhooks-add-account = Konto
admin-webhooks-add-account-help = Das Konto, dessen Ereignisse gesendet werden, oder das einzige Konto, auf dem ein eingehender Aufruf handeln darf.
admin-webhooks-add-label = Name
admin-webhooks-add-label-help = Wofür der Webhook gedacht ist, damit andere es erkennen. Optional.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Muss eine öffentliche Internetadresse erreichen. Private, Loopback- und Link-Local-Adressen werden abgelehnt.
admin-webhooks-add-events = Ereignisse
admin-webhooks-add-actions = Erlaubte Aktionen
admin-webhooks-add-submit = Webhook erstellen
admin-webhooks-add-saving = Wird erstellt…
admin-webhooks-add-check = Prüfen
admin-webhooks-add-check-ok = Die URL besteht die Prüfung. Es wurde nichts gespeichert.
admin-webhooks-event-message-received = Neue Nachricht
admin-webhooks-event-message-flags = Nachricht gelesen, markiert oder mit Label versehen
admin-webhooks-event-message-moved = Nachricht verschoben
admin-webhooks-event-message-deleted = Nachricht gelöscht
admin-webhooks-event-submission-sent = Nachricht gesendet
admin-webhooks-event-submission-failed = Nachricht konnte nicht gesendet werden
admin-webhooks-event-pim-changed = Kalender, Kontakte, Aufgaben oder Notizen geändert
admin-webhooks-action-run-rules = Die E-Mail-Regeln des Kontos auf eine Nachrichtenbeschreibung anwenden
admin-webhooks-action-sieve = Eine Regelaktion auf eine Nachricht anwenden
admin-webhooks-secret-title = Geheimnis für { $name }
admin-webhooks-secret-once = Kopieren Sie dieses Geheimnis jetzt. Es wird nicht erneut angezeigt.
admin-webhooks-secret-outbound = Jede Lieferung enthält X-Mailwoman-Signature: sha256= gefolgt vom HMAC-SHA256 des Anfragekörpers unter diesem Geheimnis.
admin-webhooks-secret-inbound = Senden Sie X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (Unix-Sekunden), X-Mailwoman-Nonce (16 bis 128 Buchstaben, Ziffern, - oder _, nie wiederverwendet) und X-Mailwoman-Signature: sha256= gefolgt vom HMAC-SHA256 von timestamp.nonce.body unter diesem Geheimnis. Aufrufe mit mehr als fünf Minuten Abweichung oder mit wiederverwendeter Nonce werden abgelehnt.
admin-webhooks-secret-done = Ich habe es kopiert
admin-webhooks-error-stale = Jemand hat die Webhooks geändert, seit Sie diese Seite geöffnet haben. Die Liste wurde neu geladen; versuchen Sie es erneut.
admin-webhooks-error-account = Geben Sie das Konto ein.
admin-webhooks-error-url = Geben Sie eine URL ein, die mit https:// oder http:// beginnt.
admin-webhooks-error-events = Wählen Sie mindestens ein Ereignis.
admin-webhooks-error-actions = Wählen Sie mindestens eine Aktion.
