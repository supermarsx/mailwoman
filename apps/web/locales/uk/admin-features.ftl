# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Профілі функцій
admin-features-intro = Профіль — це іменований набір перемикачів функцій. Призначте його домену, групі або користувачу. Індивідуальні призначення підтримують вбудовані поштові скриньки та перевірені ідентифікації проксі чи SSO; вони керують доступом до функцій Mailwoman.
admin-features-new = Новий профіль
admin-features-search = Пошук профілів
admin-features-empty = Профілів функцій поки немає.
admin-features-no-match = Немає профілів, що відповідають пошуку.
admin-features-load-error = Не вдалося завантажити каталог функцій.
admin-features-not-loaded = Каталог функцій ще не завантажено. Перезавантажте сторінку та повторіть спробу.
admin-features-stale = Інший адміністратор змінив каталог функцій. Його перезавантажено; перевірте значення та збережіть знову.
admin-features-not-found = Профілю з ідентифікатором { $id } немає.
admin-features-col-name = Назва
admin-features-col-values = Перемикачі
admin-features-col-assigned = Призначено
admin-features-values-count = { $count ->
    [one] Задано { $count } перемикач
    [few] Задано { $count } перемикачі
    [many] Задано { $count } перемикачів
   *[other] Задано { $count } перемикача
}
admin-features-assigned-count = { $count ->
    [0] Не призначено
    [one] { $count } місце
    [few] { $count } місця
    [many] { $count } місць
   *[other] { $count } місця
}
admin-features-editor = Параметри профілю
admin-features-name = Назва профілю
admin-features-name-help = Показується лише адміністраторам, до 128 символів.
admin-features-name-invalid = Введіть назву завдовжки від 1 до 128 символів.
admin-features-switches = Перемикачі функцій
admin-features-switches-help = «Не задано» залишає функцію наступному профілю, домену або типовому значенню сервера. Заблоковане значення не можуть змінити ні користувач, ні нижчий рівень.
admin-features-col-feature = Функція
admin-features-col-value = Значення
admin-features-col-lock = Блокування
admin-features-value-for = Значення для { $feature }
admin-features-unset = Не задано
admin-features-on = Дозволено
admin-features-off = Не дозволено
admin-features-lock = Заблоковано
admin-features-save = Зберегти профіль
admin-features-create = Створити профіль
admin-features-delete = Видалити профіль
admin-features-delete-title = Видалити профіль { $name }?
admin-features-delete-body = { $count ->
    [0] Профіль ніде не призначено. Його перемикачі припиняють діяти одразу.
    [one] Профіль буде вилучено з { $count } місця, де його призначено, і його перемикачі припинять діяти одразу.
    [few] Профіль буде вилучено з { $count } місць, де його призначено, і його перемикачі припинять діяти одразу.
    [many] Профіль буде вилучено з { $count } місць, де його призначено, і його перемикачі припинять діяти одразу.
   *[other] Профіль буде вилучено з { $count } місця, де його призначено, і його перемикачі припинять діяти одразу.
}
admin-features-saved = Профіль { $name } збережено.
admin-features-deleted = Профіль { $name } видалено.
admin-features-assigned = Профіль { $name } призначено.
admin-features-unassigned = Призначення профілю { $name } знято.
admin-features-assignments = Призначено
admin-features-assignments-help = Профілі діють у порядку пріоритету, вищий першим. За рівного пріоритету призначення користувачу діє раніше за групу, а групи — раніше за домен.
admin-features-assignments-empty = Цей профіль ще не призначено.
admin-features-assign-form = Призначити цей профіль
admin-features-col-scope = Застосовується до
admin-features-col-target = Назва
admin-features-col-priority = Пріоритет
admin-features-col-actions = Дії
admin-features-scope-domain = Домен
admin-features-scope-group = Група
admin-features-scope-user = Користувач
admin-features-assign = Призначити
admin-features-unassign = Зняти
admin-features-no-domain = Домени не налаштовано.
admin-features-no-group = Груп користувачів немає.
admin-features-no-user = Жоден обліковий запис поштової скриньки ще не входив.
admin-features-tools = Інструменти профілю
admin-features-preset = Почати з пресета
admin-features-preset-choose = Оберіть пресет
admin-features-preset-apply = Застосувати пресет
admin-features-preset-applied = Пресет { $name } застосовано. Перегляньте значення та збережіть.
admin-features-preset-full = Повний
admin-features-preset-standard = Стандартний
admin-features-preset-mail-only = Лише пошта
admin-features-preset-minimal = Мінімальний / кіоск
admin-features-import = Імпорт JSON
admin-features-export = Експорт JSON
admin-features-exported = Профіль експортовано у файл JSON.
admin-features-imported = { $count ->
    [one] Імпортовано { $count } значення. Перегляньте його та збережіть.
    [few] Імпортовано { $count } значення. Перегляньте їх та збережіть.
    [many] Імпортовано { $count } значень. Перегляньте їх та збережіть.
   *[other] Імпортовано { $count } значення. Перегляньте їх та збережіть.
}
admin-features-import-json = Файл не є коректним JSON.
admin-features-import-format = Файл не є профілем функцій Mailwoman.
admin-features-import-version = Цей файл профілю функцій створено в новішій версії, і його не можна імпортувати.
admin-features-clone = Клонувати
admin-features-clone-name = Копія { $name }
admin-features-filter = Знайти функцію
admin-features-filter-placeholder = Назва, параметр або ключ
admin-features-filter-none = Немає функцій, що відповідають пошуку.
admin-features-diff-summary = { $differs ->
    [one] { $differs } значення відрізняється від типового значення сервера
    [few] { $differs } значення відрізняються від типового значення сервера
    [many] { $differs } значень відрізняються від типового значення сервера
   *[other] { $differs } значення відрізняються від типового значення сервера
} · повторюють його: { $same }
admin-features-differs = Відрізняється від типового
admin-features-default-is = Типове: { $value }
admin-features-fix-values = Виправте значення, позначені як недійсні, перед збереженням.
admin-features-allow-all = Дозволити всі
admin-features-deny-all = Заборонити всі
admin-features-clear-all = Очистити
admin-features-group-allow = Дозволити всі функції в групі { $group }
admin-features-group-deny = Заборонити всі функції в групі { $group }
admin-features-group-clear = Очистити всі значення в групі { $group }
admin-features-enforcement-ui = Приховано лише в застосунку
admin-features-enforcement-partial = Деякі обмеження поки не враховуються
admin-features-other = Інші параметри, які може містити профіль
admin-features-other-help = Усі інші параметри, які адміністратор може задавати для профілю. Залиште поле порожнім, щоб зберегти значення сервера.
admin-features-cat-compose = Пошта та створення листів
admin-features-cat-reading = Читання та конфіденційність
admin-features-cat-views = Подання та пошук
admin-features-cat-organise = Організація
admin-features-cat-pim = Календар, контакти, завдання, нотатки та ключі
admin-features-cat-data = Дані
admin-features-cat-accounts = Облікові записи та доступ
admin-features-cat-integrations = Інтеграції
admin-features-cat-developer = Доступ для розробників
admin-features-cat-personal = Персоналізація
admin-features-feature-compose-sending = Обмеження надсилання
admin-features-feature-compose-sending-help = Кількість отримувачів у листі сервер відхиляє понад обмеження. Кількість листів на годину та на добу зберігається для профілю, але поки не враховується.
admin-features-feature-storage = Квота сховища
admin-features-feature-storage-help = Обмеження сховища зберігається для профілю; власна квота поштового сервера продовжує діяти.
admin-features-feature-access-protocols = Протоколи доступу до пошти
admin-features-feature-access-protocols-help = Якими протоколами поштового сервера ці користувачі можуть входити, через кому: imap, pop3, ews.
admin-features-feature-accounts-linked = Пов'язані скриньки
admin-features-feature-calendar = Календар
admin-features-feature-contacts = Контакти
admin-features-feature-tasks = Завдання
admin-features-feature-notes = Нотатки
admin-features-preview = Чинні значення
admin-features-preview-help = До чого зводиться кожна функція для одного користувача або користувача одного домену після врахування всіх профілів, груп, значень домену та блокувань.
admin-features-preview-kind = Попередній перегляд для
admin-features-preview-target = Хто
admin-features-preview-show = Показати
admin-features-preview-loading = Завантаження…
admin-features-preview-state = Чинне
admin-features-preview-from = Визначено
admin-features-preview-profiles = Профілі, що застосовуються, перемагає перший: { $names }
admin-features-preview-no-profiles = Жоден профіль не застосовується; вирішують значення домену та сервера.
admin-features-source-default = Вбудоване типове значення
admin-features-source-env-fallback = Типове значення із середовища
admin-features-source-env-pin = Середовище (закріплено)
admin-features-source-global = Значення сервера
admin-features-source-domain = Значення домену
admin-features-source-profile = Профіль
admin-features-source-profile-named = Профіль { $name }
admin-features-source-group = Значення групи
admin-features-source-admin-user = Значення, задане для цього користувача
admin-features-source-user = Власний вибір користувача
admin-features-source-locked = { $source } (заблоковано)
admin-retention-col-category = Категорія
admin-retention-col-kept = Зберігається
admin-retention-col-change = Зміна
admin-retention-cat-default = Типова
admin-retention-cat-admin = Дії адміністраторів
admin-retention-cat-security = Події безпеки
admin-retention-cat-alert = Сповіщення користувачів
admin-retention-forever = Кожен запис
admin-retention-days = { $days ->
    [one] { $days } день
    [few] { $days } дні
    [many] { $days } днів
   *[other] { $days } дня
}
admin-retention-follows-default = Типово: { $kept }
admin-retention-hold-on = Правове блокування
admin-retention-read-only = Для зміни строків зберігання потрібен дозвіл settings.write.
admin-retention-edit-for = Строк зберігання для категорії «{ $category }»
admin-retention-keep = Зберігати
admin-retention-mode-inherit = Дотримуватися типового
admin-retention-mode-forever = Кожен запис
admin-retention-mode-days = Задану кількість днів
admin-retention-days-label = Дні
admin-retention-hold = Правове блокування
admin-retention-save = Зберегти
admin-retention-bounds = Введіть ціле число від { $min } до { $max }.
admin-retention-saved = Строк зберігання для категорії «{ $category }» збережено.
admin-retention-task-none = Завдання видалення поки немає. Воно створюється щодня о 03:15 UTC, коли вперше зберігається кількість днів.
admin-retention-task-disabled = Заплановане завдання видалення вимкнено; записи видаляються лише кнопкою «Видалити зараз».
admin-retention-task-next = Наступне заплановане видалення: { $when }.
admin-retention-task-daily = Видалення виконується щодня о 03:15 UTC.
admin-retention-last = Останнє видалення, що зачепило записи: { $when }.
admin-retention-prune = Видалити зараз
admin-retention-prune-title = Видалити записи журналу аудиту зараз?
admin-retention-prune-body = Записи, старші за строк зберігання їхньої категорії, буде видалено. Категорії, що зберігають кожен запис або перебувають під правовим блокуванням, не зачіпаються. Цю дію не можна скасувати.
admin-retention-pruned = { $count ->
    [0] Немає записів, достатньо старих для видалення.
    [one] Видалено { $count } запис.
    [few] Видалено { $count } записи.
    [many] Видалено { $count } записів.
   *[other] Видалено { $count } запису.
}
admin-auth-attempted = Спроба від імені { $name }
admin-features-feature-compose-open-tracking = Відстеження відкриття повідомлень
admin-features-feature-compose-open-tracking-help = Дозволяє записувати відкриття придатних форматованих повідомлень за допомогою зображення відстеження сервера. Потрібен налаштований публічний URL.
admin-features-feature-reading-original-html = Перегляд початкового HTML
admin-features-feature-reading-original-html-help = Дозволяє користувачам вмикати початковий HTML для окремих повідомлень в ізольованому фреймі перегляду. Типово лишається очищений HTML.
admin-features-inherit = Успадковувати
admin-features-override = Задати значення
admin-features-setting-mode = Режим значення для { $setting }
admin-features-inherit-help = Успадкування вилучає значення та блокування цього профілю. Вирішує наступний застосовний профіль, домен або значення сервера; чинний попередній перегляд показує результат і його джерело.
admin-features-domain-alias = { $alias } (псевдонім домену { $domain })
admin-features-domain-aliases = { $domain } (псевдоніми: { $aliases })

# -- Machine translation (2026-11): added to match en ---------------------
admin-audit-retention-title = Зберігання журналу аудиту
admin-audit-retention-intro = Як довго зберігаються записи аудиту за категоріями. Категорія без власного значення дотримується типового; без типового значення зберігається кожен запис. Правове блокування зупиняє видалення для своєї категорії, а блокування типового значення — для всіх. Зміни та видалення записуються в журнал аудиту.
admin-audit-retention-loading = Завантаження політики зберігання…
admin-audit-retention-unavailable = Зберігання аудиту недоступне на цьому сервері.
