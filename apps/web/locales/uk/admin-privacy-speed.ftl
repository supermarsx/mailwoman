# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Швидкість
privacy-speed-unavailable = Дані про швидкість із цього сервера недоступні.
privacy-speed-empty = За цей період зображення ще не віддавалися.
privacy-speed-off = Відстеження швидкості вимкнено (налаштування: Відстеження швидкості проксі зображень). Нові дані не записуються.
privacy-speed-verdict-healthy = У нормі
privacy-speed-verdict-slow = Повільно
privacy-speed-verdict-degraded = Погіршено
privacy-speed-verdict-noData = Недостатньо даних
privacy-speed-reason-ok = Завантажені зображення та влучання в кеш вкладаються в зазначені нижче межі, а помилок мало.
privacy-speed-reason-failureRate = { $failurePct } % запитів завершилися відповіддю «зайнято», тайм-аутом або помилкою джерела.
privacy-speed-reason-missP95 = Зображення, які довелося завантажувати, віддаються повільно: їхній 95-й процентиль вищий за зазначену межу.
privacy-speed-reason-hitP95 = Зображення з кешу віддаються повільно: їхній 95-й процентиль вищий за зазначену межу.
privacy-speed-reason-fewRequests = Менше ніж { $min } запитів за період, цього недостатньо для оцінки.
privacy-speed-thresholds = Повільно за: 95-й процентиль завантаженого зображення { $slowMiss }, 95-й процентиль зображення з кешу { $slowHit } або { $slowFail } % відповідей «зайнято», тайм-аутів і помилок. Погіршено за: 95-й процентиль завантаженого зображення { $degradedMiss } або { $degradedFail } % помилок. Межі фіксовані, не налаштування.
privacy-speed-median = Медіана
privacy-speed-p95 = 95-й процентиль
privacy-speed-p99 = 99-й процентиль
privacy-speed-max = Найповільніший
privacy-speed-requests = Виміряно запитів
privacy-speed-busy = Відповіді «зайнято»
privacy-speed-timeouts = Тайм-аути
privacy-speed-upstream-errors = Помилки джерела
privacy-speed-hit = Влучання в кеш
privacy-speed-miss = Завантажені зображення (промахи кешу)
privacy-speed-split-count = { $count } запитів
privacy-speed-split-times = Медіана { $p50 }, 95-й процентиль { $p95 }, найповільніший { $max }.
privacy-speed-chart = Час віддачі зображення
privacy-speed-chart-miss = Час віддачі завантаженого зображення
privacy-speed-chart-unit = Від отримання запиту до готовності відповіді
privacy-speed-phases = На що йде час
privacy-speed-phases-help = Для запитів, що не влучили в кеш: скільки тривав кожен крок і яку частку він займає в загальному часі.
privacy-speed-phase = Крок
privacy-speed-share = Частка часу
privacy-speed-phase-queue = Очікування ресурсів
privacy-speed-phase-cache = Пошук у кеші
privacy-speed-phase-upstream = Завантаження з джерела
privacy-speed-phase-decode = Декодування та повторне кодування
privacy-speed-phase-other = Усе інше
privacy-speed-note = Виміряно лише в процесі цього сервера, зберігається в пам'яті й після перезапуску порожнє. Процентилі точні з похибкою до { $error } %. URL, хости й облікові записи не зберігаються.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = Об’єднані вибірки та пріоритетні запити

privacy-work-coalesced-series = Збірки об’єднані в одну в польоті

privacy-work-handle = Обробка частоти звернень до кешу

privacy-work-handle-series = Обробляти звернення до кешу

privacy-work-per-interval = За інтервал вибірки

privacy-work-percent = Частка пошуків дескрипторів

privacy-work-priority-series = Пріоритетні запити

privacy-work-resized = Зображення зменшено

privacy-work-resized-series = Змінено розмір

privacy-work-revalidated-series = Перевірено

privacy-work-saved = Байти, збережені шляхом зменшення масштабу

privacy-work-saved-series = Байти збережено

privacy-work-stale = Застарілі зображення надано та повторно перевірено

privacy-work-stale-series = Подається несвіжим

privacy-work-title = Роботи уникали та переформовували
