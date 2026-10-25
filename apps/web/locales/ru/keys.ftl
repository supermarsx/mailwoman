# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Управление ключами
keys-title = Ключи и сертификаты
keys-subtitle = Ключи OpenPGP и S/MIME. Закрытые ключи остаются на этом устройстве и никогда не передаются на сервер.
keys-generate = Создать ключ
keys-import = Импортировать ключ

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Ваши ключи
keys-empty-own = Ключей пока нет. Создайте или импортируйте ключ, чтобы начать.
keys-contact-keys = Ключи контактов
keys-empty-contact = Нет ключей контактов. Найдите ключ ниже.
keys-loading = Загрузка ключей…
keys-select-prompt = Выберите ключ, чтобы просмотреть и проверить его.
keys-untitled = Ключ { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = не проверен
keys-trust-tofu = доверие при первом использовании
keys-trust-verified = проверен
keys-trust-revoked = отозван

# -- Key detail --------------------------------------------------------------
keys-key-card = Ключ { $name }
keys-fingerprint = Отпечаток
keys-safe-words = Проверочные слова
keys-safe-words-help = Прочитайте их вслух вместе с контактом, чтобы подтвердить ключ по независимому каналу.
keys-safe-words-list = Проверочные слова отпечатка
keys-scan-label = Отсканируйте для проверки
keys-qr-label = QR-код отпечатка
keys-autocrypt = Autocrypt
keys-autocrypt-on = Публикуется в заголовках Autocrypt
keys-autocrypt-off = Не публикуется через Autocrypt
keys-trust-label = Доверие
keys-trust-level = Уровень доверия
keys-associate-label = Связать с контактом
keys-associate-select = Контакт для связи
keys-choose-contact = Выберите контакт…
keys-associate-button = Связать
keys-backup-label = Резервная копия
keys-backup-help = Экспортируйте сообщение настройки Autocrypt, чтобы перенести этот ключ на другое устройство.
keys-export-backup = Экспортировать резервную копию
keys-asm-label = Сообщение настройки Autocrypt
keys-no-private-backup = На этом устройстве нет закрытого ключа для резервного копирования

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Сертификат S/MIME
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = Это самоподписанный сертификат: он подписан собственным ключом, а не центром сертификации. Почтовые программы других людей не будут ему доверять, пока они не примут его вручную или пока центр сертификации не выпустит сертификат по файлу запроса ниже.
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = Действителен до { $date }.
keys-smime-not-in-app = Mailwoman пока не подписывает и не расшифровывает почту ключом S/MIME. Экспортируйте ключ, чтобы использовать его в другой почтовой программе.
keys-smime-passphrase = Парольная фраза для этого ключа S/MIME
keys-smime-export-help = Файл .p12 содержит сертификат и закрытый ключ. Ключ зашифрован указанной выше парольной фразой, и та же фраза открывает файл в другой программе. Файл .csr не содержит закрытого ключа.
keys-smime-export = Экспортировать как PKCS#12 (.p12)
keys-smime-request = Скачать запрос на сертификат (.csr)
keys-smime-issued-label = Сертификат, выпущенный для этого ключа (.pem, .crt, .cer)
keys-smime-issued-file = Файл выпущенного сертификата
keys-smime-issued-help = Заменяет сертификат, сохранённый для этого ключа. Сертификат, выпущенный для другого ключа, будет отклонён.
keys-smime-attach = Импортировать выпущенный сертификат
keys-smime-failed = Не получилось. Проверьте парольную фразу.
keys-smime-attach-failed = Сертификат не импортирован. Проверьте парольную фразу и убедитесь, что файл содержит один сертификат, выпущенный для этого ключа.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Поиск ключа контакта
keys-lookup-heading = Поиск ключа
keys-lookup-address = Адрес для поиска
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Источники поиска
keys-source = Источник { $source }
keys-consent-label = Согласие на внешний поиск
keys-consent-text = При поиске ключа выполняются запросы к внешним каталогам (WKD/VKS). У меня есть согласие этого человека на это.
keys-lookup-button = Найти
keys-looking-up = Поиск…
keys-lookup-found = Найдено ключей: { $count } — добавлены в «Ключи контактов».
keys-lookup-none = Ключ не найден.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Создание ключа
keys-type = Тип
keys-key-type = Тип ключа
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = Создаётся ключ RSA длиной 3072 бита и сертификат для указанного ниже адреса сроком на два года. Сертификат самоподписанный: почтовые программы других людей не будут ему доверять, пока они не примут его вручную или пока центр сертификации не выпустит сертификат. Mailwoman пока не подписывает и не расшифровывает им почту; после создания его можно экспортировать для другой почтовой программы.
keys-name = Имя
keys-email = Эл. почта
keys-key-passphrase = Парольная фраза ключа
keys-passphrase-help = Парольная фраза защищает закрытый ключ на этом устройстве. Она никогда не покидает браузер.
keys-cancel = Отмена
keys-generate-submit = Создать
keys-generating = Создание…
keys-generating-smime = Создание ключа RSA в этом браузере. Это может занять до минуты.
keys-generate-failed = Ключ не создан.
keys-generate-failed-smime = Ключ не создан. Сертификат может содержать только адрес в простом формате ASCII (name@domain) и имя длиной не более 64 символов.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Импорт ключа
keys-import-type = Тип импорта
keys-tab-armored = В текстовом виде (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Ключ в текстовом виде
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Парольная фраза (если ключ закрытый)
keys-import-passphrase = Парольная фраза для импорта
keys-pkcs12-file-label = Файл PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Файл PKCS#12
keys-import-password-label = Пароль импорта
keys-pkcs12-password = Пароль PKCS#12
keys-import-preview = Предпросмотр импорта
keys-preview = Предпросмотр
keys-preview-type = Тип: { $kind }
keys-preview-fingerprint-aria = Отпечаток в предпросмотре
keys-preview-fingerprint = Отпечаток: { $fp }
keys-preview-has-private = Содержит закрытый ключ (будет сохранён на этом устройстве)
keys-preview-public-only = Только открытый ключ
keys-import-submit = Импортировать
