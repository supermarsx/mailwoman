# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Смена пароля
passwd-heading = Смена пароля

# -- Forced change -----------------------------------------------------------
passwd-force-change = Администратор требует сменить пароль, прежде чем продолжить.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Текущий пароль
passwd-new-label = Новый пароль
passwd-confirm-label = Подтверждение нового пароля

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Новые пароли совпадают.
passwd-match-no = Новые пароли пока не совпадают.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = { $count ->
    [one] не менее { $count } символа
    [few] не менее { $count } символов
    [many] не менее { $count } символов
   *[other] не менее { $count } символа
}
passwd-rule-uppercase = заглавную букву
passwd-rule-lowercase = строчную букву
passwd-rule-digit = цифру
passwd-rule-symbol = специальный символ

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Эта учётная запись работает в режиме нулевого доступа. Перед применением изменения вам будет показана фраза восстановления — сохраните её, чтобы не потерять доступ к данным, если что-то пойдёт не так.

# -- Actions -----------------------------------------------------------------
passwd-continue = Продолжить
passwd-submit = Сменить пароль

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Сохраните фразу восстановления
passwd-recovery-prose = Запишите эту фразу и храните её в надёжном месте. Она показывается до смены пароля, чтобы вы могли восстановить данные, даже если новый пароль будет утерян. Она не хранится на сервере.
passwd-recovery-ack-label = Я сохранил(а) фразу восстановления
passwd-recovery-ack-text = Я сохранил(а) фразу восстановления в надёжном месте.

# -- Done phase --------------------------------------------------------------
passwd-done = Пароль изменён.
passwd-done-reencrypt = Сохранённые учётные данные серверов повторно зашифрованы новым паролем.
passwd-done-rewrap = Ключи нулевого доступа повторно защищены новым паролем.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = введите текущий пароль
passwd-error-mismatch = новый пароль и его подтверждение не совпадают
passwd-error-policy = новый пароль должен содержать: { $rules }
passwd-error-ack-first = сначала подтвердите, что сохранили фразу восстановления
passwd-error-prepare-recovery = не удалось подготовить фразу восстановления
passwd-error-change = не удалось сменить пароль
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = Сеанс завершён. Войдите снова.
passwd-error-not-configured = На этом сервере не настроена смена паролей. Обратитесь к администратору.
passwd-error-contact-admin = Серверу не удалось завершить изменение. Обратитесь к администратору.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Смените пароль, чтобы продолжить
passwd-forced-signed-in-as = Вы вошли как { $username }.
passwd-forced-explain = Администратор потребовал сменить пароль для этой учётной записи. Почта, календарь и контакты недоступны, пока смена пароля не будет выполнена.
passwd-forced-backend-note = Если смена отклоняется, хотя текущий пароль указан верно, возможно, этот сервер не может изменить ваш пароль. Обратитесь к администратору: он может настроить смену паролей или снять это требование.
passwd-forced-sign-out = Выйти
passwd-forced-still-required = Сервер принял изменение, но по-прежнему требует сменить пароль для этой учётной записи. Обратитесь к администратору.
passwd-forced-reload-failed = Пароль изменён, но не удалось загрузить учётную запись. Проверьте подключение и повторите попытку.
passwd-forced-retry = Повторить
