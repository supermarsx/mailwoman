# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Зберігання
retention-intro = Правила, які переміщують або видаляють вашу пошту через задану кількість днів. Правило може враховувати теку, мітку, відправника, розмір, прочитаність або наявність вкладення. Позначені прапорцем листи та листи з міткою keep не чіпаються, доки ви цього не зміните. Видалена пошта спочатку потрапляє до кошика й остаточно видаляється після періоду скасування.
retention-disabled = Правила зберігання вимкнено на цьому сервері. Ваші наявні правила не виконуються.
retention-never = ніколи
retention-error = Не вдалося завантажити або зберегти налаштування зберігання. Спробуйте ще раз.
retention-last-run = Останній запуск: { $when }
retention-next-run = наступний запуск приблизно через { $days } дн.
retention-next-run-unknown = наступний запуск ще не заплановано
retention-enabled = Виконувати мої правила зберігання
retention-trash-first = Спочатку переміщувати видалену пошту до кошика
retention-trash-forced = Цей сервер завжди спочатку переміщує пошту до кошика й остаточно видаляє її після періоду скасування.
retention-undo = Період скасування в днях
retention-undo-help = Пошта, переміщена правилом до кошика, зберігається там стільки часу, перш ніж буде остаточно видалена. Від 0 до { $max } дн.
retention-pause = Призупинити всі правила на 30 днів
retention-paused-until = Призупинено до { $when }
retention-resume = Відновити
retention-policy = Що дозволяє цей сервер
retention-limits-both = Правило, що видаляє пошту, чекає щонайменше { $min } дн. і щонайбільше { $max } дн.
retention-limits-min = Правило, що видаляє пошту, чекає щонайменше { $min } дн.
retention-max-rules = Можна мати до { $count } правил.
retention-permanent-allowed = Правило може видаляти пошту остаточно, якщо вимкнути крок із кошиком.
retention-permanent-denied = Правила не можуть видаляти пошту остаточно. Пошта потрапляє до кошика й остаточно видаляється через { $days } дн.
retention-protected = Теки, яких правила ніколи не торкаються: { $folders }.
retention-mandatory = Задано адміністратором: пошта в теці { $folder } старша за { $days } дн. видаляється.
retention-rules = Ваші правила
retention-add = Нове правило
retention-no-rules = Правил поки немає. Застосуйте шаблон нижче або додайте власне правило.
retention-rule-paused = призупинено
retention-rule-on = Увімкнено
retention-rule-enabled = Правило { $name } увімкнено
retention-edit-short = Змінити
retention-delete = Видалити
retention-new = Нове правило
retention-edit = Змінити правило
retention-name = Назва
retention-action = Що зробити
retention-action-keep = Зберігати завжди
retention-action-delete = Видалити
retention-action-trash = Перемістити до кошика
retention-action-archive = Перемістити до архіву
retention-action-move = Перемістити до теки
retention-action-markRead = Позначити як прочитане
retention-days = Через скільки днів
retention-target = Тека
retention-choose-folder = Виберіть теку
retention-scope = Яка пошта
retention-folder = Тека
retention-any-folder = Будь-яка тека
retention-subfolders = Включно з вкладеними теками
retention-labels = Мітки (через кому)
retention-senders = Адреси відправників (через кому)
retention-domains = Домени відправників (через кому)
retention-min-size = Мінімальний розмір у КБ
retention-max-size = Максимальний розмір у КБ
retention-read = Стан прочитання
retention-read-any = Прочитані або непрочитані
retention-read-read = Лише прочитані
retention-read-unread = Лише непрочитані
retention-attachment = Вкладення
retention-attachment-any = Із вкладенням або без
retention-attachment-with = Із вкладенням
retention-attachment-without = Без вкладення
retention-exclude = Не чіпати
retention-exclude-flagged = Листи з прапорцем (із зірочкою)
retention-exclude-unread = Непрочитані листи
retention-exclude-tags = Листи з цими мітками (через кому)
retention-exclude-note = Пошта, що перебуває під юридичним утриманням, не чіпається ніколи, незалежно від налаштувань тут.
retention-role-inbox = Вхідні
retention-role-sent = Надіслані
retention-role-archive = Архів
retention-role-junk = Спам
retention-role-trash = Кошик
retention-role-drafts = Чернетки
retention-btn-preview = Попередній перегляд
retention-save = Зберегти
retention-cancel = Скасувати
retention-preview-none = Жодне повідомлення не зміниться.
retention-preview-count = Це змінить повідомлень: { $count }.
retention-preview-truncated = Для цього попереднього перегляду прочитано лише частину поштової скриньки.
retention-preview-sample = Приклади
retention-no-subject = (без теми)
retention-do-trash = до кошика
retention-do-move = переміщено
retention-do-delete = видалено
retention-do-markRead = позначено як прочитане
retention-skipped-legalHold = Ваш обліковий запис перебуває під юридичним утриманням. Нічого не змінено.
retention-skipped-restricted = Обробку вашого облікового запису обмежено. Нічого не змінено.
retention-skipped-noRules = У вас немає активних правил, і адміністратор їх не задав.
retention-error-name = Вкажіть назву правила.
retention-error-days = Кількість днів має бути цілим числом від 0 до 3650.
retention-error-min = Правило, що видаляє пошту, має чекати щонайменше { $days } дн.
retention-error-max = Правило, що видаляє пошту, може чекати щонайбільше { $days } дн.
retention-error-target = Виберіть теку, до якої потрібно переміщувати пошту.
retention-summary-keep = зберігати завжди
retention-summary-delete = через { $days } дн. до кошика, потім видаляється остаточно
retention-summary-delete-permanent = через { $days } дн. видаляється остаточно
retention-summary-trash = через { $days } дн. до кошика
retention-summary-archive = через { $days } дн. до архіву
retention-summary-move = через { $days } дн. до теки
retention-summary-markRead = через { $days } дн. позначається як прочитане
retention-templates = Шаблони
retention-templates-help = Шаблон додає готове правило. Перевірте його мітки та кількість днів, перш ніж на нього покладатися.
retention-adopt = Додати це правило
retention-run-heading = Запустити зараз
retention-preview-all = Попередній перегляд усіх правил
retention-btn-run = Запустити мої правила зараз
retention-run-result = Змінено повідомлень: { $count }.
retention-history = Нещодавні запуски
retention-history-when = Коли
retention-history-matched = Знайдено
retention-history-changed = Змінено
retention-history-errors = Помилки
retention-context-action = Зберігання…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Календарі

