# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Налаштування
mail-compose = Написати
mail-nav-mailboxes = Теки
mail-nav-attachments = Вкладення
mail-nav-outbox = Вихідні
mail-nav-sharing = Надати доступ до теки
mail-nav-apps = Застосунки
mail-logout = Вийти
mail-offline = Офлайн
mail-module-loading = Завантаження: { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Пошук
mail-search-label = Пошук у пошті
mail-search-placeholder = Пошук у пошті — from:alice subject:invoice larger:1mb
mail-search-clear = Очистити

# -- Message list ------------------------------------------------------------
mail-list-label = Повідомлення
mail-loading = Завантаження повідомлень…
mail-empty = Немає повідомлень
mail-unknown-sender = (невідомий відправник)
mail-no-subject = (без теми)
mail-pinned = Закріплене
mail-has-attachment = Має вкладення
mail-answered = Є відповідь
mail-forwarded = Переслано
mail-unread = Непрочитане
mail-flagged = З прапорцем
# Announced on a selected row; the row also shows a check mark.
mail-selected = Вибрано
# Announced row summary for screen readers (position within the whole list).
mail-row-position = Повідомлення { $pos } з { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } повідомлення
    [few] { $count } повідомлення
    [many] { $count } повідомлень
   *[other] { $count } повідомлення
}
mail-thread-expand = Розгорнути ланцюжок, { $count ->
    [one] { $count } повідомлення
    [few] { $count } повідомлення
    [many] { $count } повідомлень
   *[other] { $count } повідомлення
}
mail-thread-collapse = Згорнути ланцюжок, { $count ->
    [one] { $count } повідомлення
    [few] { $count } повідомлення
    [many] { $count } повідомлень
   *[other] { $count } повідомлення
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Параметри перегляду
mail-reading-pane = Панель читання
mail-reading-pane-right = Праворуч
mail-reading-pane-bottom = Унизу
mail-reading-pane-off = Вимкнено

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Вибрані повідомлення
mail-bulk-count = Вибрано: { $count }
mail-mark-read = Позначити як прочитане
mail-mark-unread = Позначити як непрочитане
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Позначити прапорцем
mail-unflag-message = Зняти прапорець
mail-move-to = Перемістити до теки
mail-move-to-placeholder = Перемістити до…
mail-select-all = Вибрати все
mail-clear-selection = Скасувати вибір

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = Фільтр вхідних
mail-inbox-focused = Пріоритетні
mail-inbox-other = Інші

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Відкриті вкладки
mail-subtab-pin = Закріпити { $title }
mail-subtab-unpin = Відкріпити { $title }
mail-subtab-tearoff = Відкрити { $title } у новому вікні
mail-subtab-close = Закрити { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Закріпити
mail-unpin = Відкріпити
mail-snooze = Відкласти
mail-snooze-menu = Відкласти до
mail-snooze-later = Пізніше сьогодні
mail-snooze-tomorrow = Завтра
mail-snooze-next-week = Наступного тижня
mail-unsnooze = Скасувати відкладення
mail-label = Мітка
mail-labels-menu = Мітки
mail-flag = Позначити для подальших дій
mail-clear-flag = Зняти позначку для подальших дій
mail-archive = Архівувати
mail-delete = Видалити
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = Інші дії

# -- Reader ------------------------------------------------------------------
mail-reader-label = Повідомлення
mail-reader-empty = Виберіть повідомлення для читання
mail-back = Назад
mail-reader-from = Від: { $addr }
mail-reader-to = Кому: { $addr }
mail-reader-cc = Копія: { $addr }
mail-reader-actions = Дії з повідомленням
mail-reply = Відповісти
mail-reply-all = Відповісти всім
mail-forward = Переслати
mail-reply-failed = Не вдалося підготувати повідомлення. Спробуйте знову.
mail-reply-unthreaded = Не вдалося прочитати ідентифікатор вихідного повідомлення. Цю відповідь не буде об'єднано з ним у ланцюжок.
mail-spam = Спам
mail-export = Експортувати
mail-sweep-sender = Прибрати від відправника
mail-message-body = Текст повідомлення
mail-no-content = Немає вмісту
mail-sanitizing = Очищення…
mail-attachments = Вкладення
mail-attachment-unnamed = (без назви)
mail-attachment-open = Вкладення { $name }
mail-attachment-close = Закрити вкладення
mail-attachment-loading = Завантаження вкладення…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Зашифроване повідомлення
mail-encrypted-note = Це повідомлення захищено наскрізним шифруванням. Розблокуйте його на цьому пристрої, щоб прочитати.
mail-key-passphrase = Парольна фраза ключа
mail-decrypt = Розшифрувати
mail-decrypting = Розшифрування…
mail-decrypt-no-key = Немає приватного ключа для розшифрування цього повідомлення.
mail-decrypt-failed = Не вдалося розшифрувати

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = { $date } { $name } пише:
mail-quote-attribution-undated = { $name } пише:
# The header block above a forwarded original.
mail-forward-heading = ---------- Переслане повідомлення ----------
mail-forward-from = Від: { $addr }
mail-forward-date = Дата: { $date }
mail-forward-subject = Тема: { $subject }
mail-forward-to = Кому: { $addr }
mail-forward-cc = Копія: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Написати повідомлення
mail-compose-title = Нове повідомлення
mail-compose-title-reply = Відповідь
mail-compose-title-reply-all = Відповідь усім
mail-compose-title-forward = Пересилання
mail-compose-close = Закрити
mail-compose-from = Від
mail-compose-from-default = Типовий
mail-compose-to = Кому
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Пропоновані контакти
mail-compose-cc = Копія
mail-compose-bcc = Прихована копія
mail-compose-show-cc-bcc = Додати копію або приховану копію
mail-compose-bcc-encrypted = Зашифроване повідомлення містить ключ кожного одержувача, тому одержувача прихованої копії бачитимуть інші. Видаліть одержувачів прихованої копії або вимкніть шифрування.
mail-compose-plaintext-confirm-title = Надіслати розшифрований текст без шифрування?
mail-compose-plaintext-confirm-body = Це повідомлення цитує текст із зашифрованого повідомлення. Шифрування для нього не ввімкнено, тому цитований текст буде надіслано без захисту.
mail-compose-plaintext-confirm-send = Надіслати без шифрування
mail-compose-plaintext-confirm-cancel = Повернутися
mail-compose-subject = Тема
mail-compose-body = Текст
mail-compose-attachments = Вкладення
mail-compose-remove-attachment = Вилучити { $name }
mail-compose-attach-nextcloud = Вкласти з Nextcloud
mail-compose-close-nextcloud = Закрити Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Вкласти файл
mail-compose-uploading = Вивантаження…
mail-compose-upload-unavailable = Вивантаження файлів зараз недоступне.
mail-compose-upload-failed = Не вдалося вивантажити { $name }.
mail-compose-upload-too-large = Розмір { $name } — { $size } МБ. Максимальний розмір вивантаження — { $max } МБ.
mail-compose-send-later = Надіслати пізніше
mail-compose-cancel = Скасувати
mail-compose-send = Надіслати
mail-compose-schedule = Запланувати
mail-compose-sending = Надсилання…
mail-compose-encrypted-subject = Зашифроване повідомлення
mail-compose-dlp-blocked = Надсилання заблоковано правилом захисту від витоку даних (див. попередження вище).
mail-compose-send-failed = Не вдалося надіслати
mail-compose-send-later-past = Виберіть час надсилання в майбутньому або очистьте поле, щоб надіслати зараз.
mail-compose-options = Параметри надсилання
mail-compose-request-receipt = Запитати сповіщення про прочитання
mail-compose-request-receipt-hint = Поштова програма одержувача може запитати його, надіслати сповіщення без запиту або проігнорувати запит.
mail-compose-track-opens = Фіксувати відкриття цього повідомлення
mail-compose-track-opens-hint = До повідомлення додається зображення 1×1, яке завантажується з цього сервера. Багато поштових програм блокують зображення або завантажують їх заздалегідь, тому зафіксоване відкриття не доводить, що повідомлення прочитано, а відсутність запису не доводить, що його не прочитано. Одержувача про це не повідомляють.
mail-compose-image-type = { $name } не можна вставити в текст. Використайте зображення PNG, JPEG, GIF або WebP.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = Ви не ввійшли в систему.
mail-send-no-draft-folder = У цьому обліковому записі немає теки для зберігання чернетки.
mail-send-no-recipient = Додайте принаймні одного одержувача.
mail-send-bad-recipient = { $count ->
    [one] Це не адреса електронної пошти: { $addresses }
    [few] Це не адреси електронної пошти: { $addresses }
    [many] Це не адреси електронної пошти: { $addresses }
   *[other] Це не адреси електронної пошти: { $addresses }
}
mail-send-later-past = Час надсилання не в майбутньому. Нічого не надіслано.
mail-send-tracking-failed = Зараз неможливо фіксувати відкриття, тому нічого не надіслано. Зніміть позначку «Фіксувати відкриття цього повідомлення», щоб надіслати без цього.
mail-send-draft-refused = Сервер відхилив це повідомлення: { $reason }. Нічого не надіслано.
mail-send-submission-refused = Сервер відмовився надіслати це повідомлення: { $reason }. Нічого не надіслано.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Розблокування ключа підпису
mail-compose-sign-unlock-note = Введіть парольну фразу ключа, щоб підписати це повідомлення. Ключ залишається розблокованим лише для цього вікна створення повідомлення.
mail-compose-sign-unlock = Розблокувати
mail-compose-sign-unlocking = Розблокування…
mail-compose-sign-no-key = На цьому пристрої немає ключа підпису.
mail-compose-sign-unlock-failed = Не вдалося розблокувати ключ підпису.
mail-compose-sign-unlock-required = Розблокуйте ключ підпису, щоб надіслати це підписане повідомлення.

# Rich-text editor (W1)
mail-compose-format-plain = Звичайний текст
mail-compose-format-rich = Форматований текст
mail-compose-rt-toolbar = Форматування
mail-compose-rt-bold = Жирний
mail-compose-rt-italic = Курсив
mail-compose-rt-underline = Підкреслений
mail-compose-rt-strike = Закреслений
mail-compose-rt-bullet = Маркований список
mail-compose-rt-ordered = Нумерований список
mail-compose-rt-quote = Цитата
mail-compose-rt-link = Посилання
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Застосувати
mail-compose-rt-image = Вставити зображення

# Signature picker (W12)
mail-compose-signature = Підпис
mail-compose-signature-none = Додати підпис…

# Recall (W10)
mail-compose-recall = Відкликання
mail-compose-recall-action = Відкликати
mail-compose-recall-holding = Затримано, очікує надсилання
mail-compose-recall-scheduled = Заплановано на { $when }

# Drafts drawer (W9)
mail-compose-drafts = Чернетки
mail-compose-drafts-empty = Немає збережених чернеток
mail-compose-drafts-delete = Видалити чернетку
mail-compose-drafts-no-recipient = (без одержувача)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Вихідні
mail-outbox-refresh = Оновити
mail-outbox-empty = Немає повідомлень, що очікують надсилання
mail-outbox-send-now = Надіслати зараз
mail-outbox-cancel = Скасувати
mail-outbox-scheduled = Заплановано
mail-outbox-holding = Незабаром буде надіслано
mail-outbox-sent = Надіслано
mail-outbox-canceled = Скасовано
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = Затримано
mail-outbox-failed = Не надіслано
mail-outbox-release = Відпустити
mail-outbox-discard = Відкинути
mail-outbox-held-note = Затримано. Не буде надіслано, доки не відпустите.
mail-outbox-held-note-api-key = Затримано — створено ключем API { $name }. Не буде надіслано, доки не відпустите.
mail-outbox-held-note-oauth = Затримано — створено підключеним застосунком { $name }. Не буде надіслано, доки не відпустите.
mail-outbox-origin-api-key = Створено ключем API { $name }
mail-outbox-origin-oauth = Створено підключеним застосунком { $name }
mail-outbox-to = Кому: { $recipients }
mail-outbox-no-subject = (без теми)
mail-outbox-attempts = { $count ->
    [one] Здійснено { $count } спробу. Остання помилка: { $error }
    [few] Здійснено { $count } спроби. Остання помилка: { $error }
    [many] Здійснено { $count } спроб. Остання помилка: { $error }
   *[other] Здійснено { $count } спроби. Остання помилка: { $error }
}
mail-outbox-error = Помилка: { $error }
mail-outbox-toast-canceled = Надсилання скасовано
mail-outbox-toast-discarded = Відкинуто. Повідомлення не надіслано.
mail-outbox-toast-sent = Надіслано
mail-outbox-toast-retrying = Ще не надіслано: { $error }. Буде здійснено повторну спробу.
mail-outbox-toast-failed = Не надіслано: { $error }
mail-outbox-toast-not-released = Не вдалося надіслати: { $error }
mail-outbox-toast-not-canceled = Не вдалося скасувати: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = З'єднання втрачено — повторна спроба…
mail-net-back = Знову онлайн
mail-queue-sent = { $count ->
    [one] Надіслано { $count } зміну з черги
    [few] Надіслано { $count } зміни з черги
    [many] Надіслано { $count } змін з черги
   *[other] Надіслано { $count } зміни з черги
}
mail-queue-failed = { $count ->
    [one] Не вдалося застосувати { $count } зміну з черги
    [few] Не вдалося застосувати { $count } зміни з черги
    [many] Не вдалося застосувати { $count } змін з черги
   *[other] Не вдалося застосувати { $count } зміни з черги
}
mail-queue-notice-label = Зміни, які не вдалося застосувати
mail-queue-notice-title = { $count ->
    [one] Не вдалося застосувати { $count } зміну, внесену офлайн
    [few] Не вдалося застосувати { $count } зміни, внесені офлайн
    [many] Не вдалося застосувати { $count } змін, внесених офлайн
   *[other] Не вдалося застосувати { $count } зміни, внесеної офлайн
}
mail-queue-kind-send = Надіслати «{ $subject }»
mail-queue-kind-send-untitled = Надіслати повідомлення без теми
mail-queue-kind-move = Перемістити повідомлення
mail-queue-kind-flag = Змінити мітку або прапорець
mail-queue-kind-draft = Зберегти чернетку
mail-queue-kind-pim = Зміна в календарі, завданнях, нотатках або контактах
mail-queue-reason = Причина: { $reason }
mail-queue-reason-unknown = Сервер не вказав причину.
mail-queue-retry = Повторити
mail-queue-discard = Відкинути
mail-queue-retry-item = Повторити: { $what }
mail-queue-discard-item = Відкинути: { $what }
mail-queue-discarded = Відкинуто. Зміну не застосовано.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Прибирання повідомлень
mail-sweep-title = Прибрати від { $sender }
mail-sweep-all = Видалити всі від цього відправника
mail-sweep-keep-latest = Залишити останнє, решту видалити
mail-sweep-older-than = Видалити старші за N днів
mail-sweep-block = Видалити всі й заблокувати цього відправника
mail-sweep-days = Днів
mail-sweep-count = { $count ->
    [one] { $count } повідомлення буде переміщено до кошика
    [few] { $count } повідомлення буде переміщено до кошика
    [many] { $count } повідомлень буде переміщено до кошика
   *[other] { $count } повідомлення буде переміщено до кошика
}
mail-sweep-cancel = Скасувати
mail-sweep-run = Прибрати { $count }
mail-sweeping = Прибирання…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Закрити
mail-undo-action = Скасувати
mail-undo-send-cancel = Скасувати
mail-undo-failed = Не вдалося скасувати дію

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Мітку додано
mail-toast-label-removed = Мітку вилучено
mail-toast-pinned = Повідомлення «{ $target }» закріплено
mail-toast-unpinned = Повідомлення «{ $target }» відкріплено
mail-toast-snoozed = Відкладено
mail-toast-follow-up-set = Позначку для подальших дій встановлено
mail-toast-follow-up-cleared = Позначку для подальших дій знято
mail-toast-archived = Заархівовано
mail-toast-trashed = Переміщено до кошика
mail-toast-marked-spam = Позначено як спам
mail-toast-moved-to = Переміщено до { $folder }
mail-toast-moved = Переміщено до теки
mail-toast-archived-count = { $count ->
    [one] { $count } повідомлення заархівовано
    [few] { $count } повідомлення заархівовано
    [many] { $count } повідомлень заархівовано
   *[other] { $count } повідомлення заархівовано
}
mail-toast-trashed-count = { $count ->
    [one] { $count } повідомлення переміщено до кошика
    [few] { $count } повідомлення переміщено до кошика
    [many] { $count } повідомлень переміщено до кошика
   *[other] { $count } повідомлення переміщено до кошика
}
mail-toast-marked-spam-count = { $count ->
    [one] { $count } повідомлення позначено як спам
    [few] { $count } повідомлення позначено як спам
    [many] { $count } повідомлень позначено як спам
   *[other] { $count } повідомлення позначено як спам
}
mail-toast-moved-count = { $count ->
    [one] { $count } повідомлення переміщено
    [few] { $count } повідомлення переміщено
    [many] { $count } повідомлень переміщено
   *[other] { $count } повідомлення переміщено
}
mail-toast-moved-to-count = { $count ->
    [one] { $count } повідомлення переміщено до { $folder }
    [few] { $count } повідомлення переміщено до { $folder }
    [many] { $count } повідомлень переміщено до { $folder }
   *[other] { $count } повідомлення переміщено до { $folder }
}
mail-toast-marked-read = { $count ->
    [1] Позначено як прочитане
    [one] { $count } повідомлення позначено як прочитане
    [few] { $count } повідомлення позначено як прочитані
    [many] { $count } повідомлень позначено як прочитані
   *[other] { $count } повідомлення позначено як прочитані
}
mail-toast-marked-unread = { $count ->
    [1] Позначено як непрочитане
    [one] { $count } повідомлення позначено як непрочитане
    [few] { $count } повідомлення позначено як непрочитані
    [many] { $count } повідомлень позначено як непрочитані
   *[other] { $count } повідомлення позначено як непрочитані
}
mail-toast-flagged = { $count ->
    [1] Позначено прапорцем
    [one] { $count } повідомлення позначено прапорцем
    [few] { $count } повідомлення позначено прапорцем
    [many] { $count } повідомлень позначено прапорцем
   *[other] { $count } повідомлення позначено прапорцем
}
mail-toast-unflagged = { $count ->
    [1] Прапорець знято
    [one] Прапорець знято з { $count } повідомлення
    [few] Прапорець знято з { $count } повідомлень
    [many] Прапорець знято з { $count } повідомлень
   *[other] Прапорець знято з { $count } повідомлення
}
mail-toast-move-failed = Не вдалося перемістити повідомлення. Нічого не переміщено.
mail-toast-update-failed = Не вдалося змінити повідомлення. Нічого не змінено.
mail-toast-update-refused = { $count ->
    [1] Сервер відхилив зміну для одного повідомлення
    [one] Сервер відхилив зміну для { $count } повідомлення
    [few] Сервер відхилив зміну для { $count } повідомлень
    [many] Сервер відхилив зміну для { $count } повідомлень
   *[other] Сервер відхилив зміну для { $count } повідомлення
}
mail-toast-no-archive-folder = Немає теки «Архів»
mail-toast-no-trash-folder = Немає теки «Кошик»
mail-toast-no-spam-folder = Немає теки «Спам»
mail-toast-sweep-nothing = Немає чого прибирати
mail-toast-swept = { $count ->
    [one] Прибрано { $count } повідомлення
    [few] Прибрано { $count } повідомлення
    [many] Прибрано { $count } повідомлень
   *[other] Прибрано { $count } повідомлення
}
mail-toast-load-more-failed = Не вдалося завантажити більше повідомлень
mail-toast-export-nothing = Немає чого експортувати
mail-toast-exported = Експортовано .eml
mail-toast-export-failed = Не вдалося експортувати
mail-toast-send-queued = У черзі — буде надіслано, коли з'явиться з'єднання
mail-toast-tracking-not-bound = Повідомлення надіслано, але його відкриття не фіксуватимуться.
mail-toast-send-scheduled = Надсилання заплановано
mail-toast-sent = Повідомлення надіслано
mail-toast-send-canceled = Надсилання скасовано

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Стрічка
mail-ribbon-tab-home = Основне
mail-ribbon-tab-view = Вигляд
mail-ribbon-tab-folder = Тека
mail-ribbon-expand = Розгорнути
mail-ribbon-collapse = Згорнути
mail-ribbon-group-new = Створити
mail-ribbon-compose = Написати
mail-ribbon-group-session = Сеанс
mail-ribbon-logout = Вийти
mail-ribbon-group-theme = Тема
mail-ribbon-group-density = Щільність
mail-ribbon-group-settings = Налаштування
mail-ribbon-more = Більше…
mail-ribbon-group-folders = Теки
mail-density-compact = компактна
mail-density-cozy = зручна
mail-density-relaxed = вільна

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Рівень безпеки повідомлення
mail-maxsec-full = Повний
mail-maxsec-full-hint = Очищений HTML із зображеннями та стилями.
mail-maxsec-no-media = Без медіа
mail-maxsec-no-media-hint = Очищений HTML без жодних зображень і медіа.
mail-maxsec-plain-text = Звичайний текст
mail-maxsec-plain-text-hint = HTML не відображається — повідомлення показано лише як звичайний текст.
mail-feature-off = Вимкнено вашими налаштуваннями або адміністратором
mail-send-disabled = Надсилання вимкнено вашими налаштуваннями або адміністратором. Чернетку все одно можна зберегти.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Події
mail-events-upcoming = Лише майбутні
mail-events-filter = Фільтр подій
mail-loading-more = Завантаження старіших повідомлень…
mail-event-state-invitation = Запрошення
mail-event-state-updated = Оновлено
mail-event-state-cancelled = Скасовано
mail-event-state-reply = Відповідь
mail-event-state-counter = Запропоновано новий час
mail-event-state-event = Подія
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Показати інші дії
mail-remote-moved = Це повідомлення переміщено до теки «{ $folder }» з іншого пристрою чи застосунку. Ви й далі читаєте відкриту копію.
mail-remote-moved-unknown = Це повідомлення переміщено з іншого пристрою чи застосунку. Ви й далі читаєте відкриту копію.
mail-remote-deleted = Це повідомлення видалено з іншого пристрою чи застосунку. Ви можете читати цю копію, доки не закриєте її.
mail-remote-dismiss = Закрити
mail-reply-draft = Відповідь у процесі
mail-reply-draft-open = Відкрити чернетку
mail-draft-load-failed = Не вдалося відкрити чернетку. Спробуйте ще раз.
mail-reply-images-omitted =
    { $count ->
        [one] Не вдалося додати до відповіді { $count } зображення.
        [few] Не вдалося додати до відповіді { $count } зображення.
        [many] Не вдалося додати до відповіді { $count } зображень.
       *[other] Не вдалося додати до відповіді { $count } зображення.
    }
mail-export-forbidden = Адміністратор вимкнув експорт повідомлень для цього облікового запису.
mail-export-open-failed = Не вдалося відкрити параметри експорту. Закрийте вкладку робочої області та спробуйте ще раз.
mail-headers-title = Вихідні заголовки повідомлення
mail-headers-action-help = Перегляд вихідних заголовків повідомлення, зокрема відомостей про доставлення й автентифікацію.
mail-headers-help = Вихідні заголовки повідомлення. Перенесені рядки збережено; тут показано лише заголовки.
mail-headers-loading = Завантаження заголовків повідомлення…
mail-headers-failed = Не вдалося завантажити заголовки повідомлення. Перевірте з'єднання й спробуйте ще раз.
mail-headers-copy = Копіювати заголовки
mail-headers-copied = Заголовки скопійовано.
mail-headers-copy-failed = Не вдалося скопіювати заголовки. Виділіть текст і скопіюйте його вручну.
mail-attachment-failed = Не вдалося завантажити це вкладення. Спробуйте ще раз, коли з'явиться з'єднання.
mail-compose-rt-rule = Горизонтальна лінія
mail-compose-signature-new = Нові повідомлення
mail-compose-signature-replies = Відповіді
mail-editor-body-size = { $size } КіБ
mail-editor-body-size-label = Розмір тексту
mail-editor-body-size-help = Розмір поточного тексту в UTF-8, зокрема розмітка HTML і вбудовані зображення. Заголовки повідомлення й окремі вкладення не враховуються.
mail-outbox-load-failed = Не вдалося перевірити «Вихідні». Натисніть «Оновити» після перевірки з'єднання.
mail-outbox-unavailable = «Вихідні» недоступні. Натисніть «Оновити», щоб спробувати ще раз.
mail-maxsec-view = Вигляд: { $mode }
mail-maxsec-menu = Режим перегляду
mail-toast-tab-pinned = Вкладку «{ $target }» закріплено
mail-toast-tab-unpinned = Вкладку «{ $target }» відкріплено
mail-workspace-message = Повідомлення
mail-opened-announcement = Відкрито: { $subject }
mail-workspace-layout = Макет робочої області
mail-workspace-tabs = Вкладки
mail-workspace-side-by-side = Поруч
mail-workspace-stack = Одна під одною
mail-workspace-refresh = Оновити теку
mail-workspace-load-error = Не вдалося завантажити це повідомлення. Закрийте вкладку й спробуйте ще раз.
mail-resize-folders = Змінити розмір панелі тек
mail-resize-list = Змінити розмір списку повідомлень
mail-resize-tiles = Змінити розмір областей робочої області
mail-show-folders = Показати панель тек
mail-hide-folders = Сховати панель тек
mail-compact-folders = Компактні значки тек
mail-toggle-list = Показати або сховати список повідомлень
mail-toggle-reader = Показати або сховати область читання
mail-folder-counts = Непрочитаних: { $unread }, повідомлень: { $total }
mail-print = Друк / Зберегти як PDF
mail-go-to-top = Догори
mail-print-help = Надрукуйте це повідомлення або виберіть «Зберегти як PDF» у діалозі друку. Зовнішні зображення лишаються заблокованими.
mail-file-details = Відомості про файл і відбиток
mail-file-type = Тип файлу
mail-file-extension = Розширення
mail-file-detected = Виявлений підпис
mail-file-unknown = Підпис не розпізнано
mail-file-size = Розмір
mail-file-size-bytes = { $count } Б
mail-file-hash-unavailable = Відбиток недоступний (потрібні захищений браузер і файл не більше 64 МіБ).
mail-file-fingerprint-help = Відбиток однозначно визначає саме ці байти. Він не доводить, що файл безпечний або хто його створив.
mail-file-warning-executable = Цей файл може запускати програми або змінювати налаштування системи. Завантажуйте його, лише якщо довіряєте джерелу.
mail-file-warning-macro = Цей документ може містити макроси. Не вмикайте макроси, доки не перевірите відправника й призначення.
mail-file-warning-disguisedName = Ім'я файлу може приховувати його справжнє розширення. Перед завантаженням перевірте відомості про файл.
mail-file-warning-typeMismatch = Виявлений підпис файлу не відповідає заявленому типу. Автоматичний перегляд заблоковано.
mail-file-warning-activeDocument = Цей формат може містити активний вміст. Перегляд ізольовано; завантажена копія під час відкриття може поводитися інакше.
mail-file-preview-blocked = Автоматичний перегляд вимкнено для цього типу файлу або за невідповідності типу.
mail-quote-header-from = Від:
mail-quote-header-sent = Надіслано:
mail-quote-header-to = Кому:
mail-quote-header-cc = Копія:
mail-quote-header-subject = Тема:
mail-spam-score = Аналіз спаму
mail-spam-score-loading = Перевірка заголовків спаму…
mail-spam-score-unavailable = Оцінка спаму недоступна
mail-spam-score-unverified = Указано в заголовках повідомлення, незалежно не перевірено. Заголовки, задані відправником, можуть бути підроблені.
mail-spam-score-positive = Заголовок повідомляє про спам.
mail-spam-score-negative = Заголовок повідомляє, що це не спам.
mail-spam-score-unknown = Вердикту до цієї оцінки не додано.
mail-external-recipients = Зовнішні отримувачі
mail-external-count = Отримувачів поза довіреними доменами: { $count }
mail-external-help = Перевірте отримувачів у полях «Кому», «Копія» та «Прихована копія» перед надсиланням конфіденційних відомостей. Довіра визначається вибраним профілем відправника та чинною політикою доменів.
mail-external-reviewed = Я перевірив цих зовнішніх отримувачів.
mail-external-confirm-required = Перед надсиланням перевірте й підтвердьте зовнішніх отримувачів.
mail-quick-reply = Швидка відповідь
mail-quick-reply-help = Пропонувати компактну вкладку відповіді з уже заповненими отримувачами. За замовчуванням вимкнено. Звичайні перевірки безпеки та параметри надсилання й далі діють.
mail-quick-reply-expand = Змінити отримувачів і тему
mail-attachment-policy-loading = Перевірка політики вкладень…
mail-attachment-policy-unavailable = Не вдалося перевірити політику вкладень. Спробуйте ще раз перед відкриттям файлу.
mail-attachment-policy-blocked = Адміністратор заблокував це вкладення або його завантаження в оригіналі.
mail-attachment-policy-warning = Адміністратор радить бути обережним із цим вкладенням. Продовжуйте, лише якщо довіряєте джерелу.
mail-attachment-policy-acknowledge = Розумію — продовжити
mail-attachment-policy-no-preview = Адміністратор вимкнув перегляд вкладень. Завантаження оригіналів може лишатися доступним.
mail-time-sent = Надіслано
mail-time-received = Отримано
mail-time-unknown = Не вказано
mail-time-comparison = Час у поясі { $zone }
mail-time-privacy = Пояс для порівняння вибираєте ви. Мітки часу повідомлень не вказують на чиєсь місцезнаходження.
mail-time-event-comparison = Порівняння часу події
mail-time-ambiguous = Цей місцевий час трапляється двічі під час переведення годинників. Показано обидва можливі моменти; перед збереженням перевірте передбачуване зміщення події.
mail-bulk-add-label = Додати мітку
mail-select-messages = Вибрати повідомлення
mail-bulk-remove-label = Видалити мітку
mail-mark-spam = Позначити як спам
mail-bulk-forward = Переслати вибрані повідомлення вкладеннями
mail-bulk-download = Завантажити вибрані повідомлення як mbox
mail-bulk-forward-subject = Переслані повідомлення ({ $count })
mail-bulk-transfer-failed = Не вдалося підготувати вибрані повідомлення. Виберіть не більше 100 повідомлень загальним розміром не більше 32 МіБ і перевірте права експорту вашого облікового запису.
mail-toast-labels-added = Мітку «{ $label }» додано до повідомлень: { $count }
mail-toast-labels-removed = Мітку «{ $label }» видалено з повідомлень: { $count }
mail-action-help-reply = Відповісти відправнику на новій вкладці написання.
mail-action-help-reply-all = Відповісти відправнику та іншим отримувачам на новій вкладці написання.
mail-action-help-forward = Перевірити й переслати це повідомлення вибраним отримувачам.
mail-action-help-pin = Закріпити це повідомлення для швидкого доступу або зняти закріплення.
mail-action-help-read = Змінити позначку про прочитання повідомлення. Зміну можна скасувати.
mail-action-help-flag = Встановити або зняти прапорець повідомлення для подальшої обробки.
mail-action-help-archive = Перемістити це повідомлення до налаштованої теки архіву.
mail-action-help-delete = Перемістити це повідомлення до кошика. Переміщення можна скасувати.
mail-action-help-spam = Позначити це повідомлення як небажане й перемістити до налаштованої теки спаму.
mail-action-help-export = Відкрити сторінку експорту, щоб завантажити це повідомлення у вибраному форматі.
mail-action-help-sweep = Перегляньте інші повідомлення цього відправника та виберіть, що прибрати (видалити всі, залишити останнє, видалити старіші або заблокувати відправника), перш ніж щось буде застосовано.
mail-empty-trash = Очистити кошик
mail-trash-help = Перегляньте й безповоротно видаліть повідомлення, що зараз у кошику. Цю дію не можна скасувати.
mail-trash-preparing = Підготовка знімка повідомлень у кошику…
mail-trash-prepare-failed = Не вдалося підготувати перегляд кошика. Перевірте з'єднання й відкрийте його знову. Жодне повідомлення не видалено.
mail-trash-confirm = Безповоротно видалити з кошика повідомлення: { $count }? Цю дію не можна скасувати. Повідомлення, що надійшли після цього перегляду, виключаються; повідомлення, переміщені з кошика, пропускаються.
mail-trash-limit = Ця операція охоплює до { $count } повідомлень. У кошику лишаються інші; перегляньте наступну операцію, щоб видалити й їх.
mail-trash-permanent = Видалити безповоротно
mail-trash-stop = Зупинити після поточного запиту
mail-trash-progress = Видалено: { $deleted }; пропущено повідомлень, яких уже немає в кошику: { $skipped }; залишилося в цьому перегляді: { $remaining }.
mail-trash-stopped = Видалення зупинено. Підтверджено видалено: { $deleted }; пропущено: { $skipped }; залишилося: { $remaining }; результат невідомий: { $uncertain }. Перевірте кошик, перш ніж починати нову операцію.
mail-links-title = Посилання в повідомленні
mail-links-review = Перевірити посилання ({ $count })
mail-links-help = Посилання в повідомленні відкриваються лише після того, як ви перевірите їхню адресу призначення тут. Жодна адреса не завантажується автоматично.
mail-links-inert = Скористайтеся «Перевірити посилання» над повідомленням, щоб переглянути й відкрити цю адресу.
mail-links-limit = Показано перші 100 різних підтримуваних адрес. Решта посилань у повідомленні лишаються неактивними.
mail-links-destination = Адреса для відкриття
mail-links-external = Цю адресу вказав відправник повідомлення. Відкриття залишить Mailwoman; перевірте адресу, перш ніж продовжити.
mail-links-open = Відкрити в новій вкладці
mail-links-compose = Перевірити нове повідомлення
mail-links-warning-http = Ця адреса використовує незашифрований HTTP.
mail-links-warning-address = У цій адресі замість доменного імені використано IP-адресу.
mail-links-warning-international = Цей домен містить інтернаціоналізоване ім'я. Уважно перевірте його написання в ASCII.
mail-links-warning-mismatch = Видима вебадреса вказує на інший вузол, ніж фактична адреса призначення.
mail-links-warning-direction = Текст посилання містить символи, що змінюють напрям тексту, які можуть приховувати його адресу.
mail-outbox-uncertain = Стан доставлення невідомий
mail-outbox-uncertain-note = Надсилання розпочалося, але доставлення підтвердити не вдалося. Це повідомлення не буде автоматично надіслано повторно. Перевірте теку «Надіслані» або уточніть в отримувача, перш ніж надсилати знову.
mail-conversation-title = Показати листування
mail-conversation-help = Перегляд доступних повідомлень листування цього облікового запису з усіх тек. Відкриття повідомлення не змінює вибрану теку.
mail-conversation-count = Показано { $loaded } із { $total } повідомлень листування.
mail-conversation-other-folder = Інша тека
mail-conversation-loading = Завантаження листування…
mail-conversation-missing = Недоступних повідомлень: { $count }.
mail-conversation-error = Не вдалося завантажити листування. Спробуйте ще раз.
mail-conversation-more = Завантажити більше повідомлень
mail-conversation-limit = У цьому вигляді показується не більше { $count } повідомлень.
mail-compose-request-delivery = Запитувати сповіщення про доставлення
mail-compose-request-delivery-hint = Попросити поштові сервери одержувачів повідомляти про доставлення, збій або затримку. Це не запит і не підтвердження того, що хтось прочитав повідомлення. Повернені звіти містять заголовки, а не повний текст.
mail-compose-delivery-unavailable = Сповіщення про доставлення недоступні або вимкнені адміністратором для цього облікового запису. Явно запитане сповіщення не буде видалено непомітно.
mail-compose-account-changed = Це повідомлення належить до іншого входу. Увійдіть у потрібний обліковий запис і знову відкрийте чернетку.
mail-context-open-message = Відкрити повідомлення
mail-context-open-folder = Відкрити теку
mail-context-download = Завантажити повідомлення
mail-context-refresh = Оновити теку
mail-context-folder-settings = Налаштування теки
mail-context-failed = Не вдалося виконати дію. Спробуйте ще раз.
mail-sharing-unavailable = Спільний доступ до теки недоступний, або у вас більше немає прав на керування ним.
mail-trash-cancelling = Зупинка після поточного запиту. Підтверджено видалено: { $deleted }; пропущено: { $skipped }; залишилося: { $remaining }. Уже прийняті видалення скасувати не можна.
mail-trash-already-running = Видалення з кошика вже виконується. Скористайтеся його сповіщенням про хід, щоб зупинити наступні пакети.
mail-context-search-sender = Шукати цього відправника
mail-context-search-subject = Шукати ту саму тему
mail-drag-too-many = Перетягуйте не більше { $count } повідомлень за раз. Перед переміщенням виберіть меншу групу.
mail-subtab-reorder-help = Перетягніть, щоб змінити порядок вкладок, або натисніть Ctrl+Shift+стрілка ліворуч чи праворуч на активній вкладці.
mail-compose-ribbon = Стрічка редактора
mail-compose-ribbon-format = Форматування тексту
mail-compose-ribbon-insert = Вставлення
mail-compose-font = Шрифт
mail-compose-font-size = Розмір шрифту
mail-compose-font-inherit = За замовчуванням
mail-compose-undo = Скасувати
mail-compose-redo = Повторити
mail-compose-undo-tip = Скасувати ({ $shortcut })
mail-compose-redo-tip = Повторити ({ $shortcut })
mail-workspace-limit = Закрийте вкладку, перш ніж відкривати іншу. Можна тримати відкритими до 32 вкладок.
mail-compose-private-storage-unavailable = Локальне відновлення чернетки недоступне в цьому сеансі. Не закривайте цей редактор, доки не надішлете або не скопіюєте повідомлення.

# -- Machine translation (2026-11): added to match en ---------------------
mail-offline-storage-unavailable = Зміни офлайн недоступні, доки цей сеанс не підтвердить права на сховище. Підключіться знову й увійдіть ще раз.
mail-draft-status-idle = Ще не збережено в обліковому записі
mail-draft-status-pending = Незбережені зміни
mail-draft-status-saving = Збереження до «Чернеток»…
mail-draft-status-saved = Збережено до «Чернеток» облікового запису
mail-draft-status-error = Чернетку не збережено. Локальна копія для відновлення лишається доступною.
mail-draft-status-uncertain = Підтвердження збереження немає. Перевірте «Чернетки», перш ніж створювати ще одну копію; автоматичне збереження призупинено.
mail-draft-status-conflict = Цю чернетку змінено на іншому пристрої. Відкрийте її копію з облікового запису знову, перш ніж продовжити.
mail-draft-status-unsupported = Цей обліковий запис зараз не може підтверджувати збереження до теки «Чернетки».
mail-draft-protected = Зашифрований або розшифрований цитований вміст не зберігається відкритим текстом. Попередня чернетка в обліковому записі лишається, доки ви явно її не видалите.
mail-draft-save-now = Зберегти до «Чернеток»
mail-draft-save-failed = Не вдалося зберегти цю чернетку в обліковому записі. Перевірте редактор, перш ніж закривати його.
mail-draft-uncertain = Можливо, чернетку збережено. Автоматичні повтори призупинено; перевірте теку «Чернетки».
mail-draft-conflict = Інший пристрій змінив цю чернетку. Ваші зміни не перезаписали її копію.
mail-draft-cleanup-pending = Нову чернетку збережено, але її попередню копію видалити не вдалося. Перегляньте «Чернетки», перш ніж зберігати знову.
mail-draft-account-list = Чернетки облікового запису
mail-draft-loading = Завантаження чернеток облікового запису…
mail-draft-close-recovery = Закрити й залишити лише локальну копію для відновлення
mail-draft-discard-account = Видалити
mail-draft-discard-pending = Не вдалося підтвердити видалення чернетки. Її запис в обліковому записі лишається; повторіть «Видалити» після перевірки теки «Чернетки».
mail-draft-limit = В цьому обліковому записі вже 100 збережених чернеток. Видаліть непотрібну чернетку, перш ніж зберігати нову.
mail-draft-quota-help = Зберігається до 100 чернеток облікового запису. Під час видалення спершу вилучається підтверджена копія на сервері, потім звільняється місце.
mail-draft-discard-confirm = Підтвердьте видалення чернетки
mail-draft-discard-confirm-detail = Безповоротно видалити «{ $subject }» з «Чернеток» облікового запису? Якщо закрити редактор, чернетка збережеться.
mail-draft-discard-title = Видалити цю чернетку?
mail-draft-keep-editing = Продовжити редагування
mail-draft-discard-action = Видалити
mail-draft-close-title = Скасувати незбережені зміни?
mail-draft-close-protected = Це повідомлення містить зашифрований або розшифрований вміст, тому не зберігається як чернетка. Під час закриття ці зміни буде скасовано. Попередня чернетка облікового запису, якщо вона є, лишається в «Чернетках».
mail-draft-close-unstored = У цьому сеансі чернетки зберегти не можна, тому під час закриття повідомлення буде видалено.
mail-workspace-view-mode = Розташування виглядів
mail-workspace-tabs-help = Показувати один активний вигляд, не закриваючи інші вкладки.
mail-workspace-side-by-side-help = Показувати два вибрані вигляди поруч. Перетягніть вкладку на будь-який вигляд, щоб замінити його.
mail-workspace-stack-help = Показувати два вибрані вигляди один над одним. Перетягніть вкладку на будь-який вигляд, щоб замінити його.
mail-workspace-masonry-side-by-side = Мозаїка поруч
mail-workspace-masonry-side-by-side-help = Розташувати всі відкриті вигляди в зміщених стовпцях. На малих екранах показується активна вкладка.
mail-workspace-masonry-stack = Мозаїка одна під одною
mail-workspace-masonry-stack-help = Розташувати всі відкриті вигляди в зміщених рядках. На малих екранах показується активна вкладка.
mail-workspace-view-first = Перший вигляд
mail-workspace-view-second = Другий вигляд
mail-workspace-view-empty = Перетягніть сюди відкриту вкладку, щоб вибрати цей вигляд.
mail-popup-blocked = Дозвольте спливні вікна для Mailwoman, щоб від'єднати цю вкладку. Вкладка лишається тут.
mail-popup-unavailable = Не вдалося завантажити окреме вікно. Вкладка й незбережені зміни лишаються в головному вікні. Спробуйте ще раз після оновлення застосунку.
mail-popup-reattach = Повернути до головного вікна
mail-popup-action-reattached = Повернення до головного вікна для завершення цієї дії.
mail-popup-detached = Ця вкладка відкрита в окремому вікні. Закриття того вікна поверне її сюди.
mail-compose-from-custom = Інша дозволена адреса…
mail-compose-from-alias = Адреса відправника для цього повідомлення
mail-compose-from-alias-help = Використайте псевдонім, з якого вам дозволено надсилати. Змінюється лише ця чернетка; дозвіл перевіряється перед надсиланням.
mail-compose-from-refused = Ця адреса не дозволена для надсилання з цього облікового запису.
mail-compose-from-unavailable = Не вдалося перевірити адресу відправника. Повідомлення не надіслано.
mail-compose-from-invalid = Введіть одну дійсну адресу електронної пошти відправника.
mail-compose-from-toggle = Показати або сховати відправника для цього повідомлення
mail-compose-ribbon-security = Безпека
mail-compose-ribbon-delivery = Доставлення
mail-compose-ribbon-recipients = Отримувачі
mail-compose-ribbon-message-type = Тип повідомлення
mail-compose-recall-help = Скасуйте повідомлення, які ще утримуються для скасування надсилання або запланованого доставлення. Уже доставлені повідомлення відкликати не можна.
mail-compose-recall-loading = Перевірка повідомлень у черзі…
mail-compose-recall-empty = Немає повідомлень у черзі, які можна скасувати.
mail-compose-recall-failed = Не вдалося підтвердити стан черги. Закрийте й знову відкрийте цю панель, щоб оновити її, перш ніж виконувати іншу дію.
mail-compose-importance = Важливість
mail-compose-importance-high = Висока
mail-compose-importance-normal = Звичайна
mail-compose-importance-low = Низька
mail-insert-account-title = Вставити елемент облікового запису
mail-insert-account-help = Прикріпити експорт календаря, візитну картку контакту або вихідне повідомлення з цього облікового запису.
mail-insert-account-close = Закрити вибір елемента облікового запису
mail-insert-account-kind = Тип елемента
mail-insert-account-event = Подія календаря (.ics)
mail-insert-account-calendar = Календар (.ics)
mail-insert-account-contact = Візитна картка контакту (.vcf)
mail-insert-account-message = Вихідне повідомлення (.eml)
mail-insert-account-search = Фільтр за списком
mail-insert-account-selection = Виберіть елемент
mail-insert-account-scope = Показано лише елементи поточного облікового запису. Для вихідних повідомлень діє політика експорту сервера.
mail-insert-account-limited = Показано до 100 елементів. Фільтр шукає лише в цьому списку.
mail-insert-account-unavailable = Підтримуваних експортів облікового запису немає.
mail-insert-account-empty = Відповідних елементів немає.
mail-insert-account-attach = Прикріпити вибраний елемент
mail-insert-account-loading = Завантаження елемента облікового запису…
mail-insert-account-error = Не вдалося прикріпити елемент. Перевірте доступ і політику експорту та спробуйте ще раз. Для великих календарів може знадобитися експорт окремих подій.
mail-compose-html-source = Вихідний код HTML
mail-editor-clear-format = Очистити форматування
mail-compose-format-html = HTML
mail-compose-html-visual = Візуальний редактор
mail-compose-html-help = Редагуйте HTML безпосередньо. Зберігається й надсилається лише підтримуване форматування пошти; скрипти, зовнішні зображення й небезпечна розмітка видаляються.
mail-compose-html-processing = Підготовка безпечного HTML…
mail-compose-html-failed = Не вдалося підготувати HTML. Відредагуйте його або спробуйте ще раз перед збереженням чи надсиланням.
mail-compose-check-names = Перевірити імена
mail-compose-check-names-help = Зіставити точні імена із завантаженими контактами й перевірити формат адрес. Доставлення до поштової скриньки при цьому не перевіряється.
mail-compose-names-unresolved = Виберіть повну адресу електронної пошти для цих нерозпізнаних або неоднозначних імен: { $names }
mail-compose-names-checked = Формат отримувачів перевірено; розпізнано імен контактів: { $count }. Доставлення до поштової скриньки не перевірено.
mail-compose-ribbon-table = Таблиця
mail-compose-group-clipboard = Буфер обміну
mail-compose-group-paragraph = Абзац
mail-compose-group-editing = Редагування
mail-compose-group-view = Вигляд
mail-compose-dark-background = Темне тло
mail-compose-dark-background-help = Показувати повідомлення на темному тлі під час написання. Змінюється лише ваш екран: повідомлення надсилається без кольору тла.
mail-compose-group-assist = Помічник із написання
mail-editor-counts = Статистика редактора
mail-editor-words = { $count ->
    [one] { $count } слово
    [few] { $count } слова
    [many] { $count } слів
   *[other] { $count } слова
}
mail-editor-paragraphs = { $count ->
    [one] { $count } абзац
    [few] { $count } абзаци
    [many] { $count } абзаців
   *[other] { $count } абзацу
}
mail-editor-characters = { $count ->
    [one] { $count } символ
    [few] { $count } символи
    [many] { $count } символів
   *[other] { $count } символу
}
mail-editor-selection-required = Виділіть відповідний текст або клітинки таблиці, перш ніж використовувати цю дію.
mail-editor-clipboard-unavailable = Доступ до буфера обміну недоступний. Скористайтеся сполученням «Копіювати», «Вирізати» або «Вставити» в браузері.
mail-editor-selection-changed = Виділення змінилося. Виділіть потрібний текст і спробуйте ще раз.
mail-editor-clipboard = Буфер обміну та форматування
mail-editor-copy = Копіювати
mail-editor-cut = Вирізати
mail-editor-paste = Вставити
mail-editor-copy-format = Копіювати формат
mail-editor-apply-format = Застосувати формат
mail-editor-style = Стиль абзацу
mail-editor-text-color = Колір шрифту
mail-editor-clear-color = Колір за замовчуванням
mail-editor-highlight = Виділення кольором
mail-editor-clear-highlight = Прибрати виділення кольором
mail-editor-paragraph = Параметри абзацу
mail-editor-alignment = Вирівнювання
mail-editor-line-height = Висота рядка
mail-editor-space-before = Інтервал перед
mail-editor-space-after = Інтервал після
mail-editor-indent = Відступ
mail-editor-shading = Заливка абзацу
mail-editor-clear-shading = Прибрати заливку
mail-editor-case = Змінити регістр
mail-editor-sort = Сортувати виділені абзаци
mail-editor-sort-alpha = За алфавітом
mail-editor-sort-numeric = За числовим порядком
mail-editor-table-rows = Рядки таблиці
mail-editor-table-columns = Стовпці таблиці
mail-editor-table-insert = Вставити таблицю
mail-editor-table-limits = До 20 рядків, 10 стовпців і 200 клітинок у таблиці; не більше 10 таблиць. Вкладені таблиці не підтримуються.
mail-editor-table-row-before = Рядок вище
mail-editor-table-row-after = Рядок нижче
mail-editor-table-row-delete = Видалити рядок
mail-editor-table-column-before = Стовпець ліворуч
mail-editor-table-column-after = Стовпець праворуч
mail-editor-table-column-delete = Видалити стовпець
mail-editor-table-merge = Об'єднати клітинки
mail-editor-table-split = Розділити клітинку
mail-editor-table-header = Перемкнути рядок заголовка
mail-editor-table-delete = Видалити таблицю
mail-editor-table-shading = Заливка клітинки
mail-editor-find-replace = Знайти й замінити
mail-editor-find = Знайти текст
mail-editor-replacement = Замінити на
mail-editor-match-case = Враховувати регістр
mail-editor-previous = Попередній збіг
mail-editor-next = Наступний збіг
mail-editor-replace = Замінити вибраний збіг
mail-editor-replace-all = Замінити всі збіги
mail-editor-matches = Збігів: { $count } (не більше 1000)
mail-editor-select-all = Виділити все
mail-editor-clear-selection = Зняти виділення
mail-editor-history = Історія скасування
mail-editor-undo-groups = Скасувати останні групи змін: { $count }
mail-editor-zoom = Масштаб редактора
mail-editor-markers = Позначки абзаців
mail-editor-style-normal = Звичайний
mail-editor-style-no-spacing = Без інтервалу
mail-editor-style-title = Заголовок
mail-editor-style-subtitle = Підзаголовок
mail-editor-style-heading1 = Заголовок 1
mail-editor-style-heading2 = Заголовок 2
mail-editor-style-heading3 = Заголовок 3
mail-editor-style-heading4 = Заголовок 4
mail-editor-style-heading5 = Заголовок 5
mail-editor-style-heading6 = Заголовок 6
mail-editor-align-left = За лівим краєм
mail-editor-align-center = По центру
mail-editor-align-right = За правим краєм
mail-editor-align-justify = За шириною
mail-editor-case-lower = малі літери
mail-editor-case-upper = ВЕЛИКІ ЛІТЕРИ
mail-editor-case-title = Кожне Слово З Великої
mail-editor-case-sentence = Як у реченнях
mail-editor-action-history = Історія: { $action }
mail-editor-history-empty = Немає доступних дій
mail-editor-redo-groups = Повторити зміни: { $count }
mail-compose-identities-failed = Не вдалося завантажити профілі надсилання. Закрийте й знову відкрийте цю чернетку, щоб спробувати ще раз.
mail-folders-loading = Завантаження тек…
mail-initial-sync-error = Ваші повідомлення поки не вдалося синхронізувати.
mail-initial-sync-error-help = З'єднання з поштовим сервером перервано. Спробуйте ще раз або попросіть адміністратора перевірити журнали підключень, якщо проблема повторюється.
mail-links-trusted = Довірений домен: відкривається без перевірки
mail-links-trust-domain = Довіряти { $domain }
mail-links-trust-failed = Не вдалося зберегти довірений домен. Спробуйте ще раз.
mail-focus-always-focused = Завжди переміщувати до «Важливих»
mail-focus-always-other = Завжди переміщувати до «Інших»
mail-focus-moved-focused = Повідомлення від { $sender } тепер потрапляють до «Важливих».
mail-focus-moved-other = Повідомлення від { $sender } тепер потрапляють до «Інших».
mail-focus-save-failed = Не вдалося зберегти вибір «Важливих вхідних» для цього відправника. Спробуйте ще раз.
mail-context-search-domain = Шукати повідомлення з цього домену

# -- Machine translation (2026-11): added to match en ---------------------
mail-loading-mailbox = Завантаження: { $name }…
mail-user-folders = Користувацькі теки
mail-context-hide-folder = Сховати з бічної панелі
mail-folder-fallback = тека
mail-toast-moving = { $count ->
    [one] Переміщення { $count } повідомлення до теки «{ $folder }»…
    [few] Переміщення { $count } повідомлень до теки «{ $folder }»…
    [many] Переміщення { $count } повідомлень до теки «{ $folder }»…
   *[other] Переміщення { $count } повідомлення до теки «{ $folder }»…
}
mail-toast-move-queueing = { $count ->
    [one] Постановка в чергу { $count } повідомлення для переміщення до теки «{ $folder }»…
    [few] Постановка в чергу { $count } повідомлень для переміщення до теки «{ $folder }»…
    [many] Постановка в чергу { $count } повідомлень для переміщення до теки «{ $folder }»…
   *[other] Постановка в чергу { $count } повідомлення для переміщення до теки «{ $folder }»…
}
mail-toast-move-queued = { $count ->
    [one] У черзі { $count } повідомлення для переміщення до теки «{ $folder }»
    [few] У черзі { $count } повідомлення для переміщення до теки «{ $folder }»
    [many] У черзі { $count } повідомлень для переміщення до теки «{ $folder }»
   *[other] У черзі { $count } повідомлення для переміщення до теки «{ $folder }»
}
mail-toast-move-partial = Переміщено { $count } із { $total } повідомлень до теки «{ $folder }»; не вдалося перемістити: { $failed }
mail-send-copy-folder-unavailable = Вибрана тека для копій надісланих недоступна. Виберіть іншу теку перед надсиланням.
mail-send-copy-failed = Повідомлення прийнято, але його копію в надісланих зберегти не вдалося. Не надсилайте його повторно.
mail-compose-attach-public-key = Прикріпити мій відкритий ключ або сертифікат
mail-compose-include-hashes = Додати хеші SHA-256 прикріплених файлів
mail-compose-hashes-pending = Обчислення хешів вкладень. Надішліть, коли буде готово.
mail-compose-hashes-failed = Не вдалося перевірити байти вкладень. Вимкніть хеші або спробуйте ще раз, увімкнувши їх знову.
mail-compose-crypto-pending = Шифрування оновлюється. Зачекайте перед надсиланням.
mail-compose-public-key-unavailable = Жоден чинний відкритий ключ або сертифікат не відповідає адресі відправника.
mail-compose-sent-copy = Зберігати надіслане повідомлення в
mail-compose-sent-copy-default = Типово для облікового запису
folder-create = Створити теку
folder-name = Назва теки
folder-parent = Батьківська тека
folder-top-level = Верхній рівень
folder-created = Теку створено.
folder-create-unavailable = Створення тек недоступне для цього облікового запису або батьківської теки.
folder-create-invalid = Виберіть допустиму унікальну назву теки без роздільників ієрархії та керівних символів.
folder-create-failed = Не вдалося створити теку. Оновіть сторінку й спробуйте ще раз.
folder-cleanup = Очистити теку…
folder-cleanup-title = Очищення теки «{ $name }»
folder-cleanup-help = Знайти повідомлення старші за вибраний вік. Перевірте кількість перед видаленням. Підтеки не зачіпаються.
folder-cleanup-age = Старші за
folder-age-days = Дні
folder-age-weeks = Тижні
folder-age-months = Місяці
folder-age-years = Роки
folder-cleanup-preview = Попередній перегляд очищення
folder-cleanup-confirm = Видалити знайдені повідомлення
folder-cleanup-permanent = Безповоротно видалити знайдені повідомлення: { $count }. Цю дію не можна скасувати.
folder-cleanup-trash = Перемістити знайдені повідомлення до кошика: { $count }.
folder-cleanup-before = Отримано до { $date }
folder-cleanup-done = Очищено повідомлень: { $count }.
mail-from = Відправник
mail-subject = Тема
mail-date = Дата
mail-cancel = Скасувати
mail-bulk-selected = Вибрано листувань: { $count }
mail-bulk-select-folder = Вибрати все в теці «{ $folder }»
mail-list-order = Порядок повідомлень
mail-list-order-newest = Спершу нові
mail-list-order-oldest = Спершу старі
mail-list-order-sender = Відправник
mail-list-order-subject = Тема
mail-list-order-error = Не вдалося зберегти порядок повідомлень.
mail-bulk-selection-load-failed = Не вдалося вибрати всю теку. Перевірте з'єднання й спробуйте ще раз.
mail-privacy-bar = Конфіденційність повідомлення
mail-privacy-trackers-allow = Дозволити трекери для цього повідомлення
mail-privacy-trackers-block = Знову блокувати трекери
mail-privacy-images-loading = Показано зображень: { $loaded }/{ $total } · завантажується: { $pending }
mail-privacy-images-failed = Показано зображень: { $loaded }/{ $total } · помилок: { $failed }
mail-privacy-images-loaded = Показано зображень: { $loaded }/{ $total }
mail-privacy-images-loading-failed = Показано зображень: { $loaded }/{ $total } · завантажується: { $pending } · помилок: { $failed }
mail-reader-folder-title = Тека: { $folder }
mail-reader-move-to = Перемістити до…
mail-action-help-move = Перемістити це повідомлення до іншої теки.
folder-ops-new-subfolder = Нова підтека
folder-ops-mark-read = Позначити все як прочитане
folder-ops-this-folder = Ця тека
folder-ops-with-subfolders = Разом із підтеками
folder-ops-no-unread = Непрочитаних повідомлень немає.
folder-ops-read-unavailable = Ви не можете змінювати повідомлення в цій теці.
folder-ops-more = Ще
folder-ops-system-folder = Системну теку не можна змінити таким способом.
folder-ops-sort-subfolders = Сортувати підтеки за
folder-ops-sort-folders = Сортувати теки за
folder-ops-sort-name = Назвою
folder-ops-sort-unread = Кількістю непрочитаних
folder-ops-sort-total = Загальною кількістю
folder-ops-sort-date = Останньою активністю
folder-ops-sort-custom = Власний порядок
folder-ops-sort-asc = За зростанням
folder-ops-sort-desc = За спаданням
folder-ops-delete-with-subfolders = Видалити теку й підтеки
folder-ops-delete-unavailable = Цю теку не можна видалити.
folder-ops-delete-tree-body = Буде безповоротно видалено теки ({ $folders }) і повідомлення в них ({ $count }). Цю дію не можна скасувати.
folder-ops-copy-all-menu = Копіювати всі елементи до…
folder-ops-move-all-menu = Перемістити всі елементи до…
folder-ops-move-folder-menu = Перемістити теку до…
folder-ops-move-up = Перемістити вгору
folder-ops-move-down = Перемістити вниз
folder-ops-move-unavailable = Ви не можете перемістити цю теку або її елементи.
folder-ops-move-duplicate = У місці призначення вже є тека з такою назвою.
folder-ops-move-folder-title = Перемістити «{ $name }»
folder-ops-move-folder-body = Тека переміститься разом із підтеками та повідомленнями. Виберіть теку, до якої її помістити, або верхній рівень.
folder-ops-move-here = Перемістити сюди
folder-ops-moved-folder = Теку «{ $name }» переміщено.
folder-ops-choose-folder = Виберіть цільову теку.
folder-ops-destination = Цільова тека
folder-ops-include-subfolders = Включити повідомлення з підтек: { $count }
folder-ops-copy-all-title = Копіювання всіх елементів теки «{ $name }»
folder-ops-move-all-title = Переміщення всіх елементів теки «{ $name }»
folder-ops-copy-all-body = Додає повідомлення ({ $count }) до цільової теки. Вони лишаються на попередньому місці.
folder-ops-move-all-body = Переміщує повідомлення ({ $count }) до цільової теки. Потім це можна скасувати.
folder-ops-copy-all = Копіювати елементи
folder-ops-move-all = Перемістити елементи
folder-ops-undo = Скасувати
folder-ops-stop = Зупинити
folder-ops-marking = Позначення теки «{ $name }» як прочитаної…
folder-ops-marking-progress = Позначення теки «{ $name }» як прочитаної: { $done } із { $total }
folder-ops-marked = Позначено як прочитані повідомлень у теці «{ $name }»: { $count }.
folder-ops-copying = Копіювання: «{ $name }»…
folder-ops-copying-progress = Копіювання «{ $name }»: { $done } із { $total }
folder-ops-copied = Скопійовано повідомлень із теки «{ $name }»: { $count }.
folder-ops-moving = Переміщення: «{ $name }»…
folder-ops-moving-progress = Переміщення «{ $name }»: { $done } із { $total }
folder-ops-moved = Переміщено повідомлень із теки «{ $name }»: { $count }.
folder-ops-stopped = Операцію з текою «{ $name }» зупинено після { $count } із { $total } повідомлень.
folder-ops-partial = У теці «{ $name }» оброблено { $count } із { $total } повідомлень; відхилено: { $refused }.
folder-ops-failed = Не вдалося завершити операцію з текою «{ $name }». Оновіть сторінку й спробуйте ще раз.
folder-ops-error-forbidden = У вас немає прав на цю зміну.
folder-ops-error-exists = Тека з такою назвою там уже існує.
folder-ops-error-mailboxHasEmail = У теці ще є повідомлення.
folder-ops-error-notFound = Теки більше не існує.
folder-ops-error-failed = Не вдалося внести зміну. Оновіть сторінку й спробуйте ще раз.
folder-ops-expand = Показати підтеки теки «{ $name }»
folder-ops-collapse = Сховати підтеки теки «{ $name }»
mail-context-reply = Відповісти
mail-context-reply-all = Відповісти всім
mail-context-forward = Переслати
mail-context-quick-print = Швидкий друк
mail-context-copy = Копіювати
mail-context-copy-to-folder = Копіювати повідомлення до теки…
mail-context-copy-sender = Копіювати адресу відправника
mail-context-copy-subject = Копіювати тему
mail-context-copy-link = Копіювати посилання на повідомлення
mail-context-copied = Скопійовано.
mail-context-copy-failed = Не вдалося скопіювати до буфера обміну.
mail-context-print-failed = Не вдалося надрукувати повідомлення.
mail-reply-images-title = Зображення з вихідного повідомлення
mail-reply-images-ask = Додати до цієї відповіді зображення вихідного повідомлення?
mail-reply-images-include = Додати зображення
mail-reply-images-not-now = Не зараз
mail-reply-images-trust = Завжди довіряти домену { $domain }
mail-reply-images-loading = Додавання зображень…
mail-reply-images-cancelled = Завантаження скасовано. Зображення не додано.
mail-reply-images-failed = Не вдалося додати зображення.
mail-reply-images-retry = Завантажити знову
mail-image-cancel = Скасувати
mail-compose-from-why-match = Надсилання від імені «{ $identity }», бо повідомлення було надіслано на адресу { $address }.
mail-compose-from-why-match-plus = Надсилання від імені «{ $identity }», бо повідомлення було надіслано на адресу { $address }, що є його адресою з плюсом.
mail-compose-from-why-ambiguous = Адресі { $address } відповідає кілька профілів. Надсилання від імені «{ $identity }»; виберіть інший:
mail-compose-from-why-default = Надсилання від імені вашого профілю за замовчуванням «{ $identity }», бо жоден не відповідає початковим отримувачам.
mail-compose-from-why-forced-default = Надсилання від імені вашого профілю за замовчуванням «{ $identity }», як задано в налаштуваннях.
mail-compose-from-why-keep = Надсилання від імені «{ $identity }».
mail-compose-from-change = Змінити
reader-attachments-reason-encrypted = Це повідомлення зашифроване. Його вкладення з'являться після розблокування.
reader-attachments-reason-list-missing = Не вдалося завантажити список вкладень цього повідомлення, тому панель вкладень сховано.
reader-attachments-reason-inline-only = Вкладення цього повідомлення ({ $count }) — вбудовані зображення, показані в самому повідомленні.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] У { $count } вкладення цього повідомлення немає вмісту для завантаження, тому панель вкладень сховано.
    [few] У { $count } вкладень цього повідомлення немає вмісту для завантаження, тому панель вкладень сховано.
    [many] У { $count } вкладень цього повідомлення немає вмісту для завантаження, тому панель вкладень сховано.
   *[other] У { $count } вкладення цього повідомлення немає вмісту для завантаження, тому панель вкладень сховано.
  }
