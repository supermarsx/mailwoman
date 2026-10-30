# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Odśwież
admin-dash-unavailable = Dane pulpitu nie są jeszcze dostępne.
admin-dash-unavailable-detail = Ten serwer nie udostępnia danych pulpitu (GET /admin/dashboard zwraca 404). Pojawią się w późniejszej aktualizacji serwera. Pozostałe sekcje działają jak dotychczas.
admin-dash-error-network = Nie udało się wczytać pulpitu: serwer nie odpowiedział. Spróbuj ponownie.
admin-dash-error-status = Nie udało się wczytać pulpitu: serwer zwrócił { $status }. Spróbuj ponownie; jeśli błąd się powtórzy, sprawdź dziennik serwera.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Wymaga uwagi
admin-dash-warn-fix = Otwórz sekcję, w której można to naprawić
admin-dash-warn-blocked-accounts =
    { $count ->
        [one] Jedno konto nie może się zalogować, ponieważ jego domena nie ma konfiguracji.
        [few] { $count } konta nie mogą się zalogować, ponieważ ich domeny nie mają konfiguracji.
        [many] { $count } kont nie może się zalogować, ponieważ ich domeny nie mają konfiguracji.
       *[other] { $count } konta nie może się zalogować, ponieważ ich domeny nie mają konfiguracji.
    }
admin-dash-warn-no-trusted-proxy = Reguły IP są aktywne, ale nie skonfigurowano zaufanego serwera proxy, więc każdy klient może wyglądać, jakby łączył się z adresu serwera proxy.
admin-dash-warn-allowlist-bypass = Zmienna środowiskowa wyłącza listę dozwolonych adresów IP.
admin-dash-warn-metrics-token = Punkt końcowy metryk nie ma ustawionego tokenu.
admin-dash-warn-pending-migrations =
    { $count ->
        [one] Jedna migracja bazy danych nie została zastosowana. Uruchom ponownie serwer, aby ją zastosować.
        [few] { $count } migracje bazy danych nie zostały zastosowane. Uruchom ponownie serwer, aby je zastosować.
        [many] { $count } migracji bazy danych nie zostało zastosowanych. Uruchom ponownie serwer, aby je zastosować.
       *[other] { $count } migracji bazy danych nie zostało zastosowanych. Uruchom ponownie serwer, aby je zastosować.
    }
admin-dash-warn-cert-expiring =
    { $count ->
        [one] Jeden certyfikat serwera nadrzędnego wkrótce wygasa (według ostatniego testu połączenia).
        [few] { $count } certyfikaty serwerów nadrzędnych wkrótce wygasają (według ostatniego testu połączenia).
        [many] { $count } certyfikatów serwerów nadrzędnych wkrótce wygasa (według ostatniego testu połączenia).
       *[other] { $count } certyfikatu serwerów nadrzędnych wkrótce wygasa (według ostatniego testu połączenia).
    }
