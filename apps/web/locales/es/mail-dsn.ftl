# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Informe de estado de entrega
dsn-unverified = Este informe describe el intento de entrega del remitente. Sus afirmaciones no se verifican de forma independiente. Que se entregue no significa que el mensaje se haya leído.
dsn-reporting-mta = Servidor informante:
dsn-original-recipient = Destinatario original:
dsn-remote-mta = Servidor remoto:
dsn-last-attempt = Último intento:
dsn-retry-until = El remitente planea reintentar hasta:
dsn-action-failed = Fallo notificado
dsn-action-delayed = Retraso notificado
dsn-action-delivered = Entrega notificada
dsn-action-relayed = Reenvío notificado
dsn-action-expanded = Expansión de lista notificada

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Utilice el diagnóstico del servidor cuando solicite ayuda a su proveedor de correo electrónico.

dsn-advice-1 = Verifique las direcciones del remitente y del destinatario. Confirme la dirección actual del destinatario si es necesario.

dsn-advice-2 = Comuníquese con el destinatario de otra manera si el problema del buzón persiste.

dsn-advice-3 = Comparta este informe con el administrador del sistema de correo receptor si el problema persiste.

dsn-advice-4 = Pídele a tu administrador de correo que verifique la conectividad, DNS, y las rutas de reenvío si el problema persiste.

dsn-advice-5 = Comparta el diagnóstico con su proveedor de correo electrónico para que pueda verificar la compatibilidad y los límites de SMTP.

dsn-advice-6 = Verifique el formato del mensaje, las direcciones internacionales y los archivos adjuntos con los requisitos del destinatario.

dsn-advice-7 = Comparta el diagnóstico con su administrador de correo para identificar la regla exacta de seguridad o autenticación.

dsn-advice-success = Verifique la acción de entrega reportada arriba para saber si el mensaje fue entregado, retransmitido o ampliado a una lista.

dsn-category-0 = El servidor proporcionó un código de detalle no reconocido o específico del proveedor. Su diagnóstico a continuación puede explicar la causa exacta.

dsn-category-1 = Este es un código relacionado con la dirección. Los detalles precisos son específicos del proveedor o no se reconocen.

dsn-category-2 = Este es un código de buzón de destinatario. Los detalles precisos son específicos del proveedor o no se reconocen.

dsn-category-3 = Este es un código del sistema de recepción de correo. Los detalles precisos son específicos del proveedor o no se reconocen.

dsn-category-4 = Este es un código de red o de enrutamiento. Los detalles precisos son específicos del proveedor o no se reconocen.

dsn-category-5 = Este es un código de protocolo de correo. Los detalles precisos son específicos del proveedor o no se reconocen.

dsn-category-6 = Este es un código de contenido o formato de mensaje. Los detalles precisos son específicos del proveedor o no se reconocen.

dsn-category-7 = Este es un código de seguridad o política. Los detalles precisos son específicos del proveedor o no se reconocen; no establece por sí solo que el mensaje sea spam.

dsn-class-2 = Éxito: el servidor informa un paso de entrega positivo. Esto no prueba que el mensaje haya sido leído.

dsn-class-4 = Fallo temporal: un intento posterior puede tener éxito. El remitente aún puede volver a intentar un informe de retraso; un informe de fracaso significa que el intento ha finalizado.

dsn-class-5 = Fallo permanente: es poco probable que funcione reenviar el mismo mensaje sin cambios. Corrija la causa antes de volver a enviar.

dsn-meaning-title = ¿Qué significa este código?

dsn-next-step = que hacer

dsn-provider-barracuda = barracuda

dsn-provider-detected = Interpretación basada en el diagnóstico del servidor o nombre de host:

dsn-provider-exchange = Microsoft Exchange/Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmail/espacio de trabajo de Google

dsn-provider-help = Documentación del proveedor

dsn-provider-mimecast = mimecast

dsn-provider-proofpoint = Punto de prueba

dsn-provider-relay = SMTP relé

dsn-provider-yahoo = Yahoo/AOL

dsn-server-diagnostic = Diagnóstico del servidor

dsn-status-0-0 = El servidor no dio una razón más específica.

dsn-status-1-0 = Hay un problema con una dirección de correo electrónico.

dsn-status-1-1 = El buzón del destinatario no existe. Confirme la dirección con el destinatario.

dsn-status-1-10 = El dominio del destinatario declara explícitamente que no acepta ningún correo electrónico (nulo MX). Utilice otra dirección.

