# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Pulpit
admin-shell-group-overview = Przegląd
admin-shell-group-people = Osoby
admin-shell-group-security = Bezpieczeństwo
admin-shell-group-configuration = Konfiguracja
admin-shell-group-operations = Eksploatacja
admin-shell-menu = Menu
admin-shell-close-menu = Zamknij menu
admin-shell-search = Szukaj
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Zalogowano jako { $name }
admin-shell-language-label = Język
admin-shell-breadcrumb = Ścieżka nawigacji
admin-shell-unknown-section = Ta sekcja nie istnieje lub Twoja rola jej nie obejmuje.
admin-shell-back-to-dashboard = Wróć do pulpitu
admin-shell-cap-missing = Twoja rola nie obejmuje uprawnienia { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Wczytywanie…
admin-shell-retry = Spróbuj ponownie
admin-shell-cancel = Anuluj
admin-shell-save = Zapisz
admin-shell-not-available = Niedostępne

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Opuścić tę stronę?
admin-shell-leave-body = Zmiany na tej stronie nie zostały zapisane. Opuszczenie strony je odrzuci.
admin-shell-leave-confirm = Opuść i odrzuć zmiany

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Wpisz { $name }, aby potwierdzić

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Przeszukaj konsolę administracyjną
admin-shell-palette-label = Sekcja, użytkownik, domena lub ustawienie
admin-shell-palette-results = Wyniki
admin-shell-palette-none = Brak wyników.
admin-shell-palette-searching = Wyszukiwanie użytkowników, domen i ustawień…
admin-shell-palette-sections-only = Ten serwer nie potrafi jeszcze wyszukiwać użytkowników, domen i ustawień, więc wyświetlane są tylko sekcje.
admin-shell-palette-error = Wyszukiwanie użytkowników, domen i ustawień nie powiodło się; wyświetlane są tylko sekcje.
admin-shell-palette-kind-section = Sekcja
admin-shell-palette-kind-user = Użytkownik
admin-shell-palette-kind-domain = Domena
admin-shell-palette-kind-setting = Ustawienie

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filtry
admin-shell-table-actions = Akcje
admin-shell-table-error = Nie udało się wczytać listy.
admin-shell-table-paging = Strony: { $table }
admin-shell-table-prev = Poprzednia strona
admin-shell-table-next = Następna strona

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Najnowsza { $latest }, najwyższa { $max }
admin-shell-chart-col-time = Czas
admin-shell-chart-col-item = Element
admin-shell-chart-col-value = Wartość

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Ustawione przez
admin-shell-pinned-sr = Ta wartość pochodzi ze środowiska serwera i nie można jej tutaj zmienić.
admin-shell-lock-text = Zablokuj dla użytkowników
admin-shell-lock-setting = { $setting }
admin-shell-restore = Przywróć poprzednią wartość
admin-shell-restore-note = Zmienione przez { $actor } dnia { $time }. Poprzednia wartość: { $value }
admin-shell-restore-done = Przywrócono poprzednią wartość.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Kroki
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Krok { $n } z { $total }: { $title }
admin-shell-wizard-review = Przegląd
admin-shell-wizard-back = Wstecz
admin-shell-wizard-next = Dalej
admin-shell-wizard-saved = Zapisano.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Nie można połączyć się z serwerem. Sprawdź połączenie i spróbuj ponownie.
admin-shell-err-session = Sesja administratora wygasła. Zaloguj się ponownie.
admin-shell-err-forbidden = Twoja rola nie pozwala na tę akcję. Poproś administratora z odpowiednią rolą.
admin-shell-err-out-of-scope = Ten obiekt należy do domeny, której Twoja rola nie obejmuje.
admin-shell-err-not-found = Ten obiekt już nie istnieje. Odśwież listę.
admin-shell-err-conflict = Ktoś w międzyczasie go zmienił. Odśwież i spróbuj ponownie.
admin-shell-err-validation = Serwer odrzucił wartość. Sprawdź pola i spróbuj ponownie.
admin-shell-err-rate-limited = Zbyt wiele żądań. Odczekaj minutę i spróbuj ponownie.
admin-shell-err-server = Serwer zwrócił błąd ({ $status }). Spróbuj ponownie; jeśli błąd się powtórzy, sprawdź dziennik serwera.
admin-shell-err-upstream-dns = Nie można rozwiązać nazwy hosta. Sprawdź pisownię nazwy hosta.
admin-shell-err-upstream-connect = Nic nie odpowiada na tym hoście i porcie. Sprawdź port i ewentualną zaporę po drodze.
admin-shell-err-upstream-tls = Serwer odpowiedział, ale uzgadnianie TLS nie powiodło się. Sprawdź port i ustawienie zabezpieczeń połączenia.
admin-shell-err-upstream-auth = Serwer odrzucił dane logowania. Sprawdź nazwę użytkownika i hasło.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Zbyt wiele prób logowania. Odczekaj kilka minut i spróbuj ponownie.
admin-shell-totp-form = Logowanie administratora, drugi krok
admin-shell-totp-note = Wpisz 6-cyfrowy kod z aplikacji uwierzytelniającej skonfigurowanej dla { $name }.
admin-shell-totp-code = Kod uwierzytelniający
admin-shell-totp-verify = Zweryfikuj
admin-shell-totp-start-over = Zacznij od nowa
admin-shell-totp-invalid = Ten kod jest nieprawidłowy. Wpisz bieżący kod z aplikacji uwierzytelniającej.
admin-shell-totp-expired = Logowanie trwało zbyt długo. Zacznij od nowa i wpisz hasło.
