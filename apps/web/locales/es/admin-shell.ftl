# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Panel
admin-shell-group-overview = Resumen
admin-shell-group-people = Personas
admin-shell-group-security = Seguridad
admin-shell-group-configuration = Configuración
admin-shell-group-operations = Operaciones
admin-shell-menu = Menú
admin-shell-close-menu = Cerrar menú
admin-shell-search = Buscar
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Sesión iniciada como { $name }
admin-shell-language-label = Idioma
admin-shell-breadcrumb = Ruta de navegación
admin-shell-unknown-section = Esta sección no existe o su rol no la incluye.
admin-shell-back-to-dashboard = Volver al panel
admin-shell-cap-missing = Su rol no incluye { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Cargando…
admin-shell-retry = Reintentar
admin-shell-cancel = Cancelar
admin-shell-save = Guardar
admin-shell-not-available = No disponible

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = ¿Salir de esta página?
admin-shell-leave-body = Los cambios de esta página no se han guardado. Si sale, se descartarán.
admin-shell-leave-confirm = Salir y descartar cambios

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Escriba { $name } para confirmar

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Buscar en la consola de administración
admin-shell-palette-label = Sección, usuario, dominio o ajuste
admin-shell-palette-results = Resultados
admin-shell-palette-none = No hay coincidencias.
admin-shell-palette-searching = Buscando usuarios, dominios y ajustes…
admin-shell-palette-sections-only = Este servidor todavía no puede buscar usuarios, dominios y ajustes, así que solo se muestran secciones.
admin-shell-palette-error = La búsqueda de usuarios, dominios y ajustes falló; solo se muestran secciones.
admin-shell-palette-kind-section = Sección
admin-shell-palette-kind-user = Usuario
admin-shell-palette-kind-domain = Dominio
admin-shell-palette-kind-setting = Ajuste

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filtros
admin-shell-table-actions = Acciones
admin-shell-table-error = No se pudo cargar la lista.
admin-shell-table-paging = Páginas de { $table }
admin-shell-table-prev = Página anterior
admin-shell-table-next = Página siguiente

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Último { $latest }, máximo { $max }
admin-shell-chart-col-time = Hora
admin-shell-chart-col-item = Elemento
admin-shell-chart-col-value = Valor

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Establecido por
admin-shell-pinned-sr = Este valor procede del entorno del servidor y no se puede cambiar aquí.
admin-shell-lock-text = Bloquear para los usuarios
admin-shell-lock-setting = { $setting }
admin-shell-restore = Restaurar el valor anterior
admin-shell-restore-note = Cambiado por { $actor } el { $time }. Valor anterior: { $value }
admin-shell-restore-done = Se ha restaurado el valor anterior.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Pasos
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Paso { $n } de { $total }: { $title }
admin-shell-wizard-review = Revisar
admin-shell-wizard-back = Atrás
admin-shell-wizard-next = Siguiente
admin-shell-wizard-saved = Guardado.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = No se pudo contactar con el servidor. Compruebe la conexión e inténtelo de nuevo.
admin-shell-err-session = Su sesión de administrador ha terminado. Vuelva a iniciar sesión.
admin-shell-err-forbidden = Su rol no permite esta acción. Pídaselo a un administrador con el rol adecuado.
admin-shell-err-out-of-scope = Este objeto pertenece a un dominio que su rol no abarca.
admin-shell-err-not-found = Ya no existe. Vuelva a cargar la lista.
admin-shell-err-conflict = Alguien lo ha cambiado mientras tanto. Vuelva a cargar e inténtelo de nuevo.
admin-shell-err-validation = El servidor rechazó un valor. Revise los campos e inténtelo de nuevo.
admin-shell-err-rate-limited = Demasiadas solicitudes. Espere un minuto e inténtelo de nuevo.
admin-shell-err-server = El servidor respondió con un error ({ $status }). Inténtelo de nuevo; si se repite, revise el registro del servidor.
admin-shell-err-upstream-dns = No se pudo resolver el nombre de host. Compruebe cómo está escrito el host.
admin-shell-err-upstream-connect = Nada respondió en ese host y puerto. Compruebe el puerto y cualquier cortafuegos intermedio.
admin-shell-err-upstream-tls = El servidor respondió, pero falló el protocolo de enlace TLS. Compruebe el puerto y el ajuste de seguridad de la conexión.
admin-shell-err-upstream-auth = El servidor rechazó las credenciales. Compruebe el nombre de usuario y la contraseña.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Demasiados intentos de inicio de sesión. Espere unos minutos e inténtelo de nuevo.
admin-shell-totp-form = Inicio de sesión de administrador, segundo paso
admin-shell-totp-note = Introduzca el código de 6 dígitos de la aplicación de autenticación configurada para { $name }.
admin-shell-totp-code = Código de autenticación
admin-shell-totp-verify = Verificar
admin-shell-totp-start-over = Empezar de nuevo
admin-shell-totp-invalid = Ese código no es válido. Introduzca el código actual de su aplicación de autenticación.
admin-shell-totp-expired = El inicio de sesión tardó demasiado. Empiece de nuevo e introduzca su contraseña.
