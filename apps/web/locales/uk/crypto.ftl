# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Наскрізне шифрування
crypto-banner-e2ee-detail = Кожен одержувач має ключ — текст повідомлення шифрується на цьому пристрої.
crypto-banner-tls-label = Транспортне шифрування (TLS)
crypto-banner-tls-detail = Ключів шифрування одержувачів не знайдено; доставка захищена лише під час передавання.
crypto-banner-mixed-label = Змішаний захист
crypto-banner-mixed-detail = Частина одержувачів може отримати наскрізне шифрування; решта — лише TLS.
crypto-banner-checking-label = Перевірка ключів одержувачів
crypto-banner-checking = · перевірка…
crypto-banner-tls-only = Лише TLS для:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Захист повідомлення
crypto-encrypt-label = Зашифрувати (наскрізно)
crypto-encrypt-hint-default = Зашифрувати текст на цьому пристрої перед надсиланням.
crypto-encrypt-drafted = Чернетку зашифровано на цьому пристрої.
crypto-encrypt-no-key = Немає ключа шифрування одержувача.
crypto-sign-label = Підписати (підтвердити, що це від вас)
crypto-protect-subject = Також зашифрувати тему
crypto-reason-add-recipient = Додайте одержувача, щоб перевірити наявність ключів шифрування.
crypto-reason-tls = Немає ключа шифрування одержувача — надсилання через TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Попередження захисту від витоку даних
crypto-dlp-block = Надсилання заблоковано
crypto-dlp-require = Потрібне шифрування
crypto-dlp-warn = Зверніть увагу
crypto-dlp-notify = Адміністратора буде сповіщено
crypto-dlp-matched = Збіги: { $list }
