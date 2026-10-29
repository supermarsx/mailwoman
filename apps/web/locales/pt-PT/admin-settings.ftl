# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-settings. Source: ../en/admin-settings.ftl

admin-settings-title = Definições
admin-settings-description = Valores para todo o servidor, por domínio e por utilizador. Um valor bloqueado não pode ser alterado pelos utilizadores nem por âmbitos mais específicos.
admin-settings-list-caption = Definições
admin-settings-search = Pesquisar definições
admin-settings-empty = Nenhuma definição corresponde à pesquisa.
admin-settings-unavailable = Este servidor ainda não disponibiliza o modelo de definições.
admin-settings-col-setting = Definição
admin-settings-col-value = Valor
admin-settings-col-source = Origem
admin-settings-col-overrides = Valores específicos
admin-settings-open = Abrir
admin-settings-overrides-none = Nenhum
admin-settings-overrides = { $domains } domínios, { $users } utilizadores
admin-settings-not-found = Não existe nenhuma definição com o nome { $key }.

# Value display
admin-settings-value-on = Ativado
admin-settings-value-off = Desativado
admin-settings-value-empty = Não definido
admin-settings-value-list = { $count ->
    [one] { $count } item
   *[other] { $count } itens
}

# Detail
admin-settings-detail-key = Chave: { $key }
admin-settings-global = Para todo o servidor
admin-settings-global-help = Este valor aplica-se a todos os utilizadores, exceto se um domínio ou um utilizador tiver um valor mais específico ou se o utilizador escolher o seu próprio valor, quando tal é permitido.
admin-settings-value-label = Valor
admin-settings-json-hint = JSON. O servidor verifica o valor e indica a parte que recusa.
admin-settings-json-invalid = Isto não é JSON válido.
admin-settings-user-override = Os utilizadores podem alterá-lo
admin-settings-user-override-help = Quando ativado, os utilizadores podem escolher o seu próprio valor nas respetivas Definições. Quando desativado, aplica-se-lhes o valor definido aqui ou para o seu domínio.
admin-settings-lock-help = Quando bloqueado, os valores dos domínios e as escolhas dos utilizadores são ignorados e este valor aplica-se a todos.
admin-settings-save = Guardar
admin-settings-saved = Guardado. Aplica-se no prazo de 30 segundos em todos os processos do servidor.
admin-settings-saved-restart = Guardado. Aplica-se após um reinício.
admin-settings-dedicated = Esta definição é alterada em { $section }, que verifica primeiro o seu efeito.
admin-settings-dedicated-link = Abrir { $section }
admin-settings-pinned = Definido pela variável de ambiente { $name }. O valor guardado aqui aplica-se quando a variável não está definida.
admin-settings-lock-title = Bloquear { $setting }?
admin-settings-lock-body = O bloqueio ignora os valores mais específicos listados abaixo. Estes são mantidos e voltam a aplicar-se quando desbloquear.
admin-settings-lock-confirm = Bloquear
admin-settings-impact-domain = { $count ->
    [one] { $count } valor de domínio é ignorado
   *[other] { $count } valores de domínio são ignorados
}
admin-settings-impact-user = { $count ->
    [one] { $count } escolha própria de utilizador é ignorada
   *[other] { $count } escolhas próprias de utilizadores são ignoradas
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } valor por utilizador é ignorado
   *[other] { $count } valores por utilizador são ignorados
}
admin-settings-impact-profile = { $count ->
    [one] { $count } valor de perfil de funcionalidades é ignorado
   *[other] { $count } valores de perfil de funcionalidades são ignorados
}
admin-settings-impact-group = { $count ->
    [one] { $count } valor de grupo é ignorado
   *[other] { $count } valores de grupo são ignorados
}

# Domain values
admin-settings-domains = Valores por domínio
admin-settings-domains-help = Um valor de domínio aplica-se aos utilizadores desse domínio em vez do valor para todo o servidor.
admin-settings-domains-caption = Valores por domínio de { $setting }
admin-settings-domains-empty = Nenhum domínio tem um valor próprio.
admin-settings-domain = Domínio
admin-settings-domain-value = Valor para este domínio
admin-settings-domain-locked = Bloqueado para os utilizadores do domínio
admin-settings-domain-add = Adicionar um valor de domínio
admin-settings-domain-save = Guardar valor de domínio
admin-settings-domain-remove = Remover
admin-settings-domain-remove-title = Remover o valor de { $domain }?
admin-settings-domain-remove-body = Os utilizadores de { $domain } voltam a receber o valor para todo o servidor.
admin-settings-domain-required = Introduza um nome de domínio.
admin-settings-locked-yes = Bloqueado
admin-settings-locked-no = Não bloqueado

# Per-user values
admin-settings-users = Valores por utilizador
admin-settings-users-help = Um valor que um administrador definiu para um utilizador. Nos limites, o utilizador é identificado pelo endereço.
admin-settings-users-caption = Valores por utilizador de { $setting }
admin-settings-users-empty = Nenhum utilizador tem um valor definido por um administrador.
admin-settings-user = Utilizador
admin-settings-user-hint = ID da conta ou, nos limites, o endereço.
admin-settings-user-value = Valor para este utilizador
admin-settings-user-add = Adicionar um valor por utilizador
admin-settings-user-save = Guardar valor do utilizador
admin-settings-user-required = Introduza um utilizador.
admin-settings-user-remove-title = Remover o valor de { $user }?
admin-settings-user-remove-body = O utilizador passa a receber o valor do seu domínio ou o valor para todo o servidor.

# Sources (what decides the value now)
admin-settings-source-default = Predefinição incorporada
admin-settings-source-envFallback = Variável de ambiente (sem valor guardado)
admin-settings-source-envPin = Variável de ambiente
admin-settings-source-global = Guardado para todo o servidor

