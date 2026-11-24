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

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Reintentando…
common-waiting-online = Esperando una conexión…
common-failure-offline = Está sin conexión. Vuelva a conectarse e inténtelo de nuevo.
common-failure-network = No se pudo acceder al servidor.
common-failure-server = El servidor no respondió a tiempo.
common-failure-rate = Demasiadas solicitudes en poco tiempo. Espere un momento e inténtelo de nuevo.
common-failure-conflict = Esto se modificó en otro lugar al mismo tiempo. Recargue e inténtelo de nuevo.
common-failure-account = La cuenta con la sesión iniciada ha cambiado. Recargue la página.
common-failure-linked = No está disponible mientras haya un buzón vinculado abierto.
common-failure-refused = El servidor rechazó el cambio.
common-new-version = Hay una nueva versión de Mailwoman disponible. Si cierra este aviso, se lo recordaremos dentro de una hora.
common-new-version-reload = Recargar ahora
common-toast-close = Cerrar notificación
common-toast-seconds = { $seconds } s
common-attach-refresh = Actualizar adjuntos
common-attach-open-message = Abrir el mensaje que contiene este adjunto
common-attach-page-filter-note = Los filtros se aplican a esta página. Use Siguiente para ver mensajes más antiguos con adjuntos.
common-attach-pages = Páginas de adjuntos
common-attach-previous = Anterior
common-attach-next = Siguiente
common-attach-page = Página { $page }
common-attach-column-name = Archivo
common-attach-column-type = Tipo
common-attach-column-size = Tamaño
common-attach-column-sender = Remitente
common-attach-column-subject = Asunto
common-attach-column-date = Recibido
common-pdf-preview = Vista previa del PDF
common-pdf-link-title = ¿Abrir el enlace de este PDF?
common-pdf-link-help = El destino se abre en una pestaña nueva, fuera de Mailwoman.
common-pdf-link-open = Abrir enlace
common-open-event = Abrir evento
common-message-preview-too-large = La vista previa de los adjuntos de tipo mensaje está limitada a 4 MiB. Aún puede descargar el adjunto original.
common-message-preview-failed = No se pudo mostrar la vista previa de este adjunto de tipo mensaje. Puede que el archivo no sea compatible o esté dañado, o que el servicio de vista previa no esté disponible. Aún puede descargar el original.
common-message-preview-attachments = Archivos dentro de este mensaje
common-message-preview-download = Descargar archivo
common-message-preview-size-bytes = { $count } bytes
common-attach-cat-message = Mensajes / adjuntos de Outlook
common-error-retry-help = Esta sección no está disponible temporalmente. Inténtelo de nuevo para recargarla.
common-error-network-help = Compruebe su conexión e inténtelo de nuevo. Es posible que el servidor no esté disponible temporalmente.
common-error-page = No se pudo cargar esta página
common-error-page-help = No se pudo descargar un recurso de la página. Inténtelo de nuevo. Si el problema continúa, recargue la aplicación para obtener la última versión.
common-reload-app = Recargar la aplicación
common-new-version-reloading = Hay una nueva versión disponible, recargando…
common-attach-account-changed = La cuenta cambió mientras se cargaban los adjuntos. Vuelva a abrir Adjuntos para la cuenta actual.
common-attach-search-account = Buscar en toda la cuenta
common-attach-scanning = Buscando… { $pages } páginas de mensajes revisadas · { $count } archivos encontrados
common-attach-scan-cancelled = Búsqueda cancelada · { $count } archivos encontrados en { $pages } páginas. Los resultados están incompletos.
common-attach-scan-incomplete = Búsqueda interrumpida · { $count } archivos encontrados. Reintente para buscar en toda la cuenta.
common-attach-scan-limited = Se muestran los primeros 1000 archivos coincidentes. Acote los filtros para encontrar archivos más concretos.
common-attach-scan-complete = Búsqueda completada · { $count } archivos encontrados en { $pages } páginas de mensajes
common-attach-browse-pages = Explorar páginas
common-attach-cancel-search = Cancelar búsqueda
multiselect-selected = Seleccionado: { $label }
multiselect-remove = Quitar { $label }
multiselect-no-match = No hay opciones coincidentes
common-notification-permission-title = ¿Permitir notificaciones?
common-notification-permission-help = Reciba avisos de mensajes nuevos en este dispositivo. Puede cambiarlo en Ajustes.
common-notification-permission-allow = Permitir notificaciones
common-notification-permission-later = Ahora no
common-notification-permission-error = No se pudieron activar las notificaciones. Compruebe los permisos y los ajustes de notificación e inténtelo de nuevo.
common-notifications-enabled = Notificaciones activadas en este dispositivo.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [one] 1 seleccionado
   *[other] { $count } seleccionados
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = El servidor está ocupado. Espere unos segundos e inténtelo de nuevo.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = No hay coincidencias

