# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Zamknij

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Niedostępne — { $reason }
admin-ux-unavailable-default = ten serwer jeszcze tego nie oferuje.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Kod błędu: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Otwórz
admin-ux-list-back = Wróć do listy

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Brak niezapisanych zmian
admin-ux-form-changes =
    { $count ->
        [one] 1 niezapisana zmiana
        [few] { $count } niezapisane zmiany
        [many] { $count } niezapisanych zmian
       *[other] { $count } niezapisanej zmiany
    }
admin-ux-form-discard = Odrzuć zmiany
admin-ux-form-saving = Zapisywanie…
admin-ux-form-saved =
    { $count ->
        [one] Zapisano 1 zmianę.
        [few] Zapisano { $count } zmiany.
        [many] Zapisano { $count } zmian.
       *[other] Zapisano { $count } zmiany.
    }
admin-ux-form-fix =
    { $count ->
        [one] Nic nie zostało zapisane. Popraw pole oznaczone poniżej.
        [few] Nic nie zostało zapisane. Popraw { $count } pola oznaczone poniżej.
        [many] Nic nie zostało zapisane. Popraw { $count } pól oznaczonych poniżej.
       *[other] Nic nie zostało zapisane. Popraw { $count } pola oznaczone poniżej.
    }
admin-ux-learn-more = Więcej informacji
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = o polu { $label }
admin-ux-propagation-hot = Zaczyna obowiązywać w ciągu 30 sekund w każdym procesie serwera.
admin-ux-propagation-restart = Zaczyna obowiązywać po ponownym uruchomieniu serwera.
admin-ux-secret-set = Wartość jest zapisana. Wpisz nową, aby ją zastąpić.
admin-ux-secret-unset = Żadna wartość nie jest zapisana.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Wprowadź wartość.
admin-ux-err-number = Wprowadź liczbę.
admin-ux-err-whole = Wprowadź liczbę całkowitą.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Wprowadź { $min } lub więcej.
admin-ux-err-max = Wprowadź { $max } lub mniej.
admin-ux-err-max-length = Użyj najwyżej { $max } znaków.
admin-ux-err-url = Wprowadź pełny adres zaczynający się od https:// lub http://.
admin-ux-err-email = Wprowadź adres e-mail, na przykład name@example.org.
admin-ux-err-pattern = Ta wartość nie ma oczekiwanej postaci.
admin-ux-err-option = Wybierz jedną z wymienionych opcji.
admin-ux-err-out-of-range = Serwer odrzucił tę wartość, ponieważ wykracza poza dozwolony zakres.
admin-ux-err-too-long = Serwer odrzucił tę wartość, ponieważ jest za długa.
admin-ux-err-invalid = Serwer odrzucił tę wartość. Sprawdź ją i zapisz ponownie.
admin-ux-err-pinned = Ta wartość jest ustawiona w środowisku serwera i nie można jej tutaj zmienić.
admin-ux-err-locked = Ta wartość jest zablokowana na szerszym poziomie i nie można jej tutaj zmienić.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Domyślna
admin-ux-source-stored = Zapisana
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Ustawiona przez { $name }
admin-ux-source-env-fallback = Wartość zastępcza z { $name }
admin-ux-source-build = Ustawiona podczas kompilacji
# $scope — "global" or a domain name.
admin-ux-source-locked = Zablokowana przez { $scope }
admin-ux-source-read-only = Tylko do odczytu
admin-ux-source-restart = Wymaga ponownego uruchomienia
admin-ux-source-env-pin-reason = Środowisko serwera ustawia tę wartość przez { $name }. Zmień ją tam i uruchom serwer ponownie.
admin-ux-source-build-reason = Ta wartość jest ustalana podczas kompilacji serwera.
admin-ux-source-locked-reason = Blokada na poziomie { $scope } ustala tę wartość. Aby ją zmienić, usuń tam blokadę.
admin-ux-source-read-only-reason = Konsola pokazuje tę wartość, ale nie może jej zmienić. Dokumentacja podaje, gdzie jest ustawiana.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Ostatnio zmienione przez { $who }, { $time }
admin-ux-history-changed-by = Ostatnio zmienione przez { $who }
admin-ux-history-changed-at = Ostatnio zmienione: { $time }
admin-ux-history-never = Nie zmieniano w konsoli
admin-ux-history-link = Historia

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = Domyślnie: { $value }
admin-ux-revert = Przywróć domyślną
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Przywrócono wartość domyślną ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } zmienił(a) to { $time } w trakcie edycji.
admin-ux-stale-by = { $who } zmienił(a) to w trakcie edycji.
admin-ux-stale-unknown = Ktoś inny zmienił to w trakcie edycji.
admin-ux-stale-choice = Wczytaj ponownie, aby zobaczyć tamtą wersję i utracić swoje zmiany, albo zachowaj swoje i zapisz je w miejsce tamtych.
admin-ux-stale-reload = Wczytaj ponownie
admin-ux-stale-keep = Zachowaj moje

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Sprawdzanie, na co wpływa ta zmiana…
admin-ux-impact-heading = Ta zmiana wpływa na
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] i 1 więcej
        [few] i { $count } więcej
        [many] i { $count } więcej
       *[other] i { $count } więcej
    }
admin-ux-impact-none = Nic więcej się nie zmienia.
admin-ux-impact-unavailable = Podgląd wpływu niedostępny — serwer nie zgłosił, na co wpływa ta zmiana.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filtr
admin-ux-stream-pause = Wstrzymaj
admin-ux-stream-resume = Wznów
admin-ux-stream-clear = Wyczyść
admin-ux-stream-announce = Odczytuj nowe wpisy na głos
admin-ux-stream-connecting = Łączenie…
admin-ux-stream-live = Na żywo
admin-ux-stream-ended = Zakończono
admin-ux-stream-lost = Utracono połączenie
admin-ux-stream-reconnect = Połącz ponownie
admin-ux-stream-paused =
    { $count ->
        [0] Wstrzymano
        [one] Wstrzymano — 1 nowy wpis oczekuje
        [few] Wstrzymano — { $count } nowe wpisy oczekują
        [many] Wstrzymano — { $count } nowych wpisów oczekuje
       *[other] Wstrzymano — { $count } nowego wpisu oczekuje
    }
admin-ux-stream-no-match = Żaden wpis nie pasuje do filtra.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Pole

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Wygląd
admin-ux-appearance-title = Wygląd konsoli
admin-ux-appearance-note = Te ustawienia zmieniają tylko konsolę administracyjną, w tej przeglądarce.
admin-ux-appearance-following = Konsola używa motywu poczty webowej z tej przeglądarki, dopóki nie dokonasz wyboru tutaj. Te ustawienia zmieniają tylko konsolę administracyjną, w tej przeglądarce.
admin-ux-appearance-mode = Tryb
admin-ux-appearance-mode-light = Jasny
admin-ux-appearance-mode-dark = Ciemny
admin-ux-appearance-mode-system = Zgodnie z systemem
admin-ux-appearance-theme = Motyw
admin-ux-appearance-accent = Kolor akcentu
admin-ux-appearance-light = Jasny
admin-ux-appearance-dark = Ciemny
admin-ux-appearance-reset = Użyj motywu poczty webowej