# Categories
admin-settings-cat-interface = Interface
admin-settings-cat-appearance = Aspeto
admin-settings-cat-mail = Correio
admin-settings-cat-notifications = Notificações
admin-settings-cat-offline = Utilização offline
admin-settings-cat-assistant = Assistente de IA
admin-settings-cat-upstream = Servidor de correio
admin-settings-cat-signin = Início de sessão
admin-settings-cat-admin = Consola de administração
admin-settings-cat-passwords = Palavras-passe
admin-settings-cat-limits = Limites e utilização
admin-settings-cat-plugins = Plugins
admin-settings-cat-features = Permissões de funcionalidades
admin-settings-cat-integrations = Permissões de integrações
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Permitir ficheiros, partilha e operações de calendário, contactos e tarefas do Nextcloud associados à conta.
admin-settings-key-cap-integration-google = Ponte Google
admin-settings-help-cap-integration-google = Permitir operações da ponte Google, entre as quais a obtenção e renovação de tokens.
admin-settings-key-cap-integration-microsoft = Ponte Microsoft
admin-settings-help-cap-integration-microsoft = Permitir operações da ponte Microsoft, entre as quais a obtenção e renovação de tokens.
admin-settings-key-cap-integration-family = Integração individual
admin-settings-help-cap-integration-family = Permissão para um identificador de integração individual; a cobertura em execução está listada no catálogo de funcionalidades.
admin-settings-key-cap-ui-plugin-family = Plugin de interface individual
admin-settings-help-cap-ui-plugin-family = Permissão para um plugin de interface. Utilize a chave de política exata do catálogo de integrações.
admin-settings-key-cap-ai-assistant = Assistente de IA
admin-settings-help-cap-ai-assistant = Permitir a configuração e a invocação do assistente.
admin-settings-key-cap-plugins = Plugins
admin-settings-help-cap-plugins = Permitir operações de plugins de interface e do motor.
admin-settings-key-cap-pim-calendar = Calendário
admin-settings-help-cap-pim-calendar = Permitir métodos e subscrições de calendário.
admin-settings-key-cap-pim-contacts = Contactos
admin-settings-help-cap-pim-contacts = Permitir métodos de contactos e de livros de endereços.
admin-settings-key-cap-pim-tasks = Tarefas
admin-settings-help-cap-pim-tasks = Permitir métodos de tarefas.
admin-settings-key-cap-pim-notes = Notas
admin-settings-help-cap-pim-notes = Permitir métodos de notas.
admin-settings-key-cap-export = Exportar
admin-settings-help-cap-export = Permitir exportações de correio e PIM.
admin-settings-key-cap-import = Importar
admin-settings-help-cap-import = Permitir importações de correio e PIM.
admin-settings-key-cap-api-keys = Chaves de API
admin-settings-help-cap-api-keys = Política de chaves de API. Consulte o catálogo de funcionalidades para conhecer a cobertura da aplicação.
admin-settings-key-cap-oauth-apps = Aplicações OAuth
admin-settings-help-cap-oauth-apps = Política de aplicações OAuth. Consulte o catálogo de funcionalidades para conhecer a cobertura da aplicação.
admin-settings-key-cap-rules = Regras de correio
admin-settings-help-cap-rules = Permitir métodos geridos de regras de correio e de resposta automática de ausência.
admin-settings-key-cap-rules-forward = Reencaminhamento por regras
admin-settings-help-cap-rules-forward = Política de reencaminhamento. Consulte o catálogo de funcionalidades para conhecer a cobertura da aplicação.
admin-settings-key-cap-remote-images = Imagens remotas
admin-settings-help-cap-remote-images = Capacidade de imagens remotas. As definições de privacidade existentes continuam a reger os pedidos; consulte a cobertura da funcionalidade.
admin-settings-key-cap-user-background = Fundo personalizado
admin-settings-help-cap-user-background = Capacidade de fundo personalizado, desativada por predefinição. Consulte a cobertura da funcionalidade.
admin-settings-key-cap-user-css = CSS personalizado
admin-settings-help-cap-user-css = Capacidade de CSS personalizado, desativada por predefinição. Consulte a cobertura da funcionalidade.
admin-settings-key-limit-attachment-bytes = Limite de tamanho dos anexos
admin-settings-help-limit-attachment-bytes = Política de anexos em bytes, predefinição 50000000. A aplicação em execução continua pendente.
admin-settings-key-limit-send-per-hour = Limite de envio por hora
admin-settings-help-limit-send-per-hour = Política de submissão por hora; zero significa ilimitado. A aplicação em execução continua pendente.
admin-settings-key-limit-recipients-per-message = Destinatários por mensagem
admin-settings-help-limit-recipients-per-message = Política do número de destinatários; zero significa ilimitado. A aplicação em execução continua pendente.
admin-settings-cat-import = Importação
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Atualizações em direto
admin-settings-cat-privacy = Privacidade
admin-settings-cat-observability = Observabilidade
admin-settings-cat-egress = Ligações de saída
admin-settings-cat-updates = Atualizações
admin-settings-cat-backups = Cópias de segurança
admin-settings-cat-about = Acerca de
admin-settings-cat-config = Configuração
admin-settings-cat-jobs = Tarefas agendadas
admin-settings-cat-maintenance = Manutenção
admin-settings-cat-health = Verificações de estado
admin-settings-cat-diagnostics = Diagnóstico
admin-settings-cat-users = Utilizadores
admin-settings-cat-server-text = Textos do servidor

