# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Administração do Mailwoman
admin-nav = Seções de administração
admin-sign-out = Sair
admin-nav-domains = Domínios
admin-nav-users = Usuários
admin-nav-integrations = Integrações
admin-nav-observability = Observabilidade
admin-nav-plugins = Plugins
admin-nav-assist = Assistente de IA
admin-nav-sso = Login único (SSO)
admin-nav-servermeta = Metadados do servidor
admin-nav-rethread = Manutenção
admin-nav-2fa = Exigir dois fatores

# Shared admin actions / states
admin-delete = Excluir
admin-revoke = Revogar
admin-remove = Remover
admin-saved = Salvo.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Login de administrador
admin-login-note = Este painel funciona com uma sessão de administrador separada. A sessão termina após 30 minutos sem atividade e, no máximo, 12 horas após o login.
admin-login-session-ended = Sua sessão de administrador terminou. Entre novamente.
admin-login-username = Nome de usuário do administrador
admin-login-password = Senha
admin-login-sign-in = Entrar
admin-login-signing-in = Entrando…
admin-login-invalid = Credenciais de administrador inválidas
admin-login-unreachable = Não foi possível acessar o servidor

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domínios
admin-domains-load-error = Não foi possível carregar os domínios
admin-domains-save-error = Não foi possível salvar o domínio
admin-domains-delete-error = Não foi possível excluir o domínio
admin-domains-add = Adicionar domínio
admin-domains-name = Nome do domínio
admin-domains-name-placeholder = example.com
admin-domains-note = Um domínio é registrado pelo nome. Os nomes registrados são oferecidos na tela Exigir dois fatores quando uma regra se aplica a um domínio. Registrar um domínio não roteia nem filtra e-mails.
admin-domains-save = Salvar domínio
admin-domains-empty = Nenhum domínio ainda.
admin-domains-delete-for = Excluir { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Usuários
admin-users-load-error = Não foi possível carregar os usuários
admin-users-provision-error = Não foi possível provisionar o usuário
admin-users-flag-error = Não foi possível atualizar o sinalizador
admin-users-revoke-error = Não foi possível revogar as sessões
admin-users-provision = Provisionar usuário
admin-users-username = Nome de usuário
admin-users-domain = Domínio
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Cota em bytes (0 = ilimitada)
admin-users-quota-msgs = Cota de mensagens (0 = ilimitada)
admin-users-empty = Nenhum usuário ainda.
admin-users-col-account = Conta
admin-users-col-quota = Cota (bytes/mensagens)
admin-users-col-zeroaccess = Acesso zero
admin-users-col-flags = Sinalizadores
admin-users-col-sessions = Sessões
admin-users-zeroaccess-for = Acesso zero para { $account }
admin-users-zeroaccess-help = Apenas um registro. O armazenamento de acesso zero é ativado e desativado pelo próprio usuário, nas configurações dele, com a chave dele. Marcar esta caixa não criptografa os e-mails armazenados da conta, e desmarcá-la não os descriptografa.
admin-users-disable-for = Desativar { $account }
admin-users-disabled = desativada
admin-users-disabled-help = Bloqueia o login e encerra todas as sessões abertas desta conta. As chaves de API e os tokens dela são recusados. Isso não desativa a caixa postal no servidor de e-mail: outros clientes de e-mail ainda podem acessá-la até que você a desative lá também.
admin-users-force-change-for = Forçar troca de senha para { $account }
admin-users-force-change = forçar troca
admin-users-force-change-help = O usuário fica retido em uma tela de troca de senha, no próximo login e nas sessões já abertas, e não consegue acessar os e-mails até que a troca seja concluída. Isso exige um backend de troca de senha funcionando (MW_PASSWD_BACKEND). Sem ele, a troca não pode ser concluída e o usuário permanece nessa tela até você desmarcar esta caixa.
admin-users-revoke-for = Revogar sessões de { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integrações
admin-integrations-load-error = Não foi possível carregar as integrações
admin-integrations-revoke-error = Não foi possível revogar a chave
admin-integrations-ldap = Diretório LDAP / GAL
admin-integrations-nextcloud = Ponte Nextcloud
admin-integrations-active = Ativa
admin-integrations-configured = Configurada
admin-integrations-not-configured = Não configurada
admin-integrations-unknown = Status desconhecido
admin-integrations-config-note = “Configurada” significa que esta implantação tem configurações para a integração: uma entrada de diretório LDAP ativada no banco de dados, ou as três variáveis de ambiente MW_NEXTCLOUD_*. Ambas são lidas quando o servidor inicia. Não significa que o serviço remoto foi contatado nem que está acessível.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = Nenhum webhook registrado.
admin-integrations-keys = Chaves de API e MCP
admin-integrations-keys-empty = Nenhuma chave emitida.
admin-integrations-col-prefix = Prefixo
admin-integrations-col-account = Conta
admin-integrations-col-scopes = Escopos
admin-integrations-col-status = Status
admin-integrations-status-revoked = revogada
admin-integrations-status-active = ativa
admin-integrations-revoke-key = Revogar chave { $prefix }
admin-integrations-col-unattended = Envio sem supervisão
admin-integrations-unattended-not-requested = Não solicitado
admin-integrations-unattended-requested = Solicitado, não aprovado
admin-integrations-unattended-approved = Aprovado
admin-integrations-unattended-approve = Aprovar
admin-integrations-unattended-withdraw = Retirar
admin-integrations-unattended-approve-key = Aprovar envio sem supervisão para a chave { $prefix }
admin-integrations-unattended-withdraw-key = Retirar a aprovação do envio sem supervisão para a chave { $prefix }
admin-integrations-unattended-approve-title = Aprovar o envio sem supervisão para esta chave?
admin-integrations-unattended-withdraw-title = Retirar a aprovação desta chave?
admin-integrations-unattended-owner = Proprietário: { $account }
admin-integrations-unattended-key = Chave: { $prefix }
admin-integrations-unattended-scope = Permissões: { $permissions }. Ferramentas MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = Não foi possível ler o escopo desta chave.
admin-integrations-scope-read = ler
admin-integrations-scope-send = enviar
admin-integrations-scope-delete = excluir
admin-integrations-scope-mail = e-mail
admin-integrations-scope-pim = agenda, tarefas, notas e contatos
admin-integrations-scope-none = nenhum
admin-integrations-unattended-approve-meaning = As mensagens enviadas com esta chave pelo MCP são transmitidas sem que uma pessoa as libere.
admin-integrations-unattended-withdraw-meaning = Um envio feito com esta chave pelo MCP é recusado até que a chave seja aprovada novamente. A mensagem não é transmitida nem colocada na Caixa de saída do proprietário.
admin-integrations-unattended-approve-effect = A aprovação entra em vigor imediatamente, a partir do próximo envio da chave. Não é necessário reiniciar.
admin-integrations-unattended-withdraw-effect = A retirada entra em vigor imediatamente: a chave deixa de enviar sem supervisão a partir do próximo envio. Não é necessário reiniciar.
admin-integrations-unattended-cancel = Cancelar
admin-integrations-unattended-saving = Salvando…
admin-integrations-unattended-approved-done = Envio sem supervisão aprovado para a chave { $prefix }. Já está em vigor.
admin-integrations-unattended-withdrawn-done = Aprovação retirada para a chave { $prefix }. Esta chave não envia mais sem supervisão.
admin-integrations-unattended-error-gone = A chave { $prefix } não existe mais ou foi revogada. Nada foi alterado.
admin-integrations-unattended-error-conflict = A chave { $prefix } foi revogada ou o proprietário dela não solicitou envio sem supervisão. Nada foi aprovado.
admin-integrations-unattended-error = Não foi possível alterar a aprovação da chave { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Observabilidade
admin-obs-load-error = Não foi possível carregar os dados de observabilidade
admin-obs-export-error = Não foi possível exportar o log de auditoria
admin-obs-ban-add-error = Não foi possível adicionar o banimento
admin-obs-unban-error = Não foi possível remover o banimento
admin-obs-telemetry-note = O registro em log e a telemetria não são definidos aqui. O filtro de log (MW_LOG), o coletor OTLP (MW_OTLP_ENDPOINT) e o endpoint de métricas (MW_METRICS_TOKEN) são lidos do ambiente do servidor quando ele inicia.
admin-obs-audit = Log de auditoria
admin-obs-export = Exportar JSONL
admin-obs-audit-empty = Nenhuma entrada de auditoria.
admin-obs-col-time = Horário
admin-obs-col-actor = Autor
admin-obs-col-action = Ação
admin-obs-col-target = Alvo
admin-obs-bans = Monitor de login / lista de banimentos
admin-obs-bans-note = Esta lista é um registro. O Mailwoman não recusa conexões dos endereços nela, e uma entrada não tem efeito sobre o login. Cada login de administrador com falha grava uma linha no log do servidor sobre a qual um jail do fail2ban pode agir, e cinco falhas de um mesmo endereço adicionam esse endereço aqui. O bloqueio é feito por esse jail ou pelo seu firewall.
admin-obs-ban-add = Adicionar banimento
admin-obs-ban-ip = Endereço IP
admin-obs-ban-reason = Motivo
admin-obs-ban-btn = Banir IP
admin-obs-bans-empty = Nenhum banimento ativo.
admin-obs-unban-for = Desbanir { $ip }
admin-obs-unban-btn = Desbanir

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Plugins
admin-plugins-intro = Os plugins do engine são executados em um sandbox WebAssembly controlado por permissões. Aprove um plugin antes que ele possa ser ativado e conceda apenas as permissões de que ele precisa.
admin-plugins-empty = Nenhum plugin registrado.
admin-plugins-signed = Assinado
admin-plugins-unsigned = Não assinado
admin-plugins-approved = Aprovado
admin-plugins-enabled = Ativado
admin-plugins-approve = Aprovar
admin-plugins-enable = Ativar
admin-plugins-disable = Desativar
admin-plugins-net = rede: { $hosts }
admin-plugins-limits = limites: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = limites: { $memory } MiB · { $deadline } ms · { $fuel } de combustível
admin-plugins-allow-unsigned-for = Permitir o plugin não assinado { $name }
admin-plugins-allow-unsigned = Permitir a execução deste plugin não assinado
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = Lista de plugins de terceiros permitidos
admin-allowlist-intro = Um componente de terceiros (não próprio) só é carregado depois que um administrador aprova o seu digest SHA-256 exato. O digest abaixo é calculado pelo servidor sobre os bytes do componente no disco; aprová-lo fixa exatamente esses bytes. Os componentes próprios são fixados no build e não são gerenciados aqui.
admin-allowlist-load-error = Não foi possível carregar a lista de plugins permitidos.
admin-allowlist-present-heading = Componentes no disco
admin-allowlist-present-empty = Nenhum componente de terceiros presente. Coloque um componente no diretório de plugins de terceiros para que ele apareça aqui.
admin-allowlist-digest-label = SHA-256 calculado
admin-allowlist-status-approved = Aprovado
admin-allowlist-status-pending = Não aprovado
admin-allowlist-status-firstparty = Próprio
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Admitido por fixação de digest. Este componente não tem assinatura; a aprovação confia exatamente nos bytes cujo digest é mostrado, que é a situação esperada para um componente aprovado dessa forma.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = Permissões de alto poder ({ $caps }) não podem ser concedidas a um plugin de terceiros. O servidor as recusa independentemente da aprovação; elas estão disponíveis apenas para componentes próprios.
admin-allowlist-firstparty-note = Este id corresponde a um componente próprio. A fixação do componente próprio sempre tem precedência, então este id não pode ser aprovado como de terceiros.
admin-allowlist-approve = Aprovar digest
admin-allowlist-approve-for = Aprovar digest de { $id }
admin-allowlist-revoke = Revogar
admin-allowlist-revoke-for = Revogar fixação de { $id }
admin-allowlist-uninstall = Desinstalar
admin-allowlist-uninstall-for = Desinstalar { $id }
admin-allowlist-pins-heading = Fixações aprovadas e revogadas
admin-allowlist-pins-empty = Nenhuma fixação registrada.
admin-allowlist-pin-approved-by = Aprovado por { $by } em { $at }
admin-allowlist-pin-revoked = Revogado
admin-allowlist-cancel = Cancelar

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Aprovar o carregamento deste componente?
admin-allowlist-approve-detail = Aprovar fixa exatamente os bytes cujo SHA-256 é mostrado abaixo. Após a aprovação, somente bytes que correspondam a este digest serão carregados para este id; qualquer alteração no componente produz um digest diferente e não será carregada até ser aprovada novamente. A aprovação, por si só, não concede nenhuma permissão.
admin-allowlist-approve-confirm = Aprovar digest

# Revoke confirmation.
admin-allowlist-revoke-title = Revogar esta fixação?
admin-allowlist-revoke-detail = Revogar remove a aprovação deste digest e desativa o plugin. Isso entra em vigor no próximo carregamento; uma instância já em execução não é interrompida até lá.
admin-allowlist-revoke-confirm = Revogar fixação

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Desinstalar este plugin?
admin-allowlist-uninstall-detail = Desinstalar exclui os dados de chave/valor armazenados do plugin para todas as contas, remove as fixações dele na lista de permitidos e o desativa. O arquivo do componente no disco não é excluído; ele pode ser aprovado novamente depois.
admin-allowlist-uninstall-confirm = Desinstalar plugin

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Login único (SSO)
admin-sso-intro = Configure backends de login OIDC e SAML 2.0. Os backends ativados aparecem como botões “Entrar com…” no login da caixa postal, para toda a implantação ou para um domínio.
admin-sso-add = Adicionar um backend de login
admin-sso-edit = Editar
admin-sso-create = Adicionar backend
admin-sso-update = Salvar alterações
admin-sso-cancel = Cancelar
admin-sso-empty = Nenhum backend de SSO configurado.
admin-sso-load-error = Não foi possível carregar os backends de SSO.
admin-sso-save-error = Não foi possível salvar o backend.
admin-sso-delete-error = Não foi possível excluir o backend.

# Common fields
admin-sso-id = ID do backend
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Nome de exibição
admin-sso-display-name-placeholder = Entrar com Acme SSO
admin-sso-kind = Protocolo
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Escopo
admin-sso-scope-deployment = Toda a implantação
admin-sso-scope-domain = Um domínio
admin-sso-domain = Domínio
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Ativado
admin-sso-first-login = Política de primeiro login
admin-sso-policy-allowlist = Lista de permitidos (negar usuários desconhecidos)
admin-sso-policy-autocreate = Criar contas automaticamente no primeiro login

# OIDC fields
admin-sso-issuer = URL do emissor
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = ID do cliente
admin-sso-client-secret = Segredo do cliente
admin-sso-secret-unchanged = Deixe em branco para manter o segredo armazenado
admin-sso-redirect = URL de redirecionamento
admin-sso-scopes = Escopos
admin-sso-metadata = Metadados do SP

# SAML fields
admin-sso-sp-entity-id = ID da entidade SP
admin-sso-acs-url = URL do ACS
admin-sso-idp-metadata-url = URL dos metadados do IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML dos metadados do IdP
admin-sso-idp-metadata-xml-placeholder = Cole o XML dos metadados do IdP ou use a URL acima
admin-sso-idp-sso-url = URL de SSO do IdP
admin-sso-idp-slo-url = URL de logout (SLO) do IdP
admin-sso-idp-certs = Certificados de assinatura do IdP (PEM)
admin-sso-idp-certs-placeholder = Um certificado PEM por bloco, separados por uma linha em branco
admin-sso-nameid-format = Formato do NameID
admin-sso-want-signed = Exigir asserções assinadas
admin-sso-want-encrypted = Exigir asserções criptografadas

# Claim map
admin-sso-claims = Mapeamento de claims
admin-sso-claim-email = Claim de e-mail
admin-sso-claim-username = Claim de nome de usuário
admin-sso-claim-display = Claim de nome de exibição
admin-sso-claim-groups = Claim de grupos

# List row
admin-sso-badge-enabled = Ativado
admin-sso-badge-disabled = Desativado
admin-sso-enable = Ativar
admin-sso-disable = Desativar
admin-sso-delete = Excluir
# `name` is the backend's admin-set display name.
admin-sso-enable-for = Ativar { $name }
admin-sso-disable-for = Desativar { $name }
admin-sso-delete-for = Excluir { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = Metadados do servidor
admin-servermeta-intro = Veja e edite as anotações de servidor RFC 5464 de uma conta provisionada. As alterações são gravadas diretamente no servidor de e-mail, que decide se a conta pode defini-las.
admin-servermeta-account = Conta
admin-servermeta-select-option = Selecione uma conta…
admin-servermeta-select-prompt = Selecione uma conta para ver e editar as anotações de servidor dela.
admin-servermeta-load-error = Não foi possível carregar a lista de contas.
admin-servermeta-no-accounts = Nenhuma conta provisionada.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = Reagrupar conversas da caixa postal
admin-rethread-intro = Executa novamente o agrupamento de conversas (JWZ) sobre os e-mails armazenados de uma conta provisionada e recalcula as chaves do agrupamento. Esta é uma ação de manutenção pontual, não algo que é executado automaticamente.
admin-rethread-account = Conta
admin-rethread-select-option = Selecione uma conta…
admin-rethread-no-accounts = Nenhuma conta provisionada.
admin-rethread-load-error = Não foi possível carregar a lista de contas.
admin-rethread-run = Reagrupar conversas da caixa postal
admin-rethread-confirm-title = Reagrupar as conversas desta caixa postal?
admin-rethread-confirm-warning = O reagrupamento recalcula as chaves de agrupamento das conversas desta conta. Conversas existentes podem ser mescladas ou divididas, e os usuários podem ver as conversas reorganizadas.
admin-rethread-confirm-detail = Esta é uma ação de manutenção pontual. Ela é executada uma vez agora; pode ser executada novamente com segurança e não exclui nenhum e-mail.
admin-rethread-confirm = Reagrupar conversas da caixa postal
admin-rethread-running = Reagrupando…
admin-rethread-cancel = Cancelar
admin-rethread-summary = { $messages } mensagem(ns) reagrupada(s) em { $threads } conversa(s) em { $accounts } conta(s); { $reassigned } mensagem(ns) movida(s) para outra conversa.
admin-rethread-error = A ação de reagrupamento falhou. Se o servidor rejeitou a solicitação, nenhum agrupamento de conversas foi alterado; verifique os logs do servidor e tente novamente.

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = Índice de pesquisa
admin-searchindex-intro = A pesquisa lê um índice criado a partir dos e-mails armazenados. O servidor o reconstrói ao iniciar quando ele não corresponde aos e-mails armazenados. Os e-mails de contas com acesso zero não são indexados, então a pesquisa não encontra nada para essas contas.
admin-searchindex-count = { $documents } de { $messages } mensagem(ns) armazenada(s) estão indexadas.
admin-searchindex-on-disk = O índice é mantido em disco.
admin-searchindex-in-memory = O índice é mantido na memória e é reconstruído sempre que o servidor inicia.
admin-searchindex-progress = Uma reconstrução está em andamento: { $done } de { $total } mensagem(ns) concluída(s).
admin-searchindex-run = Reconstruir índice de pesquisa
admin-searchindex-running = Reconstruindo…
admin-searchindex-summary = { $messages } mensagem(ns) armazenada(s) lida(s) em { $accounts } conta(s): { $indexed } indexada(s), { $removed } removida(s) do índice, { $failed } não puderam ser lidas. { $zeroAccess } conta(s) com acesso zero deixada(s) sem indexação.
admin-searchindex-busy = Uma reconstrução já está em andamento. Aguarde a conclusão.
admin-searchindex-error = A reconstrução falhou. O índice mantém o que continha; verifique os logs do servidor e tente novamente.
admin-searchindex-status-error = Não foi possível ler o status do índice de pesquisa.
admin-searchindex-unavailable = Este servidor funciona no modo proxy e não tem índice de pesquisa.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
admin-2fa-title = Exigir dois fatores
admin-2fa-intro = Exija um segundo fator (chave de acesso ou app autenticador) para o login. Qualquer usuário pode cadastrar um fator por conta própria; exigi-lo aqui obriga as contas abrangidas a cadastrar um no próximo login.
admin-2fa-load-error = Não foi possível carregar a política de dois fatores.
admin-2fa-save-error = Não foi possível salvar a política de dois fatores.
admin-2fa-global = Exigir dois fatores para toda a implantação
admin-2fa-global-label = Exigir dois fatores para toda a implantação
admin-2fa-global-note = Quando ativado, todas as contas precisam ter um segundo fator. Uma regra por domínio abaixo também pode exigi-lo para um domínio sem exigi-lo em todos.
admin-2fa-domains-heading = Exigências por domínio
admin-2fa-domains-empty = Nenhuma exigência por domínio definida.
admin-2fa-col-domain = Domínio
admin-2fa-col-require = Exigido
admin-2fa-require-for = Exigir dois fatores para { $domain }
admin-2fa-add-domain = Domínio
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Exigir dois fatores para este domínio
admin-2fa-add-require-label = Exigir dois fatores para este domínio
admin-2fa-add-save = Adicionar regra de domínio

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Saída
admin-egress-heading = Rotas de proxy de saída
admin-egress-intro = Encaminhe as requisições de saída do servidor por um proxy HTTP CONNECT ou SOCKS5. O Mailwoman continua aplicando sua própria política de endereços — o proxy nunca é solicitado a resolver um nome.
admin-egress-empty = Nenhuma rota configurada. As requisições de saída são feitas diretamente.
admin-egress-none-active = Nenhuma rota em uso. As requisições de saída são feitas diretamente. Uma rota salva só transporta tráfego depois que você escolhe “Usar esta rota”.
admin-egress-in-use = As requisições de saída usam a rota “{ $id }”.
admin-egress-deactivate = Parar de usar um proxy
admin-egress-activate = Usar esta rota
admin-egress-activate-for = Usar a rota { $id }
admin-egress-activate-error = Não foi possível trocar para a rota
admin-egress-deactivate-error = Não foi possível parar de usar a rota
admin-egress-col-state = Estado
admin-egress-state-active = Em uso
admin-egress-state-inactive = Fora de uso
admin-egress-test-scope = Somente a rota em uso pode ser testada. Um teste obtém uma URL fixa pela rota e informa até onde chegou; ele não altera nada.
admin-egress-probe-url = Obtida: { $url }
admin-egress-load-error = Não foi possível carregar as rotas de saída
admin-egress-save-error = Não foi possível salvar a rota
admin-egress-delete-error = Não foi possível excluir a rota
admin-egress-invalid = São necessários um id, um host e uma porta maior que zero.
admin-egress-username-required = Uma senha exige um nome de usuário; sem ele, a rota se autenticaria como ninguém.
admin-egress-col-id = Id
admin-egress-col-route = Rota
admin-egress-col-username = Nome de usuário
admin-egress-col-credentials = Senha
admin-egress-col-plaintext = Texto simples
admin-egress-col-actions = Ações
admin-egress-cred-set = Definida
admin-egress-cred-none = Nenhuma
admin-egress-plaintext-on = Permitido
admin-egress-plaintext-off = Recusado
admin-egress-add-heading = Adicionar uma rota
admin-egress-edit-heading = Editar rota
admin-egress-id = Id
admin-egress-scheme = Esquema
admin-egress-host = Host
admin-egress-port = Porta
admin-egress-username = Nome de usuário
admin-egress-password = Senha
admin-egress-password-unchanged = Deixe em branco para manter a senha armazenada
admin-egress-allow-plaintext = Permitir origens http em texto simples por esta rota
admin-egress-allow-plaintext-note = Desativado por padrão. Uma origem https é o que impede um proxy de ler o tráfego que transporta; permitir http significa que o proxy o vê.
admin-egress-test = Testar
admin-egress-testing = Testando…
admin-egress-test-failed = Não foi possível executar o teste, então nada se sabe sobre esta rota.
admin-egress-outcome-connected = Conectado
admin-egress-outcome-auth-rejected = Autenticação rejeitada
admin-egress-outcome-refused-by-policy = Recusado pela política
admin-egress-outcome-dns-failed = O nome não foi resolvido
admin-egress-outcome-unreachable = Inacessível
admin-egress-outcome-origin-tls-failed = Falha de TLS na origem
admin-egress-outcome-route-invalid = A rota não é válida
admin-egress-outcome-unknown = Resultado não reconhecido
admin-egress-stage-dns = parou na resolução de nome
admin-egress-stage-connect = parou ao conectar ao proxy
admin-egress-stage-tunnel = parou ao abrir o túnel
admin-egress-stage-origin = parou na origem
admin-egress-stage-unknown = parou em uma etapa não reconhecida
admin-egress-proxied-yes = Passou pelo proxy
admin-egress-proxied-no = Não passou pelo proxy
