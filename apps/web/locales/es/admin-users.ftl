# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Usuarios. Los encabezados de columna ordenan la lista.
admin-userlist-search = Buscar usuarios
admin-userlist-search-placeholder = Nombre, dirección, alias o dominio
admin-userlist-count = { $count ->
    [one] { $count } usuario
   *[other] { $count } usuarios
}
admin-userlist-count-filtered = { $count ->
    [one] { $count } de { $total } usuarios coincide
   *[other] { $count } de { $total } usuarios coinciden
}
admin-userlist-range = { $from }–{ $to } de { $total }
admin-userlist-loading = Cargando usuarios…
admin-userlist-retry = Reintentar
admin-userlist-empty-filtered = Ningún usuario coincide con esta búsqueda y estos filtros.
admin-userlist-compact = Filas compactas
admin-userlist-columns = Columnas
admin-userlist-columns-legend = Mostrar columnas
admin-userlist-actions = Acciones
admin-userlist-paging = Páginas de usuarios
admin-userlist-page-size = Filas por página
admin-userlist-prev = Anterior
admin-userlist-next = Siguiente
admin-userlist-sort-by = Ordenar por
admin-userlist-sort-asc = { $column }, ascendente
admin-userlist-sort-desc = { $column }, descendente
admin-userlist-col-user = Usuario
admin-userlist-col-domain = Dominio
admin-userlist-col-status = Estado
admin-userlist-col-2fa = Doble factor
admin-userlist-col-role = Rol de consola
admin-userlist-col-last-signin = Último inicio de sesión
admin-userlist-col-last-seen = Última actividad
admin-userlist-col-created = Creado
admin-userlist-col-quota = Cuota
admin-userlist-col-profile = Perfil de funciones
admin-userlist-col-linked = Buzones vinculados
admin-userlist-state-active = Activo
admin-userlist-state-pending = Nunca ha iniciado sesión
admin-userlist-state-blocked = Bloqueado
admin-userlist-state-disabled = Desactivado
admin-userlist-state-deleted = Eliminado
admin-userlist-2fa-passkey = Clave de acceso
admin-userlist-2fa-totp = App de autenticación
admin-userlist-2fa-none = Ninguno
admin-userlist-signin-never = Nunca
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Sin actividad registrada
admin-userlist-seen-at = Última actividad { $relative } ({ $exact })
admin-userlist-quota-unlimited = Ilimitada
admin-userlist-profile-default = Predeterminado
admin-userlist-profile-via-user = Asignado a este usuario
admin-userlist-profile-via-group = Desde un grupo
admin-userlist-profile-via-domain = Desde el dominio
admin-userlist-more = Más acciones
admin-userlist-more-for = Más acciones para { $account }
admin-userlist-select = Seleccionar { $account }
admin-userlist-select-page = Seleccionar todos los usuarios de esta página
admin-userlist-filters = Filtros
admin-userlist-add-filter = Añadir filtro
admin-userlist-clear-all = Borrar todo
admin-userlist-chip-edit = Editar filtro { $label }
admin-userlist-chip-remove = Quitar filtro { $label }
admin-userlist-filter-domain = Dominio
admin-userlist-filter-status = Estado
admin-userlist-filter-tfa = Doble factor
admin-userlist-filter-role = Rol de consola
admin-userlist-filter-profile = Perfil de funciones
admin-userlist-filter-signin = Último inicio de sesión
admin-userlist-filter-created = Creado
admin-userlist-filter-no-choices = Aún no hay nada que elegir.
admin-userlist-filter-remove = Quitar filtro
admin-userlist-apply = Aplicar
admin-userlist-cancel = Cancelar
admin-userlist-role-any = Cualquier rol de consola
admin-userlist-role-none = Sin rol de consola
admin-userlist-profile-none = Sin perfil (predeterminado)
admin-userlist-signin-older = { $days ->
    [one] Hace más de { $days } día
   *[other] Hace más de { $days } días
}
admin-userlist-signin-within = { $days ->
    [one] En el último { $days } día
   *[other] En los últimos { $days } días
}
admin-userlist-signin-custom = Hace más de esta cantidad de días:
admin-userlist-signin-days = Días
admin-userlist-days-invalid = Introduzca un número entero de días entre 1 y 36500.
admin-userlist-created-from = Desde
admin-userlist-created-to = Hasta
admin-userlist-created-between = del { $from } al { $to }
admin-userlist-created-since = desde el { $date }
admin-userlist-created-until = hasta el { $date }
admin-userlist-dates-invalid = La fecha de inicio es posterior a la fecha de fin.
admin-userlist-views = Vistas guardadas
admin-userlist-view-save = Guardar esta vista…
admin-userlist-view-name = Nombre de la vista
admin-userlist-view-save-confirm = Guardar vista
admin-userlist-view-delete = Eliminar la vista «{ $name }»
admin-userlist-bulk = Acciones sobre los usuarios seleccionados
admin-userlist-selected = { $count ->
    [one] { $count } seleccionado
   *[other] { $count } seleccionados
}
admin-userlist-clear-selection = Borrar selección
admin-userlist-block = Bloquear inicio de sesión
admin-userlist-unblock = Desbloquear inicio de sesión
admin-userlist-sign-out = Cerrar sesión en todas partes
admin-userlist-delete = Eliminar acceso local
admin-userlist-bulk-profile = Asignar perfil de funciones
admin-userlist-bulk-profile-clear = Quitar el perfil propio del usuario
admin-userlist-confirm-skipped = { $count ->
    [one] Se omite { $count } usuario seleccionado: la acción no se aplica a él.
   *[other] Se omiten { $count } usuarios seleccionados: la acción no se aplica a ellos.
}
admin-userlist-confirm-block = { $count ->
    [one] ¿Bloquear el inicio de sesión de { $count } usuario?
   *[other] ¿Bloquear el inicio de sesión de { $count } usuarios?
}
admin-userlist-confirm-block-one = ¿Bloquear el inicio de sesión de { $account }?
admin-userlist-confirm-block-body = Se rechazan los nuevos inicios de sesión y finalizan las sesiones actuales de Mailwoman. El buzón de origen no se modifica.
admin-userlist-confirm-block-action = Bloquear inicio de sesión
admin-userlist-confirm-unblock = { $count ->
    [one] ¿Desbloquear el inicio de sesión de { $count } usuario?
   *[other] ¿Desbloquear el inicio de sesión de { $count } usuarios?
}
admin-userlist-confirm-unblock-one = ¿Desbloquear el inicio de sesión de { $account }?
admin-userlist-confirm-unblock-body = Se elimina el bloqueo local de inicio de sesión. Las demás restricciones de seguridad siguen vigentes.
admin-userlist-confirm-unblock-action = Desbloquear inicio de sesión
admin-userlist-confirm-signout = { $count ->
    [one] ¿Cerrar la sesión de { $count } usuario en todas partes?
   *[other] ¿Cerrar la sesión de { $count } usuarios en todas partes?
}
admin-userlist-confirm-signout-one = ¿Cerrar la sesión de { $account } en todas partes?
admin-userlist-confirm-signout-body = Finalizan todas las sesiones abiertas de Mailwoman de estos usuarios. Pueden volver a iniciar sesión.
admin-userlist-confirm-signout-action = Cerrar sesión
admin-userlist-confirm-delete = { $count ->
    [one] ¿Eliminar el acceso local de { $count } usuario?
   *[other] ¿Eliminar el acceso local de { $count } usuarios?
}
admin-userlist-confirm-delete-one = ¿Eliminar el acceso local de { $account }?
admin-userlist-confirm-delete-body = Se conserva un registro de acceso eliminado para que el usuario no se inscriba de nuevo automáticamente. Los ajustes de Mailwoman, el historial y el buzón de origen permanecen almacenados. Escriba la dirección para confirmar.
admin-userlist-confirm-delete-action = Eliminar acceso local
admin-userlist-confirm-profile = { $count ->
    [one] ¿Cambiar el perfil de funciones de { $count } usuario?
   *[other] ¿Cambiar el perfil de funciones de { $count } usuarios?
}
admin-userlist-confirm-profile-one = ¿Cambiar el perfil de funciones de { $account }?
admin-userlist-confirm-profile-body = El perfil se asigna directamente a cada usuario y se aplica desde su próxima solicitud. Se admiten identidades de buzón nativas, de proxy y SSO.
admin-users-profile-help = Seleccione Predeterminado para quitar la asignación directa de este usuario y usar los perfiles heredados de sus grupos o su dominio.
admin-users-profile-ambiguous = Este usuario tiene varias identidades de inicio de sesión. Elija la identidad deseada en Perfiles de funciones.
admin-users-profile-unavailable = No hay ninguna identidad de buzón verificada. Este usuario debe iniciar sesión antes de poder asignarle un perfil.
admin-userlist-confirm-profile-action = Cambiar perfil
admin-userlist-bulk-done-block = { $count ->
    [one] Inicio de sesión bloqueado para { $count } usuario.
   *[other] Inicio de sesión bloqueado para { $count } usuarios.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Inicio de sesión desbloqueado para { $count } usuario.
   *[other] Inicio de sesión desbloqueado para { $count } usuarios.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] Sesión cerrada para { $count } usuario.
   *[other] Sesión cerrada para { $count } usuarios.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Acceso local eliminado para { $count } usuario.
   *[other] Acceso local eliminado para { $count } usuarios.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Perfil de funciones cambiado para { $count } usuario.
   *[other] Perfil de funciones cambiado para { $count } usuarios.
}
admin-userlist-bulk-failed = { $done } completadas, { $failed } con error: { $reason }
