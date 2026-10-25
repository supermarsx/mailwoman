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
admin-dash-ms = { $ms } мс

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Стан
admin-dash-h-ok = Гаразд
admin-dash-h-attention = Потребує уваги
admin-dash-h-db = База даних
admin-dash-h-db-ok = Відповіла за { $ms }
admin-dash-h-migrations = Міграції
admin-dash-h-migrations-ok = Актуально (застосовано: { $n })
admin-dash-h-migrations-behind = Застосовано { $applied } з { $expected }
admin-dash-h-engine = Синхронізація пошти
admin-dash-h-engine-value = Зареєстровано облікових записів: { $accounts }, з помилкою синхронізації: { $errors }
admin-dash-h-outbox = Вихідні
admin-dash-h-outbox-value = Очікують: { $pending } (найстаріше { $oldest }), не вдалося за 24 години: { $failed }
admin-dash-h-outbox-none = немає
admin-dash-h-index = Пошуковий індекс
admin-dash-h-index-value = Документів: { $n }
admin-dash-h-index-rebuilding = Перебудова (документів поки що: { $n })
admin-dash-h-sandbox = Пісочниця рендерингу
admin-dash-h-sandbox-ok = Доступна: недовірені документи рендеряться в окремому процесі.
admin-dash-h-sandbox-missing = Відсутня: недовірені документи рендеряться всередині процесу сервера.
admin-dash-h-sandbox-required = Відсутня, а цей сервер її вимагає: рендеринг недовірених документів не вдається.
admin-dash-h-plugins = Середовище виконання плагінів
admin-dash-h-plugins-ok = Доступне
admin-dash-h-plugins-off = Вимкнено
admin-dash-na-proxy = цей сервер працює в режимі проксі й сам не синхронізує пошту.
admin-dash-na-unreported = сервер цього не повідомив.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Сховище
admin-dash-db-size = Файл бази даних
admin-dash-wal-size = Журнал попереднього запису
admin-dash-index-size = Пошуковий індекс
admin-dash-volume-free = Вільне місце на томі даних
admin-dash-volume-of = з { $total }
admin-dash-na-wal = Ця база даних не використовує файл журналу попереднього запису, або сервер цього не повідомив.
admin-dash-na-index = Цей сервер не має пошукового індексу.
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
