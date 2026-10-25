# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
# The sign-in screen asks for the email address and the password only. The
# server decides which mail server the address signs in to; the user never
# enters or sees one.
auth-app-name = Mailwoman
auth-email = Адрес электронной почты
auth-password = Пароль
auth-sign-in = Войти
auth-signing-in = Выполняется вход…
auth-invalid-credentials = Неверные учётные данные
# Shown under the refusal. The server answers a disabled account with the same
# response as a wrong password, so this states a possibility, not a diagnosis.
auth-refused-note = Если адрес электронной почты и пароль указаны верно, учётная запись могла быть отключена администратором. Отключённая учётная запись не может войти, пока администратор снова её не включит.
auth-unreachable = Не удалось связаться с сервером
# The server has no mail server configured for the address's domain.
auth-address-not-configured = Этот адрес не настроен на данном сервере. Обратитесь к администратору.

# -- Single sign-on (t9) -----------------------------------------------------
# The "or continue with" divider + one button per configured IdP. Only shown
# when the deployment has enabled SSO backends; otherwise the login is unchanged.
auth-sso-heading = Единый вход
auth-sso-divider = или продолжить через
# `name` is the IdP's admin-set (trusted-operator) display name.
auth-sso-button = Войти через { $name }
# Shown when an SSO round-trip fails and the IdP returns to /?sso_error — a
# uniform message that never reveals which check failed (no-leak, like the 401).
auth-sso-error = Единый вход не был завершён. Повторите попытку или войдите с паролем.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Авторизация приложения
auth-consent-title = Разрешение доступа
auth-consent-loading = Загрузка запроса…
# Rendered after the (isolated) client-name span: "<client> wants to access…".
auth-consent-intro = запрашивает доступ к вашей учётной записи.
auth-consent-approved = Клиент одобрен администратором
auth-consent-unapproved = Неизвестный клиент — не одобрен администратором
auth-consent-requesting = Запрашиваемые права
auth-consent-redirects-to = Перенаправляет на
auth-consent-for-resource = Для ресурса
auth-consent-deny = Отклонить
auth-consent-allow = Разрешить
auth-consent-error = не удалось сохранить ваше решение
