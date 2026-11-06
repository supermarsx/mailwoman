# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Kalender
calendar-prev = Zurück
calendar-next = Weiter
calendar-today = Heute
calendar-views = Kalenderansichten
calendar-new-event = Neuer Termin
calendar-import = Importieren
calendar-export = Exportieren
calendar-import-file = Kalenderdatei importieren

# -- View names --------------------------------------------------------------
calendar-view-day = Tag
calendar-view-3day = 3 Tage
calendar-view-work-week = Arbeitswoche
calendar-view-week = Woche
calendar-view-month = Monat
calendar-view-tri-month = Quartal
calendar-view-schedule = Zeitplan
calendar-view-agenda = Agenda
calendar-view-year = Jahr

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Kalender
calendar-toggle = { $name } ein-/ausblenden
calendar-color-for = Farbe für { $name }
calendar-add-calendar = Kalender hinzufügen
calendar-new-calendar-name = Neuer Kalender
calendar-holidays-heading = Feiertage
calendar-subscribe-holidays = Feiertage abonnieren
calendar-add-region = Region hinzufügen…
calendar-synced = Synchronisierter Kalender

# -- Grid / views ------------------------------------------------------------
calendar-all-day = ganztägig
calendar-all-day-full = Ganztägig
calendar-more = { $count } weitere
calendar-conflict = Konflikt
calendar-no-events-30 = Keine Termine in den nächsten 30 Tagen.
calendar-events-count = { $count ->
    [zero] Keine Termine
    [one] { $count } Termin
   *[other] { $count } Termine
}
calendar-month-grid = Monat { $month }
calendar-time-grid = Zeitraster
calendar-cell = { $date }, { $events }
calendar-announce = { $date }, { $events }
calendar-event-at = { $time } { $title }
calendar-event-allday = Ganztägig: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Neuer Termin
calendar-editor-edit = Termin bearbeiten
calendar-invitation = Einladung
calendar-invited = Sie sind eingeladen ({ $status }).
calendar-accept = Annehmen
calendar-tentative = Vorläufig
calendar-decline = Ablehnen
calendar-counter = Gegenvorschlag…
calendar-propose-start = Neuen Beginn vorschlagen
calendar-send-counter = Gegenvorschlag senden
calendar-overlaps = Dieser Termin überschneidet sich mit { $count ->
    [one] { $count } anderen Termin
   *[other] { $count } anderen Terminen
}.
calendar-field-title = Titel
calendar-untitled = (ohne Titel)
calendar-field-calendar = Kalender
calendar-field-all-day = Ganztägig
calendar-field-start = Beginn
calendar-field-duration = Dauer (Min.)
calendar-field-location = Ort
calendar-field-shows-as = Anzeigen als
calendar-shows-busy = Beschäftigt
calendar-shows-free = Frei
calendar-field-status = Status
calendar-status-confirmed = Bestätigt
calendar-status-tentative = Vorläufig
calendar-status-cancelled = Abgesagt
calendar-repeats = Wiederholung
calendar-frequency = Häufigkeit
calendar-freq-daily = Täglich
calendar-freq-weekly = Wöchentlich
calendar-freq-monthly = Monatlich
calendar-freq-yearly = Jährlich
calendar-every = alle
calendar-interval = Intervall
calendar-ends = Endet
calendar-end-mode = Endmodus
calendar-end-never = Nie
calendar-end-count = Nach N
calendar-end-until = Am Datum
calendar-occurrences = Wiederholungen
calendar-until-date = Bis Datum
calendar-reminders = Erinnerungen
calendar-reminder-before = { $min } Min. vorher
calendar-remove-reminder = Erinnerung { $min } entfernen
calendar-add-reminder = Erinnerung hinzufügen
calendar-reminder-none = + Erinnerung
calendar-reminder-5 = 5 Min.
calendar-reminder-15 = 15 Min.
calendar-reminder-30 = 30 Min.
calendar-reminder-60 = 1 Stunde
calendar-reminder-1440 = 1 Tag
calendar-attendees = Teilnehmer
calendar-add-attendee = Teilnehmer hinzufügen
calendar-attendee-invalid = Das ist keine E-Mail-Adresse. Geben Sie eine Adresse ein, z. B. name@example.com.
calendar-save-failed = Der Termin wurde nicht gespeichert. Der Server hat ihn abgelehnt.
calendar-remove-attendee = { $email } entfernen
calendar-notes = Notizen

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Rolle für { $email }
calendar-attendee-cutype-for = Typ für { $email }
calendar-role-chair = Vorsitz
calendar-role-required = Erforderlich
calendar-role-optional = Optional
calendar-role-non-participant = Nicht teilnehmend
calendar-cutype-individual = Person
calendar-cutype-group = Gruppe
calendar-cutype-resource = Ressource
calendar-cutype-room = Raum
calendar-partstat-needs-action = Keine Antwort
calendar-partstat-accepted = Angenommen
calendar-partstat-declined = Abgelehnt
calendar-partstat-tentative = Vorläufig

