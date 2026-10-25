# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Administración de Mailwoman
admin-nav = Secciones de administración
admin-sign-out = Cerrar sesión
admin-nav-domains = Dominios
admin-nav-users = Usuarios
admin-nav-integrations = Integraciones
admin-nav-observability = Observabilidad
admin-nav-plugins = Plugins
admin-nav-assist = Asistente de IA
admin-nav-sso = Inicio de sesión único
admin-nav-servermeta = Metadatos del servidor
admin-nav-rethread = Mantenimiento
admin-nav-2fa = Exigir doble factor

admin-delete = Eliminar
admin-revoke = Revocar
admin-remove = Quitar
admin-saved = Guardado.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Inicio de sesión de administrador
admin-login-note = Este panel funciona con una sesión de administrador independiente. La sesión termina tras 30 minutos sin actividad y, como máximo, 12 horas después de iniciarla.
admin-login-session-ended = Su sesión de administrador ha terminado. Vuelva a iniciar sesión.
admin-login-username = Usuario administrador
admin-login-password = Contraseña
admin-login-sign-in = Iniciar sesión
admin-login-signing-in = Iniciando sesión…
admin-login-invalid = Credenciales de administrador no válidas
admin-login-unreachable = No se pudo contactar con el servidor

# -- Domains -----------------------------------------------------------------
admin-domains-title = Dominios
admin-domains-load-error = No se pudieron cargar los dominios
admin-domains-save-error = No se pudo guardar el dominio
admin-domains-delete-error = No se pudo eliminar el dominio
admin-domains-add = Añadir dominio
admin-domains-name = Nombre de dominio
admin-domains-name-placeholder = example.com
admin-domains-note = Un dominio se registra por su nombre. Los nombres registrados se ofrecen en la pantalla Exigir doble factor cuando una regla se aplica a un solo dominio. Registrar un dominio no enruta ni filtra el correo.
admin-domains-save = Guardar dominio
admin-domains-empty = Todavía no hay dominios.
admin-domains-delete-for = Eliminar { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Usuarios
admin-users-load-error = No se pudieron cargar los usuarios
admin-users-provision-error = No se pudo dar de alta al usuario
admin-users-flag-error = No se pudo actualizar el indicador
admin-users-revoke-error = No se pudieron revocar las sesiones
admin-users-provision = Dar de alta usuario
admin-users-username = Nombre de usuario
admin-users-domain = Dominio
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Cuota en bytes (0 = ilimitada)
admin-users-quota-msgs = Cuota en mensajes (0 = ilimitada)
admin-users-empty = Todavía no hay usuarios.
admin-users-col-account = Cuenta
admin-users-col-quota = Cuota (bytes/mensajes)
admin-users-col-zeroaccess = Acceso cero
admin-users-col-flags = Indicadores
admin-users-col-sessions = Sesiones
admin-users-zeroaccess-for = Acceso cero para { $account }
admin-users-zeroaccess-help = Solo es un registro. El almacenamiento de acceso cero lo activa y desactiva el propio usuario, en su configuración y con su propia clave. Marcar esta casilla no cifra el correo almacenado de la cuenta, y desmarcarla no lo descifra.
admin-users-disable-for = Desactivar { $account }
admin-users-disabled = desactivada
admin-users-disabled-help = Bloquea el inicio de sesión y cierra todas las sesiones abiertas de esta cuenta. Sus claves de API y tokens se rechazan. No desactiva el buzón en el servidor de correo: otros clientes de correo pueden seguir accediendo hasta que también lo desactive allí.
admin-users-force-change-for = Forzar cambio de contraseña para { $account }
admin-users-force-change = forzar cambio
admin-users-force-change-help = El usuario queda retenido en una pantalla de cambio de contraseña, en el siguiente inicio de sesión y en las sesiones ya abiertas, y no puede acceder al correo hasta que el cambio se complete. Esto requiere un backend de cambio de contraseña operativo (MW_PASSWD_BACKEND). Sin él, el cambio no puede completarse y el usuario permanece en esa pantalla hasta que desmarque esta casilla.
admin-users-revoke-for = Revocar sesiones de { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integraciones
admin-integrations-load-error = No se pudieron cargar las integraciones
admin-integrations-revoke-error = No se pudo revocar la clave
admin-integrations-ldap = Directorio LDAP / GAL
admin-integrations-nextcloud = Puente de Nextcloud
admin-integrations-active = Activa
admin-integrations-configured = Configurada
admin-integrations-not-configured = No configurada
admin-integrations-unknown = Estado desconocido
admin-integrations-config-note = «Configurada» significa que esta instalación tiene ajustes para la integración: una entrada de directorio LDAP habilitada en la base de datos, o las tres variables de entorno MW_NEXTCLOUD_*. Ambas se leen al iniciar el servidor. No significa que se haya contactado con el servicio remoto ni que esté accesible.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = No hay webhooks registrados.
admin-integrations-keys = Claves de API y MCP
admin-integrations-keys-empty = No se ha emitido ninguna clave.
admin-integrations-col-prefix = Prefijo
admin-integrations-col-account = Cuenta
admin-integrations-col-scopes = Ámbitos
admin-integrations-col-status = Estado
admin-integrations-status-revoked = revocada
admin-integrations-status-active = activa
admin-integrations-revoke-key = Revocar clave { $prefix }
admin-integrations-col-unattended = Envío desatendido
admin-integrations-unattended-not-requested = No solicitado
admin-integrations-unattended-requested = Solicitado, no aprobado
admin-integrations-unattended-approved = Aprobado
admin-integrations-unattended-approve = Aprobar
admin-integrations-unattended-withdraw = Retirar
admin-integrations-unattended-approve-key = Aprobar envío desatendido para la clave { $prefix }
admin-integrations-unattended-withdraw-key = Retirar la aprobación del envío desatendido para la clave { $prefix }
admin-integrations-unattended-approve-title = ¿Aprobar el envío desatendido para esta clave?
admin-integrations-unattended-withdraw-title = ¿Retirar la aprobación de esta clave?
admin-integrations-unattended-owner = Propietario: { $account }
admin-integrations-unattended-key = Clave: { $prefix }
admin-integrations-unattended-scope = Permisos: { $permissions }. Herramientas MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = No se pudo leer el ámbito de esta clave.
admin-integrations-scope-read = lectura
admin-integrations-scope-send = envío
admin-integrations-scope-delete = eliminación
admin-integrations-scope-mail = correo
admin-integrations-scope-pim = calendario, tareas, notas y contactos
admin-integrations-scope-none = ninguno
admin-integrations-unattended-approve-meaning = Los mensajes enviados con esta clave a través de MCP se transmiten sin que una persona los libere.
admin-integrations-unattended-withdraw-meaning = Un envío hecho con esta clave a través de MCP se rechaza hasta que la clave vuelva a aprobarse. El mensaje no se transmite y no se coloca en la Bandeja de salida del propietario.
admin-integrations-unattended-approve-effect = La aprobación surte efecto de inmediato, a partir del siguiente envío de la clave. No hace falta reiniciar.
admin-integrations-unattended-withdraw-effect = La retirada surte efecto de inmediato: la clave deja de enviar de forma desatendida a partir de su siguiente envío. No hace falta reiniciar.
admin-integrations-unattended-cancel = Cancelar
admin-integrations-unattended-saving = Guardando…
admin-integrations-unattended-approved-done = Envío desatendido aprobado para la clave { $prefix }. Ya está en vigor.
admin-integrations-unattended-withdrawn-done = Aprobación retirada para la clave { $prefix }. Esta clave ya no envía de forma desatendida.
admin-integrations-unattended-error-gone = La clave { $prefix } ya no existe o fue revocada. No se ha cambiado nada.
admin-integrations-unattended-error-conflict = La clave { $prefix } está revocada o su propietario no solicitó el envío desatendido. No se ha aprobado nada.
admin-integrations-unattended-error = No se pudo cambiar la aprobación de la clave { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Observabilidad
admin-obs-load-error = No se pudieron cargar los datos de observabilidad
admin-obs-export-error = No se pudo exportar el registro de auditoría
admin-obs-ban-add-error = No se pudo añadir el bloqueo
admin-obs-unban-error = No se pudo quitar el bloqueo
admin-obs-telemetry-note = El registro y la telemetría no se configuran aquí. El filtro de registro (MW_LOG), el colector OTLP (MW_OTLP_ENDPOINT) y el endpoint de métricas (MW_METRICS_TOKEN) se leen del entorno del servidor al iniciarse.
admin-obs-audit = Registro de auditoría
admin-obs-export = Exportar JSONL
admin-obs-audit-empty = No hay entradas de auditoría.
admin-obs-col-time = Hora
admin-obs-col-actor = Autor
admin-obs-col-action = Acción
admin-obs-col-target = Objetivo
admin-obs-bans = Monitor de inicios de sesión / lista de bloqueos
admin-obs-bans-note = Esta lista es un registro. Mailwoman no rechaza conexiones de las direcciones que contiene, y una entrada no afecta al inicio de sesión. Cada inicio de sesión de administrador fallido escribe una línea en el registro del servidor sobre la que puede actuar una jaula de fail2ban, y cinco fallos desde una misma dirección la añaden aquí. El bloqueo lo realiza esa jaula o su cortafuegos.
admin-obs-ban-add = Añadir bloqueo
admin-obs-ban-ip = Dirección IP
admin-obs-ban-reason = Motivo
admin-obs-ban-btn = Bloquear IP
admin-obs-bans-empty = No hay bloqueos activos.
admin-obs-unban-for = Desbloquear { $ip }
admin-obs-unban-btn = Desbloquear

# -- Plugins (§22) -----------------------------------------------------------
admin-plugins-title = Plugins
admin-plugins-intro = Los plugins del motor se ejecutan en un entorno aislado de WebAssembly controlado por capacidades. Apruebe un plugin antes de poder habilitarlo y conceda solo las capacidades que necesite.
admin-plugins-empty = No hay plugins registrados.
admin-plugins-signed = Firmado
admin-plugins-unsigned = Sin firmar
admin-plugins-approved = Aprobado
admin-plugins-enabled = Habilitado
admin-plugins-approve = Aprobar
admin-plugins-enable = Habilitar
admin-plugins-disable = Deshabilitar
admin-plugins-net = red: { $hosts }
admin-plugins-limits = límites: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = límites: { $memory } MiB · { $deadline } ms · { $fuel } de combustible
admin-plugins-allow-unsigned-for = Permitir el plugin sin firmar { $name }
admin-plugins-allow-unsigned = Permitir que se ejecute este plugin sin firmar
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
admin-allowlist-title = Lista de plugins de terceros permitidos
admin-allowlist-intro = Un componente de terceros solo se carga después de que un administrador apruebe su resumen SHA-256 exacto. El servidor calcula el resumen siguiente sobre los bytes del componente en disco; aprobarlo fija exactamente esos bytes. Los componentes propios están fijados en la compilación y no se gestionan aquí.
admin-allowlist-load-error = No se pudo cargar la lista de plugins permitidos.
admin-allowlist-present-heading = Componentes en disco
admin-allowlist-present-empty = No hay componentes de terceros. Coloque un componente en el directorio de plugins de terceros para que aparezca aquí.
admin-allowlist-digest-label = SHA-256 calculado
admin-allowlist-status-approved = Aprobado
admin-allowlist-status-pending = No aprobado
admin-allowlist-status-firstparty = Propio
admin-allowlist-unsigned-note = Admitido por resumen fijado. Este componente no lleva firma; la aprobación confía en los bytes exactos cuyo resumen se muestra, que es lo previsto para un componente aprobado de esta forma.
admin-allowlist-highpower-note = Las capacidades de alto poder ({ $caps }) no pueden concederse a un plugin de terceros. El servidor las rechaza con independencia de la aprobación; solo están disponibles para componentes propios.
admin-allowlist-firstparty-note = Este id coincide con un componente propio. La fijación propia siempre tiene prioridad, por lo que este id no puede aprobarse como de terceros.
admin-allowlist-approve = Aprobar resumen
admin-allowlist-approve-for = Aprobar resumen para { $id }
admin-allowlist-revoke = Revocar
admin-allowlist-revoke-for = Revocar fijación de { $id }
admin-allowlist-uninstall = Desinstalar
admin-allowlist-uninstall-for = Desinstalar { $id }
admin-allowlist-pins-heading = Fijaciones aprobadas y revocadas
admin-allowlist-pins-empty = No hay fijaciones registradas.
admin-allowlist-pin-approved-by = Aprobado por { $by } el { $at }
admin-allowlist-pin-revoked = Revocado
admin-allowlist-cancel = Cancelar

admin-allowlist-approve-title = ¿Aprobar la carga de este componente?
admin-allowlist-approve-detail = Al aprobar se fijan los bytes exactos cuyo SHA-256 se muestra abajo. Tras la aprobación, solo se cargarán para este id los bytes que coincidan con este resumen; cualquier cambio en el componente produce un resumen distinto y no se cargará hasta que se vuelva a aprobar. La aprobación no concede ninguna capacidad por sí sola.
admin-allowlist-approve-confirm = Aprobar resumen

admin-allowlist-revoke-title = ¿Revocar esta fijación?
admin-allowlist-revoke-detail = Al revocar se retira la aprobación de este resumen y se deshabilita el plugin. Surte efecto en la siguiente carga; una instancia en ejecución no se detiene hasta entonces.
admin-allowlist-revoke-confirm = Revocar fijación

admin-allowlist-uninstall-title = ¿Desinstalar este plugin?
admin-allowlist-uninstall-detail = Al desinstalar se eliminan los datos clave/valor almacenados del plugin para todas las cuentas, se quitan sus fijaciones de la lista de permitidos y se deshabilita. El archivo del componente en disco no se elimina; puede volver a aprobarse más adelante.
admin-allowlist-uninstall-confirm = Desinstalar plugin

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Inicio de sesión único
admin-sso-intro = Configure backends de inicio de sesión OIDC y SAML 2.0. Los backends habilitados aparecen como botones «Iniciar sesión con…» en el inicio de sesión del buzón, para toda la instalación o para un dominio.
admin-sso-add = Añadir un backend de inicio de sesión
admin-sso-edit = Editar
admin-sso-create = Añadir backend
admin-sso-update = Guardar cambios
admin-sso-cancel = Cancelar
admin-sso-empty = No hay backends SSO configurados.
admin-sso-load-error = No se pudieron cargar los backends SSO.
admin-sso-save-error = No se pudo guardar el backend.
admin-sso-delete-error = No se pudo eliminar el backend.

admin-sso-id = ID del backend
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Nombre visible
admin-sso-display-name-placeholder = Iniciar sesión con Acme SSO
admin-sso-kind = Protocolo
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Ámbito
admin-sso-scope-deployment = Toda la instalación
admin-sso-scope-domain = Un dominio
admin-sso-domain = Dominio
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Habilitado
admin-sso-first-login = Política del primer inicio de sesión
admin-sso-policy-allowlist = Lista de permitidos (rechazar usuarios desconocidos)
admin-sso-policy-autocreate = Crear cuentas automáticamente en el primer inicio de sesión

admin-sso-issuer = URL del emisor
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = ID de cliente
admin-sso-client-secret = Secreto de cliente
admin-sso-secret-unchanged = Dejar en blanco para conservar el secreto almacenado
admin-sso-redirect = URL de redirección
admin-sso-scopes = Ámbitos
admin-sso-metadata = Metadatos del SP

admin-sso-sp-entity-id = ID de entidad del SP
admin-sso-acs-url = URL de ACS
admin-sso-idp-metadata-url = URL de metadatos del IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML de metadatos del IdP
admin-sso-idp-metadata-xml-placeholder = Pegue el XML de metadatos del IdP o use la URL de arriba
admin-sso-idp-sso-url = URL de SSO del IdP
admin-sso-idp-slo-url = URL de cierre de sesión (SLO) del IdP
admin-sso-idp-certs = Certificados de firma del IdP (PEM)
admin-sso-idp-certs-placeholder = Un certificado PEM por bloque, separados por una línea en blanco
admin-sso-nameid-format = Formato de NameID
admin-sso-want-signed = Exigir aserciones firmadas
admin-sso-want-encrypted = Exigir aserciones cifradas

admin-sso-claims = Asignación de claims
admin-sso-claim-email = Claim de correo electrónico
admin-sso-claim-username = Claim de nombre de usuario
admin-sso-claim-display = Claim de nombre visible
admin-sso-claim-groups = Claim de grupos

admin-sso-badge-enabled = Habilitado
admin-sso-badge-disabled = Deshabilitado
admin-sso-enable = Habilitar
admin-sso-disable = Deshabilitar
admin-sso-delete = Eliminar
admin-sso-enable-for = Habilitar { $name }
admin-sso-disable-for = Deshabilitar { $name }
admin-sso-delete-for = Eliminar { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = Metadatos del servidor
admin-servermeta-intro = Vea y edite las anotaciones de servidor RFC 5464 de una cuenta dada de alta. Los cambios se escriben directamente en el servidor de correo, que decide si la cuenta puede establecerlas.
admin-servermeta-account = Cuenta
admin-servermeta-select-option = Seleccione una cuenta…
admin-servermeta-select-prompt = Seleccione una cuenta para ver y editar sus anotaciones de servidor.
admin-servermeta-load-error = No se pudo cargar la lista de cuentas.
admin-servermeta-no-accounts = No hay cuentas dadas de alta.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = Reagrupar hilos del buzón
admin-rethread-intro = Vuelve a ejecutar la agrupación en conversaciones (JWZ) sobre el correo almacenado de una cuenta dada de alta y recalcula las claves de sus hilos. Es una acción de mantenimiento puntual, no algo que se ejecute automáticamente.
admin-rethread-account = Cuenta
admin-rethread-select-option = Seleccione una cuenta…
admin-rethread-no-accounts = No hay cuentas dadas de alta.
admin-rethread-load-error = No se pudo cargar la lista de cuentas.
admin-rethread-run = Reagrupar hilos del buzón
admin-rethread-confirm-title = ¿Reagrupar los hilos de este buzón?
admin-rethread-confirm-warning = Reagrupar recalcula la agrupación de conversaciones de esta cuenta. Los hilos existentes pueden fusionarse o dividirse, y los usuarios pueden ver las conversaciones reagrupadas.
admin-rethread-confirm-detail = Es una acción de mantenimiento puntual. Se ejecuta una vez ahora; puede repetirse sin riesgo y no elimina ningún correo.
admin-rethread-confirm = Reagrupar hilos del buzón
admin-rethread-running = Reagrupando…
admin-rethread-cancel = Cancelar
admin-rethread-summary = Se reagruparon { $messages } mensaje(s) en { $threads } hilo(s) en { $accounts } cuenta(s); { $reassigned } mensaje(s) pasaron a otro hilo.
admin-rethread-error = La reagrupación falló. Si el servidor rechazó la solicitud, no se cambió ninguna agrupación de hilos; revise los registros del servidor e inténtelo de nuevo.

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = Índice de búsqueda
admin-searchindex-intro = La búsqueda lee un índice creado a partir del correo almacenado. El servidor lo reconstruye al iniciarse cuando no coincide con el correo almacenado. El correo de las cuentas de acceso cero no se indexa, por lo que la búsqueda no encuentra nada en esas cuentas.
admin-searchindex-count = { $documents } de { $messages } mensaje(s) almacenado(s) están indexados.
admin-searchindex-on-disk = El índice se guarda en disco.
admin-searchindex-in-memory = El índice se guarda en memoria y se reconstruye cada vez que se inicia el servidor.
admin-searchindex-progress = Hay una reconstrucción en curso: { $done } de { $total } mensaje(s) procesados.
admin-searchindex-run = Reconstruir índice de búsqueda
admin-searchindex-running = Reconstruyendo…
admin-searchindex-summary = Se leyeron { $messages } mensaje(s) almacenado(s) en { $accounts } cuenta(s): { $indexed } indexados, { $removed } quitados del índice, { $failed } no se pudieron leer. { $zeroAccess } cuenta(s) de acceso cero quedaron sin indexar.
admin-searchindex-busy = Ya hay una reconstrucción en curso. Espere a que termine.
admin-searchindex-error = La reconstrucción falló. El índice conserva lo que tenía; revise los registros del servidor e inténtelo de nuevo.
admin-searchindex-status-error = No se pudo leer el estado del índice de búsqueda.
admin-searchindex-unavailable = Este servidor funciona en modo proxy y no tiene índice de búsqueda.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = Exigir doble factor
admin-2fa-intro = Exija un segundo factor (llave de acceso o aplicación de autenticación) para iniciar sesión. Cualquier usuario puede registrar un factor por su cuenta; exigirlo aquí obliga a las cuentas afectadas a registrarlo en su siguiente inicio de sesión.
admin-2fa-load-error = No se pudo cargar la política de doble factor.
admin-2fa-save-error = No se pudo guardar la política de doble factor.
admin-2fa-global = Exigir doble factor en toda la instalación
admin-2fa-global-label = Exigir doble factor en toda la instalación
admin-2fa-global-note = Si está activado, todas las cuentas deben tener un segundo factor. Una regla por dominio, abajo, también puede exigirlo para un dominio sin exigirlo en todas partes.
admin-2fa-domains-heading = Requisitos por dominio
admin-2fa-domains-empty = No hay requisitos por dominio.
admin-2fa-col-domain = Dominio
admin-2fa-col-require = Exigido
admin-2fa-require-for = Exigir doble factor para { $domain }
admin-2fa-add-domain = Dominio
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Exigir doble factor para este dominio
admin-2fa-add-require-label = Exigir doble factor para este dominio
admin-2fa-add-save = Añadir regla de dominio

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Salida
admin-egress-heading = Rutas de proxy de salida
admin-egress-intro = Enrute las peticiones salientes del servidor a través de un proxy HTTP CONNECT o SOCKS5. Mailwoman sigue aplicando su propia política de direcciones: nunca se pide al proxy que resuelva un nombre.
admin-egress-empty = No hay rutas configuradas. Las peticiones salientes van directas.
admin-egress-none-active = No hay ninguna ruta en uso. Las peticiones salientes van directas. Una ruta guardada solo transporta tráfico después de elegir «Usar esta ruta».
admin-egress-in-use = Las peticiones salientes usan la ruta «{ $id }».
admin-egress-deactivate = Dejar de usar un proxy
admin-egress-activate = Usar esta ruta
admin-egress-activate-for = Usar la ruta { $id }
admin-egress-activate-error = No se pudo cambiar a la ruta
admin-egress-deactivate-error = No se pudo dejar de usar la ruta
admin-egress-col-state = Estado
admin-egress-state-active = En uso
admin-egress-state-inactive = Sin usar
admin-egress-test-scope = Solo se puede probar la ruta en uso. Una prueba descarga una URL fija a través de la ruta e indica hasta dónde llegó; no cambia nada.
admin-egress-probe-url = Descargada: { $url }
admin-egress-load-error = No se pudieron cargar las rutas de salida
admin-egress-save-error = No se pudo guardar la ruta
admin-egress-delete-error = No se pudo eliminar la ruta
admin-egress-invalid = Se necesitan un id, un host y un puerto mayor que cero.
admin-egress-username-required = Una contraseña necesita un nombre de usuario; sin él, la ruta se autenticaría como nadie.
admin-egress-col-id = Id
admin-egress-col-route = Ruta
admin-egress-col-username = Usuario
admin-egress-col-credentials = Contraseña
admin-egress-col-plaintext = Texto plano
admin-egress-col-actions = Acciones
admin-egress-cred-set = Establecida
admin-egress-cred-none = Ninguna
admin-egress-plaintext-on = Permitido
admin-egress-plaintext-off = Rechazado
admin-egress-add-heading = Añadir una ruta
admin-egress-edit-heading = Editar ruta
admin-egress-id = Id
admin-egress-scheme = Esquema
admin-egress-host = Host
admin-egress-port = Puerto
admin-egress-username = Nombre de usuario
admin-egress-password = Contraseña
admin-egress-password-unchanged = Dejar en blanco para conservar la contraseña almacenada
admin-egress-allow-plaintext = Permitir orígenes http en texto plano a través de esta ruta
admin-egress-allow-plaintext-note = Desactivado de forma predeterminada. Un origen https es lo que impide que un proxy lea el tráfico que transporta; permitir http significa que el proxy lo ve.
admin-egress-test = Probar
admin-egress-testing = Probando…
admin-egress-test-failed = No se pudo ejecutar la prueba, así que no se sabe nada de esta ruta.
admin-egress-outcome-connected = Conectado
admin-egress-outcome-auth-rejected = Autenticación rechazada
admin-egress-outcome-refused-by-policy = Rechazado por la política
admin-egress-outcome-dns-failed = El nombre no se resolvió
admin-egress-outcome-unreachable = Inaccesible
admin-egress-outcome-origin-tls-failed = Falló el TLS del origen
admin-egress-outcome-route-invalid = La ruta no es válida
admin-egress-outcome-unknown = Resultado no reconocido
admin-egress-stage-dns = se detuvo en la resolución de nombres
admin-egress-stage-connect = se detuvo al conectar con el proxy
admin-egress-stage-tunnel = se detuvo al abrir el túnel
admin-egress-stage-origin = se detuvo en el origen
admin-egress-stage-unknown = se detuvo en una fase no reconocida
admin-egress-proxied-yes = Pasó por el proxy
admin-egress-proxied-no = No pasó por el proxy