reader-attachments-reason-policy-blocked = Політика вкладень блокує завантаження { $count ->
    [one] цього вкладення.
    [few] цих вкладень.
    [many] цих вкладень.
   *[other] цих вкладень.
  }
reader-attachments-reason-policy-unavailable = Не вдалося перевірити політику вкладень, тому вкладення поки не можна відкрити.
mail-drafts-popup-count = { $count ->
    [one] { $count } чернетка
    [few] { $count } чернетки
    [many] { $count } чернеток
   *[other] { $count } чернетки
  }
mail-drafts-popup-search = Пошук за темою або отримувачем
mail-drafts-popup-tab-local = Локальні чернетки
mail-drafts-popup-local-hint = Локальні чернетки зберігаються лише в цьому браузері.
mail-drafts-popup-badge-account = Обліковий запис
mail-drafts-popup-badge-local = Локальна
mail-drafts-popup-current = Відкрита зараз
mail-drafts-popup-select = Вибрати
mail-drafts-popup-select-all = Вибрати всі
mail-drafts-popup-clear = Скинути
mail-drafts-popup-done = Готово
mail-drafts-popup-selected = Вибрано: { $count }
mail-drafts-popup-select-row = Вибрати: { $subject }
mail-drafts-popup-open = Відкрити
mail-drafts-popup-discard = Видалити
mail-drafts-popup-discard-selected = Видалити вибрані
mail-drafts-popup-more = Інші дії для: { $subject }
mail-drafts-popup-retry = Повторити
mail-drafts-popup-load-failed = Не вдалося завантажити чернетки облікового запису.
mail-drafts-popup-empty-account = Немає чернеток облікового запису
mail-drafts-popup-no-match = Немає чернеток, що відповідають «{ $query }»
mail-drafts-popup-discarding = Видалення…
mail-drafts-popup-row-failed = Не вдалося видалити. Її копія в обліковому записі лишається.

