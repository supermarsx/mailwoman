# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Carpeta movida: { $name }
folder-dnd-moved-many = { $count } carpetas movidas
folder-dnd-undo = Deshacer
folder-dnd-cancel = Cancelar
folder-dnd-handle = Arrastre { $name } para moverla
folder-dnd-locked-hint = El orden de las carpetas lo fija su administrador.
folder-dnd-forced-hint = Fijada por su administrador. No se puede mover ni desfijar.
folder-dnd-disabled-hint = Arrastrar y soltar carpetas está desactivado.
folder-dnd-system-hint = Las carpetas del sistema no se pueden mover.
folder-dnd-pinned-group = Fijadas
folder-dnd-top-level = Mover al nivel superior
folder-dnd-problem-disabled = Mover carpetas está desactivado.
folder-dnd-problem-locked = El orden de las carpetas lo fija su administrador.
folder-dnd-problem-self = Una carpeta no se puede mover dentro de sí misma.
folder-dnd-problem-descendant = Una carpeta no se puede mover dentro de una de sus propias subcarpetas.
folder-dnd-problem-system = Las carpetas del sistema no se pueden mover aquí.
folder-dnd-problem-no-children = Esa carpeta no puede contener subcarpetas.
folder-dnd-problem-nesting-off = Su administrador no permite anidar carpetas.
folder-dnd-problem-depth = Eso anidaría las carpetas a demasiada profundidad.
folder-dnd-problem-forced = Su administrador ha fijado esta carpeta.
folder-dnd-problem-unknown = Esa carpeta ya no existe.
folder-dnd-problem-noop = La carpeta ya está allí.
folder-dnd-problem-collision = Ya existe allí una carpeta llamada { $name }. Cámbiele el nombre primero.
folder-dnd-problem-server = El servidor no movió { $name }. No se cambió nada.
folder-dnd-announce-up = { $name } movida hacia arriba
folder-dnd-announce-down = { $name } movida hacia abajo
folder-dnd-announce-indent = { $name } movida a la carpeta superior
folder-dnd-announce-outdent = { $name } subida un nivel
folder-dnd-nest-title = ¿Mover { $name } aquí?
folder-dnd-nest-title-many = ¿Mover { $count } carpetas aquí?
folder-dnd-nest-body = Elija si esto cambia la carpeta en el servidor de correo o solo cómo la muestra esta aplicación.
folder-dnd-nest-legend = Cómo moverla
folder-dnd-nest-local = Organizar solo en esta aplicación
folder-dnd-nest-local-help = Solo una vista. La carpeta permanece donde está en el servidor y en otras aplicaciones de correo.
folder-dnd-nest-server = Mover en el servidor
folder-dnd-nest-server-help = Cambia la carpeta en todas partes, en todas las aplicaciones de correo que usan esta cuenta.
folder-dnd-nest-remember = Recordar mi elección
folder-dnd-nest-confirm = Mover
folder-dnd-arranged-only = Organizada solo en esta aplicación
folder-dnd-arranged-only-help = Su ubicación en el servidor es distinta: { $path }
folder-dnd-apply-server = Aplicar en el servidor
folder-dnd-reset = Restablecer el orden predeterminado
folder-dnd-reset-done = Orden de carpetas restablecido
folder-dnd-move-to = Mover { $name } a…
folder-dnd-layout-link = Disposición de las carpetas
folder-dnd-reorder-enable = Reorganizar las carpetas arrastrando
folder-dnd-nest-mode = Cuando anido una carpeta
folder-dnd-nest-mode-ask = Preguntarme
folder-dnd-nest-mode-server = Mover en el servidor
folder-dnd-nest-mode-local = Organizar solo en esta aplicación
folder-dnd-new-placement = Las carpetas nuevas aparecen
folder-dnd-new-placement-end = Después de las carpetas del mismo nivel
folder-dnd-new-placement-alphabetical = En orden alfabético
folder-dnd-handles = Mostrar los tiradores de arrastre
folder-dnd-handles-hover = Al pasar el cursor
folder-dnd-handles-always = Siempre
folder-dnd-arrange-title = Organizar carpetas
folder-dnd-arrange-help = Arrastre una carpeta para reordenarla, suéltela sobre otra carpeta para anidarla, o use Alt y las teclas de flecha en una carpeta enfocada.
folder-dnd-admin-locked = Definido por su administrador
folder-dnd-arrange-menu = Mover a…
folder-star-add = Marcar carpeta con estrella
folder-star-remove = Quitar la estrella de la carpeta
folder-star-starred = Con estrella
folder-star-toggle = Marcar { $name } con estrella
folder-star-only = Mostrar solo las destacadas
folder-star-only-help = Muestra solo las carpetas con estrella y las carpetas que las contienen.
folder-dnd-indent = Mover { $name } a la carpeta superior
folder-dnd-outdent = Subir { $name } un nivel
folder-dnd-more-actions = Organizar
folder-dnd-selected-move = Mover la selección a…
folder-dnd-locked-row = Bloqueada
folder-dnd-problem-pin = Las carpetas fijadas permanecen sobre las demás. Desfíjela primero para moverla allí.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Vacío…

folder-bar-label = Acciones para las carpetas seleccionadas

folder-bar-mark-read = Marcar todo como leído

folder-bar-pin = Alfiler

folder-bar-star = estrella

folder-bar-unpin = Desanclar

folder-bar-unstar = Desestrellar

folder-dnd-handle-hint = Arrastra para mover esta carpeta

folder-pinned-group = Fijado

folder-pinned-unpin-zone = Caída aquí para desanclar

