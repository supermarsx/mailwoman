# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: settings. Source: ../en/settings.ftl

settings-title = Ustawienia
settings-appearance = Wygląd
settings-close = Zamknij ustawienia

settings-theme = Motyw
settings-density = Gęstość
settings-accent = Kolor akcentu
settings-font = Czcionka interfejsu
settings-layout = Układ

# Density options
settings-density-compact = Kompaktowa
settings-density-cozy = Wygodna
settings-density-relaxed = Swobodna

# Interface-font options
settings-font-default = Domyślna
settings-font-system = Systemowa
settings-font-serif = Szeryfowa
settings-font-mono = O stałej szerokości

# Layout options
settings-layout-default = Domyślny
settings-layout-ribbon = Wstążka

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Anuluj
settings-edit = Edytuj
settings-delete = Usuń
settings-save = Zapisz
settings-saved = Zapisano

# Two-factor authentication (S1)
settings-2fa-title = Uwierzytelnianie dwuskładnikowe
settings-2fa-intro = Dodaj drugi składnik, aby przy logowaniu wymagane było coś więcej niż hasło.
settings-2fa-policy-required = Twoja organizacja wymaga drugiego składnika dla tego konta.
settings-2fa-enabled = Włączone
settings-2fa-recovery-title = Kody odzyskiwania
settings-2fa-recovery-once = Zapisz je teraz. Są wyświetlane tylko raz i nie można ich ponownie pobrać. Każdy kod działa jeden raz.
settings-2fa-recovery-ack = Zapisałem(-am) te kody
settings-2fa-totp-title = Aplikacja uwierzytelniająca
settings-2fa-totp-enrol = Skonfiguruj aplikację uwierzytelniającą
settings-2fa-totp-scan = Dodaj ten sekret do aplikacji uwierzytelniającej, a następnie wpisz wyświetlony w niej kod.
settings-2fa-totp-uri-link = Otwórz w aplikacji uwierzytelniającej
settings-2fa-totp-code-label = Kod z aplikacji uwierzytelniającej
settings-2fa-totp-confirm = Potwierdź
settings-2fa-totp-disable = Usuń aplikację uwierzytelniającą
settings-2fa-passkey-title = Klucze dostępu
settings-2fa-passkey-remove = Usuń
settings-2fa-passkey-unsupported = Ta przeglądarka nie obsługuje kluczy dostępu.
settings-2fa-passkey-label-placeholder = Nazwa klucza dostępu (opcjonalnie)
settings-2fa-passkey-add = Dodaj klucz dostępu
settings-2fa-recovery-remaining = { $count ->
        [one] Pozostał { $count } kod odzyskiwania
        [few] Pozostały { $count } kody odzyskiwania
        [many] Pozostało { $count } kodów odzyskiwania
       *[other] Pozostało { $count } kodu odzyskiwania
    }
settings-2fa-recovery-regenerate = Wygeneruj nowe kody odzyskiwania
settings-2fa-recovery-code-label = Kod odzyskiwania
settings-2fa-error-generic = To się nie udało. Spróbuj ponownie.
settings-2fa-error-code = Ten kod nie został zweryfikowany.
settings-2fa-error-passkey = Nie udało się zarejestrować klucza dostępu.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Uwierzytelnianie dwuskładnikowe
settings-2fa-challenge-intro = Potwierdź drugi składnik, aby dokończyć logowanie.
settings-2fa-challenge-failed = Weryfikacja nie powiodła się. Spróbuj ponownie.
settings-2fa-challenge-use-passkey = Użyj klucza dostępu
settings-2fa-challenge-method = Wybierz metodę
settings-2fa-challenge-totp-tab = Kod z aplikacji uwierzytelniającej
settings-2fa-challenge-recovery-tab = Kod odzyskiwania
settings-2fa-challenge-verify = Zweryfikuj

# Active sessions (S11)
settings-sessions-title = Aktywne sesje
settings-sessions-intro = Sesje obecnie zalogowane na to konto.
settings-sessions-last-seen = Ostatnia aktywność: { $when }
settings-sessions-revoke = Wyloguj
settings-sessions-current = Ta sesja
settings-sessions-revoke-others = { $count ->
        [one] Wyloguj { $count } inną sesję
        [few] Wyloguj { $count } inne sesje
        [many] Wyloguj { $count } innych sesji
       *[other] Wyloguj { $count } innej sesji
    }