dsn-status-1-2 = El dominio del destinatario no puede recibir este correo. Verifique la parte después de @.

dsn-status-1-3 = La dirección del destinatario tiene un formato no válido. Compruebe si hay errores tipográficos o espacios adicionales.

dsn-status-1-4 = El servidor no puede identificar un destinatario único para esta dirección.

dsn-status-1-5 = La dirección del destinatario fue aceptada como válida.

dsn-status-1-6 = Este buzón se ha movido y no hay ninguna dirección de reenvío disponible. Pregunta por la nueva dirección.

dsn-status-1-7 = La dirección del remitente tiene un formato no válido. Verifique la dirección de envío de la cuenta.

dsn-status-1-8 = El dominio del remitente no es válido o no puede recibir correo de respuesta.

dsn-status-1-9 = El mensaje se entregó a un sistema que no puede proporcionar más informes de entrega.

dsn-status-2-0 = El buzón del destinatario tiene un problema no especificado.

dsn-status-2-1 = El buzón del destinatario está deshabilitado o no acepta mensajes. Póngase en contacto con el destinatario de otra manera.

dsn-status-2-2 = El buzón del destinatario está lleno. El destinatario necesita liberar espacio o aumentar su cuota.

dsn-status-2-3 = El mensaje excede el límite de tamaño de este destinatario. Reduzca los archivos adjuntos o utilice un enlace para compartir.

dsn-status-2-4 = El servidor no pudo resolver los miembros de una lista de correo. Póngase en contacto con el propietario de la lista.

dsn-status-3-0 = El sistema de correo receptor informó un problema no especificado.

dsn-status-3-1 = El servidor de correo receptor se ha quedado sin almacenamiento. Su administrador necesita liberar espacio.

dsn-status-3-2 = El servidor receptor no acepta correo, por ejemplo durante mantenimiento o sobrecarga.

dsn-status-3-3 = El sistema receptor no admite una función requerida por este mensaje.

dsn-status-3-4 = El mensaje excede el límite de tamaño del servidor receptor. Reduzca los archivos adjuntos o utilice un enlace para compartir.

dsn-status-3-5 = Un problema de configuración del servidor de correo impide la entrega. Pídale al administrador de correo que investigue.

dsn-status-3-6 = El servidor aceptó el mensaje pero cambió la prioridad de entrega solicitada.

dsn-status-4-0 = Se informó de un problema de red o de enrutamiento de correo sin más detalles.

dsn-status-4-1 = El servidor receptor no respondió al intento de conexión.

dsn-status-4-2 = La conexión se estableció pero se interrumpió antes de que finalizara la transferencia de correo.

dsn-status-4-3 = Falló una DNS o una búsqueda de directorio. Es posible que un administrador de correo necesite verificar la resolución de nombres.

dsn-status-4-4 = El servidor no pudo encontrar una ruta al sistema de correo del destinatario.

dsn-status-4-5 = La congestión del tráfico postal impidió la entrega.

dsn-status-4-6 = Se detectó un bucle de reenvío. Pídale al administrador que verifique las reglas de reenvío y enrutamiento.

dsn-status-4-7 = El plazo de entrega expiró. Verifique el diagnóstico del problema subyacente antes de reenviarlo.

dsn-status-5-0 = Los servidores de correo no pudieron completar el intercambio de protocolos de entrega.

dsn-status-5-1 = Un servidor de correo rechazó un comando no admitido o un comando enviado en el orden incorrecto.

dsn-status-5-2 = Un servidor de correo no pudo entender la sintaxis de un comando.

dsn-status-5-3 = El mensaje tiene demasiados destinatarios. Enviar a grupos más pequeños.

dsn-status-5-4 = Un servidor de correo rechazó los parámetros de un comando.

dsn-status-5-5 = Los servidores utilizan versiones incompatibles del protocolo de correo.

dsn-status-5-6 = La respuesta de autenticación fue demasiado larga para el servidor. Pídale al administrador de correo que verifique la configuración de autenticación.

dsn-status-6-0 = El servidor informó un problema no especificado con el contenido o formato del mensaje.

dsn-status-6-1 = El sistema receptor no admite el tipo de contenido de este mensaje.

dsn-status-6-10 = Este código obsoleto significa que se requiere una respuesta internacional (UTF-8) pero no es compatible, como ocurre con el código x.6.8.