pim-cal-days = Днів після завершення події

pim-cal-enabled = Видалити завершені події

pim-cal-help = Видаляйте події у вибраних вами календарях через задану кількість днів після їх завершення. Повторювані події ніколи не видаляються.

pim-cal-title = Календар подій

pim-days-range = Виберіть між { $min } та { $max } днями.

pim-held-title = Видалені нотатки можна відновити

pim-held-until = Зберігається до { $when }

pim-limits = Між { $min } і { $max } днями.

pim-notes-days = Днів із моменту останньої зміни нотатки

pim-notes-enabled = Видаліть незаймані нотатки

pim-notes-help = Видаліть нотатки, які ви не змінювали протягом встановленої кількості днів. Закріплені нотатки зберігаються. Видалену нотатку можна відновити протягом { $days } днів.

pim-notes-title = Примітки

pim-off = Вимкнено вашим адміністратором.

pim-preview = Попередній перегляд

pim-restore = Відновити

pim-result = Події { $events } та примітки { $notes }.

pim-run = Біжи зараз

pim-run-title = Біжи зараз

pim-save = зберегти

pim-skipped = { $count } варіант(ів) було пропущено, оскільки ваш адміністратор їх вимкнув.

pim-untitled = Замітка без назви

retention-action-stripAttachments = Видаліть великі вкладення

retention-do-stripAttachments = видалені вкладення

retention-error-strip-disabled = Видалення вкладень заборонено вашим адміністратором.

retention-error-strip-min = Правило, яке видаляє вкладення, має чекати щонайменше { $days } днів.

retention-fd-action = Що відбувається з поштою в { $folder }

retention-fd-days = Днів до обробки { $folder }

retention-fd-help = Встановіть, як довго лист залишатиметься в папці перед переміщенням або видаленням. Папка, для якої встановлено значення «Успадкувати», не має власного за замовчуванням; наведені вище правила та обмеження адміністратора все ще застосовуються. Підпапки слідують за папкою.

retention-fd-inherit = Успадкувати

retention-fd-mandatory = Адміністратор зберігає цю папку протягом { $days } днів

retention-fd-protected = Охороняється адміністратором

retention-fd-save = Зберегти стандартні параметри папки

retention-fd-title = Стандартні параметри папки

retention-rule-strip-off = Вимкнено вашим адміністратором

retention-strip-allowed = Правило може видаляти вкладення розміром принаймні { $kb } КБ із пошти, що має принаймні { $days } день тому. Коротка примітка замінює кожне вкладення.

retention-strip-denied = Ваш адміністратор вимкнув видалення вкладень.

retention-strip-done = З повідомлень { $count } вилучено вкладення, звільнивши { $size }.

retention-strip-inline-kept = Зображення, показані в повідомленні, зберігаються.

retention-strip-inline-removed = Зображення, показані всередині повідомлення, також видаляються.

retention-strip-limits = Видаляє вкладення розміром принаймні { $kb } КБ із пошти, що має принаймні { $days } день тому. Оригінал залишається у кошику { $undo } днів.

retention-strip-preview = Повідомлення { $count } втратять вкладення, звільняючи { $size }.

retention-strip-preview-estimate = Близько { $count } повідомлень втратить вкладення, звільнивши до { $size }.

retention-strip-skipped = { $count } правил, які видаляють вкладення, було пропущено, оскільки ваш адміністратор вимкнув це.

retention-summary-stripAttachments = через { $days } днів великі вкладення видалено
