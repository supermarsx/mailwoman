# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Atualizar
admin-dash-unavailable = Os números do painel ainda não estão disponíveis.
admin-dash-unavailable-detail = Este servidor não fornece os dados do painel (GET /admin/dashboard responde 404). Eles chegam com uma atualização futura do servidor. As outras seções funcionam como antes.
admin-dash-error-network = Não foi possível carregar o painel: o servidor não respondeu. Tente novamente.
admin-dash-error-status = Não foi possível carregar o painel: o servidor respondeu { $status }. Tente novamente; se o erro se repetir, verifique o log do servidor.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Requer atenção
admin-dash-warn-fix = Abrir a seção que corrige isto
admin-dash-warn-blocked-accounts = { $count ->
    [one] { $count } conta não consegue entrar porque o domínio dela não tem configuração.
   *[other] { $count } contas não conseguem entrar porque os domínios delas não têm configuração.
}
admin-dash-warn-no-trusted-proxy = As regras de IP estão ativas, mas nenhum proxy confiável está configurado, então todos os clientes podem parecer vir do endereço do proxy.
admin-dash-warn-allowlist-bypass = Uma variável de ambiente desativa a lista de IPs permitidos.
admin-dash-warn-metrics-token = O endpoint de métricas não tem token definido.
admin-dash-warn-pending-migrations = { $count ->
    [one] { $count } migração do banco de dados não foi aplicada. Reinicie o servidor para aplicá-la.
   *[other] { $count } migrações do banco de dados não foram aplicadas. Reinicie o servidor para aplicá-las.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] { $count } certificado upstream expira em breve (segundo o último teste de conexão).
   *[other] { $count } certificados upstream expiram em breve (segundo o último teste de conexão).
}
admin-dash-warn-unknown = Aviso do servidor { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] { $count } conta bloqueada: sem configuração de domínio
   *[other] { $count } contas bloqueadas: sem configuração de domínio
}
admin-dash-blocked-note = Estas contas entraram antes desta atualização, mas o domínio delas não tem configuração no console, então elas não conseguem entrar agora. Configurar o domínio permite que entrem novamente.
admin-dash-blocked-chart = Contas bloqueadas por domínio
admin-dash-blocked-row = { $count ->
    [one] { $count } conta
   *[other] { $count } contas
}
admin-dash-blocked-seen = (último host de e-mail usado: { $host })
admin-dash-configure-domain = Configurar { $domain }
admin-dash-blocked-no-access = Sua função não pode configurar domínios.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Lista de configuração: nada pendente
    [one] Lista de configuração: { $open } item pendente
   *[other] Lista de configuração: { $open } itens pendentes
}
admin-dash-check-done = Concluído
admin-dash-check-open = Pendente
admin-dash-check-optional = (opcional)
admin-dash-check-go = Abrir
admin-dash-check-hide = Ocultar
admin-dash-check-unhide = Mostrar novamente
admin-dash-check-hidden = { $count ->
    [one] { $count } item está oculto. Ocultar não o marca como concluído.
   *[other] { $count } itens estão ocultos. Ocultar não os marca como concluídos.
}
admin-dash-check-show-hidden = Mostrar itens ocultos
admin-dash-check-hide-hidden = Deixar os itens ocultos de fora
admin-dash-check-domain-exists = Adicionar um domínio de e-mail
admin-dash-check-domain-upstream = Atribuir um servidor de e-mail ao domínio ou definir um upstream padrão
admin-dash-check-test-connection = Executar um teste de conexão bem-sucedido (nos últimos 7 dias)
admin-dash-check-preferred-domain = Escolher um domínio preferencial, para que os usuários possam entrar sem digitá-lo
admin-dash-check-second-admin = Criar uma segunda conta de administrador ou desativar o login pelo ambiente
admin-dash-check-trusted-proxies = Configurar os proxies confiáveis à frente deste servidor
admin-dash-check-security-policy = Salvar a política de segurança
admin-dash-check-metrics-token = Definir um token para o endpoint de métricas
admin-dash-check-branding = Revisar a identidade visual

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Contas e sessões
admin-dash-users = Usuários provisionados
admin-dash-signed-in-24h = Contas que entraram, últimas 24 horas
admin-dash-signed-in-7d = Contas que entraram, últimos 7 dias
admin-dash-domains = Domínios
admin-dash-domains-detail = { $upstream } com servidor de e-mail, { $disabled } desativados
admin-dash-sessions = Sessões de webmail ativas
admin-dash-admin-sessions = Sessões de administrador ativas
admin-dash-security = Segurança de login
admin-dash-failed-webmail = Logins de webmail com falha, últimas 24 horas
admin-dash-failed-admin = Logins de administrador com falha, últimas 24 horas
admin-dash-active-blocks = Endereços bloqueados
admin-dash-allowlists = Listas de IPs permitidos
admin-dash-allowlists-value = Administração { $admin }, webmail { $webmail }
admin-dash-on = ativada
admin-dash-off = desativada
admin-dash-trusted-proxies = Proxies confiáveis
admin-dash-configured = Configurado
admin-dash-not-configured = Não configurado
admin-dash-untrusted-forwarded = { $count ->
    [one] { $count } solicitação em 24 horas trouxe cabeçalhos de encaminhamento de um par não confiável
   *[other] { $count } solicitações em 24 horas trouxeram cabeçalhos de encaminhamento de um par não confiável
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Tráfego, últimos { $mins } minutos
admin-dash-traffic-note = Contado por este processo do servidor desde que iniciou. Outros processos ou réplicas não estão incluídos.
admin-dash-traffic-empty = Nenhuma solicitação foi contada neste intervalo.
admin-dash-requests = Solicitações por minuto
admin-dash-errors = Erros do servidor (5xx) por minuto
admin-dash-p95 = Tempo de resposta, percentil 95
admin-dash-ms = { $ms } ms

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Integridade
admin-dash-h-ok = OK
admin-dash-h-attention = Requer atenção
admin-dash-h-db = Banco de dados
admin-dash-h-db-ok = Respondeu em { $ms }
admin-dash-h-migrations = Migrações
admin-dash-h-migrations-ok = Atualizado ({ $n } aplicadas)
admin-dash-h-migrations-behind = { $applied } de { $expected } aplicadas
admin-dash-h-engine = Sincronização de e-mail
admin-dash-h-engine-value = { $accounts } contas registradas, { $errors } com erro de sincronização
admin-dash-h-outbox = Caixa de saída
admin-dash-h-outbox-value = { $pending } aguardando (mais antiga: { $oldest }), { $failed } com falha em 24 horas
admin-dash-h-outbox-none = nenhuma
admin-dash-h-index = Índice de pesquisa
admin-dash-h-index-value = { $n } documentos
admin-dash-h-index-rebuilding = Reconstruindo ({ $n } documentos até agora)
admin-dash-h-sandbox = Sandbox de renderização
admin-dash-h-sandbox-ok = Disponível: documentos não confiáveis são renderizados em um processo separado.
admin-dash-h-sandbox-missing = Ausente: documentos não confiáveis são renderizados dentro do processo do servidor.
admin-dash-h-sandbox-required = Ausente, e este servidor o exige: a renderização de documentos não confiáveis falha.
admin-dash-h-plugins = Runtime de plugins
admin-dash-h-plugins-ok = Disponível
admin-dash-h-plugins-off = Desativado
admin-dash-na-proxy = este servidor funciona no modo proxy e não sincroniza e-mails por conta própria.
admin-dash-na-unreported = o servidor não informou isso.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Armazenamento
admin-dash-db-size = Arquivo do banco de dados
admin-dash-wal-size = Log de gravação antecipada (WAL)
admin-dash-index-size = Índice de pesquisa
admin-dash-volume-free = Espaço livre no volume de dados
admin-dash-volume-of = de { $total }
admin-dash-na-wal = Este banco de dados não usa um arquivo de log de gravação antecipada, ou o servidor não o informou.
admin-dash-na-index = Este servidor não tem índice de pesquisa.
admin-dash-na-volume = O servidor não consegue ler o espaço livre nesta plataforma.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Ações recentes de administradores
admin-dash-audit-empty = Nenhuma ação de administrador registrada ainda.
admin-dash-audit-when = Quando
admin-dash-audit-actor = Quem
admin-dash-audit-action = Ação
admin-dash-audit-target = Alvo
admin-dash-audit-all = Abrir o log de auditoria completo
admin-dash-server = Servidor
admin-dash-version = Versão
admin-dash-build = Build
admin-dash-mode = Modo
admin-dash-mode-engine = Engine: este servidor sincroniza e armazena e-mails
admin-dash-mode-proxy = Proxy: este servidor repassa as solicitações ao servidor de e-mail
admin-dash-uptime = Em execução há
admin-dash-base-path = Servido em
admin-dash-base-root = a raiz do site
admin-dash-duration-dh = { $d } d { $h } h
admin-dash-duration-hm = { $h } h { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
