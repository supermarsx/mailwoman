# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Szczegóły bezpieczeństwa wiadomości
security-section-auth = Uwierzytelnianie
security-section-received = Ścieżka dostarczenia
security-section-signature = Podpis
security-section-attachments = Załączniki
security-section-warnings = Ostrzeżenia
security-section-controls = Działania wobec nadawcy
security-received-empty = Brak łańcucha nagłówków Received.
security-attachments-empty = Brak załączników.
security-hop-unknown = nieznany
security-unknown = nieznane

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = pozytywny
security-auth-fail = negatywny
security-auth-none = brak
security-auth-neutral = neutralny
security-auth-temperror = błąd tymczasowy
security-auth-permerror = błąd trwały

# Expert alignment detail field labels + values.
security-detail-domain = domena
security-detail-selector = selektor
security-detail-policy = zasada
security-detail-alignment = zgodność domen
security-detail-chain-length = długość łańcucha
security-aligned = zgodne
security-not-aligned = niezgodne

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Podpis zweryfikowany
security-sig-unverified-key = Podpisano — klucz podpisującego niezweryfikowany
security-sig-invalid = Podpis jest nieprawidłowy
security-sig-none = Niepodpisano
security-chain-trusted = Zaufany łańcuch
security-chain-untrusted = Niezaufany łańcuch
security-chain-expired = Łańcuch wygasł
security-chain-unknown = Łańcuch nieznany
security-revocation-good = Nieunieważniony
security-revocation-revoked = Klucz unieważniony
security-revocation-unknown = Stan unieważnienia nieznany
security-key-changed = Klucz podpisującego zmienił się od ostatniego razu
security-fact-signer-key = Klucz podpisującego
security-fact-algorithm = Algorytm
security-fact-key-created = Klucz utworzono
security-fact-key-expires = Klucz wygasa
security-fact-chain = Łańcuch
security-fact-revocation = Unieważnienie
security-fact-key-change = Zmiana klucza

