# Machine translation (t39, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-features. Source: ../en/admin-features.ftl

## Admin › Perfis de funcionalidades (t39 N9)

admin-nav-features = Perfis de funcionalidades
admin-features-title = Perfis de funcionalidades
admin-features-intro = Um perfil é um conjunto com nome de interruptores de funcionalidades. Atribua-o a um domínio, a um grupo ou a um utilizador; passa então a aplicar-se às contas nativas aí. As contas com sessão iniciada através de um servidor upstream em proxy seguem a política desse servidor.
admin-features-new = Novo perfil
admin-features-search = Pesquisar perfis
admin-features-empty = Ainda não existem perfis de funcionalidades.
admin-features-no-match = Nenhum perfil corresponde à pesquisa.
admin-features-load-error = Não foi possível carregar o catálogo de funcionalidades.
admin-features-not-loaded = O catálogo de funcionalidades ainda não foi carregado. Recarregue a página e tente novamente.
admin-features-stale = Outro administrador alterou o catálogo de funcionalidades. Foi recarregado; verifique os valores e guarde novamente.
admin-features-not-found = Não existe nenhum perfil com o id { $id }.
admin-features-col-name = Nome
admin-features-col-values = Interruptores
admin-features-col-assigned = Atribuído a
admin-features-values-count = { $count ->
    [one] { $count } interruptor definido
   *[other] { $count } interruptores definidos
}
admin-features-assigned-count = { $count ->
    [0] Não atribuído
    [one] { $count } local
   *[other] { $count } locais
}
admin-features-editor = Definições do perfil
admin-features-name = Nome do perfil
admin-features-name-help = Visível apenas para administradores, até 128 caracteres.
admin-features-name-invalid = Introduza um nome com 1 a 128 caracteres.
admin-features-switches = Interruptores de funcionalidades
admin-features-switches-help = "Não definido" deixa a funcionalidade para o perfil seguinte, para o domínio ou para a predefinição do servidor. Um valor bloqueado não pode ser alterado pelo utilizador nem por um nível inferior.
admin-features-col-feature = Funcionalidade
admin-features-col-value = Valor
admin-features-col-lock = Bloqueio
admin-features-value-for = Valor para { $feature }
admin-features-unset = Não definido
admin-features-on = Permitido
admin-features-off = Não permitido
admin-features-lock = Bloqueado
admin-features-save = Guardar perfil
admin-features-create = Criar perfil
admin-features-delete = Eliminar perfil
admin-features-delete-title = Eliminar o perfil { $name }?
admin-features-delete-body = { $count ->
    [0] O perfil não está atribuído a nenhum local. Os seus interruptores deixam de se aplicar de imediato.
    [one] O perfil é removido do { $count } local a que está atribuído, e os seus interruptores deixam de se aplicar de imediato.
   *[other] O perfil é removido dos { $count } locais a que está atribuído, e os seus interruptores deixam de se aplicar de imediato.
}
admin-features-saved = Perfil { $name } guardado.
admin-features-deleted = Perfil { $name } eliminado.
admin-features-assigned = Perfil { $name } atribuído.
admin-features-unassigned = Atribuição de { $name } removida.
admin-features-assignments = Atribuído a
admin-features-assignments-help = Os perfis aplicam-se por ordem de prioridade, a mais alta primeiro. Com a mesma prioridade, uma atribuição a um utilizador aplica-se antes de um grupo, e um grupo antes de um domínio.
admin-features-assignments-empty = Este perfil ainda não está atribuído.
admin-features-assign-form = Atribuir este perfil
admin-features-col-scope = Aplica-se a
admin-features-col-target = Nome
admin-features-col-priority = Prioridade
admin-features-col-actions = Ações
admin-features-scope-domain = Domínio
admin-features-scope-group = Grupo
admin-features-scope-user = Utilizador
admin-features-assign = Atribuir
admin-features-unassign = Remover
admin-features-no-domain = Não há domínios configurados.
admin-features-no-group = Não existem grupos de utilizadores.
admin-features-no-user = Ainda nenhuma conta nativa iniciou sessão.

## Admin › Observabilidade › Retenção do registo de auditoria (t39 L5)

admin-retention-title = Retenção do registo de auditoria
admin-retention-intro = Durante quanto tempo as entradas de auditoria são mantidas, por categoria. Uma categoria sem valor próprio segue a predefinição; sem predefinição, todas as entradas são mantidas. Uma retenção legal impede a limpeza da sua categoria, e uma retenção na predefinição impede-a para todas. As alterações e as limpezas ficam registadas no registo de auditoria.
admin-retention-loading = A carregar a política de retenção…
admin-retention-unavailable = A retenção da auditoria não está disponível neste servidor.
admin-retention-col-category = Categoria
admin-retention-col-kept = Mantido
admin-retention-col-change = Alterar
admin-retention-cat-default = Predefinição
admin-retention-cat-admin = Ações de administradores
admin-retention-cat-security = Eventos de segurança
admin-retention-cat-alert = Alertas de utilizadores
admin-retention-forever = Todas as entradas
admin-retention-days = { $days ->
    [one] { $days } dia
   *[other] { $days } dias
}
admin-retention-follows-default = Predefinição: { $kept }
admin-retention-hold-on = Retenção legal
admin-retention-read-only = Alterar a retenção requer a permissão settings.write.
admin-retention-edit-for = Retenção para { $category }
admin-retention-keep = Manter
admin-retention-mode-inherit = Seguir a predefinição
admin-retention-mode-forever = Todas as entradas
admin-retention-mode-days = Durante um número de dias
admin-retention-days-label = Dias
admin-retention-hold = Retenção legal
admin-retention-save = Guardar
admin-retention-bounds = Introduza um número inteiro de { $min } a { $max }.
admin-retention-saved = Retenção para { $category } guardada.
admin-retention-task-none = Ainda não existe tarefa de limpeza. É criada, diariamente às 03:15 UTC, quando se guarda pela primeira vez um número de dias.
admin-retention-task-disabled = A tarefa de limpeza agendada está desativada; as entradas só são limpas com Limpar agora.
admin-retention-task-next = Próxima limpeza agendada: { $when }.
admin-retention-task-daily = Limpo diariamente às 03:15 UTC.
admin-retention-last = Última limpeza que removeu entradas: { $when }.
admin-retention-never = Ainda não foram limpas entradas.
admin-retention-prune = Limpar agora
admin-retention-prune-title = Limpar o registo de auditoria agora?
admin-retention-prune-body = As entradas mais antigas do que a retenção da sua categoria são eliminadas. As categorias que mantêm todas as entradas ou estão em retenção legal não são alteradas. Esta ação não pode ser anulada.
admin-retention-pruned = { $count ->
    [0] Nenhuma entrada era suficientemente antiga para ser removida.
    [one] { $count } entrada removida.
   *[other] { $count } entradas removidas.
}

## Admin › Registos › Autenticação (t39 L5)

admin-auth-attempted = Tentativa como { $name }
