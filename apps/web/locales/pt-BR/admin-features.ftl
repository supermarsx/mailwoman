# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Perfis de recursos
admin-features-intro = Um perfil é um conjunto nomeado de chaves de recursos. Atribua-o a um domínio, grupo ou usuário. As atribuições individuais funcionam com caixas de correio nativas e identidades de proxy ou SSO verificadas; elas controlam o acesso aos recursos do Mailwoman.
admin-features-new = Novo perfil
admin-features-search = Buscar perfis
admin-features-empty = Nenhum perfil de recursos ainda.
admin-features-no-match = Nenhum perfil corresponde à busca.
admin-features-load-error = Não foi possível carregar o catálogo de recursos.
admin-features-not-loaded = O catálogo de recursos ainda não foi carregado. Recarregue a página e tente de novo.
admin-features-stale = Outro administrador alterou o catálogo de recursos. Ele foi recarregado; confira os valores e salve de novo.
admin-features-not-found = Nenhum perfil com o id { $id }.
admin-features-col-name = Nome
admin-features-col-values = Chaves
admin-features-col-assigned = Atribuído a
admin-features-values-count = { $count ->
    [one] { $count } chave definida
   *[other] { $count } chaves definidas
 }
admin-features-assigned-count = { $count ->
    [0] Não atribuído
    [one] { $count } lugar
   *[other] { $count } lugares
 }
admin-features-editor = Configurações do perfil
admin-features-name = Nome do perfil
admin-features-name-help = Exibido apenas para administradores, até 128 caracteres.
admin-features-name-invalid = Informe um nome de 1 a 128 caracteres.
admin-features-switches = Chaves de recursos
admin-features-switches-help = "Não definido" deixa o recurso para o próximo perfil, o domínio ou o padrão do servidor. Um valor bloqueado não pode ser alterado pelo usuário nem por um nível inferior.
admin-features-col-feature = Recurso
admin-features-col-value = Valor
admin-features-col-lock = Bloqueio
admin-features-value-for = Valor de { $feature }
admin-features-unset = Não definido
admin-features-on = Permitido
admin-features-off = Não permitido
admin-features-lock = Bloqueado
admin-features-save = Salvar perfil
admin-features-create = Criar perfil
admin-features-delete = Excluir perfil
admin-features-delete-title = Excluir o perfil { $name }?
admin-features-delete-body = { $count ->
    [0] O perfil não está atribuído em lugar nenhum. As chaves dele deixam de valer imediatamente.
    [one] O perfil é removido do { $count } lugar a que está atribuído, e as chaves dele deixam de valer imediatamente.
   *[other] O perfil é removido dos { $count } lugares a que está atribuído, e as chaves dele deixam de valer imediatamente.
 }
admin-features-saved = Perfil { $name } salvo.
admin-features-deleted = Perfil { $name } excluído.
admin-features-assigned = Perfil { $name } atribuído.
admin-features-unassigned = Atribuição de { $name } removida.
admin-features-assignments = Atribuído a
admin-features-assignments-help = Os perfis valem em ordem de prioridade, do maior para o menor. Em caso de empate, a atribuição a um usuário vale antes da de um grupo, e a de um grupo antes da de um domínio.
admin-features-assignments-empty = Este perfil ainda não está atribuído.
admin-features-assign-form = Atribuir este perfil
admin-features-col-scope = Vale para
admin-features-col-target = Nome
admin-features-col-priority = Prioridade
admin-features-col-actions = Ações
admin-features-scope-domain = Domínio
admin-features-scope-group = Grupo
admin-features-scope-user = Usuário
admin-features-assign = Atribuir
admin-features-unassign = Remover
admin-features-no-domain = Nenhum domínio configurado.
admin-features-no-group = Não existem grupos de usuários.
admin-features-no-user = Nenhuma conta de caixa de correio fez login ainda.
admin-features-tools = Ferramentas do perfil
admin-features-preset = Começar de uma predefinição
admin-features-preset-choose = Escolha uma predefinição
admin-features-preset-apply = Aplicar predefinição
admin-features-preset-applied = Predefinição { $name } aplicada. Confira os valores e salve.
admin-features-preset-full = Completo
admin-features-preset-standard = Padrão
admin-features-preset-mail-only = Somente e-mail
admin-features-preset-minimal = Mínimo / quiosque
admin-features-import = Importar JSON
admin-features-export = Exportar JSON
admin-features-exported = Perfil exportado como arquivo JSON.
admin-features-imported = { $count ->
    [one] { $count } valor importado. Confira e salve.
   *[other] { $count } valores importados. Confira e salve.
 }
