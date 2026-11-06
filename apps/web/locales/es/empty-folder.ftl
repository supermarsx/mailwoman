# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Vaciar carpeta…
empty-folder-title = ¿Vaciar «{ $name }»?
empty-folder-body-trash = { $count ->
    [one] El mensaje de { $name } se moverá a la papelera, desde donde podrá restaurarlo.
   *[other] Los { $count } mensajes de { $name } se moverán a la papelera, desde donde podrá restaurarlos.
    } Las subcarpetas y los mensajes que contienen no se modifican.
empty-folder-body-destroy-trash = { $count ->
    [one] El mensaje de la papelera se eliminará de forma permanente. Esta acción no se puede deshacer.
   *[other] Los { $count } mensajes de la papelera se eliminarán de forma permanente. Esta acción no se puede deshacer.
    } Las subcarpetas y los mensajes que contienen no se modifican.
empty-folder-body-destroy-junk = { $count ->
    [one] El mensaje de { $name } se eliminará de forma permanente. Esta acción no se puede deshacer.
   *[other] Los { $count } mensajes de { $name } se eliminarán de forma permanente. Esta acción no se puede deshacer.
    } Las subcarpetas y los mensajes que contienen no se modifican.
empty-folder-body-destroy-no-trash = Esta cuenta no tiene papelera, así que { $count ->
    [one] el mensaje de { $name } se eliminará de forma permanente.
   *[other] los { $count } mensajes de { $name } se eliminarán de forma permanente.
    } Esta acción no se puede deshacer. Las subcarpetas y los mensajes que contienen no se modifican.
empty-folder-confirm-trash = Mover a la papelera
empty-folder-confirm-destroy = Eliminar permanentemente
empty-folder-cancel = Cancelar
empty-folder-already-empty = «{ $name }» ya está vacía.
empty-folder-prepare-failed = No se pudieron contar los mensajes de «{ $name }», así que no se cambió nada. Compruebe su conexión e inténtelo de nuevo.
empty-folder-syncing = «{ $name }» aún se está sincronizando, así que todavía no se puede contar. No se cambió nada; inténtelo de nuevo en un momento.
empty-folder-running = Ya se está vaciando una carpeta. Deténgala desde su notificación antes de iniciar otra.
empty-folder-progress-trash = Moviendo mensajes de { $name } a la papelera: { $done } de { $total }.
empty-folder-progress-destroy = Eliminando mensajes de { $name }: { $done } de { $total }.
empty-folder-stop = Detener
empty-folder-stopping = Deteniendo tras el lote actual. Hasta ahora, { $done } de { $total }.
empty-folder-done-trash = { $done ->
    [one] Se movió 1 mensaje de { $name } a la papelera.
   *[other] Se movieron { $done } mensajes de { $name } a la papelera.
}
empty-folder-done-destroy = { $done ->
    [one] Se eliminó 1 mensaje de { $name } de forma permanente.
   *[other] Se eliminaron { $done } mensajes de { $name } de forma permanente.
}
empty-folder-stopped = Detenido. { $done } de { $total } mensajes procesados; quedan { $remaining } en { $name }.
empty-folder-refused = Detenido. { $done } de { $total } mensajes procesados; el servidor de correo rechazó { $refused }. Quedan { $remaining } en { $name }.
empty-folder-failed = El vaciado de { $name } se detuvo con un error tras { $done } mensajes. Puede que el último lote se completara o no; compruebe la carpeta antes de volver a intentarlo.
