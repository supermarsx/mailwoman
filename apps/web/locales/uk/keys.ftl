# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Керування ключами
keys-title = Ключі та сертифікати
keys-subtitle = Ключі OpenPGP і S/MIME. Приватні ключі залишаються на цьому пристрої й ніколи не потрапляють на сервер.
keys-generate = Згенерувати ключ
keys-import = Імпортувати ключ

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Ваші ключі
keys-empty-own = Ключів ще немає. Згенеруйте або імпортуйте ключ, щоб почати.
keys-contact-keys = Ключі контактів
keys-empty-contact = Немає ключів контактів. Знайдіть ключ нижче.
keys-loading = Завантаження ключів…
keys-select-prompt = Виберіть ключ, щоб переглянути та перевірити його.
keys-untitled = Ключ { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = не перевірено
keys-trust-tofu = TOFU (довіра з першого використання)
keys-trust-verified = перевірено
keys-trust-revoked = відкликано

# -- Key detail --------------------------------------------------------------
keys-key-card = Ключ { $name }
keys-fingerprint = Відбиток
keys-safe-words = Контрольні слова
keys-safe-words-help = Прочитайте їх уголос разом із контактом, щоб підтвердити ключ поза поштою.
keys-safe-words-list = Контрольні слова відбитка
keys-scan-label = Відскануйте для перевірки
keys-qr-label = QR-код відбитка
keys-autocrypt = Autocrypt
keys-autocrypt-on = Оголошується в заголовках Autocrypt
keys-autocrypt-off = Не оголошується через Autocrypt
keys-trust-label = Довіра
keys-trust-level = Рівень довіри
keys-associate-label = Пов'язати з контактом
keys-associate-select = Контакт для зв'язування
keys-choose-contact = Виберіть контакт…
keys-associate-button = Пов'язати
keys-backup-label = Резервна копія
keys-backup-help = Експортуйте повідомлення налаштування Autocrypt (Autocrypt Setup Message), щоб перенести цей ключ на інший пристрій.
keys-export-backup = Експортувати резервну копію
keys-asm-label = Повідомлення налаштування Autocrypt
keys-no-private-backup = На цьому пристрої немає приватного ключа для резервного копіювання

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Сертифікат S/MIME
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = Цей сертифікат самопідписаний: його підписано власним ключем, а не центром сертифікації. Поштові програми інших людей не довірятимуть йому, доки вони не приймуть його вручну або доки центр сертифікації не видасть сертифікат за файлом запиту нижче.
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = Дійсний до { $date }.
keys-smime-not-in-app = Mailwoman поки не підписує й не розшифровує пошту ключем S/MIME. Експортуйте ключ, щоб використовувати його в іншій поштовій програмі.
keys-smime-passphrase = Парольна фраза для цього ключа S/MIME
keys-smime-export-help = Файл .p12 містить сертифікат і приватний ключ. Ключ зашифровано парольною фразою, вказаною вище, і та сама парольна фраза відкриває файл в іншій програмі. Файл .csr не містить приватного ключа.
keys-smime-export = Експортувати як PKCS#12 (.p12)
keys-smime-request = Завантажити запит на сертифікат (.csr)
keys-smime-issued-label = Сертифікат, виданий для цього ключа (.pem, .crt, .cer)
keys-smime-issued-file = Файл виданого сертифіката
keys-smime-issued-help = Замінює сертифікат, збережений для цього ключа. Сертифікат, створений для іншого ключа, буде відхилено.
keys-smime-attach = Імпортувати виданий сертифікат
keys-smime-failed = Не вдалося. Перевірте парольну фразу.
keys-smime-attach-failed = Сертифікат не імпортовано. Перевірте парольну фразу і те, що файл містить один сертифікат, виданий для цього ключа.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Пошук ключа контакту
keys-lookup-heading = Пошук ключа
keys-lookup-address = Адреса для пошуку
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Джерела пошуку
keys-source = Джерело { $source }
keys-consent-label = Згода на зовнішній пошук
keys-consent-text = Пошук ключа звертається до зовнішніх довідників (WKD/VKS). Я маю згоду цієї людини на це.
keys-lookup-button = Шукати
keys-looking-up = Пошук…
keys-lookup-found = { $count ->
    [one] Знайдено { $count } ключ — додано до ключів контактів.
    [few] Знайдено { $count } ключі — додано до ключів контактів.
    [many] Знайдено { $count } ключів — додано до ключів контактів.
   *[other] Знайдено { $count } ключа — додано до ключів контактів.
}
keys-lookup-none = Ключа не знайдено.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Генерування ключа
keys-type = Тип
keys-key-type = Тип ключа
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = Створює 3072-бітний ключ RSA і сертифікат для вказаної нижче адреси строком на два роки. Сертифікат самопідписаний: поштові програми інших людей не довірятимуть йому, доки вони не приймуть його вручну або доки центр сертифікації не видасть сертифікат. Mailwoman поки не підписує й не розшифровує ним пошту; згодом його можна експортувати для іншої поштової програми.
keys-name = Ім'я
keys-email = Електронна пошта
keys-key-passphrase = Парольна фраза ключа
keys-passphrase-help = Парольна фраза захищає приватний ключ на цьому пристрої. Вона ніколи не залишає браузер.
keys-cancel = Скасувати
keys-generate-submit = Згенерувати
keys-generating = Генерування…
keys-generating-smime = Генерування ключа RSA у цьому браузері. Це може тривати до хвилини.
keys-generate-failed = Ключ не згенеровано.
keys-generate-failed-smime = Ключ не згенеровано. Сертифікат може містити лише звичайну адресу ASCII (name@domain) та ім'я довжиною не більше 64 символів.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Імпорт ключа
keys-import-type = Тип імпорту
keys-tab-armored = Текстовий формат (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Ключ у текстовому форматі
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Парольна фраза (якщо ключ приватний)
keys-import-passphrase = Парольна фраза для імпорту
keys-pkcs12-file-label = Файл PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Файл PKCS#12
keys-import-password-label = Пароль імпорту
keys-pkcs12-password = Пароль PKCS#12
keys-import-preview = Попередній перегляд імпорту
keys-preview = Попередній перегляд
keys-preview-type = Тип: { $kind }
keys-preview-fingerprint-aria = Відбиток для попереднього перегляду
keys-preview-fingerprint = Відбиток: { $fp }
keys-preview-has-private = Містить приватний ключ (буде збережено на цьому пристрої)
keys-preview-public-only = Лише публічний ключ
keys-import-submit = Імпортувати
