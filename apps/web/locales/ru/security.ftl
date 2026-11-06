# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Сведения о безопасности сообщения
security-section-auth = Аутентификация
security-section-received = Путь доставки
security-section-signature = Подпись
security-section-attachments = Вложения
security-section-warnings = Предупреждения
security-section-controls = Действия с отправителем
security-received-empty = Цепочка заголовков Received недоступна.
security-attachments-empty = Нет вложений.
security-hop-unknown = неизвестно
security-unknown = неизвестно

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = пройдена
security-auth-fail = не пройдена
security-auth-none = отсутствует
security-auth-neutral = нейтрально
security-auth-temperror = временная ошибка
security-auth-permerror = постоянная ошибка

# Expert alignment detail field labels + values.
security-detail-domain = домен
security-detail-selector = селектор
security-detail-policy = политика
security-detail-alignment = выравнивание
security-detail-chain-length = длина цепочки
security-aligned = выровнено
security-not-aligned = не выровнено

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Подпись проверена
security-sig-unverified-key = Подписано — ключ подписавшего не проверен
security-sig-invalid = Подпись недействительна
security-sig-none = Не подписано
security-chain-trusted = Доверенная цепочка
security-chain-untrusted = Недоверенная цепочка
security-chain-expired = Срок действия цепочки истёк
security-chain-unknown = Цепочка неизвестна
security-revocation-good = Не отозван
security-revocation-revoked = Ключ отозван
security-revocation-unknown = Статус отзыва неизвестен
security-key-changed = Ключ подписавшего изменился с прошлого раза
security-fact-signer-key = Ключ подписавшего
security-fact-algorithm = Алгоритм
security-fact-key-created = Ключ создан
security-fact-key-expires = Срок действия ключа
security-fact-chain = Цепочка
security-fact-revocation = Отзыв
security-fact-key-change = Смена ключа

