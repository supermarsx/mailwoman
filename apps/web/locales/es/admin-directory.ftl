# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-directory. Source: ../en/admin-directory.ftl


# -- Machine translation (2026-11): added to match en ---------------------
directory-title = Configuración del directorio LDAP
directory-help = Configure los servidores de directorio, la asignación de atributos y las credenciales de servicio selladas. Los cambios surten efecto tras reiniciar el servidor. No se comprueba si el servidor es accesible.
directory-reload = Recargar la configuración guardada
directory-saved = Configuración guardada. Reinicie el servidor para aplicarla.
directory-restart = La configuración guardada difiere del directorio en ejecución. Es necesario reiniciar el servidor.
directory-proxy-blocked = Las conexiones LDAP están bloqueadas mientras haya un proxy saliente activo. Este transporte LDAP no admite túneles de proxy y no eludirá su proxy.
directory-env-pin = El entorno fija la credencial de servicio para este DN de enlace:
directory-url = URL del servidor
directory-base = DN base de búsqueda
directory-bind = DN de enlace del servicio
directory-tls = Seguridad del transporte
directory-plaintext = Texto sin cifrar (solo anónimo)
directory-priority = Prioridad
directory-connection = Política de conexión
directory-blocked = Bloquear conexiones
directory-direct = Permitir conexiones directas protegidas
directory-enabled = Incluir en las búsquedas del directorio
directory-private = Permitir direcciones de LAN privada y de bucle local
directory-consent = Las conexiones están bloqueadas. Seleccione explícitamente el acceso directo protegido para permitir este punto de conexión; las direcciones privadas requieren un consentimiento aparte.
directory-password-set = Hay una contraseña de servicio sellada almacenada.
directory-password-unset = No hay ninguna contraseña de servicio almacenada. Las búsquedas usan acceso anónimo, salvo que se aplique una credencial del entorno.
directory-secret-action = Acción sobre la contraseña almacenada
directory-keep = Mantener sin cambios
directory-replace = Establecer o reemplazar la contraseña
directory-clear = Borrar la contraseña almacenada
directory-password = Nueva contraseña de servicio
directory-attr-display-name = Atributo del nombre para mostrar
directory-attr-mail = Atributo del correo electrónico
directory-attr-member = Atributo de miembro del grupo
directory-attr-user-cert = Atributo del certificado
directory-attr-photo = Atributo de la foto
directory-remove = Quitar de la configuración
directory-add = Añadir directorio
directory-save = Guardar la configuración del directorio