# -- Machine translation (2026-11): added to match en ---------------------
mail-drafts-popup-confirm-title = { $count ->
    [one] Видалити цю чернетку?
    [few] Видалити { $count } чернетки?
    [many] Видалити { $count } чернеток?
   *[other] Видалити { $count } чернетки?
  }
mail-drafts-popup-confirm-account = Чернетки облікового запису спершу видаляються з теки «Чернетки» в обліковому записі, а їхнє місце звільняється лише після підтвердження видалення.
mail-drafts-popup-confirm-local = Локальні чернетки видаляються з цього браузера.
mail-drafts-popup-announce-done = { $count ->
    [one] Видалено { $count } чернетку
    [few] Видалено { $count } чернетки
    [many] Видалено { $count } чернеток
   *[other] Видалено { $count } чернетки
  }
mail-drafts-popup-announce-failed = Видалено: { $done }, не вдалося видалити: { $failed }
mail-check-title = Перевірте перед надсиланням
mail-check-intro = Варто ще раз переглянути таке:
mail-check-invalid = Деякі адреси отримувачів виглядають недійсними: { $list }
mail-check-subject = Тема порожня.
mail-check-attachment = У повідомленні згадується вкладення, але нічого не прикріплено.
mail-check-external = Отримувачі поза довіреними доменами: { $list }
mail-check-send-anyway = Усе одно надіслати
mail-check-review = Перевірити
mail-check-remember = Більше не питати про це
mail-check-denied-invalid = Надсилання заблоковано вашими налаштуваннями. Виправте недійсні адреси отримувачів: { $list }
mail-check-denied-subject = Надсилання заблоковано вашими налаштуваннями. Додайте тему.
mail-check-denied-attachment = Надсилання заблоковано вашими налаштуваннями. Прикріпіть згаданий файл або змініть текст повідомлення.
mail-check-denied-external = Надсилання заблоковано вашими налаштуваннями. Поза довіреними доменами: { $list }
mail-alerts-label = Проблеми з повідомленням
mail-alert-invalid = Недійсна адреса: { $address }
mail-alert-subject = Без теми
mail-recipient-chip-remove = Видалити: { $address }
mail-recipient-chips = Отримувачі в полі «{ $field }»
mail-recipient-invalid-title = Недійсна адреса електронної пошти
mail-recipient-chip-name = { $name }, { $address }. Enter — відомості про контакт, Backspace або F2 — змінити, Delete — видалити
mail-recipient-chip-name-invalid = { $address }, недійсна адреса. Enter — змінити, Delete — видалити
mail-recipient-added = Додано: { $name }
mail-recipient-removed = Видалено: { $name }
mail-recipient-removed-many = { $count ->
    [one] Видалено { $count } отримувача
    [few] Видалено { $count } отримувачів
    [many] Видалено { $count } отримувачів
   *[other] Видалено { $count } отримувача
}
mail-recipient-editing = Редагування: { $name }
mail-recipient-all-selected = { $count ->
    [one] Вибрано { $count } отримувача
    [few] Вибрано { $count } отримувачів
    [many] Вибрано { $count } отримувачів
   *[other] Вибрано { $count } отримувача
}
mail-external-line = Поза довіреними доменами: { $count } ({ $domains })
mail-external-dismiss = Закрити
mail-status-label = Стан редактора
mail-status-more = Ще сповіщень: { $count }
mail-status-more-label = Інші сповіщення
mail-status-no-recipients = Немає отримувачів
mail-status-no-attachment = Немає вкладення
mail-status-external = Зовнішні: { $count }
mail-status-external-reviewed = Зовнішніх отримувачів перевірено
mail-status-external-review = Позначити як перевірених
mail-status-from-match = Від: { $identity }
mail-status-from-match-plus = Від: { $identity }
mail-status-from-ambiguous = Від: { $identity } (підходить кілька)
mail-status-from-default = Від: профіль за замовчуванням
mail-status-from-forced-default = Від: профіль за замовчуванням
mail-status-from-keep = Від: { $identity }
mail-status-held = У черзі: { $count }
mail-status-storage = Немає локального відновлення
mail-status-protected = Не збережено відкритим текстом
mail-status-saving = Збереження…
mail-status-saved = Збережено { $time }
mail-status-saved-full = Збережено до «Чернеток» облікового запису · { $time }
mail-status-not-saved = Чернетку не збережено
mail-status-retry = Повторити
mail-status-conflict = Змінено на іншому пристрої
mail-status-uncertain = Збереження не підтверджено
mail-status-close-recovery = Закрити, залишити копію для відновлення
mail-status-unsupported = Збереження чернетки не підтверджено
mail-compose-header = Заголовок повідомлення
mail-header-label = Заголовок повідомлення
mail-header-collapse = Згорнути заголовок повідомлення
mail-header-expand = Розгорнути заголовок повідомлення: { $summary }
mail-header-from-chip = Від { $address }. Змінити відправника
mail-compose-group-timing = Час надсилання
mail-compose-group-priority = Пріоритет
mail-compose-group-receipts = Сповіщення та відстеження
mail-compose-group-keys-files = Ключі та файли
mail-compose-group-sent-copy = Копія надісланого
mail-editor-style-quote = Цитата
mail-editor-style-code = Код
mail-compose-ribbon-table-layout = Макет таблиці
mail-compose-ribbon-picture = Малюнок
mail-editor-table-group-rows = Рядки
mail-editor-table-group-columns = Стовпці
mail-editor-table-group-cells = Клітинки
mail-editor-table-row-height = Висота рядка
mail-editor-table-column-width = Ширина стовпця
mail-editor-table-size-auto = Авто
mail-editor-table-padding = Поля клітинки
mail-editor-table-padding-compact = Вузькі
mail-editor-table-padding-normal = Звичайні
mail-editor-table-padding-roomy = Широкі
mail-editor-table-align-left = Вирівняти клітинку за лівим краєм
mail-editor-table-align-center = Вирівняти клітинку по центру
mail-editor-table-align-right = Вирівняти клітинку за правим краєм
mail-picture-group-size = Розмір
mail-picture-group-arrange = Розташування
mail-picture-group-text = Текст
mail-picture-group-style = Стиль
mail-picture-group-edit = Редагування
mail-picture-width = Ширина
mail-picture-height = Висота
mail-picture-lock = Зберігати пропорції
mail-picture-reset-size = Вихідний розмір
mail-picture-align-inline = У тексті
mail-picture-align-left = За лівим краєм
mail-picture-align-center = По центру
mail-picture-align-right = За правим краєм
mail-picture-alt = Замінний текст
mail-picture-title = Назва
mail-picture-border = Межа
mail-picture-radius = Заокруглення кутів
mail-picture-none = Немає
mail-picture-round = Круглі
mail-picture-rotate-ccw = Повернути ліворуч
mail-picture-rotate-cw = Повернути праворуч
mail-picture-flip-h = Віддзеркалити по горизонталі
mail-picture-flip-v = Віддзеркалити по вертикалі
mail-picture-crop = Обрізати
mail-picture-crop-left = Зліва (%)
mail-picture-crop-top = Зверху (%)
mail-picture-crop-right = Справа (%)
mail-picture-crop-bottom = Знизу (%)
mail-picture-crop-apply = Обрізати
mail-picture-replace = Замінити
mail-picture-remove = Видалити
mail-picture-failed-no-image = Спершу виберіть малюнок.
mail-picture-failed-no-source = У цього малюнка немає локальної копії для редагування. Замініть його.
mail-picture-failed-failed = Не вдалося змінити малюнок.
mail-context-download-attachments = Завантажити вкладення
mail-context-download-named = Завантажити { $name }
mail-context-download-all = Завантажити все як ZIP
mail-context-loading-attachments = Завантаження вкладень…
mail-context-attachments-failed = Не вдалося завантажити вкладення.
mail-context-searching = Пошук повідомлення…
mail-context-go-reply = Перейти до відповіді
mail-context-go-forward = Перейти до пересилання
mail-context-reply-not-found = Відповідь не знайдено
mail-context-forward-not-found = Пересилання не знайдено
mail-no-recipients = (немає отримувачів)
mail-attachment-downloaded = Завантажено: { $name }
mail-attachment-download-failed = Не вдалося завантажити.
mail-attachments-zip-done = Вкладення завантажено як ZIP: { $count }.
mail-attachments-zip-capped = До ZIP увійшли перші вкладення ({ $count }); межа — { $max } файлів або { $size }.
mail-focus-not-found = Повідомлення не знайдено в теці «{ $folder }»
mail-focus-go-to = Перейти: { $folder }
mail-nav-views-queues = Вигляди та черги
mail-badge-unread = { $unread ->
    [one] { $unread } непрочитане повідомлення
    [few] { $unread } непрочитані повідомлення
    [many] { $unread } непрочитаних повідомлень
   *[other] { $unread } непрочитаного повідомлення
  }
