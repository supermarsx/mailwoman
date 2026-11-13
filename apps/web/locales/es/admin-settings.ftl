# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Permitir el envío
admin-settings-help-mail-allow-send = Permite a los usuarios enviar mensajes. Si está desactivado, el editor deshabilita Enviar y su atajo y el servidor rechaza los envíos; los borradores se pueden seguir guardando.
admin-settings-key-mail-allow-reply = Permitir responder
admin-settings-help-mail-allow-reply = Permite a los usuarios responder y responder a todos desde el lector y con atajos. Si está desactivado, esas acciones no se ofrecen.
admin-settings-key-mail-allow-forward = Permitir reenviar
admin-settings-help-mail-allow-forward = Permite a los usuarios reenviar mensajes en línea o como adjunto. Si está desactivado, esas acciones no se ofrecen.
admin-settings-key-mail-allow-download = Permitir descargas
admin-settings-help-mail-allow-download = Permite a los usuarios descargar mensajes y archivos adjuntos. Si está desactivado, la aplicación web no ofrece descargas y el servidor rechaza la exportación de mensajes.
admin-settings-key-mail-allow-print = Permitir imprimir
admin-settings-help-mail-allow-print = Permite a los usuarios imprimir un mensaje desde el lector. Si está desactivado, la acción Imprimir no se ofrece.
admin-settings-key-mail-show-spam = Mostrar la carpeta Spam
admin-settings-help-mail-show-spam = Muestra la carpeta Spam o Junk y los mensajes marcados como spam. Si está desactivado, se ocultan de la lista de carpetas y de los resultados de búsqueda; no se elimina nada.
admin-settings-key-mail-show-auth-status = Mostrar los resultados de autenticación del remitente
admin-settings-help-mail-show-auth-status = Muestra en el lector los resultados SPF, DKIM y DMARC de la cabecera Authentication-Results.
admin-settings-key-mail-close-on-delete = Cerrar un mensaje al eliminarlo
admin-settings-help-mail-close-on-delete = Cierra el mensaje, la pestaña o la ventana independiente abiertos cuando ese mensaje se elimina o se mueve a la Papelera. Si está desactivado, el lector mantiene su comportamiento anterior.
admin-settings-key-links-trusted-senders = Remitentes de enlaces de confianza
admin-settings-help-links-trusted-senders = Buzones de remitentes exactos cuyos enlaces de mensajes revisados pueden abrirse sin consentimiento por mensaje. Una dirección por entrada.
admin-settings-key-appearance-font-size = Tamaño de fuente de la interfaz
admin-settings-help-appearance-font-size = Tamaño de fuente de la interfaz en píxeles, de 12 a 24.
admin-settings-key-appearance-font-weight = Grosor de fuente de la interfaz
admin-settings-help-appearance-font-weight = Grosor base del texto de la interfaz: 300, 400, 500 o 600. El texto destacado conserva su jerarquía.
admin-settings-key-appearance-font-color = Color del texto de la interfaz
admin-settings-help-appearance-font-color = Color del texto de la interfaz como #RRGGBB. Déjelo vacío para seguir el tema.
admin-settings-key-appearance-low-contrast = Suavizar el contraste del texto
admin-settings-help-appearance-low-contrast = Suaviza el contraste del texto de la interfaz.
admin-settings-key-mail-message-list-order = Orden de la lista de mensajes
admin-settings-help-mail-message-list-order = Orden de la lista de mensajes: más recientes, más antiguos, remitente o asunto.
admin-settings-key-mail-message-list-style = Estilo de la lista de mensajes
admin-settings-help-mail-message-list-style = Mostrar la lista de mensajes como tarjetas o como filas con encabezados de columna.
admin-settings-key-mail-message-list-preview = Vistas previas de la lista de mensajes
admin-settings-help-mail-message-list-preview = Mostrar fragmentos del cuerpo en la lista de mensajes.
admin-settings-key-mail-after-delete = Después de eliminar un mensaje
admin-settings-help-mail-after-delete = Qué mensaje muestra el lector tras eliminar el mensaje abierto: el siguiente, el anterior o ninguno.
admin-settings-key-reader-attachments-at-end = Archivos adjuntos después del cuerpo
admin-settings-help-reader-attachments-at-end = Mostrar los archivos adjuntos después del cuerpo del mensaje en lugar de antes.
admin-settings-key-notifications-ask-permission = Ofrecer el permiso de notificaciones
admin-settings-help-notifications-ask-permission = Ofrecer el aviso de permiso de notificaciones del navegador a los usuarios en dispositivos que no han permitido ni bloqueado las notificaciones.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Ajustes
admin-settings-description = Valores para todo el servidor, por dominio y por usuario. Un valor bloqueado no puede ser cambiado por los usuarios ni por ámbitos más específicos.
admin-settings-list-caption = Ajustes
admin-settings-search = Buscar ajustes
admin-settings-empty = Ningún ajuste coincide con la búsqueda.
admin-settings-unavailable = Este servidor aún no ofrece el modelo de ajustes.
admin-settings-col-setting = Ajuste
admin-settings-col-value = Valor
admin-settings-col-source = Origen
admin-settings-col-overrides = Valores más específicos
admin-settings-open = Abrir
admin-settings-overrides-none = Ninguno
admin-settings-overrides = { $domains } dominios, { $users } usuarios
admin-settings-not-found = No existe ningún ajuste llamado { $key }.
admin-settings-value-on = Activado
admin-settings-value-off = Desactivado
admin-settings-value-empty = Sin definir
admin-settings-value-list = { $count ->
    [one] { $count } elemento
   *[other] { $count } elementos
}
admin-settings-detail-key = Clave: { $key }
admin-settings-global = Para todo el servidor
admin-settings-global-help = Este valor se aplica a todos los usuarios, salvo que un dominio o un usuario tenga un valor más específico, o que el usuario elija el suyo donde se permita.
admin-settings-value-label = Valor
admin-settings-json-hint = JSON. El servidor comprueba el valor e indica la parte que rechaza.
admin-settings-json-invalid = Este JSON no es válido.
admin-settings-user-override = Los usuarios pueden cambiarlo
admin-settings-user-override-help = Si está activado, los usuarios pueden elegir su propio valor en sus ajustes. Si está desactivado, se les aplica el valor definido aquí o para su dominio.
admin-settings-lock-help = Si está bloqueado, se ignoran los valores de dominio y las elecciones de los usuarios, y este valor se aplica a todos.
admin-settings-save = Guardar
admin-settings-saved = Guardado. Se aplica en un plazo de 30 segundos en cada proceso del servidor.
admin-settings-saved-restart = Guardado. Se aplica tras un reinicio.
admin-settings-dedicated = Este ajuste se cambia en { $section }, que comprueba primero su efecto.
admin-settings-dedicated-link = Abrir { $section }
admin-settings-pinned = Definido por la variable de entorno { $name }. El valor guardado aquí se aplica cuando la variable no está definida.
admin-settings-lock-title = ¿Bloquear { $setting }?
admin-settings-lock-body = Al bloquear se ignoran los valores más específicos que se indican abajo. Se conservan y vuelven a aplicarse al desbloquear.
admin-settings-lock-confirm = Bloquear
admin-settings-impact-domain = { $count ->
    [one] Se ignora { $count } valor de dominio
   *[other] Se ignoran { $count } valores de dominio
}
admin-settings-impact-user = { $count ->
    [one] Se ignora { $count } elección propia de un usuario
   *[other] Se ignoran { $count } elecciones propias de usuarios
}
admin-settings-impact-admin-user = { $count ->
    [one] Se ignora { $count } valor por usuario
   *[other] Se ignoran { $count } valores por usuario
}
admin-settings-impact-profile = { $count ->
    [one] Se ignora { $count } valor de perfil de funciones
   *[other] Se ignoran { $count } valores de perfiles de funciones
}
admin-settings-impact-group = { $count ->
    [one] Se ignora { $count } valor de grupo
   *[other] Se ignoran { $count } valores de grupo
}
admin-settings-domains = Valores de dominio
admin-settings-domains-help = Un valor de dominio se aplica a los usuarios de ese dominio en lugar del valor de todo el servidor.
admin-settings-domains-caption = Valores de dominio de { $setting }
admin-settings-domains-empty = Ningún dominio tiene un valor propio.
admin-settings-domain = Dominio
admin-settings-domain-value = Valor para este dominio
admin-settings-domain-locked = Bloqueado para los usuarios del dominio
admin-settings-domain-add = Añadir un valor de dominio
admin-settings-domain-save = Guardar valor de dominio
admin-settings-domain-remove = Quitar
admin-settings-domain-remove-title = ¿Quitar el valor de { $domain }?
admin-settings-domain-remove-body = Los usuarios de { $domain } vuelven a tener el valor de todo el servidor.
admin-settings-domain-required = Introduzca un nombre de dominio.
admin-settings-locked-yes = Bloqueado
admin-settings-locked-no = No bloqueado
admin-settings-users = Valores por usuario
admin-settings-users-help = Un valor que un administrador definió para un usuario. En los límites, el usuario se designa por su dirección.
admin-settings-users-caption = Valores por usuario de { $setting }
admin-settings-users-empty = Ningún usuario tiene un valor definido por un administrador.
admin-settings-user = Usuario
admin-settings-user-hint = ID de cuenta o, en los límites, la dirección.
admin-settings-user-value = Valor para este usuario
admin-settings-user-add = Añadir un valor por usuario
admin-settings-user-save = Guardar valor del usuario
admin-settings-user-required = Introduzca un usuario.
admin-settings-user-remove-title = ¿Quitar el valor de { $user }?
admin-settings-user-remove-body = El usuario recibe el valor de su dominio o el de todo el servidor.
admin-settings-source-default = Valor predeterminado integrado
admin-settings-source-envFallback = Variable de entorno (sin valor guardado)
admin-settings-source-envPin = Variable de entorno
admin-settings-source-global = Guardado para todo el servidor
admin-settings-cat-interface = Interfaz
admin-settings-cat-appearance = Apariencia
admin-settings-cat-mail = Correo
admin-settings-cat-folders = Disposición de carpetas
admin-settings-cat-notifications = Notificaciones
admin-settings-cat-offline = Uso sin conexión
admin-settings-cat-assistant = Asistente de IA
admin-settings-cat-upstream = Servidor de correo
admin-settings-cat-signin = Inicio de sesión
admin-settings-cat-admin = Consola de administración
admin-settings-cat-passwords = Contraseñas
admin-settings-cat-limits = Límites y uso
admin-settings-cat-plugins = Complementos
admin-settings-cat-features = Permisos de funciones
admin-settings-cat-integrations = Permisos de integraciones
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Permite los archivos de Nextcloud vinculados a la cuenta, el uso compartido y las operaciones de calendario, contactos y tareas.
admin-settings-key-cap-integration-google = Puente de Google
admin-settings-help-cap-integration-google = Permite las operaciones del puente de Google, incluida la obtención y renovación de tokens.
admin-settings-key-cap-integration-microsoft = Puente de Microsoft
admin-settings-help-cap-integration-microsoft = Permite las operaciones del puente de Microsoft, incluida la obtención y renovación de tokens.
admin-settings-key-cap-integration-family = Integración individual
admin-settings-help-cap-integration-family = Permiso para el identificador de una integración individual; la cobertura en tiempo de ejecución figura en el catálogo de funciones.
admin-settings-key-cap-ui-plugin-family = Complemento de interfaz individual
admin-settings-help-cap-ui-plugin-family = Permiso para un complemento de interfaz. Use la clave de política exacta del catálogo de integraciones.
admin-settings-key-cap-ai-assistant = Asistente de IA
admin-settings-help-cap-ai-assistant = Permite la configuración e invocación del asistente.
admin-settings-key-cap-plugins = Complementos
admin-settings-help-cap-plugins = Permite las operaciones de complementos de interfaz y de motor.
admin-settings-key-cap-pim-calendar = Calendario
admin-settings-help-cap-pim-calendar = Permite los métodos de calendario y las suscripciones.
admin-settings-key-cap-pim-contacts = Contactos
admin-settings-help-cap-pim-contacts = Permite los métodos de contactos y libretas de direcciones.
admin-settings-key-cap-pim-tasks = Tareas
admin-settings-help-cap-pim-tasks = Permite los métodos de tareas.
admin-settings-key-cap-pim-notes = Notas
admin-settings-help-cap-pim-notes = Permite los métodos de notas.
admin-settings-key-cap-export = Exportación
admin-settings-help-cap-export = Permite las exportaciones de correo y de datos PIM.
admin-settings-key-cap-import = Importación
admin-settings-help-cap-import = Permite las importaciones de correo y de datos PIM.
admin-settings-key-cap-api-keys = Claves de API
admin-settings-help-cap-api-keys = Política de claves de API. Consulte el catálogo de funciones para ver la cobertura de aplicación.
admin-settings-key-cap-oauth-apps = Aplicaciones OAuth
admin-settings-help-cap-oauth-apps = Política de aplicaciones OAuth. Consulte el catálogo de funciones para ver la cobertura de aplicación.
admin-settings-key-cap-rules = Reglas de correo
admin-settings-help-cap-rules = Permite los métodos gestionados de reglas de correo y de respuesta de vacaciones.
admin-settings-key-cap-rules-forward = Reenvío por reglas
admin-settings-help-cap-rules-forward = Política de reenvío. Consulte el catálogo de funciones para ver la cobertura de aplicación.
admin-settings-key-cap-remote-images = Imágenes remotas
admin-settings-help-cap-remote-images = Permiso de imágenes remotas. Los ajustes de privacidad existentes siguen rigiendo las solicitudes; compruebe la cobertura de la función.
admin-settings-key-cap-user-background = Fondo personalizado
admin-settings-help-cap-user-background = Permiso de fondo personalizado, desactivado por defecto. Compruebe la cobertura de la función.
admin-settings-key-cap-user-css = CSS personalizado
admin-settings-help-cap-user-css = Permiso de CSS personalizado, desactivado por defecto. Compruebe la cobertura de la función.
admin-settings-key-limit-attachment-bytes = Límite de tamaño de adjuntos
admin-settings-help-limit-attachment-bytes = Política de adjuntos en bytes, 50000000 por defecto. La aplicación en tiempo de ejecución sigue pendiente.
admin-settings-key-limit-send-per-hour = Límite de envío por hora
admin-settings-help-limit-send-per-hour = Política de envíos por hora; cero significa sin límite. La aplicación en tiempo de ejecución sigue pendiente.
admin-settings-key-limit-recipients-per-message = Destinatarios por mensaje
admin-settings-help-limit-recipients-per-message = Política del número de destinatarios; cero significa sin límite. La aplicación en tiempo de ejecución sigue pendiente.
admin-settings-cat-import = Importación
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Actualizaciones en vivo
admin-settings-cat-privacy = Privacidad
admin-settings-cat-observability = Observabilidad
admin-settings-cat-egress = Conexiones salientes
admin-settings-cat-updates = Actualizaciones
admin-settings-cat-backups = Copias de seguridad
admin-settings-cat-about = Acerca de
admin-settings-cat-config = Configuración
admin-settings-cat-jobs = Tareas
admin-settings-cat-maintenance = Mantenimiento
admin-settings-cat-health = Comprobaciones de estado
admin-settings-cat-diagnostics = Diagnóstico
admin-settings-cat-users = Usuarios
admin-settings-cat-server-text = Texto del servidor
admin-settings-key-ui-language = Idioma
admin-settings-help-ui-language = El idioma del cliente de correo para los usuarios que no han elegido uno. «Igual que el navegador» sigue los idiomas de cada navegador.
admin-settings-key-ui-notify-new-version = Aviso de nueva versión
admin-settings-help-ui-notify-new-version = Activado (por defecto) muestra a los usuarios con sesión iniciada un aviso con «Recargar ahora» y «Más tarde» cuando se ha desplegado una versión más reciente de la aplicación web desde que se cargó su página. Desactivado nunca lo muestra; los usuarios reciben entonces el código nuevo en su próxima recarga.
admin-settings-key-ui-undo-history-size = Tamaño del historial de deshacer y rehacer
admin-settings-help-ui-undo-history-size = Cuántas acciones de correo recientes (archivar, eliminar, mover, marcar como leído, marcar con bandera, etiquetar y similares) recuerdan los botones Deshacer y Rehacer, de 0 a 50. 0 desactiva los botones y el atajo Ctrl+Z. El valor predeterminado es 50. Los usuarios pueden elegir su propio tamaño, nunca por encima del máximo indicado abajo, salvo que un administrador lo bloquee.
admin-settings-key-folders-reorder-enabled = Arrastrar y soltar carpetas
admin-settings-help-folders-reorder-enabled = Permite reorganizar las carpetas arrastrándolas en la barra lateral y en los ajustes. Desactivado oculta los tiradores de arrastre y los comandos Mover. Los usuarios pueden desactivarlo para sí mismos salvo que esté bloqueado.
admin-settings-key-folders-reorder-allow-nesting = Anidamiento de carpetas
admin-settings-help-folders-reorder-allow-nesting = Permite soltar una carpeta sobre otra para anidarla, o devolverla al nivel superior. Desactivado mantiene cada carpeta bajo su padre actual y solo permite reordenar entre carpetas hermanas.
admin-settings-key-folders-reorder-allow-server-move = Mover carpetas en el servidor
admin-settings-help-folders-reorder-allow-server-move = Permite que anidar o desanidar cambie el padre real de la carpeta en el servidor de correo, de modo que el cambio vale en todas partes. Desactivado mantiene esos movimientos solo como una organización en esta aplicación.
admin-settings-key-folders-reorder-allow-system-folders = Mover carpetas del sistema
admin-settings-help-folders-reorder-allow-system-folders = Permite arrastrar Bandeja de entrada, Borradores, Enviados, Correo no deseado, Papelera y Archivo para reordenarlas. Desactivado las mantiene fijas arriba. Solo se reordenan en esta aplicación; nunca se renombran ni se mueven en el servidor.
admin-settings-key-folders-reorder-max-depth = Profundidad de anidamiento de carpetas
admin-settings-help-folders-reorder-max-depth = El nivel de anidamiento más profundo que puede crear un arrastre, de 1 a 16. Se rechaza un movimiento que dejaría una carpeta o una de sus subcarpetas a mayor profundidad.
admin-settings-key-folders-order-locked = Fijar el orden de las carpetas
admin-settings-help-folders-order-locked = Fija el orden de las carpetas para todos los del ámbito. El arrastre y los comandos Mover quedan desactivados, y los usuarios ven un aviso de bloqueo. Las fijaciones y el orden definido por el administrador se mantienen.
admin-settings-key-folders-pinned-admin = Carpetas fijadas a la fuerza
admin-settings-help-folders-pinned-admin = Carpetas que se muestran en un grupo Fijadas en la parte superior de la barra lateral de cada usuario. Una entrada por línea, como role:inbox o una ruta de carpeta como Shared/Support. Los usuarios no pueden desfijarlas ni moverlas. Se omite una entrada sin carpeta coincidente.
admin-settings-key-folders-order-default = Orden predeterminado de las carpetas
admin-settings-help-folders-order-default = El orden con el que empiezan las carpetas para los usuarios que no las han organizado, de la primera a la última. Una entrada por línea, como role:<role> o una ruta de carpeta. Las carpetas no listadas siguen en el orden del servidor. El orden propio de un usuario sustituye a este.
admin-settings-key-folders-new-folder-placement = Ubicación de las carpetas nuevas
admin-settings-help-folders-new-folder-placement = Dónde se coloca entre sus hermanas una carpeta que aparece en el servidor: al final o en orden alfabético.
admin-settings-key-ui-undo-history-max = Máximo del historial de deshacer y rehacer
admin-settings-help-ui-undo-history-max = El mayor número de acciones que cualquier usuario puede conservar en el historial de deshacer y rehacer, de 0 a 50. El tamaño propio de un usuario se reduce a este valor. 0 desactiva el historial para todos. El historial vive en la pestaña del navegador y se borra al cerrar sesión.
admin-settings-key-ui-bar-mail-top-bar = Barra de herramientas de correo
admin-settings-help-ui-bar-mail-top-bar = «pinned» mantiene siempre visible la barra de herramientas de correo con los controles de disposición. «auto» la oculta tras un asa delgada que la abre cuando un usuario la señala, la toca o llega a ella con el tabulador.
admin-settings-key-ui-bar-workspace-tabs = Pestañas del espacio de trabajo
admin-settings-help-ui-bar-workspace-tabs = «pinned» mantiene siempre visible la tira de pestañas del espacio de trabajo. «auto» muestra solo un asa delgada con el nombre de la pestaña activa hasta que un usuario la abre.
admin-settings-key-ui-bar-mail-ribbon = Cinta de correo
admin-settings-help-ui-bar-mail-ribbon = Para la disposición con cinta. «pinned» mantiene la cinta abierta. «auto» muestra solo los nombres de las pestañas; una pestaña abre la cinta sobre la lista de mensajes.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-ui-bar-composer-ribbon = Cinta del editor
admin-settings-help-ui-bar-composer-ribbon = Para cada editor de mensajes. «pinned» mantiene abierta la cinta del editor. «auto» muestra solo los nombres de las pestañas; al abrir una pestaña, el cuerpo del mensaje se desplaza hacia abajo en lugar de quedar tapado.
admin-settings-key-appearance-theme = Tema
admin-settings-help-appearance-theme = El tema de color que ven los usuarios hasta que elijan uno propio. Bloquearlo mantiene a todos los usuarios en este tema.
admin-settings-key-appearance-accent = Color de acento
admin-settings-help-appearance-accent = El color de acento del tema. Vacío significa el acento propio del tema.
admin-settings-key-appearance-badge-color = Color de la insignia de no leídos
admin-settings-help-appearance-badge-color = El color de las insignias del contador de no leídos. Predeterminado conserva el color del tema; Acento sigue el color de acento; accent-lighter y accent-darker usan el acento un 20 % más claro u oscuro, con el número en negro o blanco. Un bloqueo prevalece sobre la elección de la cuenta.
admin-settings-key-appearance-density = Densidad
admin-settings-help-appearance-density = Cuánto espacio ocupan las listas y los controles: compacta, cómoda o relajada.
admin-settings-key-appearance-font = Fuente de la interfaz
admin-settings-key-packages-allowed = Seguimiento de paquetes permitido
admin-settings-help-packages-allowed = Permite el seguimiento de paquetes para las cuentas afectadas. El proveedor también debe estar activado en Seguimiento de paquetes, y cada usuario debe dar su consentimiento. Desactivar este ajuste impide de inmediato nuevas llamadas al proveedor.
admin-settings-help-appearance-font = Elija una fuente incluida para las cuentas sin preferencia de fuente, o bloquee la elección para todas las cuentas afectadas. Las fuentes se cargan desde este servidor; no se hace ninguna solicitud a Google Fonts. Predeterminado usa la fuente del tema.
admin-settings-key-appearance-brand-name = Nombre del producto
admin-settings-help-appearance-brand-name = El nombre que se muestra en la página de inicio de sesión y en la pestaña del navegador.
admin-settings-key-login-notice = Aviso de inicio de sesión
admin-settings-help-login-notice = Un texto breve que se muestra en la página de inicio de sesión, por ejemplo un contacto de soporte. Déjelo vacío para no mostrar ninguno.
admin-settings-key-mail-read-receipts = Confirmaciones de lectura
admin-settings-key-receipts-dsn-allowed = Solicitudes de estado de entrega
admin-settings-help-receipts-dsn-allowed = Permite que las cuentas soliciten notificaciones de estado de entrega SMTP. Esto no solicita ni demuestra que un mensaje se haya leído.
admin-settings-help-mail-read-receipts = Qué ocurre cuando un mensaje abierto pide una confirmación de lectura: preguntar al usuario, no enviar nunca o enviar siempre.
admin-settings-key-notifications-enabled = Notificaciones de correo nuevo
admin-settings-help-notifications-enabled = Si los usuarios reciben notificaciones de correo nuevo. No cambia sus reglas de notificación.
admin-settings-key-offline-retention-days = Antigüedad de la caché sin conexión
admin-settings-help-offline-retention-days = Cuántos días conserva un dispositivo los mensajes para su uso sin conexión. 0 significa sin límite de antigüedad.
admin-settings-key-offline-budget-mb = Tamaño de la caché sin conexión
admin-settings-help-offline-budget-mb = Cuántos megabytes puede usar un dispositivo para el correo sin conexión. 0 significa sin caché sin conexión.
admin-settings-key-assistant-enabled = Asistente de IA
admin-settings-help-assistant-enabled = Si los usuarios ven el asistente de IA. El interruptor de despliegue de Asistente de IA sigue aplicándose por encima de este.
admin-settings-key-assist-profile = Perfil de proveedor de IA
admin-settings-help-assist-profile = El ID del perfil de proveedor que atiende a estos usuarios (Asistente de IA > Perfiles de proveedor). Vacío usa el punto de conexión del despliegue. Un ID de perfil que no existe desactiva el asistente de IA para estos usuarios.
admin-settings-key-upstream-default = Servidor de correo predeterminado
admin-settings-help-upstream-default = El servidor de correo en el que inicia sesión una dirección cuando ningún dominio indica uno. La tabla de dominios prevalece sobre este.
admin-settings-key-upstream-proxy-timeout-secs = Tiempo de espera del servidor de correo
admin-settings-help-upstream-proxy-timeout-secs = Cuánto puede tardar una solicitud al servidor de correo, en segundos.
admin-settings-key-signin-new-users = Usuarios nuevos
admin-settings-help-signin-new-users = Quién puede crear una cuenta en el primer inicio de sesión: cualquiera (abierto), solo las direcciones añadidas de antemano (preaprovisionado), estas más los usuarios importados (aprovisionado) o nadie nuevo (cerrado). Las cuentas existentes nunca se ven afectadas. Se cambia en la Política de inicio de sesión.
admin-settings-key-signin-new-users-mail = Inicio de sesión con contraseña: usuarios nuevos
admin-settings-help-signin-new-users-mail = Regla para un usuario de buzón nuevo que inicia sesión con contraseña (IMAP, POP3, Exchange). Heredar usa Usuarios nuevos.
admin-settings-key-signin-new-users-sso = Inicio de sesión único: usuarios nuevos
admin-settings-help-signin-new-users-sso = Regla para un usuario nuevo que inicia sesión mediante inicio de sesión único o una cabecera de confianza. Heredar usa Usuarios nuevos.
admin-settings-key-signin-new-users-linked = Buzones vinculados: usuarios nuevos
admin-settings-help-signin-new-users-linked = Regla para vincular un buzón que aún no tiene cuenta aquí, juzgada según el dominio propio del buzón vinculado. Heredar usa Usuarios nuevos.
admin-settings-key-signin-new-users-linked-imap = Buzones IMAP vinculados: usuarios nuevos
admin-settings-help-signin-new-users-linked-imap = Regla solo para buzones IMAP vinculados. Heredar usa Buzones vinculados.
admin-settings-key-signin-new-users-linked-pop3 = Buzones POP3 vinculados: usuarios nuevos
admin-settings-help-signin-new-users-linked-pop3 = Regla solo para buzones POP3 vinculados. Heredar usa Buzones vinculados.
admin-settings-key-signin-new-users-linked-ews = Buzones Exchange vinculados: usuarios nuevos
admin-settings-help-signin-new-users-linked-ews = Regla solo para buzones Exchange (EWS) vinculados. Heredar usa Buzones vinculados.
admin-settings-key-signin-preprovision-ttl-days = Vigencia de los usuarios añadidos previamente (días)
admin-settings-help-signin-preprovision-ttl-days = Cuánto tiempo sigue siendo válido un usuario pendiente añadido de antemano cuando no se indica otra vigencia.
admin-settings-key-signin-preprovision-max-ttl-days = Vigencia máxima de los usuarios añadidos previamente (días)
admin-settings-help-signin-preprovision-max-ttl-days = La vigencia más larga que un administrador puede dar a un usuario pendiente añadido de antemano.
admin-settings-key-signin-preprovision-domain-wide = Permitir entradas previas para todo un dominio
admin-settings-help-signin-preprovision-domain-wide = Permite entradas que admiten a cualquiera de un dominio entero, hasta un límite de usos. Desactivado por defecto.
admin-settings-key-signin-preprovision-retention-days = Conservar usuarios pendientes caducados (días)
admin-settings-help-signin-preprovision-retention-days = Cuánto tiempo se conserva un usuario pendiente caducado antes de eliminarlo.
admin-settings-key-signin-domain-mode = Dominios de inicio de sesión
admin-settings-help-signin-domain-mode = Si puede iniciar sesión cualquier dominio o solo los dominios de la lista. Se cambia en la Política de inicio de sesión.
admin-settings-key-signin-refusal-detail = Mensajes de rechazo
admin-settings-help-signin-refusal-detail = Si un inicio de sesión rechazado indica el motivo (cuenta desactivada, dirección no permitida) o da la misma respuesta que una contraseña incorrecta. Se cambia en la Política de inicio de sesión.
admin-settings-key-signin-ldap-bind-precheck = Comprobación LDAP antes del inicio de sesión
admin-settings-help-signin-ldap-bind-precheck = Comprueba la contraseña en el directorio antes que en el servidor de correo. Se cambia en Integraciones, tras una prueba de enlace.
admin-settings-key-twofa-pending-ttl-secs = Límite de tiempo del segundo factor
admin-settings-help-twofa-pending-ttl-secs = De cuánto tiempo dispone un usuario para introducir el segundo factor tras la contraseña, en segundos.
admin-settings-key-twofa-max-attempts = Intentos del segundo factor
admin-settings-help-twofa-max-attempts = Cuántos códigos de segundo factor incorrectos se permiten antes de que el inicio de sesión empiece de nuevo.
admin-settings-key-admin-session-idle-secs = Tiempo de inactividad de la consola
admin-settings-help-admin-session-idle-secs = Cuánto dura una sesión de la consola de administración sin ninguna solicitud, en segundos.
admin-settings-key-admin-session-absolute-secs = Límite de sesión de la consola
admin-settings-help-admin-session-absolute-secs = Lo máximo que dura una sesión de la consola de administración, en segundos, esté activa o no.
admin-settings-key-admin-session-rotate-secs = Rotación del ID de sesión de la consola
admin-settings-help-admin-session-rotate-secs = Cada cuántos segundos la consola de administración sustituye el ID de sesión mientras se usa (0 = solo tras una confirmación reforzada). La rotación nunca amplía los límites de inactividad ni absoluto.
admin-settings-key-passwd-policy-min-length = Longitud mínima de la contraseña
admin-settings-help-passwd-policy-min-length = El menor número de caracteres que puede tener una contraseña nueva.
admin-settings-key-passwd-policy-require-upper = Exigir una letra mayúscula
admin-settings-help-passwd-policy-require-upper = Una contraseña nueva debe contener una letra mayúscula.
admin-settings-key-passwd-policy-require-lower = Exigir una letra minúscula
admin-settings-help-passwd-policy-require-lower = Una contraseña nueva debe contener una letra minúscula.
admin-settings-key-passwd-policy-require-digit = Exigir un dígito
admin-settings-help-passwd-policy-require-digit = Una contraseña nueva debe contener un dígito.
admin-settings-key-passwd-policy-require-symbol = Exigir un símbolo
admin-settings-help-passwd-policy-require-symbol = Una contraseña nueva debe contener un carácter que no sea una letra ni un dígito.
admin-settings-key-limit-storage-bytes = Límite de almacenamiento
admin-settings-help-limit-storage-bytes = El mayor número de bytes que un usuario puede almacenar en este servidor. 0 significa sin límite.
admin-settings-key-limit-send-per-day = Mensajes enviados por día
admin-settings-help-limit-send-per-day = El mayor número de mensajes que un usuario puede enviar al día. 0 significa sin límite.
admin-settings-key-limit-ai-calls-per-day = Llamadas al asistente de IA por día
admin-settings-help-limit-ai-calls-per-day = El mayor número de solicitudes al asistente de IA que un usuario puede hacer al día. 0 significa sin límite.
admin-settings-key-quota-warn-thresholds = Niveles de aviso de uso
admin-settings-help-quota-warn-thresholds = Los porcentajes de un límite en los que se avisa a los usuarios, de menor a mayor.
admin-settings-key-quota-warn-enabled = Avisos de uso
admin-settings-help-quota-warn-enabled = Si se avisa a los usuarios cuando se acercan a un límite.
admin-settings-key-usage-retention-days = Historial de uso
admin-settings-help-usage-retention-days = Cuántos días de recuentos de uso diarios se conservan.
admin-settings-key-usage-meter-api = Contar el uso de la API
admin-settings-help-usage-meter-api = Si las solicitudes de API se cuentan en las cifras de uso.
admin-settings-key-usage-measure-interval-hours = Intervalo de medición del almacenamiento
admin-settings-help-usage-measure-interval-hours = Con qué frecuencia se miden los datos almacenados de cada usuario, en horas.
admin-settings-key-assistant-rate-limit-per-min = Límite de frecuencia del asistente de IA
admin-settings-help-assistant-rate-limit-per-min = El mayor número de solicitudes al asistente de IA por minuto para todo el servidor. 0 significa sin límite.
admin-settings-key-plugins-kv-max-key-bytes = Tamaño de clave de complementos
admin-settings-help-plugins-kv-max-key-bytes = La clave más larga que puede almacenar un complemento, en bytes.
admin-settings-key-plugins-kv-max-value-bytes = Tamaño de valor de complementos
admin-settings-help-plugins-kv-max-value-bytes = El mayor valor que puede almacenar un complemento bajo una clave, en bytes.
admin-settings-key-plugins-kv-max-total-bytes = Almacenamiento de complementos
admin-settings-help-plugins-kv-max-total-bytes = El mayor número de bytes que puede almacenar un complemento en total.
admin-settings-key-plugins-kv-max-keys = Claves de complementos
admin-settings-help-plugins-kv-max-keys = El mayor número de claves que puede almacenar un complemento.
admin-settings-key-import-max-messages = Número de mensajes en la importación
admin-settings-help-import-max-messages = El mayor número de mensajes que puede traer una importación.
admin-settings-key-import-max-message-bytes = Tamaño de mensaje en la importación
admin-settings-help-import-max-message-bytes = El mensaje más grande que acepta una importación, en bytes.
admin-settings-key-webhooks-max-attempts = Intentos de webhook
admin-settings-help-webhooks-max-attempts = Cuántas veces se intenta una entrega de webhook antes de abandonarla.
admin-settings-key-webhooks-timeout-secs = Tiempo de espera de webhook
admin-settings-help-webhooks-timeout-secs = Cuánto puede tardar una entrega de webhook, en segundos.
admin-settings-key-outbox-max-attempts = Intentos de envío
admin-settings-help-outbox-max-attempts = Cuántas veces se entrega un mensaje al servidor de correo antes de notificarlo como no enviado.
admin-settings-key-realtime-recheck-secs = Nueva comprobación de actualizaciones en vivo
admin-settings-help-realtime-recheck-secs = Con qué frecuencia una conexión de actualizaciones en vivo abierta comprueba que su sesión sigue siendo válida, en segundos.
admin-settings-key-privacy-watermark-enabled = Marca de agua del lector
admin-settings-help-privacy-watermark-enabled = Muestra una marca de agua con la dirección del lector sobre los mensajes abiertos.
admin-settings-key-privacy-watermark-opacity = Intensidad de la marca de agua
admin-settings-help-privacy-watermark-opacity = Lo visible que es la marca de agua, de 0,02 (tenue) a 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Historial del seguimiento de aperturas
admin-settings-help-privacy-open-tracking-retention-days = Cuántos días se conservan los registros de seguimiento de lectura.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Tokens de seguimiento sin enviar
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Cuántas horas espera un token de seguimiento a que se envíe su mensaje antes de eliminarse.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-masked-email-domain = Dominio de direcciones enmascaradas
admin-settings-help-masked-email-domain = El dominio bajo el que se crean las direcciones enmascaradas. Déjelo vacío para desactivar las direcciones enmascaradas.
admin-settings-key-abuse-report-address = Dirección de informes de abuso
admin-settings-help-abuse-report-address = Adónde se envían los informes de spam y abuso.
admin-settings-key-dlp-admin-address = Avisos de pérdida de datos
admin-settings-help-dlp-admin-address = Adónde se envían los avisos de prevención de pérdida de datos.
admin-settings-key-observability-error-forward-url = Dirección de reenvío de errores
admin-settings-help-observability-error-forward-url = Una dirección https a la que se reenvían los informes de errores de los clientes. Déjela vacía para ninguna.
admin-settings-key-egress-probe-url = Dirección de prueba saliente
admin-settings-help-egress-probe-url = La dirección https que se usa para probar un proxy saliente.
admin-settings-key-push-vapid-contact = Contacto de notificaciones push
admin-settings-help-push-vapid-contact = El contacto (mailto: o https:) que se da a los servicios push con cada notificación.
admin-settings-key-push-quiet-hours = Horas de silencio de notificaciones push
admin-settings-help-push-quiet-hours = Horas (UTC) en las que no se envía ninguna notificación push. Déjelo vacío para ninguna.
admin-settings-key-updates-check = Buscar actualizaciones
admin-settings-help-updates-check = Si el servidor busca una vez al día una versión más reciente y la muestra en el panel.
admin-settings-key-updates-feed-url = Canal de versiones
admin-settings-help-updates-feed-url = La dirección https que lee la búsqueda de actualizaciones.
admin-settings-key-updates-apply = Aplicar actualizaciones
admin-settings-help-updates-apply = Si se puede iniciar una actualización desde la consola mediante el agente de actualización del host.
admin-settings-key-backup-retention-count = Copias de seguridad conservadas
admin-settings-help-backup-retention-count = Cuántas copias de seguridad de la base de datos se conservan.
admin-settings-key-about-security-contact = Contacto de seguridad
admin-settings-help-about-security-contact = El contacto de security.txt (mailto: o https:).
admin-settings-key-about-security-policy-url = Política de seguridad
admin-settings-help-about-security-policy-url = La dirección de su política de seguridad, que figura en security.txt.
admin-settings-key-about-project-links = Enlaces de Acerca de
admin-settings-help-about-project-links = Enlaces que se muestran en la página Acerca de, cada uno con una etiqueta y una dirección https.
admin-settings-key-config-snapshots-kept = Instantáneas de configuración
admin-settings-help-config-snapshots-kept = Cuántas instantáneas de configuración se conservan.
admin-settings-key-jobs-history-days = Historial de tareas
admin-settings-help-jobs-history-days = Cuántos días de ejecuciones de tareas se conservan.
admin-settings-key-maintenance-announce-minutes-default = Antelación del aviso de mantenimiento
admin-settings-help-maintenance-announce-minutes-default = Cuántos minutos antes de una ventana de mantenimiento se informa a los usuarios.
admin-settings-key-maintenance-default-message = Mensaje de mantenimiento
admin-settings-help-maintenance-default-message = El texto que ven los usuarios durante el mantenimiento, por idioma.
admin-settings-key-health-disk-free-min-percent = Mínimo de espacio libre en disco
admin-settings-help-health-disk-free-min-percent = La comprobación de disponibilidad falla cuando el espacio libre en disco cae por debajo de este porcentaje.
admin-settings-key-observability-otlp-metrics = Enviar métricas por OTLP
admin-settings-help-observability-otlp-metrics = Envía las métricas del servidor al recolector OTLP definido al arrancar.
admin-settings-key-metrics-drilldown-top-n = Filas de detalle de métricas
admin-settings-help-metrics-drilldown-top-n = Cuántas filas muestra un desglose de métricas.
admin-settings-key-metrics-rollup-days = Historial de métricas
admin-settings-help-metrics-rollup-days = Cuántos días de resúmenes horarios de métricas se conservan.
admin-settings-key-observability-client-timing-enabled = Tiempos de carga del navegador
admin-settings-help-observability-client-timing-enabled = Recoge los tiempos de carga de página de los navegadores con sesión iniciada como histogramas agregados anónimos. Nada identifica a una persona, sesión o página. Desactívelo para detener la recogida.
admin-settings-key-observability-client-timing-sample-rate-percent = Tasa de muestreo de tiempos de carga del navegador
admin-settings-help-observability-client-timing-sample-rate-percent = Porcentaje de sesiones de navegador que informan de tiempos de carga (de 1 a 100).
admin-settings-key-observability-client-timing-min-samples = Muestras mínimas de tiempos de carga
admin-settings-help-observability-client-timing-min-samples = La mediana y el percentil 95 de los tiempos de carga permanecen ocultos hasta que un periodo tenga al menos este número de muestras.
admin-settings-key-observability-client-timing-folder-loads = Tiempos de carga de carpetas
admin-settings-help-observability-client-timing-folder-loads = Mide cuánto tarda una carpeta en mostrar su lista de mensajes, según venga de la caché del navegador o de la red. Activado por defecto. Los usuarios pueden desactivarlo para sí mismos salvo que este ajuste esté bloqueado. Requiere que los tiempos de carga del navegador estén activados.
admin-settings-key-observability-client-timing-message-loads = Tiempos de carga de mensajes
admin-settings-help-observability-client-timing-message-loads = Mide cuánto tarda un mensaje en mostrar sus cabeceras y su cuerpo saneado, según estuviera en caché o precargado, o viniera de la red. Activado por defecto. Los usuarios pueden desactivarlo para sí mismos salvo que este ajuste esté bloqueado. Requiere que los tiempos de carga del navegador estén activados.
admin-settings-key-diagnostics-retention-days = Historial de diagnósticos
admin-settings-help-diagnostics-retention-days = Cuántos días de ejecuciones de diagnóstico se conservan.
admin-settings-key-data-export-link-hours = Ventana de descarga de exportaciones de datos
admin-settings-help-data-export-link-hours = Cuántas horas puede descargarse una exportación de datos antes de eliminarse.
admin-settings-key-dsr-enabled = Solicitudes de datos
admin-settings-help-dsr-enabled = Si las personas pueden presentar solicitudes de datos (acceso, descarga, corrección, supresión, limitación, oposición) desde los ajustes.
admin-settings-key-dsr-type-access = Aceptar solicitudes de acceso
admin-settings-help-dsr-type-access = Si el tipo de solicitud de acceso se ofrece a los usuarios.
admin-settings-key-dsr-type-portability = Aceptar solicitudes de descarga (copia portátil)
admin-settings-help-dsr-type-portability = Si el tipo de solicitud de descarga (copia portátil) se ofrece a los usuarios.
admin-settings-key-dsr-type-rectification = Aceptar solicitudes de corrección
admin-settings-help-dsr-type-rectification = Si el tipo de solicitud de corrección se ofrece a los usuarios.
admin-settings-key-dsr-type-erasure = Aceptar solicitudes de supresión
admin-settings-help-dsr-type-erasure = Si el tipo de solicitud de supresión se ofrece a los usuarios.
admin-settings-key-dsr-type-restriction = Aceptar solicitudes de limitación
admin-settings-help-dsr-type-restriction = Si el tipo de solicitud de limitación se ofrece a los usuarios.
admin-settings-key-dsr-type-objection = Aceptar solicitudes de oposición
admin-settings-help-dsr-type-objection = Si el tipo de solicitud de oposición se ofrece a los usuarios.
admin-settings-key-dsr-type-other = Aceptar otras solicitudes
admin-settings-help-dsr-type-other = Si el tipo de solicitud genérica se ofrece a los usuarios.
admin-settings-key-dsr-approval-export = Aprobación de solicitudes de acceso y descarga
admin-settings-help-dsr-approval-export = auto inicia la exportación en cuanto se confirma la solicitud. admin espera a que un administrador la apruebe.
admin-settings-key-dsr-approval-erasure = Aprobación de solicitudes de supresión
admin-settings-help-dsr-approval-erasure = auto programa la supresión en cuanto se confirma la solicitud. admin espera a que un administrador la apruebe.
admin-settings-key-dsr-approval-other = Tramitación de otras solicitudes
admin-settings-help-dsr-approval-other = auto marca la solicitud como en curso una vez confirmada. admin espera a que un administrador la apruebe.
admin-settings-key-dsr-reauth-required = Reautenticación para las solicitudes
admin-settings-help-dsr-reauth-required = Exige una contraseña o un segundo factor recientes para presentar una solicitud de exportación o supresión y para crear un enlace de descarga.
admin-settings-key-dsr-confirmation-required = Mensaje de confirmación
admin-settings-help-dsr-confirmation-required = Envía un mensaje de confirmación con un código a la bandeja de entrada del solicitante; la solicitud no continúa hasta que se introduce el código.
admin-settings-key-dsr-identity-verification-required = Paso de verificación de identidad
admin-settings-help-dsr-identity-verification-required = Un administrador debe registrar que se verificó la identidad del solicitante antes de aprobar la solicitud.
admin-settings-key-dsr-erasure-grace-days = Plazo de gracia de la supresión
admin-settings-help-dsr-erasure-grace-days = Días entre la aprobación y la supresión. El solicitante puede cancelar durante este tiempo. 0 ejecuta la supresión en la siguiente pasada del proceso de trabajo.
admin-settings-key-dsr-sla-days = Días para responder a una solicitud
admin-settings-help-dsr-sla-days = El número de días desde la presentación hasta la fecha de vencimiento que se muestra a usuarios y administradores.
admin-settings-key-dsr-sla-max-extension-days = Prórroga máxima
admin-settings-help-dsr-sla-max-extension-days = El mayor número de días que un administrador puede añadir al vencimiento de una solicitud, con un motivo registrado.
admin-settings-key-dsr-sla-reminder-days = Avisos de vencimiento
admin-settings-help-dsr-sla-reminder-days = Días antes del vencimiento a partir de los cuales una solicitud se marca en la cola como próxima a vencer.
admin-settings-key-dsr-export-mail-allowed = Correo en las exportaciones
admin-settings-help-dsr-export-mail-allowed = Permite que un solicitante incluya mensajes en una exportación. Solo se incluyen los mensajes guardados en la caché propia de este servidor; el correo que permanece en el servidor de correo de origen no se incluye.
admin-settings-key-dsr-export-mail-format = Formato del correo en las exportaciones
admin-settings-help-dsr-export-mail-format = mbox escribe un archivo por carpeta; eml escribe un archivo por mensaje.
admin-settings-key-dsr-export-include-settings = Exportación: ajustes
admin-settings-help-dsr-export-include-settings = Incluir los ajustes en una exportación.
admin-settings-key-dsr-export-include-identities = Exportación: identidades
admin-settings-help-dsr-export-include-identities = Incluir las identidades en una exportación.
admin-settings-key-dsr-export-include-contacts = Exportación: contactos
admin-settings-help-dsr-export-include-contacts = Incluir los contactos en una exportación.
admin-settings-key-dsr-export-include-calendars = Exportación: calendarios
admin-settings-help-dsr-export-include-calendars = Incluir los calendarios en una exportación.
admin-settings-key-dsr-export-include-notes = Exportación: notas
admin-settings-help-dsr-export-include-notes = Incluir las notas en una exportación.
admin-settings-key-dsr-export-include-rules = Exportación: filtros y reglas
admin-settings-help-dsr-export-include-rules = Incluir los filtros y las reglas en una exportación.
admin-settings-key-dsr-export-include-sessions = Exportación: sesiones, dispositivos e historial de inicios de sesión
admin-settings-help-dsr-export-include-sessions = Incluir las sesiones, los dispositivos y el historial de inicios de sesión en una exportación.
admin-settings-key-dsr-export-include-consent = Exportación: registros de consentimiento
admin-settings-help-dsr-export-include-consent = Incluir los registros de consentimiento en una exportación.
admin-settings-key-dsr-export-include-audit = Exportación: entradas de auditoría sobre la persona
admin-settings-help-dsr-export-include-audit = Incluir en una exportación las entradas de auditoría sobre la persona.
admin-settings-key-dsr-export-max-megabytes = Tamaño máximo de la exportación
admin-settings-help-dsr-export-max-megabytes = La mayor exportación en megabytes. Las secciones que la superarían se omiten y se listan en el manifiesto.
admin-settings-key-dsr-export-format = Formatos de exportación
admin-settings-help-dsr-export-format = json escribe archivos JSON. jsonCsv escribe además archivos CSV para los datos en forma de lista.
admin-settings-key-dsr-export-encrypt = Cifrar las exportaciones en reposo
admin-settings-help-dsr-export-encrypt = Almacena las exportaciones generadas cifradas con la clave del servidor hasta que se descarguen o caduquen.
admin-settings-key-dsr-download-link-minutes = Vigencia del enlace de descarga
admin-settings-help-dsr-download-link-minutes = Minutos durante los que funciona un enlace de descarga tras crearse. La exportación en sí se conserva durante la ventana de descarga de exportaciones de datos.
admin-settings-key-dsr-download-single-use = Enlace de descarga de un solo uso
admin-settings-help-dsr-download-single-use = Un enlace de descarga deja de funcionar tras usarse una vez.
admin-settings-key-dsr-retention-completed-days = Conservación de solicitudes finalizadas
admin-settings-help-dsr-retention-completed-days = Días que se conservan una solicitud finalizada y su historial antes de eliminarlos.
admin-settings-key-dsr-notify-admins = Registrar las solicitudes nuevas para los administradores
admin-settings-help-dsr-notify-admins = Registra cada solicitud nueva como evento de seguridad en el registro de auditoría, donde la ven los administradores y los destinos de alerta.
admin-settings-key-dsr-notify-requester = Mensajes de progreso al solicitante
admin-settings-help-dsr-notify-requester = Deja un mensaje en la bandeja de entrada del solicitante cuando una solicitud cambia de estado.
admin-settings-key-dsr-contact-name = Nombre del contacto de protección de datos
admin-settings-help-dsr-contact-name = Nombre que se muestra a los usuarios en la página de solicitudes de datos.
admin-settings-key-dsr-contact-email = Dirección del contacto de protección de datos
admin-settings-help-dsr-contact-email = Dirección de correo que se muestra a los usuarios en la página de solicitudes de datos.
admin-settings-key-dsr-contact-url = Página del contacto de protección de datos
admin-settings-help-dsr-contact-url = Enlace que se muestra a los usuarios en la página de solicitudes de datos.
admin-settings-key-dsr-on-behalf-allowed = Solicitudes en nombre de un usuario
admin-settings-help-dsr-on-behalf-allowed = Permite que un administrador presente una solicitud para un usuario, por ejemplo una recibida por correo postal.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-dsr-rate-limit-per-day = Solicitudes por persona y día
admin-settings-help-dsr-rate-limit-per-day = El mayor número de solicitudes que una persona puede presentar en 24 horas.
admin-settings-key-dsr-legal-hold-subjects = Retenciones legales
admin-settings-help-dsr-legal-hold-subjects = Direcciones, o entradas @dominio, cuya supresión está bloqueada. Una solicitud de supresión que las afecte permanece en la cola como bloqueada hasta que se quite la entrada.
admin-settings-key-scrub-enabled = Depuración de una persona
admin-settings-help-scrub-enabled = Permite la acción Depurar persona y la depuración que puede seguir a una supresión. Si está desactivado, no se ejecuta ninguna de las dos.
admin-settings-key-scrub-pseudonym-algorithm = Longitud del seudónimo
admin-settings-help-scrub-pseudonym-algorithm = Cómo se genera el seudónimo con clave que sustituye a un identificador: HMAC-SHA256 con la generación de clave actual, recortado a 16 o 32 caracteres hexadecimales.
admin-settings-key-scrub-include-logs = Depurar archivos de registro
admin-settings-help-scrub-include-logs = Una depuración también reescribe el identificador en los archivos de registro rotados, incluidos los comprimidos.
admin-settings-key-scrub-auto-run-on-erasure = Depuración tras la supresión
admin-settings-help-scrub-auto-run-on-erasure = Una supresión completada también sustituye los identificadores de la persona en el historial de auditoría y los registros por el seudónimo. Desactivado por defecto.
admin-settings-key-scrub-key-retention-days = Conservación de las claves de seudónimo
admin-settings-help-scrub-key-retention-days = Días que se conserva una clave de seudónimo tras rotarla. 0 la conserva. Tras eliminarla, los seudónimos antiguos ya no pueden recalcularse y los marcadores creados con la clave ya no pueden reproducirse.
admin-settings-key-backup-propagate-erasure = Reproducir supresiones al restaurar
admin-settings-help-backup-propagate-erasure = Una restauración reproduce los marcadores de supresión más recientes que la copia de seguridad, de modo que las personas suprimidas desde entonces vuelven a eliminarse.
admin-settings-key-backup-tombstone-retention-days = Conservación de marcadores de supresión
admin-settings-help-backup-tombstone-retention-days = Días que se conserva un marcador de supresión. 0 los conserva siempre.
admin-settings-key-dsr-text-intro = Introducción de las solicitudes de datos
admin-settings-help-dsr-text-intro = Texto que se muestra sobre el formulario de solicitud. Un valor por idioma.
admin-settings-key-dsr-text-confirmation = Texto del mensaje de confirmación
admin-settings-help-dsr-text-confirmation = Texto que se añade al mensaje de confirmación. Un valor por idioma.
admin-settings-key-dsr-text-completion = Texto del mensaje de finalización
admin-settings-help-dsr-text-completion = Texto que se añade al mensaje enviado cuando se completa una solicitud. Un valor por idioma.
admin-settings-key-server-text-family = Texto del servidor
admin-settings-help-server-text-family = Texto que el servidor escribe en el correo y las notificaciones, por idioma y dominio.
admin-settings-cat-sieve = Filtros de correo
admin-settings-key-sieve-max-script-bytes = Tamaño máximo del script de filtro
admin-settings-help-sieve-max-script-bytes = Número máximo de bytes de un script de filtro Sieve.
admin-settings-key-sieve-max-scripts = Máximo de scripts de filtro
admin-settings-help-sieve-max-scripts = Número máximo de scripts guardados por cuenta.
admin-settings-key-sieve-max-redirects = Máximo de redirecciones
admin-settings-help-sieve-max-redirects = Número máximo de comandos de redirección en un script.
admin-settings-key-sieve-vacation-allowed = Acciones de vacaciones en filtros
admin-settings-help-sieve-vacation-allowed = Permite que los scripts envíen respuestas de vacaciones cuando el backend las admite.
admin-settings-key-sieve-allowed-extensions = Extensiones de filtro permitidas
admin-settings-help-sieve-allowed-extensions = Una lista opcional de nombres de extensiones Sieve en minúsculas. El backend también debe admitir cada extensión.
admin-settings-key-cap-sieve-raw-edit = Editar el código fuente del script de filtro
admin-settings-help-cap-sieve-raw-edit = Permite la edición directa de scripts Sieve para las cuentas seleccionadas.
admin-settings-key-cap-sieve-forward-external = Reenviar fuera del dominio de la cuenta
admin-settings-help-cap-sieve-forward-external = Permite el reenvío externo en scripts. Guardar estos scripts también requiere una verificación reciente.
admin-settings-key-session-max-concurrent = Sesiones simultáneas
admin-settings-help-session-max-concurrent = Número máximo de sesiones por cuenta. Las sesiones más antiguas se revocan cuando un inicio de sesión nuevo supera este límite.
admin-settings-key-session-idle-secs = Tiempo de inactividad de la sesión
admin-settings-help-session-idle-secs = Finaliza las sesiones inactivas tras este número de segundos, hasta el límite del despliegue.
admin-settings-key-session-absolute-secs = Duración de la sesión
admin-settings-help-session-absolute-secs = Finaliza las sesiones tras este número de segundos, hasta el límite del despliegue.
admin-settings-key-twofa-remember-days = Vigencia de los dispositivos recordados
admin-settings-help-twofa-remember-days = Días antes de que un dispositivo recordado deba verificar de nuevo un segundo factor. Cero desactiva esta función.
admin-settings-key-twofa-passkeys-enabled = Permitir claves de acceso
admin-settings-help-twofa-passkeys-enabled = Permite las claves de acceso como factor de autenticación.
admin-settings-key-twofa-required = Exigir autenticación de dos factores
admin-settings-help-twofa-required = Exige un segundo factor registrado antes de emitir una sesión.
admin-settings-key-twofa-passkey-attachment = Tipo de autenticador de clave de acceso
admin-settings-help-twofa-passkey-attachment = any (predeterminado): todos los autenticadores. platform: claves de acceso guardadas en el dispositivo (Windows Hello, Touch ID, bloqueo de pantalla de Android). cross-platform: llaves de seguridad y teléfonos. Lo informa el navegador; no está firmado por el autenticador.
admin-settings-key-twofa-passkey-user-verification = Verificación del usuario en claves de acceso
admin-settings-help-twofa-passkey-user-verification = required: la clave de acceso debe comprobar un PIN o un dato biométrico al añadirse y en cada inicio de sesión; se rechaza una operación sin ello. preferred (predeterminado) y discouraged solo cambian lo que se pide al navegador.
admin-settings-key-twofa-passkey-resident-key = Claves de acceso detectables
admin-settings-help-twofa-passkey-resident-key = required: la credencial debe guardarse en el autenticador. discouraged (predeterminado) o preferred. En modo motor, las claves de acceso son siempre detectables, porque también permiten iniciar sesión sin contraseña.
admin-settings-key-twofa-passkey-legacy = Claves de acceso añadidas antes de la política actual
admin-settings-help-twofa-passkey-legacy = allow (predeterminado): una clave de acceso antigua que no cumple la política sigue funcionando y se pide al usuario que la sustituya. refuse: se rechaza al iniciar sesión con un mensaje; el usuario inicia sesión con otro factor o un código de recuperación.
admin-settings-key-twofa-last-factor-removal = Quitar el último segundo factor
admin-settings-help-twofa-last-factor-removal = Cuando quitar una clave de acceso o una aplicación de autenticación dejaría la cuenta sin ninguno. allow (predeterminado): se quita y el inicio de sesión solo necesita la contraseña. block: el usuario debe añadir antes otro método de inicio de sesión (los códigos de recuperación no cuentan). Donde se exige la autenticación de dos factores, el último siempre se conserva.
admin-settings-key-privacy-remote-content = Contenido remoto
admin-settings-help-privacy-remote-content = Bloquear imágenes, preguntar primero, usar el proxy o cargar directamente cuando se permita.
admin-settings-key-privacy-trackers = Bloquear rastreadores
admin-settings-help-privacy-trackers = Bloquea las imágenes de seguimiento de correo conocidas.
admin-settings-key-privacy-clean-links = Limpiar enlaces
admin-settings-help-privacy-clean-links = Quita los parámetros de seguimiento de los enlaces de los mensajes.
admin-settings-key-privacy-max-bytes = Límite de imágenes remotas
admin-settings-help-privacy-max-bytes = Número máximo de bytes que se obtienen para una imagen remota.
admin-settings-key-privacy-cache-bytes = Caché de imágenes remotas
admin-settings-help-privacy-cache-bytes = Número máximo de bytes que se conservan en la caché de imágenes remotas.
admin-settings-key-privacy-rate-per-min = Frecuencia de solicitudes de imágenes remotas
admin-settings-help-privacy-rate-per-min = Número máximo de obtenciones de imágenes remotas por cuenta y minuto.
admin-settings-privacy-fetch-title = Obtención de imágenes remotas
admin-settings-privacy-fetch-help = Límites de tiempo, redirecciones y descargas en paralelo de las imágenes que el servidor obtiene para el lector. Una imagen que supere un límite muestra un marcador de posición.
admin-settings-key-privacy-fetch-connect-timeout-ms = Tiempo de conexión de imágenes remotas
admin-settings-help-privacy-fetch-connect-timeout-ms = Milisegundos permitidos para abrir la conexión con un host de imágenes, incluido el handshake TLS. Por defecto 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Tiempo de respuesta de imágenes remotas
admin-settings-help-privacy-fetch-headers-timeout-ms = Milisegundos permitidos desde el inicio de una solicitud hasta que el host de imágenes responde con las cabeceras. Por defecto 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Tiempo de bloqueo de imágenes remotas
admin-settings-help-privacy-fetch-idle-timeout-ms = La pausa más larga, en milisegundos, entre dos partes de la descarga de una imagen antes de detenerla. Por defecto 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Tiempo total de imágenes remotas
admin-settings-help-privacy-fetch-total-timeout-ms = Milisegundos permitidos para obtener una imagen, incluidas las redirecciones. La espera en cola tiene un presupuesto aparte, de modo que una imagen muy solicitada no agota su tiempo de red antes de empezar. Por defecto 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Tiempo de cola de imágenes remotas
admin-settings-help-privacy-fetch-queue-timeout-ms = Milisegundos máximos que las imágenes pueden esperar capacidad, además de su presupuesto de obtención. La cola acotada admite grandes ráfagas autenticadas mientras se siguen aplicando los límites de obtención y decodificación activas. Por defecto 60000.
admin-settings-key-privacy-max-redirects = Redirecciones de imágenes remotas
admin-settings-help-privacy-max-redirects = Redirecciones que se siguen para una imagen. Cada una se comprueba con las mismas reglas de dirección que la primera solicitud. Por defecto 3.
admin-settings-key-privacy-max-concurrent-per-message = Descargas de imágenes remotas por mensaje
admin-settings-help-privacy-max-concurrent-per-message = Imágenes de un mensaje que se obtienen a la vez. Las demás esperan un hueco libre. Rango 1–128; por defecto 30.
admin-settings-key-privacy-max-concurrent-per-account = Descargas de imágenes remotas por cuenta
admin-settings-help-privacy-max-concurrent-per-account = Imágenes que se obtienen a la vez para una cuenta entre todos sus mensajes abiertos. Rango 1–256; por defecto 128.
admin-settings-key-privacy-allow-direct = Permitir imágenes directas
admin-settings-key-cap-remote-content-direct = Permiso de imágenes directas
admin-settings-help-cap-remote-content-direct = Permite que esta cuenta elija imágenes directas cuando el servidor lo permita.
admin-settings-help-privacy-allow-direct = Permite la carga directa de imágenes para las cuentas con el permiso necesario.
admin-settings-key-logs-auth-retention-days = Conservación del historial de autenticación
admin-settings-help-logs-auth-retention-days = Días que se conservan los metadatos de autenticación, independientemente de la identidad duradera de la cuenta.
admin-settings-key-logs-auth-max-entries = Límite de entradas del historial de autenticación
admin-settings-help-logs-auth-max-entries = Número máximo de entradas conservadas. Los fallos repetidos idénticos en un minuto comparten una entrada.
admin-settings-key-reader-color-mode = Colores de los mensajes
admin-settings-cat-reader = Visualización de mensajes
admin-settings-key-mail-row-actions-mode = Acciones rápidas de la lista de mensajes
admin-settings-help-mail-row-actions-mode = Cómo aparecen las acciones rápidas de cada fila de mensaje: open (siempre disponibles, se muestran al pasar el cursor o al enfocar), hover (tres controles y un botón Más), click (un botón que las abre) u off (se quitan de las filas; el menú contextual y la barra de herramientas conservan todas las acciones). Hover se comporta como click en pantallas táctiles. Los usuarios pueden elegir la suya salvo que un administrador la bloquee.
admin-settings-key-mail-quick-reply = Respuesta rápida
admin-settings-help-mail-quick-reply = Ofrece una pestaña de respuesta compacta con los destinatarios ya rellenados. Desactivado por defecto; los usuarios pueden activarlo salvo que un administrador lo bloquee. Se siguen aplicando las protecciones de envío habituales.
admin-settings-key-mail-request-read-receipt = Solicitar confirmaciones de lectura por defecto
admin-settings-help-mail-request-read-receipt = Inicia los mensajes salientes compatibles con una solicitud de confirmación seleccionada. Es un valor predeterminado de redacción, no una garantía de que el destinatario envíe una confirmación.
admin-settings-help-reader-color-mode = Automático sigue los temas oscuros. Elija una representación original, adaptativa o invertida dentro del marco aislado del mensaje.
admin-settings-key-reader-font-size = Tamaño de fuente de los mensajes
admin-settings-help-reader-font-size = Tamaño de lectura predeterminado en píxeles, de 12 a 24.
admin-settings-key-reader-line-height = Altura de línea de los mensajes
admin-settings-help-reader-line-height = Interlineado de lectura de 1,2 a 2,2.
admin-settings-key-reader-content-width = Ancho de lectura de los mensajes
admin-settings-help-reader-content-width = Contenido de los mensajes cómodo, ancho o a todo el ancho del panel.
admin-settings-key-reader-loading = Aspecto de la carga de mensajes
admin-settings-help-reader-loading = Líneas de marcador de posición, un indicador de texto o ningún indicador visual. Los avisos de accesibilidad siguen disponibles.
admin-settings-key-reader-loading-delay-ms = Retraso de carga de mensajes
admin-settings-help-reader-loading-delay-ms = Retrasa solo el indicador de carga, entre 0 y 1000 milisegundos. La entrega del mensaje nunca se retrasa.
admin-settings-key-reader-zoom = Zoom de los mensajes
admin-settings-help-reader-zoom = Zoom predeterminado del cuerpo del mensaje en porcentaje, de 50 a 200; el lector usa su paso de zoom más cercano. Solo se escala el mensaje, y la impresión usa el 100 %.
admin-settings-key-reader-attachments-collapsed = Barra de adjuntos contraída
admin-settings-help-reader-attachments-collapsed = Si la barra de adjuntos del lector empieza contraída, mostrando solo el recuento y el tamaño total. Aquí se guarda la última apertura o cierre del usuario. Desactivado por defecto: se muestran los adjuntos.
admin-settings-key-login-show-domain-hint = Mostrar el dominio preferido al iniciar sesión
admin-settings-help-login-show-domain-hint = Muestra qué dominio acepta un nombre de usuario sin sufijo de correo. Oculto por defecto. Solo cambia la indicación; iniciar sesión con un nombre de usuario sigue funcionando.
admin-settings-key-signin-retry-cooldown-secs = Espera entre intentos de inicio de sesión
admin-settings-help-signin-retry-cooldown-secs = Segundos mínimos entre intentos de inicio de sesión con contraseña desde una misma red de cliente. Por defecto 3; 0 desactiva la espera. Las prohibiciones existentes por fallos se siguen aplicando.
admin-settings-key-mail-quote-style = Estilo de cabecera de respuesta y reenvío
admin-settings-key-mail-external-recipient-warning = Aviso de destinatario externo
admin-settings-help-mail-external-recipient-warning = Desactivado, avisar o exigir confirmación en el editor. Abarca Para, Cc y Cco. Es una salvaguarda del editor, no una restricción de entrega del servidor.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-mail-trusted-recipient-domains = Dominios de destinatarios de confianza
admin-settings-help-mail-trusted-recipient-domains = Hasta 100 dominios de destinatario exactos. El dominio de la identidad de envío seleccionada también es de confianza. Las entradas no válidas nunca otorgan confianza. Bloquee este ajuste para impedir que los usuarios lo sustituyan.
admin-settings-help-mail-quote-style = Use respuestas citadas clásicas o cabeceras de mensaje al estilo de Outlook. Esto fija el valor predeterminado del usuario; bloquéelo para imponer el estilo. Los borradores existentes no cambian.
admin-settings-key-signin-anti-bot-mode = Protección contra bots en el inicio de sesión
admin-settings-help-signin-anti-bot-mode = Verificación de prueba de trabajo autoalojada: off, mailbox, admin o all. No se contacta ningún servicio de verificación externo.
admin-settings-key-signin-anti-bot-difficulty = Trabajo de verificación del inicio de sesión
admin-settings-help-signin-anti-bot-difficulty = Contador máximo del solucionador, 100–20000. Los valores más altos exigen más trabajo al navegador; por defecto 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Vigencia de la verificación del inicio de sesión
admin-settings-help-signin-anti-bot-expiry-secs = Segundos que una verificación de un solo uso sigue siendo válida, 60–600; por defecto 180. Se necesita una verificación nueva tras cada intento de credenciales.
admin-settings-key-attachments-preview-allowed = Permitir vistas previas de adjuntos
admin-settings-help-attachments-preview-allowed = Desactívelo para impedir la representación de vistas previas de adjuntos; las descargas de originales tienen su propio control.
admin-settings-key-attachments-download-allowed = Permitir la entrega del adjunto original
admin-settings-help-attachments-download-allowed = Desactivarlo también impide las vistas previas que requieren los bytes originales. Las vistas previas del navegador no son una protección contra copia.
admin-settings-key-attachments-max-bytes = Bytes máximos de adjunto
admin-settings-help-attachments-max-bytes = Cero significa sin límite. Se aplica a los bytes originales entregados, incluidas las solicitudes de rango.
admin-settings-key-attachments-allowed-extensions = Extensiones de nombre de archivo permitidas
admin-settings-help-attachments-allowed-extensions = Matriz JSON de extensiones en minúsculas sin punto, como ["pdf","tar.gz"]. Vacío permite cualquier extensión; las reglas de bloqueo prevalecen.
admin-settings-key-attachments-blocked-extensions = Extensiones de nombre de archivo bloqueadas
admin-settings-help-attachments-blocked-extensions = Matriz JSON de extensiones en minúsculas. Se evalúa con los nombres de archivo MIME guardados, nunca con el nombre de la URL de descarga.
admin-settings-key-attachments-warn-extensions = Avisar sobre extensiones de nombre de archivo
admin-settings-help-attachments-warn-extensions = Matriz JSON de extensiones que requieren una confirmación explícita antes de entregar el original.
admin-settings-key-attachments-allowed-mime-types = Tipos MIME permitidos
admin-settings-help-attachments-allowed-mime-types = Matriz JSON de tipos en minúsculas o comodines de nivel superior, como ["image/*","application/pdf"]. Vacío permite cualquier tipo.
admin-settings-key-attachments-blocked-mime-types = Tipos MIME bloqueados
admin-settings-help-attachments-blocked-mime-types = Matriz JSON de tipos en minúsculas o comodines de nivel superior. Las reglas de bloqueo tienen prioridad.
admin-settings-key-attachments-warn-mime-types = Avisar sobre tipos MIME
admin-settings-help-attachments-warn-mime-types = Matriz JSON de tipos en minúsculas o comodines de nivel superior que requieren confirmación. Los metadatos MIME no son detección de malware.
admin-settings-cat-shortcuts = Atajos de teclado
admin-settings-cat-time = Zonas horarias
admin-settings-key-time-display-zone = Zona horaria de visualización
admin-settings-help-time-display-zone = Zona horaria IANA como Europe/Lisbon, o local para seguir el dispositivo actual. Las marcas de tiempo de los mensajes siguen guardadas como sus instantes originales.
admin-settings-key-time-comparison-zone = Zona horaria de comparación
admin-settings-help-time-comparison-zone = Segunda zona horaria IANA opcional para las ayudas de mensajes y calendario. Déjela vacía para ocultar la comparación; no se infiere ninguna ubicación de una dirección de correo.
admin-settings-key-time-show-helpers = Mostrar ayudas de zona horaria
admin-settings-help-time-show-helpers = Muestra comparaciones entre la hora local y la zona seleccionada para las marcas de tiempo de mensajes y los eventos del calendario. Los bloqueos de administrador pueden imponer esta elección.
admin-settings-key-shortcuts-enabled = Activar atajos de teclado
admin-settings-help-shortcuts-enabled = Permite los atajos de la aplicación. Los bloqueos pueden desactivarlos globalmente o para un dominio. Los campos de escritura y los diálogos modales conservan su gestión normal del teclado.
admin-settings-key-shortcuts-compose = Atajo de Redactar
admin-settings-help-shortcuts-compose = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-archive = Atajo de Archivar
admin-settings-help-shortcuts-archive = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-reply = Atajo de Responder
admin-settings-help-shortcuts-reply = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-reply-all = Atajo de Responder a todos
admin-settings-help-shortcuts-reply-all = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-forward = Atajo de Reenviar
admin-settings-help-shortcuts-forward = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-next = Atajo de Mensaje siguiente
admin-settings-help-shortcuts-next = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-previous = Atajo de Mensaje anterior
admin-settings-help-shortcuts-previous = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-search = Atajo de Enfocar la búsqueda
admin-settings-help-shortcuts-search = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-refresh = Atajo de Actualizar mensajes
admin-settings-help-shortcuts-refresh = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-settings = Atajo de Abrir ajustes
admin-settings-help-shortcuts-settings = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-trash = Atajo de Mover a la papelera
admin-settings-help-shortcuts-trash = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-mark-read = Atajo de Marcar como leído
admin-settings-help-shortcuts-mark-read = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-mark-unread = Atajo de Marcar como no leído
admin-settings-help-shortcuts-mark-unread = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-spam = Atajo de Marcar como spam
admin-settings-help-shortcuts-spam = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-zoom-in = Atajo de Ampliar mensaje
admin-settings-help-shortcuts-zoom-in = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-zoom-out = Atajo de Reducir mensaje
admin-settings-help-shortcuts-zoom-out = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-zoom-reset = Atajo de Restablecer el zoom del mensaje
admin-settings-help-shortcuts-zoom-reset = Use inherit para el ajuste preestablecido seleccionado, disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-shortcuts-open-in-tab = Atajo de Abrir mensaje en su propia pestaña
admin-settings-help-shortcuts-open-in-tab = Use inherit para el ajuste preestablecido seleccionado (Shift+Enter), disabled para quitar esta acción, o una combinación de teclas como Ctrl+Shift+K. Las combinaciones en conflicto o no válidas no se ejecutan.
admin-settings-key-notifications-push-allowed = Notificaciones push en segundo plano permitidas
admin-settings-help-notifications-push-allowed = Permite que las notificaciones activadas de esta cuenta se registren y reciban notificaciones push en segundo plano. Sigue siendo necesario el permiso del navegador.
admin-settings-key-offline-persistent-storage-allowed = Solicitudes de almacenamiento persistente del navegador permitidas
admin-settings-help-offline-persistent-storage-allowed = Permite una solicitud explícita para proteger los datos en caché locales de la expulsión automática del navegador. El navegador puede rechazarla; esto no concede el permiso automáticamente.
admin-settings-key-security-signin-mailbox-failure-limit = Límite de credenciales de buzón fallidas
admin-settings-help-security-signin-mailbox-failure-limit = Las credenciales rechazadas desde una misma IP de origen dentro de la ventana de recuento provocan un bloqueo temporal del inicio de sesión del buzón. Los fallos de red y de configuración no cuentan.
admin-settings-key-security-signin-admin-failure-limit = Límite de credenciales de administrador fallidas
admin-settings-help-security-signin-admin-failure-limit = Las credenciales de administrador rechazadas desde una misma IP de origen provocan un bloqueo temporal independiente del inicio de sesión de administrador. La sesión corta de administrador y los requisitos de prueba reciente no cambian.
admin-settings-key-security-signin-failure-window-secs = Ventana de recuento de credenciales fallidas
admin-settings-help-security-signin-failure-window-secs = Ventana móvil en segundos para los rechazos de credenciales y MFA. Un inicio de sesión completado con éxito restablece ese origen y esa superficie.
admin-settings-key-security-signin-lock-secs = Duración del bloqueo temporal de inicio de sesión
admin-settings-help-security-signin-lock-secs = Duración del bloqueo en segundos. Los intentos repetidos durante el bloqueo no amplían su vencimiento original. Las prohibiciones globales manuales de IP son aparte.
admin-settings-key-security-signin-geo-scope = Ámbito de la política de país en el inicio de sesión
admin-settings-help-security-signin-geo-scope = Desactivado por defecto. Se aplica solo a los inicios de sesión nuevos de buzón, de administrador o a todos. Las sesiones existentes no se revocan. Compruebe el diagnóstico del origen actual antes de activarlo; una base de datos GeoIP local no disponible rechaza el inicio de sesión.
admin-settings-key-security-signin-geo-mode = Lista de países permitidos o denegados
admin-settings-help-security-signin-geo-mode = allow permite solo los países listados; deny rechaza los países listados. Las direcciones desconocidas siguen su propia política explícita.
admin-settings-key-security-signin-geo-countries = Códigos de país del inicio de sesión
admin-settings-help-security-signin-geo-countries = Matriz JSON de códigos de país de dos letras en mayúsculas, por ejemplo ["PT", "GB"]. Usa solo la base de datos de países local (Seguridad > Base de datos de países, o MW_GEOIP_DB); las direcciones nunca se envían a un servicio de ubicación.
admin-settings-key-security-signin-geo-unknown = Política de direcciones de origen sin asignar
admin-settings-help-security-signin-geo-unknown = Permite o deniega las direcciones ausentes de una base de datos local legible, incluidas las direcciones privadas. Las bases de datos ausentes o ilegibles siempre rechazan los inicios de sesión con política activa, incluso si se permiten las direcciones desconocidas.
admin-settings-key-contacts-avatar-mode = Fotos de contactos y avatares generados
admin-settings-help-contacts-avatar-mode = Generar iniciales localmente, ocultar los avatares o usar las fotos incrustadas de las libretas de direcciones propias de cada cuenta. No se contacta ninguna URL de foto remota ni servicio de avatares externo. La política por usuario y por dominio puede fijarla un administrador.
admin-settings-cat-contacts = Fotos de contactos
admin-settings-key-reader-action-style = Cinta de acciones del mensaje
admin-settings-help-reader-action-style = Los iconos compactos son la opción predeterminada. Elija labels para iconos con texto visible; ambos conservan los nombres de las acciones y descripciones emergentes con tema. Admite bloqueos globales, de dominio y de usuario.
admin-settings-key-nextcloud-sync-interval-secs = Intervalo de actualización automática de Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Segundos entre las actualizaciones de colecciones en segundo plano para las cuentas conectadas del motor nativo. Cero significa solo manual; los valores positivos tienen un mínimo de sesenta segundos. Los fallos espacian los intentos. No se intenta ningún inicio de sesión en segundo plano.
admin-settings-key-mail-folder-role-inbox = Asignación de la carpeta Bandeja de entrada
admin-settings-help-mail-folder-role-inbox = Null usa la detección automática. Use un objeto de ruta exacta para valores compartidos por defecto o un objeto con un ID de buzón propio para una cuenta. Los destinos ausentes o en conflicto desactivan este rol; se aplican los bloqueos de administrador.
admin-settings-key-mail-folder-role-drafts = Asignación de la carpeta Borradores
admin-settings-help-mail-folder-role-drafts = Null usa la detección automática. Use un objeto de ruta exacta para valores compartidos por defecto o un objeto con un ID de buzón propio para una cuenta. Los destinos ausentes o en conflicto desactivan este rol; se aplican los bloqueos de administrador.
admin-settings-key-mail-folder-role-sent = Asignación de la carpeta Enviados
admin-settings-help-mail-folder-role-sent = Null usa la detección automática. Use un objeto de ruta exacta para valores compartidos por defecto o un objeto con un ID de buzón propio para una cuenta. Los destinos ausentes o en conflicto desactivan este rol; se aplican los bloqueos de administrador.
admin-settings-key-mail-folder-role-trash = Asignación de la papelera
admin-settings-help-mail-folder-role-trash = Null usa la detección automática. Use un objeto de ruta exacta para valores compartidos por defecto o un objeto con un ID de buzón propio para una cuenta. Los destinos ausentes o en conflicto desactivan este rol; se aplican los bloqueos de administrador.
admin-settings-key-mail-folder-role-junk = Asignación de la carpeta Correo no deseado
admin-settings-help-mail-folder-role-junk = Null usa la detección automática. Use un objeto de ruta exacta para valores compartidos por defecto o un objeto con un ID de buzón propio para una cuenta. Los destinos ausentes o en conflicto desactivan este rol; se aplican los bloqueos de administrador.
admin-settings-key-mail-folder-role-archive = Asignación de la carpeta Archivo
admin-settings-help-mail-folder-role-archive = Null usa la detección automática. Use un objeto de ruta exacta para valores compartidos por defecto o un objeto con un ID de buzón propio para una cuenta. Los destinos ausentes o en conflicto desactivan este rol; se aplican los bloqueos de administrador.
admin-settings-key-mail-request-delivery-receipt = Solicitar notificaciones de entrega por defecto
admin-settings-help-mail-request-delivery-receipt = Selecciona las notificaciones de entrega para los mensajes nuevos cuando la cuenta admite DSN de SMTP. El bloqueo controla este valor predeterminado guardado; receipts.dsnAllowed rige por separado si se permiten las solicitudes. La entrega no prueba la lectura.
admin-settings-key-security-signin-geo-enforce-sessions = Aplicar la política de país a las sesiones existentes
admin-settings-help-security-signin-geo-enforce-sessions = Desactivado por defecto. Aplica el ámbito de país de inicio de sesión configurado a las solicitudes autenticadas, al acceso bearer nativo y al contenido firmado del lector. Los clientes bloqueados aún pueden cerrar sesión. Los flujos ya abiertos se comprueban cuando se reconectan; esto no termina las conexiones existentes.
admin-settings-key-security-protection-rate-mode = Presupuestos agregados de solicitudes
admin-settings-help-security-protection-rate-mode = Desactivado por defecto. observe registra el agotamiento del presupuesto sin rechazar; enforce devuelve un 429 temporal. Los límites de concurrencia de protocolo existentes siguen siendo independientes.
admin-settings-key-security-protection-adaptive-mode = Protección adaptativa de solicitudes
admin-settings-help-security-protection-adaptive-mode = Desactivado por defecto. observe registra anomalías tipificadas explicables y acciones propuestas; enforce aplica limitaciones o bloqueos temporales acotados. Es una política de contadores determinista, no un clasificador de IA.
admin-settings-key-security-protection-client-requests = Solicitudes de cliente por ventana
admin-settings-help-security-protection-client-requests = Solicitudes por ventana desde una dirección sin sesión iniciada, y cada solicitud de inicio de sesión. Una solicitud con una sesión de buzón o de administrador válida usa en su lugar su propia asignación, mayor. Cero desactiva este presupuesto.
admin-settings-key-security-protection-account-requests = Solicitudes de cuenta por ventana
admin-settings-help-security-protection-account-requests = Solicitudes por ventana de un buzón con sesión iniciada, entre todas sus sesiones, dispositivos y direcciones. Se mantiene por encima de la asignación anónima para que el uso con sesión tenga más margen. Cero desactiva este presupuesto.
admin-settings-key-rate-limit-diagnostics-burst = Ejecuciones de diagnóstico simultáneas
admin-settings-help-rate-limit-diagnostics-burst = Ejecuciones de diagnóstico y comprobaciones de backend que una sesión de administrador puede iniciar seguidas. Iniciar una comprobación que ya se está ejecutando devuelve esa comprobación en lugar de contar de nuevo.
admin-settings-key-rate-limit-diagnostics-per-minute = Ejecuciones de diagnóstico por minuto
admin-settings-help-rate-limit-diagnostics-per-minute = Ejecuciones de diagnóstico y comprobaciones de backend por minuto que recupera una sesión de administrador.
admin-settings-key-security-protection-admin-requests = Solicitudes de administrador por ventana
admin-settings-help-security-protection-admin-requests = Solicitudes por ventana de un administrador con sesión iniciada. Se mantiene por encima de la asignación anónima. Cero desactiva este presupuesto.
admin-settings-key-security-protection-rate-window-secs = Ventana del presupuesto de solicitudes (segundos)
admin-settings-help-security-protection-rate-window-secs = Intervalo fijo del presupuesto de solicitudes. El descarte de carga por capacidad y las respuestas ordinarias de «no encontrado» no son evidencia adaptativa.
admin-settings-key-security-protection-adaptive-clients = Observar clientes para la protección adaptativa
admin-settings-help-security-protection-adaptive-clients = Activa esta dimensión de sujeto cuando el modo adaptativo está activo. Las cuentas y los administradores solo se cuentan una vez verificada su identidad.
admin-settings-key-security-protection-adaptive-accounts = Observar cuentas para la protección adaptativa
admin-settings-help-security-protection-adaptive-accounts = Activa esta dimensión de sujeto cuando el modo adaptativo está activo. Las cuentas y los administradores solo se cuentan una vez verificada su identidad.
admin-settings-key-security-protection-adaptive-admins = Observar administradores para la protección adaptativa
admin-settings-help-security-protection-adaptive-admins = Activa esta dimensión de sujeto cuando el modo adaptativo está activo. Las cuentas y los administradores solo se cuentan una vez verificada su identidad.
admin-settings-key-security-protection-short-window-secs = Ventana corta de anomalías (segundos)
admin-settings-help-security-protection-short-window-secs = Ventana de observación móvil. Las ventanas largas admiten desde horas hasta treinta días; el redondeo de intervalos puede conservar como máximo una sesenta y cuatroava parte adicional de una ventana.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protection-short-minimum-samples = Muestras mínimas de la proporción corta
admin-settings-help-security-protection-short-minimum-samples = Número mínimo de solicitudes normales más anómalas observadas antes de que se aplique el disparador por proporción. El umbral absoluto de anomalías es independiente de este mínimo.
admin-settings-key-security-protection-short-absolute-anomalies = Presupuesto absoluto de anomalías corto
admin-settings-help-security-protection-short-absolute-anomalies = Se activa cuando se acumula este número de anomalías seleccionadas, incluso si las solicitudes normales diluyen la proporción. Cero desactiva este disparador.
admin-settings-key-security-protection-short-ratio-percent = Proporción de anomalías corta (porcentaje)
admin-settings-help-security-protection-short-ratio-percent = Se activa cuando las anomalías seleccionadas alcanzan este porcentaje tras el número mínimo de muestras. Basta el disparador absoluto o el de proporción. Cero desactiva este disparador.
admin-settings-key-security-protection-long-window-secs = Ventana larga de anomalías (segundos)
admin-settings-help-security-protection-long-window-secs = Ventana de observación móvil. Las ventanas largas admiten desde horas hasta treinta días; el redondeo de intervalos puede conservar como máximo una sesenta y cuatroava parte adicional de una ventana.
admin-settings-key-security-protection-long-minimum-samples = Muestras mínimas de la proporción larga
admin-settings-help-security-protection-long-minimum-samples = Número mínimo de solicitudes normales más anómalas observadas antes de que se aplique el disparador por proporción. El umbral absoluto de anomalías es independiente de este mínimo.
admin-settings-key-security-protection-long-absolute-anomalies = Presupuesto absoluto de anomalías largo
admin-settings-help-security-protection-long-absolute-anomalies = Se activa cuando se acumula este número de anomalías seleccionadas, incluso si las solicitudes normales diluyen la proporción. Cero desactiva este disparador.
admin-settings-key-security-protection-long-ratio-percent = Proporción de anomalías larga (porcentaje)
admin-settings-help-security-protection-long-ratio-percent = Se activa cuando las anomalías seleccionadas alcanzan este porcentaje tras el número mínimo de muestras. Basta el disparador absoluto o el de proporción. Cero desactiva este disparador.
admin-settings-key-security-protection-normal-cap-per-bucket = Límite de la línea base normal por intervalo
admin-settings-help-security-protection-normal-cap-per-bucket = Acota la línea base normal aprendida en cada intervalo de la ventana móvil. Los presupuestos absolutos de anomalías independientes impiden que un relleno ilimitado de solicitudes válidas oculte el abuso.
admin-settings-key-security-protection-signals = Señales de evidencia adaptativa
admin-settings-help-security-protection-signals = Seleccione las observaciones fiables: malformadas, de tamaño excesivo, credenciales rechazadas, permiso denegado, plazo del cuerpo vencido u, opcionalmente, fallos en rutas de administración. El sondeo de rutas de administración excluye las respuestas de entidad no encontrada y los recursos estáticos; los códigos de estado genéricos, los rechazos por capacidad y las cadenas user-agent no establecen abuso.
admin-settings-key-security-protection-throttle-secs = Limitación temporal inicial (segundos)
admin-settings-help-security-protection-throttle-secs = El primer cruce del umbral inicia un plazo de limitación fijo. Las solicitudes durante ese plazo no lo amplían.
admin-settings-key-security-protection-block-secs = Bloqueo temporal reforzado (segundos)
admin-settings-help-security-protection-block-secs = Un cruce posterior del umbral antes de la recuperación puede escalar a este bloqueo acotado. No se crea ningún bloqueo permanente de cuenta.
admin-settings-key-security-protection-decay-secs = Intervalo de recuperación en calma (segundos)
admin-settings-help-security-protection-decay-secs = Tras este intervalo sin anomalías y el vencimiento de la acción actual, la escalada vuelve a su etapa inicial. La evidencia móvil caduca según sus ventanas configuradas.
admin-settings-key-security-protection-exempt-clients = Direcciones y redes de clientes exentas
admin-settings-help-security-protection-exempt-clients = Las direcciones IP exactas o redes CIDR eximen solo la dimensión de cliente. Los límites de cuenta verificados se siguen aplicando. Úselo con cuidado en redes NAT compartidas; las direcciones reenviadas solo son de confianza a través de los proxies configurados.
admin-settings-key-security-protection-exempt-accounts = IDs de cuenta verificados exentos
admin-settings-help-security-protection-exempt-accounts = Los ID exactos de cuentas de buzón verificados por el servidor eximen solo su dimensión de cuenta, nunca la del cliente de origen. Las direcciones de correo y los parámetros de cuenta aportados por quien llama no son evidencia de identidad.
admin-settings-key-security-protection-exempt-admins = IDs de administrador exentos
admin-settings-help-security-protection-exempt-admins = Los ID exactos de administradores autenticados eximen solo su dimensión de administrador. Los bloqueos existentes por fallos de inicio de sesión y los plazos estrictos de la sesión de administrador no cambian.
admin-settings-key-security-protection-storage-failure = Política ante fallos de almacenamiento de la aplicación
admin-settings-help-security-protection-storage-failure = Closed rechaza el trabajo protegido cuando no se puede comprobar un presupuesto de aplicación activado. Open lo permite temporalmente con una advertencia operativa acotada. Los fallos en modo solo observación nunca imponen una acción.
admin-settings-key-security-admin-probe-enabled = Protección contra sondeos de administración
admin-settings-help-security-admin-probe-enabled = Cuenta las solicitudes que buscan la consola de administración por red de cliente (dirección IPv4, IPv6 /64), ralentiza a un origen que insiste y después lo bloquea. Los errores tipográficos y los marcadores obsoletos quedan dentro de la tolerancia. Se cambia en Seguridad > Protección adaptativa.
admin-settings-key-security-admin-probe-grace-allowance = Tolerancia (fallos por ventana)
admin-settings-help-security-admin-probe-grace-allowance = Fallos ignorados por ventana. La puntuación disminuye de forma continua en este número de puntos por ventana.
admin-settings-key-security-admin-probe-window-secs = Ventana de tolerancia (segundos)
admin-settings-help-security-admin-probe-window-secs = La ventana a la que se aplica la tolerancia.
admin-settings-key-security-admin-probe-block-after = Fallos por encima de la tolerancia antes del bloqueo
admin-settings-help-security-admin-probe-block-after = Fallos por encima de la tolerancia que inician un bloqueo. Los fallos intermedios se responden tras el retardo de tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Retardo de tarpit (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Retardo que se añade a cada fallo contado por encima de la tolerancia. 0 desactiva el tarpit. Las demás solicitudes nunca se retrasan.
admin-settings-key-security-admin-probe-block-secs = Primer bloqueo (segundos)
admin-settings-help-security-admin-probe-block-secs = Duración del primer bloqueo. Un origen bloqueado recibe la misma respuesta para cada ruta.
admin-settings-key-security-admin-probe-max-block-secs = Bloqueo más largo (segundos)
admin-settings-help-security-admin-probe-max-block-secs = Un bloqueo repetido duplica la duración anterior, hasta este límite.
admin-settings-key-security-admin-probe-repeat-window-secs = Ventana de repetición (segundos)
admin-settings-help-security-admin-probe-repeat-window-secs = Un bloqueo que empieza dentro de este tiempo tras terminar el anterior cuenta como repetición.
admin-settings-key-security-admin-probe-block-status = Estado de la respuesta de bloqueo
admin-settings-help-security-admin-probe-block-status = Estado HTTP que se envía a un origen bloqueado, con Retry-After: 429 o 403.
admin-settings-key-security-admin-probe-signals = Fallos contados
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (un 404 a la profundidad de ruta de la consola), admin-signin-failed y admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Rutas de sondeo
admin-settings-help-security-admin-probe-probe-paths = Rutas que piden los escáneres, como /wp-admin o /.env. Una ruta que contenga una de ellas como segmentos completos cuenta como fallo. La ruta propia de la consola nunca.
admin-settings-key-security-admin-probe-allowlist = Redes que nunca se bloquean
admin-settings-help-security-admin-probe-allowlist = Direcciones IP o redes CIDR (oficina, VPN) que nunca se cuentan ni se bloquean.
admin-settings-key-security-admin-probe-audit-ip = Dirección de origen en el registro de auditoría
admin-settings-help-security-admin-probe-audit-ip = masked muestra IPv4 /24 e IPv6 /48; full muestra la dirección. La línea de fail2ban siempre lleva la dirección completa.
admin-settings-key-security-admin-probe-fail2ban-log = Línea de registro de fail2ban al bloquear
admin-settings-help-security-admin-probe-fail2ban-log = Escribe una línea mailwoman[admin-probe] con rhost=<address> cuando se bloquea un origen.
admin-settings-key-appearance-custom-css-enabled = Permitir CSS de apariencia personal
admin-settings-help-appearance-custom-css-enabled = Permite estilos visuales restringidos en el espacio de trabajo del propietario. Desactivado por defecto; los bloqueos de apariencia impuestos del buzón suspenden estos estilos.
admin-settings-key-appearance-custom-js-enabled = Permitir scripts de apariencia personales
admin-settings-help-appearance-custom-js-enabled = Permite scripts aislados que solo pueden solicitar cambios de apariencia. Desactivado por defecto; los scripts no pueden acceder al correo, a los datos de la cuenta ni al DOM de la aplicación.
admin-settings-key-mail-modifier-selection = Selección de mensajes con teclas modificadoras
admin-settings-help-mail-modifier-selection = Permite la selección de rango con Ctrl, Comando y Mayús. Bloquee este valor para imponerlo; los controles de selección visibles siguen disponibles.
admin-settings-key-mail-drag-to-folder = Arrastrar mensajes a carpetas
admin-settings-help-mail-drag-to-folder = Permite arrastrar mensajes, vinculado a la cuenta, a carpetas reales. Esto no concede permiso de escritura en el buzón; el servidor sigue autorizando cada movimiento.
admin-settings-key-admin-allow-last-superadmin-loss = Permitir perder al último superadministrador
admin-settings-help-admin-allow-last-superadmin-loss = Desactivado por defecto. Activarlo permite que cambios posteriores eliminen el último acceso de superadministrador. Un superadministrador completo debe confirmar la consecuencia con una autenticación reciente.
admin-settings-last-superadmin-title = ¿Permitir perder todo el acceso de superadministrador?
admin-settings-last-superadmin-body = Cambios de cuenta posteriores pueden dejar a nadie con capacidad para administrar este despliegue. La recuperación requeriría administración local del despliegue. Escriba ALLOW LAST SUPERADMIN LOSS para activar esta excepción.
admin-settings-last-superadmin-confirm = Permitir la pérdida de acceso
admin-settings-key-mail-folder-counts = Contadores de carpetas
admin-settings-help-mail-folder-counts = Elija junto a las carpetas: ocultos, no leídos, no leídos solo mientras haya correo no leído, total, o ambos contadores. Defina valores predeterminados globales, de dominio o de usuario y bloquee la elección cuando sea necesario. Se aplica a cada buzón vinculado de una cuenta.
admin-settings-key-mail-folder-count-scope = Carpetas con contadores
admin-settings-help-mail-folder-count-scope = Todas las carpetas, o solo las carpetas principales (Bandeja de entrada, Borradores, Enviados, Archivo, Correo no deseado, Papelera, Bandeja de salida y otras carpetas de uso especial). Con solo las carpetas principales, las carpetas personalizadas y sus subcarpetas no muestran contador. Se aplica a cada buzón vinculado de una cuenta.
admin-settings-key-mail-compact-folder-badges = Insignias secundarias de carpetas compactas
admin-settings-help-mail-compact-folder-badges = Muestra insignias más pequeñas fuera de la bandeja de entrada. Los usuarios pueden elegir su propio tamaño salvo que este ajuste esté bloqueado.
admin-settings-key-mail-focused-inbox = Bandeja de entrada prioritaria
admin-settings-help-mail-focused-inbox = Divide la bandeja de entrada en las pestañas Prioritarios y Otros; los boletines y los remitentes automáticos van a Otros. Desactivado por defecto. Los usuarios pueden activarlo o desactivarlo en Ajustes > Carpetas y barra lateral, salvo que este ajuste esté bloqueado.
admin-settings-key-mail-account-unread-badge = Contador de no leídos junto al nombre del buzón
admin-settings-help-mail-account-unread-badge = Muestra el número de no leídos de la bandeja de entrada junto a la dirección o el alias con sesión iniciada en la barra lateral. Desactivado por defecto. Los contadores de los demás buzones en el menú de buzones se muestran siempre. Se aplica a cada buzón vinculado de una cuenta.
admin-settings-key-privacy-remote-image-downloads = Descargas manuales de imágenes remotas
admin-settings-key-privacy-remote-images-toast = Notificación de carga de imágenes
admin-settings-help-privacy-remote-images-toast = Muestra una notificación en el lector mientras las imágenes remotas de un mensaje se cargan a través del proxy de privacidad, e indica cuándo se han cargado o cuántas no se pudieron cargar. Activado por defecto. Solo cambia lo que muestra el lector; qué imágenes se cargan lo sigue decidiendo la política de contenido remoto.
admin-settings-help-privacy-remote-image-downloads = Permite la descarga explícita de una imagen referenciada por el mensaje del propietario a través del proxy de privacidad protegido. Desactivado por defecto; nunca abre directamente la URL externa.
admin-settings-key-mail-resume-tabs = Reanudar pestañas del espacio de trabajo
admin-settings-help-mail-resume-tabs = Restaura tras recargar las referencias de pestañas propias de la cuenta. Desactivarlo borra el registro local del espacio de trabajo sin eliminar borradores.
admin-settings-key-metrics-default-range-seconds = Rango predeterminado del historial de solicitudes
admin-settings-help-metrics-default-range-seconds = Segundos que se muestran cuando una solicitud de historial no especifica un rango. Limitado por la conservación y el rango máximo.
admin-settings-key-metrics-max-range-seconds = Rango máximo del historial de solicitudes
admin-settings-help-metrics-max-range-seconds = Máximo de segundos por consulta de historial. La conservación sigue siendo un límite adicional.
admin-settings-key-contacts-favorites-first = Favoritos primero en las sugerencias
admin-settings-help-contacts-favorites-first = Lista los contactos favoritos antes que otras coincidencias en las sugerencias de destinatarios. Solo cambia el orden; no se oculta ninguna sugerencia. Las sustituciones por usuario y por dominio pueden bloquearse.
admin-settings-key-contacts-sender-icons = Iconos de remitente incluidos
admin-settings-help-contacts-sender-icons = Muestra iconos incluidos localmente para los dominios de remitente reconocidos, con iniciales generadas para los demás. Las fotos de contactos tienen prioridad. Los iconos no verifican la identidad de un remitente. No se contacta ningún servicio de logotipos externo; las sustituciones por usuario y por dominio pueden bloquearse.
admin-settings-key-mail-brand-avatars = Logotipos de marca junto a remitentes verificados
admin-settings-help-mail-brand-avatars = Muestra un logotipo de marca incluido localmente junto al remitente en el lector, solo cuando el mensaje superó DKIM y una comprobación DMARC alineada para el dominio propio de esa marca. Los mensajes que no superan esas comprobaciones o carecen de ellas conservan el avatar normal. No se obtiene ningún logotipo del remitente ni de un servicio de logotipos.
admin-settings-key-mail-draft-save-mode = Modo de guardado de borradores de la cuenta
admin-settings-help-mail-draft-save-mode = Guardar tras una pausa de escritura, periódicamente o solo bajo demanda. Guardar requiere una carpeta Borradores compatible en el servidor de origen; las escrituras inciertas nunca se reintentan automáticamente.
admin-settings-key-mail-draft-debounce-ms = Pausa de escritura de borradores (milisegundos)
admin-settings-help-mail-draft-debounce-ms = Espera de 500 a 10000 milisegundos tras editar antes de guardar en modo en tiempo real.
admin-settings-key-mail-draft-interval-secs = Intervalo de guardado de borradores (segundos)
admin-settings-help-mail-draft-interval-secs = Guarda los borradores editados cada 5 a 3600 segundos en modo periódico. Los borradores sin cambios no se vuelven a escribir.
admin-settings-key-login-remember-signins = Recordar las direcciones de inicios de sesión correctos
admin-settings-help-login-remember-signins = Conserva en este dispositivo hasta diez nombres de usuario o direcciones de correo usados con éxito para la lista desplegable de inicio de sesión. Nunca guarda contraseñas ni sesiones. Desactivarlo elimina las direcciones guardadas de este servidor cuando un dispositivo cargue la página de inicio de sesión o complete un inicio de sesión; no controla el autorrelleno del propio navegador.
admin-settings-key-login-show-remembered-hint = Mostrar la explicación de direcciones guardadas
admin-settings-help-login-show-remembered-hint = Muestra texto explicativo en la lista desplegable de inicios de sesión anteriores. Oculto por defecto. Solo se aplica cuando está activado recordar los inicios de sesión correctos.
admin-settings-key-mail-show-from = Mostrar el campo De del editor
admin-settings-help-mail-show-from = Muestra De por defecto en los editores nuevos. Los usuarios pueden mostrarlo para un mensaje; solo las direcciones autorizadas pueden enviar.
admin-settings-key-accounts-linking-enabled = Permitir cuentas de buzones vinculadas
admin-settings-help-accounts-linking-enabled = Desactivado por defecto. La desactivación global bloquea todo acceso vinculado; ambas cuentas deben cumplir además su política de dominio y de cuenta. Vincular requiere la autenticación real de la cuenta secundaria y una prueba reciente de la principal.
admin-settings-key-accounts-use-linked = Usar buzones vinculados
admin-settings-help-accounts-use-linked = Exclusión voluntaria del usuario del acceso vinculado. No puede anular una desactivación administrativa. Los datos del buzón se conservan cuando se desactiva el acceso.
admin-settings-key-accounts-max-linked = Máximo de buzones vinculados
admin-settings-help-accounts-max-linked = Máximo de buzones secundarios verificados por identidad principal, de 1 a 16. Por defecto 5.
admin-settings-key-accounts-store-linked-credentials = Mantener el acceso a buzones vinculados
admin-settings-help-accounts-store-linked-credentials = Mantiene sellada en el servidor la contraseña de cada buzón vinculado para que su acceso se renueve por sí solo, entre sesiones y reinicios. Nunca se conserva para cuentas de acceso cero ni para buzones con verificación en dos pasos. Desactivarlo borra las contraseñas guardadas; las vinculaciones se renuevan entonces iniciando sesión de nuevo. Activado por defecto.
admin-settings-key-privacy-strip-mailer-headers = Quitar las cabeceras de agente de envío salientes
admin-settings-help-privacy-strip-mailer-headers = Quita las cabeceras X-Mailer, User-Agent y X-MimeOLE de nivel superior del correo SMTP y EWS nativo. Los mensajes adjuntos y los contenidos de informe permanecen intactos. Los envíos mediante el proxy JMAP de origen no están cubiertos.
admin-settings-key-images-trusted-hosts = Hosts de imágenes de confianza
admin-settings-help-images-trusted-hosts = Hasta 100 hosts de imágenes, y sus subdominios, cuyas imágenes remotas se cargan sin autorización por mensaje cuando el contenido remoto está en preguntar. Nunca anula el modo de bloqueo; un host bloqueado sigue bloqueado.
admin-settings-key-images-blocked-hosts = Hosts de imágenes bloqueados
admin-settings-help-images-blocked-hosts = Hasta 100 hosts de imágenes, y sus subdominios, cuyas imágenes remotas nunca se cargan en ningún modo. Prevalece sobre los hosts de confianza y sobre cualquier autorización.
admin-settings-key-links-trusted-domains = Dominios de enlaces de confianza
admin-settings-help-links-trusted-domains = Hasta 100 dominios exactos cuyos enlaces se abren desde Revisar enlaces sin el paso de destino. Sin comodines, esquemas, puertos ni direcciones IP. Los enlaces HTTP sin cifrar siempre se revisan. Bloquee este ajuste para que los usuarios no cambien la lista.
admin-settings-key-links-trusted-subdomains = Confiar en los subdominios de los dominios de enlaces de confianza
admin-settings-help-links-trusted-subdomains = Desactivado: solo es de confianza el dominio exacto. Activado: también lo son los nombres que terminan en .<domain>. La coincidencia se hace por etiquetas completas, de modo que example.com nunca confía en evilexample.com.
admin-settings-key-cap-compose-rich-text = Redacción en texto enriquecido
admin-settings-help-cap-compose-rich-text = Si los usuarios pueden escribir mensajes con formato (HTML). Desactivado envía solo texto sin formato.
admin-settings-key-cap-compose-attachments = Adjuntos en mensajes nuevos
admin-settings-help-cap-compose-attachments = Si los usuarios pueden adjuntar archivos a los mensajes que escriben. Las imágenes insertadas en el cuerpo no se cuentan.
admin-settings-key-cap-compose-share-links = Enlaces para compartir archivos grandes
admin-settings-help-cap-compose-share-links = Si los usuarios pueden enviar un archivo grande como enlace compartido de Nextcloud en lugar de como adjunto.
admin-settings-key-cap-compose-scheduled-send = Envío programado
admin-settings-help-cap-compose-scheduled-send = Si los usuarios pueden programar un mensaje para enviarlo más tarde.
admin-settings-key-cap-compose-read-receipts = Solicitar confirmaciones de lectura
admin-settings-help-cap-compose-read-receipts = Si los usuarios pueden pedir a los destinatarios una confirmación de lectura.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-cap-signatures = Firmas
admin-settings-help-cap-signatures = Si los usuarios pueden crear y editar sus propias firmas.
admin-settings-key-cap-identities = Identidades de envío
admin-settings-help-cap-identities = Si los usuarios pueden añadir y editar las identidades con las que envían.
admin-settings-key-cap-keys = Claves OpenPGP y S/MIME
admin-settings-help-cap-keys = Si los usuarios disponen del módulo de Claves y pueden firmar y cifrar mensajes.
admin-settings-key-cap-links-trusted = Editar dominios de enlaces de confianza
admin-settings-help-cap-links-trusted = Si los usuarios pueden cambiar su propia lista de dominios de enlaces de confianza. La lista del administrador se sigue aplicando.
admin-settings-key-cap-zoom = Zoom del lector
admin-settings-help-cap-zoom = Si los usuarios pueden hacer zoom en el lector de mensajes.
admin-settings-key-cap-mail-focused-inbox = Bandeja de entrada prioritaria
admin-settings-help-cap-mail-focused-inbox = Si los usuarios pueden activar las pestañas Prioritarios y Otros de la bandeja de entrada.
admin-settings-key-cap-views-events = Vista de eventos
admin-settings-help-cap-views-events = Si los usuarios disponen de la vista de eventos del correo de calendario.
admin-settings-key-cap-views-attachments = Vista de adjuntos
admin-settings-help-cap-views-attachments = Si los usuarios disponen de la vista de adjuntos. Los mensajes siguen accesibles mediante la búsqueda.
admin-settings-key-cap-search-operators = Operadores de búsqueda
admin-settings-help-cap-search-operators = Si los usuarios pueden buscar con operadores como from:, subject: o has:attachment.
admin-settings-key-cap-saved-searches = Búsquedas guardadas
admin-settings-help-cap-saved-searches = Si los usuarios pueden guardar búsquedas.
admin-settings-key-cap-vacation = Respuesta automática de vacaciones
admin-settings-help-cap-vacation = Si los usuarios pueden establecer una respuesta automática de vacaciones.
admin-settings-key-cap-labels = Etiquetas
admin-settings-help-cap-labels = Si los usuarios pueden poner etiquetas a los mensajes.
admin-settings-key-cap-snooze = Posponer
admin-settings-help-cap-snooze = Si los usuarios pueden posponer mensajes. Finalizar un aplazamiento sigue siendo posible.
admin-settings-key-cap-follow-up = Recordatorios de seguimiento
admin-settings-help-cap-follow-up = Si los usuarios pueden establecer recordatorios de seguimiento en los mensajes. Quitar uno sigue siendo posible.
admin-settings-key-cap-masked = Direcciones enmascaradas
admin-settings-help-cap-masked = Si los usuarios pueden crear y gestionar direcciones enmascaradas.
admin-settings-key-cap-directory = Búsqueda en el directorio
admin-settings-help-cap-directory = Si los usuarios pueden buscar en el directorio de la organización.
admin-settings-key-cap-mcp = Acceso MCP
admin-settings-help-cap-mcp = Si las claves y los tokens del usuario pueden usar el punto de conexión MCP.
admin-settings-key-cap-appearance-custom = Personalización de la apariencia
admin-settings-help-cap-appearance-custom = Si los usuarios pueden cambiar su tema, acento, colores personalizados y de insignia, densidad y fuente. Desactivado mantiene la apariencia del administrador.
admin-settings-key-cap-appearance-custom-code = CSS y scripts personalizados
admin-settings-help-cap-appearance-custom-code = Si los usuarios pueden guardar su propio CSS y scripts de apariencia. Los interruptores de despliegue de Apariencia se siguen aplicando por encima de este.
admin-settings-key-cap-avatars = Avatares del buzón
admin-settings-help-cap-avatars = Si los usuarios pueden cambiar los avatares del buzón.
admin-settings-key-cap-shortcuts = Atajos de teclado personalizados
admin-settings-help-cap-shortcuts = Si los usuarios pueden cambiar los atajos de teclado. Los atajos predeterminados siguen funcionando.
admin-settings-key-cap-analytics = Analítica del producto
admin-settings-help-cap-analytics = Si se pide a los usuarios su consentimiento para la analítica y pueden enviar eventos de uso.
admin-settings-key-cap-apps-native = Aplicaciones de escritorio y móviles
admin-settings-help-cap-apps-native = Si los usuarios pueden usar las aplicaciones de escritorio y móviles.
admin-settings-key-access-protocols = Protocolos de acceso al correo
admin-settings-help-access-protocols = Con qué protocolos de servidor de correo pueden iniciar sesión estos usuarios: imap, pop3, ews. Los inicios de sesión a través de un servidor JMAP siguen a ese servidor.
admin-settings-key-limit-linked-mailboxes = Buzones vinculados por usuario
admin-settings-help-limit-linked-mailboxes = Cuántos buzones puede vincular un usuario, por debajo del máximo del servidor. 0 significa que solo se aplica el máximo del servidor.
admin-settings-key-login-show-account-help-notice = Mostrar aviso de ayuda de cuenta al iniciar sesión
admin-settings-help-login-show-account-help-notice = Muestra una breve explicación tras un inicio de sesión fallido. Oculto por defecto; el aviso no revela si una cuenta existe o está desactivada.
admin-settings-key-appearance-leetspeak = Estilo de texto leetspeak
admin-settings-help-appearance-leetspeak = Muestra las etiquetas de la interfaz y el texto de los mensajes de solo lectura en leetspeak. El mensaje original y el texto editable no cambian. Los usuarios pueden elegir texto normal salvo que este ajuste esté bloqueado.
admin-settings-key-mail-cache-enabled = Activar la caché de sesión
admin-settings-help-mail-cache-enabled = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-mailboxes = Guardar en caché las listas de mensajes de las carpetas
admin-settings-help-mail-cache-mailboxes = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-messages = Guardar en caché los mensajes abiertos
admin-settings-help-mail-cache-messages = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-fresh-seconds = Intervalo de frescura (segundos)
admin-settings-help-mail-cache-fresh-seconds = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-max-age-seconds = Antigüedad máxima de la caché (segundos)
admin-settings-help-mail-cache-max-age-seconds = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-max-entries = Máximo de entradas en caché
admin-settings-help-mail-cache-max-entries = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-max-bytes = Presupuesto de memoria (bytes)
admin-settings-help-mail-cache-max-bytes = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-prefetch-enabled = Activar la precarga especulativa
admin-settings-help-mail-cache-prefetch-enabled = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-hover-enabled = Precargar al señalar una carpeta
admin-settings-help-mail-cache-hover-enabled = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-focus-enabled = Precargar cuando una carpeta recibe el foco del teclado
admin-settings-help-mail-cache-focus-enabled = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-dwell-ms = Retardo del puntero y del foco (milisegundos)
admin-settings-help-mail-cache-dwell-ms = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-idle-enabled = Precargar carpetas durante la inactividad
admin-settings-help-mail-cache-idle-enabled = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-idle-delay-ms = Retardo de inactividad (milisegundos)
admin-settings-help-mail-cache-idle-delay-ms = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-max-concurrency = Máximo de precargas simultáneas
admin-settings-help-mail-cache-max-concurrency = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-max-queued = Máximo de precargas en cola
admin-settings-help-mail-cache-max-queued = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-preload-messages = Permitir la precarga especulativa de cuerpos de mensajes
admin-settings-help-mail-cache-preload-messages = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-excluded-folders = Carpetas excluidas de la precarga
admin-settings-help-mail-cache-excluded-folders = Conserva las carpetas y los mensajes visitados recientemente en la memoria de esta pestaña. El contenido en caché se actualiza en segundo plano. Cerrar la pestaña o cerrar sesión lo borra. La precarga nunca marca el correo como leído ni carga imágenes remotas.
admin-settings-key-mail-cache-refresh-on-open = Actualizar el contenido en caché al abrir
admin-settings-help-mail-cache-refresh-on-open = Actualizar el contenido en caché al abrir
admin-settings-key-mail-templates-enabled = Plantillas de respuesta
admin-settings-help-mail-templates-enabled = Activa las plantillas de respuesta de texto sin formato, personales y compartidas, en el editor. Los usuarios pueden excluirse salvo que este ajuste esté bloqueado.
admin-settings-key-mail-templates-allow-personal = Permitir plantillas personales
admin-settings-help-mail-templates-allow-personal = Permite a los usuarios crear sus propias plantillas de respuesta. Las plantillas compartidas las gestionan por separado los administradores.
admin-settings-key-mail-templates-shared = Plantillas de respuesta compartidas
admin-settings-help-mail-templates-shared = Gestiona respuestas reutilizables de texto sin formato para este ámbito. Las plantillas nunca definen destinatarios ni envían mensajes.
admin-settings-key-mail-templates-personal = Plantillas de respuesta personales
admin-settings-help-mail-templates-personal = Respuestas reutilizables personales de texto sin formato. Crear o editar requiere que las plantillas de respuesta y las plantillas personales estén activadas.
admin-settings-key-mail-compose-window = Ventana del editor
admin-settings-help-mail-compose-window = Abre los borradores nuevos en una pestaña del espacio de trabajo o en una ventana aparte. Las ventanas emergentes bloqueadas recurren a una pestaña.
admin-settings-key-mail-reply-copy-to-original = Mantener las respuestas en su carpeta original
admin-settings-help-mail-reply-copy-to-original = Guarda una copia de la respuesta enviada en la carpeta del mensaje original donde sea compatible.
admin-settings-key-mail-compose-check-invalid-recipients = Comprobar destinatarios no válidos antes de enviar
admin-settings-help-mail-compose-check-invalid-recipients = Qué ocurre cuando se envía un mensaje a una dirección no válida: permitirlo, avisar, avisar y recordar la elección, o rechazarlo.
admin-settings-key-mail-compose-check-missing-attachment = Comprobar si falta un adjunto antes de enviar
admin-settings-help-mail-compose-check-missing-attachment = Qué ocurre cuando el texto menciona un adjunto pero no hay ninguno adjuntado: permitirlo, avisar, avisar y recordar la elección, o rechazarlo.
admin-settings-key-mail-compose-check-empty-subject = Comprobar si el asunto está vacío antes de enviar
admin-settings-help-mail-compose-check-empty-subject = Qué ocurre cuando un mensaje no tiene asunto: permitirlo, avisar, avisar y recordar la elección, o rechazarlo.
admin-settings-key-mail-compose-auto-chip-recipients = Completar automáticamente los destinatarios escritos
admin-settings-help-mail-compose-auto-chip-recipients = Convierte una dirección escrita en una etiqueta de destinatario cuando la sigue un espacio, una coma o un punto y coma.
admin-settings-key-mail-compose-header-collapse = Cabecera del editor
admin-settings-help-mail-compose-header-collapse = Cómo se comportan las filas Para, Cc, Cco y Asunto del editor: expandidas, plegadas en un resumen de una línea, o auto (plegadas mientras se escribe el cuerpo, abiertas cuando un campo tiene el foco). Los usuarios pueden plegarlas o desplegarlas ellos mismos salvo que esté bloqueado.
admin-settings-key-mail-compose-recipient-separator = Separador de destinatarios
admin-settings-help-mail-compose-recipient-separator = El separador que se inserta tras un destinatario completado: una coma o un punto y coma.
admin-settings-key-mail-reply-from-mode = Identidad usada para respuestas y reenvíos
admin-settings-help-mail-reply-from-mode = Desde qué identidad se envía una respuesta o un reenvío: aquella a la que iba dirigido el original, la predeterminada, un selector, o la seleccionada actualmente.
admin-settings-key-mail-forward-mode = Formato de reenvío predeterminado
admin-settings-help-mail-forward-mode = Reenviar el texto en línea o adjuntar el mensaje original como archivo .eml.
admin-settings-key-mail-show-go-to-top = Mostrar el botón Ir arriba
admin-settings-help-mail-show-go-to-top = Muestra en la lista de mensajes un botón Ir arriba que vuelve al primer mensaje. Desactivado por defecto.
admin-settings-key-mail-show-create-folder-button = Mostrar el botón Crear carpeta
admin-settings-help-mail-show-create-folder-button = Muestra en la barra lateral, junto a Redactar, un botón Crear carpeta solo con icono. Desactivado por defecto.
admin-settings-key-security-protection-user-agent-mode = Modo de observación del user-agent
admin-settings-help-security-protection-user-agent-mode = enforce por defecto: las cabeceras de bots reconocidos seleccionadas se bloquean de inmediato cuando está activado el bloqueo de bots en la primera solicitud. observe registra las decisiones sin rechazo; off desactiva esta dimensión. Las afirmaciones de cabecera no son identidades verificadas.
admin-settings-key-security-protection-user-agent-block-bots = Bloquear bots reconocidos en la primera solicitud
admin-settings-help-security-protection-user-agent-block-bots = true por defecto. Las cabeceras seleccionadas de bots de búsqueda, de IA, de automatización y personalizados provocan un bloqueo HTTP 429 temporal en la primera solicitud en modo enforce. observe solo lo registra; false restablece los presupuestos de solicitudes anteriores. Las cabeceras ausentes o poco habituales, el ámbito y las excepciones conservan su comportamiento actual.
admin-settings-key-security-protection-user-agent-categories = Categorías de user-agent
admin-settings-help-security-protection-user-agent-categories = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-security-protection-user-agent-weak-requests = Umbral de solicitudes de señal débil
admin-settings-help-security-protection-user-agent-weak-requests = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protection-user-agent-crawler-requests = Umbral de solicitudes de rastreadores
admin-settings-help-security-protection-user-agent-crawler-requests = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-security-protection-user-agent-window-secs = Ventana de observación del user-agent (segundos)
admin-settings-help-security-protection-user-agent-window-secs = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-security-protection-user-agent-max-action = Respuesta máxima al user-agent
admin-settings-help-security-protection-user-agent-max-action = Escalada máxima para evidencia débil o de solicitudes repetidas, limitación por defecto. Este tope no restringe los bloqueos en la primera solicitud de las cabeceras de bots reconocidos seleccionadas.
admin-settings-key-security-protection-user-agent-scope = Ámbito de la política de user-agent
admin-settings-help-security-protection-user-agent-scope = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-security-protection-user-agent-adaptive = Observaciones adaptativas del user-agent
admin-settings-help-security-protection-user-agent-adaptive = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-security-protection-user-agent-patterns = Patrones de user-agent
admin-settings-help-security-protection-user-agent-patterns = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-security-protection-user-agent-exceptions = Excepciones de user-agent
admin-settings-help-security-protection-user-agent-exceptions = Configura esta parte de la observación opcional del user-agent. El texto user-agent aportado por el cliente es una señal débil, no una prueba de identidad ni de abuso.
admin-settings-key-mail-reader-unsafe-html-mode = HTML de mensaje potencialmente inseguro
admin-settings-help-mail-reader-unsafe-html-mode = ask ofrece por defecto un botón de HTML saneado por mensaje. strict muestra solo texto seguro; sanitized muestra automáticamente el HTML saneado. El contenido activo y los recursos remotos automáticos siguen bloqueados. Bloquee el valor para imponer esta política.
admin-settings-key-reader-attachments-density = Densidad de los adjuntos
admin-settings-help-reader-attachments-density = Botones de archivo compactos por defecto, o vistas previas cómodas. Controla la presentación cuando los adjuntos de un mensaje están expandidos.
admin-settings-key-cap-compose-open-tracking = Permitir el seguimiento de apertura de mensajes
admin-settings-help-cap-compose-open-tracking = Si esta cuenta puede registrar las aperturas de mensajes. Requiere una URL pública configurada; las cuentas desactivadas no pueden crear ni vincular tokens de seguimiento.
admin-settings-key-mail-open-tracking = Seguir las aperturas por defecto
admin-settings-help-mail-open-tracking = Elección de seguimiento por mensaje predeterminada para los mensajes con formato aptos. Los usuarios pueden anularla salvo que esté bloqueada; desactivar el permiso de seguimiento siempre prevalece.
admin-settings-key-cap-reading-original-html = Permitir la representación del HTML original
admin-settings-help-cap-reading-original-html = Si los usuarios pueden optar por la vista HTML original en mensajes concretos. El renderizador saneado predeterminado y la seguridad del marco aislado se mantienen.
admin-settings-key-mail-reader-allow-original-html = Ofrecer la vista HTML original
admin-settings-help-mail-reader-allow-original-html = Muestra la opción por mensaje de usar el HTML original cuando el permiso de lectura lo permite. Desactivado por defecto; no cambia el renderizador de mensajes predeterminado.
admin-settings-key-privacy-tracker-override-allowed = Ofrecer el consentimiento de rastreadores por mensaje
admin-settings-help-privacy-tracker-override-allowed = Muestra la acción Permitir rastreadores en la barra de privacidad del mensaje. Disponible por defecto, pero nunca activa sin una autorización explícita. Una política bloqueada de bloqueo de rastreadores sigue prohibiendo la excepción.
admin-settings-key-privacy-max-concurrent-global = Obtenciones de imágenes en paralelo globales
admin-settings-help-privacy-max-concurrent-global = Máximo de solicitudes de red de imágenes activas en este proceso del servidor. Rango 1–256; por defecto 128. Los límites por mensaje y por cuenta se siguen aplicando.
admin-settings-key-privacy-max-concurrent-pipeline = Canalizaciones de procesamiento de imágenes simultáneas
admin-settings-help-privacy-max-concurrent-pipeline = Máximo de imágenes admitidas juntas en la obtención, la decodificación y la publicación en caché. Rango 1–256; por defecto 128. Los límites de red y de decodificador se aplican por separado.
admin-settings-key-privacy-max-concurrent-decodes = Decodificadores de imágenes en paralelo
admin-settings-help-privacy-max-concurrent-decodes = Máximo de decodificaciones de imágenes que se ejecutan a la vez. Rango 1–16; por defecto 10. Los valores más altos usan más CPU y memoria.
admin-settings-key-privacy-max-queued-requests = Solicitudes de imágenes en espera
admin-settings-help-privacy-max-queued-requests = Máximo de solicitudes que esperan capacidad de red o de procesamiento de imágenes. Rango 16–16384; por defecto 2048. El desbordamiento o la caducidad en cola siguen siendo reintentables.
admin-settings-key-privacy-rate-burst = Ráfaga permitida de solicitudes de imágenes
admin-settings-help-privacy-rate-burst = Tokens de obtención por cuenta iniciales y máximos, repuestos a la frecuencia de solicitudes de imágenes configurada. Rango 1–12000; por defecto 1024. Los éxitos en caché no consumen tokens.
admin-settings-key-privacy-image-rate-mode = Dimensionamiento de la frecuencia de solicitudes de imágenes
admin-settings-help-privacy-image-rate-mode = Automático escala la ráfaga y la reposición por cuenta según la capacidad de las réplicas, la carga de la cola y el número de imágenes del mensaje, dentro de los mínimos y máximos indicados abajo. Manual usa exactamente los ajustes de ráfaga permitida y frecuencia de solicitudes. Automático por defecto. Las solicitudes que superan la asignación esperan un token hasta el plazo de la cola de obtención; la autenticación y las autorizaciones se comprueban primero.
admin-settings-key-privacy-image-rate-burst-floor = Mínimo de la ráfaga automática
admin-settings-help-privacy-image-rate-burst-floor = La ráfaga más baja por cuenta en el modo de frecuencia automático. Rango 1–12000; por defecto 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Máximo de la ráfaga automática
admin-settings-help-privacy-image-rate-burst-ceiling = La ráfaga más alta por cuenta en el modo de frecuencia automático. Rango 1–12000; por defecto 2048.
admin-settings-key-privacy-image-rate-refill-floor = Mínimo de la reposición automática
admin-settings-help-privacy-image-rate-refill-floor = La reposición más baja por cuenta, en tokens de obtención por minuto, en el modo de frecuencia automático. Rango 10–60000; por defecto 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Máximo de la reposición automática
admin-settings-help-privacy-image-rate-refill-ceiling = La reposición más alta por cuenta, en tokens de obtención por minuto, en el modo de frecuencia automático. Rango 10–60000; por defecto 6000.
admin-settings-key-privacy-image-cache-mode = Dimensionamiento de la caché de imágenes
admin-settings-help-privacy-image-cache-mode = Automático dimensiona la caché con la memoria restante tras las reservas de decodificador y canalización, y alarga la vigencia mientras la tasa de aciertos es alta y hay memoria libre. Manual usa exactamente los ajustes de tamaño de caché, número de entradas y vigencia. Automático por defecto.
admin-settings-key-privacy-image-cache-bytes-floor = Mínimo del tamaño de caché automático
admin-settings-help-privacy-image-cache-bytes-floor = La caché de imágenes más pequeña en bytes en modo automático. Rango 0–1073741824; por defecto 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Máximo del tamaño de caché automático
admin-settings-help-privacy-image-cache-bytes-ceiling = La caché de imágenes más grande en bytes en modo automático. Rango 0–1073741824; por defecto 268435456.
admin-settings-key-privacy-cache-entries = Límite de entradas de la caché de imágenes
admin-settings-help-privacy-cache-entries = Máximo de imágenes en caché; el número fijo en modo manual y el máximo en modo automático. Rango 16–65536; por defecto 4096.
admin-settings-key-privacy-cache-entry-bytes = Imagen en caché más grande
admin-settings-help-privacy-cache-entry-bytes = La mayor imagen recodificada que se conserva en la caché, en bytes. Las imágenes mayores se sirven pero no se conservan. Rango 65536–16777216; por defecto 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Mínimo de la vigencia de caché automática
admin-settings-help-privacy-image-cache-ttl-floor-secs = La vigencia de caché más corta en segundos en modo automático, usada con una tasa de aciertos baja o bajo presión de memoria. Rango 0–86400; por defecto 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Máximo de la vigencia de caché automática
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = La vigencia de caché más larga en segundos en modo automático, usada con una tasa de aciertos alta y memoria libre. Rango 0–86400; por defecto 21600.
admin-settings-key-privacy-cache-ttl-secs = Vigencia de caché de imágenes correctas
admin-settings-help-privacy-cache-ttl-secs = Segundos que las imágenes decodificadas siguen siendo reutilizables en este proceso. Rango 0–86400; por defecto 3600. Cero desactiva la reutilización de imágenes correctas; la autorización siempre se comprueba primero.
admin-settings-key-privacy-image-latency-tracking = Seguimiento de la velocidad del proxy de imágenes
admin-settings-help-privacy-image-latency-tracking = Registra cuánto tarda cada imagen que pasa por el proxy (mediana, p95, p99 y el tiempo dedicado a esperar, obtener y decodificar) como recuentos en intervalos de tiempo fijos en memoria. No se almacenan URL, hosts ni cuentas. Activado por defecto; el historial se restablece al reiniciar y es propio de cada proceso del servidor. Desactivarlo detiene los registros nuevos; la sección Velocidad sigue mostrando lo registrado.
admin-settings-key-privacy-negative-cache-ttl-secs = Vigencia de caché de imágenes fallidas
admin-settings-help-privacy-negative-cache-ttl-secs = Segundos que se recuerdan los fallos de origen aptos para evitar solicitudes repetidas. Rango 0–300; por defecto 10. Cero desactiva la reutilización de fallos. Un reintento explícito borra los fallos afectados.
admin-settings-key-privacy-image-retry-attempts = Reintentos automáticos de imágenes
admin-settings-help-privacy-image-retry-attempts = Máximo de reintentos ante fallos temporales de imágenes en una carga de mensaje visible. Rango 0–8; por defecto 3. Cero desactiva los reintentos automáticos conservando el reintento manual.
admin-settings-key-privacy-image-retry-delay-ms = Retardo de los reintentos automáticos de imágenes
admin-settings-help-privacy-image-retry-delay-ms = Milisegundos entre los reintentos automáticos de imágenes, una vez estabilizado el fotograma actual. Rango 500–60000; por defecto 5000. Los rechazos permanentes no se reintentan automáticamente.
admin-settings-key-privacy-image-decode-timeout-ms = Tiempo límite de decodificación de imágenes
admin-settings-help-privacy-image-decode-timeout-ms = Milisegundos de reloj permitidos para una decodificación de imagen en entorno aislado tras su obtención. Rango 1000–60000; por defecto 10000. Auméntelo para imágenes grandes; los tiempos límite de red siguen siendo aparte.
admin-settings-key-privacy-image-concurrency-mode = Perfil de concurrencia de imágenes
admin-settings-help-privacy-image-concurrency-mode = Automático se adapta a la capacidad de CPU, la memoria del contenedor y el número de imágenes aprobadas de cada mensaje. Pocos recursos, Equilibrado y Alto rendimiento también se adaptan, con distintos topes de seguridad. Manual usa directamente los límites numéricos. Aplicar un perfil solo cambia este ajuste.
admin-settings-cat-image-proxy = Proxy de imágenes
admin-settings-key-privacy-image-ram-max-mi-b = Máximo del nivel de RAM
admin-settings-help-privacy-image-ram-max-mi-b = Límite superior en MiB para el nivel de imágenes en memoria, aplicado además del tamaño de caché automático o manual. Cero no añade ningún tope. Manténgalo pequeño y deje que el nivel de disco aloje la mayor parte. Rango 0–1048576; por defecto 0.
admin-settings-key-privacy-image-disk-enabled = Nivel de disco de imágenes
admin-settings-help-privacy-image-disk-enabled = Conserva en disco, tras el nivel de RAM, las imágenes ya recodificadas. Desactivado por defecto; si el directorio no existe, está lleno o es de solo lectura, la caché continúa en silencio solo en RAM. Solo se almacena la salida del jail, con nombres de archivo con hash.
admin-settings-key-privacy-image-disk-path = Directorio de disco de imágenes
admin-settings-help-privacy-image-disk-path = Directorio absoluto que se usa para el nivel de disco. Vacío usa una carpeta image-cache junto al archivo de la base de datos. Monte allí un volumen con escritura cuando el sistema de archivos raíz sea de solo lectura. Las rutas relativas y .. se rechazan y dejan el nivel desactivado.
admin-settings-key-privacy-image-disk-mode = Dimensionamiento del disco de imágenes
admin-settings-help-privacy-image-disk-mode = Automático usa una décima parte del espacio que esta caché podría usar (el espacio libre más lo que ya contiene), entre el mínimo y el máximo y nunca más de la mitad de ese espacio. Manual usa exactamente el tamaño máximo indicado abajo. Automático por defecto.
admin-settings-key-privacy-image-disk-max-mi-b = Tamaño del disco de imágenes (manual)
admin-settings-help-privacy-image-disk-max-mi-b = Tamaño del nivel de disco en MiB en modo manual; la lectura muestra el tamaño efectivo en ambos casos. Rango 1–1048576; por defecto 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Mínimo del tamaño de disco automático
admin-settings-help-privacy-image-disk-floor-mi-b = El nivel de disco más pequeño en MiB en modo automático; también se usa mientras se desconoce el espacio libre. Rango 0–1048576; por defecto 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Máximo del tamaño de disco automático
admin-settings-help-privacy-image-disk-ceiling-mi-b = El nivel de disco más grande en MiB en modo automático. Rango 0–1048576; por defecto 4096.
admin-settings-key-privacy-image-disk-entries = Límite de entradas del disco
admin-settings-help-privacy-image-disk-entries = Máximo de imágenes conservadas en disco. Cada entrada también cuesta una cantidad fija de RAM en el índice, que cuenta para el nivel de RAM. Rango 16–1048576; por defecto 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Imagen de disco más grande
admin-settings-help-privacy-image-disk-entry-bytes = La mayor imagen recodificada que se escribe en disco, en bytes. Las imágenes por encima del límite de entrada de RAM pero dentro de este van directamente a disco. Rango 65536–67108864; por defecto 8388608.
admin-settings-key-privacy-image-disk-min-hits = Aciertos antes de escribir en disco
admin-settings-help-privacy-image-disk-min-hits = Aciertos que necesita una entrada de RAM antes de que su expulsión la escriba en disco en lugar de descartarla. Cero escribe cada entrada expulsada cuya puntuación supera el umbral de degradación. Rango 0–100; por defecto 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Cola de escritura en disco
admin-settings-help-privacy-image-disk-write-queue-mi-b = Datos de imagen en MiB que pueden esperar al escritor de disco en segundo plano. Cuentan para el nivel de RAM; por encima, las imágenes expulsadas se descartan en lugar de ponerse en cola. Rango 1–256; por defecto 8.
admin-settings-key-privacy-image-ttl-step-secs = Paso de vigencia por acierto
admin-settings-help-privacy-image-ttl-step-secs = Segundos que se suman a la vigencia restante de una imagen en un acierto, multiplicados por 1 más el logaritmo de su recuento de aciertos amortiguado, de modo que las imágenes muy usadas viven más. Cero mantiene cada imagen en la vigencia base. Rango 0–86400; por defecto 600.
admin-settings-key-privacy-image-ttl-max-secs = Vigencia ampliada máxima
admin-settings-help-privacy-image-ttl-max-secs = La vigencia restante más larga en segundos que pueden acumular los aciertos. Rango 60–2592000; por defecto 86400.
admin-settings-key-privacy-image-max-age-secs = Antigüedad máxima de las imágenes
admin-settings-help-privacy-image-max-age-secs = Antigüedad en segundos tras la cual una imagen se vuelve a obtener sea cual sea su número de aciertos. La ruta de obtención no conserva ETag ni Last-Modified del origen, por lo que no hay revalidación condicional; la imagen simplemente se vuelve a obtener. Rango 3600–7776000; por defecto 604800.
admin-settings-key-privacy-image-score-half-life-secs = Semivida de la puntuación
admin-settings-help-privacy-image-score-half-life-secs = Segundos para que el recuento de aciertos y la recencia de una imagen se reduzcan a la mitad en la puntuación de prioridad que decide la RAM, el disco y la expulsión. Rango 60–604800; por defecto 3600.
admin-settings-key-privacy-image-score-demote-milli = Puntuación de degradación
admin-settings-help-privacy-image-score-demote-milli = Puntuación, en milésimas, por debajo de la cual una imagen en RAM se degrada a disco o se descarta. El umbral de promoción es esta más la histéresis. Rango 0–10000; por defecto 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Histéresis de promoción
admin-settings-help-privacy-image-score-hysteresis-percent = Porcentaje en que el umbral de promoción queda por encima del umbral de degradación. Una puntuación entre ambos deja la imagen donde está. Rango 0–500; por defecto 50.
admin-settings-key-privacy-image-context-open-percent = Peso del mensaje abierto
admin-settings-help-privacy-image-context-open-percent = Peso de la puntuación en porcentaje para las imágenes solicitadas para un mensaje. Las imágenes de un mensaje cuentan como Recientes tras diez minutos de inactividad. Rango 100–1000; por defecto 200.
admin-settings-key-privacy-image-context-recent-percent = Peso del mensaje reciente
admin-settings-help-privacy-image-context-recent-percent = Peso de la puntuación en porcentaje para las imágenes usadas recientemente. Las solicitudes que no nombran ningún mensaje pesan 100. Rango 100–1000; por defecto 150.
admin-settings-key-privacy-image-account-share-percent = Cuota de RAM por cuenta
admin-settings-help-privacy-image-account-share-percent = La mayor parte del nivel de RAM que pueden llenar las inserciones de una cuenta mientras otras cuentas usan la caché; más allá, la cuenta expulsa primero sus propias imágenes de menor puntuación. Una sola cuenta puede usarlo todo. Rango 1–100; por defecto 50.
admin-settings-key-privacy-image-sweep-secs = Intervalo de barrido de la caché
admin-settings-help-privacy-image-sweep-secs = Segundos entre barridos en segundo plano que caducan imágenes, repuntúan un lote acotado, degradan imágenes frías de RAM bajo presión y vuelven a comprobar el espacio libre en disco. Rango 5–3600; por defecto 30.
admin-settings-key-privacy-image-snapshot-secs = Intervalo de instantáneas del índice
admin-settings-help-privacy-image-snapshot-secs = Segundos entre instantáneas atómicas del índice de disco, que conservan los recuentos de aciertos y las vigencias ampliadas entre reinicios sin una escritura por acierto. Cero desactiva las instantáneas; un reinicio lee entonces la cabecera de cada archivo. Rango 0–86400; por defecto 300.
admin-settings-key-privacy-image-sweep-batch = Tamaño del lote de barrido
admin-settings-help-privacy-image-sweep-batch = Entradas que un barrido de mantenimiento puede caducar, repuntuar o expulsar. Acota el trabajo hecho bajo el bloqueo de la caché. Rango 16–8192; por defecto 256.
admin-image-proxy-title = Lectura de la caché de imágenes
admin-image-proxy-unavailable = La lectura de la caché de imágenes no está disponible.
admin-image-proxy-unknown = desconocido
admin-image-proxy-ram = Nivel de RAM: { $used } usados de { $budget } (configurado { $configured }), { $entries } imágenes, { $hot } calientes, { $cold } frías, tasa de aciertos { $hit }.
admin-image-proxy-disk-off = Nivel de disco: desactivado.
admin-image-proxy-disk = Nivel de disco: { $bytes } usados de { $budget } efectivos ({ $mode }; mínimo { $floor }, máximo { $ceiling }, manual { $manual }), { $entries } imágenes, { $hot } calientes, { $cold } frías, tasa de aciertos { $hit }.
admin-image-proxy-disk-inactive = Nivel de disco: activado pero inactivo. { $reason }
admin-image-proxy-moves = Promociones por minuto: { $perMin }. Promociones totales { $promotions }, degradaciones { $demotions }, descartes { $drops }. Entradas del índice: { $index }.

# -- Machine translation (2026-11): added to match en ---------------------
admin-image-proxy-refresh = Actualizar la lectura
admin-settings-key-security-trust-enabled = Niveles de confianza
admin-settings-help-security-trust-enabled = Asigna un nivel a cada solicitud: con sesión iniciada, origen conocido (un inicio de sesión correcto reciente desde esa dirección) o desconocido. Solo cambian las asignaciones de solicitudes anónimas; las credenciales, los segundos factores y los bloqueos nunca se relajan.
admin-settings-key-security-trust-known-ttl-days = Ventana de origen conocido (días)
admin-settings-help-security-trust-known-ttl-days = Cuánto tiempo un inicio de sesión correcto mantiene conocida su dirección. Los intentos fallidos nunca cuentan.
admin-settings-key-security-trust-known-capacity = Orígenes conocidos conservados
admin-settings-help-security-trust-known-capacity = Máximo de orígenes conocidos conservados en memoria (un hash con sal, un país y un recuento de cuentas cada uno). El más antiguo se descarta primero.
admin-settings-key-security-trust-persist = Guardar los orígenes conocidos entre reinicios
admin-settings-help-security-trust-persist = Guarda la tabla de orígenes conocidos (hashes con clave, país, hora de la última vez visto, recuento de cuentas) en el almacén de ajustes y la carga al iniciar. Las entradas más antiguas que la ventana se descartan.
admin-settings-key-security-trust-persist-interval-secs = Intervalo de guardado de orígenes conocidos (segundos)
admin-settings-help-security-trust-persist-interval-secs = Segundos entre guardados, de 30 a 3600. Solo se guarda cuando la tabla ha cambiado, y una vez más al apagar de forma ordenada.
admin-settings-key-security-trust-unknown-percent = Asignación de orígenes desconocidos (%)
admin-settings-help-security-trust-unknown-percent = Asignación de solicitudes anónimas de un origen desconocido como porcentaje de los límites configurados. 100 mantiene la línea base; más bajo es más estricto.
admin-settings-key-security-trust-known-percent = Asignación de orígenes conocidos (%)
admin-settings-help-security-trust-known-percent = Asignación de solicitudes anónimas de un origen conocido. 200 duplica los presupuestos de solicitudes y reduce a la mitad la espera del inicio de sesión y la dificultad del desafío. Nunca omite credenciales, segundos factores ni bloqueos.
admin-settings-key-security-trust-ua-missing = Bots: user-agent vacío
admin-settings-help-security-trust-ua-missing = Acción para solicitudes anónimas con un user-agent vacío: allow, limit o block. Las sesiones iniciadas nunca se ven afectadas.
admin-settings-key-security-trust-ua-uncommon = Bots: user-agent no reconocido
admin-settings-help-security-trust-ua-uncommon = Acción para solicitudes anónimas con un user-agent no reconocido: allow, limit o block.
admin-settings-key-security-trust-ua-automation = Bots: scripts y automatización
admin-settings-help-security-trust-ua-automation = Acción para solicitudes anónimas que dicen ser un script, una biblioteca HTTP, un navegador sin interfaz o un bot genérico: allow, limit o block.
admin-settings-key-security-trust-ua-search = Bots: rastreadores de buscadores
admin-settings-help-security-trust-ua-search = Acción para solicitudes anónimas que dicen ser el rastreador de un motor de búsqueda: allow, limit o block.
admin-settings-key-security-trust-ua-ai-training = Bots: rastreadores de entrenamiento de IA
admin-settings-help-security-trust-ua-ai-training = Acción para solicitudes anónimas que dicen ser GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended o Applebot-Extended: allow, limit o block. Una afirmación es una cabecera, no una identidad verificada.
admin-settings-key-security-trust-ua-ai-search = Bots: rastreadores de búsqueda de IA
admin-settings-help-security-trust-ua-ai-search = Acción para solicitudes anónimas que dicen ser OAI-SearchBot, PerplexityBot o Claude-SearchBot: allow, limit o block.
admin-settings-key-security-trust-ua-ai-user = Bots: consultas de IA iniciadas por el usuario
admin-settings-help-security-trust-ua-ai-user = Acción para solicitudes anónimas que dicen ser ChatGPT-User o Claude-User: allow, limit o block.
admin-settings-key-security-trust-ua-custom = Bots: patrones personalizados
admin-settings-help-security-trust-ua-custom = Acción para solicitudes anónimas que coinciden con los patrones de user-agent personalizados: allow, limit o block.
admin-settings-key-security-trust-admin-geo-enabled = Bloquear el descubrimiento de administración desde países desconocidos
admin-settings-help-security-trust-admin-geo-enabled = Bloquea un origen que busca la consola de administración desde un país sin inicio de sesión correcto reciente. Requiere una base de datos de países legible y la protección contra sondeos de administración. MW_ADMIN_GEO_ENABLED=false lo desactiva para la recuperación.
admin-settings-key-security-trust-admin-geo-countries = Países siempre conocidos (bloqueo de admin)
admin-settings-help-security-trust-admin-geo-countries = Códigos de país ISO en mayúsculas que siempre se tratan como conocidos, además de los países de los inicios de sesión correctos recientes.
admin-settings-key-security-trust-admin-geo-lookup-failure = Origen sin país
admin-settings-help-security-trust-admin-geo-lookup-failure = allow no bloquea y registra el caso; unknown trata un origen sin país detectado como un país desconocido.
admin-settings-key-security-trust-admin-geo-threshold = Sondeos antes de un bloqueo
admin-settings-help-security-trust-admin-geo-threshold = Sondeos de descubrimiento de administración desde un país desconocido dentro de la ventana que inician un bloqueo.
admin-settings-key-security-trust-admin-geo-window-secs = Ventana de sondeos (segundos)
admin-settings-help-security-trust-admin-geo-window-secs = Ventana del umbral de sondeos.
admin-settings-key-security-trust-admin-geo-block-on-first = Bloquear en el primer sondeo
admin-settings-help-security-trust-admin-geo-block-on-first = Bloquea en el primer sondeo desde un país desconocido. Aumenta la probabilidad de dejar fuera a un administrador que viaja.
admin-settings-key-security-trust-admin-geo-block-secs = Primer bloqueo (segundos)
admin-settings-help-security-trust-admin-geo-block-secs = Duración del primer bloqueo.
admin-settings-key-security-trust-admin-geo-second-block-secs = Segundo bloqueo (segundos)
admin-settings-help-security-trust-admin-geo-second-block-secs = Duración del segundo bloqueo del mismo origen dentro del periodo de repetición.
admin-settings-key-security-trust-admin-geo-max-block-secs = Bloqueo repetido (segundos)
admin-settings-help-security-trust-admin-geo-max-block-secs = Duración del tercer bloqueo y de todos los posteriores dentro del periodo de repetición.
admin-settings-key-security-trust-admin-geo-repeat-days = Periodo de repetición (días)
admin-settings-help-security-trust-admin-geo-repeat-days = Un bloqueo dentro de este número de días del anterior cuenta como repetición y usa la siguiente duración, más larga.
admin-settings-key-security-signin-account-geo-mode = Política de país de inicio de sesión por cuenta
admin-settings-help-security-signin-account-geo-mode = Desactivado por defecto. Bloquea los inicios de sesión desde países que la cuenta no ha usado antes, o desde cualquier país fuera de su lista permitida. Las IP y los países de la lista de permitidos siempre pasan. Se aplica solo a los inicios de sesión nuevos de webmail, nunca a las sesiones existentes ni a los clientes de protocolo. La detección del país usa una base de datos de geolocalización de IP que puede ser errónea o estar desactualizada; las VPN, los operadores móviles, los proxies y los viajes pueden bloquear un inicio de sesión legítimo o dejar pasar uno extranjero. Añada IP o países de confianza a la lista de permitidos.
admin-settings-key-security-signin-account-geo-allow-countries = Países permitidos de la cuenta
admin-settings-help-security-signin-account-geo-allow-countries = Códigos de país ISO en mayúsculas siempre permitidos para la cuenta. La detección del país usa una base de datos de geolocalización de IP que puede ser errónea o estar desactualizada; las VPN, los operadores móviles, los proxies y los viajes pueden bloquear un inicio de sesión legítimo o dejar pasar uno extranjero. Añada IP o países de confianza a la lista de permitidos.
admin-settings-key-security-signin-account-geo-allow-ips = IP permitidas de la cuenta
admin-settings-help-security-signin-account-geo-allow-ips = Direcciones IP o redes CIDR siempre permitidas para la cuenta. Una sola dirección IPv6 significa su /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Países permitidos para todas las cuentas
admin-settings-help-security-signin-account-geo-global-allow-countries = Códigos de país ISO en mayúsculas permitidos para todas las cuentas, añadidos a la lista propia de cada cuenta.
admin-settings-key-security-signin-account-geo-global-allow-ips = IP permitidas para todas las cuentas
admin-settings-help-security-signin-account-geo-global-allow-ips = Direcciones IP o redes CIDR (una oficina o una VPN) permitidas para todas las cuentas, añadidas a la lista propia de cada cuenta.
admin-settings-key-security-signin-account-geo-lookup-failure = Origen sin país (política de cuenta)
admin-settings-help-security-signin-account-geo-lookup-failure = Cuando el origen no tiene un país detectado: permitir el inicio de sesión y registrarlo, o denegarlo.
admin-settings-key-security-signin-account-geo-history-days = Historial de países de la cuenta (días)
admin-settings-help-security-signin-account-geo-history-days = Cuántos días permanece un país en el historial de inicios de sesión propio de una cuenta para la política de países inusuales. Se mantiene aparte del registro de autenticación, por lo que acortar el registro no lo acorta.
admin-settings-key-security-geo-no-database = Sin base de datos de países: comportamiento predeterminado
admin-settings-help-security-geo-no-database = Qué hacen las funciones de país cuando no hay ninguna base de datos cargada: permitir, tratar el origen como un país desconocido o bloquear (block se aplica solo a los inicios de sesión). El ajuste propio de una función también se aplica y prevalece el más estricto.
admin-settings-key-security-geo-not-found = Dirección no incluida en la base de datos: comportamiento predeterminado
admin-settings-help-security-geo-not-found = Qué hacen las funciones de país cuando hay una base de datos cargada pero sin entrada para la dirección. Las direcciones privadas, de loopback y de enlace local son redes locales y nunca se ven afectadas.
admin-settings-key-security-signin-geo-no-database = Política de inicio de sesión de la instancia sin base de datos
admin-settings-help-security-signin-geo-no-database = Cuando la política de país de inicio de sesión de la instancia está activada y no hay ninguna base de datos cargada: bloquear los inicios de sesión (fail closed) o permitirlos sin comprobar. Las redes locales siempre se permiten.
admin-settings-key-security-geo-session-lookup-failure = Alertas de sesión sin país
admin-settings-help-security-geo-session-lookup-failure = Cuando no se puede determinar el país de una solicitud con sesión iniciada: ignorarlo o generar una alerta de sesión por sesión y día. Nunca bloquea ni revoca una sesión.
admin-settings-key-security-protocol-framing-mode = Guardián de protocolo: delimitación de solicitudes
admin-settings-help-security-protocol-framing-mode = Clase de request smuggling: Content-Length junto con Transfer-Encoding, Transfer-Encoding distinto de exactamente chunked (o en HTTP/1.0 y HTTP/2), Content-Length no numérico o contradictorio. enforce responde 400 y cierra la conexión; detect solo cuenta. Una exención por prefijo de ruta nunca se aplica a esta clase.
admin-settings-key-security-protocol-host-mode = Guardián de protocolo: sintaxis de Host
admin-settings-help-security-protocol-host-mode = Sintaxis de la cabecera Host: ausente en HTTP/1.1, vacía, repetida, o con información de usuario, espacios en blanco o caracteres de ruta. Se aceptan literales IPv6, puertos y un punto final. enforce responde 400 y cierra la conexión.
admin-settings-key-security-protocol-host-pinning-mode = Guardián de protocolo: Host debe coincidir con el nombre público
admin-settings-help-security-protocol-host-pinning-mode = Host que no coincide con el host de MW_PUBLIC_URL ni con uno de allowedHosts. No hace nada mientras ninguno nombre un host. detect por defecto porque los proxies y las comprobaciones de estado pueden enviar un nombre interno.
admin-settings-key-security-protocol-path-control-mode = Guardián de protocolo: caracteres de control en la ruta
admin-settings-help-security-protocol-path-control-mode = NUL, caracteres de control o CR/LF codificados (%00, %0d, %0a) en la ruta de la solicitud. enforce responde 400. Nunca se exime por prefijo de ruta.
admin-settings-key-security-protocol-target-mode = Guardián de protocolo: trucos del destino de la solicitud
admin-settings-help-security-protocol-target-mode = Codificación porcentual doble o excesiva de puntos y barras, puntos codificados, segmentos de puntos, segmentos de ruta vacíos (//), barras invertidas, bytes no ASCII sin procesar y un destino en forma absoluta que nombra una autoridad distinta de Host.
admin-settings-key-security-protocol-path-length-mode = Guardián de protocolo: longitud de ruta y de consulta
admin-settings-help-security-protocol-path-length-mode = Ruta o consulta más largas que maxPathBytes o maxQueryBytes. enforce responde 414.
admin-settings-key-security-protocol-method-mode = Guardián de protocolo: métodos HTTP
admin-settings-help-security-protocol-method-mode = Método fuera de allowedMethods (TRACE, TRACK, CONNECT y métodos desconocidos o en minúsculas por defecto). enforce responde 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Guardián de protocolo: cabeceras de valor único repetidas
admin-settings-help-security-protocol-duplicate-headers-mode = Una cabecera que debería aparecer una vez aparece más de una vez (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, cabeceras condicionales y de rango), un Content-Length repetido con valores iguales, o cabeceras Cookie repetidas en HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Guardián de protocolo: cabeceras de reenvío de pares no confiables
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* o X-Real-IP enviados por un par fuera de MW_TRUSTED_PROXIES. Las cabeceras nunca se consideran fiables en ningún caso; esto registra el intento. Solo cuenta para la protección adaptativa cuando hay proxies de confianza configurados.
admin-settings-key-security-protocol-header-hygiene-mode = Guardián de protocolo: higiene de cabeceras
admin-settings-help-security-protocol-header-hygiene-mode = Número de cabeceras, un valor individual o la cabecera Cookie por encima de los límites configurados; bytes no ASCII en cabeceras; Upgrade o Connection: upgrade fuera de la ruta WebSocket; Connection que nombra cabeceras como Content-Length o Host. enforce responde 431 para los hallazgos de tamaño y número y 400 para el resto.
admin-settings-key-security-protocol-body-mode = Guardián de protocolo: delimitación del cuerpo de la solicitud
admin-settings-help-security-protocol-body-mode = GET o HEAD con cuerpo, Content-Length por encima de maxBodyBytes (413), Expect distinto de 100-continue (417) o sin cuerpo.
admin-settings-key-security-protocol-max-path-bytes = Guardián de protocolo: ruta más larga (bytes)
admin-settings-help-security-protocol-max-path-bytes = La ruta de solicitud más larga antes de que la clase de longitud de ruta la informe.
admin-settings-key-security-protocol-max-query-bytes = Guardián de protocolo: consulta más larga (bytes)
admin-settings-help-security-protocol-max-query-bytes = La cadena de consulta más larga antes de que la clase de longitud de ruta la informe.
admin-settings-key-security-protocol-max-header-count = Guardián de protocolo: máximo de cabeceras
admin-settings-help-security-protocol-max-header-count = Máximo de campos de cabecera en una solicitud antes de que la clase de higiene de cabeceras lo informe. El límite estricto propio del listener (MW_HTTP_MAX_HEADERS) se sigue aplicando.
admin-settings-key-security-protocol-max-header-value-bytes = Guardián de protocolo: valor de cabecera más largo (bytes)
admin-settings-help-security-protocol-max-header-value-bytes = El valor de cabecera individual más largo antes de que la clase de higiene de cabeceras lo informe.
admin-settings-key-security-protocol-max-cookie-bytes = Guardián de protocolo: cabecera Cookie más grande (bytes)
admin-settings-help-security-protocol-max-cookie-bytes = Tamaño combinado de las cabeceras Cookie antes de que la clase de higiene de cabeceras lo informe.
admin-settings-key-security-protocol-max-body-bytes = Guardián de protocolo: cuerpo declarado más grande (bytes)
admin-settings-help-security-protocol-max-body-bytes = El mayor Content-Length que acepta la clase de cuerpo sin informarlo. Manténgalo por encima de la mayor subida o importación que acepte una ruta; una exención por prefijo de ruta puede elevarlo para una ruta.
admin-settings-key-security-protocol-allowed-methods = Guardián de protocolo: métodos permitidos
admin-settings-help-security-protocol-allowed-methods = Métodos que acepta la clase de método. El valor predeterminado lista los verbos HTTP ordinarios y los de WebDAV, CalDAV y CardDAV; el servidor no monta hoy rutas DAV de entrada, por lo que llegan a un 405 en el enrutador.
admin-settings-key-security-protocol-allowed-hosts = Guardián de protocolo: hosts adicionales aceptados
admin-settings-help-security-protocol-allowed-hosts = Nombres de host que acepta la clase de fijación de host además del host de MW_PUBLIC_URL, por ejemplo un nombre interno que use un proxy o una comprobación de estado. Los puertos y un punto final se ignoran.
admin-settings-key-security-protocol-exempt-prefixes = Guardián de protocolo: prefijos de ruta exentos
admin-settings-help-security-protocol-exempt-prefixes = Prefijos de ruta (cada uno empezando por / y de más de un carácter) para los que no se ejecutan las clases heurísticas, para un cliente legítimo que lo necesite, por ejemplo una ruta de subidas grandes. Coincide con segmentos de ruta completos. Los hallazgos de delimitación y de control de ruta nunca se eximen, y / se ignora.
admin-settings-key-security-health-mode = Acceso a los puntos de conexión de estado
admin-settings-help-security-health-mode = Quién puede llegar a /healthz y /readyz. internal-only: direcciones de cliente de loopback, privadas, de enlace local y locales únicas. allowlist: esas más las redes indicadas abajo. open: cualquiera. Cualquier otro recibe el mismo 404 que una ruta desconocida. Una solicitud con el token MW_HEALTH_TOKEN siempre se admite. La dirección del cliente es la resuelta mediante proxies de confianza: tras un proxy inverso sin MW_FORWARDED_MODE ni MW_TRUSTED_PROXIES, cada solicitud parece venir de la dirección privada del proxy, por lo que se admite a todo Internet.
admin-settings-key-security-health-networks = Redes de los puntos de conexión de estado
admin-settings-help-security-health-networks = Direcciones IP o redes CIDR que pueden llegar a /healthz y /readyz en modo allowlist, además de los rangos internos, por ejemplo un monitor externo de disponibilidad.
admin-settings-key-security-spray-enabled = Detección de password spraying
admin-settings-help-security-spray-enabled = Cuenta los nombres de usuario distintos con los que falla un origen. Solo se conserva en memoria un hash con clave de cada nombre de usuario.
admin-settings-key-security-spray-window-secs = Ventana de spraying (segundos)
admin-settings-help-security-spray-window-secs = Ventana en la que se cuentan por nombre de usuario distinto los inicios de sesión fallidos de un origen. Por defecto 900.
admin-settings-key-security-spray-challenge-at = Spraying: umbral de desafío
admin-settings-help-security-spray-challenge-at = Nombres de usuario distintos que fallan desde un origen dentro de la ventana y que fuerzan el desafío de prueba de trabajo para ese origen. Los inicios de sesión correctos nunca cuentan, por lo que una dirección compartida de oficina o campus no se ve afectada. Por defecto 10.
admin-settings-key-security-spray-pace-at = Spraying: umbral de ralentización
admin-settings-help-security-spray-pace-at = Nombres de usuario distintos que fallan y que añaden espera adicional entre los intentos del origen. Por defecto 15.
admin-settings-key-security-spray-lock-at = Spraying: umbral de bloqueo
admin-settings-help-security-spray-lock-at = Nombres de usuario distintos que fallan y que bloquean el origen durante la duración del bloqueo, mediante el bloqueo de inicio de sesión existente. Por defecto 25.
admin-settings-key-security-spray-pace-secs = Spraying: espera de ralentización (segundos)
admin-settings-help-security-spray-pace-secs = Segundos entre intentos de un origen en el umbral de ralentización o por encima. Por defecto 10.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-spray-lock-secs = Spraying: duración del bloqueo (segundos)
admin-settings-help-security-spray-lock-secs = Duración del bloqueo temporal. Cuando un administrador desbloquea el origen, también se restablece su recuento. Por defecto 900.
admin-settings-key-security-spray-known-multiplier = Spraying: multiplicador de orígenes conocidos
admin-settings-help-security-spray-known-multiplier = Multiplicador de los tres umbrales para un origen conocido, es decir, con un inicio de sesión correcto reciente. Por defecto 3.
admin-settings-key-security-spray-capacity = Spraying: orígenes seguidos
admin-settings-help-security-spray-capacity = Máximo de orígenes seguidos en memoria; los menos recientemente activos se descartan primero. Por defecto 20000.
admin-settings-key-security-spike-enabled = Detección de picos en toda la instancia
admin-settings-help-security-spike-enabled = Aprende una línea base de inicios de sesión fallidos y entra en modo reforzado cuando toda la instancia la supera con mucho. Nunca bloquea la instancia.
admin-settings-key-security-spike-multiplier-percent = Pico: multiplicador (%)
admin-settings-help-security-spike-multiplier-percent = Una señal cuenta cuando alcanza al menos este porcentaje de su línea base aprendida (400 = cuatro veces) y supera su mínimo absoluto.
admin-settings-key-security-spike-min-failures-per-min = Pico: fallos mínimos por minuto
admin-settings-help-security-spike-min-failures-per-min = Mínimo: inicios de sesión fallidos por minuto por debajo de los cuales la señal de tasa de fallos nunca cuenta. Por defecto 20.
admin-settings-key-security-spike-min-sources = Pico: orígenes con fallos mínimos
admin-settings-help-security-spike-min-sources = Mínimo: orígenes con fallos distintos en un intervalo de cinco minutos por debajo de los cuales esa señal nunca cuenta. Por defecto 10.
admin-settings-key-security-spike-min-usernames = Pico: nombres de usuario con fallos mínimos
admin-settings-help-security-spike-min-usernames = Mínimo: nombres de usuario con fallos distintos en un intervalo de cinco minutos por debajo de los cuales esa señal nunca cuenta. Por defecto 30.
admin-settings-key-security-spike-signals-required = Pico: señales requeridas
admin-settings-help-security-spike-signals-required = Señales que deben superar a la vez su línea base y su mínimo. La firma de un spraying distribuido cuenta como dos. Por defecto 2.
admin-settings-key-security-spike-warmup-hours = Pico: calentamiento (horas)
admin-settings-help-security-spike-warmup-hours = Horas de tráfico observado antes de que pueda empezar el modo reforzado. Hasta entonces la línea base solo aprende. Por defecto 48.
admin-settings-key-security-spike-adapt-hours = Pico: adaptación de la línea base (horas)
admin-settings-help-security-spike-adapt-hours = Constante de tiempo de la línea base. Un valor mayor absorbe el crecimiento más despacio. Un pico nunca actualiza la línea base. Por defecto 168.
admin-settings-key-security-spike-duration-secs = Pico: duración del modo reforzado (segundos)
admin-settings-help-security-spike-duration-secs = Cuánto dura el modo reforzado tras la última evidencia de un pico; se prolonga mientras las señales permanezcan por encima de la mitad de su nivel de disparo. Por defecto 1800.
admin-settings-key-security-spike-force-challenge = Pico: forzar el desafío
admin-settings-help-security-spike-force-challenge = En modo reforzado, exige el desafío de prueba de trabajo para los inicios de sesión anónimos desde orígenes desconocidos. Los usuarios autenticados y los orígenes conocidos no se ven afectados.
admin-settings-key-security-spike-pace-factor-percent = Pico: espera del inicio de sesión (%)
admin-settings-help-security-spike-pace-factor-percent = En modo reforzado, la espera del inicio de sesión para orígenes desconocidos como porcentaje de la espera habitual. Por defecto 300.
admin-settings-key-security-session-binding-enabled = Vinculación de sesión
admin-settings-help-security-session-binding-enabled = Vincula cada sesión de navegador a hashes con clave de sus atributos de cliente. Los clientes nativos con token bearer, las claves de API y los clientes de protocolo están exentos.
admin-settings-key-security-session-binding-attributes = Vinculación de sesión: atributos
admin-settings-help-security-session-binding-attributes = Atributos vinculados: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). network está desactivado por defecto porque las redes móviles cambian. Los nombres desconocidos se ignoran.
admin-settings-key-security-session-binding-strictness = Vinculación de sesión: rigor
admin-settings-help-security-session-binding-strictness = tolerant acepta actualizaciones del navegador y un único cambio de un atributo blando; strict señala cualquier diferencia blanda.
admin-settings-key-security-session-binding-webmail-action = Vinculación de sesión: acción de webmail
admin-settings-help-security-session-binding-webmail-action = Qué hace una discrepancia en una sesión de webmail: off, detect (solo registrar), step-up (pedir de nuevo la contraseña y el segundo factor) o revocar la sesión. Por defecto step-up.
admin-settings-key-security-session-binding-admin-action = Vinculación de sesión: acción de la consola de administración
admin-settings-help-security-session-binding-admin-action = Qué hace una discrepancia en una sesión de la consola de administración: off, detect, step-up o revoke. Por defecto revoke.
admin-settings-key-security-theft-enabled = Detección de robo de sesión
admin-settings-help-security-theft-enabled = Detecta la reutilización de un ID de sesión rotado, los viajes imposibles y el uso simultáneo desde distintos países.
admin-settings-key-security-theft-rotation-grace-secs = Robo: tolerancia de rotación (segundos)
admin-settings-help-security-theft-rotation-grace-secs = Tras rotar una sesión, el ID antiguo puede presentarse de nuevo durante este tiempo (una solicitud en curso) antes de contar como robo. Por defecto 30.
admin-settings-key-security-theft-min-travel-secs = Robo: tiempo mínimo de viaje (segundos)
admin-settings-help-security-theft-min-travel-secs = Un cambio de país más rápido que esto tras la solicitud anterior se trata como un viaje imposible. La detección del país puede ser inexacta; las VPN y las redes móviles pueden causar cambios falsos. Por defecto 3600.
admin-settings-key-security-theft-concurrent-use = Robo: uso simultáneo
admin-settings-help-security-theft-concurrent-use = Señala una sesión usada alternativamente desde distintos países (o redes) en cinco minutos: off, country o network. La detección del país puede ser inexacta; las VPN y las redes móviles pueden causar cambios falsos.
admin-settings-key-security-theft-webmail-action = Robo: respuesta en webmail
admin-settings-help-security-theft-webmail-action = Respuesta a un viaje imposible o al uso simultáneo en una sesión de webmail: alert, step-up o revoke. La reutilización de una sesión rotada siempre revoca. Por defecto step-up.
admin-settings-key-security-theft-admin-action = Robo: respuesta en la consola de administración
admin-settings-help-security-theft-admin-action = Respuesta a un viaje imposible o al uso simultáneo en una sesión de la consola de administración: alert, step-up o revoke. Por defecto revoke.
admin-settings-key-security-theft-retention-days = Robo: conservación de registros (días)
admin-settings-help-security-theft-retention-days = Cuánto tiempo se conservan los marcadores de sesiones rotadas y los registros de seguridad por sesión; configúrelo al menos tan largo como la vigencia de sesión más larga. Por defecto 15.
admin-settings-key-security-alerts-country-change = Alerta de cambio de país
admin-settings-help-security-alerts-country-change = Avisa cuando una sesión iniciada se usa desde un país distinto de aquel en el que empezó. La detección del país puede ser inexacta; las VPN y las redes móviles pueden causar cambios falsos. Los usuarios pueden cambiarlo salvo que esté bloqueado; que un usuario lo desactive solo se respeta si se permite silenciarlo.
admin-settings-key-security-alerts-country-change-enabled = Alerta de cambio de país: interruptor global
admin-settings-help-security-alerts-country-change-enabled = Desactivado desactiva la alerta de cambio de país para todos. La detección del país puede ser inexacta; las VPN y las redes móviles pueden causar cambios falsos.
admin-settings-key-security-alerts-country-change-cooldown-secs = Alerta de cambio de país: espera (segundos)
admin-settings-help-security-alerts-country-change-cooldown-secs = Tiempo mínimo entre alertas de cambio de país de una misma sesión. Cada país nuevo alerta una vez por sesión. Por defecto 300.
admin-settings-key-security-alerts-country-change-user-mute = Alerta de cambio de país: permitir que los usuarios la silencien
admin-settings-help-security-alerts-country-change-user-mute = Si un usuario puede desactivar su propia alerta de cambio de país. Si está desactivado, el servidor sigue alertando aunque el ajuste propio del usuario esté desactivado.
admin-settings-key-security-alerts-channels = Canales de alertas de seguridad
admin-settings-help-security-alerts-channels = Cómo llegan al usuario los avisos de seguridad de sesión: en la aplicación, como mensaje en su propio buzón, o ambos. Los usuarios pueden cambiarlo salvo que esté bloqueado.
admin-settings-key-security-alerts-email-enabled = Alertas de seguridad por correo
admin-settings-help-security-alerts-email-enabled = Permite entregar los avisos de seguridad de sesión en el propio buzón del usuario. No se envía ningún mensaje por SMTP; sin una sesión de entrega, el aviso permanece en la aplicación y en el registro de auditoría.
admin-settings-key-security-alerts-sensitivity = Sensibilidad de las alertas de seguridad
admin-settings-help-security-alerts-sensitivity = low solo avisa por hallazgos de ubicación y nunca aplica step-up ni revoca; normal sigue las acciones del administrador; high duplica el intervalo de viaje y también comprueba redes simultáneas. Los usuarios pueden cambiarlo salvo que esté bloqueado.
admin-settings-key-security-alerts-notice-retention-days = Avisos de seguridad: conservación (días)
admin-settings-help-security-alerts-notice-retention-days = Cuánto tiempo se conservan los avisos de seguridad de la aplicación por cuenta (como máximo 20 por cuenta). Por defecto 30.
admin-settings-inline-effective = En vigor:
admin-settings-inline-reset = Descartar los cambios
admin-settings-inline-domains = Valores por dominio
admin-attachments-tab-lists = Listas de permitidos y bloqueados
admin-attachments-tab-limits = Límites
admin-attachments-tab-delivery = Descarga y vista previa
admin-attachments-tab-warnings = Avisos
admin-attachments-tab-test = Probar un archivo
admin-attachments-tab-denied = No tiene acceso a esta pestaña.
admin-attachments-group-lists = Listas de permitidos y bloqueados
admin-attachments-group-limits = Límite de tamaño
admin-attachments-group-delivery = Descarga y vista previa
admin-attachments-group-warnings = Avisos
admin-attachments-add-extension = Extensión, por ejemplo pdf o .tar.gz
admin-attachments-add-mime = Tipo MIME, por ejemplo image/png o image/*
admin-attachments-add = Añadir
admin-attachments-remove = Quitar { $rule }
admin-attachments-filter = Filtrar la lista
admin-attachments-list-label = Reglas de esta lista
admin-attachments-count = { $count } de { $max } reglas
admin-attachments-empty-list = No hay reglas en esta lista.
admin-attachments-none-match = Ninguna regla coincide con el filtro.
admin-attachments-bulk = Pegar varias
admin-attachments-bulk-add = Añadir todas
admin-attachments-import = Importar un archivo de texto
admin-attachments-export = Exportar como texto
admin-attachments-err-extension = No es una extensión. Use letras, dígitos, guiones y guiones bajos, hasta 32 caracteres; los puntos pueden separar partes, como en tar.gz.
admin-attachments-err-mime = No es un tipo MIME. Use tipo/subtipo, o tipo/* para todos los subtipos.
admin-attachments-err-long = Esa regla es demasiado larga.
admin-attachments-invalid = No se añadió, no es una regla válida: { $entries }
admin-attachments-duplicates = Ya está en la lista: { $rules }
admin-attachments-overflow = Una lista admite como máximo { $max } reglas. Las demás no se añadieron.
admin-attachments-conflict-blockedOverAllowed = { $rule } se solapa con { $other } de la otra lista. Las reglas de bloqueo prevalecen, por lo que queda bloqueada.
admin-attachments-conflict-blockedOverWarn = { $rule } se solapa con { $other } de la otra lista. Un archivo bloqueado se rechaza antes de cualquier aviso, por lo que el aviso nunca se muestra.
admin-attachments-conflict-allowedAndWarn = { $rule } se solapa con { $other } de la otra lista. El archivo está permitido y muestra un aviso.
admin-attachments-bytes-unit = Unidad
admin-attachments-unlimited = Sin límite de tamaño
admin-attachments-bytes-equals = { $size } ({ $bytes } bytes)
admin-attachments-list-allowedExtensions = extensiones permitidas
admin-attachments-list-allowedMimeTypes = tipos MIME permitidos
admin-attachments-list-blockedExtensions = extensiones bloqueadas
admin-attachments-list-blockedMimeTypes = tipos MIME bloqueados
admin-attachments-list-warnExtensions = extensiones con aviso
admin-attachments-list-warnMimeTypes = tipos MIME con aviso
admin-attachments-sim-title = Probar un archivo
admin-attachments-sim-help = Pregunta al servidor cómo trata la política guardada un archivo con este nombre, tipo y tamaño. No se cambia ni se almacena nada. Los mensajes y los adjuntos nunca se leen.
admin-attachments-sim-filename = Nombre del archivo
admin-attachments-sim-mime = Tipo MIME
admin-attachments-sim-size = Tamaño
admin-attachments-sim-run = Probar
admin-attachments-sim-invalid = Introduzca un nombre de archivo y un tamaño de cero o más.
admin-attachments-sim-allowed = Permitido
admin-attachments-sim-warn = Permitido con un aviso
admin-attachments-sim-blocked = No entregado
admin-attachments-sim-preview-yes = Se permite la vista previa.
admin-attachments-sim-preview-no = No se permite la vista previa.
admin-attachments-sim-checked = Tamaño comprobado: { $size }.
admin-attachments-sim-matched = Reglas coincidentes en: { $lists }.
admin-attachments-sim-matched-none = No coincidió ninguna regla de permitir, bloquear ni avisar.
admin-attachments-reason-downloadsDisabled = Las descargas están desactivadas.
admin-attachments-reason-attachmentTooLarge = El archivo supera el límite de tamaño.
admin-attachments-reason-attachmentBlocked = Coincide una regla de bloqueo.
admin-attachments-reason-attachmentNotAllowed = Existe una lista de permitidos y nada de ella coincide.

# -- Machine translation (2026-11): added to match en ---------------------
admin-attachments-reason-attachmentContainerRestricted = Los mensajes adjuntos y los archivos TNEF se rechazan mientras haya alguna restricción activa, porque su contenido no está listado.
admin-settings-key-mail-reply-images = Imágenes en respuestas y reenvíos
admin-settings-help-mail-reply-images = Si una respuesta o un reenvío incluye las imágenes del mensaje original: ask (por defecto), permitir siempre, permitir solo para dominios de remitente de confianza y preguntar en los demás casos, o denegar siempre. Bloquee este ajuste para imponerlo. Las imágenes solo se leen cuando se incluyen.
admin-settings-key-mail-reply-image-trusted-domains = Dominios de remitente de confianza para imágenes
admin-settings-help-mail-reply-image-trusted-domains = Hasta 200 dominios de remitente exactos cuyas imágenes se incluyen sin preguntar cuando la política es dominios de confianza. Los usuarios amplían su propia lista desde el aviso de cita, salvo que este ajuste esté bloqueado.
admin-settings-key-mail-send-recall-delay-seconds = Ventana para deshacer el envío (predeterminada)
admin-settings-help-mail-send-recall-delay-seconds = Cuántos segundos espera un mensaje enviado en la bandeja de salida, donde el remitente puede recuperarlo, antes de entregarse. 0 envía de inmediato. Por defecto 10, hasta 1800 (30 minutos). Los usuarios eligen su propio valor hasta el límite indicado abajo, salvo que lo bloquee.
admin-settings-key-mail-send-recall-delay-max-seconds = Ventana para deshacer el envío (límite)
admin-settings-help-mail-send-recall-delay-max-seconds = La ventana más larga para deshacer el envío que puede elegir un usuario, en segundos. El servidor acorta una solicitud más larga a este valor. Por defecto y máximo 1800.
admin-settings-key-mail-send-allow-recall = Permitir recuperar mensajes enviados
admin-settings-help-mail-send-allow-recall = Si está desactivado, los mensajes salen de inmediato y no se pueden recuperar de la bandeja de salida. Los envíos programados no se ven afectados; aún pueden cancelarse hasta su hora.
admin-settings-key-mail-send-max-delayed-seconds = Horizonte de programación
admin-settings-help-mail-send-max-delayed-seconds = Con cuánta antelación se puede programar un mensaje, en segundos. Por defecto 7776000 (90 días), máximo 31536000 (1 año). 0 desactiva el envío programado.
admin-settings-key-mail-send-max-queued = Mensajes en cola por buzón
admin-settings-help-mail-send-max-queued = Cuántos mensajes puede tener un buzón esperando a la vez en la bandeja de salida (ventana de deshacer o programados). Por defecto 200. Los envíos adicionales se rechazan con un mensaje claro y nunca se descartan.
admin-attachments-sim-pick = Rellenar desde un archivo (opcional)
admin-attachments-mime-toggle = Mostrar tipos MIME comunes
admin-attachments-mime-hint = Elija un tipo común o escriba cualquier otro tipo/subtipo.
admin-attachments-mime-invalid = No es un tipo/subtipo como text/plain. Se enviará tal como se escribió.
admin-attachments-mime-group-documents = Documentos
admin-attachments-mime-group-images = Imágenes
admin-attachments-mime-group-archives = Archivos comprimidos
admin-attachments-mime-group-executables = Ejecutables y scripts
admin-attachments-mime-group-media = Audio y vídeo
admin-attachments-mime-group-email = Correo y calendario
admin-attachments-mime-group-other = Otros
admin-settings-key-cookies-enabled = Control de consentimiento de cookies
admin-settings-help-cookies-enabled = Muestra a los visitantes el control de consentimiento de cookies. Desactivado deja la aplicación comportándose como sin el control.
admin-settings-key-cookies-mode = Modelo de consentimiento
admin-settings-help-cookies-mode = optIn: no se almacena nada opcional hasta que el visitante acepta. optOut: se aplican los valores predeterminados de las categorías hasta que el visitante rechaza. noticeOnly: se informa al visitante, sin opción.
admin-settings-key-cookies-region-mode = Modelo de consentimiento en regiones estrictas
admin-settings-help-cookies-region-mode = El modelo que se usa para los visitantes de las regiones estrictas. same usa también allí el modelo de consentimiento anterior.
admin-settings-key-cookies-strict-regions = Regiones estrictas
admin-settings-help-cookies-strict-regions = Códigos de país ISO 3166-1 que usan el modelo de consentimiento de regiones estrictas. Por defecto, los Estados de la UE y del EEE, el Reino Unido y Suiza.
admin-settings-key-cookies-region-header = Cabecera de país
admin-settings-help-cookies-region-header = Nombre de la cabecera de solicitud, definida por un proxy o una CDN, que contiene el código de país del visitante.
admin-settings-key-cookies-unknown-region = País desconocido
admin-settings-help-cookies-unknown-region = Qué modelo se aplica cuando no se puede determinar ningún país: el de regiones estrictas o el normal.
admin-settings-key-cookies-functional-enabled = Ofrecer cookies funcionales
admin-settings-help-cookies-functional-enabled = Ofrece la categoría de preferencias y funcionalidad (tema, idioma, memoria de la disposición) en el control de consentimiento.
admin-settings-key-cookies-functional-default = Valor predeterminado de las cookies funcionales
admin-settings-help-cookies-functional-default = Estado inicial de la categoría funcional cuando el modelo de consentimiento admite valores predeterminados.
admin-settings-key-cookies-analytics-enabled = Ofrecer cookies de analítica
admin-settings-help-cookies-analytics-enabled = Ofrece la categoría de analítica en el control de consentimiento.
admin-settings-key-cookies-analytics-default = Valor predeterminado de la analítica
admin-settings-help-cookies-analytics-default = Estado inicial de la categoría de analítica cuando el modelo de consentimiento admite valores predeterminados.
admin-settings-key-cookies-marketing-enabled = Ofrecer cookies de marketing
admin-settings-help-cookies-marketing-enabled = Ofrece la categoría de marketing en el control de consentimiento.
admin-settings-key-cookies-marketing-default = Valor predeterminado del marketing
admin-settings-help-cookies-marketing-default = Estado inicial de la categoría de marketing cuando el modelo de consentimiento admite valores predeterminados.
admin-settings-key-cookies-layout = Disposición
admin-settings-help-cookies-layout = banner es una barra a lo ancho de la página, card una tarjeta flotante, modal un cuadro de diálogo que retiene la página hasta que el visitante elige.
admin-settings-key-cookies-position = Posición
admin-settings-help-cookies-position = Dónde aparece en la página el control de consentimiento.
admin-settings-key-cookies-reject-prominent = Rechazar tan destacado como aceptar
admin-settings-help-cookies-reject-prominent = Muestra Rechazar todo con el mismo tamaño y estilo que Aceptar todo.
admin-settings-key-cookies-policy-url = URL de la política de cookies
admin-settings-help-cookies-policy-url = Enlace a su política de cookies, que se muestra en el control de consentimiento.
admin-settings-key-cookies-privacy-url = URL de la política de privacidad
admin-settings-help-cookies-privacy-url = Enlace a su política de privacidad, que se muestra en el control de consentimiento.
admin-settings-key-cookies-policy-version = Versión de la política
admin-settings-help-cookies-policy-version = Aumente este número cuando cambie su uso de cookies, para que se vuelva a preguntar a los visitantes.
admin-settings-key-cookies-lifetime-days = Vigencia del consentimiento
admin-settings-help-cookies-lifetime-days = Días que una decisión sigue siendo válida antes de volver a preguntar al visitante.
admin-settings-key-cookies-reprompt-on-version = Volver a preguntar con una versión nueva de la política
admin-settings-help-cookies-reprompt-on-version = Vuelve a preguntar a los visitantes cuando la versión de la política es superior a aquella sobre la que decidieron.
admin-settings-key-cookies-honour-gpc = Respetar Global Privacy Control
admin-settings-help-cookies-honour-gpc = Trata una señal Global Privacy Control del navegador como un rechazo de la analítica y el marketing.
admin-settings-key-cookies-honour-dnt = Respetar Do Not Track
admin-settings-help-cookies-honour-dnt = Trata una señal Do Not Track del navegador como un rechazo de la analítica y el marketing.
admin-settings-key-cookies-log-consent = Mantener un registro de consentimientos
admin-settings-help-cookies-log-consent = Conserva en el servidor un registro de cada decisión de consentimiento, para estadísticas y sus propios archivos.
admin-settings-key-cookies-log-retention-days = Conservación del registro de consentimientos
admin-settings-help-cookies-log-retention-days = Días que se conservan las filas del registro de consentimientos antes de eliminarlas.
admin-settings-key-cookies-scan-interval-hours = Intervalo de análisis del almacenamiento
admin-settings-help-cookies-scan-interval-hours = Horas entre análisis automáticos de las cookies y el almacenamiento del navegador. 0 analiza solo bajo demanda.
admin-settings-key-cookies-unclassified = Almacenamiento sin clasificar
admin-settings-help-cookies-unclassified = report deja sin tocar el almacenamiento que nada clasifica y lo lista para su revisión. Los demás valores lo tratan como esa categoría hasta que lo clasifique.
admin-settings-key-cookies-text-title = Título del control de consentimiento
admin-settings-help-cookies-text-title = Título del control de consentimiento por idioma. Vacío usa el texto integrado.
admin-settings-key-cookies-text-body = Texto del control de consentimiento
admin-settings-help-cookies-text-body = Texto explicativo del control de consentimiento por idioma. Vacío usa el texto integrado.
admin-settings-key-cookies-label-necessary = Nombre de la categoría necesaria
admin-settings-help-cookies-label-necessary = Nombre de la categoría necesaria por idioma. Vacío usa el nombre integrado.
admin-settings-key-cookies-description-necessary = Descripción de la categoría necesaria
admin-settings-help-cookies-description-necessary = Descripción de la categoría necesaria por idioma. Vacío usa la descripción integrada.
admin-settings-key-cookies-label-functional = Nombre de la categoría funcional
admin-settings-help-cookies-label-functional = Nombre de la categoría funcional por idioma. Vacío usa el nombre integrado.
admin-settings-key-cookies-description-functional = Descripción de la categoría funcional
admin-settings-help-cookies-description-functional = Descripción de la categoría funcional por idioma. Vacío usa la descripción integrada.
admin-settings-key-cookies-label-analytics = Nombre de la categoría de analítica
admin-settings-help-cookies-label-analytics = Nombre de la categoría de analítica por idioma. Vacío usa el nombre integrado.
admin-settings-key-cookies-description-analytics = Descripción de la categoría de analítica
admin-settings-help-cookies-description-analytics = Descripción de la categoría de analítica por idioma. Vacío usa la descripción integrada.
admin-settings-key-cookies-label-marketing = Nombre de la categoría de marketing
admin-settings-help-cookies-label-marketing = Nombre de la categoría de marketing por idioma. Vacío usa el nombre integrado.
admin-settings-key-cookies-description-marketing = Descripción de la categoría de marketing
admin-settings-help-cookies-description-marketing = Descripción de la categoría de marketing por idioma. Vacío usa la descripción integrada.
admin-settings-key-encryption-rotation-days = Intervalo de rotación de la clave de la base de datos (días)
admin-settings-help-encryption-rotation-days = Días tras el último cambio de clave a partir de los cuales se informa de que la clave de la base de datos debe rotarse. 0 no fija fecha límite. La propia clave nunca se almacena en los ajustes.
admin-settings-key-encryption-warnings-enabled = Informar de hallazgos de cifrado
admin-settings-help-encryption-warnings-enabled = Muestra los almacenes en texto sin cifrar y una rotación de clave atrasada como hallazgo en la evaluación del despliegue.
admin-settings-key-retention-sweeper-enabled = Barrido de conservación
admin-settings-help-retention-sweeper-enabled = Ejecuta el barrido diario que elimina las filas más antiguas que los límites de esta página. Desactivarlo conserva todas las filas hasta que se vuelva a activar.
admin-settings-key-retention-sweeper-batch-rows = Filas por lote
admin-settings-help-retention-sweeper-batch-rows = Filas que elimina una sentencia de borrado. Los lotes más pequeños mantienen los bloqueos de la base de datos menos tiempo.
admin-settings-key-retention-sweeper-max-batches = Lotes por almacén y ejecución
admin-settings-help-retention-sweeper-max-batches = Máximo de sentencias de borrado que recibe un almacén en una ejecución. Un almacén con más filas antiguas continúa en la siguiente ejecución.
admin-settings-key-retention-sweeper-jitter-minutes = Dispersión de la hora de inicio (minutos)
admin-settings-help-retention-sweeper-jitter-minutes = La siguiente ejecución empieza un número aleatorio de minutos, hasta este valor, después de 24 horas, para que varios servidores no barran a la vez.
admin-settings-key-retention-sweeper-vacuum-after-rows = Recuperar espacio tras (filas)
admin-settings-help-retention-sweeper-vacuum-after-rows = Cuando una ejecución elimina al menos este número de filas, se compacta la base de datos (VACUUM). 0 nunca compacta tras un barrido; la tarea semanal de vacuum es aparte.
admin-settings-key-retention-password-change-audit-days = Registros de cambio de contraseña (días)
admin-settings-help-retention-password-change-audit-days = Días que se conserva un registro de cambio de contraseña (cuenta, método, resultado). 0 los conserva siempre.
admin-settings-key-retention-sso-login-audit-days = Registros de inicio de sesión único (días)
admin-settings-help-retention-sso-login-audit-days = Días que se conserva un registro de inicio de sesión único (proveedor, sujeto con hash, resultado). 0 los conserva siempre.
admin-settings-key-retention-dlp-audit-days = Registros de prevención de pérdida de datos (días)
admin-settings-help-retention-dlp-audit-days = Días que se conserva un registro de coincidencia con una regla de prevención de pérdida de datos. 0 los conserva siempre.
admin-settings-key-retention-assist-audit-days = Registros de acceso del asistente (días)
admin-settings-help-retention-assist-audit-days = Días que se conserva un registro de que un asistente usó una capacidad (actor, capacidad, resumen del ámbito). 0 los conserva siempre.
admin-settings-key-retention-alert-events-days = Registros de alertas disparadas (días)
admin-settings-help-retention-alert-events-days = Días que se conserva un registro de una alerta de administrador disparada. 0 los conserva siempre.
admin-settings-key-retention-job-runs-days = Historial de tareas en segundo plano (días)
admin-settings-help-retention-job-runs-days = Días que se conserva una ejecución terminada de una tarea en segundo plano. Una ejecución que sigue en curso nunca se elimina. 0 conserva el historial siempre.
admin-settings-key-retention-outbox-days = Registros de envíos completados y cancelados (días)
admin-settings-help-retention-outbox-days = Días que se conserva el registro de envío de un mensaje entregado o cancelado. Un mensaje en espera, en reintento, fallido o incierto nunca se elimina. 0 los conserva siempre.
admin-settings-key-retention-expired-credentials-grace-days = Credenciales caducadas (días tras caducar)
admin-settings-help-retention-expired-credentials-grace-days = Días que se conserva, tras caducar, un dispositivo recordado, un desafío de inicio de sesión, un token de inicio de sesión, un token OAuth, un nonce de webhook, un contador de fallos de origen o un identificador de enlace de imagen caducados. 0 lo elimina en la siguiente ejecución.
admin-settings-key-retention-ip-addresses-days = Reducir las direcciones IP almacenadas tras (días)
admin-settings-help-retention-ip-addresses-days = Pasado este número de días, la dirección IP del historial de autenticación y de las entradas de auditoría se trunca o se convierte en hash y la fila se conserva. 0 conserva las direcciones tal como se registraron.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-retention-ip-addresses-mode = Método de reducción de IP
admin-settings-help-retention-ip-addresses-mode = Truncate conserva el prefijo IPv4 /24 o IPv6 /48. Hash sustituye la dirección por un hash corto con clave que sigue agrupando la misma dirección.
admin-settings-key-retention-user-enabled = Los usuarios pueden definir reglas de conservación
admin-settings-help-retention-user-enabled = Los usuarios pueden definir sus propias reglas de conservación para carpetas y correo. Si está desactivado, las reglas de usuario existentes dejan de ejecutarse; las reglas de toda la organización de abajo siguen ejecutándose.
admin-settings-key-retention-user-min-days = Mínimo de días antes de que una regla de usuario elimine correo
admin-settings-help-retention-user-min-days = Una regla de usuario que elimina correo no puede actuar sobre correo más reciente que esto. Un valor menor se rechaza al guardar la regla y se eleva al ejecutarla.
admin-settings-key-retention-user-max-days = Máximo de días que una regla de usuario puede esperar para eliminar correo
admin-settings-help-retention-user-max-days = Una regla de usuario que elimina correo no puede esperar más que esto. 0 no fija ningún tope. Un valor mayor se rechaza al guardar la regla y se reduce al ejecutarla.
admin-settings-key-retention-user-max-rules = Máximo de reglas de conservación por usuario
admin-settings-help-retention-user-max-rules = El mayor número de reglas de conservación que puede mantener un usuario.
admin-settings-key-retention-user-allow-permanent-delete = Las reglas de usuario pueden eliminar correo de forma permanente
admin-settings-help-retention-user-allow-permanent-delete = Si está desactivado, una regla de eliminación siempre mueve el correo a la papelera y se aplica la ventana de deshacer antes de purgarlo.
admin-settings-key-retention-user-undo-days = Ventana de deshacer de las reglas de conservación (días)
admin-settings-help-retention-user-undo-days = Días que el correo movido a la papelera por una regla de conservación permanece allí antes de purgarse. Un usuario puede elegir una ventana más corta. 0 purga en la siguiente ejecución.
admin-settings-key-retention-user-mandatory = Reglas de correo de toda la organización
admin-settings-help-retention-user-mandatory = Reglas que recibe cada cuenta, como entradas rol-de-carpeta:días, por ejemplo trash:30 o junk:30. El correo de esa carpeta más antiguo que los días indicados se purga. Un usuario no puede cambiarlas, pausarlas ni excluir correo de ellas. El correo de una cuenta bajo retención legal se omite.
admin-settings-key-retention-user-protected-folders = Carpetas que las reglas de conservación nunca tocan
admin-settings-help-retention-user-protected-folders = Carpetas sobre las que no actúa ninguna regla de usuario, por rol (drafts, sent, inbox, archive) o por nombre de carpeta. Una carpeta protegida prevalece sobre una regla de toda la organización para la misma carpeta.
admin-settings-key-retention-user-templates = Plantillas de conservación ofrecidas a los usuarios
admin-settings-help-retention-user-templates = Plantillas que los usuarios pueden adoptar en los ajustes, por ID: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Ejecutar reglas de usuario para cuentas restringidas
admin-settings-help-retention-user-run-for-restricted = Si las reglas de conservación de los usuarios también se ejecutan para cuentas bajo una limitación del tratamiento. Desactivado omite esas cuentas.
admin-settings-key-retention-user-interval-hours = Horas entre ejecuciones de conservación de usuarios
admin-settings-help-retention-user-interval-hours = Horas entre las ejecuciones programadas de la tarea de conservación de usuarios para una cuenta.
admin-settings-key-retention-user-batch-messages = Mensajes por llamada de conservación
admin-settings-help-retention-user-batch-messages = Mensajes que gestiona una llamada de mover o eliminar. Se usan 200 como máximo.
admin-settings-key-retention-user-max-per-run = Máximo de mensajes modificados por cuenta y ejecución
admin-settings-help-retention-user-max-per-run = El mayor número de mensajes que la tarea de conservación de usuarios modifica para una cuenta en una ejecución. El resto espera a la siguiente ejecución.
admin-settings-key-retention-user-pause-ms = Pausa entre lotes de conservación (ms)
admin-settings-help-retention-user-pause-ms = Pausa entre lotes, en milisegundos, para que una limpieza grande no cargue el servidor de correo.
admin-settings-key-retention-class-short-days = Tope de la clase corta (días)
admin-settings-help-retention-class-short-days = Tope para los datos operativos: sesiones, tokens, límites de frecuencia, historial de tareas y métricas. Se aplica cuando el límite de clase está activado.
admin-settings-key-retention-class-medium-days = Tope de la clase media (días)
admin-settings-help-retention-class-medium-days = Tope para el historial de inicios de sesión, el historial de protección de solicitudes y los avisos de seguridad. No debe ser inferior a la clase corta; un valor menor se eleva a ella.
admin-settings-key-retention-class-long-days = Conservación de la clase larga (días)
admin-settings-help-retention-class-long-days = Días que se conserva la clase larga (auditoría de seguridad, acciones de administrador, solicitudes, consentimiento, marcadores), incluido el archivo. 0 la conserva siempre y la consola avisa. No debe ser inferior a la clase media.
admin-settings-key-retention-class-enforce = Sujetar cada almacén al tope de su clase
admin-settings-help-retention-class-enforce = Si está activado, el límite propio de cada almacén se ajusta al tope de su clase y 0 (conservar) cuenta como el tope. Si está desactivado, rigen los límites por almacén.
admin-settings-key-retention-archive-enabled = Archivar las filas antiguas de la clase larga
admin-settings-help-retention-archive-enabled = Mueve las filas antiguas de la clase larga a archivos de archivo sellados y comprimidos en lugar de eliminarlas.
admin-settings-key-retention-archive-directory = Directorio de archivo
admin-settings-help-retention-archive-directory = Directorio de los archivos de archivo, absoluto o relativo al directorio de trabajo del servidor. Se rechazan los segmentos de directorio superior (..).
admin-settings-key-retention-archive-batch-rows = Filas por archivo de archivo
admin-settings-help-retention-archive-batch-rows = Filas que se escriben en un archivo de archivo.
admin-settings-key-retention-archive-max-segments = Archivos de archivo por almacén y ejecución
admin-settings-help-retention-archive-max-segments = Archivos de archivo que una ejecución puede escribir para cada almacén. El resto espera a la siguiente ejecución.
admin-settings-key-retention-archive-key-generation = Generación de la clave de sellado del archivo
admin-settings-help-retention-archive-key-generation = Generación de la clave que sella los archivos de archivo nuevos. Auméntela para rotar la clave; los archivos anteriores se siguen verificando con su propia generación.
admin-settings-key-retention-archive-verify-days = Días entre comprobaciones del archivo
admin-settings-help-retention-archive-verify-days = Días entre las comprobaciones de integridad de los archivos de archivo.
admin-settings-key-retention-archive-legal-hold = Retención legal del archivo
admin-settings-help-retention-archive-legal-hold = Conserva cada archivo de archivo, diga lo que diga el límite de la clase larga.
admin-settings-key-logging-file-enabled = Escribir archivos de registro
admin-settings-help-logging-file-enabled = Escribe además el registro del servidor en archivos rotativos. Desactivado deja el registro solo en la salida estándar, que es lo que normalmente usa un contenedor.
admin-settings-key-logging-file-directory = Directorio de registros
admin-settings-help-logging-file-directory = Directorio de los archivos de registro, absoluto o relativo al directorio de trabajo del servidor. Se rechazan los segmentos de directorio superior (..).
admin-settings-key-logging-file-max-size-mb = Rotar por tamaño (MB)
admin-settings-help-logging-file-max-size-mb = El archivo de registro actual se cierra y se inicia uno nuevo cuando alcanza este tamaño.
admin-settings-key-logging-file-rotate-hours = Rotar por antigüedad (horas)
admin-settings-help-logging-file-rotate-hours = El archivo de registro actual se rota cuando tiene esta antigüedad. 0 rota solo por tamaño.
admin-settings-key-logging-file-max-age-days = Conservar archivos rotados (días)
admin-settings-help-logging-file-max-age-days = Se eliminan los archivos de registro rotados más antiguos que esto.
admin-settings-key-logging-file-max-files = Máximo de archivos rotados
admin-settings-help-logging-file-max-files = Se eliminan los archivos rotados más antiguos por encima de este número.
admin-settings-key-logging-file-max-total-mb = Tamaño total máximo (MB)
admin-settings-help-logging-file-max-total-mb = Se eliminan los archivos rotados más antiguos hasta que el directorio de registros quede por debajo de este tamaño.
admin-settings-key-logging-file-compress = Comprimir los archivos rotados
admin-settings-help-logging-file-compress = Los archivos de registro rotados se comprimen con gzip.
admin-settings-key-logging-level = Nivel de registro
admin-settings-help-logging-level = El nivel de los componentes que no tienen un nivel propio más abajo. La variable de entorno MW_LOG fija el nivel al arrancar hasta que se lee este ajuste.
admin-settings-key-logging-component-levels = Niveles de registro por componente
admin-settings-help-logging-component-levels = Una entrada componente=nivel por línea, por ejemplo mw_imap=debug. Los niveles son error, warn, info, debug y trace.
admin-settings-key-logging-redaction-addresses = Direcciones de correo en los registros
admin-settings-help-logging-redaction-addresses = Full deja las direcciones tal como se registran. Truncated conserva la primera letra y el dominio. Hashed sustituye la parte local por un hash corto. None elimina la dirección.
admin-settings-key-logging-redaction-ips = Direcciones IP en los registros
admin-settings-help-logging-redaction-ips = Full deja las direcciones tal como se registran. Truncated conserva el prefijo IPv4 /24 o IPv6 /48. Hashed sustituye la dirección por un hash corto. None la elimina.
admin-settings-key-logging-redaction-tokens = Secretos en los registros
admin-settings-help-logging-redaction-tokens = Elimina de las líneas de registro los tokens bearer, las claves de API, las contraseñas en URL y secretos similares.
admin-settings-key-retention-sessions-idle-days = Sesiones sin usar (días)
admin-settings-help-retention-sessions-idle-days = Días que el registro de una sesión de navegador o de cliente nativo, con su dirección y user-agent, puede quedar sin usar antes de eliminarse. Los tiempos de sesión se aplican primero; esto solo elimina las filas sobrantes. 0 las conserva.
admin-settings-key-policy-public-pages = Páginas de política públicas
admin-settings-help-policy-public-pages = Sirve la política de privacidad generada en /privacy y la política de cookies en /cookies-policy sin iniciar sesión.
admin-settings-key-policy-auto-version = Versiones de política automáticas
admin-settings-help-policy-auto-version = Registra una nueva versión de la política cuando cambia un ajuste que la política describe. Desactivado: los cambios esperan a Publicar.
admin-settings-key-policy-reconsent-on-change = Volver a pedir el consentimiento de cookies con una versión nueva de la política
admin-settings-help-policy-reconsent-on-change = Aumenta la versión de la política de cookies cuando se registra una versión nueva de la política, para que se vuelva a preguntar a los visitantes (requiere cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Nombre del responsable del tratamiento
admin-settings-help-policy-controller-name = Nombre de la organización que decide por qué y cómo se tratan los datos, que se muestra en la política de privacidad.
admin-settings-key-policy-controller-address = Dirección del responsable del tratamiento
admin-settings-help-policy-controller-address = Dirección postal del responsable del tratamiento, que se muestra en la política de privacidad.
admin-settings-key-policy-legal-bases = Bases jurídicas
admin-settings-help-policy-legal-bases = Las bases jurídicas en las que se apoya, con sus propias palabras, que se muestran en la política de privacidad.
admin-settings-key-policy-hosting-provider = Proveedor de alojamiento
admin-settings-help-policy-hosting-provider = Quién aloja los datos y dónde, que se muestra en la política de privacidad.
admin-settings-key-policy-hosting-transfers = Transferencias fuera del país de alojamiento
admin-settings-help-policy-hosting-transfers = Cualquier transferencia de datos a otros países y las garantías que utiliza, que se muestran en la política de privacidad.
admin-settings-key-policy-supervisory-authority = Autoridad de control
admin-settings-help-policy-supervisory-authority = La autoridad ante la que los usuarios pueden presentar una reclamación, que se muestra en la política de privacidad.
admin-settings-key-dpia-suggest = Sugerir evaluaciones de impacto
admin-settings-help-dpia-suggest = Lista una evaluación de impacto sugerida cuando el asistente, el seguimiento de apertura o la analítica están activados y no existe ninguna.
admin-settings-key-dpia-review-months = Intervalo de revisión de las evaluaciones de impacto
admin-settings-help-dpia-review-months = Meses hasta la fecha de revisión que se fija al aprobar una evaluación.
admin-settings-key-dpia-reminder-days = Ventana de recordatorio de las evaluaciones de impacto
admin-settings-help-dpia-reminder-days = Días antes de la fecha de revisión a partir de los cuales una evaluación se lista como pendiente.
admin-settings-key-restriction-enabled = Limitaciones del tratamiento
admin-settings-help-restriction-enabled = Si un administrador o una solicitud aprobada puede limitar el tratamiento de los datos de una persona.
admin-settings-key-restriction-auto-apply-on-approval = Limitar cuando se aprueba una solicitud
admin-settings-help-restriction-auto-apply-on-approval = Aplica la limitación en cuanto se aprueba una solicitud de limitación. Desactivado: un administrador la aplica desde Cumplimiento > Limitaciones.
admin-settings-key-restriction-mail-delivery = Correo nuevo durante una limitación
admin-settings-help-restriction-mail-delivery = Lo que el administrador eligió para el correo que llega mientras se aplica una limitación: conservarlo, ponerlo en cola o rechazarlo. Este servidor no recibe correo; el valor se registra y se muestra, y el servidor de correo actúa en consecuencia.
admin-settings-key-restriction-default-expiry-days = Duración predeterminada de la limitación (días)
admin-settings-help-restriction-default-expiry-days = Días hasta que una limitación nueva termina por sí sola. 0 la mantiene hasta que alguien la levante.
admin-settings-key-restriction-notify-user = Avisar a la persona
admin-settings-help-restriction-notify-user = Envía un aviso a la persona cuando se aplica o se levanta una limitación.
admin-settings-key-restriction-scope-indexing = Pausar la indexación de búsqueda y los embeddings
admin-settings-help-restriction-scope-indexing = Si la indexación de búsqueda y los embeddings se pausan por defecto en una limitación nueva.
admin-settings-key-restriction-scope-assistant = Pausar el asistente
admin-settings-help-restriction-scope-assistant = Si el asistente se pausa por defecto en una limitación nueva.
admin-settings-key-restriction-scope-analytics = Pausar la analítica y el seguimiento de apertura
admin-settings-help-restriction-scope-analytics = Si la analítica y el seguimiento de apertura se pausan por defecto en una limitación nueva.
admin-settings-key-restriction-scope-classification = Pausar las reglas y el archivado automático
admin-settings-help-restriction-scope-classification = Si las reglas y el archivado automático se pausan por defecto en una limitación nueva.
admin-settings-key-restriction-scope-push = Pausar las notificaciones push
admin-settings-help-restriction-scope-push = Si las notificaciones push se pausan por defecto en una limitación nueva.
admin-settings-key-restriction-scope-third-party = Pausar los servicios conectados y los complementos
admin-settings-help-restriction-scope-third-party = Si los servicios conectados y los complementos se pausan por defecto en una limitación nueva.
admin-settings-key-consent-enabled = Consentimiento por finalidad
admin-settings-help-consent-enabled = Muestra las finalidades y las opciones de consentimiento a los usuarios y las aplica en tiempo de ejecución.
admin-settings-key-consent-receipts-retention-days = Recibos de consentimiento conservados (días)
admin-settings-help-consent-receipts-retention-days = Días que se conservan los recibos de consentimiento. 0 los conserva hasta que se suprima a la persona.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-consent-art22-notice = Aviso de tratamiento automatizado
admin-settings-help-consent-art22-notice = Muestra a los usuarios el aviso en lenguaje claro sobre las funciones del asistente y de búsqueda, con los interruptores por función.
admin-settings-key-consent-analytics-basis = Base jurídica: recuentos de uso
admin-settings-help-consent-analytics-basis = La base jurídica que se muestra a los usuarios para los recuentos de uso. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-analytics-version = Versión del texto: recuentos de uso
admin-settings-help-consent-analytics-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-analytics-text = Redacción propia: recuentos de uso
admin-settings-help-consent-analytics-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-analytics-tag-basis = Base jurídica: script del proveedor de analítica
admin-settings-help-consent-analytics-tag-basis = La base jurídica que se muestra a los usuarios para el script del proveedor de analítica. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-analytics-tag-version = Versión del texto: script del proveedor de analítica
admin-settings-help-consent-analytics-tag-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-analytics-tag-text = Redacción propia: script del proveedor de analítica
admin-settings-help-consent-analytics-tag-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-assistant-basis = Base jurídica: asistente
admin-settings-help-consent-assistant-basis = La base jurídica que se muestra a los usuarios para el asistente. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-assistant-version = Versión del texto: asistente
admin-settings-help-consent-assistant-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-assistant-text = Redacción propia: asistente
admin-settings-help-consent-assistant-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-push-basis = Base jurídica: notificaciones push
admin-settings-help-consent-push-basis = La base jurídica que se muestra a los usuarios para las notificaciones push. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-push-version = Versión del texto: notificaciones push
admin-settings-help-consent-push-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-push-text = Redacción propia: notificaciones push
admin-settings-help-consent-push-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-open-tracking-basis = Base jurídica: seguimiento de apertura
admin-settings-help-consent-open-tracking-basis = La base jurídica que se muestra a los usuarios para el seguimiento de apertura. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-open-tracking-version = Versión del texto: seguimiento de apertura
admin-settings-help-consent-open-tracking-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-open-tracking-text = Redacción propia: seguimiento de apertura
admin-settings-help-consent-open-tracking-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-remote-content-basis = Base jurídica: imágenes remotas
admin-settings-help-consent-remote-content-basis = La base jurídica que se muestra a los usuarios para las imágenes remotas. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-remote-content-version = Versión del texto: imágenes remotas
admin-settings-help-consent-remote-content-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-remote-content-text = Redacción propia: imágenes remotas
admin-settings-help-consent-remote-content-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-integrations-basis = Base jurídica: servicios conectados
admin-settings-help-consent-integrations-basis = La base jurídica que se muestra a los usuarios para los servicios conectados. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-integrations-version = Versión del texto: servicios conectados
admin-settings-help-consent-integrations-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-integrations-text = Redacción propia: servicios conectados
admin-settings-help-consent-integrations-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-consent-telemetry-basis = Base jurídica: mediciones de tiempos de carga
admin-settings-help-consent-telemetry-basis = La base jurídica que se muestra a los usuarios para las mediciones de tiempos de carga. Solo el consentimiento hace que la función espere un sí; las demás bases se aplican salvo que la persona las retire. El administrador decide qué base es la adecuada.
admin-settings-key-consent-telemetry-version = Versión del texto: mediciones de tiempos de carga
admin-settings-help-consent-telemetry-version = Aumente este número cuando cambie la redacción o el tratamiento; las personas que aceptaron una versión anterior vuelven a ser consultadas.
admin-settings-key-consent-telemetry-text = Redacción propia: mediciones de tiempos de carga
admin-settings-help-consent-telemetry-text = Su propio texto para esta finalidad, por idioma. Vacío usa la redacción integrada.
admin-settings-key-breach-enabled = Registro de violaciones de seguridad de datos
admin-settings-help-breach-enabled = Activa el registro y el reloj de notificación.
admin-settings-key-breach-notice-hours = Horas para notificar a la autoridad
admin-settings-help-breach-notice-hours = Horas desde que se tiene conocimiento hasta la notificación a la autoridad. El plazo habitual es de 72.
admin-settings-key-breach-reminders-hours = Momentos de recordatorio
admin-settings-help-breach-reminders-hours = Horas antes del plazo que activan un recordatorio, por ejemplo 6, 24 y 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Repetición tras el vencimiento
admin-settings-help-breach-reminders-overdue-repeat-hours = Horas entre recordatorios una vez superado el plazo. 0 recuerda una sola vez.
admin-settings-key-breach-authority-name = Autoridad de control
admin-settings-help-breach-authority-name = Nombre impreso en la notificación a la autoridad.
admin-settings-key-breach-authority-url = Página de notificación de la autoridad
admin-settings-help-breach-authority-url = Dirección de la página de notificación de la autoridad.
admin-settings-key-breach-authority-contact = Contacto de la autoridad
admin-settings-help-breach-authority-contact = Datos de contacto de la autoridad.
admin-settings-key-breach-dpo-name = Nombre del contacto de protección de datos
admin-settings-help-breach-dpo-name = Nombre que se indica como punto de contacto en las notificaciones.
admin-settings-key-breach-dpo-email = Correo del contacto de protección de datos
admin-settings-help-breach-dpo-email = Dirección que se indica como punto de contacto en las notificaciones.
admin-settings-key-breach-dpo-phone = Teléfono del contacto de protección de datos
admin-settings-help-breach-dpo-phone = Número de teléfono que se indica en las notificaciones.
admin-settings-key-breach-template-subject = Asunto del mensaje
admin-settings-help-breach-template-subject = Asunto del mensaje a las personas afectadas, por idioma.
admin-settings-key-breach-template-body = Texto del mensaje
admin-settings-help-breach-template-body = Texto del mensaje a las personas afectadas, por idioma.
admin-settings-key-breach-notify-audit = Registrar las notificaciones en el registro de auditoría
admin-settings-help-breach-notify-audit = Los recordatorios y las detecciones se convierten en eventos de seguridad.
admin-settings-key-breach-notify-webhook-urls = Direcciones de webhook
admin-settings-help-breach-notify-webhook-urls = Direcciones a las que se avisa de los recordatorios y los mensajes. Trátelas como secretos; cada llamada va firmada.
admin-settings-key-breach-notify-operator-addresses = Buzones de operadores
admin-settings-help-breach-notify-operator-addresses = Direcciones de buzones locales que reciben los recordatorios en su propia bandeja de entrada.
admin-settings-key-breach-subjects-in-app = Aviso en la cuenta de la persona
admin-settings-help-breach-subjects-in-app = Muestra el mensaje dentro de la cuenta de la persona.
admin-settings-key-breach-subjects-inbox = Mensaje al buzón de la persona
admin-settings-help-breach-subjects-inbox = Entrega el mensaje en el propio buzón de la persona.
admin-settings-key-breach-subjects-webhook = Enviar también a webhooks
admin-settings-help-breach-subjects-webhook = Envía también el texto del mensaje a las direcciones de webhook.
admin-settings-key-breach-retention-years = Conservación del registro en años
admin-settings-help-breach-retention-years = Años que un incidente cerrado permanece en el registro.
admin-settings-key-breach-auto-enabled = Automatizaciones
admin-settings-help-breach-auto-enabled = Abre incidentes en borrador a partir de las detecciones.
admin-settings-key-breach-auto-start-clock = Reloj de los borradores
admin-settings-help-breach-auto-start-clock = Inicia el reloj cuando se abre el borrador, o solo cuando se confirma.
admin-settings-key-breach-auto-assignee = Asignar los borradores a
admin-settings-help-breach-auto-assignee = A quién se asigna un borrador.
admin-settings-key-breach-auto-escalate-after-hours = Escalar tras (horas)
admin-settings-help-breach-auto-escalate-after-hours = Horas que espera un borrador antes de escalarse. 0 nunca escala.
admin-settings-key-breach-auto-escalate-to = Escalar a
admin-settings-help-breach-auto-escalate-to = A quién se asigna un borrador escalado.
admin-settings-key-breach-auto-window-minutes = Ventana de recuento en minutos
admin-settings-help-breach-auto-window-minutes = Minutos durante los que una regla cuenta eventos.
admin-settings-key-breach-auto-theft-threshold = Alertas de robo de sesión
admin-settings-help-breach-auto-theft-threshold = Alertas dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spraying o pico de inicios de sesión
admin-settings-help-breach-auto-credential-stuffing-threshold = Eventos dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-mass-forward-threshold = Reglas de reenvío en masa
admin-settings-help-breach-auto-mass-forward-threshold = Eventos dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-role-escalation-threshold = Cambios de rol de administrador
admin-settings-help-breach-auto-role-escalation-threshold = Cambios dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-export-volume-threshold = Exportaciones inusuales
admin-settings-help-breach-auto-export-volume-threshold = Exportaciones dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-audit-chain-threshold = Fallos de la cadena de auditoría
admin-settings-help-breach-auto-audit-chain-threshold = Fallos que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-backup-anomaly-threshold = Fallos de copia de seguridad o restauración
admin-settings-help-breach-auto-backup-anomaly-threshold = Fallos dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-breach-auto-dsr-export-threshold = Muchas exportaciones de solicitudes de datos
admin-settings-help-breach-auto-dsr-export-threshold = Exportaciones dentro de la ventana que abren un borrador. 0 lo desactiva.
admin-settings-key-ratelimit-queue-ms = Espera máxima de un hueco libre
admin-settings-help-ratelimit-queue-ms = El tiempo máximo que una solicitud espera un hueco libre (concurrencia JMAP, decodificadores de imágenes, workers de vista previa) antes de rechazarse con 429 y Retry-After. Rango 0-30000 ms; por defecto 3000. Cero rechaza de inmediato.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Solicitudes JMAP simultáneas por cuenta
admin-settings-help-ratelimit-jmap-per-account-concurrent = Solicitudes JMAP que una cuenta con sesión iniciada puede ejecutar a la vez, entre sus pestañas y dispositivos. Nunca se comparten con otra cuenta ni con otro usuario tras la misma dirección. Rango 4-64; por defecto 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Solicitudes JMAP simultáneas, todas las cuentas
admin-settings-help-ratelimit-jmap-global-concurrent = Solicitudes JMAP que ejecuta a la vez todo el proceso del servidor, con todas las cuentas juntas. Una solicitud por encima espera el tiempo de cola. Rango 4-4096; por defecto 256.
admin-settings-key-ratelimit-session-burst = Informes de tiempos por sesión seguidos
admin-settings-help-ratelimit-session-burst = Informes de tiempos del cliente que una sesión puede enviar seguidos. El punto de conexión se indexa por sesión, nunca por dirección. Rango 1-1000; por defecto 60.
admin-settings-key-ratelimit-session-per-minute = Informes de tiempos recuperados por minuto
admin-settings-help-ratelimit-session-per-minute = Informes de tiempos del cliente por minuto que una sesión recupera tras su ráfaga. Rango 1-600; por defecto 30.
admin-settings-key-ratelimit-anon-burst = Solicitudes de consentimiento anónimas seguidas
admin-settings-help-ratelimit-anon-burst = Solicitudes que una dirección puede hacer seguidas a las rutas anónimas de consentimiento de cookies. Dimensionado para una oficina tras una sola dirección. Rango 1-1000; por defecto 60.
admin-settings-key-ratelimit-anon-per-minute = Solicitudes de consentimiento anónimas recuperadas por minuto
admin-settings-help-ratelimit-anon-per-minute = Solicitudes por minuto que una dirección recupera en las rutas anónimas de consentimiento de cookies. Rango 1-600; por defecto 30.
admin-settings-key-ratelimit-trusted-multiplier = Asignación de la cuenta desde un origen conocido (%)
admin-settings-help-ratelimit-trusted-multiplier = Porcentaje de la asignación de solicitudes de la cuenta que recibe un buzón con sesión iniciada cuando la solicitud proviene de un origen con un inicio de sesión correcto registrado. 100 mantiene la asignación base. Nunca relaja los presupuestos anónimo, de administrador o de user-agent ni un bloqueo activo. Rango 100-1000; por defecto 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Ya sea que al hacer clic en cualquier lugar de la barra de archivos adjuntos del lector se expanda o se contraiga. Cuando está apagado, solo lo hace el botón de flecha.

admin-settings-help-attachments-bar-download-all = Si la barra de archivos adjuntos ofrece un botón Descargar todo que guarda los archivos adjuntos permitidos como un archivo ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Cuando cada archivo adjunto en la barra muestra su botón de descarga: al pasar el mouse o enfocar (predeterminado), siempre o nunca. Las descargas aún siguen la política de archivos adjuntos y mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Si las imágenes que se muestran dentro del texto del mensaje están incluidas en Descargar todo ZIP. Desactivado de forma predeterminada.

admin-settings-help-cap-reading-delete-sender = Permita que los usuarios habiliten la acción del lector que mueve todos los mensajes coincidentes del remitente a la Papelera después de la confirmación. Deshabilitado por defecto. El perfil de característica también debe permitirlo.

admin-settings-help-compose-feature-family = Activa o desactiva un control de compositor mediante la identificación de la función. Activado por defecto; bloquearlo para evitar que los usuarios lo cambien.

admin-settings-help-compose-headers-allow-auto-submitted = Permite a los usuarios configurar el envío automático (RFC 3834) en un mensaje. Desactivado por defecto: indica a otros sistemas que no respondan.

admin-settings-help-compose-headers-allow-custom = Permite a los usuarios agregar sus propios encabezados (hasta 20, nombres verificados, encabezados protegidos rechazados).

admin-settings-help-compose-headers-allow-return-receipt-to = Permite a los usuarios solicitar recibos con el encabezado Devolución de recibo heredado. Desactivado de forma predeterminada.

admin-settings-help-compose-headers-allow-threading-edit = Permite a los usuarios editar respuestas y referencias, o iniciar un nuevo hilo a partir de una respuesta.

admin-settings-help-compose-headers-forced = Un par Nombre=valor por línea, agregado a cada mensaje enviado desde el dominio y reemplazando el valor del usuario. Se rechazan los encabezados protegidos.

admin-settings-help-compose-insert-image-resize-default = El tamaño con el que comienza el cuadro de diálogo Imagen desde archivo: original, grande (1600 px) o mediano (1024 px).

admin-settings-help-compose-insert-max-inline-image-mb = Las imágenes de mayor tamaño no se colocan en el cuerpo del mensaje; todavía se pueden adjuntar. 1 a 50.

admin-settings-help-compose-layout-default-font = Familia de fuentes, tamaño y color con el que comienza un nuevo mensaje, como familia|tamaño|#color. Vacío usa la fuente del tema.

admin-settings-help-compose-reply-to-allow-override = Cuando está desactivado, el Responder a de la identidad se utiliza tal cual y los usuarios no pueden editarlo ni eliminarlo.

admin-settings-help-compose-reply-to-show = Si el compositor muestra la fila Responder a: solo cuando una identidad o el mensaje original tiene una, siempre o nunca.

admin-settings-help-compose-sent-folder-fixed-id = Rol de carpeta o identificación utilizada cuando la regla es "carpeta fija".

admin-settings-help-compose-sent-folder-mode = Dónde va la copia de un mensaje enviado: la carpeta Enviado normal, una carpeta fija, por dominio del destinatario, por identidad o por reglas de asunto.

admin-settings-help-compose-sent-folder-per-identity = Un par identidad=carpeta por línea. Se utiliza cuando la regla es "por identidad".

admin-settings-help-compose-sent-folder-rules = Una regla por línea, domain:example.com=carpeta o subject:text=carpeta. El primer partido gana.

admin-settings-help-compose-sent-folder-skip-copy = Nunca guarde una copia de un mensaje enviado. Un administrador puede forzar que esto se active o desactive.

admin-settings-help-compose-styles-custom = Los estilos de compositor propios de un usuario como JSON, escritos por la galería de estilos. Los usuarios lo configuran; No hay nada que configurar aquí.

admin-settings-help-compose-styles-default-set = El aspecto de la galería de estilos del compositor: predeterminado, moderno, clásico o compacto.

admin-settings-help-compose-styles-max-custom = La mayoría de los estilos personalizados que un usuario puede conservar en el compositor. 0 desactiva Crear estilo.

admin-settings-help-contacts-save-from-message-add-note = Agregue la nota "Guardado desde un mensaje el <fecha>" a los contactos guardados desde un mensaje. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-contacts-save-from-message-default-book = La identificación de la libreta de direcciones que recibe los contactos guardados de un mensaje; "predeterminado" utiliza la libreta de direcciones predeterminada. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-contacts-save-from-message-default-scope = Qué participantes se marcan primero en el cuadro de diálogo Guardar contactos: solo el remitente, el remitente y Responder a, o todos. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-contacts-save-from-message-enabled = Oferta Guardar contactos de este mensaje en menús de mensajes. Cuando está apagado, el elemento está oculto. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-contacts-save-from-message-skip-automated = Deje las direcciones de no respuesta, notificación y rebote fuera del cuadro de diálogo Guardar contactos a menos que lo marque manualmente. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-contacts-save-sender-enabled = Ofrezca el elemento Guardar remitente como contacto con un solo clic en los menús de mensajes. Cuando está apagado, el elemento está oculto. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-contacts-save-sender-use-reply-to = Guarde la dirección Responder a en lugar de De cuando el remitente sea una lista de correo o una dirección de notificación. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-dsr-admin-step-up = Activado (predeterminado): un administrador que aprueba una solicitud de datos, ejecuta un borrado o aprueba de forma masiva debe primero confirmar que es él nuevamente (contraseña y segundo factor, o una clave de acceso). La confirmación dura cinco minutos. Desactivado elimina la marca adicional; la decisión aún se registra en el registro de solicitud.

admin-settings-help-export-default-format = El formato con el que comienza el diálogo de exportación.

admin-settings-help-export-default-scope = El alcance con el que comienza el diálogo de exportación: mensaje, conversación, selección o carpeta.

admin-settings-help-export-dialog-enabled = Exportar en la vista de mensajes y los menús de mensajes abre un cuadro de diálogo en su lugar. Cuando están desactivados, esos menús descargan un mensaje como .eml.

admin-settings-help-export-folder-enabled = Permita a los usuarios descargar una carpeta completa como ZIP de mensajes. Cada exportación es auditada.

admin-settings-help-export-folder-format = Los formatos que ofrece una exportación de carpetas: un ZIP de archivos .eml, un ZIP que contiene un archivo mbox, o ambos.

admin-settings-help-export-folder-include-subfolders = Si una exportación de carpeta incluye sus subcarpetas de forma predeterminada, mantenidas como una estructura de carpetas en ZIP.

admin-settings-help-export-folder-max-bytes = La mayor cantidad de bytes de mensajes que puede contener una exportación de carpeta. Se rechaza una carpeta más grande con el tamaño mostrado.

admin-settings-help-export-folder-max-messages = La mayor cantidad de mensajes que puede contener una exportación de carpeta.

admin-settings-help-export-formats = Los formatos que ofrece el cuadro de diálogo de exportación: eml, mbox, txt, md, msg, docx, csv, json, pdf. El servidor rechaza un formato creado por el servidor listado.

admin-settings-help-export-message-download-enabled = Permitir a los usuarios descargar un mensaje como .eml, solo sus encabezados o como un archivo mbox.

admin-settings-help-folders-hide-empty = Oculta carpetas vacías en la barra lateral. Los usuarios pueden cambiar el valor predeterminado o anularlo para carpetas individuales a menos que esta configuración esté bloqueada. No se elimina ningún correo.

admin-settings-help-folders-pinned-hide-from-tree = Las carpetas fijadas se muestran en un grupo Fijado encima del árbol de carpetas y también mantienen su lugar en el árbol. Activado oculta una carpeta fijada que no tiene subcarpetas en el árbol, por lo que se muestra solo en el grupo Fijado. Una carpeta fijada con subcarpetas permanece en el árbol.

admin-settings-help-mail-badge-auth = Cuando el encabezado del lector muestra la insignia de autenticación DMARC: siempre, solo cuando sea notable o nunca.

admin-settings-help-mail-badge-density = Tamaño de las credenciales en el cabezal del lector: compacto o cómodo.

admin-settings-help-mail-badge-folder = Cuando el encabezado del lector muestra la insignia de la carpeta: siempre, solo cuando sea notable o nunca.

admin-settings-help-mail-badge-headers = Muestre una insignia en el lector para los encabezados Caducar, Responder por, Sensibilidad y Reemplazar de un mensaje.

admin-settings-help-mail-badge-labels = Si las insignias de encabezado del lector muestran un ícono con texto o solo el ícono.

admin-settings-help-mail-badge-list = Cuando el encabezado del lector muestra la insignia de la lista de correo: siempre, solo cuando sea notable o nunca. Necesita un encabezado List-Id o List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Cuando el encabezado del lector muestra la insignia HTML desinfectada: siempre, solo cuando sea notable o nunca.

admin-settings-help-mail-badge-spam = Cuando el encabezado del lector muestra la insignia de puntuación de spam: siempre, solo cuando sea notable o nunca.

admin-settings-help-mail-brand-avatars-require-auth = nunca: muestra un logotipo siempre que el dominio coincida. error conocido (predeterminado): muéstrelo a menos que el resultado DKIM o DMARC almacenado en caché del mensaje sea un error; Los mensajes no marcados mantienen el logo. siempre: mostrarlo solo después de un pase DMARC alineado con DKIM para el dominio propio de la marca. Las anulaciones de usuarios y dominios se pueden bloquear.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Segundos en los que el cuerpo debe mantener el foco antes de que un encabezado completo de destinatario y asunto se pliegue en su resumen. Elija 0 para desactivar el plegado cronometrado, o 1 a 60 segundos. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-mail-compose-priority-default = La prioridad con la que comienza un nuevo mensaje en el compositor.

admin-settings-help-mail-compose-priority-enabled = Muestra el selector de prioridad Normal, Alta y Baja en el grupo de Opciones del compositor. Cuando está desactivado, el correo nuevo se envía sin encabezados de prioridad.

admin-settings-help-mail-expiry-hide-expired = Qué hacer con un mensaje cuya fecha de caducidad ya pasó: nada, mostrarlo atenuado o sugerir archivarlo.

admin-settings-help-mail-list-keep = Los remitentes y las listas de correo siempre se mantienen, uno por línea como sender:address o list:List-Id. Limpia este remitente, cancela sugerencias de suscripción y omítalas en las reglas de retención.

admin-settings-help-mail-list-page-size = Cuántos mensajes recupera la lista cada vez que un usuario se desplaza hasta el final. Valor predeterminado 50, de 20 a 200. Los usuarios eligen su propio valor a menos que lo bloquee.

admin-settings-help-mail-list-unsubscribe-mode = Cómo funciona cancelar la suscripción cuando una lista ofrece una solicitud con un solo clic: pregunte primero, envíela de inmediato o abra solo el enlace o el borrador de la lista.

admin-settings-help-mail-message-list-group = Agrupa conversaciones por fecha de recepción o envío, remitente, tema, importancia o tamaño. Los usuarios pueden anular esto a menos que estén bloqueados.

admin-settings-help-mail-priority-marks = Qué prioridad marca la lista de mensajes, las conversaciones y el lector: alta y baja, solo alta o ninguna. Los usuarios eligen en Configuración > Carpetas y barra lateral a menos que estén bloqueados.

admin-settings-help-mail-priority-marks-style = Cómo se ve una marca de prioridad: un ícono, un ícono con las palabras Alta o Baja, o una barra delgada. El icono y el texto mantienen el significado sin color.

admin-settings-help-mail-priority-send-headers = Qué encabezados recibe un mensaje de prioridad alta o baja cuando se envía: cualquiera de Importancia, Prioridad X y Prioridad X-MSMail.

admin-settings-help-mail-priority-trust-sender-headers = Si se muestra una prioridad establecida por el remitente: siempre, solo cuando el mensaje pasó una verificación DMARC alineada, o nunca. Utilice confiable o nunca contra remitentes que marcan cada mensaje como de alta prioridad.

admin-settings-help-mail-reply-from-header-priority = Los encabezados de los mensajes se leen, en este orden, para encontrar la dirección a la que se envió un mensaje al elegir la identidad para una respuesta. Nombres: Entregado a, X-Original-a, Sobre-a, X-Reenviado-a, Reenviado a, A, Cc. Un encabezado omitido no se lee.

admin-settings-help-mail-search-default-scope = Inicie nuevas búsquedas de correo normales en la carpeta actual o en todas las carpetas. Esto controla el valor predeterminado inicial, incluso cuando está bloqueado; Los usuarios aún pueden elegir un alcance diferente para una búsqueda en particular. Se conservan las consultas guardadas y de ámbito explícito.

admin-settings-help-metrics-dimensions-enabled = Almacene las estadísticas guardadas divididas por tipo de cliente (web) y clase de dispositivo (escritorio, móvil). Desactívelo para almacenar una fila combinada por métrica y minuto.

admin-settings-help-metrics-persist-enabled = Guarde las estadísticas de rendimiento (velocidad del proxy de imagen, tiempos de carga del navegador, límite de velocidad y recuentos de protección de solicitudes) en la base de datos cada intervalo de descarga y cuando el servidor se detenga, de modo que los gráficos mantengan su historial después de un reinicio. Nada identifica a una persona, dirección o mensaje. Desactívelo para mantenerlos solo en la memoria.

admin-settings-help-metrics-persist-flush-seconds = Con qué frecuencia se escriben las estadísticas de rendimiento en la base de datos (de 5 a 3600 segundos). Un servidor que muere sin previo aviso pierde como máximo esta cantidad.

admin-settings-help-observability-client-capture-console = También informe las llamadas de console.error, como máximo cinco por minuto por página. Desactivado de forma predeterminada porque las bibliotecas registran las condiciones esperadas.

admin-settings-help-observability-client-capture-csp = Informe las violaciones de la Política de seguridad de contenido que detecte el navegador, con la directiva bloqueada y el origen únicamente. Activado de forma predeterminada.

admin-settings-help-observability-client-enabled = Recopile informes de errores de la aplicación web (límites de errores, errores no detectados, fragmentos fallidos, errores de trabajadores, infracciones CSP) con seguimientos de pila eliminados, que se muestran en Observabilidad > Errores del cliente. Activado de forma predeterminada. Los informes no contienen contenido de mensajes, direcciones, tokens, nombres de archivos adjuntos ni consultas URL. Apague para detener la recolección; los informes existentes caducan con su retención.

admin-settings-help-observability-client-forward = Envíe también cada informe eliminado al receptor de errores configurado con MW_ERROR_FORWARD_URL. Desactivado de forma predeterminada. Sin un receptor configurado no se envía nada.

admin-settings-help-observability-client-include-account-hash = Agregue un hash unidireccional con clave de la cuenta a los informes enviados desde una sesión iniciada, de modo que los informes de una cuenta se puedan diferenciar sin nombrarla. Desactivado de forma predeterminada. El hash cambia cuando gira la clave de privacidad.

admin-settings-help-observability-client-max-per-session = La mayoría de los informes de error se pueden enviar al cargar una página (de 1 a 1000). Las repeticiones del mismo error dentro de 60 segundos cuentan una vez. Por defecto 20.

admin-settings-help-observability-client-sample-rate = Fracción de informes de error que envía un navegador, de 0 a 1. Valor predeterminado 1 (todos). La carga de cada página aún se detiene en el límite por sesión.

admin-settings-help-observability-client-source-maps = Resuelva las posiciones de los paquetes en los seguimientos de la pila de los archivos fuente cuando la compilación envió mapas fuente ocultos (creados con MW_SOURCEMAP=hidden). Los mapas solo los lee el servidor y nunca se muestran a los navegadores. Activado de forma predeterminada; sin mapas se muestran los marcos sin formato.

admin-settings-help-privacy-error-reports = Si un navegador envía un informe de error cuando falla parte de la aplicación. Activado de forma predeterminada. Ámbito global, de dominio y de usuario: establezca el valor predeterminado aquí o bloquéelo; las personas pueden desactivarlo por sí mismas en Configuración > Privacidad a menos que esté bloqueado.

admin-settings-help-privacy-image-cache-max-stale-secs = Segundos después de su frescura, se guarda una imagen en caché para responder cuando no se puede alcanzar el origen. Nunca por debajo de la ventana de revalidación. Rango 0–604800; predeterminado 86400.

admin-settings-help-privacy-image-cache-stale-secs = Segundos después de su frescura, una imagen almacenada en caché todavía se muestra al mismo tiempo mientras se revalida en segundo plano. Zero lo apaga. Rango 0–86400; predeterminado 3600.

admin-settings-help-privacy-image-resize-enabled = Escale imágenes remotas de gran tamaño hasta el ancho que solicite el lector (320, 640, 960, 1280 o 1600 píxeles). Las imágenes animadas se sirven sin escala. Desactivado de forma predeterminada.

admin-settings-help-privacy-image-resize-format = Auto elige el mejor codificador de la compilación, que es PNG. Automático predeterminado.

admin-settings-help-privacy-image-resize-max-pixels = Las imágenes con más píxeles decodificados se muestran sin escala; el tamaño se lee en el encabezado antes de cualquier decodificación. Rango 100000–64000000; predeterminado 4000000.

admin-settings-help-privacy-image-resize-max-width = La imagen más amplia que produce el cambio de tamaño. Rango 320–1600; predeterminado 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Cómo se nombra en Configuración > Seguridad > Sesiones la cuenta de la que proviene una sesión de acceso al buzón vinculado: la dirección completa, la dirección con la primera parte del nombre enmascarada o nada. El valor se muestra únicamente al propietario de las sesiones, nunca a otros usuarios. Predeterminado: dirección completa.

admin-settings-help-properties-folder-enabled = Oferta Propiedades en los menús de carpetas y en el acceso directo. Cuando está desactivado, el cuadro de diálogo no está disponible.

admin-settings-help-properties-folder-notes-enabled = Permitir una descripción o nota en cada carpeta.

admin-settings-help-properties-folder-notes-max-length = La mayor cantidad de caracteres que puede contener una nota de carpeta.

admin-settings-help-properties-folder-overrides-allowed = Qué configuraciones un usuario puede anular para una sola carpeta. Una anulación que no aparece en la lista se oculta y se ignora cualquier valor almacenado.

admin-settings-help-properties-folder-sections = Las secciones del cuadro de diálogo Propiedades de la carpeta: general, remota, tamaño, fuera de línea, anulaciones, notas, acciones. Una sección que no aparece en la lista está oculta.

admin-settings-help-properties-folder-size-compute = Cuando se calcula el tamaño de una carpeta en el servidor: nunca, cuando se abre el cuadro de diálogo o en segundo plano. Contar un buzón grande le cuesta al servidor un escaneo.

admin-settings-help-properties-headers-raw = Permita que el cuadro de diálogo Propiedades del mensaje enumere los encabezados sin formato. Cuando está desactivado, solo se muestra el resumen analizado.

admin-settings-help-properties-message-enabled = Oferta Propiedades en los menús de mensajes y en el acceso directo. Cuando está desactivado, el cuadro de diálogo no está disponible.

admin-settings-help-properties-message-permanent-delete = Si el cuadro de diálogo del mensaje puede eliminar un mensaje definitivamente: permitirlo, preguntarlo dos veces o rechazarlo. Un mensaje bajo retención legal nunca se elimina. La eliminación de esta manera no se puede deshacer.

admin-settings-help-properties-message-sections = Las secciones del cuadro de diálogo Propiedades del mensaje: resumen, puntuación, encabezados, archivos adjuntos, acciones. Una sección que no aparece en la lista está oculta.

admin-settings-help-properties-offline-clear-enabled = Permita a los usuarios borrar el caché sin conexión de una sola carpeta desde su cuadro de diálogo Propiedades.

admin-settings-help-properties-remote-location-visibility = Qué parte de la ubicación del lado del servidor ven los usuarios: completa (host y ruta), host enmascarado (host parcialmente oculto), solo ruta (ruta del buzón sin el host) u oculta.

admin-settings-help-properties-scoring-show = Muestre la puntuación de spam, los resultados de autenticación y las señales de seguridad en el cuadro de diálogo Propiedades del mensaje.

admin-settings-help-properties-shortcut-enabled = Abra el cuadro de diálogo Propiedades desde el teclado en una carpeta enfocada o mensaje seleccionado.

admin-settings-help-properties-shortcut-key = La combinación de teclas que abre Propiedades; Alt+Entrar de forma predeterminada.

admin-settings-help-reader-actions-archive = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-block-sender = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-colors = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-conversation = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-create-filter = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-delete = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-delete-sender = Oculto por defecto. Elija la barra de herramientas o el menú Más para habilitarlo cuando la capacidad de eliminación del remitente y el perfil de funciones lo permitan. Los usuarios pueden anular la ubicación a menos que estén bloqueados.

admin-settings-help-reader-actions-edit-draft = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-export = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-finish-followup = Elija la barra de herramientas, el menú Más u oculto para completar mensajes marcados y recordatorios de seguimiento.

admin-settings-help-reader-actions-flag = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-focus-focused = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-focus-other = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-forward = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-headers = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-links = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-move = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-not-spam = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-open-draft = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-pin = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-print = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-properties = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-read = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-reply = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-reply-all = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-save-contacts = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-save-sender = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-search = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-security = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-spam = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-sweep = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-view-source = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-reader-actions-zoom = Elija barra de herramientas, menú Más u oculto. Se siguen aplicando restricciones de funciones, permisos y contexto de mensajes. Los usuarios pueden elegir a menos que esté bloqueado.

admin-settings-help-retention-client-errors-event-days = Días en que se mantiene una ocurrencia de error de navegador individual (su seguimiento de pila y clase de navegador). Predeterminado 30. Un grupo se mantiene por más tiempo; consulte la siguiente configuración.

admin-settings-help-retention-client-errors-group-days = Días que se conserva un grupo de errores del navegador (nombre, mensaje, recuentos) después de su última visualización. Predeterminado 180. Nunca más corto que la retención de ocurrencia.

admin-settings-help-retention-metrics-day-buckets-days = Días de estadísticas de rendimiento diario que se conservan antes de que se eliminen. 0 los mantiene sin límite, lo que permite que la tabla crezca lentamente mientras el servidor esté en funcionamiento.

admin-settings-help-retention-metrics-hour-buckets-days = Días de estadísticas de rendimiento por hora que se mantienen antes de fusionarse en filas diarias (al menos 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Horas de estadísticas de rendimiento de un minuto que se conservan antes de fusionarlas en filas por horas (al menos 1).

admin-settings-help-retention-user-allow-strip-attachments = Si los usuarios pueden crear reglas que eliminen archivos adjuntos grandes del correo antiguo. Desactivado de forma predeterminada. Mientras está desactivada, la acción está oculta en el editor de reglas, una regla que la usa se rechaza cuando se guarda y las reglas guardadas anteriormente se omiten cuando se ejecuta el trabajo.

admin-settings-help-retention-user-calendar-enabled = Si los usuarios pueden elegir calendarios cuyos eventos se eliminen una cantidad determinada de días después de su finalización. Desactivado de forma predeterminada. Mientras está desactivada, la opción está oculta en Configuración, la opción se rechaza cuando se guarda y las opciones guardadas anteriormente no se ejecutan.

admin-settings-help-retention-user-calendar-max-days = La mayoría de los días un usuario puede esperar antes de que se eliminen los eventos finalizados. Un valor mayor se rechaza cuando se guarda y se reduce cuando se ejecuta.

admin-settings-help-retention-user-calendar-min-days = Menos días después de que finalizó un evento antes de que una regla de usuario pueda eliminarlo. Un valor menor se rechaza cuando se guarda y se aumenta cuando se ejecuta.

admin-settings-help-retention-user-notes-enabled = Si los usuarios pueden optar por eliminar las notas que no han cambiado durante un número determinado de días. Desactivado de forma predeterminada. Las notas eliminadas se pueden restaurar hasta que finalice la ventana de deshacer para las reglas de retención.

admin-settings-help-retention-user-notes-max-days = La mayoría de los días, un usuario puede esperar antes de que se eliminen las notas intactas. Un valor mayor se rechaza cuando se guarda y se reduce cuando se ejecuta.

admin-settings-help-retention-user-notes-min-days = La menor cantidad de días que una nota debe permanecer intacta antes de que una regla de usuario pueda eliminarla. Un valor menor se rechaza cuando se guarda y se aumenta cuando se ejecuta.

admin-settings-help-retention-user-strip-keep-inline-images = Si las imágenes que se muestran dentro del cuerpo del mensaje se conservan cuando se eliminan los archivos adjuntos.

admin-settings-help-retention-user-strip-min-age-days = El menor número de días que debe tener un mensaje antes de que una regla pueda eliminar sus archivos adjuntos. Un valor menor se rechaza cuando la regla se guarda y se genera cuando se ejecuta.

admin-settings-help-retention-user-strip-min-attachment-kb = Sólo se eliminan los archivos adjuntos de al menos este tamaño. Los archivos adjuntos más pequeños permanecen con el mensaje.

admin-settings-help-retention-user-strip-undo-days = Días que el mensaje original permanece en la Papelera después de que se eliminaron sus archivos adjuntos. -1 sigue a la ventana de deshacer para las reglas de retención. 0 elimina el original de una vez.

admin-settings-help-server-shutdown-grace-seconds = Cuánto tiempo un servidor detenido permite que finalicen las solicitudes en curso antes de cerrar las conexiones, en segundos. La variable MW_HTTP_SHUTDOWN_TIMEOUT_MS lo anula. Aplica en la próxima salida.

admin-settings-help-session-last-activity-flush-seconds = La hora de la última vez que se vio una sesión se escribe como máximo con esta frecuencia, en segundos. Zero lo escribe en cada solicitud. No cambia los límites inactivos o absolutos. Aplica en la próxima salida.

admin-settings-help-session-persist-across-restarts = Cuando está activado, las sesiones de buzón, aplicación y consola siguen siendo válidas después de que se reinicia el servidor. Cuando está apagado, cada sesión finaliza cuando se inicia el servidor. Aplica en la próxima salida.

admin-settings-key-attachments-bar-click-to-toggle = Haga clic en la barra de archivos adjuntos para alternar

admin-settings-key-attachments-bar-download-all = botón descargar todo

admin-settings-key-attachments-bar-show-download-buttons = Botones de descarga de archivos adjuntos

admin-settings-key-attachments-bar-zip-inline-images = Imágenes en línea en Descargar todo ZIP

admin-settings-key-cap-reading-delete-sender = Permitir eliminar todo el correo de un remitente

admin-settings-key-compose-feature-family = Características del compositor

admin-settings-key-compose-headers-allow-auto-submitted = Permitir envío automático

admin-settings-key-compose-headers-allow-custom = Permitir encabezados personalizados

admin-settings-key-compose-headers-allow-return-receipt-to = Permitir acuse de recibo a

admin-settings-key-compose-headers-allow-threading-edit = Permitir editar encabezados de subprocesos

admin-settings-key-compose-headers-forced = Cabezazos forzados

admin-settings-key-compose-insert-image-resize-default = Preajuste de tamaño de imagen

admin-settings-key-compose-insert-max-inline-image-mb = Imagen en línea más grande (MB)

admin-settings-key-compose-layout-default-font = Fuente de mensaje predeterminada

admin-settings-key-compose-reply-to-allow-override = Permitir cambiar Responder a

admin-settings-key-compose-reply-to-show = Fila Responder a

admin-settings-key-compose-sent-folder-fixed-id = Copia enviada: carpeta fija

admin-settings-key-compose-sent-folder-mode = Copia enviada: regla de carpeta

admin-settings-key-compose-sent-folder-per-identity = Copia enviada: carpeta por identidad

admin-settings-key-compose-sent-folder-rules = Copia enviada: reglas

admin-settings-key-compose-sent-folder-skip-copy = Copia enviada: no guardar

admin-settings-key-compose-styles-custom = Estilos personalizados

admin-settings-key-compose-styles-default-set = Conjunto de estilos

admin-settings-key-compose-styles-max-custom = Estilos personalizados por usuario

admin-settings-key-contacts-save-from-message-add-note = Nota sobre los contactos guardados de un mensaje

admin-settings-key-contacts-save-from-message-default-book = Libreta de direcciones para contactos guardados de un mensaje

admin-settings-key-contacts-save-from-message-default-scope = Los contactos se marcan primero al guardar desde un mensaje

admin-settings-key-contacts-save-from-message-enabled = Guardar contactos de un mensaje

admin-settings-key-contacts-save-from-message-skip-automated = Omitir remitentes automáticos al guardar contactos

admin-settings-key-contacts-save-sender-enabled = Guardar remitente como contacto

admin-settings-key-contacts-save-sender-use-reply-to = Guardar Responder a para los remitentes de la lista

admin-settings-key-dsr-admin-step-up = Confirmar nuevamente antes de aprobar

admin-settings-key-export-default-format = Formato predeterminado del cuadro de diálogo de exportación

admin-settings-key-export-default-scope = Alcance predeterminado del diálogo de exportación

admin-settings-key-export-dialog-enabled = Diálogo de exportación

admin-settings-key-export-folder-enabled = Exportación de carpetas

admin-settings-key-export-folder-format = Formato de exportación de carpetas

admin-settings-key-export-folder-include-subfolders = Incluir subcarpetas en la exportación

admin-settings-key-export-folder-max-bytes = Límite de tamaño de exportación de carpeta

admin-settings-key-export-folder-max-messages = Límite de mensajes de exportación de carpetas

admin-settings-key-export-formats = Exportar formatos de diálogo

admin-settings-key-export-message-download-enabled = Descarga de mensajes

admin-settings-key-folders-hide-empty = Ocultar carpetas vacías

admin-settings-key-folders-pinned-hide-from-tree = Ocultar carpetas ancladas del árbol

admin-settings-key-mail-badge-auth = Insignia de autenticación

admin-settings-key-mail-badge-density = Tamaño de la credencial del lector

admin-settings-key-mail-badge-folder = Insignia de carpeta

admin-settings-key-mail-badge-headers = Insignias de encabezado

admin-settings-key-mail-badge-labels = Etiquetas de credenciales de lector

admin-settings-key-mail-badge-list = Insignia de lista de correo

admin-settings-key-mail-badge-sanitized = Insignia HTML desinfectada

admin-settings-key-mail-badge-spam = Insignia de puntuación de spam

admin-settings-key-mail-brand-avatars-require-auth = Se necesita autenticación para los logotipos de marcas.

admin-settings-key-mail-compose-header-auto-collapse-seconds = Retraso en el colapso automático del encabezado del compositor

admin-settings-key-mail-compose-priority-default = Prioridad de mensaje predeterminada

admin-settings-key-mail-compose-priority-enabled = Selector de prioridad en el compositor.

admin-settings-key-mail-expiry-hide-expired = Mensajes caducados

admin-settings-key-mail-list-keep = mantener siempre

admin-settings-key-mail-list-page-size = Tamaño de página de la lista de mensajes

admin-settings-key-mail-list-unsubscribe-mode = Lista de correo darse de baja

admin-settings-key-mail-message-list-group = Agrupación de mensajes

admin-settings-key-mail-priority-marks = Marcas de prioridad

admin-settings-key-mail-priority-marks-style = Estilo de marca de prioridad

admin-settings-key-mail-priority-send-headers = Encabezados de prioridad escritos en el envío

admin-settings-key-mail-priority-trust-sender-headers = Confiar en la prioridad establecida por el remitente

admin-settings-key-mail-reply-from-header-priority = Encabezados que eligen la identidad de respuesta

admin-settings-key-mail-search-default-scope = Ámbito de búsqueda de correo predeterminado

admin-settings-key-metrics-dimensions-enabled = Dividir estadísticas por tipo de cliente y clase de dispositivo

admin-settings-key-metrics-persist-enabled = Mantenga estadísticas entre reinicios

admin-settings-key-metrics-persist-flush-seconds = Intervalo para guardar estadísticas (segundos)

admin-settings-key-observability-client-capture-console = Informar errores de la consola

admin-settings-key-observability-client-capture-csp = Informar violaciones CSP

admin-settings-key-observability-client-enabled = Informes de errores del navegador

admin-settings-key-observability-client-forward = Reenviar informes de errores

admin-settings-key-observability-client-include-account-hash = Agregar hash de cuenta a los informes de errores

admin-settings-key-observability-client-max-per-session = Informes de errores por carga de página

admin-settings-key-observability-client-sample-rate = Frecuencia de muestreo del informe de errores

admin-settings-key-observability-client-source-maps = Resolver seguimientos de pila con mapas de origen

admin-settings-key-privacy-error-reports = Enviar informes de errores

admin-settings-key-privacy-image-cache-max-stale-secs = Mantener obsoleto cuando falla el origen (segundos)

admin-settings-key-privacy-image-cache-stale-secs = Servir rancio mientras se revalida (segundos)

admin-settings-key-privacy-image-resize-enabled = Cambiar el tamaño de imágenes grandes

admin-settings-key-privacy-image-resize-format = Formato de imagen redimensionado

admin-settings-key-privacy-image-resize-max-pixels = Imagen más grande para cambiar de tamaño (píxeles)

admin-settings-key-privacy-image-resize-max-width = Imagen redimensionada más amplia (píxeles)

admin-settings-key-privacy-sessions-show-linked-origin = Origen del buzón vinculado en sesiones

admin-settings-key-properties-folder-enabled = Cuadro de diálogo de propiedades de carpeta

admin-settings-key-properties-folder-notes-enabled = notas de carpeta

admin-settings-key-properties-folder-notes-max-length = Longitud de la nota de la carpeta

admin-settings-key-properties-folder-overrides-allowed = Se permiten anulaciones por carpeta

admin-settings-key-properties-folder-sections = Secciones de diálogo de carpeta

admin-settings-key-properties-folder-size-compute = Cálculo del tamaño de la carpeta

admin-settings-key-properties-headers-raw = encabezados sin procesar

admin-settings-key-properties-message-enabled = Diálogo de propiedades del mensaje

admin-settings-key-properties-message-permanent-delete = Eliminación permanente de propiedades

admin-settings-key-properties-message-sections = Secciones de diálogo de mensajes

admin-settings-key-properties-offline-clear-enabled = Borrar caché sin conexión por carpeta

admin-settings-key-properties-remote-location-visibility = Detalle de ubicación remota

admin-settings-key-properties-scoring-show = Detalles de puntuación y autenticación

admin-settings-key-properties-shortcut-enabled = Acceso directo a propiedades

admin-settings-key-properties-shortcut-key = Tecla de método abreviado de propiedades

admin-settings-key-reader-actions-archive = Ubicación de la acción de archivo

admin-settings-key-reader-actions-block-sender = Bloquear la ubicación de la acción del remitente

admin-settings-key-reader-actions-colors = Ubicación de la acción de colores del mensaje

admin-settings-key-reader-actions-conversation = Ver la ubicación de la acción de la conversación

admin-settings-key-reader-actions-create-filter = Crear ubicación de acción de filtro

admin-settings-key-reader-actions-delete = Eliminar ubicación de acción

admin-settings-key-reader-actions-delete-sender = Eliminar todos los mensajes de la acción del remitente

admin-settings-key-reader-actions-edit-draft = Editar la ubicación del borrador de la acción

admin-settings-key-reader-actions-export = Ubicación de la acción de exportación

admin-settings-key-reader-actions-finish-followup = Marcar el seguimiento como acción finalizada

admin-settings-key-reader-actions-flag = Ubicación de la acción marcar/desmarcar

admin-settings-key-reader-actions-focus-focused = Mover a ubicación de acción enfocada

admin-settings-key-reader-actions-focus-other = Mover a otra ubicación de acción

admin-settings-key-reader-actions-forward = Colocación de acción hacia adelante

admin-settings-key-reader-actions-headers = Colocación de acciones de encabezados sin formato

admin-settings-key-reader-actions-links = Revisar la ubicación de la acción de los enlaces

admin-settings-key-reader-actions-move = Mover ubicación de acción

admin-settings-key-reader-actions-not-spam = Ubicación de acción no spam

admin-settings-key-reader-actions-open-draft = Abrir ubicación de acción de borrador de respuesta

admin-settings-key-reader-actions-pin = Ubicación de la acción de fijar/desanclar

admin-settings-key-reader-actions-print = Ubicación de la acción de impresión

admin-settings-key-reader-actions-properties = Ubicación de la acción de propiedades del mensaje

admin-settings-key-reader-actions-read = Ubicación de acción leída/no leída

admin-settings-key-reader-actions-reply = Ubicación de la acción de respuesta

admin-settings-key-reader-actions-reply-all = Responder a todas las acciones

admin-settings-key-reader-actions-save-contacts = Guardar ubicación de la acción de los contactos del mensaje

admin-settings-key-reader-actions-save-sender = Guardar ubicación de la acción del remitente

admin-settings-key-reader-actions-search = Ubicación de la acción de búsqueda relacionada

admin-settings-key-reader-actions-security = Ubicación de la acción en modo de seguridad

admin-settings-key-reader-actions-spam = Marcar como ubicación de acción spam

admin-settings-key-reader-actions-sweep = Barrer la ubicación de la acción del remitente

admin-settings-key-reader-actions-view-source = Ver ubicación de la acción de origen del mensaje

admin-settings-key-reader-actions-zoom = Ubicación de la acción de zoom

admin-settings-key-retention-client-errors-event-days = Ocurrencias de errores mantenidas (días)

admin-settings-key-retention-client-errors-group-days = Grupos de errores mantenidos (días)

admin-settings-key-retention-metrics-day-buckets-days = Estadísticas diarias mantenidas (días)

admin-settings-key-retention-metrics-hour-buckets-days = Estadísticas horarias mantenidas (días)

admin-settings-key-retention-metrics-minute-buckets-hours = Estadísticas de minutos guardadas (horas)

admin-settings-key-retention-user-allow-strip-attachments = Los usuarios pueden eliminar archivos adjuntos del correo antiguo.

admin-settings-key-retention-user-calendar-enabled = Los usuarios pueden eliminar eventos del calendario finalizados.

admin-settings-key-retention-user-calendar-max-days = La mayoría de los días antes de que se eliminen los eventos finalizados

admin-settings-key-retention-user-calendar-min-days = Se eliminan el menor número de días antes de que se eliminen los eventos finalizados

admin-settings-key-retention-user-notes-enabled = Los usuarios pueden eliminar notas intactas.

admin-settings-key-retention-user-notes-max-days = La mayoría de los días antes de que se eliminen las notas intactas

admin-settings-key-retention-user-notes-min-days = Menos días antes de que se eliminen las notas intactas

admin-settings-key-retention-user-strip-keep-inline-images = Mantener imágenes en línea

admin-settings-key-retention-user-strip-min-age-days = Menos días antes de que se eliminen los archivos adjuntos

admin-settings-key-retention-user-strip-min-attachment-kb = Se eliminó el archivo adjunto más pequeño (KB)

admin-settings-key-retention-user-strip-undo-days = Ventana para deshacer después de eliminar los archivos adjuntos (días)

admin-settings-key-server-shutdown-grace-seconds = Período de gracia de cierre

admin-settings-key-session-last-activity-flush-seconds = Intervalo de escritura visto por última vez

admin-settings-key-session-persist-across-restarts = Mantener sesiones durante los reinicios

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = Desactivado de forma predeterminada. Guarde instantáneas de correo cifradas con ámbito de cuenta durante las recargas del navegador sin vencimiento. Se siguen aplicando los límites de entrada y bytes existentes. Los usuarios pueden borrar las copias guardadas; Al deshabilitar esta preferencia, se eliminan del navegador activo. Esto no otorga permiso de almacenamiento persistente al navegador ni descarga archivos adjuntos.

admin-settings-help-mail-conversations-allowed = Permita que los usuarios agrupen respuestas relacionadas en conversaciones. Disponible de forma predeterminada, pero comienza la preferencia del usuario. Al deshabilitar la disponibilidad se anula la elección del usuario sin cambiar la agrupación de categorías.

admin-settings-help-mail-conversations-enabled = Desactivado de forma predeterminada. Agrupar respuestas cargadas por hilo, independientemente de Agrupar por. Establezca el valor predeterminado o bloquee esta preferencia. También se debe habilitar la disponibilidad de conversación.

admin-settings-key-mail-cache-keep-indefinitely = Mantener la caché de correo del navegador indefinidamente

admin-settings-key-mail-conversations-allowed = Hacer que los hilos de conversación estén disponibles

admin-settings-key-mail-conversations-enabled = Agrupar mensajes en conversaciones

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = Hasta 100 dominios de remitentes exactos en minúsculas cuyos enlaces de mensajes revisados se habilitan automáticamente. Los subdominios están separados. Esto no confía en los sitios web de destino ni permite imágenes remotas. Vacío por defecto; los usuarios pueden cambiarlo a menos que estén bloqueados.

admin-settings-key-links-trusted-sender-domains = Dominios de remitente confiables para enlaces de mensajes

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Muestre los detalles de Descarga y Archivo junto al título de la vista previa como íconos (predeterminado), etiquetas o íconos y etiquetas. Se puede configurar y bloquear globalmente, por dominio o por usuario.

admin-settings-help-reader-attachment-thumbnails = Desactivado de forma predeterminada. Permita que los archivos adjuntos de imágenes visibles carguen vistas previas en miniatura. Se siguen aplicando las reglas de seguridad de los archivos adjuntos. Se puede configurar y bloquear globalmente, por dominio o por usuario.

admin-settings-key-reader-attachment-action-style = Acciones de vista previa de archivos adjuntos

admin-settings-key-reader-attachment-thumbnails = Miniaturas adjuntas de imágenes

admin-settings-key-mail-compose-auto-add-mention-recipients = Añadir automáticamente a las personas mencionadas como destinatarios
admin-settings-help-mail-compose-auto-add-mention-recipients = Al seleccionar una persona de las sugerencias de @menciones, se añade a Para si aún no está en Para, Cc o Cco. Desactivado de forma predeterminada.

admin-settings-linked-disable-title = ¿Desactivar el acceso guardado a las cuentas vinculadas?
admin-settings-linked-disable-body = Guardar este cambio elimina las credenciales que renuevan el acceso a las cuentas vinculadas. Los usuarios deberán iniciar sesión de nuevo cuando sea necesario renovar el acceso. Reactivar esta opción no restaura esas credenciales.
admin-settings-linked-disable-confirm = Desactivar y borrar el acceso guardado

admin-settings-key-reader-auto-hide-header = Ocultar automáticamente las cabeceras de los mensajes
admin-settings-help-reader-auto-hide-header = Abrir cada mensaje con los detalles del remitente, destinatarios y metadatos contraídos. El asunto y las acciones siguen visibles. Usa Mostrar cabecera para ver los detalles. Desactivado de forma predeterminada.

admin-settings-key-time-region = Región
admin-settings-help-time-region = auto o un código de región de dos letras, como GB, US o PT.

admin-settings-key-time-date-format = Formato de fecha
admin-settings-help-time-date-format = auto, short, medium, long, full o un patrón como yyyy-MM-dd o dd/MM/yyyy. Pon el texto literal entre 'comillas'.

admin-settings-key-time-time-format = Formato de hora
admin-settings-help-time-time-format = auto, short, medium, long, full o HH:mm:ss / h:mm a. H = 24 horas, h = 12 horas, a = AM/PM.

admin-settings-key-time-hour-cycle = Reloj
admin-settings-help-time-hour-cycle = Los símbolos H o h de un patrón personalizado tienen prioridad.

admin-settings-key-time-first-day-of-week = Primer día de la semana
admin-settings-help-time-first-day-of-week = Automático sigue las convenciones regionales del calendario.

admin-settings-key-compliance-mail-enabled = Activar correo
admin-settings-help-compliance-mail-enabled = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-host = Servidor SMTP
admin-settings-help-compliance-mail-host = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-port = Puerto SMTP
admin-settings-help-compliance-mail-port = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-security = Modo TLS
admin-settings-help-compliance-mail-security = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-from = Dirección del remitente
admin-settings-help-compliance-mail-from = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-username = Usuario SMTP
admin-settings-help-compliance-mail-username = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-allowPrivate = Permitir servidor privado
admin-settings-help-compliance-mail-allowPrivate = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-dsr-notify-emailAdmins = Notificar a operadores de solicitudes de datos
admin-settings-help-dsr-notify-emailAdmins = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-dsr-notify-operatorAddresses = Destinatarios operadores
admin-settings-help-dsr-notify-operatorAddresses = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-dsr-notify-emailRequester = Notificar al solicitante por correo
admin-settings-help-dsr-notify-emailRequester = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-breach-notify-email = Notificar a operadores de brechas
admin-settings-help-breach-notify-email = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-breach-subjects-email = Notificar a personas afectadas por correo
admin-settings-help-breach-subjects-email = Configure un servidor SMTP con TLS. El correo usa la ruta proxy system_mail. Revise las aceptaciones inciertas antes de reenviar.

admin-settings-key-compliance-mail-requireProxy = Exigir ruta proxy
admin-settings-help-compliance-mail-requireProxy = Exigir ruta proxy (system_mail).
admin-settings-key-retention-complianceMail-days = Retención del historial de correos completados (días)
admin-settings-help-retention-complianceMail-days = Días de conservación de entregas completadas. Las pendientes e inciertas se conservan hasta su resolución.

admin-settings-key-incidents-auto-enabled = Detectar posibles incidentes automáticamente
admin-settings-help-incidents-auto-enabled = Los fallos de salud y las alertas de sesión crean posibles incidentes. Evalúe su importancia y verifique la recuperación. Una falta de monitorización no demuestra una caída.
admin-settings-key-incidents-auto-failureSamples = Muestras fallidas antes de detectar
admin-settings-help-incidents-auto-failureSamples = Los fallos de salud y las alertas de sesión crean posibles incidentes. Evalúe su importancia y verifique la recuperación. Una falta de monitorización no demuestra una caída.
admin-settings-key-incidents-nis2-trustService = Usar plazo de servicios de confianza
admin-settings-help-incidents-nis2-trustService = Los fallos de salud y las alertas de sesión crean posibles incidentes. Evalúe su importancia y verifique la recuperación. Una falta de monitorización no demuestra una caída.
admin-settings-key-incidents-notify-email = Enviar actualizaciones y recordatorios por correo
admin-settings-help-incidents-notify-email = Los fallos de salud y las alertas de sesión crean posibles incidentes. Evalúe su importancia y verifique la recuperación. Una falta de monitorización no demuestra una caída.
admin-settings-key-incidents-notify-operatorAddresses = Destinatarios de operaciones
admin-settings-help-incidents-notify-operatorAddresses = Los fallos de salud y las alertas de sesión crean posibles incidentes. Evalúe su importancia y verifique la recuperación. Una falta de monitorización no demuestra una caída.
admin-settings-key-incidents-retention-days = Retención de incidentes cerrados (días)
admin-settings-help-incidents-retention-days = Los fallos de salud y las alertas de sesión crean posibles incidentes. Evalúe su importancia y verifique la recuperación. Una falta de monitorización no demuestra una caída.

admin-settings-key-mail-snooze-presets = Opciones de posponer
admin-settings-help-mail-snooze-presets = Añade, renombra, reordena, edita u oculta opciones. Siempre puedes elegir una fecha y hora en el menú.

admin-settings-key-notifications-view-enabled = Activar la vista de notificaciones
admin-settings-help-notifications-view-enabled = Controla el centro de notificaciones. Los avisos de seguridad, de correo y del navegador conservan sus propios ajustes.
admin-settings-key-notifications-view-show-in-sidebar = Mostrar Notificaciones en la barra lateral
admin-settings-help-notifications-view-show-in-sidebar = Controla el centro de notificaciones. Los avisos de seguridad, de correo y del navegador conservan sus propios ajustes.

admin-settings-key-mail-unified-account = Cuenta virtual unificada
admin-settings-help-mail-unified-account = Añadir una cuenta virtual al selector de cuentas, con carpetas y contadores combinados. Independiente de la vista unificada de mensajes.

admin-settings-key-accounts-settings-inherit-groups = Heredar preferencias de la cuenta principal
admin-settings-help-accounts-settings-inherit-groups = Los grupos seleccionados usan los valores de la cuenta principal sin una modificación local. Las preferencias locales y las políticas administrativas prevalecen. No se heredan temas, firmas, credenciales ni identificadores de carpetas.