# One label and one help sentence per setting (UX4). Keys: crates/mw-server/src/settings_registry.rs
admin-settings-key-ui-language = Idioma
admin-settings-help-ui-language = O idioma do cliente de correio para os utilizadores que não escolheram nenhum. «Igual ao navegador» segue os idiomas de cada navegador.
admin-settings-key-ui-bar-mail-top-bar = Barra de ferramentas do correio
admin-settings-help-ui-bar-mail-top-bar = «pinned» mantém sempre visível a barra de ferramentas do correio com os controlos de disposição. «auto» oculta-a atrás de uma pega fina que a abre quando o utilizador aponta, toca ou navega até ela com a tecla Tab.
admin-settings-key-ui-bar-workspace-tabs = Separadores da área de trabalho
admin-settings-help-ui-bar-workspace-tabs = «pinned» mantém sempre visível a faixa de separadores da área de trabalho. «auto» mostra apenas uma pega fina com o nome do separador ativo até o utilizador a abrir.
admin-settings-key-ui-bar-mail-ribbon = Friso do correio
admin-settings-help-ui-bar-mail-ribbon = Para a disposição com friso. «pinned» mantém o friso aberto. «auto» mostra apenas os nomes dos separadores; um separador abre o friso sobre a lista de mensagens.
admin-settings-key-ui-bar-composer-ribbon = Friso do editor
admin-settings-help-ui-bar-composer-ribbon = Para todos os editores de mensagens. «pinned» mantém o friso do editor aberto. «auto» mostra apenas os nomes dos separadores; abrir um separador empurra o corpo da mensagem para baixo em vez de o tapar.
admin-settings-key-appearance-theme = Tema
admin-settings-help-appearance-theme = O tema de cores que os utilizadores veem até escolherem o seu. Bloqueá-lo impõe este tema a todos os utilizadores.
admin-settings-key-appearance-accent = Cor de destaque
admin-settings-help-appearance-accent = A cor de destaque do tema. Vazio significa a cor de destaque do próprio tema.
admin-settings-key-appearance-badge-color = Cor do contador de não lidas
admin-settings-help-appearance-badge-color = A cor dos contadores de não lidas. «Predefinição» mantém a cor do tema; «Destaque» segue a cor de destaque. Um bloqueio substitui a escolha da conta.
admin-settings-key-appearance-density = Densidade
admin-settings-help-appearance-density = O espaço ocupado pelas listas e controlos: compacta, confortável ou espaçosa.
admin-settings-key-appearance-font = Tipo de letra da interface
admin-settings-key-packages-allowed = Seguimento de encomendas permitido
admin-settings-help-packages-allowed = Permitir o seguimento de encomendas para as contas abrangidas. O fornecedor também tem de estar ativado em «Seguimento de encomendas» e cada utilizador tem de aderir. Desativar esta definição impede de imediato novas chamadas ao fornecedor.
admin-settings-help-appearance-font = Escolha um tipo de letra incluído para as contas sem preferência de tipo de letra ou bloqueie a escolha para todas as contas abrangidas. Os tipos de letra são carregados a partir deste servidor; não é feito nenhum pedido ao Google Fonts. «Predefinição» utiliza o tipo de letra do tema.
admin-settings-key-appearance-brand-name = Nome do produto
admin-settings-help-appearance-brand-name = O nome apresentado na página de início de sessão e no separador do navegador.
admin-settings-key-login-notice = Aviso de início de sessão
admin-settings-help-login-notice = Um texto curto apresentado na página de início de sessão, por exemplo um contacto de suporte. Deixe vazio para não apresentar nenhum.
admin-settings-key-mail-read-receipts = Confirmações de leitura
admin-settings-key-receipts-dsn-allowed = Pedidos de estado da entrega
admin-settings-help-receipts-dsn-allowed = Permitir que as contas peçam notificações SMTP do estado da entrega. Isto não pede nem prova que uma mensagem foi lida.
admin-settings-help-mail-read-receipts = O que acontece quando uma mensagem aberta pede uma confirmação de leitura: perguntar ao utilizador, nunca enviar ou enviar sempre.
admin-settings-key-notifications-enabled = Notificações de correio novo
admin-settings-help-notifications-enabled = Se os utilizadores recebem notificações de correio novo. Não altera as respetivas regras de notificação.
admin-settings-key-offline-retention-days = Idade da cache offline
admin-settings-help-offline-retention-days = Durante quantos dias um dispositivo guarda mensagens para utilização offline. 0 significa sem limite de idade.
admin-settings-key-offline-budget-mb = Tamanho da cache offline
admin-settings-help-offline-budget-mb = Quantos megabytes um dispositivo pode utilizar para correio offline. 0 significa sem cache offline.
admin-settings-key-assistant-enabled = Assistente de IA
admin-settings-help-assistant-enabled = Se os utilizadores veem o assistente de IA. O interruptor de implementação em «Assistente de IA» continua a ter precedência.
admin-settings-key-upstream-default = Servidor de correio predefinido
admin-settings-help-upstream-default = O servidor de correio no qual um endereço inicia sessão quando nenhum domínio indica outro. A tabela de domínios tem precedência sobre este valor.
admin-settings-key-upstream-proxy-timeout-secs = Tempo limite do servidor de correio
admin-settings-help-upstream-proxy-timeout-secs = Quanto tempo pode demorar um pedido ao servidor de correio, em segundos.
admin-settings-key-signin-new-users = Novos utilizadores
admin-settings-help-signin-new-users = Quem pode iniciar sessão pela primeira vez. Alterado em «Política de início de sessão».
admin-settings-key-signin-domain-mode = Domínios de início de sessão
admin-settings-help-signin-domain-mode = Se qualquer domínio pode iniciar sessão ou apenas os domínios listados. Alterado em «Política de início de sessão».
admin-settings-key-signin-refusal-detail = Mensagens de recusa
admin-settings-help-signin-refusal-detail = Se um início de sessão recusado indica o motivo (conta desativada, endereço não permitido) ou dá a mesma resposta que uma palavra-passe errada. Alterado em «Política de início de sessão».
admin-settings-key-signin-ldap-bind-precheck = Verificação LDAP antes do início de sessão
admin-settings-help-signin-ldap-bind-precheck = Verificar a palavra-passe no diretório antes do servidor de correio. Alterado em «Integrações», após um bind de teste.
admin-settings-key-twofa-pending-ttl-secs = Tempo limite do segundo fator
admin-settings-help-twofa-pending-ttl-secs = Quanto tempo o utilizador tem para introduzir o segundo fator após a palavra-passe, em segundos.
admin-settings-key-twofa-max-attempts = Tentativas do segundo fator
admin-settings-help-twofa-max-attempts = Quantos códigos de segundo fator errados são permitidos antes de o início de sessão recomeçar.
admin-settings-key-admin-session-idle-secs = Tempo limite de inatividade da consola
admin-settings-help-admin-session-idle-secs = Quanto tempo dura uma sessão da consola de administração sem pedidos, em segundos.
admin-settings-key-admin-session-absolute-secs = Limite da sessão da consola
admin-settings-help-admin-session-absolute-secs = A duração máxima de uma sessão da consola de administração, em segundos, ativa ou não.
admin-settings-key-passwd-policy-min-length = Comprimento mínimo da palavra-passe
admin-settings-help-passwd-policy-min-length = O número mínimo de carateres que uma nova palavra-passe pode ter.
admin-settings-key-passwd-policy-require-upper = Exigir uma letra maiúscula
admin-settings-help-passwd-policy-require-upper = Uma nova palavra-passe tem de conter uma letra maiúscula.
admin-settings-key-passwd-policy-require-lower = Exigir uma letra minúscula
admin-settings-help-passwd-policy-require-lower = Uma nova palavra-passe tem de conter uma letra minúscula.
admin-settings-key-passwd-policy-require-digit = Exigir um algarismo
admin-settings-help-passwd-policy-require-digit = Uma nova palavra-passe tem de conter um algarismo.
admin-settings-key-passwd-policy-require-symbol = Exigir um símbolo
admin-settings-help-passwd-policy-require-symbol = Uma nova palavra-passe tem de conter um caráter que não seja uma letra nem um algarismo.
admin-settings-key-limit-storage-bytes = Limite de armazenamento
admin-settings-help-limit-storage-bytes = O número máximo de bytes que um utilizador pode armazenar neste servidor. 0 significa sem limite.
admin-settings-key-limit-send-per-day = Mensagens enviadas por dia
admin-settings-help-limit-send-per-day = O número máximo de mensagens que um utilizador pode enviar num dia. 0 significa sem limite.
admin-settings-key-limit-ai-calls-per-day = Chamadas ao assistente de IA por dia
admin-settings-help-limit-ai-calls-per-day = O número máximo de pedidos ao assistente de IA que um utilizador pode fazer num dia. 0 significa sem limite.
admin-settings-key-quota-warn-thresholds = Níveis de aviso de utilização
admin-settings-help-quota-warn-thresholds = As percentagens de um limite a partir das quais os utilizadores são avisados, da mais baixa para a mais alta.
admin-settings-key-quota-warn-enabled = Avisos de utilização
admin-settings-help-quota-warn-enabled = Se os utilizadores são avisados quando se aproximam de um limite.
admin-settings-key-usage-retention-days = Histórico de utilização
admin-settings-help-usage-retention-days = Durante quantos dias são mantidas as contagens diárias de utilização.
admin-settings-key-usage-meter-api = Contabilizar a utilização da API
admin-settings-help-usage-meter-api = Se os pedidos à API são contabilizados nos valores de utilização.
admin-settings-key-usage-measure-interval-hours = Intervalo de medição do armazenamento
admin-settings-help-usage-measure-interval-hours = Com que frequência são medidos os dados armazenados de cada utilizador, em horas.
admin-settings-key-assistant-rate-limit-per-min = Limite de pedidos do assistente de IA
admin-settings-help-assistant-rate-limit-per-min = O número máximo de pedidos ao assistente de IA por minuto para todo o servidor. 0 significa sem limite.
admin-settings-key-plugins-kv-max-key-bytes = Tamanho da chave de plugin
admin-settings-help-plugins-kv-max-key-bytes = O comprimento máximo de uma chave que um plugin pode armazenar, em bytes.
admin-settings-key-plugins-kv-max-value-bytes = Tamanho do valor de plugin
admin-settings-help-plugins-kv-max-value-bytes = O maior valor que um plugin pode armazenar numa chave, em bytes.
admin-settings-key-plugins-kv-max-total-bytes = Armazenamento de plugins
admin-settings-help-plugins-kv-max-total-bytes = O número máximo de bytes que um plugin pode armazenar no total.
admin-settings-key-plugins-kv-max-keys = Chaves de plugin
admin-settings-help-plugins-kv-max-keys = O número máximo de chaves que um plugin pode armazenar.
admin-settings-key-import-max-messages = Número de mensagens por importação
admin-settings-help-import-max-messages = O número máximo de mensagens que uma importação pode trazer.
admin-settings-key-import-max-message-bytes = Tamanho das mensagens importadas
admin-settings-help-import-max-message-bytes = A maior mensagem que uma importação aceita, em bytes.
admin-settings-key-webhooks-max-attempts = Tentativas de webhook
admin-settings-help-webhooks-max-attempts = Quantas vezes uma entrega de webhook é tentada antes de ser abandonada.
admin-settings-key-webhooks-timeout-secs = Tempo limite de webhook
admin-settings-help-webhooks-timeout-secs = Quanto tempo pode demorar uma entrega de webhook, em segundos.
admin-settings-key-outbox-max-attempts = Tentativas de envio
admin-settings-help-outbox-max-attempts = Quantas vezes uma mensagem é entregue ao servidor de correio antes de ser comunicada como não enviada.
admin-settings-key-realtime-recheck-secs = Reverificação das atualizações em direto
admin-settings-help-realtime-recheck-secs = Com que frequência uma ligação de atualizações em direto aberta verifica se a sua sessão continua válida, em segundos.
admin-settings-key-privacy-watermark-enabled = Marca de água no leitor
admin-settings-help-privacy-watermark-enabled = Mostrar uma marca de água com o endereço do leitor sobre as mensagens abertas.
admin-settings-key-privacy-watermark-opacity = Intensidade da marca de água
admin-settings-help-privacy-watermark-opacity = O grau de visibilidade da marca de água, de 0.02 (ténue) a 0.5.
admin-settings-key-privacy-open-tracking-retention-days = Histórico do rastreio de aberturas
admin-settings-help-privacy-open-tracking-retention-days = Durante quantos dias são mantidos os registos de rastreio de leitura.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Tokens de rastreio não enviados
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Durante quantas horas um token de rastreio aguarda o envio da sua mensagem antes de ser eliminado.
admin-settings-key-masked-email-domain = Domínio dos endereços mascarados
admin-settings-help-masked-email-domain = O domínio sob o qual são criados os endereços mascarados. Deixe vazio para desativar os endereços mascarados.
admin-settings-key-abuse-report-address = Endereço para denúncias de abuso
admin-settings-help-abuse-report-address = Para onde são enviadas as denúncias de spam e abuso.
admin-settings-key-dlp-admin-address = Avisos de perda de dados
admin-settings-help-dlp-admin-address = Para onde são enviados os avisos de prevenção de perda de dados.
admin-settings-key-observability-error-forward-url = Endereço de reencaminhamento de erros
admin-settings-help-observability-error-forward-url = Um endereço https para o qual são reencaminhados os relatórios de erros do cliente. Deixe vazio para nenhum.
admin-settings-key-egress-probe-url = Endereço de teste de saída
admin-settings-help-egress-probe-url = O endereço https utilizado para testar um proxy de saída.
admin-settings-key-push-vapid-contact = Contacto push
admin-settings-help-push-vapid-contact = O contacto (mailto: ou https:) fornecido aos serviços push com cada notificação.
admin-settings-key-push-quiet-hours = Horas de silêncio push
admin-settings-help-push-quiet-hours = Horas (UTC) em que não é enviada nenhuma notificação push. Deixe vazio para nenhuma.
admin-settings-key-updates-check = Procurar atualizações
admin-settings-help-updates-check = Se o servidor procura uma versão mais recente uma vez por dia e a mostra no painel.
admin-settings-key-updates-feed-url = Feed de versões
admin-settings-help-updates-feed-url = O endereço https lido pela verificação de atualizações.
admin-settings-key-updates-apply = Aplicar atualizações
admin-settings-help-updates-apply = Se uma atualização pode ser iniciada a partir da consola através do agente de atualização do anfitrião.
admin-settings-key-backup-retention-count = Cópias de segurança mantidas
admin-settings-help-backup-retention-count = Quantas cópias de segurança da base de dados são mantidas.
admin-settings-key-about-security-contact = Contacto de segurança
admin-settings-help-about-security-contact = O contacto em security.txt (mailto: ou https:).
admin-settings-key-about-security-policy-url = Política de segurança
admin-settings-help-about-security-policy-url = O endereço da sua política de segurança, listado em security.txt.
admin-settings-key-about-project-links = Ligações da página «Acerca de»
admin-settings-help-about-project-links = Ligações apresentadas na página «Acerca de», cada uma com uma etiqueta e um endereço https.
admin-settings-key-config-snapshots-kept = Instantâneos da configuração
admin-settings-help-config-snapshots-kept = Quantos instantâneos da configuração são mantidos.
admin-settings-key-jobs-history-days = Histórico de tarefas agendadas
admin-settings-help-jobs-history-days = Durante quantos dias são mantidas as execuções de tarefas agendadas.
admin-settings-key-maintenance-announce-minutes-default = Antecedência do aviso de manutenção
admin-settings-help-maintenance-announce-minutes-default = Com quantos minutos de antecedência os utilizadores são avisados de uma janela de manutenção.
admin-settings-key-maintenance-default-message = Mensagem de manutenção
admin-settings-help-maintenance-default-message = O texto que os utilizadores veem durante a manutenção, por idioma.
admin-settings-key-health-disk-free-min-percent = Mínimo de espaço livre em disco
admin-settings-help-health-disk-free-min-percent = A verificação de prontidão falha quando o espaço livre em disco desce abaixo desta percentagem.
admin-settings-key-observability-otlp-metrics = Enviar métricas por OTLP
admin-settings-help-observability-otlp-metrics = Enviar as métricas do servidor para o coletor OTLP definido no arranque.
admin-settings-key-metrics-drilldown-top-n = Linhas de detalhe das métricas
admin-settings-help-metrics-drilldown-top-n = Quantas linhas mostra uma decomposição de métricas.
admin-settings-key-metrics-rollup-days = Histórico de métricas
admin-settings-help-metrics-rollup-days = Durante quantos dias são mantidos os resumos horários de métricas.
admin-settings-key-diagnostics-retention-days = Histórico de diagnóstico
admin-settings-help-diagnostics-retention-days = Durante quantos dias são mantidas as execuções de diagnóstico.
admin-settings-key-data-export-link-hours = Prazo de transferência da exportação de dados
admin-settings-help-data-export-link-hours = Durante quantas horas uma exportação de dados pode ser transferida antes de ser eliminada.
admin-settings-key-server-text-family = Textos do servidor
admin-settings-help-server-text-family = Texto que o servidor escreve no correio e nas notificações, por idioma e domínio.
admin-settings-cat-sieve = Filtros de correio
admin-settings-key-sieve-max-script-bytes = Tamanho máximo do script de filtro
admin-settings-help-sieve-max-script-bytes = Número máximo de bytes num único script de filtro Sieve.
admin-settings-key-sieve-max-scripts = Número máximo de scripts de filtro
admin-settings-help-sieve-max-scripts = Número máximo de scripts guardados por conta.
admin-settings-key-sieve-max-redirects = Número máximo de redirecionamentos
admin-settings-help-sieve-max-redirects = Número máximo de comandos de redirecionamento num script.
admin-settings-key-sieve-vacation-allowed = Ações de resposta automática nos filtros
admin-settings-help-sieve-vacation-allowed = Permitir que os scripts enviem respostas automáticas de ausência quando o back-end o suporta.
admin-settings-key-sieve-allowed-extensions = Extensões de filtro permitidas
admin-settings-help-sieve-allowed-extensions = Uma lista opcional de nomes de extensões Sieve em minúsculas. O back-end também tem de suportar cada extensão.
admin-settings-key-cap-sieve-raw-edit = Editar o código-fonte do script de filtro
admin-settings-help-cap-sieve-raw-edit = Permitir a edição em bruto de scripts Sieve para as contas selecionadas.
admin-settings-key-cap-sieve-forward-external = Reencaminhar para fora do domínio da conta
admin-settings-help-cap-sieve-forward-external = Permitir o reencaminhamento externo nos scripts. Guardar estes scripts também exige uma verificação recente.
admin-settings-key-session-max-concurrent = Sessões simultâneas
admin-settings-help-session-max-concurrent = Número máximo de sessões por conta. As sessões mais antigas são revogadas quando um novo início de sessão excede este limite.
admin-settings-key-session-idle-secs = Tempo limite de inatividade da sessão
admin-settings-help-session-idle-secs = Terminar as sessões inativas após este número de segundos, até ao limite da implementação.
admin-settings-key-session-absolute-secs = Duração da sessão
admin-settings-help-session-absolute-secs = Terminar as sessões após este número de segundos, até ao limite da implementação.
admin-settings-key-twofa-remember-days = Validade dos dispositivos memorizados
admin-settings-help-twofa-remember-days = Dias até um dispositivo memorizado ter de verificar novamente um segundo fator. Zero desativa esta funcionalidade.
admin-settings-key-twofa-passkeys-enabled = Permitir chaves de acesso
admin-settings-help-twofa-passkeys-enabled = Permitir chaves de acesso como fator de autenticação.
admin-settings-key-twofa-required = Exigir autenticação de dois fatores
admin-settings-help-twofa-required = Exigir um segundo fator registado antes de emitir uma sessão.
admin-settings-key-privacy-remote-content = Conteúdo remoto
admin-settings-help-privacy-remote-content = Bloquear imagens, perguntar primeiro, utilizar o proxy ou carregar diretamente quando permitido.
admin-settings-key-privacy-trackers = Bloquear rastreadores
admin-settings-help-privacy-trackers = Bloquear imagens de rastreio de e-mail conhecidas.
admin-settings-key-privacy-clean-links = Limpar ligações
admin-settings-help-privacy-clean-links = Remover parâmetros de rastreio das ligações das mensagens.
admin-settings-key-privacy-max-bytes = Limite de imagens remotas
admin-settings-help-privacy-max-bytes = Número máximo de bytes obtidos para uma imagem remota.
admin-settings-key-privacy-cache-bytes = Cache de imagens remotas
admin-settings-help-privacy-cache-bytes = Número máximo de bytes mantidos na cache de imagens remotas.
admin-settings-key-privacy-rate-per-min = Ritmo de pedidos de imagens remotas
admin-settings-help-privacy-rate-per-min = Número máximo de obtenções de imagens remotas por conta e por minuto.
admin-settings-key-privacy-allow-direct = Permitir imagens diretas
admin-settings-key-cap-remote-content-direct = Permissão de imagens diretas
admin-settings-help-cap-remote-content-direct = Permitir que esta conta selecione imagens diretas quando o servidor o permite.
admin-settings-help-privacy-allow-direct = Permitir o carregamento direto de imagens para as contas com a capacidade necessária.
admin-settings-key-logs-auth-retention-days = Retenção do histórico de autenticação
admin-settings-help-logs-auth-retention-days = Dias de metadados de autenticação mantidos, independentemente da identidade duradoura da conta.
admin-settings-key-logs-auth-max-entries = Limite de entradas do histórico de autenticação
admin-settings-help-logs-auth-max-entries = Número máximo de entradas mantidas. As falhas repetidas correspondentes num intervalo de um minuto partilham uma entrada.

