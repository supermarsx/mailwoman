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
admin-dash-h-migrations = Migrações
admin-dash-h-engine = Sincronização do correio
admin-dash-h-outbox = Caixa de saída
admin-dash-h-index = Índice de pesquisa
admin-dash-h-sandbox = Ambiente isolado de apresentação
admin-dash-h-plugins = Runtime de plugins
admin-dash-h-status-ok = OK
admin-dash-h-status-warn = Aviso
admin-dash-h-status-fail = Com falhas
admin-dash-h-status-disabled = Desativado
admin-dash-h-checked = Verificado às { $time }
admin-dash-h-refresh = Verificar de novo
admin-dash-h-refresh-failed = Não foi possível repetir a verificação (estado { $status }). É mostrado o resultado anterior.
admin-dash-h-refresh-network = Não foi possível repetir a verificação: o servidor não respondeu. É mostrado o resultado anterior.
admin-dash-h-enable = Para ativar
admin-dash-h-probe-time = A verificação demorou { $time }
admin-dash-h-history = Últimas { $n } verificações
admin-dash-h-open-diagnostics = Diagnóstico
admin-dash-h-open-logs = Registos
admin-dash-h-open-plugins = Plugins
admin-dash-h-open-maintenance = Manutenção
admin-dash-h-not-tracked = Não medido
admin-dash-h-unlimited = sem limite
admin-dash-h-m-none = nenhuma
admin-dash-h-m-ago = há { $age }
admin-dash-h-m-ping = Tempo de resposta
admin-dash-h-m-pool = Ligações
admin-dash-h-m-pool-value = { $inUse } em uso, { $idle } livres ({ $size } abertas)
admin-dash-h-m-engine = Servidor de base de dados
admin-dash-h-m-location = Localização
admin-dash-h-m-db-size = Tamanho da base de dados
admin-dash-h-m-wal-size = Registo de escrita antecipada
admin-dash-h-m-journal = Modo de diário
admin-dash-h-m-slow = Consultas lentas (última hora)
admin-dash-h-m-applied = Aplicadas
admin-dash-h-m-applied-value = { $applied } de { $expected } nesta versão
admin-dash-h-m-latest = Mais recente aplicada
admin-dash-h-m-pending = Ainda não aplicadas
admin-dash-h-m-failed = Iniciadas sem terminar
admin-dash-h-m-unknown = De uma versão mais recente
admin-dash-h-m-accounts = Caixas de correio
admin-dash-h-m-accounts-value = { $total } registadas: { $syncing } a sincronizar, { $idle } em espera, { $erroring } com falhas
admin-dash-h-m-last-sync = Desde a última sincronização concluída
admin-dash-h-m-last-sync-value = mediana { $p50 }, máximo { $max }
admin-dash-h-m-never-synced = Não sincronizadas desde o arranque do servidor
admin-dash-h-m-queue = Sincronizações em curso
admin-dash-h-m-errors-hour = Erros de sincronização (última hora)
admin-dash-h-m-by-kind = Ligações por tipo de servidor
admin-dash-h-m-kind-value = { $kind }: { $accounts } caixas, { $watch } ligações de vigilância
admin-dash-h-m-recent-errors = Erros recentes
admin-dash-h-m-queued = À espera de envio
admin-dash-h-m-queued-value = { $queued } a enviar já, { $deferred } a repetir, { $sending } em envio
admin-dash-h-m-oldest = A mais antiga em espera
admin-dash-h-m-scheduled = Agendadas ou retidas
admin-dash-h-m-scheduled-value = { $scheduled } agendadas, { $held } retidas
admin-dash-h-m-failed-total = Falhadas (ainda numa Caixa de saída)
admin-dash-h-m-failed-total-value = { $failed } falhadas, { $uncertain } com entrega desconhecida
admin-dash-h-m-sends-hour = Envios (última hora)
admin-dash-h-m-sends-hour-value = { $sent } enviados, { $deferred } adiados, { $failed } falhados
admin-dash-h-m-last-send = Último envio
admin-dash-h-m-last-send-value = { $outcome }, há { $age }
admin-dash-h-m-last-send-none = Nenhum desde o arranque do servidor
admin-dash-h-outcome-sent = Enviado
admin-dash-h-outcome-deferred = Adiado
admin-dash-h-outcome-failed = Falhado
admin-dash-h-outcome-uncertain = Entrega desconhecida
admin-dash-h-m-submission = Servidores de envio
admin-dash-h-m-submission-value = { $reachable } de { $checked } responderam (só ligação, sem início de sessão; o mais lento { $slowest }), verificado há { $age }
admin-dash-h-m-submission-none = Não está configurado nenhum servidor de envio.
admin-dash-h-m-submission-pending = Em verificação; aparece na próxima verificação.
admin-dash-h-m-documents = Documentos
admin-dash-h-m-documents-value = { $documents } de { $messages } mensagens guardadas
admin-dash-h-m-index-pending = Ainda por indexar
admin-dash-h-m-index-size = Tamanho no disco
admin-dash-h-m-index-memory = Em memória
admin-dash-h-m-last-commit = Última gravação
admin-dash-h-m-rebuild = Reconstrução
admin-dash-h-m-rebuild-running = Em curso: { $pct } ({ $done } de { $total })
admin-dash-h-m-rebuild-idle = Parada
admin-dash-h-m-version = Versão do processo
admin-dash-h-m-worker-missing = Sem binário do processo
admin-dash-h-m-render-ping = Ping (arranque, isolamento, resposta)
admin-dash-h-m-render-jobs = Tarefas em curso
admin-dash-h-m-render-failures = Problemas (última hora)
admin-dash-h-m-render-failures-value = { $failed } falhadas, { $timeouts } fora de tempo, { $busy } recusadas por ocupação
admin-dash-h-m-jail = Isolamento do núcleo
admin-dash-h-m-jail-required = Obrigatório
admin-dash-h-m-jail-optional = Não obrigatório nesta plataforma
admin-dash-h-m-jail-degraded = Reduzido: { $why }
admin-dash-h-m-plugins = Plugins
admin-dash-h-m-plugins-value = { $installed } instalados, { $enabled } ativos, { $loaded } carregados
admin-dash-h-m-plugin-list = Plugins ativos
admin-dash-h-m-plugin-loaded = carregado
admin-dash-h-m-plugin-not-loaded = não carregado ({ $reason })
admin-dash-h-m-plugin-limits = memória { $memory }, prazo { $deadline }, combustível { $fuel }
admin-dash-h-m-plugin-calls = Chamadas falhadas (última hora)
admin-dash-h-m-plugin-calls-value = { $traps } interrupções, { $limits } limites excedidos, { $errors } outros erros
admin-dash-h-m-host-ping = Bloqueio do anfitrião de plugins
admin-dash-h-t-ping = Aviso acima de { $warn }, falha acima de { $fail }
admin-dash-h-t-oldest = Aviso quando uma mensagem espera mais de { $warn }, falha acima de { $fail }
admin-dash-h-t-pending = Aviso acima de { $n } mensagens por indexar
admin-dash-h-t-render = Aviso acima de { $warn }; falha sem resposta em { $timeout }
admin-dash-h-t-sync = Falha quando { $pct } ou mais das caixas não sincronizam
admin-dash-h-chart-db = Tempo de resposta da base de dados
admin-dash-h-chart-render = Ping do processo de apresentação
admin-dash-h-chart-outbox = Mensagens à espera de envio
admin-dash-h-chart-search = Mensagens por indexar
admin-dash-na-index-proxy = Modo proxy: este servidor não guarda índice de pesquisa.
admin-dash-na-index-memory = O índice de pesquisa está em memória, por isso não ocupa espaço no disco.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Armazenamento
admin-dash-db-size = Ficheiro da base de dados
admin-dash-wal-size = Registo de escrita antecipada
admin-dash-index-size = Índice de pesquisa
admin-dash-volume-free = Espaço livre no volume de dados
admin-dash-volume-of = de { $total }
admin-dash-na-wal = Esta base de dados não utiliza um ficheiro de registo de escrita antecipada, ou o servidor não o indicou.
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
