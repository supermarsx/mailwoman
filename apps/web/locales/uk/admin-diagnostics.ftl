# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Сторінки діагностики
admin-diagx-tab-overview = Огляд
admin-diagx-tab-server = Сервер
admin-diagx-tab-connectivity = Зв'язність
admin-diagx-tab-backends = Поштові бекенди
admin-diagx-tab-posture = Стан безпеки
admin-diagx-tab-tools = Інструменти
admin-diagx-tab-history = Історія
admin-diagx-tab-report = Звіт
admin-diagx-tab-server-intro = Перевірки власних частин цього сервера: бази даних, її міграцій і ключа сервера. Виконуються разом із перевірками сервера.
admin-diagx-tab-connectivity-intro = Чи доступний цей сервер під своєю публічною назвою, чи може він дістатися до збережених бекендів і чи лишаються приватні адреси недосяжними. Інструменти доменів нижче перевіряють записи DNS, тестують TLS і визначають налаштування для одного домену.
admin-diagx-tab-backends-intro = З'єднання та узгодження захисту з кожною збереженою кінцевою точкою пошти, надсилання, правил і DAV. Облікові дані не надсилаються, пошта не доставляється.
admin-diagx-tab-posture-intro = Налаштування, що захищають входи та сеанси, і те, як fail2ban побачив би адресу запиту.
admin-diagx-tab-tools-intro = Інші місця для перевірки служби та обмеження, що діють для діагностичних запусків.
admin-diagx-tab-history-intro = Збережені запуски, спершу нові. Відкрийте запуск, щоб побачити його результати та що змінилося від попереднього запуску.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Попередження
admin-diagx-status-fail = Збій
admin-diagx-status-skipped = Пропущено
admin-diagx-status-not-run = Не запускалося
admin-diagx-status-running = Виконується
admin-diagx-run = Запустити
admin-diagx-run-again = Запустити знову
admin-diagx-running = Виконується…
admin-diagx-run-server-note = Знову виконує перевірки сервера. Кожна картка з цього запуску оновлюється.
admin-diagx-run-backends-note = Знову перевіряє всі збережені бекенди. Кожна картка бекенда оновлюється.
admin-diagx-how-to-fix = Як виправити
admin-diagx-details = Подробиці
admin-diagx-copy-details = Копіювати подробиці
admin-diagx-copy-value = Копіювати: { $label }
admin-diagx-copied = Скопійовано.
admin-diagx-copy-failed = Браузер не дозволив копіювання. Виділіть текст і скопіюйте його вручну.
admin-diagx-not-run-yet = Ще не запускалося.
admin-diagx-not-reported = Останній запуск сервера не повідомив про цю перевірку; вона не стосується цього розгортання.
admin-diagx-posture-met = Виконано
admin-diagx-posture-not-met = Не виконано
admin-diagx-loading = Завантаження…
admin-diagx-none = Немає
admin-diagx-yes = Так
admin-diagx-no = Ні
admin-diagx-open = Відкрити
admin-diagx-row-result = Результат
admin-diagx-row-duration = Тривалість
admin-diagx-row-checked = Перевірено
admin-diagx-row-id = Ідентифікатор перевірки
admin-diagx-row-domain = Домен
admin-diagx-row-backend = Бекенд
admin-diagx-row-protocol = Протокол
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Можливості
admin-diagx-row-credentials = Надіслано облікові дані
admin-diagx-row-source = Знайдено в
admin-diagx-row-oauth = Потрібен OAuth
admin-diagx-overview-intro = Останній результат кожної перевірки за групами. «Запустити всі» виконує перевірки сервера, потім перевіряє кожен збережений бекенд.
admin-diagx-run-all = Запустити всі перевірки
admin-diagx-run-all-step = Крок { $step } із { $steps }: { $suite ->
    [backends] збережені бекенди
   *[server] перевірки сервера
}.
admin-diagx-run-all-done = Усі перевірки завершено. Результати збережено в Історії.
admin-diagx-backends-progress = Перевірено бекендів: { $completed } із { $total }.
admin-diagx-counts = { $ok } OK · { $warn } попереджень · { $fail } збоїв · { $notRun } не запускалося
admin-diagx-attention = Потребує уваги
admin-diagx-attention-none = Жодна перевірка не повідомила про попередження чи збій в останньому запуску.
admin-diagx-never-run = Перевірки ще не запускалися. Запустіть усі перевірки, щоб отримати перший результат.
admin-diagx-last-server = Останні перевірки сервера
admin-diagx-last-backends = Останні перевірки бекендів
admin-diagx-never = Ніколи
admin-diagx-open-health = Стан служби
admin-diagx-check-database-connectivity = З'єднання з базою даних
admin-diagx-check-database-connectivity-purpose = База даних відповідає на простий запит.
admin-diagx-check-database-connectivity-fix = Перевірте, що база даних запущена і що MW_DB_PATH або MW_DATABASE_URL вказує на неї. Помилка з'єднання зазначена в журналі сервера.
admin-diagx-check-database-write = Запис до бази даних
admin-diagx-check-database-write-purpose = Пробний рядок записується, потім відкочується.
admin-diagx-check-database-write-fix = Перевірте вільне місце на диску та права запису у файла бази даних чи користувача. Том лише для читання або заповнений диск зупиняють запис.
admin-diagx-check-database-migrations = Міграції бази даних
admin-diagx-check-database-migrations-purpose = У базі даних є всі міграції, яких очікує ця версія сервера.
admin-diagx-check-database-migrations-fix = Перезапустіть сервер, щоб він застосував очікувані міграції, і перевірте в журналі, чи не завершилася якась міграція збоєм. Не запускайте старий сервер на новішій базі даних.
admin-diagx-check-server-key-roundtrip = Ключ сервера
admin-diagx-check-server-key-roundtrip-purpose = Ключ сервера запечатує й відкриває тестове значення.
admin-diagx-check-server-key-roundtrip-fix = Перевірте, що MW_SERVER_KEY (або файл ключа) — це ключ, з яким створено цю базу даних. Збережені секрети не можна відкрити іншим ключем.
admin-diagx-check-other-purpose = Перевірка, про яку повідомив цей сервер.
admin-diagx-check-other-fix = Відкрийте «Подробиці», щоб побачити необроблений результат, і перевірте журнал сервера на момент запуску.
admin-diagx-check-network-public-dns = Публічна назва
admin-diagx-check-network-public-dns-purpose = Ім'я хоста з MW_PUBLIC_URL розв'язується з цього сервера.
admin-diagx-check-network-public-dns-fix = Перевірте запис DNS публічної назви або задайте MW_PUBLIC_URL рівним адресі, за якою люди звертаються до цього сервера.
admin-diagx-check-network-public-port = Публічний порт
admin-diagx-check-network-public-port-purpose = Порт публічного URL приймає з'єднання з цього сервера.
admin-diagx-check-network-public-port-fix = Перевірте брандмауер, зворотний проксі та переспрямування портів. Деякі маршрутизатори не можуть підключитися до власної публічної адреси (NAT hairpinning); перевірте також ззовні вашої мережі.
admin-diagx-check-network-backend-ports = Порти бекендів
admin-diagx-check-network-backend-ports-purpose = Хост і порт кожного збереженого бекенда приймають пряме з'єднання.
admin-diagx-check-network-backend-ports-fix = Перевірте, що хости бекендів працюють і що брандмауер дозволяє цьому серверу зв'язуватися з ними. «Поштові бекенди» показують, яка кінцева точка не проходить узгодження.
admin-diagx-check-egress-private-policy = Захист приватних адрес
admin-diagx-check-egress-private-policy-purpose = Типово вихідні з'єднання з loopback і приватними адресами відхиляються.
admin-diagx-check-egress-private-policy-fix = Перегляньте налаштування вихідного трафіку в розділі «Конфіденційність». Дозволяйте приватні адреси лише для названих бекендів, яким вони потрібні.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Запити доходять до цього сервера через HTTPS.
admin-diagx-check-security-https-fix = Надавайте консоль через HTTPS або задайте MW_PUBLIC_URL з https і MW_TRUSTED_PROXIES, коли TLS завершує зворотний проксі.
admin-diagx-check-security-secure-cookies = Захищені cookie
admin-diagx-check-security-secure-cookies-purpose = Cookie сеансу надсилаються лише через HTTPS.
admin-diagx-check-security-secure-cookies-fix = Задайте MW_COOKIE_SECURE рівним true, коли сервер доступний через HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Браузерам повідомляється використовувати HTTPS для цього сайту.
admin-diagx-check-security-hsts-fix = Увімкніть Strict-Transport-Security в налаштуваннях безпеки або надсилайте його зі зворотного проксі.
admin-diagx-check-security-csrf = Захист CSRF
admin-diagx-check-security-csrf-purpose = Запити, що змінюють дані, мусять нести токен CSRF.
admin-diagx-check-security-csrf-fix = Задайте MW_CSRF_STRICT рівним true.
admin-diagx-check-security-admin-factors = Другий чинник адміністратора
admin-diagx-check-security-admin-factors-purpose = Кожен адміністратор має другий чинник входу.
admin-diagx-check-security-admin-factors-fix = Попросіть кожного адміністратора без нього додати ключ доступу або застосунок-автентифікатор у «Моїх налаштуваннях» чи вимагайте цього в Центрі безпеки.
admin-diagx-check-security-signin-pacing = Затримка повторного входу
admin-diagx-check-security-signin-pacing-purpose = Після невдалого входу настає коротке очікування.
admin-diagx-check-security-signin-pacing-fix = Увімкніть сповільнення входу в політиці безпеки.
admin-diagx-check-security-credential-locks = Блокування облікових даних
admin-diagx-check-security-credential-locks-purpose = Повторні невдалі входи на деякий час блокують облікові дані.
admin-diagx-check-security-credential-locks-fix = Увімкніть автоматичні блокування в Центрі безпеки.
admin-diagx-check-security-geography = Дані географії входів
admin-diagx-check-security-geography-purpose = Політика географії входів має потрібні дані про країни.
admin-diagx-check-security-geography-fix = Установіть або оновіть базу країн чи вимкніть політику географії.
admin-diagx-check-security-render-isolation = Ізольоване показування повідомлень
admin-diagx-check-security-render-isolation-purpose = Повідомлення відображаються в окремому ізольованому робочому процесі.
admin-diagx-check-security-render-isolation-fix = Перевірте, що MW_RENDER_BIN вказує на програму mw-render і що вона запускається. Стан служби на Панелі показує стан робочого процесу.
admin-diagx-check-security-deployment = Налаштування розгортання
admin-diagx-check-security-deployment-purpose = Налаштування безпеки розгортання можна прочитати.
admin-diagx-check-security-deployment-fix = Перевірте з'єднання з базою даних, потім відкрийте оцінку безпеки розгортання.
admin-diagx-check-security-plaintext-backends = Паролі через незашифровані з'єднання
admin-diagx-check-security-plaintext-backends-purpose = Жоден збережений бекенд не приймає паролів без TLS.
admin-diagx-check-security-plaintext-backends-fix = Змініть з'єднання домену й виберіть TLS або STARTTLS. Дозволяйте звичайні з'єднання лише в приватній мережі, яку ви контролюєте.
admin-diagx-check-fail2ban-client-address = Адреса клієнта для fail2ban
admin-diagx-check-fail2ban-client-address-purpose = Адреса, яку назвав би рядок журналу для fail2ban, — справжня адреса клієнта.
admin-diagx-check-fail2ban-client-address-fix = За зворотним проксі задайте MW_TRUSTED_PROXIES рівним адресі проксі, інакше fail2ban заблокує проксі замість клієнта. Адреса loopback очікувана, коли ви відкриваєте консоль на самому сервері.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Кінцева точка { $protocol }: з'єднання та узгодження захисту, без входу.
admin-diagx-backend-other-purpose = Примітка із запуску бекендів.
admin-diagx-backend-fix = Перевірте ім'я хоста, порт і режим TLS, збережені для цього бекенда, чи розв'язується DNS і чи чинний сертифікат для цього імені хоста. Потім запустіть знову.
admin-diagx-backend-config-fix = Відкрийте з'єднання домену й збережіть їх знову; список збережених бекендів прочитати не вдалося.
admin-diagx-backend-results = Перевірки бекендів
admin-diagx-backends-not-run = Збережені бекенди ще не перевірялися.
admin-diagx-backends-run-at = Остання перевірка { $date }: результатів { $count }.
admin-diagx-failover-title = Аварійне перемикання
admin-diagx-failover-purpose = Чи може домен перемкнутися на інший поштовий бекенд, коли перший вибір недоступний.
admin-diagx-failover-state = Аварійне перемикання { $state ->
    [on] увімкнено
   *[off] вимкнено
} · поштових бекендів: { $backends }{ $enabled ->
    [no] {" "}· домен вимкнено
   *[yes] {""}
}
admin-diagx-no-domains = Збережених доменів немає.
admin-diagx-domain-tools = Інструменти доменів
admin-diagx-domain-tools-purpose = Одноразові перевірки збереженого домену. Вони не виконують вхід і не надсилають пошту.
admin-diagx-domain = Домен
admin-diagx-tool-dns = Записи DNS
admin-diagx-tool-dns-purpose = Перевіряє записи MX, SPF, DMARC і служб (SRV) домену.
admin-diagx-tool-dns-fix = Додайте відсутні записи в свого DNS-провайдера. Без MX домен не отримує пошти; без SPF і DMARC інші сервери можуть відхиляти його пошту.
admin-diagx-srv-found = Знайдено записів служб: { $found } із { $total }
admin-diagx-tool-tls = TLS і узгодження
admin-diagx-tool-tls-purpose = Підключається до кожної збереженої кінцевої точки домену й повідомляє версію TLS, строк дії сертифіката та витрачений час.
admin-diagx-tool-tls-fix = Для недоступної кінцевої точки перевірте хост, порт і брандмауер. Оновіть сертифікат, що спливає протягом 14 днів.
admin-diagx-probe-none = У домену немає збережених кінцевих точок для перевірки.
admin-diagx-tool-detect = Визначити налаштування
admin-diagx-tool-detect-purpose = Шукає опубліковані поштові налаштування (autoconfig, autodiscover, SRV) для назви домену. Нічого не зберігається.
admin-diagx-tool-detect-fix = Опублікуйте записи autoconfig або SRV для домену чи введіть налаштування вручну в розділі «Домени».
admin-diagx-detect-name = Назва домену для пошуку
admin-diagx-tool-health = Стан служби
admin-diagx-tool-health-purpose = Синхронізація пошти, вихідні, пошуковий індекс, робочий процес відображення та плагіни — на Панелі.
admin-diagx-tool-deployment-purpose = Кожне налаштування безпеки розгортання, яке може спостерігати цей сервер, із налаштуванням, що ним керує.
admin-diagx-tool-ssh = Перевірки SSH-тунелів
admin-diagx-tool-ssh-purpose = Перевірка SSH-тунелів, що використовуються для доступу до бекендів в іншій мережі.
admin-diagx-tool-geo = Географія входів
admin-diagx-tool-geo-purpose = Звідки надходять входи та база країн, на якій ґрунтується політика географії.
admin-diagx-catalogue-purpose = Що охоплює кожен діагностичний набір і яких обмежень дотримується кожен запуск.
admin-diagx-suite-unsupported = Недоступно: { $reason }
admin-diagx-suite-server = Перевірки сервера
admin-diagx-suite-backends = Збережені бекенди
admin-diagx-run-title = { $suite ->
    [backends] Збережені бекенди
   *[server] Перевірки сервера
}, { $date }
admin-diagx-changes = Зміни
admin-diagx-changes-first = Раніших запусків такого виду для порівняння немає.
admin-diagx-changes-none = Від запуску за { $date } нічого не змінилося.
admin-diagx-changes-since = Змінилося від запуску за { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Звіт для усунення несправностей
admin-diagx-report-purpose = Звіт JSON для підтримки: прапорці середовища виконання, останні результати та вибраний вами запуск. Паролі, токени, cookie, ключі, необроблена конфігурація та пошта ніколи не включаються.
admin-diagx-report-run = Запуск для включення
admin-diagx-report-preview = Перегляд
admin-crumb-server = Сервер
admin-crumb-connectivity = Зв'язність
admin-crumb-posture = Стан безпеки
admin-crumb-tools = Інструменти
admin-crumb-history = Історія
admin-crumb-report = Звіт
admin-diagx-table-actions = Дії
admin-diagx-check-pdfjs-viewer-assets = Файли переглядача PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = Усі файли переглядача PDF на місці, мають правильний тип і відповідають цій збірці сервера.
admin-diagx-check-pdfjs-viewer-assets-fix = Розгорніть вебзбірку знову або видаліть теку pdfjs із MW_WEB_DIR, щоб використовувалася копія, вбудована в сервер. Якщо це завершується збоєм, а не попередженням, перезберіть образ сервера.

# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-probe-line = { $host } · доступний: { $reachable ->
    [yes] так
   *[no] ні
} · TLS { $tls } · сертифікат { $days ->
    [none] не повідомлено
    [one] спливає через { $days } день
    [few] спливає через { $days } дні
    [many] спливає через { $days } днів
   *[other] спливає через { $days } дня
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
