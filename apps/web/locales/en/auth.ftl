# Mailwoman — authentication & OAuth-consent strings (source locale: en).
# Covers the mailbox sign-in screen and the OAuth 2.1 authorization/consent screen.
# Admin sign-in lives in admin.ftl (separate session domain).

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = Email address
auth-password = Password
auth-sign-in = Sign in
auth-signing-in = Signing in…
auth-invalid-credentials = Invalid credentials
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = If the email address and password are correct, the account may have been disabled by an administrator. A disabled account cannot sign in until an administrator enables it again.
auth-unreachable = Could not reach the server
# The server has no mail server configured for the address's domain.
auth-address-not-configured = This address is not set up on this server. Contact your administrator.
# The server says why it refused (an administrator chose to explain refusals):
# the account is disabled, or this address may not sign in on this server.
auth-account-disabled = This account has been disabled by an administrator. It cannot sign in until an administrator enables it again.
auth-signin-not-allowed = This address may not sign in on this server. Contact your administrator.
# The id of the server's answer, for a support request.
auth-reference = Reference: { $id }

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Single sign-on
auth-sso-divider = or continue with
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Sign in with { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = Single sign-on did not complete. Please try again or sign in with your password.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Authorize application
auth-consent-title = Authorize access
auth-consent-loading = Loading request…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = wants to access your account.
auth-consent-approved = Admin-approved client
auth-consent-unapproved = Unrecognised client — not admin-approved
auth-consent-requesting = It is requesting
auth-consent-redirects-to = Redirects to
auth-consent-for-resource = For resource
auth-consent-deny = Deny
auth-consent-allow = Allow
auth-consent-error = could not record your decision
auth-domain-hint = You can sign in with just your username for { $domain }.
auth-passkey-sign-in = Sign in with a passkey
auth-passkey-failed = The passkey could not sign you in. Try again or use your password.