dsn-status-6-2 = La entrega requiere una conversión de contenido que la política del remitente o del servidor prohíbe.

dsn-status-6-3 = La entrega requiere una conversión de contenido que el servidor no puede realizar.

dsn-status-6-4 = Una conversión de contenido perdió parte de la información. Verifique el mensaje original y el diagnóstico del servidor.

dsn-status-6-5 = El servidor falló al convertir el contenido del mensaje.

dsn-status-6-6 = El servidor no pudo recuperar el contenido del mensaje de otro sistema.

dsn-status-6-7 = No se admite una dirección de correo electrónico internacional para este remitente o destinatario. Solicite una dirección alternativa.

dsn-status-6-8 = El servidor necesita una respuesta internacional (UTF-8) que el sistema de envío no puede aceptar.

dsn-status-6-9 = El mensaje contiene encabezados internacionales (UTF-8) que no se pueden entregar al menos a un destinatario.

dsn-status-7-0 = Una regla de seguridad o política de correo impidió la entrega normal. Consulte el diagnóstico del servidor para conocer la regla exacta.

dsn-status-7-1 = El servidor negó el permiso para realizar la entrega, posiblemente debido a restricciones de filtrado o retransmisión. Pídale al administrador de correo que verifique el diagnóstico.

dsn-status-7-10 = El servidor requiere una conexión cifrada o una protección más sólida antes de la autenticación.

dsn-status-7-11 = Este método de autenticación requiere una conexión cifrada SMTP (TLS).

dsn-status-7-12 = La cuenta necesita un método de autenticación o migración de contraseña. Póngase en contacto con el proveedor de la cuenta.

dsn-status-7-13 = La cuenta de envío está deshabilitada. Póngase en contacto con su administrador para restaurar el acceso.

dsn-status-7-14 = El servidor necesita una relación de confianza configurada para recuperar el mensaje. Póngase en contacto con su administrador.

dsn-status-7-15 = La prioridad del mensaje está por debajo del mínimo actual del servidor receptor.

dsn-status-7-16 = El mensaje es demasiado grande para la prioridad solicitada. Reducir su tamaño o preguntar al administrador.

dsn-status-7-17 = El buzón del destinatario ha cambiado de propietario desde la fecha requerida. Confirme el destinatario antes de volver a enviar.

dsn-status-7-18 = El dominio del destinatario ha cambiado de propietario desde la fecha requerida. Confirme el destino antes de volver a enviar.

dsn-status-7-19 = El servidor no pudo verificar la propiedad continua de la dirección del destinatario. Pregúntele al administrador sobre la verificación de validez del destinatario.

dsn-status-7-2 = El remitente no puede publicar en esta lista de correo. Póngase en contacto con el propietario de la lista.

dsn-status-7-20 = Ninguna firma DKIM pasó la verificación. Pídale al administrador de correo del remitente que verifique la firma y DNS.

dsn-status-7-21 = Se aprobó una firma DKIM, pero la política del servidor receptor no la aceptó.

dsn-status-7-22 = Ninguna firma DKIM aceptada coincide con la dirección De. Pídale al administrador de correo que verifique la alineación del dominio de firma.

dsn-status-7-23 = SPF falló: el servidor de envío no estaba autorizado por la política SPF del dominio del remitente.

dsn-status-7-24 = El servidor encontró un error al verificar SPF. Pídale al administrador de correo que verifique los registros SPF y DNS.

dsn-status-7-25 = El servidor emisor falló la verificación inversa DNS del receptor. Su administrador necesita verificar los registros de la dirección IP DNS.

dsn-status-7-26 = Fallaron varias comprobaciones de autenticación de mensajes. El diagnóstico puede identificar problemas de políticas SPF, DKIM o DMARC.

dsn-status-7-27 = El dominio del remitente declara que no acepta ningún correo electrónico de respuesta (nulo MX), lo cual el receptor rechaza.

dsn-status-7-28 = El sistema receptor detectó una posible avalancha de mensajes abusivos.

dsn-status-7-29 = La verificación ARC falló, por lo que el receptor no pudo aceptar la cadena de autenticación grabada del mensaje.

dsn-status-7-3 = La entrega requiere una conversión de formato de seguridad que el servidor no puede realizar.

dsn-status-7-30 = El mensaje requiere REQUIRETLS, pero el siguiente servidor de correo no lo admite. Pídale al administrador que verifique el soporte de entrega segura.

