# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Deshacer envío
send-delay-help = Después de pulsar Enviar, el mensaje espera este tiempo en su bandeja de salida. Hasta entonces puede recuperarlo y seguir editándolo. El servidor lo retiene, así que cerrar el navegador no lo envía antes de tiempo ni lo pierde.
send-delay-label = Retener los mensajes enviados durante
send-delay-off = Desactivado (enviar de inmediato)
send-delay-seconds = { $n } segundos
send-delay-minutes = { $n ->
    [one] 1 minuto
   *[other] { $n } minutos
    }
send-delay-custom = Personalizado…
send-delay-custom-label = Retraso personalizado en segundos
send-delay-custom-help = Segundos enteros, de 0 a { $max }.
send-delay-capped = Su administrador limita esto a { $max }.
send-delay-disabled = Su administrador ha desactivado la recuperación de mensajes enviados. Los mensajes se envían de inmediato.
send-delay-invalid = Introduzca un número entero de segundos de 0 a { $max }.
send-later-button = Enviar más tarde
send-later-menu = Opciones de envío posterior
send-later-preset-hour = Dentro de 1 hora
send-later-preset-today = Más tarde hoy, 17:00
send-later-preset-tomorrow = Mañana por la mañana, 09:00
send-later-preset-monday = El lunes por la mañana, 09:00
send-later-custom = Elegir fecha y hora…
send-later-custom-label = Enviar el
send-later-zone = Zona horaria: { $zone }
send-later-schedule = Programar envío
send-later-cancel = Cancelar
send-later-err-invalid = Introduzca una fecha y una hora válidas.
send-later-err-past = Elija una hora futura.
send-later-err-horizon = Eso queda demasiado lejos. Los mensajes se pueden programar con hasta { $days } días de antelación.
send-later-err-off = Su administrador ha desactivado el envío programado.
send-queued = Mensaje en cola — se envía en
send-queued-undo = Deshacer
send-queued-now = Enviar ahora
send-queued-recalled = Mensaje recuperado. Está de nuevo en sus borradores.
send-queued-too-late = Demasiado tarde para recuperarlo: el mensaje ya se está enviando.
send-queued-sent-now = Enviando ahora.
send-queued-scheduled = Programado para el { $when } ({ $zone }).
outbox-scheduled-badge = Programado
outbox-sends-in = Se envía en { $time }
outbox-sends-at = Se envía el { $when }
outbox-recall-edit = Recuperar y editar
outbox-recall-too-late = Ya se está enviando; ya no se puede recuperar.
outbox-failed-drafts = No se pudo enviar y se conservó en sus borradores.
outbox-queued-count = { $count ->
    [one] 1 mensaje esperando para enviarse
   *[other] { $count } mensajes esperando para enviarse
    }