mail-badge-total = { $total ->
    [one] { $total } повідомлення в цій теці
    [few] { $total } повідомлення в цій теці
    [many] { $total } повідомлень у цій теці
   *[other] { $total } повідомлення в цій теці
  }
mail-badge-both = { $unread ->
    [one] { $unread } непрочитане повідомлення
    [few] { $unread } непрочитані повідомлення
    [many] { $unread } непрочитаних повідомлень
   *[other] { $unread } непрочитаного повідомлення
  } із { $total ->
    [one] { $total } у цій теці
    [few] { $total } у цій теці
    [many] { $total } у цій теці
   *[other] { $total } у цій теці
  }
mail-badge-drafts = { $total ->
    [one] Чернетки: { $total } повідомлення
    [few] Чернетки: { $total } повідомлення
    [many] Чернетки: { $total } повідомлень
   *[other] Чернетки: { $total } повідомлення
  }
mail-badge-compact-hint = У тек, окрім «Вхідних», значок менший. Це можна змінити в розділі «Налаштування», «Теки та бічна панель».
mail-badge-outbox = { $count ->
    [one] Вихідні: { $count } повідомлення очікує надсилання
    [few] Вихідні: { $count } повідомлення очікують надсилання
    [many] Вихідні: { $count } повідомлень очікують надсилання
   *[other] Вихідні: { $count } повідомлення очікують надсилання
  }
