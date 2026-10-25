# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = Adres e-mail
auth-password = Hasło
auth-sign-in = Zaloguj się
auth-signing-in = Logowanie…
auth-invalid-credentials = Nieprawidłowe dane logowania
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = Jeśli adres e-mail i hasło są poprawne, konto mogło zostać wyłączone przez administratora. Wyłączone konto nie może się zalogować, dopóki administrator ponownie go nie włączy.
auth-unreachable = Nie można połączyć się z serwerem
# The server has no mail server configured for the address's domain.
auth-address-not-configured = Ten adres nie jest skonfigurowany na tym serwerze. Skontaktuj się z administratorem.

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Logowanie jednokrotne (SSO)
auth-sso-divider = lub kontynuuj przez
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Zaloguj się przez { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = Logowanie jednokrotne nie zostało ukończone. Spróbuj ponownie lub zaloguj się hasłem.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Autoryzuj aplikację
auth-consent-title = Autoryzuj dostęp
auth-consent-loading = Wczytywanie żądania…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = chce uzyskać dostęp do Twojego konta.
auth-consent-approved = Klient zatwierdzony przez administratora
auth-consent-unapproved = Nierozpoznany klient — niezatwierdzony przez administratora
auth-consent-requesting = Żąda uprawnień
auth-consent-redirects-to = Przekierowuje do
auth-consent-for-resource = Dla zasobu
auth-consent-deny = Odmów
auth-consent-allow = Zezwól
auth-consent-error = nie udało się zapisać Twojej decyzji
