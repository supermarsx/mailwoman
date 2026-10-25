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
contacts-favorites = ★ Ulubione
contacts-groups = Grupy
contacts-new-group = Nowa grupa
contacts-new-group-name = Nazwa nowej grupy
contacts-group-name-placeholder = Nazwa grupy
contacts-create = Utwórz
contacts-data = Dane
contacts-import = Importuj…

# -- Business card -----------------------------------------------------------
contacts-card = Kontakt { $name }
contacts-favorited = ★ W ulubionych
contacts-favorite-action = ☆ Dodaj do ulubionych
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
