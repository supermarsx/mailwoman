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

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Datumkiezer
calendar-event-range = { $title }, { $start } tot { $end }
calendar-busy-free = Wordt weergegeven als vrij
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = Nog { $count } { $count ->
    [one] afspraak
   *[other] afspraken
  } op { $date }
calendar-fb-caption-people = Bezette tijd op de dag van het conflict, per uur, voor u en elke uitgenodigde persoon.
calendar-fb-unknown = Beschikbaarheid van { $principal } kon niet worden opgehaald.
calendar-fb-loading = Beschikbaarheid laden…
calendar-import-itip-message = Dit bestand is een antwoord of annulering die per e-mail is verzonden. Open het bijbehorende bericht om het toe te passen. Er is niets geïmporteerd.
calendar-import-skipped = { $count ->
    [one] { $count } afspraak stond al in de agenda en is ongewijzigd gelaten.
   *[other] { $count } afspraken stonden al in de agenda en zijn ongewijzigd gelaten.
 }
calendar-invite-title = Agenda-uitnodiging
calendar-invite-sender = Van { $sender }
calendar-invite-add = Aan agenda toevoegen
calendar-invite-update = Afspraakupdate toepassen
calendar-invite-cancel = Annulering toepassen
calendar-invite-reply = Antwoord van deelnemer vastleggen
calendar-invite-new = Deze uitnodiging kan aan uw agenda worden toegevoegd.
calendar-invite-newer = De organisator heeft een bijgewerkte afspraak gestuurd.
calendar-invite-current = Uw agenda bevat al deze versie.
calendar-invite-stale = Dit bericht is ouder dan de afspraak in uw agenda.
calendar-invite-cancelled = De organisator heeft deze afspraak geannuleerd.
calendar-invite-response = Een deelnemer heeft een antwoord gestuurd.
calendar-invite-not-organizer = De afzender is niet de opgeslagen organisator. Er wordt niets gewijzigd.
calendar-invite-not-attendee = De afzender is geen overeenkomende deelnemer. Er wordt niets gewijzigd.
calendar-invite-unknown = Dit bericht verwijst naar een afspraak die niet in uw agenda staat.
calendar-invite-unsupported = Dit voorstel kan niet automatisch worden toegepast.
calendar-invite-no-reply = Hiermee wordt uw agenda bijgewerkt zonder een antwoord te sturen.
calendar-invite-applied = Uw agenda is bijgewerkt.
calendar-invite-unchanged = De afspraak is niet gewijzigd. Controleer de huidige status van de uitnodiging.
calendar-invite-apply-failed = De uitnodiging kon niet worden toegepast. Probeer het opnieuw.
calendar-invite-load-failed = De uitnodiging kon niet worden gecontroleerd.
calendar-invite-retry = Opnieuw proberen
calendar-field-time-zone = Tijdzone van de afspraak
calendar-field-time-zone-help = Gebruik een IANA-naam zoals Europe/Lisbon. De afspraak behoudt deze zone wanneer u hem elders bekijkt.
calendar-time-invalid = Voer een geldige tijdzone en lokale tijd in. Sommige lokale tijden bestaan niet wanneer de klok vooruit wordt gezet.
calendar-mini-prev = Vorige maand
calendar-mini-next = Volgende maand
calendar-card-region-event = Agenda-afspraak
calendar-card-kind-request = Vergaderuitnodiging
calendar-card-kind-update = Bijgewerkte uitnodiging
calendar-card-kind-cancel = Geannuleerde vergadering
calendar-card-kind-reply = Antwoord op vergadering
calendar-card-kind-counter = Nieuw tijdstip voorgesteld
calendar-card-kind-publish = { $count ->
    [one] Agenda-afspraak
   *[other] { $count } agenda-afspraken
 }
calendar-card-when = Wanneer
calendar-card-where = Waar
calendar-card-organizer = Organisator
calendar-card-attendees = Deelnemers
calendar-card-all-day = { $when } (hele dag)
calendar-card-organizer-time = Tijd van de organisator: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Elke dag
   *[other] Elke { $interval } dagen
 }
calendar-card-repeat-weekly = { $interval ->
    [one] Elke week
   *[other] Elke { $interval } weken
 }
calendar-card-repeat-weekly-on = { $interval ->
    [one] Elke week op { $days }
   *[other] Elke { $interval } weken op { $days }
 }
calendar-card-repeat-monthly = { $interval ->
    [one] Elke maand
   *[other] Elke { $interval } maanden
 }
