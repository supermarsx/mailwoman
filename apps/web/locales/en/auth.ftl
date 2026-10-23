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
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = If the username and password are correct, the account may have been disabled by an administrator. A disabled account cannot sign in until an administrator enables it again.
auth-unreachable = Could not reach the server

# -- Server lookup from the email address ------------------------------------
# The sign-in screen starts with an email address and a password, asks the
# server which mail server belongs to the address, shows the answer, and signs
# in only after the user confirms it.
auth-email = Email address
auth-discovering = Looking up the mail server…
# The button that confirms the server shown above it and signs in.
auth-discover-confirm = Sign in with this server
auth-manual-show = Enter server details manually
auth-manual-hide = Look up the server from an email address
# `tls` is what the lookup reported for the connection, not a setting.
auth-discover-found-imap = Found for { $domain }: IMAP server { $host }, port { $port }, { $tls ->
        [implicit] TLS
        [start-tls] STARTTLS
       *[none] no encryption advertised
    }.
auth-discover-found-jmap = Found for { $domain }: JMAP server { $url }
auth-discover-source = { $source ->
        [jmap] Source: the JMAP session resource published by the domain.
        [jmap-srv] Source: the domain's DNS SRV record for JMAP.
        [srv] Source: the domain's DNS SRV records.
        [thunderbird-autoconfig] Source: a Thunderbird autoconfiguration file.
        [autodiscover] Source: Microsoft Autodiscover.
        [provider-db] Source: the provider list built into this server.
       *[other] Source: { $source }.
    }
auth-discover-oauth-only = This provider requires OAuth sign-in, which this build does not offer yet.
auth-discover-not-found = No server settings were found for { $domain }. Enter the server details below.
auth-discover-invalid-email = The server did not accept { $email } as an email address. Enter the server details below.
auth-discover-rate-limited = Too many server lookups were made from this network. Enter the server details below, or wait a minute and look up the address again.
auth-discover-failed = The server lookup did not complete. Enter the server details below.
# Shown with the refusal when the sign-in that was refused used a looked-up
# server. The server answers every refusal the same way, so this names what was
# sent rather than what was wrong.
auth-discover-refused-note = That sign-in used { $url }, found for { $domain }, with the email address as the username. If the account uses a different server or username, change them above.

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
