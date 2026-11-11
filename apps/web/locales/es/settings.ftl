# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: settings. Source: ../en/settings.ftl

settings-title = Configuración
settings-appearance = Apariencia
settings-close = Cerrar configuración

settings-theme = Tema
settings-density = Densidad
settings-accent = Color de acento
settings-font = Fuente de la interfaz
settings-layout = Diseño

settings-density-compact = Compacta
settings-density-cozy = Cómoda
settings-density-relaxed = Amplia

settings-font-default = Predeterminada
settings-font-system = Sistema
settings-font-serif = Serif
settings-font-mono = Monoespaciada

settings-layout-default = Predeterminado
settings-layout-ribbon = Cinta

settings-cancel = Cancelar
settings-edit = Editar
settings-delete = Eliminar
settings-save = Guardar
settings-saved = Guardado

settings-2fa-title = Autenticación en dos pasos
settings-2fa-intro = Añada un segundo factor para exigir algo más que una contraseña al iniciar sesión.
settings-2fa-policy-required = Su organización exige un segundo factor en esta cuenta.
settings-2fa-enabled = Activada
settings-2fa-recovery-title = Códigos de recuperación
settings-2fa-recovery-once = Guárdelos ahora. Se muestran una sola vez y no se pueden volver a obtener. Cada código funciona una sola vez.
settings-2fa-recovery-ack = He guardado estos códigos
settings-2fa-totp-title = Aplicación de autenticación
settings-2fa-totp-enrol = Configurar una aplicación de autenticación
settings-2fa-totp-scan = Añada este secreto a su aplicación de autenticación y después introduzca el código que muestra.
settings-2fa-totp-uri-link = Abrir en una aplicación de autenticación
settings-2fa-totp-code-label = Código de autenticación
settings-2fa-totp-confirm = Confirmar
settings-2fa-totp-disable = Quitar aplicación de autenticación
settings-2fa-passkey-title = Llaves de acceso
settings-2fa-passkey-remove = Quitar
settings-2fa-passkey-unsupported = Este navegador no admite llaves de acceso.
settings-2fa-passkey-label-placeholder = Nombre de la llave de acceso (opcional)
settings-2fa-passkey-add = Añadir una llave de acceso
settings-2fa-recovery-remaining = { $count ->
        [one] Queda { $count } código de recuperación
       *[other] Quedan { $count } códigos de recuperación
    }
settings-2fa-recovery-regenerate = Generar nuevos códigos de recuperación
settings-2fa-recovery-code-label = Código de recuperación
settings-2fa-error-generic = No funcionó. Inténtelo de nuevo.
settings-2fa-error-code = No se pudo verificar ese código.
settings-2fa-error-passkey = No se pudo registrar la llave de acceso.

settings-2fa-challenge-title = Autenticación en dos pasos
settings-2fa-challenge-intro = Confirme su segundo factor para terminar de iniciar sesión.
settings-2fa-challenge-failed = No se pudo verificar. Inténtelo de nuevo.
settings-2fa-challenge-use-passkey = Usar una llave de acceso
settings-2fa-challenge-method = Elegir un método
settings-2fa-challenge-totp-tab = Código de autenticación
settings-2fa-challenge-recovery-tab = Código de recuperación
settings-2fa-challenge-verify = Verificar

settings-sessions-title = Sesiones activas
settings-sessions-intro = Sesiones iniciadas actualmente en esta cuenta.
settings-sessions-last-seen = Última actividad { $when }
settings-sessions-revoke = Cerrar sesión
settings-sessions-current = Esta sesión
settings-sessions-revoke-others = { $count ->
        [one] Cerrar { $count } sesión más
       *[other] Cerrar otras { $count } sesiones
    }
settings-sessions-error = No se pudo cambiar la sesión.

settings-sig-title = Firmas
settings-sig-intro = Texto que se añade a los mensajes que envía. Una firma puede ser la predeterminada.
settings-sig-default = Predeterminada
settings-sig-new = Nueva firma
settings-sig-name-label = Nombre
settings-sig-body-label = Firma
settings-sig-default-label = Usar como predeterminada
settings-sig-error-name = Asigne un nombre a la firma.
settings-sig-error-generic = No se pudo guardar la firma.

settings-ident-title = Identidades
settings-ident-intro = El nombre y la dirección con los que envía. Cada identidad puede usar su propia firma.
settings-ident-new = Nueva identidad
settings-ident-name-label = Nombre para mostrar
settings-ident-email-label = Dirección de correo electrónico
settings-ident-replyto-label = Dirección de respuesta (opcional)
settings-ident-signature-label = Firma predeterminada
settings-ident-signature-none = Ninguna
settings-ident-error-fields = Introduzca un nombre y una dirección de correo electrónico válida.
settings-ident-error-generic = No se pudo guardar la identidad.

settings-notif-title = Notificaciones
settings-notif-intro = Elija qué mensajes nuevos le notifican y cuándo guardar silencio.
settings-notif-enabled-label = Mostrar notificaciones de correo nuevo
settings-notif-quiet-title = Horas de silencio
settings-notif-quiet-enabled-label = Silenciar las notificaciones durante un horario fijo
settings-notif-quiet-start = Desde
settings-notif-quiet-end = Hasta
settings-notif-rules-title = Reglas
settings-notif-rule-match-placeholder = El remitente, el buzón o el asunto contiene…
settings-notif-rule-action-label = Acción
settings-notif-rule-notify = Notificar
settings-notif-rule-mute = Silenciar
settings-notif-rule-add = Añadir una regla
settings-notif-error = No se pudo guardar la configuración de notificaciones.

settings-search-title = Búsquedas guardadas
settings-search-intro = Mostrar una búsqueda guardada como carpeta en su lista de buzones.
settings-search-empty = Aún no tiene búsquedas guardadas.
settings-search-as-folder = Mostrar { $name } como carpeta
settings-search-as-folder-label = Mostrar como carpeta
settings-search-error = No se pudo cambiar la búsqueda guardada.

settings-kbd-title = Atajos de teclado
settings-kbd-intro = Elija un conjunto de atajos. Los cambios se aplican en este dispositivo.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Desactivados (sin atajos)
settings-kbd-action-compose = Redactar
settings-kbd-action-archive = Archivar
settings-kbd-action-reply = Responder
settings-kbd-action-next = Mensaje siguiente
settings-kbd-action-previous = Mensaje anterior
settings-kbd-action-search = Buscar

settings-offline-title = Caché sin conexión
settings-offline-intro = Cuánto correo conservar en este dispositivo para usarlo sin conexión y cómo liberar espacio.
settings-offline-budget-label = Tamaño de la caché (MB)
settings-offline-retention-label = Conservar durante (días)
settings-offline-strategy-label = Al alcanzar el límite
settings-offline-lru = Quitar lo usado hace más tiempo
settings-offline-oldest = Quitar primero lo más antiguo
settings-offline-manual = Solo cuando la vacíe
settings-offline-purge = Vaciar ahora la caché sin conexión

