# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: settings. Source: ../en/settings.ftl

settings-title = Настройки
settings-appearance = Внешний вид
settings-close = Закрыть настройки

settings-theme = Тема
settings-density = Плотность
settings-accent = Акцентный цвет
settings-font = Шрифт интерфейса
settings-layout = Макет

# Density options
settings-density-compact = Компактная
settings-density-cozy = Средняя
settings-density-relaxed = Свободная

# Interface-font options
settings-font-default = По умолчанию
settings-font-system = Системный
settings-font-serif = С засечками
settings-font-mono = Моноширинный

# Layout options
settings-layout-default = По умолчанию
settings-layout-ribbon = Лента

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Отмена
settings-edit = Изменить
settings-delete = Удалить
settings-save = Сохранить
settings-saved = Сохранено

# Two-factor authentication (S1)
settings-2fa-title = Двухфакторная аутентификация
settings-2fa-intro = Добавьте второй фактор, чтобы для входа требовался не только пароль.
settings-2fa-policy-required = Ваша организация требует второй фактор для этой учётной записи.
settings-2fa-enabled = Включено
settings-2fa-recovery-title = Коды восстановления
settings-2fa-recovery-once = Сохраните их сейчас. Они показываются один раз, и получить их повторно нельзя. Каждый код действует один раз.
settings-2fa-recovery-ack = Я сохранил(а) эти коды
settings-2fa-totp-title = Приложение-аутентификатор
settings-2fa-totp-enrol = Настроить приложение-аутентификатор
settings-2fa-totp-scan = Добавьте этот секрет в приложение-аутентификатор, затем введите показанный им код.
settings-2fa-totp-uri-link = Открыть в приложении-аутентификаторе
settings-2fa-totp-code-label = Код аутентификатора
settings-2fa-totp-confirm = Подтвердить
settings-2fa-totp-disable = Удалить аутентификатор
settings-2fa-passkey-title = Ключи доступа
settings-2fa-passkey-remove = Удалить
settings-2fa-passkey-unsupported = Этот браузер не поддерживает ключи доступа.
settings-2fa-passkey-label-placeholder = Название ключа доступа (необязательно)
settings-2fa-passkey-add = Добавить ключ доступа
settings-2fa-recovery-remaining = { $count ->
        [one] Остался { $count } код восстановления
        [few] Осталось { $count } кода восстановления
        [many] Осталось { $count } кодов восстановления
       *[other] Осталось { $count } кода восстановления
    }
settings-2fa-recovery-regenerate = Создать новые коды восстановления
settings-2fa-recovery-code-label = Код восстановления
settings-2fa-error-generic = Не получилось. Повторите попытку.
settings-2fa-error-code = Код не прошёл проверку.
settings-2fa-error-passkey = Не удалось зарегистрировать ключ доступа.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Двухфакторная аутентификация
settings-2fa-challenge-intro = Подтвердите второй фактор, чтобы завершить вход.
settings-2fa-challenge-failed = Проверка не пройдена. Повторите попытку.
settings-2fa-challenge-use-passkey = Использовать ключ доступа
settings-2fa-challenge-method = Выберите способ
settings-2fa-challenge-totp-tab = Код аутентификатора
settings-2fa-challenge-recovery-tab = Код восстановления
settings-2fa-challenge-verify = Проверить

# Active sessions (S11)
settings-sessions-title = Активные сеансы
settings-sessions-intro = Сеансы, в которых сейчас выполнен вход в эту учётную запись.
settings-sessions-last-seen = Последняя активность: { $when }
settings-sessions-revoke = Завершить сеанс
settings-sessions-current = Этот сеанс
settings-sessions-revoke-others = { $count ->
        [one] Завершить { $count } другой сеанс
        [few] Завершить { $count } других сеанса
        [many] Завершить { $count } других сеансов
       *[other] Завершить { $count } другого сеанса
    }
