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
mail-toast-pinned = Закріплено
mail-toast-unpinned = Відкріплено
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
