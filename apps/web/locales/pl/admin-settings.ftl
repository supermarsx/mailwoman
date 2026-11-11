# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Zezwalaj na wysyłanie
admin-settings-help-mail-allow-send = Pozwala użytkownikom wysyłać wiadomości. Gdy wyłączone, edytor wyłącza przycisk Wyślij i jego skrót, a serwer odrzuca wysyłkę; wersje robocze nadal można zapisywać.
admin-settings-key-mail-allow-reply = Zezwalaj na odpowiadanie
admin-settings-help-mail-allow-reply = Pozwala użytkownikom odpowiadać i odpowiadać wszystkim z czytnika oraz za pomocą skrótów. Gdy wyłączone, te akcje nie są oferowane.
admin-settings-key-mail-allow-forward = Zezwalaj na przekazywanie dalej
admin-settings-help-mail-allow-forward = Pozwala użytkownikom przekazywać wiadomości w treści lub jako załącznik. Gdy wyłączone, te akcje nie są oferowane.
admin-settings-key-mail-allow-download = Zezwalaj na pobieranie
admin-settings-help-mail-allow-download = Pozwala użytkownikom pobierać wiadomości i załączniki. Gdy wyłączone, aplikacja webowa nie oferuje pobierania, a serwer odrzuca eksport wiadomości.
admin-settings-key-mail-allow-print = Zezwalaj na drukowanie
admin-settings-help-mail-allow-print = Pozwala użytkownikom drukować wiadomość z czytnika. Gdy wyłączone, akcja Drukuj nie jest oferowana.
admin-settings-key-mail-show-spam = Pokazuj folder Spam
admin-settings-help-mail-show-spam = Pokazuje folder Spam lub Junk i wiadomości oznaczone jako spam. Gdy wyłączone, są ukryte na liście folderów i w wynikach wyszukiwania; nic nie jest usuwane.
admin-settings-key-mail-show-auth-status = Pokazuj wyniki uwierzytelniania nadawcy
admin-settings-help-mail-show-auth-status = Pokazuje w czytniku wyniki SPF, DKIM i DMARC z nagłówka Authentication-Results.
admin-settings-key-mail-close-on-delete = Zamykaj wiadomość po jej usunięciu
admin-settings-help-mail-close-on-delete = Zamyka otwartą wiadomość, kartę lub odłączone okno, gdy ta wiadomość zostanie usunięta lub przeniesiona do kosza. Gdy wyłączone, czytnik zachowuje się jak dotychczas.
admin-settings-key-links-trusted-senders = Zaufani nadawcy linków
admin-settings-help-links-trusted-senders = Dokładne skrzynki nadawców, których sprawdzone linki w wiadomościach mogą się otwierać bez zgody dla każdej wiadomości. Jeden adres na wpis.
admin-settings-key-appearance-font-size = Rozmiar czcionki interfejsu
admin-settings-help-appearance-font-size = Rozmiar czcionki interfejsu w pikselach, od 12 do 24.
admin-settings-key-appearance-font-weight = Grubość czcionki interfejsu
admin-settings-help-appearance-font-weight = Podstawowa grubość tekstu interfejsu: 300, 400, 500 lub 600. Tekst wyróżniony zachowuje swoją hierarchię.
admin-settings-key-appearance-font-color = Kolor tekstu interfejsu
admin-settings-help-appearance-font-color = Kolor tekstu interfejsu w formacie #RRGGBB. Pozostaw puste, aby podążać za motywem.
admin-settings-key-appearance-low-contrast = Złagodź kontrast tekstu
admin-settings-help-appearance-low-contrast = Łagodzi kontrast tekstu interfejsu.
admin-settings-key-mail-message-list-order = Kolejność listy wiadomości
admin-settings-help-mail-message-list-order = Kolejność listy wiadomości: najnowsze, najstarsze, nadawca lub temat.
admin-settings-key-mail-message-list-style = Styl listy wiadomości
admin-settings-help-mail-message-list-style = Pokazuj listę wiadomości jako karty albo jako wiersze z nagłówkami kolumn.
admin-settings-key-mail-message-list-preview = Podglądy na liście wiadomości
admin-settings-help-mail-message-list-preview = Pokazuj fragmenty treści na liście wiadomości.
admin-settings-key-mail-after-delete = Po usunięciu wiadomości
admin-settings-help-mail-after-delete = Którą wiadomość czytnik pokazuje po usunięciu otwartej wiadomości: następną, poprzednią lub żadną.
admin-settings-key-reader-attachments-at-end = Załączniki po treści
admin-settings-help-reader-attachments-at-end = Pokazuj załączniki po treści wiadomości zamiast przed nią.
admin-settings-key-notifications-ask-permission = Oferuj zgodę na powiadomienia
admin-settings-help-notifications-ask-permission = Oferuj użytkownikom na urządzeniach, które nie zezwoliły na powiadomienia ani ich nie zablokowały, monit przeglądarki o zgodę na powiadomienia.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Ustawienia
admin-settings-description = Wartości dla całego serwera, dla domeny i dla użytkownika. Zablokowanej wartości nie mogą zmienić użytkownicy ani węższe zakresy.
admin-settings-list-caption = Ustawienia
admin-settings-search = Szukaj ustawień
admin-settings-empty = Żadne ustawienie nie pasuje do wyszukiwania.
admin-settings-unavailable = Ten serwer nie obsługuje jeszcze modelu ustawień.
admin-settings-col-setting = Ustawienie
admin-settings-col-value = Wartość
admin-settings-col-source = Źródło
admin-settings-col-overrides = Węższe wartości
admin-settings-open = Otwórz
admin-settings-overrides-none = Brak
admin-settings-overrides = Domeny: { $domains }, użytkownicy: { $users }
admin-settings-not-found = Nie ma ustawienia o nazwie { $key }.
admin-settings-value-on = Włączone
admin-settings-value-off = Wyłączone
admin-settings-value-empty = Nie ustawiono
admin-settings-value-list = { $count ->
    [one] { $count } element
    [few] { $count } elementy
    [many] { $count } elementów
   *[other] { $count } elementu
}
admin-settings-detail-key = Klucz: { $key }
admin-settings-global = Dla całego serwera
admin-settings-global-help = Ta wartość obowiązuje wszystkich użytkowników, chyba że domena lub użytkownik ma węższą wartość albo użytkownik wybierze własną tam, gdzie jest to dozwolone.
admin-settings-value-label = Wartość
admin-settings-json-hint = JSON. Serwer sprawdza wartość i wskazuje część, której nie przyjmuje.
admin-settings-json-invalid = To nie jest prawidłowy JSON.
admin-settings-user-override = Użytkownicy mogą to zmienić
admin-settings-user-override-help = Gdy włączone, użytkownicy mogą wybrać własną wartość w swoich ustawieniach. Gdy wyłączone, obowiązuje wartość ustawiona tutaj lub dla ich domeny.
admin-settings-lock-help = Gdy zablokowane, wartości domen i wybory użytkowników są pomijane, a ta wartość obowiązuje wszystkich.
admin-settings-save = Zapisz
admin-settings-saved = Zapisano. Zmiana zacznie obowiązywać w ciągu 30 sekund w każdym procesie serwera.
admin-settings-saved-restart = Zapisano. Zmiana zacznie obowiązywać po ponownym uruchomieniu.
admin-settings-dedicated = To ustawienie zmienia się w sekcji { $section }, która najpierw sprawdza jego skutek.
admin-settings-dedicated-link = Otwórz { $section }
admin-settings-pinned = Ustawione zmienną środowiskową { $name }. Zapisana tutaj wartość obowiązuje, gdy zmienna nie jest ustawiona.
admin-settings-lock-title = Zablokować { $setting }?
admin-settings-lock-body = Blokada pomija wymienione poniżej węższe wartości. Zostają zachowane i obowiązują ponownie po odblokowaniu.
admin-settings-lock-confirm = Zablokuj
admin-settings-impact-domain = { $count ->
    [one] { $count } wartość domeny jest pomijana
    [few] { $count } wartości domen są pomijane
    [many] { $count } wartości domen jest pomijanych
   *[other] { $count } wartości domen jest pomijane
}
admin-settings-impact-user = { $count ->
    [one] { $count } własny wybór użytkownika jest pomijany
    [few] { $count } własne wybory użytkowników są pomijane
    [many] { $count } własnych wyborów użytkowników jest pomijanych
   *[other] { $count } własnego wyboru użytkowników jest pomijane
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } wartość dla użytkownika jest pomijana
    [few] { $count } wartości dla użytkowników są pomijane
    [many] { $count } wartości dla użytkowników jest pomijanych
   *[other] { $count } wartości dla użytkowników jest pomijane
}
admin-settings-impact-profile = { $count ->
    [one] { $count } wartość profilu funkcji jest pomijana
    [few] { $count } wartości profili funkcji są pomijane
    [many] { $count } wartości profili funkcji jest pomijanych
   *[other] { $count } wartości profili funkcji jest pomijane
}
admin-settings-impact-group = { $count ->
    [one] { $count } wartość grupy jest pomijana
    [few] { $count } wartości grup są pomijane
    [many] { $count } wartości grup jest pomijanych
   *[other] { $count } wartości grup jest pomijane
}
admin-settings-domains = Wartości domen
admin-settings-domains-help = Wartość domeny obowiązuje użytkowników tej domeny zamiast wartości dla całego serwera.
admin-settings-domains-caption = Wartości domen dla { $setting }
admin-settings-domains-empty = Żadna domena nie ma własnej wartości.
admin-settings-domain = Domena
admin-settings-domain-value = Wartość dla tej domeny
admin-settings-domain-locked = Zablokowane dla użytkowników domeny
admin-settings-domain-add = Dodaj wartość domeny
admin-settings-domain-save = Zapisz wartość domeny
admin-settings-domain-remove = Usuń
admin-settings-domain-remove-title = Usunąć wartość domeny { $domain }?
admin-settings-domain-remove-body = Użytkownicy domeny { $domain } znowu otrzymają wartość dla całego serwera.
admin-settings-domain-required = Wpisz nazwę domeny.
admin-settings-locked-yes = Zablokowane
admin-settings-locked-no = Niezablokowane
admin-settings-users = Wartości dla użytkowników
admin-settings-users-help = Wartość ustawiona przez administratora dla jednego użytkownika. W przypadku limitów użytkownika wskazuje się adresem.
admin-settings-users-caption = Wartości dla użytkowników: { $setting }
admin-settings-users-empty = Żaden użytkownik nie ma wartości ustawionej przez administratora.
admin-settings-user = Użytkownik
admin-settings-user-hint = Identyfikator konta lub adres w przypadku limitów.
admin-settings-user-value = Wartość dla tego użytkownika
admin-settings-user-add = Dodaj wartość dla użytkownika
admin-settings-user-save = Zapisz wartość użytkownika
admin-settings-user-required = Wpisz użytkownika.
admin-settings-user-remove-title = Usunąć wartość użytkownika { $user }?
admin-settings-user-remove-body = Użytkownik otrzyma wartość swojej domeny lub wartość dla całego serwera.
admin-settings-source-default = Wbudowana wartość domyślna
admin-settings-source-envFallback = Zmienna środowiskowa (brak zapisanej wartości)
admin-settings-source-envPin = Zmienna środowiskowa
admin-settings-source-global = Zapisane dla całego serwera
admin-settings-cat-interface = Interfejs
admin-settings-cat-appearance = Wygląd
admin-settings-cat-mail = Poczta
admin-settings-cat-notifications = Powiadomienia
admin-settings-cat-offline = Praca offline
admin-settings-cat-assistant = Asystent AI
admin-settings-cat-upstream = Serwer poczty
admin-settings-cat-signin = Logowanie
admin-settings-cat-admin = Konsola administratora
admin-settings-cat-passwords = Hasła
admin-settings-cat-limits = Limity i użycie
admin-settings-cat-plugins = Wtyczki
admin-settings-cat-features = Uprawnienia funkcji
admin-settings-cat-integrations = Uprawnienia integracji
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Zezwala na operacje na plikach, udostępnianiu oraz kalendarzu, kontaktach i zadaniach Nextcloud powiązanych z kontem.
admin-settings-key-cap-integration-google = Mostek Google
admin-settings-help-cap-integration-google = Zezwala na operacje mostka Google, w tym uzyskiwanie i odświeżanie tokenów.
admin-settings-key-cap-integration-microsoft = Mostek Microsoft
admin-settings-help-cap-integration-microsoft = Zezwala na operacje mostka Microsoft, w tym uzyskiwanie i odświeżanie tokenów.
admin-settings-key-cap-integration-family = Pojedyncza integracja
admin-settings-help-cap-integration-family = Uprawnienie dla identyfikatora pojedynczej integracji; zakres działania jest wymieniony w katalogu funkcji.
admin-settings-key-cap-ui-plugin-family = Pojedyncza wtyczka interfejsu
admin-settings-help-cap-ui-plugin-family = Uprawnienie dla jednej wtyczki interfejsu. Użyj dokładnego klucza polityki z katalogu integracji.
admin-settings-key-cap-ai-assistant = Asystent AI
admin-settings-help-cap-ai-assistant = Zezwala na konfigurację i wywoływanie asystenta.
admin-settings-key-cap-plugins = Wtyczki
admin-settings-help-cap-plugins = Zezwala na operacje wtyczek interfejsu i silnika.
admin-settings-key-cap-pim-calendar = Kalendarz
admin-settings-help-cap-pim-calendar = Zezwala na metody kalendarza i subskrypcje.
admin-settings-key-cap-pim-contacts = Kontakty
admin-settings-help-cap-pim-contacts = Zezwala na metody kontaktów i książek adresowych.
admin-settings-key-cap-pim-tasks = Zadania
admin-settings-help-cap-pim-tasks = Zezwala na metody zadań.
admin-settings-key-cap-pim-notes = Notatki
admin-settings-help-cap-pim-notes = Zezwala na metody notatek.
admin-settings-key-cap-export = Eksport
admin-settings-help-cap-export = Zezwala na eksport poczty i danych PIM.
admin-settings-key-cap-import = Import
admin-settings-help-cap-import = Zezwala na import poczty i danych PIM.
admin-settings-key-cap-api-keys = Klucze API
admin-settings-help-cap-api-keys = Polityka kluczy API. Zakres egzekwowania sprawdź w katalogu funkcji.
admin-settings-key-cap-oauth-apps = Aplikacje OAuth
admin-settings-help-cap-oauth-apps = Polityka aplikacji OAuth. Zakres egzekwowania sprawdź w katalogu funkcji.
admin-settings-key-cap-rules = Reguły poczty
admin-settings-help-cap-rules = Zezwala na zarządzane metody reguł poczty i autoodpowiedzi.
admin-settings-key-cap-rules-forward = Przekazywanie przez reguły
admin-settings-help-cap-rules-forward = Polityka przekazywania. Zakres egzekwowania sprawdź w katalogu funkcji.
admin-settings-key-cap-remote-images = Obrazy zdalne
admin-settings-help-cap-remote-images = Uprawnienie dla obrazów zdalnych. Żądania nadal podlegają dotychczasowym ustawieniom prywatności; sprawdź zakres funkcji.
admin-settings-key-cap-user-background = Własne tło
admin-settings-help-cap-user-background = Uprawnienie dla własnego tła, domyślnie wyłączone. Sprawdź zakres funkcji.
admin-settings-key-cap-user-css = Własny CSS
admin-settings-help-cap-user-css = Uprawnienie dla własnego CSS, domyślnie wyłączone. Sprawdź zakres funkcji.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-limit-attachment-bytes = Limit rozmiaru załącznika
admin-settings-help-limit-attachment-bytes = Polityka załączników w bajtach, domyślnie 50000000. Egzekwowanie w czasie działania jest jeszcze w przygotowaniu.
admin-settings-key-limit-send-per-hour = Godzinowy limit wysyłania
admin-settings-help-limit-send-per-hour = Polityka wysyłania na godzinę; zero oznacza brak limitu. Egzekwowanie w czasie działania jest jeszcze w przygotowaniu.
admin-settings-key-limit-recipients-per-message = Odbiorcy na wiadomość
admin-settings-help-limit-recipients-per-message = Polityka liczby odbiorców; zero oznacza brak limitu. Egzekwowanie w czasie działania jest jeszcze w przygotowaniu.
admin-settings-cat-import = Import
admin-settings-cat-webhooks = Webhooki
admin-settings-cat-realtime = Aktualizacje na żywo
admin-settings-cat-privacy = Prywatność
admin-settings-cat-observability = Obserwowalność
admin-settings-cat-egress = Połączenia wychodzące
admin-settings-cat-updates = Aktualizacje
admin-settings-cat-backups = Kopie zapasowe
admin-settings-cat-about = Informacje
admin-settings-cat-config = Konfiguracja
admin-settings-cat-jobs = Zadania
admin-settings-cat-maintenance = Konserwacja
admin-settings-cat-health = Kontrole stanu
admin-settings-cat-diagnostics = Diagnostyka
admin-settings-cat-users = Użytkownicy
admin-settings-cat-server-text = Teksty serwera
admin-settings-key-ui-language = Język
admin-settings-help-ui-language = Język klienta poczty dla użytkowników, którzy nie wybrali własnego. „Taki jak w przeglądarce” podąża za językami każdej przeglądarki.
admin-settings-key-ui-notify-new-version = Powiadomienie o nowej wersji
admin-settings-help-ui-notify-new-version = Włączone (domyślnie) pokazuje zalogowanym użytkownikom komunikat z przyciskami „Odśwież teraz” i „Później”, gdy po załadowaniu ich strony wdrożono nowszą wersję aplikacji internetowej. Wyłączone nigdy go nie pokazuje; nowy kod trafia wtedy do użytkowników przy następnym odświeżeniu.
admin-settings-key-ui-undo-history-size = Rozmiar historii cofania i ponawiania
admin-settings-help-ui-undo-history-size = Ile ostatnich działań na poczcie (archiwizacja, usunięcie, przeniesienie, oznaczenie jako przeczytane, flaga, etykieta itp.) zapamiętują przyciski Cofnij i Ponów, od 0 do 50. 0 wyłącza przyciski i skrót Ctrl+Z. Domyślnie 50. Użytkownicy mogą wybrać własny rozmiar, nigdy większy niż maksimum poniżej, chyba że administrator go zablokuje.
admin-settings-key-folders-reorder-enabled = Przeciąganie folderów
admin-settings-help-folders-reorder-enabled = Pozwala zmieniać kolejność folderów przez przeciąganie na pasku bocznym i w ustawieniach. Wyłączone ukrywa uchwyty przeciągania i polecenia Przenieś. Użytkownicy mogą wyłączyć to dla siebie, chyba że ustawienie jest zablokowane.
admin-settings-key-folders-reorder-allow-nesting = Zagnieżdżanie folderów
admin-settings-help-folders-reorder-allow-nesting = Pozwala upuścić folder na inny folder, aby go zagnieździć, lub z powrotem na najwyższy poziom. Wyłączone zachowuje każdy folder pod obecnym folderem nadrzędnym i pozwala tylko na zmianę kolejności wśród folderów tego samego poziomu.
admin-settings-key-folders-reorder-allow-server-move = Przenoszenie folderów na serwerze
admin-settings-help-folders-reorder-allow-server-move = Sprawia, że zagnieżdżenie lub cofnięcie zagnieżdżenia zmienia rzeczywisty folder nadrzędny na serwerze poczty, więc zmiana obowiązuje wszędzie. Wyłączone zachowuje te przeniesienia jedynie jako układ w tej aplikacji.
admin-settings-key-folders-reorder-allow-system-folders = Przenoszenie folderów systemowych
admin-settings-help-folders-reorder-allow-system-folders = Pozwala przeciągać Odebrane, Wersje robocze, Wysłane, Spam, Kosz i Archiwum, aby zmienić ich kolejność. Wyłączone utrzymuje je na górze. Kolejność zmienia się tylko w tej aplikacji; folderów nigdy nie zmienia się nazwy ani nie przenosi na serwerze.
admin-settings-key-folders-reorder-max-depth = Głębokość zagnieżdżenia folderów
admin-settings-help-folders-reorder-max-depth = Najgłębszy poziom zagnieżdżenia folderów, jaki może utworzyć przeciągnięcie, od 1 do 16. Przeniesienie, które umieściłoby folder lub jego podfolder głębiej, jest odrzucane.
admin-settings-key-folders-order-locked = Zablokuj kolejność folderów
admin-settings-help-folders-order-locked = Ustala kolejność folderów dla wszystkich w tym zakresie. Przeciąganie i polecenia Przenieś są wyłączone, a użytkownicy widzą wskazówkę o blokadzie. Przypięcia i kolejność ustawiona przez administratora pozostają.
admin-settings-key-folders-pinned-admin = Wymuszone przypięte foldery
admin-settings-help-folders-pinned-admin = Foldery pokazywane w grupie Przypięte na górze paska bocznego każdego użytkownika. Jeden wpis w wierszu, jako role:inbox albo ścieżka folderu, np. Shared/Support. Użytkownicy nie mogą ich odpiąć ani przenieść. Wpis bez pasującego folderu jest pomijany.
admin-settings-key-folders-order-default = Domyślna kolejność folderów
admin-settings-help-folders-order-default = Kolejność, w jakiej foldery zaczynają dla użytkowników, którzy ich nie ułożyli, od pierwszego do ostatniego. Jeden wpis w wierszu, jako role:<role> albo ścieżka folderu. Foldery spoza listy następują w kolejności serwera. Własna kolejność użytkownika zastępuje tę.
admin-settings-key-folders-new-folder-placement = Położenie nowego folderu
admin-settings-help-folders-new-folder-placement = Gdzie wśród folderów tego samego poziomu trafia folder, który pojawia się na serwerze: na końcu lub w kolejności alfabetycznej.
admin-settings-key-ui-undo-history-max = Maksymalna historia cofania i ponawiania
admin-settings-help-ui-undo-history-max = Największa liczba działań, jaką dowolny użytkownik może przechowywać w historii Cofnij i Ponów, od 0 do 50. Własny rozmiar użytkownika jest obniżany do tej wartości. 0 wyłącza historię dla wszystkich. Historia istnieje w karcie przeglądarki i jest czyszczona po wylogowaniu.
admin-settings-key-ui-bar-mail-top-bar = Pasek narzędzi poczty
admin-settings-help-ui-bar-mail-top-bar = „pinned” utrzymuje pasek narzędzi poczty z elementami układu stale widoczny. „auto” chowa go za wąskim uchwytem, który otwiera go po wskazaniu, dotknięciu lub przejściu do niego klawiszem Tab.
admin-settings-key-ui-bar-workspace-tabs = Karty obszaru roboczego
admin-settings-help-ui-bar-workspace-tabs = „pinned” utrzymuje pasek kart obszaru roboczego stale widoczny. „auto” pokazuje tylko wąski uchwyt z nazwą aktywnej karty, dopóki użytkownik go nie otworzy.
admin-settings-key-ui-bar-mail-ribbon = Wstążka poczty
admin-settings-help-ui-bar-mail-ribbon = Dla układu z wstążką. „pinned” utrzymuje wstążkę otwartą. „auto” pokazuje tylko nazwy kart; karta otwiera wstążkę nad listą wiadomości.
admin-settings-key-ui-bar-composer-ribbon = Wstążka edytora
admin-settings-help-ui-bar-composer-ribbon = Dla każdego okna redagowania. „pinned” utrzymuje wstążkę edytora otwartą. „auto” pokazuje tylko nazwy kart; otwarcie karty przesuwa treść wiadomości w dół zamiast ją zasłaniać.
admin-settings-key-appearance-theme = Motyw
admin-settings-help-appearance-theme = Motyw kolorystyczny widoczny dla użytkowników, dopóki nie wybiorą własnego. Zablokowanie go narzuca ten motyw wszystkim użytkownikom.
admin-settings-key-appearance-accent = Kolor akcentu
admin-settings-help-appearance-accent = Kolor akcentu motywu. Puste oznacza własny akcent motywu.
admin-settings-key-appearance-badge-color = Kolor znaczka nieprzeczytanych
admin-settings-help-appearance-badge-color = Kolor znaczków z liczbą nieprzeczytanych. Default zachowuje kolor motywu; Accent podąża za kolorem akcentu; accent-lighter i accent-darker używają akcentu jaśniejszego lub ciemniejszego o 20%, z czarnym lub białym tekstem liczby. Blokada przesłania wybór konta.
admin-settings-key-appearance-density = Gęstość
admin-settings-help-appearance-density = Ile miejsca zajmują listy i kontrolki: compact, cozy lub relaxed.
admin-settings-key-appearance-font = Czcionka interfejsu
admin-settings-key-packages-allowed = Śledzenie przesyłek dozwolone
admin-settings-help-packages-allowed = Zezwala na śledzenie przesyłek dla objętych kont. Dostawca musi być też włączony w sekcji Śledzenie przesyłek, a każdy użytkownik musi wyrazić zgodę. Wyłączenie tego ustawienia natychmiast uniemożliwia nowe wywołania dostawcy.
admin-settings-help-appearance-font = Wybierz dołączoną czcionkę dla kont bez własnej preferencji czcionki albo zablokuj wybór dla wszystkich objętych kont. Czcionki ładują się z tego serwera; nie jest wysyłane żądanie do Google Fonts. Default używa czcionki motywu.
admin-settings-key-appearance-brand-name = Nazwa produktu
admin-settings-help-appearance-brand-name = Nazwa pokazywana na stronie logowania i na karcie przeglądarki.
admin-settings-key-login-notice = Komunikat na stronie logowania
admin-settings-help-login-notice = Krótki tekst pokazywany na stronie logowania, np. kontakt do pomocy. Pozostaw puste, aby go nie było.
admin-settings-key-mail-read-receipts = Potwierdzenia odczytu
admin-settings-key-receipts-dsn-allowed = Żądania statusu dostarczenia
admin-settings-help-receipts-dsn-allowed = Zezwala kontom na żądanie powiadomień SMTP o statusie dostarczenia. Nie żąda ani nie dowodzi, że wiadomość została przeczytana.
admin-settings-help-mail-read-receipts = Co się dzieje, gdy otwarta wiadomość prosi o potwierdzenie odczytu: zapytać użytkownika, nigdy nie wysyłać lub zawsze wysyłać.
admin-settings-key-notifications-enabled = Powiadomienia o nowej poczcie
admin-settings-help-notifications-enabled = Czy użytkownicy otrzymują powiadomienia o nowej poczcie. Nie zmienia ich reguł powiadomień.
admin-settings-key-offline-retention-days = Wiek pamięci podręcznej offline
admin-settings-help-offline-retention-days = Ile dni urządzenie przechowuje wiadomości do użytku offline. 0 oznacza brak limitu wieku.
admin-settings-key-offline-budget-mb = Rozmiar pamięci podręcznej offline
admin-settings-help-offline-budget-mb = Ile megabajtów urządzenie może zajmować na pocztę offline. 0 oznacza brak pamięci podręcznej offline.
admin-settings-key-assistant-enabled = Asystent AI
admin-settings-help-assistant-enabled = Czy użytkownicy widzą asystenta AI. Przełącznik wdrożenia w sekcji Asystent AI nadal obowiązuje nad tym ustawieniem.
admin-settings-key-assist-profile = Profil dostawcy AI
admin-settings-help-assist-profile = Identyfikator profilu dostawcy obsługującego tych użytkowników (Asystent AI > Profile dostawców). Puste używa punktu końcowego wdrożenia. Identyfikator nieistniejącego profilu wyłącza asystenta AI dla tych użytkowników.
admin-settings-key-upstream-default = Domyślny serwer poczty
admin-settings-help-upstream-default = Serwer poczty, do którego loguje się adres, gdy żadna domena go nie wskazuje. Tabela domen ma pierwszeństwo przed tym ustawieniem.
admin-settings-key-upstream-proxy-timeout-secs = Limit czasu serwera poczty
admin-settings-help-upstream-proxy-timeout-secs = Jak długo, w sekundach, może trwać jedno żądanie do serwera poczty.
admin-settings-key-signin-new-users = Nowi użytkownicy
admin-settings-help-signin-new-users = Kto może utworzyć konto przy pierwszym logowaniu: każdy (open), tylko adresy dodane wcześniej (preprovisioned), te oraz zaimportowani użytkownicy (provisioned) lub nikt nowy (closed). Istniejące konta nigdy nie są dotknięte. Zmieniane w Polityce logowania.
admin-settings-key-signin-new-users-mail = Logowanie hasłem: nowi użytkownicy
admin-settings-help-signin-new-users-mail = Reguła dla nowego użytkownika skrzynki logującego się hasłem (IMAP, POP3, Exchange). Inherit używa ustawienia Nowi użytkownicy.
admin-settings-key-signin-new-users-sso = Logowanie jednokrotne: nowi użytkownicy
admin-settings-help-signin-new-users-sso = Reguła dla nowego użytkownika logującego się przez logowanie jednokrotne lub zaufany nagłówek. Inherit używa ustawienia Nowi użytkownicy.
admin-settings-key-signin-new-users-linked = Połączone skrzynki: nowi użytkownicy
admin-settings-help-signin-new-users-linked = Reguła łączenia skrzynki, która nie ma tu jeszcze konta, oceniana według domeny połączonej skrzynki. Inherit używa ustawienia Nowi użytkownicy.
admin-settings-key-signin-new-users-linked-imap = Połączone skrzynki IMAP: nowi użytkownicy
admin-settings-help-signin-new-users-linked-imap = Reguła tylko dla połączonych skrzynek IMAP. Inherit używa ustawienia Połączone skrzynki.
admin-settings-key-signin-new-users-linked-pop3 = Połączone skrzynki POP3: nowi użytkownicy
admin-settings-help-signin-new-users-linked-pop3 = Reguła tylko dla połączonych skrzynek POP3. Inherit używa ustawienia Połączone skrzynki.
admin-settings-key-signin-new-users-linked-ews = Połączone skrzynki Exchange: nowi użytkownicy
admin-settings-help-signin-new-users-linked-ews = Reguła tylko dla połączonych skrzynek Exchange (EWS). Inherit używa ustawienia Połączone skrzynki.
admin-settings-key-signin-preprovision-ttl-days = Czas ważności wcześniej dodanego użytkownika (dni)
admin-settings-help-signin-preprovision-ttl-days = Jak długo oczekujący użytkownik dodany wcześniej pozostaje ważny, gdy nie podano innego czasu.
admin-settings-key-signin-preprovision-max-ttl-days = Najdłuższy czas ważności wcześniej dodanego użytkownika (dni)
admin-settings-help-signin-preprovision-max-ttl-days = Najdłuższy czas ważności, jaki administrator może nadać oczekującemu użytkownikowi dodanemu wcześniej.
admin-settings-key-signin-preprovision-domain-wide = Zezwalaj na wpisy dla całej domeny
admin-settings-help-signin-preprovision-domain-wide = Zezwala na wpisy wpuszczające każdego z całej domeny, do limitu użyć. Domyślnie wyłączone.
admin-settings-key-signin-preprovision-retention-days = Przechowywanie wygasłych oczekujących użytkowników (dni)
admin-settings-help-signin-preprovision-retention-days = Jak długo wygasły oczekujący użytkownik jest przechowywany, zanim zostanie usunięty.
admin-settings-key-signin-domain-mode = Domeny logowania
admin-settings-help-signin-domain-mode = Czy logować może się dowolna domena, czy tylko wymienione. Zmieniane w Polityce logowania.
admin-settings-key-signin-refusal-detail = Komunikaty o odmowie
admin-settings-help-signin-refusal-detail = Czy odmowa logowania podaje powód (konto wyłączone, adres niedozwolony), czy daje taką samą odpowiedź jak przy błędnym haśle. Zmieniane w Polityce logowania.
admin-settings-key-signin-ldap-bind-precheck = Kontrola LDAP przed logowaniem
admin-settings-help-signin-ldap-bind-precheck = Sprawdza hasło w katalogu przed serwerem poczty. Zmieniane w Integracjach, po próbnym powiązaniu.
admin-settings-key-twofa-pending-ttl-secs = Limit czasu drugiego składnika
admin-settings-help-twofa-pending-ttl-secs = Ile sekund użytkownik ma na wpisanie drugiego składnika po haśle.
admin-settings-key-twofa-max-attempts = Próby drugiego składnika
admin-settings-help-twofa-max-attempts = Ile błędnych kodów drugiego składnika jest dozwolonych, zanim logowanie zacznie się od nowa.
admin-settings-key-admin-session-idle-secs = Limit bezczynności konsoli
admin-settings-help-admin-session-idle-secs = Jak długo, w sekundach, trwa sesja konsoli administratora bez żądania.
admin-settings-key-admin-session-absolute-secs = Limit sesji konsoli
admin-settings-help-admin-session-absolute-secs = Najdłuższy czas trwania sesji konsoli administratora, w sekundach, aktywnej lub nie.
admin-settings-key-admin-session-rotate-secs = Rotacja identyfikatora sesji konsoli
admin-settings-help-admin-session-rotate-secs = Jak często, w sekundach, konsola administratora zastępuje identyfikator sesji podczas użycia (0 = tylko po potwierdzeniu podwyższonych uprawnień). Rotacja nigdy nie wydłuża limitów bezczynności ani bezwzględnego.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-cat-folders = Układ folderów
admin-settings-key-passwd-policy-min-length = Minimalna długość hasła
admin-settings-help-passwd-policy-min-length = Najmniejsza liczba znaków, jaką może mieć nowe hasło.
admin-settings-key-passwd-policy-require-upper = Wymagaj wielkiej litery
admin-settings-help-passwd-policy-require-upper = Nowe hasło musi zawierać wielką literę.
admin-settings-key-passwd-policy-require-lower = Wymagaj małej litery
admin-settings-help-passwd-policy-require-lower = Nowe hasło musi zawierać małą literę.
admin-settings-key-passwd-policy-require-digit = Wymagaj cyfry
admin-settings-help-passwd-policy-require-digit = Nowe hasło musi zawierać cyfrę.
admin-settings-key-passwd-policy-require-symbol = Wymagaj symbolu
admin-settings-help-passwd-policy-require-symbol = Nowe hasło musi zawierać znak, który nie jest literą ani cyfrą.
admin-settings-key-limit-storage-bytes = Limit miejsca
admin-settings-help-limit-storage-bytes = Największa liczba bajtów, jaką jeden użytkownik może przechowywać na tym serwerze. 0 oznacza brak limitu.
admin-settings-key-limit-send-per-day = Wiadomości wysyłane dziennie
admin-settings-help-limit-send-per-day = Największa liczba wiadomości, jaką jeden użytkownik może wysłać w ciągu dnia. 0 oznacza brak limitu.
admin-settings-key-limit-ai-calls-per-day = Wywołania asystenta AI dziennie
admin-settings-help-limit-ai-calls-per-day = Największa liczba żądań do asystenta AI, jaką jeden użytkownik może wykonać w ciągu dnia. 0 oznacza brak limitu.
admin-settings-key-quota-warn-thresholds = Progi ostrzeżeń o użyciu
admin-settings-help-quota-warn-thresholds = Procenty limitu, przy których użytkownicy są ostrzegani, od najniższego.
admin-settings-key-quota-warn-enabled = Ostrzeżenia o użyciu
admin-settings-help-quota-warn-enabled = Czy użytkownicy są ostrzegani, gdy zbliżają się do limitu.
admin-settings-key-usage-retention-days = Historia użycia
admin-settings-help-usage-retention-days = Ile dni dziennych liczników użycia jest przechowywanych.
admin-settings-key-usage-meter-api = Licz użycie API
admin-settings-help-usage-meter-api = Czy żądania API są wliczane do danych o użyciu.
admin-settings-key-usage-measure-interval-hours = Odstęp pomiaru miejsca
admin-settings-help-usage-measure-interval-hours = Jak często, w godzinach, mierzone są dane przechowywane przez każdego użytkownika.
admin-settings-key-assistant-rate-limit-per-min = Limit żądań asystenta AI
admin-settings-help-assistant-rate-limit-per-min = Największa liczba żądań do asystenta AI na minutę dla całego serwera. 0 oznacza brak limitu.
admin-settings-key-plugins-kv-max-key-bytes = Rozmiar klucza wtyczki
admin-settings-help-plugins-kv-max-key-bytes = Najdłuższy klucz, jaki wtyczka może zapisać, w bajtach.
admin-settings-key-plugins-kv-max-value-bytes = Rozmiar wartości wtyczki
admin-settings-help-plugins-kv-max-value-bytes = Największa wartość, jaką wtyczka może zapisać pod jednym kluczem, w bajtach.
admin-settings-key-plugins-kv-max-total-bytes = Miejsce wtyczki
admin-settings-help-plugins-kv-max-total-bytes = Największa łączna liczba bajtów, jaką jedna wtyczka może zapisać.
admin-settings-key-plugins-kv-max-keys = Klucze wtyczki
admin-settings-help-plugins-kv-max-keys = Największa liczba kluczy, jaką jedna wtyczka może zapisać.
admin-settings-key-import-max-messages = Liczba wiadomości w imporcie
admin-settings-help-import-max-messages = Największa liczba wiadomości, jaką może wprowadzić jeden import.
admin-settings-key-import-max-message-bytes = Rozmiar wiadomości w imporcie
admin-settings-help-import-max-message-bytes = Największa wiadomość akceptowana przez import, w bajtach.
admin-settings-key-webhooks-max-attempts = Próby webhooka
admin-settings-help-webhooks-max-attempts = Ile razy próbuje się dostarczyć webhook, zanim zostanie porzucony.
admin-settings-key-webhooks-timeout-secs = Limit czasu webhooka
admin-settings-help-webhooks-timeout-secs = Jak długo, w sekundach, może trwać jedno dostarczenie webhooka.
admin-settings-key-outbox-max-attempts = Próby wysłania
admin-settings-help-outbox-max-attempts = Ile razy wiadomość jest przekazywana serwerowi poczty, zanim zostanie zgłoszona jako niewysłana.
admin-settings-key-realtime-recheck-secs = Ponowna kontrola aktualizacji na żywo
admin-settings-help-realtime-recheck-secs = Jak często, w sekundach, otwarte połączenie aktualizacji na żywo sprawdza, czy jego sesja jest nadal ważna.
admin-settings-key-privacy-watermark-enabled = Znak wodny czytelnika
admin-settings-help-privacy-watermark-enabled = Pokazuje znak wodny z adresem czytelnika nad otwartymi wiadomościami.
admin-settings-key-privacy-watermark-opacity = Intensywność znaku wodnego
admin-settings-help-privacy-watermark-opacity = Jak widoczny jest znak wodny, od 0,02 (ledwo widoczny) do 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Historia śledzenia otwarć
admin-settings-help-privacy-open-tracking-retention-days = Ile dni przechowywane są zapisy śledzenia odczytu.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Tokeny śledzenia niewysłanych wiadomości
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Ile godzin token śledzenia czeka na wysłanie wiadomości, zanim zostanie usunięty.
admin-settings-key-masked-email-domain = Domena adresów maskowanych
admin-settings-help-masked-email-domain = Domena, w której tworzone są adresy maskowane. Pozostaw puste, aby wyłączyć adresy maskowane.
admin-settings-key-abuse-report-address = Adres zgłoszeń nadużyć
admin-settings-help-abuse-report-address = Dokąd wysyłane są zgłoszenia spamu i nadużyć.
admin-settings-key-dlp-admin-address = Powiadomienia o wycieku danych
admin-settings-help-dlp-admin-address = Dokąd wysyłane są powiadomienia systemu zapobiegania utracie danych.
admin-settings-key-observability-error-forward-url = Adres przekazywania błędów
admin-settings-help-observability-error-forward-url = Adres https, na który przekazywane są raporty błędów klientów. Pozostaw puste, aby ich nie przekazywać.
admin-settings-key-egress-probe-url = Adres testu połączeń wychodzących
admin-settings-help-egress-probe-url = Adres https używany do testowania proxy wychodzącego.
admin-settings-key-push-vapid-contact = Kontakt push
admin-settings-help-push-vapid-contact = Kontakt (mailto: lub https:) przekazywany usługom push z każdym powiadomieniem.
admin-settings-key-push-quiet-hours = Godziny ciszy push
admin-settings-help-push-quiet-hours = Godziny (UTC), w których nie są wysyłane powiadomienia push. Pozostaw puste, aby nie było takich godzin.
admin-settings-key-updates-check = Sprawdzaj aktualizacje
admin-settings-help-updates-check = Czy serwer raz dziennie sprawdza, czy jest nowsze wydanie, i pokazuje je na pulpicie.
admin-settings-key-updates-feed-url = Kanał wydań
admin-settings-help-updates-feed-url = Adres https, który odczytuje sprawdzanie aktualizacji.
admin-settings-key-updates-apply = Stosuj aktualizacje
admin-settings-help-updates-apply = Czy aktualizację można uruchomić z konsoli za pośrednictwem agenta aktualizacji hosta.
admin-settings-key-backup-retention-count = Przechowywane kopie zapasowe
admin-settings-help-backup-retention-count = Ile kopii zapasowych bazy danych jest przechowywanych.
admin-settings-key-about-security-contact = Kontakt ds. bezpieczeństwa
admin-settings-help-about-security-contact = Kontakt w pliku security.txt (mailto: lub https:).
admin-settings-key-about-security-policy-url = Polityka bezpieczeństwa
admin-settings-help-about-security-policy-url = Adres polityki bezpieczeństwa, podany w pliku security.txt.
admin-settings-key-about-project-links = Linki w informacjach
admin-settings-help-about-project-links = Linki pokazywane na stronie Informacje, każdy z etykietą i adresem https.
admin-settings-key-config-snapshots-kept = Migawki konfiguracji
admin-settings-help-config-snapshots-kept = Ile migawek konfiguracji jest przechowywanych.
admin-settings-key-jobs-history-days = Historia zadań
admin-settings-help-jobs-history-days = Ile dni uruchomień zadań jest przechowywanych.
admin-settings-key-maintenance-announce-minutes-default = Wyprzedzenie powiadomienia o konserwacji
admin-settings-help-maintenance-announce-minutes-default = Ile minut przed oknem konserwacji użytkownicy są o nim informowani.
admin-settings-key-maintenance-default-message = Komunikat o konserwacji
admin-settings-help-maintenance-default-message = Tekst widoczny dla użytkowników podczas konserwacji, osobno dla każdego języka.
admin-settings-key-health-disk-free-min-percent = Minimum wolnego miejsca na dysku
admin-settings-help-health-disk-free-min-percent = Kontrola gotowości kończy się niepowodzeniem, gdy wolne miejsce na dysku spadnie poniżej tego procentu.
admin-settings-key-observability-otlp-metrics = Wysyłaj metryki przez OTLP
admin-settings-help-observability-otlp-metrics = Wysyła metryki serwera do kolektora OTLP ustawionego przy uruchomieniu.
admin-settings-key-metrics-drilldown-top-n = Wiersze szczegółów metryk
admin-settings-help-metrics-drilldown-top-n = Ile wierszy pokazuje rozbicie metryk.
admin-settings-key-metrics-rollup-days = Historia metryk
admin-settings-help-metrics-rollup-days = Ile dni godzinowych podsumowań metryk jest przechowywanych.
admin-settings-key-observability-client-timing-enabled = Czasy ładowania w przeglądarce
admin-settings-help-observability-client-timing-enabled = Zbiera czasy ładowania stron z zalogowanych przeglądarek jako anonimowe zbiorcze histogramy. Nic nie identyfikuje osoby, sesji ani strony. Wyłącz, aby zatrzymać zbieranie.
admin-settings-key-observability-client-timing-sample-rate-percent = Odsetek próbkowania czasów ładowania w przeglądarce
admin-settings-help-observability-client-timing-sample-rate-percent = Odsetek sesji przeglądarki, które zgłaszają czasy ładowania (od 1 do 100).
admin-settings-key-observability-client-timing-min-samples = Minimalna liczba próbek czasów ładowania
admin-settings-help-observability-client-timing-min-samples = Mediana i 95. percentyl czasów ładowania pozostają ukryte, dopóki okres nie ma co najmniej tylu próbek.
admin-settings-key-observability-client-timing-folder-loads = Czasy ładowania folderów
admin-settings-help-observability-client-timing-folder-loads = Mierzy, jak długo folder pokazuje listę wiadomości, z podziałem na dane z pamięci podręcznej przeglądarki i z sieci. Domyślnie włączone. Użytkownicy mogą to wyłączyć dla siebie, chyba że to ustawienie jest zablokowane. Wymaga włączonych czasów ładowania w przeglądarce.
admin-settings-key-observability-client-timing-message-loads = Czasy ładowania wiadomości
admin-settings-help-observability-client-timing-message-loads = Mierzy, jak długo wiadomość pokazuje nagłówki i oczyszczoną treść, z podziałem na dane z pamięci podręcznej lub pobrane z wyprzedzeniem oraz z sieci. Domyślnie włączone. Użytkownicy mogą to wyłączyć dla siebie, chyba że to ustawienie jest zablokowane. Wymaga włączonych czasów ładowania w przeglądarce.
admin-settings-key-diagnostics-retention-days = Historia diagnostyki
admin-settings-help-diagnostics-retention-days = Ile dni uruchomień diagnostycznych jest przechowywanych.
admin-settings-key-data-export-link-hours = Okno pobierania eksportu danych
admin-settings-help-data-export-link-hours = Ile godzin można pobierać eksport danych, zanim zostanie usunięty.
admin-settings-key-dsr-enabled = Wnioski o dane
admin-settings-help-dsr-enabled = Czy osoby mogą składać wnioski o dane (dostęp, pobranie, sprostowanie, usunięcie, ograniczenie, sprzeciw) w Ustawieniach.
admin-settings-key-dsr-type-access = Przyjmuj wnioski o dostęp
admin-settings-help-dsr-type-access = Czy typ wniosku o dostęp jest oferowany użytkownikom.
admin-settings-key-dsr-type-portability = Przyjmuj wnioski o pobranie (kopię przenośną)
admin-settings-help-dsr-type-portability = Czy typ wniosku o pobranie (kopię przenośną) jest oferowany użytkownikom.
admin-settings-key-dsr-type-rectification = Przyjmuj wnioski o sprostowanie
admin-settings-help-dsr-type-rectification = Czy typ wniosku o sprostowanie jest oferowany użytkownikom.
admin-settings-key-dsr-type-erasure = Przyjmuj wnioski o usunięcie
admin-settings-help-dsr-type-erasure = Czy typ wniosku o usunięcie jest oferowany użytkownikom.
admin-settings-key-dsr-type-restriction = Przyjmuj wnioski o ograniczenie
admin-settings-help-dsr-type-restriction = Czy typ wniosku o ograniczenie jest oferowany użytkownikom.
admin-settings-key-dsr-type-objection = Przyjmuj wnioski o sprzeciw
admin-settings-help-dsr-type-objection = Czy typ wniosku o sprzeciw jest oferowany użytkownikom.
admin-settings-key-dsr-type-other = Przyjmuj inne wnioski
admin-settings-help-dsr-type-other = Czy ogólny typ wniosku jest oferowany użytkownikom.
admin-settings-key-dsr-approval-export = Zatwierdzanie wniosków o dostęp i pobranie
admin-settings-help-dsr-approval-export = auto uruchamia eksport po potwierdzeniu wniosku. admin czeka na zatwierdzenie przez administratora.
admin-settings-key-dsr-approval-erasure = Zatwierdzanie wniosków o usunięcie
admin-settings-help-dsr-approval-erasure = auto planuje usunięcie po potwierdzeniu wniosku. admin czeka na zatwierdzenie przez administratora.
admin-settings-key-dsr-approval-other = Obsługa innych wniosków
admin-settings-help-dsr-approval-other = auto oznacza wniosek jako w toku po potwierdzeniu. admin czeka na zatwierdzenie przez administratora.
admin-settings-key-dsr-reauth-required = Ponowne uwierzytelnienie przy wnioskach
admin-settings-help-dsr-reauth-required = Wymaga świeżego hasła lub drugiego składnika, aby złożyć wniosek o eksport lub usunięcie i aby utworzyć link do pobrania.
admin-settings-key-dsr-confirmation-required = Wiadomość potwierdzająca
admin-settings-help-dsr-confirmation-required = Wysyła do skrzynki wnioskodawcy wiadomość potwierdzającą z kodem; wniosek nie jest realizowany, dopóki kod nie zostanie wpisany.
admin-settings-key-dsr-identity-verification-required = Krok weryfikacji tożsamości
admin-settings-help-dsr-identity-verification-required = Administrator musi odnotować, że tożsamość wnioskodawcy została zweryfikowana, zanim wniosek zostanie zatwierdzony.
admin-settings-key-dsr-erasure-grace-days = Okres karencji usunięcia
admin-settings-help-dsr-erasure-grace-days = Liczba dni między zatwierdzeniem a usunięciem. W tym czasie wnioskodawca może anulować. 0 wykonuje usunięcie przy następnym przebiegu procesu roboczego.
admin-settings-key-dsr-sla-days = Dni na odpowiedź na wniosek
admin-settings-help-dsr-sla-days = Liczba dni od złożenia do terminu pokazywanego użytkownikom i administratorom.
admin-settings-key-dsr-sla-max-extension-days = Najdłuższe przedłużenie
admin-settings-help-dsr-sla-max-extension-days = Największa liczba dni, jaką administrator może dodać do terminu wniosku, z zapisanym powodem.
admin-settings-key-dsr-sla-reminder-days = Oznaczenia zbliżającego się terminu
admin-settings-help-dsr-sla-reminder-days = Liczba dni przed terminem, od której wniosek jest oznaczany w kolejce jako wkrótce wymagalny.
admin-settings-key-dsr-export-mail-allowed = Poczta w eksportach
admin-settings-help-dsr-export-mail-allowed = Pozwala wnioskodawcy dołączyć wiadomości do eksportu. Uwzględniane są tylko wiadomości przechowywane w pamięci podręcznej tego serwera; poczta pozostająca na nadrzędnym serwerze poczty nie jest uwzględniana.
admin-settings-key-dsr-export-mail-format = Format poczty w eksportach
admin-settings-help-dsr-export-mail-format = mbox zapisuje jeden plik na folder; eml zapisuje jeden plik na wiadomość.
admin-settings-key-dsr-export-include-settings = Eksport: ustawienia
admin-settings-help-dsr-export-include-settings = Dołącz ustawienia do eksportu.
admin-settings-key-dsr-export-include-identities = Eksport: tożsamości
admin-settings-help-dsr-export-include-identities = Dołącz tożsamości do eksportu.
admin-settings-key-dsr-export-include-contacts = Eksport: kontakty
admin-settings-help-dsr-export-include-contacts = Dołącz kontakty do eksportu.
admin-settings-key-dsr-export-include-calendars = Eksport: kalendarze
admin-settings-help-dsr-export-include-calendars = Dołącz kalendarze do eksportu.
admin-settings-key-dsr-export-include-notes = Eksport: notatki
admin-settings-help-dsr-export-include-notes = Dołącz notatki do eksportu.
admin-settings-key-dsr-export-include-rules = Eksport: filtry i reguły
admin-settings-help-dsr-export-include-rules = Dołącz filtry i reguły do eksportu.
admin-settings-key-dsr-export-include-sessions = Eksport: sesje, urządzenia i historia logowania
admin-settings-help-dsr-export-include-sessions = Dołącz sesje, urządzenia i historię logowania do eksportu.
admin-settings-key-dsr-export-include-consent = Eksport: zapisy zgód
admin-settings-help-dsr-export-include-consent = Dołącz zapisy zgód do eksportu.
admin-settings-key-dsr-export-include-audit = Eksport: wpisy audytu dotyczące osoby
admin-settings-help-dsr-export-include-audit = Dołącz do eksportu wpisy audytu dotyczące osoby.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-dsr-export-max-megabytes = Limit rozmiaru eksportu
admin-settings-help-dsr-export-max-megabytes = Największy eksport w megabajtach. Sekcje, które by go przekroczyły, są pomijane i wymienione w manifeście.
admin-settings-key-dsr-export-format = Formaty eksportu
admin-settings-help-dsr-export-format = json zapisuje pliki JSON. jsonCsv zapisuje też pliki CSV dla danych w postaci list.
admin-settings-key-dsr-export-encrypt = Szyfruj eksporty w spoczynku
admin-settings-help-dsr-export-encrypt = Przechowuje utworzone eksporty zaszyfrowane kluczem serwera do czasu pobrania lub wygaśnięcia.
admin-settings-key-dsr-download-link-minutes = Czas ważności linku do pobrania
admin-settings-help-dsr-download-link-minutes = Liczba minut, przez które link do pobrania działa po utworzeniu. Sam eksport jest przechowywany przez okno pobierania eksportu danych.
admin-settings-key-dsr-download-single-use = Link do pobrania jednorazowy
admin-settings-help-dsr-download-single-use = Link do pobrania przestaje działać po jednokrotnym użyciu.
admin-settings-key-dsr-retention-completed-days = Przechowywanie zakończonych wniosków
admin-settings-help-dsr-retention-completed-days = Liczba dni, przez które zakończony wniosek i jego ślad są przechowywane przed usunięciem.
admin-settings-key-dsr-notify-admins = Zapisuj nowe wnioski dla administratorów
admin-settings-help-dsr-notify-admins = Zapisuje każdy nowy wniosek jako zdarzenie bezpieczeństwa w dzienniku audytu, gdzie widzą go administratorzy i odbiorniki alertów.
admin-settings-key-dsr-notify-requester = Wiadomości o postępie dla wnioskodawcy
admin-settings-help-dsr-notify-requester = Umieszcza wiadomość w skrzynce wnioskodawcy, gdy wniosek zmieni stan.
admin-settings-key-dsr-contact-name = Nazwa kontaktu ds. ochrony danych
admin-settings-help-dsr-contact-name = Nazwa pokazywana użytkownikom na stronie wniosków o dane.
admin-settings-key-dsr-contact-email = Adres kontaktu ds. ochrony danych
admin-settings-help-dsr-contact-email = Adres e-mail pokazywany użytkownikom na stronie wniosków o dane.
admin-settings-key-dsr-contact-url = Strona kontaktu ds. ochrony danych
admin-settings-help-dsr-contact-url = Link pokazywany użytkownikom na stronie wniosków o dane.
admin-settings-key-dsr-on-behalf-allowed = Wnioski w imieniu użytkownika
admin-settings-help-dsr-on-behalf-allowed = Pozwala administratorowi złożyć wniosek za użytkownika, na przykład otrzymany pocztą.
admin-settings-key-dsr-rate-limit-per-day = Wnioski na osobę dziennie
admin-settings-help-dsr-rate-limit-per-day = Największa liczba wniosków, jaką jedna osoba może złożyć w ciągu 24 godzin.
admin-settings-key-dsr-legal-hold-subjects = Blokady prawne
admin-settings-help-dsr-legal-hold-subjects = Adresy lub wpisy @domena, których usunięcie jest zablokowane. Wniosek o usunięcie dla nich pozostaje w kolejce jako zablokowany do czasu usunięcia wpisu.
admin-settings-key-scrub-enabled = Wymazywanie danych osoby
admin-settings-help-scrub-enabled = Zezwala na działanie Wymaż dane osoby oraz wymazanie, które może nastąpić po usunięciu. Gdy wyłączone, żadne z nich nie działa.
admin-settings-key-scrub-pseudonym-algorithm = Długość pseudonimu
admin-settings-help-scrub-pseudonym-algorithm = Jak tworzony jest kluczowany pseudonim zastępujący identyfikator: HMAC-SHA256 pod bieżącą generacją klucza, obcięty do 16 lub 32 znaków szesnastkowych.
admin-settings-key-scrub-include-logs = Wymazuj pliki dziennika
admin-settings-help-scrub-include-logs = Wymazanie zmienia też identyfikator w rotowanych plikach dziennika, w tym skompresowanych.
admin-settings-key-scrub-auto-run-on-erasure = Wymazanie po usunięciu
admin-settings-help-scrub-auto-run-on-erasure = Zakończone usunięcie zastępuje też identyfikatory osoby w historii audytu i dziennikach pseudonimem. Domyślnie wyłączone.
admin-settings-key-scrub-key-retention-days = Przechowywanie klucza pseudonimów
admin-settings-help-scrub-key-retention-days = Liczba dni przechowywania klucza pseudonimów po jego wycofaniu z rotacji. 0 przechowuje go bezterminowo. Po usunięciu nie można już ponownie obliczyć starych pseudonimów ani odtworzyć znaczników usunięcia utworzonych tym kluczem.
admin-settings-key-backup-propagate-erasure = Odtwarzaj usunięcia przy przywracaniu
admin-settings-help-backup-propagate-erasure = Przywracanie odtwarza znaczniki usunięcia nowsze niż kopia zapasowa, więc osoby usunięte od tego czasu są usuwane ponownie.
admin-settings-key-backup-tombstone-retention-days = Przechowywanie znaczników usunięcia
admin-settings-help-backup-tombstone-retention-days = Liczba dni przechowywania znacznika usunięcia. 0 przechowuje je bezterminowo.
admin-settings-key-dsr-text-intro = Wstęp do wniosków o dane
admin-settings-help-dsr-text-intro = Tekst pokazywany nad formularzem wniosku. Jedna wartość na język.
admin-settings-key-dsr-text-confirmation = Tekst wiadomości potwierdzającej
admin-settings-help-dsr-text-confirmation = Tekst dodawany do wiadomości potwierdzającej. Jedna wartość na język.
admin-settings-key-dsr-text-completion = Tekst wiadomości o zakończeniu
admin-settings-help-dsr-text-completion = Tekst dodawany do wiadomości wysyłanej po zakończeniu wniosku. Jedna wartość na język.
admin-settings-key-server-text-family = Teksty serwera
admin-settings-help-server-text-family = Tekst, który serwer wpisuje w pocztę i powiadomienia, osobno dla języka i domeny.
admin-settings-cat-sieve = Filtry poczty
admin-settings-key-sieve-max-script-bytes = Maksymalny rozmiar skryptu filtra
admin-settings-help-sieve-max-script-bytes = Maksymalna liczba bajtów w pojedynczym skrypcie filtra Sieve.
admin-settings-key-sieve-max-scripts = Maksymalna liczba skryptów filtrów
admin-settings-help-sieve-max-scripts = Maksymalna liczba zapisanych skryptów na konto.
admin-settings-key-sieve-max-redirects = Maksymalna liczba przekierowań
admin-settings-help-sieve-max-redirects = Maksymalna liczba poleceń przekierowania w jednym skrypcie.
admin-settings-key-sieve-vacation-allowed = Akcje autoodpowiedzi w filtrach
admin-settings-help-sieve-vacation-allowed = Zezwala skryptom na wysyłanie autoodpowiedzi, gdy obsługuje je zaplecze.
admin-settings-key-sieve-allowed-extensions = Dozwolone rozszerzenia filtrów
admin-settings-help-sieve-allowed-extensions = Opcjonalna lista nazw rozszerzeń Sieve pisanych małymi literami. Zaplecze musi też obsługiwać każde rozszerzenie.
admin-settings-key-cap-sieve-raw-edit = Edycja źródła skryptu filtra
admin-settings-help-cap-sieve-raw-edit = Zezwala na edycję surowych skryptów Sieve dla wybranych kont.
admin-settings-key-cap-sieve-forward-external = Przekazywanie poza domenę konta
admin-settings-help-cap-sieve-forward-external = Zezwala na przekazywanie zewnętrzne w skryptach. Zapisanie takich skryptów wymaga też niedawnej weryfikacji.
admin-settings-key-session-max-concurrent = Równoczesne sesje
admin-settings-help-session-max-concurrent = Maksymalna liczba sesji na konto. Najstarsze sesje są unieważniane, gdy nowe logowanie przekracza ten limit.
admin-settings-key-session-idle-secs = Limit bezczynności sesji
admin-settings-help-session-idle-secs = Kończy bezczynne sesje po tylu sekundach, do limitu wdrożenia.
admin-settings-key-session-absolute-secs = Czas życia sesji
admin-settings-help-session-absolute-secs = Kończy sesje po tylu sekundach, do limitu wdrożenia.
admin-settings-key-twofa-remember-days = Czas ważności zapamiętanego urządzenia
admin-settings-help-twofa-remember-days = Liczba dni, po których zapamiętane urządzenie musi ponownie zweryfikować drugi składnik. Zero wyłącza tę funkcję.
admin-settings-key-twofa-passkeys-enabled = Zezwalaj na klucze dostępu
admin-settings-help-twofa-passkeys-enabled = Zezwala na klucze dostępu jako składnik uwierzytelniania.
admin-settings-key-twofa-required = Wymagaj uwierzytelniania dwuskładnikowego
admin-settings-help-twofa-required = Wymaga zarejestrowanego drugiego składnika przed wydaniem sesji.
admin-settings-key-twofa-passkey-attachment = Typ authenticatora klucza dostępu
admin-settings-help-twofa-passkey-attachment = any (domyślnie): każdy authenticator. platform: klucze dostępu przechowywane na urządzeniu (Windows Hello, Touch ID, blokada ekranu Androida). cross-platform: klucze bezpieczeństwa i telefony. Informację podaje przeglądarka; nie jest podpisana przez authenticator.
admin-settings-key-twofa-passkey-user-verification = Weryfikacja użytkownika kluczem dostępu
admin-settings-help-twofa-passkey-user-verification = required: klucz dostępu musi sprawdzić PIN lub dane biometryczne przy dodawaniu i przy każdym logowaniu; procedura bez tego jest odrzucana. preferred (domyślnie) i discouraged zmieniają tylko to, o co prosi się przeglądarkę.
admin-settings-key-twofa-passkey-resident-key = Wykrywalne klucze dostępu
admin-settings-help-twofa-passkey-resident-key = required: poświadczenie musi być przechowywane w authenticatorze. discouraged (domyślnie) lub preferred. W trybie silnika klucze dostępu są zawsze wykrywalne, bo służą też do logowania bez hasła.
admin-settings-key-twofa-passkey-legacy = Klucze dostępu dodane przed obecną polityką
admin-settings-help-twofa-passkey-legacy = allow (domyślnie): starszy klucz dostępu niespełniający polityki nadal działa, a użytkownik jest proszony o jego wymianę. refuse: jest odrzucany przy logowaniu z komunikatem; użytkownik loguje się innym składnikiem lub kodem odzyskiwania.
admin-settings-key-twofa-last-factor-removal = Usuwanie ostatniego drugiego składnika
admin-settings-help-twofa-last-factor-removal = Gdy usunięcie klucza dostępu lub aplikacji uwierzytelniającej zostawiłoby konto bez żadnego. allow (domyślnie): jest usuwany, a logowanie wymaga tylko hasła. block: użytkownik musi najpierw dodać inną metodę logowania (kody odzyskiwania się nie liczą). Tam, gdzie uwierzytelnianie dwuskładnikowe jest wymagane, ostatni składnik jest zawsze zachowywany.
admin-settings-key-privacy-remote-content = Zawartość zdalna
admin-settings-help-privacy-remote-content = Blokuj obrazy, pytaj najpierw, używaj proxy lub ładuj bezpośrednio, gdy jest to dozwolone.
admin-settings-key-privacy-trackers = Blokuj trackery
admin-settings-help-privacy-trackers = Blokuje znane obrazy śledzące w e-mailach.
admin-settings-key-privacy-clean-links = Czyść linki
admin-settings-help-privacy-clean-links = Usuwa parametry śledzące z linków w wiadomościach.
admin-settings-key-privacy-max-bytes = Limit obrazu zdalnego
admin-settings-help-privacy-max-bytes = Maksymalna liczba bajtów pobieranych dla jednego obrazu zdalnego.
admin-settings-key-privacy-cache-bytes = Pamięć podręczna obrazów zdalnych
admin-settings-help-privacy-cache-bytes = Maksymalna liczba bajtów przechowywanych w pamięci podręcznej obrazów zdalnych.
admin-settings-key-privacy-rate-per-min = Częstość żądań obrazów zdalnych
admin-settings-help-privacy-rate-per-min = Maksymalna liczba pobrań obrazów zdalnych na konto na minutę.
admin-settings-privacy-fetch-title = Pobieranie obrazów zdalnych
admin-settings-privacy-fetch-help = Limity czasu, przekierowania i równoległe pobierania obrazów, które serwer pobiera dla czytelnika. Obraz przekraczający limit pokazuje symbol zastępczy.
admin-settings-key-privacy-fetch-connect-timeout-ms = Limit czasu połączenia z obrazem zdalnym
admin-settings-help-privacy-fetch-connect-timeout-ms = Milisekundy dozwolone na otwarcie połączenia z hostem obrazu, wraz z uzgadnianiem TLS. Domyślnie 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Limit czasu odpowiedzi obrazu zdalnego
admin-settings-help-privacy-fetch-headers-timeout-ms = Milisekundy dozwolone od początku żądania do odpowiedzi hosta obrazu nagłówkami. Domyślnie 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Limit czasu zatrzymania obrazu zdalnego
admin-settings-help-privacy-fetch-idle-timeout-ms = Najdłuższa przerwa, w milisekundach, między dwiema częściami pobieranego obrazu, po której pobieranie jest przerywane. Domyślnie 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Całkowity limit czasu obrazu zdalnego
admin-settings-help-privacy-fetch-total-timeout-ms = Milisekundy dozwolone na pobranie obrazu, wraz z przekierowaniami. Oczekiwanie w kolejce ma osobny budżet, więc zajęty obraz nie zużywa czasu sieciowego, zanim się zacznie. Domyślnie 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Limit czasu kolejki obrazów zdalnych
admin-settings-help-privacy-fetch-queue-timeout-ms = Maksymalna liczba milisekund, przez które obrazy mogą czekać na wolne zasoby, ponad ich budżet pobierania. Ograniczona kolejka obsługuje duże serie uwierzytelnionych żądań, a limity aktywnych pobrań i dekodowania nadal obowiązują. Domyślnie 60000.
admin-settings-key-privacy-max-redirects = Przekierowania obrazów zdalnych
admin-settings-help-privacy-max-redirects = Liczba przekierowań obsługiwanych dla jednego obrazu. Każde jest sprawdzane według tych samych reguł adresów co pierwsze żądanie. Domyślnie 3.
admin-settings-key-privacy-max-concurrent-per-message = Pobrania obrazów zdalnych na wiadomość
admin-settings-help-privacy-max-concurrent-per-message = Liczba obrazów jednej wiadomości pobieranych jednocześnie. Pozostałe czekają na wolne miejsce. Zakres 1–128; domyślnie 30.
admin-settings-key-privacy-max-concurrent-per-account = Pobrania obrazów zdalnych na konto
admin-settings-help-privacy-max-concurrent-per-account = Liczba obrazów pobieranych jednocześnie dla jednego konta ze wszystkich jego otwartych wiadomości. Zakres 1–256; domyślnie 128.
admin-settings-key-privacy-allow-direct = Zezwalaj na obrazy bezpośrednie
admin-settings-key-cap-remote-content-direct = Uprawnienie do obrazów bezpośrednich
admin-settings-help-cap-remote-content-direct = Pozwala temu kontu wybierać obrazy bezpośrednie, gdy serwer na to zezwala.
admin-settings-help-privacy-allow-direct = Zezwala na bezpośrednie ładowanie obrazów dla kont z wymaganym uprawnieniem.
admin-settings-key-logs-auth-retention-days = Przechowywanie historii uwierzytelniania
admin-settings-help-logs-auth-retention-days = Liczba dni przechowywania metadanych uwierzytelniania, niezależnie od trwałej tożsamości konta.
admin-settings-key-logs-auth-max-entries = Limit wpisów historii uwierzytelniania
admin-settings-help-logs-auth-max-entries = Maksymalna liczba przechowywanych wpisów. Pasujące powtarzające się niepowodzenia w ciągu jednej minuty dzielą jeden wpis.
admin-settings-key-reader-color-mode = Kolory wiadomości
admin-settings-cat-reader = Wyświetlanie wiadomości
admin-settings-key-mail-row-actions-mode = Szybkie akcje na liście wiadomości
admin-settings-help-mail-row-actions-mode = Jak wyglądają szybkie akcje w każdym wierszu wiadomości: open (zawsze dostępne, ujawniane po najechaniu lub fokusie), hover (trzy kontrolki i przycisk Więcej), click (jeden przycisk, który je otwiera) lub off (usunięte z wierszy; menu kontekstowe i pasek narzędzi zachowują każdą akcję). Hover działa jak click na ekranach dotykowych. Użytkownicy mogą wybrać własne, chyba że administrator to zablokuje.
admin-settings-key-mail-quick-reply = Szybka odpowiedź
admin-settings-help-mail-quick-reply = Oferuje zwartą kartę odpowiedzi z już wypełnionymi odbiorcami. Domyślnie wyłączone; użytkownicy mogą ją włączyć, chyba że zablokuje to administrator. Zwykłe zabezpieczenia wysyłania nadal obowiązują.
admin-settings-key-mail-request-read-receipt = Domyślnie żądaj potwierdzeń odczytu
admin-settings-help-mail-request-read-receipt = Rozpoczyna obsługiwane wiadomości wychodzące z wybranym żądaniem potwierdzenia. To domyślne ustawienie redagowania, a nie gwarancja, że odbiorca wyśle potwierdzenie.
admin-settings-help-reader-color-mode = Automatyczny podąża za ciemnymi motywami. Wybierz renderowanie oryginalne, adaptacyjne lub odwrócone wewnątrz izolowanej ramki wiadomości.
admin-settings-key-reader-font-size = Rozmiar czcionki wiadomości
admin-settings-help-reader-font-size = Domyślny rozmiar czytania w pikselach, od 12 do 24.
admin-settings-key-reader-line-height = Wysokość wiersza wiadomości
admin-settings-help-reader-line-height = Odstęp między wierszami podczas czytania, od 1,2 do 2,2.
admin-settings-key-reader-content-width = Szerokość czytania wiadomości
admin-settings-help-reader-content-width = Treść wiadomości o szerokości wygodnej, dużej lub na cały panel.
admin-settings-key-reader-loading = Wygląd ładowania wiadomości
admin-settings-help-reader-loading = Linie zastępcze, wskaźnik tekstowy lub brak wskaźnika wizualnego. Komunikaty dostępności pozostają dostępne.
admin-settings-key-reader-loading-delay-ms = Opóźnienie ładowania wiadomości
admin-settings-help-reader-loading-delay-ms = Opóźnia tylko wskaźnik ładowania, od 0 do 1000 milisekund. Dostarczenie wiadomości nigdy nie jest opóźniane.
admin-settings-key-reader-zoom = Powiększenie wiadomości
admin-settings-help-reader-zoom = Domyślne powiększenie treści wiadomości w procentach, od 50 do 200; czytnik używa najbliższego kroku powiększenia. Skaluje się tylko wiadomość, a drukowanie używa 100%.
admin-settings-key-reader-attachments-collapsed = Pasek załączników zwinięty
admin-settings-help-reader-attachments-collapsed = Czy pasek załączników czytnika zaczyna zwinięty, pokazując tylko liczbę i łączny rozmiar. Ostatnie otwarcie lub zamknięcie użytkownika jest tu zapisywane. Domyślnie wyłączone: załączniki są pokazane.
admin-settings-key-login-show-domain-hint = Pokazuj preferowaną domenę przy logowaniu
admin-settings-help-login-show-domain-hint = Pokazuje, która domena przyjmuje nazwę użytkownika bez sufiksu e-mail. Domyślnie ukryte. Zmienia tylko podpowiedź; logowanie nazwą użytkownika nadal działa.
admin-settings-key-signin-retry-cooldown-secs = Przerwa przed ponowną próbą logowania
admin-settings-help-signin-retry-cooldown-secs = Minimalna liczba sekund między próbami logowania hasłem z jednej sieci klienta. Domyślnie 3; 0 wyłącza przerwę. Dotychczasowe blokady po niepowodzeniach nadal obowiązują.
admin-settings-key-mail-quote-style = Styl nagłówka odpowiedzi i przekazania
admin-settings-key-mail-external-recipient-warning = Ostrzeżenie o zewnętrznym odbiorcy
admin-settings-help-mail-external-recipient-warning = Wyłączone, ostrzeżenie lub wymóg potwierdzenia w oknie redagowania. Obejmuje Do, DW i UDW. To zabezpieczenie okna redagowania, a nie ograniczenie dostarczania na serwerze.
admin-settings-key-mail-trusted-recipient-domains = Zaufane domeny odbiorców
admin-settings-help-mail-trusted-recipient-domains = Do 100 dokładnych domen odbiorców. Domena wybranej tożsamości nadawcy też jest zaufana. Nieprawidłowe wpisy nigdy nie dają zaufania. Zablokuj to ustawienie, aby uniemożliwić nadpisywanie przez użytkowników.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-quote-style = Użyj klasycznych odpowiedzi z cytatem lub nagłówków wiadomości w stylu Outlooka. To ustawia domyślną wartość użytkownika; zablokuj ją, aby wymusić styl. Istniejące wersje robocze nie zmieniają się.
admin-settings-key-signin-anti-bot-mode = Ochrona logowania przed botami
admin-settings-help-signin-anti-bot-mode = Własna weryfikacja proof-of-work: off, mailbox, admin lub all. Nie jest kontaktowana żadna zewnętrzna usługa weryfikacji.
admin-settings-key-signin-anti-bot-difficulty = Nakład weryfikacji logowania
admin-settings-help-signin-anti-bot-difficulty = Maksymalny licznik rozwiązania, 100–20000. Wyższe wartości wymagają więcej pracy przeglądarki; domyślnie 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Czas ważności weryfikacji logowania
admin-settings-help-signin-anti-bot-expiry-secs = Liczba sekund, przez które jednorazowa weryfikacja pozostaje ważna, 60–600; domyślnie 180. Po każdej próbie podania poświadczeń potrzebna jest nowa weryfikacja.
admin-settings-key-attachments-preview-allowed = Zezwalaj na podglądy załączników
admin-settings-help-attachments-preview-allowed = Wyłącz, aby uniemożliwić renderowanie podglądu załączników; pobieranie oryginałów ma własne ustawienie.
admin-settings-key-attachments-download-allowed = Zezwalaj na dostarczanie oryginalnych załączników
admin-settings-help-attachments-download-allowed = Wyłączenie uniemożliwia też podglądy wymagające oryginalnych bajtów. Podglądy w przeglądarce nie są ochroną przed kopiowaniem.
admin-settings-key-attachments-max-bytes = Maksymalna liczba bajtów załącznika
admin-settings-help-attachments-max-bytes = Zero oznacza brak limitu. Dotyczy dostarczonych bajtów oryginału, w tym żądań zakresu.
admin-settings-key-attachments-allowed-extensions = Dozwolone rozszerzenia nazw plików
admin-settings-help-attachments-allowed-extensions = Tablica JSON rozszerzeń małymi literami bez kropki, np. ["pdf","tar.gz"]. Puste zezwala na każde rozszerzenie; reguły blokujące mają pierwszeństwo.
admin-settings-key-attachments-blocked-extensions = Zablokowane rozszerzenia nazw plików
admin-settings-help-attachments-blocked-extensions = Tablica JSON rozszerzeń małymi literami. Oceniane względem zapisanych nazw plików MIME, nigdy nazwy z adresu pobierania.
admin-settings-key-attachments-warn-extensions = Ostrzegaj o rozszerzeniach nazw plików
admin-settings-help-attachments-warn-extensions = Tablica JSON rozszerzeń wymagających jawnego potwierdzenia przed dostarczeniem oryginału.
admin-settings-key-attachments-allowed-mime-types = Dozwolone typy MIME
admin-settings-help-attachments-allowed-mime-types = Tablica JSON typów małymi literami lub symboli wieloznacznych najwyższego poziomu, np. ["image/*","application/pdf"]. Puste zezwala na każdy typ.
admin-settings-key-attachments-blocked-mime-types = Zablokowane typy MIME
admin-settings-help-attachments-blocked-mime-types = Tablica JSON typów małymi literami lub symboli wieloznacznych najwyższego poziomu. Reguły blokujące mają pierwszeństwo.
admin-settings-key-attachments-warn-mime-types = Ostrzegaj o typach MIME
admin-settings-help-attachments-warn-mime-types = Tablica JSON typów małymi literami lub symboli wieloznacznych najwyższego poziomu wymagających potwierdzenia. Metadane MIME nie są wykrywaniem złośliwego oprogramowania.
admin-settings-cat-shortcuts = Skróty klawiszowe
admin-settings-cat-time = Strefy czasowe
admin-settings-key-time-display-zone = Strefa czasowa wyświetlania
admin-settings-help-time-display-zone = Strefa czasowa IANA, np. Europe/Lisbon, lub local, aby podążać za bieżącym urządzeniem. Znaczniki czasu wiadomości pozostają zapisane jako ich pierwotne chwile.
admin-settings-key-time-comparison-zone = Strefa czasowa porównania
admin-settings-help-time-comparison-zone = Opcjonalna druga strefa czasowa IANA dla pomocników wiadomości i kalendarza. Pozostaw puste, aby ukryć porównanie; lokalizacja nie jest wnioskowana z adresu e-mail.
admin-settings-key-time-show-helpers = Pokazuj pomocniki stref czasowych
admin-settings-help-time-show-helpers = Pokazuje porównania czasu lokalnego i wybranej strefy dla znaczników czasu wiadomości i wydarzeń kalendarza. Blokady administratora mogą wymusić ten wybór.
admin-settings-key-shortcuts-enabled = Włącz skróty klawiszowe
admin-settings-help-shortcuts-enabled = Zezwala na skróty aplikacji. Blokady mogą je wyłączyć globalnie lub dla domeny. Pola tekstowe i okna dialogowe zachowują zwykłą obsługę klawiatury.
admin-settings-key-shortcuts-compose = Skrót nowej wiadomości
admin-settings-help-shortcuts-compose = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-archive = Skrót archiwizacji
admin-settings-help-shortcuts-archive = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-reply = Skrót odpowiedzi
admin-settings-help-shortcuts-reply = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-reply-all = Skrót odpowiedzi wszystkim
admin-settings-help-shortcuts-reply-all = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-forward = Skrót przekazania
admin-settings-help-shortcuts-forward = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-next = Skrót następnej wiadomości
admin-settings-help-shortcuts-next = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-previous = Skrót poprzedniej wiadomości
admin-settings-help-shortcuts-previous = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-search = Skrót przejścia do wyszukiwania
admin-settings-help-shortcuts-search = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-refresh = Skrót odświeżania wiadomości
admin-settings-help-shortcuts-refresh = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-settings = Skrót otwarcia ustawień
admin-settings-help-shortcuts-settings = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-trash = Skrót przeniesienia do kosza
admin-settings-help-shortcuts-trash = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-mark-read = Skrót oznaczenia jako przeczytane
admin-settings-help-shortcuts-mark-read = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-mark-unread = Skrót oznaczenia jako nieprzeczytane
admin-settings-help-shortcuts-mark-unread = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-spam = Skrót oznaczenia jako spam
admin-settings-help-shortcuts-spam = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-zoom-in = Skrót powiększenia wiadomości
admin-settings-help-shortcuts-zoom-in = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-zoom-out = Skrót pomniejszenia wiadomości
admin-settings-help-shortcuts-zoom-out = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-zoom-reset = Skrót zerowania powiększenia wiadomości
admin-settings-help-shortcuts-zoom-reset = Użyj inherit dla wybranego zestawu, disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-shortcuts-open-in-tab = Skrót otwarcia wiadomości we własnej karcie
admin-settings-help-shortcuts-open-in-tab = Użyj inherit dla wybranego zestawu (Shift+Enter), disabled, aby usunąć tę akcję, lub kombinacji, np. Ctrl+Shift+K. Kolidujące lub nieprawidłowe kombinacje nie działają.
admin-settings-key-notifications-push-allowed = Powiadomienia push w tle dozwolone
admin-settings-help-notifications-push-allowed = Pozwala włączonym powiadomieniom tego konta rejestrować się i odbierać powiadomienia push w tle. Uprawnienie przeglądarki jest nadal wymagane.
admin-settings-key-offline-persistent-storage-allowed = Żądania trwałej pamięci przeglądarki dozwolone
admin-settings-help-offline-persistent-storage-allowed = Zezwala na jawne żądanie ochrony lokalnych danych w pamięci podręcznej przed automatycznym usunięciem przez przeglądarkę. Przeglądarka może odmówić; nie przyznaje to uprawnienia automatycznie.
admin-settings-key-security-signin-mailbox-failure-limit = Limit nieudanych poświadczeń skrzynki
admin-settings-help-security-signin-mailbox-failure-limit = Odrzucone poświadczenia z jednego źródłowego adresu IP w oknie zliczania wywołują tymczasową blokadę logowania do skrzynki. Awarie sieci i konfiguracji nie są liczone.
admin-settings-key-security-signin-admin-failure-limit = Limit nieudanych poświadczeń administratora
admin-settings-help-security-signin-admin-failure-limit = Odrzucone poświadczenia administratora z jednego źródłowego adresu IP wywołują osobną tymczasową blokadę logowania administratora. Krótka sesja administratora i wymogi świeżego potwierdzenia pozostają bez zmian.
admin-settings-key-security-signin-failure-window-secs = Okno zliczania nieudanych poświadczeń
admin-settings-help-security-signin-failure-window-secs = Okno przesuwne w sekundach dla odrzuceń poświadczeń i MFA. Pomyślnie ukończone logowanie zeruje licznik dla tego źródła i powierzchni.
admin-settings-key-security-signin-lock-secs = Czas tymczasowej blokady logowania
admin-settings-help-security-signin-lock-secs = Czas blokady w sekundach. Powtarzane próby podczas blokady nie wydłużają jej pierwotnego wygaśnięcia. Ręczne globalne blokady IP są osobne.
admin-settings-key-security-signin-geo-scope = Zakres polityki krajów logowania
admin-settings-help-security-signin-geo-scope = Domyślnie wyłączone. Stosuj tylko do nowych logowań do skrzynki, administratora lub wszystkich. Istniejące sesje nie są unieważniane. Przed włączeniem sprawdź diagnostykę bieżącego źródła; niedostępna lokalna baza GeoIP powoduje odmowę logowania.
admin-settings-key-security-signin-geo-mode = Lista dozwolonych lub zabronionych krajów
admin-settings-help-security-signin-geo-mode = Allow dopuszcza tylko wymienione kraje; deny odmawia dla wymienionych krajów. Nieznane adresy mają własną jawną politykę.
admin-settings-key-security-signin-geo-countries = Kody krajów logowania
admin-settings-help-security-signin-geo-countries = Tablica JSON dwuliterowych kodów krajów wielkimi literami, np. ["PT", "GB"]. Używa tylko lokalnej bazy krajów (Bezpieczeństwo > Baza krajów lub MW_GEOIP_DB); adresy nigdy nie są wysyłane do usługi lokalizacji.
admin-settings-key-security-signin-geo-unknown = Polityka dla nieprzypisanych adresów źródłowych
admin-settings-help-security-signin-geo-unknown = Zezwala lub odmawia dla adresów nieobecnych w czytelnej lokalnej bazie, w tym adresów prywatnych. Brakująca lub nieczytelna baza zawsze odmawia logowań objętych aktywną polityką, nawet gdy nieznane adresy są dozwolone.
admin-settings-key-contacts-avatar-mode = Zdjęcia kontaktów i generowane awatary
admin-settings-help-contacts-avatar-mode = Generuje inicjały lokalnie, ukrywa awatary lub używa osadzonych zdjęć z własnych książek adresowych każdego konta. Nie są kontaktowane zdalne adresy zdjęć ani zewnętrzna usługa awatarów. Politykę użytkownika i domeny może zablokować administrator.
admin-settings-cat-contacts = Zdjęcia kontaktów
admin-settings-key-reader-action-style = Wstążka akcji wiadomości
admin-settings-help-reader-action-style = Domyślnie kompaktowe ikony. Wybierz etykiety, aby ikony miały widoczny tekst; oba warianty zachowują nazwy akcji i motywowane podpowiedzi. Obsługuje blokady globalne, domeny i użytkownika.
admin-settings-key-nextcloud-sync-interval-secs = Odstęp automatycznego odświeżania Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Liczba sekund między odświeżeniami kolekcji w tle dla połączonych kont silnika natywnego. Zero oznacza tylko ręcznie; wartości dodatnie mają minimum sześćdziesięciu sekund. Po niepowodzeniach odstęp rośnie. Nie jest podejmowane logowanie w tle.
admin-settings-key-mail-folder-role-inbox = Mapowanie folderu Odebrane
admin-settings-help-mail-folder-role-inbox = Null używa automatycznego wykrywania. Użyj obiektu z dokładną ścieżką dla wspólnych wartości domyślnych lub obiektu z identyfikatorem skrzynki należącej do jednego konta. Brakujące lub kolidujące cele wyłączają tę rolę; blokady administratora obowiązują.
admin-settings-key-mail-folder-role-drafts = Mapowanie folderu Wersje robocze
admin-settings-help-mail-folder-role-drafts = Null używa automatycznego wykrywania. Użyj obiektu z dokładną ścieżką dla wspólnych wartości domyślnych lub obiektu z identyfikatorem skrzynki należącej do jednego konta. Brakujące lub kolidujące cele wyłączają tę rolę; blokady administratora obowiązują.
admin-settings-key-mail-folder-role-sent = Mapowanie folderu Wysłane
admin-settings-help-mail-folder-role-sent = Null używa automatycznego wykrywania. Użyj obiektu z dokładną ścieżką dla wspólnych wartości domyślnych lub obiektu z identyfikatorem skrzynki należącej do jednego konta. Brakujące lub kolidujące cele wyłączają tę rolę; blokady administratora obowiązują.
admin-settings-key-mail-folder-role-trash = Mapowanie folderu Kosz
admin-settings-help-mail-folder-role-trash = Null używa automatycznego wykrywania. Użyj obiektu z dokładną ścieżką dla wspólnych wartości domyślnych lub obiektu z identyfikatorem skrzynki należącej do jednego konta. Brakujące lub kolidujące cele wyłączają tę rolę; blokady administratora obowiązują.
admin-settings-key-mail-folder-role-junk = Mapowanie folderu Spam
admin-settings-help-mail-folder-role-junk = Null używa automatycznego wykrywania. Użyj obiektu z dokładną ścieżką dla wspólnych wartości domyślnych lub obiektu z identyfikatorem skrzynki należącej do jednego konta. Brakujące lub kolidujące cele wyłączają tę rolę; blokady administratora obowiązują.
admin-settings-key-mail-folder-role-archive = Mapowanie folderu Archiwum
admin-settings-help-mail-folder-role-archive = Null używa automatycznego wykrywania. Użyj obiektu z dokładną ścieżką dla wspólnych wartości domyślnych lub obiektu z identyfikatorem skrzynki należącej do jednego konta. Brakujące lub kolidujące cele wyłączają tę rolę; blokady administratora obowiązują.
admin-settings-key-mail-request-delivery-receipt = Domyślnie żądaj powiadomień o dostarczeniu
admin-settings-help-mail-request-delivery-receipt = Wybiera powiadomienia o dostarczeniu dla nowych wiadomości, gdy konto obsługuje SMTP DSN. Blokada steruje tą zapisaną wartością domyślną; receipts.dsnAllowed osobno określa, czy żądania są dozwolone. Dostarczenie nie dowodzi przeczytania.
admin-settings-key-security-signin-geo-enforce-sessions = Egzekwuj politykę krajów w istniejących sesjach
admin-settings-help-security-signin-geo-enforce-sessions = Domyślnie wyłączone. Stosuje skonfigurowany zakres krajów logowania do uwierzytelnionych żądań, natywnego dostępu bearer i podpisanej zawartości czytnika. Zablokowani klienci nadal mogą się wylogować. Już otwarte strumienie są sprawdzane po ponownym połączeniu; nie kończy to istniejących połączeń.
admin-settings-key-security-protection-rate-mode = Zbiorcze budżety żądań
admin-settings-help-security-protection-rate-mode = Domyślnie wyłączone. Observe zapisuje wyczerpanie budżetu bez odmowy; enforce zwraca tymczasowe 429. Istniejące limity współbieżności protokołów pozostają niezależne.
admin-settings-key-security-protection-adaptive-mode = Adaptacyjna ochrona żądań
admin-settings-help-security-protection-adaptive-mode = Domyślnie wyłączone. Observe zapisuje możliwe do wyjaśnienia typowane anomalie i proponowane działania; enforce stosuje ograniczone tymczasowe dławienie lub blokady. To deterministyczna polityka liczników, a nie klasyfikator AI.
admin-settings-key-security-protection-client-requests = Żądania klienta na okno
admin-settings-help-security-protection-client-requests = Żądania na okno z jednego adresu, który nie jest zalogowany, oraz każde żądanie logowania. Żądanie z ważną sesją skrzynki lub administratora używa własnego, większego limitu. Zero wyłącza ten budżet.
admin-settings-key-security-protection-account-requests = Żądania konta na okno
admin-settings-help-security-protection-account-requests = Żądania na okno dla jednej zalogowanej skrzynki, we wszystkich jej sesjach, urządzeniach i adresach. Utrzymywane powyżej limitu anonimowego, aby zalogowani mieli więcej miejsca. Zero wyłącza ten budżet.
admin-settings-key-rate-limit-diagnostics-burst = Jednoczesne uruchomienia diagnostyki
admin-settings-help-rate-limit-diagnostics-burst = Uruchomienia diagnostyki i kontrole zaplecza, które sesja administratora może rozpocząć z rzędu. Rozpoczęcie już trwającej kontroli zwraca tę kontrolę zamiast liczyć ponownie.
admin-settings-key-rate-limit-diagnostics-per-minute = Uruchomienia diagnostyki na minutę
admin-settings-help-rate-limit-diagnostics-per-minute = Uruchomienia diagnostyki i kontrole zaplecza na minutę, które sesja administratora odzyskuje.
admin-settings-key-security-protection-admin-requests = Żądania administratora na okno

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-protection-admin-requests = Żądania na okno dla jednego zalogowanego administratora. Utrzymywane powyżej limitu anonimowego. Zero wyłącza ten budżet.
admin-settings-key-security-protection-rate-window-secs = Okno budżetu żądań (sekundy)
admin-settings-help-security-protection-rate-window-secs = Stały odstęp budżetu żądań. Odciążanie przy przeciążeniu i zwykłe odpowiedzi „nie znaleziono” nie są adaptacyjnym dowodem.
admin-settings-key-security-protection-adaptive-clients = Obserwuj klientów w ochronie adaptacyjnej
admin-settings-help-security-protection-adaptive-clients = Włącza ten wymiar podmiotu, gdy tryb adaptacyjny jest aktywny. Konta i administratorzy są liczeni dopiero po zweryfikowaniu tożsamości.
admin-settings-key-security-protection-adaptive-accounts = Obserwuj konta w ochronie adaptacyjnej
admin-settings-help-security-protection-adaptive-accounts = Włącza ten wymiar podmiotu, gdy tryb adaptacyjny jest aktywny. Konta i administratorzy są liczeni dopiero po zweryfikowaniu tożsamości.
admin-settings-key-security-protection-adaptive-admins = Obserwuj administratorów w ochronie adaptacyjnej
admin-settings-help-security-protection-adaptive-admins = Włącza ten wymiar podmiotu, gdy tryb adaptacyjny jest aktywny. Konta i administratorzy są liczeni dopiero po zweryfikowaniu tożsamości.
admin-settings-key-security-protection-short-window-secs = Krótkie okno anomalii (sekundy)
admin-settings-help-security-protection-short-window-secs = Przesuwne okno obserwacji. Długie okna obsługują od godzin do trzydziestu dni; zaokrąglanie koszyków może zachować co najwyżej dodatkową sześćdziesiątą czwartą okna.
admin-settings-key-security-protection-short-minimum-samples = Minimalna liczba próbek krótkiego współczynnika
admin-settings-help-security-protection-short-minimum-samples = Minimalna liczba zaobserwowanych żądań normalnych i anomalnych, zanim zadziała wyzwalacz współczynnika. Bezwzględny próg anomalii jest niezależny od tego minimum.
admin-settings-key-security-protection-short-absolute-anomalies = Krótki bezwzględny budżet anomalii
admin-settings-help-security-protection-short-absolute-anomalies = Wyzwala, gdy zgromadzi się tyle wybranych anomalii, nawet jeśli normalne żądania rozcieńczają współczynnik. Zero wyłącza ten wyzwalacz.
admin-settings-key-security-protection-short-ratio-percent = Krótki współczynnik anomalii (procent)
admin-settings-help-security-protection-short-ratio-percent = Wyzwala, gdy wybrane anomalie osiągną ten odsetek po minimalnej liczbie próbek. Wystarczy wyzwalacz bezwzględny lub współczynnika. Zero wyłącza ten wyzwalacz.
admin-settings-key-security-protection-long-window-secs = Długie okno anomalii (sekundy)
admin-settings-help-security-protection-long-window-secs = Przesuwne okno obserwacji. Długie okna obsługują od godzin do trzydziestu dni; zaokrąglanie koszyków może zachować co najwyżej dodatkową sześćdziesiątą czwartą okna.
admin-settings-key-security-protection-long-minimum-samples = Minimalna liczba próbek długiego współczynnika
admin-settings-help-security-protection-long-minimum-samples = Minimalna liczba zaobserwowanych żądań normalnych i anomalnych, zanim zadziała wyzwalacz współczynnika. Bezwzględny próg anomalii jest niezależny od tego minimum.
admin-settings-key-security-protection-long-absolute-anomalies = Długi bezwzględny budżet anomalii
admin-settings-help-security-protection-long-absolute-anomalies = Wyzwala, gdy zgromadzi się tyle wybranych anomalii, nawet jeśli normalne żądania rozcieńczają współczynnik. Zero wyłącza ten wyzwalacz.
admin-settings-key-security-protection-long-ratio-percent = Długi współczynnik anomalii (procent)
admin-settings-help-security-protection-long-ratio-percent = Wyzwala, gdy wybrane anomalie osiągną ten odsetek po minimalnej liczbie próbek. Wystarczy wyzwalacz bezwzględny lub współczynnika. Zero wyłącza ten wyzwalacz.
admin-settings-key-security-protection-normal-cap-per-bucket = Limit normalnej linii bazowej na koszyk
admin-settings-help-security-protection-normal-cap-per-bucket = Ogranicza normalną linię bazową uczoną w każdym koszyku okna przesuwnego. Niezależne bezwzględne budżety anomalii zapobiegają ukryciu nadużyć przez nieograniczone dopełnianie prawidłowymi żądaniami.
admin-settings-key-security-protection-signals = Sygnały dowodowe adaptacji
admin-settings-help-security-protection-signals = Wybierz autorytatywne obserwacje: nieprawidłowe, zbyt duże, z odrzuconymi poświadczeniami, z odmową uprawnień, przekroczeniem terminu treści lub (opcjonalnie) chybienia trasy administratora. Sondowanie tras administratora pomija odpowiedzi o nieznalezieniu encji i zasoby statyczne; ogólne kody statusu, odmowy z powodu przeciążenia i ciągi user-agent nie dowodzą nadużycia.
admin-settings-key-security-protection-throttle-secs = Początkowe tymczasowe ograniczenie (sekundy)
admin-settings-help-security-protection-throttle-secs = Pierwsze przekroczenie progu rozpoczyna stały termin ograniczenia. Żądania w tym terminie go nie wydłużają.
admin-settings-key-security-protection-block-secs = Eskalowana tymczasowa blokada (sekundy)
admin-settings-help-security-protection-block-secs = Późniejsze przekroczenie progu przed odzyskaniem może eskalować do tej ograniczonej blokady. Nie jest tworzona stała blokada konta.
admin-settings-key-security-protection-decay-secs = Odstęp spokojnego odzyskiwania (sekundy)
admin-settings-help-security-protection-decay-secs = Po tym odstępie bez anomalii i wygaśnięciu bieżącej akcji eskalacja wraca do etapu początkowego. Dowody w oknie przesuwnym wygasają zgodnie ze skonfigurowanymi oknami.
admin-settings-key-security-protection-exempt-clients = Wyłączone adresy i sieci klientów
admin-settings-help-security-protection-exempt-clients = Dokładne adresy IP lub sieci CIDR wyłączają tylko wymiar klienta. Zweryfikowane limity kont nadal obowiązują. Używaj świadomie dla współdzielonych sieci NAT; przekazywane adresy są zaufane tylko przez skonfigurowane proxy.
admin-settings-key-security-protection-exempt-accounts = Wyłączone zweryfikowane identyfikatory kont
admin-settings-help-security-protection-exempt-accounts = Dokładne zweryfikowane przez serwer identyfikatory kont skrzynek wyłączają tylko ich wymiar konta, nigdy wymiar klienta źródłowego. Adresy e-mail i parametry konta podane przez wywołującego nie są dowodem tożsamości.
admin-settings-key-security-protection-exempt-admins = Wyłączone identyfikatory administratorów
admin-settings-help-security-protection-exempt-admins = Dokładne uwierzytelnione identyfikatory administratorów wyłączają tylko ich wymiar administratora. Istniejące blokady po nieudanych logowaniach i ścisłe terminy sesji administratora pozostają bez zmian.
admin-settings-key-security-protection-storage-failure = Polityka awarii magazynu egzekwowania
admin-settings-help-security-protection-storage-failure = Closed odmawia chronionej pracy, gdy nie można sprawdzić włączonego budżetu egzekwowania. Open tymczasowo na nią zezwala, z ograniczonym ostrzeżeniem operacyjnym. Awarie w trybie tylko obserwacji nigdy nie nakładają akcji egzekwowania.
admin-settings-key-security-admin-probe-enabled = Ochrona przed sondowaniem administratora
admin-settings-help-security-admin-probe-enabled = Zlicza żądania szukające konsoli administratora na sieć klienta (adres IPv4, IPv6 /64), spowalnia źródło, które nie przestaje, a potem je blokuje. Literówki i nieaktualne zakładki mieszczą się w limicie łaski. Zmieniane w Bezpieczeństwo > Ochrona adaptacyjna.
admin-settings-key-security-admin-probe-grace-allowance = Limit łaski (chybienia na okno)
admin-settings-help-security-admin-probe-grace-allowance = Chybienia ignorowane na okno. Wynik stale maleje o tyle punktów na okno.
admin-settings-key-security-admin-probe-window-secs = Okno łaski (sekundy)
admin-settings-help-security-admin-probe-window-secs = Okno, do którego odnosi się limit łaski.
admin-settings-key-security-admin-probe-block-after = Chybienia ponad łaskę przed blokadą
admin-settings-help-security-admin-probe-block-after = Chybienia ponad limit łaski, od których zaczyna się blokada. Chybienia pośrednie są obsługiwane po opóźnieniu tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Opóźnienie tarpit (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Opóźnienie dodawane do każdego zliczonego chybienia ponad limit łaski. 0 wyłącza tarpit. Inne żądania nigdy nie są opóźniane.
admin-settings-key-security-admin-probe-block-secs = Pierwsza blokada (sekundy)
admin-settings-help-security-admin-probe-block-secs = Długość pierwszej blokady. Zablokowane źródło dostaje tę samą odpowiedź dla każdej ścieżki.
admin-settings-key-security-admin-probe-max-block-secs = Najdłuższa blokada (sekundy)
admin-settings-help-security-admin-probe-max-block-secs = Powtórna blokada podwaja poprzednią długość, do tego limitu.
admin-settings-key-security-admin-probe-repeat-window-secs = Okno powtórzenia (sekundy)
admin-settings-help-security-admin-probe-repeat-window-secs = Blokada rozpoczęta w tym czasie po zakończeniu poprzedniej liczy się jako powtórna.
admin-settings-key-security-admin-probe-block-status = Status odpowiedzi blokady
admin-settings-help-security-admin-probe-block-status = Status HTTP wysyłany zablokowanemu źródłu, z Retry-After: 429 lub 403.
admin-settings-key-security-admin-probe-signals = Zliczane chybienia
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (404 na głębokości ścieżki konsoli), admin-signin-failed i admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Ścieżki sondowania
admin-settings-help-security-admin-probe-probe-paths = Ścieżki, o które pytają skanery, np. /wp-admin lub /.env. Ścieżka zawierająca jedną z nich jako całe segmenty liczy się jako chybienie. Własna ścieżka konsoli nigdy.
admin-settings-key-security-admin-probe-allowlist = Sieci nigdy niezablokowane
admin-settings-help-security-admin-probe-allowlist = Adresy IP lub sieci CIDR (biuro, VPN), które nigdy nie są zliczane ani blokowane.
admin-settings-key-security-admin-probe-audit-ip = Adres źródłowy w dzienniku audytu
admin-settings-help-security-admin-probe-audit-ip = masked pokazuje IPv4 /24 i IPv6 /48; full pokazuje adres. Wiersz fail2ban zawsze ma pełny adres.
admin-settings-key-security-admin-probe-fail2ban-log = Wiersz dziennika fail2ban przy blokadzie
admin-settings-help-security-admin-probe-fail2ban-log = Zapisuje jeden wiersz mailwoman[admin-probe] z rhost=<adres>, gdy źródło zostaje zablokowane.
admin-settings-key-appearance-custom-css-enabled = Zezwalaj na osobisty CSS wyglądu
admin-settings-help-appearance-custom-css-enabled = Zezwala na ograniczone style wizualne w przestrzeni roboczej właściciela. Domyślnie wyłączone; wymuszone blokady wyglądu skrzynki wstrzymują te style.
admin-settings-key-appearance-custom-js-enabled = Zezwalaj na osobiste skrypty wyglądu
admin-settings-help-appearance-custom-js-enabled = Zezwala na izolowane skrypty, które mogą tylko żądać zmian wyglądu. Domyślnie wyłączone; skrypty nie mają dostępu do poczty, danych konta ani DOM aplikacji.
admin-settings-key-mail-modifier-selection = Zaznaczanie wiadomości klawiszami modyfikującymi
admin-settings-help-mail-modifier-selection = Zezwala na zaznaczanie zakresu klawiszami Ctrl, Command i Shift. Zablokuj tę wartość, aby ją wymusić; widoczne kontrolki zaznaczania pozostają dostępne.
admin-settings-key-mail-drag-to-folder = Przeciąganie wiadomości do folderów
admin-settings-help-mail-drag-to-folder = Zezwala na przeciąganie wiadomości powiązanych z kontem do rzeczywistych folderów. Nie przyznaje to uprawnienia zapisu do skrzynki; serwer nadal autoryzuje każde przeniesienie.
admin-settings-key-admin-allow-last-superadmin-loss = Zezwalaj na utratę ostatniego superadministratora
admin-settings-help-admin-allow-last-superadmin-loss = Domyślnie wyłączone. Włączenie pozwala, by późniejsze zmiany usunęły ostatni dostęp superadministratora. Pełny superadministrator musi potwierdzić skutek świeżym uwierzytelnieniem.
admin-settings-last-superadmin-title = Zezwolić na utratę całego dostępu superadministratora?
admin-settings-last-superadmin-body = Późniejsze zmiany kont mogą sprawić, że nikt nie będzie mógł administrować tym wdrożeniem. Odzyskanie wymagałoby lokalnej administracji wdrożeniem. Wpisz ALLOW LAST SUPERADMIN LOSS, aby włączyć ten wyjątek.
admin-settings-last-superadmin-confirm = Zezwól na utratę dostępu
admin-settings-key-mail-folder-counts = Liczniki folderów
admin-settings-help-mail-folder-counts = Wybierz ukryte, nieprzeczytane, nieprzeczytane tylko gdy są nieprzeczytane wiadomości, łączne lub oba liczniki przy folderach. Ustaw wartości domyślne globalne, domeny lub użytkownika i zablokuj wybór, gdy trzeba. Dotyczy każdej połączonej skrzynki konta.
admin-settings-key-mail-folder-count-scope = Foldery z licznikami
admin-settings-help-mail-folder-count-scope = Wszystkie foldery lub tylko główne (Odebrane, Wersje robocze, Wysłane, Archiwum, Spam, Kosz, Skrzynka nadawcza i inne foldery specjalnego przeznaczenia). Przy tylko głównych foldery własne i ich podfoldery nie pokazują licznika. Dotyczy każdej połączonej skrzynki konta.
admin-settings-key-mail-compact-folder-badges = Kompaktowe znaczki pozostałych folderów
admin-settings-help-mail-compact-folder-badges = Pokazuje mniejsze znaczki poza folderem Odebrane. Użytkownicy mogą wybrać własny rozmiar, chyba że to ustawienie jest zablokowane.
admin-settings-key-mail-focused-inbox = Skrzynka priorytetowa
admin-settings-help-mail-focused-inbox = Dzieli Odebrane na karty Priorytetowe i Pozostałe; newslettery i automatyczni nadawcy trafiają do Pozostałych. Domyślnie wyłączone. Użytkownicy mogą to włączyć lub wyłączyć w Ustawienia > Foldery i pasek boczny, chyba że to ustawienie jest zablokowane.
admin-settings-key-mail-account-unread-badge = Liczba nieprzeczytanych przy nazwie skrzynki
admin-settings-help-mail-account-unread-badge = Pokazuje liczbę nieprzeczytanych w Odebranych obok zalogowanego adresu lub pseudonimu na pasku bocznym. Domyślnie wyłączone. Liczniki innych skrzynek w menu skrzynek są zawsze widoczne. Dotyczy każdej połączonej skrzynki konta.
admin-settings-key-privacy-remote-image-downloads = Ręczne pobieranie obrazów zdalnych
admin-settings-key-privacy-remote-images-toast = Powiadomienie o ładowaniu obrazów
admin-settings-help-privacy-remote-images-toast = Pokazuje jedno powiadomienie w czytniku, gdy obrazy zdalne wiadomości ładują się przez proxy prywatności, i informuje, kiedy się załadowały lub ile się nie udało. Domyślnie włączone. Zmienia tylko to, co pokazuje czytnik; o tym, które obrazy się ładują, nadal decyduje polityka zawartości zdalnej.
admin-settings-help-privacy-remote-image-downloads = Zezwala na jawne pobranie obrazu wskazanego przez należącą do użytkownika wiadomość przez chronione proxy prywatności. Domyślnie wyłączone; nigdy nie otwiera zewnętrznego adresu URL bezpośrednio.
admin-settings-key-mail-resume-tabs = Wznawiaj karty obszaru roboczego
admin-settings-help-mail-resume-tabs = Przywraca po odświeżeniu odniesienia do kart należących do konta. Wyłączenie czyści lokalny zapis obszaru roboczego bez usuwania wersji roboczych.
admin-settings-key-metrics-default-range-seconds = Domyślny zakres historii żądań
admin-settings-help-metrics-default-range-seconds = Liczba sekund pokazywana, gdy żądanie historii nie określa zakresu. Ograniczona przez przechowywanie i zakres maksymalny.
admin-settings-key-metrics-max-range-seconds = Maksymalny zakres historii żądań
admin-settings-help-metrics-max-range-seconds = Maksymalna liczba sekund na zapytanie o historię. Przechowywanie pozostaje dodatkowym limitem.
admin-settings-key-contacts-favorites-first = Ulubione na początku podpowiedzi
admin-settings-help-contacts-favorites-first = Umieszcza ulubione kontakty przed innymi dopasowaniami w podpowiedziach odbiorców. Zmienia się tylko kolejność; żadna podpowiedź nie jest ukrywana. Nadpisania użytkownika i domeny można zablokować.
admin-settings-key-contacts-sender-icons = Dołączone ikony nadawców

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-contacts-sender-icons = Pokazuje lokalnie dołączone ikony dla rozpoznanych domen nadawców, a dla pozostałych wygenerowane inicjały. Zdjęcia kontaktów mają pierwszeństwo. Ikony nie weryfikują tożsamości nadawcy. Nie jest kontaktowana żadna zewnętrzna usługa logo; nadpisania użytkownika i domeny można zablokować.
admin-settings-key-mail-brand-avatars = Logo marek przy zweryfikowanych nadawcach
admin-settings-help-mail-brand-avatars = Pokazuje lokalnie dołączone logo marki obok nadawcy w czytniku, tylko gdy wiadomość przeszła DKIM i zgodną kontrolę DMARC dla własnej domeny tej marki. Wiadomości, które nie przeszły tych kontroli lub ich nie mają, zachowują zwykły awatar. Żadne logo nie jest pobierane od nadawcy ani z usługi logo.
admin-settings-key-mail-draft-save-mode = Tryb zapisu wersji roboczych konta
admin-settings-help-mail-draft-save-mode = Zapisuje po przerwie w pisaniu, okresowo lub tylko na żądanie. Zapis wymaga obsługiwanego folderu Wersje robocze na serwerze nadrzędnym; niepewne zapisy nigdy nie są automatycznie ponawiane.
admin-settings-key-mail-draft-debounce-ms = Przerwa w pisaniu przed zapisem wersji roboczej (milisekundy)
admin-settings-help-mail-draft-debounce-ms = Czeka od 500 do 10000 milisekund po edycji przed zapisem w trybie czasu rzeczywistego.
admin-settings-key-mail-draft-interval-secs = Odstęp zapisu wersji roboczej (sekundy)
admin-settings-help-mail-draft-interval-secs = Zapisuje edytowane wersje robocze co 5 do 3600 sekund w trybie okresowym. Niezmienione wersje robocze nie są zapisywane ponownie.
admin-settings-key-login-remember-signins = Zapamiętuj adresy udanych logowań
admin-settings-help-login-remember-signins = Przechowuje na tym urządzeniu do dziesięciu pomyślnie użytych nazw użytkowników lub adresów e-mail na liście rozwijanej logowania. Nigdy nie zapisuje haseł ani sesji. Wyłączenie usuwa zapisane adresy tego serwera, gdy urządzenie następnym razem załaduje stronę logowania lub ukończy logowanie; nie steruje to autouzupełnianiem przeglądarki.
admin-settings-key-login-show-remembered-hint = Pokazuj objaśnienie zapisanych adresów
admin-settings-help-login-show-remembered-hint = Pokazuje tekst objaśniający na liście rozwijanej poprzednich logowań. Domyślnie ukryte. Ma zastosowanie tylko wtedy, gdy zapamiętywanie udanych logowań jest włączone.
admin-settings-key-mail-show-from = Pokazuj pole Od w oknie redagowania
admin-settings-help-mail-show-from = Domyślnie pokazuje pole Od w nowych oknach redagowania. Użytkownicy mogą je odkryć dla jednej wiadomości; wysyłać mogą tylko autoryzowane adresy.
admin-settings-key-accounts-linking-enabled = Zezwalaj na połączone konta skrzynek
admin-settings-help-accounts-linking-enabled = Domyślnie wyłączone. Globalne wyłączenie blokuje cały dostęp połączony; oba konta muszą też spełniać politykę swojej domeny i konta. Łączenie wymaga faktycznego uwierzytelnienia konta podrzędnego i świeżego potwierdzenia konta głównego.
admin-settings-key-accounts-use-linked = Używaj połączonych skrzynek
admin-settings-help-accounts-use-linked = Rezygnacja użytkownika z dostępu połączonego. Nie może przesłonić wyłączenia administracyjnego. Dane skrzynki są zachowywane po wyłączeniu dostępu.
admin-settings-key-accounts-max-linked = Maksymalna liczba połączonych skrzynek
admin-settings-help-accounts-max-linked = Maksymalna liczba zweryfikowanych skrzynek podrzędnych na tożsamość główną, od 1 do 16. Domyślnie 5.
admin-settings-key-accounts-store-linked-credentials = Zachowuj dostęp do połączonych skrzynek
admin-settings-help-accounts-store-linked-credentials = Przechowuje hasło każdej połączonej skrzynki zaszyfrowane na serwerze, aby dostęp odnawiał się sam, między sesjami i restartami. Nigdy nie jest przechowywane dla kont z zerowym dostępem ani skrzynek z weryfikacją dwuetapową. Wyłączenie usuwa zapisane hasła; połączenia odnawia się wtedy przez ponowne zalogowanie. Domyślnie włączone.
admin-settings-key-privacy-strip-mailer-headers = Usuwaj nagłówki programu pocztowego z wiadomości wychodzących
admin-settings-help-privacy-strip-mailer-headers = Usuwa nagłówki najwyższego poziomu X-Mailer, User-Agent i X-MimeOLE z poczty natywnego SMTP i EWS. Dołączone wiadomości i ładunki raportów pozostają nienaruszone. Wysyłki przez nadrzędne proxy JMAP nie są objęte.
admin-settings-key-images-trusted-hosts = Zaufane hosty obrazów
admin-settings-help-images-trusted-hosts = Do 100 hostów obrazów i ich poddomen, których obrazy zdalne ładują się bez zgody dla wiadomości, gdy zawartość zdalna jest ustawiona na pytanie. Nigdy nie przesłania trybu blokowania; zablokowany host pozostaje zablokowany.
admin-settings-key-images-blocked-hosts = Zablokowane hosty obrazów
admin-settings-help-images-blocked-hosts = Do 100 hostów obrazów i ich poddomen, których obrazy zdalne nigdy się nie ładują w żadnym trybie. Ma pierwszeństwo przed zaufanymi hostami i każdą zgodą.
admin-settings-key-links-trusted-domains = Zaufane domeny linków
admin-settings-help-links-trusted-domains = Do 100 dokładnych domen, których linki otwierają się z Przeglądu linków bez kroku miejsca docelowego. Bez symboli wieloznacznych, schematów, portów ani adresów IP. Linki zwykłego HTTP są zawsze sprawdzane. Zablokuj to ustawienie, aby użytkownicy nie zmieniali listy.
admin-settings-key-links-trusted-subdomains = Ufaj poddomenom zaufanych domen linków
admin-settings-help-links-trusted-subdomains = Wyłączone: zaufana jest tylko dokładna domena. Włączone: zaufane są też nazwy kończące się na .<domena>. Dopasowanie dotyczy całych etykiet, więc example.com nigdy nie ufa evilexample.com.
admin-settings-key-cap-compose-rich-text = Redagowanie tekstu sformatowanego
admin-settings-help-cap-compose-rich-text = Czy użytkownicy mogą pisać wiadomości sformatowane (HTML). Wyłączone wysyła tylko zwykły tekst.
admin-settings-key-cap-compose-attachments = Załączniki w nowych wiadomościach
admin-settings-help-cap-compose-attachments = Czy użytkownicy mogą dołączać pliki do pisanych wiadomości. Obrazy wstawione w treści nie są liczone.
admin-settings-key-cap-compose-share-links = Linki udostępniania dużych plików
admin-settings-help-cap-compose-share-links = Czy użytkownicy mogą wysłać duży plik jako link udostępnienia Nextcloud zamiast załącznika.
admin-settings-key-cap-compose-scheduled-send = Zaplanowane wysyłanie
admin-settings-help-cap-compose-scheduled-send = Czy użytkownicy mogą zaplanować wysłanie wiadomości na później.
admin-settings-key-cap-compose-read-receipts = Żądanie potwierdzeń odczytu
admin-settings-help-cap-compose-read-receipts = Czy użytkownicy mogą prosić odbiorców o potwierdzenie odczytu.
admin-settings-key-cap-signatures = Podpisy
admin-settings-help-cap-signatures = Czy użytkownicy mogą tworzyć i edytować własne podpisy.
admin-settings-key-cap-identities = Tożsamości nadawcy
admin-settings-help-cap-identities = Czy użytkownicy mogą dodawać i edytować tożsamości, jako które wysyłają.
admin-settings-key-cap-keys = Klucze OpenPGP i S/MIME
admin-settings-help-cap-keys = Czy użytkownicy mają moduł Klucze i mogą podpisywać oraz szyfrować wiadomości.
admin-settings-key-cap-links-trusted = Edycja zaufanych domen linków
admin-settings-help-cap-links-trusted = Czy użytkownicy mogą zmieniać własną listę zaufanych domen linków. Lista administratora nadal obowiązuje.
admin-settings-key-cap-zoom = Powiększenie w czytniku
admin-settings-help-cap-zoom = Czy użytkownicy mogą powiększać czytnik wiadomości.
admin-settings-key-cap-mail-focused-inbox = Skrzynka priorytetowa
admin-settings-help-cap-mail-focused-inbox = Czy użytkownicy mogą włączyć karty Priorytetowe i Pozostałe w skrzynce odbiorczej.
admin-settings-key-cap-views-events = Widok wydarzeń
admin-settings-help-cap-views-events = Czy użytkownicy mają widok Wydarzenia poczty kalendarzowej.
admin-settings-key-cap-views-attachments = Widok załączników
admin-settings-help-cap-views-attachments = Czy użytkownicy mają widok Załączniki. Wiadomości pozostają dostępne przez wyszukiwanie.
admin-settings-key-cap-search-operators = Operatory wyszukiwania
admin-settings-help-cap-search-operators = Czy użytkownicy mogą wyszukiwać z operatorami, takimi jak from:, subject: lub has:attachment.
admin-settings-key-cap-saved-searches = Zapisane wyszukiwania
admin-settings-help-cap-saved-searches = Czy użytkownicy mogą zapisywać wyszukiwania.
admin-settings-key-cap-vacation = Autoodpowiedź urlopowa
admin-settings-help-cap-vacation = Czy użytkownicy mogą ustawić automatyczną odpowiedź urlopową.
admin-settings-key-cap-labels = Etykiety
admin-settings-help-cap-labels = Czy użytkownicy mogą nadawać etykiety wiadomościom.
admin-settings-key-cap-snooze = Drzemka
admin-settings-help-cap-snooze = Czy użytkownicy mogą odkładać wiadomości. Zakończenie drzemki pozostaje możliwe.
admin-settings-key-cap-follow-up = Przypomnienia o kontynuacji
admin-settings-help-cap-follow-up = Czy użytkownicy mogą ustawiać przypomnienia o kontynuacji wiadomości. Usunięcie przypomnienia pozostaje możliwe.
admin-settings-key-cap-masked = Adresy maskowane
admin-settings-help-cap-masked = Czy użytkownicy mogą tworzyć adresy maskowane i nimi zarządzać.
admin-settings-key-cap-directory = Wyszukiwanie w katalogu
admin-settings-help-cap-directory = Czy użytkownicy mogą przeszukiwać katalog organizacji.
admin-settings-key-cap-mcp = Dostęp MCP
admin-settings-help-cap-mcp = Czy klucze i tokeny użytkownika mogą korzystać z punktu końcowego MCP.
admin-settings-key-cap-appearance-custom = Dostosowywanie wyglądu
admin-settings-help-cap-appearance-custom = Czy użytkownicy mogą zmieniać motyw, akcent, kolory własne i znaczków, gęstość oraz czcionkę. Wyłączone zachowuje wygląd ustawiony przez administratora.
admin-settings-key-cap-appearance-custom-code = Własny CSS i skrypty
admin-settings-help-cap-appearance-custom-code = Czy użytkownicy mogą zapisywać własny CSS i skrypty wyglądu. Przełączniki wdrożenia w sekcji Wygląd nadal obowiązują nad tym ustawieniem.
admin-settings-key-cap-avatars = Awatary skrzynek
admin-settings-help-cap-avatars = Czy użytkownicy mogą zmieniać awatary skrzynek.
admin-settings-key-cap-shortcuts = Własne skróty klawiszowe
admin-settings-help-cap-shortcuts = Czy użytkownicy mogą zmieniać skróty klawiszowe. Skróty domyślne nadal działają.
admin-settings-key-cap-analytics = Analityka produktu
admin-settings-help-cap-analytics = Czy użytkownicy są pytani o zgodę na analitykę i mogą wysyłać zdarzenia użycia.
admin-settings-key-cap-apps-native = Aplikacje na komputer i telefon
admin-settings-help-cap-apps-native = Czy użytkownicy mogą korzystać z aplikacji na komputer i telefon.
admin-settings-key-access-protocols = Protokoły dostępu do poczty
admin-settings-help-access-protocols = Którymi protokołami serwera poczty ci użytkownicy mogą się logować: imap, pop3, ews. Logowania przez serwer JMAP podlegają temu serwerowi.
admin-settings-key-limit-linked-mailboxes = Połączone skrzynki na użytkownika
admin-settings-help-limit-linked-mailboxes = Ile skrzynek użytkownik może połączyć, poniżej maksimum serwera. 0 oznacza, że obowiązuje tylko maksimum serwera.
admin-settings-key-login-show-account-help-notice = Pokazuj informację o pomocy z kontem przy logowaniu
admin-settings-help-login-show-account-help-notice = Pokazuje krótkie objaśnienie po nieudanym logowaniu. Domyślnie ukryte; informacja nie ujawnia, czy konto istnieje lub jest wyłączone.
admin-settings-key-appearance-leetspeak = Styl tekstu leetspeak
admin-settings-help-appearance-leetspeak = Wyświetla etykiety interfejsu i tekst wiadomości tylko do odczytu w leetspeak. Oryginalna wiadomość i edytowalny tekst nie zmieniają się. Użytkownicy mogą wybrać zwykły tekst, chyba że to ustawienie jest zablokowane.
admin-settings-key-mail-cache-enabled = Włącz pamięć podręczną sesji
admin-settings-help-mail-cache-enabled = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-mailboxes = Buforuj listy wiadomości folderów
admin-settings-help-mail-cache-mailboxes = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-messages = Buforuj otwarte wiadomości
admin-settings-help-mail-cache-messages = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-fresh-seconds = Odstęp świeżości (sekundy)
admin-settings-help-mail-cache-fresh-seconds = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-max-age-seconds = Maksymalny wiek danych w pamięci (sekundy)
admin-settings-help-mail-cache-max-age-seconds = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-max-entries = Maksymalna liczba wpisów w pamięci
admin-settings-help-mail-cache-max-entries = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-max-bytes = Limit pamięci (bajty)
admin-settings-help-mail-cache-max-bytes = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-prefetch-enabled = Włącz spekulacyjne wstępne ładowanie
admin-settings-help-mail-cache-prefetch-enabled = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-hover-enabled = Ładuj wstępnie po wskazaniu folderu
admin-settings-help-mail-cache-hover-enabled = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-focus-enabled = Ładuj wstępnie, gdy folder otrzyma fokus klawiatury
admin-settings-help-mail-cache-focus-enabled = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-dwell-ms = Opóźnienie wskazania i fokusu (milisekundy)
admin-settings-help-mail-cache-dwell-ms = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-idle-enabled = Ładuj foldery wstępnie podczas bezczynności
admin-settings-help-mail-cache-idle-enabled = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-idle-delay-ms = Opóźnienie bezczynności (milisekundy)
admin-settings-help-mail-cache-idle-delay-ms = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-max-concurrency = Maksymalna liczba jednoczesnych wstępnych ładowań
admin-settings-help-mail-cache-max-concurrency = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-max-queued = Maksymalna liczba wstępnych ładowań w kolejce
admin-settings-help-mail-cache-max-queued = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-preload-messages = Zezwalaj na spekulacyjne treści wiadomości
admin-settings-help-mail-cache-preload-messages = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.
admin-settings-key-mail-cache-excluded-folders = Foldery wyłączone ze wstępnego ładowania
admin-settings-help-mail-cache-excluded-folders = Przechowuje ostatnio odwiedzane foldery i wiadomości w pamięci tej karty. Zapisana zawartość odświeża się w tle. Zamknięcie karty lub wylogowanie ją czyści. Wstępne ładowanie nigdy nie oznacza poczty jako przeczytanej ani nie ładuje obrazów zdalnych.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-mail-cache-refresh-on-open = Odświeżaj zapisaną treść przy otwieraniu
admin-settings-help-mail-cache-refresh-on-open = Odświeżaj zapisaną treść przy otwieraniu
admin-settings-key-mail-templates-enabled = Szablony odpowiedzi
admin-settings-help-mail-templates-enabled = Włącz osobiste i udostępnione szablony odpowiedzi w postaci zwykłego tekstu w edytorze wiadomości. Użytkownicy mogą z nich zrezygnować, chyba że to ustawienie jest zablokowane.
admin-settings-key-mail-templates-allow-personal = Zezwalaj na szablony osobiste
admin-settings-help-mail-templates-allow-personal = Zezwalaj użytkownikom na tworzenie własnych szablonów odpowiedzi. Szablonami udostępnionymi zarządzają osobno administratorzy.
admin-settings-key-mail-templates-shared = Udostępnione szablony odpowiedzi
admin-settings-help-mail-templates-shared = Zarządzaj wielokrotnie używanymi odpowiedziami w postaci zwykłego tekstu dla tego zakresu. Szablony nigdy nie ustawiają odbiorców ani nie wysyłają wiadomości.
admin-settings-key-mail-templates-personal = Osobiste szablony odpowiedzi
admin-settings-help-mail-templates-personal = Osobiste, wielokrotnie używane odpowiedzi w postaci zwykłego tekstu. Tworzenie i edycja wymagają włączenia szablonów odpowiedzi i szablonów osobistych.
admin-settings-key-mail-compose-window = Okno edytora
admin-settings-help-mail-compose-window = Otwieraj nowe wersje robocze w karcie obszaru roboczego lub w osobnym oknie. Zablokowane wyskakujące okna otwierają się w karcie.
admin-settings-key-mail-reply-copy-to-original = Zachowuj odpowiedzi w folderze oryginału
admin-settings-help-mail-reply-copy-to-original = Zapisuj kopię wysłanej odpowiedzi w folderze oryginalnej wiadomości, jeśli jest to obsługiwane.
admin-settings-key-mail-compose-check-invalid-recipients = Sprawdzaj nieprawidłowych odbiorców przed wysłaniem
admin-settings-help-mail-compose-check-invalid-recipients = Co się dzieje po wysłaniu wiadomości na nieprawidłowy adres: zezwól, ostrzeż, ostrzeż i zapamiętaj wybór albo odmów.
admin-settings-key-mail-compose-check-missing-attachment = Sprawdzaj brakujący załącznik przed wysłaniem
admin-settings-help-mail-compose-check-missing-attachment = Co się dzieje, gdy tekst wspomina o załączniku, a żadnego nie dodano: zezwól, ostrzeż, ostrzeż i zapamiętaj wybór albo odmów.
admin-settings-key-mail-compose-check-empty-subject = Sprawdzaj pusty temat przed wysłaniem
admin-settings-help-mail-compose-check-empty-subject = Co się dzieje, gdy wiadomość nie ma tematu: zezwól, ostrzeż, ostrzeż i zapamiętaj wybór albo odmów.
admin-settings-key-mail-compose-auto-chip-recipients = Automatycznie kończ wpisywanie odbiorców
admin-settings-help-mail-compose-auto-chip-recipients = Zamieniaj wpisany adres w znacznik odbiorcy po spacji, przecinku lub średniku.
admin-settings-key-mail-compose-header-collapse = Nagłówek edytora
admin-settings-help-mail-compose-header-collapse = Zachowanie wierszy Do, DW, UDW i Temat w edytorze: rozwinięte, zwinięte do jednowierszowego podsumowania albo automatyczne (zwinięte podczas pisania treści, otwarte po wybraniu pola). Użytkownicy mogą sami zwijać i rozwijać nagłówek, chyba że to ustawienie jest zablokowane.
admin-settings-key-mail-compose-recipient-separator = Separator odbiorców
admin-settings-help-mail-compose-recipient-separator = Separator wstawiany po zakończonym odbiorcy: przecinek lub średnik.
admin-settings-key-mail-reply-from-mode = Tożsamość używana do odpowiedzi i przekazywania
admin-settings-help-mail-reply-from-mode = Z której tożsamości wysyłana jest odpowiedź lub przekazana wiadomość: tej, na którą zaadresowano oryginał, domyślnej, wybieranej z listy lub aktualnie wybranej.
admin-settings-key-mail-forward-mode = Domyślny format przekazywania
admin-settings-help-mail-forward-mode = Przekazuj tekst w treści wiadomości albo dołączaj oryginał jako plik .eml.
admin-settings-key-mail-show-go-to-top = Pokazuj przycisk Przejdź na górę
admin-settings-help-mail-show-go-to-top = Pokazuj na liście wiadomości przycisk Przejdź na górę, który przewija do pierwszej wiadomości. Domyślnie wyłączone.
admin-settings-key-mail-show-create-folder-button = Pokazuj przycisk Utwórz folder
admin-settings-help-mail-show-create-folder-button = Pokazuj w panelu bocznym obok przycisku Nowa wiadomość przycisk Utwórz folder zawierający tylko ikonę. Domyślnie wyłączone.
admin-settings-key-security-protection-user-agent-mode = Tryb obserwacji user-agent
admin-settings-help-security-protection-user-agent-mode = Domyślnie egzekwowanie: wybrane rozpoznane nagłówki botów są blokowane od razu, gdy włączone jest blokowanie botów przy pierwszym żądaniu. Obserwacja zapisuje decyzje bez odmowy; wyłączenie wyłącza ten wymiar. Deklaracje w nagłówkach nie są zweryfikowanymi tożsamościami.
admin-settings-key-security-protection-user-agent-block-bots = Blokuj rozpoznane boty przy pierwszym żądaniu
admin-settings-help-security-protection-user-agent-block-bots = Domyślnie włączone. Wybrane nagłówki botów wyszukiwarek, AI, automatyzacji i niestandardowych powodują w trybie egzekwowania tymczasową blokadę HTTP 429 już przy pierwszym żądaniu. Tryb obserwacji tylko to zapisuje; wartość false przywraca poprzednie limity żądań. Brakujące lub nietypowe nagłówki, zakres i wyjątki działają jak dotąd.
admin-settings-key-security-protection-user-agent-categories = Kategorie user-agent
admin-settings-help-security-protection-user-agent-categories = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-weak-requests = Próg żądań ze słabym sygnałem
admin-settings-help-security-protection-user-agent-weak-requests = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-crawler-requests = Próg żądań robotów indeksujących
admin-settings-help-security-protection-user-agent-crawler-requests = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-window-secs = Okno obserwacji user-agent (sekundy)
admin-settings-help-security-protection-user-agent-window-secs = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-max-action = Maksymalna reakcja na user-agent
admin-settings-help-security-protection-user-agent-max-action = Maksymalna eskalacja w przypadku słabych lub powtarzających się dowodów, domyślnie ograniczanie. Ten limit nie dotyczy blokad wybranych rozpoznanych nagłówków botów przy pierwszym żądaniu.
admin-settings-key-security-protection-user-agent-scope = Zakres zasady user-agent
admin-settings-help-security-protection-user-agent-scope = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-adaptive = Adaptacyjne obserwacje user-agent
admin-settings-help-security-protection-user-agent-adaptive = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-patterns = Wzorce user-agent
admin-settings-help-security-protection-user-agent-patterns = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-security-protection-user-agent-exceptions = Wyjątki user-agent
admin-settings-help-security-protection-user-agent-exceptions = Konfiguruje tę część opcjonalnej obserwacji user-agent. Tekst user-agent podany przez klienta jest słabym sygnałem, a nie dowodem tożsamości ani nadużycia.
admin-settings-key-mail-reader-unsafe-html-mode = Potencjalnie niebezpieczny HTML wiadomości
admin-settings-help-mail-reader-unsafe-html-mode = Domyślnie Pytaj: dla każdej wiadomości dostępny jest przycisk oczyszczonego HTML. Ścisły pokazuje tylko bezpieczny tekst; oczyszczony automatycznie pokazuje oczyszczony HTML. Aktywna treść i automatyczne zasoby zdalne pozostają zablokowane. Zablokuj, aby wymusić tę zasadę.
admin-settings-key-reader-attachments-density = Gęstość załączników
admin-settings-help-reader-attachments-density = Domyślnie zwarte przyciski plików lub wygodniejsze podglądy. Określa wygląd po rozwinięciu załączników wiadomości.
admin-settings-key-cap-compose-open-tracking = Zezwalaj na śledzenie otwarć wiadomości
admin-settings-help-cap-compose-open-tracking = Czy to konto może rejestrować otwarcia wiadomości. Wymaga skonfigurowanego publicznego adresu URL; konta z wyłączoną funkcją nie mogą tworzyć ani wiązać tokenów śledzenia.
admin-settings-key-mail-open-tracking = Domyślnie śledź otwarcia
admin-settings-help-mail-open-tracking = Domyślny wybór śledzenia dla kwalifikujących się sformatowanych wiadomości. Użytkownicy mogą go zmienić, chyba że jest zablokowany; wyłączenie możliwości śledzenia zawsze ma pierwszeństwo.
admin-settings-key-cap-reading-original-html = Zezwalaj na renderowanie oryginalnego HTML
admin-settings-help-cap-reading-original-html = Czy użytkownicy mogą włączyć widok oryginalnego HTML dla poszczególnych wiadomości. Domyślny oczyszczający renderer i zabezpieczenie izolowaną ramką pozostają aktywne.
admin-settings-key-mail-reader-allow-original-html = Oferuj widok oryginalnego HTML

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-reader-allow-original-html = Pokazuj dla każdej wiadomości opcję użycia oryginalnego HTML, gdy pozwala na to możliwość czytania. Domyślnie wyłączone; nie zmienia to domyślnego renderera wiadomości.
admin-settings-key-privacy-tracker-override-allowed = Oferuj zgodę na trackery dla wiadomości
admin-settings-help-privacy-tracker-override-allowed = Pokazuj akcję Zezwól na trackery na pasku prywatności wiadomości. Domyślnie dostępna, ale nigdy nie jest aktywna bez wyraźnej zgody. Zablokowana zasada blokowania trackerów nadal zabrania takiego obejścia.
admin-settings-key-privacy-max-concurrent-global = Globalna liczba równoległych pobrań obrazów
admin-settings-help-privacy-max-concurrent-global = Maksymalna liczba aktywnych sieciowych żądań obrazów w całym procesie serwera. Zakres 1–256; domyślnie 128. Nadal obowiązują limity wiadomości i konta.
admin-settings-key-privacy-max-concurrent-pipeline = Współbieżne potoki przetwarzania obrazów
admin-settings-help-privacy-max-concurrent-pipeline = Maksymalna liczba obrazów przyjmowanych jednocześnie do pobrania, dekodowania i zapisu w pamięci podręcznej. Zakres 1–256; domyślnie 128. Limity sieci i dekoderów są egzekwowane osobno.
admin-settings-key-privacy-max-concurrent-decodes = Równoległe dekodery obrazów
admin-settings-help-privacy-max-concurrent-decodes = Maksymalna liczba jednocześnie działających dekodowań obrazów. Zakres 1–16; domyślnie 10. Wyższe wartości zużywają więcej procesora i pamięci.
admin-settings-key-privacy-max-queued-requests = Oczekujące żądania obrazów
admin-settings-help-privacy-max-queued-requests = Maksymalna liczba żądań oczekujących na dostępność sieci lub mocy przetwarzania obrazów. Zakres 16–16384; domyślnie 2048. Przepełnienie lub wygaśnięcie w kolejce można ponowić.
admin-settings-key-privacy-rate-burst = Limit zrywu żądań obrazów
admin-settings-help-privacy-rate-burst = Początkowa i maksymalna liczba tokenów pobierania na konto, uzupełniana zgodnie ze skonfigurowaną szybkością żądań obrazów. Zakres 1–12000; domyślnie 1024. Trafienia w pamięci podręcznej nie zużywają tokenów.
admin-settings-key-privacy-image-rate-mode = Dobór szybkości żądań obrazów
admin-settings-help-privacy-image-rate-mode = Automatycznie skaluje zryw i uzupełnianie dla konta na podstawie mocy replik, obciążenia kolejki i liczby obrazów w wiadomości, w granicach poniższych dolnych i górnych limitów. Ręcznie używa dokładnie ustawień limitu zrywu i szybkości żądań. Domyślnie automatycznie. Żądania ponad limit czekają na token do upływu terminu kolejki pobierania; uwierzytelnienie i zgody są sprawdzane najpierw.
admin-settings-key-privacy-image-rate-burst-floor = Dolny limit zrywu (automatycznie)
admin-settings-help-privacy-image-rate-burst-floor = Najniższy limit zrywu na konto w trybie automatycznym. Zakres 1–12000; domyślnie 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Górny limit zrywu (automatycznie)
admin-settings-help-privacy-image-rate-burst-ceiling = Najwyższy limit zrywu na konto w trybie automatycznym. Zakres 1–12000; domyślnie 2048.
admin-settings-key-privacy-image-rate-refill-floor = Dolny limit uzupełniania (automatycznie)
admin-settings-help-privacy-image-rate-refill-floor = Najniższe uzupełnianie na konto, w tokenach pobierania na minutę, w trybie automatycznym. Zakres 10–60000; domyślnie 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Górny limit uzupełniania (automatycznie)
admin-settings-help-privacy-image-rate-refill-ceiling = Najwyższe uzupełnianie na konto, w tokenach pobierania na minutę, w trybie automatycznym. Zakres 10–60000; domyślnie 6000.
admin-settings-key-privacy-image-cache-mode = Dobór rozmiaru pamięci podręcznej obrazów
admin-settings-help-privacy-image-cache-mode = Automatycznie dobiera rozmiar pamięci podręcznej na podstawie pamięci pozostałej po rezerwacjach dekoderów i potoków oraz wydłuża czas życia, gdy współczynnik trafień jest wysoki, a pamięć wolna. Ręcznie używa dokładnie ustawień rozmiaru, liczby wpisów i czasu życia. Domyślnie automatycznie.
admin-settings-key-privacy-image-cache-bytes-floor = Dolny limit rozmiaru pamięci podręcznej (automatycznie)
admin-settings-help-privacy-image-cache-bytes-floor = Najmniejsza pamięć podręczna obrazów w bajtach w trybie automatycznym. Zakres 0–1073741824; domyślnie 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Górny limit rozmiaru pamięci podręcznej (automatycznie)
admin-settings-help-privacy-image-cache-bytes-ceiling = Największa pamięć podręczna obrazów w bajtach w trybie automatycznym. Zakres 0–1073741824; domyślnie 268435456.
admin-settings-key-privacy-cache-entries = Limit wpisów pamięci podręcznej obrazów
admin-settings-help-privacy-cache-entries = Maksymalna liczba obrazów w pamięci podręcznej; w trybie ręcznym stała liczba, w automatycznym górny limit. Zakres 16–65536; domyślnie 4096.
admin-settings-key-privacy-cache-entry-bytes = Największy obraz w pamięci podręcznej
admin-settings-help-privacy-cache-entry-bytes = Największy pojedynczy ponownie zakodowany obraz przechowywany w pamięci podręcznej, w bajtach. Większe obrazy są udostępniane, ale nie zapisywane. Zakres 65536–16777216; domyślnie 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Dolny limit czasu życia pamięci podręcznej (automatycznie)
admin-settings-help-privacy-image-cache-ttl-floor-secs = Najkrótszy czas życia pamięci podręcznej w sekundach w trybie automatycznym, stosowany przy niskim współczynniku trafień lub braku pamięci. Zakres 0–86400; domyślnie 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Górny limit czasu życia pamięci podręcznej (automatycznie)
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Najdłuższy czas życia pamięci podręcznej w sekundach w trybie automatycznym, stosowany przy wysokim współczynniku trafień i wolnej pamięci. Zakres 0–86400; domyślnie 21600.
admin-settings-key-privacy-cache-ttl-secs = Czas życia udanych obrazów w pamięci podręcznej
admin-settings-help-privacy-cache-ttl-secs = Liczba sekund, przez które zdekodowane obrazy można ponownie wykorzystać w tym procesie. Zakres 0–86400; domyślnie 3600. Zero wyłącza ponowne użycie udanych obrazów; autoryzacja jest zawsze sprawdzana najpierw.
admin-settings-key-privacy-image-latency-tracking = Śledzenie szybkości proxy obrazów
admin-settings-help-privacy-image-latency-tracking = Zapisuje w pamięci czas obsługi każdego obrazu przez proxy (mediana, p95, p99 oraz czas oczekiwania, pobierania i dekodowania) jako liczniki w stałych przedziałach czasu. Nie są zapisywane adresy URL, hosty ani konta. Domyślnie włączone; historia jest zerowana po restarcie i dotyczy pojedynczego procesu serwera. Wyłączenie zatrzymuje nowe zapisy; sekcja Szybkość nadal pokazuje to, co już zapisano.
admin-settings-key-privacy-negative-cache-ttl-secs = Czas życia nieudanych obrazów w pamięci podręcznej
admin-settings-help-privacy-negative-cache-ttl-secs = Liczba sekund, przez które kwalifikujące się błędy źródła są zapamiętywane, aby uniknąć powtarzanych żądań. Zakres 0–300; domyślnie 10. Zero wyłącza ponowne użycie błędów. Jawna ponowna próba czyści dotknięte błędy.
admin-settings-key-privacy-image-retry-attempts = Automatyczne ponowienia obrazów
admin-settings-help-privacy-image-retry-attempts = Maksymalna liczba ponowień przy tymczasowych błędach obrazów w jednym widocznym załadowaniu wiadomości. Zakres 0–8; domyślnie 3. Zero wyłącza automatyczne ponowienia, zachowując ręczne.
admin-settings-key-privacy-image-retry-delay-ms = Opóźnienie automatycznego ponowienia obrazu
admin-settings-help-privacy-image-retry-delay-ms = Liczba milisekund między automatycznymi ponowieniami obrazu, liczona od ustabilizowania bieżącej ramki. Zakres 500–60000; domyślnie 5000. Trwałe odmowy nie są ponawiane automatycznie.
admin-settings-key-privacy-image-decode-timeout-ms = Limit czasu dekodowania obrazu
admin-settings-help-privacy-image-decode-timeout-ms = Rzeczywisty czas w milisekundach dozwolony na jedno dekodowanie obrazu w piaskownicy po jego pobraniu. Zakres 1000–60000; domyślnie 10000. Zwiększ dla dużych obrazów; limity czasu sieci są osobne.
admin-settings-key-privacy-image-concurrency-mode = Profil współbieżności obrazów
admin-settings-help-privacy-image-concurrency-mode = Automatyczny dostosowuje się do mocy procesora, pamięci kontenera i liczby zatwierdzonych obrazów w każdej wiadomości. Profile Niskie zasoby, Zrównoważony i Wysoka przepustowość także się dostosowują, z różnymi limitami bezpieczeństwa. Ręczny używa bezpośrednio limitów liczbowych. Zastosowanie profilu zmienia tylko to ustawienie.
admin-settings-cat-image-proxy = Proxy obrazów
admin-settings-key-privacy-image-ram-max-mi-b = Maksimum warstwy RAM
admin-settings-help-privacy-image-ram-max-mi-b = Górna granica w MiB dla warstwy obrazów w pamięci, stosowana dodatkowo do automatycznego lub ręcznego rozmiaru pamięci podręcznej. Zero oznacza brak limitu. Zachowaj małą wartość i pozwól warstwie dyskowej przechowywać większość. Zakres 0–1048576; domyślnie 0.
admin-settings-key-privacy-image-disk-enabled = Warstwa dyskowa obrazów
admin-settings-help-privacy-image-disk-enabled = Przechowuje już ponownie zakodowane obrazy na dysku za warstwą RAM. Domyślnie wyłączone; gdy katalog nie istnieje, jest pełny lub tylko do odczytu, pamięć podręczna po cichu działa dalej tylko w RAM. Zapisywane są wyłącznie dane wyjściowe izolatki, pod haszowanymi nazwami plików.
admin-settings-key-privacy-image-disk-path = Katalog dyskowy obrazów
admin-settings-help-privacy-image-disk-path = Bezwzględna ścieżka katalogu warstwy dyskowej. Puste używa folderu image-cache obok pliku bazy danych. Zamontuj tam zapisywalny wolumin, gdy główny system plików jest tylko do odczytu. Ścieżki względne i .. są odrzucane i warstwa pozostaje wyłączona.
admin-settings-key-privacy-image-disk-mode = Dobór rozmiaru dysku obrazów
admin-settings-help-privacy-image-disk-mode = Automatycznie używa jednej dziesiątej miejsca, które ta pamięć podręczna może zająć (wolne miejsce plus to, co już przechowuje), między dolnym a górnym limitem i nigdy więcej niż połowy tego miejsca. Ręcznie używa dokładnie poniższego rozmiaru maksymalnego. Domyślnie automatycznie.
admin-settings-key-privacy-image-disk-max-mi-b = Rozmiar dysku obrazów (ręcznie)
admin-settings-help-privacy-image-disk-max-mi-b = Rozmiar warstwy dyskowej w MiB w trybie ręcznym; odczyt pokazuje rozmiar efektywny w obu trybach. Zakres 1–1048576; domyślnie 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Dolny limit rozmiaru dysku (automatycznie)
admin-settings-help-privacy-image-disk-floor-mi-b = Najmniejsza warstwa dyskowa w MiB w trybie automatycznym; używana także, gdy wolne miejsce jest nieznane. Zakres 0–1048576; domyślnie 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Górny limit rozmiaru dysku (automatycznie)
admin-settings-help-privacy-image-disk-ceiling-mi-b = Największa warstwa dyskowa w MiB w trybie automatycznym. Zakres 0–1048576; domyślnie 4096.
admin-settings-key-privacy-image-disk-entries = Limit wpisów na dysku
admin-settings-help-privacy-image-disk-entries = Maksymalna liczba obrazów przechowywanych na dysku. Każdy wpis zajmuje też stałą ilość RAM w indeksie, wliczaną do warstwy RAM. Zakres 16–1048576; domyślnie 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Największy obraz na dysku
admin-settings-help-privacy-image-disk-entry-bytes = Największy pojedynczy ponownie zakodowany obraz zapisywany na dysku, w bajtach. Obrazy powyżej limitu wpisu w RAM, ale mieszczące się w tym limicie, trafiają od razu na dysk. Zakres 65536–67108864; domyślnie 8388608.
admin-settings-key-privacy-image-disk-min-hits = Trafienia przed zapisem na dysk
admin-settings-help-privacy-image-disk-min-hits = Liczba trafień, jakie wpis w RAM musi mieć, aby jego wyrzucenie zapisało go na dysk zamiast odrzucić. Zero zapisuje każdy wyrzucany wpis, którego wynik przekracza próg degradacji. Zakres 0–100; domyślnie 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Kolejka zapisu na dysk
admin-settings-help-privacy-image-disk-write-queue-mi-b = Ilość danych obrazów w MiB, które mogą czekać na działający w tle zapis na dysk. Liczy się do warstwy RAM; po jej przekroczeniu wyrzucane obrazy są odrzucane zamiast kolejkowane. Zakres 1–256; domyślnie 8.
admin-settings-key-privacy-image-ttl-step-secs = Przyrost czasu życia na trafienie
admin-settings-help-privacy-image-ttl-step-secs = Liczba sekund dodawana do pozostałego czasu życia obrazu przy trafieniu, pomnożona przez 1 plus logarytm jego zanikającej liczby trafień, więc często używane obrazy żyją dłużej. Zero zostawia każdy obraz przy podstawowym czasie życia. Zakres 0–86400; domyślnie 600.
admin-settings-key-privacy-image-ttl-max-secs = Najdłuższy wydłużony czas życia
admin-settings-help-privacy-image-ttl-max-secs = Najdłuższy pozostały czas życia w sekundach, jaki mogą zbudować trafienia. Zakres 60–2592000; domyślnie 86400.
admin-settings-key-privacy-image-max-age-secs = Maksymalny wiek obrazu
admin-settings-help-privacy-image-max-age-secs = Wiek w sekundach, po którym obraz jest pobierany ponownie niezależnie od trafień. Ścieżka pobierania nie zachowuje ETag ani Last-Modified ze źródła, więc nie ma warunkowej rewalidacji; obraz jest po prostu pobierany od nowa. Zakres 3600–7776000; domyślnie 604800.
admin-settings-key-privacy-image-score-half-life-secs = Okres półtrwania wyniku
admin-settings-help-privacy-image-score-half-life-secs = Liczba sekund, po których liczba trafień i świeżość obrazu spadają o połowę w wyniku priorytetu decydującym o RAM, dysku i wyrzucaniu. Zakres 60–604800; domyślnie 3600.
admin-settings-key-privacy-image-score-demote-milli = Wynik degradacji
admin-settings-help-privacy-image-score-demote-milli = Wynik w tysięcznych, poniżej którego obraz w RAM jest przenoszony na dysk lub odrzucany. Próg awansu to ta wartość plus histereza. Zakres 0–10000; domyślnie 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Histereza awansu
admin-settings-help-privacy-image-score-hysteresis-percent = Procent, o jaki próg awansu leży powyżej progu degradacji. Wynik między nimi zostawia obraz tam, gdzie jest. Zakres 0–500; domyślnie 50.
admin-settings-key-privacy-image-context-open-percent = Waga otwartej wiadomości
admin-settings-help-privacy-image-context-open-percent = Waga wyniku w procentach dla obrazów żądanych dla wiadomości. Obrazy wiadomości liczą się jako ostatnie po dziesięciu minutach bezczynności. Zakres 100–1000; domyślnie 200.
admin-settings-key-privacy-image-context-recent-percent = Waga ostatniej wiadomości
admin-settings-help-privacy-image-context-recent-percent = Waga wyniku w procentach dla niedawno używanych obrazów. Żądania niewskazujące wiadomości mają wagę 100. Zakres 100–1000; domyślnie 150.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-account-share-percent = Udział konta w pamięci RAM
admin-settings-help-privacy-image-account-share-percent = Największa część warstwy RAM, jaką mogą zająć wstawienia jednego konta, gdy inne konta korzystają z pamięci podręcznej. Po jej przekroczeniu konto usuwa własne obrazy o najniższej punktacji. Pojedyncze konto może wykorzystać całość. Zakres 1–100; domyślnie 50.
admin-settings-key-privacy-image-sweep-secs = Odstęp czyszczenia pamięci podręcznej
admin-settings-help-privacy-image-sweep-secs = Liczba sekund między przebiegami w tle, które usuwają wygasłe obrazy, przeliczają punktację ograniczonej partii, przenoszą nieużywane obrazy z RAM przy niedoborze pamięci i ponownie sprawdzają wolne miejsce na dysku. Zakres 5–3600; domyślnie 30.
admin-settings-key-privacy-image-snapshot-secs = Odstęp migawek indeksu
admin-settings-help-privacy-image-snapshot-secs = Liczba sekund między atomowymi migawkami indeksu dyskowego, które zachowują liczniki trafień i wydłużone okresy ważności po restarcie bez zapisu przy każdym trafieniu. Zero wyłącza migawki; po restarcie odczytywany jest wtedy nagłówek każdego pliku. Zakres 0–86400; domyślnie 300.
admin-settings-key-privacy-image-sweep-batch = Rozmiar partii czyszczenia
admin-settings-help-privacy-image-sweep-batch = Liczba wpisów, które jeden przebieg konserwacyjny może wygasić, przeliczyć lub usunąć. Ogranicza pracę wykonywaną pod blokadą pamięci podręcznej. Zakres 16–8192; domyślnie 256.
admin-image-proxy-title = Stan pamięci podręcznej obrazów
admin-image-proxy-unavailable = Stan pamięci podręcznej obrazów jest niedostępny.
admin-image-proxy-unknown = nieznane
admin-image-proxy-ram = Warstwa RAM: użyto { $used } z { $budget } (skonfigurowano { $configured }), obrazów: { $entries }, gorących: { $hot }, zimnych: { $cold }, współczynnik trafień { $hit }.
admin-image-proxy-disk-off = Warstwa dyskowa: wyłączona.
admin-image-proxy-disk = Warstwa dyskowa: użyto { $bytes } z efektywnych { $budget } ({ $mode }; minimum { $floor }, maksimum { $ceiling }, ręcznie { $manual }), obrazów: { $entries }, gorących: { $hot }, zimnych: { $cold }, współczynnik trafień { $hit }.
admin-image-proxy-disk-inactive = Warstwa dyskowa: włączona, ale nieaktywna. { $reason }
admin-image-proxy-moves = Awansów na minutę: { $perMin }. Awansów łącznie { $promotions }, degradacji { $demotions }, odrzuceń { $drops }. Wpisy indeksu: { $index }.
admin-image-proxy-refresh = Odśwież stan
admin-settings-key-security-trust-enabled = Poziomy zaufania
admin-settings-help-security-trust-enabled = Przypisuje każdemu żądaniu poziom: zalogowany, znane źródło (niedawne udane logowanie z tego adresu) lub nieznane. Zmieniają się tylko limity żądań anonimowych; dane uwierzytelniające, drugie składniki i blokady nigdy nie są łagodzone.
admin-settings-key-security-trust-known-ttl-days = Okno znanego źródła (dni)
admin-settings-help-security-trust-known-ttl-days = Jak długo udane logowanie utrzymuje adres jako znany. Nieudane próby nigdy się nie liczą.
admin-settings-key-security-trust-known-capacity = Liczba przechowywanych znanych źródeł
admin-settings-help-security-trust-known-capacity = Największa liczba znanych źródeł przechowywanych w pamięci (każde to solony skrót, kraj i liczba kont). Najstarsze jest usuwane jako pierwsze.
admin-settings-key-security-trust-persist = Zachowaj znane źródła po restarcie
admin-settings-help-security-trust-persist = Zapisuje tabelę znanych źródeł (skróty z kluczem, kraj, czas ostatniej aktywności, liczba kont) w magazynie ustawień i wczytuje ją przy starcie. Wpisy starsze niż okno są usuwane.
admin-settings-key-security-trust-persist-interval-secs = Odstęp zapisu znanych źródeł (sekundy)
admin-settings-help-security-trust-persist-interval-secs = Liczba sekund między zapisami, od 30 do 3600. Zapis następuje tylko po zmianie tabeli oraz jeszcze raz przy normalnym zamknięciu.
admin-settings-key-security-trust-unknown-percent = Limit dla nieznanego źródła (%)
admin-settings-help-security-trust-unknown-percent = Limit żądań anonimowych z nieznanego źródła jako procent skonfigurowanych limitów. 100 zachowuje wartość bazową; mniej oznacza ostrzejsze limity.
admin-settings-key-security-trust-known-percent = Limit dla znanego źródła (%)
admin-settings-help-security-trust-known-percent = Limit żądań anonimowych ze znanego źródła. 200 podwaja budżety żądań oraz skraca o połowę odstęp po logowaniu i trudność wyzwania. Nigdy nie pomija danych uwierzytelniających, drugich składników ani blokad.
admin-settings-key-security-trust-ua-missing = Boty: pusty user-agent
admin-settings-help-security-trust-ua-missing = Działanie dla żądań anonimowych z pustym user-agent: zezwól, ogranicz lub zablokuj. Zalogowane sesje nigdy nie są dotknięte.
admin-settings-key-security-trust-ua-uncommon = Boty: nierozpoznany user-agent
admin-settings-help-security-trust-ua-uncommon = Działanie dla żądań anonimowych z nierozpoznanym user-agent: zezwól, ogranicz lub zablokuj.
admin-settings-key-security-trust-ua-automation = Boty: skrypty i automatyzacja
admin-settings-help-security-trust-ua-automation = Działanie dla żądań anonimowych podających się za skrypt, bibliotekę HTTP, przeglądarkę headless lub ogólnego bota: zezwól, ogranicz lub zablokuj.
admin-settings-key-security-trust-ua-search = Boty: roboty wyszukiwarek
admin-settings-help-security-trust-ua-search = Działanie dla żądań anonimowych podających się za robota wyszukiwarki: zezwól, ogranicz lub zablokuj.
admin-settings-key-security-trust-ua-ai-training = Boty: roboty trenujące AI
admin-settings-help-security-trust-ua-ai-training = Działanie dla żądań anonimowych podających się za GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended lub Applebot-Extended: zezwól, ogranicz lub zablokuj. Deklaracja to tylko nagłówek, nie zweryfikowana tożsamość.
admin-settings-key-security-trust-ua-ai-search = Boty: roboty wyszukiwania AI
admin-settings-help-security-trust-ua-ai-search = Działanie dla żądań anonimowych podających się za OAI-SearchBot, PerplexityBot lub Claude-SearchBot: zezwól, ogranicz lub zablokuj.
admin-settings-key-security-trust-ua-ai-user = Boty: pobieranie AI na żądanie użytkownika
admin-settings-help-security-trust-ua-ai-user = Działanie dla żądań anonimowych podających się za ChatGPT-User lub Claude-User: zezwól, ogranicz lub zablokuj.
admin-settings-key-security-trust-ua-custom = Boty: wzorce własne
admin-settings-help-security-trust-ua-custom = Działanie dla żądań anonimowych pasujących do własnych wzorców user-agent: zezwól, ogranicz lub zablokuj.
admin-settings-key-security-trust-admin-geo-enabled = Blokuj wykrywanie konsoli administratora z nieznanych krajów
admin-settings-help-security-trust-admin-geo-enabled = Blokuje źródło, które sonduje konsolę administratora z kraju bez niedawnego udanego logowania. Wymaga czytelnej bazy krajów i ochrony przed sondowaniem konsoli. MW_ADMIN_GEO_ENABLED=false wyłącza to na potrzeby odzyskiwania dostępu.
admin-settings-key-security-trust-admin-geo-countries = Zawsze znane kraje (blokada konsoli administratora)
admin-settings-help-security-trust-admin-geo-countries = Kody krajów ISO wielkimi literami, zawsze traktowane jako znane, poza krajami niedawnych udanych logowań.
admin-settings-key-security-trust-admin-geo-lookup-failure = Źródło bez kraju
admin-settings-help-security-trust-admin-geo-lookup-failure = allow nie blokuje i zapisuje to zdarzenie w dzienniku; unknown traktuje źródło bez wykrytego kraju jako nieznany kraj.
admin-settings-key-security-trust-admin-geo-threshold = Sondowania przed blokadą
admin-settings-help-security-trust-admin-geo-threshold = Liczba sondowań konsoli administratora z nieznanego kraju w oknie, która uruchamia blokadę.
admin-settings-key-security-trust-admin-geo-window-secs = Okno sondowania (sekundy)
admin-settings-help-security-trust-admin-geo-window-secs = Okno dla progu sondowań.
admin-settings-key-security-trust-admin-geo-block-on-first = Blokuj od pierwszego sondowania
admin-settings-help-security-trust-admin-geo-block-on-first = Blokuje od pierwszego sondowania z nieznanego kraju. Zwiększa ryzyko zablokowania podróżującego administratora.
admin-settings-key-security-trust-admin-geo-block-secs = Pierwsza blokada (sekundy)
admin-settings-help-security-trust-admin-geo-block-secs = Czas trwania pierwszej blokady.
admin-settings-key-security-trust-admin-geo-second-block-secs = Druga blokada (sekundy)
admin-settings-help-security-trust-admin-geo-second-block-secs = Czas trwania drugiej blokady tego samego źródła w okresie powtórzeń.
admin-settings-key-security-trust-admin-geo-max-block-secs = Blokada powtórna (sekundy)
admin-settings-help-security-trust-admin-geo-max-block-secs = Czas trwania trzeciej i każdej kolejnej blokady w okresie powtórzeń.
admin-settings-key-security-trust-admin-geo-repeat-days = Okres powtórzeń (dni)
admin-settings-help-security-trust-admin-geo-repeat-days = Blokada w ciągu tylu dni od poprzedniej jest uznawana za powtórną i stosuje następny, dłuższy czas.
admin-settings-key-security-signin-account-geo-mode = Zasada kraju logowania dla konta
admin-settings-help-security-signin-account-geo-mode = Domyślnie wyłączona. Blokuje logowania z krajów, z których konto dotąd nie korzystało, lub z dowolnego kraju spoza jego listy dozwolonych. Adresy IP i kraje z listy dozwolonych zawsze przechodzą. Dotyczy tylko nowych logowań do poczty w przeglądarce, nigdy istniejących sesji ani klientów protokołów. Wykrywanie kraju opiera się na bazie geolokalizacji IP, która może być błędna lub nieaktualna; VPN-y, sieci komórkowe, serwery proxy i podróże mogą zablokować prawidłowe logowanie lub przepuścić zagraniczne. Dodaj zaufane adresy IP lub kraje do listy dozwolonych.
admin-settings-key-security-signin-account-geo-allow-countries = Dozwolone kraje konta
admin-settings-help-security-signin-account-geo-allow-countries = Kody krajów ISO wielkimi literami, zawsze dozwolone dla konta. Wykrywanie kraju opiera się na bazie geolokalizacji IP, która może być błędna lub nieaktualna; VPN-y, sieci komórkowe, serwery proxy i podróże mogą zablokować prawidłowe logowanie lub przepuścić zagraniczne. Dodaj zaufane adresy IP lub kraje do listy dozwolonych.
admin-settings-key-security-signin-account-geo-allow-ips = Dozwolone adresy IP konta
admin-settings-help-security-signin-account-geo-allow-ips = Adresy IP lub sieci CIDR zawsze dozwolone dla konta. Pojedynczy adres IPv6 oznacza jego sieć /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Dozwolone kraje dla każdego konta
admin-settings-help-security-signin-account-geo-global-allow-countries = Kody krajów ISO wielkimi literami dozwolone dla każdego konta, dodawane do listy każdego konta.
admin-settings-key-security-signin-account-geo-global-allow-ips = Dozwolone adresy IP dla każdego konta
admin-settings-help-security-signin-account-geo-global-allow-ips = Adresy IP lub sieci CIDR (biuro lub VPN) dozwolone dla każdego konta, dodawane do listy każdego konta.
admin-settings-key-security-signin-account-geo-lookup-failure = Źródło bez kraju (zasada konta)
admin-settings-help-security-signin-account-geo-lookup-failure = Gdy dla źródła nie wykryto kraju: zezwól na logowanie i zapisz to w dzienniku albo je odrzuć.
admin-settings-key-security-signin-account-geo-history-days = Historia krajów konta (dni)
admin-settings-help-security-signin-account-geo-history-days = Przez ile dni kraj pozostaje w historii logowań konta na potrzeby zasady nietypowych krajów. Jest przechowywana oddzielnie od dziennika uwierzytelniania, więc skrócenie dziennika jej nie skraca.
admin-settings-key-security-geo-no-database = Brak bazy krajów: zachowanie domyślne
admin-settings-help-security-geo-no-database = Co robią funkcje oparte na kraju, gdy nie wczytano bazy: zezwalają, traktują źródło jako nieznany kraj albo blokują (blokada dotyczy tylko logowań). Obowiązuje też ustawienie samej funkcji; wygrywa surowsze.
admin-settings-key-security-geo-not-found = Adresu nie ma w bazie: zachowanie domyślne
admin-settings-help-security-geo-not-found = Co robią funkcje oparte na kraju, gdy baza jest wczytana, ale nie ma wpisu dla adresu. Adresy prywatne, pętli zwrotnej i link-local to sieci lokalne i nigdy nie są dotknięte.
admin-settings-key-security-signin-geo-no-database = Zasada logowania instancji bez bazy
admin-settings-help-security-signin-geo-no-database = Gdy zasada kraju logowania instancji jest włączona, a baza nie jest wczytana: blokuj logowania (zamknięcie awaryjne) lub zezwalaj bez sprawdzania. Sieci lokalne są zawsze dozwolone.
admin-settings-key-security-geo-session-lookup-failure = Alerty sesji bez kraju
admin-settings-help-security-geo-session-lookup-failure = Gdy nie można ustalić kraju żądania zalogowanej sesji: zignoruj lub zgłoś jeden alert sesji na sesję dziennie. Nigdy nie blokuje ani nie unieważnia sesji.
admin-settings-key-security-protocol-framing-mode = Straż protokołu: ramkowanie żądań
admin-settings-help-security-protocol-framing-mode = Klasa request smuggling: Content-Length razem z Transfer-Encoding, Transfer-Encoding inne niż dokładnie chunked (lub w HTTP/1.0 i HTTP/2), nienumeryczny lub sprzeczny Content-Length. Tryb enforce odpowiada 400 i zamyka połączenie; detect tylko zlicza. Wyjątek dla prefiksu trasy nigdy nie dotyczy tej klasy.
admin-settings-key-security-protocol-host-mode = Straż protokołu: składnia Host
admin-settings-help-security-protocol-host-mode = Składnia nagłówka Host: brak w HTTP/1.1, pusty, powtórzony lub zawierający dane użytkownika, białe znaki albo znaki ścieżki. Literały IPv6, porty i końcowa kropka są akceptowane. Tryb enforce odpowiada 400 i zamyka połączenie.
admin-settings-key-security-protocol-host-pinning-mode = Straż protokołu: Host musi pasować do nazwy publicznej
admin-settings-help-security-protocol-host-pinning-mode = Host niepasujący do hosta z MW_PUBLIC_URL ani do żadnego z allowedHosts. Nic nie robi, dopóki żaden z nich nie wskazuje hosta. Domyślnie detect, ponieważ serwery proxy i kontrole stanu mogą wysyłać nazwę wewnętrzną.
admin-settings-key-security-protocol-path-control-mode = Straż protokołu: znaki sterujące w ścieżce
admin-settings-help-security-protocol-path-control-mode = NUL, znaki sterujące lub zakodowane CR/LF (%00, %0d, %0a) w ścieżce żądania. Tryb enforce odpowiada 400. Nigdy nie jest wyłączany prefiksem trasy.
admin-settings-key-security-protocol-target-mode = Straż protokołu: sztuczki w celu żądania
admin-settings-help-security-protocol-target-mode = Podwójne lub nadmiarowe kodowanie procentowe kropek i ukośników, zakodowane kropki, segmenty z kropkami, puste segmenty ścieżki (//), ukośniki odwrotne, surowe bajty spoza ASCII oraz cel w postaci bezwzględnej wskazujący inny urząd niż Host.
admin-settings-key-security-protocol-path-length-mode = Straż protokołu: długość ścieżki i zapytania
admin-settings-help-security-protocol-path-length-mode = Ścieżka lub zapytanie dłuższe niż maxPathBytes lub maxQueryBytes. Tryb enforce odpowiada 414.
admin-settings-key-security-protocol-method-mode = Straż protokołu: metody HTTP
admin-settings-help-security-protocol-method-mode = Metoda spoza allowedMethods (domyślnie TRACE, TRACK, CONNECT oraz metody nieznane lub pisane małymi literami). Tryb enforce odpowiada 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Straż protokołu: powtórzone nagłówki jednowartościowe
admin-settings-help-security-protocol-duplicate-headers-mode = Nagłówek, który powinien wystąpić raz, występujący więcej razy (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, nagłówki warunkowe i zakresu), powtórzony Content-Length o równych wartościach lub powtórzone nagłówki Cookie w HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Straż protokołu: nagłówki przekazywania od niezaufanych peerów
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* lub X-Real-IP wysłane przez peera spoza MW_TRUSTED_PROXIES. Nagłówki nigdy nie są zaufane; to ustawienie rejestruje próbę. Wlicza się do ochrony adaptacyjnej tylko wtedy, gdy skonfigurowano zaufane serwery proxy.
admin-settings-key-security-protocol-header-hygiene-mode = Straż protokołu: higiena nagłówków
admin-settings-help-security-protocol-header-hygiene-mode = Liczba nagłówków, pojedyncza wartość lub nagłówek Cookie ponad skonfigurowane limity; bajty spoza ASCII w nagłówkach; Upgrade lub Connection: upgrade poza trasą WebSocket; Connection wskazujące nagłówki takie jak Content-Length lub Host. Tryb enforce odpowiada 431 dla naruszeń rozmiaru i liczby oraz 400 dla pozostałych.
admin-settings-key-security-protocol-body-mode = Straż protokołu: ramkowanie treści żądania
admin-settings-help-security-protocol-body-mode = GET lub HEAD z treścią, Content-Length powyżej maxBodyBytes (413), Expect inne niż 100-continue (417) lub bez treści.
admin-settings-key-security-protocol-max-path-bytes = Straż protokołu: najdłuższa ścieżka (bajty)
admin-settings-help-security-protocol-max-path-bytes = Najdłuższa ścieżka żądania, zanim klasa długości ścieżki ją zgłosi.
admin-settings-key-security-protocol-max-query-bytes = Straż protokołu: najdłuższe zapytanie (bajty)
admin-settings-help-security-protocol-max-query-bytes = Najdłuższy ciąg zapytania, zanim klasa długości ścieżki go zgłosi.
admin-settings-key-security-protocol-max-header-count = Straż protokołu: najwięcej nagłówków
admin-settings-help-security-protocol-max-header-count = Największa liczba pól nagłówka w jednym żądaniu, zanim klasa higieny nagłówków ją zgłosi. Obowiązuje też twardy limit samego nasłuchu (MW_HTTP_MAX_HEADERS).
admin-settings-key-security-protocol-max-header-value-bytes = Straż protokołu: najdłuższa wartość nagłówka (bajty)
admin-settings-help-security-protocol-max-header-value-bytes = Najdłuższa pojedyncza wartość nagłówka, zanim klasa higieny nagłówków ją zgłosi.
admin-settings-key-security-protocol-max-cookie-bytes = Straż protokołu: największy nagłówek Cookie (bajty)
admin-settings-help-security-protocol-max-cookie-bytes = Łączny rozmiar nagłówków Cookie, zanim klasa higieny nagłówków go zgłosi.
admin-settings-key-security-protocol-max-body-bytes = Straż protokołu: największa zadeklarowana treść (bajty)
admin-settings-help-security-protocol-max-body-bytes = Największy Content-Length, jaki klasa treści akceptuje bez zgłoszenia. Utrzymuj go powyżej największego przesyłania lub importu przyjmowanego przez trasę; wyjątek dla prefiksu trasy może go podnieść dla jednej trasy.
admin-settings-key-security-protocol-allowed-methods = Straż protokołu: dozwolone metody
admin-settings-help-security-protocol-allowed-methods = Metody akceptowane przez klasę metod. Domyślna lista zawiera zwykłe czasowniki HTTP oraz czasowniki WebDAV, CalDAV i CardDAV; serwer nie montuje dziś przychodzących tras DAV, więc te trafiają na 405 w routerze.
admin-settings-key-security-protocol-allowed-hosts = Straż protokołu: dodatkowe akceptowane hosty
admin-settings-help-security-protocol-allowed-hosts = Nazwy hostów akceptowane przez klasę przypinania hosta oprócz hosta z MW_PUBLIC_URL, na przykład nazwa wewnętrzna używana przez serwer proxy lub kontrolę stanu. Porty i końcowa kropka są ignorowane.
admin-settings-key-security-protocol-exempt-prefixes = Straż protokołu: wyłączone prefiksy tras
admin-settings-help-security-protocol-exempt-prefixes = Prefiksy tras (każdy zaczyna się od / i ma więcej niż jeden znak), dla których klasy heurystyczne nie działają, dla prawidłowego klienta, który tego potrzebuje, na przykład trasy dużego przesyłania. Dopasowuje całe segmenty ścieżki. Naruszeń ramkowania i znaków sterujących w ścieżce nigdy się nie wyłącza, a / jest ignorowany.
admin-settings-key-security-health-mode = Dostęp do punktu kontroli stanu
admin-settings-help-security-health-mode = Kto może wywołać /healthz i /readyz. internal-only: adresy klientów pętli zwrotnej, prywatne, link-local i unique-local. allowlist: te oraz poniższe sieci. open: każdy. Pozostali dostają taki sam 404 jak dla nieznanej ścieżki. Żądanie z tokenem MW_HEALTH_TOKEN jest zawsze dopuszczane. Adres klienta to ten ustalony przez zaufane serwery proxy: za zwrotnym proxy bez MW_FORWARDED_MODE i MW_TRUSTED_PROXIES każde żądanie wygląda jak prywatny adres proxy, więc dopuszczany jest cały internet.
admin-settings-key-security-health-networks = Sieci punktu kontroli stanu
admin-settings-help-security-health-networks = Adresy IP lub sieci CIDR, które mogą wywołać /healthz i /readyz w trybie allowlist, oprócz zakresów wewnętrznych, na przykład zewnętrzny monitor dostępności.
admin-settings-key-security-spray-enabled = Wykrywanie password spray
admin-settings-help-security-spray-enabled = Zlicza różne nazwy użytkowników, z którymi jedno źródło się nie loguje. W pamięci przechowywany jest tylko skrót z kluczem każdej nazwy.
admin-settings-key-security-spray-window-secs = Okno spray (sekundy)
admin-settings-help-security-spray-window-secs = Okno, w którym nieudane logowania jednego źródła są liczone według różnych nazw użytkowników. Domyślnie 900.
admin-settings-key-security-spray-challenge-at = Spray: próg wyzwania
admin-settings-help-security-spray-challenge-at = Liczba różnych nazw użytkowników z błędami z jednego źródła w oknie, która wymusza dla tego źródła wyzwanie proof-of-work. Udane logowania nigdy się nie liczą, więc wspólny adres biura lub kampusu nie jest dotknięty. Domyślnie 10.
admin-settings-key-security-spray-pace-at = Spray: próg spowolnienia
admin-settings-help-security-spray-pace-at = Liczba różnych nazw użytkowników z błędami, która dodaje dodatkowe oczekiwanie między próbami z tego źródła. Domyślnie 15.
admin-settings-key-security-spray-lock-at = Spray: próg blokady
admin-settings-help-security-spray-lock-at = Liczba różnych nazw użytkowników z błędami, która blokuje źródło na czas blokady, przez istniejącą blokadę logowania. Domyślnie 25.
admin-settings-key-security-spray-pace-secs = Spray: czas oczekiwania (sekundy)
admin-settings-help-security-spray-pace-secs = Liczba sekund między próbami ze źródła na progu spowolnienia lub powyżej. Domyślnie 10.
admin-settings-key-security-spray-lock-secs = Spray: czas blokady (sekundy)
admin-settings-help-security-spray-lock-secs = Czas trwania tymczasowej blokady. Odblokowanie źródła przez administratora zeruje też jego licznik. Domyślnie 900.
admin-settings-key-security-spray-known-multiplier = Spray: mnożnik dla znanego źródła
admin-settings-help-security-spray-known-multiplier = Mnożnik trzech progów dla znanego źródła, czyli takiego z niedawnym udanym logowaniem. Domyślnie 3.
admin-settings-key-security-spray-capacity = Spray: śledzone źródła
admin-settings-help-security-spray-capacity = Największa liczba źródeł śledzonych w pamięci; najdawniej aktywne są usuwane jako pierwsze. Domyślnie 20000.
admin-settings-key-security-spike-enabled = Wykrywanie skoków w całej instancji
admin-settings-help-security-spike-enabled = Uczy się poziomu bazowego nieudanych logowań i przechodzi w tryb podwyższony, gdy cała instancja znacznie go przekracza. Nigdy nie blokuje instancji.
admin-settings-key-security-spike-multiplier-percent = Skok: mnożnik (%)
admin-settings-help-security-spike-multiplier-percent = Sygnał się liczy, gdy wynosi co najmniej tyle procent wyuczonego poziomu bazowego (400 = czterokrotność) i przekracza swój bezwzględny próg minimalny.
admin-settings-key-security-spike-min-failures-per-min = Skok: minimum błędów na minutę
admin-settings-help-security-spike-min-failures-per-min = Próg minimalny: liczba nieudanych logowań na minutę, poniżej której sygnał częstości błędów się nie liczy. Domyślnie 20.
admin-settings-key-security-spike-min-sources = Skok: minimum źródeł z błędami
admin-settings-help-security-spike-min-sources = Próg minimalny: liczba różnych źródeł z błędami w pięciominutowym przedziale, poniżej której ten sygnał się nie liczy. Domyślnie 10.
admin-settings-key-security-spike-min-usernames = Skok: minimum nazw użytkowników z błędami
admin-settings-help-security-spike-min-usernames = Próg minimalny: liczba różnych nazw użytkowników z błędami w pięciominutowym przedziale, poniżej której ten sygnał się nie liczy. Domyślnie 30.
admin-settings-key-security-spike-signals-required = Skok: wymagane sygnały
admin-settings-help-security-spike-signals-required = Liczba sygnałów, które muszą jednocześnie przekraczać poziom bazowy i próg minimalny. Rozproszona sygnatura spray liczy się za dwa. Domyślnie 2.
admin-settings-key-security-spike-warmup-hours = Skok: rozgrzewka (godziny)
admin-settings-help-security-spike-warmup-hours = Godziny obserwowanego ruchu, zanim może się rozpocząć tryb podwyższony. Do tego czasu poziom bazowy tylko się uczy. Domyślnie 48.
admin-settings-key-security-spike-adapt-hours = Skok: adaptacja poziomu bazowego (godziny)
admin-settings-help-security-spike-adapt-hours = Stała czasowa poziomu bazowego. Dłuższa wolniej uwzględnia wzrost. Skok nigdy nie aktualizuje poziomu bazowego. Domyślnie 168.
admin-settings-key-security-spike-duration-secs = Skok: czas trybu podwyższonego (sekundy)
admin-settings-help-security-spike-duration-secs = Jak długo trwa tryb podwyższony po ostatniej oznaki skoku; jest przedłużany, dopóki sygnały pozostają powyżej połowy poziomu wyzwalającego. Domyślnie 1800.
admin-settings-key-security-spike-force-challenge = Skok: wymuś wyzwanie
admin-settings-help-security-spike-force-challenge = W trybie podwyższonym wymaga wyzwania proof-of-work przy anonimowych logowaniach z nieznanych źródeł. Uwierzytelnieni użytkownicy i znane źródła nie są dotknięci.
admin-settings-key-security-spike-pace-factor-percent = Skok: oczekiwanie przy logowaniu (%)
admin-settings-help-security-spike-pace-factor-percent = W trybie podwyższonym oczekiwanie przy logowaniu dla nieznanych źródeł jako procent zwykłego oczekiwania. Domyślnie 300.
admin-settings-key-security-session-binding-enabled = Powiązanie sesji
admin-settings-help-security-session-binding-enabled = Wiąże każdą sesję przeglądarki ze skrótami z kluczem atrybutów klienta. Klienty natywne z tokenem bearer, klucze API i klienty protokołów są wyłączone.
admin-settings-key-security-session-binding-attributes = Powiązanie sesji: atrybuty
admin-settings-help-security-session-binding-attributes = Wiązane atrybuty: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Network jest domyślnie wyłączony, bo sieci komórkowe się zmieniają. Nieznane nazwy są ignorowane.
admin-settings-key-security-session-binding-strictness = Powiązanie sesji: ścisłość
admin-settings-help-security-session-binding-strictness = Tolerant akceptuje aktualizacje przeglądarki i zmianę jednego miękkiego atrybutu; strict zgłasza każdą miękką różnicę.
admin-settings-key-security-session-binding-webmail-action = Powiązanie sesji: działanie dla poczty w przeglądarce
admin-settings-help-security-session-binding-webmail-action = Co robi niezgodność sesji poczty w przeglądarce: off, detect (tylko zapis), step-up (ponowna prośba o hasło i drugi składnik) lub unieważnienie sesji. Domyślnie step-up.
admin-settings-key-security-session-binding-admin-action = Powiązanie sesji: działanie dla konsoli administratora
admin-settings-help-security-session-binding-admin-action = Co robi niezgodność sesji konsoli administratora: off, detect, step-up lub unieważnienie. Domyślnie unieważnienie.
admin-settings-key-security-theft-enabled = Wykrywanie kradzieży sesji
admin-settings-help-security-theft-enabled = Wykrywa ponowne użycie zrotowanego identyfikatora sesji, niemożliwą podróż i równoczesne użycie z różnych krajów.
admin-settings-key-security-theft-rotation-grace-secs = Kradzież: tolerancja rotacji (sekundy)
admin-settings-help-security-theft-rotation-grace-secs = Po rotacji sesji stary identyfikator może być przedstawiony ponownie przez tyle czasu (żądanie w toku), zanim zostanie uznany za kradzież. Domyślnie 30.
admin-settings-key-security-theft-min-travel-secs = Kradzież: minimalny czas podróży (sekundy)
admin-settings-help-security-theft-min-travel-secs = Zmiana kraju szybciej niż po tylu sekundach od poprzedniego żądania jest traktowana jako niemożliwa podróż. Wykrywanie kraju może być niedokładne; VPN-y i sieci komórkowe mogą powodować fałszywe zmiany. Domyślnie 3600.
admin-settings-key-security-theft-concurrent-use = Kradzież: użycie równoczesne
admin-settings-help-security-theft-concurrent-use = Zgłasza jedną sesję używaną naprzemiennie z różnych krajów (lub sieci) w ciągu pięciu minut: off, country lub network. Wykrywanie kraju może być niedokładne; VPN-y i sieci komórkowe mogą powodować fałszywe zmiany.
admin-settings-key-security-theft-webmail-action = Kradzież: reakcja dla poczty w przeglądarce
admin-settings-help-security-theft-webmail-action = Reakcja na niemożliwą podróż lub użycie równoczesne w sesji poczty w przeglądarce: alert, step-up lub unieważnienie. Ponowne użycie zrotowanej sesji zawsze ją unieważnia. Domyślnie step-up.
admin-settings-key-security-theft-admin-action = Kradzież: reakcja dla konsoli administratora
admin-settings-help-security-theft-admin-action = Reakcja na niemożliwą podróż lub użycie równoczesne w sesji konsoli administratora: alert, step-up lub unieważnienie. Domyślnie unieważnienie.
admin-settings-key-security-theft-retention-days = Kradzież: przechowywanie rekordów (dni)
admin-settings-help-security-theft-retention-days = Jak długo przechowywane są znaczniki zrotowanych sesji i rekordy bezpieczeństwa poszczególnych sesji; ustaw co najmniej tyle, ile wynosi najdłuższy czas życia sesji. Domyślnie 15.
admin-settings-key-security-alerts-country-change = Alert o zmianie kraju
admin-settings-help-security-alerts-country-change = Ostrzega, gdy zalogowana sesja jest używana z kraju innego niż ten, w którym została rozpoczęta. Wykrywanie kraju może być niedokładne; VPN-y i sieci komórkowe mogą powodować fałszywe zmiany. Użytkownicy mogą to zmienić, chyba że ustawienie jest zablokowane; wyłączenie przez użytkownika jest honorowane tylko wtedy, gdy wyciszanie jest dozwolone.
admin-settings-key-security-alerts-country-change-enabled = Alert o zmianie kraju: przełącznik globalny

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-alerts-country-change-enabled = Wyłączenie wyłącza alert o zmianie kraju dla wszystkich. Wykrywanie kraju może być niedokładne; VPN-y i sieci komórkowe mogą powodować fałszywe zmiany.
admin-settings-key-security-alerts-country-change-cooldown-secs = Alert o zmianie kraju: przerwa (sekundy)
admin-settings-help-security-alerts-country-change-cooldown-secs = Minimalny czas między alertami o zmianie kraju w jednej sesji. Każdy nowy kraj wywołuje alert raz na sesję. Domyślnie 300.
admin-settings-key-security-alerts-country-change-user-mute = Alert o zmianie kraju: zezwól użytkownikom na wyciszenie
admin-settings-help-security-alerts-country-change-user-mute = Czy użytkownik może wyłączyć własny alert o zmianie kraju. Gdy wyłączone, serwer nadal alarmuje, nawet jeśli ustawienie użytkownika jest wyłączone.
admin-settings-key-security-alerts-channels = Kanały alertów bezpieczeństwa
admin-settings-help-security-alerts-channels = Jak ostrzeżenia o bezpieczeństwie sesji docierają do użytkownika: w aplikacji, jako wiadomość w jego własnej skrzynce lub oba sposoby. Użytkownicy mogą to zmienić, chyba że ustawienie jest zablokowane.
admin-settings-key-security-alerts-email-enabled = Alerty bezpieczeństwa e-mailem
admin-settings-help-security-alerts-email-enabled = Pozwala dostarczać ostrzeżenia o bezpieczeństwie sesji do własnej skrzynki użytkownika. Żadna wiadomość nie jest wysyłana przez SMTP; bez sesji dostarczania ostrzeżenie pozostaje w aplikacji i dzienniku audytu.
admin-settings-key-security-alerts-sensitivity = Czułość alertów bezpieczeństwa
admin-settings-help-security-alerts-sensitivity = Niska ostrzega tylko o ustaleniach dotyczących lokalizacji i nigdy nie wymusza ponownego uwierzytelnienia ani nie unieważnia; normalna podąża za działaniami administratora; wysoka podwaja odstęp podróży i sprawdza też równoczesne sieci. Użytkownicy mogą to zmienić, chyba że ustawienie jest zablokowane.
admin-settings-key-security-alerts-notice-retention-days = Powiadomienia bezpieczeństwa: przechowywanie (dni)
admin-settings-help-security-alerts-notice-retention-days = Jak długo powiadomienia bezpieczeństwa w aplikacji są przechowywane dla konta (najwyżej 20 na konto). Domyślnie 30.
admin-settings-inline-effective = Obowiązuje:
admin-settings-inline-reset = Odrzuć zmiany
admin-settings-inline-domains = Wartości dla domen
admin-attachments-tab-lists = Listy dozwolonych i blokowanych
admin-attachments-tab-limits = Limity
admin-attachments-tab-delivery = Pobieranie i podgląd
admin-attachments-tab-warnings = Ostrzeżenia
admin-attachments-tab-test = Test pliku
admin-attachments-tab-denied = Brak dostępu do tej karty.
admin-attachments-group-lists = Listy dozwolonych i blokowanych
admin-attachments-group-limits = Limit rozmiaru
admin-attachments-group-delivery = Pobieranie i podgląd
admin-attachments-group-warnings = Ostrzeżenia
admin-attachments-add-extension = Rozszerzenie, na przykład pdf lub .tar.gz
admin-attachments-add-mime = Typ MIME, na przykład image/png lub image/*
admin-attachments-add = Dodaj
admin-attachments-remove = Usuń { $rule }
admin-attachments-filter = Filtruj listę
admin-attachments-list-label = Reguły na tej liście
admin-attachments-count = Reguł: { $count } z { $max }
admin-attachments-empty-list = Brak reguł na tej liście.
admin-attachments-none-match = Żadna reguła nie pasuje do filtra.
admin-attachments-bulk = Wklej wiele
admin-attachments-bulk-add = Dodaj wszystkie
admin-attachments-import = Importuj plik tekstowy
admin-attachments-export = Eksportuj jako tekst
admin-attachments-err-extension = To nie jest rozszerzenie. Użyj liter, cyfr, myślników i podkreśleń, do 32 znaków; kropki mogą oddzielać części, jak w tar.gz.
admin-attachments-err-mime = To nie jest typ MIME. Użyj typ/podtyp lub typ/* dla każdego podtypu.
admin-attachments-err-long = Ta reguła jest za długa.
admin-attachments-invalid = Nie dodano, nieprawidłowa reguła: { $entries }
admin-attachments-duplicates = Już na liście: { $rules }
admin-attachments-overflow = Lista może zawierać najwyżej { $max } reguł. Pozostałych nie dodano.
admin-attachments-conflict-blockedOverAllowed = { $rule } pokrywa się z { $other } na drugiej liście. Reguły blokujące mają pierwszeństwo, więc jest zablokowane.
admin-attachments-conflict-blockedOverWarn = { $rule } pokrywa się z { $other } na drugiej liście. Zablokowany plik jest odrzucany przed jakimkolwiek ostrzeżeniem, więc ostrzeżenie nigdy się nie pojawi.
admin-attachments-conflict-allowedAndWarn = { $rule } pokrywa się z { $other } na drugiej liście. Plik jest dozwolony i wyświetla ostrzeżenie.
admin-attachments-bytes-unit = Jednostka
admin-attachments-unlimited = Bez limitu rozmiaru
admin-attachments-bytes-equals = { $size } ({ $bytes } bajtów)
admin-attachments-list-allowedExtensions = dozwolone rozszerzenia
admin-attachments-list-allowedMimeTypes = dozwolone typy MIME
admin-attachments-list-blockedExtensions = blokowane rozszerzenia
admin-attachments-list-blockedMimeTypes = blokowane typy MIME
admin-attachments-list-warnExtensions = rozszerzenia z ostrzeżeniem
admin-attachments-list-warnMimeTypes = typy MIME z ostrzeżeniem
admin-attachments-sim-title = Test pliku
admin-attachments-sim-help = Pyta serwer, jak zapisana polityka traktuje plik o tej nazwie, typie i rozmiarze. Nic nie jest zmieniane ani zapisywane. Wiadomości i załączniki nigdy nie są odczytywane.
admin-attachments-sim-filename = Nazwa pliku
admin-attachments-sim-mime = Typ MIME
admin-attachments-sim-size = Rozmiar
admin-attachments-sim-run = Testuj
admin-attachments-sim-invalid = Podaj nazwę pliku i rozmiar równy zero lub większy.
admin-attachments-sim-allowed = Dozwolony
admin-attachments-sim-warn = Dozwolony z ostrzeżeniem
admin-attachments-sim-blocked = Niedostarczony
admin-attachments-sim-preview-yes = Podgląd jest dozwolony.
admin-attachments-sim-preview-no = Podgląd nie jest dozwolony.
admin-attachments-sim-checked = Sprawdzony rozmiar: { $size }.
admin-attachments-sim-matched = Dopasowane reguły na listach: { $lists }.
admin-attachments-sim-matched-none = Nie dopasowano żadnej reguły zezwalającej, blokującej ani ostrzegającej.
admin-attachments-reason-downloadsDisabled = Pobieranie jest wyłączone.
admin-attachments-reason-attachmentTooLarge = Plik jest większy niż limit rozmiaru.
admin-attachments-reason-attachmentBlocked = Pasuje reguła blokująca.
admin-attachments-reason-attachmentNotAllowed = Istnieje lista dozwolonych i nic na niej nie pasuje.
admin-attachments-reason-attachmentContainerRestricted = Dołączone wiadomości i pliki TNEF są odrzucane, gdy włączone jest jakiekolwiek ograniczenie, ponieważ ich zawartość nie jest wymieniona.
admin-settings-key-mail-reply-images = Obrazy w odpowiedziach i przekazaniach
admin-settings-help-mail-reply-images = Czy odpowiedź lub przekazanie zawiera obrazy oryginalnej wiadomości: pytaj (domyślnie), zawsze zezwalaj, zezwalaj tylko dla zaufanych domen nadawców i pytaj w pozostałych przypadkach lub zawsze odmawiaj. Zablokuj to ustawienie, aby je wymusić. Obrazy są odczytywane tylko wtedy, gdy są dołączane.
admin-settings-key-mail-reply-image-trusted-domains = Zaufane domeny nadawców obrazów
admin-settings-help-mail-reply-image-trusted-domains = Do 200 dokładnych domen nadawców, których obrazy są dołączane bez pytania, gdy polityka to zaufane domeny. Użytkownicy dodają do własnej listy z monitu o cytat, chyba że to ustawienie jest zablokowane.
admin-settings-key-mail-send-recall-delay-seconds = Okno cofnięcia wysłania (domyślne)
admin-settings-help-mail-send-recall-delay-seconds = Ile sekund wysłana wiadomość czeka w Skrzynce nadawczej, gdzie nadawca może ją wycofać, zanim zostanie dostarczona. 0 wysyła od razu. Domyślnie 10, do 1800 (30 minut). Użytkownicy wybierają własną wartość do poniższego limitu, chyba że to zablokujesz.
admin-settings-key-mail-send-recall-delay-max-seconds = Okno cofnięcia wysłania (limit)
admin-settings-help-mail-send-recall-delay-max-seconds = Najdłuższe okno cofnięcia wysłania, jakie użytkownik może wybrać, w sekundach. Serwer skraca dłuższe żądanie do tej wartości. Domyślnie i maksymalnie 1800.
admin-settings-key-mail-send-allow-recall = Zezwalaj na wycofywanie wysłanych wiadomości
admin-settings-help-mail-send-allow-recall = Gdy wyłączone, wiadomości wychodzą od razu i nie można ich wycofać ze Skrzynki nadawczej. Wysyłki zaplanowane nie są dotknięte; można je nadal anulować do ich czasu.
admin-settings-key-mail-send-max-delayed-seconds = Horyzont planowania
admin-settings-help-mail-send-max-delayed-seconds = Jak daleko w przód można zaplanować wiadomość, w sekundach. Domyślnie 7776000 (90 dni), maksymalnie 31536000 (1 rok). 0 wyłącza wysyłkę zaplanowaną.
admin-settings-key-mail-send-max-queued = Wiadomości w kolejce na skrzynkę
admin-settings-help-mail-send-max-queued = Ile wiadomości jedna skrzynka może mieć jednocześnie oczekujących w Skrzynce nadawczej (okno cofnięcia lub zaplanowane). Domyślnie 200. Kolejne wysyłki są odrzucane z jasnym komunikatem i nigdy nie są gubione.
admin-attachments-sim-pick = Wypełnij na podstawie pliku (opcjonalnie)
admin-attachments-mime-toggle = Pokaż typowe typy MIME
admin-attachments-mime-hint = Wybierz typowy typ lub wpisz dowolny inny typ/podtyp.
admin-attachments-mime-invalid = To nie jest typ/podtyp, taki jak text/plain. Zostanie wysłany tak, jak wpisano.
admin-attachments-mime-group-documents = Dokumenty
admin-attachments-mime-group-images = Obrazy
admin-attachments-mime-group-archives = Archiwa
admin-attachments-mime-group-executables = Pliki wykonywalne i skrypty
admin-attachments-mime-group-media = Dźwięk i wideo
admin-attachments-mime-group-email = E-mail i kalendarz
admin-attachments-mime-group-other = Inne
admin-settings-key-cookies-enabled = Kontrola zgody na pliki cookie
admin-settings-help-cookies-enabled = Pokazuje odwiedzającym kontrolkę zgody na pliki cookie. Wyłączenie zachowuje działanie aplikacji bez tej kontrolki.
admin-settings-key-cookies-mode = Model zgody
admin-settings-help-cookies-mode = optIn: nic opcjonalnego nie jest zapisywane, dopóki odwiedzający nie wyrazi zgody. optOut: obowiązują domyślne ustawienia kategorii, dopóki odwiedzający nie odrzuci. noticeOnly: odwiedzający jest informowany, bez możliwości wyboru.
admin-settings-key-cookies-region-mode = Model zgody w regionach ścisłych
admin-settings-help-cookies-region-mode = Model stosowany dla odwiedzających z regionów ścisłych. same oznacza, że obowiązuje tam także powyższy model zgody.
admin-settings-key-cookies-strict-regions = Regiony ścisłe
admin-settings-help-cookies-strict-regions = Kody krajów ISO 3166-1 stosujące model zgody dla regionów ścisłych. Domyślnie państwa UE i EOG, Wielka Brytania i Szwajcaria.
admin-settings-key-cookies-region-header = Nagłówek kraju
admin-settings-help-cookies-region-header = Nazwa nagłówka żądania ustawianego przez serwer proxy lub CDN, który zawiera kod kraju odwiedzającego.
admin-settings-key-cookies-unknown-region = Nieznany kraj
admin-settings-help-cookies-unknown-region = Który model obowiązuje, gdy nie można ustalić kraju: model regionów ścisłych czy zwykły.
admin-settings-key-cookies-functional-enabled = Oferuj funkcjonalne pliki cookie
admin-settings-help-cookies-functional-enabled = Oferuje w kontrolce zgody kategorię preferencji i funkcji (motyw, język, zapamiętany układ).
admin-settings-key-cookies-functional-default = Domyślnie dla funkcjonalnych
admin-settings-help-cookies-functional-default = Początkowy stan kategorii funkcjonalnej, gdy model zgody dopuszcza wartości domyślne.
admin-settings-key-cookies-analytics-enabled = Oferuj analityczne pliki cookie
admin-settings-help-cookies-analytics-enabled = Oferuje w kontrolce zgody kategorię analityczną.
admin-settings-key-cookies-analytics-default = Domyślnie dla analitycznych
admin-settings-help-cookies-analytics-default = Początkowy stan kategorii analitycznej, gdy model zgody dopuszcza wartości domyślne.
admin-settings-key-cookies-marketing-enabled = Oferuj marketingowe pliki cookie
admin-settings-help-cookies-marketing-enabled = Oferuje w kontrolce zgody kategorię marketingową.
admin-settings-key-cookies-marketing-default = Domyślnie dla marketingowych
admin-settings-help-cookies-marketing-default = Początkowy stan kategorii marketingowej, gdy model zgody dopuszcza wartości domyślne.
admin-settings-key-cookies-layout = Układ
admin-settings-help-cookies-layout = banner to pasek na całej szerokości strony, card to pływająca karta, modal to okno dialogowe blokujące stronę, dopóki odwiedzający nie dokona wyboru.
admin-settings-key-cookies-position = Położenie
admin-settings-help-cookies-position = Gdzie na stronie pojawia się kontrolka zgody.
admin-settings-key-cookies-reject-prominent = Odrzucenie tak samo widoczne jak akceptacja
admin-settings-help-cookies-reject-prominent = Pokazuje Odrzuć wszystkie w tym samym rozmiarze i stylu co Zaakceptuj wszystkie.
admin-settings-key-cookies-policy-url = Adres URL polityki cookie
admin-settings-help-cookies-policy-url = Link do polityki plików cookie, pokazywany w kontrolce zgody.
admin-settings-key-cookies-privacy-url = Adres URL polityki prywatności
admin-settings-help-cookies-privacy-url = Link do polityki prywatności, pokazywany w kontrolce zgody.
admin-settings-key-cookies-policy-version = Wersja polityki
admin-settings-help-cookies-policy-version = Zwiększ ten numer, gdy zmienia się sposób użycia plików cookie, aby odwiedzający zostali zapytani ponownie.
admin-settings-key-cookies-lifetime-days = Czas ważności zgody
admin-settings-help-cookies-lifetime-days = Liczba dni, przez które decyzja pozostaje ważna, zanim odwiedzający zostanie zapytany ponownie.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-cookies-reprompt-on-version = Pytaj ponownie przy nowej wersji polityki
admin-settings-help-cookies-reprompt-on-version = Pyta odwiedzających ponownie, gdy wersja polityki jest wyższa niż ta, przy której podjęli decyzję.
admin-settings-key-cookies-honour-gpc = Respektuj Global Privacy Control
admin-settings-help-cookies-honour-gpc = Traktuje sygnał Global Privacy Control z przeglądarki jako odrzucenie plików analitycznych i marketingowych.
admin-settings-key-cookies-honour-dnt = Respektuj Do Not Track
admin-settings-help-cookies-honour-dnt = Traktuje sygnał Do Not Track z przeglądarki jako odrzucenie plików analitycznych i marketingowych.
admin-settings-key-cookies-log-consent = Prowadź dziennik zgód
admin-settings-help-cookies-log-consent = Przechowuje po stronie serwera zapis każdej decyzji o zgodzie, na potrzeby statystyk i własnej dokumentacji.
admin-settings-key-cookies-log-retention-days = Przechowywanie dziennika zgód
admin-settings-help-cookies-log-retention-days = Liczba dni, przez które wiersze dziennika zgód są przechowywane przed usunięciem.
admin-settings-key-cookies-scan-interval-hours = Odstęp skanowania pamięci
admin-settings-help-cookies-scan-interval-hours = Godziny między automatycznymi skanami plików cookie i pamięci przeglądarki. 0 skanuje tylko na żądanie.
admin-settings-key-cookies-unclassified = Nieskategoryzowana pamięć
admin-settings-help-cookies-unclassified = report pozostawia pamięć, której nic nie klasyfikuje, i wypisuje ją do przeglądu. Pozostałe wartości traktują ją jak daną kategorię, dopóki jej nie sklasyfikujesz.
admin-settings-key-cookies-text-title = Tytuł kontrolki zgody
admin-settings-help-cookies-text-title = Tytuł kontrolki zgody dla każdego języka. Puste używa tekstu wbudowanego.
admin-settings-key-cookies-text-body = Tekst kontrolki zgody
admin-settings-help-cookies-text-body = Tekst objaśniający kontrolki zgody dla każdego języka. Puste używa tekstu wbudowanego.
admin-settings-key-cookies-label-necessary = Nazwa kategorii niezbędnych
admin-settings-help-cookies-label-necessary = Nazwa kategorii niezbędnych dla każdego języka. Puste używa nazwy wbudowanej.
admin-settings-key-cookies-description-necessary = Opis kategorii niezbędnych
admin-settings-help-cookies-description-necessary = Opis kategorii niezbędnych dla każdego języka. Puste używa opisu wbudowanego.
admin-settings-key-cookies-label-functional = Nazwa kategorii funkcjonalnych
admin-settings-help-cookies-label-functional = Nazwa kategorii funkcjonalnych dla każdego języka. Puste używa nazwy wbudowanej.
admin-settings-key-cookies-description-functional = Opis kategorii funkcjonalnych
admin-settings-help-cookies-description-functional = Opis kategorii funkcjonalnych dla każdego języka. Puste używa opisu wbudowanego.
admin-settings-key-cookies-label-analytics = Nazwa kategorii analitycznych
admin-settings-help-cookies-label-analytics = Nazwa kategorii analitycznych dla każdego języka. Puste używa nazwy wbudowanej.
admin-settings-key-cookies-description-analytics = Opis kategorii analitycznych
admin-settings-help-cookies-description-analytics = Opis kategorii analitycznych dla każdego języka. Puste używa opisu wbudowanego.
admin-settings-key-cookies-label-marketing = Nazwa kategorii marketingowych
admin-settings-help-cookies-label-marketing = Nazwa kategorii marketingowych dla każdego języka. Puste używa nazwy wbudowanej.
admin-settings-key-cookies-description-marketing = Opis kategorii marketingowych
admin-settings-help-cookies-description-marketing = Opis kategorii marketingowych dla każdego języka. Puste używa opisu wbudowanego.
admin-settings-key-encryption-rotation-days = Odstęp rotacji klucza bazy danych (dni)
admin-settings-help-encryption-rotation-days = Liczba dni od ostatniej zmiany klucza, po której klucz bazy danych jest zgłaszany jako wymagający rotacji. 0 nie ustawia terminu. Sam klucz nigdy nie jest przechowywany w ustawieniach.
admin-settings-key-encryption-warnings-enabled = Zgłaszaj ustalenia dotyczące szyfrowania
admin-settings-help-encryption-warnings-enabled = Pokazuje magazyny w postaci jawnej i spóźnioną rotację klucza jako ustalenie w ocenie wdrożenia.
admin-settings-key-retention-sweeper-enabled = Czyszczenie retencji
admin-settings-help-retention-sweeper-enabled = Uruchamia codzienne czyszczenie usuwające wiersze starsze niż limity na tej stronie. Wyłączenie zachowuje każdy wiersz do ponownego włączenia.
admin-settings-key-retention-sweeper-batch-rows = Wierszy na partię
admin-settings-help-retention-sweeper-batch-rows = Liczba wierszy usuwanych jednym poleceniem usuwania. Mniejsze partie krócej utrzymują blokady bazy danych.
admin-settings-key-retention-sweeper-max-batches = Partii na magazyn na przebieg
admin-settings-help-retention-sweeper-max-batches = Największa liczba poleceń usuwania, jaką magazyn otrzymuje w jednym przebiegu. Magazyn z większą liczbą starych wierszy kontynuuje w następnym przebiegu.
admin-settings-key-retention-sweeper-jitter-minutes = Rozrzut czasu startu (minuty)
admin-settings-help-retention-sweeper-jitter-minutes = Następny przebieg rozpoczyna się losową liczbę minut, do tej wartości, po 24 godzinach, aby kilka serwerów nie czyściło jednocześnie.
admin-settings-key-retention-sweeper-vacuum-after-rows = Odzyskaj miejsce po (wierszach)
admin-settings-help-retention-sweeper-vacuum-after-rows = Gdy jeden przebieg usunie co najmniej tyle wierszy, baza danych jest kompaktowana (VACUUM). 0 nigdy nie kompaktuje po czyszczeniu; cotygodniowe zadanie vacuum jest oddzielne.
admin-settings-key-retention-password-change-audit-days = Zapisy zmian hasła (dni)
admin-settings-help-retention-password-change-audit-days = Liczba dni, przez które przechowywany jest zapis zmiany hasła (konto, metoda, wynik). 0 przechowuje je bez końca.
admin-settings-key-retention-sso-login-audit-days = Zapisy logowań jednokrotnych (dni)
admin-settings-help-retention-sso-login-audit-days = Liczba dni, przez które przechowywany jest zapis logowania jednokrotnego (dostawca, skrót podmiotu, wynik). 0 przechowuje je bez końca.
admin-settings-key-retention-dlp-audit-days = Zapisy zapobiegania utracie danych (dni)
admin-settings-help-retention-dlp-audit-days = Liczba dni, przez które przechowywany jest zapis dopasowania reguły zapobiegania utracie danych. 0 przechowuje je bez końca.
admin-settings-key-retention-assist-audit-days = Zapisy dostępu asystenta (dni)
admin-settings-help-retention-assist-audit-days = Liczba dni, przez które przechowywany jest zapis użycia przez asystenta jakiejś możliwości (podmiot, możliwość, podsumowanie zakresu). 0 przechowuje je bez końca.
admin-settings-key-retention-alert-events-days = Zapisy wywołanych alertów (dni)
admin-settings-help-retention-alert-events-days = Liczba dni, przez które przechowywany jest zapis wywołanego alertu administratora. 0 przechowuje je bez końca.
admin-settings-key-retention-job-runs-days = Historia zadań w tle (dni)
admin-settings-help-retention-job-runs-days = Liczba dni, przez które przechowywany jest zakończony przebieg zadania w tle. Trwający przebieg nigdy nie jest usuwany. 0 przechowuje historię bez końca.
admin-settings-key-retention-outbox-days = Zapisy wysłanych i anulowanych wysyłek (dni)
admin-settings-help-retention-outbox-days = Liczba dni, przez które przechowywany jest zapis wysyłki dostarczonej lub anulowanej wiadomości. Wiadomość oczekująca, ponawiana, nieudana lub niepewna nigdy nie jest usuwana. 0 przechowuje je bez końca.
admin-settings-key-retention-expired-credentials-grace-days = Wygasłe dane uwierzytelniające (dni po wygaśnięciu)
admin-settings-help-retention-expired-credentials-grace-days = Liczba dni, przez które po wygaśnięciu przechowywane jest wygasłe zapamiętane urządzenie, wyzwanie logowania, token logowania, token OAuth, nonce webhooka, licznik błędów źródła lub uchwyt linku do obrazu. 0 usuwa je w następnym przebiegu.
admin-settings-key-retention-ip-addresses-days = Ogranicz przechowywane adresy IP po (dniach)
admin-settings-help-retention-ip-addresses-days = Po tylu dniach adres IP w historii uwierzytelniania i wpisach audytu jest obcinany lub haszowany, a wiersz pozostaje. 0 zachowuje adresy w zapisanej postaci.
admin-settings-key-retention-ip-addresses-mode = Metoda ograniczania adresów IP
admin-settings-help-retention-ip-addresses-mode = Obcięcie zachowuje prefiks IPv4 /24 lub IPv6 /48. Skrót zastępuje adres krótkim skrótem z kluczem, który nadal grupuje ten sam adres.
admin-settings-key-retention-user-enabled = Użytkownicy mogą ustawiać reguły retencji
admin-settings-help-retention-user-enabled = Użytkownicy mogą ustawiać własne reguły retencji dla folderów i poczty. Gdy wyłączone, istniejące reguły użytkowników przestają działać; reguły ogólnoorganizacyjne poniżej nadal działają.
admin-settings-key-retention-user-min-days = Najmniej dni, zanim reguła użytkownika usunie pocztę
admin-settings-help-retention-user-min-days = Reguła użytkownika usuwająca pocztę nie może działać na poczcie młodszej niż ta wartość. Mniejsza wartość jest odrzucana przy zapisie reguły i podnoszona przy jej uruchomieniu.
admin-settings-key-retention-user-max-days = Najwięcej dni oczekiwania reguły użytkownika przed usunięciem poczty
admin-settings-help-retention-user-max-days = Reguła użytkownika usuwająca pocztę nie może czekać dłużej niż ta wartość. 0 nie ustawia górnej granicy. Większa wartość jest odrzucana przy zapisie reguły i obniżana przy jej uruchomieniu.
admin-settings-key-retention-user-max-rules = Najwięcej reguł retencji na użytkownika
admin-settings-help-retention-user-max-rules = Największa liczba reguł retencji, jaką może mieć jeden użytkownik.
admin-settings-key-retention-user-allow-permanent-delete = Reguły użytkowników mogą trwale usuwać pocztę
admin-settings-help-retention-user-allow-permanent-delete = Gdy wyłączone, reguła usuwania zawsze przenosi pocztę do Kosza, a przed ostatecznym usunięciem obowiązuje okno cofnięcia.
admin-settings-key-retention-user-undo-days = Okno cofnięcia dla reguł retencji (dni)
admin-settings-help-retention-user-undo-days = Liczba dni, przez które poczta przeniesiona do Kosza przez regułę retencji tam pozostaje, zanim zostanie trwale usunięta. Użytkownik może wybrać krótsze okno. 0 usuwa w następnym przebiegu.
admin-settings-key-retention-user-mandatory = Reguły poczty ogólnoorganizacyjne
admin-settings-help-retention-user-mandatory = Reguły, które dostaje każde konto, jako wpisy rola-folderu:dni, na przykład trash:30 lub junk:30. Poczta w tym folderze starsza niż podana liczba dni jest trwale usuwana. Użytkownik nie może ich zmieniać, wstrzymywać ani wyłączać z nich poczty. Poczta konta objętego blokadą prawną jest pomijana.
admin-settings-key-retention-user-protected-folders = Foldery, których reguły retencji nigdy nie dotykają
admin-settings-help-retention-user-protected-folders = Foldery, na których nie działa żadna reguła użytkownika, według roli (drafts, sent, inbox, archive) lub nazwy folderu. Folder chroniony ma pierwszeństwo przed regułą ogólnoorganizacyjną dla tego samego folderu.
admin-settings-key-retention-user-templates = Szablony retencji oferowane użytkownikom
admin-settings-help-retention-user-templates = Szablony, które użytkownicy mogą przyjąć w Ustawieniach, według identyfikatora: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Uruchamiaj reguły użytkowników dla kont z ograniczeniami
admin-settings-help-retention-user-run-for-restricted = Czy reguły retencji użytkowników działają także dla kont objętych ograniczeniem przetwarzania. Wyłączone pomija te konta.
admin-settings-key-retention-user-interval-hours = Godziny między przebiegami retencji użytkowników
admin-settings-help-retention-user-interval-hours = Godziny między zaplanowanymi przebiegami zadania retencji użytkowników dla jednego konta.
admin-settings-key-retention-user-batch-messages = Wiadomości na wywołanie retencji
admin-settings-help-retention-user-batch-messages = Liczba wiadomości obsługiwanych przez jedno wywołanie przeniesienia lub usunięcia. Używanych jest najwyżej 200.
admin-settings-key-retention-user-max-per-run = Najwięcej wiadomości zmienianych na konto na przebieg
admin-settings-help-retention-user-max-per-run = Największa liczba wiadomości, które zadanie retencji użytkowników zmienia dla jednego konta w jednym przebiegu. Reszta czeka na następny przebieg.
admin-settings-key-retention-user-pause-ms = Przerwa między partiami retencji (ms)
admin-settings-help-retention-user-pause-ms = Przerwa między partiami, w milisekundach, aby duże czyszczenie nie obciążało serwera poczty.
admin-settings-key-retention-class-short-days = Górna granica klasy krótkiej (dni)
admin-settings-help-retention-class-short-days = Górna granica dla danych operacyjnych: sesji, tokenów, limitów żądań, historii zadań i metryk. Obowiązuje, gdy limit klasy jest włączony.
admin-settings-key-retention-class-medium-days = Górna granica klasy średniej (dni)
admin-settings-help-retention-class-medium-days = Górna granica dla historii logowania, historii ochrony żądań i powiadomień bezpieczeństwa. Nie może być niższa niż klasa krótka; niższa wartość jest do niej podnoszona.
admin-settings-key-retention-class-long-days = Retencja klasy długiej (dni)
admin-settings-help-retention-class-long-days = Liczba dni, przez które przechowywana jest klasa długa (audyt bezpieczeństwa, działania administratorów, żądania, zgody, nagrobki), wraz z archiwum. 0 przechowuje ją bez końca, a konsola ostrzega. Nie może być niższa niż klasa średnia.
admin-settings-key-retention-class-enforce = Utrzymuj każdy magazyn w granicy jego klasy
admin-settings-help-retention-class-enforce = Gdy włączone, własny limit każdego magazynu jest ograniczany do górnej granicy jego klasy, a 0 (zachowaj) liczy się jako górna granica. Gdy wyłączone, obowiązują limity poszczególnych magazynów.
admin-settings-key-retention-archive-enabled = Archiwizuj wiersze klasy długiej po upływie czasu
admin-settings-help-retention-archive-enabled = Przenosi wiersze klasy długiej, które się zestarzały, do zapieczętowanych, skompresowanych plików archiwum zamiast je usuwać.
admin-settings-key-retention-archive-directory = Katalog archiwum
admin-settings-help-retention-archive-directory = Katalog plików archiwum, bezwzględny lub względem katalogu roboczego serwera. Segmenty katalogu nadrzędnego (..) są odrzucane.
admin-settings-key-retention-archive-batch-rows = Wierszy na plik archiwum
admin-settings-help-retention-archive-batch-rows = Liczba wierszy zapisywanych do jednego pliku archiwum.
admin-settings-key-retention-archive-max-segments = Plików archiwum na magazyn na przebieg
admin-settings-help-retention-archive-max-segments = Liczba plików archiwum, które jeden przebieg może zapisać dla każdego magazynu. Reszta czeka na następny przebieg.
admin-settings-key-retention-archive-key-generation = Generacja klucza pieczętowania archiwum
admin-settings-help-retention-archive-key-generation = Generacja klucza pieczętującego nowe pliki archiwum. Zwiększ ją, aby zrotować klucz; wcześniejsze pliki nadal weryfikują się własną generacją.
admin-settings-key-retention-archive-verify-days = Dni między kontrolami archiwum
admin-settings-help-retention-archive-verify-days = Liczba dni między kontrolami integralności plików archiwum.
admin-settings-key-retention-archive-legal-hold = Blokada prawna archiwum
admin-settings-help-retention-archive-legal-hold = Zachowuje każdy plik archiwum, niezależnie od limitu klasy długiej.
admin-settings-key-logging-file-enabled = Zapisuj pliki dziennika
admin-settings-help-logging-file-enabled = Zapisuje też dziennik serwera do rotowanych plików. Wyłączone ogranicza logowanie do standardowego wyjścia, czego zwykle używa kontener.
admin-settings-key-logging-file-directory = Katalog dzienników
admin-settings-help-logging-file-directory = Katalog plików dziennika, bezwzględny lub względem katalogu roboczego serwera. Segmenty katalogu nadrzędnego (..) są odrzucane.
admin-settings-key-logging-file-max-size-mb = Rotuj przy rozmiarze (MB)
admin-settings-help-logging-file-max-size-mb = Bieżący plik dziennika jest zamykany i rozpoczynany jest nowy, gdy osiągnie ten rozmiar.
admin-settings-key-logging-file-rotate-hours = Rotuj przy wieku (godziny)
admin-settings-help-logging-file-rotate-hours = Bieżący plik dziennika jest rotowany, gdy ma tyle godzin. 0 rotuje tylko według rozmiaru.
admin-settings-key-logging-file-max-age-days = Przechowuj rotowane pliki (dni)
admin-settings-help-logging-file-max-age-days = Rotowane pliki dziennika starsze niż ta wartość są usuwane.
admin-settings-key-logging-file-max-files = Najwięcej rotowanych plików
admin-settings-help-logging-file-max-files = Najstarsze rotowane pliki ponad tę liczbę są usuwane.
admin-settings-key-logging-file-max-total-mb = Największy łączny rozmiar (MB)
admin-settings-help-logging-file-max-total-mb = Najstarsze rotowane pliki są usuwane, aż katalog dzienników zmieści się poniżej tego rozmiaru.
admin-settings-key-logging-file-compress = Kompresuj rotowane pliki
admin-settings-help-logging-file-compress = Rotowane pliki dziennika są kompresowane za pomocą gzip.
admin-settings-key-logging-level = Poziom dziennika
admin-settings-help-logging-level = Poziom dla komponentów, które nie mają poniżej własnego poziomu. Zmienna środowiskowa MW_LOG ustawia poziom przy starcie, dopóki to ustawienie nie zostanie odczytane.
admin-settings-key-logging-component-levels = Poziomy dziennika komponentów
admin-settings-help-logging-component-levels = Jeden wpis komponent=poziom w wierszu, na przykład mw_imap=debug. Poziomy to error, warn, info, debug i trace.
admin-settings-key-logging-redaction-addresses = Adresy e-mail w dziennikach
admin-settings-help-logging-redaction-addresses = Full pozostawia adresy bez zmian. Truncated zachowuje pierwszą literę i domenę. Hashed zastępuje część lokalną krótkim skrótem. None usuwa adres.
admin-settings-key-logging-redaction-ips = Adresy IP w dziennikach
admin-settings-help-logging-redaction-ips = Full pozostawia adresy bez zmian. Truncated zachowuje prefiks IPv4 /24 lub IPv6 /48. Hashed zastępuje adres krótkim skrótem. None go usuwa.
admin-settings-key-logging-redaction-tokens = Sekrety w dziennikach
admin-settings-help-logging-redaction-tokens = Wycina z wierszy dziennika tokeny bearer, klucze API, hasła w adresach URL i podobne sekrety.
admin-settings-key-retention-sessions-idle-days = Nieużywane sesje (dni)
admin-settings-help-retention-sessions-idle-days = Liczba dni, przez które zapis sesji przeglądarki lub klienta natywnego, z jej adresem i user-agent, może pozostać nieużywany, zanim zostanie usunięty. Najpierw obowiązują limity czasu sesji; to usuwa tylko pozostawione wiersze. 0 je zachowuje.
admin-settings-key-policy-public-pages = Publiczne strony polityk
admin-settings-help-policy-public-pages = Udostępnia wygenerowaną politykę prywatności pod /privacy i politykę plików cookie pod /cookies-policy bez logowania.
admin-settings-key-policy-auto-version = Automatyczne wersje polityk
admin-settings-help-policy-auto-version = Zapisuje nową wersję polityki, gdy zmienia się ustawienie opisywane przez politykę. Wyłączone: zmiany czekają na Opublikuj.
admin-settings-key-policy-reconsent-on-change = Pytaj ponownie o zgodę na cookie przy nowej wersji polityki
admin-settings-help-policy-reconsent-on-change = Podnosi wersję polityki cookie, gdy zapisywana jest nowa wersja polityki, aby odwiedzający zostali zapytani ponownie (wymaga cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Nazwa administratora danych
admin-settings-help-policy-controller-name = Nazwa organizacji, która decyduje o celach i sposobach przetwarzania danych, pokazywana w polityce prywatności.
admin-settings-key-policy-controller-address = Adres administratora danych
admin-settings-help-policy-controller-address = Adres pocztowy administratora danych, pokazywany w polityce prywatności.
admin-settings-key-policy-legal-bases = Podstawy prawne
admin-settings-help-policy-legal-bases = Podstawy prawne, na których się opierasz, własnymi słowami, pokazywane w polityce prywatności.
admin-settings-key-policy-hosting-provider = Dostawca hostingu
admin-settings-help-policy-hosting-provider = Kto hostuje dane i gdzie, pokazywane w polityce prywatności.
admin-settings-key-policy-hosting-transfers = Transfery poza kraj hostingu
admin-settings-help-policy-hosting-transfers = Wszelkie transfery danych do innych krajów i stosowane zabezpieczenia, pokazywane w polityce prywatności.
admin-settings-key-policy-supervisory-authority = Organ nadzorczy
admin-settings-help-policy-supervisory-authority = Organ, do którego użytkownicy mogą składać skargi, pokazywany w polityce prywatności.
admin-settings-key-dpia-suggest = Sugeruj oceny skutków
admin-settings-help-dpia-suggest = Wypisuje sugerowaną ocenę skutków, gdy asystent, śledzenie otwarć lub analityka jest włączone, a oceny nie ma.
admin-settings-key-dpia-review-months = Odstęp przeglądu oceny skutków
admin-settings-help-dpia-review-months = Liczba miesięcy do daty przeglądu ustawianej przy zatwierdzeniu oceny.
admin-settings-key-dpia-reminder-days = Okno przypomnienia o ocenie skutków
admin-settings-help-dpia-reminder-days = Liczba dni przed datą przeglądu, od której ocena jest wymieniana jako wymagająca przeglądu.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-restriction-enabled = Ograniczenia przetwarzania
admin-settings-help-restriction-enabled = Czy administrator lub zatwierdzony wniosek może ograniczyć przetwarzanie danych osoby.
admin-settings-key-restriction-auto-apply-on-approval = Ogranicz po zatwierdzeniu wniosku
admin-settings-help-restriction-auto-apply-on-approval = Stosuje ograniczenie od razu po zatwierdzeniu wniosku o ograniczenie. Wyłączone: administrator stosuje je w Zgodność > Ograniczenia.
admin-settings-key-restriction-mail-delivery = Nowa poczta podczas ograniczenia
admin-settings-help-restriction-mail-delivery = Co administrator wybrał dla poczty przychodzącej w czasie obowiązywania ograniczenia: zachować ją, umieścić w kolejce lub odrzucić. Ten serwer nie odbiera poczty; wartość jest zapisywana i pokazywana, a działa według niej serwer poczty.
admin-settings-key-restriction-default-expiry-days = Domyślny czas ograniczenia (dni)
admin-settings-help-restriction-default-expiry-days = Liczba dni, po których nowe ograniczenie kończy się samo. 0 utrzymuje je, dopóki ktoś go nie zdejmie.
admin-settings-key-restriction-notify-user = Powiadom osobę
admin-settings-help-restriction-notify-user = Wysyła osobie powiadomienie, gdy ograniczenie jest stosowane lub zdejmowane.
admin-settings-key-restriction-scope-indexing = Wstrzymaj indeksowanie wyszukiwania i osadzenia
admin-settings-help-restriction-scope-indexing = Czy indeksowanie wyszukiwania i osadzenia są domyślnie wstrzymane w nowym ograniczeniu.
admin-settings-key-restriction-scope-assistant = Wstrzymaj asystenta
admin-settings-help-restriction-scope-assistant = Czy asystent jest domyślnie wstrzymany w nowym ograniczeniu.
admin-settings-key-restriction-scope-analytics = Wstrzymaj analitykę i śledzenie otwarć
admin-settings-help-restriction-scope-analytics = Czy analityka i śledzenie otwarć są domyślnie wstrzymane w nowym ograniczeniu.
admin-settings-key-restriction-scope-classification = Wstrzymaj reguły i automatyczne porządkowanie
admin-settings-help-restriction-scope-classification = Czy reguły i automatyczne porządkowanie są domyślnie wstrzymane w nowym ograniczeniu.
admin-settings-key-restriction-scope-push = Wstrzymaj powiadomienia push
admin-settings-help-restriction-scope-push = Czy powiadomienia push są domyślnie wstrzymane w nowym ograniczeniu.
admin-settings-key-restriction-scope-third-party = Wstrzymaj połączone usługi i wtyczki
admin-settings-help-restriction-scope-third-party = Czy połączone usługi i wtyczki są domyślnie wstrzymane w nowym ograniczeniu.
admin-settings-key-consent-enabled = Zgoda na cele przetwarzania
admin-settings-help-consent-enabled = Pokazuje użytkownikom cele i wybory zgody oraz egzekwuje je w czasie działania.
admin-settings-key-consent-receipts-retention-days = Przechowywanie potwierdzeń zgody (dni)
admin-settings-help-consent-receipts-retention-days = Liczba dni, przez które potwierdzenia zgody są przechowywane. 0 przechowuje je do usunięcia danych osoby.
admin-settings-key-consent-art22-notice = Informacja o zautomatyzowanym przetwarzaniu
admin-settings-help-consent-art22-notice = Pokazuje użytkownikom prostą informację o funkcjach asystenta i wyszukiwania wraz z przełącznikami dla poszczególnych funkcji.
admin-settings-key-consent-analytics-basis = Podstawa prawna: liczniki użycia
admin-settings-help-consent-analytics-basis = Podstawa prawna pokazywana użytkownikom dla liczników użycia. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-analytics-version = Wersja tekstu: liczniki użycia
admin-settings-help-consent-analytics-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-analytics-text = Własne sformułowanie: liczniki użycia
admin-settings-help-consent-analytics-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-analytics-tag-basis = Podstawa prawna: skrypt dostawcy analityki
admin-settings-help-consent-analytics-tag-basis = Podstawa prawna pokazywana użytkownikom dla skryptu dostawcy analityki. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-analytics-tag-version = Wersja tekstu: skrypt dostawcy analityki
admin-settings-help-consent-analytics-tag-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-analytics-tag-text = Własne sformułowanie: skrypt dostawcy analityki
admin-settings-help-consent-analytics-tag-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-assistant-basis = Podstawa prawna: asystent
admin-settings-help-consent-assistant-basis = Podstawa prawna pokazywana użytkownikom dla asystenta. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-assistant-version = Wersja tekstu: asystent
admin-settings-help-consent-assistant-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-assistant-text = Własne sformułowanie: asystent
admin-settings-help-consent-assistant-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-push-basis = Podstawa prawna: powiadomienia push
admin-settings-help-consent-push-basis = Podstawa prawna pokazywana użytkownikom dla powiadomień push. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-push-version = Wersja tekstu: powiadomienia push
admin-settings-help-consent-push-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-push-text = Własne sformułowanie: powiadomienia push
admin-settings-help-consent-push-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-open-tracking-basis = Podstawa prawna: śledzenie otwarć
admin-settings-help-consent-open-tracking-basis = Podstawa prawna pokazywana użytkownikom dla śledzenia otwarć. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-open-tracking-version = Wersja tekstu: śledzenie otwarć
admin-settings-help-consent-open-tracking-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-open-tracking-text = Własne sformułowanie: śledzenie otwarć
admin-settings-help-consent-open-tracking-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-remote-content-basis = Podstawa prawna: obrazy zdalne
admin-settings-help-consent-remote-content-basis = Podstawa prawna pokazywana użytkownikom dla obrazów zdalnych. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-remote-content-version = Wersja tekstu: obrazy zdalne
admin-settings-help-consent-remote-content-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-remote-content-text = Własne sformułowanie: obrazy zdalne
admin-settings-help-consent-remote-content-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-integrations-basis = Podstawa prawna: połączone usługi
admin-settings-help-consent-integrations-basis = Podstawa prawna pokazywana użytkownikom dla połączonych usług. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-integrations-version = Wersja tekstu: połączone usługi
admin-settings-help-consent-integrations-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-integrations-text = Własne sformułowanie: połączone usługi
admin-settings-help-consent-integrations-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-consent-telemetry-basis = Podstawa prawna: pomiary czasu ładowania
admin-settings-help-consent-telemetry-basis = Podstawa prawna pokazywana użytkownikom dla pomiarów czasu ładowania. Tylko zgoda sprawia, że funkcja czeka na „tak”; pozostałe podstawy obowiązują, dopóki osoba ich nie wycofa. O właściwej podstawie decyduje administrator.
admin-settings-key-consent-telemetry-version = Wersja tekstu: pomiary czasu ładowania
admin-settings-help-consent-telemetry-version = Zwiększ ten numer, gdy zmienia się sformułowanie lub przetwarzanie; osoby, które zgodziły się na starszą wersję, zostaną zapytane ponownie.
admin-settings-key-consent-telemetry-text = Własne sformułowanie: pomiary czasu ładowania
admin-settings-help-consent-telemetry-text = Własny tekst dla tego celu, dla każdego języka. Puste używa sformułowania wbudowanego.
admin-settings-key-breach-enabled = Rejestr naruszeń
admin-settings-help-breach-enabled = Włącza rejestr i odliczanie czasu na zgłoszenie.
admin-settings-key-breach-notice-hours = Godziny na zgłoszenie organowi
admin-settings-help-breach-notice-hours = Liczba godzin od stwierdzenia naruszenia do zgłoszenia organowi. Zwykle jest to 72.
admin-settings-key-breach-reminders-hours = Czasy przypomnień
admin-settings-help-breach-reminders-hours = Liczba godzin przed terminem, które wywołują przypomnienie, na przykład 6, 24 i 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Powtarzanie po terminie
admin-settings-help-breach-reminders-overdue-repeat-hours = Godziny między przypomnieniami po upływie terminu. 0 przypomina raz.
admin-settings-key-breach-authority-name = Organ nadzorczy
admin-settings-help-breach-authority-name = Nazwa umieszczana w zgłoszeniu do organu.
admin-settings-key-breach-authority-url = Strona zgłoszeń organu
admin-settings-help-breach-authority-url = Adres strony zgłoszeń organu.
admin-settings-key-breach-authority-contact = Kontakt do organu
admin-settings-help-breach-authority-contact = Dane kontaktowe organu.
admin-settings-key-breach-dpo-name = Imię i nazwisko kontaktu ds. ochrony danych
admin-settings-help-breach-dpo-name = Imię i nazwisko podawane jako punkt kontaktowy w zgłoszeniach.
admin-settings-key-breach-dpo-email = E-mail kontaktu ds. ochrony danych
admin-settings-help-breach-dpo-email = Adres podawany jako punkt kontaktowy w zgłoszeniach.
admin-settings-key-breach-dpo-phone = Telefon kontaktu ds. ochrony danych
admin-settings-help-breach-dpo-phone = Numer telefonu podawany w zgłoszeniach.
admin-settings-key-breach-template-subject = Temat wiadomości
admin-settings-help-breach-template-subject = Temat wiadomości do osób, których naruszenie dotyczy, dla każdego języka.
admin-settings-key-breach-template-body = Treść wiadomości
admin-settings-help-breach-template-body = Treść wiadomości do osób, których naruszenie dotyczy, dla każdego języka.
admin-settings-key-breach-notify-audit = Zapisuj powiadomienia w dzienniku audytu
admin-settings-help-breach-notify-audit = Przypomnienia i wykrycia stają się zdarzeniami bezpieczeństwa.
admin-settings-key-breach-notify-webhook-urls = Adresy webhooków
admin-settings-help-breach-notify-webhook-urls = Adresy informowane o przypomnieniach i wiadomościach. Traktuj je jak sekrety; każde wywołanie jest podpisane.
admin-settings-key-breach-notify-operator-addresses = Skrzynki operatorów
admin-settings-help-breach-notify-operator-addresses = Lokalne adresy skrzynek, które otrzymują przypomnienia we własnej skrzynce odbiorczej.
admin-settings-key-breach-subjects-in-app = Powiadomienie w koncie osoby
admin-settings-help-breach-subjects-in-app = Pokazuje wiadomość w koncie osoby.
admin-settings-key-breach-subjects-inbox = Wiadomość do skrzynki osoby
admin-settings-help-breach-subjects-inbox = Dostarcza wiadomość do własnej skrzynki osoby.
admin-settings-key-breach-subjects-webhook = Wyślij także do webhooków
admin-settings-help-breach-subjects-webhook = Wysyła treść wiadomości także na adresy webhooków.
admin-settings-key-breach-retention-years = Przechowywanie rejestru w latach
admin-settings-help-breach-retention-years = Liczba lat, przez które zamknięty incydent pozostaje w rejestrze.
admin-settings-key-breach-auto-enabled = Automatyzacje
admin-settings-help-breach-auto-enabled = Otwiera szkice incydentów na podstawie wykryć.
admin-settings-key-breach-auto-start-clock = Odliczanie dla szkicu
admin-settings-help-breach-auto-start-clock = Uruchamia odliczanie przy otwarciu szkicu lub dopiero po jego potwierdzeniu.
admin-settings-key-breach-auto-assignee = Przypisuj szkice do
admin-settings-help-breach-auto-assignee = Kto otrzymuje szkic.
admin-settings-key-breach-auto-escalate-after-hours = Eskaluj po godzinach
admin-settings-help-breach-auto-escalate-after-hours = Liczba godzin, przez które szkic czeka, zanim zostanie eskalowany. 0 nigdy nie eskaluje.
admin-settings-key-breach-auto-escalate-to = Eskaluj do
admin-settings-help-breach-auto-escalate-to = Kto otrzymuje eskalowany szkic.
admin-settings-key-breach-auto-window-minutes = Okno zliczania w minutach
admin-settings-help-breach-auto-window-minutes = Liczba minut, w których reguła zlicza zdarzenia.
admin-settings-key-breach-auto-theft-threshold = Alerty o kradzieży sesji
admin-settings-help-breach-auto-theft-threshold = Liczba alertów w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spray lub skok logowań
admin-settings-help-breach-auto-credential-stuffing-threshold = Liczba zdarzeń w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-mass-forward-threshold = Masowe reguły przekazywania
admin-settings-help-breach-auto-mass-forward-threshold = Liczba zdarzeń w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-role-escalation-threshold = Zmiany ról administratora
admin-settings-help-breach-auto-role-escalation-threshold = Liczba zmian w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-export-volume-threshold = Nietypowe eksporty
admin-settings-help-breach-auto-export-volume-threshold = Liczba eksportów w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-audit-chain-threshold = Awarie łańcucha audytu
admin-settings-help-breach-auto-audit-chain-threshold = Liczba awarii, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-backup-anomaly-threshold = Awarie kopii zapasowych lub przywracania
admin-settings-help-breach-auto-backup-anomaly-threshold = Liczba awarii w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-breach-auto-dsr-export-threshold = Wiele eksportów wniosków o dane
admin-settings-help-breach-auto-dsr-export-threshold = Liczba eksportów w oknie, która otwiera szkic. 0 wyłącza.
admin-settings-key-ratelimit-queue-ms = Najdłuższe oczekiwanie na wolne miejsce
admin-settings-help-ratelimit-queue-ms = Najdłużej, jak żądanie czeka na wolne miejsce (współbieżność JMAP, dekodery obrazów, procesy podglądu), zanim zostanie odrzucone z kodem 429 i Retry-After. Zakres 0–30000 ms; domyślnie 3000. Zero odrzuca od razu.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Równoczesne żądania JMAP na konto
admin-settings-help-ratelimit-jmap-per-account-concurrent = Liczba żądań JMAP, które jedno zalogowane konto może wykonywać jednocześnie, we wszystkich kartach i urządzeniach. Nigdy nie jest dzielona z innym kontem ani innym użytkownikiem za tym samym adresem. Zakres 4–64; domyślnie 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Równoczesne żądania JMAP, wszystkie konta
admin-settings-help-ratelimit-jmap-global-concurrent = Liczba żądań JMAP wykonywanych jednocześnie przez cały proces serwera, wszystkie konta łącznie. Żądanie ponad ten limit czeka przez czas kolejki. Zakres 4–4096; domyślnie 256.
admin-settings-key-ratelimit-session-burst = Raporty czasu na sesję z rzędu
admin-settings-help-ratelimit-session-burst = Liczba raportów czasu klienta, które jedna sesja może wysłać z rzędu. Punkt końcowy jest kluczowany sesją, nigdy adresem. Zakres 1–1000; domyślnie 60.
admin-settings-key-ratelimit-session-per-minute = Raporty czasu odzyskiwane na minutę
admin-settings-help-ratelimit-session-per-minute = Liczba raportów czasu klienta na minutę, które sesja odzyskuje po wyczerpaniu serii. Zakres 1–600; domyślnie 30.
admin-settings-key-ratelimit-anon-burst = Anonimowe żądania zgody z rzędu
admin-settings-help-ratelimit-anon-burst = Liczba żądań, które jeden adres może wysłać z rzędu do anonimowych tras zgody na cookie. Dobrana dla biura za jednym adresem. Zakres 1–1000; domyślnie 60.
admin-settings-key-ratelimit-anon-per-minute = Anonimowe żądania zgody odzyskiwane na minutę
admin-settings-help-ratelimit-anon-per-minute = Liczba żądań na minutę, które adres odzyskuje na anonimowych trasach zgody na cookie. Zakres 1–600; domyślnie 30.
admin-settings-key-ratelimit-trusted-multiplier = Limit konta ze znanego źródła (%)
admin-settings-help-ratelimit-trusted-multiplier = Procent limitu żądań konta, który otrzymuje zalogowana skrzynka, gdy żądanie pochodzi ze źródła z odnotowanym udanym logowaniem. 100 zachowuje limit bazowy. Nigdy nie łagodzi budżetów anonimowych, administratora ani user-agent, ani aktywnej blokady. Zakres 100–1000; domyślnie 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Określa, czy kliknięcie w dowolnym miejscu paska załączników czytnika powoduje jego rozwinięcie czy zwinięcie. Gdy jest wyłączony, działa tylko przycisk strzałki.

admin-settings-help-attachments-bar-download-all = Czy na pasku załączników znajduje się przycisk Pobierz wszystko, który zapisuje dozwolone załączniki w jednym pliku ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Gdy każdy załącznik na pasku wyświetla przycisk pobierania: po najechaniu kursorem lub zaznaczeniu (domyślnie), zawsze lub nigdy. Pobieranie nadal podlega zasadom załączników i mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Czy obrazy pokazane w tekście wiadomości są uwzględnione w opcji Pobierz wszystko ZIP. Domyślnie wyłączone.

admin-settings-help-cap-reading-delete-sender = Zezwalaj użytkownikom na włączenie akcji czytnika, która po potwierdzeniu przenosi wszystkie pasujące wiadomości nadawców do Kosza. Domyślnie wyłączone. Profil funkcji również musi na to pozwalać.

admin-settings-help-compose-feature-family = Włącz lub wyłącz kontrolę kompozytora za pomocą identyfikatora funkcji. Domyślnie włączone; zablokuj go, aby uniemożliwić użytkownikom jego zmianę.

admin-settings-help-compose-headers-allow-auto-submitted = Umożliwia użytkownikom ustawienie opcji Automatyczne przesyłanie (RFC 3834) w wiadomości. Domyślnie wyłączone: informuje inne systemy, aby nie odpowiadały.

admin-settings-help-compose-headers-allow-custom = Umożliwia użytkownikom dodawanie własnych nagłówków (do 20, sprawdzane są nazwy, odrzucane są chronione nagłówki).

admin-settings-help-compose-headers-allow-return-receipt-to = Umożliwia użytkownikom żądanie potwierdzeń za pomocą starszego nagłówka Return-Receipt-To. Domyślnie wyłączone.

admin-settings-help-compose-headers-allow-threading-edit = Umożliwia użytkownikom edycję odpowiedzi w odpowiedzi i odwołań lub rozpoczęcie nowego wątku na podstawie odpowiedzi.

admin-settings-help-compose-headers-forced = Jedna para Nazwa=wartość w każdym wierszu, dodawana do każdej wiadomości wysyłanej z domeny i zastępująca wartość użytkownika. Chronione nagłówki są odrzucane.

admin-settings-help-compose-insert-image-resize-default = Rozmiar okna dialogowego Obraz z pliku rozpoczyna się od: oryginalny, duży (1600 px) lub średni (1024 px).

admin-settings-help-compose-insert-max-inline-image-mb = Większe obrazy nie są umieszczane w treści wiadomości; nadal można je przymocować. 1 do 50.

admin-settings-help-compose-layout-default-font = Rodzina czcionek, rozmiar i kolor, od którego rozpoczyna się nowa wiadomość, jako rodzina|rozmiar|#kolor. Pusty używa czcionki motywu.

admin-settings-help-compose-reply-to-allow-override = Gdy ta opcja jest wyłączona, opcja Reply-To tożsamości jest używana w niezmienionej postaci i użytkownicy nie mogą jej edytować ani usunąć.

admin-settings-help-compose-reply-to-show = Określa, czy kompozytor wyświetla wiersz „Odpowiedz do”: tylko wtedy, gdy tożsamość lub oryginalna wiadomość go zawiera, zawsze lub nigdy.

admin-settings-help-compose-sent-folder-fixed-id = Rola lub identyfikator folderu używany, gdy regułą jest „folder stały”.

admin-settings-help-compose-sent-folder-mode = Gdzie trafia kopia wysłanej wiadomości: zwykły folder Wysłane, jeden stały folder, według domeny odbiorcy, tożsamości lub reguł tematu.

admin-settings-help-compose-sent-folder-per-identity = Jedna para tożsamość=folder w każdym wierszu. Używane, gdy regułą jest „według tożsamości”.

admin-settings-help-compose-sent-folder-rules = Jedna reguła w linii, domain:example.com=folder lub subject:text=folder. Pierwszy mecz wygrywa.

admin-settings-help-compose-sent-folder-skip-copy = Nigdy nie przechowuj kopii wysłanej wiadomości. Administrator może wymusić włączenie lub wyłączenie tej opcji.

admin-settings-help-compose-styles-custom = Własne style kompozytora użytkownika to JSON, napisane przez galerię stylów. Użytkownicy to ustawiają; tu nie ma co konfigurować.

admin-settings-help-compose-styles-default-set = Wygląd galerii stylów kompozytora: domyślny, nowoczesny, klasyczny lub kompaktowy.

admin-settings-help-compose-styles-max-custom = Większość niestandardowych stylów, które użytkownik może zachować w kompozytorze. 0 wyłącza opcję Utwórz styl.

admin-settings-help-contacts-save-from-message-add-note = Dodaj notatkę „Zapisano z wiadomości w dniu <data>” do kontaktów zapisanych z wiadomości. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-contacts-save-from-message-default-book = Identyfikator książki adresowej, do której trafiają kontakty zapisane z wiadomości; „domyślny” używa domyślnej książki adresowej. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-contacts-save-from-message-default-scope = Którzy uczestnicy są zaznaczani jako pierwsi w oknie dialogowym Zapisz kontakty: tylko nadawca, nadawca i Odpowiedz do czy wszyscy. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-contacts-save-from-message-enabled = Oferta Zapisz kontakty z tej wiadomości w menu wiadomości. Gdy opcja jest wyłączona, element jest ukryty. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-contacts-save-from-message-skip-automated = Pozostaw adresy „brak odpowiedzi”, „powiadomienia” i „odsyłanie” poza oknem dialogowym Zapisz kontakty, chyba że zaznaczono je ręcznie. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-contacts-save-sender-enabled = Zaoferuj opcję Zapisz nadawcę jako element kontaktu jednym kliknięciem w menu wiadomości. Gdy opcja jest wyłączona, element jest ukryty. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-contacts-save-sender-use-reply-to = Zapisz adres zwrotny do zamiast Od, jeśli nadawcą jest adres listy mailingowej lub adres powiadomienia. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-dsr-admin-step-up = Włączone (domyślnie): administrator, który zatwierdza żądanie danych, przeprowadza usuwanie lub zatwierdza zbiorczo, musi najpierw ponownie potwierdzić, że to on (hasło i drugi czynnik lub klucz dostępu). Potwierdzenie trwa pięć minut. Off usuwa dodatkową kontrolę; decyzja jest nadal zapisywana w ścieżce żądania.

admin-settings-help-export-default-format = Format, od którego rozpoczyna się okno dialogowe eksportu.

admin-settings-help-export-default-scope = Zakres, od którego rozpoczyna się okno eksportu: wiadomość, rozmowa, wybór lub folder.

admin-settings-help-export-dialog-enabled = Eksportuj w widoku wiadomości, a menu wiadomości otwiera okno dialogowe. Gdy jest wyłączone, te menu pobierają jedną wiadomość w formacie .eml.

admin-settings-help-export-folder-enabled = Zezwalaj użytkownikom na pobieranie całego folderu jako ZIP wiadomości. Każdy eksport jest audytowany.

admin-settings-help-export-folder-format = Formaty oferowane przez eksport folderów: ZIP plików .eml, ZIP zawierający plik mbox lub oba.

admin-settings-help-export-folder-include-subfolders = Określa, czy eksport folderu domyślnie obejmuje jego podfoldery, przechowywane jako struktura folderów w pliku ZIP.

admin-settings-help-export-folder-max-bytes = Najwięcej bajtów wiadomości, jakie może pomieścić eksport w jednym folderze. Większy folder o pokazanym rozmiarze jest odrzucany.

admin-settings-help-export-folder-max-messages = Najwięcej wiadomości, jakie może pomieścić eksport w jednym folderze.

admin-settings-help-export-formats = Formaty oferowane w oknie eksportu: eml, mbox, txt, md, msg, docx, csv, json, pdf. Serwer odrzuca wyszczególniony format zbudowany na serwerze.

admin-settings-help-export-message-download-enabled = Zezwalaj użytkownikom na pobieranie wiadomości w formacie .eml, samych nagłówków lub jako plik mbox.

admin-settings-help-folders-hide-empty = Ukryj puste foldery na pasku bocznym. Użytkownicy mogą zmienić ustawienie domyślne lub zastąpić je dla poszczególnych folderów, chyba że to ustawienie jest zablokowane. Żadna poczta nie jest usuwana.

admin-settings-help-folders-pinned-hide-from-tree = Przypięte foldery są wyświetlane w grupie Przypięte nad drzewem folderów i również zachowują swoje miejsce w drzewie. Włącz ukrywa przypięty folder, który nie ma podfolderów w drzewie, więc jest widoczny tylko w grupie Przypięte. Przypięty folder z podfolderami pozostaje w drzewie.

admin-settings-help-mail-badge-auth = Gdy w nagłówku czytnika znajduje się identyfikator uwierzytelniający DMARC: zawsze, tylko wtedy, gdy jest to godne uwagi, lub nigdy.

admin-settings-help-mail-badge-density = Rozmiar identyfikatorów w nagłówku czytnika: kompaktowy lub wygodny.

admin-settings-help-mail-badge-folder = Gdy nagłówek czytnika pokazuje plakietkę folderu: zawsze, tylko wtedy, gdy jest to godne uwagi, lub nigdy.

admin-settings-help-mail-badge-headers = Pokaż w czytniku plakietkę dla nagłówków Wygasa, Odpowiedź przez, Wrażliwość i Zastępuje nagłówki wiadomości.

admin-settings-help-mail-badge-labels = Określa, czy plakietki nagłówka czytnika pokazują ikonę z tekstem, czy tylko ikonę.

admin-settings-help-mail-badge-list = Kiedy w nagłówku czytnika znajduje się odznaka listy mailingowej: zawsze, tylko wtedy, gdy jest to godne uwagi, lub nigdy. Potrzebuje nagłówka List-Id lub List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Gdy w nagłówku czytnika znajduje się wyczyszczona plakietka HTML: zawsze, tylko wtedy, gdy jest to godne uwagi, lub nigdy.

admin-settings-help-mail-badge-spam = Gdy w nagłówku czytnika znajduje się odznaka oceny spamu: zawsze, tylko wtedy, gdy jest zauważalna lub nigdy.

admin-settings-help-mail-brand-avatars-require-auth = nigdy: wyświetlaj logo za każdym razem, gdy domena pasuje. znany błąd (domyślnie): pokaż go, chyba że wynik wiadomości w pamięci podręcznej DKIM lub DMARC nie zakończy się niepowodzeniem; niesprawdzone wiadomości zachowują logo. zawsze: pokazuj to dopiero po dopasowaniu DMARC z DKIM dla własnej domeny marki. Zastąpienia użytkownika i domeny można zablokować.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Sekundy, w których treść wiadomości musi być skupiona, zanim cały nagłówek odbiorcy i tematu złoży się w podsumowanie. Wybierz 0, aby wyłączyć składanie czasowe lub 1–60 sekund. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-mail-compose-priority-default = Priorytet, od którego rozpoczyna się nowa wiadomość w narzędziu Composer.

admin-settings-help-mail-compose-priority-enabled = Pokaż selektor priorytetu Normalny, Wysoki i Niski w grupie Opcje kompozytora. Gdy opcja ta jest wyłączona, nowa poczta jest wysyłana bez nagłówków priorytetowych.

admin-settings-help-mail-expiry-hide-expired = Co zrobić z wiadomością, której data ważności minęła: nic, pokaż ją przyciemnioną lub zasugeruj jej archiwizację.

admin-settings-help-mail-list-keep = Nadawcy i listy mailingowe są zawsze zachowywane, po jednym w wierszu jako sender:address lub list:List-Id. Wyczyść tego nadawcę, pomiń je i pomiń sugestie anulowania subskrypcji oraz reguły przechowywania.

admin-settings-help-mail-list-page-size = Liczba wiadomości pobieranych przez listę za każdym razem, gdy użytkownik przewinie ją do końca. Domyślnie 50, od 20 do 200. Użytkownicy wybierają własną wartość, chyba że ją zablokujesz.

admin-settings-help-mail-list-unsubscribe-mode = Jak działa opcja anulowania subskrypcji, gdy lista oferuje żądanie jednym kliknięciem: najpierw zapytaj, wyślij je od razu lub otwórz tylko łącze lub wersję roboczą listy.

admin-settings-help-mail-message-list-group = Grupuj rozmowy według daty otrzymania lub wysłania, nadawcy, tematu, ważności lub rozmiaru. Użytkownicy mogą to zmienić, chyba że są zablokowane.

admin-settings-help-mail-priority-marks = Jaki priorytet wyznacza lista wiadomości, rozmowy i pokaz czytelnika: zarówno wysoki, jak i niski, tylko wysoki lub żaden. Użytkownicy wybierają w Ustawieniach > Foldery i pasek boczny, chyba że są zablokowane.

admin-settings-help-mail-priority-marks-style = Jak wygląda znak priorytetu: ikona, ikona ze słowami Wysoki lub Niski lub cienki pasek. Ikona i tekst zachowują znaczenie bez koloru.

admin-settings-help-mail-priority-send-headers = Jakie nagłówki otrzymuje wiadomość o wysokim lub niskim priorytecie po wysłaniu: dowolne spośród: Ważność, X-Priority i X-MSMail-Priority.

admin-settings-help-mail-priority-trust-sender-headers = Określa, czy wyświetlany jest priorytet ustawiony przez nadawcę: zawsze, tylko wtedy, gdy wiadomość przeszła pomyślnie test DMARC, lub nigdy. Użyj opcji Zaufane lub Nigdy w przypadku nadawców, którzy nadają każdej wiadomości wysoki priorytet.

admin-settings-help-mail-reply-from-header-priority = Nagłówki wiadomości są czytane w tej kolejności, aby znaleźć adres, na który wiadomość została wysłana podczas wybierania tożsamości w odpowiedzi. Nazwy: Dostarczono do, X-oryginał do, koperta do, X-przekazano do, ponownie wysłano do, do, DW. Pominięty nagłówek nie jest czytany.

admin-settings-help-mail-search-default-scope = Rozpocznij nowe, normalne wyszukiwanie poczty w bieżącym folderze lub we wszystkich folderach. Kontroluje to początkowe ustawienia domyślne, w tym stany zablokowania; użytkownicy nadal mogą wybrać inny zakres dla konkretnego wyszukiwania. Zapisane i jawnie określone zapytania są zachowywane.

admin-settings-help-metrics-dimensions-enabled = Przechowuj zapisane statystyki w podziale na rodzaj klienta (internet) i klasę urządzenia (komputer stacjonarny, mobilny). Wyłącz, aby zapisać jeden połączony wiersz na metrykę i minutę.

admin-settings-help-metrics-persist-enabled = Zapisuj statystyki wydajności (szybkość proxy obrazu, czas ładowania przeglądarki, licznik szybkości i ochrony żądań) w bazie danych przy każdym interwale opróżniania i zatrzymaniu serwera, aby wykresy zachowały swoją historię po ponownym uruchomieniu. Nic nie identyfikuje osoby, adresu ani wiadomości. Wyłącz, aby zachować je tylko w pamięci.

admin-settings-help-metrics-persist-flush-seconds = Jak często statystyki wydajności są zapisywane w bazie danych (od 5 do 3600 sekund). Serwer zabity bez ostrzeżenia traci co najwyżej tyle.

admin-settings-help-observability-client-capture-console = Zgłaszaj także wywołania console.error, maksymalnie pięć minut na stronę. Domyślnie wyłączone, ponieważ biblioteki rejestrują oczekiwane warunki.

admin-settings-help-observability-client-capture-csp = Zgłaszaj naruszenia zasad bezpieczeństwa treści wykryte przez przeglądarkę, tylko w przypadku zablokowanej dyrektywy i źródła. Domyślnie włączone.

admin-settings-help-observability-client-enabled = Zbieraj raporty o błędach z aplikacji internetowej (granice błędów, nieprzechwycone błędy, nieudane fragmenty, błędy procesu roboczego, naruszenia CSP) z oczyszczonymi śladami stosu, pokazanymi w obszarze Obserwowalność > Błędy klienta. Domyślnie włączone. Raporty nie zawierają treści wiadomości, adresów, tokenów, nazw załączników ani zapytań URL. Wyłącz, aby zatrzymać zbieranie; istniejące raporty wygasają wraz z ich zachowaniem.

admin-settings-help-observability-client-forward = Wyślij także każdy usunięty raport do ujścia błędów skonfigurowanego za pomocą MW_ERROR_FORWARD_URL. Domyślnie wyłączone. Bez skonfigurowanego ujścia nic nie jest wysyłane.

admin-settings-help-observability-client-include-account-hash = Dodaj kluczowany jednokierunkowy skrót konta do raportów wysyłanych z sesji zalogowania się, aby raporty z jednego konta można było rozróżnić bez nazywania go. Domyślnie wyłączone. Hash zmienia się, gdy klucz prywatności się obraca.

admin-settings-help-observability-client-max-per-session = Większość raportów o błędach może zostać wysłana podczas ładowania jednej strony (od 1 do 1000). Powtórzenia tego samego błędu w ciągu 60 sekund liczą się raz. Domyślnie 20.

admin-settings-help-observability-client-sample-rate = Część raportów o błędach wysyłanych przez przeglądarkę, od 0 do 1. Domyślnie 1 (wszystkie). Ładowanie każdej strony nadal zatrzymuje się przy limicie na sesję.

admin-settings-help-observability-client-source-maps = Rozwiąż pozycje pakietów w śladach stosu do plików źródłowych, gdy kompilacja dostarczyła ukryte mapy źródłowe (zbudowane z MW_SOURCEMAP=hidden). Mapy są odczytywane tylko przez serwer i nigdy nie są udostępniane przeglądarkom. Domyślnie włączone; bez map pokazane są surowe klatki.

admin-settings-help-privacy-error-reports = Określa, czy przeglądarka wysyła raport o błędach, gdy część aplikacji ulegnie awarii. Domyślnie włączone. Zakres globalny, domenowy i użytkownika: ustaw tutaj wartość domyślną lub zablokuj ją; ludzie mogą wyłączyć tę funkcję samodzielnie, wybierając Ustawienia > Prywatność, chyba że jest ona zablokowana.

admin-settings-help-privacy-image-cache-max-stale-secs = Kilka sekund po jego aktualności obraz w pamięci podręcznej jest przechowywany, aby odpowiedzieć, gdy nie można dotrzeć do źródła. Nigdy poniżej okna rewalidacji. Zakres 0–604800; domyślny 86400.

admin-settings-help-privacy-image-cache-stale-secs = Kilka sekund po odświeżeniu obraz z pamięci podręcznej jest nadal wyświetlany, a jego weryfikacja odbywa się w tle. Zero to wyłącza. Zakres 0–86400; domyślnie 3600.

admin-settings-help-privacy-image-resize-enabled = Skaluj duże odległe obrazy do szerokości wymaganej przez czytnik (320, 640, 960, 1280 lub 1600 pikseli). Animowane obrazy są wyświetlane w stanie nieskalowanym. Domyślnie wyłączone.

admin-settings-help-privacy-image-resize-format = Automatycznie wybiera najlepszy koder w kompilacji, czyli PNG. Domyślny automat.

admin-settings-help-privacy-image-resize-max-pixels = Obrazy z większą liczbą zdekodowanych pikseli są wyświetlane w stanie nieskalowanym; rozmiar jest odczytywany z nagłówka przed jakimkolwiek dekodowaniem. Zakres 100000–64000000; domyślnie 4000000.

admin-settings-help-privacy-image-resize-max-width = Najszerszy obraz, jaki daje zmiana rozmiaru. Zakres 320–1600; domyślnie 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Jak Ustawienia > Zabezpieczenia > Sesje nazywają konto, z którego pochodzi sesja dostępu do połączonej skrzynki pocztowej: pełny adres, adres z zamaskowaną pierwszą częścią nazwy lub nic. Wartość jest pokazywana tylko właścicielowi sesji, nigdy innym użytkownikom. Domyślnie: pełny adres.

admin-settings-help-properties-folder-enabled = Oferuj właściwości w menu folderów i na skrócie. Gdy opcja jest wyłączona, okno dialogowe nie jest dostępne.

admin-settings-help-properties-folder-notes-enabled = Pozwól na opis lub notatkę w każdym folderze.

admin-settings-help-properties-folder-notes-max-length = Najwięcej znaków, jakie może zawierać notatka folderu.

admin-settings-help-properties-folder-overrides-allowed = Które ustawienia użytkownik może zastąpić dla pojedynczego folderu. Zastąpienie niewymienione na liście jest ukryte, a każda przechowywana wartość jest ignorowana.

admin-settings-help-properties-folder-sections = Sekcje okna dialogowego Właściwości folderu: ogólne, zdalne, rozmiar, offline, nadpisania, notatki, akcje. Sekcja, która nie jest wymieniona, jest ukryta.

admin-settings-help-properties-folder-size-compute = Po ustaleniu rozmiaru folderu na serwerze: nigdy, po otwarciu okna dialogowego lub w tle. Zliczenie dużej skrzynki pocztowej kosztuje serwer skanowanie.

admin-settings-help-properties-headers-raw = Pozwól, aby okno dialogowe Właściwości wiadomości wyświetlało listę nieprzetworzonych nagłówków. Gdy opcja jest wyłączona, wyświetlane jest tylko przeanalizowane podsumowanie.

admin-settings-help-properties-message-enabled = Właściwości oferty w menu wiadomości i na skrócie. Gdy opcja jest wyłączona, okno dialogowe nie jest dostępne.

admin-settings-help-properties-message-permanent-delete = Określa, czy okno dialogowe wiadomości może trwale usunąć wiadomość: zezwól, zapytaj dwa razy lub odmów. Wiadomość przetrzymana ze względów prawnych nigdy nie jest usuwana. Usunięcia w ten sposób nie można cofnąć.

admin-settings-help-properties-message-sections = Sekcje okna dialogowego Właściwości wiadomości: podsumowanie, punktacja, nagłówki, załączniki, akcje. Sekcja, która nie jest wymieniona, jest ukryta.

admin-settings-help-properties-offline-clear-enabled = Zezwalaj użytkownikom na czyszczenie pamięci podręcznej offline pojedynczego folderu w oknie dialogowym Właściwości.

admin-settings-help-properties-remote-location-visibility = Jaka część lokalizacji po stronie serwera jest widoczna dla użytkowników: pełna (host i ścieżka), host zamaskowany (host częściowo ukryty), tylko ścieżka (ścieżka skrzynki pocztowej bez hosta) lub ukryta.

admin-settings-help-properties-scoring-show = Pokaż wynik spamu, wyniki uwierzytelniania i sygnały bezpieczeństwa w oknie dialogowym Właściwości wiadomości.

admin-settings-help-properties-shortcut-enabled = Otwórz okno dialogowe Właściwości z klawiatury w wybranym folderze lub wybranej wiadomości.

admin-settings-help-properties-shortcut-key = Kombinacja klawiszy otwierająca Właściwości; Domyślnie Alt+Enter.

admin-settings-help-reader-actions-archive = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-block-sender = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-colors = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-conversation = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-create-filter = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-delete = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-delete-sender = Domyślnie ukryte. Wybierz pasek narzędzi lub menu Więcej, aby włączyć tę funkcję, jeśli pozwala na to możliwość usuwania nadawcy i profil funkcji. Użytkownicy mogą zastąpić miejsce docelowe, chyba że jest ono zablokowane.

admin-settings-help-reader-actions-edit-draft = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-export = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-finish-followup = Wybierz pasek narzędzi, menu Więcej lub ukryte, aby uzupełniać oznaczone wiadomości i przypomnienia.

admin-settings-help-reader-actions-flag = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-focus-focused = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-focus-other = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-forward = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-headers = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-links = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-move = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-not-spam = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-open-draft = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-pin = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-print = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-properties = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-read = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-reply = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-reply-all = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-save-contacts = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-save-sender = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-search = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-security = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-spam = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-sweep = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-view-source = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-reader-actions-zoom = Wybierz pasek narzędzi, menu Więcej lub ukryj. Nadal obowiązują ograniczenia funkcji, uprawnienia i kontekst wiadomości. Użytkownicy mogą wybrać, chyba że jest to zablokowane.

admin-settings-help-retention-client-errors-event-days = Dni, w których przechowywane jest indywidualne wystąpienie błędu przeglądarki (jego ślad stosu i klasa przeglądarki). Domyślnie 30. Grupa jest przechowywana dłużej, zobacz następne ustawienie.

admin-settings-help-retention-client-errors-group-days = Liczba dni, w których grupa błędów przeglądarki (nazwa, komunikat, liczba) jest przechowywana od chwili jej ostatniego wyświetlenia. Domyślnie 180. Nigdy nie krócej niż czas przechowywania wystąpienia.

admin-settings-help-retention-metrics-day-buckets-days = Dni dziennych statystyk wydajności przechowywane przed ich usunięciem. 0 utrzymuje je bez ograniczeń, co pozwala na powolny wzrost tabeli tak długo, jak długo działa serwer.

admin-settings-help-retention-metrics-hour-buckets-days = Dni godzinowych statystyk wydajności przechowywanych przed połączeniem ich w wiersze dzienne (co najmniej 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Godziny jednominutowych statystyk wydajności przechowywanych przed połączeniem ich w wiersze godzinowe (co najmniej 1).

admin-settings-help-retention-user-allow-strip-attachments = Określa, czy użytkownicy mogą tworzyć reguły usuwające duże załączniki ze starej poczty. Domyślnie wyłączone. Gdy opcja jest wyłączona, akcja jest ukryta w edytorze reguł, reguła, która z niej korzysta, jest odrzucana po zapisaniu, a reguły zapisane wcześniej są pomijane po uruchomieniu zadania.

admin-settings-help-retention-user-calendar-enabled = Określa, czy użytkownicy mogą wybierać kalendarze, których wydarzenia są usuwane po określonej liczbie dni po ich zakończeniu. Domyślnie wyłączone. Gdy opcja jest wyłączona, opcja jest ukryta w Ustawieniach, wybór jest odrzucany po zapisaniu, a wcześniej zapisane wybory nie są uruchamiane.

admin-settings-help-retention-user-calendar-max-days = Przez większość dni użytkownik może poczekać, zanim zakończone wydarzenia zostaną usunięte. Większa wartość jest odrzucana po zapisaniu i obniżana po uruchomieniu.

admin-settings-help-retention-user-calendar-min-days = Najmniejsza liczba dni po zakończeniu wydarzenia, zanim reguła użytkownika będzie mogła je usunąć. Mniejsza wartość jest odrzucana po zapisaniu i podnoszona po uruchomieniu.

admin-settings-help-retention-user-notes-enabled = Określa, czy użytkownicy mogą zdecydować się na usunięcie notatek niezmienionych przez określoną liczbę dni. Domyślnie wyłączone. Usunięte notatki można przywrócić do momentu zakończenia okna cofania reguł przechowywania.

admin-settings-help-retention-user-notes-max-days = Przez większość dni użytkownik może poczekać, zanim nietknięte notatki zostaną usunięte. Większa wartość jest odrzucana po zapisaniu i obniżana po uruchomieniu.

admin-settings-help-retention-user-notes-min-days = Przez kilka dni notatka musi pozostać nietknięta, zanim reguła użytkownika będzie mogła ją usunąć. Mniejsza wartość jest odrzucana po zapisaniu i podnoszona po uruchomieniu.

admin-settings-help-retention-user-strip-keep-inline-images = Określa, czy obrazy wyświetlane w treści wiadomości mają być zachowywane po usunięciu załączników.

admin-settings-help-retention-user-strip-min-age-days = Najmniej dni, w których wiadomość musi być stara, aby reguła mogła usunąć jej załączniki. Mniejsza wartość jest odrzucana po zapisaniu reguły i wywoływana po jej uruchomieniu.

admin-settings-help-retention-user-strip-min-attachment-kb = Usuwane są tylko załączniki co najmniej tego rozmiaru. Mniejsze załączniki pozostają w wiadomości.

admin-settings-help-retention-user-strip-undo-days = Liczba dni, w których oryginalna wiadomość pozostaje w Koszu po usunięciu jej załączników. Wartość -1 następuje po oknie cofania reguł przechowywania. 0 powoduje natychmiastowe usunięcie oryginału.

admin-settings-help-server-shutdown-grace-seconds = Jak długo zatrzymujący się serwer pozwala na zakończenie żądań w locie przed zamknięciem połączeń, w sekundach. Zmienna MW_HTTP_SHUTDOWN_TIMEOUT_MS zastępuje ją. Obowiązuje przy następnym uruchomieniu.

admin-settings-help-session-last-activity-flush-seconds = Czas ostatniej sesji jest zapisywany najczęściej w sekundach. Zero pisze to na każde żądanie. Nie zmienia to limitów biegu jałowego i bezwzględnego. Obowiązuje przy następnym uruchomieniu.

admin-settings-help-session-persist-across-restarts = Gdy ta opcja jest włączona, sesje skrzynki pocztowej, aplikacji i konsoli pozostają ważne po ponownym uruchomieniu serwera. Gdy opcja jest wyłączona, każda sesja kończy się wraz z uruchomieniem serwera. Obowiązuje przy następnym uruchomieniu.

admin-settings-key-attachments-bar-click-to-toggle = Kliknij pasek załączników, aby przełączyć

admin-settings-key-attachments-bar-download-all = Pobierz wszystko, przycisk

admin-settings-key-attachments-bar-show-download-buttons = Przyciski pobierania załączników

admin-settings-key-attachments-bar-zip-inline-images = Obrazy wbudowane w Pobierz wszystko ZIP

admin-settings-key-cap-reading-delete-sender = Zezwalaj na usuwanie całej poczty od nadawcy

admin-settings-key-compose-feature-family = Funkcje kompozytora

admin-settings-key-compose-headers-allow-auto-submitted = Zezwalaj na automatyczne przesyłanie

admin-settings-key-compose-headers-allow-custom = Zezwalaj na niestandardowe nagłówki

admin-settings-key-compose-headers-allow-return-receipt-to = Zezwalaj na zwrot-potwierdzenie-do

admin-settings-key-compose-headers-allow-threading-edit = Zezwalaj na edycję nagłówków wątków

admin-settings-key-compose-headers-forced = Wymuszone nagłówki

admin-settings-key-compose-insert-image-resize-default = Wstępnie ustawiony rozmiar obrazu

admin-settings-key-compose-insert-max-inline-image-mb = Największy obraz w tekście (MB)

admin-settings-key-compose-layout-default-font = Domyślna czcionka wiadomości

admin-settings-key-compose-reply-to-allow-override = Zezwalaj na zmianę odpowiedzi na

admin-settings-key-compose-reply-to-show = Odpowiedz-do wiersza

admin-settings-key-compose-sent-folder-fixed-id = Wysłana kopia: stały folder

admin-settings-key-compose-sent-folder-mode = Wysłana kopia: reguła folderu

admin-settings-key-compose-sent-folder-per-identity = Wysłana kopia: folder na tożsamość

admin-settings-key-compose-sent-folder-rules = Wysłany egzemplarz: regulamin

admin-settings-key-compose-sent-folder-skip-copy = Wysłana kopia: nie zapisuj

admin-settings-key-compose-styles-custom = Style niestandardowe

admin-settings-key-compose-styles-default-set = Zestaw stylów

admin-settings-key-compose-styles-max-custom = Style niestandardowe dla każdego użytkownika

admin-settings-key-contacts-save-from-message-add-note = Uwaga dotycząca kontaktów zapisanych z wiadomości

admin-settings-key-contacts-save-from-message-default-book = Książka adresowa kontaktów zapisanych z wiadomości

admin-settings-key-contacts-save-from-message-default-scope = Kontakty zaznaczone jako pierwsze podczas zapisywania z wiadomości

admin-settings-key-contacts-save-from-message-enabled = Zapisz kontakty z wiadomości

admin-settings-key-contacts-save-from-message-skip-automated = Pomiń automatycznych nadawców podczas zapisywania kontaktów

admin-settings-key-contacts-save-sender-enabled = Zapisz nadawcę jako kontakt

admin-settings-key-contacts-save-sender-use-reply-to = Zapisz odpowiedź do dla nadawców list

admin-settings-key-dsr-admin-step-up = Potwierdź ponownie przed zatwierdzeniem

admin-settings-key-export-default-format = Eksportuj domyślny format okna dialogowego

admin-settings-key-export-default-scope = Eksportuj domyślny zakres okna dialogowego

admin-settings-key-export-dialog-enabled = Eksportuj okno dialogowe

admin-settings-key-export-folder-enabled = Eksport folderu

admin-settings-key-export-folder-format = Format eksportu folderu

admin-settings-key-export-folder-include-subfolders = Uwzględnij podfoldery w eksporcie

admin-settings-key-export-folder-max-bytes = Limit rozmiaru eksportu folderu

admin-settings-key-export-folder-max-messages = Limit wiadomości eksportowanych do folderu

admin-settings-key-export-formats = Eksportuj formaty okien dialogowych

admin-settings-key-export-message-download-enabled = Pobieranie wiadomości

admin-settings-key-folders-hide-empty = Ukryj puste foldery

admin-settings-key-folders-pinned-hide-from-tree = Ukryj przypięte foldery z drzewa

admin-settings-key-mail-badge-auth = Odznaka uwierzytelniająca

admin-settings-key-mail-badge-density = Rozmiar plakietki czytnika

admin-settings-key-mail-badge-folder = Odznaka folderu

admin-settings-key-mail-badge-headers = Odznaki nagłówka

admin-settings-key-mail-badge-labels = Etykiety odznak czytelnika

admin-settings-key-mail-badge-list = Odznaka listy mailingowej

admin-settings-key-mail-badge-sanitized = Odkażona plakietka HTML

admin-settings-key-mail-badge-spam = Odznaka wyniku spamu

admin-settings-key-mail-brand-avatars-require-auth = Uwierzytelnienie wymagane w przypadku logo marki

admin-settings-key-mail-compose-header-auto-collapse-seconds = Opóźnienie automatycznego zwijania nagłówka kompozytora

admin-settings-key-mail-compose-priority-default = Domyślny priorytet wiadomości

admin-settings-key-mail-compose-priority-enabled = Selektor priorytetów w kompozytorze

admin-settings-key-mail-expiry-hide-expired = Wygasłe wiadomości

admin-settings-key-mail-list-keep = Zawsze trzymaj

admin-settings-key-mail-list-page-size = Rozmiar strony listy wiadomości

admin-settings-key-mail-list-unsubscribe-mode = Lista mailingowa Zrezygnuj z subskrypcji

admin-settings-key-mail-message-list-group = Grupowanie wiadomości

admin-settings-key-mail-priority-marks = Znaki priorytetowe

admin-settings-key-mail-priority-marks-style = Styl znaku priorytetu

admin-settings-key-mail-priority-send-headers = Nagłówki priorytetowe zapisywane przy wysyłaniu

admin-settings-key-mail-priority-trust-sender-headers = Zaufaj priorytetowi ustawionemu nadawcy

admin-settings-key-mail-reply-from-header-priority = Nagłówki, które wybierają tożsamość odpowiedzi

admin-settings-key-mail-search-default-scope = Domyślny zakres wyszukiwania poczty

admin-settings-key-metrics-dimensions-enabled = Podziel statystyki według rodzaju klienta i klasy urządzenia

admin-settings-key-metrics-persist-enabled = Prowadź statystyki dotyczące ponownych uruchomień

admin-settings-key-metrics-persist-flush-seconds = Interwał zapisywania statystyk (sekundy)

admin-settings-key-observability-client-capture-console = Zgłoś błędy konsoli

admin-settings-key-observability-client-capture-csp = Zgłoś naruszenia CSP

admin-settings-key-observability-client-enabled = Raporty o błędach przeglądarki

admin-settings-key-observability-client-forward = Przesyłaj dalej raporty o błędach

admin-settings-key-observability-client-include-account-hash = Dodaj skrót konta do raportów o błędach

admin-settings-key-observability-client-max-per-session = Raporty błędów na każde załadowanie strony

admin-settings-key-observability-client-sample-rate = Częstotliwość próbkowania raportu o błędach

admin-settings-key-observability-client-source-maps = Rozwiązuj ślady stosu za pomocą map źródłowych

admin-settings-key-privacy-error-reports = Wysyłaj raporty o błędach

admin-settings-key-privacy-image-cache-max-stale-secs = Zachowaj nieaktualne, gdy źródło się nie powiedzie (sekundy)

admin-settings-key-privacy-image-cache-stale-secs = Podawaj nieświeże podczas ponownej walidacji (sekundy)

admin-settings-key-privacy-image-resize-enabled = Zmień rozmiar dużych obrazów

admin-settings-key-privacy-image-resize-format = Zmieniono format obrazu

admin-settings-key-privacy-image-resize-max-pixels = Największy obraz do zmiany rozmiaru (w pikselach)

admin-settings-key-privacy-image-resize-max-width = Najszerszy przeskalowany obraz (w pikselach)

admin-settings-key-privacy-sessions-show-linked-origin = Powiązane początki skrzynek pocztowych w sesjach

admin-settings-key-properties-folder-enabled = Okno właściwości folderu

admin-settings-key-properties-folder-notes-enabled = Notatki w folderze

admin-settings-key-properties-folder-notes-max-length = Długość notatki w folderze

admin-settings-key-properties-folder-overrides-allowed = Dozwolone zastąpienie poszczególnych folderów

admin-settings-key-properties-folder-sections = Sekcje okna dialogowego folderów

admin-settings-key-properties-folder-size-compute = Obliczanie rozmiaru folderu

admin-settings-key-properties-headers-raw = Surowe nagłówki

admin-settings-key-properties-message-enabled = Okno dialogowe właściwości wiadomości

admin-settings-key-properties-message-permanent-delete = Trwałe usunięcie z właściwości

admin-settings-key-properties-message-sections = Sekcje okna dialogowego wiadomości

admin-settings-key-properties-offline-clear-enabled = Wyczyść pamięć podręczną offline dla każdego folderu

admin-settings-key-properties-remote-location-visibility = Szczegóły lokalizacji zdalnej

admin-settings-key-properties-scoring-show = Szczegóły punktacji i uwierzytelniania

admin-settings-key-properties-shortcut-enabled = Skrót właściwości

admin-settings-key-properties-shortcut-key = Klawisz skrótu Właściwości

admin-settings-key-reader-actions-archive = Rozmieszczenie działań archiwalnych

admin-settings-key-reader-actions-block-sender = Blokuj miejsce akcji nadawcy

admin-settings-key-reader-actions-colors = Rozmieszczenie akcji w kolorach wiadomości

admin-settings-key-reader-actions-conversation = Wyświetl rozmieszczenie akcji konwersacji

admin-settings-key-reader-actions-create-filter = Utwórz miejsce akcji filtra

admin-settings-key-reader-actions-delete = Usuń umiejscowienie akcji

admin-settings-key-reader-actions-delete-sender = Usuń wszystkie wiadomości z akcji nadawcy

admin-settings-key-reader-actions-edit-draft = Edytuj położenie akcji w wersji roboczej

admin-settings-key-reader-actions-export = Eksportuj rozmieszczenie akcji

admin-settings-key-reader-actions-finish-followup = Oznacz kontynuację jako akcję zakończoną

admin-settings-key-reader-actions-flag = Oznaczenie/odznaczenie miejsca akcji

admin-settings-key-reader-actions-focus-focused = Przejdź do umieszczenia skoncentrowanej akcji

admin-settings-key-reader-actions-focus-other = Przejdź do innego miejsca akcji

admin-settings-key-reader-actions-forward = Umiejscowienie akcji do przodu

admin-settings-key-reader-actions-headers = Umieszczenie akcji surowych nagłówków

admin-settings-key-reader-actions-links = Przejrzyj rozmieszczenie akcji linków

admin-settings-key-reader-actions-move = Przesuń rozmieszczenie akcji

admin-settings-key-reader-actions-not-spam = Nie umieszczanie działań spamowych

admin-settings-key-reader-actions-open-draft = Otwórz miejsce docelowe akcji w wersji roboczej odpowiedzi

admin-settings-key-reader-actions-pin = Przypnij/odepnij rozmieszczenie akcji

admin-settings-key-reader-actions-print = Wydrukuj rozmieszczenie akcji

admin-settings-key-reader-actions-properties = Umiejscowienie akcji właściwości wiadomości

admin-settings-key-reader-actions-read = Umiejscowienie akcji przeczytanej/nieprzeczytanej

admin-settings-key-reader-actions-reply = Odpowiedz na umiejscowienie akcji

admin-settings-key-reader-actions-reply-all = Odpowiedz na wszystkie miejsca akcji

admin-settings-key-reader-actions-save-contacts = Zapisz miejsce akcji kontaktów wiadomości

admin-settings-key-reader-actions-save-sender = Zapisz miejsce akcji nadawcy

admin-settings-key-reader-actions-search = Powiązane umiejscowienie akcji wyszukiwania

admin-settings-key-reader-actions-security = Umiejscowienie akcji w trybie bezpieczeństwa

admin-settings-key-reader-actions-spam = Oznacz jako miejsce akcji spamowej

admin-settings-key-reader-actions-sweep = Przeszukaj miejsce akcji nadawcy

admin-settings-key-reader-actions-view-source = Wyświetl rozmieszczenie działań w źródle wiadomości

admin-settings-key-reader-actions-zoom = Powiększ umiejscowienie akcji

admin-settings-key-retention-client-errors-event-days = Zachowane wystąpienia błędów (dni)

admin-settings-key-retention-client-errors-group-days = Zachowane grupy błędów (dni)

admin-settings-key-retention-metrics-day-buckets-days = Prowadzone statystyki dzienne (dni)

admin-settings-key-retention-metrics-hour-buckets-days = Prowadzone statystyki godzinowe (dni)

admin-settings-key-retention-metrics-minute-buckets-hours = Prowadzone statystyki minutowe (godziny)

admin-settings-key-retention-user-allow-strip-attachments = Użytkownicy mogą usuwać załączniki ze starej poczty

admin-settings-key-retention-user-calendar-enabled = Użytkownicy mogą usuwać zakończone wydarzenia z kalendarza

admin-settings-key-retention-user-calendar-max-days = Większość dni poprzedzających zakończone wydarzenia jest usuwana

admin-settings-key-retention-user-calendar-min-days = Najmniejsza liczba dni przed zakończonymi wydarzeniami jest usuwana

admin-settings-key-retention-user-notes-enabled = Użytkownicy mogą usuwać nietknięte notatki

admin-settings-key-retention-user-notes-max-days = Większość dni przed nietkniętymi notatkami jest usuwana

admin-settings-key-retention-user-notes-min-days = Najmniej dni przed usunięciem nietkniętych notatek

admin-settings-key-retention-user-strip-keep-inline-images = Zachowaj obrazy wbudowane

admin-settings-key-retention-user-strip-min-age-days = Najmniej dni przed usunięciem załączników

admin-settings-key-retention-user-strip-min-attachment-kb = Usunięto najmniejszy załącznik (KB)

admin-settings-key-retention-user-strip-undo-days = Okno cofania po usunięciu załączników (dni)

admin-settings-key-server-shutdown-grace-seconds = Okres karencji wyłączenia

admin-settings-key-session-last-activity-flush-seconds = Ostatnio widziany interwał zapisu

admin-settings-key-session-persist-across-restarts = Utrzymuj sesje po ponownym uruchomieniu

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = Domyślnie wyłączone. Zapisuj zaszyfrowane migawki poczty o zasięgu konta podczas ponownego ładowania przeglądarki bez utraty wieku. Istniejące limity wpisów i bajtów nadal obowiązują. Użytkownicy mogą wyczyścić zapisane kopie; wyłączenie tej preferencji powoduje ich usunięcie w aktywnej przeglądarce. Nie przyznaje to przeglądarce pozwolenia na trwałe przechowywanie ani pobierania załączników.

admin-settings-help-mail-conversations-allowed = Zezwalaj użytkownikom na grupowanie powiązanych odpowiedzi w wątki. Dostępne domyślnie, ale zaczynają się preferencje użytkownika. Wyłączenie dostępności zastępuje wybór użytkownika bez zmiany grupowania kategorii.

admin-settings-help-mail-conversations-enabled = Domyślnie wyłączone. Grupuj załadowane odpowiedzi według wątków, niezależnie od opcji Grupuj według. Ustaw ustawienie domyślne lub zablokuj tę preferencję. Dostępność konwersacji musi być również włączona.

admin-settings-key-mail-cache-keep-indefinitely = Przechowuj pamięć podręczną poczty przeglądarki na czas nieokreślony

admin-settings-key-mail-conversations-allowed = Udostępnij wątki konwersacji

admin-settings-key-mail-conversations-enabled = Grupuj wiadomości w rozmowy

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = Do 100 domen nadawców zawierających małe litery, których linki do sprawdzanych wiadomości są włączane automatycznie. Subdomeny są oddzielne. Nie ufa witrynom docelowym ani nie zezwala na zdalne obrazy. Domyślnie pusty; użytkownicy mogą to zmienić, chyba że są zablokowane.

admin-settings-key-links-trusted-sender-domains = Zaufane domeny nadawców dla linków do wiadomości

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Pokaż szczegóły pobierania i pliku obok tytułu podglądu jako ikony (domyślnie), etykiety lub ikony i etykiety. Można ustawić i zablokować globalnie, dla domeny lub dla użytkownika.

admin-settings-help-reader-attachment-thumbnails = Domyślnie wyłączone. Zezwalaj na widoczne załączniki obrazów, aby ładować podglądy miniatur. Zasady bezpieczeństwa załączników nadal obowiązują. Można ustawić i zablokować globalnie, dla domeny lub dla użytkownika.

admin-settings-key-reader-attachment-action-style = Działania podglądu załącznika

admin-settings-key-reader-attachment-thumbnails = Miniatury załączników obrazów

admin-settings-key-mail-compose-auto-add-mention-recipients = Automatycznie dodawaj wspomniane osoby jako odbiorców
admin-settings-help-mail-compose-auto-add-mention-recipients = Po wybraniu osoby z podpowiedzi @wzmianek dodaj ją do pola Do, chyba że jest już w polu Do, DW lub UDW. Domyślnie wyłączone.
