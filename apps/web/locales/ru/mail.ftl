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
mail-inbox-filter = Фильтр входящих
mail-inbox-focused = Важные
mail-inbox-other = Другие

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
mail-toast-pinned = Сообщение «{ $target }» закреплено
mail-toast-unpinned = Сообщение «{ $target }» откреплено
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
mail-feature-off = Отключено вашими настройками или администратором
mail-send-disabled = Отправка отключена вашими настройками или администратором. Черновик по-прежнему можно сохранить.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = События
mail-events-upcoming = Только предстоящие
mail-events-filter = Фильтр событий
mail-loading-more = Загрузка более старых сообщений…
mail-event-state-invitation = Приглашение
mail-event-state-updated = Обновлено
mail-event-state-cancelled = Отменено
mail-event-state-reply = Ответ
mail-event-state-counter = Предложено новое время
mail-event-state-event = Событие
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Показать другие действия
mail-remote-moved = Это сообщение перемещено в папку «{ $folder }» с другого устройства или из другого приложения. Вы по-прежнему читаете открытую копию.
mail-remote-moved-unknown = Это сообщение перемещено с другого устройства или из другого приложения. Вы по-прежнему читаете открытую копию.
mail-remote-deleted = Это сообщение удалено с другого устройства или из другого приложения. Вы можете читать эту копию, пока не закроете её.
mail-remote-dismiss = Закрыть
mail-reply-draft = Ответ в процессе
mail-reply-draft-open = Открыть черновик
mail-draft-load-failed = Не удалось открыть черновик. Повторите попытку.
mail-reply-images-omitted =
    { $count ->
        [one] Не удалось включить в ответ { $count } изображение.
        [few] Не удалось включить в ответ { $count } изображения.
        [many] Не удалось включить в ответ { $count } изображений.
       *[other] Не удалось включить в ответ { $count } изображения.
    }
