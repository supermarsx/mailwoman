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
