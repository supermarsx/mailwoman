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

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Забагато спроб входу. Зачекайте, перш ніж повторювати.
auth-retry-countdown = Повторіть за { $seconds } с
auth-upstream-protocol = Вхід перервано: поштовий сервер надіслав відповідь, яку цей сервер не зміг прочитати. Повторіть спробу пізніше або повідомте адміністратора.
auth-account-disabled = Цей обліковий запис вимкнено адміністратором. Вхід неможливий, доки адміністратор знову його не ввімкне.
auth-signin-not-allowed = Цій адресі не можна входити на цьому сервері. Зверніться до адміністратора.
auth-signin-not-provisioned = Для цієї адреси ще немає облікового запису, а нові облікові записи автоматично не створюються. Попросіть адміністратора додати вас.
auth-reference = Номер для звернення: { $id }
auth-domain-hint = Для { $domain } можна ввійти, указавши лише ім’я користувача.
auth-passkey-sign-in = Увійти за допомогою ключа доступу
auth-passkey-failed = Не вдалося ввійти за допомогою ключа доступу. Повторіть спробу або скористайтеся паролем.
auth-remembered = Попередні входи
auth-remembered-help = Адреси, з яких на цьому пристрої успішно входили. Після вибору підставляється лише адреса.
auth-forget-address = Забути { $address } на цьому пристрої
auth-login-timeout = Час очікування входу минув. Автоматичного повтору не було. Перевірте підключення й повторіть спробу, коли будете готові.
auth-verification-title = Перевірка під час входу
auth-verification-loading = Завантаження перевірки під час входу…
auth-verification-error = Не вдалося завершити перевірку. Повторіть перевірку, перш ніж входити.
auth-verification-required = Пройдіть нову перевірку під час входу, перш ніж повторювати спробу.
auth-identity-title = Поділитися відомостями про обліковий запис
auth-identity-sign-in = Увійдіть знову, щоб продовжити цей запит посвідчення.
auth-identity-sign-out = Вийти з Mailwoman
auth-identity-expired = Цей запит посвідчення закінчився або змінився. Поверніться до застосунку, що його надіслав, і почніть заново.
auth-identity-retry = Повторити
auth-identity-disclosure = Цей застосунок отримає наведені нижче відомості про посвідчення. Цей дозвіл не надає доступу до вашої пошти.
auth-identity-scope-openid = Постійний ідентифікатор облікового запису
auth-identity-scope-profile = Доступні відомості профілю
auth-identity-scope-email = Адреса електронної пошти, з якою ви ввійшли, без твердження, що її незалежно підтверджено
auth-identity-scope-offline_access = Продовження доступу до посвідчення, поки діє цей сеанс Mailwoman, але не довше ніж 15 днів із моменту входу
auth-identity-deny = Скасувати
auth-identity-approve = Дозволити доступ до посвідчення
auth-identity-grants = Застосунки, що використовують ваше посвідчення
auth-identity-grants-help = Відкличте доступ до посвідчення для застосунку. Окремі дозволи для API пошти при цьому не видаляються.
auth-identity-no-grants = Наразі жоден застосунок не має згоди на доступ до посвідчення.
auth-identity-revoke = Відкликати доступ до посвідчення
auth-maintenance = Вебпошту тимчасово призупинено для технічного обслуговування. Повторіть спробу пізніше.
