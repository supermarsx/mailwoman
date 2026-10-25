# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Настройки
mail-compose = Написать
mail-nav-mailboxes = Папки
mail-nav-attachments = Вложения
mail-nav-outbox = Исходящие
mail-nav-sharing = Общий доступ к папке
mail-nav-apps = Приложения
mail-logout = Выйти
mail-offline = Нет подключения
mail-module-loading = Загрузка: { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Поиск
mail-search-label = Поиск в почте
mail-search-placeholder = Поиск в почте — from:alice subject:invoice larger:1mb
mail-search-clear = Очистить

# -- Message list ------------------------------------------------------------
mail-list-label = Сообщения
mail-loading = Загрузка сообщений…
mail-empty = Нет сообщений
mail-unknown-sender = (неизвестный отправитель)
mail-no-subject = (без темы)
mail-pinned = Закреплено
mail-has-attachment = Есть вложение
mail-answered = Отвечено
mail-forwarded = Переслано
mail-unread = Не прочитано
mail-flagged = Помечено
# Announced on a selected row; the row also shows a check mark.
mail-selected = Выбрано
# Announced row summary for screen readers (position within the whole list).
mail-row-position = Сообщение { $pos } из { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } сообщение
    [few] { $count } сообщения
    [many] { $count } сообщений
   *[other] { $count } сообщения
}
mail-thread-expand = Развернуть цепочку, { $count ->
    [one] { $count } сообщение
    [few] { $count } сообщения
    [many] { $count } сообщений
   *[other] { $count } сообщения
}
mail-thread-collapse = Свернуть цепочку, { $count ->
    [one] { $count } сообщение
    [few] { $count } сообщения
    [many] { $count } сообщений
   *[other] { $count } сообщения
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Параметры вида
mail-reading-pane = Область чтения
mail-reading-pane-right = Справа
mail-reading-pane-bottom = Снизу
mail-reading-pane-off = Выкл.

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Выбранные сообщения
mail-bulk-count = Выбрано: { $count }
mail-mark-read = Отметить как прочитанные
mail-mark-unread = Отметить как непрочитанные
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Пометить
mail-unflag-message = Снять пометку
mail-move-to = Переместить в папку
mail-move-to-placeholder = Переместить в…
mail-select-all = Выбрать все
mail-clear-selection = Снять выделение

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = Сортировка входящих
mail-inbox-filter = Фильтр входящих
mail-inbox-focused = Важные
mail-inbox-other = Другие
mail-inbox-turn-off = Отключить

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Открытые вкладки
mail-subtab-pin = Закрепить { $title }
mail-subtab-unpin = Открепить { $title }
mail-subtab-tearoff = Открыть { $title } в новом окне
mail-subtab-close = Закрыть { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Закрепить
mail-unpin = Открепить
mail-snooze = Отложить
mail-snooze-menu = Отложить до
mail-snooze-later = Позже сегодня
mail-snooze-tomorrow = Завтра
mail-snooze-next-week = На следующей неделе
mail-unsnooze = Не откладывать
mail-label = Метка
mail-labels-menu = Метки
mail-flag = Пометить для отслеживания
mail-clear-flag = Снять отслеживание
mail-archive = Архивировать
mail-delete = Удалить
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = Другие действия

# -- Reader ------------------------------------------------------------------
mail-reader-label = Сообщение
mail-reader-empty = Выберите сообщение для чтения
mail-back = Назад
mail-reader-from = От: { $addr }
mail-reader-to = Кому: { $addr }
mail-reader-cc = Копия: { $addr }
mail-reader-actions = Действия с сообщением
mail-reply = Ответить
mail-reply-all = Ответить всем
mail-forward = Переслать
mail-reply-failed = Не удалось подготовить сообщение. Повторите попытку.
mail-reply-unthreaded = Не удалось прочитать идентификатор исходного сообщения. Этот ответ не будет объединён с ним в цепочку.
mail-spam = Спам
mail-export = Экспорт
mail-sweep-sender = Очистить от отправителя
mail-message-body = Текст сообщения
mail-no-content = Нет содержимого
mail-sanitizing = Очистка содержимого…
mail-attachments = Вложения
mail-attachment-unnamed = (без имени)
mail-attachment-open = Вложение { $name }
mail-attachment-close = Закрыть вложение
mail-attachment-loading = Загрузка вложения…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Зашифрованное сообщение
mail-encrypted-note = Это сообщение защищено сквозным шифрованием. Разблокируйте его на этом устройстве, чтобы прочитать.
mail-key-passphrase = Парольная фраза ключа
mail-decrypt = Расшифровать
mail-decrypting = Расшифровка…
mail-decrypt-no-key = Нет закрытого ключа для расшифровки этого сообщения.
mail-decrypt-failed = Не удалось расшифровать

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = { $date }, { $name } пишет:
mail-quote-attribution-undated = { $name } пишет:
# The header block above a forwarded original.
mail-forward-heading = ---------- Пересылаемое сообщение ----------
mail-forward-from = От: { $addr }
mail-forward-date = Дата: { $date }
mail-forward-subject = Тема: { $subject }
mail-forward-to = Кому: { $addr }
mail-forward-cc = Копия: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Создание сообщения
mail-compose-title = Новое сообщение
mail-compose-title-reply = Ответ
mail-compose-title-reply-all = Ответ всем
mail-compose-title-forward = Пересылка
mail-compose-close = Закрыть
mail-compose-from = От
mail-compose-from-default = По умолчанию
mail-compose-to = Кому
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Предлагаемые контакты
mail-compose-cc = Копия
mail-compose-bcc = Скрытая копия
mail-compose-show-cc-bcc = Добавить копию или скрытую копию
mail-compose-bcc-encrypted = В зашифрованном сообщении указан ключ каждого получателя, поэтому получатель скрытой копии будет виден остальным. Удалите получателей скрытой копии или отключите шифрование.
mail-compose-plaintext-confirm-title = Отправить расшифрованный текст без шифрования?
mail-compose-plaintext-confirm-body = Это сообщение цитирует текст из зашифрованного сообщения. Шифрование для него не включено, поэтому цитируемый текст будет отправлен без защиты.
mail-compose-plaintext-confirm-send = Отправить без шифрования
mail-compose-plaintext-confirm-cancel = Вернуться
mail-compose-subject = Тема
mail-compose-body = Текст
mail-compose-attachments = Вложения
mail-compose-remove-attachment = Удалить { $name }
mail-compose-attach-nextcloud = Вложить из Nextcloud
mail-compose-close-nextcloud = Закрыть Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Вложить файл
mail-compose-uploading = Отправка файла…
mail-compose-upload-unavailable = Отправка файлов сейчас недоступна.
mail-compose-upload-failed = Не удалось отправить { $name }.
mail-compose-upload-too-large = Размер { $name } — { $size } МБ. Максимальный размер файла — { $max } МБ.
mail-compose-send-later = Отправить позже
mail-compose-cancel = Отмена
mail-compose-send = Отправить
mail-compose-schedule = Запланировать
mail-compose-sending = Отправка…
mail-compose-encrypted-subject = Зашифрованное сообщение
mail-compose-dlp-blocked = Отправка заблокирована правилом предотвращения утечки данных (см. предупреждение выше).
mail-compose-send-failed = Не удалось отправить
mail-compose-send-later-past = Выберите время отправки в будущем или очистите поле, чтобы отправить сейчас.
mail-compose-options = Параметры отправки
mail-compose-request-receipt = Запросить уведомление о прочтении
mail-compose-request-receipt-hint = Почтовая программа получателя может спросить его, отправить уведомление без вопроса или проигнорировать запрос.
mail-compose-track-opens = Записывать, когда это сообщение открывают
mail-compose-track-opens-hint = В сообщение добавляется изображение 1×1, загружаемое с этого сервера. Многие почтовые программы блокируют изображения или загружают их заранее, поэтому записанное открытие не доказывает, что сообщение прочитано, а отсутствие записи не доказывает обратного. Получатель об этом не уведомляется.
mail-compose-image-type = { $name } нельзя вставить в текст. Используйте изображение PNG, JPEG, GIF или WebP.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = Вы не вошли в систему.
mail-send-no-draft-folder = У этой учётной записи нет папки для хранения черновика.
mail-send-no-recipient = Добавьте хотя бы одного получателя.
mail-send-bad-recipient = { $count ->
    [one] Это не адрес эл. почты: { $addresses }
    [few] Это не адреса эл. почты: { $addresses }
    [many] Это не адреса эл. почты: { $addresses }
   *[other] Это не адреса эл. почты: { $addresses }
}
mail-send-later-past = Время отправки не в будущем. Ничего не отправлено.
mail-send-tracking-failed = Сейчас нельзя записывать открытия, поэтому ничего не отправлено. Снимите флажок «Записывать, когда это сообщение открывают», чтобы отправить без этого.
mail-send-draft-refused = Сервер отклонил это сообщение: { $reason }. Ничего не отправлено.
mail-send-submission-refused = Сервер отказался отправить это сообщение: { $reason }. Ничего не отправлено.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Разблокировка ключа подписи
mail-compose-sign-unlock-note = Введите парольную фразу ключа, чтобы подписать это сообщение. Ключ остаётся разблокированным только для этого окна создания сообщения.
mail-compose-sign-unlock = Разблокировать
mail-compose-sign-unlocking = Разблокировка…
mail-compose-sign-no-key = На этом устройстве нет ключа подписи.
mail-compose-sign-unlock-failed = Не удалось разблокировать ключ подписи.
mail-compose-sign-unlock-required = Разблокируйте ключ подписи, чтобы отправить это подписанное сообщение.

# Rich-text editor (W1)
mail-compose-format-plain = Обычный текст
mail-compose-format-rich = Форматированный текст
mail-compose-rt-toolbar = Форматирование
mail-compose-rt-bold = Полужирный
mail-compose-rt-italic = Курсив
mail-compose-rt-underline = Подчёркнутый
mail-compose-rt-strike = Зачёркнутый
mail-compose-rt-bullet = Маркированный список
mail-compose-rt-ordered = Нумерованный список
mail-compose-rt-quote = Цитата
mail-compose-rt-link = Ссылка
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Применить
mail-compose-rt-image = Вставить изображение

# Signature picker (W12)
mail-compose-signature = Подпись
mail-compose-signature-none = Добавить подпись…

# Recall (W10)
mail-compose-recall = Отзыв
mail-compose-recall-action = Отозвать
mail-compose-recall-holding = Задержано, ожидает отправки
mail-compose-recall-scheduled = Запланировано на { $when }

# Drafts drawer (W9)
mail-compose-drafts = Черновики
mail-compose-drafts-empty = Нет сохранённых черновиков
mail-compose-drafts-delete = Удалить черновик
mail-compose-drafts-no-recipient = (нет получателя)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Исходящие
mail-outbox-refresh = Обновить
mail-outbox-empty = Нет сообщений, ожидающих отправки
mail-outbox-send-now = Отправить сейчас
mail-outbox-cancel = Отмена
mail-outbox-scheduled = Запланировано
mail-outbox-holding = Скоро будет отправлено
mail-outbox-sent = Отправлено
mail-outbox-canceled = Отменено
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = Задержано
mail-outbox-failed = Не отправлено
mail-outbox-release = Разрешить отправку
mail-outbox-discard = Отказаться
mail-outbox-held-note = Задержано. Не будет отправлено, пока вы не разрешите отправку.
mail-outbox-held-note-api-key = Задержано — создано ключом API { $name }. Не будет отправлено, пока вы не разрешите отправку.
mail-outbox-held-note-oauth = Задержано — создано подключённым приложением { $name }. Не будет отправлено, пока вы не разрешите отправку.
mail-outbox-origin-api-key = Создано ключом API { $name }
mail-outbox-origin-oauth = Создано подключённым приложением { $name }
mail-outbox-to = Кому: { $recipients }
mail-outbox-no-subject = (без темы)
mail-outbox-attempts = Попыток: { $count }. Последняя ошибка: { $error }
mail-outbox-error = Ошибка: { $error }
mail-outbox-toast-canceled = Отправка отменена
mail-outbox-toast-discarded = Удалено. Сообщение не отправлено.
mail-outbox-toast-sent = Отправлено
mail-outbox-toast-retrying = Пока не отправлено: { $error }. Будет выполнена повторная попытка.
mail-outbox-toast-failed = Не отправлено: { $error }
mail-outbox-toast-not-released = Не удалось отправить: { $error }
mail-outbox-toast-not-canceled = Не удалось отменить: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = Соединение потеряно — повторное подключение…
mail-net-back = Подключение восстановлено
mail-queue-sent = { $count ->
    [one] Отправлено { $count } изменение из очереди
    [few] Отправлено { $count } изменения из очереди
    [many] Отправлено { $count } изменений из очереди
   *[other] Отправлено { $count } изменения из очереди
}
mail-queue-failed = { $count ->
    [one] Не удалось применить { $count } изменение из очереди
    [few] Не удалось применить { $count } изменения из очереди
    [many] Не удалось применить { $count } изменений из очереди
   *[other] Не удалось применить { $count } изменения из очереди
}
mail-queue-notice-label = Изменения, которые не удалось применить
mail-queue-notice-title = { $count ->
    [one] Не удалось применить { $count } изменение, сделанное офлайн
    [few] Не удалось применить { $count } изменения, сделанных офлайн
    [many] Не удалось применить { $count } изменений, сделанных офлайн
   *[other] Не удалось применить { $count } изменения, сделанного офлайн
}
mail-queue-kind-send = Отправить «{ $subject }»
mail-queue-kind-send-untitled = Отправить сообщение без темы
mail-queue-kind-move = Переместить сообщение
mail-queue-kind-flag = Изменить метку или флаг
mail-queue-kind-draft = Сохранить черновик
mail-queue-kind-pim = Изменение в календаре, задачах, заметках или контактах
mail-queue-reason = Причина: { $reason }
mail-queue-reason-unknown = Сервер не сообщил причину.
mail-queue-retry = Повторить
mail-queue-discard = Отказаться
mail-queue-retry-item = Повторить: { $what }
mail-queue-discard-item = Отказаться: { $what }
mail-queue-discarded = Отменено. Изменение не применено.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Очистка сообщений
mail-sweep-title = Очистить от { $sender }
mail-sweep-all = Удалить все от этого отправителя
mail-sweep-keep-latest = Оставить последнее, удалить остальные
mail-sweep-older-than = Удалить старше N дней
mail-sweep-block = Удалить все и заблокировать отправителя
mail-sweep-days = Дней
mail-sweep-count = { $count ->
    [one] { $count } сообщение будет перемещено в корзину
    [few] { $count } сообщения будут перемещены в корзину
    [many] { $count } сообщений будут перемещены в корзину
   *[other] { $count } сообщения будут перемещены в корзину
}
mail-sweep-cancel = Отмена
mail-sweep-run = Очистить ({ $count })
mail-sweeping = Очистка…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Закрыть
mail-undo-action = Отменить
mail-undo-send-cancel = Отменить
mail-undo-failed = Не удалось отменить

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Метка добавлена
mail-toast-label-removed = Метка удалена
mail-toast-pinned = Закреплено
mail-toast-unpinned = Откреплено
mail-toast-snoozed = Отложено
mail-toast-follow-up-set = Отслеживание установлено
mail-toast-follow-up-cleared = Отслеживание снято
mail-toast-archived = Архивировано
mail-toast-trashed = Перемещено в корзину
mail-toast-marked-spam = Отмечено как спам
mail-toast-moved-to = Перемещено в { $folder }
mail-toast-moved = Перемещено в папку
mail-toast-archived-count = Архивировано сообщений: { $count }
mail-toast-trashed-count = Перемещено в корзину сообщений: { $count }
mail-toast-marked-spam-count = Отмечено как спам сообщений: { $count }
mail-toast-moved-count = Перемещено сообщений: { $count }
mail-toast-moved-to-count = Перемещено в { $folder } сообщений: { $count }
mail-toast-marked-read = { $count ->
    [one] { $count } сообщение отмечено как прочитанное
    [few] { $count } сообщения отмечены как прочитанные
    [many] { $count } сообщений отмечены как прочитанные
   *[other] { $count } сообщения отмечены как прочитанные
}
mail-toast-marked-unread = { $count ->
    [one] { $count } сообщение отмечено как непрочитанное
    [few] { $count } сообщения отмечены как непрочитанные
    [many] { $count } сообщений отмечены как непрочитанные
   *[other] { $count } сообщения отмечены как непрочитанные
}
mail-toast-flagged = { $count ->
    [one] { $count } сообщение помечено
    [few] { $count } сообщения помечены
    [many] { $count } сообщений помечены
   *[other] { $count } сообщения помечены
}
mail-toast-unflagged = { $count ->
    [one] Пометка снята с { $count } сообщения
    [few] Пометка снята с { $count } сообщений
    [many] Пометка снята с { $count } сообщений
   *[other] Пометка снята с { $count } сообщения
}
mail-toast-move-failed = Не удалось переместить сообщения. Ничего не перемещено.
mail-toast-update-failed = Не удалось изменить сообщения. Ничего не изменено.
mail-toast-update-refused = { $count ->
    [one] Сервер отклонил изменение для { $count } сообщения
    [few] Сервер отклонил изменение для { $count } сообщений
    [many] Сервер отклонил изменение для { $count } сообщений
   *[other] Сервер отклонил изменение для { $count } сообщения
}
mail-toast-no-archive-folder = Нет папки «Архив»
mail-toast-no-trash-folder = Нет папки «Корзина»
mail-toast-no-spam-folder = Нет папки «Спам»
mail-toast-sweep-nothing = Нечего очищать
mail-toast-swept = { $count ->
    [one] В корзину перемещено { $count } сообщение
    [few] В корзину перемещено { $count } сообщения
    [many] В корзину перемещено { $count } сообщений
   *[other] В корзину перемещено { $count } сообщения
}
mail-toast-load-more-failed = Не удалось загрузить больше сообщений
mail-toast-export-nothing = Нечего экспортировать
mail-toast-exported = Экспортировано в .eml
mail-toast-export-failed = Не удалось выполнить экспорт
mail-toast-send-queued = В очереди — будет отправлено после восстановления подключения
mail-toast-tracking-not-bound = Сообщение отправлено, но его открытия не будут записываться.
mail-toast-send-scheduled = Отправка запланирована
mail-toast-sent = Сообщение отправлено
mail-toast-send-canceled = Отправка отменена

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Лента
mail-ribbon-tab-home = Главная
mail-ribbon-tab-view = Вид
mail-ribbon-tab-folder = Папка
mail-ribbon-expand = Развернуть
mail-ribbon-collapse = Свернуть
mail-ribbon-group-new = Создать
mail-ribbon-compose = Написать
mail-ribbon-group-session = Сеанс
mail-ribbon-logout = Выйти
mail-ribbon-group-theme = Тема
mail-ribbon-group-density = Плотность
mail-ribbon-group-settings = Настройки
mail-ribbon-more = Ещё…
mail-ribbon-group-folders = Папки
mail-density-compact = компактная
mail-density-cozy = средняя
mail-density-relaxed = свободная

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Уровень безопасности сообщения
mail-maxsec-full = Полный
mail-maxsec-full-hint = Очищенный HTML с изображениями и стилями.
mail-maxsec-no-media = Без медиа
mail-maxsec-no-media-hint = Очищенный HTML без изображений и медиафайлов.
mail-maxsec-plain-text = Обычный текст
mail-maxsec-plain-text-hint = HTML не отображается — сообщение показано только как обычный текст.
