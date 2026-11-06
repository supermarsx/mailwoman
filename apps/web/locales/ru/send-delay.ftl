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
