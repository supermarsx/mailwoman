# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Użytkownicy. Nagłówki kolumn sortują listę.
admin-userlist-search = Szukaj użytkowników
admin-userlist-search-placeholder = Nazwa, adres, alias lub domena
admin-userlist-count = { $count ->
    [one] { $count } użytkownik
    [few] { $count } użytkowników
    [many] { $count } użytkowników
   *[other] { $count } użytkownika
}
admin-userlist-count-filtered = { $count ->
    [one] { $count } z { $total } użytkowników pasuje
    [few] { $count } z { $total } użytkowników pasuje
    [many] { $count } z { $total } użytkowników pasuje
   *[other] { $count } z { $total } użytkowników pasuje
}
admin-userlist-range = { $from }–{ $to } z { $total }
admin-userlist-loading = Wczytywanie użytkowników…
admin-userlist-retry = Spróbuj ponownie
admin-userlist-empty-filtered = Żaden użytkownik nie pasuje do tego wyszukiwania i filtrów.
admin-userlist-compact = Zwarte wiersze
admin-userlist-columns = Kolumny
admin-userlist-columns-legend = Pokaż kolumny
admin-userlist-actions = Akcje
admin-userlist-paging = Strony użytkowników
admin-userlist-page-size = Wierszy na stronę
admin-userlist-prev = Poprzednia
admin-userlist-next = Następna
admin-userlist-sort-by = Sortuj według
admin-userlist-sort-asc = { $column }, rosnąco
admin-userlist-sort-desc = { $column }, malejąco
admin-userlist-col-user = Użytkownik
admin-userlist-col-domain = Domena
admin-userlist-col-status = Stan
admin-userlist-col-2fa = Uwierzytelnianie dwuskładnikowe
admin-userlist-col-role = Rola w konsoli
admin-userlist-col-last-signin = Ostatnie logowanie
admin-userlist-col-last-seen = Ostatnio widziany
admin-userlist-col-created = Utworzono
admin-userlist-col-quota = Limit
admin-userlist-col-profile = Profil funkcji
admin-userlist-col-linked = Połączone skrzynki
admin-userlist-state-active = Aktywny
admin-userlist-state-pending = Nigdy nie logował się
admin-userlist-state-blocked = Zablokowany
admin-userlist-state-disabled = Wyłączony
admin-userlist-state-deleted = Usunięty
admin-userlist-2fa-passkey = Klucz dostępu
admin-userlist-2fa-totp = Aplikacja uwierzytelniająca
admin-userlist-2fa-none = Brak
admin-userlist-signin-never = Nigdy
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Nie zarejestrowano aktywności
admin-userlist-seen-at = Ostatnio widziany { $relative } ({ $exact })
admin-userlist-quota-unlimited = Bez limitu
admin-userlist-profile-default = Domyślny
admin-userlist-profile-via-user = Przypisany temu użytkownikowi
admin-userlist-profile-via-group = Z grupy
admin-userlist-profile-via-domain = Z domeny
admin-userlist-more = Więcej akcji
admin-userlist-more-for = Więcej akcji dla { $account }
admin-userlist-select = Zaznacz { $account }
admin-userlist-select-page = Zaznacz wszystkich użytkowników na tej stronie
admin-userlist-filters = Filtry
admin-userlist-add-filter = Dodaj filtr
admin-userlist-clear-all = Wyczyść wszystko
admin-userlist-chip-edit = Edytuj filtr { $label }
admin-userlist-chip-remove = Usuń filtr { $label }
admin-userlist-filter-domain = Domena
admin-userlist-filter-status = Stan
admin-userlist-filter-tfa = Uwierzytelnianie dwuskładnikowe
admin-userlist-filter-role = Rola w konsoli
admin-userlist-filter-profile = Profil funkcji
admin-userlist-filter-signin = Ostatnie logowanie
admin-userlist-filter-created = Utworzono
admin-userlist-filter-no-choices = Nie ma jeszcze z czego wybierać.
admin-userlist-filter-remove = Usuń filtr
admin-userlist-apply = Zastosuj
admin-userlist-cancel = Anuluj
admin-userlist-role-any = Dowolna rola w konsoli
admin-userlist-role-none = Brak roli w konsoli
admin-userlist-profile-none = Brak profilu (domyślny)
admin-userlist-signin-older = { $days ->
    [one] Ponad { $days } dzień temu
    [few] Ponad { $days } dni temu
    [many] Ponad { $days } dni temu
   *[other] Ponad { $days } dnia temu
}
admin-userlist-signin-within = { $days ->
    [one] W ostatnim { $days } dniu
    [few] W ostatnich { $days } dniach
    [many] W ostatnich { $days } dniach
   *[other] W ostatnich { $days } dniach
}
admin-userlist-signin-custom = Więcej niż tyle dni temu:
admin-userlist-signin-days = Dni
admin-userlist-days-invalid = Podaj całkowitą liczbę dni od 1 do 36500.
admin-userlist-created-from = Od
admin-userlist-created-to = Do
admin-userlist-created-between = od { $from } do { $to }
admin-userlist-created-since = od { $date }
admin-userlist-created-until = do { $date }
admin-userlist-dates-invalid = Data początkowa jest późniejsza niż data końcowa.
admin-userlist-views = Zapisane widoki
admin-userlist-view-save = Zapisz ten widok…
admin-userlist-view-name = Nazwa widoku
admin-userlist-view-save-confirm = Zapisz widok
admin-userlist-view-delete = Usuń widok „{ $name }”
admin-userlist-bulk = Akcje dla zaznaczonych użytkowników
admin-userlist-selected = { $count ->
    [one] Zaznaczono { $count }
    [few] Zaznaczono { $count }
    [many] Zaznaczono { $count }
   *[other] Zaznaczono { $count }
}
admin-userlist-clear-selection = Wyczyść zaznaczenie
admin-userlist-block = Zablokuj logowanie
admin-userlist-unblock = Odblokuj logowanie
admin-userlist-sign-out = Wyloguj wszędzie
admin-userlist-delete = Usuń dostęp lokalny
admin-userlist-bulk-profile = Przypisz profil funkcji
admin-userlist-bulk-profile-clear = Usuń własny profil użytkownika
admin-userlist-confirm-skipped = { $count ->
    [one] Zaznaczony użytkownik ({ $count }) zostanie pominięty: akcja go nie dotyczy.
    [few] Zaznaczeni użytkownicy ({ $count }) zostaną pominięci: akcja ich nie dotyczy.
    [many] Zaznaczeni użytkownicy ({ $count }) zostaną pominięci: akcja ich nie dotyczy.
   *[other] Zaznaczeni użytkownicy ({ $count }) zostaną pominięci: akcja ich nie dotyczy.
}
admin-userlist-confirm-block = { $count ->
    [one] Zablokować logowanie dla { $count } użytkownika?
    [few] Zablokować logowanie dla { $count } użytkowników?
    [many] Zablokować logowanie dla { $count } użytkowników?
   *[other] Zablokować logowanie dla { $count } użytkownika?
}
admin-userlist-confirm-block-one = Zablokować logowanie dla { $account }?
admin-userlist-confirm-block-body = Nowe logowania są odrzucane, a bieżące sesje Mailwoman kończą się. Skrzynka nadrzędna nie jest zmieniana.
admin-userlist-confirm-block-action = Zablokuj logowanie
admin-userlist-confirm-unblock = { $count ->
    [one] Odblokować logowanie dla { $count } użytkownika?
    [few] Odblokować logowanie dla { $count } użytkowników?
    [many] Odblokować logowanie dla { $count } użytkowników?
   *[other] Odblokować logowanie dla { $count } użytkownika?
}
admin-userlist-confirm-unblock-one = Odblokować logowanie dla { $account }?
admin-userlist-confirm-unblock-body = Lokalna blokada logowania zostanie usunięta. Pozostałe ograniczenia bezpieczeństwa nadal obowiązują.
admin-userlist-confirm-unblock-action = Odblokuj logowanie
admin-userlist-confirm-signout = { $count ->
    [one] Wylogować wszędzie { $count } użytkownika?
    [few] Wylogować wszędzie { $count } użytkowników?
    [many] Wylogować wszędzie { $count } użytkowników?
   *[other] Wylogować wszędzie { $count } użytkownika?
}
admin-userlist-confirm-signout-one = Wylogować { $account } wszędzie?
admin-userlist-confirm-signout-body = Wszystkie otwarte sesje Mailwoman tych użytkowników kończą się. Mogą zalogować się ponownie.
admin-userlist-confirm-signout-action = Wyloguj
admin-userlist-confirm-delete = { $count ->
    [one] Usunąć dostęp lokalny dla { $count } użytkownika?
    [few] Usunąć dostęp lokalny dla { $count } użytkowników?
    [many] Usunąć dostęp lokalny dla { $count } użytkowników?
   *[other] Usunąć dostęp lokalny dla { $count } użytkownika?
}
admin-userlist-confirm-delete-one = Usunąć dostęp lokalny dla { $account }?
admin-userlist-confirm-delete-body = Rekord usuniętego dostępu jest zachowywany, aby użytkownik nie został automatycznie zarejestrowany ponownie. Ustawienia Mailwoman, historia i skrzynka nadrzędna pozostają zapisane. Wpisz adres, aby potwierdzić.
admin-userlist-confirm-delete-action = Usuń dostęp lokalny
admin-userlist-confirm-profile = { $count ->
    [one] Zmienić profil funkcji { $count } użytkownika?
    [few] Zmienić profil funkcji { $count } użytkowników?
    [many] Zmienić profil funkcji { $count } użytkowników?
   *[other] Zmienić profil funkcji { $count } użytkownika?
}
admin-userlist-confirm-profile-one = Zmienić profil funkcji użytkownika { $account }?
admin-userlist-confirm-profile-body = Profil jest przypisywany każdemu użytkownikowi bezpośrednio i obowiązuje od jego następnego żądania. Obsługiwane są natywne tożsamości skrzynek, proxy oraz SSO.
admin-users-profile-help = Wybierz Domyślny, aby usunąć bezpośrednie przypisanie tego użytkownika i użyć profili dziedziczonych z jego grup lub domeny.
admin-users-profile-ambiguous = Ten użytkownik ma wiele tożsamości logowania. Wybierz właściwą tożsamość w Profilach funkcji.
admin-users-profile-unavailable = Brak zweryfikowanej tożsamości skrzynki. Ten użytkownik musi się zalogować, zanim będzie można przypisać profil.
admin-userlist-confirm-profile-action = Zmień profil
admin-userlist-bulk-done-block = { $count ->
    [one] Zablokowano logowanie dla { $count } użytkownika.
    [few] Zablokowano logowanie dla { $count } użytkowników.
    [many] Zablokowano logowanie dla { $count } użytkowników.
   *[other] Zablokowano logowanie dla { $count } użytkownika.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Odblokowano logowanie dla { $count } użytkownika.
    [few] Odblokowano logowanie dla { $count } użytkowników.
    [many] Odblokowano logowanie dla { $count } użytkowników.
   *[other] Odblokowano logowanie dla { $count } użytkownika.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] Wylogowano { $count } użytkownika.
    [few] Wylogowano { $count } użytkowników.
    [many] Wylogowano { $count } użytkowników.
   *[other] Wylogowano { $count } użytkownika.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Usunięto dostęp lokalny dla { $count } użytkownika.
    [few] Usunięto dostęp lokalny dla { $count } użytkowników.
    [many] Usunięto dostęp lokalny dla { $count } użytkowników.
   *[other] Usunięto dostęp lokalny dla { $count } użytkownika.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Zmieniono profil funkcji { $count } użytkownika.
    [few] Zmieniono profil funkcji { $count } użytkowników.
    [many] Zmieniono profil funkcji { $count } użytkowników.
   *[other] Zmieniono profil funkcji { $count } użytkownika.
}
admin-userlist-bulk-failed = Wykonano: { $done }, niepowodzeń: { $failed }: { $reason }
