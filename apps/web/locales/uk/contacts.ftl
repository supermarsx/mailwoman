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
contacts-favorites = Обране
contacts-groups = Групи
contacts-new-group = Нова група
contacts-new-group-name = Назва нової групи
contacts-group-name-placeholder = Назва групи
contacts-create = Створити
contacts-data = Дані
contacts-import = Імпортувати…

# -- Business card -----------------------------------------------------------
contacts-card = Контакт { $name }
contacts-favorited = В обраному
contacts-favorite-action = До обраного
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

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Дані контакту
contacts-address-recipients = Показати отримувачів
contacts-address-save = Зберегти контакт
contacts-address-compose = Надіслати лист
contacts-address-unsaved = Цю адресу не збережено у ваших контактах.
contacts-address-book = Адресна книга
contacts-address-lookup-failed = Не вдалося завантажити відомості про контакт. Спробуйте ще раз.
contacts-address-save-failed = Не вдалося зберегти контакт. Спробуйте ще раз.
contacts-address-saved = Контакт збережено.
contacts-address-loading = Завантаження контакту…
contacts-editor-person = Ім’я та робота
contacts-editor-reach = Пошта й телефон
contacts-editor-other = Нотатки та уподобання
contacts-editor-save-error = Не вдалося зберегти контакт. Ваші зміни залишилися тут. Спробуйте ще раз.
contacts-group-editor-help = Створіть групу в адресній книзі, потім додавайте учасників з їхніх карток контактів.
contacts-group-details = Відомості про групу
contacts-group-save-error = Не вдалося зберегти групу. Ваші зміни залишилися тут. Спробуйте ще раз.
contacts-edit-group = Редагувати групу контактів
contacts-edit-named-group = Редагувати групу { $name }
contacts-group-name = Назва групи
contacts-group-edit-help = Змініть назву цієї групи. Її адресна книга та склад залишаться без змін.
contacts-back-to-list = Назад до контактів
contacts-membership-title = Склад групи
contacts-membership-help = Виберіть групу, потім додавайте або вилучайте її учасників. Зміни зберігаються одразу.
contacts-membership-no-groups = Груп поки немає. Створіть групу, щоб почати додавати учасників.
contacts-membership-pick = Виберіть групу, щоб побачити її учасників.
contacts-membership-of = Учасники групи { $name }
contacts-membership-members = Учасники
contacts-membership-add = Введіть ім’я або адресу, щоб додати учасника
contacts-membership-add-group = Введіть назву групи, щоб додати цей контакт
contacts-membership-count = { $count ->
    [one] { $count } учасник
    [few] { $count } учасники
    [many] { $count } учасників
   *[other] { $count } учасника
}
contacts-membership-failed = Не вдалося зберегти деякі зміни складу. У списку показано збережений стан.
contacts-membership-manage = Керування складом групи
contacts-import-source = Джерело
contacts-import-book = Адресна книга
contacts-import-review = Перевірка
contacts-import-empty = Виберіть файл або вставте текст vCard чи CSV, потім натисніть «Попередній перегляд». Знайдені контакти з’являться тут, і будь-який із них можна пропустити.
contacts-merge-sets = Можливі дублікати
contacts-merge-set = Дублікати контакту { $name }
contacts-merge-keep = Картка, яку потрібно залишити
contacts-merge-done = Об’єднано з контактом { $name }.
contacts-address-actions = Дії з контактом
contacts-address-save-to = Зберегти до контактів
contacts-address-goto = Перейти до контакту
contacts-address-copy = Копіювати дані контакту
contacts-address-copied = Дані контакту скопійовано.
contacts-address-copy-failed = Не вдалося скопіювати дані контакту.
contacts-share = Поділитися контактом
contacts-share-email = Надіслати контакт поштою
contacts-share-copy = Копіювати як vCard
contacts-share-download = Завантажити .vcf
contacts-share-native = Поділитися…
contacts-share-copied = vCard скопійовано.
contacts-share-copy-failed = Не вдалося скопіювати vCard.
contacts-share-downloaded = vCard завантажено.
contacts-share-failed = Не вдалося поділитися контактом.
contacts-share-attach-failed = Не вдалося прикріпити файл .vcf, тому vCard міститься в тексті листа.
contacts-address-saved-favorite = Контакт збережено й додано до обраного.
contacts-favorite-add = Додати до обраного
contacts-favorite-remove = Вилучити з обраного
contacts-favorite-save = Зберегти й додати до обраного
contacts-favorite-added = { $name } додано до обраного.
contacts-favorite-removed = { $name } вилучено з обраного.
contacts-favorite-failed = Не вдалося оновити обране.
contacts-select-row = Вибрати: { $name }
contacts-selected-count = Вибрано: { $count }
contacts-select-clear = Скасувати вибір
contacts-bulk-actions = Дії з вибраними контактами
contacts-bulk-favorites-added = Додано до обраного: { $count }.
contacts-bulk-favorites-removed = Вилучено з обраного: { $count }.
contacts-row-menu = Дії для: { $name }
contacts-favorites-empty = В обраному поки порожньо. Натисніть зірочку біля контакту, щоб додати його.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } вже у ваших контактах.

