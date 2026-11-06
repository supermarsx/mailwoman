# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Веб-хуки
admin-webhooks-intro = Исходящий веб-хук отправляет подписанное уведомление на URL, когда в одной учётной записи что-то происходит. Входящий веб-хук позволяет другой системе выполнить действие почтового правила в одной учётной записи.
admin-webhooks-empty = Веб-хуков пока нет.
admin-webhooks-unavailable = Этот сервер не поддерживает управление веб-хуками.
admin-webhooks-load-error = Не удалось загрузить веб-хуки.
admin-webhooks-direction-outbound = Исходящий
admin-webhooks-direction-inbound = Входящий
admin-webhooks-col-direction = Направление
admin-webhooks-col-label = Название
admin-webhooks-col-account = Учётная запись
admin-webhooks-col-target = Отправляет на или разрешает
admin-webhooks-col-created = Создан
admin-webhooks-no-label = (без названия)
admin-webhooks-created-by = автор: { $who }
admin-webhooks-last-used = Последнее использование: { $when }
admin-webhooks-never-used = Ещё не использовался
admin-webhooks-delete = Удалить
admin-webhooks-delete-named = Удалить веб-хук { $name }
admin-webhooks-delete-title = Удалить веб-хук { $name }?
admin-webhooks-delete-body = Доставка на него прекратится, а вызовы, подписанные его секретом, будут отклоняться. Это действие нельзя отменить; новый веб-хук получит новый секрет.
admin-webhooks-deleted = Веб-хук удалён.
admin-webhooks-add = Добавить веб-хук
admin-webhooks-add-direction = Направление
admin-webhooks-add-outbound-help = Mailwoman вызывает ваш URL.
admin-webhooks-add-inbound-help = Ваша система вызывает Mailwoman.
admin-webhooks-add-account = Учётная запись
admin-webhooks-add-account-help = Учётная запись, события которой отправляются, или единственная учётная запись, с которой может работать входящий вызов.
admin-webhooks-add-label = Название
admin-webhooks-add-label-help = Для чего нужен веб-хук, чтобы другие могли его отличить. Необязательно.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Должен вести на публичный интернет-адрес. Частные адреса, адреса обратной петли и link-local отклоняются.
admin-webhooks-add-events = События
admin-webhooks-add-actions = Разрешённые действия
admin-webhooks-add-submit = Создать веб-хук
admin-webhooks-add-saving = Создание…
admin-webhooks-add-check = Проверить
admin-webhooks-add-check-ok = URL проходит проверку. Ничего не сохранено.
admin-webhooks-event-message-received = Новое сообщение
admin-webhooks-event-message-flags = Сообщение прочитано, помечено флажком или меткой
admin-webhooks-event-message-moved = Сообщение перемещено
admin-webhooks-event-message-deleted = Сообщение удалено
admin-webhooks-event-submission-sent = Сообщение отправлено
admin-webhooks-event-submission-failed = Не удалось отправить сообщение
admin-webhooks-event-pim-changed = Изменены календарь, контакты, задачи или заметки
admin-webhooks-action-run-rules = Выполнить почтовые правила учётной записи по описанию сообщения
admin-webhooks-action-sieve = Применить одно действие правила к сообщению
admin-webhooks-secret-title = Секрет для { $name }
admin-webhooks-secret-once = Скопируйте этот секрет сейчас. Повторно он показан не будет.
admin-webhooks-secret-outbound = Каждая доставка содержит X-Mailwoman-Signature: sha256= и далее HMAC-SHA256 тела запроса с этим секретом.
admin-webhooks-secret-inbound = Отправьте X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (секунды Unix), X-Mailwoman-Nonce (от 16 до 128 букв, цифр, - или _, без повторного использования) и X-Mailwoman-Signature: sha256= и далее HMAC-SHA256 от timestamp.nonce.body с этим секретом. Вызовы со временем, отличающимся более чем на пять минут, или с повторно использованным nonce отклоняются.
admin-webhooks-secret-done = Я скопировал(а)
admin-webhooks-error-stale = Кто-то изменил веб-хуки после открытия этой страницы. Список перезагружен; повторите попытку.
admin-webhooks-error-account = Укажите учётную запись.
admin-webhooks-error-url = Введите URL, начинающийся с https:// или http://.
admin-webhooks-error-events = Выберите хотя бы одно событие.
admin-webhooks-error-actions = Выберите хотя бы одно действие.
