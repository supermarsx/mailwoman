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

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Integridade
admin-dash-h-ok = OK
admin-dash-h-attention = Requer atenção
admin-dash-h-db = Banco de dados
admin-dash-h-migrations = Migrações
admin-dash-h-engine = Sincronização de e-mail
admin-dash-h-outbox = Caixa de saída
admin-dash-h-index = Índice de pesquisa
admin-dash-h-sandbox = Sandbox de renderização
admin-dash-h-plugins = Runtime de plugins

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Armazenamento
admin-dash-db-size = Arquivo do banco de dados
admin-dash-wal-size = Log de gravação antecipada (WAL)
admin-dash-index-size = Índice de pesquisa
admin-dash-volume-free = Espaço livre no volume de dados
admin-dash-volume-of = de { $total }
admin-dash-na-wal = Este banco de dados não usa um arquivo de log de gravação antecipada, ou o servidor não o informou.
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

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = Uma política de país está ativa, mas nenhum banco de dados de países utilizável está carregado, então os logins que ela abrange são recusados.
admin-dash-warn-geoip-stale = Uma política de país está ativa e o banco de dados de países está desatualizado.
admin-dash-h-status-ok = OK
admin-dash-h-status-warn = Aviso
admin-dash-h-status-fail = Com falha
admin-dash-h-status-disabled = Desativado
admin-dash-h-checked = Verificado às { $time }
admin-dash-h-refresh = Verificar novamente
admin-dash-h-refresh-failed = Não foi possível repetir a verificação de integridade (status { $status }). O resultado anterior está sendo exibido.
admin-dash-h-refresh-network = Não foi possível repetir a verificação de integridade: o servidor não respondeu. O resultado anterior está sendo exibido.
admin-dash-h-enable = Para ativá-lo
admin-dash-h-probe-time = A sondagem levou { $time }
admin-dash-h-outbox-probe-time = A agregação de status levou { $time }; o tempo do handshake de envio é exibido separadamente.
admin-dash-h-history = Últimas { $n } verificações
admin-dash-h-open-diagnostics = Diagnóstico
admin-dash-h-open-logs = Logs
admin-dash-h-open-plugins = Plugins
admin-dash-h-open-maintenance = Manutenção
admin-dash-h-not-tracked = Não monitorado
admin-dash-h-unlimited = sem limite
admin-dash-h-m-none = nenhum
admin-dash-h-m-ago = há { $age }
admin-dash-h-m-ping = Ida e volta
admin-dash-h-m-pool = Pool de conexões
admin-dash-h-m-pool-value = { $inUse } em uso, { $idle } ociosas ({ $size } abertas)
admin-dash-h-m-engine = Servidor de banco de dados
admin-dash-h-m-location = Local
admin-dash-h-m-db-size = Tamanho do banco de dados
admin-dash-h-m-wal-size = Write-ahead log
admin-dash-h-m-journal = Modo de journal
admin-dash-h-m-slow = Consultas lentas (última hora)
admin-dash-h-m-applied = Aplicadas
admin-dash-h-m-applied-value = { $applied } de { $expected } nesta build
admin-dash-h-m-latest = Mais recente aplicada
admin-dash-h-m-pending = Ainda não aplicadas
admin-dash-h-m-failed = Iniciadas, não concluídas
admin-dash-h-m-unknown = De uma build mais recente
admin-dash-h-m-accounts = Caixas de correio
admin-dash-h-m-accounts-value = { $total } registradas: { $syncing } sincronizando, { $idle } ociosas, { $erroring } com falha
admin-dash-h-m-last-sync = Desde a última sincronização concluída
admin-dash-h-m-last-sync-value = mediana { $p50 }, mais longa { $max }
admin-dash-h-m-never-synced = Sem sincronização desde que o servidor iniciou
admin-dash-h-m-queue = Sincronizações em andamento
admin-dash-h-m-errors-hour = Erros de sincronização (última hora)
admin-dash-h-m-by-kind = Conexões por backend
admin-dash-h-m-kind-value = { $kind }: { $accounts } caixas de correio, { $watch } conexões de monitoramento
admin-dash-h-m-recent-errors = Erros recentes
admin-dash-h-m-queued = Aguardando envio
admin-dash-h-m-queued-value = { $queued } na vez, { $deferred } em nova tentativa, { $sending } sendo enviadas
admin-dash-h-m-oldest = Mais antiga aguardando
admin-dash-h-m-scheduled = Agendadas ou retidas
admin-dash-h-m-scheduled-value = { $scheduled } agendadas, { $held } retidas
admin-dash-h-m-failed-total = Com falha (ainda em uma Caixa de saída)
admin-dash-h-m-failed-total-value = { $failed } com falha, { $uncertain } com entrega desconhecida
admin-dash-h-m-sends-hour = Envios (última hora)
admin-dash-h-m-sends-hour-value = { $sent } enviadas, { $deferred } adiadas, { $failed } com falha
admin-dash-h-m-last-send = Último envio
admin-dash-h-m-last-send-value = { $outcome }, há { $age }
admin-dash-h-m-last-send-none = Nenhum desde que o servidor iniciou
admin-dash-h-outcome-sent = Enviada
admin-dash-h-outcome-deferred = Adiada
admin-dash-h-outcome-failed = Com falha
admin-dash-h-outcome-uncertain = Entrega desconhecida
admin-dash-h-m-submission = Servidores de envio
admin-dash-h-m-submission-value = { $reachable } de { $checked } responderam (apenas handshake, sem login; mais lento { $slowest }), verificado há { $age }
admin-dash-h-m-submission-none = Nenhum servidor de envio configurado.
admin-dash-h-m-submission-pending = Verificação de conexão em andamento.
admin-dash-h-m-submission-unmeasured = { $count } servidor(es) exigem uma verificação autenticada no provedor; nenhuma credencial foi enviada.
admin-dash-h-m-documents = Documentos
admin-dash-h-m-documents-value = { $documents } de { $messages } mensagens armazenadas
admin-dash-h-m-index-pending = Ainda não indexados
admin-dash-h-m-index-size = Tamanho em disco
admin-dash-h-m-index-memory = Na memória
admin-dash-h-m-last-commit = Último commit
admin-dash-h-m-rebuild = Reconstrução
admin-dash-h-m-rebuild-running = Em andamento: { $pct } ({ $done } de { $total })
admin-dash-h-m-rebuild-idle = Não está em andamento
admin-dash-h-m-version = Versão do worker
admin-dash-h-m-worker-missing = Nenhum binário do worker
admin-dash-h-m-render-ping = Ping (início, jail, resposta)
admin-dash-h-m-render-jobs = Jobs em execução agora
admin-dash-h-m-render-failures = Problemas (última hora)
admin-dash-h-m-render-failures-value = { $failed } com falha, { $timeouts } expiraram, { $busy } recusados por estar ocupado
admin-dash-h-m-jail = Jail do kernel
admin-dash-h-m-jail-required = Obrigatório
admin-dash-h-m-jail-optional = Não obrigatório nesta plataforma
admin-dash-h-m-jail-degraded = Degradado: { $why }
admin-dash-h-m-plugins = Plugins
admin-dash-h-m-plugins-value = { $installed } instalados, { $enabled } ativados, { $loaded } carregados
admin-dash-h-m-plugin-list = Plugins ativados
admin-dash-h-m-plugin-loaded = carregado
admin-dash-h-m-plugin-not-loaded = não carregado ({ $reason })
admin-dash-h-m-plugin-limits = memória { $memory }, prazo { $deadline }, fuel { $fuel }
admin-dash-h-m-plugin-calls = Chamadas com falha (última hora)
admin-dash-h-m-plugin-calls-value = { $traps } traps, { $limits } estouros de limite, { $errors } outros erros
admin-dash-h-m-host-ping = Lock do host de plugins
admin-dash-h-t-ping = Aviso acima de { $warn }, falha acima de { $fail }
admin-dash-h-t-oldest = Aviso quando uma mensagem espera mais de { $warn }, falha acima de { $fail }
admin-dash-h-t-pending = Aviso acima de { $n } mensagens não indexadas
admin-dash-h-t-render = Aviso acima de { $warn }; falha quando não há resposta em { $timeout }
admin-dash-h-t-sync = Falha quando { $pct } ou mais das caixas de correio não sincronizam
admin-dash-h-chart-db = Ida e volta do banco de dados
admin-dash-h-chart-render = Ping do worker de renderização
admin-dash-h-chart-outbox = Mensagens aguardando envio
admin-dash-h-chart-search = Mensagens não indexadas
admin-dash-na-index-proxy = Modo proxy: este servidor não mantém um índice de busca.
admin-dash-na-index-memory = O índice de busca é mantido na memória, então não tem tamanho em disco.
admin-dash-customize = Personalizar painel
admin-dash-limitations = Disponibilidade dos dados