mail-export-forbidden = Администратор отключил экспорт сообщений для этой учётной записи.
mail-export-open-failed = Не удалось открыть параметры экспорта. Закройте вкладку рабочей области и повторите попытку.
mail-headers-title = Исходные заголовки сообщения
mail-headers-action-help = Просмотр исходных заголовков сообщения, включая сведения о доставке и аутентификации.
mail-headers-help = Исходные заголовки сообщения. Перенесённые строки сохранены; здесь показаны только заголовки.
mail-headers-loading = Загрузка заголовков сообщения…
mail-headers-failed = Не удалось загрузить заголовки сообщения. Проверьте подключение и повторите попытку.
mail-headers-copy = Копировать заголовки
mail-headers-copied = Заголовки скопированы.
mail-headers-copy-failed = Не удалось скопировать заголовки. Выделите текст и скопируйте его вручную.
mail-attachment-failed = Не удалось скачать это вложение. Повторите попытку, когда появится подключение.
mail-compose-rt-rule = Горизонтальная линия
mail-compose-signature-new = Новые сообщения
mail-compose-signature-replies = Ответы
mail-editor-body-size = { $size } КиБ
mail-editor-body-size-label = Размер текста
mail-editor-body-size-help = Размер текущего текста в UTF-8, включая разметку HTML и встроенные изображения. Заголовки сообщения и отдельные вложения не учитываются.
mail-outbox-load-failed = Не удалось проверить «Исходящие». Нажмите «Обновить» после проверки подключения.
mail-outbox-unavailable = «Исходящие» недоступны. Нажмите «Обновить», чтобы повторить попытку.
mail-maxsec-view = Вид: { $mode }
mail-maxsec-menu = Режим просмотра
mail-toast-tab-pinned = Вкладка «{ $target }» закреплена
mail-toast-tab-unpinned = Вкладка «{ $target }» откреплена
mail-workspace-message = Сообщение
mail-opened-announcement = Открыто: { $subject }
mail-workspace-layout = Макет рабочей области
mail-workspace-tabs = Вкладки
mail-workspace-side-by-side = Рядом
mail-workspace-stack = Друг под другом
mail-workspace-refresh = Обновить папку
mail-workspace-load-error = Не удалось загрузить это сообщение. Закройте вкладку и повторите попытку.
mail-resize-folders = Изменить размер панели папок
mail-resize-list = Изменить размер списка сообщений
mail-resize-tiles = Изменить размер областей рабочей области
mail-show-folders = Показать панель папок
mail-hide-folders = Скрыть панель папок
mail-compact-folders = Компактные значки папок
mail-toggle-list = Показать или скрыть список сообщений
mail-toggle-reader = Показать или скрыть область чтения
mail-folder-counts = Непрочитанных: { $unread }, сообщений: { $total }
mail-print = Печать / Сохранить как PDF
mail-go-to-top = Наверх
mail-print-help = Распечатайте это сообщение или выберите «Сохранить как PDF» в диалоге печати. Внешние изображения остаются заблокированными.
mail-file-details = Сведения о файле и отпечаток
mail-file-type = Тип файла
mail-file-extension = Расширение
mail-file-detected = Обнаруженная сигнатура
mail-file-unknown = Сигнатура не распознана
mail-file-size = Размер
mail-file-size-bytes = { $count } Б
mail-file-hash-unavailable = Отпечаток недоступен (нужны защищённый браузер и файл не более 64 МиБ).
mail-file-fingerprint-help = Отпечаток однозначно определяет именно эти байты. Он не доказывает, что файл безопасен или кто его создал.
mail-file-warning-executable = Этот файл может запускать программы или менять параметры системы. Скачивайте его, только если доверяете источнику.
mail-file-warning-macro = Этот документ может содержать макросы. Не включайте макросы, пока не проверите отправителя и назначение.
mail-file-warning-disguisedName = Имя файла может скрывать его настоящее расширение. Перед скачиванием проверьте сведения о файле.
mail-file-warning-typeMismatch = Обнаруженная сигнатура файла не соответствует заявленному типу. Автоматический просмотр заблокирован.
mail-file-warning-activeDocument = Этот формат может содержать активное содержимое. Просмотр изолирован; скачанная копия при открытии может вести себя иначе.
mail-file-preview-blocked = Автоматический просмотр отключён для этого типа файла или при несоответствии типа.
mail-quote-header-from = От:
mail-quote-header-sent = Отправлено:
mail-quote-header-to = Кому:
mail-quote-header-cc = Копия:
mail-quote-header-subject = Тема:
mail-spam-score = Анализ спама
mail-spam-score-loading = Проверка заголовков спама…
mail-spam-score-unavailable = Оценка спама недоступна
mail-spam-score-unverified = Указано в заголовках сообщения, независимо не проверено. Заголовки, заданные отправителем, могут быть подделаны.
mail-spam-score-positive = Заголовок сообщает о спаме.
mail-spam-score-negative = Заголовок сообщает, что это не спам.
mail-spam-score-unknown = Вердикт к этой оценке не приложен.
mail-external-recipients = Внешние получатели
mail-external-count = Получателей вне доверенных доменов: { $count }
mail-external-help = Проверьте получателей в полях «Кому», «Копия» и «Скрытая копия» перед отправкой конфиденциальных сведений. Доверие определяется выбранным профилем отправителя и действующей политикой доменов.
mail-external-reviewed = Я проверил этих внешних получателей.
mail-external-confirm-required = Перед отправкой проверьте и подтвердите внешних получателей.
mail-quick-reply = Быстрый ответ
mail-quick-reply-help = Предлагать компактную вкладку ответа с уже заполненными получателями. По умолчанию выключено. Обычные проверки безопасности и параметры отправки по-прежнему действуют.
mail-quick-reply-expand = Изменить получателей и тему
mail-attachment-policy-loading = Проверка политики вложений…
mail-attachment-policy-unavailable = Не удалось проверить политику вложений. Повторите попытку перед открытием файла.
mail-attachment-policy-blocked = Администратор заблокировал это вложение или его скачивание в исходном виде.
mail-attachment-policy-warning = Администратор рекомендует соблюдать осторожность с этим вложением. Продолжайте, только если доверяете источнику.
mail-attachment-policy-acknowledge = Понимаю — продолжить
mail-attachment-policy-no-preview = Администратор отключил просмотр вложений. Скачивание оригиналов может оставаться доступным.
mail-time-sent = Отправлено
mail-time-received = Получено
mail-time-unknown = Не указано
mail-time-comparison = Время в поясе { $zone }
mail-time-privacy = Пояс для сравнения выбираете вы. Метки времени сообщений не указывают чьё-либо местоположение.
mail-time-event-comparison = Сравнение времени события
mail-time-ambiguous = Это местное время встречается дважды при переводе часов. Показаны оба возможных момента; перед сохранением проверьте предполагаемое смещение события.
mail-bulk-add-label = Добавить метку
mail-select-messages = Выбрать сообщения
mail-bulk-remove-label = Удалить метку
mail-mark-spam = Пометить как спам
mail-bulk-forward = Переслать выбранные сообщения вложениями
mail-bulk-download = Скачать выбранные сообщения как mbox
mail-bulk-forward-subject = Пересланные сообщения ({ $count })
mail-bulk-transfer-failed = Не удалось подготовить выбранные сообщения. Выберите не более 100 сообщений общим размером не более 32 МиБ и проверьте права экспорта вашей учётной записи.
mail-toast-labels-added = Метка «{ $label }» добавлена к сообщениям: { $count }
mail-toast-labels-removed = Метка «{ $label }» удалена из сообщений: { $count }
mail-action-help-reply = Ответить отправителю на новой вкладке написания.
mail-action-help-reply-all = Ответить отправителю и остальным получателям на новой вкладке написания.
mail-action-help-forward = Проверить и переслать это сообщение выбранным получателям.
mail-action-help-pin = Закрепить это сообщение для быстрого доступа или снять закрепление.
mail-action-help-read = Изменить отметку о прочтении сообщения. Изменение можно отменить.
mail-action-help-flag = Установить или снять флажок сообщения для последующей обработки.
mail-action-help-archive = Переместить это сообщение в настроенную папку архива.
mail-action-help-delete = Переместить это сообщение в корзину. Перемещение можно отменить.
mail-action-help-spam = Пометить это сообщение как нежелательное и переместить в настроенную папку спама.
mail-action-help-export = Открыть страницу экспорта, чтобы скачать это сообщение в выбранном формате.
mail-action-help-sweep = Просмотрите остальные сообщения этого отправителя и выберите, что очистить (удалить все, оставить последнее, удалить старые или заблокировать отправителя), прежде чем что-либо будет применено.
mail-empty-trash = Очистить корзину
mail-trash-help = Просмотрите и безвозвратно удалите сообщения, находящиеся сейчас в корзине. Это действие нельзя отменить.
mail-trash-preparing = Подготовка снимка сообщений в корзине…
mail-trash-prepare-failed = Не удалось подготовить просмотр корзины. Проверьте подключение и откройте его снова. Ни одно сообщение не удалено.
mail-trash-confirm = Безвозвратно удалить из корзины сообщения: { $count }? Это действие нельзя отменить. Сообщения, поступившие после этого просмотра, исключаются; сообщения, перемещённые из корзины, пропускаются.
mail-trash-limit = Эта операция охватывает до { $count } сообщений. В корзине остаются другие; просмотрите следующую операцию, чтобы удалить и их.
mail-trash-permanent = Удалить безвозвратно
mail-trash-stop = Остановить после текущего запроса
mail-trash-progress = Удалено: { $deleted }; пропущено сообщений, которых уже нет в корзине: { $skipped }; осталось в этом просмотре: { $remaining }.
mail-trash-stopped = Удаление остановлено. Подтверждённо удалено: { $deleted }; пропущено: { $skipped }; осталось: { $remaining }; результат неизвестен: { $uncertain }. Проверьте корзину, прежде чем начинать новую операцию.
mail-links-title = Ссылки в сообщении
mail-links-review = Проверить ссылки ({ $count })
mail-links-help = Ссылки в сообщении открываются только после того, как вы проверите их адрес назначения здесь. Ни один адрес не загружается автоматически.
mail-links-inert = Используйте «Проверить ссылки» над сообщением, чтобы изучить и открыть этот адрес.
mail-links-limit = Показаны первые 100 различных поддерживаемых адресов. Остальные ссылки в сообщении остаются неактивными.
mail-links-destination = Адрес для открытия
mail-links-external = Этот адрес указан отправителем сообщения. При открытии вы покинете Mailwoman; проверьте адрес, прежде чем продолжить.
mail-links-open = Открыть в новой вкладке
mail-links-compose = Проверить новое сообщение
mail-links-warning-http = Этот адрес использует незашифрованный HTTP.
mail-links-warning-address = В этом адресе вместо доменного имени используется IP-адрес.
mail-links-warning-international = Этот домен содержит интернационализированное имя. Внимательно проверьте его написание в ASCII.
mail-links-warning-mismatch = Видимый веб-адрес указывает на другой узел, чем фактический адрес назначения.
mail-links-warning-direction = Текст ссылки содержит символы, меняющие направление текста, которые могут скрывать её адрес.
mail-outbox-uncertain = Статус доставки неизвестен
mail-outbox-uncertain-note = Отправка началась, но доставку подтвердить не удалось. Это сообщение не будет отправлено повторно автоматически. Проверьте папку «Отправленные» или уточните у получателя, прежде чем отправлять снова.
mail-conversation-title = Показать переписку
mail-conversation-help = Просмотр доступных сообщений переписки этой учётной записи из всех папок. Открытие сообщения не меняет выбранную папку.
mail-conversation-count = Показано { $loaded } из { $total } сообщений переписки.
mail-conversation-other-folder = Другая папка
mail-conversation-loading = Загрузка переписки…
mail-conversation-missing = Недоступных сообщений: { $count }.
mail-conversation-error = Не удалось загрузить переписку. Повторите попытку.
mail-conversation-more = Загрузить ещё сообщения
mail-conversation-limit = В этом виде показывается не более { $count } сообщений.
mail-compose-request-delivery = Запрашивать уведомления о доставке
mail-compose-request-delivery-hint = Попросить принимающие почтовые серверы сообщать о доставке, сбое или задержке. Это не запрос и не подтверждение того, что сообщение кто-то прочитал. Возвращаемые отчёты содержат заголовки, а не полный текст.
mail-compose-delivery-unavailable = Уведомления о доставке недоступны или отключены администратором для этой учётной записи. Явно запрошенное уведомление не будет удалено без предупреждения.
mail-compose-account-changed = Это сообщение относится к другому входу. Войдите в нужную учётную запись и снова откройте черновик.
mail-context-open-message = Открыть сообщение
mail-context-open-folder = Открыть папку
mail-context-download = Скачать сообщение
mail-context-refresh = Обновить папку
mail-context-folder-settings = Настройки папки
mail-context-failed = Не удалось выполнить действие. Повторите попытку.
mail-sharing-unavailable = Общий доступ к папке недоступен, либо у вас больше нет прав на управление им.
mail-trash-cancelling = Остановка после текущего запроса. Подтверждённо удалено: { $deleted }; пропущено: { $skipped }; осталось: { $remaining }. Уже принятые удаления отменить нельзя.
mail-trash-already-running = Удаление из корзины уже выполняется. Используйте его уведомление о ходе, чтобы остановить следующие пакеты.
mail-context-search-sender = Искать этого отправителя
mail-context-search-subject = Искать ту же тему
mail-drag-too-many = Перетаскивайте не более { $count } сообщений за раз. Перед перемещением выберите меньшую группу.
mail-subtab-reorder-help = Перетащите, чтобы изменить порядок вкладок, или нажмите Ctrl+Shift+стрелка влево либо вправо на активной вкладке.
mail-compose-ribbon = Лента редактора
mail-compose-ribbon-format = Форматирование текста
mail-compose-ribbon-insert = Вставка
mail-compose-font = Шрифт
mail-compose-font-size = Размер шрифта
mail-compose-font-inherit = По умолчанию
mail-compose-undo = Отменить
mail-compose-redo = Повторить
mail-compose-undo-tip = Отменить ({ $shortcut })
mail-compose-redo-tip = Повторить ({ $shortcut })
mail-workspace-limit = Закройте вкладку, прежде чем открывать другую. Можно держать открытыми до 32 вкладок.
mail-compose-private-storage-unavailable = Локальное восстановление черновика недоступно в этом сеансе. Не закрывайте этот редактор, пока не отправите или не скопируете сообщение.

