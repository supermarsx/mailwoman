# Mailwoman — About screens: third-party libraries and licences (source locale: en).
# t32-e75 (t32-cfg.md §4.11). Ids are `about-tp-*`. `$n` is the formatted number,
# `$count` the raw one for plural selection.

about-tp-label = Third-party libraries and licences
about-tp-search = Search by name, version or licence
about-tp-filter = Filter by licence
about-tp-filter-all = All ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] No entries match.
        [one] Showing { $shown } of { $total } entry.
       *[other] Showing { $shown } of { $total } entries.
    }
about-tp-empty = Nothing matches this search and licence.
about-tp-clear = Clear search and filter
about-tp-more = Show more ({ $n } left)

about-tp-kind-crate = Rust crate
about-tp-kind-npm = npm package
about-tp-kind-font = Font
about-tp-kind-icons = Icons
about-tp-kind-image = Images
about-tp-kind-os = Image package

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Rust crates in the server, the web app and the plugins
about-tp-group-shells = Rust crates only in the desktop and mobile apps
about-tp-group-web = npm packages in the web app
about-tp-group-assets = Fonts, icons and images
about-tp-group-image = Server image files
about-tp-group-build = Build tools: run while building, not shipped

about-tp-licence = Licence:
about-tp-licence-none = not declared
about-tp-licence-in-file = see the copyright file
about-tp-used-in = Used in: { $parts }
about-tp-artifact-server = server
about-tp-artifact-web = web app
about-tp-artifact-plugins = plugins
about-tp-artifact-desktop = desktop app
about-tp-artifact-mobile = mobile app
about-tp-note-bundled = Bundled with the web app.
about-tp-note-fonts-pull = Not bundled: downloaded by the server operator with `mailwoman fonts pull`.
about-tp-note-first-party = Made for Mailwoman, under the project's own licence.
about-tp-note-static-libc = Linked into the server binaries of the FROM-scratch image.
about-tp-note-alpine-ca = The CA certificate bundle of the FROM-scratch image, taken from Alpine Linux.
about-tp-in-images = In: { $images }
about-tp-image-runtime = the distroless runtime image
about-tp-image-static = the FROM-scratch image

about-tp-repository = Source code
about-tp-homepage = Home page
about-tp-new-tab = for { $name } (opens in a new tab)

about-tp-text-toggle =
    { $count ->
        [one] Licence text
       *[other] Licence texts ({ $count })
    }
about-tp-text-open-for = Licence text for { $name }
about-tp-dialog-title = Licence text: { $name } { $version }
about-tp-close = Close
about-tp-text-loading = Loading the licence text…
about-tp-text-error = The licence texts could not be loaded.
about-tp-retry = Try again
about-tp-text-missing = This text is missing from the list. Regenerate it with scripts/about/gen-third-party.mjs.
about-tp-text-label = Licence text of { $name }: { $file }
about-tp-standard = { $licence } licence
about-tp-standard-note = (standard text, no file in package)
about-tp-no-text = No licence text is available for this entry.
