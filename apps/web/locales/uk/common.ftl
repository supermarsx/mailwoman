# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Скасувати
common-close = Закрити
common-save = Зберегти
common-delete = Видалити
common-remove = Прибрати
common-edit = Редагувати
common-add = Додати
common-back = Назад
common-next = Далі
common-done = Готово
common-apply = Застосувати
common-confirm = Підтвердити
common-retry = Повторити
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = Якщо це повторюється, зверніться до адміністратора.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Цей застосунок не налаштовано для сервера. Зверніться до адміністратора.
common-copy = Копіювати
common-search = Пошук
common-more = Більше
common-yes = Так
common-no = Ні

# -- Generic states ----------------------------------------------------------
common-loading = Завантаження…
common-saving = Збереження…
common-empty = Тут поки нічого немає
common-error = Щось пішло не так
common-error-network = Не вдається зв’язатися із сервером. Перевірте з’єднання і спробуйте ще раз.
common-offline = Ви офлайн

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = З’єднання…
common-conn-degraded = Оновлення в реальному часі призупинено — повторне з’єднання
common-conn-offline = Ви офлайн — зміни синхронізуються, коли з’єднання відновиться
common-conn-auth-expired = Ваш сеанс завершився — увійдіть знову
common-conn-reconnect = З’єднатися знову
common-conn-sign-in-again = Увійти знову
common-conn-reconnected = З’єднання відновлено

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Нове повідомлення
common-notify-no-subject = (без теми)