# -- Attachment risk --------------------------------------------------------
security-attach-none = Brak znanego ryzyka
security-attach-macro = Zawiera makra
security-attach-executable = Plik wykonywalny
security-attach-encrypted-archive = Zaszyfrowane archiwum
security-attach-double-extension = Podwójne rozszerzenie pliku
security-attach-mismatch = niezgodność typu (zadeklarowany { $declared }, wykryty { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Adres Reply-To różni się od adresu nadawcy
security-anomaly-envelopeFromDivergence = Nadawca z koperty różni się od adresu From
security-anomaly-messageIdDomainAnomaly = Domena w Message-ID nie pasuje do nadawcy
security-anomaly-dateSkew = Data wysłania wygląda na przesuniętą
security-anomaly-punycodeSender = Nadawca używa punycode (możliwa domena podszywająca się)

# -- Sender controls --------------------------------------------------------
security-control-block = Zablokuj nadawcę
security-control-silence = Wycisz nadawcę
security-control-ignore-conversation = Ignoruj wątek
security-control-report-phishing = Zgłoś phishing
security-control-report-junk = Zgłoś spam
security-control-done-block = Nadawca zablokowany
security-control-done-silence = Nadawca wyciszony
security-control-done-ignore-conversation = Wątek ignorowany
security-control-done-report-phishing = Zgłoszono jako phishing
security-control-done-report-junk = Zgłoszono jako spam

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Przechowywanie zero-access
security-za-on = Włączone
security-za-off = Wyłączone
security-za-protects = Zero-access szyfruje przechowywaną pocztę, notatki i dane PIM, tak że serwer hostingowy przechowuje tylko szyfrogram, którego nie może odczytać. Chroni zawartość przechowywanych danych przed ciekawskim operatorem lub włamaniem na host przechowujący dane: każdy, kto odczyta bazę danych, pliki na dysku lub skradzioną kopię zapasową, widzi szyfrogram XChaCha20-Poly1305 i nie może odzyskać treści wiadomości, tematów, załączników, tekstu notatek ani indeksu wyszukiwania.

security-za-server-sees-title = Co serwer nadal widzi
security-za-sees-1 = Bloki szyfrogramu — same zaszyfrowane wiersze.
security-za-sees-2 = Nieprzejrzyste identyfikatory wierszy — identyfikatory używane do zapisywania i pobierania wierszy.
security-za-sees-3 = Rozmiary — długość każdego szyfrogramu (przybliżony rozmiar wiadomości/załącznika).
security-za-sees-4 = Znaczniki czasu — kiedy wiersze są zapisywane i aktualizowane.
security-za-sees-5 = Metadane routingu koperty potrzebne do pośredniczenia w IMAP/SMTP — serwer nadal łączy się w Twoim imieniu z nadrzędnym dostawcą poczty, więc przechodzą przez niego metadane połączenia oraz koperta routingu wymagana do wysyłania i odbierania poczty.

security-za-active-server-caveat = Zero-access chroni dane W SPOCZYNKU przed ciekawskim lub przejętym hostem. W pełni złośliwy aktywny serwer, który pośredniczy w bieżącym ruchu IMAP/SMTP, jest silniejszym przeciwnikiem i zero-access NIE chroni przed nim: taki serwer znajduje się na bieżącym połączeniu z dostawcą poczty i może obserwować lub modyfikować pocztę podczas jej przepływu, niezależnie od tego, jak zaszyfrowana jest przechowywana kopia. Wybierz zero-access, jeśli chodzi Ci o to, by „osoba prowadząca (lub przejmująca) host przechowujący dane nie mogła czytać mojej przechowywanej poczty”. Nie jest to ochrona przed operatorem, który przejmuje bieżącą ścieżkę poczty.

security-za-no-search-claim = Nie ma tu szyfrowania umożliwiającego wyszukiwanie po stronie serwera i nikt tego nie twierdzi. Wyszukiwanie odbywa się w całości po stronie klienta na treści odszyfrowanej lokalnie; serwer nigdy nie przechowuje przeszukiwalnej postaci tekstu jawnego.

security-za-recovery-tradeoff = Klucze są wyprowadzane i przechowywane wyłącznie na Twoich urządzeniach — serwer nigdy nie otrzymuje klucza w postaci jawnej. Jeśli utracisz hasło (lub klucz dostępu) ORAZ wszystkie sparowane urządzenia ORAZ frazę odzyskiwania, zaszyfrowanych danych nie da się odzyskać. Zapisz frazę odzyskiwania offline i traktuj ją jak główny sekret konta.

security-za-setup-key = Skonfiguruj klucz
security-za-key-source = Źródło klucza
security-za-passphrase = Hasło
security-za-passkey = Klucz dostępu (bez hasła)
security-za-passkey-unavailable = klucze dostępu nie są dostępne w tej przeglądarce
security-za-passphrase-label = Hasło zero-access
security-za-enable = Włącz zero-access
security-za-recovery-title = Fraza odzyskiwania
security-za-recovery-heading = Fraza odzyskiwania — zapisz ją teraz offline
security-za-recovery-note = To jedyna kopia. Każdy, kto ją ma, może odczytać Twoje dane; bez niej (i bez sparowanego urządzenia) danych nie da się odzyskać.
security-za-disable-section = Wyłącz zero-access
security-za-disable-heading = Wyłączanie zero-access
security-za-disable-note = Nowe dane będą ponownie przechowywane bez szyfrowania. Istniejące zaszyfrowane dane pozostają czytelne tylko dopóty, dopóki możesz wyprowadzić swój klucz.
security-za-disable-btn = Wyłącz zero-access
security-za-err-passphrase-len = użyj hasła o długości co najmniej 8 znaków
security-za-err-enable = nie udało się włączyć zero-access
security-za-err-disable = nie udało się wyłączyć zero-access

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Parowanie urządzeń
security-pair-heading = Sparuj urządzenie
security-pair-intro = Przenieś klucze na inne urządzenie bez przesyłania ich przez serwer — przekazuje on tylko nieprzejrzystą zapieczętowaną kopertę. Przed zaufaniem parowaniu porównaj sześć słów na obu ekranach.
security-pair-role = Rola w parowaniu
security-pair-new-role = To jest nowe urządzenie
security-pair-existing-role = Sparuj stąd inne urządzenie
security-pair-show-qr = Pokaż kod QR parowania
security-pair-scan = Zeskanuj to na istniejącym urządzeniu
security-pair-qr-label = Kod QR parowania urządzenia
security-pair-qr-default = Kod QR parowania
security-pair-copy-code = Lub skopiuj ten kod parowania
security-pair-paste-envelope = Wklej zapieczętowaną kopertę z drugiego urządzenia
security-pair-envelope-placeholder = koperta…
security-pair-envelope-label = Zapieczętowana koperta
security-pair-complete = Zakończ parowanie
security-pair-code-from-new = Kod parowania z nowego urządzenia
security-pair-code-placeholder = kod parowania…
security-pair-code-label = Kod parowania
security-pair-seal = Zapieczętuj moje klucze dla tego urządzenia
security-pair-envelope-out = Zapieczętowana koperta — wklej ją na nowym urządzeniu
security-pair-compare = Porównaj te słowa na obu urządzeniach
security-pair-sas-label = Krótki ciąg uwierzytelniający
security-pair-match = Słowa są zgodne
security-pair-abort = Różnią się — przerwij
security-pair-confirmed = Parowanie potwierdzone. Ten kanał jest uwierzytelniony.
security-pair-err-start = nie udało się rozpocząć parowania
security-pair-err-complete = nie udało się zakończyć parowania
security-pair-err-unlock = najpierw odblokuj zero-access, aby sparować inne urządzenie
security-pair-err-seal = nie udało się zapieczętować klucza głównego

# -- Machine translation (2026-11): added to match en ---------------------
security-stepup-title = Potwierdź swoją tożsamość
security-stepup-body = Potwierdź tożsamość, aby kontynuować to działanie. Potwierdzenie obowiązuje przez pięć minut w tej sesji.
security-stepup-password = Hasło
security-stepup-code = Kod z aplikacji uwierzytelniającej
security-stepup-confirm = Potwierdź i kontynuuj
security-stepup-passkey = Użyj klucza dostępu
security-stepup-cancel = Anuluj
security-stepup-failed = Potwierdzenie nie powiodło się. Spróbuj ponownie.
security-stepup-unavailable = Ta sesja nie ma dostępnej metody potwierdzenia.
security-center-title = Centrum bezpieczeństwa
security-center-history = Historia logowań
security-center-result = Wynik
security-center-all = Wszystkie wyniki
security-center-success = Udane
security-center-failure = Nieudane
security-center-blocked = Zablokowane
security-center-refresh = Odśwież
security-center-export = Eksportuj historię
security-center-devices = Zapamiętane urządzenia
security-center-remember-help = Jeśli administrator na to pozwala, zapamiętaj to urządzenie po zweryfikowaniu tożsamości. Ważne zmiany nadal wymagają ponownej weryfikacji.
security-center-label = Nazwa urządzenia
security-center-remember = Zapamiętaj to urządzenie
security-center-revoke = Cofnij
security-center-activity = Aktywność konta
security-center-error = Nie udało się wczytać ani zmienić informacji o bezpieczeństwie. Administrator mógł wyłączyć tę czynność.
security-center-first-seen = Po raz pierwszy: { $when }
security-center-passkey-mode = W trybie silnika klucze dostępu mogą logować bez hasła. W trybie proxy weryfikują drugi składnik po podaniu hasła do poczty.
security-stepup-recovery = Kod odzyskiwania (z hasłem, zamiast kodu z aplikacji uwierzytelniającej)
security-center-signin = Logowanie i klucze dostępu
security-center-sessions = Sesje i dostęp
security-center-encryption = Szyfrowanie
signin-geo-title = Polityka krajów logowania
signin-geo-help = Opcjonalnie odrzucaj nowe logowania na Twoje konto z krajów, z których wcześniej nie korzystałeś, lub z dowolnego kraju spoza listy dozwolonych. Domyślnie jest wyłączona i nigdy nie wpływa na sesje, które są już zalogowane. Dozwolone adresy IP i kraje zawsze przechodzą tę kontrolę; hasło i drugi składnik nadal są wymagane.
signin-geo-accuracy = Wykrywanie kraju korzysta z bazy geolokalizacji adresów IP, która może być błędna lub nieaktualna. VPN, operatorzy komórkowi, serwery proxy i podróże mogą zablokować prawidłowe logowanie albo przepuścić obce. Dodaj zaufane adresy IP lub kraje do listy dozwolonych.
signin-geo-locked = Administrator kontroluje to ustawienie dla Twojego konta.
signin-geo-mode = Polityka
signin-geo-mode-off = Wyłączona
signin-geo-mode-block-unusual-countries = Blokuj kraje, z których się nie logowałem
signin-geo-mode-block-foreign-countries = Blokuj kraje spoza mojej listy dozwolonych
signin-geo-countries = Dozwolone kraje (dwuliterowe kody, oddzielone przecinkami)
signin-geo-ips = Dozwolone adresy IP lub sieci (po jednym w wierszu)
signin-geo-current = Twój bieżący adres to { $ip } (wykryty kraj: { $country }).
signin-geo-no-context = Nie udało się ustalić Twojego bieżącego adresu.
signin-geo-unknown-country = nieznany
signin-geo-add-ip = Dodaj mój bieżący adres IP
signin-geo-add-country = Dodaj mój bieżący kraj
signin-geo-save = Zapisz
signin-geo-saved = Zapisano.
signin-geo-invalid = Sprawdź kody krajów i adresy IP.
signin-geo-self-lockout = To zablokowałoby logowanie z Twojego bieżącego adresu lub kraju. Najpierw dodaj swój bieżący adres IP lub kraj do listy dozwolonych.
signin-geo-error = Nie udało się zapisać polityki krajów logowania.
session-alerts-title = Powiadomienia o bezpieczeństwie sesji
session-alerts-help = Ostrzeżenia o nietypowym użyciu Twoich zalogowanych sesji. Każde powiadomienie pokazuje, co zaobserwował serwer i co zrobił.
session-alerts-none = Brak powiadomień o bezpieczeństwie sesji.
session-alerts-banner = { $count ->
    [one] Masz 1 nieprzeczytane powiadomienie o bezpieczeństwie sesji.
    [few] Masz { $count } nieprzeczytane powiadomienia o bezpieczeństwie sesji.
    [many] Masz { $count } nieprzeczytanych powiadomień o bezpieczeństwie sesji.
   *[other] Masz { $count } nieprzeczytanego powiadomienia o bezpieczeństwie sesji.
}
session-alerts-banner-review = Przejrzyj
session-alerts-kind-countryChange = Sesji użyto z innego kraju ({ $from } do { $to }).
session-alerts-kind-impossibleTravel = Sesja pojawiła się w innym kraju ({ $from } do { $to }) szybciej, niż pozwala podróż.
session-alerts-kind-concurrentUse = Jedna sesja była używana jednocześnie z różnych krajów lub sieci ({ $from } i { $to }).
session-alerts-kind-fingerprintMismatch = Sesji użyto w przeglądarce lub na urządzeniu, które nie odpowiada temu, z którego nastąpiło logowanie.
session-alerts-kind-rotationReuse = Stary identyfikator sesji został przedstawiony ponownie po odnowieniu sesji.
session-alerts-action-alerted = Działanie: wysłano Ci ostrzeżenie.
session-alerts-action-step-up = Działanie: sesja została poproszona o potwierdzenie hasła i drugiego składnika.
session-alerts-action-revoked = Działanie: sesja została wylogowana.
session-alerts-countries = Kraje: { $from } do { $to }
session-alerts-unknown-country = nieznany
session-alerts-session = Sesja { $id }
session-alerts-caveat = Wykrywanie kraju może być niedokładne; VPN i sieci komórkowe mogą powodować fałszywe zmiany.
session-alerts-not-me = To nie ja
session-alerts-dismiss = Odrzuć
session-alerts-dismiss-all = Odrzuć wszystkie
session-alerts-revoked = Twoje pozostałe sesje zostały wylogowane. Zmień teraz hasło w sekcji hasła w Logowaniu i kluczach dostępu.
session-alerts-error = Nie udało się zaktualizować powiadomień o bezpieczeństwie sesji.
session-alerts-prefs = Preferencje alertów
session-alerts-country = Ostrzegaj mnie, gdy sesja jest używana z nowego kraju
session-alerts-country-help = Jeśli administrator nie zezwala na wyciszanie, serwer nadal będzie ostrzegać, nawet gdy ta opcja jest wyłączona.
session-alerts-channels = Gdzie otrzymywać ostrzeżenia
session-alerts-channel-app = W aplikacji
session-alerts-channel-email = W mojej skrzynce
session-alerts-channel-both = W aplikacji i w mojej skrzynce
session-alerts-sensitivity = Czułość
session-alerts-sensitivity-low = Niska: tylko ustalenia dotyczące lokalizacji, bez wymuszonego wylogowania
session-alerts-sensitivity-normal = Normalna
session-alerts-sensitivity-high = Wysoka: surowsza kontrola podróży, porównuje także sieci
session-alerts-reset = Użyj ustawień domyślnych administratora
session-alerts-pref-error = Nie udało się zapisać preferencji.
signin-geo-history-title = Zaobserwowane kraje
signin-geo-history-help = Kraje, z których się pomyślnie logowałeś, przechowywane przez { $days } dni. Tryb „nietypowe kraje” korzysta z tej listy. Przechowywane są tylko kody krajów, daty i liczby, nigdy adresy.
signin-geo-history-empty = Nie zarejestrowano jeszcze żadnych krajów.
signin-geo-history-entry = { $country }: pierwszy raz { $first }, ostatni raz { $last }, logowań: { $count }
signin-geo-history-forget = Zapomnij zaobserwowane kraje
security-severity-info = Informacja
security-severity-caution = Ostrożność
security-severity-suspicious = Podejrzane
security-explain-toggle = Co to oznacza?
security-explain-label-checked = Co sprawdzono
security-explain-label-legit = Typowe nieszkodliwe przyczyny
security-explain-label-risk = Na co może wskazywać
security-explain-label-action = Sugerowane działanie
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = Domenę po znaku @ w nagłówku Message-ID porównano z domeną w adresie Od. Są różne.
security-explain-messageIdDomainAnomaly-legit = Listy dyskusyjne, platformy marketingowe i newsletterowe, usługi przekazywania, bramy firmowe i hosty poczty webowej często generują Message-ID z własną nazwą serwera.
security-explain-messageIdDomainAnomaly-risk = Sfałszowany nadawca lub wiadomość utworzona przez oprogramowanie, które nie jest systemem pocztowym nadawcy. Samo w sobie to słaby sygnał.
security-explain-messageIdDomainAnomaly-action = Sprawdź powyższe wyniki SPF, DKIM i DMARC. Jeśli dla domeny nadawcy zakończyły się powodzeniem, to najpewniej rutyna. Jeśli się nie powiodły lub ich brak, unikaj linków i załączników i potwierdź nadawcę w inny sposób.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = Adres Odpowiedz do porównano z adresem Od. Odpowiedzi trafiłyby na inny adres.
security-explain-replyToMismatch-legit = Listy dyskusyjne, działy wsparcia, asystenci piszący w czyimś imieniu i formularze kontaktowe celowo ustawiają inny adres Odpowiedz do.
security-explain-replyToMismatch-risk = Przekierowanie odpowiedzi na adres kontrolowany przez atakującego, co jest częstym wzorcem w podszywaniu się i oszustwach fakturowych.
security-explain-replyToMismatch-action = Sprawdź adres Odpowiedz do przed odpowiedzią. Jeśli wiadomość prosi o pieniądze, dane logowania lub zmianę danych, potwierdź to kanałem, któremu już ufasz.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = Nadawcę koperty (Return-Path), czyli adres, na który wracają odrzucone wiadomości, porównano z adresem Od. Są różne.
security-explain-envelopeFromDivergence-legit = Usługi poczty masowej, systemy zgłoszeń i listy dyskusyjne używają własnego adresu zwrotnego, a w polu Od pokazują adres Twojego kontaktu.
security-explain-envelopeFromDivergence-risk = Podszywanie się pod nadawcę, gdy SPF lub DMARC również się nie powiódł. Samo w sobie jest częste i zwykle nieszkodliwe.
security-explain-envelopeFromDivergence-action = Sprawdź, czy DMARC zakończył się powodzeniem z dopasowaniem. Jeśli tak, nie trzeba nic robić.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = Nagłówek Date porównano z czasem odebrania wiadomości. Różnią się znacznie.
security-explain-dateSkew-legit = Komputer nadawcy z błędnym zegarem, wiadomość przetrzymywana w kolejce lub długo ponawiana albo zaplanowana wysyłka.
security-explain-dateSkew-risk = Odtworzona lub antydatowana wiadomość. Zdarza się rzadko i sama w sobie rzadko jest szkodliwa.
security-explain-dateSkew-action = Ostrożnie traktuj w wiadomości prośby wrażliwe na czas i potwierdź je, jeśli mają znaczenie.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = Domena nadawcy zawiera punycode (etykietę xn--), czyli sposób zapisu znaków spoza alfabetu łacińskiego w nazwach domen.
security-explain-punycodeSender-legit = W ten sposób kodowane są prawdziwe domeny w językach takich jak arabski, chiński, cyrylica czy portugalski z akcentami.
security-explain-punycodeSender-risk = Domena łudząco podobna, mieszająca znaki z innych alfabetów, aby przypominać znaną markę lub osobę.
security-explain-punycodeSender-action = Porównaj domenę nadawcy znak po znaku z tą, której oczekujesz. Nie klikaj linków ani nie otwieraj załączników, dopóki nie potwierdzisz nadawcy w inny sposób, i zgłoś wiadomość, jeśli nie jest prawdziwa.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = Serwer odbierający sprawdził tę metodę, a wynik nie zgadzał się z tym, co publikuje domena nadawcy.
security-explain-auth-fail-legit = Przekazywanie i listy dyskusyjne mogą zepsuć SPF lub DKIM, a źle skonfigurowana usługa wysyłająca może zawieść w obu.
security-explain-auth-fail-risk = Wiadomość może nie pochodzić od tego, za kogo się podaje, lub została zmieniona po drodze.
security-explain-auth-fail-action = Nie ufaj nadawcy na podstawie tego dowodu. Unikaj linków i załączników i zweryfikuj innym kanałem.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = Sprawdzenia nie dało się ukończyć, ponieważ opublikowany rekord domeny jest nieprawidłowy lub zbyt złożony.
security-explain-auth-permerror-legit = Błąd w rekordach DNS nadawcy.
security-explain-auth-permerror-risk = Nic dotyczącego samej wiadomości, ale domeny nadawcy nie można zweryfikować.
security-explain-auth-permerror-action = Traktuj to jak brak wyniku i sprawdź pozostałe metody.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = W nagłówkach wiadomości nie znaleziono wyniku dla tej metody.
security-explain-auth-none-legit = Domena nadawcy nie publikuje tej metody lub serwer odbierający jej nie rejestruje.
security-explain-auth-none-risk = Domena nie daje możliwości zweryfikowania nadawcy, więc nie można wykluczyć podszywania się.
security-explain-auth-none-action = Opieraj się na pozostałych metodach i na tym, czy spodziewasz się tej wiadomości.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = Domena opublikowała rekord dla tej metody, ale nie wypowiedziała się o tej wiadomości.
security-explain-auth-neutral-legit = Domena, która wciąż ustawia swoją politykę.
security-explain-auth-neutral-risk = Niewielkie. Nadawcy nie można było potwierdzić ani odrzucić.
security-explain-auth-neutral-action = Opieraj się na pozostałych metodach i na tym, czy spodziewasz się tej wiadomości.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = Sprawdzenia nie dało się dokończyć z powodu tymczasowego problemu, zwykle nieudanego zapytania DNS.
security-explain-auth-temperror-legit = Krótka awaria DNS po stronie nadawcy lub odbiorcy.
security-explain-auth-temperror-risk = Nic dotyczącego samej wiadomości.
security-explain-auth-temperror-action = Zwykle nie trzeba nic robić. Jeśli wiadomość jest ważna, sprawdź ponownie później.
breach-notice-heading = Zawiadomienie o naruszeniu ochrony danych osobowych
breach-notice-reference = Numer referencyjny { $reference }
breach-notice-ack = Potwierdź przeczytanie

session-evidence-ip = Adres IP
session-evidence-location = Lokalizacja
session-evidence-device = Typ urządzenia
session-evidence-recognized = Rozpoznawanie urządzenia
session-evidence-browser = Przeglądarka
session-evidence-engine = Silnik przeglądarki
session-evidence-os = System operacyjny
session-evidence-client = Typ klienta
session-evidence-version = Wersja klienta
session-evidence-authentication = Uwierzytelnianie
session-evidence-fingerprint = Odcisk urządzenia
session-evidence-agent = User agent
session-evidence-unavailable = Niedostępne
session-evidence-known = Rozpoznane urządzenie
session-evidence-new = Nierozpoznane urządzenie
session-evidence-technical = Szczegóły techniczne
session-evidence-legacy = Nie zapisano szczegółów urządzenia dla tego starszego powiadomienia.
session-evidence-help = Dane zgłasza klient. Rozpoznanie jest informacyjne i nie dowodzi bezpieczeństwa. Odcisk to odniesienie właściwe dla konta, nie identyfikator sprzętu.
session-evidence-device-desktop = Komputer
session-evidence-device-phone = Telefon
session-evidence-device-tablet = Tablet
session-evidence-device-unknown = Nieznane urządzenie
session-evidence-client-browser = Przeglądarka internetowa
session-alerts-kind-countryUnavailable = Nie udało się ustalić kraju żądania sesji.
session-alerts-kind-unknown = Sprawdź nietypową aktywność sesji.

notification-center-help = Sprawdź ważne działania i aktualizacje. Odrzucanie i archiwizacja porządkują listę, ale nie rozwiązują problemów bezpieczeństwa.
notification-center-active = Aktywne
notification-center-archived = Zarchiwizowane
notification-center-dismissed = Odrzucone
notification-center-type = Typ
notification-center-type-all = Wszystkie typy
notification-center-type-security = Bezpieczeństwo
notification-center-type-session = Ostrzeżenia sesji
notification-center-type-login = Logowania
notification-center-type-update = Aktualizacje
notification-center-login = Pomyślne logowanie
notification-center-security = Aktywność zabezpieczeń konta
notification-center-newDevice = Logowanie z nierozpoznanego urządzenia
notification-center-newCountry = Logowanie z nowego kraju
notification-center-twoFactorChanged = Zmieniono uwierzytelnianie dwuskładnikowe
notification-center-delegateAdded = Przyznano dostęp do skrzynki
notification-center-forwardingAdded = Dodano regułę przekazywania
notification-center-apiKeyCreated = Utworzono klucz API
notification-center-error = Nie udało się wczytać lub zmienić powiadomień. Odśwież widok.
notification-center-empty = Brak powiadomień w tym widoku.
notification-center-open = Otwórz powiązane działanie
notification-center-dismiss = Odrzuć
notification-center-archive = Archiwizuj
notification-center-restore = Przywróć