contacts-save-already-badge = Вже збережено

contacts-save-already-many = Усі { $count } контактів уже збережено.

contacts-save-automated = Це виглядає як автоматичний відправник (без відповіді, сповіщення).

contacts-save-book = Адресна книга

contacts-save-book-readonly = { $name } (тільки для читання)

contacts-save-cancel = Скасувати

contacts-save-choose = Виберіть контакти…

contacts-save-contacts = Зберегти контакти з цього повідомлення

contacts-save-dialog-title = Зберегти контакти з цього повідомлення

contacts-save-email = Електронна пошта

contacts-save-failed = Не вдалося зберегти контакт. Спробуйте знову.

contacts-save-group = Група

contacts-save-group-none = Жодної групи

contacts-save-loading = Завантаження адрес…

contacts-save-name = Ім'я

contacts-save-new = Збережіть нові контакти

contacts-save-no-book = Жодна адресна книга не може приймати нові контакти.

contacts-save-no-sender = Це повідомлення не має адреси відправника.

contacts-save-note = Збережено з повідомлення на { $date }

contacts-save-nothing = Нікого не потрібно зберігати: усі вже є у ваших контактах, це ви, або виглядає автоматично.

contacts-save-open = відкритий

contacts-save-open-contact = Відкрити контакт

contacts-save-open-link = Відкрити контакт

contacts-save-own = Це одна з ваших власних адрес.

contacts-save-partial = Збережено { $saved } з { $total } контактів. Інших врятувати не вдалося.

contacts-save-pick = Заощаджуйте { $email }

contacts-save-recipients = Зберегти всіх одержувачів

contacts-save-removed = Збережені контакти видалено.

contacts-save-role-bcc = Прихована копія

contacts-save-role-cc = Cc

contacts-save-role-from = Від

contacts-save-role-replyTo = Відповісти

contacts-save-role-sender = Відправник

contacts-save-role-to = до

contacts-save-saved-heading = Вже у ваших контактах

contacts-save-saved-many = Збережено { $count } контактів у { $book }

contacts-save-saved-one = Збережено { $name } до { $book }

contacts-save-sender = Зберегти відправника як контакт

contacts-save-settings-book = Адресна книга

contacts-save-settings-book-default = Адресна книга за замовчуванням

contacts-save-settings-enabled = Пропозиція Зберегти контакти з цього повідомлення

contacts-save-settings-enabled-help = Додає елемент до меню повідомлень і до меню «Ще» читача.

contacts-save-settings-note = Додайте примітку з датою

contacts-save-settings-note-help = У примітці написано «Збережено з повідомлення», а потім дата.

contacts-save-settings-reply-to = Використовуйте Reply-To для списків і відправників сповіщень

contacts-save-settings-reply-to-help = Зберігає адресу «Відповісти» замість «Від» для цих відправників.

contacts-save-settings-scope = Позначено першим у діалоговому вікні

contacts-save-settings-scope-all = Всім на повідомлення

contacts-save-settings-scope-sender = Лише відправник

contacts-save-settings-sender = Пропозиція Зберегти відправника як контакт

contacts-save-settings-sender-help = Пункт меню повідомлень, який можна зробити одним клацанням миші.

contacts-save-settings-skip = Пропустити автоматичних відправників

contacts-save-settings-skip-help = Виключає адреси для невідповідей, сповіщень і відмов, якщо ви не позначите їх.

contacts-save-settings-title = Зберегти контакти з повідомлень

contacts-save-show-automated = Показати автоматичні та невідповідні адреси

contacts-save-submit = { $count ->
        [one] Зберегти { $count } контакт
        [few] Зберегти { $count } контакти
        [many] Зберегти { $count } контактів
       *[other] Збережіть { $count } контакту
    }

contacts-save-undo = Скасувати
