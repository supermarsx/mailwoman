# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Gestión de claves
keys-title = Claves y certificados
keys-subtitle = Claves OpenPGP y S/MIME. Las claves privadas permanecen en este dispositivo y nunca llegan al servidor.
keys-generate = Generar clave
keys-import = Importar clave

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Sus claves
keys-empty-own = Aún no hay claves. Genere o importe una para empezar.
keys-contact-keys = Claves de contactos
keys-empty-contact = No hay claves de contactos. Busque una abajo.
keys-loading = Cargando claves…
keys-select-prompt = Seleccione una clave para verla y verificarla.
keys-untitled = Clave { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = no verificada
keys-trust-tofu = tofu
keys-trust-verified = verificada
keys-trust-revoked = revocada

# -- Key detail --------------------------------------------------------------
keys-key-card = Clave { $name }
keys-fingerprint = Huella digital
keys-safe-words = Palabras de seguridad
keys-safe-words-help = Léalas en voz alta con el contacto para confirmar la clave por otro canal.
keys-safe-words-list = Palabras de seguridad de la huella digital
keys-scan-label = Escanear para verificar
keys-qr-label = Código QR de la huella digital
keys-autocrypt = Autocrypt
keys-autocrypt-on = Anunciada en los encabezados Autocrypt
keys-autocrypt-off = No anunciada mediante Autocrypt
keys-trust-label = Confianza
keys-trust-level = Nivel de confianza
keys-associate-label = Asociar con un contacto
keys-associate-select = Contacto que asociar
keys-choose-contact = Elija un contacto…
keys-associate-button = Asociar
keys-backup-label = Copia de seguridad
keys-backup-help = Exporte un mensaje de configuración de Autocrypt para trasladar esta clave a otro dispositivo.
keys-export-backup = Exportar copia de seguridad
keys-asm-label = Mensaje de configuración de Autocrypt
keys-no-private-backup = No hay ninguna clave privada en este dispositivo para hacer una copia de seguridad

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Certificado S/MIME
keys-smime-self-signed = Este certificado es autofirmado: lo firmó su propia clave, no una autoridad de certificación. Los programas de correo de otras personas no confiarán en él a menos que lo acepten manualmente, o hasta que una autoridad de certificación emita un certificado a partir del archivo de solicitud de abajo.
keys-smime-valid-until = Válido hasta el { $date }.
keys-smime-not-in-app = Mailwoman aún no firma ni descifra correo con una clave S/MIME. Exporte la clave para usarla en otro programa de correo.
keys-smime-passphrase = Frase de contraseña de esta clave S/MIME
keys-smime-export-help = El archivo .p12 contiene el certificado y la clave privada. La clave se cifra con la frase de contraseña de arriba, y la misma frase abre el archivo en otro programa. El archivo .csr no contiene ninguna clave privada.
keys-smime-export = Exportar como PKCS#12 (.p12)
keys-smime-request = Descargar solicitud de certificado (.csr)
keys-smime-issued-label = Certificado emitido para esta clave (.pem, .crt, .cer)
keys-smime-issued-file = Archivo del certificado emitido
keys-smime-issued-help = Sustituye el certificado guardado para esta clave. Se rechaza un certificado creado para otra clave.
keys-smime-attach = Importar certificado emitido
keys-smime-failed = No funcionó. Compruebe la frase de contraseña.
keys-smime-attach-failed = No se importó el certificado. Compruebe la frase de contraseña y que el archivo contiene un solo certificado emitido para esta clave.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Buscar la clave de un contacto
keys-lookup-heading = Buscar una clave
keys-lookup-address = Dirección que buscar
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Fuentes de búsqueda
keys-source = Fuente { $source }
keys-consent-label = Consentimiento para la búsqueda externa
keys-consent-text = Buscar una clave consulta directorios externos (WKD/VKS). Tengo el consentimiento de esta persona para hacerlo.
keys-lookup-button = Buscar
keys-looking-up = Buscando…
keys-lookup-found = Se encontraron { $count } clave(s) — añadidas a Claves de contactos.
keys-lookup-none = No se encontró ninguna clave.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Generar una clave
keys-type = Tipo
keys-key-type = Tipo de clave
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-generate-smime-note = Crea una clave RSA de 3072 bits y un certificado para la dirección de abajo, válido durante dos años. El certificado es autofirmado: los programas de correo de otras personas no confiarán en él a menos que lo acepten manualmente, o hasta que una autoridad de certificación emita uno. Mailwoman aún no firma ni descifra correo con él; después puede exportarlo para otro programa de correo.
keys-name = Nombre
keys-email = Correo electrónico
keys-key-passphrase = Frase de contraseña de la clave
keys-passphrase-help = La frase de contraseña protege la clave privada en este dispositivo. Nunca sale del navegador.
keys-cancel = Cancelar
keys-generate-submit = Generar
keys-generating = Generando…
keys-generating-smime = Generando una clave RSA en este navegador. Puede tardar hasta un minuto.
keys-generate-failed = No se generó ninguna clave.
keys-generate-failed-smime = No se generó ninguna clave. Un certificado solo puede llevar una dirección ASCII simple (nombre@dominio) y un nombre de 64 caracteres como máximo.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Importar una clave
keys-import-type = Tipo de importación
keys-tab-armored = Blindada (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Clave blindada
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Frase de contraseña (si la clave es privada)
keys-import-passphrase = Frase de contraseña de importación
keys-pkcs12-file-label = Archivo PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Archivo PKCS#12
keys-import-password-label = Contraseña de importación
keys-pkcs12-password = Contraseña PKCS#12
keys-import-preview = Vista previa de la importación
keys-preview = Vista previa
keys-preview-type = Tipo: { $kind }
keys-preview-fingerprint-aria = Huella digital de la vista previa
keys-preview-fingerprint = Huella digital: { $fp }
keys-preview-has-private = Incluye una clave privada (se guardará en este dispositivo)
keys-preview-public-only = Solo clave pública
keys-import-submit = Importar
