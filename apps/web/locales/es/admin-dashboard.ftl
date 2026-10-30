# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Actualizar
admin-dash-unavailable = Las cifras del panel todavía no están disponibles.
admin-dash-unavailable-detail = Este servidor no proporciona los datos del panel (GET /admin/dashboard responde 404). Llegarán con una actualización posterior del servidor. Las demás secciones funcionan como antes.
admin-dash-error-network = No se pudo cargar el panel: el servidor no respondió. Inténtelo de nuevo.
admin-dash-error-status = No se pudo cargar el panel: el servidor respondió { $status }. Inténtelo de nuevo; si se repite, revise el registro del servidor.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Requiere atención
admin-dash-warn-fix = Abrir la sección que lo soluciona
admin-dash-warn-blocked-accounts = { $count ->
    [one] Una cuenta no puede iniciar sesión porque su dominio no tiene configuración.
   *[other] { $count } cuentas no pueden iniciar sesión porque sus dominios no tienen configuración.
}
admin-dash-warn-no-trusted-proxy = Hay reglas de IP activas, pero no hay ningún proxy de confianza configurado, por lo que todos los clientes pueden parecer procedentes de la dirección del proxy.
admin-dash-warn-allowlist-bypass = Una variable de entorno desactiva la lista de IP permitidas.
admin-dash-warn-metrics-token = El endpoint de métricas no tiene token.
admin-dash-warn-pending-migrations = { $count ->
    [one] Hay una migración de base de datos sin aplicar. Reinicie el servidor para aplicarla.
   *[other] Hay { $count } migraciones de base de datos sin aplicar. Reinicie el servidor para aplicarlas.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Un certificado del servidor de origen caduca pronto (según la última prueba de conexión).
   *[other] { $count } certificados del servidor de origen caducan pronto (según la última prueba de conexión).
}
admin-dash-warn-unknown = Advertencia del servidor { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] Una cuenta bloqueada: sin configuración de dominio
   *[other] { $count } cuentas bloqueadas: sin configuración de dominio
}
admin-dash-blocked-note = Estas cuentas iniciaron sesión antes de esta actualización, pero su dominio no tiene configuración en la consola, así que ahora no pueden iniciar sesión. Configurar el dominio les permite volver a hacerlo.
admin-dash-blocked-chart = Cuentas bloqueadas por dominio
admin-dash-blocked-row = { $count ->
    [one] una cuenta
   *[other] { $count } cuentas
}
admin-dash-blocked-seen = (último servidor de correo usado: { $host })
admin-dash-configure-domain = Configurar { $domain }
admin-dash-blocked-no-access = Su rol no permite configurar dominios.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Lista de configuración: nada pendiente
    [one] Lista de configuración: un elemento pendiente
   *[other] Lista de configuración: { $open } elementos pendientes
}
admin-dash-check-done = Hecho
admin-dash-check-open = Pendiente
admin-dash-check-optional = (opcional)
admin-dash-check-go = Abrir
admin-dash-check-hide = Ocultar
admin-dash-check-unhide = Volver a mostrar
admin-dash-check-hidden = { $count ->
    [one] Hay un elemento oculto. Ocultarlo no lo marca como hecho.
   *[other] Hay { $count } elementos ocultos. Ocultarlos no los marca como hechos.
}
admin-dash-check-show-hidden = Mostrar elementos ocultos
admin-dash-check-hide-hidden = No mostrar elementos ocultos
admin-dash-check-domain-exists = Añadir un dominio de correo
admin-dash-check-domain-upstream = Asignar un servidor de correo al dominio o establecer un servidor de origen predeterminado
admin-dash-check-test-connection = Ejecutar una prueba de conexión correcta (en los últimos 7 días)
admin-dash-check-preferred-domain = Elegir un dominio preferido para que los usuarios inicien sesión sin escribirlo
admin-dash-check-second-admin = Crear una segunda cuenta de administrador o desactivar el inicio de sesión por entorno
admin-dash-check-trusted-proxies = Configurar los proxies de confianza situados delante de este servidor
admin-dash-check-security-policy = Guardar la política de seguridad
admin-dash-check-metrics-token = Establecer un token para el endpoint de métricas
admin-dash-check-branding = Revisar la imagen de marca

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Cuentas y sesiones
admin-dash-users = Usuarios dados de alta
admin-dash-signed-in-24h = Cuentas con sesión iniciada, últimas 24 horas
admin-dash-signed-in-7d = Cuentas con sesión iniciada, últimos 7 días
admin-dash-domains = Dominios
admin-dash-domains-detail = { $upstream } con servidor de correo, { $disabled } desactivados
admin-dash-sessions = Sesiones de webmail activas
admin-dash-admin-sessions = Sesiones de administrador activas
admin-dash-security = Seguridad del inicio de sesión
admin-dash-failed-webmail = Inicios de sesión de webmail fallidos, últimas 24 horas
admin-dash-failed-admin = Inicios de sesión de administrador fallidos, últimas 24 horas
admin-dash-active-blocks = Direcciones bloqueadas
admin-dash-allowlists = Listas de IP permitidas
admin-dash-allowlists-value = Administración { $admin }, webmail { $webmail }
admin-dash-on = activada
admin-dash-off = desactivada
admin-dash-trusted-proxies = Proxies de confianza
admin-dash-configured = Configurados
admin-dash-not-configured = Sin configurar
admin-dash-untrusted-forwarded = { $count ->
    [one] Una solicitud en 24 horas llevaba cabeceras de reenvío de un par que no es de confianza
   *[other] { $count } solicitudes en 24 horas llevaban cabeceras de reenvío de un par que no es de confianza
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Tráfico, últimos { $mins } minutos
admin-dash-traffic-note = Contado por este proceso del servidor desde que se inició. No se incluyen otros procesos ni réplicas.
admin-dash-traffic-empty = No se contó ninguna solicitud en este intervalo.
admin-dash-requests = Solicitudes por minuto
admin-dash-errors = Errores del servidor (5xx) por minuto
admin-dash-p95 = Tiempo de respuesta, percentil 95

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Estado
admin-dash-h-ok = Correcto
admin-dash-h-attention = Requiere atención
admin-dash-h-db = Base de datos
admin-dash-h-migrations = Migraciones
admin-dash-h-engine = Sincronización de correo
admin-dash-h-outbox = Bandeja de salida
admin-dash-h-index = Índice de búsqueda
admin-dash-h-sandbox = Entorno aislado de renderizado
admin-dash-h-plugins = Entorno de ejecución de plugins

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Almacenamiento
admin-dash-db-size = Archivo de base de datos
admin-dash-wal-size = Registro de escritura anticipada
admin-dash-index-size = Índice de búsqueda
admin-dash-volume-free = Espacio libre en el volumen de datos
admin-dash-volume-of = de { $total }
admin-dash-na-wal = Esta base de datos no usa un archivo de registro de escritura anticipada, o el servidor no lo indicó.
admin-dash-na-volume = El servidor no puede leer el espacio libre en esta plataforma.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Acciones de administración recientes
admin-dash-audit-empty = Todavía no hay acciones de administración registradas.
admin-dash-audit-when = Cuándo
admin-dash-audit-actor = Quién
admin-dash-audit-action = Acción
admin-dash-audit-target = Objetivo
admin-dash-audit-all = Abrir el registro de auditoría completo
admin-dash-server = Servidor
admin-dash-version = Versión
admin-dash-build = Compilación
admin-dash-mode = Modo
admin-dash-mode-engine = Motor: este servidor sincroniza y almacena el correo
admin-dash-mode-proxy = Proxy: este servidor transmite las solicitudes al servidor de correo
admin-dash-uptime = En ejecución desde hace
admin-dash-base-path = Publicado en
admin-dash-base-root = la raíz del sitio
admin-dash-duration-dh = { $d } d { $h } h
admin-dash-duration-hm = { $h } h { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