settings-dir-title = Dirección de la interfaz
settings-dir-intro = Seguir el idioma o forzar una dirección. De derecha a izquierda refleja el diseño.
settings-dir-auto = Automática
settings-dir-ltr = De izquierda a derecha
settings-dir-rtl = De derecha a izquierda
settings-dir-preview-title = Vista previa
settings-dir-preview-body = Este texto se refleja cuando la dirección es de derecha a izquierda.
settings-mail-features = Funciones de correo
settings-mail-feature-allowSend = Permitir el envío de mensajes
settings-mail-feature-allowSend-help = Si está desactivado, Enviar y su atajo quedan deshabilitados. Los borradores se pueden seguir guardando.
settings-mail-feature-allowReply = Permitir responder y responder a todos
settings-mail-feature-allowReply-help = Si está desactivado, las acciones Responder y Responder a todos y sus atajos no se ofrecen.
settings-mail-feature-allowForward = Permitir reenviar
settings-mail-feature-allowForward-help = Si está desactivado, las acciones Reenviar y su atajo no se ofrecen.
settings-mail-feature-allowDownload = Permitir descargar mensajes y archivos adjuntos
settings-mail-feature-allowDownload-help = Si está desactivado, no se ofrece la descarga de mensajes ni de archivos adjuntos.
settings-mail-feature-allowPrint = Permitir imprimir
settings-mail-feature-allowPrint-help = Si está desactivado, la acción Imprimir no se ofrece.
settings-mail-feature-showSpam = Mostrar la carpeta Spam
settings-mail-feature-showSpam-help = Si está desactivado, la carpeta Spam y los mensajes marcados como spam se ocultan de la lista de carpetas y de los resultados de búsqueda. No se elimina nada.
settings-mail-feature-showAuthStatus = Mostrar los resultados de autenticación del remitente
settings-mail-feature-showAuthStatus-help = Muestra en el lector los resultados SPF, DKIM y DMARC de un mensaje.
settings-mail-feature-closeOnDelete = Cerrar un mensaje al eliminarlo
settings-mail-feature-closeOnDelete-help = Cierra el mensaje, la pestaña o la ventana abiertos cuando ese mensaje se elimina o se mueve a la Papelera.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Ruta de ajustes
settings-crumb-loading = Cargando nombre
settings-crumb-new = Nuevo
settings-crumb-edit = Editar
settings-badge-color = Color de la insignia de no leídos
settings-badge-color-accent = Igual que el acento
settings-badge-color-accent-lighter = Acento más claro
settings-badge-color-accent-darker = Acento más oscuro
settings-badge-color-option = Insignias { $color }
settings-appearance-theme-card = Tema y cambio
settings-appearance-theme-card-help = Elija un tema y si sigue al sistema o a la hora del día.
settings-appearance-colours-card = Colores
settings-appearance-colours-card-help = El acento usado en acciones y selección, y el color de los contadores de no leídos.
settings-appearance-text-card = Texto y diseño
settings-appearance-text-card-help = La densidad de la interfaz, la fuente que usa y cómo se organiza el correo.
settings-sig-create-variant = Crear variante
settings-sig-error-existing = Ya existe una firma con este nombre. Elija otro nombre.
settings-sig-save-failed = No se pudo guardar la firma. { $reason }
settings-ident-signature-new = Mensajes nuevos
settings-ident-signature-account = Usar la firma de la cuenta
settings-ident-signature-replies = Respuestas
settings-ident-signature-inherit = Usar la firma de mensajes nuevos
settings-ident-signature-unavailable = Firma no disponible: { $name }
settings-ident-signature-help = Cree variantes de firma en Firmas y asigne una aquí. La misma variante puede usarse para ambos casos.
settings-ident-save-failed = No se pudo guardar la identidad. { $reason }
settings-search-save-failed = No se pudo guardar la búsqueda guardada. { $reason }
settings-kbd-preset-title = Preajuste de atajos
settings-editor-discard-title = ¿Descartar los cambios sin guardar?
settings-editor-discard-detail = Sus cambios no se han guardado. Siga editando o descarte este borrador para salir del editor.
settings-editor-keep = Seguir editando
settings-editor-discard = Descartar cambios
settings-editor-missing = Este elemento ya no existe. Vuelva a la lista y elija otro elemento.
settings-editor-back = Volver a la lista
settings-sig-edit-title = Editar firma
settings-ident-edit-title = Editar identidad
settings-sections = Secciones de ajustes
settings-category-account = Cuenta
settings-category-mail = Correo
settings-category-interface = Interfaz
settings-category-integrations = Integraciones
settings-category-about = Acerca de
settings-nav-preferences = Teclado y pantalla
settings-nav-rules = Reglas de correo
settings-nav-access = Acceso a la API
settings-nav-metadata = Información del servidor
settings-folders-title = Carpetas y barra lateral
settings-folders-help = Fije carpetas por encima de las demás, elija un icono y arrastre o use los botones de mover para organizar las carpetas dentro de cada grupo. Estas opciones se sincronizan con esta cuenta y no mueven mensajes ni renombran carpetas.
settings-folders-pin = Fijar { $name }
settings-folders-icon = Icono de { $name }
settings-folders-automatic = Función de carpeta automática
settings-folders-drag = Arrastrar { $name } para reordenar
settings-folders-up = Subir { $name }
settings-folders-down = Bajar { $name }
settings-folders-sync-idle = Inicie sesión para sincronizar las preferencias de carpetas.
settings-folders-sync-loading = Cargando preferencias de la cuenta…
settings-folders-sync-saving = Guardando preferencias de la cuenta…
settings-folders-sync-synced = Preferencias sincronizadas con esta cuenta.
settings-folders-sync-error = No se pudieron sincronizar las preferencias. Sus cambios permanecen en esta sesión y se reintentarán cuando vuelva la conexión.
settings-folder-icon-folder = Carpeta
settings-folder-icon-star = Estrella
settings-folder-icon-flag = Bandera
settings-folder-icon-tag = Etiqueta
settings-folder-icon-clock = Reloj
settings-folder-icon-calendar = Calendario
settings-folder-icon-file = Documento
settings-folder-icon-image = Imagen
settings-folder-icon-lock = Candado
settings-folder-icon-users = Personas
settings-folder-icon-archive = Archivo
settings-folder-icon-mail = Sobre
settings-folder-icon-attachment = Adjunto
settings-folder-icon-check = Marca de verificación
settings-folder-icon-alert = Alerta
settings-search-new = Nueva carpeta de búsqueda
settings-search-editor = Carpeta de búsqueda
settings-search-editor-help = Los mensajes deben cumplir todas las condiciones. Deje una condición vacía para ignorarla. Las carpetas de búsqueda muestran los mensajes coincidentes sin moverlos ni copiarlos.
settings-search-name = Nombre de la carpeta
settings-search-name-required = Introduzca un nombre para esta carpeta de búsqueda.
settings-search-invalid = Esta búsqueda guardada tiene un filtro no válido. No se puede editar con seguridad.
settings-search-saved = Carpeta de búsqueda «{ $name }» guardada.
settings-search-field-text = Buscar en todo el texto del mensaje
settings-search-field-from = El remitente contiene
settings-search-field-to = El destinatario contiene
settings-search-field-subject = El asunto contiene
settings-search-field-body = El cuerpo del mensaje contiene
settings-search-field-filename = El nombre de archivo del adjunto contiene
settings-search-field-inMailbox = Limitar al identificador de buzón (opcional)
settings-search-field-after = Recibido después de (fecha y hora UTC)
settings-search-field-before = Recibido antes de (fecha y hora UTC)
settings-search-attachment = Adjuntos
settings-search-any = Cualquier mensaje
settings-search-with-attachments = Con adjuntos
settings-search-without-attachments = Sin adjuntos
settings-search-unread = Solo mensajes sin leer
settings-search-flagged = Solo mensajes marcados
settings-sessions-search = Buscar cuenta, dispositivo o IP
settings-sessions-account = Cuenta
settings-sessions-device = Dispositivo y cliente
settings-sessions-location = IP y último país conocido
settings-sessions-location-unavailable = País no disponible
settings-sessions-activity = Actividad
settings-sessions-actions = Acciones
settings-sessions-revoke-device = Cerrar sesión en { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Redacción y respuestas
settings-quote-style = Estilo de encabezado de respuestas y reenvíos
settings-quote-classic = Respuesta citada clásica
settings-quote-outlook = Encabezado al estilo de Outlook
settings-quote-help = El estilo de Outlook añade De, Enviado, Para, Cc y Asunto encima del mensaje original, tanto en texto enriquecido como en texto sin formato. Los borradores existentes no cambian.
settings-external-warning = Avisos de destinatarios externos
settings-external-off = Desactivado
settings-external-warn = Mostrar un aviso
settings-external-confirm = Exigir confirmación antes de enviar
settings-external-help = Compara Para, Cc y Cco con el dominio de la identidad seleccionada y la lista de dominios de confianza. Esto protege los envíos desde este editor; no es una regla de entrega del servidor.
settings-trusted-domains = Dominios de destinatarios de confianza
settings-trusted-domains-help = Introduzca hasta 100 dominios exactos, uno por línea. Los subdominios deben listarse por separado. El dominio de la identidad de envío siempre es de confianza. No se aceptan URL, direcciones de correo ni comodines.
settings-trusted-domains-invalid = Introduzca hasta 100 nombres de dominio válidos, sin URL, direcciones de correo ni comodines.
settings-trusted-domains-save = Guardar dominios de confianza
settings-shortcuts-enabled = Activar atajos de la aplicación
settings-shortcuts-action = Acción
settings-shortcuts-binding = Asignación
settings-shortcuts-edit = Editar atajo
settings-shortcuts-edit-action = Editar el atajo de { $action }
settings-shortcuts-help = Use inherit para su preajuste seleccionado, disabled para ningún atajo, o una combinación como Ctrl+Shift+K. Ctrl equivale a Cmd en macOS. Las asignaciones en conflicto están inactivas. Los campos de texto y los cuadros de diálogo conservan sus teclas habituales; las combinaciones reservadas por el navegador pueden no estar disponibles.
settings-shortcuts-invalid = Introduzca inherit, disabled o una tecla única válida con los modificadores opcionales Ctrl, Alt y Shift.
settings-shortcuts-conflict = Dos acciones usan el mismo atajo. Cambie una asignación; los atajos en conflicto no se ejecutarán.

# -- Machine translation (2026-11): added to match en ---------------------
settings-shortcuts-discard = ¿Descartar el cambio de atajo sin guardar?
settings-shortcuts-save = Guardar atajo
settings-shortcuts-back = Volver a los atajos
settings-shortcuts-off = Su preajuste Desactivado desactiva todos los atajos, incluidas las asignaciones personalizadas. Elija otro preajuste en Preferencias para usar estas asignaciones.
settings-kbd-action-replyAll = Responder a todos
settings-kbd-action-forward = Reenviar
settings-kbd-action-refresh = Actualizar mensajes
settings-kbd-action-settings = Abrir ajustes
settings-kbd-action-trash = Mover a la papelera
settings-kbd-action-markRead = Marcar como leído
settings-kbd-action-markUnread = Marcar como no leído
settings-kbd-action-spam = Marcar como spam
settings-kbd-action-zoomIn = Acercar mensaje
settings-kbd-action-zoomOut = Alejar mensaje
settings-kbd-action-zoomReset = Restablecer zoom del mensaje
settings-kbd-action-renameFolder = Renombrar carpeta
settings-kbd-action-openInTab = Abrir mensaje en su propia pestaña
settings-notif-device-title = Permiso en este dispositivo
settings-notif-device-help = Sus preferencias de notificación siguen a su cuenta. El permiso del navegador pertenece a este dispositivo; Mailwoman solo lo pide cuando usted decide concederlo.
settings-notif-ask-permission = Ofrecer el permiso de notificaciones en dispositivos nuevos
settings-notif-ask-permission-help = Mostrar una solicitud de permiso cuando este dispositivo no haya permitido ni bloqueado las notificaciones. Activado por defecto; esta preferencia se guarda al instante.
settings-notif-permission-default = No se ha solicitado el permiso del navegador.
settings-notif-permission-granted = Las notificaciones del navegador están permitidas en este dispositivo.
settings-notif-permission-denied = Las notificaciones están bloqueadas. Cambie el permiso de este sitio en los ajustes de su navegador para permitirlas.
settings-notif-permission-unsupported = Este navegador no admite aquí las notificaciones del sistema.
settings-notif-permission-request = Permitir notificaciones en este dispositivo
settings-notif-permission-error = El navegador no pudo solicitar el permiso de notificaciones.
settings-notif-timezone = Zona horaria de las horas de silencio (IANA)
settings-notif-timezone-device = Usar la zona horaria de este dispositivo
settings-permissions-background-title = Notificaciones en segundo plano en este navegador
settings-permissions-background-help = Guarde primero las preferencias de notificación y permita las notificaciones del navegador. Registre este navegador para recibir la actividad de la cuenta cuando la aplicación esté cerrada.
settings-permissions-push-unsupported = Web Push en segundo plano no está disponible en este navegador.
settings-permissions-push-registered = Este navegador tiene una suscripción push. Regístrela para vincularla a la cuenta actual.
settings-permissions-push-unregistered = Este navegador no tiene ninguna suscripción push.
settings-permissions-push-register = Registrar notificaciones en segundo plano
settings-permissions-push-remove = Quitar la suscripción del navegador
settings-permissions-push-policy = Su administrador no permite las notificaciones push en segundo plano.
settings-permissions-storage-title = Almacenamiento del navegador
settings-permissions-storage-help = El almacenamiento persistente protege los datos locales de este sitio de la expulsión automática cuando el navegador lo concede. Es local del dispositivo y no cambia la retención en el servidor. Puede borrar los datos del sitio en los ajustes del navegador; se pueden perder cambios sin conexión aún no enviados.
settings-permissions-storage-unsupported = Las solicitudes de almacenamiento persistente no están disponibles en este navegador.
settings-permissions-storage-persistent = El navegador ha concedido almacenamiento persistente a este sitio.
settings-permissions-storage-evictable = El navegador puede expulsar automáticamente los datos locales de este sitio.
settings-permissions-storage-usage = Almacenamiento actual del sitio: { $mib } MiB.
settings-permissions-storage-request = Solicitar almacenamiento persistente
settings-permissions-storage-policy = Su administrador no permite las solicitudes de almacenamiento persistente.
settings-permissions-error = No se pudo completar la acción del navegador. Compruebe los ajustes guardados de la cuenta y los permisos del navegador para este sitio.
settings-notif-background-rules = Las reglas de contenido se evalúan mientras Mailwoman está abierto. Las notificaciones del sistema en segundo plano se suprimen mientras haya reglas de contenido configuradas, porque un despertar opaco en segundo plano no contiene los detalles del mensaje.
settings-nav-regional = Configuración regional
settings-time-title = Zonas horarias
settings-time-help = Elija cómo se muestran las horas de los mensajes y una zona de comparación opcional para programar. Las ubicaciones nunca se deducen de las direcciones de correo.
settings-time-displayZone = Zona horaria de visualización
settings-time-displayZone-help = Use local para este dispositivo, o un nombre IANA como Europe/Lisbon.
settings-time-comparisonZone = Zona horaria de comparación
settings-time-comparisonZone-help = Introduzca un nombre IANA como America/New_York, o déjelo vacío para ocultar las comparaciones.
settings-time-helpers = Mostrar ayudas de zona horaria
settings-time-save = Guardar zona horaria
settings-time-invalid = Introduzca una zona horaria IANA válida. Use local solo para la zona de visualización, o deje vacía la zona de comparación.
settings-time-discard = ¿Descartar los cambios de zona horaria sin guardar?
settings-folders-name = Carpeta
settings-folders-actions = Acciones
settings-folders-edit = Editar la apariencia de { $name }
settings-folders-icon-help = Elija uno de cincuenta iconos temáticos, o mantenga la selección automática según la finalidad de la carpeta. Esto sigue a su cuenta y no renombra el buzón.
settings-folders-save = Guardar apariencia de la carpeta
settings-folders-back = Volver a las carpetas
settings-folders-discard = ¿Descartar los cambios de apariencia de la carpeta sin guardar?
settings-folder-labels = Etiquetas de la carpeta
settings-folder-labels-help = Organice esta carpeta con hasta 32 etiquetas de la cuenta. Esto no etiqueta ni mueve los mensajes que contiene. Gestione los nombres y colores de las etiquetas en Etiquetas.
settings-folders-leave-pending = Estos cambios siguen en cola para sincronizarse. ¿Salir de este editor?
settings-folders-retry = Reintentar el guardado
settings-folder-role-title = Carpetas especiales
settings-folder-role-help = Elija la carpeta real que se usa para cada función de correo. Automático usa la detección del servidor. Usar el valor predeterminado del administrador elimina su anulación. Los destinos en conflicto o ausentes desactivan esa función en lugar de elegir otra carpeta.
settings-folder-role-auto = Detección automática
settings-folder-role-inherit = Usar el valor predeterminado del administrador
settings-folder-role-path = Ruta configurada: { $path }
settings-folder-role-missing = La carpeta configurada no está disponible
settings-folder-role-conflict = Este destino falta o entra en conflicto con otra función. Esa función no estará disponible hasta que se corrija la asignación.
settings-folder-role-refresh-error = Guardado, pero no se pudo actualizar la lista de carpetas. Actualice las carpetas antes de usar la nueva asignación.
settings-folder-role-unsupported = Estas anulaciones requieren carpetas de correo nativas. Las cuentas JMAP externas conservan las funciones de carpeta de su servidor.
settings-folder-role-inbox = Bandeja de entrada
settings-folder-role-drafts = Borradores
settings-folder-role-sent = Correo enviado
settings-folder-role-trash = Papelera
settings-folder-role-junk = Correo no deseado y spam
settings-folder-role-archive = Archivo
settings-sessions-select = Seleccionar
settings-sessions-select-device = Seleccionar la sesión de { $device }
settings-sessions-select-visible = Seleccionar todas las demás sesiones visibles
settings-sessions-current-protected = Esta sesión permanece iniciada y no se puede seleccionar aquí. Use Cerrar sesión para finalizarla.
settings-sessions-selected-title = Cerrar las sesiones seleccionadas ({ $count })
settings-sessions-selected-confirm = Finalizar solo las otras sesiones seleccionadas. Esta sesión permanece iniciada. Las solicitudes fallidas u omitidas se notificarán sin reintentos automáticos.
settings-sessions-selected-action = Cerrar las sesiones seleccionadas
settings-sessions-cancel = Cancelar
settings-sessions-selected-result = Cerradas: { $revoked }. Ya finalizadas: { $unchanged }. Sin confirmar: { $failed }. No intentadas: { $skipped }.
settings-sessions-refresh-failed = No se pudo actualizar la lista de sesiones. Vuelva a cargarla antes de seleccionar más sesiones.
settings-sig-format = Formato de la firma
settings-sig-format-plain = Texto sin formato
settings-sig-format-html = Texto enriquecido (HTML)
settings-sig-html-help = El HTML está limitado a 2 MiB y se sanea antes de guardar y enviar. Se eliminan las imágenes remotas y el contenido activo.
settings-sig-html-source = Editar código HTML
settings-sig-editor-loading = Cargando el editor de firmas…
settings-sig-rich-body = Firma enriquecida
settings-sig-plain-alternative = Alternativa en texto sin formato
settings-sig-plain-help = Los mensajes de texto sin formato usan este texto. Las ediciones enriquecidas lo actualizan hasta que lo personalice; las ediciones del código HTML conservan su alternativa.
settings-sig-account-changed = La cuenta con la que inició sesión ha cambiado. Vuelva a abrir esta firma en la cuenta correcta.
settings-sig-image-type = { $name } no es una imagen PNG, JPEG, GIF o WebP.
settings-sig-image-size = { $name } no cabe: el HTML de una firma, imágenes incluidas, está limitado a 2 MiB.
settings-sig-too-large = Esta firma supera los 2 MiB de HTML. Quite una imagen o parte del formato.
settings-sig-images-help = Las imágenes se almacenan en la firma y se envían como imágenes insertadas en cada mensaje.
settings-folder-icon-inbox = Bandeja de entrada
settings-folder-icon-sent = Enviados
settings-folder-icon-edit = Editar
settings-folder-icon-delete = Eliminar
settings-folder-icon-spam = Spam
settings-folder-icon-search = Búsqueda
settings-folder-icon-pin = Chincheta
settings-folder-icon-link = Enlace
settings-folder-icon-shield = Escudo
settings-folder-icon-unlock = Desbloquear
settings-folder-icon-fileAudio = Audio
settings-folder-icon-fileVideo = Vídeo
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Código
settings-folder-icon-print = Imprimir
settings-folder-icon-download = Descargar
settings-folder-icon-preferences = Preferencias
settings-folder-icon-dashboard = Panel
settings-folder-icon-analytics = Analítica
settings-folder-icon-nextcloud = Archivos en la nube
settings-folder-icon-domains = Dominios
settings-folder-icon-twofactor = Autenticación
settings-folder-icon-assistant = Asistente
settings-folder-icon-integrations = Integraciones
settings-folder-icon-logs = Registros
settings-folder-icon-home = Inicio
settings-folder-icon-briefcase = Maletín
settings-folder-icon-heart = Corazón
settings-folder-icon-shopping = Compras
settings-folder-icon-education = Educación
settings-folder-icon-travel = Viajes
settings-folder-icon-coffee = Café
settings-folder-icon-gift = Regalo

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-icon-bank = Banco
settings-folder-icon-book = Libro
settings-nav-keyboard-display = Teclado y pantalla
settings-2fa-passkey-policy-disabled = Su administrador ha desactivado el inicio de sesión y el registro con claves de acceso para esta cuenta. Las credenciales existentes siguen listadas para que pueda gestionarlas o eliminarlas.
settings-mail-interactions = Selección y movimiento de mensajes
settings-mail-modifier-selection = Seleccionar mensajes con teclas modificadoras
settings-mail-modifier-selection-help = Ctrl o Comando alterna mensajes sueltos. Mayús selecciona un rango, y Mayús con las flechas lo amplía. El botón de selección visible sigue disponible cuando esto está desactivado.
settings-mail-drag-folder = Arrastrar mensajes a carpetas
settings-mail-drag-folder-help = Arrastre un mensaje seleccionado para mover la selección, o uno no seleccionado para mover solo ese mensaje. Solo se aceptan carpetas de esta cuenta. Quienes usan pantalla táctil o teclado pueden usar la barra de herramientas de selección.
settings-search-sections = Buscar en los ajustes
settings-search-no-sections = No hay secciones de ajustes coincidentes.
settings-time-local = Zona horaria local del dispositivo
settings-time-none = Sin zona horaria de comparación
settings-folder-counts = Recuentos de carpetas
settings-folder-counts-none = Ocultar recuentos
settings-folder-counts-unread = Mensajes sin leer
settings-folder-counts-total = Total de mensajes
settings-folder-counts-both = Mensajes sin leer y total
settings-folder-counts-unreadIfAny = Mensajes sin leer, solo si los hay
settings-folder-count-scope = Carpetas con recuentos
settings-folder-count-scope-all = Todas las carpetas
settings-folder-count-scope-main = Solo carpetas principales
settings-folder-count-scope-help = Las carpetas principales son Bandeja de entrada, Borradores, Enviados, Archivo, Correo no deseado, Papelera, Bandeja de salida y otras carpetas de uso especial. Las carpetas personalizadas y sus subcarpetas no muestran recuento.
settings-account-unread-badge = Mostrar el recuento de no leídos junto al nombre del buzón
settings-folder-counts-compact = Insignias más pequeñas fuera de la Bandeja de entrada
settings-focused-inbox = Bandeja Destacados
settings-focused-inbox-help = Dividir la Bandeja de entrada en las pestañas Destacados y Otros sobre la lista. Los mensajes de boletines y remitentes automáticos van a Otros. Las demás carpetas y los resultados de búsqueda muestran todos los mensajes.
settings-focused-senders-title = Gestionar remitentes de Destacados
settings-focused-senders-help = Remitentes que eligió mover siempre a Destacados o a Otros. Quite uno para que vuelva a decidir la clasificación automática.
settings-focused-senders-empty = Aún no hay remitentes. Use Mover siempre a Destacados o Mover siempre a Otros en un mensaje.
settings-focused-senders-to-focused = Siempre Destacados
settings-focused-senders-to-other = Siempre Otros
settings-focused-senders-remove = Quitar
settings-focused-senders-remove-label = Quitar { $sender }
settings-focused-senders-remove-failed = No se pudo quitar el remitente. Inténtelo de nuevo.
settings-folder-counts-reset = Heredar recuentos de carpetas
settings-folder-counts-reset-size = Heredar tamaño de insignia
settings-mail-resume-tabs = Reanudar mis pestañas abiertas
settings-mail-resume-tabs-help = Restaurar las referencias de pestañas de la cuenta al volver a abrir Mailwoman. Los cuerpos de los mensajes y las URL de adjuntos no se guardan en el registro del espacio de trabajo.
settings-row-actions-mode = Acciones rápidas de la lista de mensajes
settings-row-actions-mode-open = Siempre abiertas
settings-row-actions-mode-hover = Mostrar al pasar el cursor
settings-row-actions-mode-click = Mostrar al hacer clic
settings-row-actions-mode-off = Desactivado
settings-row-actions-mode-help = Cómo aparecen las acciones rápidas en cada fila de mensaje. Al pasar el cursor muestra tres controles y un botón Más; al hacer clic muestra un botón que los abre; en pantallas táctiles, pasar el cursor se comporta como un clic. Desactivado las quita de las filas (el menú contextual y la barra de herramientas siguen teniendo todas las acciones).
settings-mail-feature-showGoToTop = Mostrar un botón Ir arriba en la lista de mensajes
settings-mail-feature-showGoToTop-help = Añade un botón que desplaza la lista de mensajes hasta el primer mensaje. Desactivado por defecto.
settings-mail-feature-showCreateFolderButton = Mostrar un botón Crear carpeta junto a Redactar
settings-mail-feature-showCreateFolderButton-help = Añade en la barra lateral un botón solo con icono que crea una carpeta. Desactivado por defecto.
settings-draft-save = Borradores de la cuenta
settings-draft-save-help = Guardar en la carpeta Borradores de su cuenta para acceder desde otro dispositivo. El guardado es discreto; solo los fallos muestran una notificación. Las cuentas no compatibles y el contenido descifrado protegido nunca se suben de forma silenciosa.
settings-draft-save-mode = Guardar automáticamente
settings-draft-realtime = Tras una pausa al escribir
settings-draft-periodic = A intervalos regulares
settings-draft-manual = Solo cuando elijo Guardar
settings-draft-debounce = Pausa al escribir (milisegundos)
settings-draft-interval = Intervalo de guardado (segundos)
settings-permissions-push-other-registration = Este navegador tiene una suscripción que no está registrada en esta cuenta. Quítela antes de volver a registrarse.
settings-show-from = Mostrar De al redactar
settings-show-from-help = Empezar con el campo De visible. También puede mostrarlo para un solo mensaje desde la cinta. Solo se aceptan direcciones de envío autorizadas.
settings-sessions-table = Tabla de sesiones activas
settings-folders-table = Lista de carpetas
settings-kbd-reference = Ver asignaciones del preajuste
settings-kbd-reference-help = Solo referencia del preajuste. Las asignaciones personalizadas y la política del administrador pueden cambiar los atajos activos.
settings-sessions-client-details = Detalles del cliente
settings-2fa-passkey-rules = Su organización solo acepta claves de acceso que:
settings-2fa-passkey-rule-platform = estén almacenadas en este dispositivo (Windows Hello, Touch ID, bloqueo de pantalla de Android)
settings-2fa-passkey-rule-cross-platform = estén en una llave de seguridad o en un teléfono
settings-2fa-passkey-rule-user-verification = comprueben su PIN, huella digital o rostro
settings-2fa-passkey-rule-resident-key = estén almacenadas en el autenticador (detectables)
settings-2fa-passkey-gap-allowed = Añadida antes de la política actual y no la cumple. Sigue funcionando; añada una nueva clave de acceso y elimine esta.
settings-2fa-passkey-gap-refused = No cumple la política actual y ya no se acepta para iniciar sesión. Añada una nueva clave de acceso y elimine esta.
settings-2fa-passkey-browser-refused-policy = Su navegador no encontró ningún autenticador que cumpla la política de claves de acceso de su organización, o se canceló la solicitud.
settings-2fa-policy-user-verification = Su organización exige una clave de acceso que compruebe su PIN, huella digital o rostro. Este autenticador no le verificó.
settings-2fa-policy-platform = Su organización solo acepta claves de acceso almacenadas en este dispositivo (como Windows Hello, Touch ID o el bloqueo de pantalla de Android). No se aceptan llaves de seguridad ni teléfonos.
settings-2fa-policy-cross-platform = Su organización solo acepta llaves de seguridad o claves de acceso de teléfono. No se aceptan las claves de acceso almacenadas en este dispositivo.
settings-2fa-policy-attachment-unknown = Su navegador no indicó de qué tipo de autenticador se trata, por lo que no se puede comprobar con la política de claves de acceso de su organización. Pruebe con otro navegador.
settings-2fa-policy-resident-key = Su organización exige una clave de acceso almacenada en el autenticador. Este autenticador no almacenó ninguna.
settings-2fa-policy-legacy = Esta clave de acceso se añadió antes de la política actual de su organización y ya no la cumple. Inicie sesión con otro factor o un código de recuperación y luego añada una nueva clave de acceso.
settings-2fa-policy-last-factor-blocked = Su organización no permite quitar su último segundo factor. Añada primero otro método de inicio de sesión.
settings-2fa-policy-last-factor-required = Su organización exige autenticación en dos pasos. Añada otro método de inicio de sesión antes de quitar este.
settings-2fa-last-factor-hint = Añada primero otro método de inicio de sesión.
settings-2fa-last-factor-tooltip = Añada primero otro método de inicio de sesión. Su organización exige al menos una clave de acceso o una aplicación de autenticación en su cuenta.
settings-2fa-challenge-passkey-not-used = No se usó la clave de acceso: la solicitud se canceló o caducó, o este autenticador no puede hacer lo que exige su organización (como una comprobación de PIN, huella digital o rostro). Inténtelo de nuevo o use otro método.
settings-folders-hidden-help = Las carpetas ocultas y sus subcarpetas quedan fuera de la barra lateral. No se eliminan ni se restringe su acceso. Mostrar aquí una subcarpeta también muestra sus carpetas superiores ocultas. La Bandeja de entrada siempre está disponible.
settings-folders-collapse-user = Contraer las carpetas del usuario en la barra lateral
settings-folders-show = Mostrar { $name } en la barra lateral
settings-folders-hide = Ocultar { $name } de la barra lateral
settings-folders-show-short = Mostrar
settings-folders-hide-short = Ocultar
settings-folders-search = Buscar carpetas
settings-folders-select = Seleccionar carpeta: { $name }
settings-folders-selected = Carpetas seleccionadas: { $count }
settings-folders-select-visible = Seleccionar visibles
settings-folders-select-all = Seleccionar todas las carpetas
settings-folders-clear = Borrar selección
settings-folders-pin-selected = Fijar seleccionadas
settings-folders-unpin-selected = Desfijar seleccionadas
settings-folders-show-selected = Mostrar seleccionadas
settings-folders-hide-selected = Ocultar seleccionadas
settings-folders-visible = Mostradas
settings-folders-hidden = Ocultas
settings-folders-pinned = Fijadas
settings-folders-location = Ubicación
settings-folders-counts = { $unread } sin leer / { $total } en total
settings-folders-more = Más acciones para { $name }
settings-folders-empty = Ninguna carpeta coincide con su búsqueda.
settings-folders-root = Nivel superior
settings-folders-order-help = Arrastre las carpetas para reordenarlas en la barra lateral. Las carpetas fijadas permanecen juntas.
cache-title = Caché de correo y precarga
cache-help = Mantener las carpetas y mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Al cerrar la pestaña o cerrar sesión se borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
cache-enabled = Activar la caché de sesión
cache-mailboxes = Guardar en caché las listas de mensajes de las carpetas
cache-messages = Guardar en caché los mensajes abiertos
cache-freshSeconds = Intervalo de frescura (segundos)
cache-maxAgeSeconds = Antigüedad máxima en caché (segundos)
cache-maxEntries = Máximo de entradas en caché
cache-maxBytes = Presupuesto de memoria (bytes)
cache-prefetchEnabled = Activar la precarga especulativa
cache-hoverEnabled = Precargar al apuntar a una carpeta
cache-focusEnabled = Precargar cuando una carpeta recibe el foco del teclado
cache-dwellMs = Retardo del puntero y del foco (milisegundos)
cache-idleEnabled = Precargar carpetas en reposo
cache-idleDelayMs = Retardo de reposo (milisegundos)
cache-maxConcurrency = Máximo de precargas simultáneas
cache-maxQueued = Máximo de precargas en cola
cache-preloadMessages = Permitir cuerpos de mensaje especulativos
cache-excludedFolders = Carpetas excluidas de la precarga
cache-exclusions-help = Una regla por línea: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive, o id: seguido de un ID de carpeta exacto.
cache-refreshOnOpen = Actualizar el contenido en caché al abrir
cache-clear = Borrar ahora el correo en caché
settings-compose-window = Abrir mensajes nuevos en
settings-compose-window-tab = Una pestaña del espacio de trabajo
settings-compose-window-new = Una ventana nueva
settings-compose-window-help = Si su navegador bloquea una ventana nueva, el editor se abre en una pestaña del espacio de trabajo.

# -- Machine translation (2026-11): added to match en ---------------------
settings-compose-read-receipt = Solicitar siempre confirmaciones de lectura
settings-compose-delivery-receipt = Solicitar siempre confirmaciones de entrega
settings-compose-reply-copy = Guardar una copia de las respuestas en la carpeta original
settings-compose-receipts-help = Las confirmaciones son solicitudes. El destinatario o su servidor de correo pueden rechazarlas o ignorarlas.
settings-compose-forward = Formato de reenvío predeterminado
settings-compose-forward-inline = Texto del mensaje en el cuerpo
settings-compose-forward-attachment = Mensaje original como archivo adjunto .eml
settings-compose-saving = Guardando la preferencia de redacción…
settings-compose-saved = Preferencia de redacción guardada
settings-ident-organization = Organización
settings-ident-auto-bcc = Cco automático
settings-ident-auto-bcc-help = Añada hasta 20 direcciones en Cco al enviar desde esta identidad. Separe las direcciones con comas.
settings-ident-pgp-key = Clave OpenPGP
settings-ident-pgp-none = Usar la clave predeterminada
settings-ident-pgp-unavailable = La clave guardada no está disponible: elija otra clave
settings-ident-pgp-help = Elija una de sus claves de firma OpenPGP existentes. El material de la clave privada permanece en su almacén de claves.
settings-ident-error-optional = Compruebe la dirección de respuesta, las direcciones Cco automáticas, la organización y la clave seleccionada.
settings-compose-open-tracking = Registrar la apertura de mensajes de forma predeterminada
settings-compose-open-tracking-help = Añade una imagen de seguimiento a los mensajes con formato que cumplan los requisitos. Las aperturas dependen de que el destinatario cargue las imágenes y no prueban que el mensaje se haya leído. Su administrador puede desactivar el seguimiento o bloquear este valor predeterminado.
settings-compose-open-tracking-inherit = Usar el valor de seguimiento heredado
settings-reply-images = Imágenes en respuestas y reenvíos
settings-reply-images-ask = Preguntar cada vez
settings-reply-images-allow = Incluir siempre
settings-reply-images-trusted = Incluir para dominios de remitente de confianza; preguntar en los demás casos
settings-reply-images-deny = No incluir nunca
settings-reply-images-help = Elija si una respuesta o un reenvío incluye las imágenes del mensaje original. Las imágenes incluidas se copian en el borrador como archivos adjuntos.
settings-reply-image-domains = Dominios de remitente de confianza para imágenes
settings-reply-image-domains-help = Introduzca hasta 200 dominios de remitente exactos, uno por línea. Se usa cuando la opción anterior es dominios de confianza.
settings-reply-image-domains-save = Guardar dominios de imágenes de confianza
settings-ident-group-basics = Datos básicos
settings-ident-group-replies = Gestión de respuestas
settings-ident-group-signature = Firma
settings-ident-group-sending = Opciones de envío
settings-ident-name-required = Introduzca un nombre para mostrar.
settings-ident-email-invalid = Introduzca una dirección de correo electrónico válida, por ejemplo nombre@example.org.
settings-ident-email-duplicate = Ya tiene una identidad con esta dirección.
settings-ident-replyto-invalid = Introduzca una dirección de correo electrónico válida o deje el campo vacío.
settings-ident-replyto-help = Las respuestas a los mensajes enviados desde esta identidad van a esta dirección en lugar de a la del remitente.
settings-ident-bcc-invalid = Compruebe la lista Cco: use direcciones válidas separadas por comas, como máximo 20.
settings-ident-delete-title = ¿Eliminar la identidad { $email }?
settings-ident-delete-body = Ya no podrá enviar mensajes desde esta identidad. Los borradores y los mensajes enviados no se ven afectados.
settings-ident-empty = Aún no tiene identidades. Añada una para elegir su nombre, dirección y firma al redactar.
settings-sig-delete-title = ¿Eliminar la firma { $name }?
settings-sig-delete-body = Las identidades que la usan volverán a la firma de la cuenta. Los mensajes ya enviados no se ven afectados.
settings-sig-deleted = Firma eliminada.
settings-reply-from = Responder desde
settings-reply-from-match = La dirección a la que se envió el mensaje
settings-reply-from-default = Siempre mi identidad predeterminada
settings-reply-from-ask = Preguntarme cuando coincidan varias identidades
settings-reply-from-keep = Mantener la identidad ya seleccionada
settings-reply-from-help = Elija qué identidad se usa al responder o reenviar. Si ninguna identidad coincide con la dirección a la que se envió el mensaje, se usa su identidad predeterminada. Las direcciones con signo más, como nombre+etiqueta@example.org, coinciden con nombre@example.org.
settings-ident-default-badge = Predeterminada
settings-ident-make-default = Establecer como predeterminada
settings-ident-default-set = { $email } es ahora su identidad predeterminada.
settings-ident-create-variant = Crear variante
settings-ident-variant-count = { $count } { $count ->
        [one] variante
       *[other] variantes
    }
settings-ident-inherits = Usa { $base } para: { $fields }
settings-ident-field-replyTo = dirección de respuesta
settings-ident-field-autoBcc = Cco automático
settings-ident-field-organization = organización
settings-ident-field-pgpKeyId = clave OpenPGP
settings-ident-field-signatureName = firma
settings-ident-delete-default-body = Esta es su identidad predeterminada. La identidad más antigua que quede pasará a ser la predeterminada. Ya no podrá enviar mensajes desde esta identidad. Los borradores y los mensajes enviados no se ven afectados.
settings-ident-variants-choice-title = ¿Eliminar { $email }?
settings-ident-variants-choice-body = Esta identidad tiene { $count } { $count ->
        [one] variante
       *[other] variantes
    }. Elimínela { $count ->
        [one] también
       *[other] también
    } o consérvela { $count ->
        [one] como identidad independiente
       *[other] como identidades independientes
    }. Una variante conservada copia los ajustes que heredaba de esta identidad.
settings-ident-delete-with-variants = Eliminar identidad y variantes
settings-ident-delete-keep-variants = Eliminar identidad, conservar variantes
settings-ident-new-variant = Nueva variante
settings-ident-edit-variant-title = Editar variante
settings-ident-variant-intro = Una variante de { $base }. Deje un campo vacío para usar el valor de esa identidad.
settings-ident-label-label = Etiqueta de la variante
settings-ident-label-required = Introduzca una etiqueta para esta variante, por ejemplo Boletines.
settings-ident-inherit-hint = Vacío: usa { $value } de la identidad base.
settings-ident-inherit-none = nada
settings-check-policy-allow = Enviar sin preguntar
settings-check-policy-warn = Preguntar cada vez
settings-check-policy-warn-remember = Preguntar, con opción de no volver a preguntar
settings-check-policy-deny = Bloquear el envío
settings-check-invalid = Direcciones de destinatarios no válidas
settings-check-attachment = Archivo adjunto mencionado pero ausente
settings-check-subject = Asunto vacío
settings-check-help = Qué hacer cuando la comprobación detecta un problema. Al preguntar, siempre se ofrece Enviar de todos modos; solo Bloquear el envío detiene un mensaje.
settings-external-warn-remember = Preguntar, con opción de no volver a preguntar
settings-external-deny = Bloquear el envío
settings-auto-chip = Finalizar un destinatario al escribir un espacio, una coma o un punto y coma
settings-auto-chip-help = Una dirección válida se convierte en un destinatario finalizado y el separador se añade automáticamente.
settings-recipient-separator = Separador de destinatarios
settings-recipient-separator-comma = Coma
settings-recipient-separator-semicolon = Punto y coma
settings-header-collapse = Encabezado del mensaje (Para, Cc, Asunto)
settings-header-collapse-expanded = Mostrar siempre
settings-header-collapse-collapsed = Contraer en una línea de resumen
settings-header-collapse-auto = Contraer mientras escribo el cuerpo
settings-header-collapse-help = Contraído, el encabezado muestra una línea como Para: Ana +2 y se abre al hacer clic en él o cuando un campo requiere atención.
settings-retention-title = Conservación
settings-folder-icon-camera = Cámara
settings-folder-icon-music = Música
settings-folder-icon-film = Película
settings-folder-icon-lightbulb = Idea
settings-folder-icon-rocket = Cohete
settings-folder-icon-trophy = Trofeo
settings-folder-icon-leaf = Hoja
settings-folder-icon-globe = Globo terráqueo
settings-folder-icon-cloud = Nube
settings-folder-icon-bookmark = Marcador
settings-folder-icon-wallet = Cartera
settings-folder-icon-medical = Médico
settings-folder-icon-bell = Campana
settings-folder-icon-key = Llave
settings-folder-icon-receipt = Recibo
settings-folder-icon-cart = Carrito de la compra
settings-folder-icon-package = Paquete
settings-folder-icon-wrench = Herramientas
settings-folder-icon-chart = Gráfico
settings-folder-icon-team = Equipo
settings-folder-icon-store = Tienda
settings-folder-icon-office = Oficina
settings-folder-icon-laptop = Portátil
settings-folder-icon-crown = Corona
settings-folder-icon-family = Familia
settings-folder-icon-group-general = General
settings-folder-icon-group-mail = Correo
settings-folder-icon-group-work = Trabajo
settings-folder-icon-group-finance = Finanzas y compras
settings-folder-icon-group-travel = Viajes y hogar
settings-folder-icon-group-personal = Personal y ocio
settings-folder-icon-group-tech = Tecnología y herramientas
settings-folder-icon-search-label = Buscar iconos
settings-folder-icon-no-match = Ningún icono coincide

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Autenticación (DMARC)

settings-badge-auth-help = El resultado DMARC; abre todos los detalles de seguridad.

settings-badge-density = Tamaño de la insignia

settings-badge-density-comfortable = Cómodo

settings-badge-density-compact = Compacto

settings-badge-density-help = Altura y espacio compactos o cómodos para cada credencial.

settings-badge-folder = Carpeta

settings-badge-folder-help = Donde se almacena el mensaje. Sólo cuando es notable lo muestra como basura y basura.

settings-badge-labels = Etiquetas de insignia

settings-badge-labels-help = Icono con texto, o el icono solo (el texto completo permanece en la información sobre herramientas y para los lectores de pantalla).

settings-badge-labels-icon = Solo icono

settings-badge-labels-text = Icono y texto

settings-badge-list = lista de correo

settings-badge-list-help = La lista de correo de la que procede el mensaje; abre los detalles y acciones de la suscripción. Solo cuando es notable lo muestra cuando el correo sigue llegando después de cancelar la suscripción.

settings-badge-mode-always = siempre

settings-badge-mode-never = nunca

settings-badge-mode-notable = Sólo cuando es notable

settings-badge-sanitized = Desinfectado HTML

settings-badge-sanitized-help = Qué le hizo el desinfectante HTML al mensaje. Solo cuando es notable lo muestra cuando algo fue bloqueado.

settings-badge-spam = Puntuación de spam

settings-badge-spam-help = La puntuación que el servidor receptor escribió en los encabezados del mensaje.

settings-folders-hide-empty = Ocultar carpetas vacías en la barra lateral

settings-folders-hide-empty-help = Las carpetas reaparecen cuando contienen mensajes. La bandeja de entrada, las carpetas de búsqueda, las carpetas con recuentos desconocidos y las carpetas principales de las carpetas visibles permanecen mostradas. Anule carpetas individuales en Propiedades, Pantalla; todas las carpetas permanecen en esta lista.

settings-folders-list-card = Lista de carpetas y orden

settings-header-auto-collapse = Doblar el encabezado después de escribir (segundos)

settings-header-auto-collapse-help = Cuando el cuerpo ha estado enfocado durante tantos segundos y el encabezado tiene un destinatario y un asunto, se pliega a la línea de resumen. 0 desactiva esto.

settings-message-badges = Insignias de mensaje

settings-message-badges-help = Elija qué insignias aparecen en el encabezado de un mensaje abierto. Las marcas de prioridad se establecen en Carpetas y barra lateral. También puede hacer clic derecho en una insignia para ocultarla.

settings-reply-from-headers = Cabeceras que eligen la identidad

settings-reply-from-headers-help = Un nombre de encabezado por línea, probado en este orden: Entregado a, X-Original-a, Sobre-a, X-Reenviado-a, Reenviado-a, A, Cc. Un encabezado omitido no se lee.

settings-reply-from-headers-invalid = Utilice solo estos nombres, uno por línea: Entregado a, X-Original-a, Sobre-a, X-Reenviado-a, Reenviado-a, A, Cc.

settings-reply-from-headers-save = Guardar orden de encabezado

settings-sessions-linked-created = Enlace creado

settings-sessions-linked-details = Detalles del enlace

settings-sessions-linked-kind = Tipo de enlace

settings-sessions-linked-kind-linked = Buzón vinculado

settings-sessions-linked-last-used = último usado

settings-sessions-linked-line = Acceso al buzón vinculado · desde { $origin }

settings-sessions-linked-line-hidden = Acceso al buzón vinculado

settings-sessions-linked-origin = viene de

settings-sessions-linked-removed = Desconocido (cuenta eliminada)

settings-sessions-linked-revoke = Revocar el acceso al enlace

settings-sessions-linked-revoke-label = Revocar el acceso al enlace para { $mailbox }

settings-sessions-linked-target = Buzón abierto

settings-sessions-origin-full = dirección completa

settings-sessions-origin-help = Controla cómo esta tabla nombra la cuenta de la que proviene una sesión de buzón vinculado. Sólo tú lo ves, en tus propias sesiones.

settings-sessions-origin-hidden = Oculto

settings-sessions-origin-masked-local-part = Dirección con la primera parte enmascarada.

settings-sessions-origin-setting = Mostrar de dónde proviene el acceso al buzón vinculado

settings-shortcuts-search = Buscar atajos

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = Mantenga las carpetas y los mensajes visitados recientemente listos para abrir. De forma predeterminada, las copias permanecen en la memoria de esta pestaña. Habilite el almacenamiento indefinido para mantener copias cifradas durante las recargas. La precarga nunca marca el correo leído ni carga imágenes remotas.

cache-keepIndefinitely = Mantener el correo en caché indefinidamente

cache-keepIndefinitely-help = Desactivado de forma predeterminada. Mantenga copias cifradas en este navegador durante las recargas sin vencimiento. Los mensajes en línea aún se actualizan. Se siguen aplicando los límites de entrada y bytes. Desactivar esto o borrar el correo en caché elimina las copias guardadas. La limpieza de datos del navegador también puede eliminarlos. Los archivos adjuntos no se descargan para su uso sin conexión.