# -- Machine translation (2026-11): added to match en ---------------------
mail-offline-storage-unavailable = Изменения офлайн недоступны, пока этот сеанс не подтвердит права на хранилище. Подключитесь снова и войдите ещё раз.
mail-draft-status-idle = Ещё не сохранено в учётной записи
mail-draft-status-pending = Несохранённые изменения
mail-draft-status-saving = Сохранение в «Черновики»…
mail-draft-status-saved = Сохранено в «Черновики» учётной записи
mail-draft-status-error = Черновик не сохранён. Локальная копия для восстановления остаётся доступной.
mail-draft-status-uncertain = Подтверждение сохранения отсутствует. Проверьте «Черновики», прежде чем создавать ещё одну копию; автоматическое сохранение приостановлено.
mail-draft-status-conflict = Этот черновик изменён на другом устройстве. Откройте его копию из учётной записи заново, прежде чем продолжить.
mail-draft-status-unsupported = Эта учётная запись сейчас не может подтверждать сохранение в папку «Черновики».
mail-draft-protected = Зашифрованное или расшифрованное цитируемое содержимое не сохраняется открытым текстом. Прежний черновик в учётной записи остаётся, пока вы явно его не удалите.
mail-draft-save-now = Сохранить в «Черновики»
mail-draft-save-failed = Не удалось сохранить этот черновик в учётной записи. Проверьте редактор, прежде чем закрывать его.
mail-draft-uncertain = Возможно, черновик сохранён. Автоматические повторы приостановлены; проверьте папку «Черновики».
mail-draft-conflict = Другое устройство изменило этот черновик. Ваши правки не перезаписали его копию.
mail-draft-cleanup-pending = Новый черновик сохранён, но его прежнюю копию удалить не удалось. Проверьте «Черновики», прежде чем сохранять снова.
mail-draft-account-list = Черновики учётной записи
mail-draft-loading = Загрузка черновиков учётной записи…
mail-draft-close-recovery = Закрыть и оставить только локальную копию для восстановления
mail-draft-discard-account = Удалить
mail-draft-discard-pending = Не удалось подтвердить удаление черновика. Его запись в учётной записи остаётся; повторите «Удалить» после проверки папки «Черновики».
mail-draft-limit = В этой учётной записи уже 100 сохранённых черновиков. Удалите ненужный черновик, прежде чем сохранять новый.
mail-draft-quota-help = Хранится до 100 черновиков учётной записи. При удалении сначала удаляется подтверждённая копия на сервере, затем освобождается место.
mail-draft-discard-confirm = Подтвердите удаление черновика
mail-draft-discard-confirm-detail = Безвозвратно удалить «{ $subject }» из «Черновиков» учётной записи? Если закрыть редактор, черновик сохранится.
mail-draft-discard-title = Удалить этот черновик?
mail-draft-keep-editing = Продолжить редактирование
mail-draft-discard-action = Удалить
mail-draft-close-title = Отменить несохранённые изменения?
mail-draft-close-protected = Это сообщение содержит зашифрованное или расшифрованное содержимое, поэтому не сохраняется как черновик. При закрытии эти изменения будут отброшены. Прежний черновик учётной записи, если он есть, остаётся в «Черновиках».
mail-draft-close-unstored = В этом сеансе черновики сохранить нельзя, поэтому при закрытии сообщение будет удалено.
mail-workspace-view-mode = Расположение видов
mail-workspace-tabs-help = Показывать один активный вид, не закрывая остальные вкладки.
mail-workspace-side-by-side-help = Показывать два выбранных вида рядом. Перетащите вкладку на любой вид, чтобы заменить его.
mail-workspace-stack-help = Показывать два выбранных вида один над другим. Перетащите вкладку на любой вид, чтобы заменить его.
mail-workspace-masonry-side-by-side = Мозаика рядом
mail-workspace-masonry-side-by-side-help = Расположить все открытые виды в смещённых столбцах. На малых экранах показывается активная вкладка.
mail-workspace-masonry-stack = Мозаика друг под другом
mail-workspace-masonry-stack-help = Расположить все открытые виды в смещённых строках. На малых экранах показывается активная вкладка.
mail-workspace-view-first = Первый вид
mail-workspace-view-second = Второй вид
mail-workspace-view-empty = Перетащите сюда открытую вкладку, чтобы выбрать этот вид.
mail-popup-blocked = Разрешите всплывающие окна для Mailwoman, чтобы отделить эту вкладку. Вкладка остаётся здесь.
mail-popup-unavailable = Не удалось загрузить отдельное окно. Вкладка и несохранённые изменения остаются в главном окне. Повторите попытку после обновления приложения.
mail-popup-reattach = Вернуть в главное окно
mail-popup-action-reattached = Возврат в главное окно для завершения этого действия.
mail-popup-detached = Эта вкладка открыта в отдельном окне. Закрытие того окна вернёт её сюда.
mail-compose-from-custom = Другой разрешённый адрес…
mail-compose-from-alias = Адрес отправителя для этого сообщения
mail-compose-from-alias-help = Используйте псевдоним, с которого вам разрешено отправлять. Меняется только этот черновик; разрешение проверяется перед отправкой.
mail-compose-from-refused = Этот адрес не разрешён для отправки из этой учётной записи.
mail-compose-from-unavailable = Не удалось проверить адрес отправителя. Сообщение не отправлено.
mail-compose-from-invalid = Введите один действительный адрес электронной почты отправителя.
mail-compose-from-toggle = Показать или скрыть отправителя для этого сообщения
mail-compose-ribbon-security = Безопасность
mail-compose-ribbon-delivery = Доставка
mail-compose-ribbon-recipients = Получатели
mail-compose-ribbon-message-type = Тип сообщения
mail-compose-recall-help = Отмените сообщения, которые ещё удерживаются для отмены отправки или запланированной доставки. Уже доставленные сообщения отозвать нельзя.
mail-compose-recall-loading = Проверка сообщений в очереди…
mail-compose-recall-empty = Нет сообщений в очереди, которые можно отменить.
mail-compose-recall-failed = Не удалось подтвердить состояние очереди. Закройте и снова откройте эту панель, чтобы обновить её, прежде чем выполнять другое действие.
mail-compose-importance = Важность
mail-compose-importance-high = Высокая
mail-compose-importance-normal = Обычная
mail-compose-importance-low = Низкая
mail-insert-account-title = Вставить элемент учётной записи
mail-insert-account-help = Прикрепить экспорт календаря, визитную карточку контакта или исходное сообщение из этой учётной записи.
mail-insert-account-close = Закрыть выбор элемента учётной записи
mail-insert-account-kind = Тип элемента
mail-insert-account-event = Событие календаря (.ics)
mail-insert-account-calendar = Календарь (.ics)
mail-insert-account-contact = Визитная карточка контакта (.vcf)
mail-insert-account-message = Исходное сообщение (.eml)
mail-insert-account-search = Фильтр по списку
mail-insert-account-selection = Выберите элемент
mail-insert-account-scope = Показаны только элементы текущей учётной записи. Для исходных сообщений действует политика экспорта сервера.
mail-insert-account-limited = Показано до 100 элементов. Фильтр ищет только в этом списке.
mail-insert-account-unavailable = Поддерживаемых экспортов учётной записи нет.
mail-insert-account-empty = Подходящих элементов нет.
mail-insert-account-attach = Прикрепить выбранный элемент
mail-insert-account-loading = Загрузка элемента учётной записи…
mail-insert-account-error = Не удалось прикрепить элемент. Проверьте доступ и политику экспорта и повторите попытку. Для больших календарей может понадобиться экспорт отдельных событий.
mail-compose-html-source = Исходный код HTML
mail-editor-clear-format = Очистить форматирование
mail-compose-format-html = HTML
mail-compose-html-visual = Визуальный редактор
mail-compose-html-help = Редактируйте HTML напрямую. Сохраняется и отправляется только поддерживаемое форматирование почты; скрипты, внешние изображения и небезопасная разметка удаляются.
mail-compose-html-processing = Подготовка безопасного HTML…
mail-compose-html-failed = Не удалось подготовить HTML. Отредактируйте его или повторите попытку перед сохранением или отправкой.
mail-compose-check-names = Проверить имена
mail-compose-check-names-help = Сопоставить точные имена с загруженными контактами и проверить формат адресов. Доставка в почтовый ящик при этом не проверяется.
mail-compose-names-unresolved = Выберите полный адрес электронной почты для этих нераспознанных или неоднозначных имён: { $names }
mail-compose-names-checked = Формат получателей проверен; распознано имён контактов: { $count }. Доставка в почтовый ящик не проверена.
mail-compose-ribbon-table = Таблица
mail-compose-group-clipboard = Буфер обмена
mail-compose-group-paragraph = Абзац
mail-compose-group-editing = Редактирование
mail-compose-group-view = Вид
mail-compose-dark-background = Тёмный фон
mail-compose-dark-background-help = Показывать сообщение на тёмном фоне во время написания. Меняется только ваш экран: сообщение отправляется без цвета фона.
mail-compose-group-assist = Помощник по письму
mail-editor-counts = Статистика редактора
mail-editor-words = { $count ->
    [one] { $count } слово
    [few] { $count } слова
    [many] { $count } слов
   *[other] { $count } слова
}
mail-editor-paragraphs = { $count ->
    [one] { $count } абзац
    [few] { $count } абзаца
    [many] { $count } абзацев
   *[other] { $count } абзаца
}
mail-editor-characters = { $count ->
    [one] { $count } символ
    [few] { $count } символа
    [many] { $count } символов
   *[other] { $count } символа
}
mail-editor-selection-required = Выделите подходящий текст или ячейки таблицы, прежде чем использовать это действие.
mail-editor-clipboard-unavailable = Доступ к буферу обмена недоступен. Используйте сочетание «Копировать», «Вырезать» или «Вставить» в браузере.
mail-editor-selection-changed = Выделение изменилось. Выделите нужный текст и повторите попытку.
mail-editor-clipboard = Буфер обмена и форматирование
mail-editor-copy = Копировать
mail-editor-cut = Вырезать
mail-editor-paste = Вставить
mail-editor-copy-format = Копировать формат
mail-editor-apply-format = Применить формат
mail-editor-style = Стиль абзаца
mail-editor-text-color = Цвет шрифта
mail-editor-clear-color = Цвет по умолчанию
mail-editor-highlight = Выделение цветом
mail-editor-clear-highlight = Убрать выделение цветом
mail-editor-paragraph = Параметры абзаца
mail-editor-alignment = Выравнивание
mail-editor-line-height = Высота строки
mail-editor-space-before = Интервал перед
mail-editor-space-after = Интервал после
mail-editor-indent = Отступ
mail-editor-shading = Заливка абзаца
mail-editor-clear-shading = Убрать заливку
mail-editor-case = Изменить регистр
mail-editor-sort = Сортировать выделенные абзацы
mail-editor-sort-alpha = По алфавиту
mail-editor-sort-numeric = По числовому порядку
mail-editor-table-rows = Строки таблицы
mail-editor-table-columns = Столбцы таблицы
mail-editor-table-insert = Вставить таблицу
mail-editor-table-limits = До 20 строк, 10 столбцов и 200 ячеек в таблице; не более 10 таблиц. Вложенные таблицы не поддерживаются.
mail-editor-table-row-before = Строка выше
mail-editor-table-row-after = Строка ниже
mail-editor-table-row-delete = Удалить строку
mail-editor-table-column-before = Столбец слева
mail-editor-table-column-after = Столбец справа
mail-editor-table-column-delete = Удалить столбец
mail-editor-table-merge = Объединить ячейки
mail-editor-table-split = Разделить ячейку
mail-editor-table-header = Переключить строку заголовка
mail-editor-table-delete = Удалить таблицу
mail-editor-table-shading = Заливка ячейки
mail-editor-find-replace = Найти и заменить
mail-editor-find = Найти текст
mail-editor-replacement = Заменить на
mail-editor-match-case = Учитывать регистр
mail-editor-previous = Предыдущее совпадение
mail-editor-next = Следующее совпадение
mail-editor-replace = Заменить выбранное совпадение
mail-editor-replace-all = Заменить все совпадения
mail-editor-matches = Совпадений: { $count } (не более 1000)
mail-editor-select-all = Выделить всё
mail-editor-clear-selection = Снять выделение
mail-editor-history = История отмены
mail-editor-undo-groups = Отменить последние группы изменений: { $count }
mail-editor-zoom = Масштаб редактора
mail-editor-markers = Знаки абзацев
mail-editor-style-normal = Обычный
mail-editor-style-no-spacing = Без интервала
mail-editor-style-title = Заголовок
mail-editor-style-subtitle = Подзаголовок
mail-editor-style-heading1 = Заголовок 1
mail-editor-style-heading2 = Заголовок 2
mail-editor-style-heading3 = Заголовок 3
mail-editor-style-heading4 = Заголовок 4
mail-editor-style-heading5 = Заголовок 5
mail-editor-style-heading6 = Заголовок 6
mail-editor-align-left = По левому краю
mail-editor-align-center = По центру
mail-editor-align-right = По правому краю
mail-editor-align-justify = По ширине
mail-editor-case-lower = строчные
mail-editor-case-upper = ПРОПИСНЫЕ
mail-editor-case-title = Каждое Слово С Прописной
mail-editor-case-sentence = Как в предложениях
mail-editor-action-history = История: { $action }
mail-editor-history-empty = Нет доступных действий
mail-editor-redo-groups = Повторить изменения: { $count }
mail-compose-identities-failed = Не удалось загрузить профили отправки. Закройте и снова откройте этот черновик, чтобы повторить попытку.
mail-folders-loading = Загрузка папок…
mail-initial-sync-error = Пока не удалось синхронизировать ваши сообщения.
mail-initial-sync-error-help = Соединение с почтовым сервером прервано. Повторите попытку или попросите администратора проверить журналы подключений, если проблема повторяется.
mail-links-trusted = Доверенный домен: открывается без проверки
mail-links-trust-domain = Доверять { $domain }
mail-links-trust-failed = Не удалось сохранить доверенный домен. Повторите попытку.
mail-focus-always-focused = Всегда перемещать в «Важные»
mail-focus-always-other = Всегда перемещать в «Остальные»
mail-focus-moved-focused = Сообщения от { $sender } теперь попадают в «Важные».
mail-focus-moved-other = Сообщения от { $sender } теперь попадают в «Остальные».
mail-focus-save-failed = Не удалось сохранить выбор «Важных входящих» для этого отправителя. Повторите попытку.
mail-context-search-domain = Искать сообщения с этого домена