combobox-toggle = Mostrar opciones para { $label }

common-attach-cat-archive = Archivos

common-attach-cat-document = Documentos

common-attach-cat-spreadsheet = Hojas de cálculo

common-conn-reconnecting = Reconectando…

common-viewer-pdf-details = Detalles técnicos

common-viewer-pdf-details-help = Incluya estas líneas cuando informe el problema. No contienen ningún contenido documental.

timezone-group-other = Otro

timezone-use-browser = Utilice la zona horaria del navegador ({ $zone })

common-context-actions = Acciones para { $name }
common-open = Abrir
common-new = Nuevo elemento
common-refresh = Actualizar
common-duplicate = Duplicar
common-copy-fingerprint = Copiar huella digital
common-export-public-key = Exportar clave pública o certificado
common-context-show = Mostrar
common-context-show-only = Mostrar solo este calendario
common-context-color = Color
common-context-import = Importar
common-context-export = Exportar
common-context-pinned = Elementos fijados
common-context-unpin = Desfijar
common-context-my-day = Añadir a Mi día
common-context-remove-my-day = Quitar de Mi día
common-notify-new-count = Mensajes nuevos: { $count }
common-network-internal = Red interna
common-module-notifications = Notificaciones
common-actions = Acciones

attach-action-forward-files = Reenviar archivos
attach-action-forward-messages = Reenviar mensajes asociados
attach-action-delete-messages = Eliminar mensajes asociados
attach-action-locate = Localizar mensaje
attach-action-select = Seleccionar adjuntos

workspace-view-properties = Propiedades de vistas
workspace-view-help = Elige visibilidad, nombres y contadores de la barra lateral. Las restricciones del administrador siempre se aplican.
workspace-view-visibility = Visibilidad
workspace-view-mode-visible = Mostrar siempre
workspace-view-mode-hidden = Oculta
workspace-view-mode-notable = Solo con contenido relevante
workspace-view-name = Nombre mostrado
workspace-view-count = Mostrar contador
workspace-view-notable-notifications = Visible con notificaciones sin leer.
workspace-view-notable-calendar = Visible con eventos en los próximos siete días.
workspace-view-notable-tasks = Visible con tareas pendientes.
workspace-view-notable-notes = Visible con notas guardadas.
workspace-view-notable-contacts = Visible con contactos.
workspace-view-notable-keys = Visible con claves guardadas.
workspace-view-notable-attachments = Visible con adjuntos.
workspace-view-notable-outbox = Visible con mensajes pendientes de envío.
workspace-view-notable-templates = Visible con plantillas guardadas.

common-conn-accounts = Conexiones de buzones
common-conn-account-online = Actualizado
common-conn-account-syncing = Sincronizando
common-conn-account-offline = Sin conexión
common-conn-account-degraded = No disponible temporalmente
common-conn-account-auth-expired = Se requiere iniciar sesión
common-conn-account-retry = Reintento en { $seconds } s
common-conn-account-recovered = { $account }: conexión restablecida
common-conn-offline-queued = Sin conexión — { $count } cambios en cola
common-conn-account-problem = { $account }: { $status }

common-conn-background-unavailable = Mantén Mailwoman abierto para sincronizar los cambios pendientes en este dispositivo.

notification-center-action-success = { $action } · { $count }

common-loading-still = Esta página sigue cargando. Espera…

attach-action-open-split = Abrir en vista dividida
attach-action-forward-new = Reenviar en un correo nuevo

common-conn-unstable = Conexión inestable — desconexiones frecuentes. Esperando una conexión estable.
common-conn-account-unstable = Conexión inestable
common-conn-account-instability = La conexión con { $account } es inestable y se reconecta continuamente.

# Error reporting controls must work even when a lazy page/catalog cannot load.
ce-copied = Copiado
ce-copy = Copiar detalles
ce-ref = Referencia de error
ce-status-failed = Informe no enviado
ce-status-off = Los informes están desactivados
ce-status-pending = Enviando informe…
ce-status-reported = Reportado
ce-send = Enviar informe
ce-send-retry = Reintentar envío
ce-report-help = Los administradores pueden encontrar los informes enviados en Observabilidad → Errores del cliente mediante esta referencia.
