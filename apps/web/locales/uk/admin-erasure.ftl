# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Видалення та знеособлення
compliance-card-erasure-coverage = Замініть ідентифікатори однієї людини в історії аудиту та журналах і подивіться, у яких резервних копіях ще лишилися видалені люди.
admin-erasure-title = Видалення та знеособлення
admin-erasure-intro = Знеособлення людини замінює її адресу та ідентифікатори облікового запису в історії аудиту й у ротованих файлах журналів сталим псевдонімом із ключем. Ланцюг хешів аудиту перераховується й запечатується знову, тож перевірка проходить. Кожне видалення та знеособлення також залишає мітку, тому відновлення старої резервної копії знову видалить цих людей.
admin-erasure-docs = Реєстр того, що видаляється, експортується й зберігається для кожної таблиці, міститься в docs/admin/erasure-coverage.md.
admin-erasure-scrub = Знеособити людину
admin-erasure-scrub-note = Спершу запустіть попередній перегляд: він підраховує рядки та записи журналу, які змінилися б, і нічого не змінює.
admin-erasure-address = Адреса
admin-erasure-preview = Попередній перегляд
admin-erasure-run = Знеособити людину
admin-erasure-dry-note = Лише попередній перегляд. Нічого не змінено.
admin-erasure-done-note = Знеособлено. Тепер людина відображається як { $pseudonym }.
admin-erasure-nothing = В історії цю людину не згадано.
admin-erasure-col-table = Таблиця або файл
admin-erasure-col-rows = Рядки
admin-erasure-confirm-title = Знеособити цю людину в історії?
admin-erasure-confirm-body = Записи аудиту та журналів, де названо цю людину, переписуються з псевдонімом. Це не можна скасувати; без ключа адресу за псевдонімом відновити неможливо.
admin-erasure-settings = Налаштування
admin-erasure-keys = Ключі псевдонімів
admin-erasure-keys-note = Псевдоніми створюються поточним поколінням ключа. Ротація додає покоління; попередні залишаються, доки їх не видалить заданий вище строк зберігання.
admin-erasure-rotate = Змінити ключ
admin-erasure-keys-none = Ключ ще не створено. Перше знеособлення створить його.
admin-erasure-col-gen = Покоління
admin-erasure-col-created = Створено
admin-erasure-col-state = Стан
admin-erasure-key-current = Поточний
admin-erasure-key-retired = Виведений, збережений
admin-erasure-key-destroy = Виведений, буде видалено { $when }
admin-erasure-chain = Ланцюг аудиту
admin-erasure-chain-note = Кожен запис аудиту містить хеш попереднього. Знеособлення переписує записи й знову запечатує ланцюг, зберігаючи старий і новий хеш вершини.
admin-erasure-seal = Запечатати нові записи
admin-erasure-chain-ok = Ланцюг перевірено: запечатано записів — { $rows }, очікують запечатування — { $unsealed }.
admin-erasure-chain-bad = Ланцюг не проходить перевірку; перший хибний запис на позиції { $seq }.
admin-erasure-col-time = Час
admin-erasure-col-range = Позиції
admin-erasure-backups = Резервні копії
admin-erasure-backups-note = Резервна копія записує водяний знак міток під час створення. Поточний водяний знак: { $watermark }.
admin-erasure-col-label = Мітка
admin-erasure-col-location = Розташування
admin-erasure-col-watermark = Водяний знак
admin-erasure-col-marker = Видалено з
admin-erasure-col-actions = Дії
admin-erasure-backups-none = Резервних копій не зареєстровано.
admin-erasure-contains-erased = Містить видалених суб'єктів ({ $count })
admin-erasure-no-marker = Немає
admin-erasure-forget = Забути
admin-erasure-register = Зареєструвати резервну копію
