# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Контакты
contacts-search = Поиск контактов
contacts-new = Новый контакт
contacts-find-duplicates = Найти дубликаты
contacts-loading = Загрузка…
contacts-none = Нет контактов.
contacts-list = Список контактов
contacts-favorite = Добавить в избранное: { $name }
contacts-export-vcard = Экспорт в vCard
contacts-export-csv = Экспорт в CSV
contacts-select-hint = Выберите контакт, чтобы увидеть его карточку.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Адресные книги и группы
contacts-address-books = Адресные книги
contacts-all = Все контакты
contacts-favorites = Избранное
contacts-groups = Группы
contacts-new-group = Новая группа
contacts-new-group-name = Название новой группы
contacts-group-name-placeholder = Название группы
contacts-create = Создать
contacts-data = Данные
contacts-import = Импорт…

# -- Business card -----------------------------------------------------------
contacts-card = Контакт { $name }
contacts-favorited = В избранном
contacts-favorite-action = В избранное
contacts-email = Эл. почта
contacts-phone = Телефон
contacts-dates = Даты
contacts-groups-label = Группы
contacts-notes = Заметки
contacts-security-key = Ключ безопасности
contacts-key-on-file = Ключ или сертификат сохранён (только для отображения, пока не появится поддержка PGP/S-MIME).
contacts-no-key = Ключ не сохранён.
contacts-directory-security = Безопасность в каталоге
contacts-member-of = Состоит в
contacts-group-membership = Участники группы { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Новый контакт
contacts-edit-contact = Изменить контакт
contacts-full-name = Полное имя
contacts-given = Имя
contacts-given-placeholder = Имя
contacts-surname = Фамилия
contacts-surname-placeholder = Фамилия
contacts-organization = Организация
contacts-job-title = Должность
contacts-job-title-placeholder = Должность
contacts-email-n = Эл. почта { $n }
contacts-email-label = Метка адреса эл. почты { $n }
contacts-email-label-placeholder = рабочий
contacts-remove-email = Удалить адрес эл. почты { $n }
contacts-add-email = Добавить адрес эл. почты
contacts-phone-n = Телефон { $n }
contacts-remove-phone = Удалить телефон { $n }
contacts-add-phone = Добавить телефон
contacts-security-key-label = Ключ безопасности (непрозрачное значение)
contacts-security-key-placeholder = Ключ PGP / сертификат S-MIME (только для отображения)
contacts-favorite-word = Избранное

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Импорт контактов
contacts-imported = Импортировано: { $count ->
    [one] { $count } контакт
    [few] { $count } контакта
    [many] { $count } контактов
   *[other] { $count } контакта
}.
contacts-import-hint = Вставьте vCard (.vcf) или CSV либо выберите файл. Формат определяется автоматически.
contacts-import-file = Импортировать файл
contacts-paste = Вставьте vCard или CSV
contacts-preview = Предпросмотр
contacts-map-columns = Сопоставление столбцов
contacts-col = Столбец
contacts-maps-to = Соответствует
contacts-sample = Пример
contacts-map-column = Сопоставить столбец { $header }
contacts-preview-count = Предпросмотр ({ $count })
contacts-import-preview = Предпросмотр импорта
contacts-no-name = (без имени)
contacts-no-email = нет адреса эл. почты
contacts-import-n = Импортировать { $count ->
    [one] { $count } контакт
    [few] { $count } контакта
    [many] { $count } контактов
   *[other] { $count } контакта
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Объединение дубликатов
contacts-merge-review-hint = Проверьте объединённую карточку. Остальные карточки будут помечены как удалённые (это можно отменить).
contacts-merged-preview = Предпросмотр объединения
contacts-emails = Адреса эл. почты
contacts-phones = Телефоны
contacts-merge-confirm = Объединить контакты
contacts-no-duplicates = Дубликаты не найдены.
contacts-possible-duplicates = Возможных дубликатов: { $count }
contacts-review-merge = Проверить объединение

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Данные контакта
contacts-address-recipients = Показать получателей
contacts-address-save = Сохранить контакт
contacts-address-compose = Отправить письмо
contacts-address-unsaved = Этот адрес не сохранён в ваших контактах.
contacts-address-book = Адресная книга
contacts-address-lookup-failed = Не удалось загрузить сведения о контакте. Повторите попытку.
contacts-address-save-failed = Не удалось сохранить контакт. Повторите попытку.
contacts-address-saved = Контакт сохранён.
contacts-address-loading = Загрузка контакта…
contacts-editor-person = Имя и работа
contacts-editor-reach = Почта и телефон
contacts-editor-other = Заметки и предпочтения
contacts-editor-save-error = Не удалось сохранить контакт. Ваши изменения остались здесь. Повторите попытку.
contacts-group-editor-help = Создайте группу в адресной книге, затем добавляйте участников из их карточек контактов.
contacts-group-details = Сведения о группе
contacts-group-save-error = Не удалось сохранить группу. Ваши изменения остались здесь. Повторите попытку.
contacts-edit-group = Изменить группу контактов
contacts-edit-named-group = Изменить группу { $name }
contacts-group-name = Название группы
contacts-group-edit-help = Измените название этой группы. Её адресная книга и состав останутся прежними.
contacts-back-to-list = Назад к контактам
contacts-membership-title = Состав группы
contacts-membership-help = Выберите группу, затем добавляйте или удаляйте её участников. Изменения сохраняются сразу.
contacts-membership-no-groups = Групп пока нет. Создайте группу, чтобы начать добавлять участников.
contacts-membership-pick = Выберите группу, чтобы увидеть её участников.
contacts-membership-of = Участники группы { $name }
contacts-membership-members = Участники
contacts-membership-add = Введите имя или адрес, чтобы добавить участника
contacts-membership-add-group = Введите название группы, чтобы добавить этот контакт
contacts-membership-count = { $count ->
    [one] { $count } участник
    [few] { $count } участника
    [many] { $count } участников
   *[other] { $count } участника
}
contacts-membership-failed = Не удалось сохранить некоторые изменения состава. В списке показано сохранённое состояние.
contacts-membership-manage = Управление составом группы
contacts-import-source = Источник
contacts-import-book = Адресная книга
contacts-import-review = Проверка
contacts-import-empty = Выберите файл или вставьте текст vCard или CSV, затем нажмите «Предпросмотр». Найденные контакты появятся здесь, и любой из них можно исключить.
contacts-merge-sets = Возможные дубликаты
contacts-merge-set = Дубликаты контакта { $name }
contacts-merge-keep = Карточка, которую нужно оставить
contacts-merge-done = Объединено с контактом { $name }.
contacts-address-actions = Действия с контактом
contacts-address-save-to = Сохранить в контакты
contacts-address-goto = Перейти к контакту
contacts-address-copy = Копировать данные контакта
contacts-address-copied = Данные контакта скопированы.
contacts-address-copy-failed = Не удалось скопировать данные контакта.
contacts-share = Поделиться контактом
contacts-share-email = Отправить контакт по почте
contacts-share-copy = Копировать как vCard
contacts-share-download = Скачать .vcf
contacts-share-native = Поделиться…
contacts-share-copied = vCard скопирована.
contacts-share-copy-failed = Не удалось скопировать vCard.
contacts-share-downloaded = vCard скачана.
contacts-share-failed = Не удалось поделиться контактом.
contacts-share-attach-failed = Не удалось прикрепить файл .vcf, поэтому vCard находится в тексте письма.
contacts-address-saved-favorite = Контакт сохранён и добавлен в избранное.
contacts-favorite-add = Добавить в избранное
contacts-favorite-remove = Удалить из избранного
contacts-favorite-save = Сохранить и добавить в избранное
contacts-favorite-added = { $name } добавлен в избранное.
contacts-favorite-removed = { $name } удалён из избранного.
contacts-favorite-failed = Не удалось обновить избранное.
contacts-select-row = Выбрать: { $name }
contacts-selected-count = Выбрано: { $count }
contacts-select-clear = Снять выделение
contacts-bulk-actions = Действия с выбранными контактами
contacts-bulk-favorites-added = Добавлено в избранное: { $count }.
contacts-bulk-favorites-removed = Удалено из избранного: { $count }.
contacts-row-menu = Действия для: { $name }
contacts-favorites-empty = В избранном пока пусто. Нажмите на звёздочку у контакта, чтобы добавить его.
