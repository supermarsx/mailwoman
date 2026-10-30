# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Agenda
calendar-prev = Vorige
calendar-next = Volgende
calendar-today = Vandaag
calendar-views = Agendaweergaven
calendar-new-event = Nieuwe afspraak
calendar-import = Importeren
calendar-export = Exporteren
calendar-import-file = Agendabestand importeren

# -- View names --------------------------------------------------------------
calendar-view-day = Dag
calendar-view-3day = 3 dagen
calendar-view-work-week = Werkweek
calendar-view-week = Week
calendar-view-month = Maand
calendar-view-tri-month = Kwartaal
calendar-view-schedule = Planning
calendar-view-agenda = Agendalijst
calendar-view-year = Jaar

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Agenda's
calendar-toggle = { $name } tonen/verbergen
calendar-color-for = Kleur voor { $name }
calendar-add-calendar = Agenda toevoegen
calendar-new-calendar-name = Nieuwe agenda
calendar-holidays-heading = Feestdagen
calendar-subscribe-holidays = Abonneren op feestdagen
calendar-add-region = Regio toevoegen…
calendar-synced = Gesynchroniseerde agenda

# -- Grid / views ------------------------------------------------------------
calendar-all-day = hele dag
calendar-all-day-full = Hele dag
calendar-more = nog { $count }
calendar-conflict = conflict
calendar-no-events-30 = Geen afspraken in de komende 30 dagen.
calendar-events-count = { $count ->
    [0] Geen afspraken
    [one] { $count } afspraak
   *[other] { $count } afspraken
}
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Maand { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Tijdraster
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }, { $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }, { $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = Hele dag: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Nieuwe afspraak
calendar-editor-edit = Afspraak bewerken
calendar-invitation = Uitnodiging
calendar-invited = Je bent uitgenodigd ({ $status }).
calendar-accept = Accepteren
calendar-tentative = Voorlopig
calendar-decline = Weigeren
calendar-counter = Tegenvoorstel…
calendar-propose-start = Nieuwe begintijd voorstellen
calendar-send-counter = Tegenvoorstel verzenden
calendar-overlaps = Deze afspraak overlapt met { $count ->
    [one] { $count } andere afspraak
   *[other] { $count } andere afspraken
}.
calendar-field-title = Titel
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (geen titel)
calendar-field-calendar = Agenda
calendar-field-all-day = Hele dag
calendar-field-start = Begin
calendar-field-duration = Duur (min)
calendar-field-location = Locatie
calendar-field-shows-as = Weergeven als
calendar-shows-busy = Bezet
calendar-shows-free = Vrij
calendar-field-status = Status
calendar-status-confirmed = Bevestigd
calendar-status-tentative = Voorlopig
calendar-status-cancelled = Geannuleerd
calendar-repeats = Herhaling
calendar-frequency = Frequentie
calendar-freq-daily = Dagelijks
calendar-freq-weekly = Wekelijks
calendar-freq-monthly = Maandelijks
calendar-freq-yearly = Jaarlijks
calendar-every = elke
calendar-interval = Interval
calendar-ends = Eindigt
calendar-end-mode = Einde
calendar-end-never = Nooit
calendar-end-count = Na N keer
calendar-end-until = Op datum
calendar-occurrences = Aantal keer
calendar-until-date = Tot datum
calendar-reminders = Herinneringen
calendar-reminder-before = { $min } min van tevoren
calendar-remove-reminder = Herinnering { $min } verwijderen
calendar-add-reminder = Herinnering toevoegen
calendar-reminder-none = + herinnering
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 uur
calendar-reminder-1440 = 1 dag
calendar-attendees = Deelnemers
calendar-add-attendee = Deelnemer toevoegen
calendar-attendee-invalid = Dat is geen e-mailadres. Voer één adres in, zoals name@example.com.
calendar-save-failed = De afspraak is niet opgeslagen. De server heeft haar geweigerd.
calendar-remove-attendee = { $email } verwijderen
calendar-notes = Notities

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Rol van { $email }
calendar-attendee-cutype-for = Type van { $email }
calendar-role-chair = Voorzitter
calendar-role-required = Verplicht
calendar-role-optional = Optioneel
calendar-role-non-participant = Geen deelnemer
calendar-cutype-individual = Persoon
calendar-cutype-group = Groep
calendar-cutype-resource = Middel
calendar-cutype-room = Ruimte
calendar-partstat-needs-action = Geen antwoord
calendar-partstat-accepted = Geaccepteerd
calendar-partstat-declined = Geweigerd
calendar-partstat-tentative = Voorlopig

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = { $start } tot { $end }: { $title }
calendar-schedule-free = { $gap } vrij
calendar-gap-hm = { $h } u { $m } min
calendar-gap-h = { $h } u
calendar-gap-m = { $m } min

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = { $count ->
    [one] { $count } conflict oplossen
   *[other] { $count } conflicten oplossen
}
calendar-resolver-title = Conflicten oplossen
calendar-resolver-none = Geen conflicten om op te lossen.
calendar-resolver-pick = Conflict
calendar-resolver-pair-n = { $n } van { $total }: { $a } tegenover { $b }
calendar-resolver-time = Tijd
calendar-resolver-overlap = Overlap { $start } – { $end }
calendar-resolver-reschedule = Latere afspraak verplaatsen
calendar-resolver-shorten = Eerdere afspraak inkorten
calendar-resolver-tentative = Latere als voorlopig markeren
calendar-resolver-double-book = Dubbel boeken (als vrij markeren)
calendar-resolver-keep = Beide behouden
calendar-events-count-people = { $count ->
    [0] Geen deelnemers
    [one] { $count } deelnemer
   *[other] { $count } deelnemers
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Agenda's
calendar-fb-own = Van jou
calendar-fb-unavailable = Je bezette tijd kon niet worden geladen.
calendar-fb-cell = { $principal } om { $hour }: { $status }
calendar-fb-busy = Bezet
calendar-fb-tentative = Voorlopig
calendar-fb-free = Vrij

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Snel een afspraak toevoegen
calendar-quick-add-placeholder = Snel toevoegen, bijv. Lunch Friday 1pm
calendar-quick-add-btn = Toevoegen
calendar-quick-add-do = Snel toevoegen

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Categorieën
calendar-add-category = Categorie toevoegen
calendar-category-placeholder = Een categorie toevoegen
calendar-remove-category = Categorie { $name } verwijderen
calendar-filter-heading = Filter
calendar-filter-category = Filteren op categorie
calendar-filter-category-placeholder = Te tonen categorie

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Bijlagen
calendar-attachment-title = Naam van de bijlage
calendar-attachment-title-placeholder = Naam (optioneel)
calendar-attachment-uri = Link naar de bijlage
calendar-attachment-add = Bijlage toevoegen
calendar-remove-attachment = Bijlage { $name } verwijderen

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Abonneren via URL
calendar-subscribe-url = Agenda-URL
calendar-subscribe-add = Abonneren
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = De agenda op dat adres kon niet worden opgehaald en is daarom niet toegevoegd.
calendar-subscribe-hol-many-locations = Dit feestdagenbestand bevat meer dan één locatie. Download het, gebruik dan Importeren en kies de gewenste locaties.
calendar-subscribe-refresh = { $name } vernieuwen
calendar-subscribe-refresh-btn = Vernieuwen
calendar-subscribe-refresh-failed = De agendafeed kon niet worden opgehaald. De afspraken zijn ongewijzigd gelaten.
calendar-import-failed = Dat bestand kon niet als agenda worden gelezen. Er is niets geïmporteerd.
calendar-import-done = { $count ->
    [one] 1 afspraak geïmporteerd.
   *[other] { $count } afspraken geïmporteerd.
}
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Locaties kiezen om te importeren
calendar-import-locations-intro = Dit bestand bevat feestdagen voor { $count } locaties. Alleen de locaties die je selecteert, worden geïmporteerd.
calendar-import-locations-filter = Locaties filteren
calendar-import-locations-list = Locaties in het bestand
calendar-import-locations-none = Geen locatie komt overeen met het filter.
calendar-import-locations-picked = { $count ->
    [one] 1 locatie geselecteerd.
   *[other] { $count } locaties geselecteerd.
}
calendar-import-locations-import = Selectie importeren
calendar-export-failed = De agenda kon niet worden geëxporteerd.
calendar-quick-add-failed = Die afspraak kon niet worden toegevoegd.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Delen
calendar-share-for = { $name } delen
calendar-share-title = { $name } delen
calendar-share-intro = Mensen met wie je deelt, kunnen deze agenda zien. Geef lees- en schrijftoegang zodat ze afspraken kunnen toevoegen en bewerken.
calendar-share-people = Gedeeld met
calendar-share-empty = Deze agenda is met niemand gedeeld.
calendar-share-add = Een persoon toevoegen
calendar-share-new-access = Toegang voor de nieuwe persoon
calendar-share-access-for = Toegang voor { $principal }
calendar-share-remove = { $principal } verwijderen
calendar-share-read = Alleen lezen
calendar-share-readwrite = Lezen en schrijven
