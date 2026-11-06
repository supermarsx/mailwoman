# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Профили функций
admin-features-intro = Профиль — это именованный набор переключателей функций. Назначьте его домену, группе или пользователю. Индивидуальные назначения поддерживают встроенные почтовые ящики и проверенные идентификации прокси или SSO; они управляют доступом к функциям Mailwoman.
admin-features-new = Новый профиль
admin-features-search = Поиск профилей
admin-features-empty = Профилей функций пока нет.
admin-features-no-match = Нет профилей, подходящих под поиск.
admin-features-load-error = Не удалось загрузить каталог функций.
admin-features-not-loaded = Каталог функций ещё не загружен. Перезагрузите страницу и повторите попытку.
admin-features-stale = Другой администратор изменил каталог функций. Он перезагружен; проверьте значения и сохраните снова.
admin-features-not-found = Профиля с идентификатором { $id } нет.
admin-features-col-name = Название
admin-features-col-values = Переключатели
admin-features-col-assigned = Назначен
admin-features-values-count = { $count ->
    [one] Задан { $count } переключатель
    [few] Заданы { $count } переключателя
    [many] Задано { $count } переключателей
   *[other] Задано { $count } переключателя
}
admin-features-assigned-count = { $count ->
    [0] Не назначен
    [one] { $count } место
    [few] { $count } места
    [many] { $count } мест
   *[other] { $count } места
}
admin-features-editor = Параметры профиля
admin-features-name = Название профиля
admin-features-name-help = Показывается только администраторам, до 128 символов.
admin-features-name-invalid = Введите название длиной от 1 до 128 символов.
admin-features-switches = Переключатели функций
admin-features-switches-help = «Не задано» оставляет функцию следующему профилю, домену или значению сервера по умолчанию. Заблокированное значение нельзя изменить ни пользователю, ни более низкому уровню.
admin-features-col-feature = Функция
admin-features-col-value = Значение
admin-features-col-lock = Блокировка
admin-features-value-for = Значение для { $feature }
admin-features-unset = Не задано
admin-features-on = Разрешено
admin-features-off = Не разрешено
admin-features-lock = Заблокировано
admin-features-save = Сохранить профиль
admin-features-create = Создать профиль
admin-features-delete = Удалить профиль
admin-features-delete-title = Удалить профиль { $name }?
admin-features-delete-body = { $count ->
    [0] Профиль нигде не назначен. Его переключатели перестают действовать сразу.
    [one] Профиль будет убран из { $count } места, где он назначен, и его переключатели перестанут действовать сразу.
    [few] Профиль будет убран из { $count } мест, где он назначен, и его переключатели перестанут действовать сразу.
    [many] Профиль будет убран из { $count } мест, где он назначен, и его переключатели перестанут действовать сразу.
   *[other] Профиль будет убран из { $count } места, где он назначен, и его переключатели перестанут действовать сразу.
}
admin-features-saved = Профиль { $name } сохранён.
admin-features-deleted = Профиль { $name } удалён.
admin-features-assigned = Профиль { $name } назначен.
admin-features-unassigned = Назначение профиля { $name } снято.
admin-features-assignments = Назначен
admin-features-assignments-help = Профили действуют в порядке приоритета, высший первым. При равном приоритете назначение пользователю действует раньше группы, а группы — раньше домена.
admin-features-assignments-empty = Этот профиль ещё не назначен.
admin-features-assign-form = Назначить этот профиль
admin-features-col-scope = Применяется к
admin-features-col-target = Название
admin-features-col-priority = Приоритет
admin-features-col-actions = Действия
admin-features-scope-domain = Домен
admin-features-scope-group = Группа
admin-features-scope-user = Пользователь
admin-features-assign = Назначить
admin-features-unassign = Снять
admin-features-no-domain = Домены не настроены.
admin-features-no-group = Групп пользователей нет.
admin-features-no-user = Ещё ни одна учётная запись почтового ящика не входила.
admin-features-tools = Инструменты профиля
admin-features-preset = Начать с предустановки
admin-features-preset-choose = Выберите предустановку
admin-features-preset-apply = Применить предустановку
admin-features-preset-applied = Предустановка { $name } применена. Проверьте значения и сохраните.
admin-features-preset-full = Полная
admin-features-preset-standard = Стандартная
admin-features-preset-mail-only = Только почта
admin-features-preset-minimal = Минимальная / киоск
admin-features-import = Импорт JSON
admin-features-export = Экспорт JSON
admin-features-exported = Профиль экспортирован в файл JSON.
admin-features-imported = { $count ->
    [one] Импортировано { $count } значение. Проверьте его и сохраните.
    [few] Импортировано { $count } значения. Проверьте их и сохраните.
    [many] Импортировано { $count } значений. Проверьте их и сохраните.
   *[other] Импортировано { $count } значения. Проверьте их и сохраните.
}
admin-features-import-json = Файл не является корректным JSON.
admin-features-import-format = Файл не является профилем функций Mailwoman.
admin-features-import-version = Этот файл профиля функций создан в более новой версии и не может быть импортирован.
admin-features-clone = Клонировать
admin-features-clone-name = Копия { $name }
admin-features-filter = Найти функцию
admin-features-filter-placeholder = Название, параметр или ключ
admin-features-filter-none = Нет функций, подходящих под поиск.
admin-features-diff-summary = { $differs ->
    [one] { $differs } значение отличается от значения сервера по умолчанию
    [few] { $differs } значения отличаются от значения сервера по умолчанию
    [many] { $differs } значений отличаются от значения сервера по умолчанию
   *[other] { $differs } значения отличаются от значения сервера по умолчанию
} · повторяют его: { $same }
admin-features-differs = Отличается от значения по умолчанию
admin-features-default-is = По умолчанию: { $value }
admin-features-fix-values = Исправьте значения, отмеченные как недопустимые, перед сохранением.
admin-features-allow-all = Разрешить все
admin-features-deny-all = Запретить все
admin-features-clear-all = Очистить
admin-features-group-allow = Разрешить все функции в группе { $group }
admin-features-group-deny = Запретить все функции в группе { $group }
admin-features-group-clear = Очистить все значения в группе { $group }
admin-features-enforcement-ui = Скрыто только в приложении
admin-features-enforcement-partial = Некоторые ограничения пока не учитываются
admin-features-other = Другие параметры, которые может содержать профиль
admin-features-other-help = Все прочие параметры, которые администратор может задавать для профиля. Оставьте поле пустым, чтобы сохранить значение сервера.
admin-features-cat-compose = Почта и составление писем
admin-features-cat-reading = Чтение и конфиденциальность
admin-features-cat-views = Представления и поиск
admin-features-cat-organise = Организация
admin-features-cat-pim = Календарь, контакты, задачи, заметки и ключи
admin-features-cat-data = Данные
admin-features-cat-accounts = Учётные записи и доступ
admin-features-cat-integrations = Интеграции
admin-features-cat-developer = Доступ для разработчиков
admin-features-cat-personal = Персонализация
admin-features-feature-compose-sending = Ограничения отправки
admin-features-feature-compose-sending-help = Число получателей в письме сервер отклоняет сверх ограничения. Число писем в час и в сутки сохраняется для профиля, но пока не учитывается.
admin-features-feature-storage = Квота хранилища
admin-features-feature-storage-help = Ограничение хранилища сохраняется для профиля; собственная квота почтового сервера продолжает действовать.
admin-features-feature-access-protocols = Протоколы доступа к почте
admin-features-feature-access-protocols-help = Какими протоколами почтового сервера эти пользователи могут входить, через запятую: imap, pop3, ews.
admin-features-feature-accounts-linked = Связанные ящики
admin-features-feature-calendar = Календарь
admin-features-feature-contacts = Контакты
admin-features-feature-tasks = Задачи
admin-features-feature-notes = Заметки
admin-features-preview = Действующие значения
admin-features-preview-help = К чему приводится каждая функция для одного пользователя или пользователя одного домена после учёта всех профилей, групп, значений домена и блокировок.
admin-features-preview-kind = Предпросмотр для
admin-features-preview-target = Кто
admin-features-preview-show = Показать
admin-features-preview-loading = Загрузка…
admin-features-preview-state = Действующее
admin-features-preview-from = Определено
admin-features-preview-profiles = Применяемые профили, побеждает первый: { $names }
admin-features-preview-no-profiles = Ни один профиль не применяется; решают значения домена и сервера.
admin-features-source-default = Встроенное значение по умолчанию
admin-features-source-env-fallback = Значение по умолчанию из окружения
admin-features-source-env-pin = Окружение (закреплено)
admin-features-source-global = Значение сервера
admin-features-source-domain = Значение домена
admin-features-source-profile = Профиль
admin-features-source-profile-named = Профиль { $name }
admin-features-source-group = Значение группы
admin-features-source-admin-user = Значение, заданное для этого пользователя
admin-features-source-user = Собственный выбор пользователя
admin-features-source-locked = { $source } (заблокировано)
admin-retention-col-category = Категория
admin-retention-col-kept = Хранится
admin-retention-col-change = Изменение
admin-retention-cat-default = По умолчанию
admin-retention-cat-admin = Действия администраторов
admin-retention-cat-security = События безопасности
admin-retention-cat-alert = Оповещения пользователей
admin-retention-forever = Каждая запись
admin-retention-days = { $days ->
    [one] { $days } день
    [few] { $days } дня
    [many] { $days } дней
   *[other] { $days } дня
}
admin-retention-follows-default = По умолчанию: { $kept }
admin-retention-hold-on = Правовая блокировка
admin-retention-read-only = Для изменения сроков хранения нужно разрешение settings.write.
admin-retention-edit-for = Срок хранения для категории «{ $category }»
admin-retention-keep = Хранить
admin-retention-mode-inherit = Следовать значению по умолчанию
admin-retention-mode-forever = Каждая запись
admin-retention-mode-days = Заданное число дней
admin-retention-days-label = Дни
admin-retention-hold = Правовая блокировка
admin-retention-save = Сохранить
admin-retention-bounds = Введите целое число от { $min } до { $max }.
admin-retention-saved = Срок хранения для категории «{ $category }» сохранён.
admin-retention-task-none = Задачи удаления пока нет. Она создаётся ежедневно в 03:15 UTC, когда впервые сохраняется число дней.
admin-retention-task-disabled = Плановая задача удаления отключена; записи удаляются только по кнопке «Удалить сейчас».
admin-retention-task-next = Следующее плановое удаление: { $when }.
admin-retention-task-daily = Удаление выполняется ежедневно в 03:15 UTC.
admin-retention-last = Последнее удаление, затронувшее записи: { $when }.
admin-retention-prune = Удалить сейчас
admin-retention-prune-title = Удалить записи журнала аудита сейчас?
admin-retention-prune-body = Записи старше срока хранения их категории будут удалены. Категории, которые хранят каждую запись или находятся под правовой блокировкой, не затрагиваются. Это действие нельзя отменить.
admin-retention-pruned = { $count ->
    [0] Нет записей, достаточно старых для удаления.
    [one] Удалена { $count } запись.
    [few] Удалено { $count } записи.
    [many] Удалено { $count } записей.
   *[other] Удалено { $count } записи.
}
admin-auth-attempted = Попытка от имени { $name }
admin-features-feature-compose-open-tracking = Отслеживание открытия сообщений
admin-features-feature-compose-open-tracking-help = Разрешает записывать открытия подходящих форматированных сообщений с помощью отслеживающего изображения сервера. Требуется настроенный публичный URL.
admin-features-feature-reading-original-html = Просмотр исходного HTML
admin-features-feature-reading-original-html-help = Позволяет пользователям включать исходный HTML для отдельных сообщений в изолированном фрейме просмотра. По умолчанию остаётся очищенный HTML.
admin-features-inherit = Наследовать
admin-features-override = Задать значение
admin-features-setting-mode = Режим значения для { $setting }
admin-features-inherit-help = Наследование убирает значение и блокировку этого профиля. Решает следующий применимый профиль, домен или значение сервера; действующий предпросмотр показывает результат и его источник.
admin-features-domain-alias = { $alias } (псевдоним домена { $domain })
admin-features-domain-aliases = { $domain } (псевдонимы: { $aliases })

# -- Machine translation (2026-11): added to match en ---------------------
admin-retention-title = Хранение журнала аудита
admin-retention-intro = Как долго хранятся записи аудита по категориям. Категория без собственного значения следует значению по умолчанию; без значения по умолчанию хранится каждая запись. Правовая блокировка останавливает удаление для своей категории, а блокировка значения по умолчанию — для всех. Изменения и удаления записываются в журнал аудита.
admin-retention-loading = Загрузка политики хранения…
admin-retention-unavailable = Хранение аудита недоступно на этом сервере.
admin-retention-never = Записи ещё не удалялись.
