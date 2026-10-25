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
