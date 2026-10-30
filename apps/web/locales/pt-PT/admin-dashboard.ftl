# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Atualizar
admin-dash-unavailable = Os números do painel ainda não estão disponíveis.
admin-dash-unavailable-detail = Este servidor não fornece os dados do painel (GET /admin/dashboard responde 404). Ficarão disponíveis numa atualização posterior do servidor. As outras secções funcionam como antes.
admin-dash-error-network = Não foi possível carregar o painel: o servidor não respondeu. Tente novamente.
admin-dash-error-status = Não foi possível carregar o painel: o servidor respondeu { $status }. Tente novamente; se o problema se repetir, consulte o registo do servidor.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Requer atenção
admin-dash-warn-fix = Abrir a secção que resolve isto
admin-dash-warn-blocked-accounts = { $count ->
    [one] Uma conta não consegue iniciar sessão porque o respetivo domínio não tem configuração.
   *[other] { $count } contas não conseguem iniciar sessão porque os respetivos domínios não têm configuração.
}
admin-dash-warn-no-trusted-proxy = Há regras de IP ativas, mas não está configurado nenhum proxy fidedigno, pelo que todos os clientes podem parecer vir do endereço do proxy.
admin-dash-warn-allowlist-bypass = Uma variável de ambiente desativa a lista de permissões de IP.
admin-dash-warn-metrics-token = O endpoint de métricas não tem nenhum token definido.
admin-dash-warn-geoip-missing = Uma política de países está ativa mas não está carregada nenhuma base de dados de países utilizável, por isso os inícios de sessão abrangidos são recusados.
admin-dash-warn-geoip-stale = Uma política de países está ativa e a base de dados de países está desatualizada.
admin-dash-warn-pending-migrations = { $count ->
    [one] Uma migração da base de dados não foi aplicada. Reinicie o servidor para a aplicar.
   *[other] { $count } migrações da base de dados não foram aplicadas. Reinicie o servidor para as aplicar.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Um certificado a montante expira em breve (segundo o último teste de ligação).
   *[other] { $count } certificados a montante expiram em breve (segundo o último teste de ligação).
}
admin-dash-warn-unknown = Aviso do servidor { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] Uma conta bloqueada: sem configuração de domínio
   *[other] { $count } contas bloqueadas: sem configuração de domínio
}
admin-dash-blocked-note = Estas contas iniciaram sessão antes desta atualização, mas o respetivo domínio não tem configuração na consola, pelo que não conseguem iniciar sessão agora. Configurar o domínio permite-lhes voltar a iniciar sessão.
admin-dash-blocked-chart = Contas bloqueadas por domínio
admin-dash-blocked-row = { $count ->
    [one] uma conta
   *[other] { $count } contas
}
admin-dash-blocked-seen = (último anfitrião de correio utilizado: { $host })
admin-dash-configure-domain = Configurar { $domain }
admin-dash-blocked-no-access = A sua função não permite configurar domínios.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Lista de configuração: nada pendente
    [one] Lista de configuração: um item pendente
   *[other] Lista de configuração: { $open } itens pendentes
}
admin-dash-check-done = Concluído
admin-dash-check-open = Pendente
admin-dash-check-optional = (opcional)
admin-dash-check-go = Abrir
admin-dash-check-hide = Ocultar
admin-dash-check-unhide = Mostrar novamente
admin-dash-check-hidden = { $count ->
    [one] Um item está oculto. Ocultar não o marca como concluído.
   *[other] { $count } itens estão ocultos. Ocultar não os marca como concluídos.
}
admin-dash-check-show-hidden = Mostrar itens ocultos
admin-dash-check-hide-hidden = Não mostrar itens ocultos
admin-dash-check-domain-exists = Adicionar um domínio de correio
admin-dash-check-domain-upstream = Atribuir um servidor de correio ao domínio ou definir um servidor a montante predefinido
admin-dash-check-test-connection = Executar um teste de ligação com êxito (nos últimos 7 dias)
admin-dash-check-preferred-domain = Escolher um domínio preferido, para que os utilizadores possam iniciar sessão sem o escrever
admin-dash-check-second-admin = Criar uma segunda conta de administrador ou desativar o início de sessão por variável de ambiente
admin-dash-check-trusted-proxies = Configurar os proxies fidedignos à frente deste servidor
admin-dash-check-security-policy = Guardar a política de segurança
admin-dash-check-metrics-token = Definir um token para o endpoint de métricas
admin-dash-check-branding = Rever a personalização da marca

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Contas e sessões
admin-dash-users = Utilizadores aprovisionados
admin-dash-signed-in-24h = Contas com sessão iniciada, últimas 24 horas
admin-dash-signed-in-7d = Contas com sessão iniciada, últimos 7 dias
admin-dash-domains = Domínios
admin-dash-domains-detail = { $upstream } com servidor de correio, { $disabled } desativados
admin-dash-sessions = Sessões de webmail ativas
admin-dash-admin-sessions = Sessões de administração ativas
admin-dash-security = Segurança do início de sessão
admin-dash-failed-webmail = Inícios de sessão no webmail falhados, últimas 24 horas
admin-dash-failed-admin = Inícios de sessão de administração falhados, últimas 24 horas
admin-dash-active-blocks = Endereços bloqueados
admin-dash-allowlists = Listas de permissões de IP
admin-dash-allowlists-value = Administração { $admin }, webmail { $webmail }
admin-dash-on = ativada
admin-dash-off = desativada
admin-dash-trusted-proxies = Proxies fidedignos
admin-dash-configured = Configurado
admin-dash-not-configured = Não configurado
admin-dash-untrusted-forwarded = { $count ->
    [one] Um pedido em 24 horas trazia cabeçalhos de reencaminhamento de um par não fidedigno
   *[other] { $count } pedidos em 24 horas traziam cabeçalhos de reencaminhamento de um par não fidedigno
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Tráfego, últimos { $mins } minutos
admin-dash-traffic-note = Contabilizado por este processo do servidor desde que foi iniciado. Não inclui outros processos nem réplicas.
admin-dash-traffic-empty = Não foram contabilizados pedidos neste intervalo.
admin-dash-requests = Pedidos por minuto
admin-dash-errors = Erros do servidor (5xx) por minuto
admin-dash-p95 = Tempo de resposta, percentil 95

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Estado
admin-dash-h-ok = OK
admin-dash-h-attention = Requer atenção
admin-dash-h-db = Base de dados
admin-dash-h-db-ok = Respondeu em { $ms }
admin-dash-h-migrations = Migrações
admin-dash-h-migrations-ok = Atualizadas ({ $n } aplicadas)
admin-dash-h-migrations-behind = { $applied } de { $expected } aplicadas
admin-dash-h-engine = Sincronização do correio
admin-dash-h-engine-value = { $accounts } contas registadas, { $errors } com erro de sincronização
admin-dash-h-outbox = Caixa de saída
admin-dash-h-outbox-value = { $pending } em espera (a mais antiga: { $oldest }), { $failed } falhadas em 24 horas
admin-dash-h-outbox-none = nenhuma
admin-dash-h-index = Índice de pesquisa
admin-dash-h-index-value = { $n } documentos
admin-dash-h-index-rebuilding = A reconstruir ({ $n } documentos até agora)
admin-dash-h-sandbox = Ambiente isolado de apresentação
admin-dash-h-sandbox-ok = Disponível: os documentos não fidedignos são apresentados num processo separado.
admin-dash-h-sandbox-missing = Em falta: os documentos não fidedignos são apresentados dentro do processo do servidor.
admin-dash-h-sandbox-required = Em falta, e este servidor exige-o: a apresentação de documentos não fidedignos falha.
admin-dash-h-plugins = Runtime de plugins
admin-dash-h-plugins-ok = Disponível
admin-dash-h-plugins-off = Desativado
admin-dash-na-proxy = este servidor funciona em modo proxy e não sincroniza o correio diretamente.
admin-dash-na-unreported = o servidor não o indicou.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Armazenamento
admin-dash-db-size = Ficheiro da base de dados
admin-dash-wal-size = Registo de escrita antecipada
admin-dash-index-size = Índice de pesquisa
admin-dash-volume-free = Espaço livre no volume de dados
admin-dash-volume-of = de { $total }
admin-dash-na-wal = Esta base de dados não utiliza um ficheiro de registo de escrita antecipada, ou o servidor não o indicou.
admin-dash-na-index = Este servidor não tem índice de pesquisa.
admin-dash-na-volume = O servidor não consegue ler o espaço livre nesta plataforma.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Ações de administração recentes
admin-dash-audit-empty = Ainda não há ações de administração registadas.
admin-dash-audit-when = Quando
admin-dash-audit-actor = Quem
admin-dash-audit-action = Ação
admin-dash-audit-target = Alvo
admin-dash-audit-all = Abrir o registo de auditoria completo
admin-dash-server = Servidor
admin-dash-version = Versão
admin-dash-build = Compilação
admin-dash-mode = Modo
admin-dash-mode-engine = Motor: este servidor sincroniza e guarda o correio
admin-dash-mode-proxy = Proxy: este servidor passa os pedidos ao servidor de correio
admin-dash-uptime = Em execução há
admin-dash-base-path = Servido em
admin-dash-base-root = a raiz do site
admin-dash-duration-dh = { $d } d { $h } h
admin-dash-duration-hm = { $h } h { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
admin-dash-customize = Personalizar painel
admin-dash-limitations = Disponibilidade dos dados
