# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Календар
calendar-prev = Назад
calendar-next = Далі
calendar-today = Сьогодні
calendar-views = Подання календаря
calendar-new-event = Нова подія
calendar-import = Імпортувати
calendar-export = Експортувати
calendar-import-file = Імпортувати файл календаря

# -- View names --------------------------------------------------------------
calendar-view-day = День
calendar-view-3day = 3 дні
calendar-view-work-week = Робочий тиждень
calendar-view-week = Тиждень
calendar-view-month = Місяць
calendar-view-tri-month = Квартал
calendar-view-schedule = Розклад
calendar-view-agenda = Порядок денний
calendar-view-year = Рік

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Календарі
calendar-toggle = Показати або приховати { $name }
calendar-color-for = Колір для { $name }
calendar-add-calendar = Додати календар
calendar-new-calendar-name = Новий календар
calendar-holidays-heading = Свята
calendar-subscribe-holidays = Підписатися на свята
calendar-add-region = Додати регіон…
calendar-synced = Синхронізований календар

# -- Grid / views ------------------------------------------------------------
calendar-all-day = весь день
calendar-all-day-full = Весь день
calendar-more = Ще { $count }
calendar-conflict = конфлікт
calendar-no-events-30 = Немає подій на наступні 30 днів.
calendar-events-count = { $count ->
    [0] Немає подій
    [one] { $count } подія
    [few] { $count } події
    [many] { $count } подій
   *[other] { $count } події
}
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Місяць { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Часова сітка
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }, { $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }, { $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = Весь день: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Нова подія
calendar-editor-edit = Редагувати подію
calendar-invitation = Запрошення
calendar-invited = Вас запрошено ({ $status }).
calendar-accept = Прийняти
calendar-tentative = Можливо
calendar-decline = Відхилити
calendar-counter = Запропонувати інше…
calendar-propose-start = Запропонувати новий початок
calendar-send-counter = Надіслати пропозицію
calendar-overlaps = Ця подія перетинається з { $count ->
    [one] { $count } іншою подією
    [few] { $count } іншими подіями
    [many] { $count } іншими подіями
   *[other] { $count } іншої події
}.
calendar-field-title = Назва
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (без назви)
calendar-field-calendar = Календар
calendar-field-all-day = Весь день
calendar-field-start = Початок
calendar-field-duration = Тривалість (хв)
calendar-field-location = Місце
calendar-field-shows-as = Показувати як
calendar-shows-busy = Зайнято
calendar-shows-free = Вільно
calendar-field-status = Стан
calendar-status-confirmed = Підтверджено
calendar-status-tentative = Попередньо
calendar-status-cancelled = Скасовано
calendar-repeats = Повторення
calendar-frequency = Частота
calendar-freq-daily = Щодня
calendar-freq-weekly = Щотижня
calendar-freq-monthly = Щомісяця
calendar-freq-yearly = Щороку
calendar-every = кожні
calendar-interval = Інтервал
calendar-ends = Закінчення
calendar-end-mode = Спосіб закінчення
calendar-end-never = Ніколи
calendar-end-count = Після N разів
calendar-end-until = У дату
calendar-occurrences = Кількість повторень
calendar-until-date = До дати
calendar-reminders = Нагадування
calendar-reminder-before = за { $min } хв
calendar-remove-reminder = Вилучити нагадування { $min }
calendar-add-reminder = Додати нагадування
calendar-reminder-none = + нагадування
calendar-reminder-5 = 5 хв
calendar-reminder-15 = 15 хв
calendar-reminder-30 = 30 хв
calendar-reminder-60 = 1 година
calendar-reminder-1440 = 1 день
calendar-attendees = Учасники
calendar-add-attendee = Додати учасника
calendar-attendee-invalid = Це не адреса електронної пошти. Введіть одну адресу, наприклад name@example.com.
calendar-save-failed = Подію не збережено. Сервер її відхилив.
calendar-remove-attendee = Вилучити { $email }
calendar-notes = Нотатки

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Роль для { $email }
calendar-attendee-cutype-for = Тип для { $email }
calendar-role-chair = Головує
calendar-role-required = Обов’язковий
calendar-role-optional = Необов’язковий
calendar-role-non-participant = Не учасник
calendar-cutype-individual = Особа
calendar-cutype-group = Група
calendar-cutype-resource = Ресурс
calendar-cutype-room = Приміщення
calendar-partstat-needs-action = Немає відповіді
calendar-partstat-accepted = Прийнято
calendar-partstat-declined = Відхилено
calendar-partstat-tentative = Можливо

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = З { $start } до { $end }: { $title }
calendar-schedule-free = Вільно { $gap }
calendar-gap-hm = { $h } год { $m } хв
calendar-gap-h = { $h } год
calendar-gap-m = { $m } хв

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Розв’язати { $count ->
    [one] { $count } конфлікт
    [few] { $count } конфлікти
    [many] { $count } конфліктів
   *[other] { $count } конфлікту
}
calendar-resolver-title = Розв’язання конфліктів
calendar-resolver-none = Немає конфліктів для розв’язання.
calendar-resolver-pick = Конфлікт
calendar-resolver-pair-n = { $n } з { $total }: { $a } і { $b }
calendar-resolver-time = Час
calendar-resolver-overlap = Перетин { $start } – { $end }
calendar-resolver-reschedule = Перенести пізнішу подію
calendar-resolver-shorten = Скоротити ранішу подію
calendar-resolver-tentative = Позначити пізнішу як попередню
calendar-resolver-double-book = Подвійне бронювання (позначити як вільно)
calendar-resolver-keep = Залишити обидві
calendar-events-count-people = { $count ->
    [0] Немає учасників
    [one] { $count } учасник
    [few] { $count } учасники
    [many] { $count } учасників
   *[other] { $count } учасника
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Календарі
calendar-fb-own = Ваші
calendar-fb-unavailable = Не вдалося завантажити ваш зайнятий час.
calendar-fb-cell = { $principal } о { $hour }: { $status }
calendar-fb-busy = Зайнято
calendar-fb-tentative = Можливо
calendar-fb-free = Вільно

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Швидко додати подію
calendar-quick-add-placeholder = Швидке додавання, напр. Обід у п’ятницю о 13:00
calendar-quick-add-btn = Додати
calendar-quick-add-do = Швидко додати

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Категорії
calendar-add-category = Додати категорію
calendar-category-placeholder = Додайте категорію
calendar-remove-category = Вилучити категорію { $name }
calendar-filter-heading = Фільтр
calendar-filter-category = Фільтр за категорією
calendar-filter-category-placeholder = Категорія для показу

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Вкладення
calendar-attachment-title = Назва вкладення
calendar-attachment-title-placeholder = Назва (необов’язково)
calendar-attachment-uri = Посилання на вкладення
calendar-attachment-add = Додати вкладення
calendar-remove-attachment = Вилучити вкладення { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Підписатися за URL
calendar-subscribe-url = URL календаря
calendar-subscribe-add = Підписатися
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Не вдалося отримати календар за цією адресою, тому його не додано.
calendar-subscribe-hol-many-locations = Цей файл свят містить більше ніж одне місце. Завантажте його, потім скористайтеся імпортом і виберіть потрібні місця.
calendar-subscribe-refresh = Оновити { $name }
calendar-subscribe-refresh-btn = Оновити
calendar-subscribe-refresh-failed = Не вдалося отримати стрічку календаря. Її події залишено без змін.
calendar-import-failed = Не вдалося прочитати цей файл як календар. Нічого не імпортовано.
calendar-import-done = { $count ->
    [one] Імпортовано { $count } подію.
    [few] Імпортовано { $count } події.
    [many] Імпортовано { $count } подій.
   *[other] Імпортовано { $count } події.
}
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Виберіть місця для імпорту
calendar-import-locations-intro = Цей файл містить свята для кількох місць (усього: { $count }). Імпортуються лише вибрані місця.
calendar-import-locations-filter = Фільтр місць
calendar-import-locations-list = Місця у файлі
calendar-import-locations-none = Жодне місце не відповідає фільтру.
calendar-import-locations-picked = { $count ->
    [one] Вибрано { $count } місце.
    [few] Вибрано { $count } місця.
    [many] Вибрано { $count } місць.
   *[other] Вибрано { $count } місця.
}
calendar-import-locations-import = Імпортувати вибрані
calendar-export-failed = Не вдалося експортувати календар.
calendar-quick-add-failed = Не вдалося додати цю подію.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Поділитися
calendar-share-for = Надати спільний доступ до { $name }
calendar-share-title = Спільний доступ до { $name }
calendar-share-intro = Люди, з якими ви поділилися, можуть бачити цей календар. Надайте доступ для читання і запису, щоб вони могли додавати та редагувати події.
calendar-share-people = Спільний доступ
calendar-share-empty = Цим календарем ні з ким не поділилися.
calendar-share-add = Додати особу
calendar-share-new-access = Доступ для нової особи
calendar-share-access-for = Доступ для { $principal }
calendar-share-remove = Вилучити { $principal }
calendar-share-read = Лише читання
calendar-share-readwrite = Читання і запис

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Навігатор датами
calendar-event-range = { $title }, з { $start } до { $end }
calendar-busy-free = Відображається як вільний час
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = Ще { $count } { $count ->
    [one] подія
    [few] події
    [many] подій
   *[other] події
  } — { $date }
calendar-fb-caption-people = Зайнятий час у день конфлікту за годинами для вас і кожного запрошеного.
calendar-fb-unknown = Не вдалося отримати відомості про доступність для: { $principal }.
calendar-fb-loading = Завантаження відомостей про доступність…
calendar-import-itip-message = Цей файл — відповідь або скасування, надіслані поштою. Відкрийте лист, з яким він надійшов, щоб застосувати його. Нічого не імпортовано.
calendar-import-skipped = { $count ->
    [one] { $count } подія вже була в календарі й залишилася без змін.
    [few] { $count } події вже були в календарі й залишилися без змін.
    [many] { $count } подій уже були в календарі й залишилися без змін.
   *[other] { $count } події вже були в календарі й залишилися без змін.
}
calendar-invite-title = Запрошення до календаря
calendar-invite-sender = Від: { $sender }
calendar-invite-add = Додати до календаря
calendar-invite-update = Застосувати оновлення події
calendar-invite-cancel = Застосувати скасування
calendar-invite-reply = Записати відповідь учасника
calendar-invite-new = Це запрошення можна додати до вашого календаря.
calendar-invite-newer = Організатор надіслав оновлену подію.
calendar-invite-current = У вашому календарі вже є ця версія.
calendar-invite-stale = Це повідомлення старіше за подію у вашому календарі.
calendar-invite-cancelled = Організатор скасував цю подію.
calendar-invite-response = Учасник надіслав відповідь.
calendar-invite-not-organizer = Відправник не є збереженим організатором. Нічого не буде змінено.
calendar-invite-not-attendee = Відправник не є відповідним учасником. Нічого не буде змінено.
calendar-invite-unknown = Це повідомлення стосується події, якої немає у вашому календарі.
calendar-invite-unsupported = Цю пропозицію не можна застосувати автоматично.
calendar-invite-no-reply = Це оновлює ваш календар без надсилання відповіді.
calendar-invite-applied = Ваш календар оновлено.
calendar-invite-unchanged = Подію не змінено. Перевірте поточний стан запрошення.
calendar-invite-apply-failed = Не вдалося застосувати запрошення. Спробуйте ще раз.
calendar-invite-load-failed = Не вдалося перевірити запрошення.
calendar-invite-retry = Повторити
calendar-field-time-zone = Часовий пояс події
calendar-field-time-zone-help = Використовуйте назву IANA, наприклад Europe/Lisbon. Подія зберігає цей пояс під час перегляду деінде.
calendar-time-invalid = Вкажіть чинний часовий пояс і місцевий час. Деякого місцевого часу не існує під час переведення годинників уперед.
calendar-mini-prev = Попередній місяць
calendar-mini-next = Наступний місяць
calendar-card-region-event = Подія календаря
calendar-card-kind-request = Запрошення на зустріч
calendar-card-kind-update = Оновлене запрошення
calendar-card-kind-cancel = Скасована зустріч
calendar-card-kind-reply = Відповідь на зустріч
calendar-card-kind-counter = Запропоновано новий час
calendar-card-kind-publish = { $count ->
    [one] Подія календаря
    [few] { $count } події календаря
    [many] { $count } подій календаря
   *[other] { $count } події календаря
}
calendar-card-when = Коли
calendar-card-where = Де
calendar-card-organizer = Організатор
calendar-card-attendees = Учасники
calendar-card-all-day = { $when } (увесь день)
calendar-card-organizer-time = Час організатора: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Щодня
    [few] Кожні { $interval } дні
    [many] Кожні { $interval } днів
   *[other] Кожні { $interval } дня
}
calendar-card-repeat-weekly = { $interval ->
    [one] Щотижня
    [few] Кожні { $interval } тижні
    [many] Кожні { $interval } тижнів
   *[other] Кожні { $interval } тижня
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] Щотижня: { $days }
    [few] Кожні { $interval } тижні: { $days }
    [many] Кожні { $interval } тижнів: { $days }
   *[other] Кожні { $interval } тижня: { $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] Щомісяця
    [few] Кожні { $interval } місяці
    [many] Кожні { $interval } місяців
   *[other] Кожні { $interval } місяця
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] Щомісяця, { $day }-го числа
    [few] Кожні { $interval } місяці, { $day }-го числа
    [many] Кожні { $interval } місяців, { $day }-го числа
   *[other] Кожні { $interval } місяця, { $day }-го числа
}
calendar-card-repeat-yearly = { $interval ->
    [one] Щороку
    [few] Кожні { $interval } роки
    [many] Кожні { $interval } років
   *[other] Кожні { $interval } року
}
calendar-card-repeat-until = { $rule } до { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] один раз
    [few] { $count } рази
    [many] { $count } разів
   *[other] { $count } разу
}
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] пропущено 1 дату
    [few] пропущено { $count } дати
    [many] пропущено { $count } дат
   *[other] пропущено { $count } дати
})
calendar-card-repeat-custom = Повторюється за особливим розкладом
calendar-card-join = Приєднатися: { $provider }
calendar-card-organizer-is = Організатор: { $who }
calendar-card-attendee-summary = { $count ->
    [one] { $count } учасник
    [few] { $count } учасники
    [many] { $count } учасників
   *[other] { $count } учасника
} · прийняли: { $accepted }, під питанням: { $tentative }, відхилили: { $declined }
calendar-card-people-all = Показати всіх ({ $count })
calendar-card-people-fewer = Показати менше
calendar-card-rsvp-accepted = Прийнято
calendar-card-rsvp-tentative = Під питанням
calendar-card-rsvp-declined = Відхилено
calendar-card-rsvp-none = Відповіді поки немає
calendar-card-in-calendar = У вашому календарі
calendar-card-cancelled = Скасовано
calendar-card-open = Відкрити в календарі
calendar-card-agenda = Ваш календар на { $day }
calendar-card-agenda-title = Ваш день
calendar-card-agenda-this-event = Ця подія
calendar-card-agenda-unavailable = Не вдалося завантажити ваш календар на цей день.
calendar-card-conflicts = { $count ->
    [one] Перетинається з: { $titles }
    [few] Перетинається з { $count } подіями: { $titles }
    [many] Перетинається з { $count } подіями: { $titles }
   *[other] Перетинається з { $count } подіями: { $titles }
}
calendar-card-no-conflicts = Перетинів у вашому календарі немає
calendar-card-invited-as = Вас запросили як { $address }.
calendar-card-not-invited = У цьому запрошенні не вказано жодної з ваших адрес. Його можна додати до календаря, але відповісти на нього не можна.
calendar-card-not-invited-short = У цьому запрошенні не вказано жодної з ваших адрес.
calendar-card-out-of-date = Це запрошення застаріле. Новіша версія вже є у вашому календарі.
calendar-card-counter = Новий час запропоновано: { $who }. Пропозиції не застосовуються автоматично.
calendar-card-todos-skipped = { $count ->
    [one] У файлі також є 1 завдання, його не додано.
    [few] У файлі також є { $count } завдання, їх не додано.
    [many] У файлі також є { $count } завдань, їх не додано.
   *[other] У файлі також є { $count } завдання, їх не додано.
}
calendar-card-respond = Відповісти на запрошення
calendar-card-accept = Прийняти
calendar-card-tentative = Під питанням
calendar-card-decline = Відхилити
calendar-card-propose = Запропонувати інший час
calendar-card-propose-start = Новий початок
calendar-card-propose-send = Надіслати пропозицію
calendar-card-propose-invalid = Вкажіть чинний новий час початку.
calendar-card-send-response = Надіслати відповідь: { $organizer }
calendar-card-comment = Коментар (необов’язково)
calendar-card-calendar = Календар
calendar-card-events = Події в цьому файлі
calendar-card-add-one = Додати
calendar-card-add-all = Додати всі ({ $count })
calendar-card-added = { $count ->
    [one] Додано до вашого календаря.
    [few] До вашого календаря додано { $count } події.
    [many] До вашого календаря додано { $count } подій.
   *[other] До вашого календаря додано { $count } події.
}
calendar-card-remove = Вилучити з календаря
calendar-card-removed = Вилучено з вашого календаря.
calendar-card-record-reply = Оновити статус учасника
calendar-card-answered-sent = { $answer }. Вашу відповідь надіслано: { $organizer }.
calendar-card-answered-not-sent = { $answer }. Ваш календар оновлено, але надіслати відповідь не вдалося.
calendar-card-answered-silently = { $answer }. Відповідь не надсилалася.
calendar-card-proposal-sent = Вашу пропозицію надіслано: { $organizer }.
calendar-card-not-sent = Не вдалося надіслати повідомлення.
calendar-card-more = { $count ->
    [one] ще 1
    [few] ще { $count }
    [many] ще { $count }
   *[other] ще { $count }
}

calendar-card-replied = Відповідь від { $who }: { $answer ->
    [accepted] прийнято.
    [tentative] прийнято під питанням.
    [declined] відхилено.
   *[other] відповіді поки немає.
}