# -- Machine translation (2026-11): added to match en ---------------------
mail-loading-mailbox = Загрузка: { $name }…
mail-user-folders = Пользовательские папки
mail-context-hide-folder = Скрыть с боковой панели
mail-folder-fallback = папка
mail-toast-moving = { $count ->
    [one] Перемещение { $count } сообщения в папку «{ $folder }»…
    [few] Перемещение { $count } сообщений в папку «{ $folder }»…
    [many] Перемещение { $count } сообщений в папку «{ $folder }»…
   *[other] Перемещение { $count } сообщения в папку «{ $folder }»…
}
mail-toast-move-queueing = { $count ->
    [one] Постановка в очередь { $count } сообщения для перемещения в папку «{ $folder }»…
    [few] Постановка в очередь { $count } сообщений для перемещения в папку «{ $folder }»…
    [many] Постановка в очередь { $count } сообщений для перемещения в папку «{ $folder }»…
   *[other] Постановка в очередь { $count } сообщения для перемещения в папку «{ $folder }»…
}
mail-toast-move-queued = { $count ->
    [one] В очереди { $count } сообщение для перемещения в папку «{ $folder }»
    [few] В очереди { $count } сообщения для перемещения в папку «{ $folder }»
    [many] В очереди { $count } сообщений для перемещения в папку «{ $folder }»
   *[other] В очереди { $count } сообщения для перемещения в папку «{ $folder }»
}
mail-toast-move-partial = Перемещено { $count } из { $total } сообщений в папку «{ $folder }»; не удалось переместить: { $failed }
mail-send-copy-folder-unavailable = Выбранная папка для копий отправленных недоступна. Выберите другую папку перед отправкой.
mail-send-copy-failed = Сообщение принято, но его копию в отправленных сохранить не удалось. Не отправляйте его повторно.
mail-compose-attach-public-key = Прикрепить мой открытый ключ или сертификат
mail-compose-include-hashes = Включить хеши SHA-256 прикреплённых файлов
mail-compose-hashes-pending = Вычисление хешей вложений. Отправьте, когда будет готово.
mail-compose-hashes-failed = Не удалось проверить байты вложений. Отключите хеши или повторите попытку, включив их снова.
mail-compose-crypto-pending = Шифрование обновляется. Подождите перед отправкой.
mail-compose-public-key-unavailable = Ни один действующий открытый ключ или сертификат не соответствует адресу отправителя.
mail-compose-sent-copy = Сохранять отправленное сообщение в
mail-compose-sent-copy-default = По умолчанию для учётной записи
folder-create = Создать папку
folder-name = Название папки
folder-parent = Родительская папка
folder-top-level = Верхний уровень
folder-created = Папка создана.
folder-create-unavailable = Создание папок недоступно для этой учётной записи или родительской папки.
folder-create-invalid = Выберите допустимое уникальное название папки без разделителей иерархии и управляющих символов.
folder-create-failed = Не удалось создать папку. Обновите страницу и повторите попытку.
folder-cleanup = Очистить папку…
folder-cleanup-title = Очистка папки «{ $name }»
folder-cleanup-help = Найти сообщения старше выбранного возраста. Проверьте количество перед удалением. Подпапки не затрагиваются.
folder-cleanup-age = Старше
folder-age-days = Дни
folder-age-weeks = Недели
folder-age-months = Месяцы
folder-age-years = Годы
folder-cleanup-preview = Предпросмотр очистки
folder-cleanup-confirm = Удалить найденные сообщения
folder-cleanup-permanent = Безвозвратно удалить найденные сообщения: { $count }. Это действие нельзя отменить.
folder-cleanup-trash = Переместить найденные сообщения в корзину: { $count }.
folder-cleanup-before = Получено до { $date }
folder-cleanup-done = Очищено сообщений: { $count }.
mail-from = Отправитель
mail-subject = Тема
mail-date = Дата
mail-cancel = Отмена
mail-bulk-selected = Выбрано переписок: { $count }
mail-bulk-select-folder = Выбрать всё в папке «{ $folder }»
mail-list-order = Порядок сообщений
mail-list-order-newest = Сначала новые
mail-list-order-oldest = Сначала старые
mail-list-order-sender = Отправитель
mail-list-order-subject = Тема
mail-list-order-error = Не удалось сохранить порядок сообщений.
mail-bulk-selection-load-failed = Не удалось выбрать всю папку. Проверьте подключение и повторите попытку.
mail-privacy-bar = Конфиденциальность сообщения
mail-privacy-trackers-allow = Разрешить трекеры для этого сообщения
mail-privacy-trackers-block = Снова блокировать трекеры
mail-privacy-images-loading = Показано изображений: { $loaded }/{ $total } · загружается: { $pending }
mail-privacy-images-failed = Показано изображений: { $loaded }/{ $total } · ошибок: { $failed }
mail-privacy-images-loaded = Показано изображений: { $loaded }/{ $total }
mail-privacy-images-loading-failed = Показано изображений: { $loaded }/{ $total } · загружается: { $pending } · ошибок: { $failed }
mail-reader-folder-title = Папка: { $folder }
mail-reader-move-to = Переместить в…
mail-action-help-move = Переместить это сообщение в другую папку.
folder-ops-new-subfolder = Новая подпапка
folder-ops-mark-read = Отметить всё как прочитанное
folder-ops-this-folder = Эта папка
folder-ops-with-subfolders = Включая подпапки
folder-ops-no-unread = Непрочитанных сообщений нет.
folder-ops-read-unavailable = Вы не можете изменять сообщения в этой папке.
folder-ops-more = Ещё
folder-ops-system-folder = Системную папку нельзя изменить таким способом.
folder-ops-sort-subfolders = Сортировать подпапки по
folder-ops-sort-folders = Сортировать папки по
folder-ops-sort-name = Названию
folder-ops-sort-unread = Числу непрочитанных
folder-ops-sort-total = Общему числу
folder-ops-sort-date = Последней активности
folder-ops-sort-custom = Свой порядок
folder-ops-sort-asc = По возрастанию
folder-ops-sort-desc = По убыванию
folder-ops-delete-with-subfolders = Удалить папку и подпапки
folder-ops-delete-unavailable = Эту папку нельзя удалить.
folder-ops-delete-tree-body = Будут безвозвратно удалены папки ({ $folders }) и сообщения в них ({ $count }). Это действие нельзя отменить.
folder-ops-copy-all-menu = Копировать все элементы в…
folder-ops-move-all-menu = Переместить все элементы в…
folder-ops-move-folder-menu = Переместить папку в…
folder-ops-move-up = Переместить вверх
folder-ops-move-down = Переместить вниз
folder-ops-move-unavailable = Вы не можете переместить эту папку или её элементы.
folder-ops-move-duplicate = В целевом расположении уже есть папка с таким названием.
folder-ops-move-folder-title = Переместить «{ $name }»
folder-ops-move-folder-body = Папка переместится вместе с подпапками и сообщениями. Выберите папку, в которую её поместить, или верхний уровень.
folder-ops-move-here = Переместить сюда
folder-ops-moved-folder = Папка «{ $name }» перемещена.
folder-ops-choose-folder = Выберите целевую папку.
folder-ops-destination = Целевая папка
folder-ops-include-subfolders = Включить сообщения из подпапок: { $count }
folder-ops-copy-all-title = Копирование всех элементов папки «{ $name }»
folder-ops-move-all-title = Перемещение всех элементов папки «{ $name }»
folder-ops-copy-all-body = Добавляет сообщения ({ $count }) в целевую папку. Они остаются на прежнем месте.
folder-ops-move-all-body = Перемещает сообщения ({ $count }) в целевую папку. Потом это можно отменить.
folder-ops-copy-all = Копировать элементы
folder-ops-move-all = Переместить элементы
folder-ops-undo = Отменить
folder-ops-stop = Остановить
folder-ops-marking = Отметка папки «{ $name }» как прочитанной…
folder-ops-marking-progress = Отметка папки «{ $name }» как прочитанной: { $done } из { $total }
folder-ops-marked = Отмечено как прочитанные сообщений в папке «{ $name }»: { $count }.
folder-ops-copying = Копирование: «{ $name }»…
folder-ops-copying-progress = Копирование «{ $name }»: { $done } из { $total }
folder-ops-copied = Скопировано сообщений из папки «{ $name }»: { $count }.
folder-ops-moving = Перемещение: «{ $name }»…
folder-ops-moving-progress = Перемещение «{ $name }»: { $done } из { $total }
folder-ops-moved = Перемещено сообщений из папки «{ $name }»: { $count }.
folder-ops-stopped = Операция с папкой «{ $name }» остановлена после { $count } из { $total } сообщений.
folder-ops-partial = В папке «{ $name }» обработано { $count } из { $total } сообщений; отклонено: { $refused }.
folder-ops-failed = Не удалось завершить операцию с папкой «{ $name }». Обновите страницу и повторите попытку.
folder-ops-error-forbidden = У вас нет прав на это изменение.
folder-ops-error-exists = Папка с таким названием там уже существует.
folder-ops-error-mailboxHasEmail = В папке ещё есть сообщения.
folder-ops-error-notFound = Папки больше не существует.
folder-ops-error-failed = Не удалось внести изменение. Обновите страницу и повторите попытку.
folder-ops-expand = Показать подпапки папки «{ $name }»
folder-ops-collapse = Скрыть подпапки папки «{ $name }»
mail-context-reply = Ответить
mail-context-reply-all = Ответить всем
mail-context-forward = Переслать
mail-context-quick-print = Быстрая печать
mail-context-copy = Копировать
mail-context-copy-to-folder = Копировать сообщение в папку…
mail-context-copy-sender = Копировать адрес отправителя
mail-context-copy-subject = Копировать тему
mail-context-copy-link = Копировать ссылку на сообщение
mail-context-copied = Скопировано.
mail-context-copy-failed = Не удалось скопировать в буфер обмена.
mail-context-print-failed = Не удалось распечатать сообщение.
mail-reply-images-title = Изображения из исходного сообщения
mail-reply-images-ask = Включить в этот ответ изображения исходного сообщения?
mail-reply-images-include = Включить изображения
mail-reply-images-not-now = Не сейчас
mail-reply-images-trust = Всегда доверять домену { $domain }
mail-reply-images-loading = Добавление изображений…
mail-reply-images-cancelled = Загрузка отменена. Изображения не добавлены.
mail-reply-images-failed = Не удалось добавить изображения.
mail-reply-images-retry = Загрузить снова
mail-image-cancel = Отмена
mail-compose-from-why-match = Отправка от имени «{ $identity }», так как сообщение было отправлено на адрес { $address }.
mail-compose-from-why-match-plus = Отправка от имени «{ $identity }», так как сообщение было отправлено на адрес { $address }, являющийся его адресом с плюсом.
mail-compose-from-why-ambiguous = Адресу { $address } соответствует несколько профилей. Отправка от имени «{ $identity }»; выберите другой:
mail-compose-from-why-default = Отправка от имени вашего профиля по умолчанию «{ $identity }», так как ни один не соответствует исходным получателям.
mail-compose-from-why-forced-default = Отправка от имени вашего профиля по умолчанию «{ $identity }», как задано в настройках.
mail-compose-from-why-keep = Отправка от имени «{ $identity }».
mail-compose-from-change = Изменить
reader-attachments-reason-encrypted = Это сообщение зашифровано. Его вложения появятся после разблокировки.
reader-attachments-reason-list-missing = Не удалось загрузить список вложений этого сообщения, поэтому панель вложений скрыта.
reader-attachments-reason-inline-only = Вложения этого сообщения ({ $count }) — встроенные изображения, показанные в самом сообщении.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] У { $count } вложения этого сообщения нет содержимого для скачивания, поэтому панель вложений скрыта.
    [few] У { $count } вложений этого сообщения нет содержимого для скачивания, поэтому панель вложений скрыта.
    [many] У { $count } вложений этого сообщения нет содержимого для скачивания, поэтому панель вложений скрыта.
   *[other] У { $count } вложения этого сообщения нет содержимого для скачивания, поэтому панель вложений скрыта.
  }
