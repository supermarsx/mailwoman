# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Contenido remoto

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } imagen remota bloqueada
   *[other] { $count } imágenes remotas bloqueadas
}
remote-blocked-trackers = { $count ->
    [one] { $count } rastreador bloqueado
   *[other] { $count } rastreadores bloqueados
} de { $blocked ->
    [one] { $blocked } imagen remota
   *[other] { $blocked } imágenes remotas
}
remote-blocked-hosts = Hosts bloqueados

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Cargar imágenes
remote-grant-sender = Cargar siempre de { $sender }
remote-grant-domain = Cargar siempre de { $domain }
remote-grant-all = Cargar siempre todas las imágenes remotas

remote-grant-done-single = Imágenes remotas cargadas para este mensaje.
remote-grant-done-per-sender = Las imágenes remotas se cargarán de este remitente.
remote-grant-done-per-domain = Las imágenes remotas se cargarán de este dominio.
remote-grant-done-all = Las imágenes remotas se cargarán en todo el correo.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Se están cargando las imágenes remotas de este mensaje.
remote-revoke = Desactivar
remote-revoke-done = Imágenes remotas desactivadas. Se volverán a bloquear la próxima vez que se abra.
