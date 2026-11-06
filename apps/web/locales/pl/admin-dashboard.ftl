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

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = Zasada dotycząca krajów jest aktywna, ale nie wczytano żadnej użytecznej bazy krajów, więc objęte nią logowania są odrzucane.
admin-dash-warn-geoip-stale = Zasada dotycząca krajów jest aktywna, a baza krajów jest nieaktualna.
admin-dash-h-status-ok = OK
admin-dash-h-status-warn = Ostrzeżenie
admin-dash-h-status-fail = Błąd
admin-dash-h-status-disabled = Wyłączone
admin-dash-h-checked = Sprawdzono o { $time }
admin-dash-h-refresh = Sprawdź ponownie
admin-dash-h-refresh-failed = Nie udało się powtórzyć sprawdzenia stanu (status { $status }). Pokazano poprzedni wynik.
admin-dash-h-refresh-network = Nie udało się powtórzyć sprawdzenia stanu: serwer nie odpowiedział. Pokazano poprzedni wynik.
admin-dash-h-enable = Aby to włączyć
admin-dash-h-probe-time = Sonda trwała { $time }
admin-dash-h-outbox-probe-time = Agregacja stanu trwała { $time }; czas uzgadniania wysyłki jest pokazany osobno.
admin-dash-h-history = Ostatnie sprawdzenia: { $n }
admin-dash-h-open-diagnostics = Diagnostyka
admin-dash-h-open-logs = Dzienniki
admin-dash-h-open-plugins = Wtyczki
admin-dash-h-open-maintenance = Konserwacja
admin-dash-h-not-tracked = Nie śledzone
admin-dash-h-unlimited = bez limitu
admin-dash-h-m-none = brak
admin-dash-h-m-ago = { $age } temu
admin-dash-h-m-ping = Czas odpowiedzi
admin-dash-h-m-pool = Pula połączeń
admin-dash-h-m-pool-value = w użyciu: { $inUse }, bezczynnych: { $idle } (otwartych: { $size })
admin-dash-h-m-engine = Serwer bazy danych
admin-dash-h-m-location = Lokalizacja
admin-dash-h-m-db-size = Rozmiar bazy danych
admin-dash-h-m-wal-size = Dziennik zapisu z wyprzedzeniem
admin-dash-h-m-journal = Tryb dziennika
admin-dash-h-m-slow = Wolne zapytania (ostatnia godzina)
admin-dash-h-m-applied = Zastosowane
admin-dash-h-m-applied-value = { $applied } z { $expected } w tej wersji
admin-dash-h-m-latest = Najnowsza zastosowana
admin-dash-h-m-pending = Jeszcze niezastosowane
admin-dash-h-m-failed = Rozpoczęte, niezakończone
admin-dash-h-m-unknown = Z nowszej wersji
admin-dash-h-m-accounts = Skrzynki
admin-dash-h-m-accounts-value = zarejestrowanych: { $total }, synchronizuje się: { $syncing }, bezczynnych: { $idle }, z błędami: { $erroring }
admin-dash-h-m-last-sync = Od ostatniej zakończonej synchronizacji
admin-dash-h-m-last-sync-value = mediana { $p50 }, najdłuższa { $max }
admin-dash-h-m-never-synced = Brak synchronizacji od uruchomienia serwera
admin-dash-h-m-queue = Trwające synchronizacje
admin-dash-h-m-errors-hour = Błędy synchronizacji (ostatnia godzina)
admin-dash-h-m-by-kind = Połączenia według backendu
admin-dash-h-m-kind-value = { $kind }: skrzynek { $accounts }, połączeń obserwujących { $watch }
admin-dash-h-m-recent-errors = Ostatnie błędy
admin-dash-h-m-queued = Oczekuje na wysłanie
admin-dash-h-m-queued-value = do wysłania: { $queued }, ponawianych: { $deferred }, wysyłanych: { $sending }
admin-dash-h-m-oldest = Najdłużej oczekująca
admin-dash-h-m-scheduled = Zaplanowane lub wstrzymane
admin-dash-h-m-scheduled-value = zaplanowanych: { $scheduled }, wstrzymanych: { $held }
admin-dash-h-m-failed-total = Nieudane (nadal w skrzynce nadawczej)
admin-dash-h-m-failed-total-value = nieudanych: { $failed }, z nieznanym stanem dostarczenia: { $uncertain }
admin-dash-h-m-sends-hour = Wysyłki (ostatnia godzina)
admin-dash-h-m-sends-hour-value = wysłanych: { $sent }, odroczonych: { $deferred }, nieudanych: { $failed }
admin-dash-h-m-last-send = Ostatnia wysyłka
admin-dash-h-m-last-send-value = { $outcome }, { $age } temu
admin-dash-h-m-last-send-none = Brak od uruchomienia serwera
admin-dash-h-outcome-sent = Wysłano
admin-dash-h-outcome-deferred = Odroczono
admin-dash-h-outcome-failed = Nie powiodło się
admin-dash-h-outcome-uncertain = Nieznany stan dostarczenia
admin-dash-h-m-submission = Serwery wysyłki
admin-dash-h-m-submission-value = odpowiedziało { $reachable } z { $checked } (tylko uzgodnienie, bez logowania; najwolniejszy { $slowest }), sprawdzono { $age } temu
admin-dash-h-m-submission-none = Nie skonfigurowano żadnego serwera wysyłki.
admin-dash-h-m-submission-pending = Trwa sprawdzanie połączenia.
admin-dash-h-m-submission-unmeasured = Serwery wymagające uwierzytelnionego sprawdzenia u dostawcy: { $count }; nie wysłano danych uwierzytelniających.
admin-dash-h-m-documents = Dokumenty
admin-dash-h-m-documents-value = { $documents } z { $messages } zapisanych wiadomości
admin-dash-h-m-index-pending = Jeszcze nie zaindeksowano
admin-dash-h-m-index-size = Rozmiar na dysku
admin-dash-h-m-index-memory = W pamięci
admin-dash-h-m-last-commit = Ostatni zapis
admin-dash-h-m-rebuild = Przebudowa
admin-dash-h-m-rebuild-running = W toku: { $pct } ({ $done } z { $total })
admin-dash-h-m-rebuild-idle = Nie trwa
admin-dash-h-m-version = Wersja procesu roboczego
admin-dash-h-m-worker-missing = Brak pliku wykonywalnego procesu roboczego
admin-dash-h-m-render-ping = Ping (start, izolacja, odpowiedź)
admin-dash-h-m-render-jobs = Zadania w toku
admin-dash-h-m-render-failures = Problemy (ostatnia godzina)
admin-dash-h-m-render-failures-value = nieudanych: { $failed }, po przekroczeniu czasu: { $timeouts }, odrzuconych z powodu zajętości: { $busy }
admin-dash-h-m-jail = Izolacja jądra (jail)
admin-dash-h-m-jail-required = Wymagana
admin-dash-h-m-jail-optional = Niewymagana na tej platformie
admin-dash-h-m-jail-degraded = Pogorszona: { $why }
admin-dash-h-m-plugins = Wtyczki
admin-dash-h-m-plugins-value = zainstalowanych: { $installed }, włączonych: { $enabled }, wczytanych: { $loaded }
admin-dash-h-m-plugin-list = Włączone wtyczki
admin-dash-h-m-plugin-loaded = wczytana
admin-dash-h-m-plugin-not-loaded = niewczytana ({ $reason })
admin-dash-h-m-plugin-limits = pamięć { $memory }, termin { $deadline }, paliwo { $fuel }
admin-dash-h-m-plugin-calls = Błędy wywołań (ostatnia godzina)
admin-dash-h-m-plugin-calls-value = pułapek: { $traps }, przekroczeń limitów: { $limits }, innych błędów: { $errors }
admin-dash-h-m-host-ping = Blokada hosta wtyczek
admin-dash-h-t-ping = Ostrzeżenie powyżej { $warn }, błąd powyżej { $fail }
admin-dash-h-t-oldest = Ostrzeżenie, gdy wiadomość czeka dłużej niż { $warn }, błąd powyżej { $fail }
admin-dash-h-t-pending = Ostrzeżenie powyżej { $n } niezaindeksowanych wiadomości
admin-dash-h-t-render = Ostrzeżenie powyżej { $warn }; błąd, gdy brak odpowiedzi w ciągu { $timeout }
admin-dash-h-t-sync = Błąd, gdy synchronizacja nie powiedzie się dla { $pct } lub więcej skrzynek
admin-dash-h-chart-db = Czas odpowiedzi bazy danych
admin-dash-h-chart-render = Ping procesu renderowania
admin-dash-h-chart-outbox = Wiadomości oczekujące na wysłanie
admin-dash-h-chart-search = Niezaindeksowane wiadomości
admin-dash-na-index-proxy = Tryb proxy: ten serwer nie przechowuje indeksu wyszukiwania.
admin-dash-na-index-memory = Indeks wyszukiwania jest przechowywany w pamięci, więc nie ma rozmiaru na dysku.
admin-dash-customize = Dostosuj pulpit
admin-dash-limitations = Dostępność danych
