# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Detalles de seguridad del mensaje
security-section-auth = Autenticación
security-section-received = Ruta de entrega
security-section-signature = Firma
security-section-attachments = Adjuntos
security-section-warnings = Advertencias
security-section-controls = Controles del remitente
security-received-empty = No hay cadena Received disponible.
security-attachments-empty = No hay adjuntos.
security-hop-unknown = desconocido
security-unknown = desconocido

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = superado
security-auth-fail = fallido
security-auth-none = ausente
security-auth-neutral = neutro
security-auth-temperror = error temporal
security-auth-permerror = error permanente

security-detail-domain = dominio
security-detail-selector = selector
security-detail-policy = política
security-detail-alignment = alineación
security-detail-chain-length = longitud de la cadena
security-aligned = alineado
security-not-aligned = no alineado

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Firma verificada
security-sig-unverified-key = Firmado — clave del firmante no verificada
security-sig-invalid = La firma no es válida
security-sig-none = Sin firmar
security-chain-trusted = Cadena de confianza
security-chain-untrusted = Cadena no fiable
security-chain-expired = Cadena caducada
security-chain-unknown = Cadena desconocida
security-revocation-good = No revocada
security-revocation-revoked = Clave revocada
security-revocation-unknown = Revocación desconocida
security-key-changed = La clave del firmante ha cambiado desde la última vez
security-fact-signer-key = Clave del firmante
security-fact-algorithm = Algoritmo
security-fact-key-created = Clave creada
security-fact-key-expires = La clave caduca
security-fact-chain = Cadena
security-fact-revocation = Revocación
security-fact-key-change = Cambio de clave