reader-attachments-reason-policy-blocked = Политика вложений блокирует скачивание { $count ->
    [one] этого вложения.
    [few] этих вложений.
    [many] этих вложений.
   *[other] этих вложений.
  }
reader-attachments-reason-policy-unavailable = Не удалось проверить политику вложений, поэтому вложения пока нельзя открыть.
mail-drafts-popup-count = { $count ->
    [one] { $count } черновик
    [few] { $count } черновика
    [many] { $count } черновиков
   *[other] { $count } черновика
  }
mail-drafts-popup-search = Поиск по теме или получателю
mail-drafts-popup-tab-local = Локальные черновики
mail-drafts-popup-local-hint = Локальные черновики хранятся только в этом браузере.
mail-drafts-popup-badge-account = Учётная запись
mail-drafts-popup-badge-local = Локальный
mail-drafts-popup-current = Открыт сейчас
mail-drafts-popup-select = Выбрать
mail-drafts-popup-select-all = Выбрать все
mail-drafts-popup-clear = Сбросить
mail-drafts-popup-done = Готово
mail-drafts-popup-selected = Выбрано: { $count }
mail-drafts-popup-select-row = Выбрать: { $subject }
mail-drafts-popup-open = Открыть
mail-drafts-popup-discard = Удалить
mail-drafts-popup-discard-selected = Удалить выбранные
mail-drafts-popup-more = Другие действия для: { $subject }
mail-drafts-popup-retry = Повторить
mail-drafts-popup-load-failed = Не удалось загрузить черновики учётной записи.
mail-drafts-popup-empty-account = Нет черновиков учётной записи
mail-drafts-popup-no-match = Нет черновиков, подходящих под «{ $query }»
mail-drafts-popup-discarding = Удаление…
mail-drafts-popup-row-failed = Не удалось удалить. Его копия в учётной записи остаётся.

