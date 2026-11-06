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

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Navegador de datas
calendar-event-range = { $title }, de { $start } a { $end }
calendar-busy-free = Aparece como livre
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = Mais { $count } { $count ->
    [one] evento
   *[other] eventos
  } em { $date }
calendar-fb-caption-people = Horários ocupados no dia do conflito, por hora, para você e cada pessoa convidada.
calendar-fb-unknown = Não foi possível obter a disponibilidade de { $principal }.
calendar-fb-loading = Carregando disponibilidade…
calendar-import-itip-message = Este arquivo é uma resposta ou cancelamento enviado por e-mail. Abra a mensagem que o acompanha para aplicá-lo. Nada foi importado.
calendar-import-skipped = { $count ->
    [one] { $count } evento já estava na agenda e foi mantido como está.
   *[other] { $count } eventos já estavam na agenda e foram mantidos como estão.
 }
calendar-invite-title = Convite de calendário
calendar-invite-sender = De { $sender }
calendar-invite-add = Adicionar ao calendário
calendar-invite-update = Aplicar atualização do evento
calendar-invite-cancel = Aplicar cancelamento
calendar-invite-reply = Registrar resposta do participante
calendar-invite-new = Este convite pode ser adicionado ao seu calendário.
calendar-invite-newer = O organizador enviou um evento atualizado.
calendar-invite-current = Seu calendário já tem esta versão.
calendar-invite-stale = Esta mensagem é mais antiga que o evento no seu calendário.
calendar-invite-cancelled = O organizador cancelou este evento.
calendar-invite-response = Um participante enviou uma resposta.
calendar-invite-not-organizer = O remetente não é o organizador armazenado. Nada será alterado.
calendar-invite-not-attendee = O remetente não é um participante correspondente. Nada será alterado.
calendar-invite-unknown = Esta mensagem se refere a um evento que não está no seu calendário.
calendar-invite-unsupported = Esta proposta não pode ser aplicada automaticamente.
calendar-invite-no-reply = Isso atualiza seu calendário sem enviar uma resposta.
calendar-invite-applied = Seu calendário foi atualizado.
calendar-invite-unchanged = O evento não foi alterado. Confira o status atual do convite.
calendar-invite-apply-failed = Não foi possível aplicar o convite. Tente novamente.
calendar-invite-load-failed = Não foi possível verificar o convite.
calendar-invite-retry = Tentar novamente
calendar-field-time-zone = Fuso horário do evento
calendar-field-time-zone-help = Use um nome IANA como Europe/Lisbon. O evento mantém este fuso quando visto em outro lugar.
calendar-time-invalid = Informe um fuso horário e um horário local válidos. Alguns horários locais não existem quando os relógios são adiantados.
calendar-mini-prev = Mês anterior
calendar-mini-next = Próximo mês
calendar-card-region-event = Evento do calendário
calendar-card-kind-request = Convite para reunião
calendar-card-kind-update = Convite atualizado
calendar-card-kind-cancel = Reunião cancelada
calendar-card-kind-reply = Resposta à reunião
calendar-card-kind-counter = Novo horário proposto
calendar-card-kind-publish = { $count ->
    [one] Evento do calendário
   *[other] { $count } eventos do calendário
 }
calendar-card-when = Quando
calendar-card-where = Onde
calendar-card-organizer = Organizador
calendar-card-attendees = Participantes
calendar-card-all-day = { $when } (dia inteiro)
calendar-card-organizer-time = Horário do organizador: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Todos os dias
   *[other] A cada { $interval } dias
 }
calendar-card-repeat-weekly = { $interval ->
    [one] Toda semana
   *[other] A cada { $interval } semanas
 }
calendar-card-repeat-weekly-on = { $interval ->
    [one] Toda semana: { $days }
   *[other] A cada { $interval } semanas: { $days }
 }
calendar-card-repeat-monthly = { $interval ->
    [one] Todo mês
   *[other] A cada { $interval } meses
 }
