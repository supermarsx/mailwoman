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
calendar-fb-caption = Ваше занятое время в день конфликта, по часам.
calendar-fb-whose = Календари
calendar-fb-own = Ваш
calendar-fb-own-only = Доступность участников не показывается. Сервер сообщает занятое время только для ваших собственных календарей.
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
