# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Retenção
retention-intro = Regras que movem ou excluem seus e-mails depois de um número de dias. Uma regra pode considerar uma pasta, um marcador, um remetente, um tamanho, se a mensagem foi lida ou se tem anexo. E-mails sinalizados e com o marcador keep são deixados em paz, a menos que você mude isso. Os e-mails excluídos vão primeiro para a Lixeira e são apagados definitivamente após o prazo de desfazer.
retention-disabled = As regras de retenção estão desativadas neste servidor. Suas regras existentes não são executadas.
retention-never = nunca
retention-error = Não foi possível carregar ou salvar as configurações de retenção. Tente novamente.
retention-last-run = Última execução: { $when }
retention-next-run = próxima execução em cerca de { $days } dia(s)
retention-next-run-unknown = próxima execução ainda não agendada
retention-enabled = Executar minhas regras de retenção
retention-trash-first = Mover e-mails excluídos para a Lixeira primeiro
retention-trash-forced = Este servidor sempre move os e-mails para a Lixeira primeiro e os apaga definitivamente após o prazo de desfazer.
retention-undo = Prazo de desfazer em dias
retention-undo-help = Os e-mails que uma regra moveu para a Lixeira ficam lá por esse tempo antes de serem apagados definitivamente. De 0 a { $max } dias.
retention-pause = Pausar todas as regras por 30 dias
retention-paused-until = Pausadas até { $when }
retention-resume = Retomar
retention-policy = O que este servidor permite
retention-limits-both = Uma regra que exclui e-mails espera no mínimo { $min } dias e no máximo { $max } dias.
retention-limits-min = Uma regra que exclui e-mails espera no mínimo { $min } dias.
retention-max-rules = Você pode manter até { $count } regras.
retention-permanent-allowed = Uma regra pode excluir e-mails permanentemente se você desativar a etapa da Lixeira.
retention-permanent-denied = As regras não podem excluir e-mails permanentemente. Os e-mails vão para a Lixeira e são apagados após { $days } dias.
retention-protected = Pastas que as regras nunca tocam: { $folders }.
retention-mandatory = Definido pelo administrador: e-mails em { $folder } com mais de { $days } dias são excluídos.
retention-rules = Suas regras
retention-add = Nova regra
retention-no-rules = Você ainda não tem regras. Adote um modelo abaixo ou adicione uma.
retention-rule-paused = pausada
retention-rule-on = Ativada
retention-rule-enabled = A regra { $name } está ativada
retention-edit-short = Editar
retention-delete = Excluir
retention-new = Nova regra
retention-edit = Editar regra
retention-name = Nome
retention-action = O que fazer
retention-action-keep = Manter para sempre
retention-action-delete = Excluir
retention-action-trash = Mover para a Lixeira
retention-action-archive = Mover para o Arquivo
retention-action-move = Mover para uma pasta
retention-action-markRead = Marcar como lida
retention-days = Após dias
retention-target = Pasta
retention-choose-folder = Escolha uma pasta
retention-scope = Quais e-mails
retention-folder = Pasta
retention-any-folder = Qualquer pasta
retention-subfolders = Incluir subpastas
retention-labels = Marcadores (separados por vírgula)
retention-senders = Endereços de remetentes (separados por vírgula)
retention-domains = Domínios de remetentes (separados por vírgula)
retention-min-size = Tamanho mínimo em KB
retention-max-size = Tamanho máximo em KB
retention-read = Estado de leitura
retention-read-any = Lidas ou não lidas
retention-read-read = Somente lidas
retention-read-unread = Somente não lidas
retention-attachment = Anexos
retention-attachment-any = Com ou sem
retention-attachment-with = Com anexo
retention-attachment-without = Sem anexo
retention-exclude = Deixar em paz
retention-exclude-flagged = E-mails sinalizados (com estrela)
retention-exclude-unread = E-mails não lidos
retention-exclude-tags = E-mails com estes marcadores (separados por vírgula)
retention-exclude-note = E-mails sob retenção legal nunca são tocados, seja qual for a configuração aqui.
retention-role-inbox = Caixa de entrada
retention-role-sent = Enviados
retention-role-archive = Arquivo
retention-role-junk = Spam
retention-role-trash = Lixeira
retention-role-drafts = Rascunhos
retention-btn-preview = Pré-visualizar
retention-save = Salvar
retention-cancel = Cancelar
retention-preview-none = Nenhuma mensagem seria alterada.
retention-preview-count = Isso alteraria { $count } mensagem(ns).
retention-preview-truncated = Apenas parte da caixa de correio foi lida para esta pré-visualização.
retention-preview-sample = Exemplos
retention-no-subject = (sem assunto)
retention-do-trash = para a Lixeira
retention-do-move = movidas
retention-do-delete = excluídas
retention-do-markRead = marcadas como lidas
retention-skipped-legalHold = Sua conta está sob retenção legal. Nada é alterado.
retention-skipped-restricted = O processamento da sua conta está restrito. Nada é alterado.
retention-skipped-noRules = Você não tem regras ativas e o administrador não definiu nenhuma.
retention-error-name = Dê um nome à regra.
retention-error-days = Os dias devem ser um número inteiro de 0 a 3650.
retention-error-min = Uma regra que exclui e-mails deve esperar no mínimo { $days } dias.
retention-error-max = Uma regra que exclui e-mails pode esperar no máximo { $days } dias.
retention-error-target = Escolha a pasta para onde mover os e-mails.
retention-summary-keep = manter para sempre
retention-summary-delete = após { $days } dias, para a Lixeira, depois apagado
retention-summary-delete-permanent = após { $days } dias, excluído definitivamente
retention-summary-trash = após { $days } dias, para a Lixeira
retention-summary-archive = após { $days } dias, para o Arquivo
retention-summary-move = após { $days } dias, para uma pasta
retention-summary-markRead = após { $days } dias, marcado como lido
retention-templates = Modelos
retention-templates-help = Um modelo adiciona uma regra pronta. Confira os marcadores e os dias antes de confiar nela.
retention-adopt = Adicionar esta regra
retention-run-heading = Executar agora
retention-preview-all = Pré-visualizar todas as regras
retention-btn-run = Executar minhas regras agora
retention-run-result = { $count } mensagem(ns) alterada(s).
retention-history = Execuções recentes
retention-history-when = Quando
retention-history-matched = Correspondentes
retention-history-changed = Alteradas
retention-history-errors = Erros
retention-context-action = Retenção…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Calendários

