# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-email = Dirección de correo
auth-password = Contraseña
auth-sign-in = Iniciar sesión
auth-signing-in = Iniciando sesión…
auth-invalid-credentials = Credenciales no válidas
auth-refused-note = Si la dirección de correo y la contraseña son correctas, es posible que un administrador haya desactivado la cuenta. Una cuenta desactivada no puede iniciar sesión hasta que un administrador la vuelva a activar.
auth-unreachable = No se pudo conectar con el servidor
auth-address-not-configured = Esta dirección no está configurada en este servidor. Póngase en contacto con su administrador.

# -- Single sign-on (t9) -----------------------------------------------------
auth-sso-heading = Inicio de sesión único
auth-sso-divider = o continuar con
auth-sso-button = Iniciar sesión con { $name }
auth-sso-error = El inicio de sesión único no se completó. Inténtelo de nuevo o inicie sesión con su contraseña.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Autorizar aplicación
auth-consent-title = Autorizar acceso
auth-consent-loading = Cargando solicitud…
auth-consent-intro = quiere acceder a su cuenta.
auth-consent-approved = Cliente aprobado por el administrador
auth-consent-unapproved = Cliente no reconocido — no aprobado por el administrador
auth-consent-requesting = Solicita
auth-consent-redirects-to = Redirige a
auth-consent-for-resource = Para el recurso
auth-consent-deny = Denegar
auth-consent-allow = Permitir
auth-consent-error = no se pudo registrar su decisión

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Demasiados intentos de inicio de sesión. Espere antes de volver a intentarlo.
auth-retry-countdown = Vuelva a intentarlo en { $seconds } s
auth-upstream-protocol = Inicio de sesión interrumpido: el servidor de correo envió una respuesta que este servidor no pudo leer. Inténtelo más tarde o avise a su administrador.
auth-account-disabled = Un administrador ha desactivado esta cuenta. No podrá iniciar sesión hasta que un administrador la vuelva a activar.
auth-signin-not-allowed = Esta dirección no puede iniciar sesión en este servidor. Póngase en contacto con su administrador.
auth-signin-not-provisioned = Todavía no existe una cuenta para esta dirección y las cuentas nuevas no se crean automáticamente. Pida a su administrador que le dé de alta.
auth-reference = Referencia: { $id }
auth-domain-hint = Para { $domain } puede iniciar sesión solo con su nombre de usuario.
auth-passkey-sign-in = Iniciar sesión con una llave de acceso
auth-passkey-failed = La llave de acceso no permitió iniciar sesión. Inténtelo de nuevo o use su contraseña.
auth-remembered = Inicios de sesión anteriores
auth-remembered-help = Direcciones usadas correctamente en este dispositivo. Al elegir una solo se rellena la dirección.
auth-forget-address = Olvidar { $address } en este dispositivo
auth-login-timeout = Se agotó el tiempo del inicio de sesión. No se hizo ningún reintento automático. Compruebe la conexión e inténtelo de nuevo cuando quiera.
auth-verification-title = Verificación de inicio de sesión
auth-verification-loading = Cargando la verificación de inicio de sesión…
auth-verification-error = No se pudo completar la verificación. Repita la verificación antes de iniciar sesión.
auth-verification-required = Complete una nueva verificación de inicio de sesión antes de volver a intentarlo.
auth-identity-title = Compartir su identidad
auth-identity-sign-in = Inicie sesión de nuevo para continuar con esta solicitud de identidad.
auth-identity-sign-out = Cerrar sesión en Mailwoman
auth-identity-expired = Esta solicitud de identidad ha caducado o ha cambiado. Vuelva a la aplicación solicitante y empiece de nuevo.
auth-identity-retry = Reintentar
auth-identity-disclosure = Esta aplicación recibe la información de identidad que se muestra abajo. Este permiso no concede acceso a su correo.
auth-identity-scope-openid = Un identificador de identidad estable
auth-identity-scope-profile = La información de perfil disponible
auth-identity-scope-email = Su dirección de correo con la que ha iniciado sesión, sin garantía de que se haya verificado de forma independiente
auth-identity-scope-offline_access = Renovar el acceso a la identidad mientras esta sesión de Mailwoman siga siendo válida, hasta 15 días desde el inicio de sesión
auth-identity-deny = Cancelar
auth-identity-approve = Permitir el acceso a la identidad
auth-identity-grants = Aplicaciones que usan su identidad
auth-identity-grants-help = Revoque el acceso a la identidad de una aplicación. Esto no elimina los permisos independientes de la API de correo.
auth-identity-no-grants = Ninguna aplicación tiene actualmente consentimiento de identidad.
auth-identity-revoke = Revocar el acceso a la identidad
auth-maintenance = El correo web está en pausa temporalmente por mantenimiento. Inténtelo de nuevo más tarde.