mail-color-menu = Кольори повідомлень
mail-color-button = Кольори повідомлень: { $mode }
mail-color-use-default = Використовувати мій варіант за замовчуванням ({ $mode })
mail-color-make-default = Зробити це моїм варіантом за замовчуванням
mail-color-locked = Заблоковано адміністратором
mail-color-confirm-title = Змінити кольори повідомлень за замовчуванням?
mail-color-confirm-body = Усі повідомлення використовуватимуть режим «{ $mode }», доки ви не зміните його для окремого повідомлення.
mail-color-confirm = Зробити типовим
mail-color-default-failed = Не вдалося зберегти кольори повідомлень за замовчуванням.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mail-attachments-none = Вибрані повідомлення не мають вкладень для завантаження.

mail-attachments-zip-all-blocked = Нічого не завантажено: політика вкладень блокує всі файли { $skipped }.

mail-attachments-zip-gathering = Збір вкладень { $count } повідомлень…

mail-attachments-zip-skipped = { $skipped ->
        [one] { $skipped } файл, заблокований політикою вкладень, вилучено.
        [few] { $skipped } файли, заблоковані політикою вкладень, були пропущені.
        [many] { $skipped } файлів, заблокованих політикою вкладень, пропущено.
       *[other] { $skipped } файли, заблоковані політикою вкладень, були пропущені.
    }

