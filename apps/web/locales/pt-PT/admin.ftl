# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Administração do Mailwoman
admin-nav = Secções de administração
admin-sign-out = Terminar sessão
admin-nav-domains = Domínios
admin-nav-users = Utilizadores
admin-nav-integrations = Integrações
admin-nav-observability = Observabilidade
admin-nav-plugins = Plugins
admin-nav-assist = Assistente de IA
admin-nav-sso = Início de sessão único
admin-nav-servermeta = Metadados do servidor
admin-nav-rethread = Manutenção
admin-nav-2fa = Exigir dois fatores
admin-nav-security-policy = Política de segurança
admin-nav-oauth-clients = Clientes OAuth

# Shared admin actions / states
admin-delete = Eliminar
admin-revoke = Revogar
admin-remove = Remover
admin-saved = Guardado.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Início de sessão de administrador
admin-login-note = Este painel funciona numa sessão de administrador separada. A sessão termina após 30 minutos sem atividade e, no máximo, 12 horas após o início de sessão.
admin-login-session-ended = A sua sessão de administrador terminou. Inicie sessão novamente.
admin-login-username = Nome de utilizador do administrador
admin-login-password = Palavra-passe
admin-login-sign-in = Iniciar sessão
admin-login-signing-in = A iniciar sessão…
admin-login-invalid = Credenciais de administrador inválidas
admin-login-unreachable = Não foi possível contactar o servidor

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domínios
admin-domains-load-error = Não foi possível carregar os domínios
admin-domains-save-error = Não foi possível guardar o domínio
admin-domains-delete-error = Não foi possível eliminar o domínio
admin-domains-add = Adicionar domínio
admin-domains-name = Nome do domínio
admin-domains-name-placeholder = example.com
admin-domains-note = Um domínio é registado pelo nome. Os nomes registados são propostos no ecrã «Exigir dois fatores» quando uma regra se aplica a um domínio. Registar um domínio não encaminha nem filtra correio.
admin-domains-save = Guardar domínio
admin-domains-empty = Ainda não há domínios.
admin-domains-delete-for = Eliminar { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Utilizadores
admin-users-load-error = Não foi possível carregar os utilizadores
admin-users-provision-error = Não foi possível aprovisionar o utilizador
admin-users-flag-error = Não foi possível atualizar o indicador
admin-users-revoke-error = Não foi possível revogar as sessões
admin-users-provision = Aprovisionar utilizador
admin-users-username = Nome de utilizador
admin-users-domain = Domínio
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Quota em bytes (0 = ilimitada)
admin-users-quota-msgs = Quota de mensagens (0 = ilimitada)
admin-users-empty = Ainda não há utilizadores.
admin-users-col-account = Conta
admin-users-col-quota = Quota (bytes/mensagens)
admin-users-col-zeroaccess = Acesso zero
admin-users-col-flags = Indicadores
admin-users-col-sessions = Sessões
admin-users-zeroaccess-for = Acesso zero para { $account }
admin-users-zeroaccess-help = Apenas um registo. O armazenamento de acesso zero é ativado e desativado pelo utilizador, nas suas próprias definições, com a sua própria chave. Marcar esta caixa não encripta o correio guardado da conta, e desmarcá-la não o desencripta.
admin-users-disable-for = Desativar { $account }
admin-users-disabled = desativado
admin-users-disabled-help = Bloqueia o início de sessão e termina todas as sessões abertas desta conta. As chaves de API e os tokens da conta são recusados. Isto não desativa a caixa de correio no servidor de correio: outros clientes de correio continuam a aceder-lhe até que também a desative lá.
admin-users-force-change-for = Forçar alteração da palavra-passe de { $account }
admin-users-force-change = forçar alteração
admin-users-force-change-help = O utilizador fica retido num ecrã de alteração da palavra-passe, no próximo início de sessão e nas sessões já abertas, e não consegue aceder ao correio até a alteração ser concluída. Isto requer um back-end de alteração de palavra-passe funcional (MW_PASSWD_BACKEND). Sem ele, a alteração não pode ser concluída e o utilizador permanece nesse ecrã até desmarcar esta caixa.
admin-users-revoke-for = Revogar sessões de { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integrações
admin-integrations-load-error = Não foi possível carregar as integrações
admin-integrations-revoke-error = Não foi possível revogar a chave
admin-integrations-ldap = Diretório LDAP / GAL
admin-integrations-nextcloud = Ponte Nextcloud
admin-integrations-active = Ativo
admin-integrations-configured = Configurado
admin-integrations-not-configured = Não configurado
admin-integrations-unknown = Estado desconhecido
admin-integrations-config-note = «Configurado» significa que esta implementação tem definições para a integração: uma entrada de diretório LDAP ativada na base de dados, ou as três variáveis de ambiente MW_NEXTCLOUD_*. Ambas são lidas quando o servidor arranca. Não significa que o serviço remoto tenha sido contactado nem que esteja acessível.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = Nenhum webhook registado.
admin-integrations-keys = Chaves de API e MCP
admin-integrations-keys-empty = Nenhuma chave emitida.
admin-integrations-col-prefix = Prefixo
admin-integrations-col-account = Conta
admin-integrations-col-scopes = Âmbitos
admin-integrations-col-status = Estado
admin-integrations-status-revoked = revogada
admin-integrations-status-active = ativa
admin-integrations-revoke-key = Revogar a chave { $prefix }
admin-integrations-col-unattended = Envio automático
admin-integrations-unattended-not-requested = Não pedido
admin-integrations-unattended-requested = Pedido, não aprovado
admin-integrations-unattended-approved = Aprovado
admin-integrations-unattended-approve = Aprovar
admin-integrations-unattended-withdraw = Retirar
admin-integrations-unattended-approve-key = Aprovar o envio automático para a chave { $prefix }
admin-integrations-unattended-withdraw-key = Retirar a aprovação do envio automático para a chave { $prefix }
admin-integrations-unattended-approve-title = Aprovar o envio automático para esta chave?
admin-integrations-unattended-withdraw-title = Retirar a aprovação desta chave?
admin-integrations-unattended-owner = Proprietário: { $account }
admin-integrations-unattended-key = Chave: { $prefix }
admin-integrations-unattended-scope = Permissões: { $permissions }. Ferramentas MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = Não foi possível ler o âmbito desta chave.
admin-integrations-scope-read = ler
admin-integrations-scope-send = enviar
admin-integrations-scope-delete = eliminar
admin-integrations-scope-mail = correio
admin-integrations-scope-pim = calendário, tarefas, notas e contactos
admin-integrations-scope-none = nenhum
admin-integrations-unattended-approve-meaning = As mensagens enviadas com esta chave através de MCP são transmitidas sem que uma pessoa as liberte.
admin-integrations-unattended-withdraw-meaning = Um envio feito com esta chave através de MCP é recusado até a chave ser novamente aprovada. A mensagem não é transmitida nem é colocada na Caixa de saída do proprietário.
admin-integrations-unattended-approve-effect = A aprovação produz efeito imediatamente, a partir do próximo envio da chave. Não é necessário reiniciar.
admin-integrations-unattended-withdraw-effect = A retirada produz efeito imediatamente: a chave deixa de enviar automaticamente a partir do próximo envio. Não é necessário reiniciar.
admin-integrations-unattended-cancel = Cancelar
admin-integrations-unattended-saving = A guardar…
admin-integrations-unattended-approved-done = Envio automático aprovado para a chave { $prefix }. Já está em vigor.
admin-integrations-unattended-withdrawn-done = Aprovação retirada para a chave { $prefix }. Esta chave deixou de enviar automaticamente.
admin-integrations-unattended-error-gone = A chave { $prefix } já não existe ou foi revogada. Nada foi alterado.
admin-integrations-unattended-error-conflict = A chave { $prefix } está revogada ou o seu proprietário não pediu o envio automático. Nada foi aprovado.
admin-integrations-unattended-error = Não foi possível alterar a aprovação da chave { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Observabilidade
admin-obs-load-error = Não foi possível carregar os dados de observabilidade
admin-obs-export-error = Não foi possível exportar o registo de auditoria
admin-obs-ban-add-error = Não foi possível adicionar o banimento
admin-obs-unban-error = Não foi possível remover o banimento
admin-obs-telemetry-note = A telemetria operacional está desativada por predefinição. Configure o coletor, os sinais e os limites na página dedicada à telemetria. MW_OTLP_ENDPOINT e MW_METRICS_TOKEN continuam a ser substituições da implementação; as alterações exigem o reinício do servidor. MW_LOG controla o filtro do registo de arranque.
admin-obs-telemetry-open = Configurar telemetria operacional
admin-obs-audit = Registo de auditoria
admin-obs-export = Exportar JSONL
admin-obs-audit-empty = Sem entradas de auditoria.
admin-obs-col-time = Hora
admin-obs-col-actor = Autor
admin-obs-col-action = Ação
admin-obs-col-target = Alvo
admin-obs-bans = Lista de IPs banidos
admin-obs-bans-note = As entradas ativas bloqueiam os pedidos à aplicação a partir do IP de cliente fidedigno, também nas sessões existentes. As entradas expiradas deixam de bloquear. As verificações de estado continuam disponíveis. Os endereços reencaminhados só são considerados fidedignos a partir de proxies configurados; banir um proxy ou um endereço de loopback pode bloquear todas as pessoas que usam esse endereço. O Fail2ban e as regras de firewall funcionam de forma independente.
admin-obs-ban-add = Adicionar banimento
admin-obs-ban-ip = Endereço IP
admin-obs-ban-reason = Motivo
admin-obs-ban-btn = Banir IP
admin-obs-bans-empty = Sem banimentos ativos.
admin-obs-unban-for = Remover banimento de { $ip }
admin-obs-unban-btn = Desbanir

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Plugins
admin-plugins-intro = Os plugins do motor são executados numa sandbox WebAssembly controlada por capacidades. Aprove um plugin antes de o poder ativar e conceda apenas as capacidades de que necessita.
admin-plugins-empty = Não há plugins registados.
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
admin-allowlist-title = Lista de permissões de plugins de terceiros
admin-allowlist-intro = Um componente de terceiros (não próprio) só é carregado depois de um administrador aprovar o seu digest SHA-256 exato. O digest abaixo é calculado pelo servidor sobre os bytes do componente no disco; aprová-lo fixa esses bytes exatos. Os componentes próprios são fixados na compilação e não são geridos aqui.
admin-allowlist-load-error = Não foi possível carregar a lista de permissões de plugins.
admin-allowlist-present-heading = Componentes no disco
admin-allowlist-present-empty = Não há componentes de terceiros. Coloque um componente no diretório de plugins de terceiros para que apareça aqui.
admin-allowlist-digest-label = SHA-256 calculado
admin-allowlist-status-approved = Aprovado
admin-allowlist-status-pending = Não aprovado
admin-allowlist-status-firstparty = Próprio
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Admitido por fixação do digest. Este componente não tem assinatura; a aprovação confia nos bytes exatos cujo digest é apresentado, o que é a situação esperada para um componente aprovado desta forma.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = As capacidades de alto poder ({ $caps }) não podem ser concedidas a um plugin de terceiros. O servidor recusa-as independentemente da aprovação; estão disponíveis apenas para componentes próprios.
admin-allowlist-firstparty-note = Este id corresponde a um componente próprio. A fixação do componente próprio tem sempre precedência, pelo que este id não pode ser aprovado como de terceiros.
admin-allowlist-approve = Aprovar digest
admin-allowlist-approve-for = Aprovar o digest de { $id }
admin-allowlist-revoke = Revogar
admin-allowlist-revoke-for = Revogar a fixação de { $id }
admin-allowlist-uninstall = Desinstalar
admin-allowlist-uninstall-for = Desinstalar { $id }
admin-allowlist-pins-heading = Fixações aprovadas e revogadas
admin-allowlist-pins-empty = Nenhuma fixação registada.
admin-allowlist-pin-approved-by = Aprovado por { $by } em { $at }
admin-allowlist-pin-revoked = Revogado
admin-allowlist-cancel = Cancelar

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Aprovar o carregamento deste componente?
admin-allowlist-approve-detail = A aprovação fixa os bytes exatos cujo SHA-256 é apresentado abaixo. Após a aprovação, apenas os bytes que correspondam a este digest serão carregados para este id; qualquer alteração ao componente produz um digest diferente e não será carregada até ser novamente aprovada. A aprovação, por si só, não concede capacidades.
admin-allowlist-approve-confirm = Aprovar digest

# Revoke confirmation.
admin-allowlist-revoke-title = Revogar esta fixação?
admin-allowlist-revoke-detail = A revogação remove a aprovação deste digest e desativa o plugin. Produz efeito no próximo carregamento; uma instância já em execução só é parada nessa altura.
admin-allowlist-revoke-confirm = Revogar fixação

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Desinstalar este plugin?
admin-allowlist-uninstall-detail = A desinstalação elimina os dados chave/valor guardados do plugin para todas as contas, remove as suas fixações na lista de permissões e desativa-o. O ficheiro do componente no disco não é eliminado; pode voltar a ser aprovado mais tarde.
admin-allowlist-uninstall-confirm = Desinstalar plugin
admin-allowlist-col-component = Componente
admin-allowlist-col-state = Estado
admin-allowlist-col-approved = Aprovado
admin-allowlist-approve-done = O digest de { $id } foi aprovado.
admin-allowlist-revoke-done = A fixação de { $id } foi revogada e o plugin foi desativado.
admin-allowlist-uninstall-done = { $id } foi desinstalado.

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Início de sessão único
admin-sso-intro = Configure back-ends de início de sessão OIDC e SAML 2.0. Os back-ends ativados aparecem como botões «Iniciar sessão com…» no início de sessão da caixa de correio, com âmbito em toda a implementação ou num domínio.
admin-sso-add = Adicionar um back-end de início de sessão
admin-sso-edit = Editar
admin-sso-create = Adicionar back-end
admin-sso-update = Guardar alterações
admin-sso-cancel = Cancelar
admin-sso-empty = Nenhum back-end SSO configurado.
admin-sso-load-error = Não foi possível carregar os back-ends SSO.
admin-sso-save-error = Não foi possível guardar o back-end.
admin-sso-delete-error = Não foi possível eliminar o back-end.

# Common fields
admin-sso-id = ID do back-end
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Nome apresentado
admin-sso-display-name-placeholder = Iniciar sessão com Acme SSO
admin-sso-kind = Protocolo
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Âmbito
admin-sso-scope-deployment = Toda a implementação
admin-sso-scope-domain = Um domínio
admin-sso-domain = Domínio
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Ativado
admin-sso-first-login = Política do primeiro início de sessão
admin-sso-policy-allowlist = Lista de permissões (recusar utilizadores desconhecidos)
admin-sso-policy-autocreate = Criar contas automaticamente no primeiro início de sessão

# OIDC fields
admin-sso-issuer = URL do emissor
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = ID de cliente
admin-sso-client-secret = Segredo do cliente
admin-sso-secret-unchanged = Deixe em branco para manter o segredo guardado
admin-sso-redirect = URL de redirecionamento
admin-sso-scopes = Âmbitos
admin-sso-metadata = Metadados do SP

# SAML fields
admin-sso-sp-entity-id = ID de entidade do SP
admin-sso-acs-url = URL do ACS
admin-sso-idp-metadata-url = URL dos metadados do IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML dos metadados do IdP
admin-sso-idp-metadata-xml-placeholder = Cole o XML dos metadados do IdP ou utilize o URL acima
admin-sso-idp-sso-url = URL de SSO do IdP
admin-sso-idp-slo-url = URL de fim de sessão (SLO) do IdP
admin-sso-idp-certs = Certificados de assinatura do IdP (PEM)
admin-sso-idp-certs-placeholder = Um certificado PEM por bloco, separados por uma linha em branco
admin-sso-nameid-format = Formato do NameID
admin-sso-want-signed = Exigir asserções assinadas
admin-sso-want-encrypted = Exigir asserções encriptadas

# Claim map
admin-sso-claims = Mapeamento de claims
admin-sso-claim-email = Claim de e-mail
admin-sso-claim-username = Claim de nome de utilizador
admin-sso-claim-display = Claim de nome apresentado
admin-sso-claim-groups = Claim de grupos

# List row
admin-sso-badge-enabled = Ativado
admin-sso-badge-disabled = Desativado
admin-sso-enable = Ativar
admin-sso-disable = Desativar
admin-sso-delete = Eliminar
# `name` is the backend's admin-set display name.
admin-sso-enable-for = Ativar { $name }
admin-sso-disable-for = Desativar { $name }
admin-sso-delete-for = Eliminar { $name }

# Rebuilt list + detail (f33, t32-cfg §4.12). `name` and `id` are isolated values.
admin-sso-col-name = Nome
admin-sso-col-state = Estado
admin-sso-search = Pesquisar fornecedores
admin-sso-no-match = Nenhum fornecedor corresponde à pesquisa.
admin-sso-scope-for-domain = Domínio { $domain }
admin-sso-config-unreadable = Configuração ilegível
admin-sso-config-unreadable-note = Não foi possível ler a configuração guardada deste fornecedor. Preencha o formulário e guarde para a substituir.
admin-sso-not-found = Não existe nenhum fornecedor de início de sessão com o id { $id }.
admin-sso-edit-title = Editar fornecedor de início de sessão
admin-sso-enabled-done = { $name } está ativado.
admin-sso-disabled-done = { $name } está desativado.
admin-sso-deleted-done = { $name } foi eliminado.
admin-sso-disable-title = Desativar { $name }?
admin-sso-disable-body = O seu botão sai da página de início de sessão e ninguém pode iniciar sessão com ele até ser novamente ativado. As contas são mantidas.
admin-sso-delete-title = Eliminar { $name }?
admin-sso-delete-body = O fornecedor e o seu segredo guardado são removidos e ninguém pode iniciar sessão com ele. As contas criadas através dele são mantidas.
admin-sso-err-required = { $field } é obrigatório.
admin-sso-on = Ativado

admin-sso-group-basics = Fornecedor
admin-sso-id-help = Um id curto com letras minúsculas, algarismos e hífenes; passa a fazer parte dos endereços deste servidor para o fornecedor.
admin-sso-id-fixed = O id não pode ser alterado porque o fornecedor conhece os endereços deste servidor através dele; elimine o fornecedor e adicione-o novamente para usar outro id.
admin-sso-id-pattern = Utilize letras minúsculas, algarismos e hífenes, começando por uma letra ou um algarismo.
admin-sso-id-taken = Outro fornecedor já utiliza o id { $id }.
admin-sso-display-name-help = O texto do botão na página de início de sessão.
admin-sso-kind-help = OpenID Connect para fornecedores como Keycloak, Entra ID ou Google; SAML 2.0 para fornecedores que publicam metadados SAML.
admin-sso-scope-help = «Toda a implementação» mostra o botão a todos; «Um domínio» mostra-o apenas para endereços desse domínio.
admin-sso-domain-help = O domínio de correio cuja página de início de sessão mostra este fornecedor.
admin-sso-domain-choose = Escolha um domínio
admin-sso-domain-invalid = Introduza um nome de domínio como example.org.
admin-sso-first-login-help = O que acontece quando alguém inicia sessão pela primeira vez: é recusado, a menos que a sua conta exista, ou é criada uma conta para essa pessoa.
admin-sso-enabled-help = Desativado mantém as definições, mas retira o botão da página de início de sessão.

admin-sso-group-oidc = OpenID Connect
admin-sso-issuer-help = O endereço do emissor do fornecedor; este servidor lê daí o documento de descoberta.
admin-sso-client-id-help = O id de cliente com que este servidor foi registado no fornecedor.
admin-sso-client-secret-help = O segredo do cliente fornecido pelo fornecedor; é guardado selado e nunca volta a ser apresentado.
admin-sso-redirect-help = Deixe vazio para usar o endereço deste servidor /api/sso/<id>/callback; registe o mesmo endereço no fornecedor.
admin-sso-scopes-help = Os âmbitos pedidos no início de sessão; openid é obrigatório.
admin-sso-scopes-openid = Os âmbitos têm de incluir openid.
admin-sso-scopes-invalid = Um âmbito é uma palavra sem espaços nem aspas.

admin-sso-group-saml-idp = Fornecedor de identidade (SAML)
admin-sso-group-saml-idp-help = Leia o ficheiro de metadados do fornecedor para preencher estes campos, ou introduza-os manualmente.
admin-sso-meta = Metadados do fornecedor
admin-sso-meta-help = Carregue ou cole os metadados do fornecedor para ver o que contêm e preencher a partir deles os campos abaixo.
admin-sso-meta-none-stored = Não há metadados guardados com este fornecedor.
admin-sso-meta-stored = Estão guardados com este fornecedor metadados de { $entity }.
admin-sso-meta-stored-unnamed = Estão guardados metadados com este fornecedor.
admin-sso-meta-file = Ficheiro de metadados
admin-sso-meta-paste = Ou cole os metadados
admin-sso-meta-not-xml = Isto não é um documento XML.
admin-sso-meta-no-idp = Estes metadados não descrevem um fornecedor de identidade (sem IDPSSODescriptor).
admin-sso-meta-preview = O que dizem os metadados
admin-sso-meta-entity = ID de entidade do fornecedor
admin-sso-meta-missing = Não consta dos metadados
admin-sso-meta-sso-binding = Indicado apenas para um binding diferente de HTTP-Redirect, que é o que este servidor utiliza
admin-sso-meta-no-certs = Nenhum certificado de assinatura nos metadados
admin-sso-meta-use = Usar estes valores
admin-sso-meta-used = O endereço de início de sessão, o endereço de fim de sessão e os certificados foram preenchidos a partir dos metadados. Guarde para os manter.
admin-sso-idp-sso-url-help = Para onde as pessoas são enviadas para iniciar sessão (binding HTTP-Redirect).
admin-sso-idp-slo-url-help = Para onde as pessoas são enviadas para terminar sessão; deixe vazio se o fornecedor não tiver nenhum.
admin-sso-idp-certs-help = Os certificados com que o fornecedor assina; uma resposta assinada com qualquer outro certificado é recusada.
admin-sso-cert-required = Adicione pelo menos um certificado de assinatura.
admin-sso-idp-metadata-url-help = Mantido apenas para referência: este servidor não obtém metadados a partir dele.

admin-sso-cert-valid = Válido até { $date }
admin-sso-cert-expiring =
    { $days ->
        [0] Expira hoje
        [one] Expira dentro de 1 dia
       *[other] Expira dentro de { $days } dias
    }
admin-sso-cert-expired = Expirou em { $date }
admin-sso-cert-not-yet = Válido a partir de { $date }
admin-sso-cert-unreadable = Certificado ilegível
admin-sso-cert-none-found = Nenhum certificado PEM encontrado. Um certificado começa por -----BEGIN CERTIFICATE-----.
admin-sso-cert-empty = Ainda não há certificados.
admin-sso-cert-no-subject = Certificado sem nome
admin-sso-cert-remove = Remover o certificado { $subject }
admin-sso-cert-file = Adicionar a partir de um ficheiro
admin-sso-cert-paste = Ou cole um certificado
admin-sso-cert-add = Adicionar certificado

admin-sso-group-saml-sp = Este servidor (SAML)
admin-sso-sp-entity-id-help = O nome que este servidor usa perante o fornecedor; deixe vazio para usar o seu endereço de metadados /api/sso/<id>/metadata.
admin-sso-acs-url-help = Para onde o fornecedor envia a sua resposta; deixe vazio para usar /api/sso/<id>/acs neste servidor.
admin-sso-nameid-format-help = O tipo de identificador de utilizador pedido ao fornecedor.
admin-sso-want-signed-help = Recusar asserções que não estejam assinadas por um dos certificados acima.
admin-sso-want-encrypted-help = Recusar asserções que não estejam encriptadas para este servidor; requer a chave do fornecedor de serviços abaixo.
admin-sso-sp-key = Chave privada do fornecedor de serviços
admin-sso-sp-key-help = A chave privada PEM com que este servidor desencripta as asserções; é guardada selada e nunca volta a ser apresentada.

admin-sso-claims-help = Que claim ou atributo da resposta do fornecedor preenche cada parte da conta.
admin-sso-claim-email-help = O endereço de e-mail da pessoa.
admin-sso-claim-username-help = O nome com que a conta é associada ou criada.
admin-sso-claim-display-help = O nome apresentado aos outros.
admin-sso-claim-groups-help = Pertença a grupos; deixe vazio se não for utilizado.

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = Metadados do servidor
admin-servermeta-intro = Veja e edite as anotações de servidor RFC 5464 de uma conta aprovisionada. As alterações são escritas diretamente no servidor de correio, que decide se a conta as pode definir.
admin-servermeta-account = Conta
admin-servermeta-select-option = Selecione uma conta…
admin-servermeta-select-prompt = Selecione uma conta para ver e editar as suas anotações de servidor.
admin-servermeta-load-error = Não foi possível carregar a lista de contas.
admin-servermeta-no-accounts = Não há contas aprovisionadas.
admin-servermeta-add-user = Adicionar um utilizador
admin-servermeta-account-help = As anotações são lidas e escritas no servidor de correio desta conta, que decide o que a conta pode definir.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = Reagrupar conversas da caixa de correio
admin-rethread-intro = Volta a executar o agrupamento em conversas (JWZ) sobre o correio guardado de uma conta aprovisionada e recalcula o agrupamento das conversas. É uma ação de manutenção pontual, não algo que seja executado automaticamente.
admin-rethread-account = Conta
admin-rethread-select-option = Selecione uma conta…
admin-rethread-no-accounts = Não há contas aprovisionadas.
admin-rethread-load-error = Não foi possível carregar a lista de contas.
admin-rethread-run = Reagrupar conversas
admin-rethread-confirm-title = Reagrupar as conversas desta caixa de correio?
admin-rethread-confirm-warning = O reagrupamento recalcula o agrupamento das conversas desta conta. As conversas existentes podem ser fundidas ou divididas, e os utilizadores podem ver as conversas reagrupadas.
admin-rethread-confirm-detail = É uma ação de manutenção pontual. É executada uma vez, agora; pode ser repetida com segurança e não elimina correio.
admin-rethread-confirm = Reagrupar conversas
admin-rethread-running = A reagrupar…
admin-rethread-cancel = Cancelar
admin-rethread-summary = { $messages } mensagem(ns) reagrupada(s) em { $threads } conversa(s) em { $accounts } conta(s); { $reassigned } mensagem(ns) movida(s) para outra conversa.
admin-rethread-error = A ação de reagrupamento falhou. Se o servidor rejeitou o pedido, nenhum agrupamento foi alterado; verifique os registos do servidor e tente novamente.

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = Índice de pesquisa
admin-searchindex-intro = A pesquisa lê um índice construído a partir do correio guardado. O servidor reconstrói-o no arranque quando não corresponde ao correio guardado. O correio das contas de acesso zero não é indexado, pelo que a pesquisa não encontra nada nessas contas.
admin-searchindex-count = { $documents } de { $messages } mensagem(ns) guardada(s) estão indexadas.
admin-searchindex-on-disk = O índice é mantido no disco.
admin-searchindex-in-memory = O índice é mantido em memória e é reconstruído sempre que o servidor arranca.
admin-searchindex-progress = Reconstrução em curso: { $done } de { $total } mensagem(ns) concluída(s).
admin-searchindex-run = Reconstruir índice de pesquisa
admin-searchindex-running = A reconstruir…
admin-searchindex-summary = Lidas { $messages } mensagem(ns) guardada(s) em { $accounts } conta(s): { $indexed } indexada(s), { $removed } removida(s) do índice, { $failed } ilegível(eis). { $zeroAccess } conta(s) de acesso zero não indexada(s).
admin-searchindex-busy = Já está em curso uma reconstrução. Aguarde que termine.
admin-searchindex-error = A reconstrução falhou. O índice mantém o que tinha; verifique os registos do servidor e tente novamente.
admin-searchindex-status-error = Não foi possível ler o estado do índice de pesquisa.
admin-searchindex-unavailable = Este servidor funciona em modo proxy e não tem índice de pesquisa.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = Exigir dois fatores
admin-2fa-intro = Exija um segundo fator (chave de acesso ou aplicação de autenticação) para o início de sessão. Qualquer utilizador pode registar um fator por iniciativa própria; exigi-lo aqui obriga as contas abrangidas a registá-lo no próximo início de sessão.
admin-2fa-load-error = Não foi possível carregar a política de dois fatores.
admin-2fa-save-error = Não foi possível guardar a política de dois fatores.
admin-2fa-global = Exigir dois fatores em toda a implementação
admin-2fa-global-label = Exigir dois fatores em toda a implementação
admin-2fa-global-note = Quando ativado, todas as contas têm de ter um segundo fator. Uma regra por domínio abaixo também pode exigi-lo para um domínio sem o exigir em todo o lado.
admin-2fa-domains-heading = Requisitos por domínio
admin-2fa-domains-empty = Nenhum requisito por domínio definido.
admin-2fa-col-domain = Domínio
admin-2fa-col-require = Obrigatório
admin-2fa-require-for = Exigir dois fatores para { $domain }
admin-2fa-add-domain = Domínio
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Exigir dois fatores para este domínio
admin-2fa-add-require-label = Exigir dois fatores para este domínio
admin-2fa-add-save = Adicionar regra de domínio

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Saída
admin-egress-heading = Rotas de proxy de saída
admin-egress-intro = Encaminhe os pedidos de saída do servidor através de um proxy HTTP CONNECT ou SOCKS5. O Mailwoman continua a aplicar a sua própria política de endereços — nunca é pedido ao proxy que resolva um nome.
admin-egress-empty = Nenhuma rota configurada. Os pedidos de saída são diretos.
admin-egress-none-active = Nenhuma rota está em uso. Os pedidos de saída são diretos. Uma rota guardada só transporta tráfego depois de escolher «Usar esta rota».
admin-egress-in-use = Os pedidos de saída usam a rota «{ $id }».
admin-egress-deactivate = Deixar de usar um proxy
admin-egress-activate = Usar esta rota
admin-egress-activate-for = Usar a rota { $id }
admin-egress-activate-error = Não foi possível mudar para a rota
admin-egress-deactivate-error = Não foi possível deixar de usar a rota
admin-egress-col-state = Estado
admin-egress-state-active = Em uso
admin-egress-state-inactive = Fora de uso
admin-egress-test-scope = Só é possível testar a rota em uso. Um teste obtém um URL fixo através da rota e indica até onde chegou; não altera nada.
admin-egress-probe-url = Obtido: { $url }
admin-egress-load-error = Não foi possível carregar as rotas de saída
admin-egress-save-error = Não foi possível guardar a rota
admin-egress-delete-error = Não foi possível eliminar a rota
admin-egress-invalid = São obrigatórios um id, um anfitrião e uma porta superior a zero.
admin-egress-username-required = Uma palavra-passe requer um nome de utilizador; sem ele, a rota autenticar-se-ia como ninguém.
admin-egress-col-id = Id
admin-egress-col-route = Rota
admin-egress-col-username = Nome de utilizador
admin-egress-col-credentials = Palavra-passe
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
admin-egress-host = Anfitrião
admin-egress-port = Porta
admin-egress-username = Nome de utilizador
admin-egress-password = Palavra-passe
admin-egress-password-unchanged = Deixe em branco para manter a palavra-passe guardada
admin-egress-allow-plaintext = Permitir origens http em texto simples através desta rota
admin-egress-allow-plaintext-note = Desativado por predefinição. Uma origem https é o que impede um proxy de ler o tráfego que transporta; permitir http significa que o proxy o vê.
admin-egress-test = Testar
admin-egress-testing = A testar…
admin-egress-test-failed = Não foi possível executar o teste, pelo que nada se sabe sobre esta rota.
admin-egress-outcome-connected = Ligado
admin-egress-outcome-auth-rejected = Autenticação rejeitada
admin-egress-outcome-refused-by-policy = Recusado pela política
admin-egress-outcome-dns-failed = O nome não foi resolvido
admin-egress-outcome-unreachable = Inacessível
admin-egress-outcome-origin-tls-failed = Falha de TLS na origem
admin-egress-outcome-route-invalid = A rota não é válida
admin-egress-outcome-unknown = Resultado não reconhecido
admin-egress-stage-dns = parou na resolução do nome
admin-egress-stage-connect = parou durante a ligação ao proxy
admin-egress-stage-tunnel = parou durante a abertura do túnel
admin-egress-stage-origin = parou na origem
admin-egress-stage-unknown = parou numa fase não reconhecida
admin-egress-proxied-yes = Passou pelo proxy
admin-egress-proxied-no = Não passou pelo proxy
admin-nav-sessions = Sessões
admin-sessions-domain = Filtro de domínio
admin-sessions-account = Filtro de ID de conta
admin-sessions-client = Filtro de tipo de cliente
admin-sessions-preview = Pré-visualizar revogação de sessões
admin-sessions-count = Sessões selecionadas: { $count }
admin-sessions-confirm = Revogar estas sessões
admin-sessions-truncated = São apresentadas no máximo 1000 sessões. Restrinja os filtros para inspecionar outras sessões.
admin-security-center-title = Centro de segurança
admin-security-center-policy-help = Defina políticas de autenticação e de sessão globalmente ou para um domínio, grupo, perfil ou conta. Os limites de tempo da implementação continuam a aplicar-se.
admin-security-center-posture = Lista de verificação de segurança
admin-security-center-coverage = Contas com um fator de autenticação: { $enrolled } de { $total } contas conhecidas
admin-security-center-admins = Administradores sem segundo fator: { $count }
admin-security-center-keys = Chaves de aplicação ativas sem utilização há mais de 90 dias: { $count }
admin-security-center-allowlist-on = Está configurada uma lista de permissões ou predefinição a montante
admin-security-center-allowlist-off = Não está configurada nenhuma lista de permissões nem predefinição a montante
admin-security-center-env-on = O início de sessão de administrador por ambiente está ativado
admin-security-center-env-off = O início de sessão de administrador por ambiente está desativado

admin-users-back = Voltar aos utilizadores
admin-users-identity = Identidade do utilizador
admin-users-id = Identificador da conta
admin-users-last-signin = Último início de sessão

admin-sso-filter-scope = Filtrar por âmbito
admin-sso-all-scopes = Todos os âmbitos
admin-sso-back = Voltar aos fornecedores de início de sessão
admin-sso-editor-intro = Configure este fornecedor de início de sessão e, em seguida, guarde ou cancele para voltar à tabela de fornecedores.

admin-session-mailbox-title = Sessões de caixa de correio
admin-session-console-title = Sessões de administrador
admin-session-empty = Nenhuma sessão corresponde a estes filtros.
admin-session-search = Pesquisar conta, dispositivo ou IP
admin-session-account = Conta
admin-session-device = Dispositivo e cliente
admin-session-ip = Endereço IP
admin-session-activity = Última atividade
admin-session-created = Criada
admin-session-expiry = Expiração
admin-session-status = Estado
admin-session-active = Ativa
admin-session-idle-expiry = Expiração por inatividade
admin-session-revoke-account = Revogar a sessão de { $account }
admin-session-revoke-help = Isto termina esta sessão. A conta pode voltar a iniciar sessão.
admin-session-console-metadata = Os metadados de dispositivo, IP e localização do administrador não são registados para estas sessões.

admin-session-current = Esta sessão
admin-session-revoke = Revogar sessão

admin-egress-scope = Âmbito da rota
admin-egress-scope-help = Utilize global ou domain:example.org para um domínio configurado existente. Os aliases de domínio usam o seu domínio canónico. Apenas as classes de tráfego selecionadas usam esta rota.
admin-egress-classes = Tráfego transportado por esta rota
admin-egress-classes-help = Selecione pelo menos uma classe. As outras classes mantêm a sua própria decisão de rota. O push continua a ser apenas HTTPS e para endereços públicos; selecionar um proxy nunca permite destinos de push privados.
admin-egress-class-remote_content = Imagens e ficheiros remotos de mensagens
admin-egress-class-ics_feeds = Feeds de subscrição de calendários
admin-egress-class-keyservers = Servidores de chaves públicas
admin-egress-class-webhooks = Webhooks de saída
admin-egress-class-admin_detect = Deteção e verificações do administrador
admin-egress-class-pki = Infraestrutura de certificados
admin-egress-class-dns_auth = Pedidos DNS de autenticação de correio
admin-egress-class-unsubscribe = Pedidos de cancelamento de subscrição de listas
admin-egress-class-siem = Coletores de eventos de segurança
admin-egress-class-analytics = Análises de utilização opcionais
admin-egress-class-nextcloud = Ficheiros Nextcloud e pedidos DAV
admin-egress-class-oauth = Pedidos a fornecedores OAuth
admin-egress-class-update_check = Verificações de atualizações
admin-egress-class-geoip = Transferências da base de dados de países
admin-egress-class-sieve = Transporte de regras ManageSieve
admin-egress-class-push = Entrega de push do browser

admin-obs-ban-expiry = Expiração (hora local deste dispositivo)
admin-obs-ban-expiry-help = Deixe vazio para um banimento permanente. A expiração tem de ser no futuro. As verificações de estado continuam disponíveis, mas isto pode bloquear a sua própria sessão ativa.
admin-obs-ban-expired = Expirado
admin-obs-ban-until = Ativo até
admin-obs-ban-permanent = Ativo até ser removido
admin-obs-ban-save = Guardar banimento de IP
admin-obs-ban-cancel = Cancelar
admin-obs-ban-invalid-expiry = Escolha uma expiração futura ou deixe-a vazia.
admin-obs-ban-status = Estado e expiração
admin-obs-ban-search = Pesquisar banimentos de IP
admin-obs-ban-active-only = Apenas banimentos ativos

admin-egress-class-package_tracking = Fornecedor opcional de seguimento de encomendas

admin-obs-locks-title = Bloqueios temporários de início de sessão
admin-obs-locks-note = Credenciais falhadas podem bloquear temporariamente o início de sessão a partir de um IP de origem. Os contadores de administrador e de caixa de correio são separados. Estes bloqueios expiram automaticamente; tentativas repetidas não os prolongam. Os banimentos globais manuais de IP são geridos separadamente.
admin-obs-locks-back = Voltar à observabilidade
admin-obs-locks-refresh = Atualizar bloqueios
admin-obs-locks-configure = Configurar limites de início de sessão
admin-obs-lock-surface = Superfície de início de sessão
admin-obs-lock-surface-all = Todas as superfícies de início de sessão
admin-obs-lock-surface-admin = Administrador
admin-obs-lock-surface-webmail = Caixa de correio
admin-obs-lock-failures = Credenciais rejeitadas
admin-obs-lock-until = Bloqueado até
admin-obs-locks-empty = Sem bloqueios temporários de início de sessão.
admin-obs-locks-search = Pesquisar bloqueios temporários de início de sessão
admin-obs-lock-unlock = Desbloquear
admin-obs-lock-unlock-target = Desbloquear o início de sessão { $surface } para { $ip }
admin-obs-lock-confirm = Repor o contador de credenciais falhadas e permitir outra tentativa de início de sessão a partir desta origem. Os banimentos globais manuais de IP continuam a aplicar-se. É necessária uma nova verificação de segurança.
admin-obs-lock-unlocked = Início de sessão { $surface } desbloqueado para { $ip }.

admin-obs-geo-title = Política de países para início de sessão
admin-obs-geo-note = Este diagnóstico avalia o seu endereço de origem fidedigno. A política de países afeta apenas os novos inícios de sessão; as sessões existentes não são revogadas. Reveja o resultado antes de ativar uma política, sem esquecer a decisão sobre o início de sessão de administrador.
admin-obs-geo-unavailable = A base de dados local de países está em falta ou ilegível. Neste estado, as políticas de países ativas recusam o início de sessão, mesmo quando os endereços não mapeados são permitidos. Transfira ou carregue uma base de dados na página Base de dados de países; tem efeito sem reiniciar. Quando MW_GEOIP_DB está definida, corrija ou substitua o ficheiro que indica.
admin-obs-geo-refresh = Atualizar diagnóstico
admin-obs-geo-configure = Configurar política de países
admin-obs-geo-field = Detalhe da política
admin-obs-geo-value = Valor atual
admin-obs-geo-empty = Nenhum diagnóstico disponível.
admin-obs-geo-field-source = IP de origem fidedigno
admin-obs-geo-field-country = País resolvido
admin-obs-geo-field-database = Base de dados local de países
admin-obs-geo-field-scope = Âmbito do início de sessão
admin-obs-geo-field-mode = Modo da lista de países
admin-obs-geo-field-countries = Códigos de país
admin-obs-geo-field-unknown = Política para endereços não mapeados
admin-obs-geo-field-admin = A sua decisão de início de sessão de administrador
admin-obs-geo-field-mailbox = A sua decisão de início de sessão na caixa de correio
admin-obs-geo-value-off = Desativado
admin-obs-geo-value-allow = Permitir
admin-obs-geo-value-deny = Recusar
admin-obs-geo-value-unavailable = Indisponível — as políticas ativas recusam o início de sessão
admin-obs-geo-value-ready = Pronta
admin-obs-geo-value-unknown = Desconhecido
admin-obs-geo-value-empty = Lista vazia
admin-obs-geo-value-admin = Início de sessão de administrador
admin-obs-geo-value-mailbox = Início de sessão na caixa de correio
admin-obs-geo-value-all = Todos os inícios de sessão

admin-obs-geo-field-sessions = Pedidos autenticados existentes
admin-obs-geo-sessions-on = Verificações de país ativadas
admin-obs-geo-sessions-off = Apenas verificações no início de sessão
admin-obs-geo-session-note = A política de países configurada também verifica pedidos autenticados, acesso nativo por bearer e conteúdo assinado do leitor. Os clientes bloqueados podem terminar sessão. Isto não apaga sessões nem termina streams já abertos; os streams são novamente verificados quando voltam a ligar-se. Reveja a sua decisão de administrador antes de o ativar.
admin-obs-geo-configure-sessions = Configurar verificações de sessões existentes
admin-egress-class-plugins = Pedidos de plugins e fornecedores aprovados
admin-egress-destinations-title = Permissões de destinos internos
admin-egress-destinations-help = Permita que uma classe de tráfego contacte um serviço privado aprovado. A seleção de rota e as permissões de nome de anfitrião dos plugins continuam a aplicar-se. As outras classes de tráfego não recebem acesso.
admin-egress-destinations-back = Voltar às rotas de saída
admin-egress-destinations-add = Adicionar permissão de destino
admin-egress-destinations-empty = Não há permissões de destinos privados configuradas.
admin-egress-destination-class = Classe de tráfego
admin-egress-destination-choose = Escolha uma classe de tráfego
admin-egress-destination-class-help = Selecione o único serviço autorizado a usar esta permissão. As permissões de análises nunca autorizam plugins nem conteúdo de correio.
admin-egress-destination-target = Destino
admin-egress-destination-target-help = Introduza um nome de anfitrião ou endereço IP exato, opcionalmente com uma porta (ews.example.test:443), ou uma rede privada delimitada como 10.20.0.0/16. URLs, metadados, endereços de loopback e link-local são recusados.
admin-egress-destination-note = Nota administrativa
admin-egress-destination-actions = Ações
admin-egress-destination-revoke = Revogar permissão
admin-egress-destination-confirm = Os novos pedidos a este destino privado serão recusados, a menos que outra permissão explícita o abranja.
admin-egress-destination-confirm-button = Confirmar revogação

admin-deployment-title = Avaliação de segurança da implementação
admin-deployment-back = Voltar aos diagnósticos
admin-deployment-refresh = Atualizar avaliação
admin-deployment-help = Observações só de leitura deste processo do servidor e deste pedido. Estas verificações não certificam a implementação nem contactam serviços externos.
admin-deployment-search = Procurar uma verificação
admin-deployment-status = Estado
admin-deployment-status-all = Todas as verificações
admin-deployment-status-pass = Presente
admin-deployment-status-warning = Requer revisão
admin-deployment-status-info = Informação
admin-deployment-empty = Nenhuma verificação corresponde aos filtros.
admin-deployment-check = Verificação
admin-deployment-observed = Valor observado
admin-deployment-settings = Configuração
admin-deployment-limits = O que esta avaliação não testa
admin-deployment-snapshot = Instantâneo obtido às { $time }.
admin-deployment-yes = Sim
admin-deployment-no = Não
admin-deployment-check-https = Pedido HTTPS
admin-deployment-check-secureCookies = Cookies seguros
admin-deployment-check-hsts = Segurança de transporte estrita
admin-deployment-check-csrf = Proteção CSRF estrita
admin-deployment-check-adminSessions = Limites das sessões de administrador
admin-deployment-check-adminFactors = Administradores ativos sem segundo fator
admin-deployment-check-mailboxFactors = Registo de fatores nas caixas de correio
admin-deployment-check-signinPacing = Atraso entre tentativas de início de sessão (segundos)
admin-deployment-check-credentialLocks = Bloqueios temporários por falha de credenciais
admin-deployment-check-geography = Política de países
admin-deployment-check-renderIsolation = Isolamento da pré-visualização de documentos
admin-deployment-value-https = HTTPS efetivo: { $effectiveRequestHttps }; HTTPS externo configurado: { $configuredHttps }.
admin-deployment-value-adminSessions = Limite de inatividade: { $idleSeconds } segundos; limite absoluto: { $absoluteSeconds } segundos.
admin-deployment-value-mailboxFactors = { $enrolled } de { $knownAccounts } contas conhecidas têm um fator.
admin-deployment-value-credentialLocks = Limite de administrador: { $administratorLimit }; limite de caixa de correio: { $mailboxLimit }; janela: { $windowSeconds } segundos; bloqueio: { $lockSeconds } segundos.
admin-deployment-value-geography = Âmbito: { $scope }; verificações de pedidos existentes: { $existingRequests }; base de dados local pronta: { $localDatabaseReady }.
admin-deployment-value-renderIsolation = Renderizador isolado obrigatório: { $jailRequired }; worker configurado: { $workerConfigured }.
admin-deployment-limit-no-external-tls-or-firewall-probe = Os certificados TLS públicos, a configuração do proxy inverso e a exposição na firewall não são testados.
admin-deployment-limit-no-upstream-authentication = A autenticação a montante e o acesso às caixas de correio não são testados.
admin-deployment-limit-configured-worker-not-executed = O worker de documentos configurado não é executado por esta avaliação.
admin-deployment-limit-not-a-security-certification = Uma definição comunicada é prova de configuração, não uma garantia de proteção completa.
admin-egress-destination-note-help = Descreva por que motivo este serviço privado é aprovado. Esta nota destina-se aos administradores e nunca é enviada a montante.

admin-session-select = Selecionar
admin-session-select-account = Selecionar a sessão de { $account }
admin-session-select-visible = Selecionar todas as sessões visíveis
admin-session-revoke-selected = Revogar sessões selecionadas ({ $count })
admin-session-batch-confirm = Revogar apenas as sessões selecionadas. Poderá ser-lhe pedido que confirme a sua identidade. As sessões falhadas ou ignoradas serão indicadas.
admin-session-batch-current = A sua sessão de administrador atual está selecionada. Será revogada em último lugar, o que termina a sua sessão, apenas se as outras sessões selecionadas forem tratadas com êxito. Se outro pedido falhar, esta sessão é mantida para que possa rever o resultado.
admin-session-batch-confirm-button = Confirmar revogações selecionadas
admin-session-batch-result = Revogadas: { $revoked }. Já terminadas: { $unchanged }. Não confirmadas: { $failed }. Não tentadas: { $skipped }.
admin-session-refresh-failed = Não foi possível atualizar a lista de sessões. Recarregue-a antes de selecionar mais sessões.

admin-deployment-check-connectionNegotiation = Limites de negociação de ligações de entrada
admin-deployment-value-connectionNegotiation = Configuração de arranque do listener integrado: prazo de TLS { $tlsTimeoutMs } ms, até { $tlsMaximum } handshakes; prazo do cabeçalho PROXY { $proxyTimeoutMs } ms, até { $proxyMaximum } negociações. As alterações exigem reinício. Os limites de proxies externos não são medidos aqui.

admin-deployment-check-httpAdmission = Admissão HTTP e limites de pedidos
admin-deployment-value-httpAdmission = Configuração de arranque do listener integrado: { $connections } ligações, { $requests } pedidos até à conclusão da resposta, { $headerBytes } bytes de cabeçalho e { $headers } cabeçalhos. Prazo de abertura HTTP e de cabeçalho HTTP/1 { $headerTimeoutMs } ms; prazo do corpo do pedido { $bodyTimeoutMs } ms; { $http2Streams } streams HTTP/2 por ligação; drenagem ordenada { $shutdownTimeoutMs } ms. Os limites fixos exigem reinício. Os streams de resposta não estão sujeitos ao prazo do corpo do pedido.

admin-personal-dashboard-keep-checklist = Manter em destaque a lista de configuração concluída
admin-personal-dashboard-keep-checklist-help = Caso contrário, concluir todos os itens recolhe a lista abaixo dos outros widgets do painel. Um item por concluir repõe a sua posição habitual.

admin-protection-title = Proteção agregada e adaptativa
admin-protection-back = Voltar à observabilidade
admin-protection-refresh = Atualizar
admin-protection-explanation = A proteção opcional e determinística compara pedidos invulgares tipificados com pedidos normais em janelas curtas e longas. Contagens de anomalias independentes resistem à diluição pelo tráfego normal. Ambos os modos estão desativados por predefinição; os limites de segurança dos protocolos continuam ativos.
admin-protection-current-policy = Política atual
admin-protection-modes = Orçamentos de pedidos: { $rate }. Proteção adaptativa: { $adaptive }.
admin-protection-mode-off = Desativado
admin-protection-mode-observe = Apenas observar
admin-protection-mode-enforce = Aplicar
admin-protection-retention = No máximo { $subjects } registos de sujeitos e { $events } decisões; as decisões expiram após { $days } dias. As alterações propagam-se às outras réplicas em { $seconds } segundos.
admin-protection-coverage = Os contadores de cliente usam o endereço de origem fidedigno. Os contadores de conta requerem uma sessão verificada, uma chave de conta nativa ou uma autoridade de conteúdo assinado. As contas proxy são separadas por domínio de serviço; as chaves proxy antigas sem esse domínio não passam a aplicação ao nível da conta. As recusas anteriores do analisador HTTP e da capacidade de ligação são limites de segurança separados, não observações adaptativas. Os streams existentes são verificados no pedido seguinte.
admin-protection-configure = Configurar modos, janelas, limiares e isenções
admin-protection-subjects = Estado atual dos sujeitos
admin-protection-subject = Impressão digital do sujeito
admin-protection-kind-client = Endereço do cliente
admin-protection-kind-account = Conta de caixa de correio
admin-protection-kind-admin = Administrador
admin-protection-short = Curta: invulgares / normais
admin-protection-long = Longa: invulgares / normais
admin-protection-action-until = Prazo da ação atual
admin-protection-monitoring = Nenhum prazo ativo
admin-protection-observed-at = Última observação
admin-protection-counts-note = As contagens mostram observações invulgares / normais à hora de observação indicada. As impressões digitais são hashes com finalidade específica; os endereços e IDs de conta em bruto não são guardados neste histórico. Um prazo fixo não se prolonga quando os pedidos bloqueados se repetem.
admin-protection-empty = Atualmente não é mantido nenhum estado de sujeitos.
admin-protection-next = Página seguinte
admin-protection-history = Decisões de proteção
admin-protection-history-empty = Não são mantidas decisões de proteção.
admin-protection-reason = Acionador
admin-protection-action = Decisão
admin-protection-evidence = Evidência
admin-protection-evidence-counts = Janela curta: { $shortBad } invulgares, { $shortNormal } normais. Janela longa: { $longBad } invulgares, { $longNormal } normais.
admin-protection-evidence-thresholds = Janela curta aplicada: { $shortSecs } segundos, absoluto { $shortAbsolute }, proporção { $shortRatio }% após { $shortMinimum } amostras. Janela longa aplicada: { $longSecs } segundos, absoluto { $longAbsolute }, proporção { $longRatio }% após { $longMinimum } amostras. Qualquer limiar ativado pode acionar a proteção.
admin-protection-no-signal = Nenhum sinal de anomalia
admin-protection-reason-request-rate = Orçamento agregado de pedidos
admin-protection-reason-short-absolute = Contagem de anomalias na janela curta
admin-protection-reason-short-ratio = Proporção de pedidos invulgares na janela curta
admin-protection-reason-long-absolute = Contagem de anomalias na janela longa
admin-protection-reason-long-ratio = Proporção de pedidos invulgares na janela longa
admin-protection-reason-manual-clear = Estado limpo pelo administrador
admin-protection-action-throttle = Limitação temporária
admin-protection-action-block = Bloqueio temporário
admin-protection-action-cleared = Estado limpo
admin-protection-enforced = Aplicado
admin-protection-observed = Apenas observado
admin-protection-clear = Limpar estado
admin-protection-clear-target = Limpar o estado de proteção de { $subject }
admin-protection-clear-body = Limpe os contadores e a ação temporária deste sujeito depois de confirmar a sua identidade. Isto não altera a política nem remove banimentos manuais. Novas observações podem voltar a acionar a proteção. Se o estado mudar antes, atualize antes de tentar novamente.
admin-protection-cleared = Estado de proteção limpo para { $subject }.

admin-accounts-last-super-protected = Este é o último Super admin ativo. As ações que removem este acesso estão desativadas. Conceda primeiro acesso Super admin sem restrições a outra conta.
admin-accounts-last-super-allowed = Este é o último Super admin ativo. A política explícita da implementação permite perder este acesso; seria necessária a administração local da implementação para o recuperar.
admin-accounts-last-super-confirm = Esta ação pode deixar a implementação sem qualquer acesso Super admin. A recuperação requer a administração local da implementação. Confirme apenas se pretende perder este acesso.

admin-history-title = Histórico de pedidos
admin-history-period = Período
admin-history-default = Predefinição do administrador
admin-history-custom = Período personalizado
admin-history-period-3600 = Últimos 60 minutos
admin-history-period-21600 = Últimas 6 horas
admin-history-period-86400 = Últimas 24 horas
admin-history-period-604800 = Últimos 7 dias
admin-history-period-2592000 = Últimos 30 dias
admin-history-from = De (hora local)
admin-history-to = Até (hora local)
admin-history-help = As contagens incluem os pedidos tratados por este servidor. A latência mede quando a resposta está pronta, não quando termina uma transferência em stream. As lacunas significam que não há histórico registado; os intervalos parciais incluem apenas o tráfego observado.
admin-history-range = { $from } a { $to } · { $seconds } segundos por intervalo
admin-history-partial = O histórico abrange { $observed } de { $requested } minutos pedidos. Os períodos em falta não são contados como zero.
admin-history-requests = Pedidos por intervalo
admin-history-load-error = Não foi possível carregar o histórico de pedidos. Atualize para tentar novamente.
admin-history-error-invalid = Introduza datas de início e de fim válidas.
admin-history-error-order = O fim tem de ser posterior ao início.
admin-history-error-future = O período não pode terminar no futuro.
admin-history-error-limit = Este período excede o limite configurado pelo administrador.

admin-history-refresh = Atualizar histórico de pedidos
admin-history-errors = Erros do servidor (5xx) por intervalo
admin-users-legacy-alias = Esta conta inicia sessão através de um alias de domínio. Registo administrativo existente: { $address }. As suas definições e histórico são preservados; os registos não foram fundidos nem renomeados.

admin-history-latency-resolution = As amostras inferiores a um milissegundo mantêm os microssegundos quando disponíveis; os instantâneos mais antigos mantêm a sua precisão original. Os tempos de 1 ms a 1 segundo usam milissegundos inteiros, e os tempos mais lentos usam intervalos delimitados mais largos. Os percentis descrevem o tempo até a resposta estar pronta, não a transferência completa.
admin-history-latency-legacy = Este intervalo inclui estimativas de latência grosseiras antigas ou mistas. A sua precisão de medição original não pode ser recuperada.

admin-users-access-title = Acesso ao Mailwoman
admin-users-access-help = Estes controlos afetam apenas o início de sessão local. O correio, as definições e o histórico a montante são preservados. As outras restrições de política ou de indicadores da conta continuam a aplicar-se.
admin-users-block-for = Bloquear o início de sessão de { $account }
admin-users-unblock-for = Desbloquear o início de sessão de { $account }
admin-users-delete-for = Remover o acesso ao Mailwoman de { $account }
admin-users-block-help = Bloquear novos inícios de sessão e revogar as sessões atuais do Mailwoman.
admin-users-unblock-help = Remover esta retenção de acesso do diretório. As outras restrições continuam a aplicar-se.
admin-users-delete-help = Eliminar o acesso local e preservar a caixa de correio a montante e o histórico local.
admin-users-deleted = Acesso local eliminado
admin-users-lifecycle-block-title = Bloquear o início de sessão no Mailwoman
admin-users-lifecycle-unblock-title = Desbloquear o início de sessão no Mailwoman
admin-users-lifecycle-delete-title = Eliminar o acesso local ao Mailwoman
admin-users-lifecycle-block-body = Isto altera a retenção de início de sessão do diretório. O bloqueio revoga as sessões atuais. Não altera a caixa de correio a montante nem se sobrepõe a outras restrições da conta.
admin-users-lifecycle-delete-body = Isto mantém um registo de acesso eliminado para impedir a reinscrição automática. As definições e o histórico do Mailwoman e a caixa de correio a montante continuam guardados. Esta ação não pode ser anulada a partir desta página.
admin-users-lifecycle-block-confirm = Bloquear início de sessão
admin-users-lifecycle-unblock-confirm = Desbloquear início de sessão
admin-users-lifecycle-delete-confirm = Eliminar acesso local

admin-users-lifecycle-unblock-body = Remover o bloqueio de início de sessão do diretório local. As outras restrições de segurança continuam a aplicar-se; isto não restaura um acesso eliminado.
admin-users-legacy-clear = Limpar bloqueio de início de sessão antigo
admin-users-legacy-block-help = Um indicador de início de sessão antigo também bloqueia este utilizador. Limpe-o explicitamente antes de alterar o bloqueio do diretório. Os outros indicadores e restrições de segurança continuam em vigor.
admin-users-lifecycle-legacy-title = Limpar o bloqueio de início de sessão antigo?
admin-users-lifecycle-legacy-body = Limpar apenas o indicador antigo de desativação. Preservar todos os outros indicadores e o estado de acesso do diretório. Isto, por si só, não garante que o início de sessão seja permitido.
admin-users-lifecycle-legacy-confirm = Limpar bloqueio antigo

admin-transfer-title = Configuração e cópias de segurança
admin-transfer-scope = Os ficheiros portáteis contêm os valores das definições globais registadas e os respetivos bloqueios. Excluem domínios, dados de utilizadores, credenciais, variáveis de ambiente e a configuração dedicada das integrações. As cópias de segurança de recuperação completa são separadas.
admin-transfer-export = Exportar definições
admin-transfer-import = Importar definições
admin-transfer-backups = Destinos das cópias de segurança
admin-transfer-export-help = Transfira as políticas globais guardadas como um ficheiro JSON com versão. As políticas não definidas continuam por definir; as confirmações de segurança dedicadas são excluídas.
admin-transfer-download = Transferir definições
admin-transfer-exported = { $count } políticas exportadas; { $excluded } políticas dedicadas foram excluídas.
admin-transfer-import-help = Escolha um ficheiro de definições para validar cada entrada e rever as alterações antes de as aplicar. As chaves em falta ficam inalteradas. As alterações usam verificações de revisão individuais; uma alteração simultânea pode parar a importação depois de entradas anteriores terem sido aplicadas.
admin-transfer-file = Ficheiro JSON de definições
admin-transfer-too-large = Escolha um ficheiro de definições com menos de 2 MiB.
admin-transfer-review = Rever alterações às definições
admin-transfer-key = Definição
admin-transfer-before = Política atual
admin-transfer-after = Política importada
admin-transfer-apply = Aplicar alterações revistas
admin-transfer-confirm = Aplicar as políticas revistas? Isto pode alterar predefinições e bloqueios para os utilizadores. É necessária nova autenticação. Se uma alteração simultânea parar a importação, o resultado indica quantas políticas anteriores foram aplicadas.
admin-transfer-imported = { $count } políticas aplicadas.
admin-transfer-partial = { $count } políticas aplicadas; a importação parou em { $key } (HTTP { $status }). Recarregue e reveja novamente o ficheiro antes de continuar.
admin-transfer-uncertain = { $count } alterações de políticas anteriores confirmadas. O resultado para { $key } é incerto após um erro do servidor (HTTP { $status }); pode já ter sido guardado. Recarregue as definições atuais e reveja uma nova pré-visualização antes de tentar novamente.
admin-transfer-backup-help = Os instantâneos completos da implementação incluem a base de dados, os carregamentos, a configuração e a chave do servidor. As cópias para destinos exigem encriptação do arquivo e verificação da soma de verificação. Configure os destinos no anfitrião da implementação; o servidor web não recebe privilégios sobre o sistema de ficheiros do anfitrião nem sobre o Docker.
admin-transfer-destinations = Destinos de cópia de segurança suportados
admin-transfer-method = Método
admin-transfer-setup = Configuração
admin-transfer-method-local = Diretório local
admin-transfer-method-s3 = Armazenamento de objetos compatível com S3
admin-transfer-method-sftp = SFTP / túnel SSH
admin-transfer-method-webdav = WebDAV
admin-transfer-method-smb = SMB / CIFS
admin-transfer-method-nfs = NFS
admin-transfer-setup-local = Diretório absoluto existente no anfitrião da implementação.
admin-transfer-setup-s3 = Remoto rclone S3 com nome, com bucket/caminho e credenciais geridas pelo anfitrião.
admin-transfer-setup-sftp = Remoto SFTP com nome e chaves de anfitrião fixadas; um túnel SSH gerido pelo operador pode fornecer a ligação.
admin-transfer-setup-webdav = Remoto WebDAV HTTPS com nome. Para o Nextcloud, use o método Nextcloud.
admin-transfer-setup-smb = Remoto SMB com nome, com partilha/caminho e credenciais geridas pelo anfitrião.
admin-transfer-setup-nfs = Montagem NFS existente. Uma montagem ausente é recusada em vez de se escrever no disco local.
admin-transfer-backup-template = Transferir modelo de destinos
admin-transfer-backup-host = Ative e configure os destinos no modelo e, em seguida, defina BACKUP_TARGETS_FILE e BACKUP_RECIPIENT em deploy.env. O comando de cópia de segurança existente do deploy.sh e o temporizador do systemd utilizam-nos. Todos os destinos do modelo começam desativados; transferir o modelo não altera uma implementação em execução.
admin-transfer-method-nextcloud = Nextcloud
admin-transfer-setup-nextcloud = URL HTTPS do servidor Nextcloud, nome de utilizador, um ficheiro de palavra-passe de aplicação legível apenas pela conta de implementação e uma pasta de destino. Usa a via WebDAV com envios em blocos e cria a pasta quando não existe.
admin-transfer-full-backups = Cópias de segurança completas da implementação
admin-ncbackup-title = Cópias de segurança das definições para o Nextcloud
admin-ncbackup-help = Este servidor envia a exportação das definições (políticas globais registadas, sem credenciais) para uma pasta Nextcloud sua, verifica cada cópia transferindo-a novamente e remove as cópias mais antigas além da retenção abaixo. Usa a sua própria palavra-passe de aplicação, separada das contas Nextcloud que os utilizadores associam.
admin-ncbackup-no-permission = Enviar cópias de segurança das definições requer a permissão de exportação da configuração.
admin-ncbackup-url = URL do servidor Nextcloud
admin-ncbackup-url-help = Apenas HTTPS, por exemplo https://cloud.example.com ou https://example.com/nextcloud. O caminho WebDAV é acrescentado automaticamente.
admin-ncbackup-username = Nome de utilizador Nextcloud
admin-ncbackup-password = Palavra-passe de aplicação
admin-ncbackup-password-help = Crie uma palavra-passe de aplicação no Nextcloud em Definições pessoais → Segurança → Dispositivos e sessões. Não use a palavra-passe de início de sessão da conta.
admin-ncbackup-password-stored = Está guardada uma palavra-passe de aplicação, que não pode ser mostrada. Introduza uma nova apenas para a substituir.
admin-ncbackup-password-docs = Como criar uma palavra-passe de aplicação
admin-ncbackup-folder = Pasta de destino
admin-ncbackup-folder-help = Uma pasta nos ficheiros deste utilizador, por exemplo Backups/Mailwoman. As pastas em falta são criadas.
admin-ncbackup-keep = Cópias a manter
admin-ncbackup-keep-days = Dias a manter (opcional)
admin-ncbackup-keep-help = Como BACKUP_KEEP e BACKUP_KEEP_DAYS: uma cópia fora das mais recentes só é removida quando também é mais antiga do que este número de dias. Só são removidos ficheiros com o nome mailwoman-settings-<hora>.json.
admin-ncbackup-ca = Autoridade de certificação privada
admin-ncbackup-ca-label = Certificados de AC adicionais (PEM)
admin-ncbackup-ca-help = Apenas para um servidor cujo certificado foi assinado por uma AC privada. Deixe vazio para certificados de confiança pública.
admin-ncbackup-webdav = Pasta WebDAV:
admin-ncbackup-save = Guardar destino
admin-ncbackup-test = Testar ligação
admin-ncbackup-run = Fazer cópia agora
admin-ncbackup-remove = Remover destino
admin-ncbackup-saved = Destino Nextcloud guardado.
admin-ncbackup-removed = Destino Nextcloud removido.
admin-ncbackup-test-ok = A ligação funciona: sessão iniciada, a pasta está acessível e um ficheiro de teste foi escrito, lido de volta e eliminado.
admin-ncbackup-test-ok-created = A ligação funciona: sessão iniciada, a pasta foi criada e um ficheiro de teste foi escrito, lido de volta e eliminado.
admin-ncbackup-run-ok = { $file } ({ $bytes } bytes) enviado e verificado em { $folder }. Cópias antigas removidas: { $removed }.
admin-ncbackup-last-ok = Última cópia: { $file }, { $at }.
admin-ncbackup-last-failed = A última cópia falhou, { $at }: { $reason }
admin-ncbackup-step-failed = { $step } falhou: { $reason }
admin-ncbackup-step-config = Configuração
admin-ncbackup-step-auth = Início de sessão
admin-ncbackup-step-folder = Acesso à pasta
admin-ncbackup-step-write = Envio
admin-ncbackup-step-verify = Verificação
admin-ncbackup-step-delete = Eliminação
admin-ncbackup-step-prune = Retenção
admin-ncbackup-error-authRejected = O Nextcloud rejeitou o nome de utilizador ou a palavra-passe de aplicação. Crie uma nova palavra-passe de aplicação e guarde-a.
admin-ncbackup-error-userNotFound = O Nextcloud não tem ficheiros para este nome de utilizador. Verifique o URL do servidor e o nome de utilizador.
admin-ncbackup-error-folderIsFile = O caminho de destino é um ficheiro, não uma pasta.
admin-ncbackup-error-folderCreateDenied = A pasta não existe e esta conta não a pode criar (HTTP { $status }). Crie-a no Nextcloud ou escolha outra pasta.
admin-ncbackup-error-writeDenied = Esta conta não pode escrever na pasta (HTTP { $status }).
admin-ncbackup-error-fileExists = Já existe um ficheiro com o mesmo nome; nada foi substituído.
admin-ncbackup-error-quotaExceeded = A conta Nextcloud não tem espaço de armazenamento disponível.
admin-ncbackup-error-verifyMismatch = A cópia lida de volta do Nextcloud é diferente da que foi enviada.
admin-ncbackup-error-deleteDenied = O Nextcloud recusou eliminar o ficheiro (HTTP { $status }).
admin-ncbackup-error-redirected = O Nextcloud redirecionou o pedido (HTTP { $status }). Introduza o URL HTTPS final do servidor.
admin-ncbackup-error-refused = O Nextcloud respondeu HTTP { $status }.
admin-ncbackup-error-egressBlocked = A política de ligações de saída bloqueia este endereço. Um servidor numa rede privada precisa de uma rota de proxy de saída para o tráfego Nextcloud.
admin-ncbackup-error-timeout = O Nextcloud não respondeu a tempo.
admin-ncbackup-error-tooLarge = A resposta ou o ficheiro excede o limite de 8 MiB.
admin-ncbackup-error-invalidRequest = Não foi possível construir o pedido a partir desta configuração.
admin-ncbackup-error-unreachable = Não foi possível ligar, ou o certificado não pôde ser verificado. Verifique o URL e, para uma AC privada, os certificados de AC.
admin-ncbackup-error-noAppPassword = Não está guardada nenhuma palavra-passe de aplicação.
admin-ncbackup-error-credentialUnreadable = A palavra-passe de aplicação guardada não pode ser aberta com esta chave do servidor. Introduza-a novamente.
admin-ncbackup-error-invalidCa = Os certificados de AC guardados são inválidos.
admin-ncbackup-error-invalidListing = O Nextcloud devolveu uma listagem de pasta que não pôde ser lida.