dsn-status-7-4 = El sistema de entrega no admite una característica de seguridad requerida.

dsn-status-7-5 = El servidor no pudo validar ni descifrar el mensaje, por ejemplo porque faltaba una clave o no era válida.

dsn-status-7-6 = El servidor no admite ningún algoritmo criptográfico requerido por el mensaje.

dsn-status-7-7 = Una verificación de integridad indica que el mensaje fue modificado o dañado.

dsn-status-7-8 = El servidor de envío rechazó las credenciales de inicio de sesión. Verifique la configuración de autenticación de la cuenta.

dsn-status-7-9 = El método de autenticación es más débil de lo que permite el servidor. Se requiere un método más fuerte.

dsn-vendor-advice-account = Pídale al administrador de envío que investigue un posible abuso de la cuenta, proteja la cuenta y solicite la eliminación de la restricción.

dsn-vendor-advice-admin = Comparta este diagnóstico con el administrador de correo de la organización para revisar la configuración o política coincidente.

dsn-vendor-advice-auth = Verifique el método de autenticación y autorización SMTP admitidos por la cuenta. Complete cualquier inicio de sesión requerido a través de la página de cuenta oficial del proveedor.

dsn-vendor-advice-authentication = Pídale al administrador del remitente que verifique SPF, DKIM, DMARC y la alineación del dominio utilizando el diagnóstico exacto.

dsn-vendor-advice-connector = Pídale al administrador que verifique la autorización de retransmisión, la coincidencia del conector IP/certificado, los dominios aceptados y la ruta de correo real.

dsn-vendor-advice-content = Revise los registros de diagnóstico y puerta de enlace con el administrador del destinatario para identificar el contenido bloqueado y la política aplicable.

dsn-vendor-advice-dns = Pídale al administrador de correo emisor o al proveedor IP que corrija DNS, invierta DNS y la identidad del servidor SMTP.

dsn-vendor-advice-domain = Solicite al administrador de Microsoft 365 que registre y verifique el dominio de envío y revise los límites de envío del inquilino.

dsn-vendor-advice-format = Pídale a la aplicación de envío o al administrador de correo que corrija la generación del mensaje y luego cree un mensaje nuevo.

dsn-vendor-advice-rate = Reduzca el volumen de envío y espere a que se borre la cuota o restricción. Verifique la acción informada antes de asumir que este intento todavía está en cola.

dsn-vendor-advice-recipient-admin = Comuníquese con el destinatario o propietario del grupo de otra manera y solicite a su administrador que revise el permiso o la política correspondiente.

dsn-vendor-advice-reputation = Pídale al administrador de envío que investigue el tráfico no deseado y la reputación, luego siga la revisión oficial del proveedor o el proceso de eliminación de la lista.

dsn-vendor-advice-route = Pídale al administrador de correo que verifique el destino, las reglas de reenvío, el servicio de correo y la ruta de red/firewall.

dsn-vendor-advice-tls = Solicite a los administradores de correo que reparen TLS, certificados o registros de seguridad DNS para que la ruta segura requerida pueda tener éxito.

dsn-vendor-barracuda-reputation = El servicio de reputación de Barracuda enumera o bloquea la dirección de envío IP.

dsn-vendor-certificate-chain = El certificado remoto TLS no se pudo validar con la confianza requerida o la política MTA-STS.

dsn-vendor-certificate-expired = El servidor de correo de destino presentó un certificado TLS caducado.

dsn-vendor-certificate-host = El certificado de destino TLS no coincide con el nombre de host del servidor de correo requerido.

dsn-vendor-connection-refused = El servidor de destino rechazó la conexión de red. Es necesario comprobar su servicio de correo, firewall o ruta de red.

dsn-vendor-content-security = La puerta de enlace rechazó el contenido del mensaje o un archivo adjunto según sus reglas de filtrado de seguridad. Consulte su diagnóstico o registros para el elemento específico.

dsn-vendor-credentials = El servicio SMTP rechazó las credenciales de inicio de sesión proporcionadas.

dsn-vendor-dane = El certificado TLS o la configuración TLSA del destino no cumplieron con la validación DANE.

dsn-vendor-dkim = El receptor no aceptó la configuración de autenticación o firma DKIM del mensaje.

dsn-vendor-dmarc = La política DMARC del receptor no se cumplió. Verifique la autenticación SPF o DKIM y la alineación con el dominio De visible.

