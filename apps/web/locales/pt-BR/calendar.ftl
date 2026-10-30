# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Agenda
calendar-prev = Anterior
calendar-next = Próximo
calendar-today = Hoje
calendar-views = Visualizações da agenda
calendar-new-event = Novo evento
calendar-import = Importar
calendar-export = Exportar
calendar-import-file = Importar arquivo de agenda

# -- View names --------------------------------------------------------------
calendar-view-day = Dia
calendar-view-3day = 3 dias
calendar-view-work-week = Semana útil
calendar-view-week = Semana
calendar-view-month = Mês
calendar-view-tri-month = Trimestre
calendar-view-schedule = Programação
calendar-view-agenda = Lista
calendar-view-year = Ano

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Agendas
calendar-toggle = Alternar { $name }
calendar-color-for = Cor de { $name }
calendar-add-calendar = Adicionar agenda
calendar-new-calendar-name = Nova agenda
calendar-holidays-heading = Feriados
calendar-subscribe-holidays = Assinar feriados
calendar-add-region = Adicionar uma região…
calendar-synced = Agenda sincronizada

# -- Grid / views ------------------------------------------------------------
calendar-all-day = dia inteiro
calendar-all-day-full = Dia inteiro
calendar-more = Mais { $count }
calendar-conflict = conflito
calendar-no-events-30 = Nenhum evento nos próximos 30 dias.
calendar-events-count = { $count ->
    [0] Nenhum evento
    [one] { $count } evento
   *[other] { $count } eventos
}
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Mês de { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Grade de horários
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }, { $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }, { $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = Dia inteiro: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Novo evento
calendar-editor-edit = Editar evento
calendar-invitation = Convite
calendar-invited = Você foi convidado ({ $status }).
calendar-accept = Aceitar
calendar-tentative = Talvez
calendar-decline = Recusar
calendar-counter = Contraproposta…
calendar-propose-start = Propor novo início
calendar-send-counter = Enviar contraproposta
calendar-overlaps = Este evento se sobrepõe a { $count ->
    [one] { $count } outro evento
   *[other] { $count } outros eventos
}.
calendar-field-title = Título
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (sem título)
calendar-field-calendar = Agenda
calendar-field-all-day = Dia inteiro
calendar-field-start = Início
calendar-field-duration = Duração (min)
calendar-field-location = Local
calendar-field-shows-as = Mostrar como
calendar-shows-busy = Ocupado
calendar-shows-free = Livre
calendar-field-status = Status
calendar-status-confirmed = Confirmado
calendar-status-tentative = Provisório
calendar-status-cancelled = Cancelado
calendar-repeats = Repetição
calendar-frequency = Frequência
calendar-freq-daily = Diariamente
calendar-freq-weekly = Semanalmente
calendar-freq-monthly = Mensalmente
calendar-freq-yearly = Anualmente
calendar-every = a cada
calendar-interval = Intervalo
calendar-ends = Termina
calendar-end-mode = Modo de término
calendar-end-never = Nunca
calendar-end-count = Após N
calendar-end-until = Na data
calendar-occurrences = Ocorrências
calendar-until-date = Até a data
calendar-reminders = Lembretes
calendar-reminder-before = { $min } min antes
calendar-remove-reminder = Remover lembrete { $min }
calendar-add-reminder = Adicionar lembrete
calendar-reminder-none = + lembrete
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 hora
calendar-reminder-1440 = 1 dia
calendar-attendees = Participantes
calendar-add-attendee = Adicionar participante
calendar-attendee-invalid = Isso não é um endereço de e-mail. Informe um endereço, como name@example.com.
calendar-save-failed = O evento não foi salvo. O servidor o recusou.
calendar-remove-attendee = Remover { $email }
calendar-notes = Notas

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Função de { $email }
calendar-attendee-cutype-for = Tipo de { $email }
calendar-role-chair = Organizador
calendar-role-required = Obrigatório
calendar-role-optional = Opcional
calendar-role-non-participant = Não participante
calendar-cutype-individual = Pessoa
calendar-cutype-group = Grupo
calendar-cutype-resource = Recurso
calendar-cutype-room = Sala
calendar-partstat-needs-action = Sem resposta
calendar-partstat-accepted = Aceito
calendar-partstat-declined = Recusado
calendar-partstat-tentative = Talvez

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = { $start } até { $end }: { $title }
calendar-schedule-free = { $gap } livre
calendar-gap-hm = { $h } h { $m } min
calendar-gap-h = { $h } h
calendar-gap-m = { $m } min

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Resolver { $count ->
    [one] { $count } conflito
   *[other] { $count } conflitos
}
calendar-resolver-title = Resolver conflitos
calendar-resolver-none = Nenhum conflito a resolver.
calendar-resolver-pick = Conflito
calendar-resolver-pair-n = { $n } de { $total }: { $a } x { $b }
calendar-resolver-time = Horário
calendar-resolver-overlap = Sobreposição { $start } – { $end }
calendar-resolver-reschedule = Reagendar o evento posterior
calendar-resolver-shorten = Encurtar o evento anterior
calendar-resolver-tentative = Marcar o posterior como provisório
calendar-resolver-double-book = Agendamento duplo (marcar como livre)
calendar-resolver-keep = Manter ambos
calendar-events-count-people = { $count ->
    [0] Nenhum participante
    [one] { $count } participante
   *[other] { $count } participantes
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Agendas
calendar-fb-own = Suas
calendar-fb-unavailable = Não foi possível carregar seu horário ocupado.
calendar-fb-cell = { $principal } às { $hour }: { $status }
calendar-fb-busy = Ocupado
calendar-fb-tentative = Provisório
calendar-fb-free = Livre

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Adicionar evento rápido
calendar-quick-add-placeholder = Adição rápida, ex.: Almoço sexta 13h
calendar-quick-add-btn = Adicionar
calendar-quick-add-do = Adição rápida

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Categorias
calendar-add-category = Adicionar categoria
calendar-category-placeholder = Adicionar uma categoria
calendar-remove-category = Remover categoria { $name }
calendar-filter-heading = Filtro
calendar-filter-category = Filtrar por categoria
calendar-filter-category-placeholder = Categoria a exibir

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Anexos
calendar-attachment-title = Nome do anexo
calendar-attachment-title-placeholder = Nome (opcional)
calendar-attachment-uri = Link do anexo
calendar-attachment-add = Adicionar anexo
calendar-remove-attachment = Remover anexo { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Assinar por URL
calendar-subscribe-url = URL da agenda
calendar-subscribe-add = Assinar
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Não foi possível obter a agenda nesse endereço, então ela não foi adicionada.
calendar-subscribe-hol-many-locations = Este arquivo de feriados lista mais de um local. Baixe-o, use Importar e escolha os locais desejados.
calendar-subscribe-refresh = Atualizar { $name }
calendar-subscribe-refresh-btn = Atualizar
calendar-subscribe-refresh-failed = Não foi possível obter o feed da agenda. Os eventos foram mantidos como estavam.
calendar-import-failed = Não foi possível ler esse arquivo como uma agenda. Nada foi importado.
calendar-import-done = { $count ->
    [one] 1 evento importado.
   *[other] { $count } eventos importados.
}
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Escolher locais a importar
calendar-import-locations-intro = Este arquivo contém feriados de { $count } locais. Somente os locais selecionados são importados.
calendar-import-locations-filter = Filtrar locais
calendar-import-locations-list = Locais no arquivo
calendar-import-locations-none = Nenhum local corresponde ao filtro.
calendar-import-locations-picked = { $count ->
    [one] 1 local selecionado.
   *[other] { $count } locais selecionados.
}
calendar-import-locations-import = Importar selecionados
calendar-export-failed = Não foi possível exportar a agenda.
calendar-quick-add-failed = Não foi possível adicionar esse evento.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Compartilhar
calendar-share-for = Compartilhar { $name }
calendar-share-title = Compartilhar { $name }
calendar-share-intro = As pessoas com quem você compartilha podem ver esta agenda. Conceda acesso de leitura e gravação para permitir que adicionem e editem eventos.
calendar-share-people = Compartilhada com
calendar-share-empty = Esta agenda não está compartilhada com ninguém.
calendar-share-add = Adicionar uma pessoa
calendar-share-new-access = Acesso da nova pessoa
calendar-share-access-for = Acesso de { $principal }
calendar-share-remove = Remover { $principal }
calendar-share-read = Somente leitura
calendar-share-readwrite = Leitura e gravação
