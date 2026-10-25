# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Advertencia de plugin de interfaz sin firmar
plugins-unsigned-title = Hay plugins sin firmar en ejecución
plugins-unsigned-text = Estos plugins se admitieron sin una firma verificada. Se ejecutan en un entorno aislado, pero su código no se ha comprobado con una clave de confianza.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plugin: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Los plugins del motor se ejecutan en un entorno aislado de WebAssembly. Un plugin solo se ejecuta después de registrarse, aprobarse, recibir al menos una capacidad y habilitarse. Solo puede usar las capacidades concedidas aquí.
plugins-admin-unsigned-banner = Hay cargado un plugin de terceros sin firma. Se ejecuta porque un administrador lo permitió; el servidor solo comprueba su archivo con el resumen aprobado.
plugins-admin-load-error = No se pudieron cargar los plugins.
plugins-admin-register-heading = Registrar un plugin
plugins-admin-register-component = Componente
plugins-admin-register-third-party = Componente de terceros
plugins-admin-register-id = Id del plugin
plugins-admin-register-id-hint = El servidor lee el componente del archivo con el nombre de este id en el directorio de plugins de terceros. Apruebe antes el resumen de ese archivo en la lista de permitidos de abajo.
plugins-admin-register-name = Nombre
plugins-admin-register-version = Versión
plugins-admin-register-capabilities = Capacidades que declara el plugin
plugins-admin-register-hosts = Hosts con los que puede contactar el plugin
plugins-admin-register-hosts-hint-first-party = Nombres de host separados por comas. Déjelo vacío para usar los hosts predeterminados del componente.
plugins-admin-register-hosts-hint-third-party = Nombres de host separados por comas. Déjelo vacío para no permitir acceso a la red.
plugins-admin-register-submit = Registrar
plugins-admin-trust-first-party = Propio, resumen integrado
plugins-admin-trust-third-party = De terceros, resumen aprobado
plugins-admin-status-loaded = Cargado
plugins-admin-status-not-loaded = No cargado
plugins-admin-status-restart = Requiere reinicio
plugins-admin-restart-to-load = Es necesario reiniciar para cargar este plugin.
plugins-admin-restart-to-apply = Es necesario reiniciar para aplicar estos cambios. Algunas cuentas de este puente no reciben servicio hasta entonces.
plugins-admin-reason-not-approved = No cargado: ningún administrador ha aprobado este plugin.
plugins-admin-reason-disabled = No cargado: el plugin no está habilitado.
plugins-admin-reason-unsigned-not-allowed = No cargado: el plugin no tiene firma y no se ha permitido que se ejecute sin ella.
plugins-admin-reason-no-grant = No cargado: no se ha concedido ninguna de las capacidades que necesita.
plugins-admin-reason-component-unavailable = No cargado: ningún archivo de componente superó la comprobación del resumen.
plugins-admin-reason-load-failed = No cargado: el servidor no pudo cargar el componente. Consulte el registro del servidor.
plugins-admin-reason-proxy-mode = No cargado: este servidor funciona en modo proxy y no procesa el correo por sí mismo.
plugins-admin-reason-no-account-binding = No cargado: no hay ninguna cuenta vinculada a este puente.
plugins-admin-reason-no-host-caller = No cargado: esta versión del servidor no llama a los hooks que proporciona este plugin.
plugins-admin-reason-another-classifier-active = No cargado: ya hay otro clasificador de spam cargado. Solo se ejecuta uno a la vez.
plugins-admin-running-with = En ejecución con: { $capabilities }
plugins-admin-hosts = Hosts: { $hosts }
plugins-admin-hosts-none = Hosts: ninguno
plugins-admin-grants-heading = Capacidades concedidas
plugins-admin-grants-hint = El plugin solo puede usar las capacidades marcadas aquí. Al guardar se sustituye la concesión anterior.
plugins-admin-grant-for = Conceder { $capability } a { $name }
plugins-admin-grants-save = Guardar concesiones
plugins-admin-endpoint = Dirección del clasificador
plugins-admin-endpoint-for = Dirección del clasificador para { $name }
plugins-admin-endpoint-hint = Dónde contacta el plugin con su demonio. El host también debe estar en la lista de hosts del plugin. Déjelo vacío para usar el valor predeterminado del componente.
plugins-admin-endpoint-save = Guardar dirección
plugins-admin-test = Probar clasificador
plugins-admin-test-verdict = Veredicto de la prueba: { $verdict }
plugins-admin-uninstall = Desinstalar
plugins-admin-uninstall-confirm = Confirmar desinstalación
plugins-admin-uninstall-note = Al desinstalar se eliminan el registro, sus concesiones y sus ajustes almacenados.
plugins-admin-uninstall-detail = Al desinstalar se eliminan los datos almacenados del plugin para todas las cuentas, sus fijaciones de la lista de permitidos, y su registro y concesiones. El archivo del componente en disco no se elimina.
plugins-admin-revoke-detail = Al revocar se retira la aprobación de este resumen y se deshabilita el plugin. Si el plugin está en ejecución, se detiene de inmediato.
plugins-admin-error-already-registered = Este plugin ya está registrado. Desinstálelo antes de volver a registrarlo.
plugins-admin-error-component-unavailable = Ningún archivo de componente de este plugin coincide con el resumen integrado en el servidor.
plugins-admin-error-digest-not-approved = No se encontró ningún archivo de componente con un resumen aprobado para este id. Apruebe antes el resumen del archivo en la lista de permitidos.
plugins-admin-error-not-approved = Apruebe el plugin antes de habilitarlo.
plugins-admin-error-unsigned-not-allowed = Este plugin no tiene firma. Permita que se ejecute sin firmar antes de habilitarlo.
plugins-admin-error-not-loaded = El plugin no está cargado, así que no se puede probar.
plugins-admin-error-classifier-error = La llamada al clasificador falló: { $detail }
plugins-admin-error-other = El servidor rechazó la solicitud: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = Plugins de interfaz
plugins-admin-ui-title = Plugins de interfaz
plugins-admin-ui-intro = Los plugins de interfaz añaden controles al cliente web. Cada uno se ejecuta en un marco aislado y solo puede usar las capacidades concedidas aquí. Un plugin se ofrece a los usuarios con sesión iniciada una vez aprobado y habilitado.
plugins-admin-ui-empty = No hay plugins de interfaz registrados.
plugins-admin-ui-load-error = No se pudieron cargar los plugins de interfaz.
plugins-admin-ui-register-heading = Registrar un plugin de interfaz
plugins-admin-ui-manifest = Manifiesto (JSON)
plugins-admin-ui-bundle = Archivo del paquete
plugins-admin-ui-bundle-hint = Obligatorio para un plugin firmado: la firma se comprueba con este archivo.
plugins-admin-ui-allow-unsigned = Permitir que este plugin se registre sin firma
plugins-admin-ui-register = Registrar
plugins-admin-ui-manifest-invalid = El manifiesto no es JSON válido.
plugins-admin-ui-refused = El servidor rechazó la solicitud: { $detail }
plugins-admin-ui-approve = Aprobar y habilitar
plugins-admin-ui-extension-points = Puntos de extensión: { $points }
plugins-admin-ui-grants-heading = Conceder una capacidad
plugins-admin-ui-grants-note = El servidor no indica qué capacidades ya están concedidas. Conceder de nuevo una capacidad sustituye su concesión anterior.
plugins-admin-ui-grant = Conceder { $capability }
plugins-admin-ui-grant-for = Conceder { $capability } a { $name }
plugins-admin-ui-grant-hosts = Hosts para { $capability } en { $name }
plugins-admin-ui-grant-hosts-hint = Nombres de host separados por comas de los que el plugin puede descargar datos.
plugins-admin-ui-granted = { $capability } concedida.
plugins-admin-ui-delete = Eliminar
plugins-admin-ui-delete-confirm = Confirmar eliminación