settings-folders-bulk-delete-body = { $folders ->
        [one] Se eliminarán { $folders } carpeta y los { $messages } mensajes que contiene. Esto no se puede deshacer.
        [many] { $folders } carpetas (con sus subcarpetas) y los { $messages } mensajes que contengan se eliminarán. Esto no se puede deshacer.
       *[other] { $folders } carpetas (con sus subcarpetas) y los { $messages } mensajes que contengan se eliminarán. Esto no se puede deshacer.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [one] Eliminar { $count } carpeta
        [many] Eliminar { $count } carpetas
       *[other] Eliminar { $count } carpetas
    }

settings-folders-bulk-delete-done = { $count ->
        [one] Eliminada { $count } carpeta
        [many] Eliminadas { $count } carpetas
       *[other] Eliminadas { $count } carpetas
    }

settings-folders-bulk-delete-failed = No se eliminó nada. No eliminado: { $names }

settings-folders-bulk-delete-list = Carpetas para eliminar

settings-folders-bulk-delete-none = Ninguna de las carpetas seleccionadas se puede eliminar (se omiten las carpetas del sistema y las carpetas que no puede eliminar).

settings-folders-bulk-delete-others = Tampoco eliminado: { $names }

settings-folders-bulk-delete-partial = Carpetas { $done } eliminadas. No eliminado: { $names }

settings-folders-bulk-delete-partial-done = Carpetas { $done } eliminadas.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [one] No eliminado porque la cuenta está bajo retención legal: { $names }
        [many] No eliminado porque la cuenta está bajo retención legal ({ $count } carpetas): { $names }
       *[other] No eliminado porque la cuenta está bajo retención legal ({ $count } carpetas): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [one] No eliminado porque el procesamiento de la cuenta está restringido: { $names }
        [many] No eliminado porque el procesamiento de la cuenta está restringido ({ $count } carpetas): { $names }
       *[other] No eliminado porque el procesamiento de la cuenta está restringido ({ $count } carpetas): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } las carpetas seleccionadas se omiten porque no se pueden eliminar: { $names }

settings-folders-bulk-delete-subfolders = Elimina también las subcarpetas de cada carpeta.

settings-folders-bulk-delete-subfolders-off = Solo se eliminan las carpetas seleccionadas. Se conserva una carpeta que todavía tiene una subcarpeta fuera de la selección.

settings-folders-bulk-delete-title = { $count ->
        [one] ¿Eliminar { $count } carpeta?
        [many] ¿Eliminar { $count } carpetas?
       *[other] ¿Eliminar { $count } carpetas?
    }

settings-folders-bulk-empty-body = { $total } mensajes en carpetas { $folders } se eliminarán: { $toTrash } se mueven a la Papelera y se pueden deshacer, { $permanent } se eliminan definitivamente y no se pueden recuperar. Las subcarpetas no se tocan.

settings-folders-bulk-empty-confirm = vaciar las carpetas

settings-folders-bulk-empty-counting = Contando los mensajes en cada carpeta…

settings-folders-bulk-empty-done = Vació { $count } carpetas, { $messages } mensajes

settings-folders-bulk-empty-fail-refused = el servidor rechazó parte del mismo; { $remaining } mensajes quedan

settings-folders-bulk-empty-fail-stopped = detenido; { $remaining } mensajes quedan

settings-folders-bulk-empty-fail-uncertain = se desconoce el resultado del último lote; revisa la carpeta

settings-folders-bulk-empty-list = Carpetas para vaciar

settings-folders-bulk-empty-partial = Carpetas { $done } vaciadas; { $failed } falló. Ver la lista.

settings-folders-bulk-empty-permanent = { $count } mensajes se eliminan definitivamente

settings-folders-bulk-empty-progress = { $done } de { $of } carpetas terminadas, { $messages } mensajes procesados. Ahora: { $name }

settings-folders-bulk-empty-skip-count-failed = no se pudo contar, se omitió

settings-folders-bulk-empty-skip-empty = ya vacío, saltado

settings-folders-bulk-empty-skip-permanent-denied = se eliminaría definitivamente, lo cual el administrador ha desactivado, omitido

settings-folders-bulk-empty-skip-search = una búsqueda guardada, omitida

settings-folders-bulk-empty-skip-unsupported = esta cuenta no puede vaciarla, omitida

settings-folders-bulk-empty-title = { $count ->
        [one] ¿Vaciar { $count } carpeta?
        [many] ¿Vaciar { $count } carpetas?
       *[other] ¿Vaciar { $count } carpetas?
    }

settings-folders-bulk-empty-to-trash = { $count } mensajes se mueven a la Papelera

settings-folders-bulk-empty-undo-label = Carpetas { $count } vaciadas (mensajes { $messages })

settings-folders-delete-selected = Eliminar…

settings-folders-select-user = Seleccionar todas las carpetas de usuario

settings-folders-subscribe-failed = Las suscripciones no se pudieron cambiar.

settings-folders-subscribe-partial = Se cambiaron { $done } carpetas; { $failed } no se pudo cambiar.

settings-folders-subscribe-selected = Suscríbete

settings-folders-subscribed = { $count ->
        [one] Suscrito a { $count } carpeta
        [many] Suscrito a { $count } carpetas
       *[other] Suscrito a { $count } carpetas
    }

settings-folders-unsubscribe-selected = Darse de baja

settings-folders-unsubscribed = { $count ->
        [one] Cancelar suscripción a { $count } carpeta
        [many] Cancelar suscripción de { $count } carpetas
       *[other] Cancelar suscripción de { $count } carpetas
    }