pim-cal-days = Dias após o término de um evento

pim-cal-enabled = Excluir eventos encerrados

pim-cal-help = Exclua eventos nos calendários que você escolher um determinado número de dias após seu término. Eventos recorrentes nunca são excluídos.

pim-cal-title = Eventos do calendário

pim-days-range = Escolha entre { $min } e { $max } dias.

pim-held-title = Notas removidas que você pode restaurar

pim-held-until = Mantido até { $when }

pim-limits = Entre { $min } e { $max } dias.

pim-notes-days = Dias desde que uma nota foi alterada pela última vez

pim-notes-enabled = Remover notas intocadas

pim-notes-help = Remova notas que você não alterou por um determinado número de dias. As notas fixadas são mantidas. Uma nota removida pode ser restaurada por { $days } dias.

pim-notes-title = Notas

pim-off = Desativado pelo seu administrador.

pim-preview = Visualização

pim-restore = Restaurar

pim-result = { $events } evento(s) e { $notes } nota(s).

pim-run = Corra agora

pim-run-title = Corra agora

pim-save = Salvar

pim-skipped = { $count } opções foram ignoradas porque seu administrador as desativou.

pim-untitled = Nota sem título

retention-action-stripAttachments = Remover anexos grandes

retention-do-stripAttachments = anexos removidos

retention-error-strip-disabled = A remoção de anexos foi desativada pelo seu administrador.

retention-error-strip-min = Uma regra que remove anexos deve esperar pelo menos { $days } dias.

retention-fd-action = O que acontece com o correio em { $folder }

retention-fd-days = Dias antes de { $folder } ser processado

retention-fd-help = Defina quanto tempo o e-mail permanece em uma pasta antes de ser movido ou excluído. Uma pasta definida como "Herdar" não possui um padrão próprio; as regras acima e os limites do administrador ainda se aplicam. As subpastas seguem suas pastas.

retention-fd-inherit = Herdar

retention-fd-mandatory = O administrador mantém esta pasta por { $days } dias

retention-fd-protected = Protegido pelo administrador

retention-fd-save = Salvar padrões de pasta

retention-fd-title = Padrões de pasta

retention-rule-strip-off = Desativado pelo seu administrador

retention-strip-allowed = Uma regra pode remover anexos de pelo menos { $kb } KB de e-mails com pelo menos { $days } dias. Uma breve nota substitui cada anexo.

retention-strip-denied = A remoção de anexos está desativada pelo administrador.

retention-strip-done = Anexos removidos de { $count } mensagem(s), liberando { $size }.

retention-strip-inline-kept = As imagens mostradas dentro da mensagem são mantidas.

retention-strip-inline-removed = As imagens mostradas dentro da mensagem também são removidas.

retention-strip-limits = Remove anexos de pelo menos { $kb } KB de e-mails com pelo menos { $days } dias. O original permanece na Lixeira por { $undo } dias.

retention-strip-preview = { $count } mensagens perderiam anexos, liberando { $size }.

retention-strip-preview-estimate = Cerca de { $count } mensagens perderiam anexos, liberando até { $size }.

retention-strip-skipped = { $count } regras que removem anexos foram ignoradas porque seu administrador desativou esta opção.

retention-summary-stripAttachments = após { $days } dias, anexos grandes removidos
