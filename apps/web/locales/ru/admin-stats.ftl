# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Ключевые показатели
admin-stats-charts = Графики
admin-stats-tables = Таблицы
admin-stats-group-traffic = Трафик за этот период
admin-stats-group-storage = Хранилище и доставка, сейчас
admin-stats-refresh = Обновить статистику
admin-stats-loading = Загрузка статистики…
admin-stats-requests-total = Запросы
admin-stats-errors-total = Ошибки сервера (5xx)
admin-stats-p95-peak = Самый медленный 95-й процентиль
admin-stats-requests = Запросы
admin-stats-errors = Ошибки сервера (5xx)
admin-stats-not-found = Не найдено (404)
admin-stats-latency = Время ответа
admin-stats-series-p50 = Медиана
admin-stats-series-p95 = 95-й процентиль
admin-stats-unit-count = Количество за { $bucket }
admin-stats-unit-duration = Время за { $bucket } в показанной единице
admin-stats-unit-accounts = Заблокированные учётные записи по доменам
admin-stats-chart-about = Линейный график с { $from } по { $to }.
admin-stats-bars-about = { $count ->
    [one] Столбчатая диаграмма с одним доменом.
    [few] Столбчатая диаграмма с { $count } доменами.
    [many] Столбчатая диаграмма с { $count } доменами.
   *[other] Столбчатая диаграмма с { $count } домена.
}
admin-stats-view-table = Показать в виде таблицы
admin-stats-legend = Легенда
admin-stats-empty-chart = За этот период пока ничего не измерено. Выберите более длинный период или зайдите позже, когда сервер обработает больше запросов.
admin-stats-empty-blocked = Заблокированных учётных записей нет.
admin-stats-delta-vs = по сравнению с предыдущим периодом
admin-stats-delta-same = Столько же, сколько в предыдущем периоде
admin-stats-delta-sr-up = Больше на { $change }, чем в предыдущем периоде.
admin-stats-delta-sr-down = Меньше на { $change }, чем в предыдущем периоде.
admin-stats-delta-sr-same = Столько же, сколько в предыдущем периоде.
admin-ct-group = Время загрузки на стороне клиента
admin-ct-metric-timeToUsable = Время до готовности к работе
admin-ct-metric-ttfb = Первый байт
admin-ct-metric-dcl = DOM готов
admin-ct-metric-load = Загрузка страницы
admin-ct-metric-fcp = Первая отрисовка содержимого
admin-ct-metric-lcp = Отрисовка самого крупного содержимого
admin-ct-metric-folderOpen = Открытие папки
admin-ct-metric-folderLoadCold = Загрузка папки из сети
admin-ct-metric-folderLoadCached = Загрузка папки из кэша
admin-ct-metric-messageLoadCold = Загрузка сообщения из сети
admin-ct-metric-messageLoadCached = Загрузка сообщения из кэша или предзагрузки
admin-ct-device-desktop = Компьютер
admin-ct-device-mobile = Мобильное устройство
admin-ct-device-unknown = Неизвестно
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } замер
        [few] { $count } замера
        [many] { $count } замеров
       *[other] { $count } замера
}
admin-ct-no-samples = За этот период замеров нет.
admin-ct-few-samples = Всего замеров: { $count }; нужно не менее { $min }.
admin-ct-chart-usable = Время до готовности к работе
admin-ct-group-folder = Время загрузки папок
admin-ct-group-message = Время загрузки сообщений
admin-ct-chart-folder-cold = Загрузка папки из сети
admin-ct-chart-folder-cached = Загрузка папки из кэша
admin-ct-chart-message-cold = Загрузка сообщения из сети
admin-ct-chart-message-cached = Загрузка сообщения из кэша или предзагрузки
admin-ct-folder-off = Время загрузки папок по умолчанию не измеряется. Измеряются только те, кто это включил.
admin-ct-message-off = Время загрузки сообщений по умолчанию не измеряется. Измеряются только те, кто это включил.
admin-ct-chart-note = Процентили показываются только для интервалов, где не менее { $min } замеров.
admin-ct-empty-chart = Ни в одном интервале нет { $min } замеров, необходимых для показа медианы. Выберите более длинный период.
admin-ct-col-samples = Замеры
admin-ct-col-metric = Показатель
admin-ct-col-device = Устройство
admin-ct-table-title = Время загрузки на стороне клиента по устройствам
admin-ct-table-caption = Замеры, медиана и 95-й процентиль для каждого показателя и класса устройств.
admin-ct-empty-table = За этот период браузеры не сообщали о времени загрузки.
admin-ct-failed = Не удалось загрузить время загрузки на стороне клиента.
admin-ct-disabled = Сбор времени загрузки на стороне клиента выключен. Приведённые ниже данные относятся ко времени до выключения.
admin-ct-note = Только из браузеров с выполненным входом, в выборку попадает { $rate }% сеансов, данные анонимны и агрегированы. Время до готовности к работе отсчитывается от начала навигации до показа первых строк списка сообщений. Значения оценочные, в пределах примерно { $error }% от истинного.
