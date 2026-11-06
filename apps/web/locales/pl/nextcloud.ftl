# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = W górę
nextcloud-loading = Ładowanie…
nextcloud-list-error = Nie udało się wyświetlić zawartości tego folderu.
nextcloud-empty = Ten folder jest pusty.
nextcloud-file-list = Pliki Nextcloud
nextcloud-open-folder = Otwórz folder { $name }
nextcloud-select-file = Wybierz { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Załącz z Nextcloud
nextcloud-attach-action = { $count ->
    [0] Załącz
    [one] Załącz { $count } plik
    [few] Załącz { $count } pliki
    [many] Załącz { $count } plików
   *[other] Załącz { $count } pliku
  }
nextcloud-error-select-file = wybierz co najmniej jeden plik do załączenia
nextcloud-error-attach-failed = nie udało się załączyć wybranych plików

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Zapisz w Nextcloud
nextcloud-save-title = Zapisz „{ $name }” w Nextcloud
nextcloud-save-action = Zapisz tutaj ({ $dir })
nextcloud-saved = Zapisano w { $path }
nextcloud-error-save-failed = nie udało się zapisać w Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Utwórz link udostępniania
nextcloud-share-title = Link udostępniania
nextcloud-share-intro = Utwórz publiczny link do { $path } zamiast załączać plik.
nextcloud-protect-password = Chroń hasłem
nextcloud-password-label = Hasło
nextcloud-share-password = Hasło udostępniania
nextcloud-set-expiry = Ustaw datę wygaśnięcia
nextcloud-expires-on = Wygasa
nextcloud-expiry-date = Data wygaśnięcia
nextcloud-create-link = Utwórz link
nextcloud-password-protected = Chroniony hasłem
nextcloud-no-password = Bez hasła
nextcloud-expires = wygasa { $date }
nextcloud-no-expiry = bez daty wygaśnięcia
nextcloud-error-need-password = wpisz hasło lub wyłącz ochronę hasłem
nextcloud-error-need-expiry = wybierz datę wygaśnięcia lub wyłącz wygasanie
nextcloud-error-share-failed = nie udało się utworzyć linku udostępniania

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = Plik { $path } przekracza limit załączników. Prześlij go do folderu „Mail attachments” w Twoim Nextcloud i wstaw w wiadomości link do udostępnienia.
nextcloud-upload-and-link = Prześlij i wstaw link
nextcloud-share-not-now = Nie teraz
nextcloud-upload-progress = Przesyłanie: { $percent }%
nextcloud-upload-progress-label = Postęp przesyłania
nextcloud-share-inserted = Wstawiono w wiadomości link do udostępnienia pliku { $name }.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = Link działa do { $date }.
nextcloud-share-body-password = Link jest chroniony hasłem.
nextcloud-account-title = Konta Nextcloud
nextcloud-account-help = Połącz własne konto z instancją zatwierdzoną przez administratora. Dane logowania są przechowywane w postaci zaszyfrowanej wyłącznie dla tego konta pocztowego.
nextcloud-account-load-error = Nie udało się wczytać połączeń Nextcloud. Odśwież stronę, aby spróbować ponownie.
nextcloud-account-unlink-error = Nie udało się usunąć połączenia. Odśwież stronę przed ponowną próbą.
nextcloud-account-reload = Odśwież
nextcloud-account-no-instances = Brak dostępnych włączonych instancji Nextcloud. Poproś administratora o skonfigurowanie jednej.
nextcloud-account-ready = Połączono i gotowe
nextcloud-account-stale = Konfiguracja instancji uległa zmianie. Zweryfikuj połączenie ponownie.
nextcloud-account-unlinked = Niepołączone
nextcloud-account-relink = Zweryfikuj połączenie ponownie
nextcloud-account-link = Połącz konto
nextcloud-account-unlink = Odłącz konto
nextcloud-account-password-help = W osobistych ustawieniach zabezpieczeń Nextcloud utwórz hasło aplikacji i skopiuj tutaj jego nazwę użytkownika oraz hasło. Weryfikacja wykonuje jedno ograniczone żądanie; odrzucone dane logowania nie są ponawiane.
nextcloud-account-username = Nazwa użytkownika hasła aplikacji Nextcloud
nextcloud-account-password = Hasło aplikacji Nextcloud
nextcloud-account-verify = Zweryfikuj i połącz
nextcloud-account-verify-error = Weryfikacja lub zapis nie powiodły się. Sprawdź instancję, nazwę użytkownika hasła aplikacji i hasło. Pole hasła zostało wyczyszczone; istniejące działające połączenie nie zostało zastąpione.
nextcloud-admin-title = Instancje Nextcloud
nextcloud-admin-help = Zatwierdź instancje HTTPS, które użytkownicy poczty mogą łączyć indywidualnie. Konta pocztowe nie otrzymują żadnego wspólnego hasła usługi. Operacje na plikach są ograniczone do 8 MiB. Użytkownicy silnika natywnego po połączeniu jawnie wykrywają i podłączają kolekcje kalendarzy, zadań i książek adresowych.
nextcloud-admin-add = Dodaj instancję Nextcloud
nextcloud-admin-edit = Edytuj instancję Nextcloud
nextcloud-admin-enabled = Włączona
nextcloud-admin-disabled = Wyłączona
nextcloud-admin-missing = Ta instancja już nie istnieje. Wróć do listy instancji.
nextcloud-admin-private-help = Adresy prywatne wymagają jawnego uprawnienia ruchu wychodzącego Nextcloud. Skonfigurowane trasy proxy pozostają nadrzędne; odrzucone żądanie proxy nigdy nie przechodzi na połączenie bezpośrednie.
nextcloud-admin-revalidation = Zapisanie jakiejkolwiek zmiany instancji wymaga od użytkowników ponownej weryfikacji ich połączeń. Wyłączenie instancji natychmiast zatrzymuje nowe operacje.
nextcloud-admin-authority = Autorytet instancji
nextcloud-admin-name = Nazwa wyświetlana
nextcloud-admin-name-help = Krótka nazwa pokazywana użytkownikom przy wyborze instancji.
nextcloud-admin-url = Adres URL instalacji HTTPS
nextcloud-admin-url-help = Użyj katalogu głównego instalacji, np. https://cloud.example.org/ lub https://example.org/nextcloud/. Certyfikat musi być zaufany i zgodny z nazwą hosta. Dane logowania, parametry zapytania i fragmenty są odrzucane.
nextcloud-admin-enable = Zezwól użytkownikom na łączenie tej instancji
nextcloud-admin-ca = Dodatkowe urzędy certyfikacji (PEM)
nextcloud-admin-ca-help = Opcjonalne prywatne certyfikaty urzędów certyfikacji dla tego serwera Nextcloud. Zaufanie publiczne pozostaje dostępne, a weryfikacja nazwy hosta nadal jest wymagana. Zmiany wymagają od użytkowników ponownej weryfikacji ich połączeń; zaufanie do certyfikatów proxy jest odrębne.
nextcloud-instance-select = Instancja Nextcloud
nextcloud-instance-placeholder = Wybierz połączoną instancję
nextcloud-instances-empty = Brak zweryfikowanych połączeń Nextcloud. Dodaj lub zweryfikuj połączenie w Ustawienia użytkownika → Nextcloud.
nextcloud-instances-error = Nie udało się wczytać połączonych instancji. Otwórz selektor ponownie, aby spróbować jeszcze raz.
nextcloud-save-warning = Administrator wymaga potwierdzenia, zanim ten załącznik będzie mógł opuścić skrzynkę pocztową.
nextcloud-save-acknowledge = Rozumiem ostrzeżenie dotyczące załącznika i chcę zapisać ten plik w wybranej instancji Nextcloud.
nextcloud-save-attachment = Zapisz { $name } w Nextcloud
nextcloud-save-close = Zamknij miejsce docelowe Nextcloud
nextcloud-save-options = Zapisz załączniki w Nextcloud
nextcloud-pim-title = Kontakty i kalendarze
nextcloud-pim-help = Wykryj kolekcje z tego zweryfikowanego konta Nextcloud, a następnie jawnie podłącz te, które chcesz. Synchronizuj, aby pobrać bieżące dane; edycje w podłączonych kolekcjach korzystają ze zdalnej kolekcji i kończą się niepowodzeniem, jeśli jej połączenie lub uprawnienia zostaną cofnięte.
nextcloud-pim-category = Kategoria kolekcji
nextcloud-pim-calendar = Kalendarze
nextcloud-pim-contacts = Książki adresowe
nextcloud-pim-tasks = Listy zadań
nextcloud-pim-discover = Wykryj kolekcje
nextcloud-pim-discovered = Dostępne kolekcje
nextcloud-pim-empty = Nie zwrócono pasujących kolekcji.
nextcloud-pim-connect = Podłącz
nextcloud-pim-connected = Podłączone kolekcje
nextcloud-pim-sync = Synchronizuj teraz
nextcloud-pim-sync-needed = Kolekcja podłączona. Zsynchronizuj teraz, aby wczytać jej zawartość.
nextcloud-pim-synced = Synchronizacja zakończona.
nextcloud-pim-error = Operacja nie powiodła się. Sprawdź połączenie konta i uprawnienia, a następnie ponów próbę jawnie. Istniejące dane z pamięci podręcznej nie zostały odrzucone.
nextcloud-pim-native-required = Synchronizacja kontaktów i kalendarza wymaga natywnego silnika poczty. Dostęp do plików pozostaje dostępny przez połączenie konta.
nextcloud-pim-disconnect = Odłącz
nextcloud-pim-disconnect-help = Odłączenie zachowuje dane z pamięci podręcznej i zatrzymuje zdalne zapisy. Ponownie połączone konta wymagają ponownego wykrycia i jawnego podłączenia.
nextcloud-pim-ready = Gotowe
nextcloud-pim-disconnected = Odłączone lub połączenie zmienione
nextcloud-pim-unnamed = Kolekcja bez nazwy
nextcloud-pim-back = Wróć do kont Nextcloud
nextcloud-pim-create-title = Nowa kolekcja w Nextcloud
nextcloud-pim-create-help = Tworzy pusty kalendarz lub listę zadań na tym koncie Nextcloud i podłącza ją tutaj.
nextcloud-pim-create-name = Nazwa nowej kolekcji
nextcloud-pim-create = Utwórz w Nextcloud
nextcloud-pim-created = Utworzono { $name } w Nextcloud i podłączono. Zsynchronizuj teraz, aby ją wczytać.
nextcloud-pim-interval = Interwał automatycznego odświeżania (sekundy)
nextcloud-pim-interval-help = Zero oznacza wyłącznie odświeżanie ręczne. Dodatnie interwały mają minimum 60 sekund. Połączone konta odświeżają się w tle; błędy powodują wydłużanie odstępów, a odrzucone dane logowania wstrzymują dalsze żądania do czasu ponownego połączenia.
nextcloud-pim-interval-reset = Użyj domyślnej wartości administratora
nextcloud-admin-empty = Nie ma jeszcze zatwierdzonych instancji. Dodaj instancję, aby udostępnić ją do łączenia kont.
nextcloud-admin-status = Stan
nextcloud-admin-open = Otwórz { $name }
