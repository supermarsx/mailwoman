# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Сквозное шифрование
crypto-banner-e2ee-detail = У каждого получателя есть ключ — текст сообщения шифруется на этом устройстве.
crypto-banner-tls-label = Шифрование при передаче (TLS)
crypto-banner-tls-detail = Ключи шифрования получателей не найдены; доставка защищена только при передаче.
crypto-banner-mixed-label = Смешанная защита
crypto-banner-mixed-detail = Часть получателей может получить сообщение со сквозным шифрованием; остальным оно будет доставлено только через TLS.
crypto-banner-checking-label = Проверка ключей получателей
crypto-banner-checking = · проверка…
crypto-banner-tls-only = Только TLS для:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Безопасность сообщения
crypto-encrypt-label = Зашифровать (сквозное шифрование)
crypto-encrypt-hint-default = Зашифровать текст на этом устройстве перед отправкой.
crypto-encrypt-drafted = Черновик зашифрован на этом устройстве.
crypto-encrypt-no-key = Нет доступного ключа шифрования получателя.
crypto-sign-label = Подписать (подтвердить, что сообщение от вас)
crypto-protect-subject = Также зашифровать тему
crypto-reason-add-recipient = Добавьте получателя, чтобы проверить наличие ключей шифрования.
crypto-reason-tls = Нет доступного ключа шифрования получателя — отправка через TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Предупреждения о предотвращении утечки данных
crypto-dlp-block = Отправка заблокирована
crypto-dlp-require = Требуется шифрование
crypto-dlp-warn = Внимание
crypto-dlp-notify = Администратор получит уведомление
crypto-dlp-matched = Совпадения: { $list }
