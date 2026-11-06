# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Contenido remoto

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = No se pudieron actualizar los permisos de las imágenes remotas. { $reason }
remote-banner-blocked = Imágenes remotas bloqueadas
remote-banner-blocked-policy = Imágenes remotas bloqueadas por sus ajustes de privacidad
remote-banner-shown = Imágenes mostradas
remote-banner-proxied = Imágenes cargadas a través del servidor
remote-banner-direct = Imágenes cargadas desde los servidores del remitente
remote-banner-trackers = { $count ->
    [one] { $count } rastreador eliminado
   *[other] { $count } rastreadores eliminados
}
remote-banner-trackers-limited = Al menos { $count } rastreadores eliminados
remote-banner-show = Mostrar imágenes
remote-banner-show-help = Cargar las imágenes de este mensaje a través del servidor, para que el remitente no vea su dirección IP
remote-banner-more = Más opciones de imágenes
remote-banner-always-sender = Mostrar siempre las imágenes de { $sender }
remote-banner-always-domain = Mostrar siempre las imágenes de { $domain }
remote-banner-always-all = Mostrar siempre las imágenes en todo el correo
remote-banner-settings = Ajustes de privacidad
remote-banner-undo = Deshacer
remote-banner-undo-help-single = Volver a bloquear las imágenes de este mensaje
remote-banner-undo-help-per-sender = Dejar de mostrar automáticamente las imágenes de { $sender }
remote-banner-undo-help-per-domain = Dejar de mostrar automáticamente las imágenes de { $domain }
remote-banner-undo-help-all = Dejar de mostrar automáticamente las imágenes en todo el correo
remote-banner-blocked-help = Las imágenes alojadas en otros servidores pueden indicar al remitente cuándo y dónde abrió este mensaje. Permanecen bloqueadas hasta que decida mostrarlas.
remote-banner-help-block = Sus ajustes de privacidad bloquean todas las imágenes remotas. Cámbielo en los ajustes de privacidad.
remote-banner-help-ask = Las imágenes se cargan a través de este servidor, que oculta su dirección IP al remitente.
remote-banner-help-proxy = Las imágenes se cargan a través de este servidor, que oculta su dirección IP al remitente.
remote-banner-help-direct = Las imágenes se cargan directamente desde los servidores del remitente, que pueden ver su dirección IP.
remote-banner-hosts = Hosts bloqueados: { $hosts }
remote-banner-trackers-help = Las imágenes de seguimiento se eliminan antes de mostrar el mensaje. Las imágenes que el servidor solo reconoce como rastreadores después de descargarlas se cuentan durante un tiempo limitado.
remote-images-toast-loading = Cargando imágenes…
remote-images-toast-done = Imágenes cargadas
remote-images-toast-failed = { $count ->
    [one] { $count } imagen no se pudo cargar
   *[other] { $count } imágenes no se pudieron cargar
}
remote-images-toast-retry = Reintentar
remote-images-toast-progress = Cargando imágenes… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Cargando imágenes… { $loaded }/{ $total } · Reintento { $attempt }/{ $max }
remote-images-toast-cancel = Cancelar
remote-images-toast-cancelled = Carga de imágenes cancelada
remote-images-toast-load-again = Cargar de nuevo
remote-banner-hosts-manage = Orígenes de imágenes…
image-hosts-title = Imágenes por origen
image-hosts-help = Elija qué ocurre con las imágenes de cada origen. Nunca mostrar prevalece sobre Mostrar siempre y se aplica a todos los mensajes.
image-hosts-none = Este mensaje no tiene imágenes remotas.
image-hosts-for = Imágenes de { $host }
image-hosts-allow = Mostrar siempre
image-hosts-ask = Preguntar
image-hosts-block = Nunca mostrar
image-hosts-failed = No se pudo guardar el ajuste del origen de imágenes.
image-tip-blocked = Imagen bloqueada. Elija Mostrar imágenes para cargarla.
image-tip-loading = Cargando imagen…
image-tip-error = Imagen no disponible
