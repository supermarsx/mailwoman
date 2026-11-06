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

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Слишком много попыток входа. Подождите, прежде чем повторять.
auth-retry-countdown = Повторите через { $seconds } с
auth-upstream-protocol = Вход прерван: почтовый сервер прислал ответ, который этот сервер не смог прочитать. Повторите попытку позже или сообщите администратору.
auth-account-disabled = Эта учётная запись отключена администратором. Вход невозможен, пока администратор не включит её снова.
auth-signin-not-allowed = Этому адресу нельзя входить на этом сервере. Обратитесь к администратору.
auth-signin-not-provisioned = Для этого адреса ещё нет учётной записи, а новые учётные записи автоматически не создаются. Попросите администратора добавить Вас.
auth-reference = Номер для обращения: { $id }
auth-domain-hint = Для { $domain } можно войти, указав только имя пользователя.
auth-passkey-sign-in = Войти с ключом доступа
auth-passkey-failed = Не удалось войти с ключом доступа. Повторите попытку или используйте пароль.
auth-remembered = Прежние входы
auth-remembered-help = Адреса, с которых на этом устройстве успешно входили. При выборе подставляется только адрес.
auth-forget-address = Забыть { $address } на этом устройстве
auth-login-timeout = Время ожидания входа истекло. Автоматического повтора не было. Проверьте подключение и повторите попытку, когда будете готовы.
auth-verification-title = Проверка при входе
auth-verification-loading = Загрузка проверки при входе…
auth-verification-error = Не удалось завершить проверку. Повторите проверку, прежде чем входить.
auth-verification-required = Пройдите новую проверку при входе, прежде чем повторять попытку.
auth-identity-title = Поделиться сведениями об учётной записи
auth-identity-sign-in = Войдите снова, чтобы продолжить этот запрос удостоверения.
auth-identity-sign-out = Выйти из Mailwoman
auth-identity-expired = Этот запрос удостоверения истёк или изменился. Вернитесь в запрашивающее приложение и начните заново.
auth-identity-retry = Повторить
auth-identity-disclosure = Это приложение получит приведённые ниже сведения об удостоверении. Это разрешение не даёт доступа к Вашей почте.
auth-identity-scope-openid = Постоянный идентификатор учётной записи
auth-identity-scope-profile = Доступные сведения профиля
auth-identity-scope-email = Адрес электронной почты, с которым Вы вошли, без утверждения, что он независимо подтверждён
auth-identity-scope-offline_access = Продление доступа к удостоверению, пока действует эта сессия Mailwoman, но не более 15 дней с момента входа
auth-identity-deny = Отмена
auth-identity-approve = Разрешить доступ к удостоверению
auth-identity-grants = Приложения, использующие Ваше удостоверение
auth-identity-grants-help = Отзовите доступ к удостоверению для приложения. Отдельные разрешения для почтового API при этом не удаляются.
auth-identity-no-grants = Сейчас ни у одного приложения нет согласия на доступ к удостоверению.
auth-identity-revoke = Отозвать доступ к удостоверению
auth-maintenance = Веб-почта временно приостановлена на техническое обслуживание. Повторите попытку позже.
