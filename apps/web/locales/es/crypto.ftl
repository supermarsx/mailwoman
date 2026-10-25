# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Cifrado de extremo a extremo
crypto-banner-e2ee-detail = Todos los destinatarios tienen clave — el cuerpo del mensaje se cifra en este dispositivo.
crypto-banner-tls-label = Cifrado de transporte (TLS)
crypto-banner-tls-detail = No se encontraron claves de cifrado de los destinatarios; la entrega solo está protegida en tránsito.
crypto-banner-mixed-label = Protección mixta
crypto-banner-mixed-detail = Algunos destinatarios pueden recibir cifrado de extremo a extremo; otros solo TLS.
crypto-banner-checking-label = Comprobando claves de los destinatarios
crypto-banner-checking = · comprobando…
crypto-banner-tls-only = Solo TLS para:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Seguridad del mensaje
crypto-encrypt-label = Cifrar (de extremo a extremo)
crypto-encrypt-hint-default = Cifrar el cuerpo en este dispositivo antes de enviarlo.
crypto-encrypt-drafted = Borrador cifrado en este dispositivo.
crypto-encrypt-no-key = No hay ninguna clave de cifrado del destinatario disponible.
crypto-sign-label = Firmar (verificar que es suyo)
crypto-protect-subject = Cifrar también el asunto
crypto-reason-add-recipient = Añada un destinatario para buscar claves de cifrado.
crypto-reason-tls = No hay ninguna clave de cifrado del destinatario disponible — se envía por TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Advertencias de prevención de pérdida de datos
crypto-dlp-block = Envío bloqueado
crypto-dlp-require = Cifrado obligatorio
crypto-dlp-warn = Atención
crypto-dlp-notify = Se notificará al administrador
crypto-dlp-matched = Coincidencias: { $list }
