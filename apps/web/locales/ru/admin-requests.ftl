# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = Не найдено (404) по интервалам
admin-history-nf-total = Не найдено (404) за этот период
admin-history-nf-share = { $share } запросов
admin-history-nf-not-tracked-window = В этот период ошибки 404 не отслеживались.
admin-history-nf-not-tracked-before = Ошибки 404 не отслеживались до { $time }. Более ранние периоды показаны как недоступные, а не как ноль.
admin-history-nf-paths-title = Самые частые пути без результата
admin-history-nf-paths-help = До { $limit } путей в час, с { $from } по { $to }, с округлением до целых часов. Строки запроса удаляются, а адреса электронной почты, числа, идентификаторы, токены и длинные значения заменяются заполнителями, например {"{"}token{"}"}. Ожидаемые промахи самого приложения, такие как запросы значка браузера, не учитываются.
admin-history-nf-paths-approximate = Поступило больше различных путей, чем хранится, поэтому некоторые значения указаны диапазоном, а не точным числом.
admin-history-nf-paths-empty = Ответов «не найдено» за этот период нет.
admin-history-nf-paths-error = Не удалось загрузить пути без результата. Обновите страницу, чтобы повторить попытку.
admin-history-nf-paths-method = Метод
admin-history-nf-paths-path = Путь
admin-history-nf-paths-count = Количество
admin-history-nf-paths-range = от { $min } до { $max }
admin-history-nf-paths-client = От собственных клиентов приложения
admin-history-nf-paths-client-help = Пути, которые запрашивают веб-приложение, его service worker, а также настольные и мобильные приложения: файлы сборки, скрипт воркера, API, JMAP, DAV и маршруты консоли. Промах здесь обычно означает устаревший клиент или неверный маршрут.
admin-history-nf-paths-external = Прочий трафик
admin-history-nf-paths-external-help = Пути, которые приложение никогда не запрашивает, например сканеры, ищущие PHP-файлы, файлы с точкой или другое ПО, а также ссылки с других сайтов.
admin-history-nf-paths-all = Пути
admin-history-nf-paths-all-help = Этот сервер не отличает пути клиентов от прочего трафика.
