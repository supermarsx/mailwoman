# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Supresión y depuración
compliance-card-erasure-coverage = Sustituya los identificadores de una persona en el historial de auditoría y en los registros, y vea qué copias de seguridad aún contienen a personas suprimidas.
admin-erasure-title = Supresión y depuración
admin-erasure-intro = «Depurar persona» sustituye la dirección y los ID de cuenta de una persona en el historial de auditoría y en los archivos de registro rotados por un seudónimo estable generado con una clave. La cadena de hashes de auditoría se vuelve a sellar para que la verificación siga siendo válida. Cada supresión y depuración deja además una marca de supresión, de modo que al restaurar una copia de seguridad más antigua se vuelve a suprimir a esas personas.
admin-erasure-docs = El registro de lo que se suprime, se exporta y se conserva en cada tabla está en docs/admin/erasure-coverage.md.
admin-erasure-scrub = Depurar persona
admin-erasure-scrub-note = Ejecute primero la vista previa: cuenta las filas y líneas de registro que cambiarían y no modifica nada.
admin-erasure-address = Dirección
admin-erasure-preview = Vista previa
admin-erasure-run = Depurar persona
admin-erasure-dry-note = Solo vista previa. No se ha cambiado nada.
admin-erasure-done-note = Depurada. La persona aparece ahora como { $pseudonym }.
admin-erasure-nothing = Ningún historial nombra a esta persona.
admin-erasure-col-table = Tabla o archivo
admin-erasure-col-rows = Filas
admin-erasure-confirm-title = ¿Depurar a esta persona del historial?
admin-erasure-confirm-body = Las entradas de auditoría y las líneas de registro que nombran a la persona se reescriben con el seudónimo. No se puede deshacer; sin la clave no es posible recuperar la dirección a partir del seudónimo.
admin-erasure-settings = Ajustes
admin-erasure-keys = Claves de seudónimos
admin-erasure-keys-note = Los seudónimos se generan con la generación de clave actual. Al rotar se añade una generación; las anteriores permanecen hasta que la retención establecida arriba las elimine.
admin-erasure-rotate = Rotar clave
admin-erasure-keys-none = Todavía no se ha creado ninguna clave. La primera depuración crea una.
admin-erasure-col-gen = Generación
admin-erasure-col-created = Creada
admin-erasure-col-state = Estado
admin-erasure-key-current = Actual
admin-erasure-key-retired = Retirada, conservada
admin-erasure-key-destroy = Retirada, se eliminará el { $when }
admin-erasure-chain = Cadena de auditoría
admin-erasure-chain-note = Cada entrada de auditoría contiene el hash de la anterior. Una depuración reescribe entradas, vuelve a sellar la cadena y registra el hash de cabecera antiguo y el nuevo.
admin-erasure-seal = Sellar las entradas nuevas
admin-erasure-chain-ok = La cadena se verifica: { $rows } entradas selladas, { $unsealed } pendientes de sellar.
admin-erasure-chain-bad = La cadena no se verifica; primera entrada incorrecta en la posición { $seq }.
admin-erasure-col-time = Hora
admin-erasure-col-range = Posiciones
admin-erasure-backups = Copias de seguridad
admin-erasure-backups-note = Una copia de seguridad registra la marca de agua de las supresiones en el momento de crearse. La marca de agua actual es { $watermark }.
admin-erasure-col-label = Etiqueta
admin-erasure-col-location = Ubicación
admin-erasure-col-watermark = Marca de agua
admin-erasure-col-marker = Suprimido desde
admin-erasure-col-actions = Acciones
admin-erasure-backups-none = No hay copias de seguridad registradas.
admin-erasure-contains-erased = Contiene interesados suprimidos ({ $count })
admin-erasure-no-marker = Ninguno
admin-erasure-forget = Olvidar
admin-erasure-register = Registrar copia de seguridad
