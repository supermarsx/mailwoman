# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Панель мониторинга
admin-shell-group-overview = Обзор
admin-shell-group-people = Пользователи
admin-shell-group-security = Безопасность
admin-shell-group-configuration = Конфигурация
admin-shell-group-operations = Эксплуатация
admin-shell-menu = Меню
admin-shell-close-menu = Закрыть меню
admin-shell-search = Поиск
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Вы вошли как { $name }
admin-shell-language-label = Язык
admin-shell-breadcrumb = Навигационная цепочка
admin-shell-unknown-section = Этот раздел не существует, или ваша роль не даёт к нему доступа.
admin-shell-back-to-dashboard = Вернуться к панели мониторинга
admin-shell-cap-missing = Ваша роль не включает право { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Загрузка…
admin-shell-retry = Повторить попытку
admin-shell-cancel = Отмена
admin-shell-save = Сохранить
admin-shell-not-available = Недоступно

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Покинуть эту страницу?
admin-shell-leave-body = Изменения на этой странице не сохранены. При уходе они будут потеряны.
admin-shell-leave-confirm = Уйти и отменить изменения

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Введите { $name } для подтверждения

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Поиск по консоли администратора
admin-shell-palette-label = Раздел, пользователь, домен или настройка
admin-shell-palette-results = Результаты
admin-shell-palette-none = Ничего не найдено.
admin-shell-palette-searching = Поиск пользователей, доменов и настроек…
admin-shell-palette-sections-only = Этот сервер пока не умеет искать пользователей, домены и настройки, поэтому показаны только разделы.
admin-shell-palette-error = Поиск пользователей, доменов и настроек не удался; показаны только разделы.
admin-shell-palette-kind-section = Раздел
admin-shell-palette-kind-user = Пользователь
admin-shell-palette-kind-domain = Домен
admin-shell-palette-kind-setting = Настройка

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Фильтры
admin-shell-table-actions = Действия
admin-shell-table-error = Не удалось загрузить список.
admin-shell-table-paging = Страницы: { $table }
admin-shell-table-prev = Предыдущая страница
admin-shell-table-next = Следующая страница

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Последнее: { $latest }, максимум: { $max }
admin-shell-chart-col-time = Время
admin-shell-chart-col-item = Элемент
admin-shell-chart-col-value = Значение

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Задано через
admin-shell-pinned-sr = Это значение берётся из окружения сервера и не может быть изменено здесь.
admin-shell-lock-text = Заблокировать для пользователей
admin-shell-lock-setting = { $setting }
admin-shell-restore = Восстановить предыдущее значение
admin-shell-restore-note = Изменено: { $actor }, { $time }. Предыдущее значение: { $value }
admin-shell-restore-done = Предыдущее значение восстановлено.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Шаги
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Шаг { $n } из { $total }: { $title }
admin-shell-wizard-review = Проверка
admin-shell-wizard-back = Назад
admin-shell-wizard-next = Далее
admin-shell-wizard-saved = Сохранено.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Не удалось связаться с сервером. Проверьте подключение и повторите попытку.
admin-shell-err-session = Сеанс администратора завершён. Войдите снова.
admin-shell-err-forbidden = Ваша роль не разрешает это действие. Обратитесь к администратору с подходящей ролью.
admin-shell-err-out-of-scope = Этот объект относится к домену, который не входит в вашу роль.
admin-shell-err-not-found = Объект больше не существует. Обновите список.
admin-shell-err-conflict = За это время кто-то изменил объект. Обновите страницу и повторите попытку.
admin-shell-err-validation = Сервер отклонил одно из значений. Проверьте поля и повторите попытку.
admin-shell-err-rate-limited = Слишком много запросов. Подождите минуту и повторите попытку.
admin-shell-err-server = Сервер вернул ошибку ({ $status }). Повторите попытку; если ошибка повторится, проверьте журнал сервера.
admin-shell-err-upstream-dns = Не удалось разрешить имя хоста. Проверьте написание имени хоста.
admin-shell-err-upstream-connect = На этом хосте и порту никто не ответил. Проверьте порт и межсетевые экраны на пути.
admin-shell-err-upstream-tls = Сервер ответил, но согласование TLS не удалось. Проверьте порт и настройку защиты соединения.
admin-shell-err-upstream-auth = Сервер отклонил учётные данные. Проверьте имя пользователя и пароль.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Слишком много попыток входа. Подождите несколько минут и повторите попытку.
admin-shell-totp-form = Вход администратора, второй шаг
admin-shell-totp-note = Введите 6-значный код из приложения-аутентификатора, настроенного для { $name }.
admin-shell-totp-code = Код аутентификации
admin-shell-totp-verify = Подтвердить
admin-shell-totp-start-over = Начать заново
admin-shell-totp-invalid = Этот код недействителен. Введите текущий код из приложения-аутентификатора.
admin-shell-totp-expired = Вход занял слишком много времени. Начните заново и введите пароль.
