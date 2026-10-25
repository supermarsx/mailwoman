# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = E-mailadres
auth-password = Wachtwoord
auth-sign-in = Inloggen
auth-signing-in = Inloggen…
auth-invalid-credentials = Ongeldige inloggegevens
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = Als het e-mailadres en het wachtwoord kloppen, is het account mogelijk door een beheerder uitgeschakeld. Met een uitgeschakeld account kun je pas weer inloggen als een beheerder het opnieuw inschakelt.
auth-unreachable = Kan de server niet bereiken
# The server has no mail server configured for the address's domain.
auth-address-not-configured = Dit adres is niet ingesteld op deze server. Neem contact op met je beheerder.

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Single sign-on
auth-sso-divider = of ga verder met
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Inloggen met { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = Single sign-on is niet voltooid. Probeer het opnieuw of log in met je wachtwoord.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Applicatie autoriseren
auth-consent-title = Toegang autoriseren
auth-consent-loading = Verzoek laden…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = wil toegang tot je account.
auth-consent-approved = Door beheerder goedgekeurde client
auth-consent-unapproved = Onbekende client — niet goedgekeurd door een beheerder
auth-consent-requesting = Gevraagd wordt
auth-consent-redirects-to = Stuurt door naar
auth-consent-for-resource = Voor resource
auth-consent-deny = Weigeren
auth-consent-allow = Toestaan
auth-consent-error = je beslissing kon niet worden vastgelegd
