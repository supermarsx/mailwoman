# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Скорость
privacy-speed-unavailable = Данные о скорости с этого сервера недоступны.
privacy-speed-empty = За этот период изображения ещё не отдавались.
privacy-speed-off = Отслеживание скорости выключено (настройка: Отслеживание скорости прокси изображений). Новые данные не записываются.
privacy-speed-verdict-healthy = В норме
privacy-speed-verdict-slow = Медленно
privacy-speed-verdict-degraded = Ухудшено
privacy-speed-verdict-noData = Недостаточно данных
privacy-speed-reason-ok = Загруженные изображения и попадания в кэш укладываются в указанные ниже пределы, а ошибок мало.
privacy-speed-reason-failureRate = { $failurePct } % запросов завершились ответом «занято», тайм-аутом или ошибкой источника.
privacy-speed-reason-missP95 = Изображения, которые пришлось загружать, отдаются медленно: их 95-й процентиль выше указанного предела.
privacy-speed-reason-hitP95 = Изображения из кэша отдаются медленно: их 95-й процентиль выше указанного предела.
privacy-speed-reason-fewRequests = Менее { $min } запросов за период, этого недостаточно для оценки.
privacy-speed-thresholds = Медленно при: 95-й процентиль загруженного изображения { $slowMiss }, 95-й процентиль изображения из кэша { $slowHit } или { $slowFail } % ответов «занято», тайм-аутов и ошибок. Ухудшено при: 95-й процентиль загруженного изображения { $degradedMiss } или { $degradedFail } % ошибок. Пределы фиксированные, не настройки.
privacy-speed-median = Медиана
privacy-speed-p95 = 95-й процентиль
privacy-speed-p99 = 99-й процентиль
privacy-speed-max = Самый медленный
privacy-speed-requests = Измерено запросов
privacy-speed-busy = Ответы «занято»
privacy-speed-timeouts = Тайм-ауты
privacy-speed-upstream-errors = Ошибки источника
privacy-speed-hit = Попадания в кэш
privacy-speed-miss = Загруженные изображения (промахи кэша)
privacy-speed-split-count = { $count } запросов
privacy-speed-split-times = Медиана { $p50 }, 95-й процентиль { $p95 }, самый медленный { $max }.
privacy-speed-chart = Время отдачи изображения
privacy-speed-chart-miss = Время отдачи загруженного изображения
privacy-speed-chart-unit = От получения запроса до готовности ответа
privacy-speed-phases = На что уходит время
privacy-speed-phases-help = Для запросов, не попавших в кэш: сколько длился каждый шаг и какую долю он занимает в общем времени.
privacy-speed-phase = Шаг
privacy-speed-share = Доля времени
privacy-speed-phase-queue = Ожидание ресурсов
privacy-speed-phase-cache = Поиск в кэше
privacy-speed-phase-upstream = Загрузка из источника
privacy-speed-phase-decode = Декодирование и повторное кодирование
privacy-speed-phase-other = Всё остальное
privacy-speed-note = Измерено только в процессе этого сервера, хранится в памяти и после перезапуска пусто. Процентили точны с погрешностью до { $error } %. URL, хосты и учётные записи не сохраняются.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = Объединенные выборки и приоритетные запросы

privacy-work-coalesced-series = Захваты объединены с одним в полете

privacy-work-handle = Обработка частоты попадания в кеш

privacy-work-handle-series = Обработка попаданий в кэш

privacy-work-per-interval = За интервал выборки

privacy-work-percent = Доля запросов дескрипторов

privacy-work-priority-series = Приоритетные запросы

privacy-work-resized = Изображения уменьшены

privacy-work-resized-series = Изменён размер

privacy-work-revalidated-series = повторно подтверждено

privacy-work-saved = Байты, сэкономленные за счет уменьшения масштаба

privacy-work-saved-series = Сохранено байт

privacy-work-stale = Устаревшие изображения подаются и проходят повторную проверку.

privacy-work-stale-series = Подается несвежим

privacy-work-title = Работу избегают и меняют ее форму