admin-settings-key-reader-color-mode = Cores das mensagens
admin-settings-cat-reader = Apresentação das mensagens
admin-settings-key-mail-quick-reply = Resposta rápida
admin-settings-help-mail-quick-reply = Oferecer um separador de resposta compacto com os destinatários já preenchidos. Desativado por predefinição; os utilizadores podem ativá-lo, exceto se um administrador o bloquear. As proteções normais de envio continuam a aplicar-se.
admin-settings-key-mail-request-read-receipt = Pedir confirmação de leitura por predefinição
admin-settings-help-mail-request-read-receipt = Iniciar as mensagens enviadas suportadas com o pedido de confirmação selecionado. Trata-se de uma predefinição de escrita, não de uma garantia de que o destinatário enviará uma confirmação.
admin-settings-help-reader-color-mode = «Automático» segue os temas escuros. Escolha a apresentação original, adaptativa ou invertida dentro da moldura isolada da mensagem.
admin-settings-key-reader-font-size = Tamanho do texto das mensagens
admin-settings-help-reader-font-size = Tamanho de leitura predefinido em píxeis, de 12 a 24.
admin-settings-key-reader-line-height = Altura de linha das mensagens
admin-settings-help-reader-line-height = Espaçamento entre linhas de leitura, de 1.2 a 2.2.
admin-settings-key-reader-content-width = Largura de leitura das mensagens
admin-settings-help-reader-content-width = Conteúdo da mensagem com largura confortável, larga ou em todo o painel.
admin-settings-key-reader-loading = Aspeto do carregamento das mensagens
admin-settings-help-reader-loading = Linhas de marcação de posição, um indicador de texto ou nenhum indicador visual. Os anúncios de acessibilidade continuam disponíveis.
admin-settings-key-reader-loading-delay-ms = Atraso do carregamento das mensagens
admin-settings-help-reader-loading-delay-ms = Atrasar apenas o indicador de carregamento, entre 0 e 1000 milissegundos. A apresentação da mensagem nunca é atrasada.

