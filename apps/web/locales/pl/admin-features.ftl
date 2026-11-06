# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Profile funkcji
admin-features-intro = Profil to nazwany zestaw przełączników funkcji. Przypisz go do domeny, grupy lub użytkownika. Przypisania indywidualne obsługują natywne skrzynki oraz zweryfikowane tożsamości proxy lub SSO; kontrolują dostęp do funkcji Mailwoman.
admin-features-new = Nowy profil
admin-features-search = Szukaj profili
admin-features-empty = Brak profili funkcji.
admin-features-no-match = Żaden profil nie pasuje do wyszukiwania.
admin-features-load-error = Nie udało się wczytać katalogu funkcji.
admin-features-not-loaded = Katalog funkcji nie został jeszcze wczytany. Wczytaj stronę ponownie i spróbuj jeszcze raz.
admin-features-stale = Inny administrator zmienił katalog funkcji. Został wczytany ponownie; sprawdź wartości i zapisz jeszcze raz.
admin-features-not-found = Brak profilu o identyfikatorze { $id }.
admin-features-col-name = Nazwa
admin-features-col-values = Przełączniki
admin-features-col-assigned = Przypisano do
admin-features-values-count = { $count ->
    [one] Ustawiono { $count } przełącznik
    [few] Ustawiono { $count } przełączniki
    [many] Ustawiono { $count } przełączników
   *[other] Ustawiono { $count } przełącznika
}
admin-features-assigned-count = { $count ->
    [0] Nieprzypisany
    [one] { $count } miejsce
    [few] { $count } miejsca
    [many] { $count } miejsc
   *[other] { $count } miejsca
}
admin-features-editor = Ustawienia profilu
admin-features-name = Nazwa profilu
admin-features-name-help = Widoczna tylko dla administratorów, do 128 znaków.
admin-features-name-invalid = Podaj nazwę o długości od 1 do 128 znaków.
admin-features-switches = Przełączniki funkcji
admin-features-switches-help = „Nie ustawiono” zostawia funkcję następnemu profilowi, domenie lub ustawieniu domyślnemu serwera. Zablokowanej wartości nie może zmienić użytkownik ani niższy poziom.
admin-features-col-feature = Funkcja
admin-features-col-value = Wartość
admin-features-col-lock = Blokada
admin-features-value-for = Wartość dla { $feature }
admin-features-unset = Nie ustawiono
admin-features-on = Dozwolone
admin-features-off = Niedozwolone
admin-features-lock = Zablokowane
admin-features-save = Zapisz profil
admin-features-create = Utwórz profil
admin-features-delete = Usuń profil
admin-features-delete-title = Usunąć profil { $name }?
admin-features-delete-body = { $count ->
    [0] Profil nie jest nigdzie przypisany. Jego przełączniki natychmiast przestają obowiązywać.
    [one] Profil zostanie usunięty z { $count } miejsca, do którego jest przypisany, a jego przełączniki natychmiast przestaną obowiązywać.
    [few] Profil zostanie usunięty z { $count } miejsc, do których jest przypisany, a jego przełączniki natychmiast przestaną obowiązywać.
    [many] Profil zostanie usunięty z { $count } miejsc, do których jest przypisany, a jego przełączniki natychmiast przestaną obowiązywać.
   *[other] Profil zostanie usunięty z { $count } miejsca, do którego jest przypisany, a jego przełączniki natychmiast przestaną obowiązywać.
}
admin-features-saved = Profil { $name } zapisany.
admin-features-deleted = Profil { $name } usunięty.
admin-features-assigned = Profil { $name } przypisany.
admin-features-unassigned = Przypisanie profilu { $name } usunięte.
admin-features-assignments = Przypisano do
admin-features-assignments-help = Profile obowiązują w kolejności priorytetu, wyższy pierwszy. Przy równym priorytecie przypisanie użytkownika obowiązuje przed grupą, a grupa przed domeną.
admin-features-assignments-empty = Ten profil nie jest jeszcze przypisany.
admin-features-assign-form = Przypisz ten profil
admin-features-col-scope = Dotyczy
admin-features-col-target = Nazwa
admin-features-col-priority = Priorytet
admin-features-col-actions = Akcje
admin-features-scope-domain = Domena
admin-features-scope-group = Grupa
admin-features-scope-user = Użytkownik
admin-features-assign = Przypisz
admin-features-unassign = Usuń
admin-features-no-domain = Nie skonfigurowano domen.
admin-features-no-group = Nie istnieją grupy użytkowników.
admin-features-no-user = Żadne konto skrzynki jeszcze się nie zalogowało.
admin-features-tools = Narzędzia profilu
admin-features-preset = Zacznij od ustawienia wstępnego
admin-features-preset-choose = Wybierz ustawienie wstępne
admin-features-preset-apply = Zastosuj ustawienie wstępne
admin-features-preset-applied = Zastosowano ustawienie wstępne { $name }. Przejrzyj wartości i zapisz.
admin-features-preset-full = Pełny
admin-features-preset-standard = Standardowy
admin-features-preset-mail-only = Tylko poczta
admin-features-preset-minimal = Minimalny / kiosk
admin-features-import = Importuj JSON
admin-features-export = Eksportuj JSON
admin-features-exported = Profil wyeksportowany jako plik JSON.
admin-features-imported = { $count ->
    [one] Zaimportowano { $count } wartość. Przejrzyj ją i zapisz.
    [few] Zaimportowano { $count } wartości. Przejrzyj je i zapisz.
    [many] Zaimportowano { $count } wartości. Przejrzyj je i zapisz.
   *[other] Zaimportowano { $count } wartości. Przejrzyj je i zapisz.
}
admin-features-import-json = Plik nie jest prawidłowym JSON.
admin-features-import-format = Plik nie jest profilem funkcji Mailwoman.
admin-features-import-version = Ten plik profilu funkcji pochodzi z nowszej wersji i nie można go zaimportować.
admin-features-clone = Sklonuj
admin-features-clone-name = Kopia { $name }
admin-features-filter = Znajdź funkcję
admin-features-filter-placeholder = Nazwa, ustawienie lub klucz
admin-features-filter-none = Żadna funkcja nie pasuje do wyszukiwania.
admin-features-diff-summary = { $differs ->
    [one] { $differs } wartość różni się od ustawienia domyślnego serwera
    [few] { $differs } wartości różnią się od ustawienia domyślnego serwera
    [many] { $differs } wartości różni się od ustawienia domyślnego serwera
   *[other] { $differs } wartości różni się od ustawienia domyślnego serwera
} · { $same } powtarza je
admin-features-differs = Różni się od domyślnej
admin-features-default-is = Domyślnie: { $value }
admin-features-fix-values = Popraw wartości oznaczone jako nieprawidłowe przed zapisem.
admin-features-allow-all = Zezwól na wszystkie
admin-features-deny-all = Nie zezwalaj na żadną
admin-features-clear-all = Wyczyść
admin-features-group-allow = Zezwól na każdą funkcję w grupie { $group }
admin-features-group-deny = Nie zezwalaj na żadną funkcję w grupie { $group }
admin-features-group-clear = Wyczyść każdą wartość w grupie { $group }
admin-features-enforcement-ui = Ukryte tylko w aplikacji
admin-features-enforcement-partial = Niektóre limity nie są jeszcze zliczane
admin-features-other = Inne ustawienia, które może zawierać profil
admin-features-other-help = Każde inne ustawienie, które administrator może ustawić dla profilu. Zostaw pole puste, aby zachować wartość serwera.
admin-features-cat-compose = Poczta i redagowanie
admin-features-cat-reading = Czytanie i prywatność
admin-features-cat-views = Widoki i wyszukiwanie
admin-features-cat-organise = Porządkowanie
admin-features-cat-pim = Kalendarz, kontakty, zadania, notatki i klucze
admin-features-cat-data = Dane
admin-features-cat-accounts = Konta i dostęp
admin-features-cat-integrations = Integracje
admin-features-cat-developer = Dostęp dla programistów
admin-features-cat-personal = Personalizacja
admin-features-feature-compose-sending = Limity wysyłania
admin-features-feature-compose-sending-help = Liczba odbiorców na wiadomość jest odrzucana przez serwer po przekroczeniu limitu. Wiadomości na godzinę i na dobę są zapisywane dla profilu, ale nie są jeszcze zliczane.
admin-features-feature-storage = Limit miejsca
admin-features-feature-storage-help = Limit miejsca jest zapisywany dla profilu; własny limit serwera poczty nadal obowiązuje.
admin-features-feature-access-protocols = Protokoły dostępu do poczty
admin-features-feature-access-protocols-help = Jakimi protokołami serwera poczty ci użytkownicy mogą się logować, oddzielonymi przecinkami: imap, pop3, ews.
admin-features-feature-accounts-linked = Połączone skrzynki
admin-features-feature-calendar = Kalendarz
admin-features-feature-contacts = Kontakty
admin-features-feature-tasks = Zadania
admin-features-feature-notes = Notatki
admin-features-preview = Wartości efektywne
admin-features-preview-help = Do czego sprowadza się każda funkcja dla jednego użytkownika lub użytkownika jednej domeny, po uwzględnieniu każdego profilu, grupy, wartości domeny i blokady.
admin-features-preview-kind = Podgląd dla
admin-features-preview-target = Kto
admin-features-preview-show = Pokaż
admin-features-preview-loading = Wczytywanie…
admin-features-preview-state = Efektywna
admin-features-preview-from = Rozstrzyga
admin-features-preview-profiles = Obowiązujące profile, pierwszy wygrywa: { $names }
admin-features-preview-no-profiles = Nie obowiązuje żaden profil; rozstrzygają wartości domeny i serwera.
admin-features-source-default = Wbudowana wartość domyślna
admin-features-source-env-fallback = Wartość domyślna ze środowiska
admin-features-source-env-pin = Środowisko (przypięte)
admin-features-source-global = Wartość serwera
admin-features-source-domain = Wartość domeny
admin-features-source-profile = Profil
admin-features-source-profile-named = Profil { $name }
admin-features-source-group = Wartość grupy
admin-features-source-admin-user = Wartość ustawiona dla tego użytkownika
admin-features-source-user = Własny wybór użytkownika
admin-features-source-locked = { $source } (zablokowane)
admin-retention-col-category = Kategoria
admin-retention-col-kept = Przechowywane
admin-retention-col-change = Zmiana
admin-retention-cat-default = Domyślna
admin-retention-cat-admin = Akcje administratora
admin-retention-cat-security = Zdarzenia bezpieczeństwa
admin-retention-cat-alert = Alerty użytkowników
admin-retention-forever = Każdy wpis
admin-retention-days = { $days ->
    [one] { $days } dzień
    [few] { $days } dni
    [many] { $days } dni
   *[other] { $days } dnia
}
admin-retention-follows-default = Domyślnie: { $kept }
admin-retention-hold-on = Blokada prawna
admin-retention-read-only = Zmiana retencji wymaga uprawnienia settings.write.
admin-retention-edit-for = Retencja dla kategorii { $category }
admin-retention-keep = Przechowuj
admin-retention-mode-inherit = Stosuj wartość domyślną
admin-retention-mode-forever = Każdy wpis
admin-retention-mode-days = Przez określoną liczbę dni
admin-retention-days-label = Dni
admin-retention-hold = Blokada prawna
admin-retention-save = Zapisz
admin-retention-bounds = Podaj liczbę całkowitą od { $min } do { $max }.
admin-retention-saved = Retencja dla kategorii { $category } zapisana.
admin-retention-task-none = Brak zadania usuwania. Zostanie utworzone, codziennie o 03:15 UTC, gdy po raz pierwszy zapiszesz liczbę dni.
admin-retention-task-disabled = Zaplanowane zadanie usuwania jest wyłączone; wpisy są usuwane tylko przez Usuń teraz.
admin-retention-task-next = Następne zaplanowane usuwanie: { $when }.
admin-retention-task-daily = Usuwanie codziennie o 03:15 UTC.
admin-retention-last = Ostatnie usuwanie, które usunęło wpisy: { $when }.
admin-retention-prune = Usuń teraz
admin-retention-prune-title = Usunąć teraz wpisy z dziennika audytu?
admin-retention-prune-body = Wpisy starsze niż retencja ich kategorii zostaną usunięte. Kategorie, które zachowują każdy wpis lub mają blokadę prawną, nie są zmieniane. Tej operacji nie można cofnąć.
admin-retention-pruned = { $count ->
    [0] Żaden wpis nie był wystarczająco stary, aby go usunąć.
    [one] Usunięto { $count } wpis.
    [few] Usunięto { $count } wpisy.
    [many] Usunięto { $count } wpisów.
   *[other] Usunięto { $count } wpisu.
}
admin-auth-attempted = Próbowano jako { $name }
admin-features-feature-compose-open-tracking = Śledzenie otwarć wiadomości
admin-features-feature-compose-open-tracking-help = Zezwala na rejestrowanie otwarć kwalifikujących się sformatowanych wiadomości za pomocą obrazu śledzącego serwera. Wymaga skonfigurowanego publicznego adresu URL.
admin-features-feature-reading-original-html = Widok oryginalnego HTML
admin-features-feature-reading-original-html-help = Zezwala użytkownikom na włączenie oryginalnego HTML dla pojedynczych wiadomości, w izolowanej ramce czytnika. Domyślnie pozostaje oczyszczony HTML.
admin-features-inherit = Dziedzicz
admin-features-override = Ustaw wartość
admin-features-setting-mode = Tryb wartości dla { $setting }
admin-features-inherit-help = Dziedziczenie usuwa wartość i blokadę tego profilu. O wyniku decyduje następny obowiązujący profil, domena lub wartość serwera; podgląd efektywny pokazuje wynik i jego źródło.
admin-features-domain-alias = { $alias } (alias domeny { $domain })
admin-features-domain-aliases = { $domain } (aliasy: { $aliases })

# -- Machine translation (2026-11): added to match en ---------------------
admin-retention-title = Retencja dziennika audytu
admin-retention-intro = Jak długo przechowywane są wpisy audytu, według kategorii. Kategoria bez własnej wartości stosuje wartość domyślną; bez wartości domyślnej zachowywany jest każdy wpis. Blokada prawna wstrzymuje usuwanie dla swojej kategorii, a blokada na wartości domyślnej wstrzymuje je dla wszystkich. Zmiany i usunięcia są zapisywane w dzienniku audytu.
admin-retention-loading = Wczytywanie zasad retencji…
admin-retention-unavailable = Retencja audytu nie jest dostępna na tym serwerze.
admin-retention-never = Nie usunięto jeszcze żadnych wpisów.
