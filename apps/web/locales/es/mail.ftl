# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Configuración
mail-compose = Redactar
mail-nav-mailboxes = Buzones
mail-nav-attachments = Adjuntos
mail-nav-outbox = Bandeja de salida
mail-nav-sharing = Compartir carpeta
mail-nav-apps = Aplicaciones
mail-logout = Cerrar sesión
mail-offline = Sin conexión
mail-module-loading = Cargando { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Buscar
mail-search-label = Buscar correo
mail-search-placeholder = Buscar correo — from:alice subject:factura larger:1mb
mail-search-clear = Borrar

# -- Message list ------------------------------------------------------------
mail-list-label = Mensajes
mail-loading = Cargando mensajes…
mail-empty = No hay mensajes
mail-unknown-sender = (remitente desconocido)
mail-no-subject = (sin asunto)
mail-pinned = Fijado
mail-has-attachment = Tiene adjunto
mail-answered = Respondido
mail-forwarded = Reenviado
mail-unread = No leído
mail-flagged = Marcado
mail-selected = Seleccionado
mail-row-position = Mensaje { $pos } de { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } mensaje
   *[other] { $count } mensajes
}
mail-thread-expand = Expandir conversación, { $count ->
    [one] { $count } mensaje
   *[other] { $count } mensajes
}
mail-thread-collapse = Contraer conversación, { $count ->
    [one] { $count } mensaje
   *[other] { $count } mensajes
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Opciones de vista
mail-reading-pane = Panel de lectura
mail-reading-pane-right = Derecha
mail-reading-pane-bottom = Abajo
mail-reading-pane-off = Desactivado

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Mensajes seleccionados
mail-bulk-count = { $count } seleccionados
mail-mark-read = Marcar como leído
mail-mark-unread = Marcar como no leído
mail-flag-message = Marcar
mail-unflag-message = Quitar marca
mail-move-to = Mover a carpeta
mail-move-to-placeholder = Mover a…
mail-select-all = Seleccionar todo
mail-clear-selection = Borrar selección

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = Filtro de la bandeja de entrada
mail-inbox-focused = Prioritarios
mail-inbox-other = Otros

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Pestañas abiertas
mail-subtab-pin = Fijar { $title }
mail-subtab-unpin = Desfijar { $title }
mail-subtab-tearoff = Abrir { $title } en una ventana nueva
mail-subtab-close = Cerrar { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Fijar
mail-unpin = Desfijar
mail-snooze = Posponer
mail-snooze-menu = Posponer hasta
mail-snooze-later = Más tarde hoy
mail-snooze-tomorrow = Mañana
mail-snooze-next-week = La próxima semana
mail-unsnooze = No posponer
mail-label = Etiqueta
mail-labels-menu = Etiquetas
mail-flag = Marcar para seguimiento
mail-clear-flag = Quitar seguimiento
mail-archive = Archivar
mail-delete = Eliminar
mail-more-actions = Más acciones

# -- Reader ------------------------------------------------------------------
mail-reader-label = Mensaje
mail-reader-empty = Seleccione un mensaje para leerlo
mail-back = Atrás
mail-reader-from = De: { $addr }
mail-reader-to = Para: { $addr }
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Acciones del mensaje
mail-reply = Responder
mail-reply-all = Responder a todos
mail-forward = Reenviar
mail-reply-failed = No se pudo preparar el mensaje. Inténtelo de nuevo.
mail-reply-unthreaded = No se pudo leer el ID del mensaje original. Esta respuesta no quedará en la misma conversación.
mail-spam = Spam
mail-export = Exportar
mail-sweep-sender = Barrer remitente
mail-message-body = Cuerpo del mensaje
mail-no-content = Sin contenido
mail-sanitizing = Saneando…
mail-attachments = Adjuntos
mail-attachment-unnamed = (sin nombre)
mail-attachment-open = Adjunto { $name }
mail-attachment-close = Cerrar adjunto
mail-attachment-loading = Cargando adjunto…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Mensaje cifrado
mail-encrypted-note = Este mensaje está cifrado de extremo a extremo. Desbloquéelo en este dispositivo para leerlo.
mail-key-passphrase = Frase de contraseña de la clave
mail-decrypt = Descifrar
mail-decrypting = Descifrando…
mail-decrypt-no-key = No hay ninguna clave privada disponible para descifrar este mensaje.
mail-decrypt-failed = Error al descifrar

# -- Reply and forward: text written into the outgoing message --------------
mail-quote-attribution = El { $date }, { $name } escribió:
mail-quote-attribution-undated = { $name } escribió:
mail-forward-heading = ---------- Mensaje reenviado ----------
mail-forward-from = De: { $addr }
mail-forward-date = Fecha: { $date }
mail-forward-subject = Asunto: { $subject }
mail-forward-to = Para: { $addr }
mail-forward-cc = Cc: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Redactar mensaje
mail-compose-title = Mensaje nuevo
mail-compose-title-reply = Responder
mail-compose-title-reply-all = Responder a todos
mail-compose-title-forward = Reenviar
mail-compose-close = Cerrar
mail-compose-from = De
mail-compose-from-default = Predeterminado
mail-compose-to = Para
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Sugerencias de contactos
mail-compose-cc = Cc
mail-compose-bcc = Cco
mail-compose-show-cc-bcc = Añadir Cc o Cco
mail-compose-bcc-encrypted = Un mensaje cifrado incluye la clave de cada destinatario, por lo que un destinatario en Cco sería visible para los demás. Quite los destinatarios en Cco o desactive el cifrado.
mail-compose-plaintext-confirm-title = ¿Enviar texto descifrado sin cifrar?
mail-compose-plaintext-confirm-body = Este mensaje cita texto de un mensaje cifrado. No está configurado para cifrarse, así que el texto citado se enviaría sin protección.
mail-compose-plaintext-confirm-send = Enviar sin cifrar
mail-compose-plaintext-confirm-cancel = Volver
mail-compose-subject = Asunto
mail-compose-body = Cuerpo
mail-compose-attachments = Adjuntos
mail-compose-remove-attachment = Quitar { $name }
mail-compose-attach-nextcloud = Adjuntar desde Nextcloud
mail-compose-close-nextcloud = Cerrar Nextcloud
mail-compose-attach-file = Adjuntar un archivo
mail-compose-uploading = Subiendo…
mail-compose-upload-unavailable = La subida de archivos no está disponible en este momento.
mail-compose-upload-failed = No se pudo subir { $name }.
mail-compose-upload-too-large = { $name } ocupa { $size } MB. El tamaño máximo de subida es { $max } MB.
mail-compose-send-later = Enviar más tarde
mail-compose-cancel = Cancelar
mail-compose-send = Enviar
mail-compose-schedule = Programar
mail-compose-sending = Enviando…
mail-compose-encrypted-subject = Mensaje cifrado
mail-compose-dlp-blocked = El envío está bloqueado por una regla de prevención de pérdida de datos (vea la advertencia de arriba).
mail-compose-send-failed = Error al enviar
mail-compose-send-later-past = Elija una hora de envío en el futuro o vacíe el campo para enviar ahora.
mail-compose-options = Opciones de envío
mail-compose-request-receipt = Solicitar confirmación de lectura
mail-compose-request-receipt-hint = El programa de correo del destinatario puede preguntarle, enviar una confirmación sin preguntar o ignorar la solicitud.
mail-compose-track-opens = Registrar cuándo se abre este mensaje
mail-compose-track-opens-hint = Se añade al mensaje una imagen de 1×1 que se carga desde este servidor. Muchos programas de correo bloquean las imágenes o las cargan por adelantado, así que una apertura registrada no demuestra que el mensaje se haya leído, y la falta de registro no demuestra que no se haya leído. No se avisa al destinatario.
mail-compose-image-type = { $name } no se puede colocar en el texto. Use una imagen PNG, JPEG, GIF o WebP.
mail-send-not-signed-in = No ha iniciado sesión.
mail-send-no-draft-folder = Esta cuenta no tiene ninguna carpeta para guardar el borrador.
mail-send-no-recipient = Añada al menos un destinatario.
mail-send-bad-recipient = { $count ->
    [one] Esto no es una dirección de correo electrónico: { $addresses }
   *[other] Estas no son direcciones de correo electrónico: { $addresses }
}
mail-send-later-past = La hora de envío no está en el futuro. No se envió nada.
mail-send-tracking-failed = Ahora no se pueden registrar las aperturas, así que no se envió nada. Desmarque «Registrar cuándo se abre este mensaje» para enviarlo sin ello.
mail-send-draft-refused = El servidor rechazó este mensaje: { $reason }. No se envió nada.
mail-send-submission-refused = El servidor se negó a enviar este mensaje: { $reason }. No se envió nada.
mail-compose-sign-unlock-title = Desbloquear clave de firma
mail-compose-sign-unlock-note = Introduzca la frase de contraseña de su clave para firmar este mensaje. Permanece desbloqueada solo para esta ventana de redacción.
mail-compose-sign-unlock = Desbloquear
mail-compose-sign-unlocking = Desbloqueando…
mail-compose-sign-no-key = No hay ninguna clave de firma disponible en este dispositivo.
mail-compose-sign-unlock-failed = No se pudo desbloquear la clave de firma.
mail-compose-sign-unlock-required = Desbloquee su clave de firma para enviar este mensaje firmado.

mail-compose-format-plain = Texto sin formato
mail-compose-format-rich = Texto enriquecido
mail-compose-rt-toolbar = Formato
mail-compose-rt-bold = Negrita
mail-compose-rt-italic = Cursiva
mail-compose-rt-underline = Subrayado
mail-compose-rt-strike = Tachado
mail-compose-rt-bullet = Lista con viñetas
mail-compose-rt-ordered = Lista numerada
mail-compose-rt-quote = Cita
mail-compose-rt-link = Enlace
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Aplicar
mail-compose-rt-image = Insertar una imagen

mail-compose-signature = Firma
mail-compose-signature-none = Añadir una firma…

mail-compose-recall = Recuperar
mail-compose-recall-action = Recuperar
mail-compose-recall-holding = Retenido, en espera de envío
mail-compose-recall-scheduled = Programado para { $when }

mail-compose-drafts = Borradores
mail-compose-drafts-empty = No hay borradores guardados
mail-compose-drafts-delete = Eliminar borrador
mail-compose-drafts-no-recipient = (sin destinatario)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Bandeja de salida
mail-outbox-refresh = Actualizar
mail-outbox-empty = No hay nada pendiente de envío
mail-outbox-send-now = Enviar ahora
mail-outbox-cancel = Cancelar
mail-outbox-scheduled = Programado
mail-outbox-holding = Se enviará en breve
mail-outbox-sent = Enviado
mail-outbox-canceled = Cancelado
mail-outbox-held = Retenido
mail-outbox-failed = No enviado
mail-outbox-release = Liberar
mail-outbox-discard = Descartar
mail-outbox-held-note = Retenido. No se envía hasta que se libere.
mail-outbox-held-note-api-key = Retenido — creado por la clave de API { $name }. No se envía hasta que se libere.
mail-outbox-held-note-oauth = Retenido — creado por la aplicación conectada { $name }. No se envía hasta que se libere.
mail-outbox-origin-api-key = Creado por la clave de API { $name }
mail-outbox-origin-oauth = Creado por la aplicación conectada { $name }
mail-outbox-to = Para { $recipients }
mail-outbox-no-subject = (sin asunto)
mail-outbox-attempts = Intentado { $count } veces. Último error: { $error }
mail-outbox-error = Error: { $error }
mail-outbox-toast-canceled = Envío cancelado
mail-outbox-toast-discarded = Descartado. El mensaje no se envió.
mail-outbox-toast-sent = Enviado
mail-outbox-toast-retrying = Aún no enviado: { $error }. Se volverá a intentar.
mail-outbox-toast-failed = No enviado: { $error }
mail-outbox-toast-not-released = No se pudo enviar: { $error }
mail-outbox-toast-not-canceled = No se pudo cancelar: { $error }

# -- Network status and the offline queue ------------------------------------
mail-net-lost = Conexión perdida — reintentando…
mail-net-back = Conexión restablecida
mail-queue-sent = { $count ->
    [one] Se envió { $count } cambio en cola
   *[other] Se enviaron { $count } cambios en cola
}
mail-queue-failed = { $count ->
    [one] Falló { $count } cambio en cola
   *[other] Fallaron { $count } cambios en cola
}
mail-queue-notice-label = Cambios que no se pudieron aplicar
mail-queue-notice-title = { $count ->
    [one] No se pudo aplicar { $count } cambio hecho sin conexión
   *[other] No se pudieron aplicar { $count } cambios hechos sin conexión
}
mail-queue-kind-send = Enviar “{ $subject }”
mail-queue-kind-send-untitled = Enviar un mensaje sin asunto
mail-queue-kind-move = Mover un mensaje
mail-queue-kind-flag = Cambiar una etiqueta o marca
mail-queue-kind-draft = Guardar un borrador
mail-queue-kind-pim = Cambio de calendario, tarea, nota o contacto
mail-queue-reason = Motivo: { $reason }
mail-queue-reason-unknown = El servidor no indicó el motivo.
mail-queue-retry = Reintentar
mail-queue-discard = Descartar
mail-queue-retry-item = Reintentar: { $what }
mail-queue-discard-item = Descartar: { $what }
mail-queue-discarded = Descartado. El cambio no se aplicó.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Barrer mensajes
mail-sweep-title = Barrer { $sender }
mail-sweep-all = Eliminar todos los de este remitente
mail-sweep-keep-latest = Conservar el más reciente y eliminar el resto
mail-sweep-older-than = Eliminar los de más de N días
mail-sweep-block = Eliminar todos y bloquear a este remitente
mail-sweep-days = Días
mail-sweep-count = { $count ->
    [one] { $count } mensaje se moverá a la Papelera
   *[other] { $count } mensajes se moverán a la Papelera
}
mail-sweep-cancel = Cancelar
mail-sweep-run = Barrer { $count }
mail-sweeping = Barriendo…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Cerrar
mail-undo-action = Deshacer
mail-undo-send-cancel = Cancelar
mail-undo-failed = No se pudo deshacer

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Etiqueta añadida
mail-toast-label-removed = Etiqueta quitada
mail-toast-pinned = Fijado
mail-toast-unpinned = Desfijado
mail-toast-snoozed = Pospuesto
mail-toast-follow-up-set = Seguimiento establecido
mail-toast-follow-up-cleared = Seguimiento quitado
mail-toast-archived = Archivado
mail-toast-trashed = Movido a la Papelera
mail-toast-marked-spam = Marcado como spam
mail-toast-moved-to = Movido a { $folder }
mail-toast-moved = Movido a la carpeta
mail-toast-archived-count = { $count } mensajes archivados
mail-toast-trashed-count = { $count } mensajes movidos a la Papelera
mail-toast-marked-spam-count = { $count } mensajes marcados como spam
mail-toast-moved-count = { $count } mensajes movidos
mail-toast-moved-to-count = { $count } mensajes movidos a { $folder }
mail-toast-marked-read = { $count ->
    [one] Marcado como leído
   *[other] { $count } mensajes marcados como leídos
}
mail-toast-marked-unread = { $count ->
    [one] Marcado como no leído
   *[other] { $count } mensajes marcados como no leídos
}
mail-toast-flagged = { $count ->
    [one] Marcado
   *[other] { $count } mensajes marcados
}
mail-toast-unflagged = { $count ->
    [one] Marca quitada
   *[other] Marca quitada de { $count } mensajes
}
mail-toast-move-failed = No se pudieron mover los mensajes. No se movió nada.
mail-toast-update-failed = No se pudieron cambiar los mensajes. No se cambió nada.
mail-toast-update-refused = { $count ->
    [one] El servidor rechazó el cambio para un mensaje
   *[other] El servidor rechazó el cambio para { $count } mensajes
}
mail-toast-no-archive-folder = No hay carpeta Archivo
mail-toast-no-trash-folder = No hay carpeta Papelera
mail-toast-no-spam-folder = No hay carpeta Spam
mail-toast-sweep-nothing = No hay nada que barrer
mail-toast-swept = { $count ->
    [one] Se barrió { $count } mensaje
   *[other] Se barrieron { $count } mensajes
}
mail-toast-load-more-failed = No se pudieron cargar más mensajes
mail-toast-export-nothing = No hay nada que exportar
mail-toast-exported = Exportado como .eml
mail-toast-export-failed = Error al exportar
mail-toast-send-queued = En cola — se enviará al recuperar la conexión
mail-toast-tracking-not-bound = El mensaje se envió, pero sus aperturas no se registrarán.
mail-toast-send-scheduled = Envío programado
mail-toast-sent = Mensaje enviado
mail-toast-send-canceled = Envío cancelado

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Cinta
mail-ribbon-tab-home = Inicio
mail-ribbon-tab-view = Vista
mail-ribbon-tab-folder = Carpeta
mail-ribbon-expand = Expandir
mail-ribbon-collapse = Contraer
mail-ribbon-group-new = Nuevo
mail-ribbon-compose = Redactar
mail-ribbon-group-session = Sesión
mail-ribbon-logout = Cerrar sesión
mail-ribbon-group-theme = Tema
mail-ribbon-group-density = Densidad
mail-ribbon-group-settings = Configuración
mail-ribbon-more = Más…
mail-ribbon-group-folders = Carpetas
mail-density-compact = compacta
mail-density-cozy = cómoda
mail-density-relaxed = amplia

# -- Message security level: the three-position switch in the reader toolbar
mail-maxsec-label = Nivel de seguridad del mensaje
mail-maxsec-full = Completo
mail-maxsec-full-hint = HTML saneado con imágenes y estilos.
mail-maxsec-no-media = Sin multimedia
mail-maxsec-no-media-hint = HTML saneado sin imágenes ni contenido multimedia.
mail-maxsec-plain-text = Texto sin formato
mail-maxsec-plain-text-hint = No se representa HTML — el mensaje se muestra solo como texto sin formato.
mail-feature-off = Desactivado por sus ajustes o por su administrador
mail-send-disabled = El envío está desactivado por sus ajustes o por su administrador. Todavía puede guardar un borrador.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Eventos
mail-events-upcoming = Solo próximos
mail-events-filter = Filtro de eventos
mail-loading-more = Cargando mensajes anteriores…
mail-event-state-invitation = Invitación
mail-event-state-updated = Actualizado
mail-event-state-cancelled = Cancelado
mail-event-state-reply = Respuesta
mail-event-state-counter = Nueva hora propuesta
mail-event-state-event = Evento
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Mostrar más acciones
mail-remote-moved = Este mensaje se movió a { $folder } desde otro dispositivo o aplicación. Sigue leyendo la copia que abrió.
mail-remote-moved-unknown = Este mensaje se movió desde otro dispositivo o aplicación. Sigue leyendo la copia que abrió.
mail-remote-deleted = Este mensaje se eliminó desde otro dispositivo o aplicación. Puede seguir leyendo esta copia hasta que la cierre.
mail-remote-dismiss = Descartar
mail-reply-draft = Respuesta en curso
mail-reply-draft-open = Abrir borrador
mail-draft-load-failed = No se pudo abrir el borrador. Inténtelo de nuevo.
mail-reply-images-omitted =
    { $count ->
        [one] No se pudo incluir una imagen en la respuesta.
       *[other] No se pudieron incluir { $count } imágenes en la respuesta.
    }
mail-export-forbidden = Su administrador ha desactivado la exportación de mensajes para esta cuenta.
mail-export-open-failed = No se pudieron abrir las opciones de exportación. Cierre una pestaña del espacio de trabajo e inténtelo de nuevo.
mail-headers-title = Encabezados sin procesar del mensaje
mail-headers-action-help = Ver los encabezados originales del mensaje, incluidos los detalles de entrega y autenticación.
mail-headers-help = Encabezados originales del mensaje. Se conservan las líneas plegadas; aquí solo se muestran encabezados.
mail-headers-loading = Cargando encabezados del mensaje…
mail-headers-failed = No se pudieron cargar los encabezados del mensaje. Compruebe su conexión e inténtelo de nuevo.
mail-headers-copy = Copiar encabezados
mail-headers-copied = Encabezados copiados.
mail-headers-copy-failed = No se pudieron copiar los encabezados. Seleccione el texto para copiarlo manualmente.
mail-attachment-failed = No se pudo descargar este adjunto. Inténtelo de nuevo cuando haya conexión.
mail-compose-rt-rule = Línea horizontal
mail-compose-signature-new = Mensajes nuevos
mail-compose-signature-replies = Respuestas
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = Tamaño del cuerpo
mail-editor-body-size-help = Tamaño del cuerpo actual en UTF-8, incluido el marcado HTML y las imágenes incrustadas. No se incluyen los encabezados del mensaje ni los adjuntos independientes.
mail-outbox-load-failed = No se pudo comprobar la bandeja de salida. Compruebe su conexión y pruebe con Actualizar.
mail-outbox-unavailable = Bandeja de salida no disponible. Use Actualizar para intentarlo de nuevo.
mail-maxsec-view = Vista: { $mode }
mail-maxsec-menu = Modo de vista
mail-toast-tab-pinned = Pestaña «{ $target }» fijada
mail-toast-tab-unpinned = Pestaña «{ $target }» desfijada
mail-workspace-message = Mensaje
mail-opened-announcement = Abierto: { $subject }
mail-workspace-layout = Diseño del espacio de trabajo
mail-workspace-tabs = Pestañas
mail-workspace-side-by-side = Lado a lado
mail-workspace-stack = Apilado
mail-workspace-refresh = Actualizar carpeta
mail-workspace-load-error = No se pudo cargar este mensaje. Cierre la pestaña e inténtelo de nuevo.
mail-resize-folders = Cambiar el tamaño de la barra lateral de carpetas
mail-resize-list = Cambiar el tamaño de la lista de mensajes
mail-resize-tiles = Cambiar el tamaño de los paneles del espacio de trabajo
mail-show-folders = Mostrar barra lateral de carpetas
mail-hide-folders = Ocultar barra lateral de carpetas
mail-compact-folders = Iconos de carpeta compactos
mail-toggle-list = Mostrar u ocultar la lista de mensajes
mail-toggle-reader = Mostrar u ocultar el panel de lectura
mail-folder-counts = { $unread } sin leer, { $total } mensajes
mail-print = Imprimir / Guardar como PDF
mail-go-to-top = Ir arriba
mail-print-help = Imprima este mensaje o elija Guardar como PDF en el cuadro de impresión. Las imágenes externas siguen bloqueadas.
mail-file-details = Detalles del archivo y huella digital
mail-file-type = Tipo de archivo
mail-file-extension = Extensión
mail-file-detected = Firma detectada
mail-file-unknown = Sin firma reconocida
mail-file-size = Tamaño
mail-file-size-bytes = { $count } bytes
mail-file-hash-unavailable = Huella digital no disponible (requiere un navegador seguro y un archivo de hasta 64 MiB).
mail-file-fingerprint-help = Una huella digital identifica exactamente estos bytes. No demuestra que el archivo sea seguro ni quién lo creó.
mail-file-warning-executable = Este archivo puede ejecutar programas o cambiar la configuración del sistema. Descárguelo solo si confía en su origen.
mail-file-warning-macro = Este documento puede contener macros. No active las macros salvo que haya verificado al remitente y la finalidad.
mail-file-warning-disguisedName = El nombre del archivo puede disimular su extensión real. Compruebe los detalles del archivo antes de descargarlo.
mail-file-warning-typeMismatch = La firma detectada no coincide con el tipo declarado. Se ha bloqueado la vista previa automática.
mail-file-warning-activeDocument = Este formato puede contener contenido activo. Su vista previa está aislada; abrir una copia descargada puede comportarse de otro modo.
mail-file-preview-blocked = La vista previa automática está desactivada para este tipo de archivo o por discrepancia de tipo.
mail-quote-header-from = De:
mail-quote-header-sent = Enviado:
mail-quote-header-to = Para:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Asunto:
mail-spam-score = Análisis de spam
mail-spam-score-loading = Comprobando encabezados de spam…
mail-spam-score-unavailable = Puntuación de spam no disponible
mail-spam-score-unverified = Indicado por los encabezados del mensaje, sin verificación independiente. Los encabezados aportados por el remitente pueden estar falsificados.
mail-spam-score-positive = El encabezado indica spam.
mail-spam-score-negative = El encabezado indica un resultado de no spam.
mail-spam-score-unknown = No se proporcionó ningún veredicto con esta puntuación.
mail-external-recipients = Destinatarios externos
mail-external-count = { $count } destinatario(s) fuera de los dominios de confianza
mail-external-help = Compruebe los destinatarios de Para, Cc y Cco antes de enviar información confidencial. La confianza se basa en la identidad de envío seleccionada y su política de dominio efectiva.
mail-external-reviewed = He comprobado estos destinatarios externos.
mail-external-confirm-required = Revise y confirme los destinatarios externos antes de enviar.
mail-quick-reply = Respuesta rápida
mail-quick-reply-help = Ofrecer una pestaña de respuesta compacta con los destinatarios ya rellenados. Desactivado por defecto. Siguen aplicándose las comprobaciones de seguridad y opciones de envío habituales.
mail-quick-reply-expand = Editar destinatarios y asunto
mail-attachment-policy-loading = Comprobando la política de adjuntos…
mail-attachment-policy-unavailable = No se pudo comprobar la política de adjuntos. Inténtelo de nuevo antes de abrir este archivo.
mail-attachment-policy-blocked = Su administrador ha bloqueado este adjunto o su descarga original.
mail-attachment-policy-warning = Su administrador recomienda precaución con este adjunto. Continúe solo si confía en su origen.
mail-attachment-policy-acknowledge = Lo entiendo, continuar
mail-attachment-policy-no-preview = Su administrador ha desactivado las vistas previas de adjuntos. Las descargas originales pueden seguir disponibles.
mail-time-sent = Enviado
mail-time-received = Recibido
mail-time-unknown = No indicado
mail-time-comparison = Hora en { $zone }
mail-time-privacy = La zona de comparación es su selección. Las marcas de tiempo de los mensajes no identifican la ubicación de nadie.
mail-time-event-comparison = Comparación de la hora del evento
mail-time-ambiguous = Esta hora local se produce dos veces al cambiar la hora. Se muestran ambos instantes posibles; compruebe el desfase previsto del evento antes de guardar.
mail-bulk-add-label = Añadir etiqueta
mail-select-messages = Seleccionar mensajes
mail-bulk-remove-label = Quitar etiqueta
mail-mark-spam = Marcar como spam
mail-bulk-forward = Reenviar los mensajes seleccionados como adjuntos
mail-bulk-download = Descargar los mensajes seleccionados como mbox
mail-bulk-forward-subject = Mensajes reenviados ({ $count })
mail-bulk-transfer-failed = No se pudieron preparar los mensajes seleccionados. Seleccione hasta 100 mensajes que sumen como máximo 32 MiB y compruebe los permisos de exportación de su cuenta.
mail-toast-labels-added = Etiqueta { $label } añadida a { $count } mensajes
mail-toast-labels-removed = Etiqueta { $label } quitada de { $count } mensajes
mail-action-help-reply = Responder al remitente en una nueva pestaña de redacción.
mail-action-help-reply-all = Responder al remitente y a los demás destinatarios en una nueva pestaña de redacción.
mail-action-help-forward = Revisar este mensaje y reenviarlo a los destinatarios que elija.
mail-action-help-pin = Mantener este mensaje fijado para acceder fácilmente, o quitar la fijación.
mail-action-help-read = Cambiar si este mensaje está marcado como leído. Puede deshacer el cambio.
mail-action-help-flag = Poner o quitar la marca de seguimiento del mensaje.
mail-action-help-archive = Mover este mensaje a la carpeta de archivo configurada.
mail-action-help-delete = Mover este mensaje a la papelera. Puede deshacer el movimiento.
mail-action-help-spam = Marcar este mensaje como correo no deseado y moverlo a la carpeta de spam configurada.
mail-action-help-export = Abrir la página de exportación para descargar este mensaje en el formato que elija.
mail-action-help-sweep = Revisar las acciones para otros mensajes de este remitente antes de aplicarlas.
mail-empty-trash = Vaciar papelera
mail-trash-help = Revise y elimine definitivamente los mensajes que están ahora en la papelera. No se puede deshacer.
mail-trash-preparing = Preparando una instantánea de los mensajes de la papelera…
mail-trash-prepare-failed = No se pudo preparar la vista previa de la papelera. Compruebe su conexión y vuelva a abrirla. No se eliminó ningún mensaje.
mail-trash-confirm = ¿Eliminar definitivamente { $count } mensajes de la papelera? No se puede deshacer. Los mensajes que lleguen después de esta vista previa quedan excluidos; los mensajes sacados de la papelera se omiten.
mail-trash-limit = Esta operación incluye hasta { $count } mensajes. Quedan más en la papelera; revise otra operación después para eliminarlos.
mail-trash-permanent = Eliminar definitivamente
mail-trash-stop = Detener tras la solicitud actual
mail-trash-progress = { $deleted } eliminados; { $skipped } mensajes omitidos por no estar ya en la papelera; { $remaining } restantes en esta vista previa.
mail-trash-stopped = Eliminación detenida. Eliminados confirmados: { $deleted }; omitidos: { $skipped }; restantes: { $remaining }; resultado desconocido: { $uncertain }. Compruebe la papelera antes de iniciar otra operación.
mail-links-title = Enlaces del mensaje
mail-links-review = Revisar enlaces ({ $count })
mail-links-help = Los enlaces del mensaje solo se abren después de revisar aquí su destino. No se obtiene ningún destino automáticamente.
mail-links-inert = Use Revisar enlaces sobre el mensaje para inspeccionar y abrir este destino.
mail-links-limit = Se listan los primeros 100 destinos compatibles distintos. Los demás enlaces permanecen inactivos en el mensaje.
mail-links-destination = Destino que se abrirá
mail-links-external = Este destino procede del remitente del mensaje. Al abrirlo saldrá de Mailwoman; revise la dirección antes de continuar.
mail-links-open = Abrir en una pestaña nueva
mail-links-compose = Revisar un mensaje nuevo
mail-links-warning-http = Esta dirección usa HTTP sin cifrar.
mail-links-warning-address = Este destino usa una dirección IP en lugar de un nombre de dominio.
mail-links-warning-international = Este dominio contiene un nombre internacionalizado. Compruebe con atención su ortografía ASCII.
mail-links-warning-mismatch = La dirección web visible nombra un host distinto del destino real.
mail-links-warning-direction = El texto del enlace contiene caracteres de cambio de dirección que pueden disimular su dirección.
mail-outbox-uncertain = Estado de entrega desconocido
mail-outbox-uncertain-note = El envío comenzó, pero no se pudo confirmar la entrega. Este mensaje no se reenviará automáticamente. Compruebe Enviados o confirme con el destinatario antes de volver a enviar.
mail-conversation-title = Ver conversación
mail-conversation-help = Examine los mensajes disponibles de esta conversación en esta cuenta, en todas las carpetas. Abrir un mensaje no cambia su selección de carpeta.
mail-conversation-count = Mostrando { $loaded } de { $total } mensajes de la conversación.
mail-conversation-other-folder = Otra carpeta
mail-conversation-loading = Cargando conversación…
mail-conversation-missing = { $count } mensaje(s) ya no están disponibles.
mail-conversation-error = No se pudo cargar la conversación. Inténtelo de nuevo.
mail-conversation-more = Cargar más mensajes

# -- Machine translation (2026-11): added to match en ---------------------
mail-conversation-limit = Esta vista está limitada a { $count } mensajes.
mail-compose-request-delivery = Solicitar notificaciones de entrega
mail-compose-request-delivery-hint = Pedir a los servidores de correo receptores que informen de la entrega, el fallo o el retraso. Esto no solicita ni demuestra que alguien haya leído el mensaje. Los informes devueltos incluyen encabezados, no el cuerpo completo.
mail-compose-delivery-unavailable = Las notificaciones de entrega no están disponibles o su administrador las ha desactivado para esta cuenta. Una notificación solicitada explícitamente no se quitará de forma silenciosa.
mail-compose-account-changed = Esta redacción pertenece a otro inicio de sesión. Inicie sesión en la cuenta correcta y vuelva a abrir el borrador.
mail-context-open-message = Abrir mensaje
mail-context-open-folder = Abrir carpeta
mail-context-download = Descargar mensaje
mail-context-refresh = Actualizar carpeta
mail-context-folder-settings = Ajustes de la carpeta
mail-context-failed = No se pudo completar la acción. Inténtelo de nuevo.
mail-sharing-unavailable = Compartir carpetas no está disponible o ya no tiene permiso para gestionarlo.
mail-trash-cancelling = Deteniendo tras la solicitud actual. Eliminados confirmados: { $deleted }; omitidos: { $skipped }; restantes: { $remaining }. Las eliminaciones ya aceptadas no se pueden deshacer.
mail-trash-already-running = La eliminación de la papelera ya está en curso. Use su notificación de progreso para detener los lotes futuros.
mail-context-search-sender = Buscar este remitente
mail-context-search-subject = Buscar el mismo asunto
mail-drag-too-many = Arrastre como máximo { $count } mensajes a la vez. Seleccione un grupo más pequeño antes de mover.
mail-subtab-reorder-help = Arrastre para reordenar las pestañas, o pulse Ctrl+Mayús+Izquierda o Derecha en la pestaña activa.
mail-compose-ribbon = Cinta del editor
mail-compose-ribbon-format = Dar formato al texto
mail-compose-ribbon-insert = Insertar
mail-compose-font = Fuente
mail-compose-font-size = Tamaño de fuente
mail-compose-font-inherit = Predeterminado
mail-compose-undo = Deshacer
mail-compose-redo = Rehacer
mail-compose-undo-tip = Deshacer ({ $shortcut })
mail-compose-redo-tip = Rehacer ({ $shortcut })
mail-workspace-limit = Cierre una pestaña antes de abrir otra. Se pueden conservar hasta 32 pestañas.
mail-compose-private-storage-unavailable = La recuperación local de borradores no está disponible en esta sesión. Mantenga abierto este editor hasta que envíe o copie su mensaje.
mail-offline-storage-unavailable = Los cambios sin conexión no estarán disponibles hasta que esta sesión haya verificado la autoridad de almacenamiento. Vuelva a conectarse e inicie sesión de nuevo.
mail-draft-status-idle = Aún no guardado en la cuenta
mail-draft-status-pending = Cambios sin guardar
mail-draft-status-saving = Guardando en Borradores…
mail-draft-status-saved = Guardado en los borradores de la cuenta
mail-draft-status-error = El borrador no se guardó. Su copia de recuperación local sigue disponible.
mail-draft-status-uncertain = Falta la confirmación del guardado. Compruebe Borradores antes de crear otra copia; el guardado automático está en pausa.
mail-draft-status-conflict = Este borrador cambió en otro dispositivo. Vuelva a abrir su copia de la cuenta antes de continuar.
mail-draft-status-unsupported = Esta cuenta no puede confirmar por ahora los guardados en su carpeta de Borradores.
mail-draft-protected = El contenido citado cifrado o descifrado no se guarda como texto sin formato. Cualquier borrador anterior de la cuenta permanece hasta que lo descarte explícitamente.
mail-draft-save-now = Guardar en Borradores
mail-draft-save-failed = No se pudo guardar este borrador en su cuenta. Compruebe el editor antes de cerrarlo.
mail-draft-uncertain = Es posible que el borrador se haya guardado. Los reintentos automáticos están en pausa; compruebe su carpeta de Borradores.
mail-draft-conflict = Otro dispositivo cambió este borrador. Sus ediciones no han sobrescrito su copia.
mail-draft-cleanup-pending = Se guardó el nuevo borrador, pero no se pudo quitar su copia anterior. Revise Borradores antes de volver a guardar.
mail-draft-account-list = Borradores de la cuenta
mail-draft-loading = Cargando borradores de la cuenta…
mail-draft-close-recovery = Cerrar y conservar solo la recuperación local
mail-draft-discard-account = Descartar
mail-draft-discard-pending = No se pudo confirmar la eliminación del borrador. Su registro en la cuenta permanece; vuelva a intentar Descartar tras revisar la carpeta de Borradores.
mail-draft-limit = Esta cuenta ya tiene 100 borradores guardados. Descarte un borrador de la cuenta que no necesite antes de guardar uno nuevo.
mail-draft-quota-help = Se conservan hasta 100 borradores de la cuenta. Descartar elimina la copia confirmada en el servidor antes de liberar su espacio.
mail-draft-discard-confirm = Confirmar eliminación del borrador
mail-draft-discard-confirm-detail = ¿Eliminar definitivamente «{ $subject }» de los borradores de la cuenta? Si cierra el editor en su lugar, el borrador se conserva.
mail-draft-discard-title = ¿Descartar este borrador?
mail-draft-keep-editing = Seguir editando
mail-draft-discard-action = Descartar
mail-draft-close-title = ¿Descartar los cambios sin guardar?
mail-draft-close-protected = Este mensaje contiene contenido cifrado o descifrado, por lo que no se guarda como borrador. Al cerrarlo se descartan estos cambios. Un borrador anterior de la cuenta, si lo hay, permanece en Borradores.
mail-draft-close-unstored = En esta sesión no se pueden guardar borradores, así que al cerrar este mensaje se descarta.
mail-workspace-view-mode = Disposición de vistas
mail-workspace-tabs-help = Mostrar una sola vista activa manteniendo abiertas las demás pestañas.
mail-workspace-side-by-side-help = Mostrar dos vistas seleccionadas una junto a otra. Arrastre una pestaña a una vista para reemplazarla.
mail-workspace-stack-help = Mostrar dos vistas seleccionadas una encima de otra. Arrastre una pestaña a una vista para reemplazarla.
mail-workspace-masonry-side-by-side = Mosaico lado a lado
mail-workspace-masonry-side-by-side-help = Organizar todas las vistas abiertas en columnas escalonadas. En pantallas pequeñas se muestra la pestaña activa.
mail-workspace-masonry-stack = Mosaico apilado
mail-workspace-masonry-stack-help = Organizar todas las vistas abiertas en filas escalonadas. En pantallas pequeñas se muestra la pestaña activa.
mail-workspace-view-first = Primera vista
mail-workspace-view-second = Segunda vista
mail-workspace-view-empty = Arrastre aquí una pestaña abierta para elegir esta vista.
mail-popup-blocked = Permita las ventanas emergentes de Mailwoman para separar esta pestaña. Su pestaña permanece aquí.
mail-popup-unavailable = No se pudo cargar la ventana separada. Su pestaña y los cambios sin guardar permanecen en la ventana principal. Inténtelo de nuevo tras actualizar la aplicación.
mail-popup-reattach = Volver a la ventana principal
mail-popup-action-reattached = Se volvió a la ventana principal para completar esta acción.
mail-popup-detached = Esta pestaña está abierta en una ventana aparte. Al cerrar esa ventana, vuelve aquí.
mail-compose-from-custom = Otra dirección autorizada…
mail-compose-from-alias = Dirección de remitente para este mensaje
mail-compose-from-alias-help = Use un alias desde el que esté autorizado a enviar. Esto solo cambia este borrador; la autorización se comprueba antes de enviar.
mail-compose-from-refused = Esta dirección no está autorizada para enviar desde esta cuenta.
mail-compose-from-unavailable = No se pudo verificar esta dirección de remitente. Su mensaje no se ha enviado.
mail-compose-from-invalid = Introduzca una dirección de correo válida para el remitente.
mail-compose-from-toggle = Mostrar u ocultar el remitente de este mensaje
mail-compose-ribbon-security = Seguridad
mail-compose-ribbon-delivery = Entrega
mail-compose-ribbon-recipients = Destinatarios
mail-compose-ribbon-message-type = Tipo de mensaje
mail-compose-recall-help = Cancelar mensajes que aún están retenidos para deshacer o para entrega programada. Los mensajes ya entregados no se pueden recuperar.
mail-compose-recall-loading = Comprobando mensajes en cola…
mail-compose-recall-empty = No hay mensajes en cola disponibles para cancelar.
mail-compose-recall-failed = No se pudo confirmar el estado de la cola. Cierre y vuelva a abrir este panel para actualizarlo antes de otra acción.
mail-compose-importance = Importancia
mail-compose-importance-high = Alta
mail-compose-importance-normal = Normal
mail-compose-importance-low = Baja
mail-insert-account-title = Insertar elemento de la cuenta
mail-insert-account-help = Adjuntar una exportación de calendario, una tarjeta de contacto o un mensaje original de esta cuenta.
mail-insert-account-close = Cerrar el selector de elementos de la cuenta
mail-insert-account-kind = Tipo de elemento
mail-insert-account-event = Evento de calendario (.ics)
mail-insert-account-calendar = Calendario (.ics)
mail-insert-account-contact = Tarjeta de contacto (.vcf)
mail-insert-account-message = Mensaje original (.eml)
mail-insert-account-search = Filtrar elementos listados
mail-insert-account-selection = Seleccione un elemento
mail-insert-account-scope = Solo se listan elementos de la cuenta actual. Los mensajes originales siguen la política de exportación del servidor.
mail-insert-account-limited = Se muestran hasta 100 elementos. El filtro solo busca en esta lista.
mail-insert-account-unavailable = No hay exportaciones de cuenta compatibles disponibles.
mail-insert-account-empty = No hay elementos coincidentes.
mail-insert-account-attach = Adjuntar elemento seleccionado
mail-insert-account-loading = Cargando elemento de la cuenta…
mail-insert-account-error = No se pudo adjuntar el elemento. Compruebe su acceso y la política de exportación e inténtelo de nuevo. Los calendarios grandes pueden requerir exportaciones de eventos individuales.
mail-compose-html-source = Código HTML
mail-editor-clear-format = Borrar formato
mail-compose-format-html = HTML
mail-compose-html-visual = Editor visual
mail-compose-html-help = Edite el HTML directamente. Solo se guarda y envía el formato de correo compatible; se eliminan scripts, imágenes externas y marcado no seguro.
mail-compose-html-processing = Preparando HTML seguro…
mail-compose-html-failed = No se pudo preparar el HTML. Edítelo o inténtelo de nuevo antes de guardar o enviar.
mail-compose-check-names = Comprobar nombres
mail-compose-check-names-help = Resolver nombres exactos a partir de sus contactos cargados y comprobar el formato de las direcciones. No verifica la entrega en el buzón.
mail-compose-names-unresolved = Elija una dirección de correo completa para estos nombres sin resolver o ambiguos: { $names }
mail-compose-names-checked = Formato de destinatarios comprobado; { $count } nombre(s) de contacto resuelto(s). No se ha verificado la entrega en el buzón.
mail-compose-ribbon-table = Tabla
mail-compose-group-clipboard = Portapapeles
mail-compose-group-paragraph = Párrafo
mail-compose-group-editing = Edición
mail-compose-group-view = Vista
mail-compose-dark-background = Fondo oscuro
mail-compose-dark-background-help = Mostrar el mensaje sobre fondo oscuro mientras escribe. Solo cambia su pantalla: el mensaje se envía sin color de fondo.
mail-compose-group-assist = Asistente de escritura
mail-editor-counts = Estadísticas del editor
mail-editor-words = { $count ->
    [one] { $count } palabra
   *[other] { $count } palabras
}
mail-editor-paragraphs = { $count ->
    [one] { $count } párrafo
   *[other] { $count } párrafos
}
mail-editor-characters = { $count ->
    [one] { $count } carácter
   *[other] { $count } caracteres
}
mail-editor-selection-required = Seleccione texto o celdas de tabla adecuados antes de usar esta acción.
mail-editor-clipboard-unavailable = No hubo acceso al portapapeles. Use los atajos de Copiar, Cortar o Pegar de su navegador.
mail-editor-selection-changed = La selección cambió. Seleccione el texto deseado e inténtelo de nuevo.
mail-editor-clipboard = Portapapeles y formato
mail-editor-copy = Copiar
mail-editor-cut = Cortar