calendar-card-repeat-monthly-day = { $interval ->
    [one] Elke maand op dag { $day }
   *[other] Elke { $interval } maanden op dag { $day }
 }
calendar-card-repeat-yearly = { $interval ->
    [one] Elk jaar
   *[other] Elke { $interval } jaar
 }
calendar-card-repeat-until = { $rule } tot { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] eenmaal
   *[other] { $count } keer
 }
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 datum overgeslagen
   *[other] { $count } datums overgeslagen
 })
calendar-card-repeat-custom = Herhaalt volgens een aangepast schema
calendar-card-join = Deelnemen via { $provider }
calendar-card-organizer-is = Georganiseerd door { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 deelnemer
   *[other] { $count } deelnemers
 }
calendar-card-people-all = Alle { $count } tonen
calendar-card-people-fewer = Minder tonen
calendar-card-rsvp-accepted = Geaccepteerd
calendar-card-rsvp-tentative = Onder voorbehoud
calendar-card-rsvp-declined = Afgewezen
calendar-card-rsvp-none = Nog geen antwoord
calendar-card-in-calendar = In uw agenda
calendar-card-cancelled = Geannuleerd
calendar-card-open = Openen in agenda
calendar-card-agenda = Uw agenda op { $day }
calendar-card-agenda-title = Uw dag
calendar-card-agenda-this-event = Deze afspraak
calendar-card-agenda-unavailable = Uw agenda voor deze dag kon niet worden geladen.
calendar-card-conflicts = { $count ->
    [one] Conflicteert met { $titles }
   *[other] Conflicteert met { $count } afspraken: { $titles }
 }
calendar-card-no-conflicts = Geen conflicten in uw agenda
calendar-card-invited-as = U bent uitgenodigd als { $address }.
calendar-card-not-invited = Deze uitnodiging vermeldt geen van uw adressen. U kunt hem aan uw agenda toevoegen, maar niet beantwoorden.
calendar-card-not-invited-short = Deze uitnodiging vermeldt geen van uw adressen.
calendar-card-out-of-date = Deze uitnodiging is verouderd. Er staat al een nieuwere versie in uw agenda.
calendar-card-replied = { $who } { $answer ->
    [accepted] heeft geaccepteerd.
    [tentative] heeft onder voorbehoud geaccepteerd.
    [declined] heeft afgewezen.
   *[other] heeft nog niet geantwoord.
 }
calendar-card-counter = { $who } heeft een nieuw tijdstip voorgesteld. Voorstellen worden niet automatisch toegepast.
calendar-card-todos-skipped = { $count ->
    [one] Het bestand bevat ook 1 taak, die niet wordt toegevoegd.
   *[other] Het bestand bevat ook { $count } taken, die niet worden toegevoegd.
 }
calendar-card-respond = Op de uitnodiging reageren
calendar-card-accept = Accepteren
calendar-card-tentative = Onder voorbehoud
calendar-card-decline = Afwijzen
calendar-card-propose = Nieuw tijdstip voorstellen
calendar-card-propose-start = Nieuwe starttijd
calendar-card-propose-send = Voorstel verzenden
calendar-card-propose-invalid = Voer een geldige nieuwe starttijd in.
calendar-card-send-response = Een antwoord sturen naar { $organizer }
calendar-card-comment = Opmerking (optioneel)
calendar-card-calendar = Agenda
calendar-card-events = Afspraken in dit bestand
calendar-card-add-one = Toevoegen
calendar-card-add-all = Alles toevoegen ({ $count })
calendar-card-added = { $count ->
    [one] Aan uw agenda toegevoegd.
   *[other] { $count } afspraken aan uw agenda toegevoegd.
 }
calendar-card-remove = Uit agenda verwijderen
calendar-card-removed = Uit uw agenda verwijderd.
calendar-card-record-reply = Status van deelnemer bijwerken
calendar-card-answered-sent = { $answer }. Uw antwoord is verzonden naar { $organizer }.
calendar-card-answered-not-sent = { $answer }. Uw agenda is bijgewerkt, maar het antwoord kon niet worden verzonden.
calendar-card-answered-silently = { $answer }. Er is geen antwoord verzonden.
calendar-card-proposal-sent = Uw voorstel is verzonden naar { $organizer }.
calendar-card-not-sent = Het bericht kon niet worden verzonden.
calendar-card-more = { $count ->
    [one] nog 1
   *[other] nog { $count }
 }