# -- Schedule view (distinct from Agenda) ------------------------------------
calendar-schedule-row = { $start } bis { $end }: { $title }
calendar-schedule-free = { $gap } frei
calendar-gap-hm = { $h } Std. { $m } Min.
calendar-gap-h = { $h } Std.
calendar-gap-m = { $m } Min.

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = { $count ->
    [one] { $count } Konflikt lösen
   *[other] { $count } Konflikte lösen
}
calendar-resolver-title = Konflikte lösen
calendar-resolver-none = Keine Konflikte zu lösen.
calendar-resolver-pick = Konflikt
calendar-resolver-pair-n = { $n } von { $total }: { $a } gegen { $b }
calendar-resolver-time = Zeit
calendar-resolver-overlap = Überschneidung { $start } – { $end }
calendar-resolver-reschedule = Späteren Termin verschieben
calendar-resolver-shorten = Früheren Termin kürzen
calendar-resolver-tentative = Späteren als vorläufig markieren
calendar-resolver-double-book = Doppelt buchen (als frei markieren)
calendar-resolver-keep = Beide behalten
calendar-events-count-people = { $count ->
    [zero] Keine Teilnehmer
    [one] { $count } Teilnehmer
   *[other] { $count } Teilnehmer
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Kalender
calendar-fb-own = Ihre
calendar-fb-unavailable = Ihre belegte Zeit konnte nicht geladen werden.
calendar-fb-cell = { $principal } um { $hour }: { $status }
calendar-fb-busy = Beschäftigt
calendar-fb-tentative = Vorläufig
calendar-fb-free = Frei

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Termin schnell hinzufügen
calendar-quick-add-placeholder = Schnell hinzufügen, z. B. Mittagessen Freitag 13 Uhr
calendar-quick-add-btn = Hinzufügen
calendar-quick-add-do = Schnell hinzufügen

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Kategorien
calendar-add-category = Kategorie hinzufügen
calendar-category-placeholder = Kategorie hinzufügen
calendar-remove-category = Kategorie { $name } entfernen
calendar-filter-heading = Filter
calendar-filter-category = Nach Kategorie filtern
calendar-filter-category-placeholder = Anzuzeigende Kategorie

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Anhänge
calendar-attachment-title = Name des Anhangs
calendar-attachment-title-placeholder = Name (optional)
calendar-attachment-uri = Link zum Anhang
calendar-attachment-add = Anhang hinzufügen
calendar-remove-attachment = Anhang { $name } entfernen

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Per URL abonnieren
calendar-subscribe-url = Kalender-URL
calendar-subscribe-add = Abonnieren
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Der Kalender unter dieser Adresse konnte nicht abgerufen werden und wurde daher nicht hinzugefügt.
calendar-subscribe-hol-many-locations = Diese Feiertagsdatei enthält mehr als einen Ort. Laden Sie sie herunter, verwenden Sie dann „Importieren“ und wählen Sie die gewünschten Orte aus.
calendar-subscribe-refresh = { $name } aktualisieren
calendar-subscribe-refresh-btn = Aktualisieren
calendar-subscribe-refresh-failed = Der Kalender-Feed konnte nicht abgerufen werden. Seine Termine wurden unverändert gelassen.
calendar-import-failed = Diese Datei konnte nicht als Kalender gelesen werden. Es wurde nichts importiert.
calendar-import-done = { $count ->
    [one] 1 Termin importiert.
   *[other] { $count } Termine importiert.
}
calendar-import-locations-title = Zu importierende Orte auswählen
calendar-import-locations-intro = Diese Datei enthält Feiertage für { $count } Orte. Nur die ausgewählten Orte werden importiert.
calendar-import-locations-filter = Orte filtern
calendar-import-locations-list = Orte in der Datei
calendar-import-locations-none = Kein Ort entspricht dem Filter.
calendar-import-locations-picked = { $count ->
    [one] 1 Ort ausgewählt.
   *[other] { $count } Orte ausgewählt.
}
calendar-import-locations-import = Auswahl importieren
calendar-export-failed = Der Kalender konnte nicht exportiert werden.
calendar-quick-add-failed = Dieser Termin konnte nicht hinzugefügt werden.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Teilen
calendar-share-for = { $name } teilen
calendar-share-title = { $name } teilen
calendar-share-intro = Personen, mit denen Sie teilen, können diesen Kalender sehen. Erteilen Sie Lese- und Schreibzugriff, damit sie Termine hinzufügen und bearbeiten können.
calendar-share-people = Geteilt mit
calendar-share-empty = Dieser Kalender ist mit niemandem geteilt.
calendar-share-add = Person hinzufügen
calendar-share-new-access = Zugriff für die neue Person
calendar-share-access-for = Zugriff für { $principal }
calendar-share-remove = { $principal } entfernen
calendar-share-read = Nur lesen
calendar-share-readwrite = Lesen & schreiben

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Datumsnavigation
calendar-event-range = { $title }, { $start } bis { $end }
calendar-busy-free = Wird als frei angezeigt
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = { $count ->
    [one] { $count } weiterer Termin
   *[other] { $count } weitere Termine
  } am { $date }
calendar-fb-caption-people = Belegte Zeiten am Tag des Konflikts, nach Stunde, für Sie und jede eingeladene Person.
calendar-fb-unknown = Die Verfügbarkeit von { $principal } konnte nicht abgerufen werden.
calendar-fb-loading = Verfügbarkeit wird geladen …
calendar-import-itip-message = Diese Datei ist eine per E-Mail gesendete Antwort oder Absage. Öffnen Sie die zugehörige Nachricht, um sie anzuwenden. Es wurde nichts importiert.
calendar-import-skipped = { $count ->
    [one] { $count } Termin war bereits im Kalender und blieb unverändert.
   *[other] { $count } Termine waren bereits im Kalender und blieben unverändert.
    }
calendar-invite-title = Kalendereinladung
calendar-invite-sender = Von { $sender }
calendar-invite-add = Zum Kalender hinzufügen
calendar-invite-update = Terminaktualisierung anwenden
calendar-invite-cancel = Absage anwenden
calendar-invite-reply = Antwort des Teilnehmers erfassen
calendar-invite-new = Diese Einladung kann zu Ihrem Kalender hinzugefügt werden.
calendar-invite-newer = Der Organisator hat einen aktualisierten Termin gesendet.
calendar-invite-current = Ihr Kalender enthält bereits diese Version.
calendar-invite-stale = Diese Nachricht ist älter als der Termin in Ihrem Kalender.
calendar-invite-cancelled = Der Organisator hat diesen Termin abgesagt.
calendar-invite-response = Ein Teilnehmer hat geantwortet.
calendar-invite-not-organizer = Der Absender ist nicht der gespeicherte Organisator. Es wird nichts geändert.
calendar-invite-not-attendee = Der Absender ist kein passender Teilnehmer. Es wird nichts geändert.
calendar-invite-unknown = Diese Nachricht bezieht sich auf einen Termin, der nicht in Ihrem Kalender ist.
calendar-invite-unsupported = Dieser Vorschlag kann nicht automatisch angewendet werden.
calendar-invite-no-reply = Dadurch wird Ihr Kalender aktualisiert, ohne eine Antwort zu senden.
calendar-invite-applied = Ihr Kalender wurde aktualisiert.
calendar-invite-unchanged = Der Termin wurde nicht geändert. Prüfen Sie den aktuellen Status der Einladung.
calendar-invite-apply-failed = Die Einladung konnte nicht angewendet werden. Versuchen Sie es erneut.
calendar-invite-load-failed = Die Einladung konnte nicht geprüft werden.
calendar-invite-retry = Erneut versuchen
calendar-field-time-zone = Zeitzone des Termins
calendar-field-time-zone-help = Verwenden Sie einen IANA-Namen wie Europe/Lisbon. Der Termin behält diese Zeitzone, wenn er an anderer Stelle angezeigt wird.
calendar-time-invalid = Geben Sie eine gültige Zeitzone und Ortszeit ein. Manche Ortszeiten existieren nicht, wenn die Uhr vorgestellt wird.
calendar-mini-prev = Vorheriger Monat
calendar-mini-next = Nächster Monat
calendar-card-region-event = Kalendertermin
calendar-card-kind-request = Besprechungseinladung
calendar-card-kind-update = Aktualisierte Einladung
calendar-card-kind-cancel = Abgesagte Besprechung
calendar-card-kind-reply = Antwort auf die Besprechung
calendar-card-kind-counter = Neue Zeit vorgeschlagen
calendar-card-kind-publish = { $count ->
    [one] Kalendertermin
   *[other] { $count } Kalendertermine
    }
calendar-card-when = Wann
calendar-card-where = Wo
calendar-card-organizer = Organisator
calendar-card-attendees = Teilnehmer
calendar-card-all-day = { $when } (ganztägig)
calendar-card-organizer-time = Zeit des Organisators: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Jeden Tag
   *[other] Alle { $interval } Tage
    }
