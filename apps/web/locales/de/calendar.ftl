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
calendar-fb-caption = Ihre belegte Zeit am Tag des Konflikts, nach Stunden.
calendar-fb-whose = Kalender
calendar-fb-own = Ihre
calendar-fb-own-only = Die Verfügbarkeit der Teilnehmer wird nicht angezeigt. Der Server meldet belegte Zeiten nur für Ihre eigenen Kalender.
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
