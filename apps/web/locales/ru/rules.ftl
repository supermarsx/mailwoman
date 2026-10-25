# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: rules. Source: ../en/rules.ftl

rules-title = Правила и фильтры
rules-intro = Управляйте скриптами фильтрации и визуальными правилами. Доступные функции зависят от аккаунта и настроек администратора.
rules-empty = Правил пока нет.
rules-unavailable = Этот почтовый сервер не поддерживает почтовые правила.
rules-load-failed = Не удалось загрузить правила.
rules-new = Новое правило
rules-list-label = Почтовые правила
rules-enabled = Включено
rules-delete-named = Удалить правило { $name }

# Builder
rules-builder-title = Конструктор правил
rules-builder-hint = Выберите правило для изменения или создайте новое.
rules-name-label = Название правила
rules-match-legend = Когда выполнять
rules-match-all = Все условия выполнены
rules-match-any = Выполнено любое условие
rules-conditions-legend = Условия
rules-actions-legend = Действия
rules-add-condition = Добавить условие
rules-add-action = Добавить действие
rules-remove-condition = Удалить условие
rules-remove-action = Удалить действие
rules-cond-type-label = Поле
rules-cond-op-label = Оператор
rules-cond-value-label = Значение для сравнения
rules-act-type-label = Действие
rules-act-value-label = Значение действия
rules-save = Сохранить правило
rules-editing = Изменение: { $name }
rules-save-refused = Правило не сохранено: { $reason }

# Condition fields and operators
rules-cond-from = От
rules-cond-to = Кому
rules-cond-subject = Тема
rules-cond-thread-unsupported = Цепочка { $op } { $value }: условия по цепочкам не поддерживаются, поэтому правило нельзя сохранить, пока это условие не удалено.
rules-op-contains = содержит
rules-op-is = равно

# Actions
rules-act-move = Переместить в папку
rules-act-tag = Добавить тег
rules-act-archive = Архивировать
rules-act-suppress = Не показывать уведомление
rules-act-stop = Остановить обработку

# Where it runs
rules-runs-server = Сервер (Sieve)
rules-runs-server-detail = Загружается на почтовый сервер и применяется там.
rules-runs-engine = Движок
rules-runs-engine-detail = Применяется при синхронизации новых входящих; ответы и перемещения выполняются при следующей синхронизации.

# Panes
rules-panes-label = Инструменты правил
rules-tab-builder = Конструктор
rules-tab-raw = Исходный Sieve
rules-tab-dryrun = Пробный запуск

# Raw editor
rules-raw-label = Исходный код Sieve
rules-raw-reset = Вернуть сгенерированный
rules-lint-clean = Проблем не найдено.

# Dry run
rules-dryrun-title = Пробный запуск
rules-dryrun-help = Введите пример сообщения, чтобы увидеть, какие правила сработают.
rules-dryrun-results = Результаты пробного запуска
rules-dryrun-none = Нет включённых правил для проверки.
rules-dryrun-nomatch = Нет совпадения
rules-dryrun-stopped = Не достигнуто (предыдущее правило остановило обработку)

rules-scripts-label = Скрипты Sieve
rules-script-active = Активен
rules-script-new = Новый скрипт
rules-script-export = Экспортировать скрипт
rules-script-import = Импортировать скрипт
rules-script-too-large = Скрипт превышает допустимый размер.
rules-script-read-only = Администратор отключил прямое редактирование скриптов.
rules-script-validate = Проверить скрипт
rules-script-deactivate = Отключить
rules-script-activate = Активировать
rules-script-saved = Скрипт сохранён.

rules-vacation-title = Ответ об отсутствии
rules-vacation-enabled = Включить автоматические ответы
rules-vacation-from = Начало (UTC)
rules-vacation-to = Окончание (UTC)
rules-vacation-body = Текст ответа
rules-vacation-saved = Ответ об отсутствии сохранён.
rules-script-template = Шаблон скрипта
rules-forward-title = Пересылка
rules-forward-address = Пересылать на
rules-forward-copy = Сохранять копию в этом аккаунте
rules-forward-refused = Администратор запретил пересылку на этот домен.
