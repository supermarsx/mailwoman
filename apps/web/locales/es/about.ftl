# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: about. Source: ../en/about.ftl

about-tp-label = Bibliotecas de terceros y licencias
about-tp-search = Buscar por nombre, versión o licencia
about-tp-filter = Filtrar por licencia
about-tp-filter-all = Todas ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Ninguna entrada coincide.
        [one] Mostrando { $shown } de { $total } entrada.
        [many] Mostrando { $shown } de { $total } entradas.
       *[other] Mostrando { $shown } de { $total } entradas.
    }
about-tp-empty = Nada coincide con esta búsqueda y licencia.
about-tp-clear = Borrar búsqueda y filtro
about-tp-more = Mostrar más (quedan { $n })

about-tp-kind-crate = Crate de Rust
about-tp-kind-npm = Paquete npm
about-tp-kind-font = Fuente
about-tp-kind-icons = Iconos
about-tp-kind-image = Imágenes
about-tp-kind-os = Paquete de imagen

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Crates de Rust en el servidor, la aplicación web y los plugins
about-tp-group-shells = Crates de Rust solo en las aplicaciones de escritorio y móvil
about-tp-group-web = Paquetes npm en la aplicación web
about-tp-group-assets = Fuentes, iconos e imágenes
about-tp-group-image = Archivos de la imagen del servidor
about-tp-group-build = Herramientas de compilación: se usan al compilar, no se distribuyen

about-tp-licence = Licencia:
about-tp-licence-none = no declarada
about-tp-licence-in-file = ver el archivo de copyright
about-tp-used-in = Se usa en: { $parts }
about-tp-artifact-server = servidor
about-tp-artifact-web = aplicación web
about-tp-artifact-plugins = plugins
about-tp-artifact-desktop = aplicación de escritorio
about-tp-artifact-mobile = aplicación móvil
about-tp-note-bundled = Incluido en la aplicación web.
about-tp-note-fonts-pull = No incluido: lo descarga el operador del servidor con `mailwoman fonts pull`.
about-tp-note-first-party = Hecho para Mailwoman, bajo la licencia propia del proyecto.
about-tp-note-static-libc = Enlazado en los binarios del servidor de la imagen FROM-scratch.
about-tp-note-alpine-ca = El paquete de certificados de CA de la imagen FROM-scratch, tomado de Alpine Linux.
about-tp-in-images = En: { $images }
about-tp-image-runtime = la imagen de ejecución distroless
about-tp-image-static = la imagen FROM-scratch

about-tp-repository = Código fuente
about-tp-homepage = Página de inicio
about-tp-new-tab = para { $name } (se abre en una pestaña nueva)

about-tp-text-toggle =
    { $count ->
        [one] Texto de la licencia
        [many] Textos de las licencias ({ $count })
       *[other] Textos de las licencias ({ $count })
    }
about-tp-text-open-for = Texto de la licencia de { $name }
about-tp-dialog-title = Texto de la licencia: { $name } { $version }
about-tp-close = Cerrar
about-tp-text-loading = Cargando el texto de la licencia…
about-tp-text-error = No se pudieron cargar los textos de las licencias.
about-tp-retry = Reintentar
about-tp-text-missing = Este texto falta en la lista. Vuelva a generarlo con scripts/about/gen-third-party.mjs.
about-tp-text-label = Texto de la licencia de { $name }: { $file }
about-tp-standard = Licencia { $licence }
about-tp-standard-note = (texto estándar, sin archivo en el paquete)
about-tp-no-text = No hay texto de licencia disponible para esta entrada.
