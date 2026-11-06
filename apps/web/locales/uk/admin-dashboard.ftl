# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Оновити
admin-dash-unavailable = Показники панелі ще недоступні.
admin-dash-unavailable-detail = Цей сервер не надає даних для панелі (GET /admin/dashboard повертає 404). Вони з’являться з наступним оновленням сервера. Інші розділи працюють як раніше.
admin-dash-error-network = Не вдалося завантажити панель: сервер не відповів. Спробуйте ще раз.
admin-dash-error-status = Не вдалося завантажити панель: сервер повернув { $status }. Спробуйте ще раз; якщо це повториться, перевірте журнал сервера.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Потребує уваги
admin-dash-warn-fix = Відкрити розділ, де це виправляється
admin-dash-warn-blocked-accounts = { $count ->
    [one] { $count } обліковий запис не може ввійти, бо його домен не має конфігурації.
    [few] { $count } облікові записи не можуть ввійти, бо їхні домени не мають конфігурації.
    [many] { $count } облікових записів не можуть ввійти, бо їхні домени не мають конфігурації.
   *[other] { $count } облікового запису не можуть ввійти, бо їхні домени не мають конфігурації.
}
admin-dash-warn-no-trusted-proxy = Правила IP активні, але довірений проксі не налаштовано, тому всі клієнти можуть виглядати так, ніби приходять з адреси проксі.
admin-dash-warn-allowlist-bypass = Змінна середовища вимикає список дозволених IP.
admin-dash-warn-metrics-token = Для кінцевої точки метрик не задано токен.
admin-dash-warn-pending-migrations = { $count ->
    [one] { $count } міграцію бази даних не застосовано. Перезапустіть сервер, щоб застосувати її.
    [few] { $count } міграції бази даних не застосовано. Перезапустіть сервер, щоб застосувати їх.
    [many] { $count } міграцій бази даних не застосовано. Перезапустіть сервер, щоб застосувати їх.
   *[other] { $count } міграції бази даних не застосовано. Перезапустіть сервер, щоб застосувати їх.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Термін дії { $count } сертифіката вищестоящого сервера скоро спливає (за останньою перевіркою з’єднання).
    [few] Термін дії { $count } сертифікатів вищестоящих серверів скоро спливає (за останньою перевіркою з’єднання).
    [many] Термін дії { $count } сертифікатів вищестоящих серверів скоро спливає (за останньою перевіркою з’єднання).
   *[other] Термін дії { $count } сертифіката вищестоящих серверів скоро спливає (за останньою перевіркою з’єднання).
}
admin-dash-warn-unknown = Попередження сервера { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] { $count } обліковий запис заблоковано: немає конфігурації домену
    [few] { $count } облікові записи заблоковано: немає конфігурації домену
    [many] { $count } облікових записів заблоковано: немає конфігурації домену
   *[other] { $count } облікового запису заблоковано: немає конфігурації домену
}
admin-dash-blocked-note = Ці облікові записи входили до цього оновлення, але їхній домен не має конфігурації в консолі, тому тепер вони не можуть увійти. Налаштування домену знову дозволить їм входити.
admin-dash-blocked-chart = Заблоковані облікові записи за доменами
admin-dash-blocked-row = { $count ->
    [one] { $count } обліковий запис
    [few] { $count } облікові записи
    [many] { $count } облікових записів
   *[other] { $count } облікового запису
}
admin-dash-blocked-seen = (останній використаний поштовий хост { $host })
admin-dash-configure-domain = Налаштувати { $domain }
admin-dash-blocked-no-access = Ваша роль не дозволяє налаштовувати домени.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Контрольний список налаштування: відкритих пунктів немає
    [one] Контрольний список налаштування: відкрито { $open } пункт
    [few] Контрольний список налаштування: відкрито { $open } пункти
    [many] Контрольний список налаштування: відкрито { $open } пунктів
   *[other] Контрольний список налаштування: відкрито { $open } пункту
}
admin-dash-check-done = Виконано
admin-dash-check-open = Відкрито
admin-dash-check-optional = (необов’язково)
admin-dash-check-go = Відкрити
admin-dash-check-hide = Приховати
admin-dash-check-unhide = Показати знову
admin-dash-check-hidden = { $count ->
    [one] { $count } пункт приховано. Приховування не позначає його виконаним.
    [few] { $count } пункти приховано. Приховування не позначає їх виконаними.
    [many] { $count } пунктів приховано. Приховування не позначає їх виконаними.
   *[other] { $count } пункту приховано. Приховування не позначає їх виконаними.
}
admin-dash-check-show-hidden = Показати приховані пункти
admin-dash-check-hide-hidden = Не показувати приховані пункти
admin-dash-check-domain-exists = Додати поштовий домен
admin-dash-check-domain-upstream = Призначити домену поштовий сервер або задати типовий вищестоящий сервер
admin-dash-check-test-connection = Виконати успішну перевірку з’єднання (за останні 7 днів)
admin-dash-check-preferred-domain = Вибрати бажаний домен, щоб користувачі могли входити, не вводячи його
admin-dash-check-second-admin = Створити другий обліковий запис адміністратора або вимкнути вхід через змінні середовища
admin-dash-check-trusted-proxies = Налаштувати довірені проксі перед цим сервером
admin-dash-check-security-policy = Зберегти політику безпеки
admin-dash-check-metrics-token = Задати токен для кінцевої точки метрик
admin-dash-check-branding = Переглянути брендування

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Облікові записи та сеанси
admin-dash-users = Створені користувачі
admin-dash-signed-in-24h = Облікові записи, що входили, за останні 24 години
admin-dash-signed-in-7d = Облікові записи, що входили, за останні 7 днів
admin-dash-domains = Домени
admin-dash-domains-detail = З поштовим сервером: { $upstream }, вимкнено: { $disabled }
admin-dash-sessions = Активні сеанси вебпошти
admin-dash-admin-sessions = Активні сеанси адміністраторів
admin-dash-security = Безпека входу
admin-dash-failed-webmail = Невдалі входи до вебпошти за останні 24 години
admin-dash-failed-admin = Невдалі входи адміністраторів за останні 24 години
admin-dash-active-blocks = Заблоковані адреси
admin-dash-allowlists = Списки дозволених IP
admin-dash-allowlists-value = Адміністрування { $admin }, вебпошта { $webmail }
admin-dash-on = увімк.
admin-dash-off = вимк.
admin-dash-trusted-proxies = Довірені проксі
admin-dash-configured = Налаштовано
admin-dash-not-configured = Не налаштовано
admin-dash-untrusted-forwarded = { $count ->
    [one] { $count } запит за 24 години містив заголовки переспрямування від недовіреного вузла
    [few] { $count } запити за 24 години містили заголовки переспрямування від недовіреного вузла
    [many] { $count } запитів за 24 години містили заголовки переспрямування від недовіреного вузла
   *[other] { $count } запиту за 24 години містили заголовки переспрямування від недовіреного вузла
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Трафік за останні { $mins } хв
admin-dash-traffic-note = Підраховано цим процесом сервера від його запуску. Інші процеси чи репліки не враховано.
admin-dash-traffic-empty = За цей проміжок запитів не зафіксовано.
admin-dash-requests = Запити за хвилину
admin-dash-errors = Помилки сервера (5xx) за хвилину
admin-dash-p95 = Час відповіді, 95-й процентиль

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Стан
admin-dash-h-ok = Гаразд
admin-dash-h-attention = Потребує уваги
admin-dash-h-db = База даних
admin-dash-h-migrations = Міграції
admin-dash-h-engine = Синхронізація пошти
admin-dash-h-outbox = Вихідні
admin-dash-h-index = Пошуковий індекс
admin-dash-h-sandbox = Пісочниця рендерингу
admin-dash-h-plugins = Середовище виконання плагінів

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Сховище
admin-dash-db-size = Файл бази даних
admin-dash-wal-size = Журнал попереднього запису
admin-dash-index-size = Пошуковий індекс
admin-dash-volume-free = Вільне місце на томі даних
admin-dash-volume-of = з { $total }
admin-dash-na-wal = Ця база даних не використовує файл журналу попереднього запису, або сервер цього не повідомив.
admin-dash-na-volume = Сервер не може визначити вільне місце на цій платформі.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Останні дії адміністраторів
admin-dash-audit-empty = Дій адміністраторів ще не записано.
admin-dash-audit-when = Коли
admin-dash-audit-actor = Хто
admin-dash-audit-action = Дія
admin-dash-audit-target = Об’єкт
admin-dash-audit-all = Відкрити повний журнал аудиту
admin-dash-server = Сервер
admin-dash-version = Версія
admin-dash-build = Збірка
admin-dash-mode = Режим
admin-dash-mode-engine = Рушій: цей сервер синхронізує і зберігає пошту
admin-dash-mode-proxy = Проксі: цей сервер передає запити поштовому серверу
admin-dash-uptime = Працює
admin-dash-base-path = Адреса розміщення
admin-dash-base-root = корінь сайту
admin-dash-duration-dh = { $d } д { $h } год
admin-dash-duration-hm = { $h } год { $m } хв
admin-dash-duration-m = { $m } хв
admin-dash-duration-s = { $s } с

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = Політика за країнами активна, але придатну базу країн не завантажено, тому входи, на які вона поширюється, відхиляються.
admin-dash-warn-geoip-stale = Політика за країнами активна, а база країн застаріла.
admin-dash-h-status-ok = Норма
admin-dash-h-status-warn = Попередження
admin-dash-h-status-fail = Збій
admin-dash-h-status-disabled = Вимкнено
admin-dash-h-checked = Перевірено о { $time }
admin-dash-h-refresh = Перевірити знову
admin-dash-h-refresh-failed = Не вдалося повторити перевірку стану (статус { $status }). Показано попередній результат.
admin-dash-h-refresh-network = Не вдалося повторити перевірку стану: сервер не відповів. Показано попередній результат.
admin-dash-h-enable = Щоб увімкнути
admin-dash-h-probe-time = Перевірка тривала { $time }
admin-dash-h-outbox-probe-time = Збирання стану тривало { $time }; час рукостискання під час надсилання показано окремо.
admin-dash-h-history = Останні перевірки: { $n }
admin-dash-h-open-diagnostics = Діагностика
admin-dash-h-open-logs = Журнали
admin-dash-h-open-plugins = Плагіни
admin-dash-h-open-maintenance = Обслуговування
admin-dash-h-not-tracked = Не відстежується
admin-dash-h-unlimited = без обмежень
admin-dash-h-m-none = немає
admin-dash-h-m-ago = { $age } тому
admin-dash-h-m-ping = Час відгуку
admin-dash-h-m-pool = Пул з'єднань
admin-dash-h-m-pool-value = використовується: { $inUse }, простоює: { $idle } (відкрито: { $size })
admin-dash-h-m-engine = Сервер бази даних
admin-dash-h-m-location = Розташування
admin-dash-h-m-db-size = Розмір бази даних
admin-dash-h-m-wal-size = Журнал випереджувального запису
admin-dash-h-m-journal = Режим журналу
admin-dash-h-m-slow = Повільні запити (за останню годину)
admin-dash-h-m-applied = Застосовано
admin-dash-h-m-applied-value = { $applied } з { $expected } у цій збірці
admin-dash-h-m-latest = Остання застосована
admin-dash-h-m-pending = Ще не застосовано
admin-dash-h-m-failed = Розпочато, не завершено
admin-dash-h-m-unknown = З новішої збірки
admin-dash-h-m-accounts = Поштові скриньки
admin-dash-h-m-accounts-value = зареєстровано: { $total }, синхронізується: { $syncing }, простоює: { $idle }, з помилками: { $erroring }
admin-dash-h-m-last-sync = Від останньої завершеної синхронізації
admin-dash-h-m-last-sync-value = медіана { $p50 }, максимум { $max }
admin-dash-h-m-never-synced = Синхронізації від запуску сервера не було
admin-dash-h-m-queue = Синхронізації, що виконуються
admin-dash-h-m-errors-hour = Помилки синхронізації (за останню годину)
admin-dash-h-m-by-kind = З'єднання за бекендами
admin-dash-h-m-kind-value = { $kind }: скриньок { $accounts }, з'єднань спостереження { $watch }
admin-dash-h-m-recent-errors = Нещодавні помилки
admin-dash-h-m-queued = Очікує надсилання
admin-dash-h-m-queued-value = до надсилання: { $queued }, повторюється: { $deferred }, надсилається: { $sending }
admin-dash-h-m-oldest = Найдовше очікує
admin-dash-h-m-scheduled = Заплановано або утримано
admin-dash-h-m-scheduled-value = заплановано: { $scheduled }, утримано: { $held }
admin-dash-h-m-failed-total = Невдалі (досі у вихідних)
admin-dash-h-m-failed-total-value = невдалих: { $failed }, з невідомим результатом доставлення: { $uncertain }
admin-dash-h-m-sends-hour = Надсилання (за останню годину)
admin-dash-h-m-sends-hour-value = надіслано: { $sent }, відкладено: { $deferred }, невдало: { $failed }
admin-dash-h-m-last-send = Останнє надсилання
admin-dash-h-m-last-send-value = { $outcome }, { $age } тому
admin-dash-h-m-last-send-none = Не було від запуску сервера
admin-dash-h-outcome-sent = Надіслано
admin-dash-h-outcome-deferred = Відкладено
admin-dash-h-outcome-failed = Невдало
admin-dash-h-outcome-uncertain = Результат доставлення невідомий
admin-dash-h-m-submission = Сервери надсилання
admin-dash-h-m-submission-value = відповіли { $reachable } з { $checked } (лише рукостискання, без входу; найповільніший { $slowest }), перевірено { $age } тому
admin-dash-h-m-submission-none = Сервери надсилання не налаштовано.
admin-dash-h-m-submission-pending = Триває перевірка з'єднання.
admin-dash-h-m-submission-unmeasured = Серверів, що потребують перевірки з автентифікацією в провайдера: { $count }; облікові дані не надсилалися.
admin-dash-h-m-documents = Документи
admin-dash-h-m-documents-value = { $documents } з { $messages } збережених повідомлень
admin-dash-h-m-index-pending = Ще не проіндексовано
admin-dash-h-m-index-size = Розмір на диску
admin-dash-h-m-index-memory = У пам'яті
admin-dash-h-m-last-commit = Остання фіксація
admin-dash-h-m-rebuild = Перебудова
admin-dash-h-m-rebuild-running = Виконується: { $pct } ({ $done } з { $total })
admin-dash-h-m-rebuild-idle = Не виконується
admin-dash-h-m-version = Версія обробника
admin-dash-h-m-worker-missing = Немає виконуваного файлу обробника
admin-dash-h-m-render-ping = Пінг (запуск, ізоляція, відповідь)
admin-dash-h-m-render-jobs = Завдання, що виконуються
admin-dash-h-m-render-failures = Проблеми (за останню годину)
admin-dash-h-m-render-failures-value = невдало: { $failed }, за тайм-аутом: { $timeouts }, відхилено через зайнятість: { $busy }
admin-dash-h-m-jail = Ізоляція ядра (jail)
admin-dash-h-m-jail-required = Потрібна
admin-dash-h-m-jail-optional = Не потрібна на цій платформі
admin-dash-h-m-jail-degraded = Погіршена: { $why }
admin-dash-h-m-plugins = Плагіни
admin-dash-h-m-plugins-value = встановлено: { $installed }, увімкнено: { $enabled }, завантажено: { $loaded }
admin-dash-h-m-plugin-list = Увімкнені плагіни
admin-dash-h-m-plugin-loaded = завантажено
admin-dash-h-m-plugin-not-loaded = не завантажено ({ $reason })
admin-dash-h-m-plugin-limits = пам'ять { $memory }, строк { $deadline }, паливо { $fuel }
admin-dash-h-m-plugin-calls = Збої викликів (за останню годину)
admin-dash-h-m-plugin-calls-value = пасток: { $traps }, перевищень лімітів: { $limits }, інших помилок: { $errors }
admin-dash-h-m-host-ping = Блокування хоста плагінів
admin-dash-h-t-ping = Попередження вище { $warn }, збій вище { $fail }
admin-dash-h-t-oldest = Попередження, коли повідомлення чекає довше за { $warn }, збій — довше за { $fail }
admin-dash-h-t-pending = Попередження за понад { $n } непроіндексованих повідомлень
admin-dash-h-t-render = Попередження вище { $warn }; збій, якщо немає відповіді протягом { $timeout }
admin-dash-h-t-sync = Збій, коли не вдається синхронізувати { $pct } або більше скриньок
admin-dash-h-chart-db = Час відгуку бази даних
admin-dash-h-chart-render = Пінг обробника відтворення
admin-dash-h-chart-outbox = Повідомлення, що очікують надсилання
admin-dash-h-chart-search = Непроіндексовані повідомлення
admin-dash-na-index-proxy = Режим proxy: цей сервер не веде пошуковий індекс.
admin-dash-na-index-memory = Пошуковий індекс зберігається в пам'яті, тому в нього немає розміру на диску.
admin-dash-customize = Налаштувати панель
admin-dash-limitations = Доступність даних
