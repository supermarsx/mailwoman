# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Proveedor de identidad
admin-identity-description = Permita que las aplicaciones aprobadas inicien sesión con Mailwoman. Es independiente de los proveedores de inicio de sesión externos y del acceso a la API de correo. Se requiere el código de autorización con PKCE; las demás concesiones no están disponibles.
admin-identity-configure = Configurar el emisor
admin-identity-new = Añadir cliente de identidad
admin-identity-edit = Editar cliente
admin-identity-enabled = Activado
admin-identity-disabled = Desactivado
admin-identity-all = Todos
admin-identity-discovery = Abrir el documento de descubrimiento
admin-identity-search = Buscar clientes
admin-identity-status = Estado
admin-identity-clients = Clientes de identidad
admin-identity-client = Identidad del cliente
admin-identity-client-id = ID de cliente
admin-identity-name = Nombre
admin-identity-method = Autenticación del cliente
admin-identity-public = Cliente público (PKCE, sin secreto)
admin-identity-pkce = Todos los clientes deben usar PKCE S256. Los secretos de los clientes confidenciales se generan aquí y se muestran una sola vez.
admin-identity-rotate-secret = Reemplazar el secreto de cliente existente
admin-identity-redirects = Destinos registrados
admin-identity-redirect-help = Un URI HTTPS exacto por línea; se permite HTTP de bucle local numérico para aplicaciones locales. Se rechazan los comodines y los fragmentos.
admin-identity-callbacks = URI de retorno de autorización
admin-identity-logout-uris = URI de redirección tras cerrar sesión
admin-identity-scopes = Ámbitos de identidad permitidos
admin-identity-actions = Acciones
admin-identity-delete = Eliminar cliente
admin-identity-delete-confirm = ¿Eliminar el acceso de identidad de { $name }? Las concesiones de identidad existentes dejan de funcionar y se elimina el registro del cliente solo de identidad.
admin-identity-empty = No hay clientes de identidad que coincidan.
admin-identity-cancel = Cancelar
admin-identity-save = Guardar
admin-identity-done = Hecho
admin-identity-secret = Secreto de cliente generado
admin-identity-secret-once = Copie ahora este secreto en la aplicación dependiente. No se puede volver a mostrar. No coloque nunca un secreto confidencial en código del navegador.
admin-identity-issuer = URL HTTPS pública del emisor
admin-identity-issuer-help = Use el origen HTTPS exacto de este despliegue y la ruta /oidc, incluido cualquier prefijo del despliegue. Cambiar el emisor o la configuración revoca las concesiones de identidad existentes.
admin-identity-rotate-key = Rotar la clave de firma
admin-identity-key-help = La rotación requiere una verificación reciente del administrador. Las claves públicas anteriores siguen disponibles durante 15 minutos para que los tokens emitidos hace poco puedan completar su corta vida útil.
