# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Visualización de mensajes
reader-settings-help = Elija cómo se muestran los mensajes. Los valores predeterminados del administrador se aplican hasta que elija otros. El contenido del mensaje permanece aislado de la aplicación.
reader-colorMode = Colores del mensaje
reader-colorMode-help = Automático adapta los colores del mensaje en los temas oscuros. Original conserva los colores del remitente; adaptable usa el tema activo; inversión invierte los colores conservando los de las imágenes.
reader-fontSize = Tamaño de letra de lectura (px)
reader-fontSize-help = Tamaño de letra base del texto del mensaje, de 12 a 24 píxeles. El formato del remitente aún puede definir el tamaño de los títulos.
reader-lineHeight = Interlineado de lectura
reader-lineHeight-help = Espacio entre las líneas del texto del mensaje, de 1,2 a 2,2 veces el tamaño de letra.
reader-contentWidth = Ancho de lectura
reader-contentWidth-help = Limitar las líneas largas para una lectura cómoda o usar todo el panel del mensaje.
reader-loading = Aspecto de la carga
reader-loading-help = Elija líneas de marcador discretas, un indicador de texto o ningún indicador visual. Los lectores de pantalla siguen recibiendo el estado de carga.
reader-loadingDelayMs = Retraso del indicador de carga (ms)
reader-loadingDelayMs-help = Evita que un indicador parpadee en solicitudes rápidas. Solo se retrasa el indicador, nunca el mensaje.
reader-option-auto = Automático con el tema
reader-option-original = Colores originales del mensaje
reader-option-adaptive = Adaptar al tema
reader-option-invert = Invertir los colores del mensaje
reader-option-comfortable = Cómodo
reader-option-wide = Ancho
reader-option-full = Panel completo
reader-option-skeleton = Líneas de marcador
reader-option-indicator = Indicador de texto
reader-option-none = Sin indicador visual
reader-inherit = Usar el valor predeterminado del administrador
reader-actionStyle = Cinta de acciones del mensaje
reader-actionStyle-help = Los iconos compactos dejan más espacio al mensaje. Cada acción conserva su nombre para el teclado y una descripción emergente. Elija etiquetas visibles si prefiere texto junto a los iconos.
reader-option-icons = Iconos compactos
reader-option-labels = Iconos y etiquetas
reader-zoom-group = Zoom del mensaje
reader-zoom-in = Acercar
reader-zoom-in-help = Agrandar el mensaje. Solo cambia el tamaño del mensaje.
reader-zoom-out = Alejar
reader-zoom-out-help = Reducir el mensaje. Solo cambia el tamaño del mensaje.
reader-zoom-level = { $level } %
reader-zoom-reset-label = Zoom del mensaje { $level } %. Restablecer al 100 %
reader-zoom-reset-help = Restablecer el mensaje al 100 %
reader-zoom-reset-menu = Restablecer zoom ({ $level } %)
reader-zoom = Zoom del mensaje (%)
reader-zoom-help = El tamaño con el que se abren los mensajes, del 50 % al 200 %. Solo se escala el cuerpo del mensaje, nunca la aplicación, y la impresión siempre usa el 100 %. Los botones de zoom junto a las acciones del mensaje también lo cambian.
reader-attachments-summary = { $count ->
    [one] 1 adjunto
   *[other] { $count } adjuntos
    } · { $size }
reader-attachments-show = Mostrar adjuntos
reader-attachments-hide = Ocultar adjuntos
reader-attachmentsDensity = Densidad de adjuntos
reader-attachmentsDensity-help = Compacta muestra botones de archivo pequeños; cómoda muestra vistas previas más grandes al expandir.
reader-option-compact = Compacta
reader-attachmentsCollapsed = Contraer los adjuntos de forma predeterminada
reader-attachmentsCollapsed-help = Cada mensaje empieza con un resumen compacto de adjuntos. Puede expandir ese mensaje sin cambiar este valor predeterminado.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Texto y colores
reader-card-text-help = Tamaño, espaciado, anchura y colores del texto de los mensajes.
reader-card-actions = Acciones y archivos adjuntos
reader-card-actions-help = La cinta de acciones del mensaje y cómo se inician los archivos adjuntos.
reader-card-loading = Carga
reader-card-loading-help = Lo que ve mientras se abre un mensaje.
settings-mail-card-list = Lista de mensajes
settings-mail-card-list-help = Acciones de fila y qué hace la lista cuando actúa sobre un mensaje.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Descargar { $name }

reader-attachments-download-all = Descargar todo

reader-attachments-download-all-blocked = La política de archivos adjuntos bloquea todos los archivos aquí.

reader-attachments-download-all-busy = Preparando el archivo ZIP

reader-attachments-download-all-label = Descargue todos los archivos adjuntos como un archivo ZIP

reader-attachments-download-all-none = No hay archivos adjuntos para descargar.

reader-attachments-download-all-warned = { $count ->
        [one] Se omitió un archivo que necesita confirmación. Descárgalo solo.
        [many] Se omitieron { $count } archivos que necesitan confirmación. Descárgalos uno por uno.
       *[other] Se omitieron { $count } archivos que necesitan confirmación. Descárgalos uno por uno.
    }

reader-attachments-download-blocked = { $name } no se puede descargar: { $reason }

reader-attachments-download-label = Descargar { $name } ({ $size })

reader-attachments-download-off = Las descargas están desactivadas

reader-attachments-inline-group = { $count ->
        [one] { $count } imagen en línea
        [many] { $count } imágenes en línea
       *[other] { $count } imágenes en línea
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } imagen en línea
        [many] { $count } imágenes en línea
       *[other] { $count } imágenes en línea
    } · { $size }

reader-attachmentsBarClickToToggle = Haga clic en la barra para expandir o contraer

reader-attachmentsBarClickToToggle-help = Al hacer clic en cualquier lugar de la barra de archivos adjuntos, se alterna. Cuando está apagado, solo lo hace el botón de flecha.

reader-attachmentsBarDownloadAll = botón descargar todo

reader-attachmentsBarDownloadAll-help = Muestra un botón en la barra que descarga los archivos adjuntos como un archivo ZIP.

reader-attachmentsBarShowDownloadButtons = Botones de descarga en archivos adjuntos

reader-attachmentsBarShowDownloadButtons-help = Muestre el botón de descarga de cada archivo adjunto al pasar el cursor o al enfocar, siempre o nunca.

reader-attachmentsBarZipInlineImages = Incluir imágenes en línea en el ZIP

reader-attachmentsBarZipInlineImages-help = Las imágenes que aparecen dentro del texto del mensaje no se incluyen en Descargar todo a menos que esté activado.

reader-card-attachments-bar = Barra de archivos adjuntos

reader-card-attachments-bar-help = Cómo responde la barra encima de los archivos adjuntos de un mensaje y qué ofrece. Las líneas marcadas como bloqueadas las establece su administrador.

reader-option-always = siempre

reader-option-hover = Al pasar el mouse o enfocar

reader-option-never = nunca