admin-dash-warn-unknown = Ostrzeżenie serwera { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title =
    { $count ->
        [one] Jedno konto zablokowane: brak konfiguracji domeny
        [few] { $count } konta zablokowane: brak konfiguracji domeny
        [many] { $count } kont zablokowanych: brak konfiguracji domeny
       *[other] { $count } konta zablokowane: brak konfiguracji domeny
    }
admin-dash-blocked-note = Te konta logowały się przed tą aktualizacją, ale ich domena nie ma konfiguracji w konsoli, więc teraz nie mogą się zalogować. Skonfigurowanie domeny pozwoli im ponownie się logować.
admin-dash-blocked-chart = Zablokowane konta według domeny
admin-dash-blocked-row =
    { $count ->
        [one] jedno konto
        [few] { $count } konta
        [many] { $count } kont
       *[other] { $count } konta
    }
admin-dash-blocked-seen = (ostatnio używany host poczty { $host })
admin-dash-configure-domain = Skonfiguruj { $domain }
admin-dash-blocked-no-access = Twoja rola nie pozwala konfigurować domen.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist =
    { $open ->
        [0] Lista kontrolna konfiguracji: nic do zrobienia
        [one] Lista kontrolna konfiguracji: jedna otwarta pozycja
        [few] Lista kontrolna konfiguracji: { $open } otwarte pozycje
        [many] Lista kontrolna konfiguracji: { $open } otwartych pozycji
       *[other] Lista kontrolna konfiguracji: { $open } otwartej pozycji
    }
admin-dash-check-done = Gotowe
admin-dash-check-open = Otwarte
admin-dash-check-optional = (opcjonalne)
admin-dash-check-go = Otwórz
admin-dash-check-hide = Ukryj
admin-dash-check-unhide = Pokaż ponownie
admin-dash-check-hidden =
    { $count ->
        [one] Jedna pozycja jest ukryta. Ukrycie nie oznacza jej jako wykonanej.
        [few] { $count } pozycje są ukryte. Ukrycie nie oznacza ich jako wykonanych.
        [many] { $count } pozycji jest ukrytych. Ukrycie nie oznacza ich jako wykonanych.
       *[other] { $count } pozycji jest ukrytych. Ukrycie nie oznacza ich jako wykonanych.
    }
admin-dash-check-show-hidden = Pokaż ukryte pozycje
admin-dash-check-hide-hidden = Pomiń ukryte pozycje
admin-dash-check-domain-exists = Dodaj domenę pocztową
admin-dash-check-domain-upstream = Przypisz domenie serwer pocztowy lub ustaw domyślny serwer nadrzędny
admin-dash-check-test-connection = Wykonaj udany test połączenia (w ciągu ostatnich 7 dni)
admin-dash-check-preferred-domain = Wybierz preferowaną domenę, aby użytkownicy mogli się logować bez jej wpisywania
admin-dash-check-second-admin = Utwórz drugie konto administratora lub wyłącz logowanie przez zmienne środowiskowe
admin-dash-check-trusted-proxies = Skonfiguruj zaufane serwery proxy działające przed tym serwerem
admin-dash-check-security-policy = Zapisz politykę bezpieczeństwa
admin-dash-check-metrics-token = Ustaw token dla punktu końcowego metryk
admin-dash-check-branding = Przejrzyj identyfikację wizualną

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Konta i sesje
admin-dash-users = Utworzeni użytkownicy
admin-dash-signed-in-24h = Konta zalogowane w ciągu ostatnich 24 godzin
admin-dash-signed-in-7d = Konta zalogowane w ciągu ostatnich 7 dni
admin-dash-domains = Domeny
admin-dash-domains-detail = Z serwerem pocztowym: { $upstream }, wyłączone: { $disabled }
admin-dash-sessions = Aktywne sesje poczty webowej
admin-dash-admin-sessions = Aktywne sesje administratorów
admin-dash-security = Bezpieczeństwo logowania
admin-dash-failed-webmail = Nieudane logowania do poczty webowej, ostatnie 24 godziny
admin-dash-failed-admin = Nieudane logowania administratorów, ostatnie 24 godziny
admin-dash-active-blocks = Zablokowane adresy
admin-dash-allowlists = Listy dozwolonych IP
admin-dash-allowlists-value = Administracja: { $admin }, poczta webowa: { $webmail }
admin-dash-on = wł.
admin-dash-off = wył.
admin-dash-trusted-proxies = Zaufane serwery proxy
admin-dash-configured = Skonfigurowano
admin-dash-not-configured = Nie skonfigurowano
admin-dash-untrusted-forwarded =
    { $count ->
        [one] Jedno żądanie w ciągu 24 godzin zawierało nagłówki przekazywania od niezaufanego nadawcy
        [few] { $count } żądania w ciągu 24 godzin zawierały nagłówki przekazywania od niezaufanego nadawcy
        [many] { $count } żądań w ciągu 24 godzin zawierało nagłówki przekazywania od niezaufanego nadawcy
       *[other] { $count } żądania w ciągu 24 godzin zawierało nagłówki przekazywania od niezaufanego nadawcy
    }

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Ruch, ostatnie { $mins } min
admin-dash-traffic-note = Zliczane przez ten proces serwera od jego uruchomienia. Inne procesy ani repliki nie są uwzględniane.
admin-dash-traffic-empty = W tym przedziale nie zliczono żadnych żądań.
admin-dash-requests = Żądania na minutę
admin-dash-errors = Błędy serwera (5xx) na minutę
admin-dash-p95 = Czas odpowiedzi, 95. percentyl

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Stan
admin-dash-h-ok = OK
admin-dash-h-attention = Wymaga uwagi
admin-dash-h-db = Baza danych
admin-dash-h-migrations = Migracje
admin-dash-h-engine = Synchronizacja poczty
admin-dash-h-outbox = Skrzynka nadawcza
admin-dash-h-index = Indeks wyszukiwania
admin-dash-h-sandbox = Piaskownica renderowania
admin-dash-h-plugins = Środowisko wtyczek

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Pamięć
admin-dash-db-size = Plik bazy danych
admin-dash-wal-size = Dziennik zapisu z wyprzedzeniem (WAL)
admin-dash-index-size = Indeks wyszukiwania
admin-dash-volume-free = Wolne miejsce na woluminie danych
admin-dash-volume-of = z { $total }
admin-dash-na-wal = Ta baza danych nie używa pliku dziennika zapisu z wyprzedzeniem lub serwer tego nie zgłosił.
admin-dash-na-volume = Serwer nie może odczytać wolnego miejsca na tej platformie.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Ostatnie działania administratorów
admin-dash-audit-empty = Nie zarejestrowano jeszcze żadnych działań administratorów.
admin-dash-audit-when = Kiedy
admin-dash-audit-actor = Kto
admin-dash-audit-action = Działanie
admin-dash-audit-target = Obiekt
admin-dash-audit-all = Otwórz pełny dziennik audytu
admin-dash-server = Serwer
admin-dash-version = Wersja
admin-dash-build = Kompilacja
admin-dash-mode = Tryb
admin-dash-mode-engine = Silnik: ten serwer synchronizuje i przechowuje pocztę
admin-dash-mode-proxy = Proxy: ten serwer przekazuje żądania do serwera pocztowego
admin-dash-uptime = Czas działania
admin-dash-base-path = Udostępniany pod
admin-dash-base-root = katalog główny witryny
admin-dash-duration-dh = { $d } d { $h } godz.
admin-dash-duration-hm = { $h } godz. { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
