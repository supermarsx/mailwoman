# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Usuários. Os cabeçalhos das colunas ordenam a lista.
admin-userlist-search = Buscar usuários
admin-userlist-search-placeholder = Nome, endereço, alias ou domínio
admin-userlist-count = { $count ->
    [one] { $count } usuário
   *[other] { $count } usuários
 }
admin-userlist-count-filtered = { $count ->
    [one] { $count } de { $total } usuários corresponde
   *[other] { $count } de { $total } usuários correspondem
 }
admin-userlist-range = { $from }–{ $to } de { $total }
admin-userlist-loading = Carregando usuários…
admin-userlist-retry = Tentar novamente
admin-userlist-empty-filtered = Nenhum usuário corresponde a esta busca e a estes filtros.
admin-userlist-compact = Linhas compactas
admin-userlist-columns = Colunas
admin-userlist-columns-legend = Mostrar colunas
admin-userlist-actions = Ações
admin-userlist-paging = Páginas de usuários
admin-userlist-page-size = Linhas por página
admin-userlist-prev = Anterior
admin-userlist-next = Próxima
admin-userlist-sort-by = Ordenar por
admin-userlist-sort-asc = { $column }, crescente
admin-userlist-sort-desc = { $column }, decrescente
admin-userlist-col-user = Usuário
admin-userlist-col-domain = Domínio
admin-userlist-col-status = Status
admin-userlist-col-2fa = Verificação em duas etapas
admin-userlist-col-role = Função no console
admin-userlist-col-last-signin = Último login
admin-userlist-col-last-seen = Última atividade
admin-userlist-col-created = Criado em
admin-userlist-col-quota = Cota
admin-userlist-col-profile = Perfil de recursos
admin-userlist-col-linked = Caixas de correio vinculadas
admin-userlist-state-active = Ativo
admin-userlist-state-pending = Nunca fez login
admin-userlist-state-blocked = Bloqueado
admin-userlist-state-disabled = Desativado
admin-userlist-state-deleted = Excluído
admin-userlist-2fa-passkey = Passkey
admin-userlist-2fa-totp = App autenticador
admin-userlist-2fa-none = Nenhuma
admin-userlist-signin-never = Nunca
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Nenhuma atividade registrada
admin-userlist-seen-at = Última atividade { $relative } ({ $exact })
admin-userlist-quota-unlimited = Ilimitada
admin-userlist-profile-default = Padrão
admin-userlist-profile-via-user = Atribuído a este usuário
admin-userlist-profile-via-group = De um grupo
admin-userlist-profile-via-domain = Do domínio
admin-userlist-more = Mais ações
admin-userlist-more-for = Mais ações para { $account }
admin-userlist-select = Selecionar { $account }
admin-userlist-select-page = Selecionar todos os usuários desta página
admin-userlist-filters = Filtros
admin-userlist-add-filter = Adicionar filtro
admin-userlist-clear-all = Limpar tudo
admin-userlist-chip-edit = Editar filtro { $label }
admin-userlist-chip-remove = Remover filtro { $label }
admin-userlist-filter-domain = Domínio
admin-userlist-filter-status = Status
admin-userlist-filter-tfa = Verificação em duas etapas
admin-userlist-filter-role = Função no console
admin-userlist-filter-profile = Perfil de recursos
admin-userlist-filter-signin = Último login
admin-userlist-filter-created = Criado em
admin-userlist-filter-no-choices = Ainda não há nada para escolher.
admin-userlist-filter-remove = Remover filtro
admin-userlist-apply = Aplicar
admin-userlist-cancel = Cancelar
admin-userlist-role-any = Qualquer função no console
admin-userlist-role-none = Nenhuma função no console
admin-userlist-profile-none = Nenhum perfil (padrão)
admin-userlist-signin-older = { $days ->
    [one] Há mais de { $days } dia
   *[other] Há mais de { $days } dias
 }
admin-userlist-signin-within = { $days ->
    [one] No último { $days } dia
   *[other] Nos últimos { $days } dias
 }
admin-userlist-signin-custom = Há mais deste número de dias:
admin-userlist-signin-days = Dias
admin-userlist-days-invalid = Informe um número inteiro de dias de 1 a 36500.
admin-userlist-created-from = De
admin-userlist-created-to = Até
admin-userlist-created-between = { $from } a { $to }
admin-userlist-created-since = desde { $date }
admin-userlist-created-until = até { $date }
admin-userlist-dates-invalid = A data inicial é posterior à data final.
admin-userlist-views = Visualizações salvas
admin-userlist-view-save = Salvar esta visualização…
admin-userlist-view-name = Nome da visualização
admin-userlist-view-save-confirm = Salvar visualização
admin-userlist-view-delete = Excluir a visualização “{ $name }”
admin-userlist-bulk = Ações nos usuários selecionados
admin-userlist-selected = { $count ->
    [one] { $count } selecionado
   *[other] { $count } selecionados
 }
