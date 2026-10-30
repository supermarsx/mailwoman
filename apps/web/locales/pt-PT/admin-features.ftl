# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-features. Source: ../en/admin-features.ftl

## Admin › Perfis de funcionalidades (t39 N9)

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

## t98: grouped editor, presets, import and export, effective preview
admin-features-tools = Ferramentas do perfil
admin-features-preset = Começar a partir de um modelo
admin-features-preset-choose = Escolha um modelo
admin-features-preset-apply = Aplicar modelo
admin-features-preset-applied = Modelo { $name } aplicado. Reveja os valores e guarde.
admin-features-preset-full = Completo
admin-features-preset-standard = Padrão
admin-features-preset-mail-only = Só correio
admin-features-preset-minimal = Mínimo / quiosque
admin-features-import = Importar JSON
admin-features-export = Exportar JSON
admin-features-exported = Perfil exportado como ficheiro JSON.
admin-features-imported = { $count ->
    [one] Importado { $count } valor. Reveja-o e guarde.
   *[other] Importados { $count } valores. Reveja-os e guarde.
}
admin-features-import-json = O ficheiro não é JSON válido.
admin-features-import-format = O ficheiro não é um perfil de funcionalidades do Mailwoman.
admin-features-import-version = Este ficheiro de perfil vem de uma versão mais recente e não pode ser importado.
admin-features-clone = Duplicar
admin-features-clone-name = Cópia de { $name }
admin-features-filter = Procurar uma funcionalidade
admin-features-filter-placeholder = Nome, definição ou chave
admin-features-filter-none = Nenhuma funcionalidade corresponde à pesquisa.
admin-features-diff-summary = { $differs ->
    [one] { $differs } valor difere da predefinição do servidor
   *[other] { $differs } valores diferem da predefinição do servidor
} · { $same } repetem-na
admin-features-differs = Difere da predefinição
admin-features-default-is = Predefinição: { $value }
admin-features-fix-values = Corrija os valores marcados como inválidos antes de guardar.
admin-features-allow-all = Permitir tudo
admin-features-deny-all = Não permitir nada
admin-features-clear-all = Limpar
admin-features-group-allow = Permitir todas as funcionalidades em { $group }
admin-features-group-deny = Não permitir nenhuma funcionalidade em { $group }
admin-features-group-clear = Limpar todos os valores em { $group }
admin-features-enforcement-ui = Só escondido na aplicação
admin-features-enforcement-partial = Alguns limites ainda não são contados
admin-features-other = Outras definições que um perfil pode ter
admin-features-other-help = Todas as outras definições que um administrador pode definir por perfil. Deixe um campo vazio para manter o valor do servidor.
admin-features-cat-compose = Correio e composição
admin-features-cat-reading = Leitura e privacidade
admin-features-cat-views = Vistas e pesquisa
admin-features-cat-organise = Organizar
admin-features-cat-pim = Calendário, contactos, tarefas, notas e chaves
admin-features-cat-data = Dados
admin-features-cat-accounts = Contas e acesso
admin-features-cat-integrations = Integrações
admin-features-cat-developer = Acesso para programadores
admin-features-cat-personal = Personalização
admin-features-feature-compose-sending = Limites de envio
admin-features-feature-compose-sending-help = Acima do limite de destinatários por mensagem o servidor recusa o envio. As mensagens por hora e por dia ficam guardadas no perfil mas ainda não são contadas.
admin-features-feature-storage = Quota de armazenamento
admin-features-feature-storage-help = O limite de armazenamento fica guardado no perfil; a quota do próprio servidor de correio continua a aplicar-se.
admin-features-feature-access-protocols = Protocolos de acesso ao correio
admin-features-feature-access-protocols-help = Com que protocolos de servidor de correio estes utilizadores podem entrar, separados por vírgulas: imap, pop3, ews.
admin-features-feature-accounts-linked = Caixas de correio ligadas
admin-features-feature-calendar = Calendário
admin-features-feature-contacts = Contactos
admin-features-feature-tasks = Tarefas
admin-features-feature-notes = Notas
admin-features-preview = Valores efetivos
admin-features-preview-help = O valor de cada funcionalidade para um utilizador ou para um utilizador de um domínio, depois de todos os perfis, grupos, valores de domínio e bloqueios.
admin-features-preview-kind = Pré-visualizar para
admin-features-preview-target = Quem
admin-features-preview-show = Mostrar
admin-features-preview-loading = A carregar…
admin-features-preview-state = Efetivo
admin-features-preview-from = Decidido por
admin-features-preview-profiles = Perfis que se aplicam, o primeiro prevalece: { $names }
admin-features-preview-no-profiles = Nenhum perfil se aplica; decidem os valores do domínio e do servidor.
admin-features-source-default = Predefinição incorporada
admin-features-source-env-fallback = Predefinição do ambiente
admin-features-source-env-pin = Ambiente (fixado)
admin-features-source-global = Valor do servidor
admin-features-source-domain = Valor do domínio
admin-features-source-profile = Perfil
admin-features-source-profile-named = Perfil { $name }
admin-features-source-group = Valor do grupo
admin-features-source-admin-user = Valor definido para este utilizador
admin-features-source-user = A escolha do próprio utilizador
admin-features-source-locked = { $source } (bloqueado)

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
