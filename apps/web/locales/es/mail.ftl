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