# -- Attachment risk --------------------------------------------------------
security-attach-none = Известных рисков нет
security-attach-macro = Содержит макросы
security-attach-executable = Исполняемый файл
security-attach-encrypted-archive = Зашифрованный архив
security-attach-double-extension = Двойное расширение файла
security-attach-mismatch = несоответствие типа (заявлен { $declared }, определён { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Адрес Reply-To отличается от адреса отправителя
security-anomaly-envelopeFromDivergence = Отправитель в конверте отличается от адреса From
security-anomaly-messageIdDomainAnomaly = Домен в Message-ID не совпадает с доменом отправителя
security-anomaly-dateSkew = Дата отправки выглядит смещённой
security-anomaly-punycodeSender = Отправитель использует punycode (возможно, домен-двойник)

# -- Sender controls --------------------------------------------------------
security-control-block = Заблокировать отправителя
security-control-silence = Отключить уведомления от отправителя
security-control-ignore-conversation = Игнорировать цепочку
security-control-report-phishing = Сообщить о фишинге
security-control-report-junk = Сообщить о спаме
security-control-done-block = Отправитель заблокирован
security-control-done-silence = Уведомления от отправителя отключены
security-control-done-ignore-conversation = Цепочка игнорируется
security-control-done-report-phishing = Отмечено как фишинг
security-control-done-report-junk = Отмечено как спам

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Хранилище с нулевым доступом
security-za-on = Включено
security-za-off = Выключено
security-za-protects = Режим нулевого доступа шифрует хранимую почту, заметки и данные PIM, так что сервер хостинга хранит только шифротекст, который не может прочитать. Он защищает содержимое хранимых данных от любопытного оператора или взлома хоста хранилища: любой, кто прочитает базу данных, файлы на диске или украденную резервную копию, увидит шифротекст XChaCha20-Poly1305 и не сможет восстановить тексты и темы сообщений, вложения, текст заметок или поисковый индекс.

security-za-server-sees-title = Что сервер по-прежнему видит
security-za-sees-1 = Блоки шифротекста — сами зашифрованные записи.
security-za-sees-2 = Непрозрачные идентификаторы записей — идентификаторы, используемые для хранения и получения записей.
security-za-sees-3 = Размеры — длина каждого шифротекста (приблизительный размер сообщения или вложения).
security-za-sees-4 = Отметки времени — когда записи создаются и обновляются.
security-za-sees-5 = Метаданные маршрутизации конверта, нужные для проксирования IMAP/SMTP — сервер по-прежнему подключается к вашему вышестоящему почтовому провайдеру от вашего имени, поэтому через него проходят метаданные подключения и конверт маршрутизации, необходимый для отправки и получения почты.

security-za-active-server-caveat = Режим нулевого доступа защищает данные В ХРАНИЛИЩЕ от любопытного или взломанного хоста. Полностью вредоносный активный сервер, проксирующий ваш текущий трафик IMAP/SMTP, — более сильный противник, и режим нулевого доступа от него НЕ защищает: такой сервер находится в живом соединении с вашим почтовым провайдером и может наблюдать за проходящей почтой или изменять её, независимо от того, как зашифрована хранимая копия. Выбирайте режим нулевого доступа, если ваша задача — «я не хочу, чтобы тот, кто управляет хостом хранилища (или взломает его), читал мою хранимую почту». Это не защита от оператора, который вмешивается в живой путь почты.

security-za-no-search-claim = Здесь нет шифрования с поиском на стороне сервера, и такого заявления не делается. Поиск полностью выполняется на клиенте по содержимому, расшифрованному локально; сервер никогда не хранит открытый текст в пригодной для поиска форме.

security-za-recovery-tradeoff = Ваши ключи создаются и хранятся только на ваших устройствах — сервер никогда не получает ключ в открытом виде. Если вы потеряете парольную фразу (или ключ доступа), И все сопряжённые устройства, И фразу восстановления, зашифрованные данные восстановить будет невозможно. Сохраните фразу восстановления офлайн и относитесь к ней как к главному секрету учётной записи.

security-za-setup-key = Настройка ключа
security-za-key-source = Источник ключа
security-za-passphrase = Парольная фраза
security-za-passkey = Ключ доступа (без пароля)
security-za-passkey-unavailable = ключи доступа недоступны в этом браузере
security-za-passphrase-label = Парольная фраза нулевого доступа
security-za-enable = Включить нулевой доступ
security-za-recovery-title = Фраза восстановления
security-za-recovery-heading = Фраза восстановления — сохраните её офлайн прямо сейчас
security-za-recovery-note = Это единственная копия. Любой, у кого она есть, может прочитать ваши данные; без неё (и без сопряжённого устройства) данные восстановить невозможно.
security-za-disable-section = Отключение нулевого доступа
security-za-disable-heading = Выключить нулевой доступ
security-za-disable-note = Новые данные снова будут храниться в незашифрованном виде. Уже зашифрованные данные остаются доступными для чтения, только пока вы можете получить свой ключ.
security-za-disable-btn = Отключить нулевой доступ
security-za-err-passphrase-len = используйте парольную фразу длиной не менее 8 символов
security-za-err-enable = не удалось включить нулевой доступ
security-za-err-disable = не удалось отключить нулевой доступ

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Сопряжение устройств
security-pair-heading = Сопряжение устройства
security-pair-intro = Перенесите ключи на другое устройство, не передавая их через сервер — он пересылает только непрозрачный запечатанный конверт. Сравните шесть слов на обоих экранах, прежде чем доверять сопряжению.
security-pair-role = Роль при сопряжении
security-pair-new-role = Это новое устройство
security-pair-existing-role = Подключить другое устройство отсюда
security-pair-show-qr = Показать QR-код сопряжения
security-pair-scan = Отсканируйте его на вашем текущем устройстве
security-pair-qr-label = QR-код сопряжения устройства
security-pair-qr-default = QR-код сопряжения
security-pair-copy-code = Или скопируйте этот код сопряжения
security-pair-paste-envelope = Вставьте запечатанный конверт с другого устройства
security-pair-envelope-placeholder = конверт…
security-pair-envelope-label = Запечатанный конверт
security-pair-complete = Завершить сопряжение
security-pair-code-from-new = Код сопряжения с нового устройства
security-pair-code-placeholder = код сопряжения…
security-pair-code-label = Код сопряжения
security-pair-seal = Запечатать мои ключи для этого устройства
security-pair-envelope-out = Запечатанный конверт — вставьте его на новом устройстве
security-pair-compare = Сравните эти слова на обоих устройствах
security-pair-sas-label = Короткая строка аутентификации
security-pair-match = Слова совпадают
security-pair-abort = Слова различаются — прервать
security-pair-confirmed = Сопряжение подтверждено. Этот канал аутентифицирован.
security-pair-err-start = не удалось начать сопряжение
security-pair-err-complete = не удалось завершить сопряжение
security-pair-err-unlock = сначала разблокируйте нулевой доступ, чтобы подключить другое устройство
security-pair-err-seal = не удалось запечатать корневой ключ

# -- Machine translation (2026-11): added to match en ---------------------
security-stepup-title = Подтвердите, что это вы
security-stepup-body = Подтвердите свою личность, чтобы продолжить это действие. Подтверждение действует пять минут в этом сеансе.
security-stepup-password = Пароль
security-stepup-code = Код из приложения-аутентификатора
security-stepup-confirm = Подтвердить и продолжить
security-stepup-passkey = Использовать ключ доступа
security-stepup-cancel = Отмена
security-stepup-failed = Не удалось подтвердить. Повторите попытку.
security-stepup-unavailable = В этом сеансе нет доступного способа подтверждения.
security-center-title = Центр безопасности
security-center-history = История входов
security-center-result = Результат
security-center-all = Все результаты
security-center-success = Успешные
security-center-failure = Неудачные
security-center-blocked = Заблокированные
security-center-refresh = Обновить
security-center-export = Экспортировать историю
security-center-devices = Запомненные устройства
security-center-remember-help = Если администратор это разрешает, запомните это устройство после подтверждения личности. Для важных изменений по-прежнему потребуется новое подтверждение.
security-center-label = Название устройства
security-center-remember = Запомнить это устройство
security-center-revoke = Отозвать
security-center-activity = Активность учётной записи
security-center-error = Не удалось загрузить или изменить сведения о безопасности. Возможно, администратор отключил это действие.
security-center-first-seen = Впервые замечено: { $when }
security-center-passkey-mode = В режиме движка ключи доступа позволяют входить без пароля. В режиме прокси они подтверждают второй фактор после пароля от почты.
security-stepup-recovery = Код восстановления (вместе с паролем, вместо кода из приложения-аутентификатора)
security-center-signin = Вход и ключи доступа
security-center-sessions = Сеансы и доступ
security-center-encryption = Шифрование
signin-geo-title = Политика стран для входа
signin-geo-help = При желании можно отклонять новые входы в вашу учётную запись из стран, откуда вы ещё не входили, или из любой страны вне вашего списка разрешённых. По умолчанию политика выключена и никогда не влияет на уже выполненные входы. Разрешённые IP-адреса и страны всегда проходят эту проверку; пароль и второй фактор по-прежнему требуются.
signin-geo-accuracy = Страна определяется по базе геолокации IP-адресов, которая может быть неточной или устаревшей. VPN, мобильные операторы, прокси и поездки могут заблокировать законный вход или пропустить посторонний. Добавьте доверенные IP-адреса или страны в список разрешённых.
signin-geo-locked = Этим параметром для вашей учётной записи управляет администратор.
signin-geo-mode = Политика
signin-geo-mode-off = Выключена
signin-geo-mode-block-unusual-countries = Блокировать страны, из которых я не входил
signin-geo-mode-block-foreign-countries = Блокировать страны вне моего списка разрешённых
signin-geo-countries = Разрешённые страны (двухбуквенные коды через запятую)
signin-geo-ips = Разрешённые IP-адреса или сети (по одному в строке)
signin-geo-current = Ваш текущий адрес: { $ip } (определённая страна: { $country }).
signin-geo-no-context = Не удалось определить ваш текущий адрес.
signin-geo-unknown-country = неизвестно
signin-geo-add-ip = Добавить мой текущий IP-адрес
signin-geo-add-country = Добавить мою текущую страну
signin-geo-save = Сохранить
signin-geo-saved = Сохранено.
signin-geo-invalid = Проверьте коды стран и IP-адреса.
signin-geo-self-lockout = Это заблокирует вход с вашего текущего адреса или из вашей страны. Сначала добавьте текущий IP-адрес или страну в список разрешённых.
signin-geo-error = Не удалось сохранить политику стран для входа.
session-alerts-title = Уведомления о безопасности сеансов
session-alerts-help = Предупреждения о необычном использовании ваших сеансов. В каждом уведомлении указано, что заметил сервер и что он сделал.
session-alerts-none = Уведомлений о безопасности сеансов нет.
session-alerts-banner = { $count ->
    [one] У вас { $count } непрочитанное уведомление о безопасности сеансов.
    [few] У вас { $count } непрочитанных уведомления о безопасности сеансов.
    [many] У вас { $count } непрочитанных уведомлений о безопасности сеансов.
   *[other] У вас { $count } непрочитанного уведомления о безопасности сеансов.
}
session-alerts-banner-review = Просмотреть
session-alerts-kind-countryChange = Сеанс использовался из другой страны ({ $from } → { $to }).
session-alerts-kind-impossibleTravel = Сеанс появился в другой стране ({ $from } → { $to }) быстрее, чем это возможно при поездке.
session-alerts-kind-concurrentUse = Один сеанс одновременно использовался из разных стран или сетей ({ $from } и { $to }).
session-alerts-kind-fingerprintMismatch = Сеанс использовался из браузера или с устройства, не совпадающих с теми, с которых выполнялся вход.
session-alerts-kind-rotationReuse = Старый идентификатор сеанса был предъявлен снова после обновления сеанса.
session-alerts-action-alerted = Действие: вы получили предупреждение.
session-alerts-action-step-up = Действие: для сеанса запрошено подтверждение пароля и второго фактора.
session-alerts-action-revoked = Действие: сеанс завершён.
session-alerts-countries = Страны: { $from } → { $to }
session-alerts-unknown-country = неизвестно
session-alerts-session = Сеанс { $id }
session-alerts-caveat = Страна может определяться неточно; VPN и мобильные сети могут вызывать ложные изменения.
session-alerts-not-me = Это был не я
session-alerts-dismiss = Скрыть
session-alerts-dismiss-all = Скрыть все
session-alerts-revoked = Ваши остальные сеансы завершены. Сейчас же смените пароль в разделе пароля на странице «Вход и ключи доступа».
session-alerts-error = Не удалось обновить уведомления о безопасности сеансов.
session-alerts-prefs = Настройки оповещений
session-alerts-country = Предупреждать, когда сеанс используется из новой страны
session-alerts-country-help = Если администратор не разрешает отключать оповещения, сервер продолжит предупреждать, даже когда этот параметр выключен.
session-alerts-channels = Где получать предупреждения
session-alerts-channel-app = В приложении
session-alerts-channel-email = В моём почтовом ящике
session-alerts-channel-both = В приложении и в моём почтовом ящике
session-alerts-sensitivity = Чувствительность
session-alerts-sensitivity-low = Низкая: только выводы о местоположении, без принудительного выхода
session-alerts-sensitivity-normal = Обычная
session-alerts-sensitivity-high = Высокая: более строгая проверка поездок, также сравниваются сети
session-alerts-reset = Использовать значения администратора по умолчанию
session-alerts-pref-error = Не удалось сохранить настройку.
signin-geo-history-title = Замеченные страны
signin-geo-history-help = Страны, из которых вы успешно входили; хранятся { $days } дн. Режим «необычные страны» использует этот список. Хранятся только коды стран, даты и счётчики, но не адреса.
signin-geo-history-empty = Страны пока не записаны.
signin-geo-history-entry = { $country }: первый вход { $first }, последний { $last }, входов: { $count }
signin-geo-history-forget = Забыть замеченные страны
security-severity-info = Информация
security-severity-caution = Осторожность
security-severity-suspicious = Подозрительно
security-explain-toggle = Что это значит?
security-explain-label-checked = Что проверялось
security-explain-label-legit = Типичные безвредные причины
security-explain-label-risk = О чём это может говорить
security-explain-label-action = Рекомендуемое действие
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = Домен после символа @ в заголовке Message-ID сравнили с доменом в адресе отправителя (From). Они различаются.
security-explain-messageIdDomainAnomaly-legit = Рассылки, маркетинговые и новостные платформы, службы пересылки, корпоративные шлюзы и хосты веб-почты часто формируют Message-ID с именем собственного сервера.
security-explain-messageIdDomainAnomaly-risk = Подделанный отправитель или письмо, созданное программой, которая не является почтовой системой отправителя. Само по себе это слабый признак.
security-explain-messageIdDomainAnomaly-action = Посмотрите результаты SPF, DKIM и DMARC выше. Если для домена отправителя проверки пройдены, скорее всего, это обычная ситуация. Если они не пройдены или отсутствуют, не открывайте ссылки и вложения и подтвердите отправителя другим способом.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = Адрес Reply-To сравнили с адресом From. Ответы уйдут на другой адрес.
security-explain-replyToMismatch-legit = Рассылки, службы поддержки, помощники, пишущие от чужого имени, и контактные формы намеренно задают другой Reply-To.
security-explain-replyToMismatch-risk = Перенаправление ответов на адрес, который контролирует злоумышленник, — частый приём при выдаче себя за другого и мошенничестве со счетами.
security-explain-replyToMismatch-action = Проверьте адрес Reply-To перед ответом. Если в письме просят деньги, учётные данные или изменение реквизитов, подтвердите это по каналу, которому вы уже доверяете.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = Отправителя конверта (Return-Path), на который идут уведомления о недоставке, сравнили с адресом From. Они различаются.
security-explain-envelopeFromDivergence-legit = Сервисы массовой рассылки, системы заявок и рассылки используют собственный адрес для возвратов, а в поле From показывают адрес вашего контакта.
security-explain-envelopeFromDivergence-risk = Подмена отправителя, если SPF или DMARC тоже не пройдены. Само по себе это обычное и, как правило, безвредное явление.
security-explain-envelopeFromDivergence-action = Проверьте, пройден ли DMARC с выравниванием. Если да, ничего делать не нужно.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = Заголовок Date сравнили со временем получения письма. Они сильно расходятся.
security-explain-dateSkew-legit = Компьютер отправителя с неверными часами, письмо, долго пролежавшее в очереди или повторно отправлявшееся, либо отложенная отправка.
security-explain-dateSkew-risk = Повторно отправленное или датированное задним числом письмо. Это бывает редко и само по себе редко опасно.
security-explain-dateSkew-action = Осторожно относитесь к срочным просьбам в письме и подтверждайте их, если они важны.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = Домен отправителя содержит punycode (метку xn--) — так в доменных именах записываются нелатинские символы.
security-explain-punycodeSender-legit = Так кодируются настоящие домены на арабском, китайском, кириллице или португальском с диакритикой.
security-explain-punycodeSender-risk = Похожий домен, в котором смешаны символы разных алфавитов, чтобы походить на известный бренд или человека.
security-explain-punycodeSender-action = Сравните домен отправителя с ожидаемым посимвольно. Не переходите по ссылкам и не открывайте вложения, пока не подтвердите отправителя другим способом, и сообщите о письме, если оно поддельное.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = Принимающий сервер проверил этот метод, и результат не совпал с тем, что публикует домен отправителя.
security-explain-auth-fail-legit = Пересылка и рассылки могут нарушить SPF или DKIM, а неправильно настроенный отправляющий сервис может не пройти любую из проверок.
security-explain-auth-fail-risk = Письмо может быть не от того, кем назван отправитель, или было изменено по пути.
security-explain-auth-fail-action = Не доверяйте отправителю на основании этого признака. Не открывайте ссылки и вложения и проверьте по другому каналу.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = Проверку не удалось завершить, потому что опубликованная запись домена недопустима или слишком сложна.
security-explain-auth-permerror-legit = Ошибка в DNS-записях отправителя.
security-explain-auth-permerror-risk = Ничего о самом письме, но домен отправителя проверить нельзя.
security-explain-auth-permerror-action = Считайте это отсутствием результата и проверьте остальные методы.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = В заголовках письма не найден результат для этого метода.
security-explain-auth-none-legit = Домен отправителя не публикует этот метод, либо принимающий сервер его не записывает.
security-explain-auth-none-risk = Домен не даёт возможности проверить отправителя, поэтому подмену исключить нельзя.
security-explain-auth-none-action = Опирайтесь на другие методы и на то, ожидали ли вы это письмо.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = Домен опубликовал запись для этого метода, но ничего не заявил об этом письме.
security-explain-auth-neutral-legit = Домен, который ещё настраивает свою политику.
security-explain-auth-neutral-risk = Небольшой. Отправителя нельзя было ни подтвердить, ни отклонить.
security-explain-auth-neutral-action = Опирайтесь на другие методы и на то, ожидали ли вы это письмо.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = Проверку не удалось завершить из-за временной проблемы, обычно сбоя DNS-запроса.
security-explain-auth-temperror-legit = Краткий сбой DNS у отправителя или получателя.
security-explain-auth-temperror-risk = Ничего о самом письме.
security-explain-auth-temperror-action = Обычно действий не требуется. Если письмо важное, проверьте ещё раз позже.
breach-notice-heading = Уведомление о нарушении защиты персональных данных
breach-notice-reference = Номер обращения { $reference }
breach-notice-ack = Подтвердить ознакомление
