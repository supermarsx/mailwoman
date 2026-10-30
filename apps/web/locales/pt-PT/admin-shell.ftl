# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Painel
admin-shell-nav-settings = Definições
admin-config-title = Configuração efetiva
admin-diag-title = Diagnóstico
admin-diag-description = Execute verificações do servidor ou verificações de ligação para os endpoints de correio e DAV guardados. Estas verificações não se autenticam junto dos fornecedores, não entregam correio nem testam serviços arbitrários.
admin-diag-run = Executar verificações do servidor
admin-diag-refresh = Atualizar histórico
admin-diag-result = Resultado da verificação
admin-diag-checks = Resultados das verificações
admin-diag-check = Verificação
admin-diag-status = Estado
admin-diag-duration = Duração
admin-diag-history = Execuções anteriores
admin-diag-empty = Não existem execuções de diagnóstico guardadas.
admin-diag-more = Execuções mais antigas
admin-diag-not-saved = Os resultados são apresentados, mas não foi possível guardar o histórico ou a auditoria. Verifique a disponibilidade da base de dados antes de sair desta página.
admin-diag-unavailable = Este servidor ainda não disponibiliza o conjunto de diagnósticos.
admin-config-description = Consulte o relatório de configuração do servidor, com os dados sensíveis ocultados. Os valores da política global podem ter substituições por domínio, grupo e utilizador. Altere as definições nos respetivos editores.
admin-config-refresh = Atualizar configuração
admin-config-search = Pesquisar configuração
admin-config-group = Área
admin-config-all = Todas as áreas
admin-config-all-sources = Todas as origens
admin-config-clear = Limpar filtros
admin-config-no-results = Nenhuma entrada de configuração corresponde a estes filtros.
admin-config-count = { $count } entradas correspondentes
admin-config-table = Configuração visível para a sua função de administrador
admin-config-key = Definição
admin-config-value = Valor reportado
admin-config-source = Origem
admin-config-change = Como as alterações se aplicam
admin-config-unknown = Valor em execução indisponível
admin-config-empty = Não definido
admin-config-secret-set = Segredo configurado (oculto)
admin-config-secret-unset = Segredo não configurado
admin-config-input = Entrada configurada
admin-config-input-note = Esta entrada não prova qual o valor atualmente utilizado pelo subsistema.
admin-config-locked = Bloqueado pela política
admin-config-restart = Requer reinício ou recompilação
admin-config-live = Ver o comportamento da definição
admin-config-open-setting = Abrir editor de definições
admin-config-more = Mostrar mais entradas
admin-config-unavailable = Este servidor não disponibiliza um relatório de configuração.
admin-config-source-default = Predefinição
admin-config-source-stored = Política guardada
admin-config-source-env-pin = Substituição pelo ambiente
admin-config-source-env-fallback = Alternativa do ambiente
admin-config-source-boot = Capturado no arranque
admin-config-source-build = Valor da compilação
admin-config-source-refused = Não suportado
admin-shell-group-overview = Visão geral
admin-shell-group-people = Pessoas
admin-shell-group-security = Segurança
admin-shell-group-configuration = Configuração
admin-shell-group-operations = Operações
admin-shell-menu = Menu
admin-shell-close-menu = Fechar menu
admin-shell-search = Pesquisar
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Sessão iniciada como { $name }
admin-shell-language-label = Idioma
admin-shell-breadcrumb = Trilho de navegação
admin-shell-unknown-section = Esta secção não existe ou não está incluída na sua função.
admin-shell-back-to-dashboard = Voltar ao painel
admin-shell-cap-missing = A sua função não inclui { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = A carregar…
admin-shell-retry = Tentar novamente
admin-shell-cancel = Cancelar
admin-shell-save = Guardar
admin-shell-not-available = Não disponível

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Sair desta página?
admin-shell-leave-body = As alterações nesta página não foram guardadas. Se sair, serão descartadas.
admin-shell-leave-confirm = Sair e descartar alterações

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Escreva { $name } para confirmar

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Pesquisar na consola de administração
admin-shell-palette-label = Secção, utilizador, domínio ou definição
admin-shell-palette-results = Resultados
admin-shell-palette-none = Sem correspondências.
admin-shell-palette-searching = A pesquisar utilizadores, domínios e definições…
admin-shell-palette-sections-only = Este servidor ainda não permite pesquisar utilizadores, domínios e definições, pelo que só são listadas secções.
admin-shell-palette-error = A pesquisa de utilizadores, domínios e definições falhou; só são listadas secções.
admin-shell-palette-kind-section = Secção
admin-shell-palette-kind-user = Utilizador
admin-shell-palette-kind-domain = Domínio
admin-shell-palette-kind-setting = Definição

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filtros
admin-shell-table-actions = Ações
admin-shell-table-error = Não foi possível carregar a lista.
admin-shell-table-paging = Páginas de { $table }
admin-shell-table-prev = Página anterior
admin-shell-table-next = Página seguinte

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Último { $latest }, máximo { $max }
admin-shell-chart-col-time = Hora
admin-shell-chart-col-item = Item
admin-shell-chart-col-value = Valor

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Definido por
admin-shell-pinned-sr = Este valor provém do ambiente do servidor e não pode ser alterado aqui.
admin-shell-lock-text = Bloquear para os utilizadores
admin-shell-lock-setting = { $setting }
admin-shell-restore = Restaurar valor anterior
admin-shell-restore-note = Alterado por { $actor } em { $time }. Valor anterior: { $value }
admin-shell-restore-done = O valor anterior foi restaurado.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Passos
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Passo { $n } de { $total }: { $title }
admin-shell-wizard-review = Rever
admin-shell-wizard-back = Anterior
admin-shell-wizard-next = Seguinte
admin-shell-wizard-saved = Guardado.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Não foi possível contactar o servidor. Verifique a ligação e tente novamente.
admin-shell-err-session = A sua sessão de administração terminou. Inicie sessão novamente.
admin-shell-err-forbidden = A sua função não permite esta ação. Peça a um administrador com a função adequada.
admin-shell-err-out-of-scope = Este objeto pertence a um domínio que a sua função não abrange.
admin-shell-err-not-found = Já não existe. Recarregue a lista.
admin-shell-err-conflict = Outra pessoa alterou-o entretanto. Recarregue e tente novamente.
admin-shell-err-validation = O servidor recusou um valor. Verifique os campos e tente novamente.
admin-shell-err-rate-limited = Demasiados pedidos. Aguarde um minuto e tente novamente.
admin-shell-err-rate-limited-in = Demasiados pedidos, tente novamente dentro de { $seconds } s.
admin-shell-err-server = O servidor respondeu com um erro ({ $status }). Tente novamente; se o erro se repetir, consulte o registo do servidor.
admin-shell-err-upstream-dns = Não foi possível resolver o nome de anfitrião. Verifique se o nome está bem escrito.
admin-shell-err-upstream-connect = Nada respondeu nesse anfitrião e porta. Verifique a porta e qualquer firewall intermédia.
admin-shell-err-upstream-tls = O servidor respondeu, mas o handshake TLS falhou. Verifique a porta e a definição de segurança da ligação.
admin-shell-err-upstream-auth = O servidor recusou as credenciais. Verifique o nome de utilizador e a palavra-passe.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Demasiadas tentativas de início de sessão. Aguarde alguns minutos e tente novamente.
admin-shell-totp-form = Início de sessão de administrador, segundo passo
admin-shell-totp-note = Introduza o código de 6 dígitos da aplicação de autenticação configurada para { $name }.
admin-shell-totp-code = Código de autenticação
admin-shell-totp-verify = Verificar
admin-shell-totp-start-over = Recomeçar
admin-shell-totp-invalid = Esse código não é válido. Introduza o código atual da sua aplicação de autenticação.
admin-shell-totp-expired = O início de sessão demorou demasiado. Recomece e introduza a sua palavra-passe.
admin-diag-retention-failed = Os resultados foram guardados, mas não foi possível remover o histórico de diagnóstico expirado.
admin-auth-title = Registos de autenticação
admin-auth-description = Metadados de autenticação conservados até { $days } dias, sujeitos ao limite de entradas. As falhas repetidas correspondentes são agrupadas num intervalo de um minuto. Os detalhes desconhecidos não são inferidos.
admin-auth-unavailable = O histórico de autenticação não está disponível neste servidor.
admin-auth-unknown = Indisponível
admin-auth-search = Conta, nome de utilizador tentado ou IP
admin-auth-surface = Área de início de sessão
admin-auth-all = Todas
admin-auth-admin = Administração
admin-auth-webmail = Webmail
admin-auth-result = Resultado
admin-auth-success = Êxito
admin-auth-failure = Falha
admin-auth-blocked = Bloqueado
admin-auth-backend = Back-end (identificador exato)
admin-auth-reason = Motivo (código exato)
admin-auth-apply = Aplicar filtros
admin-auth-time = Primeira tentativa
admin-auth-account = Conta ou nome de utilizador tentado
admin-auth-method = Método
admin-auth-ip = IP de origem
admin-auth-count = Tentativas
admin-auth-last = Última:
admin-auth-device = Detalhes do dispositivo
admin-auth-empty = Sem tentativas correspondentes.
admin-auth-more = Tentativas mais antigas
admin-logs-title = Registos
admin-app-logs-title = Registos da aplicação
admin-app-logs-description = Avisos e erros capturados por este processo do servidor, sujeitos ao respetivo filtro de registo. Até 1000 entradas das últimas 24 horas; são limpas no reinício. Os eventos correspondentes num intervalo de um minuto são agrupados. Os campos sensíveis são ocultados.
admin-app-logs-discarded = Entradas descartadas do buffer deste processo: { $count }
admin-app-logs-unavailable = Os registos da aplicação não estão disponíveis neste servidor.
admin-app-logs-search = Pesquisar detalhes ocultados
admin-app-logs-target = Componente
admin-app-logs-level = Gravidade
admin-app-logs-details = Detalhes ocultados
admin-app-logs-empty = Não há avisos nem erros correspondentes no buffer deste processo.
admin-config-filters = Procurar configuração

admin-config-entries = Entradas de configuração
admin-shell-sidebar-resize = Redimensionar barra lateral de navegação
admin-shell-sidebar-show = Mostrar barra lateral de navegação
admin-shell-sidebar-hide = Ocultar barra lateral de navegação
admin-diag-backends = Verificar todos os back-ends guardados
admin-diag-cancel = Cancelar verificações
admin-diag-cancelling = Cancelamento pedido. As verificações em curso estão a ser interrompidas.
admin-diag-copy = Copiar relatório de resolução de problemas
admin-diag-export = Exportar relatório de resolução de problemas
admin-diag-copied = Relatório de resolução de problemas (com dados ocultados) copiado.
admin-diag-limited = O inventário excedeu o orçamento limitado de verificações. Esta execução é parcial.
admin-diag-redaction = As verificações utilizam destinos guardados e não enviam credenciais. Os relatórios incluem indicadores de segurança em execução, nomes de domínio, identificadores de back-end e resultados; nunca palavras-passe, tokens, configuração em bruto nem correio. Não são testadas ações autenticadas junto de fornecedores, serviços arbitrários nem pipes.
admin-diag-omit-domains = Substituir nomes de domínio e IDs de back-end no relatório
admin-diag-filter = Filtrar verificações, domínios ou back-ends
admin-diag-all = Todos os estados
admin-diag-catalogue = Verificações e limites suportados
admin-diag-cancelled = Verificações canceladas. As ligações não concluídas foram interrompidas.
admin-diag-backends-permission = Requer as permissões globais de execução de diagnóstico e de configuração dos servidores de origem dos domínios.
admin-diag-no-checks = Não foram registadas verificações nesta execução. Isto não implica que algum endpoint tenha tido êxito.
admin-diag-no-match = Nenhuma verificação corresponde a este filtro.
admin-diag-limits = Até { $checks } verificações, { $concurrency } ligações simultâneas, { $check } segundos por verificação (tempo em fila incluído) e { $run } segundos para o trabalho das sondas.

admin-login-retry-countdown = Tente novamente dentro de { $seconds } segundos.
admin-analytics-title = Análises de utilização

admin-attachments-title = Política de anexos
admin-attachments-description = Controle a entrega de anexos com base nos tipos MIME e nomes de ficheiro guardados. As regras aplicam-se às definições globais e de domínio. Uma pré-visualização que recebe os bytes originais não consegue impedir que esses bytes sejam guardados; isto não é uma análise de malware nem uma proteção contra cópia.
admin-attachments-policy-title = Regras de entrega e pré-visualização
admin-attachments-unavailable = As definições da política de anexos não estão disponíveis neste servidor.
admin-attachments-edit = Editar política

admin-attachments-refresh = Atualizar regras

admin-nav-nextcloud = Nextcloud
admin-nav-package-tracking = Seguimento de encomendas

admin-diag-started = Iniciada
admin-diag-suite = Conjunto
admin-diag-failed = Verificações falhadas
admin-diag-open-run = Ver execução de { $date }

admin-integrations-tab-webhooks = Webhooks

admin-obs-statistics = Estatísticas
admin-obs-p50 = Tempo de resposta, mediana
admin-obs-accounts = Contas e sessões
admin-obs-count-provisionedUsers = Utilizadores aprovisionados
admin-obs-count-accountsSignedIn24h = Contas com sessão iniciada, últimas 24 horas
admin-obs-count-accountsSignedIn7d = Contas com sessão iniciada, últimos 7 dias
admin-obs-count-domains = Domínios
admin-obs-count-activeSessions = Sessões de webmail ativas
admin-obs-count-activeAdminSessions = Sessões de administração ativas
admin-obs-db-latency = Tempo de consulta da base de dados
admin-obs-outbox-pending = Mensagens pendentes na caixa de saída
admin-obs-outbox-failed-hour = Envios falhados, última hora

admin-users-configure = Configurar
admin-users-configure-for = Configurar { $account }

admin-dash-reorder = Reordenar { $name }
admin-dash-reorder-help = Arraste para mover esta secção. Com o foco do teclado, utilize as setas para cima ou para baixo para alterar a sua posição.

# -- Níveis do trilho de navegação (t95) ---------------------------------------
admin-crumb-loading = A carregar o nome
admin-crumb-item = Item
admin-crumb-new = Novo
admin-crumb-edit = Editar
admin-crumb-clone = Copiar
admin-crumb-guided = Configuração guiada
admin-crumb-overview = Vista geral
admin-crumb-configure = Configuração
admin-crumb-clients = Clientes
admin-crumb-register = Registar
admin-crumb-manage = Gerir
admin-crumb-connections = Ligações
admin-crumb-diagnostics = Diagnóstico
admin-crumb-policy = Política
admin-crumb-aliases = Aliases
admin-crumb-alias-new = Novo alias
admin-crumb-alias-edit = Editar alias
admin-crumb-alias-clone = Copiar alias
admin-crumb-backends = Servidores de origem
admin-crumb-bans = Lista de IPs banidos
admin-crumb-locks = Bloqueios temporários de início de sessão
admin-crumb-geography = Política de países para início de sessão
admin-crumb-geoip = Base de dados de países
admin-crumb-protection = Proteção agregada e adaptativa
admin-crumb-inbound = Acesso de entrada
admin-crumb-deployment = Verificações da implementação
admin-crumb-console = Endereço da consola
admin-crumb-sessions = Sessões
admin-crumb-audit = Registo de auditoria
admin-crumb-statistics = Estatísticas
admin-crumb-telemetry = Telemetria operacional
admin-crumb-directory = Diretório LDAP / GAL
admin-crumb-webhooks = Webhooks
admin-crumb-keys = Chaves de API e MCP
admin-crumb-hosting = Painéis de alojamento
admin-crumb-ssh = Túneis SSH
admin-crumb-domains = Domínios
admin-crumb-mailboxes = Caixas de correio
admin-crumb-application = Registos da aplicação
admin-crumb-authentication = Registos de autenticação
admin-crumb-jobs = Tarefas de reparação
admin-crumb-mode = Modo de manutenção
admin-crumb-destinations = Permissões de destinos internos
admin-crumb-proxies = Rotas de proxy de saída
admin-crumb-totp = Aplicação de autenticação
admin-crumb-security = Segurança da implementação
admin-crumb-entity-user = Utilizador
admin-crumb-entity-admin = Administrador
admin-crumb-entity-role = Função
admin-crumb-entity-sso = Fornecedor de início de sessão
admin-crumb-entity-idp-client = Cliente do fornecedor de identidade
admin-crumb-entity-plugin = Plugin
admin-crumb-entity-ui-plugin = Plugin de interface
admin-crumb-entity-profile = Perfil de funcionalidades
admin-crumb-entity-nextcloud = Instância Nextcloud
admin-crumb-entity-hosting = Ligação ao painel de alojamento
admin-crumb-entity-ssh = Túnel SSH
admin-crumb-entity-egress-proxy = Rota de proxy
admin-crumb-entity-egress-destination = Destino interno
admin-crumb-entity-backend = Servidor de origem
admin-crumb-backend-mail = Correio
admin-crumb-backend-submission = Envio
admin-crumb-backend-rules = Regras
