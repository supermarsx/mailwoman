# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
send-delay-title = Anular envio
send-delay-help = Depois de premir Enviar, a mensagem espera na Caixa de saída durante este tempo. Até lá pode recuperá-la e continuar a editar. O servidor guarda-a, por isso fechar o navegador não a envia antes de tempo nem a perde.
send-delay-label = Reter as mensagens enviadas durante
send-delay-off = Desativado (enviar de imediato)
send-delay-seconds = { $n } segundos
send-delay-minutes = { $n ->
    [one] 1 minuto
   *[other] { $n } minutos
}
send-delay-custom = Personalizado…
send-delay-custom-label = Atraso personalizado em segundos
send-delay-custom-help = Segundos inteiros, de 0 a { $max }.
send-delay-capped = O seu administrador limita este valor a { $max }.
send-delay-disabled = A recuperação de mensagens enviadas está desativada pelo administrador. As mensagens são enviadas de imediato.
send-delay-invalid = Introduza um número inteiro de segundos entre 0 e { $max }.

send-later-button = Enviar mais tarde
send-later-menu = Opções de envio posterior
send-later-preset-hour = Dentro de 1 hora
send-later-preset-today = Hoje à tarde, 17:00
send-later-preset-tomorrow = Amanhã de manhã, 09:00
send-later-preset-monday = Segunda-feira de manhã, 09:00
send-later-custom = Escolher data e hora…
send-later-custom-label = Enviar em
send-later-zone = Fuso horário: { $zone }
send-later-schedule = Agendar envio
send-later-cancel = Cancelar
send-later-err-invalid = Introduza uma data e hora válidas.
send-later-err-past = Escolha uma hora no futuro.
send-later-err-horizon = Demasiado longe. As mensagens podem ser agendadas até { $days } dias de antecedência.
send-later-err-off = O envio agendado está desativado pelo administrador.

send-queued = Mensagem em fila — envia dentro de
send-queued-undo = Anular
send-queued-now = Enviar agora
send-queued-recalled = Mensagem recuperada. Está de volta aos rascunhos.
send-queued-too-late = Tarde demais para recuperar: a mensagem já está a ser enviada.
send-queued-sent-now = A enviar agora.
send-queued-scheduled = Agendada para { $when } ({ $zone }).

outbox-scheduled-badge = Agendada
outbox-sends-in = Envia dentro de { $time }
outbox-sends-at = Envia { $when }
outbox-recall-edit = Recuperar e editar
outbox-recall-too-late = Já está a ser enviada; já não pode ser recuperada.
outbox-failed-drafts = Não foi possível enviar; foi mantida nos rascunhos.
outbox-queued-count = { $count ->
    [one] 1 mensagem à espera de envio
   *[other] { $count } mensagens à espera de envio
}

## Leitor da Caixa de saída, editar hora de envio, indicador de mensagens em fila
outbox-open = Abrir
outbox-reader-label = Mensagem em fila
outbox-reader-actions = Ações da mensagem em fila
outbox-reader-loading = A carregar a mensagem…
outbox-reader-unavailable = Não foi possível carregar a mensagem.
outbox-reader-to = Para: { $recipients }
outbox-reader-cc = Cc: { $recipients }
outbox-status-held = Retida até a libertar
outbox-status-queued = Em fila
outbox-status-scheduled = Agendada
outbox-status-sending = A enviar
outbox-status-retrying = A tentar novamente
outbox-status-failed = Não enviada
outbox-status-uncertain = Entrega desconhecida
outbox-status-sent = Enviada
outbox-status-canceled = Cancelada
outbox-status-sends-at-in = Envio { $when } (daqui a { $time })
outbox-status-next-attempt = Próxima tentativa { $when }
outbox-status-attempts = { $count ->
    [one] 1 tentativa até agora
   *[other] { $count } tentativas até agora
}
outbox-status-last-error = Último erro: { $error }
outbox-edit-time = Editar hora de envio
outbox-edit-time-current = Atualmente { $when }
outbox-edit-time-save = Guardar hora de envio
outbox-edit-time-conflict = A hora de envio mudou ou a mensagem já foi enviada. A Caixa de saída foi recarregada.
outbox-edit-time-refused = Não foi possível alterar a hora de envio. { $error }
outbox-recall-to-drafts = Recuperar para rascunhos
outbox-send-again = Enviar novamente
outbox-send-again-queued = A mensagem ficou em fila para ser enviada novamente.
outbox-send-again-failed = Não foi possível voltar a pôr a mensagem em fila. { $error }
outbox-delete = Eliminar
outbox-delete-confirm = Eliminar mensagem?
outbox-deleted = A mensagem em fila foi eliminada.
outbox-delete-failed = Não foi possível eliminar a mensagem. { $error }
outbox-failed-toast = Não foi possível enviar uma mensagem. { $error }
outbox-failed-uncertain-toast = Uma mensagem pode ou não ter sido entregue. Verifique os Enviados antes de a enviar novamente.
outbox-chip-title = Mensagens a aguardar envio
outbox-chip-next = Próximo envio: { $when }
outbox-chip-view-all = Abrir Caixa de saída
