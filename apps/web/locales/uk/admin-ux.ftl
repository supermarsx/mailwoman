# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Закрити

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Недоступно — { $reason }
admin-ux-unavailable-default = цей сервер поки цього не підтримує.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Код помилки: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Відкрити
admin-ux-list-back = Назад до списку

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Незбережених змін немає
admin-ux-form-changes =
    { $count ->
        [one] { $count } незбережена зміна
        [few] { $count } незбережені зміни
        [many] { $count } незбережених змін
       *[other] { $count } незбереженої зміни
    }
admin-ux-form-discard = Відкинути зміни
admin-ux-form-saving = Збереження…
admin-ux-form-saved =
    { $count ->
        [one] Збережено { $count } зміну.
        [few] Збережено { $count } зміни.
        [many] Збережено { $count } змін.
       *[other] Збережено { $count } зміни.
    }
admin-ux-form-fix =
    { $count ->
        [one] Нічого не збережено. Виправте { $count } поле, позначене нижче.
        [few] Нічого не збережено. Виправте { $count } поля, позначені нижче.
        [many] Нічого не збережено. Виправте { $count } полів, позначених нижче.
       *[other] Нічого не збережено. Виправте { $count } поля, позначені нижче.
    }
admin-ux-learn-more = Докладніше
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = про параметр «{ $label }»
admin-ux-propagation-hot = Застосовується протягом 30 секунд у всіх процесах сервера.
admin-ux-propagation-restart = Застосовується після перезапуску сервера.
admin-ux-secret-set = Значення збережено. Введіть нове, щоб замінити його.
admin-ux-secret-unset = Значення не збережено.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Введіть значення.
admin-ux-err-number = Введіть число.
admin-ux-err-whole = Введіть ціле число.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Введіть { $min } або більше.
admin-ux-err-max = Введіть { $max } або менше.
admin-ux-err-max-length = Використайте не більше ніж { $max } символів.
admin-ux-err-url = Введіть повну адресу, що починається з https:// або http://.
admin-ux-err-email = Введіть адресу електронної пошти, наприклад name@example.org.
admin-ux-err-pattern = Це значення має неочікуваний формат.
admin-ux-err-option = Виберіть один із наведених варіантів.
admin-ux-err-out-of-range = Сервер відхилив це значення, бо воно поза дозволеним діапазоном.
admin-ux-err-too-long = Сервер відхилив це значення, бо воно задовге.
admin-ux-err-invalid = Сервер відхилив це значення. Перевірте його і збережіть знову.
admin-ux-err-pinned = Це значення задано в середовищі сервера, і його не можна змінити тут.
admin-ux-err-locked = Це значення заблоковано на ширшому рівні, і його не можна змінити тут.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Типове
admin-ux-source-stored = Збережено
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Задано через { $name }
admin-ux-source-env-fallback = Резервне значення з { $name }
admin-ux-source-build = Задано під час збирання
# $scope — "global" or a domain name.
admin-ux-source-locked = Заблоковано: { $scope }
admin-ux-source-read-only = Лише читання
admin-ux-source-restart = Потрібен перезапуск
admin-ux-source-env-pin-reason = Середовище сервера задає це значення через { $name }. Змініть його там і перезапустіть сервер.
admin-ux-source-build-reason = Це значення фіксується під час збирання сервера.
admin-ux-source-locked-reason = Це значення зафіксовано блокуванням на рівні { $scope }. Щоб змінити його, зніміть блокування там.
admin-ux-source-read-only-reason = Консоль показує це значення, але не може його змінити. Де воно задається, вказано в документації.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Остання зміна: { $who }, { $time }
admin-ux-history-changed-by = Остання зміна: { $who }
admin-ux-history-changed-at = Остання зміна: { $time }
admin-ux-history-never = Не змінювалося через консоль
admin-ux-history-link = Історія

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = Типове: { $value }
admin-ux-revert = Повернути типове значення
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Повернуто типове значення ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } змінив(-ла) це { $time }, поки ви редагували.
admin-ux-stale-by = { $who } змінив(-ла) це, поки ви редагували.
admin-ux-stale-unknown = Хтось інший змінив це, поки ви редагували.
admin-ux-stale-choice = Перезавантажте, щоб побачити чужу версію й утратити свої правки, або залиште свої й збережіть їх поверх.
admin-ux-stale-reload = Перезавантажити
admin-ux-stale-keep = Залишити мої

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Перевірка того, на що впливає ця зміна…
admin-ux-impact-heading = Ця зміна впливає на
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] і ще { $count }
        [few] і ще { $count }
        [many] і ще { $count }
       *[other] і ще { $count }
    }
admin-ux-impact-none = Більше нічого не змінюється.
admin-ux-impact-unavailable = Попередній перегляд наслідків недоступний — сервер не повідомив, на що впливає ця зміна.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Фільтр
admin-ux-stream-pause = Призупинити
admin-ux-stream-resume = Відновити
admin-ux-stream-clear = Очистити
admin-ux-stream-announce = Зачитувати нові записи вголос
admin-ux-stream-connecting = Підключення…
admin-ux-stream-live = У реальному часі
admin-ux-stream-ended = Завершено
admin-ux-stream-lost = З’єднання втрачено
admin-ux-stream-reconnect = Перепідключитися
admin-ux-stream-paused =
    { $count ->
        [0] Призупинено
        [one] Призупинено — затримано { $count } новий запис
        [few] Призупинено — затримано { $count } нові записи
        [many] Призупинено — затримано { $count } нових записів
       *[other] Призупинено — затримано { $count } нового запису
    }
admin-ux-stream-no-match = Немає записів, що відповідають фільтру.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Поле

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Вигляд
admin-ux-appearance-title = Вигляд консолі
admin-ux-appearance-note = Ці налаштування змінюють лише консоль адміністратора і лише в цьому браузері.
admin-ux-appearance-following = Консоль використовує тему вебпошти цього браузера, доки ви не виберете іншу тут. Ці налаштування змінюють лише консоль адміністратора і лише в цьому браузері.
admin-ux-appearance-mode = Режим
admin-ux-appearance-mode-light = Світлий
admin-ux-appearance-mode-dark = Темний
admin-ux-appearance-mode-system = Як у системі
admin-ux-appearance-theme = Тема
admin-ux-appearance-accent = Акцентний колір
admin-ux-appearance-light = Світла
admin-ux-appearance-dark = Темна
admin-ux-appearance-reset = Використовувати тему вебпошти
