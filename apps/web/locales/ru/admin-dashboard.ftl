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
admin-dash-h-db-ok = Ответ за { $ms }
admin-dash-h-migrations = Миграции
admin-dash-h-migrations-ok = Актуальны (применено: { $n })
admin-dash-h-migrations-behind = Применено { $applied } из { $expected }
admin-dash-h-engine = Синхронизация почты
admin-dash-h-engine-value = Зарегистрировано учётных записей: { $accounts }, с ошибкой синхронизации: { $errors }
admin-dash-h-outbox = Исходящие
admin-dash-h-outbox-value = Ожидают: { $pending } (самое старое: { $oldest }), с ошибкой за 24 часа: { $failed }
admin-dash-h-outbox-none = нет
admin-dash-h-index = Поисковый индекс
admin-dash-h-index-value = Документов: { $n }
admin-dash-h-index-rebuilding = Перестраивается (пока документов: { $n })
admin-dash-h-sandbox = Песочница отрисовки
admin-dash-h-sandbox-ok = Доступна: недоверенные документы отрисовываются в отдельном процессе.
admin-dash-h-sandbox-missing = Отсутствует: недоверенные документы отрисовываются внутри процесса сервера.
admin-dash-h-sandbox-required = Отсутствует, а этот сервер её требует: отрисовка недоверенных документов завершается ошибкой.
admin-dash-h-plugins = Среда выполнения плагинов
admin-dash-h-plugins-ok = Доступна
admin-dash-h-plugins-off = Отключена
admin-dash-na-proxy = этот сервер работает в режиме прокси и сам не синхронизирует почту.
admin-dash-na-unreported = сервер не сообщил эти данные.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Хранилище
admin-dash-db-size = Файл базы данных
admin-dash-wal-size = Журнал упреждающей записи
admin-dash-index-size = Поисковый индекс
admin-dash-volume-free = Свободное место на томе данных
admin-dash-volume-of = из { $total }
admin-dash-na-wal = Эта база данных не использует файл журнала упреждающей записи, или сервер не сообщил эти данные.
admin-dash-na-index = На этом сервере нет поискового индекса.
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
