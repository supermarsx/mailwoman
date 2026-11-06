# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Календарь
calendar-prev = Назад
calendar-next = Вперёд
calendar-today = Сегодня
calendar-views = Представления календаря
calendar-new-event = Новое событие
calendar-import = Импорт
calendar-export = Экспорт
calendar-import-file = Импортировать файл календаря

# -- View names --------------------------------------------------------------
calendar-view-day = День
calendar-view-3day = 3 дня
calendar-view-work-week = Рабочая неделя
calendar-view-week = Неделя
calendar-view-month = Месяц
calendar-view-tri-month = Квартал
calendar-view-schedule = Расписание
calendar-view-agenda = Повестка
calendar-view-year = Год

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Календари
calendar-toggle = Показать или скрыть { $name }
calendar-color-for = Цвет для { $name }
calendar-add-calendar = Добавить календарь
calendar-new-calendar-name = Новый календарь
calendar-holidays-heading = Праздники
calendar-subscribe-holidays = Подписаться на праздники
calendar-add-region = Добавить регион…
calendar-synced = Синхронизированный календарь

# -- Grid / views ------------------------------------------------------------
calendar-all-day = весь день
calendar-all-day-full = Весь день
calendar-more = Ещё { $count }
calendar-conflict = конфликт
calendar-no-events-30 = Нет событий в ближайшие 30 дней.
calendar-events-count = { $count ->
    [one] { $count } событие
    [few] { $count } события
    [many] { $count } событий
   *[other] { $count } события
}
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Месяц: { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Сетка времени
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }, { $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }, { $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = Весь день: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Новое событие
calendar-editor-edit = Изменить событие
calendar-invitation = Приглашение
calendar-invited = Вы приглашены ({ $status }).
calendar-accept = Принять
calendar-tentative = Под вопросом
calendar-decline = Отклонить
calendar-counter = Предложить другое время…
calendar-propose-start = Предложить новое время начала
calendar-send-counter = Отправить предложение
calendar-overlaps = Это событие пересекается с { $count ->
    [one] { $count } другим событием
    [few] { $count } другими событиями
    [many] { $count } другими событиями
   *[other] { $count } другими событиями
}.
calendar-field-title = Название
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (без названия)
calendar-field-calendar = Календарь
calendar-field-all-day = Весь день
calendar-field-start = Начало
calendar-field-duration = Длительность (мин)
calendar-field-location = Место
calendar-field-shows-as = Показывать как
calendar-shows-busy = Занят
calendar-shows-free = Свободен
calendar-field-status = Статус
calendar-status-confirmed = Подтверждено
calendar-status-tentative = Под вопросом
calendar-status-cancelled = Отменено
calendar-repeats = Повторение
calendar-frequency = Частота
calendar-freq-daily = Ежедневно
calendar-freq-weekly = Еженедельно
calendar-freq-monthly = Ежемесячно
calendar-freq-yearly = Ежегодно
calendar-every = каждые
calendar-interval = Интервал
calendar-ends = Окончание
calendar-end-mode = Способ окончания
calendar-end-never = Никогда
calendar-end-count = После N раз
calendar-end-until = В дату
calendar-occurrences = Повторений
calendar-until-date = До даты
calendar-reminders = Напоминания
calendar-reminder-before = за { $min } мин
calendar-remove-reminder = Удалить напоминание { $min }
calendar-add-reminder = Добавить напоминание
calendar-reminder-none = + напоминание
calendar-reminder-5 = 5 мин
calendar-reminder-15 = 15 мин
calendar-reminder-30 = 30 мин
calendar-reminder-60 = 1 час
calendar-reminder-1440 = 1 день
calendar-attendees = Участники
calendar-add-attendee = Добавить участника
calendar-attendee-invalid = Это не адрес электронной почты. Введите один адрес, например name@example.com.
calendar-save-failed = Событие не сохранено. Сервер его отклонил.
calendar-remove-attendee = Удалить { $email }
calendar-notes = Заметки

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Роль для { $email }
calendar-attendee-cutype-for = Тип для { $email }
calendar-role-chair = Председатель
calendar-role-required = Обязательный
calendar-role-optional = Необязательный
calendar-role-non-participant = Не участник
calendar-cutype-individual = Человек
calendar-cutype-group = Группа
calendar-cutype-resource = Ресурс
calendar-cutype-room = Помещение
calendar-partstat-needs-action = Нет ответа
calendar-partstat-accepted = Принято
calendar-partstat-declined = Отклонено
calendar-partstat-tentative = Под вопросом

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = С { $start } до { $end }: { $title }
calendar-schedule-free = Свободно: { $gap }
calendar-gap-hm = { $h } ч { $m } мин
calendar-gap-h = { $h } ч
calendar-gap-m = { $m } мин

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Разрешить { $count ->
    [one] { $count } конфликт
    [few] { $count } конфликта
    [many] { $count } конфликтов
   *[other] { $count } конфликта
}
calendar-resolver-title = Разрешение конфликтов
calendar-resolver-none = Нет конфликтов для разрешения.
calendar-resolver-pick = Конфликт
calendar-resolver-pair-n = { $n } из { $total }: { $a } и { $b }
calendar-resolver-time = Время
calendar-resolver-overlap = Пересечение { $start } – { $end }
calendar-resolver-reschedule = Перенести более позднее событие
calendar-resolver-shorten = Сократить более раннее событие
calendar-resolver-tentative = Отметить более позднее как «под вопросом»
calendar-resolver-double-book = Двойное бронирование (отметить как свободное)
calendar-resolver-keep = Оставить оба
calendar-events-count-people = { $count ->
    [one] { $count } участник
    [few] { $count } участника
    [many] { $count } участников
   *[other] { $count } участника
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Календари
calendar-fb-own = Ваш
calendar-fb-unavailable = Не удалось загрузить ваше занятое время.
calendar-fb-cell = { $principal } в { $hour }: { $status }
calendar-fb-busy = Занят
calendar-fb-tentative = Под вопросом
calendar-fb-free = Свободен

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Быстрое добавление события
calendar-quick-add-placeholder = Быстрое добавление, например: Обед в пятницу в 13:00
calendar-quick-add-btn = Добавить
calendar-quick-add-do = Быстро добавить

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Категории
calendar-add-category = Добавить категорию
calendar-category-placeholder = Добавьте категорию
calendar-remove-category = Удалить категорию { $name }
calendar-filter-heading = Фильтр
calendar-filter-category = Фильтр по категории
calendar-filter-category-placeholder = Категория для показа

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Вложения
calendar-attachment-title = Название вложения
calendar-attachment-title-placeholder = Название (необязательно)
calendar-attachment-uri = Ссылка на вложение
calendar-attachment-add = Добавить вложение
calendar-remove-attachment = Удалить вложение { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Подписка по URL
calendar-subscribe-url = URL календаря
calendar-subscribe-add = Подписаться
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Не удалось получить календарь по этому адресу, поэтому он не был добавлен.
calendar-subscribe-hol-many-locations = В этом файле праздников указано несколько местоположений. Скачайте его, затем воспользуйтесь импортом и выберите нужные местоположения.
calendar-subscribe-refresh = Обновить { $name }
calendar-subscribe-refresh-btn = Обновить
calendar-subscribe-refresh-failed = Не удалось получить ленту календаря. Её события остались без изменений.
calendar-import-failed = Этот файл не удалось прочитать как календарь. Ничего не импортировано.
calendar-import-done = { $count ->
    [one] Импортировано { $count } событие.
    [few] Импортировано { $count } события.
    [many] Импортировано { $count } событий.
   *[other] Импортировано { $count } события.
}
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Выберите местоположения для импорта
calendar-import-locations-intro = В этом файле есть праздники для нескольких местоположений ({ $count }). Импортируются только выбранные.
calendar-import-locations-filter = Фильтр местоположений
calendar-import-locations-list = Местоположения в файле
calendar-import-locations-none = Ни одно местоположение не соответствует фильтру.
calendar-import-locations-picked = { $count ->
    [one] Выбрано { $count } местоположение.
    [few] Выбрано { $count } местоположения.
    [many] Выбрано { $count } местоположений.
   *[other] Выбрано { $count } местоположения.
}
calendar-import-locations-import = Импортировать выбранные
calendar-export-failed = Не удалось экспортировать календарь.
calendar-quick-add-failed = Не удалось добавить это событие.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Поделиться
calendar-share-for = Поделиться { $name }
calendar-share-title = Общий доступ: { $name }
calendar-share-intro = Люди, с которыми вы поделились, видят этот календарь. Предоставьте доступ на чтение и запись, чтобы они могли добавлять и изменять события.
calendar-share-people = Доступ предоставлен
calendar-share-empty = Этот календарь никому не доступен.
calendar-share-add = Добавить человека
calendar-share-new-access = Доступ для нового человека
calendar-share-access-for = Доступ для { $principal }
calendar-share-remove = Удалить { $principal }
calendar-share-read = Только чтение
calendar-share-readwrite = Чтение и запись

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Навигатор по датам
calendar-event-range = { $title }, с { $start } по { $end }
calendar-busy-free = Отображается как свободное время
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = Ещё { $count } { $count ->
    [one] событие
    [few] события
    [many] событий
   *[other] события
  } — { $date }
calendar-fb-caption-people = Занятое время в день конфликта по часам для вас и каждого приглашённого.
calendar-fb-unknown = Не удалось получить сведения о доступности для: { $principal }.
calendar-fb-loading = Загрузка сведений о доступности…
calendar-import-itip-message = Этот файл — ответ или отмена, отправленные по почте. Откройте письмо, с которым он пришёл, чтобы применить его. Ничего не импортировано.
calendar-import-skipped = { $count ->
    [one] { $count } событие уже было в календаре и осталось без изменений.
    [few] { $count } события уже были в календаре и остались без изменений.
    [many] { $count } событий уже были в календаре и остались без изменений.
   *[other] { $count } события уже были в календаре и остались без изменений.
}
calendar-invite-title = Приглашение в календарь
calendar-invite-sender = От: { $sender }
calendar-invite-add = Добавить в календарь
calendar-invite-update = Применить обновление события
calendar-invite-cancel = Применить отмену
calendar-invite-reply = Записать ответ участника
calendar-invite-new = Это приглашение можно добавить в ваш календарь.
calendar-invite-newer = Организатор прислал обновлённое событие.
calendar-invite-current = В вашем календаре уже есть эта версия.
calendar-invite-stale = Это сообщение старше события в вашем календаре.
calendar-invite-cancelled = Организатор отменил это событие.
calendar-invite-response = Участник прислал ответ.
calendar-invite-not-organizer = Отправитель не является сохранённым организатором. Ничего не будет изменено.
calendar-invite-not-attendee = Отправитель не является подходящим участником. Ничего не будет изменено.
calendar-invite-unknown = Это сообщение относится к событию, которого нет в вашем календаре.
calendar-invite-unsupported = Это предложение нельзя применить автоматически.
calendar-invite-no-reply = Это обновляет ваш календарь без отправки ответа.
calendar-invite-applied = Ваш календарь обновлён.
calendar-invite-unchanged = Событие не изменено. Проверьте текущее состояние приглашения.
calendar-invite-apply-failed = Не удалось применить приглашение. Повторите попытку.
calendar-invite-load-failed = Не удалось проверить приглашение.
calendar-invite-retry = Повторить
calendar-field-time-zone = Часовой пояс события
calendar-field-time-zone-help = Используйте название IANA, например Europe/Lisbon. Событие сохраняет этот пояс при просмотре в другом месте.
calendar-time-invalid = Укажите допустимый часовой пояс и местное время. Некоторого местного времени не существует при переводе часов вперёд.
calendar-mini-prev = Предыдущий месяц
calendar-mini-next = Следующий месяц
calendar-card-region-event = Событие календаря
calendar-card-kind-request = Приглашение на встречу
calendar-card-kind-update = Обновлённое приглашение
calendar-card-kind-cancel = Отменённая встреча
calendar-card-kind-reply = Ответ на встречу
calendar-card-kind-counter = Предложено новое время
calendar-card-kind-publish = { $count ->
    [one] Событие календаря
    [few] { $count } события календаря
    [many] { $count } событий календаря
   *[other] { $count } события календаря
}
calendar-card-when = Когда
calendar-card-where = Где
calendar-card-organizer = Организатор
calendar-card-attendees = Участники
calendar-card-all-day = { $when } (весь день)
calendar-card-organizer-time = Время организатора: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Каждый день
    [few] Каждые { $interval } дня
    [many] Каждые { $interval } дней
   *[other] Каждые { $interval } дня
}
calendar-card-repeat-weekly = { $interval ->
    [one] Каждую неделю
    [few] Каждые { $interval } недели
    [many] Каждые { $interval } недель
   *[other] Каждые { $interval } недели
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] Каждую неделю: { $days }
    [few] Каждые { $interval } недели: { $days }
    [many] Каждые { $interval } недель: { $days }
   *[other] Каждые { $interval } недели: { $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] Каждый месяц
    [few] Каждые { $interval } месяца
    [many] Каждые { $interval } месяцев
   *[other] Каждые { $interval } месяца
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] Каждый месяц, { $day }-го числа
    [few] Каждые { $interval } месяца, { $day }-го числа
    [many] Каждые { $interval } месяцев, { $day }-го числа
   *[other] Каждые { $interval } месяца, { $day }-го числа
}
calendar-card-repeat-yearly = { $interval ->
    [one] Каждый год
    [few] Каждые { $interval } года
    [many] Каждые { $interval } лет
   *[other] Каждые { $interval } года
}
calendar-card-repeat-until = { $rule } до { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] один раз
    [few] { $count } раза
    [many] { $count } раз
   *[other] { $count } раза
}
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] пропущена 1 дата
    [few] пропущено { $count } даты
    [many] пропущено { $count } дат
   *[other] пропущено { $count } даты
})
calendar-card-repeat-custom = Повторяется по особому расписанию
calendar-card-join = Присоединиться: { $provider }
calendar-card-organizer-is = Организатор: { $who }
calendar-card-attendee-summary = { $count ->
    [one] { $count } участник
    [few] { $count } участника
    [many] { $count } участников
   *[other] { $count } участника
} · приняли: { $accepted }, под вопросом: { $tentative }, отклонили: { $declined }
calendar-card-people-all = Показать всех ({ $count })
calendar-card-people-fewer = Показать меньше
calendar-card-rsvp-accepted = Принято
calendar-card-rsvp-tentative = Под вопросом
calendar-card-rsvp-declined = Отклонено
calendar-card-rsvp-none = Ответа пока нет
calendar-card-in-calendar = В вашем календаре
calendar-card-cancelled = Отменено
calendar-card-open = Открыть в календаре
calendar-card-agenda = Ваш календарь на { $day }
calendar-card-agenda-title = Ваш день
calendar-card-agenda-this-event = Это событие
calendar-card-agenda-unavailable = Не удалось загрузить ваш календарь на этот день.
calendar-card-conflicts = { $count ->
    [one] Пересекается с: { $titles }
    [few] Пересекается с { $count } событиями: { $titles }
    [many] Пересекается с { $count } событиями: { $titles }
   *[other] Пересекается с { $count } событиями: { $titles }
}
calendar-card-no-conflicts = Пересечений в вашем календаре нет
calendar-card-invited-as = Вас пригласили как { $address }.
calendar-card-not-invited = В этом приглашении не указан ни один из ваших адресов. Его можно добавить в календарь, но ответить на него нельзя.
calendar-card-not-invited-short = В этом приглашении не указан ни один из ваших адресов.
calendar-card-out-of-date = Это приглашение устарело. Более новая версия уже есть в вашем календаре.
calendar-card-counter = Новое время предложено: { $who }. Предложения не применяются автоматически.
calendar-card-todos-skipped = { $count ->
    [one] В файле также есть 1 задача, она не добавляется.
    [few] В файле также есть { $count } задачи, они не добавляются.
    [many] В файле также есть { $count } задач, они не добавляются.
   *[other] В файле также есть { $count } задачи, они не добавляются.
}
calendar-card-respond = Ответить на приглашение
calendar-card-accept = Принять
calendar-card-tentative = Под вопросом
calendar-card-decline = Отклонить
calendar-card-propose = Предложить другое время
calendar-card-propose-start = Новое начало
calendar-card-propose-send = Отправить предложение
calendar-card-propose-invalid = Укажите допустимое новое время начала.
calendar-card-send-response = Отправить ответ: { $organizer }
calendar-card-comment = Комментарий (необязательно)
calendar-card-calendar = Календарь
calendar-card-events = События в этом файле
calendar-card-add-one = Добавить
calendar-card-add-all = Добавить все ({ $count })
calendar-card-added = { $count ->
    [one] Добавлено в ваш календарь.
    [few] В ваш календарь добавлено { $count } события.
    [many] В ваш календарь добавлено { $count } событий.
   *[other] В ваш календарь добавлено { $count } события.
}
calendar-card-remove = Удалить из календаря
calendar-card-removed = Удалено из вашего календаря.
calendar-card-record-reply = Обновить статус участника
calendar-card-answered-sent = { $answer }. Ваш ответ отправлен: { $organizer }.
calendar-card-answered-not-sent = { $answer }. Ваш календарь обновлён, но отправить ответ не удалось.
calendar-card-answered-silently = { $answer }. Ответ не отправлялся.
calendar-card-proposal-sent = Ваше предложение отправлено: { $organizer }.
calendar-card-not-sent = Не удалось отправить сообщение.
calendar-card-more = { $count ->
    [one] ещё 1
    [few] ещё { $count }
    [many] ещё { $count }
   *[other] ещё { $count }
}

calendar-card-replied = Ответ от { $who }: { $answer ->
    [accepted] принято.
    [tentative] принято под вопросом.
    [declined] отклонено.
   *[other] ответа пока нет.
}
