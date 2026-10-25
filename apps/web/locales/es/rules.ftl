# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: rules. Source: ../en/rules.ftl

rules-title = Reglas y filtros
rules-intro = Administra scripts de filtrado y reglas visuales. Las funciones disponibles dependen de la cuenta y de los ajustes del administrador.
rules-empty = Todavía no hay reglas.
rules-unavailable = Este servidor de correo no admite reglas de correo.
rules-load-failed = No se pudieron cargar las reglas.
rules-new = Nueva regla
rules-list-label = Reglas de correo
rules-enabled = Activada
rules-delete-named = Eliminar regla { $name }

rules-builder-title = Editor de reglas
rules-builder-hint = Seleccione una regla para editarla o cree una nueva.
rules-name-label = Nombre de la regla
rules-match-legend = Cuándo ejecutar
rules-match-all = Cumplir todas las condiciones
rules-match-any = Cumplir cualquier condición
rules-conditions-legend = Condiciones
rules-actions-legend = Acciones
rules-add-condition = Añadir condición
rules-add-action = Añadir acción
rules-remove-condition = Quitar condición
rules-remove-action = Quitar acción
rules-cond-type-label = Campo
rules-cond-op-label = Operador
rules-cond-value-label = Valor que buscar
rules-act-type-label = Acción
rules-act-value-label = Valor de la acción
rules-save = Guardar regla
rules-editing = Editando { $name }
rules-save-refused = La regla no se guardó: { $reason }

rules-cond-from = De
rules-cond-to = Para
rules-cond-subject = Asunto
rules-cond-thread-unsupported = Hilo { $op } { $value }: las condiciones de hilo no se admiten, así que la regla no se puede guardar hasta que se quite esta condición.
rules-op-contains = contiene
rules-op-is = es

rules-act-move = Mover a buzón
rules-act-tag = Añadir etiqueta
rules-act-archive = Archivar
rules-act-suppress = Suprimir notificación
rules-act-stop = Detener el procesamiento

rules-runs-server = Servidor (Sieve)
rules-runs-server-detail = Se sube al servidor de correo y se aplica allí.
rules-runs-engine = Motor
rules-runs-engine-detail = Se aplica al sincronizar los mensajes nuevos; las respuestas y los movimientos esperan a la próxima sincronización.

rules-panes-label = Herramientas de reglas
rules-tab-builder = Editor
rules-tab-raw = Sieve sin procesar
rules-tab-dryrun = Simulación

rules-raw-label = Código Sieve sin procesar
rules-raw-reset = Restablecer al generado
rules-lint-clean = No se encontraron problemas.

rules-dryrun-title = Simulación
rules-dryrun-help = Introduzca un mensaje de ejemplo para ver qué reglas se activarían.
rules-dryrun-results = Resultados de la simulación
rules-dryrun-none = No hay reglas activadas que evaluar.
rules-dryrun-nomatch = Sin coincidencia
rules-dryrun-stopped = No alcanzada (una regla anterior detuvo el procesamiento)

rules-scripts-label = Scripts de Sieve
rules-script-active = Activo
rules-script-new = Nuevo script
rules-script-export = Exportar script
rules-script-import = Importar script
rules-script-too-large = El script supera el límite de tamaño.
rules-script-read-only = La edición directa de scripts está desactivada por el administrador.
rules-script-validate = Validar script
rules-script-deactivate = Desactivar
rules-script-activate = Activar
rules-script-saved = Script guardado.

rules-vacation-title = Respuesta de ausencia
rules-vacation-enabled = Activar respuestas automáticas
rules-vacation-from = Inicio (UTC)
rules-vacation-to = Fin (UTC)
rules-vacation-body = Texto de respuesta
rules-vacation-saved = Respuesta de ausencia guardada.
rules-script-template = Plantilla de script
rules-forward-title = Reenvío
rules-forward-address = Reenviar a
rules-forward-copy = Conservar una copia en esta cuenta
rules-forward-refused = El administrador no permite reenviar a este dominio.
