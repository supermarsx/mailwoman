# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Отмена отправки
send-delay-help = После нажатия «Отправить» сообщение ждёт в исходящих столько времени. До этого его можно отозвать и продолжить редактирование. Сообщение хранится на сервере, поэтому закрытие браузера не приведёт ни к преждевременной отправке, ни к потере.
send-delay-label = Удерживать отправляемые сообщения
send-delay-off = Выключено (отправлять сразу)
send-delay-seconds = { $n } с
send-delay-minutes = { $n ->
    [one] { $n } минуту
    [few] { $n } минуты
    [many] { $n } минут
   *[other] { $n } минуты
}
send-delay-custom = Другое…
send-delay-custom-label = Своя задержка в секундах
send-delay-custom-help = Целое число секунд от 0 до { $max }.
send-delay-capped = Администратор ограничивает это значение до { $max }.
send-delay-disabled = Администратор отключил отзыв отправленных сообщений. Сообщения отправляются сразу.
send-delay-invalid = Введите целое число секунд от 0 до { $max }.
send-later-button = Отправить позже
send-later-menu = Параметры отложенной отправки
send-later-preset-hour = Через 1 час
send-later-preset-today = Сегодня позже, 17:00
send-later-preset-tomorrow = Завтра утром, 09:00
send-later-preset-monday = В понедельник утром, 09:00
send-later-custom = Выбрать дату и время…
send-later-custom-label = Отправить в
send-later-zone = Часовой пояс: { $zone }
send-later-schedule = Запланировать отправку
send-later-cancel = Отмена
send-later-err-invalid = Введите корректные дату и время.
send-later-err-past = Выберите время в будущем.
send-later-err-horizon = Слишком далёкая дата. Сообщения можно планировать не более чем на { $days } дн. вперёд.
send-later-err-off = Администратор отключил отложенную отправку.
send-queued = Сообщение в очереди — будет отправлено через
send-queued-undo = Отменить
send-queued-now = Отправить сейчас
send-queued-recalled = Сообщение отозвано. Оно снова в черновиках.
send-queued-too-late = Слишком поздно для отзыва: сообщение уже отправляется.
send-queued-sent-now = Отправка.
send-queued-scheduled = Запланировано на { $when } ({ $zone }).
outbox-scheduled-badge = Запланировано
outbox-sends-in = Отправка через { $time }
outbox-sends-at = Отправка: { $when }
outbox-recall-edit = Отозвать и изменить
outbox-recall-too-late = Уже отправляется; отозвать больше нельзя.
outbox-failed-drafts = Не удалось отправить; сообщение сохранено в черновиках.
outbox-queued-count = { $count ->
    [one] { $count } сообщение ожидает отправки
    [few] { $count } сообщения ожидают отправки
    [many] { $count } сообщений ожидают отправки
   *[other] { $count } сообщения ожидают отправки
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = Следующая отправка: { $when }

outbox-chip-title = Сообщения, ожидающие отправки

outbox-chip-view-all = Открыть исходящие

outbox-delete = Удалить

outbox-delete-confirm = Удалить сообщение?

outbox-delete-failed = Сообщение не удалось удалить. { $error }

outbox-deleted = Сообщение в очереди было удалено.

outbox-edit-time = Изменить время отправки

outbox-edit-time-conflict = Время отправки изменилось или сообщение уже было отправлено. Исходящие были перезагружены.

outbox-edit-time-current = Сейчас { $when }

outbox-edit-time-refused = Время отправки изменить невозможно. { $error }

outbox-edit-time-save = Экономьте время отправки

outbox-failed-toast = Не удалось отправить сообщение. { $error }

outbox-failed-uncertain-toast = Сообщение может быть доставлено, а может и не быть доставлено. Проверьте «Отправлено», прежде чем отправлять его снова.

outbox-open = Открыть

outbox-reader-actions = Действия с сообщениями в очереди

outbox-reader-cc = Копия: { $recipients }

outbox-reader-label = Сообщение в очереди

outbox-reader-loading = Загрузка сообщения…

outbox-reader-to = Кому: { $recipients }

outbox-reader-unavailable = Сообщение не удалось загрузить.

outbox-recall-to-drafts = Вернуть в черновики

outbox-send-again = Отправить еще раз

outbox-send-again-failed = Не удалось снова поставить сообщение в очередь. { $error }

outbox-send-again-queued = Сообщение ставится в очередь для повторной отправки.

outbox-sent-copy-folder = Будет сохранено через { $folder }

outbox-sent-copy-missing = Будет сохранено в папке Отправленные (выбранная папка недоступна)

outbox-sent-copy-skip = Ни одна копия не будет сохранена.

outbox-status-attempts = { $count ->
        [one] пока { $count } попытка
        [few] пока { $count } попытки
        [many] { $count } попыток пока
       *[other] пока { $count } попытки
    }

outbox-status-canceled = Отменено

outbox-status-failed = Не отправлено

outbox-status-held = Удерживается, пока вы его не отпустите

outbox-status-last-error = Последняя ошибка: { $error }

outbox-status-next-attempt = Следующая попытка { $when }

outbox-status-queued = В очереди

outbox-status-retrying = Повторная попытка

outbox-status-scheduled = Запланировано

outbox-status-sending = Отправка

outbox-status-sends-at-in = Отправляет { $when } (в { $time })

outbox-status-sent = Отправлено

outbox-status-uncertain = Доставка неизвестна