admin-settings-key-login-show-domain-hint = Mostrar o domínio preferido no início de sessão
admin-settings-help-login-show-domain-hint = Mostrar qual o domínio que aceita um nome de utilizador sem sufixo de e-mail. Oculto por predefinição. Isto só altera a sugestão; iniciar sessão com um nome de utilizador continua a funcionar.

admin-settings-key-signin-retry-cooldown-secs = Intervalo entre tentativas de início de sessão
admin-settings-help-signin-retry-cooldown-secs = Número mínimo de segundos entre tentativas de início de sessão com palavra-passe a partir de uma rede cliente. Predefinição 3; 0 desativa o intervalo. Os bloqueios por falhas existentes continuam a aplicar-se.

admin-settings-key-mail-quote-style = Estilo do cabeçalho de resposta e reencaminhamento
admin-settings-key-mail-external-recipient-warning = Aviso de destinatário externo
admin-settings-help-mail-external-recipient-warning = Desativado, avisar ou exigir confirmação no editor de mensagens. Abrange Para, Cc e Bcc. É uma salvaguarda do editor de mensagens, não uma restrição de entrega no servidor.
admin-settings-key-mail-trusted-recipient-domains = Domínios de destinatários fidedignos
admin-settings-help-mail-trusted-recipient-domains = Até 100 domínios de destinatários exatos. O domínio da identidade de envio selecionada também é considerado fidedigno. As entradas inválidas nunca concedem confiança. Bloqueie esta definição para impedir substituições pelos utilizadores.
admin-settings-help-mail-quote-style = Utilizar respostas com citação clássica ou cabeçalhos de mensagem ao estilo do Outlook. Isto define a predefinição do utilizador; bloqueie-a para impor o estilo. Os rascunhos existentes não são alterados.

admin-settings-key-signin-anti-bot-mode = Proteção contra bots no início de sessão
admin-settings-help-signin-anti-bot-mode = Verificação de prova de trabalho alojada localmente: desativada, caixa de correio, administração ou todos. Não é contactado nenhum serviço de verificação externo.
admin-settings-key-signin-anti-bot-difficulty = Trabalho de verificação no início de sessão
admin-settings-help-signin-anti-bot-difficulty = Contador máximo do resolvedor, 100–20000. Valores mais altos exigem mais trabalho do navegador; predefinição 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Validade da verificação de início de sessão
admin-settings-help-signin-anti-bot-expiry-secs = Segundos durante os quais uma verificação de utilização única permanece válida, 60–600; predefinição 180. É necessária uma nova verificação após cada tentativa com credenciais.

admin-settings-key-attachments-preview-allowed = Permitir pré-visualizações de anexos
admin-settings-help-attachments-preview-allowed = Desative para impedir a apresentação de pré-visualizações de anexos; as transferências originais têm o seu próprio controlo.

admin-settings-key-attachments-download-allowed = Permitir a entrega dos anexos originais
admin-settings-help-attachments-download-allowed = Desativar também impede as pré-visualizações que exigem os bytes originais. As pré-visualizações no navegador não são uma proteção contra cópia.

admin-settings-key-attachments-max-bytes = Bytes máximos por anexo
admin-settings-help-attachments-max-bytes = Zero significa ilimitado. Aplica-se aos bytes originais entregues, também nos pedidos de intervalo.

admin-settings-key-attachments-allowed-extensions = Extensões de ficheiro permitidas
admin-settings-help-attachments-allowed-extensions = Matriz JSON de extensões em minúsculas sem ponto, como ["pdf","tar.gz"]. Vazio permite qualquer extensão; as regras de bloqueio prevalecem.

admin-settings-key-attachments-blocked-extensions = Extensões de ficheiro bloqueadas
admin-settings-help-attachments-blocked-extensions = Matriz JSON de extensões em minúsculas. Avaliadas em relação aos nomes de ficheiro MIME guardados, nunca ao nome no URL de transferência.

admin-settings-key-attachments-warn-extensions = Avisar para extensões de ficheiro
admin-settings-help-attachments-warn-extensions = Matriz JSON de extensões que exigem confirmação explícita antes da entrega do original.

admin-settings-key-attachments-allowed-mime-types = Tipos MIME permitidos
admin-settings-help-attachments-allowed-mime-types = Matriz JSON de tipos em minúsculas ou de carateres universais de nível superior, como ["image/*","application/pdf"]. Vazio permite qualquer tipo.

admin-settings-key-attachments-blocked-mime-types = Tipos MIME bloqueados
admin-settings-help-attachments-blocked-mime-types = Matriz JSON de tipos em minúsculas ou de carateres universais de nível superior. As regras de bloqueio têm precedência.

admin-settings-key-attachments-warn-mime-types = Avisar para tipos MIME
admin-settings-help-attachments-warn-mime-types = Matriz JSON de tipos em minúsculas ou de carateres universais de nível superior que exigem confirmação. Os metadados MIME não são deteção de malware.

admin-settings-cat-shortcuts = Atalhos de teclado
admin-settings-cat-time = Fusos horários
admin-settings-key-time-display-zone = Fuso horário de apresentação
admin-settings-help-time-display-zone = Fuso horário IANA, como Europe/Lisbon, ou local para seguir o dispositivo atual. As horas das mensagens continuam guardadas como os instantes originais.
admin-settings-key-time-comparison-zone = Fuso horário de comparação
admin-settings-help-time-comparison-zone = Segundo fuso horário IANA opcional para os auxiliares de mensagens e calendário. Deixe vazio para ocultar a comparação; nenhuma localização é inferida a partir de um endereço de e-mail.
admin-settings-key-time-show-helpers = Mostrar auxiliares de fuso horário
admin-settings-help-time-show-helpers = Mostrar comparações entre a hora local e a do fuso selecionado nas horas das mensagens e nos eventos do calendário. Os bloqueios de administrador podem impor esta escolha.
admin-settings-key-shortcuts-enabled = Ativar atalhos de teclado
admin-settings-help-shortcuts-enabled = Permitir os atalhos da aplicação. Os bloqueios podem desativá-los globalmente ou para um domínio. Os campos de texto e as caixas de diálogo modais mantêm o seu tratamento normal do teclado.
admin-settings-key-shortcuts-compose = Atalho de escrever
admin-settings-help-shortcuts-compose = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-archive = Atalho de arquivar
admin-settings-help-shortcuts-archive = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-reply = Atalho de responder
admin-settings-help-shortcuts-reply = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-reply-all = Atalho de responder a todos
admin-settings-help-shortcuts-reply-all = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-forward = Atalho de reencaminhar
admin-settings-help-shortcuts-forward = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-next = Atalho da mensagem seguinte
admin-settings-help-shortcuts-next = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-previous = Atalho da mensagem anterior
admin-settings-help-shortcuts-previous = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-search = Atalho de focar a pesquisa
admin-settings-help-shortcuts-search = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-refresh = Atalho de atualizar mensagens
admin-settings-help-shortcuts-refresh = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-settings = Atalho de abrir definições
admin-settings-help-shortcuts-settings = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-trash = Atalho de mover para o lixo
admin-settings-help-shortcuts-trash = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-mark-read = Atalho de marcar como lida
admin-settings-help-shortcuts-mark-read = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-mark-unread = Atalho de marcar como não lida
admin-settings-help-shortcuts-mark-unread = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.
admin-settings-key-shortcuts-spam = Atalho de marcar como spam
admin-settings-help-shortcuts-spam = Utilize inherit para a predefinição selecionada, disabled para remover esta ação ou uma combinação como Ctrl+Shift+K. As combinações em conflito ou inválidas não são executadas.