mail-attachments-zip-unreadable = Вкладені файли { $count } повідомлень не можна було прочитати та були пропущені.

mail-bulk-download-attachments = Завантажити вкладення як ZIP

mail-compose-from-via = Він відповідав заголовку { $header }.

mail-compose-from-why-match-domain = Надсилання з { $identity }, оскільки повідомлення було доставлено на { $address }, адресу в тому ж домені.

mail-compose-from-why-match-pattern = Надсилання з { $identity }, оскільки повідомлення було надіслано на { $address }, що відповідає шаблону адреси цієї особи.

mail-compose-ribbon-autohide = Автоматичне приховування стрічки (Ctrl+F1)

mail-compose-ribbon-pin = Завжди показувати стрічку (Ctrl+F1)

mail-context-download-selected-attachments = { $count ->
        [one] Завантажити вкладення вибраного повідомлення як ZIP
        [few] Завантажте вкладення { $count } вибраних повідомлень як ZIP
        [many] Завантажити вкладення { $count } вибраних повідомлень як ZIP
       *[other] Завантажте вкладення { $count } вибраних повідомлень як ZIP
    }

mail-context-download-thread-attachments = Завантажте всі { $count } вкладених файлів розмови як ZIP

mail-context-open-new-tab = Відкрити в новій вкладці

