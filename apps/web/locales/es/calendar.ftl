# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Calendario
calendar-prev = Anterior
calendar-next = Siguiente
calendar-today = Hoy
calendar-views = Vistas del calendario
calendar-new-event = Nuevo evento
calendar-import = Importar
calendar-export = Exportar
calendar-import-file = Importar archivo de calendario

# -- View names --------------------------------------------------------------
calendar-view-day = Día
calendar-view-3day = 3 días
calendar-view-work-week = Semana laboral
calendar-view-week = Semana
calendar-view-month = Mes
calendar-view-tri-month = Trimestre
calendar-view-schedule = Programación
calendar-view-agenda = Agenda
calendar-view-year = Año

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Calendarios
calendar-toggle = Mostrar u ocultar { $name }
calendar-color-for = Color de { $name }
calendar-add-calendar = Añadir calendario
calendar-new-calendar-name = Nuevo calendario
calendar-holidays-heading = Festivos
calendar-subscribe-holidays = Suscribirse a festivos
calendar-add-region = Añadir una región…
calendar-synced = Calendario sincronizado

# -- Grid / views ------------------------------------------------------------
calendar-all-day = todo el día
calendar-all-day-full = Todo el día
calendar-more = { $count } más
calendar-conflict = conflicto
calendar-no-events-30 = No hay eventos en los próximos 30 días.
calendar-events-count = { $count ->
    [zero] Ningún evento
    [one] { $count } evento
    [many] { $count } de eventos
   *[other] { $count } eventos
}
calendar-month-grid = Mes de { $month }
calendar-time-grid = Cuadrícula horaria
calendar-cell = { $date }, { $events }
calendar-announce = { $date }, { $events }
calendar-event-at = { $time } { $title }
calendar-event-allday = Todo el día: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Nuevo evento
calendar-editor-edit = Editar evento
calendar-invitation = Invitación
calendar-invited = Está invitado ({ $status }).
calendar-accept = Aceptar
calendar-tentative = Provisional
calendar-decline = Rechazar
calendar-counter = Contraproponer…
calendar-propose-start = Proponer nuevo inicio
calendar-send-counter = Enviar contrapropuesta
calendar-overlaps = Este evento se solapa con { $count ->
    [one] { $count } evento más
    [many] { $count } de eventos más
   *[other] { $count } eventos más
}.
calendar-field-title = Título
calendar-untitled = (sin título)
calendar-field-calendar = Calendario
calendar-field-all-day = Todo el día
calendar-field-start = Inicio
calendar-field-duration = Duración (min)
calendar-field-location = Ubicación
calendar-field-shows-as = Mostrar como
calendar-shows-busy = Ocupado
calendar-shows-free = Libre
calendar-field-status = Estado
calendar-status-confirmed = Confirmado
calendar-status-tentative = Provisional
calendar-status-cancelled = Cancelado
calendar-repeats = Repetición
calendar-frequency = Frecuencia
calendar-freq-daily = Diaria
calendar-freq-weekly = Semanal
calendar-freq-monthly = Mensual
calendar-freq-yearly = Anual
calendar-every = cada
calendar-interval = Intervalo
calendar-ends = Termina
calendar-end-mode = Modo de fin
calendar-end-never = Nunca
calendar-end-count = Tras N veces
calendar-end-until = En una fecha
calendar-occurrences = Repeticiones
calendar-until-date = Hasta la fecha
calendar-reminders = Recordatorios
calendar-reminder-before = { $min } min antes
calendar-remove-reminder = Quitar recordatorio { $min }
calendar-add-reminder = Añadir recordatorio
calendar-reminder-none = + recordatorio
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 hora
calendar-reminder-1440 = 1 día
calendar-attendees = Asistentes
calendar-add-attendee = Añadir asistente
calendar-attendee-invalid = Eso no es una dirección de correo. Introduzca una dirección, como name@example.com.
calendar-save-failed = El evento no se guardó. El servidor lo rechazó.
calendar-remove-attendee = Quitar a { $email }
calendar-notes = Notas

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Función de { $email }
calendar-attendee-cutype-for = Tipo de { $email }
calendar-role-chair = Presidente
calendar-role-required = Obligatorio
calendar-role-optional = Opcional
calendar-role-non-participant = No participante
calendar-cutype-individual = Persona
calendar-cutype-group = Grupo
calendar-cutype-resource = Recurso
calendar-cutype-room = Sala
calendar-partstat-needs-action = Sin respuesta
calendar-partstat-accepted = Aceptado
calendar-partstat-declined = Rechazado
calendar-partstat-tentative = Provisional

