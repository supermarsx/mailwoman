# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Perfiles de funciones
admin-features-intro = Un perfil es un conjunto con nombre de interruptores de funciones. Asígnelo a un dominio, un grupo o un usuario. Las asignaciones individuales admiten buzones nativos e identidades de proxy o SSO verificadas; controlan el acceso a las funciones de Mailwoman.
admin-features-new = Nuevo perfil
admin-features-search = Buscar perfiles
admin-features-empty = Aún no hay perfiles de funciones.
admin-features-no-match = Ningún perfil coincide con la búsqueda.
admin-features-load-error = No se pudo cargar el catálogo de funciones.
admin-features-not-loaded = El catálogo de funciones aún no está cargado. Recargue la página e inténtelo de nuevo.
admin-features-stale = Otro administrador cambió el catálogo de funciones. Se ha recargado; compruebe los valores y guarde de nuevo.
admin-features-not-found = No hay ningún perfil con el id { $id }.
admin-features-col-name = Nombre
admin-features-col-values = Interruptores
admin-features-col-assigned = Asignado a
admin-features-values-count = { $count ->
    [one] { $count } interruptor definido
   *[other] { $count } interruptores definidos
}
admin-features-assigned-count = { $count ->
    [0] Sin asignar
    [one] { $count } lugar
   *[other] { $count } lugares
}
admin-features-editor = Ajustes del perfil
admin-features-name = Nombre del perfil
admin-features-name-help = Visible solo para los administradores, hasta 128 caracteres.
admin-features-name-invalid = Escriba un nombre de 1 a 128 caracteres.
admin-features-switches = Interruptores de funciones
admin-features-switches-help = «Sin definir» deja la función al siguiente perfil, al dominio o al valor predeterminado del servidor. Un valor bloqueado no puede cambiarlo el usuario ni un nivel inferior.
admin-features-col-feature = Función
admin-features-col-value = Valor
admin-features-col-lock = Bloqueo
admin-features-value-for = Valor para { $feature }
admin-features-unset = Sin definir
admin-features-on = Permitido
admin-features-off = No permitido
admin-features-lock = Bloqueado
admin-features-save = Guardar perfil
admin-features-create = Crear perfil
admin-features-delete = Eliminar perfil
admin-features-delete-title = ¿Eliminar el perfil { $name }?
admin-features-delete-body = { $count ->
    [0] El perfil no está asignado en ningún sitio. Sus interruptores dejan de aplicarse de inmediato.
    [one] El perfil se quita del { $count } lugar al que está asignado y sus interruptores dejan de aplicarse de inmediato.
   *[other] El perfil se quita de los { $count } lugares a los que está asignado y sus interruptores dejan de aplicarse de inmediato.
}
admin-features-saved = Perfil { $name } guardado.
admin-features-deleted = Perfil { $name } eliminado.
admin-features-assigned = Perfil { $name } asignado.
admin-features-unassigned = Asignación de { $name } quitada.
admin-features-assignments = Asignado a
admin-features-assignments-help = Los perfiles se aplican por orden de prioridad, primero los más altos. Con la misma prioridad, una asignación a un usuario se aplica antes que la de un grupo, y la de un grupo antes que la de un dominio.
admin-features-assignments-empty = Este perfil aún no está asignado.
admin-features-assign-form = Asignar este perfil
admin-features-col-scope = Se aplica a
admin-features-col-target = Nombre
admin-features-col-priority = Prioridad
admin-features-col-actions = Acciones
admin-features-scope-domain = Dominio
admin-features-scope-group = Grupo
admin-features-scope-user = Usuario
admin-features-assign = Asignar
admin-features-unassign = Quitar
admin-features-no-domain = No hay dominios configurados.
admin-features-no-group = No existen grupos de usuarios.
admin-features-no-user = Ninguna cuenta de buzón ha iniciado sesión todavía.
admin-features-tools = Herramientas del perfil
admin-features-preset = Empezar desde un ajuste preestablecido
admin-features-preset-choose = Elegir un ajuste preestablecido
admin-features-preset-apply = Aplicar ajuste preestablecido
admin-features-preset-applied = Ajuste preestablecido { $name } aplicado. Revise los valores y guarde.
admin-features-preset-full = Completo
admin-features-preset-standard = Estándar
admin-features-preset-mail-only = Solo correo
admin-features-preset-minimal = Mínimo / quiosco
admin-features-import = Importar JSON
admin-features-export = Exportar JSON
admin-features-exported = Perfil exportado como archivo JSON.
admin-features-imported = { $count ->
    [one] Se importó { $count } valor. Revíselo y guarde.
   *[other] Se importaron { $count } valores. Revíselos y guarde.
}
admin-features-import-json = El archivo no es JSON válido.
admin-features-import-format = El archivo no es un perfil de funciones de Mailwoman.
admin-features-import-version = Este archivo de perfil de funciones procede de una versión más reciente y no se puede importar.
admin-features-clone = Clonar
admin-features-clone-name = Copia de { $name }
admin-features-filter = Buscar una función
admin-features-filter-placeholder = Nombre, ajuste o clave
admin-features-filter-none = Ninguna función coincide con la búsqueda.
admin-features-diff-summary = { $differs ->
    [one] { $differs } valor difiere del predeterminado del servidor
   *[other] { $differs } valores difieren del predeterminado del servidor
} · { $same } lo repiten
admin-features-differs = Difiere del predeterminado
admin-features-default-is = Predeterminado: { $value }
admin-features-fix-values = Corrija los valores marcados como no válidos antes de guardar.
admin-features-allow-all = Permitir todo
admin-features-deny-all = No permitir nada
admin-features-clear-all = Borrar
admin-features-group-allow = Permitir todas las funciones de { $group }
admin-features-group-deny = No permitir ninguna función de { $group }
admin-features-group-clear = Borrar todos los valores de { $group }
admin-features-enforcement-ui = Oculto solo en la aplicación
admin-features-enforcement-partial = Algunos límites aún no se cuentan
admin-features-other = Otros ajustes que puede contener un perfil
admin-features-other-help = Cualquier otro ajuste que un administrador puede definir por perfil. Deje un campo vacío para conservar el valor del servidor.
admin-features-cat-compose = Correo y redacción
admin-features-cat-reading = Lectura y privacidad
admin-features-cat-views = Vistas y búsqueda
admin-features-cat-organise = Organizar
admin-features-cat-pim = Calendario, contactos, tareas, notas y claves
admin-features-cat-data = Datos
admin-features-cat-accounts = Cuentas y acceso
admin-features-cat-integrations = Integraciones
admin-features-cat-developer = Acceso de desarrollador
admin-features-cat-personal = Personalización
admin-features-feature-compose-sending = Límites de envío
admin-features-feature-compose-sending-help = El servidor rechaza los destinatarios por mensaje que superen el límite. Los mensajes por hora y por día se guardan en el perfil, pero aún no se cuentan.
admin-features-feature-storage = Cuota de almacenamiento
admin-features-feature-storage-help = El límite de almacenamiento se guarda en el perfil; la cuota propia del servidor de correo sigue aplicándose.
admin-features-feature-access-protocols = Protocolos de acceso al correo
admin-features-feature-access-protocols-help = Con qué protocolos del servidor de correo pueden iniciar sesión estos usuarios, separados por comas: imap, pop3, ews.
admin-features-feature-accounts-linked = Buzones vinculados
admin-features-feature-calendar = Calendario
admin-features-feature-contacts = Contactos
admin-features-feature-tasks = Tareas
admin-features-feature-notes = Notas
admin-features-preview = Valores efectivos
admin-features-preview-help = En qué se resuelve cada función para un usuario o para un usuario de un dominio, tras aplicar todos los perfiles, grupos, valores de dominio y bloqueos.
admin-features-preview-kind = Vista previa para
admin-features-preview-target = Quién
admin-features-preview-show = Mostrar
admin-features-preview-loading = Cargando…
admin-features-preview-state = Efectivo
admin-features-preview-from = Decidido por
admin-features-preview-profiles = Perfiles que se aplican, gana el primero: { $names }
admin-features-preview-no-profiles = No se aplica ningún perfil; deciden los valores del dominio y del servidor.
admin-features-source-default = Predeterminado integrado
admin-features-source-env-fallback = Predeterminado del entorno
admin-features-source-env-pin = Entorno (fijado)
admin-features-source-global = Valor del servidor
admin-features-source-domain = Valor del dominio
admin-features-source-profile = Perfil
admin-features-source-profile-named = Perfil { $name }
admin-features-source-group = Valor del grupo
admin-features-source-admin-user = Valor definido para este usuario
admin-features-source-user = Elección propia del usuario
admin-features-source-locked = { $source } (bloqueado)
admin-retention-col-category = Categoría
admin-retention-col-kept = Conservado
admin-retention-col-change = Cambiar
admin-retention-cat-default = Predeterminado
admin-retention-cat-admin = Acciones de administrador
admin-retention-cat-security = Eventos de seguridad
admin-retention-cat-alert = Alertas de usuario
admin-retention-forever = Todas las entradas
admin-retention-days = { $days ->
    [one] { $days } día
   *[other] { $days } días
}
admin-retention-follows-default = Predeterminado: { $kept }
admin-retention-hold-on = Retención legal
admin-retention-read-only = Cambiar la retención requiere el permiso settings.write.
admin-retention-edit-for = Retención para { $category }
admin-retention-keep = Conservar
admin-retention-mode-inherit = Seguir el predeterminado
admin-retention-mode-forever = Todas las entradas
admin-retention-mode-days = Durante un número de días
admin-retention-days-label = Días
admin-retention-hold = Retención legal
admin-retention-save = Guardar
admin-retention-bounds = Introduzca un número entero de { $min } a { $max }.
admin-retention-saved = Retención para { $category } guardada.
admin-retention-task-none = Aún no hay tarea de depuración. Se crea, cada día a las 03:15 UTC, cuando se guarda por primera vez un número de días.
admin-retention-task-disabled = La tarea de depuración programada está desactivada; las entradas solo se depuran con Depurar ahora.
admin-retention-task-next = Próxima depuración programada: { $when }.
admin-retention-task-daily = Se depura a diario a las 03:15 UTC.
admin-retention-last = Última depuración que eliminó entradas: { $when }.
admin-retention-prune = Depurar ahora
admin-retention-prune-title = ¿Depurar ahora el registro de auditoría?
admin-retention-prune-body = Se eliminan las entradas más antiguas que la retención de su categoría. Las categorías que conservan todas las entradas o están bajo retención legal no se modifican. No se puede deshacer.
admin-retention-pruned = { $count ->
    [0] Ninguna entrada era lo bastante antigua para eliminarla.
    [one] Se eliminó { $count } entrada.
   *[other] Se eliminaron { $count } entradas.
}
admin-auth-attempted = Intentado como { $name }
admin-features-feature-compose-open-tracking = Seguimiento de apertura de mensajes
admin-features-feature-compose-open-tracking-help = Permite registrar las aperturas de mensajes con formato aptos mediante la imagen de seguimiento del servidor. Requiere una URL pública configurada.
admin-features-feature-reading-original-html = Vista del HTML original
admin-features-feature-reading-original-html-help = Permite a los usuarios activar el HTML original en mensajes concretos, dentro del marco de lectura aislado. El HTML depurado sigue siendo el predeterminado.
admin-features-inherit = Heredar
admin-features-override = Definir un valor
admin-features-setting-mode = Modo de valor para { $setting }
admin-features-inherit-help = Heredar quita el valor y el bloqueo de este perfil. Decide el siguiente perfil aplicable, el dominio o el valor del servidor; la vista previa de valores efectivos muestra el resultado y su origen.
admin-features-domain-alias = { $alias } (alias de { $domain })
admin-features-domain-aliases = { $domain } (alias: { $aliases })
admin-audit-retention-title = Conservación del registro de auditoría
admin-audit-retention-intro = Cuánto tiempo se conservan las entradas de auditoría, por categoría. Una categoría sin valor propio sigue el predeterminado; sin predeterminado se conservan todas las entradas. Una retención legal detiene la depuración de su categoría, y una retención sobre el predeterminado la detiene en todas. Los cambios y las depuraciones quedan registrados en el registro de auditoría.
admin-audit-retention-loading = Cargando la política de conservación…
admin-audit-retention-unavailable = La conservación de auditoría no está disponible en este servidor.