# -- Machine translation (2026-11): added to match en ---------------------
mail-drafts-popup-confirm-title = { $count ->
    [one] Удалить этот черновик?
    [few] Удалить { $count } черновика?
    [many] Удалить { $count } черновиков?
   *[other] Удалить { $count } черновика?
  }
mail-drafts-popup-confirm-account = Черновики учётной записи сначала удаляются из папки «Черновики» в учётной записи, а их место освобождается только после подтверждения удаления.
mail-drafts-popup-confirm-local = Локальные черновики удаляются из этого браузера.
mail-drafts-popup-announce-done = { $count ->
    [one] Удалён { $count } черновик
    [few] Удалено { $count } черновика
    [many] Удалено { $count } черновиков
   *[other] Удалено { $count } черновика
  }
mail-drafts-popup-announce-failed = Удалено: { $done }, не удалось удалить: { $failed }
mail-check-title = Проверьте перед отправкой
mail-check-intro = Стоит ещё раз взглянуть на следующее:
mail-check-invalid = Некоторые адреса получателей выглядят недействительными: { $list }
mail-check-subject = Тема пуста.
mail-check-attachment = В сообщении упоминается вложение, но ничего не прикреплено.
mail-check-external = Получатели вне доверенных доменов: { $list }
mail-check-send-anyway = Всё равно отправить
mail-check-review = Проверить
mail-check-remember = Больше не спрашивать об этом
mail-check-denied-invalid = Отправка заблокирована вашими настройками. Исправьте недействительные адреса получателей: { $list }
mail-check-denied-subject = Отправка заблокирована вашими настройками. Добавьте тему.
mail-check-denied-attachment = Отправка заблокирована вашими настройками. Прикрепите упомянутый файл или измените текст сообщения.
mail-check-denied-external = Отправка заблокирована вашими настройками. Вне доверенных доменов: { $list }
mail-alerts-label = Проблемы с сообщением
mail-alert-invalid = Недействительный адрес: { $address }
mail-alert-subject = Без темы
mail-recipient-chip-remove = Удалить: { $address }
mail-recipient-chips = Получатели в поле «{ $field }»
mail-recipient-invalid-title = Недействительный адрес электронной почты
mail-recipient-chip-name = { $name }, { $address }. Enter — сведения о контакте, Backspace или F2 — изменить, Delete — удалить
mail-recipient-chip-name-invalid = { $address }, недействительный адрес. Enter — изменить, Delete — удалить
mail-recipient-added = Добавлен: { $name }
mail-recipient-removed = Удалён: { $name }
mail-recipient-removed-many = { $count ->
    [one] Удалён { $count } получатель
    [few] Удалено { $count } получателя
    [many] Удалено { $count } получателей
   *[other] Удалено { $count } получателя
}
mail-recipient-editing = Изменение: { $name }
mail-recipient-all-selected = { $count ->
    [one] Выбран { $count } получатель
    [few] Выбрано { $count } получателя
    [many] Выбрано { $count } получателей
   *[other] Выбрано { $count } получателя
}
mail-external-line = Вне доверенных доменов: { $count } ({ $domains })
mail-external-dismiss = Закрыть
mail-status-label = Состояние редактора
mail-status-more = Ещё уведомлений: { $count }
mail-status-more-label = Другие уведомления
mail-status-no-recipients = Нет получателей
mail-status-no-attachment = Нет вложения
mail-status-external = Внешние: { $count }
mail-status-external-reviewed = Внешние получатели проверены
mail-status-external-review = Отметить как проверенных
mail-status-from-match = От: { $identity }
mail-status-from-match-plus = От: { $identity }
mail-status-from-ambiguous = От: { $identity } (подходит несколько)
mail-status-from-default = От: профиль по умолчанию
mail-status-from-forced-default = От: профиль по умолчанию
mail-status-from-keep = От: { $identity }
mail-status-held = В очереди: { $count }
mail-status-storage = Нет локального восстановления
mail-status-protected = Не сохранено открытым текстом
mail-status-saving = Сохранение…
mail-status-saved = Сохранено { $time }
mail-status-saved-full = Сохранено в «Черновики» учётной записи · { $time }
mail-status-not-saved = Черновик не сохранён
mail-status-retry = Повторить
mail-status-conflict = Изменено на другом устройстве
mail-status-uncertain = Сохранение не подтверждено
mail-status-close-recovery = Закрыть, оставить копию для восстановления
mail-status-unsupported = Сохранение черновика не подтверждено
mail-compose-header = Заголовок сообщения
mail-header-label = Заголовок сообщения
mail-header-collapse = Свернуть заголовок сообщения
mail-header-expand = Развернуть заголовок сообщения: { $summary }
mail-header-from-chip = От { $address }. Изменить отправителя
mail-compose-group-timing = Время отправки
mail-compose-group-priority = Приоритет
mail-compose-group-receipts = Уведомления и отслеживание
mail-compose-group-keys-files = Ключи и файлы
mail-compose-group-sent-copy = Копия отправленного
mail-editor-style-quote = Цитата
mail-editor-style-code = Код
mail-compose-ribbon-table-layout = Макет таблицы
mail-compose-ribbon-picture = Рисунок
mail-editor-table-group-rows = Строки
mail-editor-table-group-columns = Столбцы
mail-editor-table-group-cells = Ячейки
mail-editor-table-row-height = Высота строки
mail-editor-table-column-width = Ширина столбца
mail-editor-table-size-auto = Авто
mail-editor-table-padding = Поля ячейки
mail-editor-table-padding-compact = Узкие
mail-editor-table-padding-normal = Обычные
mail-editor-table-padding-roomy = Широкие
mail-editor-table-align-left = Выровнять ячейку по левому краю
mail-editor-table-align-center = Выровнять ячейку по центру
mail-editor-table-align-right = Выровнять ячейку по правому краю
mail-picture-group-size = Размер
mail-picture-group-arrange = Расположение
mail-picture-group-text = Текст
mail-picture-group-style = Стиль
mail-picture-group-edit = Правка
mail-picture-width = Ширина
mail-picture-height = Высота
mail-picture-lock = Сохранять пропорции
mail-picture-reset-size = Исходный размер
mail-picture-align-inline = В тексте
mail-picture-align-left = По левому краю
mail-picture-align-center = По центру
mail-picture-align-right = По правому краю
mail-picture-alt = Замещающий текст
mail-picture-title = Название
mail-picture-border = Граница
mail-picture-radius = Скругление углов
mail-picture-none = Нет
mail-picture-round = Круглые
mail-picture-rotate-ccw = Повернуть влево
mail-picture-rotate-cw = Повернуть вправо
mail-picture-flip-h = Отразить по горизонтали
mail-picture-flip-v = Отразить по вертикали
mail-picture-crop = Обрезать
mail-picture-crop-left = Слева (%)
mail-picture-crop-top = Сверху (%)
mail-picture-crop-right = Справа (%)
mail-picture-crop-bottom = Снизу (%)
mail-picture-crop-apply = Обрезать
mail-picture-replace = Заменить
mail-picture-remove = Удалить
mail-picture-failed-no-image = Сначала выберите рисунок.
mail-picture-failed-no-source = У этого рисунка нет локальной копии для правки. Замените его.
mail-picture-failed-failed = Не удалось изменить рисунок.
mail-context-download-attachments = Скачать вложения
mail-context-download-named = Скачать { $name }
mail-context-download-all = Скачать все как ZIP
mail-context-loading-attachments = Загрузка вложений…
mail-context-attachments-failed = Не удалось загрузить вложения.
mail-context-searching = Поиск сообщения…
mail-context-go-reply = Перейти к ответу
mail-context-go-forward = Перейти к пересылке
mail-context-reply-not-found = Ответ не найден
mail-context-forward-not-found = Пересылка не найдена
mail-no-recipients = (нет получателей)
mail-attachment-downloaded = Скачано: { $name }
mail-attachment-download-failed = Не удалось скачать.
mail-attachments-zip-done = Вложения скачаны как ZIP: { $count }.
mail-attachments-zip-capped = В ZIP вошли первые вложения ({ $count }); предел — { $max } файлов или { $size }.
mail-focus-not-found = Сообщение не найдено в папке «{ $folder }»
mail-focus-go-to = Перейти: { $folder }
mail-nav-views-queues = Виды и очереди
mail-badge-unread = { $unread ->
    [one] { $unread } непрочитанное сообщение
    [few] { $unread } непрочитанных сообщения
    [many] { $unread } непрочитанных сообщений
   *[other] { $unread } непрочитанного сообщения
  }
