# Mailwoman — authentication & OAuth-consent strings (source locale: en).
# Covers the mailbox sign-in screen and the OAuth 2.1 authorization/consent screen.
# Admin sign-in lives in admin.ftl (separate session domain).

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = Email or username
auth-login-rate-limited = Too many sign-in attempts. Please wait before trying again.
auth-retry-countdown = Try again in { $seconds }s
auth-password = Password
auth-sign-in = Sign in
auth-signing-in = Signing in…
auth-invalid-credentials = Invalid credentials
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = If the email address and password are correct, the account may have been disabled by an administrator. A disabled account cannot sign in until an administrator enables it again.
auth-unreachable = Could not reach the server
auth-upstream-protocol = Sign-in stopped: the mail server sent a reply this server could not read. Try again later, or tell your administrator.
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

auth-remembered = Previous sign-ins
auth-remembered-help = Addresses used successfully on this device. Choosing one fills the address only.
auth-forget-address = Forget { $address } on this device
auth-login-timeout = Sign-in timed out. No automatic retry was made. Check the connection and try again when ready.

auth-verification-title = Sign-in verification
auth-verification-loading = Loading sign-in verification…
auth-verification-error = Verification could not be completed. Try the verification again before signing in.
auth-verification-required = Complete a new sign-in verification before trying again.

auth-identity-title = Share your identity
auth-identity-sign-in = Sign in again to continue this identity request.
auth-identity-sign-out = Sign out of Mailwoman
auth-identity-expired = This identity request expired or changed. Return to the requesting application and start again.
auth-identity-retry = Retry
auth-identity-disclosure = This application receives the identity information below. This permission does not grant access to your mail.
auth-identity-scope-openid = A stable identity identifier
auth-identity-scope-profile = Available profile information
auth-identity-scope-email = Your signed-in email address, without a claim that it was independently verified
auth-identity-scope-offline_access = Renew identity access while this Mailwoman session remains valid, up to 15 days from sign-in
auth-identity-deny = Cancel
auth-identity-approve = Allow identity access
auth-identity-grants = Applications using your identity
auth-identity-grants-help = Revoke identity access for an application. This does not remove separate mail API permissions.
auth-identity-no-grants = No applications currently have identity consent.
auth-identity-revoke = Revoke identity access
auth-maintenance = Webmail is temporarily paused for maintenance. Please try again later.
