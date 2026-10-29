# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Calendário
calendar-mini = Navegador de datas
calendar-prev = Anterior
calendar-next = Seguinte
calendar-today = Hoje
calendar-views = Vistas do calendário
calendar-new-event = Novo evento
calendar-import = Importar
calendar-export = Exportar
calendar-import-file = Importar ficheiro de calendário

# -- View names --------------------------------------------------------------
calendar-view-day = Dia
calendar-view-3day = 3 dias
calendar-view-work-week = Semana de trabalho
calendar-view-week = Semana
calendar-view-month = Mês
calendar-view-tri-month = Trimestre
calendar-view-schedule = Programação
calendar-view-agenda = Agenda
calendar-view-year = Ano

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Calendários
calendar-toggle = Mostrar/ocultar { $name }
calendar-color-for = Cor de { $name }
calendar-add-calendar = Adicionar calendário
calendar-new-calendar-name = Novo calendário
calendar-holidays-heading = Feriados
calendar-subscribe-holidays = Subscrever feriados
calendar-add-region = Adicionar uma região…
calendar-synced = Calendário sincronizado

# -- Grid / views ------------------------------------------------------------
calendar-all-day = dia inteiro
calendar-all-day-full = Dia inteiro
calendar-more = Mais { $count }
calendar-conflict = conflito
calendar-no-events-30 = Sem eventos nos próximos 30 dias.
calendar-events-count = { $count ->
    [zero] Sem eventos
    [one] { $count } evento
   *[other] { $count } eventos
}
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Mês de { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Grelha horária
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
calendar-invited = Recebeu um convite ({ $status }).
calendar-accept = Aceitar
calendar-tentative = Provisório
calendar-decline = Recusar
calendar-counter = Contraproposta…
calendar-propose-start = Propor novo início
calendar-send-counter = Enviar contraproposta
calendar-overlaps = Este evento sobrepõe-se a { $count ->
    [one] { $count } outro evento
   *[other] { $count } outros eventos
}.
calendar-field-title = Título
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (sem título)
calendar-field-calendar = Calendário
calendar-field-all-day = Dia inteiro
calendar-field-start = Início
calendar-field-duration = Duração (min)
calendar-field-location = Local
calendar-field-shows-as = Mostrar como
calendar-shows-busy = Ocupado
calendar-shows-free = Livre
calendar-field-status = Estado
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
calendar-end-mode = Modo de fim
calendar-end-never = Nunca
calendar-end-count = Após N
calendar-end-until = Numa data
calendar-occurrences = Ocorrências
calendar-until-date = Até à data
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
calendar-attendee-invalid = Isto não é um endereço de e-mail. Introduza um endereço, como name@example.com.
calendar-save-failed = O evento não foi guardado. O servidor recusou-o.
calendar-remove-attendee = Remover { $email }
calendar-notes = Notas

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Função de { $email }
calendar-attendee-cutype-for = Tipo de { $email }
calendar-role-chair = Moderador
calendar-role-required = Obrigatório
calendar-role-optional = Opcional
calendar-role-non-participant = Não participante
calendar-cutype-individual = Pessoa
calendar-cutype-group = Grupo
calendar-cutype-resource = Recurso
calendar-cutype-room = Sala
calendar-partstat-needs-action = Sem resposta
calendar-partstat-accepted = Aceite
calendar-partstat-declined = Recusado
calendar-partstat-tentative = Provisório

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = { $start } a { $end }: { $title }
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
calendar-resolver-none = Não há conflitos para resolver.
calendar-resolver-pick = Conflito
calendar-resolver-pair-n = { $n } de { $total }: { $a } vs. { $b }
calendar-resolver-time = Hora
calendar-resolver-overlap = Sobreposição { $start } – { $end }
calendar-resolver-reschedule = Reagendar o evento posterior
calendar-resolver-shorten = Encurtar o evento anterior
calendar-resolver-tentative = Marcar o posterior como provisório
calendar-resolver-double-book = Sobrepor (marcar como livre)
calendar-resolver-keep = Manter ambos
calendar-events-count-people = { $count ->
    [zero] Sem participantes
    [one] { $count } participante
   *[other] { $count } participantes
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-caption-people = Tempo ocupado no dia do conflito, por hora, para si e para cada pessoa convidada.
calendar-fb-whose = Calendários
calendar-fb-own = O seu
# A grid row for an invited person whose busy time no source could give
# (no shared calendar on this server, no answer from the calendar server).
calendar-fb-unknown = Não foi possível obter a disponibilidade de { $principal }.
calendar-fb-unavailable = Não foi possível carregar o seu tempo ocupado.
calendar-fb-loading = A carregar a disponibilidade…
calendar-fb-cell = { $principal } às { $hour }: { $status }
calendar-fb-busy = Ocupado
calendar-fb-tentative = Provisório
calendar-fb-free = Livre

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Adicionar evento rápido
calendar-quick-add-placeholder = Adição rápida, p. ex. Lunch Friday 1pm
calendar-quick-add-btn = Adicionar
calendar-quick-add-do = Adição rápida

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Categorias
calendar-add-category = Adicionar categoria
calendar-category-placeholder = Adicionar uma categoria
calendar-remove-category = Remover categoria { $name }
calendar-filter-heading = Filtro
calendar-filter-category = Filtrar por categoria
calendar-filter-category-placeholder = Categoria a mostrar

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Anexos
calendar-attachment-title = Nome do anexo
calendar-attachment-title-placeholder = Nome (opcional)
calendar-attachment-uri = Ligação do anexo
calendar-attachment-add = Adicionar anexo
calendar-remove-attachment = Remover anexo { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Subscrever por URL
calendar-subscribe-url = URL do calendário
calendar-subscribe-add = Subscrever
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Não foi possível obter o calendário nesse endereço, pelo que não foi adicionado.
calendar-subscribe-hol-many-locations = Este ficheiro de feriados inclui mais do que um local. Transfira-o e, em seguida, utilize Importar e escolha os locais pretendidos.
calendar-subscribe-refresh = Atualizar { $name }
calendar-subscribe-refresh-btn = Atualizar
calendar-subscribe-refresh-failed = Não foi possível obter o feed do calendário. Os respetivos eventos ficaram como estavam.
calendar-import-failed = Não foi possível ler esse ficheiro como calendário. Nada foi importado.
# The file is an iTIP CANCEL, REPLY or COUNTER: a scheduling message that only
# means something with the e-mail it came in (whose sender it is checked against).
calendar-import-itip-message = Este ficheiro é uma resposta ou um cancelamento enviado por e-mail. Abra a mensagem em que veio para o aplicar. Nada foi importado.
# Appended to calendar-import-done when some events were already there.
calendar-import-skipped = { $count ->
    [one] { $count } evento já estava no calendário e ficou como estava.
   *[other] { $count } eventos já estavam no calendário e ficaram como estavam.
}
calendar-import-done = { $count ->
    [one] 1 evento importado.
   *[other] { $count } eventos importados.
}
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Escolher locais a importar
calendar-import-locations-intro = Este ficheiro contém feriados de { $count } locais. Só são importados os locais que selecionar.
calendar-import-locations-filter = Filtrar locais
calendar-import-locations-list = Locais no ficheiro
calendar-import-locations-none = Nenhum local corresponde ao filtro.
calendar-import-locations-picked = { $count ->
    [one] 1 local selecionado.
   *[other] { $count } locais selecionados.
}
calendar-import-locations-import = Importar selecionados
calendar-export-failed = Não foi possível exportar o calendário.
calendar-quick-add-failed = Não foi possível adicionar esse evento.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Partilhar
calendar-share-for = Partilhar { $name }
calendar-share-title = Partilhar { $name }
calendar-share-intro = As pessoas com quem partilha podem ver este calendário. Dê acesso de leitura e escrita para lhes permitir adicionar e editar eventos.
calendar-share-people = Partilhado com
calendar-share-empty = Este calendário não está partilhado com ninguém.
calendar-share-add = Adicionar uma pessoa
calendar-share-new-access = Acesso da nova pessoa
calendar-share-access-for = Acesso de { $principal }
calendar-share-remove = Remover { $principal }
calendar-share-read = Só leitura
calendar-share-readwrite = Leitura e escrita

# -- Received invitations ---------------------------------------------------
calendar-invite-title = Convite de calendário
calendar-invite-sender = De { $sender }
calendar-invite-add = Adicionar ao calendário
calendar-invite-update = Aplicar atualização do evento
calendar-invite-cancel = Aplicar cancelamento
calendar-invite-reply = Registar resposta do participante
calendar-invite-new = Este convite pode ser adicionado ao seu calendário.
calendar-invite-newer = O organizador enviou um evento atualizado.
calendar-invite-current = O seu calendário já tem esta versão.
calendar-invite-stale = Esta mensagem é mais antiga do que o evento no seu calendário.
calendar-invite-cancelled = O organizador cancelou este evento.
calendar-invite-response = Um participante enviou uma resposta.
calendar-invite-not-organizer = O remetente não é o organizador registado. Nada será alterado.
calendar-invite-not-attendee = O remetente não corresponde a nenhum participante. Nada será alterado.
calendar-invite-unknown = Esta mensagem refere-se a um evento que não está no seu calendário.
calendar-invite-unsupported = Esta proposta não pode ser aplicada automaticamente.
calendar-invite-no-reply = Isto atualiza o seu calendário sem enviar uma resposta.
calendar-invite-applied = O seu calendário foi atualizado.
calendar-invite-unchanged = O evento não foi alterado. Reveja o estado atual do convite.
calendar-invite-apply-failed = Não foi possível aplicar o convite. Tente novamente.
calendar-invite-load-failed = Não foi possível verificar o convite.
calendar-invite-retry = Tentar novamente

calendar-field-time-zone = Fuso horário do evento
calendar-field-time-zone-help = Utilize um nome IANA, como Europe/Lisbon. O evento mantém este fuso quando é visto noutro local.
calendar-time-invalid = Introduza um fuso horário e uma hora local válidos. Algumas horas locais não existem quando os relógios avançam.

calendar-mini-prev = Mês anterior
calendar-mini-next = Mês seguinte

## t53 — o cartão de evento ao estilo Outlook no leitor e na pré-visualização de .ics
calendar-card-region-event = Evento de calendário
calendar-card-kind-request = Convite para reunião
calendar-card-kind-update = Convite atualizado
calendar-card-kind-cancel = Reunião cancelada
calendar-card-kind-reply = Resposta a reunião
calendar-card-kind-counter = Nova hora proposta
calendar-card-kind-publish = { $count ->
    [one] Evento de calendário
   *[other] { $count } eventos de calendário
}
calendar-card-when = Quando
calendar-card-where = Onde
calendar-card-organizer = Organizador
calendar-card-attendees = Participantes
calendar-card-all-day = { $when } (todo o dia)
calendar-card-organizer-time = Hora do organizador: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Todos os dias
   *[other] De { $interval } em { $interval } dias
}
calendar-card-repeat-weekly = { $interval ->
    [one] Todas as semanas
   *[other] De { $interval } em { $interval } semanas
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] Todas as semanas: { $days }
   *[other] De { $interval } em { $interval } semanas: { $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] Todos os meses
   *[other] De { $interval } em { $interval } meses
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] Todos os meses, no dia { $day }
   *[other] De { $interval } em { $interval } meses, no dia { $day }
}
calendar-card-repeat-yearly = { $interval ->
    [one] Todos os anos
   *[other] De { $interval } em { $interval } anos
}
calendar-card-repeat-until = { $rule }, até { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] uma vez
   *[other] { $count } vezes
}
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 data excluída
   *[other] { $count } datas excluídas
})
calendar-card-repeat-custom = Repete-se com uma periodicidade personalizada
calendar-card-join = Entrar no { $provider }
calendar-card-organizer-is = Organizado por { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 participante
   *[other] { $count } participantes
} · { $accepted } aceitaram, { $tentative } provisórios, { $declined } recusaram
calendar-card-people-all = Mostrar todos ({ $count })
calendar-card-people-fewer = Mostrar menos
calendar-card-rsvp-accepted = Aceite
calendar-card-rsvp-tentative = Provisório
calendar-card-rsvp-declined = Recusado
calendar-card-rsvp-none = Ainda sem resposta
calendar-card-in-calendar = No seu calendário
calendar-card-cancelled = Cancelado
calendar-card-open = Abrir no calendário
calendar-card-agenda = O seu calendário em { $day }
calendar-card-agenda-title = O seu dia
calendar-card-agenda-this-event = Este evento
calendar-card-agenda-unavailable = Não foi possível carregar o seu calendário para este dia.
calendar-card-conflicts = { $count ->
    [one] Em conflito com { $titles }
   *[other] Em conflito com { $count } eventos: { $titles }
}
calendar-card-no-conflicts = Sem conflitos no seu calendário
calendar-card-invited-as = Foi convidado como { $address }.
calendar-card-not-invited = Este convite não inclui nenhum dos seus endereços. Pode adicioná-lo ao calendário, mas não pode responder.
calendar-card-not-invited-short = Este convite não inclui nenhum dos seus endereços.
calendar-card-out-of-date = Este convite está desatualizado. Já existe uma versão mais recente no seu calendário.
calendar-card-replied = { $who } { $answer ->
    [accepted] aceitou.
    [tentative] aceitou provisoriamente.
    [declined] recusou.
   *[other] ainda não respondeu.
}
calendar-card-counter = { $who } propôs uma nova hora. As propostas não são aplicadas automaticamente.
calendar-card-todos-skipped = { $count ->
    [one] O ficheiro também contém 1 tarefa, que não é adicionada.
   *[other] O ficheiro também contém { $count } tarefas, que não são adicionadas.
}
calendar-card-respond = Responder ao convite
calendar-card-accept = Aceitar
calendar-card-tentative = Provisório
calendar-card-decline = Recusar
calendar-card-propose = Propor nova hora
calendar-card-propose-start = Novo início
calendar-card-propose-send = Enviar proposta
calendar-card-propose-invalid = Introduza uma nova hora de início válida.
calendar-card-send-response = Enviar uma resposta a { $organizer }
calendar-card-comment = Comentário (opcional)
calendar-card-calendar = Calendário
calendar-card-events = Eventos neste ficheiro
calendar-card-add-one = Adicionar
calendar-card-add-all = Adicionar todos ({ $count })
calendar-card-added = { $count ->
    [one] Adicionado ao seu calendário.
   *[other] { $count } eventos adicionados ao seu calendário.
}
calendar-card-remove = Remover do calendário
calendar-card-removed = Removido do seu calendário.
calendar-card-record-reply = Atualizar o estado do participante
calendar-card-answered-sent = { $answer }. A sua resposta foi enviada a { $organizer }.
calendar-card-answered-not-sent = { $answer }. O seu calendário foi atualizado, mas não foi possível enviar a resposta.
calendar-card-answered-silently = { $answer }. Não foi enviada nenhuma resposta.
calendar-card-proposal-sent = A sua proposta foi enviada a { $organizer }.
calendar-card-not-sent = Não foi possível enviar a mensagem.
calendar-card-more = { $count ->
    [one] mais 1
   *[other] mais { $count }
}
