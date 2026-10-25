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
