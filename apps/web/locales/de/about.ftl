# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: about. Source: ../en/about.ftl

about-tp-label = Bibliotheken von Drittanbietern und Lizenzen
about-tp-search = Nach Name, Version oder Lizenz suchen
about-tp-filter = Nach Lizenz filtern
about-tp-filter-all = Alle ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Keine Einträge gefunden.
        [one] { $shown } von { $total } Eintrag angezeigt.
       *[other] { $shown } von { $total } Einträgen angezeigt.
    }
about-tp-empty = Nichts entspricht dieser Suche und Lizenz.
about-tp-clear = Suche und Filter zurücksetzen
about-tp-more = Mehr anzeigen (noch { $n })

about-tp-kind-crate = Rust-Crate
about-tp-kind-npm = npm-Paket
about-tp-kind-font = Schriftart
about-tp-kind-icons = Symbole
about-tp-kind-image = Bilder
about-tp-kind-os = Image-Paket

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Rust-Crates im Server, in der Web-App und in den Plugins
about-tp-group-shells = Rust-Crates nur in den Desktop- und Mobil-Apps
about-tp-group-web = npm-Pakete in der Web-App
about-tp-group-assets = Schriftarten, Symbole und Bilder
about-tp-group-image = Dateien des Server-Images
about-tp-group-build = Build-Werkzeuge: laufen beim Bauen, werden nicht ausgeliefert

about-tp-licence = Lizenz:
about-tp-licence-none = nicht angegeben
about-tp-licence-in-file = siehe Copyright-Datei
about-tp-used-in = Verwendet in: { $parts }
about-tp-artifact-server = Server
about-tp-artifact-web = Web-App
about-tp-artifact-plugins = Plugins
about-tp-artifact-desktop = Desktop-App
about-tp-artifact-mobile = Mobil-App
about-tp-note-bundled = In der Web-App enthalten.
about-tp-note-fonts-pull = Nicht enthalten: wird vom Serverbetreiber mit `mailwoman fonts pull` heruntergeladen.
about-tp-note-first-party = Für Mailwoman erstellt, unter der eigenen Lizenz des Projekts.
about-tp-note-static-libc = In die Server-Binärdateien des FROM-scratch-Images gelinkt.
about-tp-note-alpine-ca = Das CA-Zertifikatsbündel des FROM-scratch-Images, übernommen aus Alpine Linux.
about-tp-in-images = In: { $images }
about-tp-image-runtime = dem Distroless-Laufzeit-Image
about-tp-image-static = dem FROM-scratch-Image

about-tp-repository = Quellcode
about-tp-homepage = Homepage
about-tp-new-tab = für { $name } (öffnet in einem neuen Tab)

about-tp-text-toggle =
    { $count ->
        [one] Lizenztext
       *[other] Lizenztexte ({ $count })
    }
about-tp-text-open-for = Lizenztext für { $name }
about-tp-dialog-title = Lizenztext: { $name } { $version }
about-tp-close = Schließen
about-tp-text-loading = Lizenztext wird geladen…
about-tp-text-error = Die Lizenztexte konnten nicht geladen werden.
about-tp-retry = Erneut versuchen
about-tp-text-missing = Dieser Text fehlt in der Liste. Erzeugen Sie ihn mit scripts/about/gen-third-party.mjs neu.
about-tp-text-label = Lizenztext von { $name }: { $file }
about-tp-standard = { $licence }-Lizenz
about-tp-standard-note = (Standardtext, keine Datei im Paket)
about-tp-no-text = Für diesen Eintrag ist kein Lizenztext verfügbar.
