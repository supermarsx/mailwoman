# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Páginas de diagnóstico
admin-diagx-tab-overview = Resumen
admin-diagx-tab-server = Servidor
admin-diagx-tab-connectivity = Conectividad
admin-diagx-tab-backends = Backends de correo
admin-diagx-tab-posture = Postura de seguridad
admin-diagx-tab-tools = Herramientas
admin-diagx-tab-history = Historial
admin-diagx-tab-report = Informe
admin-diagx-tab-server-intro = Comprobaciones de los componentes propios de este servidor: la base de datos, sus migraciones y la clave del servidor. Se ejecutan junto con las comprobaciones del servidor.
admin-diagx-tab-connectivity-intro = Indica si este servidor es accesible con su nombre público, si puede llegar a los backends guardados y si mantiene fuera de alcance las direcciones privadas. Las herramientas de dominio de abajo buscan registros DNS, prueban TLS y descubren la configuración de un dominio.
admin-diagx-tab-backends-intro = Una conexión y un acuerdo de seguridad con cada punto de conexión de correo, envío, reglas y DAV guardado. No se envían credenciales ni se entrega correo.
admin-diagx-tab-posture-intro = Ajustes que protegen los inicios de sesión y las sesiones, y cómo vería fail2ban la dirección de una solicitud.
admin-diagx-tab-tools-intro = Otros lugares donde comprobar el servicio y los límites que se aplican a las ejecuciones de diagnóstico.
admin-diagx-tab-history-intro = Ejecuciones guardadas, las más recientes primero. Abra una ejecución para ver sus resultados y lo que cambió desde la ejecución anterior.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Advertencia
admin-diagx-status-fail = Fallido
admin-diagx-status-skipped = Omitido
admin-diagx-status-not-run = No ejecutado
admin-diagx-status-running = En ejecución
admin-diagx-run = Ejecutar
admin-diagx-run-again = Ejecutar de nuevo
admin-diagx-running = Ejecutando…
admin-diagx-run-server-note = Vuelve a ejecutar las comprobaciones del servidor. Se actualizan todas las tarjetas de esa ejecución.
admin-diagx-run-backends-note = Vuelve a comprobar todos los backends guardados. Se actualizan todas las tarjetas de backend.
admin-diagx-how-to-fix = Cómo solucionarlo
admin-diagx-details = Detalles
admin-diagx-copy-details = Copiar detalles
admin-diagx-copy-value = Copiar { $label }
admin-diagx-copied = Copiado.
admin-diagx-copy-failed = El navegador no permitió copiar. Seleccione el texto y cópielo manualmente.
admin-diagx-not-run-yet = Aún no se ha ejecutado.
admin-diagx-not-reported = La última ejecución del servidor no informó de esta comprobación; no se aplica a esta implantación.
admin-diagx-posture-met = Cumplido
admin-diagx-posture-not-met = No cumplido
admin-diagx-loading = Cargando…
admin-diagx-none = Ninguno
admin-diagx-yes = Sí
admin-diagx-no = No
admin-diagx-open = Abrir
admin-diagx-row-result = Resultado
admin-diagx-row-duration = Duración
admin-diagx-row-checked = Comprobado
admin-diagx-row-id = ID de la comprobación
admin-diagx-row-domain = Dominio
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protocolo
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Capacidades
admin-diagx-row-credentials = Credenciales enviadas
admin-diagx-row-source = Encontrado en
admin-diagx-row-oauth = Requiere OAuth
admin-diagx-overview-intro = El último resultado de cada comprobación, por grupo. Ejecutar todo lanza las comprobaciones del servidor y después comprueba cada backend guardado.
admin-diagx-run-all = Ejecutar todas las comprobaciones
admin-diagx-run-all-step = Paso { $step } de { $steps }: { $suite ->
    [backends] backends guardados
   *[server] comprobaciones del servidor
}.
admin-diagx-run-all-done = Todas las comprobaciones han terminado. Los resultados se guardan en el historial.
admin-diagx-backends-progress = { $completed } de { $total } backends comprobados.
admin-diagx-counts = { $ok } OK · { $warn } advertencias · { $fail } fallidas · { $notRun } sin ejecutar
admin-diagx-attention = Requiere atención
admin-diagx-attention-none = Ninguna comprobación informó de una advertencia o un fallo en su última ejecución.
admin-diagx-never-run = Aún no se ha ejecutado ninguna comprobación. Ejecute todas las comprobaciones para obtener un primer resultado.
admin-diagx-last-server = Últimas comprobaciones del servidor
admin-diagx-last-backends = Últimas comprobaciones de backends
admin-diagx-never = Nunca
admin-diagx-open-health = Estado del servicio
admin-diagx-check-database-connectivity = Conexión a la base de datos
admin-diagx-check-database-connectivity-purpose = La base de datos responde a una consulta sencilla.
admin-diagx-check-database-connectivity-fix = Compruebe que la base de datos está en marcha y que MW_DB_PATH o MW_DATABASE_URL apunta a ella. El registro del servidor indica el error de conexión.
admin-diagx-check-database-write = Escrituras en la base de datos
admin-diagx-check-database-write-purpose = Se puede escribir una fila de prueba y luego se revierte.
admin-diagx-check-database-write-fix = Compruebe el espacio libre en disco y que el archivo de base de datos o el usuario puedan escribir. Un volumen de solo lectura o un disco lleno impide las escrituras.
admin-diagx-check-database-migrations = Migraciones de la base de datos
admin-diagx-check-database-migrations-purpose = La base de datos tiene todas las migraciones que espera esta versión del servidor.
admin-diagx-check-database-migrations-fix = Reinicie el servidor para que aplique las migraciones pendientes y revise su registro por si alguna migración falló. No ejecute un servidor más antiguo sobre una base de datos más reciente.
admin-diagx-check-server-key-roundtrip = Clave del servidor
admin-diagx-check-server-key-roundtrip-purpose = La clave del servidor sella y abre un valor de prueba.
admin-diagx-check-server-key-roundtrip-fix = Compruebe que MW_SERVER_KEY (o el archivo de clave) es la clave con la que se creó esta base de datos. Los secretos almacenados no se pueden abrir con otra clave.
admin-diagx-check-other-purpose = Una comprobación informada por este servidor.
admin-diagx-check-other-fix = Abra los detalles para ver el resultado sin procesar y revise el registro del servidor en el momento de la ejecución.
admin-diagx-check-network-public-dns = Nombre público
admin-diagx-check-network-public-dns-purpose = El nombre de host de MW_PUBLIC_URL se resuelve desde este servidor.
admin-diagx-check-network-public-dns-fix = Compruebe el registro DNS del nombre público, o establezca MW_PUBLIC_URL con la dirección que usan las personas para llegar a este servidor.
admin-diagx-check-network-public-port = Puerto público
admin-diagx-check-network-public-port-purpose = El puerto de la URL pública acepta una conexión desde este servidor.
admin-diagx-check-network-public-port-fix = Compruebe el cortafuegos, el proxy inverso y el reenvío de puertos. Algunos routers no pueden conectarse a su propia dirección pública (NAT hairpinning); pruebe también desde fuera de su red.
admin-diagx-check-network-backend-ports = Puertos de backend
admin-diagx-check-network-backend-ports-purpose = El host y el puerto de cada backend guardado aceptan una conexión directa.
admin-diagx-check-network-backend-ports-fix = Compruebe que los hosts de los backends están activos y que un cortafuegos permite a este servidor llegar a ellos. Backends de correo muestra qué punto de conexión falla en el acuerdo.
admin-diagx-check-egress-private-policy = Protección de direcciones privadas
admin-diagx-check-egress-private-policy-purpose = De forma predeterminada, se rechazan las conexiones salientes a direcciones de loopback y privadas.
admin-diagx-check-egress-private-policy-fix = Revise los ajustes de salida en Privacidad. Permita las direcciones privadas solo para los backends con nombre que las necesiten.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Las solicitudes llegan a este servidor por HTTPS.
admin-diagx-check-security-https-fix = Sirva la consola por HTTPS, o establezca MW_PUBLIC_URL en https y MW_TRUSTED_PROXIES cuando un proxy inverso termine TLS.
admin-diagx-check-security-secure-cookies = Cookies seguras
admin-diagx-check-security-secure-cookies-purpose = Las cookies de sesión solo se envían por HTTPS.
admin-diagx-check-security-secure-cookies-fix = Establezca MW_COOKIE_SECURE en true una vez que se acceda al servidor por HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Se indica a los navegadores que usen HTTPS para este sitio.
admin-diagx-check-security-hsts-fix = Active Strict-Transport-Security en los ajustes de seguridad, o envíelo desde el proxy inverso.
admin-diagx-check-security-csrf = Protección CSRF
admin-diagx-check-security-csrf-purpose = Las solicitudes que modifican datos deben llevar el token CSRF.
admin-diagx-check-security-csrf-fix = Establezca MW_CSRF_STRICT en true.
admin-diagx-check-security-admin-factors = Segundo factor de los administradores
admin-diagx-check-security-admin-factors-purpose = Todos los administradores tienen un segundo factor de inicio de sesión.
admin-diagx-check-security-admin-factors-fix = Pida a cada administrador que no lo tenga que añada una clave de acceso o una aplicación de autenticación en Mis preferencias, o exíjalo en el centro de seguridad.
admin-diagx-check-security-signin-pacing = Retraso entre reintentos de inicio de sesión
admin-diagx-check-security-signin-pacing-purpose = Tras un inicio de sesión fallido sigue una breve espera.
admin-diagx-check-security-signin-pacing-fix = Active el ritmo de inicio de sesión en la política de seguridad.
admin-diagx-check-security-credential-locks = Bloqueos de credenciales
admin-diagx-check-security-credential-locks-purpose = Los inicios de sesión fallidos repetidos bloquean la credencial durante un tiempo.
admin-diagx-check-security-credential-locks-fix = Active los bloqueos automáticos en el centro de seguridad.
admin-diagx-check-security-geography = Datos de geografía de inicios de sesión
admin-diagx-check-security-geography-purpose = La política de geografía de inicios de sesión dispone de los datos de países que necesita.
admin-diagx-check-security-geography-fix = Instale o actualice la base de datos de países, o desactive la política de geografía.
admin-diagx-check-security-render-isolation = Renderizado aislado de mensajes
admin-diagx-check-security-render-isolation-purpose = Los mensajes se renderizan en el worker independiente y aislado.
admin-diagx-check-security-render-isolation-fix = Compruebe que MW_RENDER_BIN apunta al programa mw-render y que este arranca. El estado del servicio en el panel muestra el estado del worker.
admin-diagx-check-security-deployment = Ajustes de implantación
admin-diagx-check-security-deployment-purpose = Se pueden leer los ajustes de seguridad de la implantación.
admin-diagx-check-security-deployment-fix = Compruebe la conexión a la base de datos y abra después la evaluación de seguridad de la implantación.
admin-diagx-check-security-plaintext-backends = Contraseñas por conexiones sin cifrar
admin-diagx-check-security-plaintext-backends-purpose = Ningún backend guardado acepta contraseñas sin TLS.
admin-diagx-check-security-plaintext-backends-fix = Edite las conexiones del dominio y elija TLS o STARTTLS. Permita conexiones sin cifrar solo en una red privada que usted controle.
admin-diagx-check-fail2ban-client-address = Dirección de cliente para fail2ban
admin-diagx-check-fail2ban-client-address-purpose = La dirección que nombraría una línea de registro de fail2ban es la dirección real del cliente.
admin-diagx-check-fail2ban-client-address-fix = Detrás de un proxy inverso, establezca MW_TRUSTED_PROXIES en la dirección del proxy, o fail2ban bloqueará el proxy en lugar del cliente. Es normal ver una dirección de loopback cuando abre la consola en el propio servidor.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Punto de conexión { $protocol }: conexión y acuerdo de seguridad, sin iniciar sesión.
admin-diagx-backend-other-purpose = Una nota de la ejecución de backends.
admin-diagx-backend-fix = Compruebe el nombre de host, el puerto y el modo TLS guardados para este backend, que el DNS se resuelve y que el certificado es válido para el nombre de host. Después vuelva a ejecutar.
admin-diagx-backend-config-fix = Abra las conexiones del dominio y guárdelas de nuevo; no se pudo leer la lista de backends guardados.
admin-diagx-backend-results = Comprobaciones de backends
admin-diagx-backends-not-run = Los backends guardados aún no se han comprobado.
admin-diagx-backends-run-at = Última comprobación { $date }: { $count } resultados.
admin-diagx-failover-title = Conmutación por error
admin-diagx-failover-purpose = Indica si un dominio puede pasar a otro backend de correo cuando no se puede acceder al primero.
admin-diagx-failover-state = Conmutación por error { $state ->
    [on] activada
   *[off] desactivada
} · { $backends } backends de correo{ $enabled ->
    [no] {" "}· dominio desactivado
   *[yes] {""}
}
admin-diagx-no-domains = No hay dominios guardados.
admin-diagx-domain-tools = Herramientas de dominio
admin-diagx-domain-tools-purpose = Comprobaciones puntuales de un dominio guardado. No inician sesión ni envían correo.
admin-diagx-domain = Dominio
admin-diagx-tool-dns = Registros DNS
admin-diagx-tool-dns-purpose = Busca los registros MX, SPF, DMARC y de servicio (SRV) del dominio.
admin-diagx-tool-dns-fix = Añada los registros que faltan en su proveedor de DNS. Sin MX el dominio no recibe correo; sin SPF y DMARC otros servidores pueden rechazar su correo.
admin-diagx-srv-found = { $found } de { $total } registros de servicio encontrados
admin-diagx-tool-tls = TLS y acuerdo de conexión
admin-diagx-tool-tls-purpose = Se conecta a cada punto de conexión guardado del dominio e informa de la versión de TLS, la caducidad del certificado y el tiempo empleado.
admin-diagx-tool-tls-fix = Si un punto de conexión no es accesible, compruebe el host, el puerto y el cortafuegos. Renueve un certificado que caduque en menos de 14 días.
admin-diagx-probe-none = El dominio no tiene puntos de conexión guardados que probar.
admin-diagx-probe-line = { $host } · accesible: { $reachable ->
    [yes] sí
   *[no] no
} · TLS { $tls } · certificado { $days ->
    [none] no informado
   *[other] caduca en { $days } días
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = Descubrir ajustes
admin-diagx-tool-detect-purpose = Busca los ajustes de correo publicados (autoconfig, autodiscover, SRV) para un nombre de dominio. No se guarda nada.
admin-diagx-tool-detect-fix = Publique registros autoconfig o SRV para el dominio, o introduzca los ajustes manualmente en Dominios.
admin-diagx-detect-name = Nombre de dominio que buscar
admin-diagx-tool-health = Estado del servicio
admin-diagx-tool-health-purpose = Sincronización de correo, bandeja de salida, índice de búsqueda, worker de renderizado y plugins, en el panel.
admin-diagx-tool-deployment-purpose = Cada ajuste de seguridad de la implantación que este servidor puede observar, con el ajuste que lo controla.
admin-diagx-tool-ssh = Pruebas de túnel SSH
admin-diagx-tool-ssh-purpose = Pruebe los túneles SSH usados para llegar a backends en otra red.
admin-diagx-tool-geo = Geografía de inicios de sesión
admin-diagx-tool-geo-purpose = De dónde proceden los inicios de sesión y la base de datos de países que respalda la política de geografía.
admin-diagx-catalogue-purpose = Qué cubre cada conjunto de diagnóstico y los límites que respeta cada ejecución.
admin-diagx-suite-unsupported = No disponible: { $reason }
admin-diagx-suite-server = Comprobaciones del servidor
admin-diagx-suite-backends = Backends guardados
admin-diagx-run-title = { $suite ->
    [backends] Backends guardados
   *[server] Comprobaciones del servidor
}, { $date }
admin-diagx-changes = Cambios
admin-diagx-changes-first = No hay una ejecución anterior de este tipo con la que comparar.
admin-diagx-changes-none = No ha cambiado nada desde la ejecución del { $date }.
admin-diagx-changes-since = Cambiado desde la ejecución del { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Informe de resolución de problemas
admin-diagx-report-purpose = Un informe JSON para el soporte: indicadores de ejecución, los últimos resultados y la ejecución que elija. Nunca se incluyen contraseñas, tokens, cookies, claves, configuración sin procesar ni correo.
admin-diagx-report-run = Ejecución que incluir
admin-diagx-report-preview = Vista previa
admin-crumb-server = Servidor
admin-crumb-connectivity = Conectividad
admin-crumb-posture = Postura de seguridad
admin-crumb-tools = Herramientas
admin-crumb-history = Historial
admin-crumb-report = Informe
admin-diagx-table-actions = Acciones
admin-diagx-check-pdfjs-viewer-assets = Archivos del visor de PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = Los archivos del visor de PDF están todos presentes, con el tipo correcto y coinciden con esta compilación del servidor.
admin-diagx-check-pdfjs-viewer-assets-fix = Vuelva a implantar la compilación web, o elimine la carpeta pdfjs de MW_WEB_DIR para que se use la copia integrada en el servidor. Si la comprobación falla en lugar de solo advertir, reconstruya la imagen del servidor.