mail-badge-total = { $total ->
    [one] { $total } сообщение в этой папке
    [few] { $total } сообщения в этой папке
    [many] { $total } сообщений в этой папке
   *[other] { $total } сообщения в этой папке
  }
mail-badge-both = { $unread ->
    [one] { $unread } непрочитанное сообщение
    [few] { $unread } непрочитанных сообщения
    [many] { $unread } непрочитанных сообщений
   *[other] { $unread } непрочитанного сообщения
  } из { $total ->
    [one] { $total } в этой папке
    [few] { $total } в этой папке
    [many] { $total } в этой папке
   *[other] { $total } в этой папке
  }
mail-badge-drafts = { $total ->
    [one] Черновики: { $total } сообщение
    [few] Черновики: { $total } сообщения
    [many] Черновики: { $total } сообщений
   *[other] Черновики: { $total } сообщения
  }
mail-badge-compact-hint = У папок, кроме «Входящих», значок меньше. Это можно изменить в разделе «Настройки», «Папки и боковая панель».
mail-badge-outbox = { $count ->
    [one] Исходящие: { $count } сообщение ожидает отправки
    [few] Исходящие: { $count } сообщения ожидают отправки
    [many] Исходящие: { $count } сообщений ожидают отправки
   *[other] Исходящие: { $count } сообщения ожидают отправки
  }
mail-color-menu = Цвета сообщений
mail-color-button = Цвета сообщений: { $mode }
mail-color-use-default = Использовать мой вариант по умолчанию ({ $mode })
mail-color-make-default = Сделать это моим вариантом по умолчанию
mail-color-locked = Заблокировано администратором
mail-color-confirm-title = Изменить цвета сообщений по умолчанию?
mail-color-confirm-body = Все сообщения будут использовать режим «{ $mode }», пока вы не измените его для отдельного сообщения.
mail-color-confirm = Сделать по умолчанию
mail-color-default-failed = Не удалось сохранить цвета сообщений по умолчанию.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mail-attachments-none = Выбранные сообщения не имеют вложений для загрузки.

mail-attachments-zip-all-blocked = Ничего не было загружено: политика вложений блокирует все файлы { $skipped }.

mail-attachments-zip-gathering = Сбор вложений сообщений { $count }…

mail-attachments-zip-skipped = { $skipped ->
        [one] { $skipped } файл, заблокированный политикой вложений, был исключен.
        [few] { $skipped } файла, заблокированные политикой вложений, были исключены.
        [many] { $skipped } файлов, заблокированных политикой вложений, были исключены.
       *[other] { $skipped } файла, заблокированные политикой вложений, были исключены.
    }

mail-attachments-zip-unreadable = Вложения сообщений { $count } не удалось прочитать и были исключены.

mail-bulk-download-attachments = Загрузите вложения как ZIP

mail-compose-from-via = Он соответствовал заголовку { $header }.

mail-compose-from-why-match-domain = Отправка из { $identity }, поскольку сообщение было доставлено на { $address }, адрес в том же домене.

mail-compose-from-why-match-pattern = Отправка с { $identity }, поскольку сообщение было отправлено на { $address }, что соответствует шаблону адреса этого идентификатора.

mail-compose-ribbon-autohide = Автоматическое скрытие ленты (Ctrl+F1)

