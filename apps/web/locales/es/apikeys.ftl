# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Claves de API
apikeys-heading = Claves de API
apikeys-intro = Cree claves con alcance limitado para scripts e integraciones. Cada clave se guarda con hash, se muestra una sola vez y se puede revocar individualmente. Conceda el alcance mínimo que funcione.

apikeys-label = Etiqueta
apikeys-label-aria = Etiqueta de la clave
apikeys-label-placeholder = p. ej., script de copia de seguridad

apikeys-create = Crear clave
apikeys-saved = Ya la he guardado

apikeys-reveal-warning = Copie este secreto ahora — se muestra una sola vez y no se puede volver a obtener.
apikeys-copy = Copiar
apikeys-copy-aria = Copiar el secreto al portapapeles
apikeys-copied = Copiado al portapapeles

apikeys-error-need-label = asigne una etiqueta a la clave para reconocerla más tarde
apikeys-error-create = no se pudo crear la clave
apikeys-error-revoke = no se pudo revocar la clave

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Claves existentes
apikeys-existing = Claves existentes
apikeys-none = Aún no hay claves.
apikeys-created = creada el { $date }
apikeys-last-used = último uso el { $date }
apikeys-never-used = nunca usada
apikeys-revoked-at = revocada el { $date }
apikeys-revoke = Revocar
apikeys-status-active = Activa
apikeys-status-revoked = Revocada

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Alcance
apikeys-capabilities = Capacidades
apikeys-cap-read = Leer
apikeys-cap-send = Enviar
apikeys-cap-delete = Eliminar
apikeys-surface = Ámbito
apikeys-surface-mail-label = Correo
apikeys-surface-pim-label = PIM (calendario / tareas / notas / contactos)
apikeys-accounts = Cuentas
apikeys-all-accounts = Todas las cuentas
apikeys-folders = Carpetas
apikeys-all-folders = Todas las carpetas
apikeys-folder-ids = ID de carpetas (separados por comas)
apikeys-folder-subset-aria = Subconjunto de carpetas
apikeys-constraints = Restricciones
apikeys-expiry = Caducidad (RFC 3339, vacío = sin caducidad)
apikeys-expiry-aria = Caducidad
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Límite de frecuencia (solicitudes/min, vacío = ilimitado)
apikeys-rate-limit-aria = Límite de frecuencia
apikeys-ip-allowlist = Lista de IP permitidas (CIDR/IP, separadas por comas; vacío = cualquiera)
apikeys-ip-allowlist-aria = Lista de IP permitidas
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = leer
apikeys-verb-send = enviar
apikeys-verb-delete = eliminar
apikeys-summary-mail = correo
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = sin acciones
apikeys-summary-nothing = nada
apikeys-summary-none = ninguno
apikeys-summary-verbs-on = { $verbs } en { $surfaces }
apikeys-summary-all-accounts = todas las cuentas
apikeys-summary-accounts = cuentas: { $ids }
apikeys-summary-all-folders = todas las carpetas
apikeys-summary-folders = carpetas: { $ids }
apikeys-summary-expires = caduca el { $date }
apikeys-summary-rate = { $n } solic./min
apikeys-summary-ips = IP: { $ips }
apikeys-summary-mcp-tools = Herramientas MCP: { $tools }
apikeys-summary-unattended = Envío DESATENDIDO

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Claves MCP
apikeys-mcp-heading = Claves MCP
apikeys-mcp-intro = Conceda a un agente de IA (mediante el Model Context Protocol) exactamente las herramientas que elija. Los cuerpos de los mensajes devueltos al agente se marcan como entrada no fiable. Cada herramienta se concede individualmente.
apikeys-mcp-label-aria = Etiqueta de la clave MCP
apikeys-mcp-label-placeholder = p. ej., agente asistente
apikeys-mcp-create = Crear clave MCP
apikeys-mcp-reveal-warning = Copie este secreto ahora — se muestra una sola vez.
apikeys-mcp-error-need-label = asigne una etiqueta a la clave MCP
apikeys-mcp-error-need-tool = conceda al menos una herramienta
apikeys-mcp-error-create = no se pudo crear la clave MCP

apikeys-tools = Herramientas
apikeys-tools-group = Herramientas MCP
apikeys-outbox-suffix = (sujeto a la Bandeja de salida)

apikeys-unattended-aria = Envío desatendido
apikeys-unattended-allow = Permitir el envío desatendido (omite la Bandeja de salida — requiere la refrendación de un administrador)
apikeys-unattended-pending = El envío desatendido permanece inactivo hasta que un administrador refrende esta clave.
apikeys-unattended-disclosure = De forma predeterminada, un envío concedido llega a su Bandeja de salida y espera a que usted lo confirme en la aplicación — la automatización no puede enviar por sí sola. El envío desatendido ELIMINA ese paso de intervención humana, de modo que esta clave puede enviar correo sin confirmación. Además, requiere que un administrador refrende la clave. Concédalo solo a automatizaciones en las que confíe plenamente; una clave de envío desatendido comprometida puede enviar correo en su nombre sin ningún aviso.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Buscar correo
apikeys-tool-mail-search-desc = Buscar mensajes. Los resultados se marcan como entrada no fiable.
apikeys-tool-mail-read-label = Leer correo
apikeys-tool-mail-read-desc = Leer el cuerpo de un mensaje. Los cuerpos se marcan como entrada no fiable.
apikeys-tool-folders-list-label = Listar carpetas
apikeys-tool-folders-list-desc = Listar las carpetas del buzón.
apikeys-tool-drafts-create-label = Crear borradores
apikeys-tool-drafts-create-desc = Crear un borrador de mensaje (nunca se envía automáticamente).
apikeys-tool-mail-send-label = Enviar correo
apikeys-tool-mail-send-desc = Poner un mensaje en la Bandeja de salida para confirmarlo en la aplicación.
apikeys-tool-calendar-read-label = Leer calendario
apikeys-tool-calendar-read-desc = Leer eventos del calendario.
apikeys-tool-calendar-propose-label = Proponer eventos
apikeys-tool-calendar-propose-desc = Proponer (sin confirmar) eventos del calendario.
apikeys-tool-tasks-read-label = Leer tareas
apikeys-tool-tasks-read-desc = Leer tareas.
apikeys-tool-tasks-write-label = Escribir tareas
apikeys-tool-tasks-write-desc = Crear o actualizar tareas.
apikeys-tool-contacts-read-label = Leer contactos
apikeys-tool-contacts-read-desc = Leer contactos.
