# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Вебхуки
admin-webhooks-intro = Вихідний вебхук надсилає підписане сповіщення на URL, коли в одному обліковому записі щось відбувається. Вхідний вебхук дозволяє іншій системі виконати дію поштового правила в одному обліковому записі.
admin-webhooks-empty = Вебхуків поки немає.
admin-webhooks-unavailable = Цей сервер не підтримує керування вебхуками.
admin-webhooks-load-error = Не вдалося завантажити вебхуки.
admin-webhooks-direction-outbound = Вихідний
admin-webhooks-direction-inbound = Вхідний
admin-webhooks-col-direction = Напрямок
admin-webhooks-col-label = Назва
admin-webhooks-col-account = Обліковий запис
admin-webhooks-col-target = Надсилає на або дозволяє
admin-webhooks-col-created = Створено
admin-webhooks-no-label = (без назви)
admin-webhooks-created-by = автор: { $who }
admin-webhooks-last-used = Востаннє використано: { $when }
admin-webhooks-never-used = Ще не використовувався
admin-webhooks-delete = Видалити
admin-webhooks-delete-named = Видалити вебхук { $name }
admin-webhooks-delete-title = Видалити вебхук { $name }?
admin-webhooks-delete-body = Доставлення на нього припиниться, а виклики, підписані його секретом, відхилятимуться. Цю дію не можна скасувати; новий вебхук отримає новий секрет.
admin-webhooks-deleted = Вебхук видалено.
admin-webhooks-add = Додати вебхук
admin-webhooks-add-direction = Напрямок
admin-webhooks-add-outbound-help = Mailwoman викликає ваш URL.
admin-webhooks-add-inbound-help = Ваша система викликає Mailwoman.
admin-webhooks-add-account = Обліковий запис
admin-webhooks-add-account-help = Обліковий запис, події якого надсилаються, або єдиний обліковий запис, з яким може працювати вхідний виклик.
admin-webhooks-add-label = Назва
admin-webhooks-add-label-help = Для чого призначений вебхук, щоб інші могли його розрізнити. Необов'язково.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Має вести на публічну інтернет-адресу. Приватні адреси, адреси зворотної петлі та link-local відхиляються.
admin-webhooks-add-events = Події
admin-webhooks-add-actions = Дозволені дії
admin-webhooks-add-submit = Створити вебхук
admin-webhooks-add-saving = Створення…
admin-webhooks-add-check = Перевірити
admin-webhooks-add-check-ok = URL проходить перевірку. Нічого не збережено.
admin-webhooks-event-message-received = Нове повідомлення
admin-webhooks-event-message-flags = Повідомлення прочитано, позначено прапорцем або міткою
admin-webhooks-event-message-moved = Повідомлення переміщено
admin-webhooks-event-message-deleted = Повідомлення видалено
admin-webhooks-event-submission-sent = Повідомлення надіслано
admin-webhooks-event-submission-failed = Не вдалося надіслати повідомлення
admin-webhooks-event-pim-changed = Змінено календар, контакти, завдання або нотатки
admin-webhooks-action-run-rules = Виконати поштові правила облікового запису за описом повідомлення
admin-webhooks-action-sieve = Застосувати одну дію правила до повідомлення
admin-webhooks-secret-title = Секрет для { $name }
admin-webhooks-secret-once = Скопіюйте цей секрет зараз. Повторно його показано не буде.
admin-webhooks-secret-outbound = Кожне доставлення містить X-Mailwoman-Signature: sha256= і далі HMAC-SHA256 тіла запиту з цим секретом.
admin-webhooks-secret-inbound = Надішліть X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (секунди Unix), X-Mailwoman-Nonce (від 16 до 128 літер, цифр, - або _, без повторного використання) і X-Mailwoman-Signature: sha256= і далі HMAC-SHA256 від timestamp.nonce.body з цим секретом. Виклики з часом, що відрізняється більш ніж на п'ять хвилин, або з повторно використаним nonce відхиляються.
admin-webhooks-secret-done = Я скопіював(ла)
admin-webhooks-error-stale = Хтось змінив вебхуки після відкриття цієї сторінки. Список перезавантажено; спробуйте ще раз.
admin-webhooks-error-account = Вкажіть обліковий запис.
admin-webhooks-error-url = Введіть URL, що починається з https:// або http://.
admin-webhooks-error-events = Оберіть принаймні одну подію.
admin-webhooks-error-actions = Оберіть принаймні одну дію.