settings-sessions-error = Nie udało się zmienić sesji.

# Signatures (W12)
settings-sig-title = Podpisy
settings-sig-intro = Tekst dołączany do wysyłanych wiadomości. Jeden podpis może być domyślny.
settings-sig-default = Domyślny
settings-sig-new = Nowy podpis
settings-sig-name-label = Nazwa
settings-sig-body-label = Podpis
settings-sig-default-label = Użyj jako domyślnego
settings-sig-error-name = Nadaj podpisowi nazwę.
settings-sig-error-generic = Nie udało się zapisać podpisu.

# Identities
settings-ident-title = Tożsamości
settings-ident-intro = Nazwa i adres, pod którymi wysyłasz wiadomości. Każda tożsamość może mieć własny podpis.
settings-ident-new = Nowa tożsamość
settings-ident-name-label = Wyświetlana nazwa
settings-ident-email-label = Adres e-mail
settings-ident-replyto-label = Adres do odpowiedzi (opcjonalnie)
settings-ident-signature-label = Domyślny podpis
settings-ident-signature-none = Brak
settings-ident-error-fields = Wpisz nazwę i prawidłowy adres e-mail.
settings-ident-error-generic = Nie udało się zapisać tożsamości.

# Notification rules + quiet hours (W15)
settings-notif-title = Powiadomienia
settings-notif-intro = Wybierz, o których nowych wiadomościach chcesz być powiadamiany(-a) i kiedy zachować ciszę.
settings-notif-enabled-label = Pokazuj powiadomienia o nowej poczcie
settings-notif-quiet-title = Godziny ciszy
settings-notif-quiet-enabled-label = Wyciszaj powiadomienia w ustalonych godzinach
settings-notif-quiet-start = Od
settings-notif-quiet-end = Do
settings-notif-rules-title = Reguły
settings-notif-rule-match-placeholder = Nadawca, folder lub temat zawiera…
settings-notif-rule-action-label = Działanie
settings-notif-rule-notify = Powiadamiaj
settings-notif-rule-mute = Wycisz
settings-notif-rule-add = Dodaj regułę
settings-notif-error = Nie udało się zapisać ustawień powiadomień.

# Saved searches → search folders (W13)
settings-search-title = Zapisane wyszukiwania
settings-search-intro = Pokaż zapisane wyszukiwanie jako folder na liście folderów.
settings-search-empty = Nie masz jeszcze zapisanych wyszukiwań.
settings-search-as-folder = Pokaż { $name } jako folder
settings-search-as-folder-label = Pokaż jako folder
settings-search-error = Nie udało się zmienić zapisanego wyszukiwania.

# Keyboard shortcuts (W14)
settings-kbd-title = Skróty klawiszowe
settings-kbd-intro = Wybierz zestaw skrótów. Zmiany obowiązują na tym urządzeniu.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Wyłączone (bez skrótów)
settings-kbd-action-compose = Nowa wiadomość
settings-kbd-action-archive = Archiwizuj
settings-kbd-action-reply = Odpowiedz
settings-kbd-action-next = Następna wiadomość
settings-kbd-action-previous = Poprzednia wiadomość
settings-kbd-action-search = Szukaj

# Offline cache (W16)
settings-offline-title = Pamięć podręczna offline
settings-offline-intro = Ile poczty przechowywać na tym urządzeniu do użytku offline i jak odzyskiwać miejsce.
settings-offline-budget-label = Limit pamięci podręcznej (MB)
settings-offline-retention-label = Przechowuj przez (dni)
settings-offline-strategy-label = Po osiągnięciu limitu
settings-offline-lru = Usuwaj najdawniej używane
settings-offline-oldest = Usuwaj najpierw najstarsze
settings-offline-manual = Tylko po ręcznym wyczyszczeniu
settings-offline-purge = Wyczyść teraz pamięć podręczną offline

# Interface direction (W20)
settings-dir-title = Kierunek interfejsu
settings-dir-intro = Zgodnie z językiem lub wymuszony kierunek. Układ od prawej do lewej odwraca rozmieszczenie elementów.
settings-dir-auto = Automatyczny
settings-dir-ltr = Od lewej do prawej
settings-dir-rtl = Od prawej do lewej
settings-dir-preview-title = Podgląd
settings-dir-preview-body = Ten tekst jest odwracany, gdy kierunek jest ustawiony od prawej do lewej.
