# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooks
admin-webhooks-intro = Un webhook saliente envía un aviso firmado a una URL cuando ocurre algo en una cuenta. Un webhook entrante permite que otro sistema ejecute una acción de regla de correo en una cuenta.
admin-webhooks-empty = Aún no hay webhooks.
admin-webhooks-unavailable = Este servidor no ofrece la gestión de webhooks.
admin-webhooks-load-error = No se pudieron cargar los webhooks.
admin-webhooks-direction-outbound = Saliente
admin-webhooks-direction-inbound = Entrante
admin-webhooks-col-direction = Dirección
admin-webhooks-col-label = Nombre
admin-webhooks-col-account = Cuenta
admin-webhooks-col-target = Envía a o permite
admin-webhooks-col-created = Creado
admin-webhooks-no-label = (sin nombre)
admin-webhooks-created-by = por { $who }
admin-webhooks-last-used = Último uso { $when }
admin-webhooks-never-used = Aún sin usar
admin-webhooks-delete = Eliminar
admin-webhooks-delete-named = Eliminar el webhook { $name }
admin-webhooks-delete-title = ¿Eliminar el webhook { $name }?
admin-webhooks-delete-body = Se detienen los envíos a él, o se rechazan las llamadas firmadas con su secreto. No se puede deshacer; un webhook nuevo recibe un secreto nuevo.
admin-webhooks-deleted = Webhook eliminado.
admin-webhooks-add = Añadir un webhook
admin-webhooks-add-direction = Dirección
admin-webhooks-add-outbound-help = Mailwoman llama a su URL.
admin-webhooks-add-inbound-help = Su sistema llama a Mailwoman.
admin-webhooks-add-account = Cuenta
admin-webhooks-add-account-help = La cuenta cuyos eventos se envían, o la única cuenta sobre la que puede actuar una llamada entrante.
admin-webhooks-add-label = Nombre
admin-webhooks-add-label-help = Para qué sirve el webhook, de modo que otros lo sepan. Opcional.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Debe llegar a una dirección pública de internet. Se rechazan las direcciones privadas, de bucle local y de enlace local.
admin-webhooks-add-events = Eventos
admin-webhooks-add-actions = Acciones permitidas
admin-webhooks-add-submit = Crear webhook
admin-webhooks-add-saving = Creando…
admin-webhooks-add-check = Comprobar
admin-webhooks-add-check-ok = La URL supera la comprobación. No se guardó nada.
admin-webhooks-event-message-received = Mensaje nuevo
admin-webhooks-event-message-flags = Mensaje leído, marcado o etiquetado
admin-webhooks-event-message-moved = Mensaje movido
admin-webhooks-event-message-deleted = Mensaje eliminado
admin-webhooks-event-submission-sent = Mensaje enviado
admin-webhooks-event-submission-failed = No se pudo enviar el mensaje
admin-webhooks-event-pim-changed = Calendario, contactos, tareas o notas modificados
admin-webhooks-action-run-rules = Ejecutar las reglas de correo de la cuenta sobre la descripción de un mensaje
admin-webhooks-action-sieve = Aplicar una acción de regla a un mensaje
admin-webhooks-secret-title = Secreto de { $name }
admin-webhooks-secret-once = Copie este secreto ahora. No se volverá a mostrar.
admin-webhooks-secret-outbound = Cada entrega incluye X-Mailwoman-Signature: sha256= seguido del HMAC-SHA256 del cuerpo de la solicitud con este secreto.
admin-webhooks-secret-inbound = Envíe X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (segundos Unix), X-Mailwoman-Nonce (de 16 a 128 letras, dígitos, - o _, nunca reutilizado) y X-Mailwoman-Signature: sha256= seguido del HMAC-SHA256 de timestamp.nonce.body con este secreto. Se rechazan las llamadas con más de cinco minutos de desfase o que reutilicen un nonce.
admin-webhooks-secret-done = Lo he copiado
admin-webhooks-error-stale = Alguien ha cambiado los webhooks desde que abrió esta página. Se recargó la lista; inténtelo de nuevo.
admin-webhooks-error-account = Introduzca la cuenta.
admin-webhooks-error-url = Introduzca una URL que empiece por https:// o http://.
admin-webhooks-error-events = Elija al menos un evento.
admin-webhooks-error-actions = Elija al menos una acción.
