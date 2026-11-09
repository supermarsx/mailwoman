# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Desfazer envio
send-delay-help = Depois que você pressiona Enviar, a mensagem espera na Caixa de saída por este tempo. Até lá, você pode recuperá-la e continuar editando. O servidor a mantém, então fechar o navegador não a envia antes da hora nem a perde.
send-delay-label = Reter mensagens enviadas por
send-delay-off = Desativado (enviar imediatamente)
send-delay-seconds = { $n } segundos
send-delay-minutes = { $n ->
    [one] 1 minuto
   *[other] { $n } minutos
}
send-delay-custom = Personalizado…
send-delay-custom-label = Atraso personalizado em segundos
send-delay-custom-help = Segundos inteiros, de 0 a { $max }.
send-delay-capped = O administrador limita isto a { $max }.
send-delay-disabled = A recuperação de mensagens enviadas está desativada pelo administrador. As mensagens são enviadas imediatamente.
send-delay-invalid = Informe um número inteiro de segundos de 0 a { $max }.
send-later-button = Enviar mais tarde
send-later-menu = Opções de envio posterior
send-later-preset-hour = Em 1 hora
send-later-preset-today = Mais tarde hoje, 17:00
send-later-preset-tomorrow = Amanhã de manhã, 09:00
send-later-preset-monday = Segunda-feira de manhã, 09:00
send-later-custom = Escolher data e hora…
send-later-custom-label = Enviar em
send-later-zone = Fuso horário: { $zone }
send-later-schedule = Agendar envio
send-later-cancel = Cancelar
send-later-err-invalid = Informe uma data e uma hora válidas.
send-later-err-past = Escolha um horário no futuro.
send-later-err-horizon = Isso é longe demais. As mensagens podem ser agendadas com até { $days } dias de antecedência.
send-later-err-off = O envio agendado está desativado pelo administrador.
send-queued = Mensagem na fila — será enviada em
send-queued-undo = Desfazer
send-queued-now = Enviar agora
send-queued-recalled = Mensagem recuperada. Ela voltou para seus rascunhos.
send-queued-too-late = Tarde demais para recuperar: a mensagem já está sendo enviada.
send-queued-sent-now = Enviando agora.
send-queued-scheduled = Agendada para { $when } ({ $zone }).
outbox-scheduled-badge = Agendada
outbox-sends-in = Será enviada em { $time }
outbox-sends-at = Será enviada { $when }
outbox-recall-edit = Recuperar e editar
outbox-recall-too-late = Já está sendo enviada; não pode mais ser recuperada.
outbox-failed-drafts = Não foi possível enviar e a mensagem foi mantida nos seus rascunhos.
outbox-queued-count = { $count ->
    [one] 1 mensagem aguardando envio
   *[other] { $count } mensagens aguardando envio
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = Próximo envio: { $when }

outbox-chip-title = Mensagens esperando para serem enviadas

outbox-chip-view-all = Abrir caixa de saída

outbox-delete = Excluir

outbox-delete-confirm = Excluir mensagem?

outbox-delete-failed = A mensagem não pôde ser excluída. { $error }

outbox-deleted = A mensagem na fila foi excluída.

outbox-edit-time = Editar horário de envio

outbox-edit-time-conflict = O horário de envio mudou ou a mensagem já foi enviada. A caixa de saída foi recarregada.

outbox-edit-time-current = Atualmente { $when }

outbox-edit-time-refused = A hora de envio não pôde ser alterada. { $error }

outbox-edit-time-save = Economize tempo de envio

outbox-failed-toast = Não foi possível enviar uma mensagem. { $error }

outbox-failed-uncertain-toast = Uma mensagem pode ou não ter sido entregue. Marque Enviado antes de enviá-lo novamente.

outbox-open = Abrir

outbox-reader-actions = Ações de mensagens na fila

outbox-reader-cc = CC: { $recipients }

outbox-reader-label = Mensagem na fila

outbox-reader-loading = Carregando a mensagem…

outbox-reader-to = Para: { $recipients }

outbox-reader-unavailable = A mensagem não pôde ser carregada.

outbox-recall-to-drafts = Lembre-se dos rascunhos

outbox-send-again = Enviar novamente

outbox-send-again-failed = Não foi possível colocar a mensagem na fila novamente. { $error }

outbox-send-again-queued = A mensagem está na fila para ser enviada novamente.

outbox-sent-copy-folder = Será salvo em { $folder }

outbox-sent-copy-missing = Será salvo em Enviados (a pasta escolhida não está disponível)

outbox-sent-copy-skip = Nenhuma cópia será salva

outbox-status-attempts = { $count ->
        [0] { $count } tentativas até agora
        [one] { $count } tentativa até agora
        [many] { $count } tentativas até agora
       *[other] { $count } tentativas até agora
    }

outbox-status-canceled = Cancelado

outbox-status-failed = Não enviado

outbox-status-held = Mantido até você soltá-lo

outbox-status-last-error = Último erro: { $error }

outbox-status-next-attempt = Próxima tentativa { $when }

outbox-status-queued = Na fila

outbox-status-retrying = Tentando novamente

outbox-status-scheduled = Agendado

outbox-status-sending = Enviando

outbox-status-sends-at-in = Envia { $when } (em { $time })

outbox-status-sent = Enviado

outbox-status-uncertain = Entrega desconhecida
