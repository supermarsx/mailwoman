# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: about. Source: ../en/about.ftl

about-tp-label = Biblioteki zewnętrzne i licencje
about-tp-search = Szukaj według nazwy, wersji lub licencji
about-tp-filter = Filtruj według licencji
about-tp-filter-all = Wszystkie ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Brak pasujących pozycji.
        [one] Wyświetlono { $shown } z { $total } pozycji.
        [few] Wyświetlono { $shown } z { $total } pozycji.
        [many] Wyświetlono { $shown } z { $total } pozycji.
       *[other] Wyświetlono { $shown } z { $total } pozycji.
    }
about-tp-empty = Nic nie pasuje do tego wyszukiwania i licencji.
about-tp-clear = Wyczyść wyszukiwanie i filtr
about-tp-more = Pokaż więcej (pozostało: { $n })

about-tp-kind-crate = Paczka Rust (crate)
about-tp-kind-npm = Pakiet npm
about-tp-kind-font = Czcionka
about-tp-kind-icons = Ikony
about-tp-kind-image = Obrazy
about-tp-kind-os = Pakiet obrazu

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Paczki Rust w serwerze, aplikacji webowej i wtyczkach
about-tp-group-shells = Paczki Rust tylko w aplikacjach desktopowych i mobilnych
about-tp-group-web = Pakiety npm w aplikacji webowej
about-tp-group-assets = Czcionki, ikony i obrazy
about-tp-group-image = Pliki obrazu serwera
about-tp-group-build = Narzędzia kompilacji: uruchamiane podczas budowania, nie są dostarczane

about-tp-licence = Licencja:
about-tp-licence-none = nie zadeklarowano
about-tp-licence-in-file = zobacz plik copyright
about-tp-used-in = Używane w: { $parts }
about-tp-artifact-server = serwer
about-tp-artifact-web = aplikacja webowa
about-tp-artifact-plugins = wtyczki
about-tp-artifact-desktop = aplikacja desktopowa
about-tp-artifact-mobile = aplikacja mobilna
about-tp-note-bundled = Dołączone do aplikacji webowej.
about-tp-note-fonts-pull = Niedołączone: pobierane przez operatora serwera poleceniem `mailwoman fonts pull`.
about-tp-note-first-party = Stworzone dla Mailwoman, na własnej licencji projektu.
about-tp-note-static-libc = Statycznie dołączone do plików binarnych serwera w obrazie FROM-scratch.
about-tp-note-alpine-ca = Pakiet certyfikatów CA obrazu FROM-scratch, pobrany z Alpine Linux.
about-tp-in-images = W: { $images }
about-tp-image-runtime = obraz uruchomieniowy distroless
about-tp-image-static = obraz FROM-scratch

about-tp-repository = Kod źródłowy
about-tp-homepage = Strona domowa
about-tp-new-tab = dla { $name } (otwiera się w nowej karcie)

about-tp-text-toggle =
    { $count ->
        [one] Tekst licencji
        [few] Teksty licencji ({ $count })
        [many] Teksty licencji ({ $count })
       *[other] Teksty licencji ({ $count })
    }
about-tp-text-open-for = Tekst licencji dla { $name }
about-tp-dialog-title = Tekst licencji: { $name } { $version }
about-tp-close = Zamknij
about-tp-text-loading = Wczytywanie tekstu licencji…
about-tp-text-error = Nie udało się wczytać tekstów licencji.
about-tp-retry = Spróbuj ponownie
about-tp-text-missing = Brak tego tekstu na liście. Wygeneruj go ponownie za pomocą scripts/about/gen-third-party.mjs.
about-tp-text-label = Tekst licencji { $name }: { $file }
about-tp-standard = Licencja { $licence }
about-tp-standard-note = (tekst standardowy, brak pliku w pakiecie)
about-tp-no-text = Dla tej pozycji nie jest dostępny tekst licencji.