admin-features-import-json = O arquivo não é um JSON válido.
admin-features-import-format = O arquivo não é um perfil de recursos do Mailwoman.
admin-features-import-version = Este arquivo de perfil de recursos vem de uma versão mais recente e não pode ser importado.
admin-features-clone = Clonar
admin-features-clone-name = Cópia de { $name }
admin-features-filter = Encontrar um recurso
admin-features-filter-placeholder = Nome, configuração ou chave
admin-features-filter-none = Nenhum recurso corresponde à busca.
admin-features-diff-summary = { $differs ->
    [one] { $differs } valor difere do padrão do servidor
   *[other] { $differs } valores diferem do padrão do servidor
} · { $same } o repetem
admin-features-differs = Difere do padrão
admin-features-default-is = Padrão: { $value }
admin-features-fix-values = Corrija os valores marcados como inválidos antes de salvar.
admin-features-allow-all = Permitir tudo
admin-features-deny-all = Não permitir nada
admin-features-clear-all = Limpar
admin-features-group-allow = Permitir todos os recursos de { $group }
admin-features-group-deny = Não permitir nenhum recurso de { $group }
admin-features-group-clear = Limpar todos os valores de { $group }
admin-features-enforcement-ui = Oculto somente no app
admin-features-enforcement-partial = Alguns limites ainda não são contados
admin-features-other = Outras configurações que um perfil pode conter
admin-features-other-help = Todas as outras configurações que um administrador pode definir por perfil. Deixe um campo vazio para manter o valor do servidor.
admin-features-cat-compose = E-mail e redação
admin-features-cat-reading = Leitura e privacidade
admin-features-cat-views = Visualizações e busca
admin-features-cat-organise = Organização
admin-features-cat-pim = Calendário, contatos, tarefas, notas e chaves
admin-features-cat-data = Dados
admin-features-cat-accounts = Contas e acesso
admin-features-cat-integrations = Integrações
admin-features-cat-developer = Acesso de desenvolvedor
admin-features-cat-personal = Personalização
admin-features-feature-compose-sending = Limites de envio
admin-features-feature-compose-sending-help = O servidor recusa destinatários por mensagem acima do limite. Mensagens por hora e por dia são armazenadas no perfil, mas ainda não são contadas.
admin-features-feature-storage = Cota de armazenamento
admin-features-feature-storage-help = O limite de armazenamento é guardado no perfil; a cota própria do servidor de e-mail continua valendo.
admin-features-feature-access-protocols = Protocolos de acesso ao e-mail
admin-features-feature-access-protocols-help = Com quais protocolos do servidor de e-mail estes usuários podem fazer login, separados por vírgulas: imap, pop3, ews.
admin-features-feature-accounts-linked = Caixas de correio vinculadas
admin-features-feature-calendar = Calendário
admin-features-feature-contacts = Contatos
admin-features-feature-tasks = Tarefas
admin-features-feature-notes = Notas
admin-features-preview = Valores efetivos
admin-features-preview-help = No que cada recurso resulta para um usuário ou para um usuário de um domínio, depois de todos os perfis, grupos, valores de domínio e bloqueios.
admin-features-preview-kind = Prévia para
admin-features-preview-target = Quem
admin-features-preview-show = Mostrar
admin-features-preview-loading = Carregando…
admin-features-preview-state = Efetivo
admin-features-preview-from = Definido por
admin-features-preview-profiles = Perfis que se aplicam, o primeiro vence: { $names }
admin-features-preview-no-profiles = Nenhum perfil se aplica; o domínio e os valores do servidor decidem.
admin-features-source-default = Padrão embutido
admin-features-source-env-fallback = Padrão do ambiente
admin-features-source-env-pin = Ambiente (fixado)
admin-features-source-global = Valor do servidor
admin-features-source-domain = Valor do domínio
admin-features-source-profile = Perfil
admin-features-source-profile-named = Perfil { $name }
admin-features-source-group = Valor do grupo
admin-features-source-admin-user = Valor definido para este usuário
admin-features-source-user = Escolha do próprio usuário
admin-features-source-locked = { $source } (bloqueado)
admin-features-feature-compose-open-tracking = Rastreamento de abertura de mensagens
admin-features-feature-compose-open-tracking-help = Permite registrar a abertura de mensagens formatadas elegíveis usando a imagem de rastreamento do servidor. Requer uma URL pública configurada.
admin-features-feature-reading-original-html = Visualização do HTML original
admin-features-feature-reading-original-html-help = Permite que os usuários optem pelo HTML original em mensagens individuais, dentro do quadro de leitura isolado. O HTML higienizado continua sendo o padrão.
admin-features-inherit = Herdar
admin-features-override = Definir um valor
admin-features-setting-mode = Modo do valor de { $setting }
admin-features-inherit-help = Herdar remove o valor e o bloqueio deste perfil. O próximo perfil, domínio ou valor do servidor aplicável decide; a prévia efetiva mostra o resultado e sua origem.
admin-features-domain-alias = { $alias } (alias de { $domain })
admin-features-domain-aliases = { $domain } (aliases: { $aliases })

