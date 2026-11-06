# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Страницы диагностики
admin-diagx-tab-overview = Обзор
admin-diagx-tab-server = Сервер
admin-diagx-tab-connectivity = Связность
admin-diagx-tab-backends = Почтовые бэкенды
admin-diagx-tab-posture = Состояние безопасности
admin-diagx-tab-tools = Инструменты
admin-diagx-tab-history = История
admin-diagx-tab-report = Отчёт
admin-diagx-tab-server-intro = Проверки собственных частей этого сервера: базы данных, её миграций и ключа сервера. Выполняются вместе с проверками сервера.
admin-diagx-tab-connectivity-intro = Доступен ли этот сервер под своим публичным именем, может ли он достучаться до сохранённых бэкендов и остаются ли частные адреса недоступными. Инструменты доменов ниже проверяют записи DNS, тестируют TLS и определяют настройки для одного домена.
admin-diagx-tab-backends-intro = Соединение и согласование защиты с каждой сохранённой конечной точкой почты, отправки, правил и DAV. Учётные данные не отправляются, почта не доставляется.
admin-diagx-tab-posture-intro = Настройки, защищающие входы и сеансы, и то, как fail2ban увидел бы адрес запроса.
admin-diagx-tab-tools-intro = Другие места для проверки службы и ограничения, действующие для диагностических запусков.
admin-diagx-tab-history-intro = Сохранённые запуски, сначала новые. Откройте запуск, чтобы увидеть его результаты и что изменилось с предыдущего запуска.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Предупреждение
admin-diagx-status-fail = Сбой
admin-diagx-status-skipped = Пропущено
admin-diagx-status-not-run = Не запускалось
admin-diagx-status-running = Выполняется
admin-diagx-run = Запустить
admin-diagx-run-again = Запустить снова
admin-diagx-running = Выполняется…
admin-diagx-run-server-note = Снова выполняет проверки сервера. Каждая карточка из этого запуска обновляется.
admin-diagx-run-backends-note = Снова проверяет все сохранённые бэкенды. Каждая карточка бэкенда обновляется.
admin-diagx-how-to-fix = Как исправить
admin-diagx-details = Подробности
admin-diagx-copy-details = Копировать подробности
admin-diagx-copy-value = Копировать: { $label }
admin-diagx-copied = Скопировано.
admin-diagx-copy-failed = Браузер не разрешил копирование. Выделите текст и скопируйте его вручную.
admin-diagx-not-run-yet = Ещё не запускалось.
admin-diagx-not-reported = Последний запуск сервера не сообщил об этой проверке; она не относится к этому развёртыванию.
admin-diagx-posture-met = Выполнено
admin-diagx-posture-not-met = Не выполнено
admin-diagx-loading = Загрузка…
admin-diagx-none = Нет
admin-diagx-yes = Да
admin-diagx-no = Нет
admin-diagx-open = Открыть
admin-diagx-row-result = Результат
admin-diagx-row-duration = Длительность
admin-diagx-row-checked = Проверено
admin-diagx-row-id = Идентификатор проверки
admin-diagx-row-domain = Домен
admin-diagx-row-backend = Бэкенд
admin-diagx-row-protocol = Протокол
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Возможности
admin-diagx-row-credentials = Отправлены учётные данные
admin-diagx-row-source = Найдено в
admin-diagx-row-oauth = Требуется OAuth
admin-diagx-overview-intro = Последний результат каждой проверки по группам. «Запустить все» выполняет проверки сервера, затем проверяет каждый сохранённый бэкенд.
admin-diagx-run-all = Запустить все проверки
admin-diagx-run-all-step = Шаг { $step } из { $steps }: { $suite ->
    [backends] сохранённые бэкенды
   *[server] проверки сервера
}.
admin-diagx-run-all-done = Все проверки завершены. Результаты сохранены в Истории.
admin-diagx-backends-progress = Проверено бэкендов: { $completed } из { $total }.
admin-diagx-counts = { $ok } OK · { $warn } предупреждений · { $fail } сбоев · { $notRun } не запускалось
admin-diagx-attention = Требует внимания
admin-diagx-attention-none = Ни одна проверка не сообщила о предупреждении или сбое в последнем запуске.
admin-diagx-never-run = Проверки ещё не запускались. Запустите все проверки, чтобы получить первый результат.
admin-diagx-last-server = Последние проверки сервера
admin-diagx-last-backends = Последние проверки бэкендов
admin-diagx-never = Никогда
admin-diagx-open-health = Состояние службы
admin-diagx-check-database-connectivity = Соединение с базой данных
admin-diagx-check-database-connectivity-purpose = База данных отвечает на простой запрос.
admin-diagx-check-database-connectivity-fix = Проверьте, что база данных запущена и что MW_DB_PATH или MW_DATABASE_URL указывает на неё. Ошибка соединения указана в журнале сервера.
admin-diagx-check-database-write = Запись в базу данных
admin-diagx-check-database-write-purpose = Пробная строка записывается, затем откатывается.
admin-diagx-check-database-write-fix = Проверьте свободное место на диске и права записи у файла базы данных или пользователя. Том только для чтения или заполненный диск останавливают запись.
admin-diagx-check-database-migrations = Миграции базы данных
admin-diagx-check-database-migrations-purpose = В базе данных есть все миграции, которых ожидает эта версия сервера.
admin-diagx-check-database-migrations-fix = Перезапустите сервер, чтобы он применил ожидающие миграции, и проверьте в журнале, не завершилась ли какая-либо миграция сбоем. Не запускайте старый сервер на более новой базе данных.
admin-diagx-check-server-key-roundtrip = Ключ сервера
admin-diagx-check-server-key-roundtrip-purpose = Ключ сервера запечатывает и открывает тестовое значение.
admin-diagx-check-server-key-roundtrip-fix = Проверьте, что MW_SERVER_KEY (или файл ключа) — тот ключ, с которым создана эта база данных. Сохранённые секреты нельзя открыть другим ключом.
admin-diagx-check-other-purpose = Проверка, о которой сообщил этот сервер.
admin-diagx-check-other-fix = Откройте «Подробности», чтобы увидеть необработанный результат, и проверьте журнал сервера на момент запуска.
admin-diagx-check-network-public-dns = Публичное имя
admin-diagx-check-network-public-dns-purpose = Имя хоста из MW_PUBLIC_URL разрешается с этого сервера.
admin-diagx-check-network-public-dns-fix = Проверьте запись DNS публичного имени или задайте MW_PUBLIC_URL равным адресу, по которому люди обращаются к этому серверу.
admin-diagx-check-network-public-port = Публичный порт
admin-diagx-check-network-public-port-purpose = Порт публичного URL принимает соединение с этого сервера.
admin-diagx-check-network-public-port-fix = Проверьте файрвол, обратный прокси и проброс портов. Некоторые маршрутизаторы не могут подключиться к собственному публичному адресу (NAT hairpinning); проверьте также из-за пределов вашей сети.
admin-diagx-check-network-backend-ports = Порты бэкендов
admin-diagx-check-network-backend-ports-purpose = Хост и порт каждого сохранённого бэкенда принимают прямое соединение.
admin-diagx-check-network-backend-ports-fix = Проверьте, что хосты бэкендов работают и что файрвол разрешает этому серверу связываться с ними. «Почтовые бэкенды» показывают, какая конечная точка не проходит согласование.
admin-diagx-check-egress-private-policy = Защита частных адресов
admin-diagx-check-egress-private-policy-purpose = По умолчанию исходящие соединения с loopback и частными адресами отклоняются.
admin-diagx-check-egress-private-policy-fix = Проверьте настройки исходящего трафика в разделе «Конфиденциальность». Разрешайте частные адреса только для названных бэкендов, которым они нужны.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Запросы доходят до этого сервера по HTTPS.
admin-diagx-check-security-https-fix = Предоставляйте консоль по HTTPS или задайте MW_PUBLIC_URL с https и MW_TRUSTED_PROXIES, когда TLS завершает обратный прокси.
admin-diagx-check-security-secure-cookies = Защищённые cookie
admin-diagx-check-security-secure-cookies-purpose = Cookie сеанса отправляются только по HTTPS.
admin-diagx-check-security-secure-cookies-fix = Задайте MW_COOKIE_SECURE равным true, когда сервер доступен по HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Браузерам сообщается использовать HTTPS для этого сайта.
admin-diagx-check-security-hsts-fix = Включите Strict-Transport-Security в настройках безопасности или отправляйте его с обратного прокси.
admin-diagx-check-security-csrf = Защита CSRF
admin-diagx-check-security-csrf-purpose = Запросы, изменяющие данные, должны нести токен CSRF.
admin-diagx-check-security-csrf-fix = Задайте MW_CSRF_STRICT равным true.
admin-diagx-check-security-admin-factors = Второй фактор администратора
admin-diagx-check-security-admin-factors-purpose = У каждого администратора есть второй фактор входа.
admin-diagx-check-security-admin-factors-fix = Попросите каждого администратора без него добавить ключ доступа или приложение-аутентификатор в «Моих настройках» либо потребуйте этого в Центре безопасности.
admin-diagx-check-security-signin-pacing = Задержка повторного входа
admin-diagx-check-security-signin-pacing-purpose = После неудачного входа следует короткое ожидание.
admin-diagx-check-security-signin-pacing-fix = Включите замедление входа в политике безопасности.
admin-diagx-check-security-credential-locks = Блокировки учётных данных
admin-diagx-check-security-credential-locks-purpose = Повторные неудачные входы на время блокируют учётные данные.
admin-diagx-check-security-credential-locks-fix = Включите автоматические блокировки в Центре безопасности.
admin-diagx-check-security-geography = Данные географии входов
admin-diagx-check-security-geography-purpose = У политики географии входов есть нужные данные о странах.
admin-diagx-check-security-geography-fix = Установите или обновите базу стран либо выключите политику географии.
admin-diagx-check-security-render-isolation = Изолированный показ сообщений
admin-diagx-check-security-render-isolation-purpose = Сообщения отображаются в отдельном изолированном рабочем процессе.
admin-diagx-check-security-render-isolation-fix = Проверьте, что MW_RENDER_BIN указывает на программу mw-render и что она запускается. Состояние службы на Панели показывает состояние рабочего процесса.
admin-diagx-check-security-deployment = Настройки развёртывания
admin-diagx-check-security-deployment-purpose = Настройки безопасности развёртывания можно прочитать.
admin-diagx-check-security-deployment-fix = Проверьте соединение с базой данных, затем откройте оценку безопасности развёртывания.
admin-diagx-check-security-plaintext-backends = Пароли по незашифрованным соединениям
admin-diagx-check-security-plaintext-backends-purpose = Ни один сохранённый бэкенд не принимает пароли без TLS.
admin-diagx-check-security-plaintext-backends-fix = Измените соединения домена и выберите TLS или STARTTLS. Разрешайте обычные соединения только в частной сети, которую вы контролируете.
admin-diagx-check-fail2ban-client-address = Адрес клиента для fail2ban
admin-diagx-check-fail2ban-client-address-purpose = Адрес, который назвала бы строка журнала для fail2ban, — настоящий адрес клиента.
admin-diagx-check-fail2ban-client-address-fix = За обратным прокси задайте MW_TRUSTED_PROXIES равным адресу прокси, иначе fail2ban заблокирует прокси вместо клиента. Адрес loopback ожидаем, когда вы открываете консоль на самом сервере.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Конечная точка { $protocol }: соединение и согласование защиты, без входа.
admin-diagx-backend-other-purpose = Примечание из запуска бэкендов.
admin-diagx-backend-fix = Проверьте имя хоста, порт и режим TLS, сохранённые для этого бэкенда, разрешается ли DNS и действителен ли сертификат для этого имени хоста. Затем запустите снова.
admin-diagx-backend-config-fix = Откройте соединения домена и сохраните их снова; список сохранённых бэкендов прочитать не удалось.
admin-diagx-backend-results = Проверки бэкендов
admin-diagx-backends-not-run = Сохранённые бэкенды ещё не проверялись.
admin-diagx-backends-run-at = Последняя проверка { $date }: результатов { $count }.
admin-diagx-failover-title = Аварийное переключение
admin-diagx-failover-purpose = Может ли домен переключиться на другой почтовый бэкенд, когда первый выбор недоступен.
admin-diagx-failover-state = Аварийное переключение { $state ->
    [on] включено
   *[off] выключено
} · почтовых бэкендов: { $backends }{ $enabled ->
    [no] {" "}· домен отключён
   *[yes] {""}
}
admin-diagx-no-domains = Сохранённых доменов нет.
admin-diagx-domain-tools = Инструменты доменов
admin-diagx-domain-tools-purpose = Разовые проверки сохранённого домена. Они не выполняют вход и не отправляют почту.
admin-diagx-domain = Домен
admin-diagx-tool-dns = Записи DNS
admin-diagx-tool-dns-purpose = Проверяет записи MX, SPF, DMARC и служб (SRV) домена.
admin-diagx-tool-dns-fix = Добавьте недостающие записи у вашего DNS-провайдера. Без MX домен не получает почту; без SPF и DMARC другие серверы могут отклонять его почту.
admin-diagx-srv-found = Найдено записей служб: { $found } из { $total }
admin-diagx-tool-tls = TLS и согласование
admin-diagx-tool-tls-purpose = Подключается к каждой сохранённой конечной точке домена и сообщает версию TLS, срок действия сертификата и затраченное время.
admin-diagx-tool-tls-fix = Для недоступной конечной точки проверьте хост, порт и файрвол. Обновите сертификат, истекающий в течение 14 дней.
admin-diagx-probe-none = У домена нет сохранённых конечных точек для проверки.
admin-diagx-tool-detect = Определить настройки
admin-diagx-tool-detect-purpose = Ищет опубликованные почтовые настройки (autoconfig, autodiscover, SRV) для имени домена. Ничего не сохраняется.
admin-diagx-tool-detect-fix = Опубликуйте записи autoconfig или SRV для домена либо введите настройки вручную в разделе «Домены».
admin-diagx-detect-name = Имя домена для поиска
admin-diagx-tool-health = Состояние службы
admin-diagx-tool-health-purpose = Синхронизация почты, исходящие, поисковый индекс, рабочий процесс отображения и плагины — на Панели.
admin-diagx-tool-deployment-purpose = Каждая настройка безопасности развёртывания, которую может наблюдать этот сервер, с управляющей ею настройкой.
admin-diagx-tool-ssh = Проверки SSH-туннелей
admin-diagx-tool-ssh-purpose = Проверка SSH-туннелей, используемых для доступа к бэкендам в другой сети.
admin-diagx-tool-geo = География входов
admin-diagx-tool-geo-purpose = Откуда приходят входы и база стран, на которой основана политика географии.
admin-diagx-catalogue-purpose = Что охватывает каждый диагностический набор и каких ограничений придерживается каждый запуск.
admin-diagx-suite-unsupported = Недоступно: { $reason }
admin-diagx-suite-server = Проверки сервера
admin-diagx-suite-backends = Сохранённые бэкенды
admin-diagx-run-title = { $suite ->
    [backends] Сохранённые бэкенды
   *[server] Проверки сервера
}, { $date }
admin-diagx-changes = Изменения
admin-diagx-changes-first = Более раннего запуска такого вида для сравнения нет.
admin-diagx-changes-none = С запуска от { $date } ничего не изменилось.
admin-diagx-changes-since = Изменилось с запуска от { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Отчёт для устранения неполадок
admin-diagx-report-purpose = Отчёт JSON для поддержки: флаги среды выполнения, последние результаты и выбранный вами запуск. Пароли, токены, cookie, ключи, необработанная конфигурация и почта никогда не включаются.
admin-diagx-report-run = Запуск для включения
admin-diagx-report-preview = Просмотр
admin-crumb-server = Сервер
admin-crumb-connectivity = Связность
admin-crumb-posture = Состояние безопасности
admin-crumb-tools = Инструменты
admin-crumb-history = История
admin-crumb-report = Отчёт
admin-diagx-table-actions = Действия
admin-diagx-check-pdfjs-viewer-assets = Файлы просмотрщика PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = Все файлы просмотрщика PDF на месте, имеют правильный тип и соответствуют этой сборке сервера.
admin-diagx-check-pdfjs-viewer-assets-fix = Разверните веб-сборку заново или удалите папку pdfjs из MW_WEB_DIR, чтобы использовалась копия, встроенная в сервер. Если это завершается сбоем, а не предупреждением, пересоберите образ сервера.

# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-probe-line = { $host } · доступен: { $reachable ->
    [yes] да
   *[no] нет
} · TLS { $tls } · сертификат { $days ->
    [none] не сообщён
    [one] истекает через { $days } день
    [few] истекает через { $days } дня
    [many] истекает через { $days } дней
   *[other] истекает через { $days } дня
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
