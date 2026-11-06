# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Senden rückgängig machen
send-delay-help = Nach dem Klick auf „Senden“ wartet die Nachricht so lange in Ihrem Postausgang. Bis dahin können Sie sie zurückrufen und weiter bearbeiten. Der Server hält sie zurück, sodass das Schließen des Browsers sie weder vorzeitig sendet noch verloren gehen lässt.
send-delay-label = Gesendete Nachrichten zurückhalten für
send-delay-off = Aus (sofort senden)
send-delay-seconds = { $n } Sekunden
send-delay-minutes = { $n ->
    [one] 1 Minute
   *[other] { $n } Minuten
    }
send-delay-custom = Benutzerdefiniert …
send-delay-custom-label = Benutzerdefinierte Verzögerung in Sekunden
send-delay-custom-help = Ganze Sekunden, 0 bis { $max }.
send-delay-capped = Ihre Administration begrenzt dies auf { $max }.
send-delay-disabled = Das Zurückrufen gesendeter Nachrichten ist von Ihrer Administration deaktiviert. Nachrichten werden sofort gesendet.
send-delay-invalid = Geben Sie eine ganze Zahl von Sekunden von 0 bis { $max } ein.
send-later-button = Später senden
send-later-menu = Optionen für „Später senden“
send-later-preset-hour = In 1 Stunde
send-later-preset-today = Heute später, 17:00
send-later-preset-tomorrow = Morgen früh, 09:00
send-later-preset-monday = Montag früh, 09:00
send-later-custom = Datum und Uhrzeit wählen …
send-later-custom-label = Senden am
send-later-zone = Zeitzone: { $zone }
send-later-schedule = Senden planen
send-later-cancel = Abbrechen
send-later-err-invalid = Geben Sie ein gültiges Datum und eine gültige Uhrzeit ein.
send-later-err-past = Wählen Sie einen Zeitpunkt in der Zukunft.
send-later-err-horizon = Das liegt zu weit in der Zukunft. Nachrichten können bis zu { $days } Tage im Voraus geplant werden.
send-later-err-off = Das geplante Senden ist von Ihrer Administration deaktiviert.
send-queued = Nachricht in der Warteschlange — wird gesendet in
send-queued-undo = Rückgängig
send-queued-now = Jetzt senden
send-queued-recalled = Nachricht zurückgerufen. Sie befindet sich wieder in Ihren Entwürfen.
send-queued-too-late = Zu spät zum Zurückrufen: Die Nachricht wird bereits gesendet.
send-queued-sent-now = Wird jetzt gesendet.
send-queued-scheduled = Geplant für { $when } ({ $zone }).
outbox-scheduled-badge = Geplant
outbox-sends-in = Wird gesendet in { $time }
outbox-sends-at = Wird gesendet { $when }
outbox-recall-edit = Zurückrufen und bearbeiten
outbox-recall-too-late = Wird bereits gesendet; ein Zurückrufen ist nicht mehr möglich.
outbox-failed-drafts = Konnte nicht gesendet werden und wurde in Ihren Entwürfen aufbewahrt.
outbox-queued-count = { $count ->
    [one] 1 Nachricht wartet auf das Senden
   *[other] { $count } Nachrichten warten auf das Senden
    }
