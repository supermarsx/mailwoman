# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = Endereço de e-mail
auth-password = Senha
auth-sign-in = Entrar
auth-signing-in = Entrando…
auth-invalid-credentials = Credenciais inválidas
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = Se o endereço de e-mail e a senha estiverem corretos, a conta pode ter sido desativada por um administrador. Uma conta desativada não pode entrar até que um administrador a ative novamente.
auth-unreachable = Não foi possível acessar o servidor
# The server has no mail server configured for the address's domain.
auth-address-not-configured = Este endereço não está configurado neste servidor. Entre em contato com o administrador.

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Login único (SSO)
auth-sso-divider = ou continue com
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Entrar com { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = O login único não foi concluído. Tente novamente ou entre com sua senha.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Autorizar aplicativo
auth-consent-title = Autorizar acesso
auth-consent-loading = Carregando solicitação…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = quer acessar sua conta.
auth-consent-approved = Cliente aprovado pelo administrador
auth-consent-unapproved = Cliente não reconhecido — não aprovado pelo administrador
auth-consent-requesting = Está solicitando
auth-consent-redirects-to = Redireciona para
auth-consent-for-resource = Para o recurso
auth-consent-deny = Negar
auth-consent-allow = Permitir
auth-consent-error = não foi possível registrar sua decisão

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Muitas tentativas de login. Aguarde antes de tentar novamente.
auth-retry-countdown = Tente novamente em { $seconds } s
auth-upstream-protocol = Login interrompido: o servidor de e-mail enviou uma resposta que este servidor não conseguiu ler. Tente novamente mais tarde ou avise o administrador.
auth-account-disabled = Esta conta foi desativada por um administrador. Ela não poderá entrar até que um administrador a ative novamente.
auth-signin-not-allowed = Este endereço não pode entrar neste servidor. Entre em contato com o administrador.
auth-signin-not-provisioned = Ainda não existe uma conta para este endereço, e novas contas não são criadas automaticamente. Peça ao administrador que adicione você.
auth-reference = Referência: { $id }
auth-domain-hint = Para { $domain }, você pode entrar apenas com o nome de usuário.
auth-passkey-sign-in = Entrar com uma passkey
auth-passkey-failed = Não foi possível entrar com a passkey. Tente novamente ou use sua senha.
auth-remembered = Logins anteriores
auth-remembered-help = Endereços usados com sucesso neste dispositivo. Ao escolher um, somente o endereço é preenchido.
auth-forget-address = Esquecer { $address } neste dispositivo
auth-login-timeout = O login expirou. Nenhuma nova tentativa automática foi feita. Verifique a conexão e tente novamente quando quiser.
auth-verification-title = Verificação de login
auth-verification-loading = Carregando a verificação de login…
auth-verification-error = Não foi possível concluir a verificação. Refaça a verificação antes de entrar.
auth-verification-required = Conclua uma nova verificação de login antes de tentar novamente.
auth-identity-title = Compartilhar sua identidade
auth-identity-sign-in = Entre novamente para continuar esta solicitação de identidade.
auth-identity-sign-out = Sair do Mailwoman
auth-identity-expired = Esta solicitação de identidade expirou ou foi alterada. Volte ao aplicativo solicitante e comece de novo.
auth-identity-retry = Tentar novamente
auth-identity-disclosure = Este aplicativo recebe as informações de identidade abaixo. Esta permissão não concede acesso ao seu e-mail.
auth-identity-scope-openid = Um identificador de identidade estável
auth-identity-scope-profile = As informações de perfil disponíveis
auth-identity-scope-email = Seu endereço de e-mail de login, sem afirmar que ele foi verificado de forma independente
auth-identity-scope-offline_access = Renovar o acesso à identidade enquanto esta sessão do Mailwoman permanecer válida, por até 15 dias após o login
auth-identity-deny = Cancelar
auth-identity-approve = Permitir acesso à identidade
auth-identity-grants = Aplicativos que usam sua identidade
auth-identity-grants-help = Revogue o acesso à identidade de um aplicativo. Isso não remove as permissões separadas da API de e-mail.
auth-identity-no-grants = Nenhum aplicativo tem consentimento de identidade no momento.
auth-identity-revoke = Revogar acesso à identidade
auth-maintenance = O webmail está temporariamente pausado para manutenção. Tente novamente mais tarde.