dsn-vendor-dnssec = No se pudieron validar los registros DNSSEC del dominio de destino.

dsn-vendor-exchange-auth = La aplicación o el dispositivo no se autenticó correctamente para el envío de Microsoft 365 SMTP.

dsn-vendor-exchange-connector = Microsoft no pudo autorizar la retransmisión a través del conector de inquilino configurado. Es posible que su certificado, envío IP o dominio aceptado ya no coincidan.

dsn-vendor-exchange-domain-blocked = Microsoft restringió este dominio de envío después de detectar actividad sospechosa de spam.

dsn-vendor-exchange-forward-relay = Un sistema descendente rechazó el correo reenviado o retransmitido. Es posible que la autenticación se haya visto afectada por el reenvío o una puerta de enlace que no sea de Microsoft.

dsn-vendor-exchange-forwarding = La política de Microsoft 365 bloquea el reenvío automático a destinatarios externos. El administrador de la organización debe revisar la política de reenvío.

dsn-vendor-exchange-generic = Exchange informó un rechazo general de la entrega. El diagnóstico identifica el problema real, que puede implicar el formato del mensaje o la política de archivos adjuntos.

dsn-vendor-exchange-group-allow = El remitente no está en la lista de remitentes permitidos de este grupo de Exchange.

dsn-vendor-exchange-group-internal = Este grupo de distribución de Exchange rechaza remitentes fuera de su organización a menos que su propietario o administrador lo permita.

dsn-vendor-exchange-internal-only = Este destinatario de Exchange acepta remitentes autenticados de su propia organización; este mensaje fue tratado como externo o no autenticado.

dsn-vendor-exchange-ipv6 = El destino no acepta esta entrega superior a IPv6. El administrador de envío debe comprobar la ruta o utilizar un transporte aceptado.

dsn-vendor-exchange-journaling = La organización está intentando registrar mensajes locales en Microsoft 365 sin la configuración de archivo requerida.

dsn-vendor-exchange-onmicrosoft-limit = La organización alcanzó el límite de destinatarios externos para su dominio de envío onmicrosoft.com. El envío regular debe utilizar un dominio personalizado verificado.

dsn-vendor-exchange-public-folder = Esta carpeta pública de Exchange no acepta este remitente externo o no autenticado.

dsn-vendor-exchange-receive-rate = Este destinatario de Exchange alcanzó el límite de recepción por hora en todos los remitentes.

dsn-vendor-exchange-recipient = Exchange no pudo resolver este destinatario en su directorio. Verifique la dirección y elimine cualquier entrada de autocompletar desactualizada antes de crear un mensaje nuevo.

dsn-vendor-exchange-recipient-access = Exchange rechazó el acceso a este destinatario o dominio de destino. Verifique la dirección del destinatario y la configuración del dominio aceptado de la organización.

dsn-vendor-exchange-recipient-rate = Exchange ha recibido demasiados mensajes de este remitente a este destinatario dentro de su ventana horaria.

dsn-vendor-exchange-sender-blocked = Microsoft restringió esta cuenta de envío después de detectar sospechas de spam o abuso. Esto puede requerir que un administrador proteja y desbloquee la cuenta.

dsn-vendor-exchange-temporary-restriction = Microsoft está restringiendo temporalmente este envío de IP o tráfico mientras evalúa actividades sospechosas.

dsn-vendor-exchange-tenant-blocked = Microsoft restringió el envío para esta organización porque su tráfico se consideraba sospechoso.

dsn-vendor-exchange-tenant-blocklist = La dirección o el dominio de un destinatario está bloqueado por la lista de inquilinos permitidos/bloqueados de la organización remitente. Un destino bloqueado puede rechazar el mensaje completo.

dsn-vendor-exchange-tenant-limit = La organización Microsoft 365 ha alcanzado su límite diario continuo para destinatarios externos.

dsn-vendor-exchange-unregistered = Microsoft rechazó el envío desde un dominio que no está registrado y verificado correctamente en el inquilino.

dsn-vendor-expired = El servicio de envío se dio por vencido después de que expiró el plazo de entrega. Verifique el diagnóstico para detectar la falla de conexión o enrutamiento que causó el retraso.

dsn-vendor-fortimail-relay = FortiMail denegó esta entrega según su política de control de acceso o retransmisión. El administrador de la puerta de enlace debe verificar la regla de recepción coincidente.

dsn-vendor-gmail-app-password = Google requiere una contraseña de aplicación o un método de autorización de cuenta compatible para este inicio de sesión SMTP.