admin-settings-key-notifications-push-allowed = Notificações push em segundo plano permitidas
admin-settings-help-notifications-push-allowed = Permitir que as notificações ativadas desta conta se registem e recebam push em segundo plano. A permissão do navegador continua a ser necessária.
admin-settings-key-offline-persistent-storage-allowed = Pedidos de armazenamento persistente no navegador permitidos
admin-settings-help-offline-persistent-storage-allowed = Permitir um pedido explícito para proteger os dados locais em cache contra a remoção automática pelo navegador. O navegador pode recusar; isto não concede a permissão automaticamente.

admin-settings-key-security-signin-mailbox-failure-limit = Limite de credenciais falhadas da caixa de correio
admin-settings-help-security-signin-mailbox-failure-limit = As credenciais rejeitadas a partir de um IP de origem dentro da janela de contagem desencadeiam um bloqueio temporário do início de sessão na caixa de correio. As falhas de rede e de configuração não contam.
admin-settings-key-security-signin-admin-failure-limit = Limite de credenciais falhadas de administrador
admin-settings-help-security-signin-admin-failure-limit = As credenciais de administrador rejeitadas a partir de um IP de origem desencadeiam um bloqueio temporário separado do início de sessão de administração. A sessão de administração curta e os requisitos de prova recente mantêm-se inalterados.
admin-settings-key-security-signin-failure-window-secs = Janela de contagem de credenciais falhadas
admin-settings-help-security-signin-failure-window-secs = Janela deslizante em segundos para rejeições de credenciais e MFA. Um início de sessão concluído com êxito repõe essa origem e essa área.
admin-settings-key-security-signin-lock-secs = Duração do bloqueio temporário do início de sessão
admin-settings-help-security-signin-lock-secs = Duração do bloqueio em segundos. As tentativas repetidas durante o bloqueio não prolongam a sua expiração original. As proibições manuais globais de IP são independentes.

admin-settings-key-security-signin-geo-scope = Âmbito da política de países no início de sessão
admin-settings-help-security-signin-geo-scope = Desativado por predefinição. Aplicar apenas a novos inícios de sessão na caixa de correio, de administrador ou a todos. As sessões existentes não são revogadas. Consulte o diagnóstico da origem atual antes de ativar; uma base de dados GeoIP local indisponível recusa o início de sessão.
admin-settings-key-security-signin-geo-mode = Lista de países permitidos ou negados
admin-settings-help-security-signin-geo-mode = «Permitir» admite apenas os países listados; «negar» recusa os países listados. Os endereços desconhecidos utilizam a sua própria política explícita.
admin-settings-key-security-signin-geo-countries = Códigos de país do início de sessão
admin-settings-help-security-signin-geo-countries = Matriz JSON de códigos de país de duas letras maiúsculas, por exemplo ["PT", "GB"]. Utiliza apenas o MW_GEOIP_DB fornecido pelo operador; não é contactado nenhum serviço de localização externo.
admin-settings-key-security-signin-geo-unknown = Política para endereços de origem sem correspondência
admin-settings-help-security-signin-geo-unknown = Permitir ou negar endereços ausentes de uma base de dados local legível, endereços privados incluídos. Bases de dados em falta ou ilegíveis recusam sempre os inícios de sessão com a política ativa, mesmo quando os endereços desconhecidos são permitidos.

admin-settings-key-contacts-avatar-mode = Fotografias de contactos e avatares gerados
admin-settings-help-contacts-avatar-mode = Gerar iniciais localmente, ocultar avatares ou utilizar fotografias incorporadas dos livros de endereços de cada conta. Não são contactados URLs de fotografias remotas nem serviços de avatares externos. A política de utilizador e de domínio pode ser fixada por um administrador.
admin-settings-cat-contacts = Fotografias de contactos

admin-settings-key-reader-action-style = Friso de ações da mensagem
admin-settings-help-reader-action-style = Os ícones compactos são a predefinição. Escolha etiquetas para ícones com texto visível; ambos mantêm os nomes das ações e as dicas com o tema. Suporta bloqueios de política globais, de domínio e de utilizador.
admin-settings-key-nextcloud-sync-interval-secs = Intervalo de atualização automática do Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Segundos entre atualizações das coleções em segundo plano para as contas associadas com o motor nativo. Zero significa apenas manual; os valores positivos têm um mínimo de sessenta segundos. As falhas aumentam o intervalo. Não é tentado nenhum início de sessão em segundo plano.
admin-settings-key-mail-folder-role-inbox = Mapeamento da pasta Caixa de entrada
admin-settings-help-mail-folder-role-inbox = Null utiliza a deteção automática. Utilize um objeto com o caminho exato para predefinições partilhadas ou um objeto com o ID de uma caixa de correio própria para uma conta. Destinos em falta ou em conflito desativam esta função; aplicam-se os bloqueios de administrador.
admin-settings-key-mail-folder-role-drafts = Mapeamento da pasta Rascunhos
admin-settings-help-mail-folder-role-drafts = Null utiliza a deteção automática. Utilize um objeto com o caminho exato para predefinições partilhadas ou um objeto com o ID de uma caixa de correio própria para uma conta. Destinos em falta ou em conflito desativam esta função; aplicam-se os bloqueios de administrador.
admin-settings-key-mail-folder-role-sent = Mapeamento da pasta Enviados
admin-settings-help-mail-folder-role-sent = Null utiliza a deteção automática. Utilize um objeto com o caminho exato para predefinições partilhadas ou um objeto com o ID de uma caixa de correio própria para uma conta. Destinos em falta ou em conflito desativam esta função; aplicam-se os bloqueios de administrador.
admin-settings-key-mail-folder-role-trash = Mapeamento da pasta Lixo
admin-settings-help-mail-folder-role-trash = Null utiliza a deteção automática. Utilize um objeto com o caminho exato para predefinições partilhadas ou um objeto com o ID de uma caixa de correio própria para uma conta. Destinos em falta ou em conflito desativam esta função; aplicam-se os bloqueios de administrador.
admin-settings-key-mail-folder-role-junk = Mapeamento da pasta Correio indesejado
admin-settings-help-mail-folder-role-junk = Null utiliza a deteção automática. Utilize um objeto com o caminho exato para predefinições partilhadas ou um objeto com o ID de uma caixa de correio própria para uma conta. Destinos em falta ou em conflito desativam esta função; aplicam-se os bloqueios de administrador.
admin-settings-key-mail-folder-role-archive = Mapeamento da pasta Arquivo
admin-settings-help-mail-folder-role-archive = Null utiliza a deteção automática. Utilize um objeto com o caminho exato para predefinições partilhadas ou um objeto com o ID de uma caixa de correio própria para uma conta. Destinos em falta ou em conflito desativam esta função; aplicam-se os bloqueios de administrador.

admin-settings-key-mail-request-delivery-receipt = Pedir notificações de entrega por predefinição
admin-settings-help-mail-request-delivery-receipt = Selecionar notificações de entrega para as novas mensagens quando a conta suporta SMTP DSN. O bloqueio controla esta predefinição guardada; receipts.dsnAllowed determina separadamente se os pedidos são permitidos. A entrega não prova a leitura.