admin-userlist-clear-selection = Limpar seleção
admin-userlist-block = Bloquear login
admin-userlist-unblock = Desbloquear login
admin-userlist-sign-out = Sair de todos os lugares
admin-userlist-delete = Excluir acesso local
admin-userlist-bulk-profile = Atribuir perfil de recursos
admin-userlist-bulk-profile-clear = Remover o perfil próprio do usuário
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } usuário selecionado será ignorado: a ação não se aplica a ele.
   *[other] { $count } usuários selecionados serão ignorados: a ação não se aplica a eles.
 }
admin-userlist-confirm-block = { $count ->
    [one] Bloquear o login de { $count } usuário?
   *[other] Bloquear o login de { $count } usuários?
 }
admin-userlist-confirm-block-one = Bloquear o login de { $account }?
admin-userlist-confirm-block-body = Novos logins são recusados e as sessões atuais do Mailwoman são encerradas. A caixa de correio upstream não é alterada.
admin-userlist-confirm-block-action = Bloquear login
admin-userlist-confirm-unblock = { $count ->
    [one] Desbloquear o login de { $count } usuário?
   *[other] Desbloquear o login de { $count } usuários?
 }
admin-userlist-confirm-unblock-one = Desbloquear o login de { $account }?
admin-userlist-confirm-unblock-body = O bloqueio local de login é removido. As outras restrições de segurança continuam valendo.
admin-userlist-confirm-unblock-action = Desbloquear login
admin-userlist-confirm-signout = { $count ->
    [one] Desconectar { $count } usuário de todos os lugares?
   *[other] Desconectar { $count } usuários de todos os lugares?
 }
admin-userlist-confirm-signout-one = Desconectar { $account } de todos os lugares?
admin-userlist-confirm-signout-body = Todas as sessões abertas do Mailwoman desses usuários são encerradas. Eles podem fazer login novamente.
admin-userlist-confirm-signout-action = Desconectar
admin-userlist-confirm-delete = { $count ->
    [one] Excluir o acesso local de { $count } usuário?
   *[other] Excluir o acesso local de { $count } usuários?
 }
admin-userlist-confirm-delete-one = Excluir o acesso local de { $account }?
admin-userlist-confirm-delete-body = Um registro de acesso excluído é mantido para que o usuário não seja cadastrado novamente de forma automática. As configurações do Mailwoman, o histórico e a caixa de correio upstream continuam armazenados. Digite o endereço para confirmar.
admin-userlist-confirm-delete-action = Excluir acesso local
admin-userlist-confirm-profile = { $count ->
    [one] Alterar o perfil de recursos de { $count } usuário?
   *[other] Alterar o perfil de recursos de { $count } usuários?
 }
admin-userlist-confirm-profile-one = Alterar o perfil de recursos de { $account }?
admin-userlist-confirm-profile-body = O perfil é atribuído diretamente a cada usuário e vale a partir da próxima solicitação. Identidades de caixa de correio nativas, de proxy e de SSO são compatíveis.
admin-users-profile-help = Selecione Padrão para remover a atribuição direta deste usuário e usar os perfis herdados de seus grupos ou do domínio.
admin-users-profile-ambiguous = Este usuário tem várias identidades de login. Escolha a identidade desejada em Perfis de recursos.
admin-users-profile-unavailable = Nenhuma identidade de caixa de correio verificada está disponível. Este usuário precisa fazer login antes que um perfil possa ser atribuído.
admin-userlist-confirm-profile-action = Alterar perfil
admin-userlist-bulk-done-block = { $count ->
    [one] Login bloqueado para { $count } usuário.
   *[other] Login bloqueado para { $count } usuários.
 }
admin-userlist-bulk-done-unblock = { $count ->
    [one] Login desbloqueado para { $count } usuário.
   *[other] Login desbloqueado para { $count } usuários.
 }
admin-userlist-bulk-done-signout = { $count ->
    [one] { $count } usuário desconectado.
   *[other] { $count } usuários desconectados.
 }
admin-userlist-bulk-done-delete = { $count ->
    [one] Acesso local excluído para { $count } usuário.
   *[other] Acesso local excluído para { $count } usuários.
 }
admin-userlist-bulk-done-profile = { $count ->
    [one] Perfil de recursos alterado para { $count } usuário.
   *[other] Perfil de recursos alterado para { $count } usuários.
 }
admin-userlist-bulk-failed = { $done } concluídos, { $failed } com falha: { $reason }