calendar-card-repeat-weekly = { $interval ->
    [one] Jede Woche
   *[other] Alle { $interval } Wochen
    }
calendar-card-repeat-weekly-on = { $interval ->
    [one] Jede Woche: { $days }
   *[other] Alle { $interval } Wochen: { $days }
    }
calendar-card-repeat-monthly = { $interval ->
    [one] Jeden Monat
   *[other] Alle { $interval } Monate
    }
calendar-card-repeat-monthly-day = { $interval ->
    [one] Jeden Monat am { $day }.
   *[other] Alle { $interval } Monate am { $day }.
    }
calendar-card-repeat-yearly = { $interval ->
    [one] Jedes Jahr
   *[other] Alle { $interval } Jahre
    }
calendar-card-repeat-until = { $rule } bis { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] einmal
   *[other] { $count }-mal
    }
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 Termin ausgelassen
   *[other] { $count } Termine ausgelassen
    })
calendar-card-repeat-custom = Wiederholt sich nach einem benutzerdefinierten Zeitplan
calendar-card-join = { $provider } beitreten
calendar-card-organizer-is = Organisiert von { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 Teilnehmer
   *[other] { $count } Teilnehmer
    }
calendar-card-people-all = Alle { $count } anzeigen
calendar-card-people-fewer = Weniger anzeigen
calendar-card-rsvp-accepted = Zugesagt
calendar-card-rsvp-tentative = Vorläufig
calendar-card-rsvp-declined = Abgelehnt
calendar-card-rsvp-none = Noch keine Antwort
calendar-card-in-calendar = In Ihrem Kalender
calendar-card-cancelled = Abgesagt
calendar-card-open = Im Kalender öffnen
calendar-card-agenda = Ihr Kalender am { $day }
calendar-card-agenda-title = Ihr Tag
calendar-card-agenda-this-event = Dieser Termin
calendar-card-agenda-unavailable = Ihr Kalender für diesen Tag konnte nicht geladen werden.
calendar-card-conflicts = { $count ->
    [one] Überschneidung mit { $titles }
   *[other] Überschneidung mit { $count } Terminen: { $titles }
    }
