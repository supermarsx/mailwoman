# Mailwoman — authentication & OAuth-consent strings (source locale: en).
# Covers the mailbox sign-in screen and the OAuth 2.1 authorization/consent screen.
# Admin sign-in lives in admin.ftl (separate session domain).

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-jmap-url = JMAP server URL
auth-jmap-url-placeholder = https://jmap.example.org
auth-username = Username
auth-password = Password
auth-sign-in = Sign in
auth-signing-in = Signing in…
auth-invalid-credentials = Invalid credentials
auth-unreachable = Could not reach the server
auth-mock-hint = Mock account: testuser@example.org / testpass

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
