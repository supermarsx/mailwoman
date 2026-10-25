# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Painel
admin-shell-group-overview = Visão geral
admin-shell-group-people = Pessoas
admin-shell-group-security = Segurança
admin-shell-group-configuration = Configuração
admin-shell-group-operations = Operações
admin-shell-menu = Menu
admin-shell-close-menu = Fechar menu
admin-shell-search = Pesquisar
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Conectado como { $name }
admin-shell-language-label = Idioma
admin-shell-breadcrumb = Trilha de navegação
admin-shell-unknown-section = Esta seção não existe ou não está incluída na sua função.
admin-shell-back-to-dashboard = Voltar ao painel
admin-shell-cap-missing = Sua função não inclui { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Carregando…
admin-shell-retry = Tentar novamente
admin-shell-cancel = Cancelar
admin-shell-save = Salvar
admin-shell-not-available = Indisponível

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Sair desta página?
admin-shell-leave-body = Suas alterações nesta página não foram salvas. Sair as descarta.
admin-shell-leave-confirm = Sair e descartar alterações

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Digite { $name } para confirmar

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Pesquisar no console de administração
admin-shell-palette-label = Seção, usuário, domínio ou configuração
admin-shell-palette-results = Resultados
admin-shell-palette-none = Nada corresponde.
admin-shell-palette-searching = Pesquisando usuários, domínios e configurações…
admin-shell-palette-sections-only = Este servidor ainda não consegue pesquisar usuários, domínios e configurações, então apenas as seções são listadas.
admin-shell-palette-error = A pesquisa de usuários, domínios e configurações falhou; apenas as seções são listadas.
admin-shell-palette-kind-section = Seção
admin-shell-palette-kind-user = Usuário
admin-shell-palette-kind-domain = Domínio
admin-shell-palette-kind-setting = Configuração

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filtros
admin-shell-table-actions = Ações
admin-shell-table-error = Não foi possível carregar a lista.
admin-shell-table-paging = Páginas de { $table }
admin-shell-table-prev = Página anterior
admin-shell-table-next = Próxima página

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Mais recente { $latest }, máximo { $max }
admin-shell-chart-col-time = Horário
admin-shell-chart-col-item = Item
admin-shell-chart-col-value = Valor

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Definido por
admin-shell-pinned-sr = Este valor vem do ambiente do servidor e não pode ser alterado aqui.
admin-shell-lock-text = Bloquear para os usuários
admin-shell-lock-setting = { $setting }
admin-shell-restore = Restaurar valor anterior
admin-shell-restore-note = Alterado por { $actor } em { $time }. Valor anterior: { $value }
admin-shell-restore-done = O valor anterior foi restaurado.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Etapas
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Etapa { $n } de { $total }: { $title }
admin-shell-wizard-review = Revisar
admin-shell-wizard-back = Voltar
admin-shell-wizard-next = Avançar
admin-shell-wizard-saved = Salvo.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Não foi possível acessar o servidor. Verifique a conexão e tente novamente.
admin-shell-err-session = Sua sessão de administrador terminou. Entre novamente.
admin-shell-err-forbidden = Sua função não permite esta ação. Peça a um administrador com a função adequada.
admin-shell-err-out-of-scope = Este objeto pertence a um domínio que sua função não abrange.
admin-shell-err-not-found = Isto não existe mais. Recarregue a lista.
admin-shell-err-conflict = Alguém alterou isto nesse meio-tempo. Recarregue e tente novamente.
admin-shell-err-validation = O servidor recusou um valor. Verifique os campos e tente novamente.
admin-shell-err-rate-limited = Solicitações demais. Aguarde um minuto e tente novamente.
admin-shell-err-server = O servidor respondeu com um erro ({ $status }). Tente novamente; se o erro se repetir, verifique o log do servidor.
admin-shell-err-upstream-dns = Não foi possível resolver o nome do host. Verifique a grafia do host.
admin-shell-err-upstream-connect = Nada respondeu nesse host e porta. Verifique a porta e qualquer firewall no caminho.
admin-shell-err-upstream-tls = O servidor respondeu, mas o handshake TLS falhou. Verifique a porta e a configuração de segurança da conexão.
admin-shell-err-upstream-auth = O servidor recusou as credenciais. Verifique o nome de usuário e a senha.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Tentativas de login demais. Aguarde alguns minutos e tente novamente.
admin-shell-totp-form = Login de administrador, segunda etapa
admin-shell-totp-note = Informe o código de 6 dígitos do app autenticador configurado para { $name }.
admin-shell-totp-code = Código de autenticação
admin-shell-totp-verify = Verificar
admin-shell-totp-start-over = Recomeçar
admin-shell-totp-invalid = Esse código não é válido. Informe o código atual do seu app autenticador.
admin-shell-totp-expired = O login demorou demais. Recomece e informe sua senha.
