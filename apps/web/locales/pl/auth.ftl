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

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Zbyt wiele prób logowania. Poczekaj przed ponowną próbą.
auth-retry-countdown = Spróbuj ponownie za { $seconds } s
auth-upstream-protocol = Logowanie przerwane: serwer poczty wysłał odpowiedź, której ten serwer nie potrafił odczytać. Spróbuj ponownie później lub powiadom administratora.
auth-account-disabled = To konto zostało wyłączone przez administratora. Logowanie będzie niemożliwe, dopóki administrator go ponownie nie włączy.
auth-signin-not-allowed = Ten adres nie może się logować na tym serwerze. Skontaktuj się z administratorem.
auth-signin-not-provisioned = Dla tego adresu nie istnieje jeszcze konto, a nowe konta nie są tworzone automatycznie. Poproś administratora o dodanie Cię.
auth-reference = Odniesienie: { $id }
auth-domain-hint = Dla domeny { $domain } możesz się zalogować, podając tylko nazwę użytkownika.
auth-passkey-sign-in = Zaloguj się kluczem dostępu
auth-passkey-failed = Nie udało się zalogować kluczem dostępu. Spróbuj ponownie lub użyj hasła.
auth-remembered = Poprzednie logowania
auth-remembered-help = Adresy użyte pomyślnie na tym urządzeniu. Wybranie jednego wypełnia tylko adres.
auth-forget-address = Zapomnij adres { $address } na tym urządzeniu
auth-login-timeout = Przekroczono czas logowania. Nie podjęto automatycznej ponownej próby. Sprawdź połączenie i spróbuj ponownie, gdy będziesz gotów.
auth-verification-title = Weryfikacja logowania
auth-verification-loading = Ładowanie weryfikacji logowania…
auth-verification-error = Nie udało się ukończyć weryfikacji. Ponów weryfikację przed zalogowaniem.
auth-verification-required = Ukończ nową weryfikację logowania, zanim spróbujesz ponownie.
auth-identity-title = Udostępnij swoją tożsamość
auth-identity-sign-in = Zaloguj się ponownie, aby kontynuować to żądanie tożsamości.
auth-identity-sign-out = Wyloguj z Mailwoman
auth-identity-expired = To żądanie tożsamości wygasło lub uległo zmianie. Wróć do aplikacji, która o nie prosi, i zacznij od nowa.
auth-identity-retry = Spróbuj ponownie
auth-identity-disclosure = Ta aplikacja otrzyma poniższe informacje o tożsamości. To uprawnienie nie daje dostępu do Twojej poczty.
auth-identity-scope-openid = Stały identyfikator tożsamości
auth-identity-scope-profile = Dostępne informacje profilowe
auth-identity-scope-email = Twój adres e-mail używany do logowania, bez twierdzenia, że został niezależnie zweryfikowany
auth-identity-scope-offline_access = Odnawianie dostępu do tożsamości, dopóki ta sesja Mailwoman jest ważna, maksymalnie przez 15 dni od zalogowania
auth-identity-deny = Anuluj
auth-identity-approve = Zezwól na dostęp do tożsamości
auth-identity-grants = Aplikacje używające Twojej tożsamości
auth-identity-grants-help = Cofnij dostęp do tożsamości dla aplikacji. Nie usuwa to osobnych uprawnień do API poczty.
auth-identity-no-grants = Żadna aplikacja nie ma obecnie zgody na dostęp do tożsamości.
auth-identity-revoke = Cofnij dostęp do tożsamości
auth-maintenance = Poczta internetowa jest chwilowo wstrzymana z powodu prac konserwacyjnych. Spróbuj ponownie później.
