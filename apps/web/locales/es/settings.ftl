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
