# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Kalendarz
calendar-prev = Poprzedni
calendar-next = Następny
calendar-today = Dzisiaj
calendar-views = Widoki kalendarza
calendar-new-event = Nowe wydarzenie
calendar-import = Importuj
calendar-export = Eksportuj
calendar-import-file = Importuj plik kalendarza

# -- View names --------------------------------------------------------------
calendar-view-day = Dzień
calendar-view-3day = 3 dni
calendar-view-work-week = Tydzień roboczy
calendar-view-week = Tydzień
calendar-view-month = Miesiąc
calendar-view-tri-month = Kwartał
calendar-view-schedule = Harmonogram
calendar-view-agenda = Agenda
calendar-view-year = Rok

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Kalendarze
calendar-toggle = Przełącz { $name }
calendar-color-for = Kolor dla { $name }
calendar-add-calendar = Dodaj kalendarz
calendar-new-calendar-name = Nowy kalendarz
calendar-holidays-heading = Święta
calendar-subscribe-holidays = Subskrybuj święta
calendar-add-region = Dodaj region…
calendar-synced = Synchronizowany kalendarz

# -- Grid / views ------------------------------------------------------------
calendar-all-day = cały dzień
calendar-all-day-full = Cały dzień
calendar-more = jeszcze { $count }
calendar-conflict = konflikt
calendar-no-events-30 = Brak wydarzeń w ciągu najbliższych 30 dni.
calendar-events-count =
    { $count ->
        [0] Brak wydarzeń
        [one] { $count } wydarzenie
        [few] { $count } wydarzenia
        [many] { $count } wydarzeń
       *[other] { $count } wydarzenia
    }
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Miesiąc { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Siatka czasu
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }, { $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }, { $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = Cały dzień: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Nowe wydarzenie
calendar-editor-edit = Edytuj wydarzenie
calendar-invitation = Zaproszenie
calendar-invited = Otrzymano zaproszenie ({ $status }).
calendar-accept = Akceptuj
calendar-tentative = Wstępnie
calendar-decline = Odrzuć
calendar-counter = Zaproponuj zmianę…
calendar-propose-start = Zaproponuj nowy początek
calendar-send-counter = Wyślij propozycję
calendar-overlaps =
    To wydarzenie nakłada się na { $count ->
        [one] { $count } inne wydarzenie
        [few] { $count } inne wydarzenia
        [many] { $count } innych wydarzeń
       *[other] { $count } innego wydarzenia
    }.
calendar-field-title = Tytuł
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (bez tytułu)
calendar-field-calendar = Kalendarz
calendar-field-all-day = Cały dzień
calendar-field-start = Początek
calendar-field-duration = Czas trwania (min)
calendar-field-location = Miejsce
calendar-field-shows-as = Pokazuj jako
calendar-shows-busy = Zajęty
calendar-shows-free = Wolny
calendar-field-status = Status
calendar-status-confirmed = Potwierdzone
calendar-status-tentative = Wstępne
calendar-status-cancelled = Odwołane
calendar-repeats = Powtarzanie
calendar-frequency = Częstotliwość
calendar-freq-daily = Codziennie
calendar-freq-weekly = Co tydzień
calendar-freq-monthly = Co miesiąc
calendar-freq-yearly = Co rok
calendar-every = co
calendar-interval = Interwał
calendar-ends = Koniec
calendar-end-mode = Sposób zakończenia
calendar-end-never = Nigdy
calendar-end-count = Po N razach
calendar-end-until = W dniu
calendar-occurrences = Liczba wystąpień
calendar-until-date = Data zakończenia
calendar-reminders = Przypomnienia
calendar-reminder-before = { $min } min przed
calendar-remove-reminder = Usuń przypomnienie { $min }
calendar-add-reminder = Dodaj przypomnienie
calendar-reminder-none = + przypomnienie
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 godzina
calendar-reminder-1440 = 1 dzień
calendar-attendees = Uczestnicy
calendar-add-attendee = Dodaj uczestnika
calendar-attendee-invalid = To nie jest adres e-mail. Wpisz jeden adres, np. name@example.com.
calendar-save-failed = Wydarzenie nie zostało zapisane. Serwer je odrzucił.
calendar-remove-attendee = Usuń { $email }
calendar-notes = Notatki

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Rola dla { $email }
calendar-attendee-cutype-for = Typ dla { $email }
calendar-role-chair = Prowadzący
calendar-role-required = Wymagany
calendar-role-optional = Opcjonalny
calendar-role-non-participant = Nieuczestniczący
calendar-cutype-individual = Osoba
calendar-cutype-group = Grupa
calendar-cutype-resource = Zasób
calendar-cutype-room = Sala
calendar-partstat-needs-action = Brak odpowiedzi
calendar-partstat-accepted = Zaakceptowano
calendar-partstat-declined = Odrzucono
calendar-partstat-tentative = Wstępnie

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = Od { $start } do { $end }: { $title }
calendar-schedule-free = { $gap } wolnego
calendar-gap-hm = { $h } godz. { $m } min
calendar-gap-h = { $h } godz.
calendar-gap-m = { $m } min

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts =
    Rozwiąż { $count ->
        [one] { $count } konflikt
        [few] { $count } konflikty
        [many] { $count } konfliktów
       *[other] { $count } konfliktu
    }
calendar-resolver-title = Rozwiąż konflikty
calendar-resolver-none = Brak konfliktów do rozwiązania.
calendar-resolver-pick = Konflikt
calendar-resolver-pair-n = { $n } z { $total }: { $a } a { $b }
calendar-resolver-time = Czas
calendar-resolver-overlap = Nakładanie się { $start } – { $end }
calendar-resolver-reschedule = Przełóż późniejsze wydarzenie
calendar-resolver-shorten = Skróć wcześniejsze wydarzenie
calendar-resolver-tentative = Oznacz późniejsze jako wstępne
calendar-resolver-double-book = Zarezerwuj podwójnie (oznacz jako wolny)
calendar-resolver-keep = Zachowaj oba
calendar-events-count-people =
    { $count ->
        [0] Brak uczestników
        [one] { $count } uczestnik
        [few] { $count } uczestników
        [many] { $count } uczestników
       *[other] { $count } uczestnika
    }

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Kalendarze
calendar-fb-own = Twoje
calendar-fb-unavailable = Nie udało się wczytać Twojego zajętego czasu.
calendar-fb-cell = { $principal } o { $hour }: { $status }
calendar-fb-busy = Zajęty
calendar-fb-tentative = Wstępnie
calendar-fb-free = Wolny

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Szybkie dodawanie wydarzenia
calendar-quick-add-placeholder = Szybkie dodawanie, np. Lunch piątek 13:00
calendar-quick-add-btn = Dodaj
calendar-quick-add-do = Dodaj szybko

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Kategorie
calendar-add-category = Dodaj kategorię
calendar-category-placeholder = Dodaj kategorię
calendar-remove-category = Usuń kategorię { $name }
calendar-filter-heading = Filtr
calendar-filter-category = Filtruj według kategorii
calendar-filter-category-placeholder = Kategoria do wyświetlenia

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Załączniki
calendar-attachment-title = Nazwa załącznika
calendar-attachment-title-placeholder = Nazwa (opcjonalnie)
calendar-attachment-uri = Link do załącznika
calendar-attachment-add = Dodaj załącznik
calendar-remove-attachment = Usuń załącznik { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Subskrybuj przez URL
calendar-subscribe-url = URL kalendarza
calendar-subscribe-add = Subskrybuj
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Nie udało się pobrać kalendarza spod tego adresu, więc nie został dodany.
calendar-subscribe-hol-many-locations = Ten plik świąt zawiera więcej niż jedną lokalizację. Pobierz go, a następnie użyj opcji Importuj i wybierz potrzebne lokalizacje.
calendar-subscribe-refresh = Odśwież { $name }
calendar-subscribe-refresh-btn = Odśwież
calendar-subscribe-refresh-failed = Nie udało się pobrać kanału kalendarza. Jego wydarzenia pozostały bez zmian.
calendar-import-failed = Nie udało się odczytać tego pliku jako kalendarza. Niczego nie zaimportowano.
calendar-import-done =
    { $count ->
        [one] Zaimportowano 1 wydarzenie.
        [few] Zaimportowano { $count } wydarzenia.
        [many] Zaimportowano { $count } wydarzeń.
       *[other] Zaimportowano { $count } wydarzenia.
    }
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Wybierz lokalizacje do zaimportowania
calendar-import-locations-intro = Ten plik zawiera święta dla wielu lokalizacji (liczba: { $count }). Importowane są tylko wybrane lokalizacje.
calendar-import-locations-filter = Filtruj lokalizacje
calendar-import-locations-list = Lokalizacje w pliku
calendar-import-locations-none = Żadna lokalizacja nie pasuje do filtru.
calendar-import-locations-picked =
    { $count ->
        [one] Wybrano 1 lokalizację.
        [few] Wybrano { $count } lokalizacje.
        [many] Wybrano { $count } lokalizacji.
       *[other] Wybrano { $count } lokalizacji.
    }
calendar-import-locations-import = Importuj wybrane
calendar-export-failed = Nie udało się wyeksportować kalendarza.
calendar-quick-add-failed = Nie udało się dodać tego wydarzenia.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Udostępnij
calendar-share-for = Udostępnij { $name }
calendar-share-title = Udostępnij { $name }
calendar-share-intro = Osoby, którym udostępnisz kalendarz, mogą go wyświetlać. Przyznaj dostęp do odczytu i zapisu, aby mogły dodawać i edytować wydarzenia.
calendar-share-people = Udostępniono
calendar-share-empty = Ten kalendarz nie jest nikomu udostępniony.
calendar-share-add = Dodaj osobę
calendar-share-new-access = Dostęp dla nowej osoby
calendar-share-access-for = Dostęp dla { $principal }
calendar-share-remove = Usuń { $principal }
calendar-share-read = Tylko odczyt
calendar-share-readwrite = Odczyt i zapis
