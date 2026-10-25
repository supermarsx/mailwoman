# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Subir
nextcloud-loading = Cargando…
nextcloud-list-error = No se pudo listar esta carpeta.
nextcloud-empty = Esta carpeta está vacía.
nextcloud-file-list = Archivos de Nextcloud
nextcloud-open-folder = Abrir carpeta { $name }
nextcloud-select-file = Seleccionar { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Adjuntar desde Nextcloud
nextcloud-attach-action = { $count ->
    [0] Adjuntar
    [one] Adjuntar { $count } archivo
    [many] Adjuntar { $count } de archivos
   *[other] Adjuntar { $count } archivos
  }
nextcloud-error-select-file = seleccione al menos un archivo para adjuntar
nextcloud-error-attach-failed = no se pudieron adjuntar los archivos seleccionados

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Guardar en Nextcloud
nextcloud-save-title = Guardar «{ $name }» en Nextcloud
nextcloud-save-action = Guardar aquí ({ $dir })
nextcloud-saved = Guardado en { $path }
nextcloud-error-save-failed = no se pudo guardar en Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Crear enlace para compartir
nextcloud-share-title = Enlace para compartir
nextcloud-share-intro = Crear un enlace público a { $path } en lugar de adjuntar el archivo.
nextcloud-protect-password = Proteger con contraseña
nextcloud-password-label = Contraseña
nextcloud-share-password = Contraseña del enlace
nextcloud-set-expiry = Establecer fecha de caducidad
nextcloud-expires-on = Caduca el
nextcloud-expiry-date = Fecha de caducidad
nextcloud-create-link = Crear enlace
nextcloud-password-protected = Protegido con contraseña
nextcloud-no-password = Sin contraseña
nextcloud-expires = caduca el { $date }
nextcloud-no-expiry = sin caducidad
nextcloud-error-need-password = introduzca una contraseña o desactive la protección con contraseña
nextcloud-error-need-expiry = elija una fecha de caducidad o desactive la caducidad
nextcloud-error-share-failed = no se pudo crear el enlace para compartir
