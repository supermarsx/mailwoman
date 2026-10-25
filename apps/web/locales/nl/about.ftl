# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: about. Source: ../en/about.ftl

about-tp-label = Bibliotheken en licenties van derden
about-tp-search = Zoeken op naam, versie of licentie
about-tp-filter = Filteren op licentie
about-tp-filter-all = Alle ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Geen items gevonden.
        [one] { $shown } van { $total } item weergegeven.
       *[other] { $shown } van { $total } items weergegeven.
    }
about-tp-empty = Niets komt overeen met deze zoekopdracht en licentie.
about-tp-clear = Zoekopdracht en filter wissen
about-tp-more = Meer tonen (nog { $n })

about-tp-kind-crate = Rust-crate
about-tp-kind-npm = npm-pakket
about-tp-kind-font = Lettertype
about-tp-kind-icons = Pictogrammen
about-tp-kind-image = Afbeeldingen
about-tp-kind-os = Imagepakket

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Rust-crates in de server, de webapp en de plug-ins
about-tp-group-shells = Rust-crates die alleen in de desktop- en mobiele apps zitten
about-tp-group-web = npm-pakketten in de webapp
about-tp-group-assets = Lettertypen, pictogrammen en afbeeldingen
about-tp-group-image = Bestanden in het serverimage
about-tp-group-build = Buildtools: gebruikt tijdens het bouwen, niet meegeleverd

about-tp-licence = Licentie:
about-tp-licence-none = niet opgegeven
about-tp-licence-in-file = zie het copyrightbestand
about-tp-used-in = Gebruikt in: { $parts }
about-tp-artifact-server = server
about-tp-artifact-web = webapp
about-tp-artifact-plugins = plug-ins
about-tp-artifact-desktop = desktop-app
about-tp-artifact-mobile = mobiele app
about-tp-note-bundled = Meegeleverd met de webapp.
about-tp-note-fonts-pull = Niet meegeleverd: gedownload door de serverbeheerder met `mailwoman fonts pull`.
about-tp-note-first-party = Gemaakt voor Mailwoman, onder de eigen licentie van het project.
about-tp-note-static-libc = Gelinkt in de serverbinaries van het FROM-scratch-image.
about-tp-note-alpine-ca = De CA-certificaatbundel van het FROM-scratch-image, overgenomen uit Alpine Linux.
about-tp-in-images = In: { $images }
about-tp-image-runtime = het distroless-runtime-image
about-tp-image-static = het FROM-scratch-image

about-tp-repository = Broncode
about-tp-homepage = Startpagina
about-tp-new-tab = voor { $name } (opent in een nieuw tabblad)

about-tp-text-toggle =
    { $count ->
        [one] Licentietekst
       *[other] Licentieteksten ({ $count })
    }
about-tp-text-open-for = Licentietekst voor { $name }
about-tp-dialog-title = Licentietekst: { $name } { $version }
about-tp-close = Sluiten
about-tp-text-loading = Licentietekst laden…
about-tp-text-error = De licentieteksten konden niet worden geladen.
about-tp-retry = Opnieuw proberen
about-tp-text-missing = Deze tekst ontbreekt in de lijst. Genereer hem opnieuw met scripts/about/gen-third-party.mjs.
about-tp-text-label = Licentietekst van { $name }: { $file }
about-tp-standard = { $licence }-licentie
about-tp-standard-note = (standaardtekst, geen bestand in het pakket)
about-tp-no-text = Er is geen licentietekst beschikbaar voor dit item.