# -- Machine translation (2026-11): added to match en ---------------------
admin-retention-col-category = Categoria
admin-retention-col-kept = Mantido
admin-retention-col-change = Alterar
admin-retention-cat-default = Padrão
admin-retention-cat-admin = Ações de administradores
admin-retention-cat-security = Eventos de segurança
admin-retention-cat-alert = Alertas de usuários
admin-retention-forever = Todas as entradas
admin-retention-days = { $days ->
    [one] { $days } dia
   *[other] { $days } dias
 }
admin-retention-follows-default = Padrão: { $kept }
admin-retention-hold-on = Retenção legal
admin-retention-read-only = Alterar a retenção exige a permissão settings.write.
admin-retention-edit-for = Retenção de { $category }
admin-retention-keep = Manter
admin-retention-mode-inherit = Seguir o padrão
admin-retention-mode-forever = Todas as entradas
admin-retention-mode-days = Por um número de dias
admin-retention-days-label = Dias
admin-retention-hold = Retenção legal
admin-retention-save = Salvar
admin-retention-bounds = Informe um número inteiro de { $min } a { $max }.
admin-retention-saved = Retenção de { $category } salva.
admin-retention-task-none = Ainda não há tarefa de limpeza. Ela é criada, diariamente às 03:15 UTC, quando um número de dias é salvo pela primeira vez.
admin-retention-task-disabled = A tarefa de limpeza agendada está desativada; as entradas só são limpas com Limpar agora.
admin-retention-task-next = Próxima limpeza agendada: { $when }.
admin-retention-task-daily = Limpeza diária às 03:15 UTC.
admin-retention-last = Última limpeza que removeu entradas: { $when }.
admin-retention-prune = Limpar agora
admin-retention-prune-title = Limpar o log de auditoria agora?
admin-retention-prune-body = As entradas mais antigas que a retenção da sua categoria são excluídas. As categorias que mantêm todas as entradas ou estão sob retenção legal não são alteradas. Isso não pode ser desfeito.
admin-retention-pruned = { $count ->
    [0] Nenhuma entrada era antiga o suficiente para ser removida.
    [one] { $count } entrada removida.
   *[other] { $count } entradas removidas.
 }
admin-auth-attempted = Tentado como { $name }
