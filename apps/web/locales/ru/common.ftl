# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = ОК
common-cancel = Отмена
common-close = Закрыть
common-save = Сохранить
common-delete = Удалить
common-remove = Убрать
common-edit = Изменить
common-add = Добавить
common-back = Назад
common-next = Далее
common-done = Готово
common-apply = Применить
common-confirm = Подтвердить
common-retry = Повторить
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = Если ошибка повторяется, обратитесь к администратору.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Это приложение не настроено для работы с сервером. Обратитесь к администратору.
common-copy = Копировать
common-search = Поиск
common-more = Ещё
common-yes = Да
common-no = Нет

# -- Generic states ----------------------------------------------------------
common-loading = Загрузка…
common-saving = Сохранение…
common-empty = Здесь пока ничего нет
common-error = Что-то пошло не так
common-error-network = Не удаётся связаться с сервером. Проверьте подключение и повторите попытку.
common-offline = Нет подключения к сети

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Подключение…
common-conn-degraded = Обновления в реальном времени приостановлены — переподключение
common-conn-offline = Нет подключения к сети — изменения синхронизируются после переподключения
common-conn-auth-expired = Срок действия сеанса истёк — войдите снова
common-conn-reconnect = Переподключиться
common-conn-sign-in-again = Войти снова
common-conn-reconnected = Подключение восстановлено

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Новое сообщение
common-notify-no-subject = (без темы)

