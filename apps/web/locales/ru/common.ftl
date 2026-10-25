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
