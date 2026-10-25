# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Cambiar contraseña
passwd-heading = Cambiar contraseña

# -- Forced change -----------------------------------------------------------
passwd-force-change = Su administrador le exige cambiar la contraseña antes de continuar.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Contraseña actual
passwd-new-label = Nueva contraseña
passwd-confirm-label = Confirmar nueva contraseña

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Las nuevas contraseñas coinciden.
passwd-match-no = Las nuevas contraseñas aún no coinciden.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = al menos { $count } caracteres
passwd-rule-uppercase = una letra mayúscula
passwd-rule-lowercase = una letra minúscula
passwd-rule-digit = un dígito
passwd-rule-symbol = un símbolo

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Esta cuenta es de acceso cero. Antes de aplicar el cambio se le mostrará una frase de recuperación — guárdela para poder seguir accediendo a sus datos si algo sale mal.

# -- Actions -----------------------------------------------------------------
passwd-continue = Continuar
passwd-submit = Cambiar contraseña

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Guarde su frase de recuperación
passwd-recovery-prose = Anote esta frase y guárdela en un lugar seguro. Se muestra antes del cambio de contraseña para que pueda recuperar sus datos aunque pierda la nueva contraseña. No se guarda en el servidor.
passwd-recovery-ack-label = He guardado mi frase de recuperación
passwd-recovery-ack-text = He guardado mi frase de recuperación en un lugar seguro.

# -- Done phase --------------------------------------------------------------
passwd-done = Su contraseña se ha cambiado.
passwd-done-reencrypt = Sus credenciales de servidor guardadas se han vuelto a cifrar con la nueva contraseña.
passwd-done-rewrap = Sus claves de acceso cero se han vuelto a envolver con la nueva contraseña.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = introduzca su contraseña actual
passwd-error-mismatch = la nueva contraseña y su confirmación no coinciden
passwd-error-policy = la nueva contraseña necesita { $rules }
passwd-error-ack-first = confirme primero que ha guardado la frase de recuperación
passwd-error-prepare-recovery = no se pudo preparar la frase de recuperación
passwd-error-change = no se pudo cambiar la contraseña
passwd-error-session-ended = Su sesión ha terminado. Vuelva a iniciar sesión.
passwd-error-not-configured = Este servidor no está configurado para cambiar contraseñas. Póngase en contacto con su administrador.
passwd-error-contact-admin = El servidor no pudo completar el cambio. Póngase en contacto con su administrador.

# -- Forced change screen ----------------------------------------------------
passwd-forced-title = Cambie su contraseña para continuar
passwd-forced-signed-in-as = Sesión iniciada como { $username }.
passwd-forced-explain = Un administrador ha exigido un cambio de contraseña para esta cuenta. El correo, el calendario y los contactos no están disponibles hasta que el cambio se complete.
passwd-forced-backend-note = Si el cambio se rechaza aunque su contraseña actual sea correcta, es posible que este servidor no pueda cambiar su contraseña. Póngase en contacto con su administrador: puede configurar los cambios de contraseña o quitar el requisito.
passwd-forced-sign-out = Cerrar sesión
passwd-forced-still-required = El servidor aceptó el cambio, pero sigue exigiendo un cambio de contraseña para esta cuenta. Póngase en contacto con su administrador.
passwd-forced-reload-failed = La contraseña se cambió, pero no se pudo cargar su cuenta. Compruebe su conexión e inténtelo de nuevo.
passwd-forced-retry = Reintentar
