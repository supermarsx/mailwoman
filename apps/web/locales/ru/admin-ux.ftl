# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Закрыть

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Недоступно — { $reason }
admin-ux-unavailable-default = этот сервер пока этого не поддерживает.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Код ошибки: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Открыть
admin-ux-list-back = Вернуться к списку

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Несохранённых изменений нет
admin-ux-form-changes =
    { $count ->
        [one] { $count } несохранённое изменение
        [few] { $count } несохранённых изменения
        [many] { $count } несохранённых изменений
       *[other] { $count } несохранённого изменения
    }
admin-ux-form-discard = Отменить изменения
admin-ux-form-saving = Сохранение…
admin-ux-form-saved =
    { $count ->
        [one] Сохранено { $count } изменение.
        [few] Сохранено { $count } изменения.
        [many] Сохранено { $count } изменений.
       *[other] Сохранено { $count } изменения.
    }
admin-ux-form-fix =
    { $count ->
        [one] Ничего не сохранено. Исправьте { $count } поле, отмеченное ниже.
        [few] Ничего не сохранено. Исправьте { $count } поля, отмеченные ниже.
        [many] Ничего не сохранено. Исправьте { $count } полей, отмеченных ниже.
       *[other] Ничего не сохранено. Исправьте { $count } поля, отмеченные ниже.
    }
admin-ux-learn-more = Подробнее
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = о параметре «{ $label }»
admin-ux-propagation-hot = Применяется в течение 30 секунд во всех процессах сервера.
admin-ux-propagation-restart = Применяется после перезапуска сервера.
admin-ux-secret-set = Значение сохранено. Введите новое, чтобы заменить его.
admin-ux-secret-unset = Значение не сохранено.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Введите значение.
admin-ux-err-number = Введите число.
admin-ux-err-whole = Введите целое число.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Введите { $min } или больше.
admin-ux-err-max = Введите { $max } или меньше.
admin-ux-err-max-length = Используйте не более { $max } символов.
admin-ux-err-url = Введите полный адрес, начинающийся с https:// или http://.
admin-ux-err-email = Введите адрес электронной почты, например name@example.org.
admin-ux-err-pattern = Значение имеет неверный формат.
admin-ux-err-option = Выберите один из предложенных вариантов.
admin-ux-err-out-of-range = Сервер отклонил это значение, так как оно вне допустимого диапазона.
admin-ux-err-too-long = Сервер отклонил это значение, так как оно слишком длинное.
admin-ux-err-invalid = Сервер отклонил это значение. Проверьте его и сохраните снова.
admin-ux-err-pinned = Это значение задано в окружении сервера и не может быть изменено здесь.
admin-ux-err-locked = Это значение заблокировано на более широком уровне и не может быть изменено здесь.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = По умолчанию
admin-ux-source-stored = Сохранено
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Задано через { $name }
admin-ux-source-env-fallback = Резервное значение из { $name }
admin-ux-source-build = Задано при сборке
# $scope — "global" or a domain name.
admin-ux-source-locked = Заблокировано: { $scope }
admin-ux-source-read-only = Только чтение
admin-ux-source-restart = Нужен перезапуск
admin-ux-source-env-pin-reason = Окружение сервера задаёт это значение через { $name }. Измените его там и перезапустите сервер.
admin-ux-source-build-reason = Это значение фиксируется при сборке сервера.
admin-ux-source-locked-reason = Это значение зафиксировано блокировкой на уровне { $scope }. Чтобы изменить его, снимите блокировку там.
admin-ux-source-read-only-reason = Консоль показывает это значение, но не может его изменить. Где оно задаётся, указано в документации.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Последнее изменение: { $who }, { $time }
admin-ux-history-changed-by = Последнее изменение: { $who }
admin-ux-history-changed-at = Последнее изменение: { $time }
admin-ux-history-never = Не изменялось через консоль
admin-ux-history-link = История

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = По умолчанию: { $value }
admin-ux-revert = Вернуть значение по умолчанию
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Возвращено значение по умолчанию ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } изменил(а) это { $time }, пока вы редактировали.
admin-ux-stale-by = { $who } изменил(а) это, пока вы редактировали.
admin-ux-stale-unknown = Кто-то другой изменил это, пока вы редактировали.
admin-ux-stale-choice = Перезагрузите, чтобы увидеть чужую версию и потерять свои правки, или оставьте свои и сохраните их поверх.
admin-ux-stale-reload = Перезагрузить
admin-ux-stale-keep = Оставить мои

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Проверка того, на что влияет это изменение…
admin-ux-impact-heading = Это изменение затрагивает
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] и ещё { $count }
        [few] и ещё { $count }
        [many] и ещё { $count }
       *[other] и ещё { $count }
    }
admin-ux-impact-none = Больше ничего не меняется.
admin-ux-impact-unavailable = Предпросмотр последствий недоступен — сервер не сообщил, на что влияет это изменение.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Фильтр
admin-ux-stream-pause = Приостановить
admin-ux-stream-resume = Возобновить
admin-ux-stream-clear = Очистить
admin-ux-stream-announce = Зачитывать новые записи вслух
admin-ux-stream-connecting = Подключение…
admin-ux-stream-live = В реальном времени
admin-ux-stream-ended = Завершено
admin-ux-stream-lost = Соединение потеряно
admin-ux-stream-reconnect = Переподключиться
admin-ux-stream-paused =
    { $count ->
        [0] Приостановлено
        [one] Приостановлено — задержана { $count } новая запись
        [few] Приостановлено — задержаны { $count } новые записи
        [many] Приостановлено — задержано { $count } новых записей
       *[other] Приостановлено — задержано { $count } новой записи
    }
admin-ux-stream-no-match = Нет записей, соответствующих фильтру.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Поле

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Оформление
admin-ux-appearance-title = Оформление консоли
admin-ux-appearance-note = Эти настройки меняют только консоль администратора и только в этом браузере.
admin-ux-appearance-following = Консоль использует тему веб-почты этого браузера, пока вы не выберете здесь другую. Эти настройки меняют только консоль администратора и только в этом браузере.
admin-ux-appearance-mode = Режим
admin-ux-appearance-mode-light = Светлый
admin-ux-appearance-mode-dark = Тёмный
admin-ux-appearance-mode-system = Как в системе
admin-ux-appearance-theme = Тема
admin-ux-appearance-accent = Акцентный цвет
admin-ux-appearance-light = Светлая
admin-ux-appearance-dark = Тёмная
admin-ux-appearance-reset = Использовать тему веб-почты
