# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: settings. Source: ../en/settings.ftl

settings-title = Налаштування
settings-appearance = Вигляд
settings-close = Закрити налаштування

settings-theme = Тема
settings-density = Щільність
settings-accent = Акцентний колір
settings-font = Шрифт інтерфейсу
settings-layout = Компонування

# Density options
settings-density-compact = Компактна
settings-density-cozy = Зручна
settings-density-relaxed = Вільна

# Interface-font options
settings-font-default = Типовий
settings-font-system = Системний
settings-font-serif = Із засічками
settings-font-mono = Моноширинний

# Layout options
settings-layout-default = Типове
settings-layout-ribbon = Стрічка

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Скасувати
settings-edit = Редагувати
settings-delete = Видалити
settings-save = Зберегти
settings-saved = Збережено

# Two-factor authentication (S1)
settings-2fa-title = Двофакторна автентифікація
settings-2fa-intro = Додайте другий фактор, щоб для входу був потрібен не лише пароль.
settings-2fa-policy-required = Ваша організація вимагає другий фактор для цього облікового запису.
settings-2fa-enabled = Увімкнено
settings-2fa-recovery-title = Коди відновлення
settings-2fa-recovery-once = Збережіть їх зараз. Їх показано один раз, і отримати їх знову неможливо. Кожен код можна використати лише один раз.
settings-2fa-recovery-ack = Я зберіг(-ла) ці коди
settings-2fa-totp-title = Застосунок-автентифікатор
settings-2fa-totp-enrol = Налаштувати застосунок-автентифікатор
settings-2fa-totp-scan = Додайте цей секрет до застосунку-автентифікатора, а потім введіть код, який він покаже.
settings-2fa-totp-uri-link = Відкрити в застосунку-автентифікаторі
settings-2fa-totp-code-label = Код автентифікатора
settings-2fa-totp-confirm = Підтвердити
settings-2fa-totp-disable = Вилучити автентифікатор
settings-2fa-passkey-title = Ключі доступу
settings-2fa-passkey-remove = Вилучити
settings-2fa-passkey-unsupported = Цей браузер не підтримує ключі доступу.
settings-2fa-passkey-label-placeholder = Назва ключа доступу (необов'язково)
settings-2fa-passkey-add = Додати ключ доступу
settings-2fa-recovery-remaining = { $count ->
        [one] Залишився { $count } код відновлення
        [few] Залишилося { $count } коди відновлення
        [many] Залишилося { $count } кодів відновлення
       *[other] Залишилося { $count } коду відновлення
    }
settings-2fa-recovery-regenerate = Згенерувати нові коди відновлення
settings-2fa-recovery-code-label = Код відновлення
settings-2fa-error-generic = Не вдалося. Спробуйте знову.
settings-2fa-error-code = Код не пройшов перевірку.
settings-2fa-error-passkey = Не вдалося зареєструвати ключ доступу.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Двофакторна автентифікація
settings-2fa-challenge-intro = Підтвердьте другий фактор, щоб завершити вхід.
settings-2fa-challenge-failed = Перевірку не пройдено. Спробуйте знову.
settings-2fa-challenge-use-passkey = Використати ключ доступу
settings-2fa-challenge-method = Виберіть спосіб
settings-2fa-challenge-totp-tab = Код автентифікатора
settings-2fa-challenge-recovery-tab = Код відновлення
settings-2fa-challenge-verify = Перевірити

# Active sessions (S11)
settings-sessions-title = Активні сеанси
settings-sessions-intro = Сеанси, у яких зараз виконано вхід у цей обліковий запис.
settings-sessions-last-seen = Остання активність: { $when }
settings-sessions-revoke = Вийти
settings-sessions-current = Цей сеанс
settings-sessions-revoke-others = { $count ->
        [one] Завершити { $count } інший сеанс
        [few] Завершити { $count } інші сеанси
        [many] Завершити { $count } інших сеансів
       *[other] Завершити { $count } іншого сеансу
    }
settings-sessions-error = Не вдалося змінити сеанс.

# Signatures (W12)
settings-sig-title = Підписи
settings-sig-intro = Текст, що додається до надісланих вами повідомлень. Один підпис можна зробити типовим.
settings-sig-default = Типовий
settings-sig-new = Новий підпис
settings-sig-name-label = Назва
settings-sig-body-label = Підпис
settings-sig-default-label = Використовувати як типовий
settings-sig-error-name = Дайте підпису назву.
settings-sig-error-generic = Не вдалося зберегти підпис.

# Identities
settings-ident-title = Профілі відправника
settings-ident-intro = Ім'я та адреса, від яких ви надсилаєте пошту. Кожен профіль відправника може мати власний підпис.
settings-ident-new = Новий профіль відправника
settings-ident-name-label = Відображуване ім'я
settings-ident-email-label = Адреса електронної пошти
settings-ident-replyto-label = Адреса для відповіді (необов'язково)
settings-ident-signature-label = Типовий підпис
settings-ident-signature-none = Немає
settings-ident-error-fields = Введіть ім'я та дійсну адресу електронної пошти.
settings-ident-error-generic = Не вдалося зберегти профіль відправника.

# Notification rules + quiet hours (W15)
settings-notif-title = Сповіщення
settings-notif-intro = Виберіть, про які нові повідомлення сповіщати і коли не турбувати.
settings-notif-enabled-label = Показувати сповіщення про нову пошту
settings-notif-quiet-title = Тихі години
settings-notif-quiet-enabled-label = Вимикати сповіщення у визначений час
settings-notif-quiet-start = З
settings-notif-quiet-end = До
settings-notif-rules-title = Правила
settings-notif-rule-match-placeholder = Відправник, тека або тема містить…
settings-notif-rule-action-label = Дія
settings-notif-rule-notify = Сповіщати
settings-notif-rule-mute = Без звуку
settings-notif-rule-add = Додати правило
settings-notif-error = Не вдалося зберегти налаштування сповіщень.

# Saved searches → search folders (W13)
settings-search-title = Збережені пошуки
settings-search-intro = Показувати збережений пошук як теку у списку тек.
settings-search-empty = У вас ще немає збережених пошуків.
settings-search-as-folder = Показувати { $name } як теку
settings-search-as-folder-label = Показувати як теку
settings-search-error = Не вдалося змінити збережений пошук.

# Keyboard shortcuts (W14)
settings-kbd-title = Комбінації клавіш
settings-kbd-intro = Виберіть набір комбінацій клавіш. Зміни застосовуються на цьому пристрої.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Вимкнено (без комбінацій)
settings-kbd-action-compose = Написати
settings-kbd-action-archive = Архівувати
settings-kbd-action-reply = Відповісти
settings-kbd-action-next = Наступне повідомлення
settings-kbd-action-previous = Попереднє повідомлення
settings-kbd-action-search = Пошук

# Offline cache (W16)
settings-offline-title = Офлайн-кеш
settings-offline-intro = Скільки пошти зберігати на цьому пристрої для роботи офлайн і як звільняти місце.
settings-offline-budget-label = Обсяг кешу (МБ)
settings-offline-retention-label = Зберігати (днів)
settings-offline-strategy-label = Коли обсяг вичерпано
settings-offline-lru = Видаляти найдовше не використовувані
settings-offline-oldest = Видаляти спершу найстаріші
settings-offline-manual = Лише коли я очищу вручну
settings-offline-purge = Очистити офлайн-кеш зараз

# Interface direction (W20)
settings-dir-title = Напрямок інтерфейсу
settings-dir-intro = Відповідно до мови або примусово заданий напрямок. Справа наліво дзеркально відображає компонування.
settings-dir-auto = Автоматично
settings-dir-ltr = Зліва направо
settings-dir-rtl = Справа наліво
settings-dir-preview-title = Попередній перегляд
settings-dir-preview-body = Цей текст дзеркально відображається, коли напрямок — справа наліво.
