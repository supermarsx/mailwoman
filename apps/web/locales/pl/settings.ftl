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
settings-mail-features = Funkcje poczty
settings-mail-feature-allowSend = Zezwalaj na wysyłanie wiadomości
settings-mail-feature-allowSend-help = Gdy wyłączone, przycisk Wyślij i jego skrót są wyłączone. Wersje robocze nadal można zapisywać.
settings-mail-feature-allowReply = Zezwalaj na odpowiadanie i odpowiadanie wszystkim
settings-mail-feature-allowReply-help = Gdy wyłączone, akcje Odpowiedz i Odpowiedz wszystkim oraz ich skróty nie są oferowane.
settings-mail-feature-allowForward = Zezwalaj na przekazywanie dalej
settings-mail-feature-allowForward-help = Gdy wyłączone, akcje Przekaż dalej i ich skrót nie są oferowane.
settings-mail-feature-allowDownload = Zezwalaj na pobieranie wiadomości i załączników
settings-mail-feature-allowDownload-help = Gdy wyłączone, pobieranie wiadomości i załączników nie jest oferowane.
settings-mail-feature-allowPrint = Zezwalaj na drukowanie
settings-mail-feature-allowPrint-help = Gdy wyłączone, akcja Drukuj nie jest oferowana.
settings-mail-feature-showSpam = Pokazuj folder Spam
settings-mail-feature-showSpam-help = Gdy wyłączone, folder Spam i wiadomości oznaczone jako spam są ukryte na liście folderów i w wynikach wyszukiwania. Nic nie jest usuwane.
settings-mail-feature-showAuthStatus = Pokazuj wyniki uwierzytelniania nadawcy
settings-mail-feature-showAuthStatus-help = Pokazuje w czytniku wyniki SPF, DKIM i DMARC wiadomości.
settings-mail-feature-closeOnDelete = Zamykaj wiadomość po jej usunięciu
settings-mail-feature-closeOnDelete-help = Zamyka otwartą wiadomość, kartę lub okno, gdy ta wiadomość zostanie usunięta lub przeniesiona do kosza.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Ścieżka ustawień
settings-crumb-loading = Wczytywanie nazwy
settings-crumb-new = Nowy
settings-crumb-edit = Edytuj
settings-badge-color = Kolor znacznika nieprzeczytanych
settings-badge-color-accent = Taki jak kolor akcentu
settings-badge-color-accent-lighter = Jaśniejszy akcent
settings-badge-color-accent-darker = Ciemniejszy akcent
settings-badge-color-option = Znaczniki: { $color }
settings-appearance-theme-card = Motyw i przełączanie
settings-appearance-theme-card-help = Wybierz motyw i to, czy ma zależeć od systemu lub pory dnia.
settings-appearance-colours-card = Kolory
settings-appearance-colours-card-help = Akcent używany dla akcji i zaznaczenia oraz kolor liczników nieprzeczytanych.
settings-appearance-text-card = Tekst i układ
settings-appearance-text-card-help = Jak gęsty jest interfejs, jakiej czcionki używa i jak układana jest poczta.
settings-sig-create-variant = Utwórz wariant
settings-sig-error-existing = Podpis o tej nazwie już istnieje. Wybierz inną nazwę.
settings-sig-save-failed = Nie udało się zapisać podpisu. { $reason }
settings-ident-signature-new = Nowe wiadomości
settings-ident-signature-account = Użyj podpisu konta
settings-ident-signature-replies = Odpowiedzi
settings-ident-signature-inherit = Użyj podpisu nowych wiadomości
settings-ident-signature-unavailable = Niedostępny podpis: { $name }
settings-ident-signature-help = Utwórz warianty podpisu w sekcji Podpisy, a następnie przypisz jeden tutaj. Ten sam wariant można użyć w obu przypadkach.
settings-ident-save-failed = Nie udało się zapisać tożsamości. { $reason }
settings-search-save-failed = Nie udało się zapisać zapisanego wyszukiwania. { $reason }
settings-kbd-preset-title = Zestaw skrótów
settings-editor-discard-title = Odrzucić niezapisane zmiany?
settings-editor-discard-detail = Twoje zmiany nie zostały zapisane. Kontynuuj edycję lub odrzuć ten szkic, aby opuścić edytor.
settings-editor-keep = Kontynuuj edycję
settings-editor-discard = Odrzuć zmiany
settings-editor-missing = Ten element już nie istnieje. Wróć do listy i wybierz inny.
settings-editor-back = Wróć do listy
settings-sig-edit-title = Edytuj podpis
settings-ident-edit-title = Edytuj tożsamość
settings-sections = Sekcje ustawień
settings-category-account = Konto
settings-category-mail = Poczta
settings-category-interface = Interfejs
settings-category-integrations = Integracje
settings-category-about = Informacje
settings-nav-preferences = Klawiatura i wyświetlanie
settings-nav-rules = Reguły poczty
settings-nav-access = Dostęp do API
settings-nav-metadata = Informacje o serwerze
settings-folders-title = Foldery i pasek boczny
settings-folders-help = Przypnij foldery nad pozostałymi, wybierz ikonę i przeciągnij lub użyj przycisków przenoszenia, aby uporządkować foldery w każdej grupie. Te wybory są synchronizowane z tym kontem i nie przenoszą wiadomości ani nie zmieniają nazw folderów.
settings-folders-pin = Przypnij { $name }
settings-folders-icon = Ikona dla { $name }
settings-folders-automatic = Automatyczna rola folderu
settings-folders-drag = Przeciągnij { $name }, aby zmienić kolejność
settings-folders-up = Przesuń { $name } w górę
settings-folders-down = Przesuń { $name } w dół
settings-folders-sync-idle = Zaloguj się, aby synchronizować preferencje folderów.
settings-folders-sync-loading = Wczytywanie preferencji konta…
settings-folders-sync-saving = Zapisywanie preferencji konta…
settings-folders-sync-synced = Preferencje zsynchronizowane z tym kontem.
settings-folders-sync-error = Nie udało się zsynchronizować preferencji. Zmiany pozostają w tej sesji i zostaną ponowione po przywróceniu połączenia.
settings-folder-icon-folder = Folder
settings-folder-icon-star = Gwiazdka
settings-folder-icon-flag = Flaga
settings-folder-icon-tag = Etykieta
settings-folder-icon-clock = Zegar
settings-folder-icon-calendar = Kalendarz
settings-folder-icon-file = Dokument
settings-folder-icon-image = Obraz
settings-folder-icon-lock = Kłódka
settings-folder-icon-users = Ludzie
settings-folder-icon-archive = Archiwum
settings-folder-icon-mail = Koperta
settings-folder-icon-attachment = Załącznik
settings-folder-icon-check = Znacznik wyboru
settings-folder-icon-alert = Alert
settings-search-new = Nowy folder wyszukiwania
settings-search-editor = Folder wyszukiwania
settings-search-editor-help = Wiadomości muszą spełniać wszystkie warunki. Pozostaw warunek pusty, aby go pominąć. Foldery wyszukiwania pokazują pasujące wiadomości bez ich przenoszenia ani kopiowania.
settings-search-name = Nazwa folderu
settings-search-name-required = Wpisz nazwę tego folderu wyszukiwania.
settings-search-invalid = To zapisane wyszukiwanie ma nieprawidłowy filtr. Nie można go bezpiecznie edytować.
settings-search-saved = Zapisano folder wyszukiwania „{ $name }”.
settings-search-field-text = Szukaj w całym tekście wiadomości
settings-search-field-from = Nadawca zawiera
settings-search-field-to = Odbiorca zawiera
settings-search-field-subject = Temat zawiera
settings-search-field-body = Treść wiadomości zawiera
settings-search-field-filename = Nazwa pliku załącznika zawiera
settings-search-field-inMailbox = Ogranicz do identyfikatora skrzynki (opcjonalnie)
settings-search-field-after = Odebrane po (data i godzina UTC)
settings-search-field-before = Odebrane przed (data i godzina UTC)
settings-search-attachment = Załączniki
settings-search-any = Dowolna wiadomość
settings-search-with-attachments = Z załącznikami
settings-search-without-attachments = Bez załączników
settings-search-unread = Tylko nieprzeczytane wiadomości
settings-search-flagged = Tylko oflagowane wiadomości
settings-sessions-search = Szukaj konta, urządzenia lub adresu IP
settings-sessions-account = Konto
settings-sessions-device = Urządzenie i klient
settings-sessions-location = IP i ostatni znany kraj
settings-sessions-location-unavailable = Kraj niedostępny
settings-sessions-activity = Aktywność
settings-sessions-actions = Akcje
settings-sessions-revoke-device = Wyloguj { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Pisanie i odpowiedzi
settings-quote-style = Styl nagłówka odpowiedzi i przekazania
settings-quote-classic = Klasyczna odpowiedź z cytatem
settings-quote-outlook = Nagłówek w stylu Outlooka
settings-quote-help = Styl Outlooka dodaje pola Od, Wysłano, Do, DW i Temat nad oryginalną wiadomością w tekście sformatowanym i zwykłym. Istniejące wersje robocze nie zmieniają się.
settings-external-warning = Ostrzeżenia o odbiorcach zewnętrznych
settings-external-off = Wyłączone
settings-external-warn = Pokaż ostrzeżenie
settings-external-confirm = Wymagaj potwierdzenia przed wysłaniem
settings-external-help = Sprawdź pola Do, DW i UDW pod kątem domeny wybranej tożsamości i listy zaufanych domen. Chroni to wysyłanie z tego edytora; nie jest to reguła dostarczania na serwerze.
settings-trusted-domains = Zaufane domeny odbiorców
settings-trusted-domains-help = Wpisz do 100 dokładnych domen, po jednej w wierszu. Subdomeny trzeba wymienić osobno. Domena samej tożsamości nadawcy jest zawsze zaufana. Adresy URL, adresy e-mail i symbole wieloznaczne nie są akceptowane.
settings-trusted-domains-invalid = Wpisz do 100 prawidłowych nazw domen, bez adresów URL, adresów e-mail i symboli wieloznacznych.
settings-trusted-domains-save = Zapisz zaufane domeny
settings-shortcuts-enabled = Włącz skróty aplikacji
settings-shortcuts-action = Akcja
settings-shortcuts-binding = Przypisanie
settings-shortcuts-edit = Edytuj skrót
settings-shortcuts-edit-action = Edytuj skrót: { $action }
settings-shortcuts-help = Użyj „inherit”, aby dziedziczyć wybrany zestaw, „disabled”, aby nie mieć skrótu, lub kombinacji, takiej jak Ctrl+Shift+K. Ctrl oznacza Cmd w macOS. Kolidujące przypisania są nieaktywne. Pola tekstowe i okna dialogowe zachowują zwykłe klawisze; kombinacje zarezerwowane przez przeglądarkę mogą być niedostępne.
settings-shortcuts-invalid = Wpisz inherit, disabled lub prawidłowy pojedynczy klawisz z opcjonalnymi modyfikatorami Ctrl, Alt i Shift.
settings-shortcuts-conflict = Dwie akcje używają tego samego skrótu. Zmień jedno przypisanie; kolidujące skróty nie będą działać.
settings-shortcuts-discard = Odrzucić niezapisaną zmianę skrótu?
settings-shortcuts-save = Zapisz skrót
settings-shortcuts-back = Wróć do skrótów
settings-shortcuts-off = Zestaw Wyłączony wyłącza wszystkie skróty, także własne przypisania. Wybierz inny zestaw w Preferencjach, aby używać tych przypisań.
settings-kbd-action-replyAll = Odpowiedz wszystkim
settings-kbd-action-forward = Przekaż dalej
settings-kbd-action-refresh = Odśwież wiadomości
settings-kbd-action-settings = Otwórz ustawienia
settings-kbd-action-trash = Przenieś do Kosza
settings-kbd-action-markRead = Oznacz jako przeczytane
settings-kbd-action-markUnread = Oznacz jako nieprzeczytane
settings-kbd-action-spam = Oznacz jako spam
settings-kbd-action-zoomIn = Powiększ wiadomość
settings-kbd-action-zoomOut = Pomniejsz wiadomość
settings-kbd-action-zoomReset = Zresetuj powiększenie wiadomości
settings-kbd-action-renameFolder = Zmień nazwę folderu
settings-kbd-action-openInTab = Otwórz wiadomość w osobnej karcie
settings-notif-device-title = Uprawnienie na tym urządzeniu
settings-notif-device-help = Preferencje powiadomień należą do Twojego konta. Uprawnienie przeglądarki należy do tego urządzenia; Mailwoman pyta tylko wtedy, gdy zdecydujesz się je przyznać.
settings-notif-ask-permission = Proponuj uprawnienie do powiadomień na nowych urządzeniach
settings-notif-ask-permission-help = Pokaż prośbę o uprawnienie, gdy to urządzenie nie zezwoliło na powiadomienia ani ich nie zablokowało. Domyślnie włączone; ta preferencja zapisuje się od razu.
settings-notif-permission-default = Nie poproszono jeszcze o uprawnienie przeglądarki.
settings-notif-permission-granted = Powiadomienia przeglądarki są dozwolone na tym urządzeniu.
settings-notif-permission-denied = Powiadomienia są zablokowane. Zmień uprawnienie tej witryny w ustawieniach przeglądarki, aby je zezwolić.
settings-notif-permission-unsupported = Ta przeglądarka nie obsługuje tutaj powiadomień systemowych.
settings-notif-permission-request = Zezwól na powiadomienia na tym urządzeniu
settings-notif-permission-error = Przeglądarka nie mogła poprosić o uprawnienie do powiadomień.
settings-notif-timezone = Strefa czasowa godzin ciszy (IANA)
settings-notif-timezone-device = Użyj strefy czasowej tego urządzenia
settings-permissions-background-title = Powiadomienia w tle w tej przeglądarce
settings-permissions-background-help = Najpierw zapisz preferencje powiadomień i zezwól na powiadomienia przeglądarki. Zarejestruj tę przeglądarkę, aby otrzymywać aktywność konta, gdy aplikacja jest zamknięta.
settings-permissions-push-unsupported = Web Push w tle jest niedostępny w tej przeglądarce.
settings-permissions-push-registered = Ta przeglądarka ma subskrypcję push. Zarejestruj ją, aby powiązać z bieżącym kontem.
settings-permissions-push-unregistered = Ta przeglądarka nie ma subskrypcji push.
settings-permissions-push-register = Zarejestruj powiadomienia w tle
settings-permissions-push-remove = Usuń subskrypcję przeglądarki
settings-permissions-push-policy = Administrator nie zezwala na powiadomienia push w tle.
settings-permissions-storage-title = Magazyn przeglądarki
settings-permissions-storage-help = Trwały magazyn chroni lokalne dane tej witryny przed automatycznym usunięciem, jeśli przeglądarka go przyzna. Dotyczy tylko tego urządzenia i nie zmienia retencji na serwerze. Dane witryny można wyczyścić w ustawieniach przeglądarki; niewysłane zmiany offline mogą zostać utracone.
settings-permissions-storage-unsupported = Żądania trwałego magazynu są niedostępne w tej przeglądarce.
settings-permissions-storage-persistent = Przeglądarka przyznała tej witrynie trwały magazyn.
settings-permissions-storage-evictable = Przeglądarka może automatycznie usunąć lokalne dane tej witryny.
settings-permissions-storage-usage = Bieżące użycie magazynu witryny: { $mib } MiB.
settings-permissions-storage-request = Poproś o trwały magazyn
settings-permissions-storage-policy = Administrator nie zezwala na żądania trwałego magazynu.
settings-permissions-error = Nie udało się ukończyć akcji w przeglądarce. Sprawdź zapisane ustawienia konta i uprawnienia przeglądarki dla tej witryny.
settings-notif-background-rules = Reguły treści są oceniane, gdy Mailwoman jest otwarty. Systemowe powiadomienia w tle są wstrzymane, gdy skonfigurowano jakiekolwiek reguły treści, ponieważ nieprzejrzyste wybudzenie w tle nie zawiera szczegółów wiadomości.
settings-nav-regional = Ustawienia regionalne
settings-time-title = Strefy czasowe
settings-time-help = Wybierz, jak wyświetlane są godziny wiadomości, oraz opcjonalną strefę porównawczą do planowania. Lokalizacja nigdy nie jest wnioskowana z adresów e-mail.
settings-time-displayZone = Strefa czasowa wyświetlania
settings-time-displayZone-help = Użyj „local” dla tego urządzenia lub nazwy IANA, takiej jak Europe/Lisbon.
settings-time-comparisonZone = Strefa czasowa porównawcza
settings-time-comparisonZone-help = Wpisz nazwę IANA, taką jak America/New_York, lub pozostaw puste, aby ukryć porównania.
settings-time-helpers = Pokaż pomoce dotyczące stref czasowych
settings-time-save = Zapisz strefę czasową
settings-time-invalid = Wpisz prawidłową strefę czasową IANA. „local” stosuj tylko dla strefy wyświetlania lub pozostaw strefę porównawczą pustą.
settings-time-discard = Odrzucić niezapisane zmiany stref czasowych?
settings-folders-name = Folder
settings-folders-actions = Akcje
settings-folders-edit = Edytuj wygląd: { $name }
settings-folders-icon-help = Wybierz jedną z pięćdziesięciu tematycznych ikon lub pozostaw automatyczny wybór na podstawie przeznaczenia folderu. Dotyczy Twojego konta i nie zmienia nazwy skrzynki.
settings-folders-save = Zapisz wygląd folderu
settings-folders-back = Wróć do folderów
settings-folders-discard = Odrzucić niezapisane zmiany wyglądu folderu?
settings-folder-labels = Etykiety folderu
settings-folder-labels-help = Uporządkuj ten folder za pomocą maksymalnie 32 etykiet konta. Nie oznacza to etykietami ani nie przenosi wiadomości w nim zawartych. Nazwy i kolory etykiet zarządzaj w sekcji Etykiety.
settings-folders-leave-pending = Te zmiany nadal czekają w kolejce na synchronizację. Opuścić ten edytor?
settings-folders-retry = Ponów zapis
settings-folder-role-title = Foldery specjalne
settings-folder-role-help = Wybierz rzeczywisty folder używany dla każdej roli poczty. Automatycznie korzysta z wykrywania przez serwer. Użyj ustawienia administratora usuwa Twoje nadpisanie. Kolidujące lub brakujące miejsca docelowe wyłączają tę rolę zamiast wybierać inny folder.
settings-folder-role-auto = Wykrywanie automatyczne
settings-folder-role-inherit = Użyj ustawienia administratora
settings-folder-role-path = Skonfigurowana ścieżka: { $path }
settings-folder-role-missing = Skonfigurowany folder jest niedostępny
settings-folder-role-conflict = Ten folder docelowy nie istnieje lub koliduje z inną rolą. Ta rola jest niedostępna, dopóki mapowanie nie zostanie poprawione.
settings-folder-role-refresh-error = Zapisano, ale nie udało się odświeżyć listy folderów. Odśwież foldery przed użyciem nowego mapowania.
settings-folder-role-unsupported = Te nadpisania wymagają natywnych folderów poczty. Konta JMAP z serwera nadrzędnego zachowują role folderów swojego serwera.
settings-folder-role-inbox = Odebrane
settings-folder-role-drafts = Wersje robocze
settings-folder-role-sent = Wysłane
settings-folder-role-trash = Kosz
settings-folder-role-junk = Spam
settings-folder-role-archive = Archiwum
settings-sessions-select = Zaznacz
settings-sessions-select-device = Zaznacz sesję dla { $device }
settings-sessions-select-visible = Zaznacz wszystkie widoczne inne sesje
settings-sessions-current-protected = Ta sesja pozostaje zalogowana i nie można jej tu zaznaczyć. Użyj opcji Wyloguj, aby ją zakończyć.
settings-sessions-selected-title = Wyloguj zaznaczone sesje ({ $count })
settings-sessions-selected-confirm = Zakończ tylko zaznaczone inne sesje. Ta sesja pozostaje zalogowana. Nieudane lub pominięte żądania zostaną zgłoszone bez automatycznych ponowień.
settings-sessions-selected-action = Wyloguj zaznaczone sesje
settings-sessions-cancel = Anuluj
settings-sessions-selected-result = Wylogowano: { $revoked }. Już zakończone: { $unchanged }. Nie udało się potwierdzić: { $failed }. Nie podjęto próby: { $skipped }.
settings-sessions-refresh-failed = Nie udało się odświeżyć listy sesji. Wczytaj ją ponownie przed zaznaczeniem kolejnych sesji.
settings-sig-format = Format podpisu
settings-sig-format-plain = Zwykły tekst
settings-sig-format-html = Tekst sformatowany (HTML)
settings-sig-html-help = HTML jest ograniczony do 2 MiB i oczyszczany przed zapisem i wysłaniem. Obrazy zdalne i zawartość aktywna są usuwane.
settings-sig-html-source = Edytuj źródło HTML
settings-sig-editor-loading = Wczytywanie edytora podpisu…
settings-sig-rich-body = Podpis sformatowany
settings-sig-plain-alternative = Alternatywa w postaci zwykłego tekstu
settings-sig-plain-help = Wiadomości w zwykłym tekście używają tego tekstu. Zmiany w edytorze sformatowanym aktualizują go, dopóki go nie dostosujesz; edycje źródła HTML zachowują Twoją alternatywę.
settings-sig-account-changed = Zalogowane konto zostało zmienione. Otwórz ten podpis ponownie na właściwym koncie.
settings-sig-image-type = { $name } nie jest obrazem PNG, JPEG, GIF ani WebP.
settings-sig-image-size = { $name } się nie mieści: HTML podpisu wraz z obrazami jest ograniczony do 2 MiB.
settings-sig-too-large = Ten podpis ma ponad 2 MiB kodu HTML. Usuń obraz lub część formatowania.
settings-sig-images-help = Obrazy są przechowywane w podpisie i wysyłane jako obrazy wbudowane w każdej wiadomości.
settings-folder-icon-inbox = Odebrane
settings-folder-icon-sent = Wysłane
settings-folder-icon-edit = Edytuj
settings-folder-icon-delete = Usuń
settings-folder-icon-spam = Spam
settings-folder-icon-search = Szukaj
settings-folder-icon-pin = Pinezka
settings-folder-icon-link = Link
settings-folder-icon-shield = Tarcza
settings-folder-icon-unlock = Otwarta kłódka
settings-folder-icon-fileAudio = Audio
settings-folder-icon-fileVideo = Wideo
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Kod
settings-folder-icon-print = Drukuj
settings-folder-icon-download = Pobierz
settings-folder-icon-preferences = Preferencje
settings-folder-icon-dashboard = Pulpit
settings-folder-icon-analytics = Analityka
settings-folder-icon-nextcloud = Pliki w chmurze
settings-folder-icon-domains = Domeny
settings-folder-icon-twofactor = Uwierzytelnianie
settings-folder-icon-assistant = Asystent
settings-folder-icon-integrations = Integracje
settings-folder-icon-logs = Dzienniki
settings-folder-icon-home = Dom
settings-folder-icon-briefcase = Teczka
settings-folder-icon-heart = Serce
settings-folder-icon-shopping = Zakupy
settings-folder-icon-education = Edukacja
settings-folder-icon-travel = Podróże
settings-folder-icon-coffee = Kawa
settings-folder-icon-gift = Prezent
settings-folder-icon-bank = Bank
settings-folder-icon-book = Książka
settings-nav-keyboard-display = Klawiatura i wyświetlanie
settings-2fa-passkey-policy-disabled = Administrator wyłączył logowanie i rejestrację kluczy dostępu dla tego konta. Istniejące dane uwierzytelniające pozostają na liście, abyś mógł nimi zarządzać lub je usunąć.
settings-mail-interactions = Zaznaczanie i przenoszenie wiadomości
settings-mail-modifier-selection = Zaznaczaj wiadomości klawiszami modyfikującymi
settings-mail-modifier-selection-help = Ctrl lub Command przełącza pojedyncze wiadomości. Shift zaznacza zakres, a Shift ze strzałkami go rozszerza. Widoczny przycisk zaznaczania pozostaje dostępny, gdy ta opcja jest wyłączona.
settings-mail-drag-folder = Przeciągaj wiadomości do folderów
settings-mail-drag-folder-help = Przeciągnij zaznaczoną wiadomość, aby przenieść zaznaczenie, lub niezaznaczoną, aby przenieść tylko ją. Akceptowane są tylko foldery tego konta. Użytkownicy dotyku i klawiatury mogą użyć paska narzędzi zaznaczenia.
settings-search-sections = Szukaj w ustawieniach

# -- Machine translation (2026-11): added to match en ---------------------
settings-search-no-sections = Brak pasujących sekcji ustawień.
settings-time-local = Strefa czasowa urządzenia
settings-time-none = Brak strefy czasowej do porównania
settings-folder-counts = Liczniki folderów
settings-folder-counts-none = Ukryj liczniki
settings-folder-counts-unread = Wiadomości nieprzeczytane
settings-folder-counts-total = Wszystkie wiadomości
settings-folder-counts-both = Nieprzeczytane i wszystkie wiadomości
settings-folder-counts-unreadIfAny = Wiadomości nieprzeczytane, tylko gdy są
settings-folder-count-scope = Foldery z licznikami
settings-folder-count-scope-all = Wszystkie foldery
settings-folder-count-scope-main = Tylko foldery główne
settings-folder-count-scope-help = Foldery główne to Odebrane, Wersje robocze, Wysłane, Archiwum, Spam, Kosz, Skrzynka nadawcza i inne foldery specjalne. Foldery własne i ich podfoldery nie pokazują licznika.
settings-account-unread-badge = Pokaż liczbę nieprzeczytanych obok nazwy skrzynki
settings-folder-counts-compact = Mniejsze znaczniki poza folderem Odebrane
settings-focused-inbox = Skrzynka priorytetowa
settings-focused-inbox-help = Podziel folder Odebrane na karty Priorytetowe i Pozostałe nad listą. Wiadomości z biuletynów i od nadawców automatycznych trafiają do Pozostałych. Inne foldery i wyniki wyszukiwania pokazują wszystkie wiadomości.
settings-focused-senders-title = Zarządzaj nadawcami priorytetowymi
settings-focused-senders-help = Nadawcy, których wiadomości zawsze przenosisz do Priorytetowych lub Pozostałych. Usuń nadawcę, aby o sortowaniu znów decydował system.
settings-focused-senders-empty = Brak nadawców. Użyj opcji Zawsze przenoś do Priorytetowych lub Zawsze przenoś do Pozostałych przy wiadomości.
settings-focused-senders-to-focused = Zawsze Priorytetowe
settings-focused-senders-to-other = Zawsze Pozostałe
settings-focused-senders-remove = Usuń
settings-focused-senders-remove-label = Usuń: { $sender }
settings-focused-senders-remove-failed = Nie udało się usunąć nadawcy. Spróbuj ponownie.
settings-folder-counts-reset = Dziedzicz liczniki folderów
settings-folder-counts-reset-size = Dziedzicz rozmiar znacznika
settings-mail-resume-tabs = Przywracaj otwarte karty
settings-mail-resume-tabs-help = Przywróć odwołania do kart konta po ponownym otwarciu Mailwoman. Treści wiadomości i adresy URL załączników nie są zapisywane w rekordzie przestrzeni roboczej.
settings-row-actions-mode = Szybkie akcje na liście wiadomości
settings-row-actions-mode-open = Zawsze widoczne
settings-row-actions-mode-hover = Pokaż po najechaniu
settings-row-actions-mode-click = Pokaż po kliknięciu
settings-row-actions-mode-off = Wyłączone
settings-row-actions-mode-help = Sposób wyświetlania szybkich akcji w wierszu wiadomości. Po najechaniu pokazuje trzy kontrolki i przycisk Więcej; po kliknięciu pokazuje jeden przycisk, który je otwiera; na ekranach dotykowych najechanie działa jak kliknięcie. Wyłączone usuwa je z wierszy (menu kontekstowe i pasek narzędzi nadal zawierają każdą akcję).
settings-mail-feature-showGoToTop = Pokaż przycisk Na górę na liście wiadomości
settings-mail-feature-showGoToTop-help = Dodaje przycisk przewijający listę wiadomości do pierwszej wiadomości. Domyślnie wyłączone.
settings-mail-feature-showCreateFolderButton = Pokaż przycisk Utwórz folder obok Nowej wiadomości
settings-mail-feature-showCreateFolderButton-help = Dodaje na pasku bocznym przycisk z samą ikoną, który tworzy folder. Domyślnie wyłączone.
settings-draft-save = Wersje robocze konta
settings-draft-save-help = Zapisuj w folderze Wersje robocze konta, aby mieć dostęp na innym urządzeniu. Zapisywanie jest ciche; powiadomienie pojawia się tylko przy błędach. Nieobsługiwane konta i chroniona odszyfrowana treść nigdy nie są wysyłane po cichu.
settings-draft-save-mode = Zapisuj automatycznie
settings-draft-realtime = Po przerwie w pisaniu
settings-draft-periodic = W regularnych odstępach
settings-draft-manual = Tylko po wybraniu Zapisz
settings-draft-debounce = Przerwa w pisaniu (milisekundy)
settings-draft-interval = Odstęp zapisu (sekundy)
settings-permissions-push-other-registration = Ta przeglądarka ma subskrypcję niezarejestrowaną na tym koncie. Usuń ją przed ponowną rejestracją.
settings-show-from = Pokazuj pole Od podczas pisania
settings-show-from-help = Zaczynaj z widocznym polem Od. Możesz też pokazać je dla jednej wiadomości na wstążce. Akceptowane są tylko autoryzowane adresy nadawcy.
settings-sessions-table = Tabela aktywnych sesji
settings-folders-table = Lista folderów
settings-kbd-reference = Zobacz powiązania wstępnie ustawione
settings-kbd-reference-help = Tylko odniesienie do ustawień wstępnych. Własne powiązania i zasady administratora mogą zmieniać aktywne skróty.
settings-sessions-client-details = Szczegóły klienta
settings-2fa-passkey-rules = Twoja organizacja akceptuje tylko klucze dostępu, które:
settings-2fa-passkey-rule-platform = są przechowywane na tym urządzeniu (Windows Hello, Touch ID, blokada ekranu Androida)
settings-2fa-passkey-rule-cross-platform = znajdują się na kluczu bezpieczeństwa lub telefonie
settings-2fa-passkey-rule-user-verification = sprawdzają PIN, odcisk palca lub twarz
settings-2fa-passkey-rule-resident-key = są przechowywane na uwierzytelniaczu (wykrywalne)
settings-2fa-passkey-gap-allowed = Dodano przed obecnymi zasadami i nie spełnia ich. Nadal działa; dodaj nowy klucz dostępu i usuń ten.
settings-2fa-passkey-gap-refused = Nie spełnia obecnych zasad i nie jest już akceptowany przy logowaniu. Dodaj nowy klucz dostępu i usuń ten.
settings-2fa-passkey-browser-refused-policy = Przeglądarka nie znalazła uwierzytelniacza spełniającego zasady kluczy dostępu Twojej organizacji albo żądanie zostało anulowane.
settings-2fa-policy-user-verification = Twoja organizacja wymaga klucza dostępu, który sprawdza PIN, odcisk palca lub twarz. Ten uwierzytelniacz nie zweryfikował tożsamości.
settings-2fa-policy-platform = Twoja organizacja akceptuje tylko klucze dostępu przechowywane na tym urządzeniu (takie jak Windows Hello, Touch ID lub blokada ekranu Androida). Klucze bezpieczeństwa i telefony nie są akceptowane.
settings-2fa-policy-cross-platform = Twoja organizacja akceptuje tylko klucze bezpieczeństwa lub klucze dostępu z telefonu. Klucze dostępu przechowywane na tym urządzeniu nie są akceptowane.
settings-2fa-policy-attachment-unknown = Przeglądarka nie podała rodzaju tego uwierzytelniacza, więc nie można go sprawdzić pod kątem zasad kluczy dostępu Twojej organizacji. Spróbuj w innej przeglądarce.
settings-2fa-policy-resident-key = Twoja organizacja wymaga klucza dostępu przechowywanego na uwierzytelniaczu. Ten uwierzytelniacz go nie zapisał.
settings-2fa-policy-legacy = Ten klucz dostępu dodano przed obecnymi zasadami Twojej organizacji i już ich nie spełnia. Zaloguj się innym składnikiem lub kodem odzyskiwania, a potem dodaj nowy klucz dostępu.
settings-2fa-policy-last-factor-blocked = Twoja organizacja nie pozwala usunąć ostatniego drugiego składnika. Najpierw dodaj inną metodę logowania.
settings-2fa-policy-last-factor-required = Twoja organizacja wymaga uwierzytelniania dwuskładnikowego. Dodaj inną metodę logowania przed usunięciem tej.
settings-2fa-last-factor-hint = Najpierw dodaj inną metodę logowania.
settings-2fa-last-factor-tooltip = Najpierw dodaj inną metodę logowania. Twoja organizacja wymaga na koncie co najmniej jednego klucza dostępu lub aplikacji uwierzytelniającej.
settings-2fa-challenge-passkey-not-used = Klucz dostępu nie został użyty: żądanie anulowano lub przekroczyło limit czasu albo ten uwierzytelniacz nie spełnia wymagań Twojej organizacji (takich jak PIN, odcisk palca lub twarz). Spróbuj ponownie lub użyj innej metody.
settings-folders-hidden-help = Ukryte foldery i ich podfoldery nie pojawiają się na pasku bocznym. Nie są usuwane ani objęte ograniczeniem dostępu. Pokaż tutaj podfolder, aby pokazać także jego ukryte foldery nadrzędne. Odebrane zawsze pozostają dostępne.
settings-folders-collapse-user = Zwijaj foldery użytkownika na pasku bocznym
settings-folders-show = Pokaż { $name } na pasku bocznym
settings-folders-hide = Ukryj { $name } na pasku bocznym
settings-folders-show-short = Pokaż
settings-folders-hide-short = Ukryj
settings-folders-search = Szukaj folderów
settings-folders-select = Wybierz folder: { $name }
settings-folders-selected = Wybrane foldery: { $count }
settings-folders-select-visible = Zaznacz widoczne
settings-folders-select-all = Zaznacz wszystkie foldery
settings-folders-clear = Wyczyść zaznaczenie
settings-folders-pin-selected = Przypnij zaznaczone
settings-folders-unpin-selected = Odepnij zaznaczone
settings-folders-show-selected = Pokaż zaznaczone
settings-folders-hide-selected = Ukryj zaznaczone
settings-folders-visible = Widoczny
settings-folders-hidden = Ukryty
settings-folders-pinned = Przypięty
settings-folders-location = Położenie
settings-folders-counts = { $unread } nieprzeczytanych / { $total } razem
settings-folders-more = Więcej akcji dla: { $name }
settings-folders-empty = Żaden folder nie pasuje do wyszukiwania.
settings-folders-root = Najwyższy poziom
settings-folders-order-help = Przeciągaj foldery, aby zmienić ich kolejność na pasku bocznym. Przypięte foldery pozostają razem.
cache-title = Pamięć podręczna poczty i wstępne ładowanie
cache-help = Przechowuj ostatnio odwiedzone foldery i wiadomości w pamięci tej karty. Zapisana treść odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie wczytuje zdalnych obrazów.
cache-enabled = Włącz pamięć podręczną sesji
cache-mailboxes = Zapisuj listy wiadomości folderów
cache-messages = Zapisuj otwarte wiadomości
cache-freshSeconds = Okres aktualności (sekundy)
cache-maxAgeSeconds = Maksymalny wiek wpisu (sekundy)
cache-maxEntries = Maksymalna liczba wpisów
cache-maxBytes = Limit pamięci (bajty)
cache-prefetchEnabled = Włącz wstępne ładowanie spekulacyjne
cache-hoverEnabled = Ładuj wstępnie po wskazaniu folderu
cache-focusEnabled = Ładuj wstępnie, gdy folder otrzyma fokus klawiatury
cache-dwellMs = Opóźnienie wskaźnika i fokusu (milisekundy)
cache-idleEnabled = Ładuj foldery wstępnie w czasie bezczynności
cache-idleDelayMs = Opóźnienie bezczynności (milisekundy)
cache-maxConcurrency = Maksymalna liczba równoczesnych ładowań wstępnych
cache-maxQueued = Maksymalna liczba ładowań wstępnych w kolejce
cache-preloadMessages = Zezwalaj na spekulacyjne ładowanie treści wiadomości
cache-excludedFolders = Foldery wyłączone z ładowania wstępnego
cache-exclusions-help = Jedna reguła w wierszu: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive lub id: z dokładnym identyfikatorem folderu.
cache-refreshOnOpen = Odświeżaj zapisaną treść przy otwieraniu
cache-clear = Wyczyść pamięć podręczną poczty
settings-compose-window = Otwieraj nowe wiadomości w
settings-compose-window-tab = Karcie przestrzeni roboczej
settings-compose-window-new = Nowym oknie
settings-compose-window-help = Jeśli przeglądarka zablokuje nowe okno, edytor otworzy się w karcie przestrzeni roboczej.
settings-compose-read-receipt = Zawsze żądaj potwierdzenia przeczytania
settings-compose-delivery-receipt = Zawsze żądaj potwierdzenia dostarczenia
settings-compose-reply-copy = Zapisuj kopię odpowiedzi w folderze oryginału
settings-compose-receipts-help = Potwierdzenia są tylko prośbami. Odbiorca lub jego serwer poczty może je odrzucić albo zignorować.
settings-compose-forward = Domyślny format przekazywania
settings-compose-forward-inline = Treść wiadomości w tekście
settings-compose-forward-attachment = Oryginalna wiadomość jako załącznik .eml
settings-compose-saving = Zapisywanie preferencji redagowania…
settings-compose-saved = Zapisano preferencję redagowania
settings-ident-organization = Organizacja
settings-ident-auto-bcc = Automatyczna Ukryta kopia
settings-ident-auto-bcc-help = Dodaj do 20 adresów w Ukrytej kopii przy wysyłaniu z tej tożsamości. Oddziel adresy przecinkami.
settings-ident-pgp-key = Klucz OpenPGP
settings-ident-pgp-none = Użyj klucza domyślnego
settings-ident-pgp-unavailable = Zapisany klucz niedostępny — wybierz inny klucz
settings-ident-pgp-help = Wybierz jeden z istniejących kluczy podpisujących OpenPGP. Materiał klucza prywatnego pozostaje w Twoim magazynie kluczy.
settings-ident-error-optional = Sprawdź adresy Odpowiedz do i automatycznej Ukrytej kopii, organizację oraz wybrany klucz.
settings-compose-open-tracking = Domyślnie rejestruj otwarcia wiadomości
settings-compose-open-tracking-help = Dodaje obraz śledzący do kwalifikujących się sformatowanych wiadomości. Otwarcia zależą od wczytania obrazów przez odbiorcę i nie dowodzą przeczytania wiadomości. Administrator może wyłączyć śledzenie lub zablokować tę wartość domyślną.
settings-compose-open-tracking-inherit = Użyj odziedziczonej wartości domyślnej śledzenia
settings-reply-images = Obrazy w odpowiedziach i przekazaniach
settings-reply-images-ask = Pytaj za każdym razem
settings-reply-images-allow = Zawsze dołączaj
settings-reply-images-trusted = Dołączaj dla zaufanych domen nadawców, w pozostałych przypadkach pytaj
settings-reply-images-deny = Nigdy nie dołączaj
settings-reply-images-help = Wybierz, czy odpowiedź lub przekazanie zawiera obrazy z oryginalnej wiadomości. Dołączone obrazy są kopiowane do wersji roboczej jako załączniki.
settings-reply-image-domains = Zaufane domeny nadawców obrazów
settings-reply-image-domains-help = Wpisz do 200 dokładnych domen nadawców, po jednej w wierszu. Używane, gdy powyżej wybrano zaufane domeny.
settings-reply-image-domains-save = Zapisz zaufane domeny obrazów
settings-ident-group-basics = Podstawy
settings-ident-group-replies = Obsługa odpowiedzi
settings-ident-group-signature = Podpis
settings-ident-group-sending = Opcje wysyłania
settings-ident-name-required = Wpisz nazwę wyświetlaną.
settings-ident-email-invalid = Wpisz prawidłowy adres e-mail, na przykład name@example.org.
settings-ident-email-duplicate = Masz już tożsamość z tym adresem.
settings-ident-replyto-invalid = Wpisz prawidłowy adres e-mail lub pozostaw to pole puste.
settings-ident-replyto-help = Odpowiedzi na wiadomości wysłane z tej tożsamości trafiają na ten adres zamiast na adres nadawcy.
settings-ident-bcc-invalid = Sprawdź listę Ukrytej kopii: użyj prawidłowych adresów oddzielonych przecinkami, najwyżej 20.
settings-ident-delete-title = Usunąć tożsamość { $email }?
settings-ident-delete-body = Nie będzie można już wysyłać z tej tożsamości. Wersje robocze i wysłane wiadomości nie zostaną zmienione.
settings-ident-empty = Nie masz jeszcze tożsamości. Dodaj ją, aby wybierać nazwę, adres i podpis podczas pisania.
settings-sig-delete-title = Usunąć podpis { $name }?
settings-sig-delete-body = Tożsamości, które go używają, wrócą do podpisu konta. Wysłane już wiadomości nie zostaną zmienione.
settings-sig-deleted = Podpis usunięty.
settings-reply-from = Odpowiadaj z adresu
settings-reply-from-match = Adres, na który wysłano wiadomość
settings-reply-from-default = Zawsze z mojej tożsamości domyślnej
settings-reply-from-ask = Pytaj, gdy pasuje kilka tożsamości
settings-reply-from-keep = Zachowaj już wybraną tożsamość
settings-reply-from-help = Przy odpowiadaniu lub przekazywaniu wybierz, której tożsamości użyć. Jeśli żadna tożsamość nie pasuje do adresu, na który wysłano wiadomość, używana jest tożsamość domyślna. Adresy z plusem, takie jak name+tag@example.org, pasują do name@example.org.
settings-ident-default-badge = Domyślna
settings-ident-make-default = Ustaw jako domyślną
settings-ident-default-set = { $email } jest teraz tożsamością domyślną.
settings-ident-create-variant = Utwórz wariant
settings-ident-variant-count = { $count } { $count ->
        [one] wariant
        [few] warianty
        [many] wariantów
       *[other] wariantu
    }
settings-ident-inherits = Używa { $base } dla: { $fields }
settings-ident-field-replyTo = adresu Odpowiedz do
settings-ident-field-autoBcc = automatycznej Ukrytej kopii
settings-ident-field-organization = organizacji
settings-ident-field-pgpKeyId = klucza OpenPGP
settings-ident-field-signatureName = podpisu
settings-ident-delete-default-body = To jest Twoja tożsamość domyślna. Domyślną zostanie najstarsza z pozostałych tożsamości. Nie będzie można już wysyłać z tej tożsamości. Wersje robocze i wysłane wiadomości nie zostaną zmienione.
settings-ident-variants-choice-title = Usunąć { $email }?
settings-ident-variants-choice-body = Ta tożsamość ma { $count } { $count ->
        [one] wariant
        [few] warianty
        [many] wariantów
       *[other] wariantu
    }. Usuń { $count ->
        [one] go
       *[other] je
    } także albo zachowaj { $count ->
        [one] go
       *[other] je
    } jako osobne tożsamości. Zachowany wariant kopiuje ustawienia odziedziczone z tej tożsamości.
settings-ident-delete-with-variants = Usuń tożsamość i warianty
settings-ident-delete-keep-variants = Usuń tożsamość, zachowaj warianty
settings-ident-new-variant = Nowy wariant
settings-ident-edit-variant-title = Edytuj wariant
settings-ident-variant-intro = Wariant tożsamości { $base }. Pozostaw pole puste, aby użyć wartości z tej tożsamości.
settings-ident-label-label = Etykieta wariantu
settings-ident-label-required = Wpisz etykietę tego wariantu, na przykład Biuletyny.
settings-ident-inherit-hint = Puste: używa wartości { $value } z tożsamości bazowej.
settings-ident-inherit-none = nic
settings-check-policy-allow = Wyślij bez pytania
settings-check-policy-warn = Pytaj za każdym razem
settings-check-policy-warn-remember = Pytaj i pozwól przestać pytać
settings-check-policy-deny = Zablokuj wysyłanie
settings-check-invalid = Nieprawidłowe adresy odbiorców
settings-check-attachment = Wspomniano o załączniku, ale go brak
settings-check-subject = Pusty temat
settings-check-help = Co zrobić, gdy kontrola wykryje problem. Pytanie zawsze oferuje Wyślij mimo to; wiadomość zatrzymuje tylko Zablokuj wysyłanie.
settings-external-warn-remember = Pytaj i pozwól przestać pytać
settings-external-deny = Zablokuj wysyłanie
settings-auto-chip = Kończ odbiorcę po wpisaniu spacji, przecinka lub średnika
settings-auto-chip-help = Prawidłowy adres staje się gotowym odbiorcą, a separator jest dodawany automatycznie.
settings-recipient-separator = Separator odbiorców
settings-recipient-separator-comma = Przecinek
settings-recipient-separator-semicolon = Średnik
settings-header-collapse = Nagłówek wiadomości (Do, DW, Temat)
settings-header-collapse-expanded = Zawsze pokazuj
settings-header-collapse-collapsed = Zwiń do jednego wiersza
settings-header-collapse-auto = Zwijaj podczas pisania treści
settings-header-collapse-help = Po zwinięciu nagłówek pokazuje jeden wiersz, na przykład Do: Ana +2, i rozwija się po kliknięciu lub gdy pole wymaga uwagi.
settings-retention-title = Retencja
settings-folder-icon-camera = Aparat
settings-folder-icon-music = Muzyka
settings-folder-icon-film = Film
settings-folder-icon-lightbulb = Pomysł
settings-folder-icon-rocket = Rakieta
settings-folder-icon-trophy = Puchar
settings-folder-icon-leaf = Liść
settings-folder-icon-globe = Glob
settings-folder-icon-cloud = Chmura
settings-folder-icon-bookmark = Zakładka
settings-folder-icon-wallet = Portfel
settings-folder-icon-medical = Medycyna
settings-folder-icon-bell = Dzwonek
settings-folder-icon-key = Klucz
settings-folder-icon-receipt = Paragon
settings-folder-icon-cart = Koszyk
settings-folder-icon-package = Paczka
settings-folder-icon-wrench = Narzędzia
settings-folder-icon-chart = Wykres
settings-folder-icon-team = Zespół
settings-folder-icon-store = Sklep
settings-folder-icon-office = Biuro
settings-folder-icon-laptop = Laptop
settings-folder-icon-crown = Korona
settings-folder-icon-family = Rodzina
settings-folder-icon-group-general = Ogólne
settings-folder-icon-group-mail = Poczta
settings-folder-icon-group-work = Praca
settings-folder-icon-group-finance = Finanse i zakupy
settings-folder-icon-group-travel = Podróże i dom
settings-folder-icon-group-personal = Osobiste i rozrywka
settings-folder-icon-group-tech = Technika i narzędzia
settings-folder-icon-search-label = Szukaj ikon
settings-folder-icon-no-match = Brak pasujących ikon

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Uwierzytelnianie (DMARC)

settings-badge-auth-help = Wynik DMARC; otwiera pełne szczegóły bezpieczeństwa.

settings-badge-density = Rozmiar odznaki

settings-badge-density-comfortable = Wygodne

settings-badge-density-compact = Kompaktowy

settings-badge-density-help = Kompaktowa lub wygodna wysokość i odstępy dla każdej plakietki.

settings-badge-folder = Folder

settings-badge-folder-help = Miejsce przechowywania wiadomości. Tylko wtedy, gdy Notable pokaże to dla Junk and Trash.

settings-badge-labels = Etykiety odznak

settings-badge-labels-help = Ikona z tekstem lub sama ikona (pełny tekst pozostaje w podpowiedzi i dla czytników ekranu).

settings-badge-labels-icon = Tylko ikona

settings-badge-labels-text = Ikona i tekst

settings-badge-list = Lista mailingowa

settings-badge-list-help = Lista mailingowa, z której pochodzi wiadomość; otwiera szczegóły subskrypcji i działania. Tylko wtedy, gdy Notable pokazuje to, gdy poczta wciąż przychodzi po rezygnacji z subskrypcji.

settings-badge-mode-always = Zawsze

settings-badge-mode-never = Nigdy

settings-badge-mode-notable = Tylko wtedy, gdy jest to zauważalne

settings-badge-sanitized = Zdezynfekowany HTML

settings-badge-sanitized-help = Co środek dezynfekujący HTML zrobił z wiadomością. Tylko wtedy, gdy zauważalne, pokazuje to, gdy coś zostało zablokowane.

settings-badge-spam = Wynik spamu

settings-badge-spam-help = Wynik, który serwer odbierający zapisał w nagłówkach wiadomości.

settings-folders-hide-empty = Ukryj puste foldery na pasku bocznym

settings-folders-hide-empty-help = Foldery pojawiają się ponownie, gdy zawierają wiadomości. Skrzynka odbiorcza, foldery wyszukiwania, foldery o nieznanej liczbie i foldery nadrzędne widocznych folderów pozostają widoczne. Zastąp poszczególne foldery w obszarze Właściwości, Wyświetlacz; każdy folder pozostanie na tej liście.

settings-folders-list-card = Lista folderów i kolejność

settings-header-auto-collapse = Złóż nagłówek po napisaniu (sekundy)

settings-header-auto-collapse-help = Kiedy treść była skupiona przez tyle sekund, a nagłówek zawiera odbiorcę i temat, składa się do linii podsumowania. 0 wyłącza to.

settings-message-badges = Odznaki wiadomości

settings-message-badges-help = Wybierz, które plakietki mają się pojawiać w nagłówku otwartej wiadomości. Znaki priorytetu ustawia się w obszarze Foldery i pasek boczny. Możesz także kliknąć plakietkę prawym przyciskiem myszy, aby ją ukryć.

settings-reply-from-headers = Nagłówki, które wybierają tożsamość

settings-reply-from-headers-help = Jedna nazwa nagłówka w każdym wierszu, wypróbowana w następującej kolejności: Dostarczono do, X-Oryginał do, Koperta do, X-Przesłano do, Wysłano ponownie do, Do, DW. Pominięty nagłówek nie jest czytany.

settings-reply-from-headers-invalid = Używaj tylko tych nazw, po jednej w każdym wierszu: Dostarczono do, X-Oryginał do, Koperta do, X-Przesłano do, Wysłano ponownie do, Do, DW.

settings-reply-from-headers-save = Zapisz kolejność nagłówków

settings-sessions-linked-created = Utworzono łącze

settings-sessions-linked-details = Szczegóły linku

settings-sessions-linked-kind = Typ łącza

settings-sessions-linked-kind-linked = Połączona skrzynka pocztowa

settings-sessions-linked-last-used = Ostatnio używany

settings-sessions-linked-line = Dostęp do połączonej skrzynki pocztowej · od { $origin }

settings-sessions-linked-line-hidden = Dostęp do połączonej skrzynki pocztowej

settings-sessions-linked-origin = Pochodzi z

settings-sessions-linked-removed = Nieznane (konto usunięte)

settings-sessions-linked-revoke = Cofnij dostęp do linku

settings-sessions-linked-revoke-label = Odbierz dostęp do linku dla { $mailbox }

settings-sessions-linked-target = Skrzynka pocztowa otwarta

settings-sessions-origin-full = Pełny adres

settings-sessions-origin-help = Kontroluje sposób, w jaki ta tabela nazywa konto, z którego pochodzi sesja połączonej skrzynki pocztowej. Tylko Ty to widzisz podczas swoich własnych sesji.

settings-sessions-origin-hidden = Ukryty

settings-sessions-origin-masked-local-part = Adres z zamaskowaną pierwszą częścią

settings-sessions-origin-setting = Pokaż, skąd pochodzi dostęp do połączonej skrzynki pocztowej

settings-shortcuts-search = Wyszukaj skróty

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = Przechowuj ostatnio odwiedzane foldery i wiadomości gotowe do otwarcia. Domyślnie kopie pozostają w pamięci tej zakładki. Włącz nieograniczone przechowywanie, aby zachować zaszyfrowane kopie podczas ponownego ładowania. Wstępne ładowanie nigdy nie oznacza przeczytania poczty ani nie ładuje zdalnych obrazów.

cache-keepIndefinitely = Przechowuj pocztę w pamięci podręcznej na czas nieokreślony

cache-keepIndefinitely-help = Domyślnie wyłączone. Przechowuj zaszyfrowane kopie w tej przeglądarce podczas ponownego ładowania bez wygaśnięcia wieku. Wiadomości online są nadal odświeżane. Limity wpisów i bajtów nadal obowiązują. Wyłączenie tej opcji lub wyczyszczenie poczty w pamięci podręcznej powoduje usunięcie zapisanych kopii. Wyczyszczenie danych przeglądarki może również je usunąć. Załączniki nie są pobierane do użytku w trybie offline.

settings-notif-view-enabled = Włącz widok powiadomień
settings-notif-view-sidebar = Pokaż Powiadomienia na pasku bocznym
settings-notif-view-help = Steruje centrum powiadomień. Alerty bezpieczeństwa, poczty i przeglądarki zachowują własne ustawienia.

settings-badge-importance = Ważność
settings-badge-importance-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-follow-up = Monit
settings-badge-follow-up-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-snooze = Odłożenie
settings-badge-snooze-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-receipts = Potwierdzenia odczytu
settings-badge-receipts-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-delivery = Stan dostarczenia
settings-badge-delivery-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-dates = Daty wiadomości
settings-badge-dates-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-sensitivity = Poufność
settings-badge-sensitivity-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-reply-by = Termin odpowiedzi
settings-badge-reply-by-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-expires = Wygaśnięcie
settings-badge-expires-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
settings-badge-supersedes = Zastąpione wiadomości
settings-badge-supersedes-help = Wybierz zawsze, tylko gdy istotne lub nigdy. Wyróżnia nietypowe wartości, aktywne przypomnienia, potwierdzenia, dostarczenia lub bliskie terminy. Dane wiadomości zostają zachowane.