mail-compose-ribbon-pin = Всегда показывать ленту (Ctrl+F1)

mail-context-download-selected-attachments = { $count ->
        [one] Загрузите вложения выбранного сообщения как ZIP
        [few] Загрузите вложения к { $count } выбранным сообщениям как ZIP
        [many] Загрузите вложения из { $count } выбранных сообщений как ZIP
       *[other] Загрузите вложения из { $count } выбранных сообщений как ZIP
    }

mail-context-download-thread-attachments = Загрузите все вложения беседы { $count } как ZIP.

mail-context-open-new-tab = Открыть в новой вкладке

mail-delete-sender-confirm = Переместить сообщения { $count } в корзину.

mail-delete-sender-detail = Переместить сообщения { $count } из { $sender } в корзину? Сообщения, уже находящиеся в корзине, исключаются. Вы можете отменить это действие.

mail-delete-sender-empty = За пределами корзины нет сообщений от этого отправителя.

mail-delete-sender-failed = Не удалось проверить каждое соответствующее сообщение и его разрешения. Ничего не было перемещено; Попробуйте еще раз.

mail-delete-sender-help = Найдите сообщения этого отправителя во всех папках и после подтверждения переместите их в корзину.

mail-delete-sender-title = Удалить все сообщения от этого отправителя

mail-draft-continue = Продолжить редактирование

mail-draft-edit = Изменить черновик

mail-draft-edit-failed = Черновик не удалось открыть для редактирования.

mail-draft-edit-help = Откройте этот черновик в композиторе, чтобы продолжить его редактирование.

mail-draft-notice = Это черновик

mail-file-copied = Скопировано в буфер обмена.

mail-file-copy-failed = Не удалось скопировать в буфер обмена.

mail-file-copy-fingerprint = Копировать отпечаток пальца

mail-file-copy-name = Скопировать имя файла

mail-file-created = Создано

mail-file-creator = Создано с

mail-file-details-button = Детали файла

mail-file-details-tooltip = Показать сведения о файле и отпечаток пальца

mail-file-encryption = Защита паролем

mail-file-encryption-none = Пароль не запрашивался

mail-file-encryption-password = Чтобы открыть его, требовался пароль

mail-file-fingerprint = Отпечаток пальца

mail-file-name = Имя файла

mail-file-pages = Страницы

mail-file-pdf-version = PDF версия

mail-file-producer = Продюсер

mail-file-verify-help = Сравните эти группы с отпечатком пальца, который отправитель дал вам другим способом, например, по телефону или на своем веб-сайте. Если все группы совпадают, у вас точно одинаковые байты.

mail-file-verify-title = Проверьте отпечаток пальца

mail-follow-up-finished = Последующие действия завершены

mail-group-by = Группировать по

mail-header-attachments = { $count ->
        [one] { $count } вложение
        [few] { $count } вложения
        [many] { $count } вложений
       *[other] { $count } вложения
    }

mail-header-expand-short = Развернуть заголовок сообщения (Alt+H)

mail-list-group-date-unknown = Неизвестная дата

mail-list-group-importance = Важность

mail-list-group-locked = Группировка сообщений заблокирована администратором.

mail-list-group-no-subject = (Нет темы)

mail-list-group-none = Нет группировки

mail-list-group-priority-high = Высокая важность

mail-list-group-priority-low = Низкая важность

mail-list-group-priority-normal = Нормальная важность

mail-list-group-received = Дата получения

mail-list-group-sender = От

mail-list-group-sent = Дата отправки

mail-list-group-size = Размер

mail-list-group-size-huge = 10 МБ и выше

mail-list-group-size-large = 1–10 МБ

mail-list-group-size-medium = 100 КБ–1 МБ

mail-list-group-size-small = 25–100 КБ

mail-list-group-size-tiny = Менее 25 КБ

mail-list-group-subject = Тема

mail-list-order-importance = Важность, высокая в первую очередь

mail-list-order-importance-asc = Важность, сначала низкая

mail-list-order-locked = Порядок сообщений заблокирован вашим администратором.

mail-list-order-sender-desc = Отправитель, от Я до А

mail-list-order-sent-newest = Отправлено, сначала самое новое

mail-list-order-sent-oldest = Отправлено, сначала самое старое

mail-list-order-size = Размер, сначала самый большой

mail-list-order-size-asc = Размер, сначала самый маленький

mail-list-order-subject-desc = Тема: от Я до А

mail-mark-answered = Отметить как ответ

mail-mark-as = Отметить как

mail-mark-finished = Отметить как завершенное

mail-mark-forwarded = Отметить как перенаправленное

mail-mark-not-forwarded = Отметить как не пересланное

mail-mark-unanswered = Отметить как оставшийся без ответа

mail-mentions-me = Упоминает меня

mail-mentions-me-help = Поиск сообщений, в которых упоминается один из ваших адресов электронной почты.

mail-move-search = Поиск по папкам

mail-order-by = Заказать по

mail-reader-folder-go = Хранится в { $folder } · Перейти в папку

mail-reply-draft-discard = Отменить ответ

mail-reply-draft-discard-detail = Удалить этот ответ навсегда? Это невозможно отменить.

mail-reply-draft-discard-title = Удалить этот ответ?

mail-status-from-match-domain = От { $identity }

mail-status-from-match-pattern = От { $identity }

mail-status-from-via = Отвечаю с { $identity }, потому что оно соответствует { $header }

mail-toast-follow-up-finished = Продолжение отмечено как завершенное

mail-toast-marked-answered = { $count ->
        [one] { $count } сообщение отмечено как отвеченное
        [few] { $count } сообщения отмечены как отвеченные
        [many] { $count } сообщений отмечены как отвеченные
       *[other] { $count } сообщения отмечены как отвеченные
    }

mail-toast-marked-forwarded = { $count ->
        [one] { $count } сообщение помечено как пересланное
        [few] { $count } сообщения отмечены как пересланные
        [many] { $count } сообщений помечены как пересланные
       *[other] { $count } сообщения отмечены как пересланные
    }

mail-toast-marked-not-forwarded = { $count ->
        [one] { $count } сообщение помечено как не пересланное
        [few] { $count } сообщения отмечены как не пересланные
        [many] { $count } сообщений помечены как не пересланные
       *[other] { $count } сообщения отмечены как не пересланные
    }

mail-toast-marked-unanswered = { $count ->
        [one] { $count } сообщение помечено как оставшееся без ответа
        [few] { $count } сообщения отмечены как неотвеченные
        [many] { $count } сообщений помечены как неотвеченные
       *[other] { $count } сообщения отмечены как неотвеченные
    }

mail-toast-move-failed-reason = Не удалось переместить сообщения ({ $reason }). Ничего не было перемещено.

mail-toast-update-refused-reason = { $count ->
        [one] Сервер отказал в перемещении по одному сообщению: { $reason }
        [few] Сервер отклонил перемещение сообщений ({ $count }): { $reason }
        [many] Сервер отклонил перемещение для { $count } сообщений: { $reason }
       *[other] Сервер отказал в перемещении на { $count } сообщения: { $reason }
    }

reader-badge-hide = Скрыть этот значок

reader-badge-locked = Устанавливается вашим администратором

reader-badge-menu = Значок сообщения

reader-badge-notable = Показывать только тогда, когда это заметно

reader-sanitized-always = Активный контент, например сценарии, остается отключенным.

reader-sanitized-clean = Сообщение HTML не требует изменений.

reader-sanitized-clean-detail = Из этого сообщения HTML ничего не было удалено или заблокировано.

reader-sanitized-counts = { $blocked } удаленные элементы заблокированы, { $trackers } элементы отслеживания.

reader-sanitized-detail = Сообщение HTML было очищено перед показом. Удаленный контент соответствует вашим настройкам конфиденциальности.

reader-sanitized-hosts = Удаленные хосты

reader-sanitized-remote = Удаленный контент заблокирован

reader-sanitized-short = Продезинфицирован HTML

reader-sanitized-short-blocked = HTML заблокирован

reader-sanitized-short-clean = HTML чистый

reader-sanitized-trackers = Отслеживание предметов

mail-draft-view = Просмотреть черновик
mail-reply-draft-view = Просмотреть ответ
