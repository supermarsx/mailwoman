# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooks
admin-webhooks-intro = Um webhook de saída envia um aviso assinado a um URL quando algo acontece em uma conta. Um webhook de entrada permite que outro sistema execute uma ação de regra de e-mail em uma conta.
admin-webhooks-empty = Nenhum webhook ainda.
admin-webhooks-unavailable = Este servidor não oferece o gerenciamento de webhooks.
admin-webhooks-load-error = Não foi possível carregar os webhooks.
admin-webhooks-direction-outbound = Saída
admin-webhooks-direction-inbound = Entrada
admin-webhooks-col-direction = Direção
admin-webhooks-col-label = Nome
admin-webhooks-col-account = Conta
admin-webhooks-col-target = Envia para ou permite
admin-webhooks-col-created = Criado
admin-webhooks-no-label = (sem nome)
admin-webhooks-created-by = por { $who }
admin-webhooks-last-used = Último uso { $when }
admin-webhooks-never-used = Ainda não usado
admin-webhooks-delete = Excluir
admin-webhooks-delete-named = Excluir o webhook { $name }
admin-webhooks-delete-title = Excluir o webhook { $name }?
admin-webhooks-delete-body = As entregas a ele param, ou as chamadas assinadas com o segredo dele são recusadas. Isso não pode ser desfeito; um novo webhook recebe um novo segredo.
admin-webhooks-deleted = Webhook excluído.
admin-webhooks-add = Adicionar um webhook
admin-webhooks-add-direction = Direção
admin-webhooks-add-outbound-help = O Mailwoman chama o seu URL.
admin-webhooks-add-inbound-help = O seu sistema chama o Mailwoman.
admin-webhooks-add-account = Conta
admin-webhooks-add-account-help = A conta cujos eventos são enviados, ou a única conta em que uma chamada de entrada pode agir.
admin-webhooks-add-label = Nome
admin-webhooks-add-label-help = Para que serve o webhook, para que os outros saibam. Opcional.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Deve alcançar um endereço público da internet. Endereços privados, de loopback e link-local são recusados.
admin-webhooks-add-events = Eventos
admin-webhooks-add-actions = Ações permitidas
admin-webhooks-add-submit = Criar webhook
admin-webhooks-add-saving = Criando…
admin-webhooks-add-check = Verificar
admin-webhooks-add-check-ok = O URL passou na verificação. Nada foi salvo.
admin-webhooks-event-message-received = Nova mensagem
admin-webhooks-event-message-flags = Mensagem lida, sinalizada ou marcada
admin-webhooks-event-message-moved = Mensagem movida
admin-webhooks-event-message-deleted = Mensagem excluída
admin-webhooks-event-submission-sent = Mensagem enviada
admin-webhooks-event-submission-failed = Não foi possível enviar a mensagem
admin-webhooks-event-pim-changed = Agenda, contatos, tarefas ou notas alterados
admin-webhooks-action-run-rules = Executar as regras de e-mail da conta em uma descrição de mensagem
admin-webhooks-action-sieve = Aplicar uma ação de regra a uma mensagem
admin-webhooks-secret-title = Segredo de { $name }
admin-webhooks-secret-once = Copie este segredo agora. Ele não será exibido novamente.
admin-webhooks-secret-outbound = Cada entrega carrega X-Mailwoman-Signature: sha256= seguido do HMAC-SHA256 do corpo da solicitação com este segredo.
admin-webhooks-secret-inbound = Envie X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (segundos Unix), X-Mailwoman-Nonce (16 a 128 letras, dígitos, - ou _, nunca reutilizado) e X-Mailwoman-Signature: sha256= seguido do HMAC-SHA256 de timestamp.nonce.body com este segredo. As chamadas com mais de cinco minutos de diferença, ou que reutilizem um nonce, são recusadas.
admin-webhooks-secret-done = Já copiei
admin-webhooks-error-stale = Alguém alterou os webhooks desde que você abriu esta página. A lista foi recarregada; tente novamente.
admin-webhooks-error-account = Informe a conta.
admin-webhooks-error-url = Informe um URL que comece com https:// ou http://.
admin-webhooks-error-events = Escolha pelo menos um evento.
admin-webhooks-error-actions = Escolha pelo menos uma ação.
