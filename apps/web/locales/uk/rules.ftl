# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: rules. Source: ../en/rules.ftl

rules-title = Правила та фільтри
rules-intro = Керуйте сценаріями фільтрації та візуальними правилами. Доступні функції залежать від облікового запису й налаштувань адміністратора.
rules-empty = Правил ще немає.
rules-unavailable = Цей поштовий сервер не підтримує поштові правила.
rules-load-failed = Не вдалося завантажити правила.
rules-new = Нове правило
rules-list-label = Поштові правила
rules-enabled = Увімкнено
rules-delete-named = Видалити правило { $name }

# Builder
rules-builder-title = Конструктор правил
rules-builder-hint = Виберіть правило для редагування або створіть нове.
rules-name-label = Назва правила
rules-match-legend = Коли виконувати
rules-match-all = Відповідає всім умовам
rules-match-any = Відповідає будь-якій умові
rules-conditions-legend = Умови
rules-actions-legend = Дії
rules-add-condition = Додати умову
rules-add-action = Додати дію
rules-remove-condition = Видалити умову
rules-remove-action = Видалити дію
rules-cond-type-label = Поле
rules-cond-op-label = Оператор
rules-cond-value-label = Значення для порівняння
rules-act-type-label = Дія
rules-act-value-label = Значення дії
rules-save = Зберегти правило
rules-editing = Редагування { $name }
rules-save-refused = Правило не збережено: { $reason }

# Condition fields and operators
rules-cond-from = Від
rules-cond-to = Кому
rules-cond-subject = Тема
rules-cond-thread-unsupported = Ланцюжок { $op } { $value }: умови для ланцюжків не підтримуються, тому правило не можна зберегти, доки цю умову не видалено.
rules-op-contains = містить
rules-op-is = дорівнює

# Actions
rules-act-move = Перемістити до теки
rules-act-tag = Додати тег
rules-act-archive = Архівувати
rules-act-suppress = Не сповіщати
rules-act-stop = Зупинити обробку

# Where it runs
rules-runs-server = Сервер (Sieve)
rules-runs-server-detail = Завантажується на поштовий сервер і застосовується там.
rules-runs-engine = Рушій
rules-runs-engine-detail = Застосовується під час синхронізації нових вхідних; відповіді та переміщення чекають наступної синхронізації.

# Panes
rules-panes-label = Інструменти правил
rules-tab-builder = Конструктор
rules-tab-raw = Код Sieve
rules-tab-dryrun = Пробний запуск

# Raw editor
rules-raw-label = Вихідний код Sieve
rules-raw-reset = Повернути згенерований
rules-lint-clean = Проблем не знайдено.

# Dry run
rules-dryrun-title = Пробний запуск
rules-dryrun-help = Введіть зразок повідомлення, щоб побачити, які правила спрацюють.
rules-dryrun-results = Результати пробного запуску
rules-dryrun-none = Немає увімкнених правил для перевірки.
rules-dryrun-nomatch = Немає збігу
rules-dryrun-stopped = Не досягнуто (попереднє правило зупинило обробку)

rules-scripts-label = Скрипти Sieve
rules-script-active = Активний
rules-script-new = Новий скрипт
rules-script-export = Експортувати скрипт
rules-script-import = Імпортувати скрипт
rules-script-too-large = Скрипт перевищує допустимий розмір.
rules-script-read-only = Адміністратор вимкнув пряме редагування скриптів.
rules-script-validate = Перевірити скрипт
rules-script-deactivate = Вимкнути
rules-script-activate = Активувати
rules-script-saved = Скрипт збережено.

rules-vacation-title = Відповідь про відсутність
rules-vacation-enabled = Увімкнути автоматичні відповіді
rules-vacation-from = Початок (UTC)
rules-vacation-to = Кінець (UTC)
rules-vacation-body = Текст відповіді
rules-vacation-saved = Відповідь про відсутність збережено.
rules-script-template = Шаблон сценарію
rules-forward-title = Пересилання
rules-forward-address = Пересилати на
rules-forward-copy = Зберігати копію в цьому обліковому записі
rules-forward-refused = Адміністратор заборонив пересилання на цей домен.
