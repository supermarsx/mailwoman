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
