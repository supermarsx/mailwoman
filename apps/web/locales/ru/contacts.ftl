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
contacts-favorites = ★ Избранное
contacts-groups = Группы
contacts-new-group = Новая группа
contacts-new-group-name = Название новой группы
contacts-group-name-placeholder = Название группы
contacts-create = Создать
contacts-data = Данные
contacts-import = Импорт…

# -- Business card -----------------------------------------------------------
contacts-card = Контакт { $name }
contacts-favorited = ★ В избранном
contacts-favorite-action = ☆ В избранное
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