# -- Attachment risk --------------------------------------------------------
security-attach-none = Sin riesgo conocido
security-attach-macro = Contiene macros
security-attach-executable = Archivo ejecutable
security-attach-encrypted-archive = Archivo comprimido cifrado
security-attach-double-extension = Doble extensión de archivo
security-attach-mismatch = tipo no coincidente (declarado { $declared }, detectado { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = La dirección Reply-To es distinta del remitente
security-anomaly-envelopeFromDivergence = El remitente del sobre es distinto de la dirección From
security-anomaly-messageIdDomainAnomaly = El dominio de Message-ID no coincide con el remitente
security-anomaly-dateSkew = La fecha de envío parece desfasada
security-anomaly-punycodeSender = El remitente usa punycode (posible dominio de imitación)

# -- Sender controls --------------------------------------------------------
security-control-block = Bloquear remitente
security-control-silence = Silenciar remitente
security-control-ignore-conversation = Ignorar conversación
security-control-report-phishing = Denunciar phishing
security-control-report-junk = Denunciar correo no deseado
security-control-done-block = Remitente bloqueado
security-control-done-silence = Remitente silenciado
security-control-done-ignore-conversation = Conversación ignorada
security-control-done-report-phishing = Denunciado como phishing
security-control-done-report-junk = Denunciado como correo no deseado

security-za-title = Almacenamiento de acceso cero
security-za-on = Activado
security-za-off = Desactivado
security-za-protects = El acceso cero cifra su correo, sus notas y sus datos PIM almacenados, de modo que el servidor de alojamiento solo guarda texto cifrado que no puede leer. Protege el contenido de sus datos almacenados frente a un operador curioso o una brecha en el host de almacenamiento: quien lea la base de datos, los archivos en disco o una copia de seguridad robada ve texto cifrado XChaCha20-Poly1305 y no puede recuperar cuerpos de mensajes, asuntos, adjuntos, texto de notas ni el índice de búsqueda.

security-za-server-sees-title = Lo que el servidor sigue viendo
security-za-sees-1 = Bloques de texto cifrado — las propias filas cifradas.
security-za-sees-2 = ID de fila opacos — los identificadores usados para guardar y obtener filas.
security-za-sees-3 = Tamaños — la longitud de cada texto cifrado (tamaño aproximado del mensaje o adjunto).
security-za-sees-4 = Marcas de tiempo — cuándo se escriben y actualizan las filas.
security-za-sees-5 = Metadatos de enrutamiento del sobre necesarios para hacer de proxy de IMAP/SMTP — el servidor sigue conectándose a su proveedor de correo en su nombre, por lo que los metadatos de conexión y el sobre de enrutamiento necesarios para enviar y recibir correo pasan por él.

security-za-active-server-caveat = El acceso cero protege los datos EN REPOSO frente a un host curioso o comprometido. Un servidor activo totalmente malicioso que hace de proxy de su tráfico IMAP/SMTP en vivo es un adversario más fuerte, y el acceso cero NO protege contra él: ese servidor está en la conexión en vivo con su proveedor de correo y podría observar o manipular el correo mientras pasa, sin importar cómo esté cifrada la copia almacenada. Elija el acceso cero cuando su preocupación sea «no quiero que quien gestione (o vulnere) el host de almacenamiento lea mi correo almacenado». No es una defensa contra un operador que subvierta la ruta del correo en vivo.

security-za-no-search-claim = Aquí no hay cifrado con búsqueda en el servidor, y no se afirma tal cosa. La búsqueda se ejecuta íntegramente en el cliente sobre el contenido que ha descifrado localmente; el servidor nunca guarda una forma buscable de su texto sin cifrar.

security-za-recovery-tradeoff = Sus claves se derivan y se guardan solo en sus dispositivos — el servidor nunca recibe una clave sin cifrar. Si pierde su frase de contraseña (o llave de acceso) Y todos los dispositivos emparejados Y su frase de recuperación, los datos cifrados no se pueden recuperar. Guarde la frase de recuperación sin conexión y trátela como equivalente al secreto maestro de su cuenta.

security-za-setup-key = Configurar su clave
security-za-key-source = Origen de la clave
security-za-passphrase = Frase de contraseña
security-za-passkey = Llave de acceso (sin contraseña)
security-za-passkey-unavailable = las llaves de acceso no están disponibles en este navegador
security-za-passphrase-label = Frase de contraseña de acceso cero
security-za-enable = Activar acceso cero
security-za-recovery-title = Frase de recuperación
security-za-recovery-heading = Frase de recuperación — guárdela ahora sin conexión
security-za-recovery-note = Esta es la única copia. Cualquiera que la tenga puede leer sus datos; sin ella (y sin un dispositivo emparejado) sus datos no se pueden recuperar.
security-za-disable-section = Desactivar acceso cero
security-za-disable-heading = Desactivar el acceso cero
security-za-disable-note = Los datos nuevos se volverán a guardar sin cifrar. Los datos cifrados existentes solo seguirán siendo legibles mientras pueda derivar su clave.
security-za-disable-btn = Desactivar acceso cero
security-za-err-passphrase-len = use una frase de contraseña de al menos 8 caracteres
security-za-err-enable = no se pudo activar el acceso cero
security-za-err-disable = no se pudo desactivar el acceso cero

security-pair-title = Emparejamiento de dispositivos
security-pair-heading = Emparejar un dispositivo
security-pair-intro = Traslade sus claves a otro dispositivo sin enviarlas a través del servidor — este solo retransmite un sobre sellado opaco. Compare las seis palabras en ambas pantallas antes de confiar en el emparejamiento.
security-pair-role = Función en el emparejamiento
security-pair-new-role = Este es el dispositivo nuevo
security-pair-existing-role = Emparejar otro dispositivo desde aquí
security-pair-show-qr = Mostrar QR de emparejamiento
security-pair-scan = Escanee esto en su dispositivo existente
security-pair-qr-label = Código QR de emparejamiento de dispositivo
security-pair-qr-default = Código QR de emparejamiento
security-pair-copy-code = O copie este código de emparejamiento
security-pair-paste-envelope = Pegue el sobre sellado del otro dispositivo
security-pair-envelope-placeholder = sobre…
security-pair-envelope-label = Sobre sellado
security-pair-complete = Completar emparejamiento
security-pair-code-from-new = Código de emparejamiento del dispositivo nuevo
security-pair-code-placeholder = código de emparejamiento…
security-pair-code-label = Código de emparejamiento
security-pair-seal = Sellar mis claves para ese dispositivo
security-pair-envelope-out = Sobre sellado — péguelo en el dispositivo nuevo
security-pair-compare = Compare estas palabras en ambos dispositivos
security-pair-sas-label = Cadena corta de autenticación
security-pair-match = Las palabras coinciden
security-pair-abort = Son distintas — cancelar
security-pair-confirmed = Emparejamiento confirmado. Este canal está autenticado.
security-pair-err-start = no se pudo iniciar el emparejamiento
security-pair-err-complete = no se pudo completar el emparejamiento
security-pair-err-unlock = desbloquee primero el acceso cero para emparejar otro dispositivo
security-pair-err-seal = no se pudo sellar la clave raíz