dsn-vendor-gmail-auth-method = Google rechazó un método de autenticación no compatible, como el método retirado XOAUTH.

dsn-vendor-gmail-authentication = Google no aceptó la autenticación del remitente. El diagnóstico distingue la falta de SPF/DKIM, SPF falla grave y DMARC rechazo.

dsn-vendor-gmail-browser-login = Google requiere un inicio de sesión en la cuenta o una verificación de seguridad en el navegador antes de permitir este inicio de sesión SMTP.

dsn-vendor-gmail-custom-policy = La regla de correo personalizada de un administrador de Google Workspace rechazó el mensaje. Esto no significa necesariamente que el filtro de spam de Google lo haya rechazado.

dsn-vendor-gmail-dmarc-record = Google requiere un registro DMARC con una política declarada para este dominio de envío.

dsn-vendor-gmail-rate = Google aplicó un límite de envío o retransmisión, o detectó una tasa de mensajes inusual. El diagnóstico identifica la cuenta afectada, IP, o dominio.

dsn-vendor-gmail-relay = Google Workspace no autorizó el envío de IP, dominio o cuenta para su servicio de retransmisión SMTP.

dsn-vendor-gmail-reputation = El filtrado de Google rechazó el mensaje porque consideró que el correo no era solicitado o que la reputación del envío IP/dominio era demasiado baja.

dsn-vendor-gmail-spf-record = Google considera que la configuración SPF del dominio del remitente no es válida o es sospechosa.

dsn-vendor-greylist = La puerta de enlace aplazó temporalmente esta entrega mediante listas grises. Un servidor de envío configurado correctamente normalmente lo vuelve a intentar más tarde.

dsn-vendor-headers = El servicio receptor rechazó encabezados de mensajes no válidos o faltantes. La aplicación de envío debe generar correo con el formato correcto.

dsn-vendor-helo = El servidor de envío utilizó una identidad HELO/EHLO inaceptable. Su saludo debe identificar correctamente el servidor emisor.

dsn-vendor-ip-block = El servicio receptor rechazó el envío IP debido a su reputación o a una entrada en la lista de bloqueo.

dsn-vendor-mimecast-block = Mimecast rechazó el remitente o el dominio bajo una política administrativa o de remitente bloqueado.

dsn-vendor-mimecast-content = Una política de examen de contenido de Mimecast rechazó el mensaje.

dsn-vendor-mimecast-spoof = La política anti-spoofing de Mimecast rechazó la identidad del remitente.

dsn-vendor-mta-sts-host = El servidor de correo de destino no está permitido por la política MTA-STS del dominio.

dsn-vendor-proofpoint-block = Proofpoint bloqueó el envío IP o el dominio después de detectar spam o actividad maliciosa.

dsn-vendor-proofpoint-throttle = Proofpoint pospuso temporalmente este envío IP porque su patrón de envío se consideró inusual.

dsn-vendor-ptr = El envío IP carece de un reverso aceptable DNS, o su nombre PTR no se resuelve de nuevo en el mismo IP.

dsn-vendor-recipient-ip-block = La organización receptora bloquea explícitamente la dirección de envío IP. Su administrador controla este bloque local.

dsn-vendor-relay = El servidor de correo no transmitirá este mensaje al destino. Verifique la autenticación SMTP, las IP de envío confiables, los conectores y los dominios aceptados.

dsn-vendor-routing-loop = El mensaje entró en un bucle de reenvío o enrutamiento y superó el número permitido de saltos.

dsn-vendor-sending-limit = La cuenta emisora ha excedido una cuota de mensajes o destinatarios. Los reenvíos inmediatos repetidos no eliminarán el límite.

dsn-vendor-spf = El receptor no aceptó la autenticación SPF para el dominio de envío IP y el sobre-remitente.

dsn-vendor-starttls = La conexión cifrada SMTP requerida no se pudo establecer o no fue compatible con el siguiente servidor.

dsn-vendor-yahoo-deferred = Yahoo aplazó temporalmente este mensaje, normalmente mientras evaluaba el volumen, la reputación o las quejas de los destinatarios. Verifique el diagnóstico para la causa indicada.

dsn-vendor-yahoo-policy = Yahoo rechazó el mensaje según su política de correo. La autenticación, la alineación DMARC o el contenido del mensaje pueden estar involucrados; el diagnóstico da la razón específica.