mail-delete-sender-confirm = Перемістіть { $count } повідомлень у кошик

mail-delete-sender-detail = Перемістити { $count } повідомлень із { $sender } у кошик? Повідомлення, які вже знаходяться в кошику, виключаються. Ви можете скасувати цю дію.

mail-delete-sender-empty = Немає повідомлень від цього відправника поза кошиком.

mail-delete-sender-failed = Не вдалося перевірити кожне відповідне повідомлення та його дозволи. Нічого не зрушили; спробуйте ще раз.

mail-delete-sender-help = Знайдіть повідомлення цього відправника в усіх папках і після підтвердження перемістіть їх у кошик.

mail-delete-sender-title = Видалити всі повідомлення від цього відправника

mail-draft-continue = Продовжити редагування

mail-draft-edit = Редагувати чернетку

mail-draft-edit-failed = Не вдалося відкрити чернетку для редагування

mail-draft-edit-help = Відкрийте цю чернетку в композиторі, щоб продовжити її редагування

mail-draft-notice = Це чернетка

mail-file-copied = Скопійовано в буфер обміну.

mail-file-copy-failed = Не вдалося скопіювати в буфер обміну.

mail-file-copy-fingerprint = Копіювати відбиток пальця

mail-file-copy-name = Копіювати назву файлу

