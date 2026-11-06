# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Anuluj
common-close = Zamknij
common-save = Zapisz
common-delete = Usuń
common-remove = Usuń
common-edit = Edytuj
common-add = Dodaj
common-back = Wstecz
common-next = Dalej
common-done = Gotowe
common-apply = Zastosuj
common-confirm = Potwierdź
common-retry = Ponów
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = Jeśli problem się powtarza, skontaktuj się z administratorem.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Ta aplikacja nie została skonfigurowana dla żadnego serwera. Skontaktuj się z administratorem.
common-copy = Kopiuj
common-search = Szukaj
common-more = Więcej
common-yes = Tak
common-no = Nie

# -- Generic states ----------------------------------------------------------
common-loading = Wczytywanie…
common-saving = Zapisywanie…
common-empty = Na razie nic tu nie ma
common-error = Coś poszło nie tak
common-error-network = Nie można połączyć się z serwerem. Sprawdź połączenie i spróbuj ponownie.
common-offline = Brak połączenia z siecią

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Łączenie…
common-conn-degraded = Aktualizacje na żywo wstrzymane — ponowne łączenie
common-conn-offline = Brak połączenia z siecią — zmiany zostaną zsynchronizowane po ponownym połączeniu
common-conn-auth-expired = Sesja wygasła — zaloguj się ponownie
common-conn-reconnect = Połącz ponownie
common-conn-sign-in-again = Zaloguj się ponownie
common-conn-reconnected = Połączono ponownie

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Nowa wiadomość
common-notify-no-subject = (brak tematu)