calendar-card-no-conflicts = Keine Überschneidungen in Ihrem Kalender
calendar-card-invited-as = Sie wurden als { $address } eingeladen.
calendar-card-not-invited = In dieser Einladung ist keine Ihrer Adressen aufgeführt. Sie können sie zu Ihrem Kalender hinzufügen, aber nicht beantworten.
calendar-card-not-invited-short = In dieser Einladung ist keine Ihrer Adressen aufgeführt.
calendar-card-out-of-date = Diese Einladung ist veraltet. Eine neuere Version ist bereits in Ihrem Kalender.
calendar-card-replied = { $who } { $answer ->
    [accepted] hat zugesagt.
    [tentative] hat vorläufig zugesagt.
    [declined] hat abgelehnt.
   *[other] hat noch nicht geantwortet.
    }
calendar-card-counter = { $who } hat eine neue Zeit vorgeschlagen. Vorschläge werden nicht automatisch angewendet.
calendar-card-todos-skipped = { $count ->
    [one] Die Datei enthält außerdem 1 Aufgabe, die nicht hinzugefügt wird.
   *[other] Die Datei enthält außerdem { $count } Aufgaben, die nicht hinzugefügt werden.
    }
calendar-card-respond = Auf die Einladung antworten
calendar-card-accept = Annehmen
calendar-card-tentative = Vorläufig
calendar-card-decline = Ablehnen
calendar-card-propose = Neue Zeit vorschlagen
calendar-card-propose-start = Neuer Beginn
calendar-card-propose-send = Vorschlag senden
calendar-card-propose-invalid = Geben Sie eine gültige neue Startzeit ein.
calendar-card-send-response = Antwort an { $organizer } senden
calendar-card-comment = Kommentar (optional)
calendar-card-calendar = Kalender
calendar-card-events = Termine in dieser Datei
calendar-card-add-one = Hinzufügen
calendar-card-add-all = Alle hinzufügen ({ $count })
calendar-card-added = { $count ->
    [one] Zu Ihrem Kalender hinzugefügt.
   *[other] { $count } Termine zu Ihrem Kalender hinzugefügt.
    }
calendar-card-remove = Aus Kalender entfernen
calendar-card-removed = Aus Ihrem Kalender entfernt.
calendar-card-record-reply = Teilnehmerstatus aktualisieren
calendar-card-answered-sent = { $answer }. Ihre Antwort wurde an { $organizer } gesendet.
calendar-card-answered-not-sent = { $answer }. Ihr Kalender wurde aktualisiert, die Antwort konnte aber nicht gesendet werden.
calendar-card-answered-silently = { $answer }. Es wurde keine Antwort gesendet.
calendar-card-proposal-sent = Ihr Vorschlag wurde an { $organizer } gesendet.
calendar-card-not-sent = Die Nachricht konnte nicht gesendet werden.
calendar-card-more = { $count ->
    [one] 1 weiterer
   *[other] { $count } weitere
    }