mail-file-created = Створено

mail-file-creator = Створено за допомогою

mail-file-details-button = Деталі файлу

mail-file-details-tooltip = Показати деталі файлу та відбиток пальця

mail-file-encryption = Захист паролем

mail-file-encryption-none = Пароль не запитувався

mail-file-encryption-password = Щоб відкрити його, потрібен був пароль

mail-file-fingerprint = Відбиток пальця

mail-file-name = Ім'я файлу

mail-file-pages = сторінки

mail-file-pdf-version = Версія PDF

mail-file-producer = Продюсер

mail-file-verify-help = Порівняйте ці групи з відбитком пальця, який відправник надав вам іншим шляхом, наприклад, зателефонувавши або зателефонувавши на свій веб-сайт. Якщо кожна група збігається, у вас точно однакові байти.

mail-file-verify-title = Перевірте відбиток пальця

mail-follow-up-finished = Подальше спостереження завершено

mail-group-by = Групувати за

mail-header-attachments = { $count ->
        [one] { $count } вкладення
        [few] { $count } вкладення
        [many] { $count } вкладень
       *[other] { $count } вкладення
    }

mail-header-expand-short = Розгорнути заголовок повідомлення (Alt+H)

mail-list-group-date-unknown = Невідома дата

mail-list-group-importance = Важливість

mail-list-group-locked = Групування повідомлень заблоковано вашим адміністратором.

mail-list-group-no-subject = (без теми)

mail-list-group-none = Без групування

mail-list-group-priority-high = Висока важливість

mail-list-group-priority-low = Низька важливість

mail-list-group-priority-normal = Нормальна важливість

mail-list-group-received = Дата отримання

mail-list-group-sender = Від

mail-list-group-sent = Дата відправлення

mail-list-group-size = Розмір

mail-list-group-size-huge = 10 МБ і вище

mail-list-group-size-large = 1–10 Мб

mail-list-group-size-medium = 100 КБ–1 МБ

mail-list-group-size-small = 25–100 Кб

mail-list-group-size-tiny = Менше 25 КБ

mail-list-group-subject = Тема

mail-list-order-importance = Важливість, перш за все висока

mail-list-order-importance-asc = Важливість, спочатку низька

mail-list-order-locked = Порядок повідомлень заблоковано вашим адміністратором.

mail-list-order-sender-desc = Відправник, від Я до А

mail-list-order-sent-newest = Надіслано, спочатку найновіші

mail-list-order-sent-oldest = Надіслано, спочатку найстаріші

mail-list-order-size = Розмір, спочатку найбільший

mail-list-order-size-asc = Розмір, спочатку найменший

mail-list-order-subject-desc = Тема від Я до А

mail-mark-answered = Позначити як відповідь

mail-mark-as = Позначити як

mail-mark-finished = Позначити як завершене

mail-mark-forwarded = Позначити як переслане

mail-mark-not-forwarded = Позначити як не переслане

mail-mark-unanswered = Позначити як без відповіді

mail-mentions-me = Згадує мене

mail-mentions-me-help = Шукайте повідомлення, у яких згадується одна з ваших електронних адрес

mail-move-search = Пошук папок

mail-order-by = Замовити за

mail-reader-folder-go = Зберігається в { $folder } · Перейти до папки

mail-reply-draft-discard = Скасувати відповідь

mail-reply-draft-discard-detail = Видалити цю відповідь назавжди? Це неможливо скасувати.

mail-reply-draft-discard-title = Відхилити цю відповідь?

mail-status-from-match-domain = Від { $identity }

mail-status-from-match-pattern = Від { $identity }

mail-status-from-via = Відповідь від { $identity }, оскільки вона збігається з { $header }

mail-toast-follow-up-finished = Подальші дії позначено як завершені

mail-toast-marked-answered = { $count ->
        [one] { $count } повідомлення позначено як відповідь
        [few] { $count } повідомлення позначено як відповідь
        [many] { $count } повідомлень позначено як відповіді
       *[other] { $count } повідомлення позначено як відповідь
    }

mail-toast-marked-forwarded = { $count ->
        [one] { $count } повідомлення позначено як переслане
        [few] { $count } повідомлення позначено як переслані
        [many] { $count } повідомлень позначено як переслані
       *[other] { $count } повідомлення позначено як переслані
    }

mail-toast-marked-not-forwarded = { $count ->
        [one] { $count } повідомлення позначено як непереслане
        [few] { $count } повідомлення позначено як непереслані
        [many] { $count } повідомлень позначено як непереслані
       *[other] { $count } повідомлення позначено як непереслані
    }

mail-toast-marked-unanswered = { $count ->
        [one] { $count } повідомлення позначено як без відповіді
        [few] { $count } повідомлення позначено як без відповіді
        [many] { $count } повідомлень позначено як без відповіді
       *[other] { $count } повідомлення позначено як без відповіді
    }

mail-toast-move-failed-reason = Не вдалося перемістити повідомлення ({ $reason }). Нічого не зрушили.

mail-toast-update-refused-reason = { $count ->
        [one] Сервер відмовив у переміщенні через одне повідомлення: { $reason }
        [few] Сервер відмовив у переміщенні через { $count } повідомлення: { $reason }
        [many] Сервер відмовив у переміщенні для { $count } повідомлень: { $reason }
       *[other] Сервер відмовив у переміщенні на { $count } повідомлення: { $reason }
    }

reader-badge-hide = Сховати цей значок

reader-badge-locked = Встановлено вашим адміністратором

reader-badge-menu = Значок повідомлення

reader-badge-notable = Показувати лише тоді, коли помітно

reader-sanitized-always = Активний вміст, наприклад сценарії, залишається вимкненим.

reader-sanitized-clean = Повідомлення HTML не потребує змін

reader-sanitized-clean-detail = З HTML цього повідомлення нічого не видалено чи заблоковано.

reader-sanitized-counts = { $blocked } віддалені елементи заблоковано, { $trackers } елементи відстеження.

reader-sanitized-detail = Повідомлення HTML було очищено перед показом. Віддалений вміст відповідає вашим налаштуванням конфіденційності.

reader-sanitized-hosts = Віддалені хости

reader-sanitized-remote = Віддалений вміст заблоковано

reader-sanitized-short = Продезінфіковано HTML

reader-sanitized-short-blocked = HTML заблоковано

reader-sanitized-short-clean = HTML чистий

reader-sanitized-trackers = Відстеження елементів

mail-draft-view = Переглянути чернетку
mail-reply-draft-view = Переглянути відповідь