admin-settings-key-security-signin-geo-enforce-sessions = Aplicar a política de países às sessões existentes
admin-settings-help-security-signin-geo-enforce-sessions = Desativado por predefinição. Aplicar o âmbito configurado da política de países do início de sessão aos pedidos autenticados, ao acesso nativo por bearer e ao conteúdo assinado do leitor. Os clientes bloqueados continuam a poder terminar sessão. Os fluxos já abertos são verificados quando voltam a ligar-se; isto não termina as ligações existentes.

admin-settings-key-security-protection-rate-mode = Orçamentos agregados de pedidos
admin-settings-help-security-protection-rate-mode = Desativado por predefinição. «Observe» regista o esgotamento do orçamento sem recusar; «enforce» devolve um 429 temporário. Os limites de concorrência dos protocolos existentes mantêm-se independentes.
admin-settings-key-security-protection-adaptive-mode = Proteção adaptativa de pedidos
admin-settings-help-security-protection-adaptive-mode = Desativado por predefinição. «Observe» regista anomalias tipificadas explicáveis e as ações propostas; «enforce» aplica limitações ou bloqueios temporários delimitados. É uma política determinística de contadores, não um classificador de IA.
admin-settings-key-security-protection-client-requests = Pedidos por cliente e por janela
admin-settings-help-security-protection-client-requests = Quantidade agregada de pedidos permitida entre réplicas para esta dimensão de sujeito verificado. Zero desativa esta dimensão; os clientes de origem utilizam apenas o resolvedor de endereços do proxy fidedigno.
admin-settings-key-security-protection-account-requests = Pedidos por conta e por janela
admin-settings-help-security-protection-account-requests = Quantidade agregada de pedidos permitida entre réplicas para esta dimensão de sujeito verificado. Zero desativa esta dimensão; os clientes de origem utilizam apenas o resolvedor de endereços do proxy fidedigno.
admin-settings-key-security-protection-admin-requests = Pedidos de administrador por janela
admin-settings-help-security-protection-admin-requests = Quantidade agregada de pedidos permitida entre réplicas para esta dimensão de sujeito verificado. Zero desativa esta dimensão; os clientes de origem utilizam apenas o resolvedor de endereços do proxy fidedigno.
admin-settings-key-security-protection-rate-window-secs = Janela do orçamento de pedidos (segundos)
admin-settings-help-security-protection-rate-window-secs = Intervalo fixo do orçamento de pedidos. A rejeição de carga por capacidade e as respostas comuns de não encontrado não são prova adaptativa.
admin-settings-key-security-protection-adaptive-clients = Observar clientes para a proteção adaptativa
admin-settings-help-security-protection-adaptive-clients = Ativar esta dimensão de sujeito quando o modo adaptativo está ativo. As contas e os administradores só são contabilizados depois de a sua identidade ser verificada.
admin-settings-key-security-protection-adaptive-accounts = Observar contas para a proteção adaptativa
admin-settings-help-security-protection-adaptive-accounts = Ativar esta dimensão de sujeito quando o modo adaptativo está ativo. As contas e os administradores só são contabilizados depois de a sua identidade ser verificada.
admin-settings-key-security-protection-adaptive-admins = Observar administradores para a proteção adaptativa
admin-settings-help-security-protection-adaptive-admins = Ativar esta dimensão de sujeito quando o modo adaptativo está ativo. As contas e os administradores só são contabilizados depois de a sua identidade ser verificada.
admin-settings-key-security-protection-short-window-secs = Janela curta de anomalias (segundos)
admin-settings-help-security-protection-short-window-secs = Janela de observação deslizante. As janelas longas suportam de horas até trinta dias; o arredondamento por buckets pode reter, no máximo, um sexagésimo quarto adicional de uma janela.
admin-settings-key-security-protection-short-minimum-samples = Amostras mínimas da proporção curta
admin-settings-help-security-protection-short-minimum-samples = Número mínimo de pedidos normais e anómalos observados antes de se aplicar o acionador por proporção. O limiar absoluto de anomalias é independente deste mínimo.
admin-settings-key-security-protection-short-absolute-anomalies = Orçamento absoluto curto de anomalias
admin-settings-help-security-protection-short-absolute-anomalies = Acionar quando se acumula este número de anomalias selecionadas, mesmo quando os pedidos normais diluem a proporção. Zero desativa este acionador.
admin-settings-key-security-protection-short-ratio-percent = Proporção curta de anomalias (percentagem)
admin-settings-help-security-protection-short-ratio-percent = Acionar quando as anomalias selecionadas atingem esta percentagem após o número mínimo de amostras. Basta o acionador absoluto ou o acionador por proporção. Zero desativa este acionador.
admin-settings-key-security-protection-long-window-secs = Janela longa de anomalias (segundos)
admin-settings-help-security-protection-long-window-secs = Janela de observação deslizante. As janelas longas suportam de horas até trinta dias; o arredondamento por buckets pode reter, no máximo, um sexagésimo quarto adicional de uma janela.
admin-settings-key-security-protection-long-minimum-samples = Amostras mínimas da proporção longa
admin-settings-help-security-protection-long-minimum-samples = Número mínimo de pedidos normais e anómalos observados antes de se aplicar o acionador por proporção. O limiar absoluto de anomalias é independente deste mínimo.
admin-settings-key-security-protection-long-absolute-anomalies = Orçamento absoluto longo de anomalias
admin-settings-help-security-protection-long-absolute-anomalies = Acionar quando se acumula este número de anomalias selecionadas, mesmo quando os pedidos normais diluem a proporção. Zero desativa este acionador.
admin-settings-key-security-protection-long-ratio-percent = Proporção longa de anomalias (percentagem)
admin-settings-help-security-protection-long-ratio-percent = Acionar quando as anomalias selecionadas atingem esta percentagem após o número mínimo de amostras. Basta o acionador absoluto ou o acionador por proporção. Zero desativa este acionador.
admin-settings-key-security-protection-normal-cap-per-bucket = Limite da base normal por bucket
admin-settings-help-security-protection-normal-cap-per-bucket = Limitar a base normal aprendida em cada bucket da janela deslizante. Os orçamentos absolutos de anomalias independentes impedem que o enchimento ilimitado com pedidos válidos oculte abusos.
admin-settings-key-security-protection-signals = Sinais de prova adaptativa
admin-settings-help-security-protection-signals = Selecione observações fidedignas de pedidos malformados, demasiado grandes, credenciais rejeitadas, permissões negadas, prazo do corpo excedido ou, opcionalmente, rotas de administração inexistentes. A sondagem de rotas de administração exclui as respostas de entidade não encontrada e os recursos estáticos; códigos de estado genéricos, recusas por capacidade e cadeias de user-agent não comprovam abuso.
admin-settings-key-security-protection-throttle-secs = Limitação temporária inicial (segundos)
admin-settings-help-security-protection-throttle-secs = A primeira ultrapassagem do limiar inicia um prazo de limitação fixo. Os pedidos durante esse prazo não o prolongam.
admin-settings-key-security-protection-block-secs = Bloqueio temporário agravado (segundos)
admin-settings-help-security-protection-block-secs = Uma ultrapassagem posterior do limiar antes da recuperação pode agravar-se para este bloqueio delimitado. Não é criado nenhum bloqueio permanente de conta.
admin-settings-key-security-protection-decay-secs = Intervalo de recuperação sem atividade (segundos)
admin-settings-help-security-protection-decay-secs = Após este intervalo sem anomalias e a expiração da ação atual, o agravamento regressa à fase inicial. A prova acumulada expira de acordo com as janelas configuradas.
admin-settings-key-security-protection-exempt-clients = Endereços e redes de clientes isentos
admin-settings-help-security-protection-exempt-clients = Endereços IP exatos ou redes CIDR que isentam apenas a dimensão de cliente. Os limites de contas verificadas continuam a aplicar-se. Utilize deliberadamente para redes NAT partilhadas; os endereços reencaminhados só são considerados fidedignos através dos proxies configurados.
admin-settings-key-security-protection-exempt-accounts = IDs de contas verificadas isentas
admin-settings-help-security-protection-exempt-accounts = IDs exatos de contas de caixa de correio verificados pelo servidor que isentam apenas a sua dimensão de conta, nunca a dimensão de cliente de origem. Os endereços de e-mail e os parâmetros de conta fornecidos pelo chamador não constituem prova de identidade.
admin-settings-key-security-protection-exempt-admins = IDs de administradores isentos
admin-settings-help-security-protection-exempt-admins = IDs exatos de administradores autenticados que isentam apenas a sua dimensão de administrador. Os bloqueios existentes por falhas de início de sessão e os prazos estritos das sessões de administrador mantêm-se inalterados.
admin-settings-key-security-protection-storage-failure = Política para falhas do armazenamento de aplicação
admin-settings-help-security-protection-storage-failure = «Closed» recusa o trabalho protegido quando não é possível verificar um orçamento de aplicação ativado. «Open» permite-o temporariamente com um aviso operacional delimitado. As falhas em modo só de observação nunca impõem uma ação de aplicação.

