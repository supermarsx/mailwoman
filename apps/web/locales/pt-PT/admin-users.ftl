# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-users. Source: ../en/admin-users.ftl

## Admin › Lista de utilizadores (t107): a tabela, a pesquisa, os filtros, as vistas guardadas e as ações em lote.

admin-userlist-caption = Utilizadores. Os cabeçalhos das colunas ordenam a lista.
admin-userlist-search = Pesquisar utilizadores
admin-userlist-search-placeholder = Nome, endereço, alias ou domínio
admin-userlist-count = { $count ->
    [one] { $count } utilizador
   *[other] { $count } utilizadores
}
admin-userlist-count-filtered = { $count ->
    [one] { $count } de { $total } utilizadores corresponde
   *[other] { $count } de { $total } utilizadores correspondem
}
admin-userlist-range = { $from }–{ $to } de { $total }
admin-userlist-loading = A carregar os utilizadores…
admin-userlist-retry = Tentar novamente
admin-userlist-empty-filtered = Nenhum utilizador corresponde a esta pesquisa e a estes filtros.
admin-userlist-compact = Linhas compactas
admin-userlist-columns = Colunas
admin-userlist-columns-legend = Mostrar colunas
admin-userlist-actions = Ações
admin-userlist-paging = Páginas de utilizadores
admin-userlist-page-size = Linhas por página
admin-userlist-prev = Anterior
admin-userlist-next = Seguinte
admin-userlist-sort-by = Ordenar por
admin-userlist-sort-asc = { $column }, ascendente
admin-userlist-sort-desc = { $column }, descendente

## Colunas
admin-userlist-col-user = Utilizador
admin-userlist-col-domain = Domínio
admin-userlist-col-status = Estado
admin-userlist-col-2fa = Dois fatores
admin-userlist-col-role = Função na consola
admin-userlist-col-last-signin = Último início de sessão
admin-userlist-col-last-seen = Última atividade
admin-userlist-col-created = Criado
admin-userlist-col-quota = Quota
admin-userlist-col-profile = Perfil de funcionalidades
admin-userlist-col-linked = Caixas de correio ligadas

## Valores das células
admin-userlist-state-active = Ativo
admin-userlist-state-pending = Nunca iniciou sessão
admin-userlist-state-blocked = Bloqueado
admin-userlist-state-disabled = Desativado
admin-userlist-state-deleted = Eliminado
admin-userlist-2fa-passkey = Chave de acesso
admin-userlist-2fa-totp = Aplicação de autenticação
admin-userlist-2fa-none = Nenhum
admin-userlist-signin-never = Nunca
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Sem atividade registada
admin-userlist-seen-at = Última atividade { $relative } ({ $exact })
admin-userlist-quota-unlimited = Ilimitada
admin-userlist-profile-default = Predefinido
admin-userlist-profile-via-user = Atribuído a este utilizador
admin-userlist-profile-via-group = De um grupo
admin-userlist-profile-via-domain = Do domínio
admin-userlist-more = Mais ações
admin-userlist-more-for = Mais ações para { $account }
admin-userlist-select = Selecionar { $account }
admin-userlist-select-page = Selecionar todos os utilizadores desta página

## Filtros
admin-userlist-filters = Filtros
admin-userlist-add-filter = Adicionar filtro
admin-userlist-clear-all = Limpar tudo
admin-userlist-chip-edit = Editar o filtro { $label }
admin-userlist-chip-remove = Remover o filtro { $label }
admin-userlist-filter-domain = Domínio
admin-userlist-filter-status = Estado
admin-userlist-filter-tfa = Dois fatores
admin-userlist-filter-role = Função na consola
admin-userlist-filter-profile = Perfil de funcionalidades
admin-userlist-filter-signin = Último início de sessão
admin-userlist-filter-created = Criado
admin-userlist-filter-no-choices = Ainda não há nada para escolher.
admin-userlist-filter-remove = Remover filtro
admin-userlist-apply = Aplicar
admin-userlist-cancel = Cancelar
admin-userlist-role-any = Qualquer função na consola
admin-userlist-role-none = Sem função na consola
admin-userlist-profile-none = Sem perfil (predefinido)
admin-userlist-signin-older = { $days ->
    [one] Há mais de { $days } dia
   *[other] Há mais de { $days } dias
}
admin-userlist-signin-within = { $days ->
    [one] No último { $days } dia
   *[other] Nos últimos { $days } dias
}
admin-userlist-signin-custom = Há mais de este número de dias:
admin-userlist-signin-days = Dias
admin-userlist-days-invalid = Introduza um número inteiro de dias entre 1 e 36500.
admin-userlist-created-from = De
admin-userlist-created-to = Até
admin-userlist-created-between = { $from } a { $to }
admin-userlist-created-since = desde { $date }
admin-userlist-created-until = até { $date }
admin-userlist-dates-invalid = A data de início é posterior à data de fim.

