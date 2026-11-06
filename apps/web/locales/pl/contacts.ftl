# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Kontakty
contacts-search = Szukaj kontaktów
contacts-new = Nowy kontakt
contacts-find-duplicates = Znajdź duplikaty
contacts-loading = Ładowanie…
contacts-none = Brak kontaktów.
contacts-list = Lista kontaktów
contacts-favorite = Dodaj do ulubionych: { $name }
contacts-export-vcard = Eksportuj vCard
contacts-export-csv = Eksportuj CSV
contacts-select-hint = Wybierz kontakt, aby zobaczyć jego wizytówkę.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Książki adresowe i grupy
contacts-address-books = Książki adresowe
contacts-all = Wszystkie kontakty
contacts-favorites = Ulubione
contacts-groups = Grupy
contacts-new-group = Nowa grupa
contacts-new-group-name = Nazwa nowej grupy
contacts-group-name-placeholder = Nazwa grupy
contacts-create = Utwórz
contacts-data = Dane
contacts-import = Importuj…

# -- Business card -----------------------------------------------------------
contacts-card = Kontakt { $name }
contacts-favorited = W ulubionych
contacts-favorite-action = Dodaj do ulubionych
contacts-email = E-mail
contacts-phone = Telefon
contacts-dates = Daty
contacts-groups-label = Grupy
contacts-notes = Notatki
contacts-security-key = Klucz bezpieczeństwa
contacts-key-on-file = Zapisano klucz/certyfikat (tylko do wyświetlenia, dopóki nie zostanie dodana obsługa PGP/S-MIME).
contacts-no-key = Brak zapisanego klucza.
contacts-directory-security = Bezpieczeństwo w katalogu
contacts-member-of = Członek grup
contacts-group-membership = Członkostwo: { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Nowy kontakt
contacts-edit-contact = Edytuj kontakt
contacts-full-name = Imię i nazwisko
contacts-given = Imię
contacts-given-placeholder = Imię
contacts-surname = Nazwisko
contacts-surname-placeholder = Nazwisko
contacts-organization = Organizacja
contacts-job-title = Stanowisko
contacts-job-title-placeholder = Stanowisko
contacts-email-n = E-mail { $n }
contacts-email-label = Etykieta adresu e-mail { $n }
contacts-email-label-placeholder = praca
contacts-remove-email = Usuń e-mail { $n }
contacts-add-email = Dodaj e-mail
contacts-phone-n = Telefon { $n }
contacts-remove-phone = Usuń telefon { $n }
contacts-add-phone = Dodaj telefon
contacts-security-key-label = Klucz bezpieczeństwa (nieprzetwarzany symbol zastępczy)
contacts-security-key-placeholder = Klucz PGP / certyfikat S-MIME (tylko do wyświetlenia)
contacts-favorite-word = Ulubiony

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Importuj kontakty
contacts-imported = Zaimportowano { $count ->
    [one] { $count } kontakt
    [few] { $count } kontakty
    [many] { $count } kontaktów
   *[other] { $count } kontaktu
}.
contacts-import-hint = Wklej vCard (.vcf) lub CSV albo wybierz plik. Format zostanie wykryty automatycznie.
contacts-import-file = Importuj plik
contacts-paste = Wklej vCard lub CSV
contacts-preview = Podgląd
contacts-map-columns = Przypisz kolumny
contacts-col = Kolumna
contacts-maps-to = Przypisana do
contacts-sample = Przykład
contacts-map-column = Przypisz kolumnę { $header }
contacts-preview-count = Podgląd ({ $count })
contacts-import-preview = Podgląd importu
contacts-no-name = (bez nazwy)
contacts-no-email = brak adresu e-mail
contacts-import-n = Importuj { $count ->
    [one] { $count } kontakt
    [few] { $count } kontakty
    [many] { $count } kontaktów
   *[other] { $count } kontaktu
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Scal duplikaty
contacts-merge-review-hint = Sprawdź scaloną wizytówkę. Pozostałe wizytówki zostaną oznaczone jako usunięte (można to cofnąć).
contacts-merged-preview = Podgląd po scaleniu
contacts-emails = Adresy e-mail
contacts-phones = Telefony
contacts-merge-confirm = Scal kontakty
contacts-no-duplicates = Nie znaleziono duplikatów.
contacts-possible-duplicates = Możliwe duplikaty: { $count }
contacts-review-merge = Sprawdź scalenie

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Dane kontaktu
contacts-address-recipients = Pokaż odbiorców
contacts-address-save = Zapisz kontakt
contacts-address-compose = Wyślij wiadomość
contacts-address-unsaved = Ten adres nie jest zapisany w Twoich kontaktach.
contacts-address-book = Książka adresowa
contacts-address-lookup-failed = Nie udało się wczytać informacji o kontakcie. Spróbuj ponownie.
contacts-address-save-failed = Nie udało się zapisać kontaktu. Spróbuj ponownie.
contacts-address-saved = Zapisano kontakt.
contacts-address-loading = Wczytywanie kontaktu…
contacts-editor-person = Imię i praca
contacts-editor-reach = E-mail i telefon
contacts-editor-other = Notatki i preferencje
contacts-editor-save-error = Nie udało się zapisać kontaktu. Twoje zmiany nadal tu są. Spróbuj ponownie.
contacts-group-editor-help = Utwórz grupę w książce adresowej, a następnie dodaj członków z ich kart kontaktów.
contacts-group-details = Szczegóły grupy
contacts-group-save-error = Nie udało się zapisać grupy. Twoje zmiany nadal tu są. Spróbuj ponownie.
contacts-edit-group = Edytuj grupę kontaktów
contacts-edit-named-group = Edytuj grupę { $name }
contacts-group-name = Nazwa grupy
contacts-group-edit-help = Zaktualizuj nazwę tej grupy. Jej książka adresowa i członkowie pozostaną bez zmian.
contacts-back-to-list = Wróć do kontaktów
contacts-membership-title = Członkostwo w grupie
contacts-membership-help = Wybierz grupę, a następnie dodaj lub usuń jej członków. Zmiany są zapisywane na bieżąco.
contacts-membership-no-groups = Nie ma jeszcze grup. Utwórz grupę, aby zacząć dodawać członków.
contacts-membership-pick = Wybierz grupę, aby zobaczyć jej członków.
contacts-membership-of = Członkowie grupy { $name }
contacts-membership-members = Członkowie
contacts-membership-add = Wpisz imię lub adres, aby dodać członka
contacts-membership-add-group = Wpisz nazwę grupy, aby dodać ten kontakt
contacts-membership-count = { $count ->
    [one] 1 członek
    [few] { $count } członków
    [many] { $count } członków
   *[other] { $count } członka
}
contacts-membership-failed = Nie udało się zapisać niektórych zmian członkostwa. Lista pokazuje zapisany stan.
contacts-membership-manage = Zarządzaj członkostwem w grupie
contacts-import-source = Źródło
contacts-import-book = Książka adresowa
contacts-import-review = Przegląd
contacts-import-empty = Wybierz plik lub wklej tekst vCard albo CSV, a następnie wybierz Podgląd. Znalezione kontakty pojawią się tutaj i możesz pominąć dowolny z nich.
contacts-merge-sets = Możliwe duplikaty
contacts-merge-set = Duplikaty kontaktu { $name }
contacts-merge-keep = Karta do zachowania
contacts-merge-done = Scalono z kontaktem { $name }.
contacts-address-actions = Akcje kontaktu
contacts-address-save-to = Zapisz w kontaktach
contacts-address-goto = Przejdź do kontaktu
contacts-address-copy = Kopiuj dane kontaktu
contacts-address-copied = Skopiowano dane kontaktu.
contacts-address-copy-failed = Nie udało się skopiować danych kontaktu.
contacts-share = Udostępnij kontakt
contacts-share-email = Wyślij kontakt e-mailem
contacts-share-copy = Kopiuj jako vCard
contacts-share-download = Pobierz .vcf
contacts-share-native = Udostępnij…
contacts-share-copied = Skopiowano vCard.
contacts-share-copy-failed = Nie udało się skopiować vCard.
contacts-share-downloaded = Pobrano vCard.
contacts-share-failed = Nie udało się udostępnić kontaktu.
contacts-share-attach-failed = Nie udało się dołączyć pliku .vcf, więc vCard znajduje się w treści wiadomości.
contacts-address-saved-favorite = Zapisano kontakt i dodano do ulubionych.
contacts-favorite-add = Dodaj do ulubionych
contacts-favorite-remove = Usuń z ulubionych
contacts-favorite-save = Zapisz i dodaj do ulubionych
contacts-favorite-added = Dodano { $name } do ulubionych.
contacts-favorite-removed = Usunięto { $name } z ulubionych.
contacts-favorite-failed = Nie udało się zaktualizować ulubionych.
contacts-select-row = Zaznacz: { $name }
contacts-selected-count = Zaznaczono: { $count }
contacts-select-clear = Wyczyść zaznaczenie
contacts-bulk-actions = Akcje dla zaznaczonych kontaktów
contacts-bulk-favorites-added = Dodano do ulubionych: { $count }.
contacts-bulk-favorites-removed = Usunięto z ulubionych: { $count }.
contacts-row-menu = Akcje dla: { $name }
contacts-favorites-empty = Brak ulubionych. Użyj gwiazdki przy kontakcie, aby dodać.
