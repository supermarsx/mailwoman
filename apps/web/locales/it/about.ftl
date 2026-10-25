# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: about. Source: ../en/about.ftl

about-tp-label = Librerie di terze parti e licenze
about-tp-search = Cerca per nome, versione o licenza
about-tp-filter = Filtra per licenza
about-tp-filter-all = Tutte ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Nessuna voce corrispondente.
        [one] { $shown } di { $total } voce mostrata.
       *[other] { $shown } di { $total } voci mostrate.
    }
about-tp-empty = Nessun risultato per questa ricerca e licenza.
about-tp-clear = Cancella ricerca e filtro
about-tp-more = Mostra altri ({ $n } rimanenti)

about-tp-kind-crate = Crate Rust
about-tp-kind-npm = Pacchetto npm
about-tp-kind-font = Font
about-tp-kind-icons = Icone
about-tp-kind-image = Immagini
about-tp-kind-os = Pacchetto dell'immagine

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Crate Rust nel server, nell'app web e nei plugin
about-tp-group-shells = Crate Rust solo nelle app desktop e mobile
about-tp-group-web = Pacchetti npm nell'app web
about-tp-group-assets = Font, icone e immagini
about-tp-group-image = File dell'immagine del server
about-tp-group-build = Strumenti di build: usati durante la build, non distribuiti

about-tp-licence = Licenza:
about-tp-licence-none = non dichiarata
about-tp-licence-in-file = vedere il file copyright
about-tp-used-in = Usato in: { $parts }
about-tp-artifact-server = server
about-tp-artifact-web = app web
about-tp-artifact-plugins = plugin
about-tp-artifact-desktop = app desktop
about-tp-artifact-mobile = app mobile
about-tp-note-bundled = Incluso nell'app web.
about-tp-note-fonts-pull = Non incluso: scaricato dall'operatore del server con `mailwoman fonts pull`.
about-tp-note-first-party = Realizzato per Mailwoman, con la licenza del progetto stesso.
about-tp-note-static-libc = Collegato ai binari del server dell'immagine FROM-scratch.
about-tp-note-alpine-ca = Il bundle dei certificati CA dell'immagine FROM-scratch, preso da Alpine Linux.
about-tp-in-images = In: { $images }
about-tp-image-runtime = l'immagine runtime distroless
about-tp-image-static = l'immagine FROM-scratch

about-tp-repository = Codice sorgente
about-tp-homepage = Sito web
about-tp-new-tab = per { $name } (si apre in una nuova scheda)

about-tp-text-toggle =
    { $count ->
        [one] Testo della licenza
       *[other] Testi delle licenze ({ $count })
    }
about-tp-text-open-for = Testo della licenza per { $name }
about-tp-dialog-title = Testo della licenza: { $name } { $version }
about-tp-close = Chiudi
about-tp-text-loading = Caricamento del testo della licenza…
about-tp-text-error = Impossibile caricare i testi delle licenze.
about-tp-retry = Riprova
about-tp-text-missing = Questo testo manca dall'elenco. Rigenerarlo con scripts/about/gen-third-party.mjs.
about-tp-text-label = Testo della licenza di { $name }: { $file }
about-tp-standard = Licenza { $licence }
about-tp-standard-note = (testo standard, nessun file nel pacchetto)
about-tp-no-text = Nessun testo di licenza disponibile per questa voce.