settings-sessions-error = Не удалось изменить сеанс.

# Signatures (W12)
settings-sig-title = Подписи
settings-sig-intro = Текст, добавляемый к отправляемым сообщениям. Одна подпись может быть подписью по умолчанию.
settings-sig-default = По умолчанию
settings-sig-new = Новая подпись
settings-sig-name-label = Название
settings-sig-body-label = Подпись
settings-sig-default-label = Использовать по умолчанию
settings-sig-error-name = Укажите название подписи.
settings-sig-error-generic = Не удалось сохранить подпись.

# Identities
settings-ident-title = Профили отправителя
settings-ident-intro = Имя и адрес, от которых вы отправляете почту. У каждого профиля может быть своя подпись.
settings-ident-new = Новый профиль отправителя
settings-ident-name-label = Отображаемое имя
settings-ident-email-label = Адрес эл. почты
settings-ident-replyto-label = Адрес для ответа (необязательно)
settings-ident-signature-label = Подпись по умолчанию
settings-ident-signature-none = Нет
settings-ident-error-fields = Введите имя и корректный адрес эл. почты.
settings-ident-error-generic = Не удалось сохранить профиль отправителя.

# Notification rules + quiet hours (W15)
settings-notif-title = Уведомления
settings-notif-intro = Выберите, о каких новых сообщениях вас уведомлять и когда не беспокоить.
settings-notif-enabled-label = Показывать уведомления о новой почте
settings-notif-quiet-title = Тихие часы
settings-notif-quiet-enabled-label = Отключать уведомления в заданное время
settings-notif-quiet-start = С
settings-notif-quiet-end = До
settings-notif-rules-title = Правила
settings-notif-rule-match-placeholder = Отправитель, папка или тема содержит…
settings-notif-rule-action-label = Действие
settings-notif-rule-notify = Уведомлять
settings-notif-rule-mute = Не уведомлять
settings-notif-rule-add = Добавить правило
settings-notif-error = Не удалось сохранить настройки уведомлений.

# Saved searches → search folders (W13)
settings-search-title = Сохранённые поиски
settings-search-intro = Показывать сохранённый поиск как папку в списке папок.
settings-search-empty = У вас пока нет сохранённых поисков.
settings-search-as-folder = Показывать { $name } как папку
settings-search-as-folder-label = Показывать как папку
settings-search-error = Не удалось изменить сохранённый поиск.

# Keyboard shortcuts (W14)
settings-kbd-title = Сочетания клавиш
settings-kbd-intro = Выберите набор сочетаний клавиш. Изменения применяются на этом устройстве.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Выкл. (без сочетаний клавиш)
settings-kbd-action-compose = Написать
settings-kbd-action-archive = Архивировать
settings-kbd-action-reply = Ответить
settings-kbd-action-next = Следующее сообщение
settings-kbd-action-previous = Предыдущее сообщение
settings-kbd-action-search = Поиск

# Offline cache (W16)
settings-offline-title = Офлайн-кеш
settings-offline-intro = Сколько почты хранить на этом устройстве для работы офлайн и как освобождать место.
settings-offline-budget-label = Размер кеша (МБ)
settings-offline-retention-label = Хранить (дней)
settings-offline-strategy-label = При достижении лимита
settings-offline-lru = Удалять давно не использованное
settings-offline-oldest = Удалять самое старое
settings-offline-manual = Только при ручной очистке
settings-offline-purge = Очистить офлайн-кеш сейчас

# Interface direction (W20)
settings-dir-title = Направление интерфейса
settings-dir-intro = Следовать языку или задать направление принудительно. При направлении справа налево макет зеркально отражается.
settings-dir-auto = Автоматически
settings-dir-ltr = Слева направо
settings-dir-rtl = Справа налево
settings-dir-preview-title = Предпросмотр
settings-dir-preview-body = Этот текст отражается зеркально, когда выбрано направление справа налево.
