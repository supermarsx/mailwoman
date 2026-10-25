# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Zarządzanie kluczami
keys-title = Klucze i certyfikaty
keys-subtitle = Klucze OpenPGP i S/MIME. Klucze prywatne pozostają na tym urządzeniu i nigdy nie trafiają na serwer.
keys-generate = Wygeneruj klucz
keys-import = Importuj klucz

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Twoje klucze
keys-empty-own = Brak kluczy. Wygeneruj lub zaimportuj klucz, aby zacząć.
keys-contact-keys = Klucze kontaktów
keys-empty-contact = Brak kluczy kontaktów. Wyszukaj klucz poniżej.
keys-loading = Ładowanie kluczy…
keys-select-prompt = Wybierz klucz, aby go wyświetlić i zweryfikować.
keys-untitled = Klucz { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = niezweryfikowany
keys-trust-tofu = TOFU
keys-trust-verified = zweryfikowany
keys-trust-revoked = unieważniony

# -- Key detail --------------------------------------------------------------
keys-key-card = Klucz { $name }
keys-fingerprint = Odcisk palca
keys-safe-words = Słowa kontrolne
keys-safe-words-help = Odczytaj je na głos razem z kontaktem, aby potwierdzić klucz innym kanałem.
keys-safe-words-list = Słowa kontrolne odcisku palca
keys-scan-label = Zeskanuj, aby zweryfikować
keys-qr-label = Kod QR odcisku palca
keys-autocrypt = Autocrypt
keys-autocrypt-on = Ogłaszany w nagłówkach Autocrypt
keys-autocrypt-off = Nieogłaszany przez Autocrypt
keys-trust-label = Zaufanie
keys-trust-level = Poziom zaufania
keys-associate-label = Powiąż z kontaktem
keys-associate-select = Kontakt do powiązania
keys-choose-contact = Wybierz kontakt…
keys-associate-button = Powiąż
keys-backup-label = Kopia zapasowa
keys-backup-help = Wyeksportuj wiadomość konfiguracyjną Autocrypt (Autocrypt Setup Message), aby przenieść ten klucz na inne urządzenie.
keys-export-backup = Eksportuj kopię zapasową
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Na tym urządzeniu nie ma klucza prywatnego, z którego można utworzyć kopię zapasową

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Certyfikat S/MIME
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = Ten certyfikat jest samopodpisany: został podpisany własnym kluczem, a nie przez urząd certyfikacji. Programy pocztowe innych osób nie będą mu ufać, dopóki ręcznie go nie zaakceptują lub dopóki urząd certyfikacji nie wystawi certyfikatu na podstawie poniższego pliku żądania.
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = Ważny do { $date }.
keys-smime-not-in-app = Mailwoman nie podpisuje ani nie odszyfrowuje jeszcze poczty kluczem S/MIME. Wyeksportuj klucz, aby używać go w innym programie pocztowym.
keys-smime-passphrase = Hasło tego klucza S/MIME
keys-smime-export-help = Plik .p12 zawiera certyfikat i klucz prywatny. Klucz jest zaszyfrowany powyższym hasłem i to samo hasło otwiera plik w innym programie. Plik .csr nie zawiera klucza prywatnego.
keys-smime-export = Eksportuj jako PKCS#12 (.p12)
keys-smime-request = Pobierz żądanie certyfikatu (.csr)
keys-smime-issued-label = Certyfikat wystawiony dla tego klucza (.pem, .crt, .cer)
keys-smime-issued-file = Plik wystawionego certyfikatu
keys-smime-issued-help = Zastępuje certyfikat przechowywany dla tego klucza. Certyfikat utworzony dla innego klucza zostanie odrzucony.
keys-smime-attach = Importuj wystawiony certyfikat
keys-smime-failed = To się nie udało. Sprawdź hasło.
keys-smime-attach-failed = Certyfikat nie został zaimportowany. Sprawdź hasło oraz to, czy plik zawiera jeden certyfikat wystawiony dla tego klucza.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Wyszukaj klucz kontaktu
keys-lookup-heading = Wyszukaj klucz
keys-lookup-address = Adres do wyszukania
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Źródła wyszukiwania
keys-source = Źródło { $source }
keys-consent-label = Zgoda na wyszukiwanie zewnętrzne
keys-consent-text = Wyszukiwanie klucza łączy się z zewnętrznymi katalogami (WKD/VKS). Mam zgodę tej osoby na takie wyszukiwanie.
keys-lookup-button = Wyszukaj
keys-looking-up = Wyszukiwanie…
keys-lookup-found = Znalezione klucze: { $count } — dodano do kluczy kontaktów.
keys-lookup-none = Nie znaleziono klucza.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Wygeneruj klucz
keys-type = Typ
keys-key-type = Typ klucza
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = Tworzy 3072-bitowy klucz RSA i certyfikat dla poniższego adresu, ważny przez dwa lata. Certyfikat jest samopodpisany: programy pocztowe innych osób nie będą mu ufać, dopóki ręcznie go nie zaakceptują lub dopóki urząd certyfikacji nie wystawi certyfikatu. Mailwoman nie podpisuje ani nie odszyfrowuje jeszcze nim poczty; później można go wyeksportować do innego programu pocztowego.
keys-name = Nazwa
keys-email = E-mail
keys-key-passphrase = Hasło klucza
keys-passphrase-help = Hasło chroni klucz prywatny na tym urządzeniu. Nigdy nie opuszcza przeglądarki.
keys-cancel = Anuluj
keys-generate-submit = Wygeneruj
keys-generating = Generowanie…
keys-generating-smime = Generowanie klucza RSA w tej przeglądarce. Może to potrwać do minuty.
keys-generate-failed = Nie wygenerowano klucza.
keys-generate-failed-smime = Nie wygenerowano klucza. Certyfikat może zawierać tylko zwykły adres ASCII (nazwa@domena) i nazwę o długości najwyżej 64 znaków.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Importuj klucz
keys-import-type = Typ importu
keys-tab-armored = ASCII armor (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Klucz w formacie ASCII armor
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Hasło (jeśli klucz jest prywatny)
keys-import-passphrase = Hasło importu
keys-pkcs12-file-label = Plik PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Plik PKCS#12
keys-import-password-label = Hasło importu
keys-pkcs12-password = Hasło PKCS#12
keys-import-preview = Podgląd importu
keys-preview = Podgląd
keys-preview-type = Typ: { $kind }
keys-preview-fingerprint-aria = Odcisk palca w podglądzie
keys-preview-fingerprint = Odcisk palca: { $fp }
keys-preview-has-private = Zawiera klucz prywatny (zostanie zapisany na tym urządzeniu)
keys-preview-public-only = Tylko klucz publiczny
keys-import-submit = Importuj