## Vistas guardadas
admin-userlist-views = Vistas guardadas
admin-userlist-view-save = Guardar esta vista…
admin-userlist-view-name = Nome da vista
admin-userlist-view-save-confirm = Guardar vista
admin-userlist-view-delete = Eliminar a vista «{ $name }»

## Ações por linha e em lote
admin-userlist-bulk = Ações sobre os utilizadores selecionados
admin-userlist-selected = { $count ->
    [one] { $count } selecionado
   *[other] { $count } selecionados
}
admin-userlist-clear-selection = Limpar seleção
admin-userlist-block = Bloquear início de sessão
admin-userlist-unblock = Desbloquear início de sessão
admin-userlist-sign-out = Terminar todas as sessões
admin-userlist-delete = Eliminar o acesso local
admin-userlist-bulk-profile = Atribuir perfil de funcionalidades
admin-userlist-bulk-profile-clear = Remover o perfil próprio do utilizador
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } utilizador selecionado é ignorado: a ação não se aplica a ele.
   *[other] { $count } utilizadores selecionados são ignorados: a ação não se aplica a eles.
}
admin-userlist-confirm-block = { $count ->
    [one] Bloquear o início de sessão de { $count } utilizador?
   *[other] Bloquear o início de sessão de { $count } utilizadores?
}
admin-userlist-confirm-block-one = Bloquear o início de sessão de { $account }?
admin-userlist-confirm-block-body = Os novos inícios de sessão são recusados e as sessões atuais do Mailwoman terminam. A caixa de correio de origem não é alterada.
admin-userlist-confirm-block-action = Bloquear início de sessão
admin-userlist-confirm-unblock = { $count ->
    [one] Desbloquear o início de sessão de { $count } utilizador?
   *[other] Desbloquear o início de sessão de { $count } utilizadores?
}
admin-userlist-confirm-unblock-one = Desbloquear o início de sessão de { $account }?
admin-userlist-confirm-unblock-body = O bloqueio local do início de sessão é removido. As outras restrições de segurança continuam a aplicar-se.
admin-userlist-confirm-unblock-action = Desbloquear início de sessão
admin-userlist-confirm-signout = { $count ->
    [one] Terminar todas as sessões de { $count } utilizador?
   *[other] Terminar todas as sessões de { $count } utilizadores?
}
admin-userlist-confirm-signout-one = Terminar todas as sessões de { $account }?
admin-userlist-confirm-signout-body = Todas as sessões abertas do Mailwoman destes utilizadores terminam. Podem voltar a iniciar sessão.
admin-userlist-confirm-signout-action = Terminar sessões
admin-userlist-confirm-delete = { $count ->
    [one] Eliminar o acesso local de { $count } utilizador?
   *[other] Eliminar o acesso local de { $count } utilizadores?
}
admin-userlist-confirm-delete-one = Eliminar o acesso local de { $account }?
admin-userlist-confirm-delete-body = É mantido um registo de acesso eliminado para que o utilizador não seja inscrito de novo automaticamente. As definições, o histórico do Mailwoman e a caixa de correio de origem ficam guardados. Escreva o endereço para confirmar.
admin-userlist-confirm-delete-action = Eliminar o acesso local
admin-userlist-confirm-profile = { $count ->
    [one] Alterar o perfil de funcionalidades de { $count } utilizador?
   *[other] Alterar o perfil de funcionalidades de { $count } utilizadores?
}
admin-userlist-confirm-profile-one = Alterar o perfil de funcionalidades de { $account }?
admin-userlist-confirm-profile-body = O perfil é atribuído diretamente a cada utilizador e aplica-se a partir do pedido seguinte. As contas que não são caixas de correio nativas não podem receber um perfil e são indicadas como falhadas.
admin-userlist-confirm-profile-action = Alterar perfil
admin-userlist-bulk-done-block = { $count ->
    [one] Bloqueado o início de sessão de { $count } utilizador.
   *[other] Bloqueado o início de sessão de { $count } utilizadores.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Desbloqueado o início de sessão de { $count } utilizador.
   *[other] Desbloqueado o início de sessão de { $count } utilizadores.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] Terminadas as sessões de { $count } utilizador.
   *[other] Terminadas as sessões de { $count } utilizadores.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Eliminado o acesso local de { $count } utilizador.
   *[other] Eliminado o acesso local de { $count } utilizadores.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Alterado o perfil de funcionalidades de { $count } utilizador.
   *[other] Alterado o perfil de funcionalidades de { $count } utilizadores.
}
admin-userlist-bulk-failed = { $done } concluídos, { $failed } falhados: { $reason }
