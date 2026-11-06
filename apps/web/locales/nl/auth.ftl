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

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Te veel inlogpogingen. Wacht even voordat u het opnieuw probeert.
auth-retry-countdown = Probeer het opnieuw over { $seconds } s
auth-upstream-protocol = Inloggen gestopt: de mailserver stuurde een antwoord dat deze server niet kon lezen. Probeer het later opnieuw of neem contact op met uw beheerder.
auth-account-disabled = Dit account is door een beheerder uitgeschakeld. Inloggen is pas weer mogelijk als een beheerder het opnieuw inschakelt.
auth-signin-not-allowed = Dit adres mag niet inloggen op deze server. Neem contact op met uw beheerder.
auth-signin-not-provisioned = Er bestaat nog geen account voor dit adres en nieuwe accounts worden niet automatisch aangemaakt. Vraag uw beheerder u toe te voegen.
auth-reference = Referentie: { $id }
auth-domain-hint = Voor { $domain } kunt u inloggen met alleen uw gebruikersnaam.
auth-passkey-sign-in = Inloggen met een passkey
auth-passkey-failed = Inloggen met de passkey is niet gelukt. Probeer het opnieuw of gebruik uw wachtwoord.
auth-remembered = Eerdere aanmeldingen
auth-remembered-help = Adressen die op dit apparaat succesvol zijn gebruikt. Als u er een kiest, wordt alleen het adres ingevuld.
auth-forget-address = { $address } op dit apparaat vergeten
auth-login-timeout = Inloggen is verlopen. Er is niet automatisch opnieuw geprobeerd. Controleer de verbinding en probeer het opnieuw wanneer u klaar bent.
auth-verification-title = Verificatie van inloggen
auth-verification-loading = Verificatie van inloggen laden…
auth-verification-error = De verificatie kon niet worden voltooid. Probeer de verificatie opnieuw voordat u inlogt.
auth-verification-required = Voltooi een nieuwe verificatie van inloggen voordat u het opnieuw probeert.
auth-identity-title = Uw identiteit delen
auth-identity-sign-in = Log opnieuw in om dit identiteitsverzoek voort te zetten.
auth-identity-sign-out = Uitloggen bij Mailwoman
auth-identity-expired = Dit identiteitsverzoek is verlopen of gewijzigd. Ga terug naar de aanvragende applicatie en begin opnieuw.
auth-identity-retry = Opnieuw proberen
auth-identity-disclosure = Deze applicatie ontvangt de onderstaande identiteitsgegevens. Deze toestemming geeft geen toegang tot uw e-mail.
auth-identity-scope-openid = Een vaste identiteitsidentificatie
auth-identity-scope-profile = Beschikbare profielgegevens
auth-identity-scope-email = Uw e-mailadres waarmee u bent ingelogd, zonder bewering dat het onafhankelijk is geverifieerd
auth-identity-scope-offline_access = Identiteitstoegang verlengen zolang deze Mailwoman-sessie geldig blijft, tot 15 dagen na het inloggen
auth-identity-deny = Annuleren
auth-identity-approve = Identiteitstoegang toestaan
auth-identity-grants = Applicaties die uw identiteit gebruiken
auth-identity-grants-help = Trek de identiteitstoegang van een applicatie in. Afzonderlijke e-mail-API-rechten blijven bestaan.
auth-identity-no-grants = Er heeft momenteel geen enkele applicatie toestemming voor identiteit.
auth-identity-revoke = Identiteitstoegang intrekken
auth-maintenance = Webmail is tijdelijk gepauzeerd voor onderhoud. Probeer het later opnieuw.
