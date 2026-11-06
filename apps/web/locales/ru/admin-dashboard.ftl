# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Обновить
admin-dash-unavailable = Показатели панели мониторинга пока недоступны.
admin-dash-unavailable-detail = Этот сервер не предоставляет данные панели мониторинга (GET /admin/dashboard возвращает 404). Они появятся в одном из следующих обновлений сервера. Остальные разделы работают как прежде.
admin-dash-error-network = Не удалось загрузить панель мониторинга: сервер не ответил. Повторите попытку.
admin-dash-error-status = Не удалось загрузить панель мониторинга: сервер вернул { $status }. Повторите попытку; если ошибка повторится, проверьте журнал сервера.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Требует внимания
admin-dash-warn-fix = Открыть раздел, где это исправляется
admin-dash-warn-blocked-accounts = { $count ->
    [one] { $count } учётная запись не может войти, так как для её домена нет конфигурации.
    [few] { $count } учётные записи не могут войти, так как для их доменов нет конфигурации.
    [many] { $count } учётных записей не могут войти, так как для их доменов нет конфигурации.
   *[other] { $count } учётной записи не могут войти, так как для их доменов нет конфигурации.
}
admin-dash-warn-no-trusted-proxy = Правила IP действуют, но доверенный прокси не настроен, поэтому все клиенты могут выглядеть так, будто приходят с адреса прокси.
admin-dash-warn-allowlist-bypass = Переменная окружения отключает список разрешённых IP.
admin-dash-warn-metrics-token = Для конечной точки метрик не задан токен.
admin-dash-warn-pending-migrations = { $count ->
    [one] { $count } миграция базы данных не применена. Перезапустите сервер, чтобы применить её.
    [few] { $count } миграции базы данных не применены. Перезапустите сервер, чтобы применить их.
    [many] { $count } миграций базы данных не применены. Перезапустите сервер, чтобы применить их.
   *[other] { $count } миграции базы данных не применены. Перезапустите сервер, чтобы применить их.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Срок действия { $count } сертификата вышестоящего сервера скоро истекает (по последней проверке подключения).
    [few] Срок действия { $count } сертификатов вышестоящих серверов скоро истекает (по последней проверке подключения).
    [many] Срок действия { $count } сертификатов вышестоящих серверов скоро истекает (по последней проверке подключения).
   *[other] Срок действия { $count } сертификата вышестоящих серверов скоро истекает (по последней проверке подключения).
}
admin-dash-warn-unknown = Предупреждение сервера { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] Заблокирована { $count } учётная запись: нет конфигурации домена
    [few] Заблокированы { $count } учётные записи: нет конфигурации домена
    [many] Заблокировано { $count } учётных записей: нет конфигурации домена
   *[other] Заблокировано { $count } учётной записи: нет конфигурации домена
}
admin-dash-blocked-note = Эти учётные записи входили до этого обновления, но для их домена в консоли нет конфигурации, поэтому сейчас они не могут войти. После настройки домена вход снова станет возможен.
admin-dash-blocked-chart = Заблокированные учётные записи по доменам
admin-dash-blocked-row = { $count ->
    [one] { $count } учётная запись
    [few] { $count } учётные записи
    [many] { $count } учётных записей
   *[other] { $count } учётной записи
}
admin-dash-blocked-seen = (последний использованный почтовый хост { $host })
admin-dash-configure-domain = Настроить { $domain }
admin-dash-blocked-no-access = Ваша роль не позволяет настраивать домены.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Список настройки: открытых пунктов нет
    [one] Список настройки: открыт { $open } пункт
    [few] Список настройки: открыто { $open } пункта
    [many] Список настройки: открыто { $open } пунктов
   *[other] Список настройки: открыто { $open } пункта
}
admin-dash-check-done = Выполнено
admin-dash-check-open = Открыто
admin-dash-check-optional = (необязательно)
admin-dash-check-go = Открыть
admin-dash-check-hide = Скрыть
admin-dash-check-unhide = Показать снова
admin-dash-check-hidden = { $count ->
    [one] Скрыт { $count } пункт. Скрытие не отмечает его как выполненный.
    [few] Скрыто { $count } пункта. Скрытие не отмечает их как выполненные.
    [many] Скрыто { $count } пунктов. Скрытие не отмечает их как выполненные.
   *[other] Скрыто { $count } пункта. Скрытие не отмечает их как выполненные.
}
admin-dash-check-show-hidden = Показать скрытые пункты
admin-dash-check-hide-hidden = Не показывать скрытые пункты
admin-dash-check-domain-exists = Добавьте почтовый домен
admin-dash-check-domain-upstream = Укажите почтовый сервер для домена или задайте вышестоящий сервер по умолчанию
admin-dash-check-test-connection = Выполните успешную проверку подключения (за последние 7 дней)
admin-dash-check-preferred-domain = Выберите предпочтительный домен, чтобы пользователи могли входить, не вводя его
admin-dash-check-second-admin = Создайте вторую учётную запись администратора или отключите вход через переменные окружения
admin-dash-check-trusted-proxies = Настройте доверенные прокси перед этим сервером
admin-dash-check-security-policy = Сохраните политику безопасности
admin-dash-check-metrics-token = Задайте токен для конечной точки метрик
admin-dash-check-branding = Проверьте оформление бренда

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Учётные записи и сеансы
admin-dash-users = Созданные пользователи
admin-dash-signed-in-24h = Учётные записи, выполнившие вход за последние 24 часа
admin-dash-signed-in-7d = Учётные записи, выполнившие вход за последние 7 дней
admin-dash-domains = Домены
admin-dash-domains-detail = с почтовым сервером: { $upstream }, отключено: { $disabled }
admin-dash-sessions = Активные сеансы веб-почты
admin-dash-admin-sessions = Активные сеансы администраторов
admin-dash-security = Безопасность входа
admin-dash-failed-webmail = Неудачные входы в веб-почту за последние 24 часа
admin-dash-failed-admin = Неудачные входы администраторов за последние 24 часа
admin-dash-active-blocks = Заблокированные адреса
admin-dash-allowlists = Списки разрешённых IP
admin-dash-allowlists-value = Администраторы: { $admin }, веб-почта: { $webmail }
admin-dash-on = вкл.
admin-dash-off = выкл.
admin-dash-trusted-proxies = Доверенные прокси
admin-dash-configured = Настроено
admin-dash-not-configured = Не настроено
admin-dash-untrusted-forwarded = { $count ->
    [one] { $count } запрос за 24 часа содержал заголовки пересылки от недоверенного узла
    [few] { $count } запроса за 24 часа содержали заголовки пересылки от недоверенного узла
    [many] { $count } запросов за 24 часа содержали заголовки пересылки от недоверенного узла
   *[other] { $count } запроса за 24 часа содержали заголовки пересылки от недоверенного узла
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Трафик за последние { $mins } мин
admin-dash-traffic-note = Подсчитано этим процессом сервера с момента его запуска. Другие процессы и реплики не учитываются.
admin-dash-traffic-empty = За этот период запросов не было.
admin-dash-requests = Запросов в минуту
admin-dash-errors = Ошибок сервера (5xx) в минуту
admin-dash-p95 = Время ответа, 95-й процентиль

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Состояние
admin-dash-h-ok = ОК
admin-dash-h-attention = Требует внимания
admin-dash-h-db = База данных
admin-dash-h-migrations = Миграции
admin-dash-h-engine = Синхронизация почты
admin-dash-h-outbox = Исходящие
admin-dash-h-index = Поисковый индекс
admin-dash-h-sandbox = Песочница отрисовки
admin-dash-h-plugins = Среда выполнения плагинов

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Хранилище
admin-dash-db-size = Файл базы данных
admin-dash-wal-size = Журнал упреждающей записи
admin-dash-index-size = Поисковый индекс
admin-dash-volume-free = Свободное место на томе данных
admin-dash-volume-of = из { $total }
admin-dash-na-wal = Эта база данных не использует файл журнала упреждающей записи, или сервер не сообщил эти данные.
admin-dash-na-volume = На этой платформе сервер не может определить свободное место.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Последние действия администраторов
admin-dash-audit-empty = Действия администраторов пока не записаны.
admin-dash-audit-when = Когда
admin-dash-audit-actor = Кто
admin-dash-audit-action = Действие
admin-dash-audit-target = Объект
admin-dash-audit-all = Открыть полный журнал аудита
admin-dash-server = Сервер
admin-dash-version = Версия
admin-dash-build = Сборка
admin-dash-mode = Режим
admin-dash-mode-engine = Движок: этот сервер синхронизирует и хранит почту
admin-dash-mode-proxy = Прокси: этот сервер передаёт запросы почтовому серверу
admin-dash-uptime = Время работы
admin-dash-base-path = Адрес размещения
admin-dash-base-root = корень сайта
admin-dash-duration-dh = { $d } д { $h } ч
admin-dash-duration-hm = { $h } ч { $m } мин
admin-dash-duration-m = { $m } мин
admin-dash-duration-s = { $s } с

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = Политика по странам активна, но пригодная база стран не загружена, поэтому подпадающие под неё входы отклоняются.
admin-dash-warn-geoip-stale = Политика по странам активна, а база стран устарела.
admin-dash-h-status-ok = Норма
admin-dash-h-status-warn = Предупреждение
admin-dash-h-status-fail = Сбой
admin-dash-h-status-disabled = Отключено
admin-dash-h-checked = Проверено в { $time }
admin-dash-h-refresh = Проверить снова
admin-dash-h-refresh-failed = Не удалось повторить проверку состояния (статус { $status }). Показан предыдущий результат.
admin-dash-h-refresh-network = Не удалось повторить проверку состояния: сервер не ответил. Показан предыдущий результат.
admin-dash-h-enable = Чтобы включить
admin-dash-h-probe-time = Проверка заняла { $time }
admin-dash-h-outbox-probe-time = Сбор состояния занял { $time }; время рукопожатия при отправке показано отдельно.
admin-dash-h-history = Последние проверки: { $n }
admin-dash-h-open-diagnostics = Диагностика
admin-dash-h-open-logs = Журналы
admin-dash-h-open-plugins = Плагины
admin-dash-h-open-maintenance = Обслуживание
admin-dash-h-not-tracked = Не отслеживается
admin-dash-h-unlimited = без ограничений
admin-dash-h-m-none = нет
admin-dash-h-m-ago = { $age } назад
admin-dash-h-m-ping = Время отклика
admin-dash-h-m-pool = Пул подключений
admin-dash-h-m-pool-value = используется: { $inUse }, простаивает: { $idle } (открыто: { $size })
admin-dash-h-m-engine = Сервер базы данных
admin-dash-h-m-location = Расположение
admin-dash-h-m-db-size = Размер базы данных
admin-dash-h-m-wal-size = Журнал упреждающей записи
admin-dash-h-m-journal = Режим журнала
admin-dash-h-m-slow = Медленные запросы (за последний час)
admin-dash-h-m-applied = Применено
admin-dash-h-m-applied-value = { $applied } из { $expected } в этой сборке
admin-dash-h-m-latest = Последняя применённая
admin-dash-h-m-pending = Ещё не применено
admin-dash-h-m-failed = Начато, не завершено
admin-dash-h-m-unknown = Из более новой сборки
admin-dash-h-m-accounts = Почтовые ящики
admin-dash-h-m-accounts-value = зарегистрировано: { $total }, синхронизируется: { $syncing }, простаивает: { $idle }, с ошибками: { $erroring }
admin-dash-h-m-last-sync = С момента последней завершённой синхронизации
admin-dash-h-m-last-sync-value = медиана { $p50 }, максимум { $max }
admin-dash-h-m-never-synced = Синхронизации с момента запуска сервера не было
admin-dash-h-m-queue = Выполняющиеся синхронизации
admin-dash-h-m-errors-hour = Ошибки синхронизации (за последний час)
admin-dash-h-m-by-kind = Подключения по бэкендам
admin-dash-h-m-kind-value = { $kind }: ящиков { $accounts }, подключений наблюдения { $watch }
admin-dash-h-m-recent-errors = Недавние ошибки
admin-dash-h-m-queued = Ожидает отправки
admin-dash-h-m-queued-value = к отправке: { $queued }, повторяется: { $deferred }, отправляется: { $sending }
admin-dash-h-m-oldest = Дольше всех ожидает
admin-dash-h-m-scheduled = Запланировано или удержано
admin-dash-h-m-scheduled-value = запланировано: { $scheduled }, удержано: { $held }
admin-dash-h-m-failed-total = Неудачные (всё ещё в исходящих)
admin-dash-h-m-failed-total-value = неудачных: { $failed }, с неизвестным результатом доставки: { $uncertain }
admin-dash-h-m-sends-hour = Отправки (за последний час)
admin-dash-h-m-sends-hour-value = отправлено: { $sent }, отложено: { $deferred }, неудачно: { $failed }
admin-dash-h-m-last-send = Последняя отправка
admin-dash-h-m-last-send-value = { $outcome }, { $age } назад
admin-dash-h-m-last-send-none = Не было с момента запуска сервера
admin-dash-h-outcome-sent = Отправлено
admin-dash-h-outcome-deferred = Отложено
admin-dash-h-outcome-failed = Неудачно
admin-dash-h-outcome-uncertain = Результат доставки неизвестен
admin-dash-h-m-submission = Серверы отправки
admin-dash-h-m-submission-value = ответили { $reachable } из { $checked } (только рукопожатие, без входа; самый медленный { $slowest }), проверено { $age } назад
admin-dash-h-m-submission-none = Серверы отправки не настроены.
admin-dash-h-m-submission-pending = Выполняется проверка подключения.
admin-dash-h-m-submission-unmeasured = Серверов, требующих проверки с аутентификацией у провайдера: { $count }; учётные данные не отправлялись.
admin-dash-h-m-documents = Документы
admin-dash-h-m-documents-value = { $documents } из { $messages } сохранённых сообщений
admin-dash-h-m-index-pending = Ещё не проиндексировано
admin-dash-h-m-index-size = Размер на диске
admin-dash-h-m-index-memory = В памяти
admin-dash-h-m-last-commit = Последняя фиксация
admin-dash-h-m-rebuild = Перестроение
admin-dash-h-m-rebuild-running = Выполняется: { $pct } ({ $done } из { $total })
admin-dash-h-m-rebuild-idle = Не выполняется
admin-dash-h-m-version = Версия обработчика
admin-dash-h-m-worker-missing = Нет исполняемого файла обработчика
admin-dash-h-m-render-ping = Пинг (запуск, изоляция, ответ)
admin-dash-h-m-render-jobs = Выполняющиеся задания
admin-dash-h-m-render-failures = Проблемы (за последний час)
admin-dash-h-m-render-failures-value = неудачно: { $failed }, по тайм-ауту: { $timeouts }, отклонено из-за занятости: { $busy }
admin-dash-h-m-jail = Изоляция ядра (jail)
admin-dash-h-m-jail-required = Требуется
admin-dash-h-m-jail-optional = Не требуется на этой платформе
admin-dash-h-m-jail-degraded = Ухудшена: { $why }
admin-dash-h-m-plugins = Плагины
admin-dash-h-m-plugins-value = установлено: { $installed }, включено: { $enabled }, загружено: { $loaded }
admin-dash-h-m-plugin-list = Включённые плагины
admin-dash-h-m-plugin-loaded = загружен
admin-dash-h-m-plugin-not-loaded = не загружен ({ $reason })
admin-dash-h-m-plugin-limits = память { $memory }, срок { $deadline }, топливо { $fuel }
admin-dash-h-m-plugin-calls = Сбои вызовов (за последний час)
admin-dash-h-m-plugin-calls-value = ловушек: { $traps }, превышений лимитов: { $limits }, других ошибок: { $errors }
admin-dash-h-m-host-ping = Блокировка хоста плагинов
admin-dash-h-t-ping = Предупреждение выше { $warn }, сбой выше { $fail }
admin-dash-h-t-oldest = Предупреждение, когда сообщение ждёт дольше { $warn }, сбой — дольше { $fail }
admin-dash-h-t-pending = Предупреждение при более чем { $n } непроиндексированных сообщениях
admin-dash-h-t-render = Предупреждение выше { $warn }; сбой, если нет ответа в течение { $timeout }
admin-dash-h-t-sync = Сбой, когда не удаётся синхронизировать { $pct } или более ящиков
admin-dash-h-chart-db = Время отклика базы данных
admin-dash-h-chart-render = Пинг обработчика отрисовки
admin-dash-h-chart-outbox = Сообщения, ожидающие отправки
admin-dash-h-chart-search = Непроиндексированные сообщения
admin-dash-na-index-proxy = Режим proxy: этот сервер не ведёт поисковый индекс.
admin-dash-na-index-memory = Поисковый индекс хранится в памяти, поэтому у него нет размера на диске.
admin-dash-customize = Настроить панель
admin-dash-limitations = Доступность данных
