# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Ключові показники
admin-stats-charts = Графіки
admin-stats-tables = Таблиці
admin-stats-group-traffic = Трафік за цей період
admin-stats-group-storage = Сховище та доставлення, зараз
admin-stats-refresh = Оновити статистику
admin-stats-loading = Завантаження статистики…
admin-stats-requests-total = Запити
admin-stats-errors-total = Помилки сервера (5xx)
admin-stats-p95-peak = Найповільніший 95-й процентиль
admin-stats-requests = Запити
admin-stats-errors = Помилки сервера (5xx)
admin-stats-not-found = Не знайдено (404)
admin-stats-latency = Час відповіді
admin-stats-series-p50 = Медіана
admin-stats-series-p95 = 95-й процентиль
admin-stats-unit-count = Кількість за { $bucket }
admin-stats-unit-duration = Час за { $bucket } у показаній одиниці
admin-stats-unit-accounts = Заблоковані облікові записи за доменами
admin-stats-chart-about = Лінійний графік від { $from } до { $to }.
admin-stats-bars-about = { $count ->
    [one] Стовпчикова діаграма з одним доменом.
    [few] Стовпчикова діаграма з { $count } доменами.
    [many] Стовпчикова діаграма з { $count } доменами.
   *[other] Стовпчикова діаграма з { $count } домена.
}
admin-stats-view-table = Показати як таблицю
admin-stats-legend = Легенда
admin-stats-empty-chart = За цей період ще нічого не виміряно. Оберіть довший період або зайдіть пізніше, коли сервер обробить більше запитів.
admin-stats-empty-blocked = Заблокованих облікових записів немає.
admin-stats-delta-vs = порівняно з попереднім періодом
admin-stats-delta-same = Стільки ж, як у попередньому періоді
admin-stats-delta-sr-up = Більше на { $change }, ніж у попередньому періоді.
admin-stats-delta-sr-down = Менше на { $change }, ніж у попередньому періоді.
admin-stats-delta-sr-same = Стільки ж, як у попередньому періоді.
admin-ct-group = Час завантаження на стороні клієнта
admin-ct-metric-timeToUsable = Час до готовності до роботи
admin-ct-metric-ttfb = Перший байт
admin-ct-metric-dcl = DOM готовий
admin-ct-metric-load = Завантаження сторінки
admin-ct-metric-fcp = Перше відображення вмісту
admin-ct-metric-lcp = Відображення найбільшого вмісту
admin-ct-metric-folderOpen = Відкриття теки
admin-ct-metric-folderLoadCold = Завантаження теки з мережі
admin-ct-metric-folderLoadCached = Завантаження теки з кешу
admin-ct-metric-messageLoadCold = Завантаження повідомлення з мережі
admin-ct-metric-messageLoadCached = Завантаження повідомлення з кешу або попереднього завантаження
admin-ct-device-desktop = Комп'ютер
admin-ct-device-mobile = Мобільний пристрій
admin-ct-device-unknown = Невідомо
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } замір
        [few] { $count } заміри
        [many] { $count } замірів
       *[other] { $count } заміру
}
admin-ct-no-samples = За цей період замірів немає.
admin-ct-few-samples = Лише замірів: { $count }; потрібно щонайменше { $min }.
admin-ct-chart-usable = Час до готовності до роботи
admin-ct-group-folder = Час завантаження тек
admin-ct-group-message = Час завантаження повідомлень
admin-ct-chart-folder-cold = Завантаження теки з мережі
admin-ct-chart-folder-cached = Завантаження теки з кешу
admin-ct-chart-message-cold = Завантаження повідомлення з мережі
admin-ct-chart-message-cached = Завантаження повідомлення з кешу або попереднього завантаження
admin-ct-folder-off = Час завантаження тек типово не вимірюється. Вимірюються лише ті, хто це ввімкнув.
admin-ct-message-off = Час завантаження повідомлень типово не вимірюється. Вимірюються лише ті, хто це ввімкнув.
admin-ct-chart-note = Процентилі показуються лише для інтервалів, де щонайменше { $min } замірів.
admin-ct-empty-chart = Жоден інтервал не має { $min } замірів, потрібних для показу медіани. Оберіть довший період.
admin-ct-col-samples = Заміри
admin-ct-col-metric = Показник
admin-ct-col-device = Пристрій
admin-ct-table-title = Час завантаження на стороні клієнта за пристроями
admin-ct-table-caption = Заміри, медіана та 95-й процентиль для кожного показника і класу пристроїв.
admin-ct-empty-table = За цей період браузери не повідомляли про час завантаження.
admin-ct-failed = Не вдалося завантажити час завантаження на стороні клієнта.
admin-ct-disabled = Збирання часу завантаження на стороні клієнта вимкнено. Наведені нижче дані стосуються часу до вимкнення.
admin-ct-note = Лише з браузерів із виконаним входом, у вибірку потрапляє { $rate }% сеансів, дані анонімні й агреговані. Час до готовності до роботи відраховується від початку навігації до показу перших рядків списку повідомлень. Значення оцінкові, в межах приблизно { $error }% від істинного.