admin-settings-key-appearance-custom-css-enabled = Permitir CSS de aspeto pessoal
admin-settings-help-appearance-custom-css-enabled = Permitir estilos visuais restritos na área de trabalho do proprietário. Desativado por predefinição; os bloqueios de aspeto impostos à caixa de correio suspendem estes estilos.
admin-settings-key-appearance-custom-js-enabled = Permitir scripts de aspeto pessoal
admin-settings-help-appearance-custom-js-enabled = Permitir scripts isolados que só podem pedir alterações de aspeto. Desativado por predefinição; os scripts não conseguem aceder ao correio, aos dados da conta nem ao DOM da aplicação.

admin-settings-key-mail-modifier-selection = Seleção de mensagens com teclas modificadoras
admin-settings-help-mail-modifier-selection = Permitir a seleção de intervalos com Ctrl, Command e Shift. Bloqueie este valor para o impor; os controlos de seleção visíveis continuam disponíveis.
admin-settings-key-mail-drag-to-folder = Arrastar mensagens para pastas
admin-settings-help-mail-drag-to-folder = Permitir arrastar mensagens da conta para pastas reais. Isto não concede permissão de escrita na caixa de correio; o servidor continua a autorizar cada movimentação.

admin-settings-key-admin-allow-last-superadmin-loss = Permitir perder o último Super admin
admin-settings-help-admin-allow-last-superadmin-loss = Desativado por predefinição. Ativar isto permite que alterações posteriores removam o último acesso de Super admin. Um Super admin completo tem de confirmar a consequência com uma autenticação recente.
admin-settings-last-superadmin-title = Permitir perder todo o acesso de Super admin?
admin-settings-last-superadmin-body = Alterações posteriores às contas podem deixar ninguém capaz de administrar esta implementação. A recuperação exigiria administração local da implementação. Escreva ALLOW LAST SUPERADMIN LOSS para ativar esta exceção.
admin-settings-last-superadmin-confirm = Permitir a perda de acesso

admin-settings-key-mail-folder-counts = Contagens das pastas
admin-settings-help-mail-folder-counts = Escolha ocultas, não lidas, total ou ambas as contagens junto às pastas. Defina predefinições globais, de domínio ou de utilizador e bloqueie a escolha quando necessário.
admin-settings-key-mail-compact-folder-badges = Contadores compactos nas pastas secundárias
admin-settings-help-mail-compact-folder-badges = Mostrar contadores mais pequenos fora da Caixa de entrada. Os utilizadores podem escolher o seu próprio tamanho, exceto se esta definição estiver bloqueada.

admin-settings-key-privacy-remote-image-downloads = Transferências manuais de imagens remotas
admin-settings-key-privacy-remote-images-toast = Notificação de carregamento de imagens
admin-settings-help-privacy-remote-images-toast = Mostrar uma notificação no leitor enquanto as imagens remotas de uma mensagem carregam através do proxy de privacidade, e indicar quando carregaram ou quantas não foi possível carregar. Ativado por predefinição. Só altera o que o leitor mostra; as imagens que carregam continuam a ser decididas pela política de conteúdo remoto.
admin-settings-help-privacy-remote-image-downloads = Permitir a transferência explícita de uma imagem referenciada pela mensagem própria através do proxy de privacidade protegido. Desativado por predefinição; isto nunca abre diretamente o URL externo.
admin-settings-key-mail-resume-tabs = Retomar separadores da área de trabalho
admin-settings-help-mail-resume-tabs = Restaurar as referências dos separadores da conta após recarregar. Desativar isto limpa o registo local da área de trabalho sem eliminar rascunhos.

admin-settings-key-metrics-default-range-seconds = Intervalo predefinido do histórico de pedidos
admin-settings-help-metrics-default-range-seconds = Segundos apresentados quando um pedido de histórico não especifica um intervalo. Limitado pela retenção e pelo intervalo máximo.
admin-settings-key-metrics-max-range-seconds = Intervalo máximo do histórico de pedidos
admin-settings-help-metrics-max-range-seconds = Número máximo de segundos por consulta de histórico. A retenção continua a ser um limite adicional.

admin-settings-key-contacts-sender-icons = Ícones de remetente incluídos
admin-settings-help-contacts-sender-icons = Mostrar ícones incluídos localmente para domínios de remetente reconhecidos, com iniciais geradas para os restantes. As fotografias de contactos têm prioridade. Os ícones não verificam a identidade do remetente. Não é contactado nenhum serviço de logótipos externo; as substituições de utilizador e de domínio podem ser bloqueadas.

admin-settings-key-mail-draft-save-mode = Modo de gravação de rascunhos da conta
admin-settings-help-mail-draft-save-mode = Guardar após uma pausa na escrita, periodicamente ou apenas a pedido. A gravação exige uma pasta Rascunhos suportada no servidor de origem; as escritas incertas nunca são repetidas automaticamente.
admin-settings-key-mail-draft-debounce-ms = Pausa na escrita de rascunhos (milissegundos)
admin-settings-help-mail-draft-debounce-ms = Aguardar 500 a 10000 milissegundos após a edição antes de guardar no modo em tempo real.
admin-settings-key-mail-draft-interval-secs = Intervalo de gravação de rascunhos (segundos)
admin-settings-help-mail-draft-interval-secs = Guardar os rascunhos editados a cada 5 a 3600 segundos no modo periódico. Os rascunhos sem alterações não são escritos novamente.

admin-settings-key-login-remember-signins = Memorizar endereços de inícios de sessão bem-sucedidos
admin-settings-help-login-remember-signins = Manter neste dispositivo até dez nomes de utilizador ou endereços de e-mail utilizados com êxito, para a lista pendente de início de sessão. Nunca guarda palavras-passe nem sessões. Desativar remove os endereços guardados deste servidor quando um dispositivo voltar a carregar a página de início de sessão ou concluir um início de sessão; não controla o preenchimento automático do próprio navegador.
admin-settings-key-login-show-remembered-hint = Mostrar explicação dos endereços guardados
admin-settings-help-login-show-remembered-hint = Mostrar texto explicativo na lista pendente de inícios de sessão anteriores. Oculto por predefinição. Só se aplica quando a memorização de inícios de sessão bem-sucedidos está ativada.

admin-settings-key-mail-show-from = Mostrar o campo De no editor de mensagens
admin-settings-help-mail-show-from = Mostrar o campo De por predefinição nos novos editores de mensagens. Os utilizadores podem mostrá-lo para uma mensagem; só os endereços autorizados podem enviar.
admin-settings-key-accounts-linking-enabled = Permitir contas de caixa de correio associadas
admin-settings-help-accounts-linking-enabled = Desativado por predefinição. A desativação global bloqueia todo o acesso associado; ambas as contas também têm de cumprir a política do respetivo domínio e conta. A associação exige uma autenticação real da conta secundária e uma prova recente da conta principal.
admin-settings-key-accounts-use-linked = Utilizar caixas de correio associadas
admin-settings-help-accounts-use-linked = Exclusão voluntária do utilizador do acesso associado. Não pode substituir a desativação administrativa. Os dados da caixa de correio são preservados quando o acesso é desativado.
admin-settings-key-accounts-max-linked = Número máximo de caixas de correio associadas
admin-settings-help-accounts-max-linked = Número máximo de caixas de correio secundárias verificadas por identidade principal, de 1 a 16. Predefinição 5.

admin-settings-key-privacy-strip-mailer-headers = Remover cabeçalhos do cliente de correio enviados
admin-settings-help-privacy-strip-mailer-headers = Remover X-Mailer, User-Agent e X-MimeOLE de nível superior das mensagens SMTP e EWS nativas. As mensagens anexadas e os conteúdos dos relatórios permanecem intactos. Os envios através do proxy JMAP do servidor de origem não estão abrangidos.
