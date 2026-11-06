# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = Не знайдено (404) за інтервалами
admin-history-nf-total = Не знайдено (404) за цей період
admin-history-nf-share = { $share } запитів
admin-history-nf-not-tracked-window = Протягом цього періоду помилки 404 не відстежувалися.
admin-history-nf-not-tracked-before = Помилки 404 не відстежувалися до { $time }. Раніші періоди показано як недоступні, а не як нуль.
admin-history-nf-paths-title = Найчастіші шляхи без результату
admin-history-nf-paths-help = До { $limit } шляхів на годину, з { $from } по { $to }, із округленням до повних годин. Рядки запиту видаляються, а адреси електронної пошти, числа, ідентифікатори, токени та довгі значення замінюються заповнювачами, наприклад {"{"}token{"}"}. Очікувані промахи самого застосунку, як-от запити значка браузера, не враховуються.
admin-history-nf-paths-approximate = Надійшло більше різних шляхів, ніж зберігається, тому деякі значення подано діапазоном, а не точним числом.
admin-history-nf-paths-empty = Відповідей «не знайдено» за цей період немає.
admin-history-nf-paths-error = Не вдалося завантажити шляхи без результату. Оновіть сторінку, щоб повторити спробу.
admin-history-nf-paths-method = Метод
admin-history-nf-paths-path = Шлях
admin-history-nf-paths-count = Кількість
admin-history-nf-paths-range = від { $min } до { $max }
admin-history-nf-paths-client = Від власних клієнтів застосунку
admin-history-nf-paths-client-help = Шляхи, які запитують вебзастосунок, його service worker, а також настільні й мобільні застосунки: файли збірки, скрипт воркера, API, JMAP, DAV і маршрути консолі. Промах тут зазвичай означає застарілого клієнта або хибний маршрут.
admin-history-nf-paths-external = Інший трафік
admin-history-nf-paths-external-help = Шляхи, яких застосунок ніколи не запитує, наприклад сканери, що шукають PHP-файли, файли з крапкою чи інше ПЗ, а також посилання з інших сайтів.
admin-history-nf-paths-all = Шляхи
admin-history-nf-paths-all-help = Цей сервер не відрізняє шляхи клієнтів від іншого трафіку.
