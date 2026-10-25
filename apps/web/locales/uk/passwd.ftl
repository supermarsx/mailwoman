# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Зміна пароля
passwd-heading = Зміна пароля

# -- Forced change -----------------------------------------------------------
passwd-force-change = Адміністратор вимагає змінити пароль, перш ніж продовжити.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Поточний пароль
passwd-new-label = Новий пароль
passwd-confirm-label = Підтвердьте новий пароль

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Нові паролі збігаються.
passwd-match-no = Нові паролі поки не збігаються.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = { $count ->
    [one] щонайменше { $count } символ
    [few] щонайменше { $count } символи
    [many] щонайменше { $count } символів
   *[other] щонайменше { $count } символу
}
passwd-rule-uppercase = велику літеру
passwd-rule-lowercase = малу літеру
passwd-rule-digit = цифру
passwd-rule-symbol = спеціальний символ

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Цей обліковий запис використовує нульовий доступ (zero-access). Перед застосуванням зміни вам буде показано фразу відновлення — збережіть її, щоб мати доступ до своїх даних, якщо щось піде не так.

# -- Actions -----------------------------------------------------------------
passwd-continue = Продовжити
passwd-submit = Змінити пароль

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Збережіть фразу відновлення
passwd-recovery-prose = Запишіть цю фразу та зберігайте її в надійному місці. Її показано до зміни пароля, щоб ви могли відновити свої дані, навіть якщо новий пароль буде втрачено. Вона не зберігається на сервері.
passwd-recovery-ack-label = Я зберіг(-ла) фразу відновлення
passwd-recovery-ack-text = Я зберіг(-ла) фразу відновлення в надійному місці.

# -- Done phase --------------------------------------------------------------
passwd-done = Пароль змінено.
passwd-done-reencrypt = Збережені облікові дані серверів повторно зашифровано новим паролем.
passwd-done-rewrap = Ключі нульового доступу повторно захищено новим паролем.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = введіть поточний пароль
passwd-error-mismatch = новий пароль і його підтвердження не збігаються
passwd-error-policy = новий пароль має містити: { $rules }
passwd-error-ack-first = спершу підтвердьте, що зберегли фразу відновлення
passwd-error-prepare-recovery = не вдалося підготувати фразу відновлення
passwd-error-change = не вдалося змінити пароль
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = Ваш сеанс завершився. Увійдіть знову.
passwd-error-not-configured = Цей сервер не налаштовано для зміни паролів. Зверніться до адміністратора.
passwd-error-contact-admin = Сервер не зміг завершити зміну. Зверніться до адміністратора.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Змініть пароль, щоб продовжити
passwd-forced-signed-in-as = Ви увійшли як { $username }.
passwd-forced-explain = Адміністратор вимагає змінити пароль для цього облікового запису. Пошта, календар і контакти недоступні, доки зміну не буде виконано.
passwd-forced-backend-note = Якщо зміну відхилено, хоча поточний пароль правильний, цей сервер, можливо, не може змінити ваш пароль. Зверніться до адміністратора: він може налаштувати зміну паролів або скасувати цю вимогу.
passwd-forced-sign-out = Вийти
passwd-forced-still-required = Сервер прийняв зміну, але все ще вимагає змінити пароль для цього облікового запису. Зверніться до адміністратора.
passwd-forced-reload-failed = Пароль змінено, але не вдалося завантажити обліковий запис. Перевірте з'єднання та спробуйте знову.
passwd-forced-retry = Спробувати знову
