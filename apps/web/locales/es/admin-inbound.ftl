# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Acceso entrante
admin-inbound-help = Restrinja por separado las IP de origen de confianza para la consola de administración y para el webmail. Ambas políticas empiezan desactivadas. Estas reglas no cambian los proxies de confianza, los ámbitos de API ni el acceso saliente.
admin-inbound-unscoped = Esta política de todo el despliegue requiere un administrador con acceso de seguridad sin ámbito restringido.
admin-inbound-admin = Consola de administración y API
admin-inbound-admin-help = Incluye el inicio de sesión y este editor. La autenticación existente de un administrador no omite la política de red.
admin-inbound-webmail = Webmail y API de cuenta
admin-inbound-webmail-help = Abarca el correo, DAV, el contenido firmado, SSO, OIDC y las rutas de API. Las conexiones ya actualizadas conservan la decisión tomada en su negociación.
admin-inbound-enabled = Restringir esta superficie a la lista de permitidos
admin-inbound-cidrs = IP y CIDR de rescate
admin-inbound-cidrs-help = Una IP literal o un CIDR por línea, hasta 128. Se requiere al menos uno cuando está activada. Incluya un origen de confianza que pueda usar para la recuperación.
admin-inbound-hostnames = Nombres de host DNS permitidos
admin-inbound-hostnames-help = Un nombre DNS ASCII en minúsculas por línea. Use IDNA ASCII para nombres internacionales. Sin URL, puertos, comodines, IP literales ni puntos finales. Los nombres solo conceden las direcciones resueltas en ese momento.
admin-inbound-dns = Límites de resolución DNS
admin-inbound-dns-help = Los nombres nuevos, caducados y fallidos no conceden acceso. Las respuestas correctas respetan su TTL de DNS y el límite de caché configurado. Las solicitudes nunca esperan a la resolución DNS.
admin-inbound-servers = Servidores DNS recursivos
admin-inbound-servers-help = Déjelo vacío para usar los resolutores del sistema, o introduzca hasta cuatro sockets IP:puerto, uno por línea (por ejemplo 10.0.0.53:53 o [fd00::53]:53). Los servidores explícitos sustituyen a los resolutores del sistema sin alternativa pública. La confianza en el resolutor es responsabilidad suya; no se garantiza una validación DNSSEC independiente.
admin-inbound-dns-timeoutMs = Tiempo de espera de la consulta (milisegundos)
admin-inbound-dns-cacheSeconds = Duración máxima de la caché (segundos)
admin-inbound-dns-maxHosts = Máximo de nombres de host distintos
admin-inbound-dns-maxAddresses = Máximo de direcciones por nombre de host
admin-inbound-dns-concurrency = Consultas simultáneas
admin-inbound-bounds = Intervalo permitido: { $min }–{ $max }.
admin-inbound-lockout = Esta política puede bloquear de inmediato su propio acceso, incluido este editor. Confirme que las IP de rescate cubren su origen real tal como lo determinan los proxies de confianza configurados. No hay ninguna excepción automática para el bucle local ni para un administrador con sesión iniciada. La recuperación tras un bloqueo requiere una restauración en el lado del despliegue; un inicio de sesión normal no puede eludirlo.
admin-inbound-confirm-title = ¿Aplicar la política de acceso entrante?
admin-inbound-confirm-word = APLICAR
admin-inbound-save = Aplicar la política confirmada
admin-inbound-review = Revisar los cambios
admin-inbound-refresh = Actualizar la política guardada y el estado de DNS
admin-inbound-reset = Restablecer el borrador a la política cargada
admin-inbound-saved = El servidor aceptó la política de acceso entrante.
admin-inbound-conflict = La política guardada cambió mientras la editaba. Restablezca el borrador y actualice para cargar su revisión actual antes de volver a editar.
admin-inbound-no-revision = El servidor no devolvió una revisión de la política. Actualice antes de guardar.
admin-inbound-status = Concesiones DNS actuales
admin-inbound-status-help = Es una instantánea de la última actualización del estado, no una cuenta atrás ni una prueba de conectividad. Actualice para ver las concesiones actuales. Puede haber un error junto a una respuesta anterior aún válida hasta su caducidad original.
admin-inbound-refreshing = Hay una actualización DNS acotada en curso.
admin-inbound-idle = No había ninguna actualización DNS en curso en la última lectura del estado.
admin-inbound-no-hosts = No hay nombres de host activos en las políticas activadas.
admin-inbound-hostname = Nombre de host
admin-inbound-state = Concesión de acceso
admin-inbound-addresses = Direcciones utilizables
admin-inbound-validity = Validez en la última actualización
admin-inbound-error = Código de resultado DNS
admin-inbound-usable = Utilizable
admin-inbound-unavailable = Sin concesión actual
admin-inbound-seconds = { $seconds } segundos
admin-inbound-on = Activada
admin-inbound-off = Desactivada
admin-inbound-invalid-dns = Los límites de DNS o el número de resolutores están fuera de los límites permitidos.
admin-inbound-rescue-required = Cada superficie activada requiere al menos una IP o CIDR de rescate.
admin-inbound-too-many = La política supera el límite configurado de nombres de host o direcciones literales.
admin-inbound-invalid-host = Use nombres DNS ASCII en minúsculas, sin URL, puertos, comodines, IP literales ni puntos finales.
