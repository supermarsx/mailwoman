# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = Aceptar
common-cancel = Cancelar
common-close = Cerrar
common-save = Guardar
common-delete = Eliminar
common-remove = Quitar
common-edit = Editar
common-add = Añadir
common-back = Atrás
common-next = Siguiente
common-done = Hecho
common-apply = Aplicar
common-confirm = Confirmar
common-retry = Reintentar
common-contact-administrator = Si esto sigue ocurriendo, póngase en contacto con su administrador.
common-shell-not-configured = Esta aplicación no se ha configurado para ningún servidor. Póngase en contacto con su administrador.
common-copy = Copiar
common-search = Buscar
common-more = Más
common-yes = Sí
common-no = No

# -- Generic states ----------------------------------------------------------
common-loading = Cargando…
common-saving = Guardando…
common-empty = Aquí no hay nada todavía
common-error = Algo salió mal
common-error-network = No se puede conectar con el servidor. Compruebe su conexión e inténtelo de nuevo.
common-offline = Sin conexión

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Conectando…
common-conn-degraded = Actualizaciones en tiempo real en pausa — reconectando
common-conn-offline = Sin conexión — los cambios se sincronizarán al reconectar
common-conn-auth-expired = Su sesión ha caducado — inicie sesión de nuevo
common-conn-reconnect = Reconectar
common-conn-sign-in-again = Volver a iniciar sesión
common-conn-reconnected = Reconectado

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Mensaje nuevo
common-notify-no-subject = (sin asunto)

# -- Generic labels ----------------------------------------------------------
common-required = Obligatorio
common-optional = Opcional

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Abrir carpetas y aplicaciones
common-nav-close = Cerrar carpetas y aplicaciones

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Adjuntos
common-attach-search = Buscar adjuntos
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Filtrar por tipo
common-attach-loading = Cargando adjuntos…
common-attach-empty = Ningún adjunto coincide.
common-attach-cat-all = todos
common-attach-cat-image = imagen
common-attach-cat-pdf = pdf
common-attach-cat-text = texto
common-attach-cat-audio = audio
common-attach-cat-video = vídeo
common-attach-cat-other = otros

# -- Password fields (components/PasswordInput.tsx): the button at the end of
common-password-show = Mostrar contraseña
common-password-hide = Ocultar contraseña

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
common-module-mail = Correo
common-module-calendar = Calendario
common-module-tasks = Tareas
common-module-notes = Notas
common-module-contacts = Contactos
common-module-keys = Claves

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
common-viewer-loading = Cargando visor…
common-viewer-loading-audio = Cargando audio…
common-viewer-loading-image = Cargando imagen…
common-viewer-loading-video = Cargando vídeo…
common-viewer-rendering-pdf = Generando PDF…
common-viewer-pdf-failed = No se pudo mostrar este PDF.
common-viewer-no-preview = No hay vista previa para este tipo de archivo.
common-viewer-unknown-type = tipo desconocido
common-viewer-download = Descargar
common-viewer-attachments = Adjuntos
