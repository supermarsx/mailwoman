# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = E-mail ou nome de utilizador
auth-login-rate-limited = Demasiadas tentativas de início de sessão. Aguarde antes de tentar novamente.
auth-retry-countdown = Tente novamente dentro de { $seconds } s
auth-password = Palavra-passe
auth-sign-in = Iniciar sessão
auth-signing-in = A iniciar sessão…
auth-invalid-credentials = Credenciais inválidas
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = Se o endereço de e-mail e a palavra-passe estiverem corretos, a conta pode ter sido desativada por um administrador. Uma conta desativada não pode iniciar sessão até que um administrador a volte a ativar.
auth-unreachable = Não foi possível contactar o servidor
auth-upstream-protocol = O início de sessão parou: o servidor de correio enviou uma resposta que este servidor não conseguiu ler. Tente novamente mais tarde ou contacte o administrador.
# The server has no mail server configured for the address's domain.
auth-address-not-configured = Este endereço não está configurado neste servidor. Contacte o seu administrador.
# The server says why it refused (an administrator chose to explain refusals):
# the account is disabled, or this address may not sign in on this server.
auth-account-disabled = Esta conta foi desativada por um administrador. Não pode iniciar sessão até que um administrador a volte a ativar.
auth-signin-not-allowed = Este endereço não pode iniciar sessão neste servidor. Contacte o seu administrador.
# The id of the server's answer, for a support request.
auth-reference = Referência: { $id }

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Início de sessão único
auth-sso-divider = ou continuar com
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Iniciar sessão com { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = O início de sessão único não foi concluído. Tente novamente ou inicie sessão com a sua palavra-passe.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Autorizar aplicação
auth-consent-title = Autorizar acesso
auth-consent-loading = A carregar o pedido…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = pretende aceder à sua conta.
auth-consent-approved = Cliente aprovado pelo administrador
auth-consent-unapproved = Cliente não reconhecido — não aprovado pelo administrador
auth-consent-requesting = Está a pedir
auth-consent-redirects-to = Redireciona para
auth-consent-for-resource = Para o recurso
auth-consent-deny = Recusar
auth-consent-allow = Permitir
auth-consent-error = não foi possível registar a sua decisão
auth-domain-hint = Pode iniciar sessão apenas com o seu nome de utilizador de { $domain }.
auth-passkey-sign-in = Iniciar sessão com uma chave de acesso
auth-passkey-failed = Não foi possível iniciar sessão com a chave de acesso. Tente novamente ou utilize a sua palavra-passe.

auth-remembered = Inícios de sessão anteriores
auth-remembered-help = Endereços utilizados com êxito neste dispositivo. Escolher um preenche apenas o endereço.
auth-forget-address = Esquecer { $address } neste dispositivo
auth-login-timeout = O início de sessão excedeu o tempo limite. Não foi feita nenhuma nova tentativa automática. Verifique a ligação e tente novamente quando estiver pronto.

auth-verification-title = Verificação de início de sessão
auth-verification-loading = A carregar a verificação de início de sessão…
auth-verification-error = Não foi possível concluir a verificação. Repita a verificação antes de iniciar sessão.
auth-verification-required = Conclua uma nova verificação de início de sessão antes de tentar novamente.

auth-identity-title = Partilhar a sua identidade
auth-identity-sign-in = Inicie sessão novamente para continuar este pedido de identidade.
auth-identity-sign-out = Terminar sessão no Mailwoman
auth-identity-expired = Este pedido de identidade expirou ou foi alterado. Volte à aplicação que o fez e comece de novo.
auth-identity-retry = Tentar novamente
auth-identity-disclosure = Esta aplicação recebe as informações de identidade abaixo. Esta permissão não dá acesso ao seu correio.
auth-identity-scope-openid = Um identificador de identidade estável
auth-identity-scope-profile = Informações de perfil disponíveis
auth-identity-scope-email = O endereço de e-mail com sessão iniciada, sem garantia de que tenha sido verificado de forma independente
auth-identity-scope-offline_access = Renovar o acesso à identidade enquanto esta sessão do Mailwoman for válida, até 15 dias após o início de sessão
auth-identity-deny = Cancelar
auth-identity-approve = Permitir acesso à identidade
auth-identity-grants = Aplicações que utilizam a sua identidade
auth-identity-grants-help = Revogue o acesso à identidade de uma aplicação. Isto não remove as permissões separadas da API de correio.
auth-identity-no-grants = Nenhuma aplicação tem atualmente consentimento de identidade.
auth-identity-revoke = Revogar acesso à identidade
auth-maintenance = O webmail está temporariamente em pausa para manutenção. Tente novamente mais tarde.