calendar-card-repeat-monthly-day = { $interval ->
    [one] Todo mês, no dia { $day }
   *[other] A cada { $interval } meses, no dia { $day }
 }
calendar-card-repeat-yearly = { $interval ->
    [one] Todo ano
   *[other] A cada { $interval } anos
 }
calendar-card-repeat-until = { $rule } até { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] uma vez
   *[other] { $count } vezes
 }
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 data ignorada
   *[other] { $count } datas ignoradas
 })
calendar-card-repeat-custom = Repete em uma programação personalizada
calendar-card-join = Participar pelo { $provider }
calendar-card-organizer-is = Organizado por { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 participante
   *[other] { $count } participantes
 } · { $accepted } aceitaram, { $tentative } talvez, { $declined } recusaram
calendar-card-people-all = Mostrar todos ({ $count })
calendar-card-people-fewer = Mostrar menos
calendar-card-rsvp-accepted = Aceito
calendar-card-rsvp-tentative = Provisório
calendar-card-rsvp-declined = Recusado
calendar-card-rsvp-none = Sem resposta
calendar-card-in-calendar = No seu calendário
calendar-card-cancelled = Cancelado
calendar-card-open = Abrir no calendário
calendar-card-agenda = Seu calendário em { $day }
calendar-card-agenda-title = Seu dia
calendar-card-agenda-this-event = Este evento
calendar-card-agenda-unavailable = Não foi possível carregar seu calendário deste dia.
calendar-card-conflicts = { $count ->
    [one] Conflito com { $titles }
   *[other] Conflito com { $count } eventos: { $titles }
 }
calendar-card-no-conflicts = Sem conflitos no seu calendário
calendar-card-invited-as = Você foi convidado como { $address }.
calendar-card-not-invited = Este convite não lista nenhum dos seus endereços. Você pode adicioná-lo ao calendário, mas não pode respondê-lo.
calendar-card-not-invited-short = Este convite não lista nenhum dos seus endereços.
calendar-card-out-of-date = Este convite está desatualizado. Uma versão mais recente já está no seu calendário.
calendar-card-replied = { $who } { $answer ->
    [accepted] aceitou.
    [tentative] aceitou provisoriamente.
    [declined] recusou.
   *[other] ainda não respondeu.
 }
calendar-card-counter = { $who } propôs um novo horário. As propostas não são aplicadas automaticamente.
calendar-card-todos-skipped = { $count ->
    [one] O arquivo também contém 1 tarefa, que não é adicionada.
   *[other] O arquivo também contém { $count } tarefas, que não são adicionadas.
 }
calendar-card-respond = Responder ao convite
calendar-card-accept = Aceitar
calendar-card-tentative = Provisório
calendar-card-decline = Recusar
calendar-card-propose = Propor novo horário
calendar-card-propose-start = Novo início
calendar-card-propose-send = Enviar proposta
calendar-card-propose-invalid = Informe um novo horário de início válido.
calendar-card-send-response = Enviar uma resposta para { $organizer }
calendar-card-comment = Comentário (opcional)
calendar-card-calendar = Calendário
calendar-card-events = Eventos neste arquivo
calendar-card-add-one = Adicionar
calendar-card-add-all = Adicionar todos ({ $count })
calendar-card-added = { $count ->
    [one] Adicionado ao seu calendário.
   *[other] { $count } eventos adicionados ao seu calendário.
 }
calendar-card-remove = Remover do calendário
calendar-card-removed = Removido do seu calendário.
calendar-card-record-reply = Atualizar status do participante
calendar-card-answered-sent = { $answer }. Sua resposta foi enviada para { $organizer }.
calendar-card-answered-not-sent = { $answer }. Seu calendário foi atualizado, mas não foi possível enviar a resposta.
calendar-card-answered-silently = { $answer }. Nenhuma resposta foi enviada.
calendar-card-proposal-sent = Sua proposta foi enviada para { $organizer }.
calendar-card-not-sent = Não foi possível enviar a mensagem.
calendar-card-more = { $count ->
    [one] mais 1
   *[other] mais { $count }
 }
