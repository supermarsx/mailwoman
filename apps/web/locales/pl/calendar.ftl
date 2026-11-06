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

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Nawigator dat
calendar-event-range = { $title }, od { $start } do { $end }
calendar-busy-free = Wyświetla się jako wolny
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = Jeszcze { $count } { $count ->
    [one] wydarzenie
    [few] wydarzenia
    [many] wydarzeń
   *[other] wydarzenia
  } – { $date }
calendar-fb-caption-people = Zajęty czas w dniu konfliktu, godzina po godzinie, dla Ciebie i każdej zaproszonej osoby.
calendar-fb-unknown = Nie udało się pobrać dostępności dla: { $principal }.
calendar-fb-loading = Wczytywanie dostępności…
calendar-import-itip-message = Ten plik to odpowiedź lub anulowanie wysłane e-mailem. Otwórz wiadomość, z którą przyszedł, aby je zastosować. Nic nie zaimportowano.
calendar-import-skipped = { $count ->
    [one] { $count } wydarzenie już było w kalendarzu i zostało bez zmian.
    [few] { $count } wydarzenia już były w kalendarzu i zostały bez zmian.
    [many] { $count } wydarzeń już było w kalendarzu i zostało bez zmian.
   *[other] { $count } wydarzenia już było w kalendarzu i zostało bez zmian.
}
calendar-invite-title = Zaproszenie do kalendarza
calendar-invite-sender = Od: { $sender }
calendar-invite-add = Dodaj do kalendarza
calendar-invite-update = Zastosuj aktualizację wydarzenia
calendar-invite-cancel = Zastosuj anulowanie
calendar-invite-reply = Zapisz odpowiedź uczestnika
calendar-invite-new = To zaproszenie można dodać do Twojego kalendarza.
calendar-invite-newer = Organizator wysłał zaktualizowane wydarzenie.
calendar-invite-current = Twój kalendarz zawiera już tę wersję.
calendar-invite-stale = Ta wiadomość jest starsza niż wydarzenie w Twoim kalendarzu.
calendar-invite-cancelled = Organizator anulował to wydarzenie.
calendar-invite-response = Uczestnik wysłał odpowiedź.
calendar-invite-not-organizer = Nadawca nie jest zapisanym organizatorem. Nic nie zostanie zmienione.
calendar-invite-not-attendee = Nadawca nie jest pasującym uczestnikiem. Nic nie zostanie zmienione.
calendar-invite-unknown = Ta wiadomość dotyczy wydarzenia, którego nie ma w Twoim kalendarzu.
calendar-invite-unsupported = Tej propozycji nie można zastosować automatycznie.
calendar-invite-no-reply = To aktualizuje Twój kalendarz bez wysyłania odpowiedzi.
calendar-invite-applied = Kalendarz został zaktualizowany.
calendar-invite-unchanged = Wydarzenie nie zostało zmienione. Sprawdź bieżący stan zaproszenia.
calendar-invite-apply-failed = Nie udało się zastosować zaproszenia. Spróbuj ponownie.
calendar-invite-load-failed = Nie udało się sprawdzić zaproszenia.
calendar-invite-retry = Spróbuj ponownie
calendar-field-time-zone = Strefa czasowa wydarzenia
calendar-field-time-zone-help = Użyj nazwy IANA, np. Europe/Lisbon. Wydarzenie zachowuje tę strefę, gdy jest wyświetlane gdzie indziej.
calendar-time-invalid = Podaj prawidłową strefę czasową i czas lokalny. Niektóre czasy lokalne nie istnieją podczas przestawiania zegarów do przodu.
calendar-mini-prev = Poprzedni miesiąc
calendar-mini-next = Następny miesiąc
calendar-card-region-event = Wydarzenie w kalendarzu
calendar-card-kind-request = Zaproszenie na spotkanie
calendar-card-kind-update = Zaktualizowane zaproszenie
calendar-card-kind-cancel = Anulowane spotkanie
calendar-card-kind-reply = Odpowiedź na spotkanie
calendar-card-kind-counter = Zaproponowano nowy termin
calendar-card-kind-publish = { $count ->
    [one] Wydarzenie w kalendarzu
    [few] { $count } wydarzenia w kalendarzu
    [many] { $count } wydarzeń w kalendarzu
   *[other] { $count } wydarzenia w kalendarzu
}
calendar-card-when = Kiedy
calendar-card-where = Gdzie
calendar-card-organizer = Organizator
calendar-card-attendees = Uczestnicy
calendar-card-all-day = { $when } (cały dzień)
calendar-card-organizer-time = Czas organizatora: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Codziennie
    [few] Co { $interval } dni
    [many] Co { $interval } dni
   *[other] Co { $interval } dni
}
calendar-card-repeat-weekly = { $interval ->
    [one] Co tydzień
    [few] Co { $interval } tygodnie
    [many] Co { $interval } tygodni
   *[other] Co { $interval } tygodnia
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] Co tydzień: { $days }
    [few] Co { $interval } tygodnie: { $days }
    [many] Co { $interval } tygodni: { $days }
   *[other] Co { $interval } tygodnia: { $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] Co miesiąc
    [few] Co { $interval } miesiące
    [many] Co { $interval } miesięcy
   *[other] Co { $interval } miesiąca
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] Co miesiąc, dnia { $day }
    [few] Co { $interval } miesiące, dnia { $day }
    [many] Co { $interval } miesięcy, dnia { $day }
   *[other] Co { $interval } miesiąca, dnia { $day }
}
calendar-card-repeat-yearly = { $interval ->
    [one] Co roku
    [few] Co { $interval } lata
    [many] Co { $interval } lat
   *[other] Co { $interval } roku
}
calendar-card-repeat-until = { $rule } do { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] jeden raz
    [few] { $count } razy
    [many] { $count } razy
   *[other] { $count } raza
}
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] pominięto 1 datę
    [few] pominięto { $count } daty
    [many] pominięto { $count } dat
   *[other] pominięto { $count } daty
})
calendar-card-repeat-custom = Powtarza się według własnego harmonogramu
calendar-card-join = Dołącz: { $provider }
calendar-card-organizer-is = Organizator: { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 uczestnik
    [few] { $count } uczestników
    [many] { $count } uczestników
   *[other] { $count } uczestnika
} · zaakceptowało: { $accepted }, wstępnie: { $tentative }, odrzuciło: { $declined }
calendar-card-people-all = Pokaż wszystkich ({ $count })
calendar-card-people-fewer = Pokaż mniej
calendar-card-rsvp-accepted = Zaakceptowano
calendar-card-rsvp-tentative = Wstępnie
calendar-card-rsvp-declined = Odrzucono
calendar-card-rsvp-none = Brak odpowiedzi
calendar-card-in-calendar = W Twoim kalendarzu
calendar-card-cancelled = Anulowano
calendar-card-open = Otwórz w kalendarzu
calendar-card-agenda = Twój kalendarz na { $day }
calendar-card-agenda-title = Twój dzień
calendar-card-agenda-this-event = To wydarzenie
calendar-card-agenda-unavailable = Nie udało się wczytać Twojego kalendarza na ten dzień.
calendar-card-conflicts = { $count ->
    [one] Koliduje z: { $titles }
    [few] Koliduje z { $count } wydarzeniami: { $titles }
    [many] Koliduje z { $count } wydarzeniami: { $titles }
   *[other] Koliduje z { $count } wydarzeniami: { $titles }
}
calendar-card-no-conflicts = Brak kolizji w Twoim kalendarzu
calendar-card-invited-as = Zaproszono Cię jako { $address }.
calendar-card-not-invited = To zaproszenie nie zawiera żadnego z Twoich adresów. Możesz dodać je do kalendarza, ale nie możesz na nie odpowiedzieć.
calendar-card-not-invited-short = To zaproszenie nie zawiera żadnego z Twoich adresów.
calendar-card-out-of-date = To zaproszenie jest nieaktualne. Nowsza wersja jest już w Twoim kalendarzu.
calendar-card-counter = Nowy termin zaproponowany przez: { $who }. Propozycje nie są stosowane automatycznie.
calendar-card-todos-skipped = { $count ->
    [one] Plik zawiera też 1 zadanie, które nie zostanie dodane.
    [few] Plik zawiera też { $count } zadania, które nie zostaną dodane.
    [many] Plik zawiera też { $count } zadań, które nie zostaną dodane.
   *[other] Plik zawiera też { $count } zadania, które nie zostaną dodane.
}
calendar-card-respond = Odpowiedz na zaproszenie
calendar-card-accept = Zaakceptuj
calendar-card-tentative = Wstępnie
calendar-card-decline = Odrzuć
calendar-card-propose = Zaproponuj nowy termin
calendar-card-propose-start = Nowy początek
calendar-card-propose-send = Wyślij propozycję
calendar-card-propose-invalid = Podaj prawidłowy nowy czas rozpoczęcia.
calendar-card-send-response = Wyślij odpowiedź do: { $organizer }
calendar-card-comment = Komentarz (opcjonalnie)
calendar-card-calendar = Kalendarz
calendar-card-events = Wydarzenia w tym pliku
calendar-card-add-one = Dodaj
calendar-card-add-all = Dodaj wszystkie ({ $count })
calendar-card-added = { $count ->
    [one] Dodano do kalendarza.
    [few] Dodano do kalendarza { $count } wydarzenia.
    [many] Dodano do kalendarza { $count } wydarzeń.
   *[other] Dodano do kalendarza { $count } wydarzenia.
}
calendar-card-remove = Usuń z kalendarza
calendar-card-removed = Usunięto z Twojego kalendarza.
calendar-card-record-reply = Zaktualizuj stan uczestnika
calendar-card-answered-sent = { $answer }. Twoja odpowiedź została wysłana do: { $organizer }.
calendar-card-answered-not-sent = { $answer }. Kalendarz został zaktualizowany, ale nie udało się wysłać odpowiedzi.
calendar-card-answered-silently = { $answer }. Nie wysłano odpowiedzi.
calendar-card-proposal-sent = Twoja propozycja została wysłana do: { $organizer }.
calendar-card-not-sent = Nie udało się wysłać wiadomości.
calendar-card-more = { $count ->
    [one] 1 więcej
    [few] { $count } więcej
    [many] { $count } więcej
   *[other] { $count } więcej
}

calendar-card-replied = Odpowiedź od { $who }: { $answer ->
    [accepted] zaakceptowano.
    [tentative] wstępnie zaakceptowano.
    [declined] odrzucono.
   *[other] jeszcze nie odpowiedziano.
}
