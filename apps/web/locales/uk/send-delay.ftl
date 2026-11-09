# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Скасування надсилання
send-delay-help = Після натискання «Надіслати» повідомлення чекає у вихідних стільки часу. До цього його можна відкликати й далі редагувати. Повідомлення зберігається на сервері, тому закриття браузера не призведе ні до передчасного надсилання, ні до втрати.
send-delay-label = Утримувати надіслані повідомлення
send-delay-off = Вимкнено (надсилати одразу)
send-delay-seconds = { $n } с
send-delay-minutes = { $n ->
    [one] { $n } хвилину
    [few] { $n } хвилини
    [many] { $n } хвилин
   *[other] { $n } хвилини
}
send-delay-custom = Інше…
send-delay-custom-label = Власна затримка в секундах
send-delay-custom-help = Ціле число секунд від 0 до { $max }.
send-delay-capped = Адміністратор обмежує це значення до { $max }.
send-delay-disabled = Адміністратор вимкнув відкликання надісланих повідомлень. Повідомлення надсилаються одразу.
send-delay-invalid = Введіть ціле число секунд від 0 до { $max }.
send-later-button = Надіслати пізніше
send-later-menu = Параметри відкладеного надсилання
send-later-preset-hour = Через 1 годину
send-later-preset-today = Сьогодні пізніше, 17:00
send-later-preset-tomorrow = Завтра вранці, 09:00
send-later-preset-monday = У понеділок вранці, 09:00
send-later-custom = Вибрати дату й час…
send-later-custom-label = Надіслати о
send-later-zone = Часовий пояс: { $zone }
send-later-schedule = Запланувати надсилання
send-later-cancel = Скасувати
send-later-err-invalid = Введіть дійсні дату й час.
send-later-err-past = Виберіть час у майбутньому.
send-later-err-horizon = Занадто далека дата. Повідомлення можна планувати щонайбільше на { $days } дн. наперед.
send-later-err-off = Адміністратор вимкнув відкладене надсилання.
send-queued = Повідомлення в черзі — буде надіслано через
send-queued-undo = Скасувати
send-queued-now = Надіслати зараз
send-queued-recalled = Повідомлення відкликано. Воно знову в чернетках.
send-queued-too-late = Занадто пізно для відкликання: повідомлення вже надсилається.
send-queued-sent-now = Надсилання.
send-queued-scheduled = Заплановано на { $when } ({ $zone }).
outbox-scheduled-badge = Заплановано
outbox-sends-in = Надсилання через { $time }
outbox-sends-at = Надсилання: { $when }
outbox-recall-edit = Відкликати й змінити
outbox-recall-too-late = Уже надсилається; відкликати більше не можна.
outbox-failed-drafts = Не вдалося надіслати; повідомлення збережено в чернетках.
outbox-queued-count = { $count ->
    [one] { $count } повідомлення очікує надсилання
    [few] { $count } повідомлення очікують надсилання
    [many] { $count } повідомлень очікують надсилання
   *[other] { $count } повідомлення очікують надсилання
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = Наступне надсилання: { $when }

outbox-chip-title = Повідомлення очікують на надсилання

outbox-chip-view-all = Відкрийте вихідні

outbox-delete = Видалити

outbox-delete-confirm = Видалити повідомлення?

outbox-delete-failed = Не вдалося видалити повідомлення. { $error }

outbox-deleted = Повідомлення в черзі видалено.

outbox-edit-time = Редагувати час надсилання

outbox-edit-time-conflict = Час надсилання змінено або повідомлення вже надіслано. Вихідні перезавантажено.

outbox-edit-time-current = Зараз { $when }

outbox-edit-time-refused = Неможливо змінити час надсилання. { $error }

outbox-edit-time-save = Економія часу надсилання

outbox-failed-toast = Не вдалося надіслати повідомлення. { $error }

outbox-failed-uncertain-toast = Повідомлення може бути доставлено або не доставлено. Поставте прапорець «Надіслано» перед повторним надсиланням.

outbox-open = відкритий

outbox-reader-actions = Дії з повідомленнями в черзі

outbox-reader-cc = Копія: { $recipients }

outbox-reader-label = Повідомлення в черзі

outbox-reader-loading = Завантаження повідомлення…

outbox-reader-to = Кому: { $recipients }

outbox-reader-unavailable = Не вдалося завантажити повідомлення.

outbox-recall-to-drafts = Відкликання на чернетки

outbox-send-again = Надіслати знову

outbox-send-again-failed = Не вдалося знову поставити повідомлення в чергу. { $error }

outbox-send-again-queued = Повідомлення поставлено в чергу для повторного надсилання.

outbox-sent-copy-folder = Буде збережено через { $folder }

outbox-sent-copy-missing = Буде збережено в Надіслані (вибрана папка недоступна)

outbox-sent-copy-skip = Жодна копія не буде збережена

outbox-status-attempts = { $count ->
        [one] Наразі { $count } спроба
        [few] Поки що { $count } спроби
        [many] Наразі { $count } спроб
       *[other] Наразі { $count } спроби
    }

outbox-status-canceled = Скасовано

outbox-status-failed = Не відправлено

outbox-status-held = Утримується, доки ви не відпустите

outbox-status-last-error = Остання помилка: { $error }

outbox-status-next-attempt = Наступна спроба { $when }

outbox-status-queued = У черзі

outbox-status-retrying = Повторна спроба

outbox-status-scheduled = За розкладом

outbox-status-sending = Відправка

outbox-status-sends-at-in = Надсилає { $when } (в { $time })

outbox-status-sent = Надіслано

outbox-status-uncertain = Доставка невідома
