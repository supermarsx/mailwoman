# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Strony diagnostyki
admin-diagx-tab-overview = Przegląd
admin-diagx-tab-server = Serwer
admin-diagx-tab-connectivity = Łączność
admin-diagx-tab-backends = Backendy poczty
admin-diagx-tab-posture = Stan bezpieczeństwa
admin-diagx-tab-tools = Narzędzia
admin-diagx-tab-history = Historia
admin-diagx-tab-report = Raport
admin-diagx-tab-server-intro = Kontrole własnych elementów tego serwera: bazy danych, jej migracji i klucza serwera. Uruchamiają się razem z kontrolami serwera.
admin-diagx-tab-connectivity-intro = Czy ten serwer jest osiągalny pod swoją publiczną nazwą, czy może dotrzeć do zapisanych backendów i czy adresy prywatne pozostają poza zasięgiem. Poniższe narzędzia domen sprawdzają rekordy DNS, testują TLS i wykrywają ustawienia dla jednej domeny.
admin-diagx-tab-backends-intro = Połączenie i uzgodnienie zabezpieczeń z każdym zapisanym punktem końcowym poczty, wysyłki, reguł i DAV. Nie są wysyłane dane uwierzytelniające i nie jest dostarczana żadna poczta.
admin-diagx-tab-posture-intro = Ustawienia chroniące logowania i sesje oraz to, jak fail2ban widziałby adres żądania.
admin-diagx-tab-tools-intro = Inne miejsca do sprawdzenia usługi oraz limity obowiązujące przy uruchomieniach diagnostycznych.
admin-diagx-tab-history-intro = Zapisane uruchomienia, od najnowszego. Otwórz uruchomienie, aby zobaczyć jego wyniki i zmiany od poprzedniego.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Ostrzeżenie
admin-diagx-status-fail = Niepowodzenie
admin-diagx-status-skipped = Pominięto
admin-diagx-status-not-run = Nie uruchomiono
admin-diagx-status-running = Trwa
admin-diagx-run = Uruchom
admin-diagx-run-again = Uruchom ponownie
admin-diagx-running = Uruchamianie…
admin-diagx-run-server-note = Uruchamia kontrole serwera ponownie. Każda karta z tego uruchomienia się aktualizuje.
admin-diagx-run-backends-note = Ponownie sprawdza wszystkie zapisane backendy. Każda karta backendu się aktualizuje.
admin-diagx-how-to-fix = Jak naprawić
admin-diagx-details = Szczegóły
admin-diagx-copy-details = Kopiuj szczegóły
admin-diagx-copy-value = Kopiuj: { $label }
admin-diagx-copied = Skopiowano.
admin-diagx-copy-failed = Przeglądarka nie zezwoliła na kopiowanie. Zaznacz tekst i skopiuj go ręcznie.
admin-diagx-not-run-yet = Jeszcze nie uruchomiono.
admin-diagx-not-reported = Ostatnie uruchomienie serwera nie zgłosiło tej kontroli; nie dotyczy ona tego wdrożenia.
admin-diagx-posture-met = Spełnione
admin-diagx-posture-not-met = Niespełnione
admin-diagx-loading = Wczytywanie…
admin-diagx-none = Brak
admin-diagx-yes = Tak
admin-diagx-no = Nie
admin-diagx-open = Otwórz
admin-diagx-row-result = Wynik
admin-diagx-row-duration = Czas trwania
admin-diagx-row-checked = Sprawdzono
admin-diagx-row-id = Identyfikator kontroli
admin-diagx-row-domain = Domena
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protokół
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Możliwości
admin-diagx-row-credentials = Wysłane dane uwierzytelniające
admin-diagx-row-source = Znaleziono w
admin-diagx-row-oauth = Wymaga OAuth
admin-diagx-overview-intro = Najnowszy wynik każdej kontroli, według grup. Uruchom wszystkie uruchamia kontrole serwera, a potem sprawdza każdy zapisany backend.
admin-diagx-run-all = Uruchom wszystkie kontrole
admin-diagx-run-all-step = Krok { $step } z { $steps }: { $suite ->
    [backends] zapisane backendy
   *[server] kontrole serwera
}.
admin-diagx-run-all-done = Wszystkie kontrole zakończone. Wyniki zapisano w Historii.
admin-diagx-backends-progress = Sprawdzono backendy: { $completed } z { $total }.
admin-diagx-counts = { $ok } OK · { $warn } ostrzeżeń · { $fail } niepowodzeń · { $notRun } nieuruchomionych
admin-diagx-attention = Wymaga uwagi
admin-diagx-attention-none = Żadna kontrola nie zgłosiła ostrzeżenia ani niepowodzenia w ostatnim uruchomieniu.
admin-diagx-never-run = Nie uruchomiono jeszcze żadnych kontroli. Uruchom wszystkie kontrole, aby uzyskać pierwszy wynik.
admin-diagx-last-server = Ostatnie kontrole serwera
admin-diagx-last-backends = Ostatnie kontrole backendów
admin-diagx-never = Nigdy
admin-diagx-open-health = Stan usługi
admin-diagx-check-database-connectivity = Połączenie z bazą danych
admin-diagx-check-database-connectivity-purpose = Baza danych odpowiada na proste zapytanie.
admin-diagx-check-database-connectivity-fix = Sprawdź, czy baza danych działa i czy MW_DB_PATH lub MW_DATABASE_URL na nią wskazuje. Błąd połączenia podaje dziennik serwera.
admin-diagx-check-database-write = Zapisy do bazy danych
admin-diagx-check-database-write-purpose = Można zapisać wiersz próbny, a następnie jest on wycofywany.
admin-diagx-check-database-write-fix = Sprawdź wolne miejsce na dysku i czy plik bazy danych lub użytkownik mają prawo zapisu. Wolumin tylko do odczytu lub pełny dysk zatrzymuje zapisy.
admin-diagx-check-database-migrations = Migracje bazy danych
admin-diagx-check-database-migrations-purpose = Baza danych ma każdą migrację, jakiej oczekuje ta wersja serwera.
admin-diagx-check-database-migrations-fix = Uruchom serwer ponownie, aby zastosował oczekujące migracje, i sprawdź w dzienniku, czy jakaś migracja się nie powiodła. Nie uruchamiaj starszego serwera na nowszej bazie danych.
admin-diagx-check-server-key-roundtrip = Klucz serwera
admin-diagx-check-server-key-roundtrip-purpose = Klucz serwera zapieczętowuje i otwiera wartość testową.
admin-diagx-check-server-key-roundtrip-fix = Sprawdź, czy MW_SERVER_KEY (lub plik klucza) jest kluczem, z którym utworzono tę bazę danych. Zapisanych sekretów nie można otworzyć innym kluczem.
admin-diagx-check-other-purpose = Kontrola zgłoszona przez ten serwer.
admin-diagx-check-other-fix = Otwórz Szczegóły, aby zobaczyć surowy wynik, i sprawdź dziennik serwera z czasu uruchomienia.
admin-diagx-check-network-public-dns = Nazwa publiczna
admin-diagx-check-network-public-dns-purpose = Nazwa hosta z MW_PUBLIC_URL jest rozwiązywana z tego serwera.
admin-diagx-check-network-public-dns-fix = Sprawdź rekord DNS nazwy publicznej lub ustaw MW_PUBLIC_URL na adres, którego ludzie używają do połączenia z tym serwerem.
admin-diagx-check-network-public-port = Port publiczny
admin-diagx-check-network-public-port-purpose = Port publicznego adresu URL przyjmuje połączenie z tego serwera.
admin-diagx-check-network-public-port-fix = Sprawdź zaporę, zwrotne proxy i przekierowanie portów. Niektóre routery nie mogą połączyć się ze swoim własnym adresem publicznym (NAT hairpinning); sprawdź też spoza swojej sieci.
admin-diagx-check-network-backend-ports = Porty backendów
admin-diagx-check-network-backend-ports-purpose = Host i port każdego zapisanego backendu przyjmują bezpośrednie połączenie.
admin-diagx-check-network-backend-ports-fix = Sprawdź, czy hosty backendów działają i czy zapora pozwala temu serwerowi się z nimi połączyć. Backendy poczty pokazują, który punkt końcowy nie przechodzi uzgodnienia.
admin-diagx-check-egress-private-policy = Ochrona adresów prywatnych
admin-diagx-check-egress-private-policy-purpose = Domyślnie połączenia wychodzące do pętli zwrotnej i adresów prywatnych są odrzucane.
admin-diagx-check-egress-private-policy-fix = Przejrzyj ustawienia ruchu wychodzącego w Prywatności. Zezwalaj na adresy prywatne tylko dla wskazanych backendów, które ich potrzebują.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Żądania docierają do tego serwera przez HTTPS.
admin-diagx-check-security-https-fix = Udostępnij konsolę przez HTTPS lub ustaw MW_PUBLIC_URL na https oraz MW_TRUSTED_PROXIES, gdy TLS kończy zwrotne proxy.
admin-diagx-check-security-secure-cookies = Bezpieczne pliki cookie
admin-diagx-check-security-secure-cookies-purpose = Pliki cookie sesji są wysyłane tylko przez HTTPS.
admin-diagx-check-security-secure-cookies-fix = Ustaw MW_COOKIE_SECURE na true, gdy serwer jest dostępny przez HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Przeglądarkom mówi się, aby dla tej witryny używały HTTPS.
admin-diagx-check-security-hsts-fix = Włącz Strict-Transport-Security w ustawieniach bezpieczeństwa lub wysyłaj go ze zwrotnego proxy.
admin-diagx-check-security-csrf = Ochrona CSRF
admin-diagx-check-security-csrf-purpose = Żądania zmieniające dane muszą zawierać token CSRF.
admin-diagx-check-security-csrf-fix = Ustaw MW_CSRF_STRICT na true.
admin-diagx-check-security-admin-factors = Drugi składnik administratora
admin-diagx-check-security-admin-factors-purpose = Każdy administrator ma drugi składnik logowania.
admin-diagx-check-security-admin-factors-fix = Poproś każdego administratora bez niego o dodanie klucza dostępu lub aplikacji uwierzytelniającej w Moich preferencjach albo wymagaj tego w Centrum bezpieczeństwa.
admin-diagx-check-security-signin-pacing = Opóźnienie ponownego logowania
admin-diagx-check-security-signin-pacing-purpose = Po nieudanym logowaniu następuje krótkie oczekiwanie.
admin-diagx-check-security-signin-pacing-fix = Włącz spowalnianie logowania w zasadach bezpieczeństwa.
admin-diagx-check-security-credential-locks = Blokady danych uwierzytelniających
admin-diagx-check-security-credential-locks-purpose = Powtarzane nieudane logowania blokują dane uwierzytelniające na pewien czas.
admin-diagx-check-security-credential-locks-fix = Włącz automatyczne blokady w Centrum bezpieczeństwa.
admin-diagx-check-security-geography = Dane geograficzne logowania
admin-diagx-check-security-geography-purpose = Zasada geografii logowania ma potrzebne dane o krajach.
admin-diagx-check-security-geography-fix = Zainstaluj lub zaktualizuj bazę krajów albo wyłącz zasadę geografii.
admin-diagx-check-security-render-isolation = Izolowane renderowanie wiadomości
admin-diagx-check-security-render-isolation-purpose = Wiadomości są renderowane w oddzielnym, izolowanym procesie roboczym.
admin-diagx-check-security-render-isolation-fix = Sprawdź, czy MW_RENDER_BIN wskazuje program mw-render i czy się uruchamia. Stan usługi na Pulpicie pokazuje stan procesu roboczego.
admin-diagx-check-security-deployment = Ustawienia wdrożenia
admin-diagx-check-security-deployment-purpose = Ustawienia bezpieczeństwa wdrożenia można odczytać.
admin-diagx-check-security-deployment-fix = Sprawdź połączenie z bazą danych, a potem otwórz ocenę bezpieczeństwa wdrożenia.
admin-diagx-check-security-plaintext-backends = Hasła przez połączenia nieszyfrowane
admin-diagx-check-security-plaintext-backends-purpose = Żaden zapisany backend nie przyjmuje haseł bez TLS.
admin-diagx-check-security-plaintext-backends-fix = Edytuj połączenia domeny i wybierz TLS lub STARTTLS. Zezwalaj na zwykłe połączenia tylko w sieci prywatnej, którą kontrolujesz.
admin-diagx-check-fail2ban-client-address = Adres klienta dla fail2ban
admin-diagx-check-fail2ban-client-address-purpose = Adres, który wskazałby wiersz dziennika dla fail2ban, jest rzeczywistym adresem klienta.
admin-diagx-check-fail2ban-client-address-fix = Za zwrotnym proxy ustaw MW_TRUSTED_PROXIES na adres proxy, bo inaczej fail2ban zablokuje proxy zamiast klienta. Adres pętli zwrotnej jest oczekiwany, gdy otwierasz konsolę na samym serwerze.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Punkt końcowy { $protocol }: połączenie i uzgodnienie zabezpieczeń, bez logowania.
admin-diagx-backend-other-purpose = Uwaga z uruchomienia backendów.
admin-diagx-backend-fix = Sprawdź nazwę hosta, port i tryb TLS zapisane dla tego backendu, czy DNS się rozwiązuje i czy certyfikat jest ważny dla tej nazwy hosta. Następnie uruchom ponownie.
admin-diagx-backend-config-fix = Otwórz połączenia domeny i zapisz je ponownie; nie udało się odczytać zapisanej listy backendów.
admin-diagx-backend-results = Kontrole backendów
admin-diagx-backends-not-run = Zapisane backendy nie zostały jeszcze sprawdzone.
admin-diagx-backends-run-at = Ostatnio sprawdzono { $date }: wyników { $count }.
admin-diagx-failover-title = Przełączanie awaryjne
admin-diagx-failover-purpose = Czy domena może przełączyć się na inny backend poczty, gdy pierwszy wybór jest nieosiągalny.
admin-diagx-failover-state = Przełączanie awaryjne { $state ->
    [on] włączone
   *[off] wyłączone
} · backendów poczty: { $backends }{ $enabled ->
    [no] {" "}· domena wyłączona
   *[yes] {""}
}
admin-diagx-no-domains = Brak zapisanych domen.
admin-diagx-domain-tools = Narzędzia domen
admin-diagx-domain-tools-purpose = Jednorazowe kontrole zapisanej domeny. Nie logują się ani nie wysyłają poczty.
admin-diagx-domain = Domena
admin-diagx-tool-dns = Rekordy DNS
admin-diagx-tool-dns-purpose = Sprawdza rekordy MX, SPF, DMARC i usług (SRV) domeny.
admin-diagx-tool-dns-fix = Dodaj brakujące rekordy u dostawcy DNS. Bez MX domena nie odbiera poczty; bez SPF i DMARC inne serwery mogą odrzucać jej pocztę.
admin-diagx-srv-found = Znaleziono rekordów usług: { $found } z { $total }
admin-diagx-tool-tls = TLS i uzgodnienie
admin-diagx-tool-tls-purpose = Łączy się z każdym zapisanym punktem końcowym domeny i podaje wersję TLS, wygaśnięcie certyfikatu i czas trwania.
admin-diagx-tool-tls-fix = Dla niedostępnego punktu końcowego sprawdź host, port i zaporę. Odnów certyfikat, który wygasa w ciągu 14 dni.
admin-diagx-probe-none = Domena nie ma zapisanych punktów końcowych do przetestowania.
admin-diagx-tool-detect = Wykryj ustawienia
admin-diagx-tool-detect-purpose = Sprawdza opublikowane ustawienia poczty (autoconfig, autodiscover, SRV) dla nazwy domeny. Nic nie jest zapisywane.
admin-diagx-tool-detect-fix = Opublikuj rekordy autoconfig lub SRV dla domeny albo wpisz ustawienia ręcznie w Domenach.
admin-diagx-detect-name = Nazwa domeny do sprawdzenia
admin-diagx-tool-health = Stan usługi
admin-diagx-tool-health-purpose = Synchronizacja poczty, skrzynka nadawcza, indeks wyszukiwania, proces renderowania i wtyczki, na Pulpicie.
admin-diagx-tool-deployment-purpose = Każde ustawienie bezpieczeństwa wdrożenia, które ten serwer może zaobserwować, wraz z ustawieniem, które je kontroluje.
admin-diagx-tool-ssh = Testy tuneli SSH
admin-diagx-tool-ssh-purpose = Testuj tunele SSH używane do łączenia się z backendami w innej sieci.
admin-diagx-tool-geo = Geografia logowania
admin-diagx-tool-geo-purpose = Skąd pochodzą logowania i baza krajów stojąca za zasadą geografii.
admin-diagx-catalogue-purpose = Co obejmuje każdy zestaw diagnostyczny i jakich limitów przestrzega każde uruchomienie.
admin-diagx-suite-unsupported = Niedostępne: { $reason }
admin-diagx-suite-server = Kontrole serwera
admin-diagx-suite-backends = Zapisane backendy
admin-diagx-run-title = { $suite ->
    [backends] Zapisane backendy
   *[server] Kontrole serwera
}, { $date }
admin-diagx-changes = Zmiany
admin-diagx-changes-first = Brak wcześniejszego uruchomienia tego rodzaju do porównania.
admin-diagx-changes-none = Nic się nie zmieniło od uruchomienia z { $date }.
admin-diagx-changes-since = Zmiany od uruchomienia z { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Raport rozwiązywania problemów
admin-diagx-report-purpose = Raport JSON dla pomocy technicznej: flagi środowiska uruchomieniowego, najnowsze wyniki i wybrane uruchomienie. Hasła, tokeny, pliki cookie, klucze, surowa konfiguracja i poczta nigdy nie są dołączane.
admin-diagx-report-run = Uruchomienie do dołączenia
admin-diagx-report-preview = Podgląd
admin-crumb-server = Serwer
admin-crumb-connectivity = Łączność
admin-crumb-posture = Stan bezpieczeństwa
admin-crumb-tools = Narzędzia
admin-crumb-history = Historia
admin-crumb-report = Raport
admin-diagx-table-actions = Działania
admin-diagx-check-pdfjs-viewer-assets = Pliki przeglądarki PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = Wszystkie pliki przeglądarki PDF są obecne, mają właściwy typ i pasują do tej kompilacji serwera.
admin-diagx-check-pdfjs-viewer-assets-fix = Wdróż ponownie kompilację webową lub usuń folder pdfjs z MW_WEB_DIR, aby użyć kopii wbudowanej w serwer. Jeśli to kończy się niepowodzeniem, a nie ostrzeżeniem, przebuduj obraz serwera.

# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-probe-line = { $host } · osiągalny: { $reachable ->
    [yes] tak
   *[no] nie
} · TLS { $tls } · certyfikat { $days ->
    [none] nie zgłoszono
    [one] wygasa za { $days } dzień
    [few] wygasa za { $days } dni
    [many] wygasa za { $days } dni
   *[other] wygasa za { $days } dnia
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