# -- Generic labels ----------------------------------------------------------
common-required = Обязательно
common-optional = Необязательно

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Открыть папки и приложения
common-nav-close = Закрыть папки и приложения

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Вложения
common-attach-search = Поиск вложений
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Фильтр по типу
common-attach-loading = Загрузка вложений…
common-attach-empty = Нет подходящих вложений.
common-attach-cat-all = все
common-attach-cat-image = изображения
common-attach-cat-pdf = pdf
common-attach-cat-text = текст
common-attach-cat-audio = аудио
common-attach-cat-video = видео
common-attach-cat-other = другое

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = Показать пароль
common-password-hide = Скрыть пароль

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = Почта
common-module-calendar = Календарь
common-module-tasks = Задачи
common-module-notes = Заметки
common-module-contacts = Контакты
common-module-keys = Ключи

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = Загрузка средства просмотра…
common-viewer-loading-audio = Загрузка аудио…
common-viewer-loading-image = Загрузка изображения…
common-viewer-loading-video = Загрузка видео…
common-viewer-rendering-pdf = Отрисовка PDF…
common-viewer-pdf-failed = Не удалось отобразить этот PDF.
common-viewer-no-preview = Для этого типа файлов нет встроенного просмотра.
common-viewer-unknown-type = неизвестный тип
common-viewer-download = Скачать
common-viewer-attachments = Вложения

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Повторная попытка…
common-waiting-online = Ожидание подключения…
common-failure-offline = Нет подключения. Подключитесь и повторите попытку.
common-failure-network = Не удалось связаться с сервером.
common-failure-server = Сервер не ответил вовремя.
common-failure-rate = Слишком много запросов за короткое время. Подождите немного и повторите.
common-failure-conflict = Это одновременно изменили в другом месте. Перезагрузите и повторите.
common-failure-account = Вошедшая учётная запись изменилась. Перезагрузите страницу.
common-failure-linked = Это недоступно, пока открыт связанный почтовый ящик.
common-failure-refused = Сервер отклонил изменение.
common-new-version = Доступна новая версия Mailwoman. Если закрыть это сообщение, оно появится снова через час.
common-new-version-reload = Перезагрузить сейчас
common-toast-close = Закрыть уведомление
common-toast-seconds = { $seconds } с
common-attach-refresh = Обновить вложения
common-attach-open-message = Открыть сообщение с этим вложением
common-attach-page-filter-note = Фильтры применяются к этой странице. Нажмите «Далее», чтобы просмотреть более старые сообщения с вложениями.
common-attach-pages = Страницы вложений
common-attach-previous = Назад
common-attach-next = Далее
common-attach-page = Страница { $page }
common-attach-column-name = Файл
common-attach-column-type = Тип
common-attach-column-size = Размер
common-attach-column-sender = Отправитель
common-attach-column-subject = Тема
common-attach-column-date = Получено
common-pdf-preview = Предпросмотр PDF
common-pdf-link-title = Открыть ссылку из этого PDF?
common-pdf-link-help = Адрес откроется в новой вкладке за пределами Mailwoman.
common-pdf-link-open = Открыть ссылку
common-open-event = Открыть событие
common-message-preview-too-large = Предпросмотр вложений сообщений ограничен 4 МиБ. Исходное вложение по-прежнему можно скачать.
common-message-preview-failed = Не удалось показать предпросмотр этого вложения сообщения. Файл может не поддерживаться или быть повреждён, либо служба предпросмотра недоступна. Исходный файл по-прежнему можно скачать.
common-message-preview-attachments = Файлы внутри этого сообщения
common-message-preview-download = Скачать файл
common-message-preview-size-bytes = { $count ->
    [one] { $count } байт
    [few] { $count } байта
    [many] { $count } байт
   *[other] { $count } байта
}
common-attach-cat-message = Сообщения / вложения Outlook
common-error-retry-help = Этот раздел временно недоступен. Повторите попытку, чтобы загрузить его снова.
common-error-network-help = Проверьте подключение и повторите попытку. Возможно, сервер временно недоступен.
common-error-page = Не удалось загрузить эту страницу
common-error-page-help = Не удалось скачать ресурс страницы. Повторите попытку. Если проблема не исчезнет, перезагрузите приложение, чтобы получить последнюю версию.
common-reload-app = Перезагрузить приложение
common-new-version-reloading = Доступна новая версия, выполняется перезагрузка…
common-attach-account-changed = Учётная запись изменилась во время загрузки вложений. Откройте «Вложения» снова для текущей учётной записи.
common-attach-search-account = Искать во всей учётной записи
common-attach-scanning = Поиск… Проверено страниц сообщений: { $pages } · найдено файлов: { $count }
common-attach-scan-cancelled = Поиск отменён · найдено файлов: { $count } на страницах: { $pages }. Результаты неполные.
common-attach-scan-incomplete = Поиск прерван · найдено файлов: { $count }. Повторите, чтобы искать во всей учётной записи.
common-attach-scan-limited = Показаны первые 1000 подходящих файлов. Уточните фильтры, чтобы найти нужные файлы.
common-attach-scan-complete = Поиск завершён · найдено файлов: { $count } на страницах сообщений: { $pages }
common-attach-browse-pages = Просмотр по страницам
common-attach-cancel-search = Отменить поиск
multiselect-selected = Выбрано: { $label }
multiselect-remove = Удалить: { $label }
multiselect-no-match = Нет подходящих вариантов
common-notification-permission-title = Разрешить уведомления?
common-notification-permission-help = Получайте уведомления о новых сообщениях на этом устройстве. Это можно изменить в настройках.
common-notification-permission-allow = Разрешить уведомления
common-notification-permission-later = Не сейчас
common-notification-permission-error = Не удалось включить уведомления. Проверьте разрешения и настройки уведомлений и повторите попытку.
common-notifications-enabled = Уведомления на этом устройстве включены.
multiselect-count = { $count ->
    [one] Выбран { $count }
    [few] Выбрано { $count }
    [many] Выбрано { $count }
   *[other] Выбрано { $count }
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = Сервер занят. Подождите несколько секунд и повторите попытку.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = Нет совпадений

combobox-toggle = Показать параметры для { $label }

common-attach-cat-archive = Архивы

common-attach-cat-document = Документы

common-attach-cat-spreadsheet = Таблицы

common-conn-reconnecting = Повторное подключение…

common-viewer-pdf-details = Технические детали

common-viewer-pdf-details-help = Включите эти строки, когда сообщаете о проблеме. Они не содержат содержания документа.

timezone-group-other = Другое

timezone-use-browser = Использовать часовой пояс браузера ({ $zone })
