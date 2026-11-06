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