# -- Generic labels ----------------------------------------------------------
common-required = Wymagane
common-optional = Opcjonalne

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Otwórz foldery i aplikacje
common-nav-close = Zamknij foldery i aplikacje

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Załączniki
common-attach-search = Szukaj załączników
common-attach-search-placeholder = filename:raport type:pdf larger:1mb from:alice
common-attach-filter-type = Filtruj według typu
common-attach-loading = Wczytywanie załączników…
common-attach-empty = Brak pasujących załączników.
common-attach-cat-all = wszystkie
common-attach-cat-image = obraz
common-attach-cat-pdf = pdf
common-attach-cat-text = tekst
common-attach-cat-audio = audio
common-attach-cat-video = wideo
common-attach-cat-other = inne

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = Pokaż hasło
common-password-hide = Ukryj hasło

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = Poczta
common-module-calendar = Kalendarz
common-module-tasks = Zadania
common-module-notes = Notatki
common-module-contacts = Kontakty
common-module-keys = Klucze

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = Wczytywanie podglądu…
common-viewer-loading-audio = Wczytywanie dźwięku…
common-viewer-loading-image = Wczytywanie obrazu…
common-viewer-loading-video = Wczytywanie wideo…
common-viewer-rendering-pdf = Renderowanie PDF…
common-viewer-pdf-failed = Nie udało się wyrenderować tego pliku PDF.
common-viewer-no-preview = Brak podglądu dla tego typu pliku.
common-viewer-unknown-type = nieznany typ
common-viewer-download = Pobierz
common-viewer-attachments = Załączniki

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Ponawianie…
common-waiting-online = Oczekiwanie na połączenie…
common-failure-offline = Brak połączenia. Połącz się ponownie i spróbuj jeszcze raz.
common-failure-network = Nie udało się połączyć z serwerem.
common-failure-server = Serwer nie odpowiedział w wyznaczonym czasie.
common-failure-rate = Zbyt wiele żądań w krótkim czasie. Chwilę poczekaj i spróbuj ponownie.
common-failure-conflict = Ta zmiana została jednocześnie wprowadzona gdzie indziej. Załaduj ponownie i spróbuj jeszcze raz.
common-failure-account = Zalogowane konto uległo zmianie. Załaduj stronę ponownie.
common-failure-linked = Ta opcja jest niedostępna, gdy otwarta jest powiązana skrzynka.
common-failure-refused = Serwer odrzucił zmianę.
common-new-version = Dostępna jest nowa wersja Mailwoman. Zamknięcie tego komunikatu przypomni o niej za godzinę.
common-new-version-reload = Załaduj teraz
common-toast-close = Zamknij powiadomienie
common-toast-seconds = { $seconds } s
common-attach-refresh = Odśwież załączniki
common-attach-open-message = Otwórz wiadomość zawierającą ten załącznik
common-attach-page-filter-note = Filtry dotyczą tej strony. Użyj opcji Dalej, aby przeglądać starsze wiadomości z załącznikami.
common-attach-pages = Strony załączników
common-attach-previous = Wstecz
common-attach-next = Dalej
common-attach-page = Strona { $page }
common-attach-column-name = Plik
common-attach-column-type = Typ
common-attach-column-size = Rozmiar
common-attach-column-sender = Nadawca
common-attach-column-subject = Temat
common-attach-column-date = Odebrano
common-pdf-preview = Podgląd PDF
common-pdf-link-title = Otworzyć link z tego pliku PDF?
common-pdf-link-help = Cel zostanie otwarty w nowej karcie poza Mailwoman.
common-pdf-link-open = Otwórz link
common-open-event = Otwórz wydarzenie
common-message-preview-too-large = Podgląd załączników wiadomości jest ograniczony do 4 MiB. Nadal możesz pobrać oryginalny załącznik.
common-message-preview-failed = Nie udało się wyświetlić podglądu tego załącznika wiadomości. Plik może być nieobsługiwany lub uszkodzony albo proces podglądu jest niedostępny. Nadal możesz pobrać oryginał.
common-message-preview-attachments = Pliki wewnątrz tej wiadomości
common-message-preview-download = Pobierz plik
common-message-preview-size-bytes = { $count } B
common-attach-cat-message = Wiadomości / załączniki Outlooka
common-error-retry-help = Ta sekcja jest chwilowo niedostępna. Spróbuj ponownie, aby ją załadować.
common-error-network-help = Sprawdź połączenie i spróbuj ponownie. Serwer może być chwilowo niedostępny.
common-error-page = Nie udało się załadować tej strony
common-error-page-help = Nie udało się pobrać zasobu strony. Spróbuj ponownie. Jeśli problem się powtarza, załaduj aplikację ponownie, aby uzyskać najnowszą wersję.
common-reload-app = Załaduj aplikację ponownie
common-new-version-reloading = Dostępna jest nowa wersja, trwa ponowne ładowanie…
common-attach-account-changed = Konto zmieniło się podczas wczytywania załączników. Otwórz Załączniki ponownie dla bieżącego konta.
common-attach-search-account = Przeszukaj całe konto
common-attach-scanning = Wyszukiwanie… Sprawdzone strony wiadomości: { $pages } · znalezione pliki: { $count }
common-attach-scan-cancelled = Wyszukiwanie anulowane · znalezione pliki: { $count } na stronach: { $pages }. Wyniki są niepełne.
common-attach-scan-incomplete = Wyszukiwanie przerwane · znalezione pliki: { $count }. Ponów, aby przeszukać całe konto.
common-attach-scan-limited = Wyświetlono pierwsze 1000 pasujących plików. Zawęź filtry, aby znaleźć bardziej konkretne pliki.
common-attach-scan-complete = Wyszukiwanie zakończone · znalezione pliki: { $count } na stronach wiadomości: { $pages }
common-attach-browse-pages = Przeglądaj strony
common-attach-cancel-search = Anuluj wyszukiwanie
multiselect-selected = Wybrano: { $label }
multiselect-remove = Usuń: { $label }
multiselect-no-match = Brak pasujących opcji
common-notification-permission-title = Zezwolić na powiadomienia?
common-notification-permission-help = Otrzymuj powiadomienia o nowych wiadomościach na tym urządzeniu. Możesz to zmienić w Ustawieniach.
common-notification-permission-allow = Zezwól na powiadomienia
common-notification-permission-later = Nie teraz
common-notification-permission-error = Nie udało się włączyć powiadomień. Sprawdź uprawnienia i ustawienia powiadomień, a następnie spróbuj ponownie.
common-notifications-enabled = Powiadomienia zostały włączone na tym urządzeniu.
multiselect-count = { $count ->
    [one] Wybrano 1
    [few] Wybrano { $count }
    [many] Wybrano { $count }
   *[other] Wybrano { $count }
  }
