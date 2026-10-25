# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: about. Source: ../en/about.ftl

about-tp-label = Bibliothèques tierces et licences
about-tp-search = Rechercher par nom, version ou licence
about-tp-filter = Filtrer par licence
about-tp-filter-all = Toutes ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Aucune entrée ne correspond.
        [one] { $shown } entrée affichée sur { $total }.
        [many] { $shown } entrées affichées sur { $total }.
       *[other] { $shown } entrées affichées sur { $total }.
    }
about-tp-empty = Rien ne correspond à cette recherche et à cette licence.
about-tp-clear = Effacer la recherche et le filtre
about-tp-more = Afficher plus ({ $n } restantes)

about-tp-kind-crate = Crate Rust
about-tp-kind-npm = Paquet npm
about-tp-kind-font = Police
about-tp-kind-icons = Icônes
about-tp-kind-image = Images
about-tp-kind-os = Paquet d’image

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Crates Rust du serveur, de l’application web et des plugins
about-tp-group-shells = Crates Rust des applications de bureau et mobiles uniquement
about-tp-group-web = Paquets npm de l’application web
about-tp-group-assets = Polices, icônes et images
about-tp-group-image = Fichiers de l’image serveur
about-tp-group-build = Outils de compilation : utilisés à la compilation, non distribués

about-tp-licence = Licence :
about-tp-licence-none = non déclarée
about-tp-licence-in-file = voir le fichier de copyright
about-tp-used-in = Utilisé dans : { $parts }
about-tp-artifact-server = serveur
about-tp-artifact-web = application web
about-tp-artifact-plugins = plugins
about-tp-artifact-desktop = application de bureau
about-tp-artifact-mobile = application mobile
about-tp-note-bundled = Inclus dans l’application web.
about-tp-note-fonts-pull = Non inclus : téléchargé par l’opérateur du serveur avec `mailwoman fonts pull`.
about-tp-note-first-party = Conçu pour Mailwoman, sous la licence propre du projet.
about-tp-note-static-libc = Lié aux binaires serveur de l’image FROM-scratch.
about-tp-note-alpine-ca = Le paquet de certificats CA de l’image FROM-scratch, repris d’Alpine Linux.
about-tp-in-images = Dans : { $images }
about-tp-image-runtime = l’image d’exécution distroless
about-tp-image-static = l’image FROM-scratch

about-tp-repository = Code source
about-tp-homepage = Page d’accueil
about-tp-new-tab = pour { $name } (s’ouvre dans un nouvel onglet)

about-tp-text-toggle =
    { $count ->
        [one] Texte de licence
        [many] Textes de licence ({ $count })
       *[other] Textes de licence ({ $count })
    }
about-tp-text-open-for = Texte de licence de { $name }
about-tp-dialog-title = Texte de licence : { $name } { $version }
about-tp-close = Fermer
about-tp-text-loading = Chargement du texte de licence…
about-tp-text-error = Impossible de charger les textes de licence.
about-tp-retry = Réessayer
about-tp-text-missing = Ce texte manque dans la liste. Régénérez-la avec scripts/about/gen-third-party.mjs.
about-tp-text-label = Texte de licence de { $name } : { $file }
about-tp-standard = Licence { $licence }
about-tp-standard-note = (texte standard, aucun fichier dans le paquet)
about-tp-no-text = Aucun texte de licence n’est disponible pour cette entrée.
