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

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } supera el límite de archivos adjuntos. Súbalo a la carpeta «Mail attachments» de su Nextcloud e inserte en su lugar un enlace para compartir en el mensaje.
nextcloud-upload-and-link = Subir e insertar enlace
nextcloud-share-not-now = Ahora no
nextcloud-upload-progress = Subiendo: { $percent } %
nextcloud-upload-progress-label = Progreso de la subida
nextcloud-share-inserted = Se insertó en el mensaje el enlace para compartir de { $name }.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = El enlace funciona hasta el { $date }.
nextcloud-share-body-password = El enlace está protegido con contraseña.
nextcloud-account-title = Cuentas de Nextcloud
nextcloud-account-help = Vincule su propia cuenta a una instancia aprobada por su administrador. Las credenciales se almacenan cifradas solo para esta cuenta de correo.
nextcloud-account-load-error = No se pudieron cargar los vínculos de Nextcloud. Recargue para intentarlo de nuevo.
nextcloud-account-unlink-error = No se pudo eliminar el vínculo. Recargue antes de intentarlo de nuevo.
nextcloud-account-reload = Recargar
nextcloud-account-no-instances = No hay instancias de Nextcloud habilitadas. Pida a su administrador que configure una.
nextcloud-account-ready = Vinculada y lista
nextcloud-account-stale = La configuración de la instancia cambió. Verifique el vínculo de nuevo.
nextcloud-account-unlinked = No vinculada
nextcloud-account-relink = Verificar el vínculo de nuevo
nextcloud-account-link = Vincular cuenta
nextcloud-account-unlink = Desvincular cuenta
nextcloud-account-password-help = En los ajustes de seguridad personales de Nextcloud, cree una contraseña de aplicación y copie aquí su nombre de usuario y su contraseña. La verificación realiza una sola solicitud acotada; las credenciales rechazadas no se reintentan.
nextcloud-account-username = Nombre de usuario de la contraseña de aplicación de Nextcloud
nextcloud-account-password = Contraseña de aplicación de Nextcloud
nextcloud-account-verify = Verificar y vincular
nextcloud-account-verify-error = Falló la verificación o el guardado. Compruebe la instancia, el nombre de usuario y la contraseña de aplicación. El campo de contraseña se ha vaciado; no se reemplazó un vínculo existente que funcionaba.
nextcloud-admin-title = Instancias de Nextcloud
nextcloud-admin-help = Apruebe instancias HTTPS que los usuarios de correo puedan vincular individualmente. No se da ninguna contraseña de servicio compartida a las cuentas de correo. Las operaciones con archivos están limitadas a 8 MiB. Los usuarios del motor nativo descubren y conectan explícitamente las colecciones de calendarios, tareas y libretas de direcciones tras vincular.
nextcloud-admin-add = Añadir instancia de Nextcloud
nextcloud-admin-edit = Editar instancia de Nextcloud
nextcloud-admin-enabled = Habilitada
nextcloud-admin-disabled = Deshabilitada
nextcloud-admin-missing = Esta instancia ya no existe. Vuelva a la lista de instancias.
nextcloud-admin-private-help = Las direcciones privadas requieren un permiso de salida de Nextcloud explícito. Las rutas de proxy configuradas siguen siendo las que prevalecen; una solicitud rechazada por el proxy nunca recurre a una conexión directa.
nextcloud-admin-revalidation = Guardar cualquier cambio en una instancia obliga a los usuarios a verificar sus vínculos de nuevo. Deshabilitar una instancia detiene de inmediato las operaciones nuevas.
nextcloud-admin-authority = Autoridad de la instancia
nextcloud-admin-name = Nombre para mostrar
nextcloud-admin-name-help = Un nombre corto que se muestra a los usuarios al elegir una instancia.
nextcloud-admin-url = URL de instalación HTTPS
nextcloud-admin-url-help = Use la raíz de la instalación, como https://cloud.example.org/ o https://example.org/nextcloud/. El certificado debe ser de confianza y coincidir con el nombre de host. Se rechazan las credenciales, los parámetros de consulta y los fragmentos.
nextcloud-admin-enable = Permitir que los usuarios vinculen esta instancia
nextcloud-admin-ca = Autoridades de certificación adicionales (PEM)
nextcloud-admin-ca-help = Certificados de CA privadas opcionales para este servidor de Nextcloud. La confianza pública sigue disponible y la verificación del nombre de host sigue siendo obligatoria. Los cambios obligan a los usuarios a verificar sus vínculos de nuevo; la confianza en los certificados del proxy es independiente.
nextcloud-instance-select = Instancia de Nextcloud
nextcloud-instance-placeholder = Elija una instancia vinculada
nextcloud-instances-empty = No hay vínculos de Nextcloud verificados. Añada o verifique un vínculo en Ajustes de usuario → Nextcloud.
nextcloud-instances-error = No se pudieron cargar las instancias vinculadas. Vuelva a abrir el selector para intentarlo de nuevo.
nextcloud-save-warning = Su administrador exige una confirmación antes de que este archivo adjunto pueda salir del buzón.
nextcloud-save-acknowledge = Entiendo la advertencia sobre el archivo adjunto y quiero guardar este archivo en la instancia de Nextcloud seleccionada.
nextcloud-save-attachment = Guardar { $name } en Nextcloud
nextcloud-save-close = Cerrar el destino de Nextcloud
nextcloud-save-options = Guardar archivos adjuntos en Nextcloud
nextcloud-pim-title = Contactos y calendarios
nextcloud-pim-help = Descubra las colecciones de esta cuenta de Nextcloud verificada y conecte explícitamente las que desee. Sincronice para obtener los datos actuales; las ediciones de las colecciones conectadas usan la colección remota y fallan si se revoca su vínculo o sus permisos.
nextcloud-pim-category = Categoría de colección
nextcloud-pim-calendar = Calendarios
nextcloud-pim-contacts = Libretas de direcciones
nextcloud-pim-tasks = Listas de tareas
nextcloud-pim-discover = Descubrir colecciones
nextcloud-pim-discovered = Colecciones disponibles
nextcloud-pim-empty = No se devolvió ninguna colección coincidente.
nextcloud-pim-connect = Conectar
nextcloud-pim-connected = Colecciones conectadas
nextcloud-pim-sync = Sincronizar ahora
nextcloud-pim-sync-needed = Colección conectada. Sincronice ahora para cargar su contenido.
nextcloud-pim-synced = Sincronización completada.
nextcloud-pim-error = La operación falló. Compruebe el vínculo de la cuenta y los permisos, y vuelva a intentarlo de forma explícita. Los datos ya almacenados en caché no se descartaron.
nextcloud-pim-native-required = La sincronización de contactos y calendarios requiere el motor de correo nativo. El acceso a archivos sigue disponible mediante el vínculo de su cuenta.
nextcloud-pim-disconnect = Desconectar
nextcloud-pim-disconnect-help = Al desconectar se conservan los datos en caché y se detienen las escrituras remotas. Las cuentas vinculadas de nuevo requieren descubrimiento y conexión explícita otra vez.
nextcloud-pim-ready = Lista
nextcloud-pim-disconnected = Desconectada o vínculo cambiado
nextcloud-pim-unnamed = Colección sin nombre
nextcloud-pim-back = Volver a las cuentas de Nextcloud
nextcloud-pim-create-title = Nueva colección en Nextcloud
nextcloud-pim-create-help = Crea un calendario o una lista de tareas vacíos en esta cuenta de Nextcloud y los conecta aquí.
nextcloud-pim-create-name = Nombre de la nueva colección
nextcloud-pim-create = Crear en Nextcloud
nextcloud-pim-created = { $name } se creó en Nextcloud y se conectó. Sincronice ahora para cargarla.
nextcloud-pim-interval = Intervalo de actualización automática (segundos)
nextcloud-pim-interval-help = Cero usa solo la actualización manual. Los intervalos positivos tienen un mínimo de 60 segundos. Las cuentas conectadas se actualizan en segundo plano; los fallos aumentan la espera y las credenciales rechazadas detienen nuevas solicitudes hasta que vuelva a vincular.
nextcloud-pim-interval-reset = Usar el valor predeterminado del administrador
nextcloud-admin-empty = Aún no hay instancias aprobadas. Añada una instancia para que esté disponible para vincular cuentas.
nextcloud-admin-status = Estado
nextcloud-admin-open = Abrir { $name }