# -- Schedule view (distinct from Agenda) ------------------------------------
calendar-schedule-row = De { $start } a { $end }: { $title }
calendar-schedule-free = { $gap } libre
calendar-gap-hm = { $h } h { $m } min
calendar-gap-h = { $h } h
calendar-gap-m = { $m } min

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Resolver { $count ->
    [one] { $count } conflicto
    [many] { $count } de conflictos
   *[other] { $count } conflictos
}
calendar-resolver-title = Resolver conflictos
calendar-resolver-none = No hay conflictos que resolver.
calendar-resolver-pick = Conflicto
calendar-resolver-pair-n = { $n } de { $total }: { $a } frente a { $b }
calendar-resolver-time = Hora
calendar-resolver-overlap = Solapamiento { $start } – { $end }
calendar-resolver-reschedule = Reprogramar el evento posterior
calendar-resolver-shorten = Acortar el evento anterior
calendar-resolver-tentative = Marcar el posterior como provisional
calendar-resolver-double-book = Reserva doble (marcar libre)
calendar-resolver-keep = Mantener ambos
calendar-events-count-people = { $count ->
    [zero] Ningún asistente
    [one] { $count } asistente
    [many] { $count } de asistentes
   *[other] { $count } asistentes
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Calendarios
calendar-fb-own = Suyos
calendar-fb-unavailable = No se pudo cargar su tiempo ocupado.
calendar-fb-cell = { $principal } a las { $hour }: { $status }
calendar-fb-busy = Ocupado
calendar-fb-tentative = Provisional
calendar-fb-free = Libre

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Añadir evento rápido
calendar-quick-add-placeholder = Añadir rápido, p. ej. Comida viernes 13:00
calendar-quick-add-btn = Añadir
calendar-quick-add-do = Añadir rápido

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Categorías
calendar-add-category = Añadir categoría
calendar-category-placeholder = Añadir una categoría
calendar-remove-category = Quitar categoría { $name }
calendar-filter-heading = Filtro
calendar-filter-category = Filtrar por categoría
calendar-filter-category-placeholder = Categoría que mostrar

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Adjuntos
calendar-attachment-title = Nombre del adjunto
calendar-attachment-title-placeholder = Nombre (opcional)
calendar-attachment-uri = Enlace del adjunto
calendar-attachment-add = Añadir adjunto
calendar-remove-attachment = Quitar adjunto { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Suscribirse por URL
calendar-subscribe-url = URL del calendario
calendar-subscribe-add = Suscribirse
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = No se pudo obtener el calendario de esa dirección, así que no se añadió.
calendar-subscribe-hol-many-locations = Este archivo de festivos incluye más de una ubicación. Descárguelo, use Importar y elija las ubicaciones que quiera.
calendar-subscribe-refresh = Actualizar { $name }
calendar-subscribe-refresh-btn = Actualizar
calendar-subscribe-refresh-failed = No se pudo obtener el feed del calendario. Sus eventos se dejaron como estaban.
calendar-import-failed = Ese archivo no se pudo leer como calendario. No se importó nada.
calendar-import-done = { $count ->
    [one] Se importó 1 evento.
    [many] Se importaron { $count } de eventos.
   *[other] Se importaron { $count } eventos.
}
calendar-import-locations-title = Elegir ubicaciones que importar
calendar-import-locations-intro = Este archivo contiene festivos de { $count } ubicaciones. Solo se importan las ubicaciones que seleccione.
calendar-import-locations-filter = Filtrar ubicaciones
calendar-import-locations-list = Ubicaciones del archivo
calendar-import-locations-none = Ninguna ubicación coincide con el filtro.
calendar-import-locations-picked = { $count ->
    [one] 1 ubicación seleccionada.
    [many] { $count } de ubicaciones seleccionadas.
   *[other] { $count } ubicaciones seleccionadas.
}
calendar-import-locations-import = Importar selección
calendar-export-failed = No se pudo exportar el calendario.
calendar-quick-add-failed = No se pudo añadir ese evento.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Compartir
calendar-share-for = Compartir { $name }
calendar-share-title = Compartir { $name }
calendar-share-intro = Las personas con las que comparta pueden ver este calendario. Dé acceso de lectura y escritura para que puedan añadir y editar eventos.
calendar-share-people = Compartido con
calendar-share-empty = Este calendario no está compartido con nadie.
calendar-share-add = Añadir una persona
calendar-share-new-access = Acceso para la nueva persona
calendar-share-access-for = Acceso para { $principal }
calendar-share-remove = Quitar a { $principal }
calendar-share-read = Solo lectura
calendar-share-readwrite = Lectura y escritura
