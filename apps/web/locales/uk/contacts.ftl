# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Контакти
contacts-search = Пошук контактів
contacts-new = Новий контакт
contacts-find-duplicates = Знайти дублікати
contacts-loading = Завантаження…
contacts-none = Немає контактів.
contacts-list = Список контактів
contacts-favorite = Додати { $name } до обраного
contacts-export-vcard = Експортувати vCard
contacts-export-csv = Експортувати CSV
contacts-select-hint = Виберіть контакт, щоб переглянути його картку.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Адресні книги та групи
contacts-address-books = Адресні книги
contacts-all = Усі контакти
contacts-favorites = ★ Обране
contacts-groups = Групи
contacts-new-group = Нова група
contacts-new-group-name = Назва нової групи
contacts-group-name-placeholder = Назва групи
contacts-create = Створити
contacts-data = Дані
contacts-import = Імпортувати…

# -- Business card -----------------------------------------------------------
contacts-card = Контакт { $name }
contacts-favorited = ★ В обраному
contacts-favorite-action = ☆ До обраного
contacts-email = Електронна пошта
contacts-phone = Телефон
contacts-dates = Дати
contacts-groups-label = Групи
contacts-notes = Нотатки
contacts-security-key = Ключ безпеки
contacts-key-on-file = Ключ або сертифікат збережено (лише для показу, доки не з'явиться підтримка PGP/S-MIME).
contacts-no-key = Ключа не збережено.
contacts-directory-security = Безпека в довіднику
contacts-member-of = Учасник груп
contacts-group-membership = Членство { $name } у групах

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Новий контакт
contacts-edit-contact = Редагувати контакт
contacts-full-name = Повне ім'я
contacts-given = Ім'я
contacts-given-placeholder = Ім'я
contacts-surname = Прізвище
contacts-surname-placeholder = Прізвище
contacts-organization = Організація
contacts-job-title = Посада
contacts-job-title-placeholder = Посада
contacts-email-n = Електронна пошта { $n }
contacts-email-label = Мітка електронної пошти { $n }
contacts-email-label-placeholder = робоча
contacts-remove-email = Видалити електронну пошту { $n }
contacts-add-email = Додати електронну пошту
contacts-phone-n = Телефон { $n }
contacts-remove-phone = Видалити телефон { $n }
contacts-add-phone = Додати телефон
contacts-security-key-label = Ключ безпеки (непрозорий заповнювач)
contacts-security-key-placeholder = Ключ PGP / сертифікат S-MIME (лише для показу)
contacts-favorite-word = Обране

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Імпорт контактів
contacts-imported = Імпортовано { $count ->
    [one] { $count } контакт
    [few] { $count } контакти
    [many] { $count } контактів
   *[other] { $count } контакту
}.
contacts-import-hint = Вставте vCard (.vcf) або CSV чи виберіть файл. Формат визначається автоматично.
contacts-import-file = Файл для імпорту
contacts-paste = Вставте vCard або CSV
contacts-preview = Попередній перегляд
contacts-map-columns = Зіставлення стовпців
contacts-col = Стовпець
contacts-maps-to = Відповідає
contacts-sample = Приклад
contacts-map-column = Зіставити стовпець { $header }
contacts-preview-count = Попередній перегляд ({ $count })
contacts-import-preview = Попередній перегляд імпорту
contacts-no-name = (без імені)
contacts-no-email = без електронної пошти
contacts-import-n = Імпортувати { $count ->
    [one] { $count } контакт
    [few] { $count } контакти
    [many] { $count } контактів
   *[other] { $count } контакту
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Об'єднання дублікатів
contacts-merge-review-hint = Перевірте об'єднану картку. Інші картки буде позначено як видалені (це можна скасувати).
contacts-merged-preview = Попередній перегляд об'єднання
contacts-emails = Адреси електронної пошти
contacts-phones = Телефони
contacts-merge-confirm = Об'єднати контакти
contacts-no-duplicates = Дублікатів не знайдено.
contacts-possible-duplicates = Можливі дублікати: { $count }
contacts-review-merge = Перевірити об'єднання
