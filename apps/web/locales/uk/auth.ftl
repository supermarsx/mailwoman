# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = Адреса електронної пошти
auth-password = Пароль
auth-sign-in = Увійти
auth-signing-in = Вхід…
auth-invalid-credentials = Неправильні облікові дані
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = Якщо адреса електронної пошти і пароль правильні, можливо, обліковий запис вимкнув адміністратор. Вимкнений обліковий запис не може ввійти, доки адміністратор знову його не ввімкне.
auth-unreachable = Не вдалося зв’язатися із сервером
# The server has no mail server configured for the address's domain.
auth-address-not-configured = Ця адреса не налаштована на цьому сервері. Зверніться до адміністратора.

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Єдиний вхід (SSO)
auth-sso-divider = або продовжити через
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Увійти через { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = Єдиний вхід не завершився. Спробуйте ще раз або увійдіть за паролем.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Авторизація застосунку
auth-consent-title = Надання доступу
auth-consent-loading = Завантаження запиту…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = запитує доступ до вашого облікового запису.
auth-consent-approved = Клієнт, схвалений адміністратором
auth-consent-unapproved = Нерозпізнаний клієнт — не схвалений адміністратором
auth-consent-requesting = Запитувані права
auth-consent-redirects-to = Перенаправляє на
auth-consent-for-resource = Для ресурсу
auth-consent-deny = Відхилити
auth-consent-allow = Дозволити
auth-consent-error = не вдалося зберегти ваше рішення
