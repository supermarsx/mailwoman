# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: rules. Source: ../en/rules.ftl

rules-title = Правила та фільтри
rules-intro = Автоматично сортувати, позначати тегами або обробляти вхідну пошту. Правила, що використовують лише стандартні умови, виконуються на поштовому сервері як Sieve; решта виконується рушієм під час доставлення.
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
rules-runs-engine-detail = Застосовується рушієм під час доставлення.

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