# -- Generic labels ----------------------------------------------------------
common-required = Обов’язково
common-optional = Необов’язково

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Відкрити теки та застосунки
common-nav-close = Закрити теки та застосунки

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Вкладення
common-attach-search = Пошук вкладень
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Фільтр за типом
common-attach-loading = Завантаження вкладень…
common-attach-empty = Немає відповідних вкладень.
common-attach-cat-all = усі
common-attach-cat-image = зображення
common-attach-cat-pdf = pdf
common-attach-cat-text = текст
common-attach-cat-audio = аудіо
common-attach-cat-video = відео
common-attach-cat-other = інше

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = Показати пароль
common-password-hide = Приховати пароль

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = Пошта
common-module-calendar = Календар
common-module-tasks = Завдання
common-module-notes = Нотатки
common-module-contacts = Контакти
common-module-keys = Ключі

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = Завантаження переглядача…
common-viewer-loading-audio = Завантаження аудіо…
common-viewer-loading-image = Завантаження зображення…
common-viewer-loading-video = Завантаження відео…
common-viewer-rendering-pdf = Відтворення PDF…
common-viewer-pdf-failed = Не вдалося відтворити цей PDF.
common-viewer-no-preview = Для цього типу файлів немає вбудованого попереднього перегляду.
common-viewer-unknown-type = невідомий тип
common-viewer-download = Завантажити
common-viewer-attachments = Вкладення

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Повторна спроба…
common-waiting-online = Очікування підключення…
common-failure-offline = Немає підключення. Підключіться й повторіть спробу.
common-failure-network = Не вдалося зв’язатися із сервером.
common-failure-server = Сервер не відповів вчасно.
common-failure-rate = Забагато запитів за короткий час. Зачекайте трохи й повторіть.
common-failure-conflict = Це одночасно змінили в іншому місці. Перезавантажте й повторіть.
common-failure-account = Обліковий запис, в який ви увійшли, змінився. Перезавантажте сторінку.
common-failure-linked = Це недоступно, поки відкрито пов’язану поштову скриньку.
common-failure-refused = Сервер відхилив зміну.
common-new-version = Доступна нова версія Mailwoman. Якщо закрити це повідомлення, воно з’явиться знову за годину.
common-new-version-reload = Перезавантажити зараз
common-toast-close = Закрити сповіщення
common-toast-seconds = { $seconds } с
common-attach-refresh = Оновити вкладення
common-attach-open-message = Відкрити повідомлення з цим вкладенням
common-attach-page-filter-note = Фільтри застосовуються до цієї сторінки. Натисніть «Далі», щоб переглянути старіші повідомлення з вкладеннями.
common-attach-pages = Сторінки вкладень
common-attach-previous = Назад
common-attach-next = Далі
common-attach-page = Сторінка { $page }
common-attach-column-name = Файл
common-attach-column-type = Тип
common-attach-column-size = Розмір
common-attach-column-sender = Відправник
common-attach-column-subject = Тема
common-attach-column-date = Отримано
common-pdf-preview = Попередній перегляд PDF
common-pdf-link-title = Відкрити посилання з цього PDF?
common-pdf-link-help = Адресу буде відкрито в новій вкладці поза межами Mailwoman.
common-pdf-link-open = Відкрити посилання
common-open-event = Відкрити подію
common-message-preview-too-large = Попередній перегляд вкладень повідомлень обмежено 4 МіБ. Початкове вкладення все одно можна завантажити.
common-message-preview-failed = Не вдалося показати попередній перегляд цього вкладення повідомлення. Файл може не підтримуватися або бути пошкодженим, або служба перегляду недоступна. Початковий файл усе одно можна завантажити.
common-message-preview-attachments = Файли всередині цього повідомлення
common-message-preview-download = Завантажити файл
common-message-preview-size-bytes = { $count ->
    [one] { $count } байт
    [few] { $count } байти
    [many] { $count } байтів
   *[other] { $count } байта
}
common-attach-cat-message = Повідомлення / вкладення Outlook
common-error-retry-help = Цей розділ тимчасово недоступний. Повторіть спробу, щоб завантажити його знову.
common-error-network-help = Перевірте підключення й повторіть спробу. Можливо, сервер тимчасово недоступний.
common-error-page = Не вдалося завантажити цю сторінку
common-error-page-help = Не вдалося завантажити ресурс сторінки. Повторіть спробу. Якщо проблема не зникне, перезавантажте застосунок, щоб отримати останню версію.
common-reload-app = Перезавантажити застосунок
common-new-version-reloading = Доступна нова версія, виконується перезавантаження…
common-attach-account-changed = Обліковий запис змінився під час завантаження вкладень. Відкрийте «Вкладення» знову для поточного облікового запису.
common-attach-search-account = Шукати в усьому обліковому записі
common-attach-scanning = Пошук… Перевірено сторінок повідомлень: { $pages } · знайдено файлів: { $count }
common-attach-scan-cancelled = Пошук скасовано · знайдено файлів: { $count } на сторінках: { $pages }. Результати неповні.
common-attach-scan-incomplete = Пошук перервано · знайдено файлів: { $count }. Повторіть, щоб шукати в усьому обліковому записі.
common-attach-scan-limited = Показано перші 1000 відповідних файлів. Уточніть фільтри, щоб знайти потрібні файли.
common-attach-scan-complete = Пошук завершено · знайдено файлів: { $count } на сторінках повідомлень: { $pages }
common-attach-browse-pages = Перегляд за сторінками
common-attach-cancel-search = Скасувати пошук
multiselect-selected = Вибрано: { $label }
multiselect-remove = Вилучити: { $label }
multiselect-no-match = Немає відповідних варіантів
common-notification-permission-title = Дозволити сповіщення?
common-notification-permission-help = Отримуйте сповіщення про нові повідомлення на цьому пристрої. Це можна змінити в налаштуваннях.
common-notification-permission-allow = Дозволити сповіщення
common-notification-permission-later = Не зараз
common-notification-permission-error = Не вдалося ввімкнути сповіщення. Перевірте дозволи та налаштування сповіщень і повторіть спробу.
common-notifications-enabled = Сповіщення на цьому пристрої ввімкнено.
multiselect-count = { $count ->
    [one] Вибрано { $count }
    [few] Вибрано { $count }
    [many] Вибрано { $count }
   *[other] Вибрано { $count }
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = Сервер зайнятий. Зачекайте кілька секунд і спробуйте ще раз.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = Немає збігів

combobox-toggle = Показати варіанти для { $label }

common-attach-cat-archive = Архіви

common-attach-cat-document = Документи

common-attach-cat-spreadsheet = Електронні таблиці

common-conn-reconnecting = Повторне підключення…

common-viewer-pdf-details = Технічні деталі

common-viewer-pdf-details-help = Додайте ці рядки, коли повідомляєте про проблему. Вони не містять вмісту документа.

timezone-group-other = інше

timezone-use-browser = Використовувати часовий пояс браузера ({ $zone })

common-context-actions = Дії для { $name }
common-open = Відкрити
common-new = Новий елемент
common-refresh = Оновити
common-duplicate = Дублювати
common-copy-fingerprint = Копіювати відбиток
common-export-public-key = Експортувати відкритий ключ або сертифікат
common-context-show = Показати
common-context-show-only = Показати лише цей календар
common-context-color = Колір
common-context-import = Імпорт
common-context-export = Експорт
common-context-pinned = Закріплені елементи
common-context-unpin = Відкріпити
common-context-my-day = Додати до Мого дня
common-context-remove-my-day = Вилучити з Мого дня
common-notify-new-count = Нові повідомлення: { $count }
common-network-internal = Внутрішня мережа
common-module-notifications = Сповіщення
common-actions = Дії
