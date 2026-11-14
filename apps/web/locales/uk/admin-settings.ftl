# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Дозволити надсилання
admin-settings-help-mail-allow-send = Дозволяє користувачам надсилати повідомлення. Якщо вимкнено, редактор вимикає «Надіслати» та її скорочення, а сервер відхиляє надсилання; чернетки все одно можна зберігати.
admin-settings-key-mail-allow-reply = Дозволити відповіді
admin-settings-help-mail-allow-reply = Дозволяє користувачам відповідати і відповідати всім із засобу читання та за допомогою скорочень. Якщо вимкнено, ці дії недоступні.
admin-settings-key-mail-allow-forward = Дозволити пересилання
admin-settings-help-mail-allow-forward = Дозволяє користувачам пересилати повідомлення в тексті листа або як вкладення. Якщо вимкнено, ці дії недоступні.
admin-settings-key-mail-allow-download = Дозволити завантаження
admin-settings-help-mail-allow-download = Дозволяє користувачам завантажувати повідомлення і вкладення. Якщо вимкнено, вебзастосунок не пропонує завантаження, а сервер відхиляє експорт повідомлень.
admin-settings-key-mail-allow-print = Дозволити друк
admin-settings-help-mail-allow-print = Дозволяє користувачам друкувати повідомлення із засобу читання. Якщо вимкнено, дія «Друк» недоступна.
admin-settings-key-mail-show-spam = Показувати теку «Спам»
admin-settings-help-mail-show-spam = Показує теку «Спам» або «Junk» і повідомлення, позначені як спам. Якщо вимкнено, вони приховані зі списку тек і з результатів пошуку; нічого не видаляється.
admin-settings-key-mail-show-auth-status = Показувати результати автентифікації відправника
admin-settings-help-mail-show-auth-status = Показує в засобі читання результати SPF, DKIM і DMARC із заголовка Authentication-Results.
admin-settings-key-mail-close-on-delete = Закривати повідомлення під час видалення
admin-settings-help-mail-close-on-delete = Закриває відкрите повідомлення, вкладку або окреме вікно, коли це повідомлення видалено або переміщено до кошика. Якщо вимкнено, засіб читання поводиться як раніше.
admin-settings-key-links-trusted-senders = Довірені відправники посилань
admin-settings-help-links-trusted-senders = Точні поштові скриньки відправників, посилання в перевірених повідомленнях яких можна відкривати без згоди для кожного повідомлення. Одна адреса на запис.
admin-settings-key-appearance-font-size = Розмір шрифту інтерфейсу
admin-settings-help-appearance-font-size = Розмір шрифту інтерфейсу в пікселях, від 12 до 24.
admin-settings-key-appearance-font-weight = Насиченість шрифту інтерфейсу
admin-settings-help-appearance-font-weight = Базова насиченість тексту інтерфейсу: 300, 400, 500 або 600. Виділений текст зберігає свою ієрархію.
admin-settings-key-appearance-font-color = Колір тексту інтерфейсу
admin-settings-help-appearance-font-color = Колір тексту інтерфейсу у форматі #RRGGBB. Залиште порожнім, щоб слідувати темі.
admin-settings-key-appearance-low-contrast = Пом'якшити контраст тексту
admin-settings-help-appearance-low-contrast = Пом'якшує контраст тексту інтерфейсу.
admin-settings-key-mail-message-list-order = Порядок списку повідомлень
admin-settings-help-mail-message-list-order = Порядок списку повідомлень: найновіші, найстаріші, відправник або тема.
admin-settings-key-mail-message-list-style = Стиль списку повідомлень
admin-settings-help-mail-message-list-style = Показувати список повідомлень як картки або як рядки із заголовками стовпців.
admin-settings-key-mail-message-list-preview = Попередній перегляд у списку повідомлень
admin-settings-help-mail-message-list-preview = Показувати фрагменти тексту в списку повідомлень.
admin-settings-key-mail-after-delete = Після видалення повідомлення
admin-settings-help-mail-after-delete = Яке повідомлення показується після видалення відкритого: наступне, попереднє або жодне.
admin-settings-key-reader-attachments-at-end = Вкладення після тексту
admin-settings-help-reader-attachments-at-end = Показувати вкладення після тексту повідомлення, а не перед ним.
admin-settings-key-notifications-ask-permission = Пропонувати дозвіл на сповіщення
admin-settings-help-notifications-ask-permission = Пропонувати запит браузера на дозвіл сповіщень користувачам на пристроях, де сповіщення не дозволено і не заблоковано.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Налаштування
admin-settings-description = Значення для всього сервера, для домену та для користувача. Заблоковане значення не можуть змінити користувачі та вужчі області.
admin-settings-list-caption = Налаштування
admin-settings-search = Пошук налаштувань
admin-settings-empty = Жодне налаштування не відповідає пошуку.
admin-settings-unavailable = Цей сервер ще не підтримує модель налаштувань.
admin-settings-col-setting = Налаштування
admin-settings-col-value = Значення
admin-settings-col-source = Джерело
admin-settings-col-overrides = Вужчі значення
admin-settings-open = Відкрити
admin-settings-overrides-none = Немає
admin-settings-overrides = Доменів: { $domains }, користувачів: { $users }
admin-settings-not-found = Налаштування з назвою { $key } немає.
admin-settings-value-on = Увімкнено
admin-settings-value-off = Вимкнено
admin-settings-value-empty = Не задано
admin-settings-value-list = { $count ->
    [one] { $count } елемент
    [few] { $count } елементи
    [many] { $count } елементів
   *[other] { $count } елемента
}
admin-settings-detail-key = Ключ: { $key }
admin-settings-global = Для всього сервера
admin-settings-global-help = Це значення діє для всіх користувачів, якщо тільки домен або користувач не має вужчого значення чи користувач не обрав власне там, де це дозволено.
admin-settings-value-label = Значення
admin-settings-json-hint = JSON. Сервер перевіряє значення й вказує частину, яку він не приймає.
admin-settings-json-invalid = Це некоректний JSON.
admin-settings-user-override = Користувачі можуть змінювати
admin-settings-user-override-help = Якщо ввімкнено, користувачі можуть обрати власне значення у своїх налаштуваннях. Якщо вимкнено, діє значення, задане тут або для їхнього домену.
admin-settings-lock-help = Коли заблоковано, значення доменів і вибір користувачів ігноруються, а це значення діє для всіх.
admin-settings-save = Зберегти
admin-settings-saved = Збережено. Зміна набуде чинності протягом 30 секунд у кожному процесі сервера.
admin-settings-saved-restart = Збережено. Зміна набуде чинності після перезапуску.
admin-settings-dedicated = Це налаштування змінюється в розділі { $section }, який спершу перевіряє його наслідки.
admin-settings-dedicated-link = Відкрити { $section }
admin-settings-pinned = Задано змінною середовища { $name }. Збережене тут значення діє, якщо змінну не задано.
admin-settings-lock-title = Заблокувати { $setting }?
admin-settings-lock-body = Блокування ігнорує вужчі значення зі списку нижче. Їх буде збережено, і вони знову діятимуть після розблокування.
admin-settings-lock-confirm = Заблокувати
admin-settings-impact-domain = { $count ->
    [one] { $count } значення домену ігнорується
    [few] { $count } значення доменів ігноруються
    [many] { $count } значень доменів ігноруються
   *[other] { $count } значення доменів ігноруються
}
admin-settings-impact-user = { $count ->
    [one] { $count } власний вибір користувача ігнорується
    [few] { $count } власні вибори користувачів ігноруються
    [many] { $count } власних виборів користувачів ігноруються
   *[other] { $count } власного вибору користувачів ігноруються
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } значення для користувача ігнорується
    [few] { $count } значення для користувачів ігноруються
    [many] { $count } значень для користувачів ігноруються
   *[other] { $count } значення для користувачів ігноруються
}
admin-settings-impact-profile = { $count ->
    [one] { $count } значення профілю функцій ігнорується
    [few] { $count } значення профілів функцій ігноруються
    [many] { $count } значень профілів функцій ігноруються
   *[other] { $count } значення профілів функцій ігноруються
}
admin-settings-impact-group = { $count ->
    [one] { $count } значення групи ігнорується
    [few] { $count } значення груп ігноруються
    [many] { $count } значень груп ігноруються
   *[other] { $count } значення груп ігноруються
}
admin-settings-domains = Значення доменів
admin-settings-domains-help = Значення домену діє для користувачів цього домену замість значення для всього сервера.
admin-settings-domains-caption = Значення доменів для { $setting }
admin-settings-domains-empty = Жоден домен не має власного значення.
admin-settings-domain = Домен
admin-settings-domain-value = Значення для цього домену
admin-settings-domain-locked = Заблоковано для користувачів домену
admin-settings-domain-add = Додати значення домену
admin-settings-domain-save = Зберегти значення домену
admin-settings-domain-remove = Видалити
admin-settings-domain-remove-title = Видалити значення домену { $domain }?
admin-settings-domain-remove-body = Користувачі домену { $domain } знову отримають значення для всього сервера.
admin-settings-domain-required = Введіть назву домену.
admin-settings-locked-yes = Заблоковано
admin-settings-locked-no = Не заблоковано
admin-settings-users = Значення для користувачів
admin-settings-users-help = Значення, задане адміністратором для одного користувача. Для лімітів користувача вказують адресою.
admin-settings-users-caption = Значення для користувачів: { $setting }
admin-settings-users-empty = Жоден користувач не має значення, заданого адміністратором.
admin-settings-user = Користувач
admin-settings-user-hint = Ідентифікатор облікового запису або адреса для лімітів.
admin-settings-user-value = Значення для цього користувача
admin-settings-user-add = Додати значення для користувача
admin-settings-user-save = Зберегти значення користувача
admin-settings-user-required = Введіть користувача.
admin-settings-user-remove-title = Видалити значення користувача { $user }?
admin-settings-user-remove-body = Користувач отримає значення свого домену або значення для всього сервера.
admin-settings-source-default = Вбудоване типове значення
admin-settings-source-envFallback = Змінна середовища (збереженого значення немає)
admin-settings-source-envPin = Змінна середовища
admin-settings-source-global = Збережено для всього сервера
admin-settings-cat-interface = Інтерфейс
admin-settings-cat-appearance = Вигляд
admin-settings-cat-mail = Пошта
admin-settings-cat-notifications = Сповіщення
admin-settings-cat-offline = Робота офлайн
admin-settings-cat-assistant = ШІ-асистент
admin-settings-cat-upstream = Поштовий сервер
admin-settings-cat-signin = Вхід
admin-settings-cat-admin = Консоль адміністратора
admin-settings-cat-passwords = Паролі
admin-settings-cat-limits = Ліміти та використання
admin-settings-cat-plugins = Плагіни
admin-settings-cat-features = Дозволи функцій
admin-settings-cat-integrations = Дозволи інтеграцій
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Дозволяє операції з файлами, спільним доступом, а також із календарем, контактами й завданнями Nextcloud, пов’язаними з обліковим записом.
admin-settings-key-cap-integration-google = Міст Google
admin-settings-help-cap-integration-google = Дозволяє операції мосту Google, зокрема отримання й оновлення токенів.
admin-settings-key-cap-integration-microsoft = Міст Microsoft
admin-settings-help-cap-integration-microsoft = Дозволяє операції мосту Microsoft, зокрема отримання й оновлення токенів.
admin-settings-key-cap-integration-family = Окрема інтеграція
admin-settings-help-cap-integration-family = Дозвіл для ідентифікатора окремої інтеграції; охоплення під час роботи наведено в каталозі функцій.
admin-settings-key-cap-ui-plugin-family = Окремий плагін інтерфейсу
admin-settings-help-cap-ui-plugin-family = Дозвіл для одного плагіна інтерфейсу. Використовуйте точний ключ політики з каталогу інтеграцій.
admin-settings-key-cap-ai-assistant = ШІ-асистент
admin-settings-help-cap-ai-assistant = Дозволяє налаштування та виклик асистента.
admin-settings-key-cap-plugins = Плагіни
admin-settings-help-cap-plugins = Дозволяє операції плагінів інтерфейсу та рушія.
admin-settings-key-cap-pim-calendar = Календар
admin-settings-help-cap-pim-calendar = Дозволяє методи календаря та підписки.
admin-settings-key-cap-pim-contacts = Контакти
admin-settings-help-cap-pim-contacts = Дозволяє методи контактів і адресних книг.
admin-settings-key-cap-pim-tasks = Завдання
admin-settings-help-cap-pim-tasks = Дозволяє методи завдань.
admin-settings-key-cap-pim-notes = Нотатки
admin-settings-help-cap-pim-notes = Дозволяє методи нотаток.
admin-settings-key-cap-export = Експорт
admin-settings-help-cap-export = Дозволяє експорт пошти та даних PIM.
admin-settings-key-cap-import = Імпорт
admin-settings-help-cap-import = Дозволяє імпорт пошти та даних PIM.
admin-settings-key-cap-api-keys = Ключі API
admin-settings-help-cap-api-keys = Політика ключів API. Охоплення застосування дивіться в каталозі функцій.
admin-settings-key-cap-oauth-apps = Застосунки OAuth
admin-settings-help-cap-oauth-apps = Політика застосунків OAuth. Охоплення застосування дивіться в каталозі функцій.
admin-settings-key-cap-rules = Правила пошти
admin-settings-help-cap-rules = Дозволяє керовані методи правил пошти й автовідповіді.
admin-settings-key-cap-rules-forward = Пересилання правилами
admin-settings-help-cap-rules-forward = Політика пересилання. Охоплення застосування дивіться в каталозі функцій.
admin-settings-key-cap-remote-images = Віддалені зображення
admin-settings-help-cap-remote-images = Дозвіл для віддалених зображень. Запитами, як і раніше, керують наявні налаштування приватності; перевірте охоплення функцій.
admin-settings-key-cap-user-background = Власне тло
admin-settings-help-cap-user-background = Дозвіл для власного тла, типово вимкнено. Перевірте охоплення функцій.
admin-settings-key-cap-user-css = Власний CSS
admin-settings-help-cap-user-css = Дозвіл для власного CSS, типово вимкнено. Перевірте охоплення функцій.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-limit-attachment-bytes = Ліміт розміру вкладення
admin-settings-help-limit-attachment-bytes = Політика вкладень у байтах, типово 50000000. Застосування під час роботи ще не реалізоване.
admin-settings-key-limit-send-per-hour = Ліміт надсилання за годину
admin-settings-help-limit-send-per-hour = Політика надсилання за годину; нуль означає без обмежень. Застосування під час роботи ще не реалізоване.
admin-settings-key-limit-recipients-per-message = Одержувачів у листі
admin-settings-help-limit-recipients-per-message = Політика кількості одержувачів; нуль означає без обмежень. Застосування під час роботи ще не реалізоване.
admin-settings-cat-import = Імпорт
admin-settings-cat-webhooks = Вебхуки
admin-settings-cat-realtime = Живі оновлення
admin-settings-cat-privacy = Приватність
admin-settings-cat-observability = Спостережуваність
admin-settings-cat-egress = Вихідні з’єднання
admin-settings-cat-updates = Оновлення
admin-settings-cat-backups = Резервні копії
admin-settings-cat-about = Про програму
admin-settings-cat-config = Конфігурація
admin-settings-cat-jobs = Завдання
admin-settings-cat-maintenance = Обслуговування
admin-settings-cat-health = Перевірки стану
admin-settings-cat-diagnostics = Діагностика
admin-settings-cat-users = Користувачі
admin-settings-cat-server-text = Тексти сервера
admin-settings-key-ui-language = Мова
admin-settings-help-ui-language = Мова поштового клієнта для користувачів, які не обрали власну. «Як у браузері» наслідує мови кожного браузера.
admin-settings-key-ui-notify-new-version = Сповіщення про нову версію
admin-settings-help-ui-notify-new-version = Увімкнено (типово): користувачам, які увійшли, показується сповіщення з кнопками «Оновити зараз» і «Пізніше», якщо після завантаження їхньої сторінки було розгорнуто новішу версію вебзастосунку. Вимкнено: сповіщення не показується, і користувачі отримують новий код під час наступного перезавантаження.
admin-settings-key-ui-undo-history-size = Розмір історії скасування та повтору
admin-settings-help-ui-undo-history-size = Скільки останніх дій із поштою (архівація, видалення, переміщення, позначення прочитаним, прапорець, мітка тощо) запам’ятовують кнопки «Скасувати» та «Повторити», від 0 до 50. 0 вимикає кнопки та скорочення Ctrl+Z. Типово 50. Користувачі можуть обрати власний розмір, але не більший за максимум нижче, якщо адміністратор не заблокує його.
admin-settings-key-folders-reorder-enabled = Перетягування тек
admin-settings-help-folders-reorder-enabled = Дозволяє змінювати порядок тек перетягуванням на бічній панелі та в налаштуваннях. Якщо вимкнено, маркери перетягування та команди «Перемістити» приховано. Користувачі можуть вимкнути це для себе, якщо налаштування не заблоковано.
admin-settings-key-folders-reorder-allow-nesting = Вкладання тек
admin-settings-help-folders-reorder-allow-nesting = Дозволяє перетягнути теку на іншу теку, щоб вкласти її, або назад на верхній рівень. Якщо вимкнено, кожна тека залишається у своїй батьківській, а порядок можна змінювати лише серед тек одного рівня.
admin-settings-key-folders-reorder-allow-server-move = Переміщувати теки на сервері
admin-settings-help-folders-reorder-allow-server-move = Дозволяє вкладанню чи виведенню змінювати справжню батьківську теку на поштовому сервері, тож зміна видна всюди. Якщо вимкнено, такі переміщення лишаються розташуванням лише в цьому застосунку.
admin-settings-key-folders-reorder-allow-system-folders = Переміщувати системні теки
admin-settings-help-folders-reorder-allow-system-folders = Дозволяє перетягувати «Вхідні», «Чернетки», «Надіслані», «Спам», «Кошик» та «Архів», змінюючи їхній порядок. Якщо вимкнено, вони лишаються закріпленими вгорі. Порядок змінюється лише в цьому застосунку; на сервері їх ніколи не перейменовують і не переміщують.
admin-settings-key-folders-reorder-max-depth = Глибина вкладення тек
admin-settings-help-folders-reorder-max-depth = Найглибший рівень вкладення тек, який може створити перетягування, від 1 до 16. Переміщення, після якого тека або її підтека опинилася б глибше, відхиляється.
admin-settings-key-folders-order-locked = Зафіксувати порядок тек
admin-settings-help-folders-order-locked = Фіксує порядок тек для всіх у цій області. Перетягування та команди «Перемістити» вимкнено, користувачі бачать підказку про блокування. Закріплення та порядок, заданий адміністратором, зберігаються.
admin-settings-key-folders-pinned-admin = Примусово закріплені теки
admin-settings-help-folders-pinned-admin = Теки, що показуються в групі «Закріплені» вгорі бічної панелі кожного користувача. Один запис у рядку: role:inbox або шлях теки, наприклад Shared/Support. Користувачі не можуть їх відкріпити чи перемістити. Запис без відповідної теки пропускається.
admin-settings-key-folders-order-default = Типовий порядок тек
admin-settings-help-folders-order-default = Порядок, у якому теки показуються користувачам, що не впорядкували їх, від першої до останньої. Один запис у рядку: role:<role> або шлях теки. Теки, яких немає у списку, йдуть далі в порядку сервера. Власний порядок користувача замінює цей.
admin-settings-key-folders-new-folder-placement = Розміщення нової теки
admin-settings-help-folders-new-folder-placement = Куди серед тек одного рівня потрапляє тека, що з’явилася на сервері: в кінець або за алфавітом.
admin-settings-key-ui-undo-history-max = Максимум історії скасування та повтору
admin-settings-help-ui-undo-history-max = Найбільша кількість дій, яку будь-який користувач може зберігати в історії скасування та повтору, від 0 до 50. Власний розмір користувача знижується до цього значення. 0 вимикає історію для всіх. Історія зберігається у вкладці браузера й очищується під час виходу.
admin-settings-key-ui-bar-mail-top-bar = Панель інструментів пошти
admin-settings-help-ui-bar-mail-top-bar = «pinned» залишає панель інструментів пошти з елементами розкладки завжди видимою. «auto» ховає її за вузьким маркером, який відкриває її під час наведення, дотику або переходу клавішею Tab.
admin-settings-key-ui-bar-workspace-tabs = Вкладки робочої області
admin-settings-help-ui-bar-workspace-tabs = «pinned» залишає смугу вкладок робочої області завжди видимою. «auto» показує лише вузький маркер із назвою активної вкладки, доки користувач не відкриє її.
admin-settings-key-ui-bar-mail-ribbon = Стрічка пошти
admin-settings-help-ui-bar-mail-ribbon = Для розкладки зі стрічкою. «pinned» залишає стрічку відкритою. «auto» показує лише назви вкладок; вкладка відкриває стрічку поверх списку листів.
admin-settings-key-ui-bar-composer-ribbon = Стрічка редактора
admin-settings-help-ui-bar-composer-ribbon = Для кожного вікна створення листа. «pinned» залишає стрічку редактора відкритою. «auto» показує лише назви вкладок; відкриття вкладки зсуває текст листа вниз, а не закриває його.
admin-settings-key-appearance-theme = Тема
admin-settings-help-appearance-theme = Кольорова тема, яку бачать користувачі, доки не оберуть власну. Блокування закріплює цю тему за всіма користувачами.
admin-settings-key-appearance-accent = Акцентний колір
admin-settings-help-appearance-accent = Акцентний колір теми. Порожньо означає власний акцент теми.
admin-settings-key-appearance-badge-color = Колір значка непрочитаних
admin-settings-help-appearance-badge-color = Колір значків із кількістю непрочитаних. Default зберігає колір теми; Accent наслідує акцентний колір; accent-lighter і accent-darker використовують акцент світліший або темніший на 20% з чорним чи білим текстом числа. Блокування перевизначає вибір облікового запису.
admin-settings-key-appearance-density = Щільність
admin-settings-help-appearance-density = Скільки місця займають списки та елементи керування: compact, cozy або relaxed.
admin-settings-key-appearance-font = Шрифт інтерфейсу
admin-settings-key-packages-allowed = Відстеження посилок дозволено
admin-settings-help-packages-allowed = Дозволяє відстеження посилок для охоплених облікових записів. Постачальник також має бути ввімкнений у розділі «Відстеження посилок», а кожен користувач має погодитися. Вимкнення цього налаштування негайно припиняє нові звернення до постачальника.
admin-settings-help-appearance-font = Оберіть вбудований шрифт для облікових записів без власного вибору шрифту або заблокуйте вибір для всіх охоплених облікових записів. Шрифти завантажуються з цього сервера; запит до Google Fonts не виконується. Default використовує шрифт теми.
admin-settings-key-appearance-brand-name = Назва продукту
admin-settings-help-appearance-brand-name = Назва, що показується на сторінці входу та у вкладці браузера.
admin-settings-key-login-notice = Повідомлення на сторінці входу
admin-settings-help-login-notice = Короткий текст на сторінці входу, наприклад контакт підтримки. Залиште порожнім, щоб його не було.
admin-settings-key-mail-read-receipts = Сповіщення про прочитання
admin-settings-key-receipts-dsn-allowed = Запити статусу доставки
admin-settings-help-receipts-dsn-allowed = Дозволяє обліковим записам запитувати сповіщення SMTP про статус доставки. Це не запитує й не доводить, що лист прочитано.
admin-settings-help-mail-read-receipts = Що відбувається, коли відкритий лист запитує сповіщення про прочитання: запитати користувача, ніколи не надсилати або завжди надсилати.
admin-settings-key-notifications-enabled = Сповіщення про нову пошту
admin-settings-help-notifications-enabled = Чи отримують користувачі сповіщення про нову пошту. Не змінює їхні правила сповіщень.
admin-settings-key-offline-retention-days = Вік офлайн-кешу
admin-settings-help-offline-retention-days = Скільки днів пристрій зберігає листи для роботи офлайн. 0 означає відсутність обмеження за віком.
admin-settings-key-offline-budget-mb = Розмір офлайн-кешу
admin-settings-help-offline-budget-mb = Скільки мегабайтів пристрій може займати під пошту офлайн. 0 означає відсутність офлайн-кешу.
admin-settings-key-assistant-enabled = ШІ-асистент
admin-settings-help-assistant-enabled = Чи бачать користувачі ШІ-асистента. Перемикач розгортання в розділі «ШІ-асистент» і далі діє вище за це налаштування.
admin-settings-key-assist-profile = Профіль постачальника ШІ
admin-settings-help-assist-profile = Ідентифікатор профілю постачальника, що обслуговує цих користувачів (ШІ-асистент > Профілі постачальників). Порожньо — використовується кінцева точка розгортання. Ідентифікатор неіснуючого профілю вимикає ШІ-асистента для цих користувачів.
admin-settings-key-upstream-default = Типовий поштовий сервер
admin-settings-help-upstream-default = Поштовий сервер, на який входить адреса, коли жоден домен не вказує іншого. Таблиця доменів має пріоритет над цим налаштуванням.
admin-settings-key-upstream-proxy-timeout-secs = Тайм-аут поштового сервера
admin-settings-help-upstream-proxy-timeout-secs = Скільки секунд може тривати один запит до поштового сервера.
admin-settings-key-signin-new-users = Нові користувачі
admin-settings-help-signin-new-users = Хто може створити обліковий запис під час першого входу: будь-хто (open), лише заздалегідь додані адреси (preprovisioned), вони й імпортовані користувачі (provisioned) або ніхто новий (closed). Наявних облікових записів це не стосується. Змінюється в політиці входу.
admin-settings-key-signin-new-users-mail = Вхід за паролем: нові користувачі
admin-settings-help-signin-new-users-mail = Правило для нового користувача поштової скриньки, що входить за паролем (IMAP, POP3, Exchange). Inherit використовує «Нові користувачі».
admin-settings-key-signin-new-users-sso = Єдиний вхід: нові користувачі
admin-settings-help-signin-new-users-sso = Правило для нового користувача, що входить через єдиний вхід або довірений заголовок. Inherit використовує «Нові користувачі».
admin-settings-key-signin-new-users-linked = Пов’язані скриньки: нові користувачі
admin-settings-help-signin-new-users-linked = Правило прив’язування скриньки, для якої тут ще немає облікового запису, за доменом самої скриньки. Inherit використовує «Нові користувачі».
admin-settings-key-signin-new-users-linked-imap = Пов’язані скриньки IMAP: нові користувачі
admin-settings-help-signin-new-users-linked-imap = Правило лише для пов’язаних скриньок IMAP. Inherit використовує «Пов’язані скриньки».
admin-settings-key-signin-new-users-linked-pop3 = Пов’язані скриньки POP3: нові користувачі
admin-settings-help-signin-new-users-linked-pop3 = Правило лише для пов’язаних скриньок POP3. Inherit використовує «Пов’язані скриньки».
admin-settings-key-signin-new-users-linked-ews = Пов’язані скриньки Exchange: нові користувачі
admin-settings-help-signin-new-users-linked-ews = Правило лише для пов’язаних скриньок Exchange (EWS). Inherit використовує «Пов’язані скриньки».
admin-settings-key-signin-preprovision-ttl-days = Строк дії заздалегідь доданого користувача (днів)
admin-settings-help-signin-preprovision-ttl-days = Як довго залишається чинним очікуваний користувач, доданий заздалегідь, якщо не задано іншого строку.
admin-settings-key-signin-preprovision-max-ttl-days = Найдовший строк заздалегідь доданого користувача (днів)
admin-settings-help-signin-preprovision-max-ttl-days = Найдовший строк, який адміністратор може задати очікуваному користувачу, доданому заздалегідь.
admin-settings-key-signin-preprovision-domain-wide = Дозволити записи для всього домену
admin-settings-help-signin-preprovision-domain-wide = Дозволяє записи, що допускають будь-кого з усього домену, до ліміту використань. Типово вимкнено.
admin-settings-key-signin-preprovision-retention-days = Зберігати прострочених очікуваних користувачів (днів)
admin-settings-help-signin-preprovision-retention-days = Як довго зберігається прострочений очікуваний користувач, перш ніж його буде видалено.
admin-settings-key-signin-domain-mode = Домени для входу
admin-settings-help-signin-domain-mode = Чи може входити будь-який домен, чи лише перелічені. Змінюється в політиці входу.
admin-settings-key-signin-refusal-detail = Повідомлення про відмову
admin-settings-help-signin-refusal-detail = Чи вказує відмова у вході причину (обліковий запис вимкнено, адресу не дозволено), чи дає ту саму відповідь, що й за неправильного пароля. Змінюється в політиці входу.
admin-settings-key-signin-ldap-bind-precheck = Перевірка LDAP перед входом
admin-settings-help-signin-ldap-bind-precheck = Перевіряти пароль у каталозі перед поштовим сервером. Змінюється в розділі «Інтеграції» після пробного прив’язування.
admin-settings-key-twofa-pending-ttl-secs = Ліміт часу другого фактора
admin-settings-help-twofa-pending-ttl-secs = Скільки секунд користувач має на введення другого фактора після пароля.
admin-settings-key-twofa-max-attempts = Спроби другого фактора
admin-settings-help-twofa-max-attempts = Скільки неправильних кодів другого фактора дозволено, перш ніж вхід почнеться заново.
admin-settings-key-admin-session-idle-secs = Тайм-аут бездіяльності консолі
admin-settings-help-admin-session-idle-secs = Скільки секунд сеанс консолі адміністратора триває без запитів.
admin-settings-key-admin-session-absolute-secs = Межа сеансу консолі
admin-settings-help-admin-session-absolute-secs = Найбільша тривалість сеансу консолі адміністратора в секундах, активного чи ні.
admin-settings-key-admin-session-rotate-secs = Зміна ідентифікатора сеансу консолі
admin-settings-help-admin-session-rotate-secs = Як часто, у секундах, консоль адміністратора замінює ідентифікатор сеансу під час роботи (0 = лише після підтвердження підвищення прав). Зміна ніколи не продовжує меж бездіяльності та абсолютної межі.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-cat-folders = Розкладка тек
admin-settings-key-passwd-policy-min-length = Мінімальна довжина пароля
admin-settings-help-passwd-policy-min-length = Найменша кількість символів у новому паролі.
admin-settings-key-passwd-policy-require-upper = Вимагати велику літеру
admin-settings-help-passwd-policy-require-upper = Новий пароль має містити велику літеру.
admin-settings-key-passwd-policy-require-lower = Вимагати малу літеру
admin-settings-help-passwd-policy-require-lower = Новий пароль має містити малу літеру.
admin-settings-key-passwd-policy-require-digit = Вимагати цифру
admin-settings-help-passwd-policy-require-digit = Новий пароль має містити цифру.
admin-settings-key-passwd-policy-require-symbol = Вимагати символ
admin-settings-help-passwd-policy-require-symbol = Новий пароль має містити символ, що не є літерою чи цифрою.
admin-settings-key-limit-storage-bytes = Ліміт сховища
admin-settings-help-limit-storage-bytes = Найбільша кількість байтів, яку один користувач може зберігати на цьому сервері. 0 означає без обмежень.
admin-settings-key-limit-send-per-day = Надісланих листів на день
admin-settings-help-limit-send-per-day = Найбільша кількість листів, яку один користувач може надіслати за день. 0 означає без обмежень.
admin-settings-key-limit-ai-calls-per-day = Звернень до ШІ-асистента на день
admin-settings-help-limit-ai-calls-per-day = Найбільша кількість запитів до ШІ-асистента, яку один користувач може зробити за день. 0 означає без обмежень.
admin-settings-key-quota-warn-thresholds = Рівні попередження про використання
admin-settings-help-quota-warn-thresholds = Відсотки ліміту, за яких користувачі отримують попередження, від найменшого.
admin-settings-key-quota-warn-enabled = Попередження про використання
admin-settings-help-quota-warn-enabled = Чи попереджати користувачів, коли вони наближаються до ліміту.
admin-settings-key-usage-retention-days = Історія використання
admin-settings-help-usage-retention-days = Скільки днів зберігаються щоденні лічильники використання.
admin-settings-key-usage-meter-api = Враховувати використання API
admin-settings-help-usage-meter-api = Чи враховуються запити API в показниках використання.
admin-settings-key-usage-measure-interval-hours = Інтервал вимірювання сховища
admin-settings-help-usage-measure-interval-hours = Як часто, у годинах, вимірюється обсяг даних кожного користувача.
admin-settings-key-assistant-rate-limit-per-min = Обмеження частоти запитів до ШІ-асистента
admin-settings-help-assistant-rate-limit-per-min = Найбільша кількість запитів до ШІ-асистента за хвилину для всього сервера. 0 означає без обмежень.
admin-settings-key-plugins-kv-max-key-bytes = Розмір ключа плагіна
admin-settings-help-plugins-kv-max-key-bytes = Найдовший ключ, який плагін може зберегти, у байтах.
admin-settings-key-plugins-kv-max-value-bytes = Розмір значення плагіна
admin-settings-help-plugins-kv-max-value-bytes = Найбільше значення, яке плагін може зберегти під одним ключем, у байтах.
admin-settings-key-plugins-kv-max-total-bytes = Сховище плагіна
admin-settings-help-plugins-kv-max-total-bytes = Найбільша загальна кількість байтів, яку може зберегти один плагін.
admin-settings-key-plugins-kv-max-keys = Ключі плагіна
admin-settings-help-plugins-kv-max-keys = Найбільша кількість ключів, яку може зберегти один плагін.
admin-settings-key-import-max-messages = Кількість листів в імпорті
admin-settings-help-import-max-messages = Найбільша кількість листів, яку може принести один імпорт.
admin-settings-key-import-max-message-bytes = Розмір листа в імпорті
admin-settings-help-import-max-message-bytes = Найбільший лист, який приймає імпорт, у байтах.
admin-settings-key-webhooks-max-attempts = Спроби вебхука
admin-settings-help-webhooks-max-attempts = Скільки разів намагаються доставити вебхук, перш ніж відмовитися.
admin-settings-key-webhooks-timeout-secs = Тайм-аут вебхука
admin-settings-help-webhooks-timeout-secs = Скільки секунд може тривати одна доставка вебхука.
admin-settings-key-outbox-max-attempts = Спроби надсилання
admin-settings-help-outbox-max-attempts = Скільки разів лист передається поштовому серверу, перш ніж буде повідомлено, що його не надіслано.
admin-settings-key-realtime-recheck-secs = Повторна перевірка живих оновлень
admin-settings-help-realtime-recheck-secs = Як часто, у секундах, відкрите з’єднання живих оновлень перевіряє, що його сеанс досі чинний.
admin-settings-key-privacy-watermark-enabled = Водяний знак читача
admin-settings-help-privacy-watermark-enabled = Показувати водяний знак з адресою читача поверх відкритих листів.
admin-settings-key-privacy-watermark-opacity = Інтенсивність водяного знака
admin-settings-help-privacy-watermark-opacity = Наскільки помітний водяний знак, від 0,02 (ледь помітний) до 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Історія відстеження відкриттів
admin-settings-help-privacy-open-tracking-retention-days = Скільки днів зберігаються записи відстеження прочитання.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Токени відстеження ненадісланих листів
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Скільки годин токен відстеження чекає на надсилання листа, перш ніж його буде видалено.
admin-settings-key-masked-email-domain = Домен прихованих адрес
admin-settings-help-masked-email-domain = Домен, у якому створюються приховані адреси. Залиште порожнім, щоб вимкнути приховані адреси.
admin-settings-key-abuse-report-address = Адреса для скарг на зловживання
admin-settings-help-abuse-report-address = Куди надсилаються скарги на спам і зловживання.
admin-settings-key-dlp-admin-address = Сповіщення про витік даних
admin-settings-help-dlp-admin-address = Куди надсилаються сповіщення системи запобігання витоку даних.
admin-settings-key-observability-error-forward-url = Адреса пересилання помилок
admin-settings-help-observability-error-forward-url = Адреса https, на яку пересилаються звіти про помилки клієнтів. Залиште порожнім, щоб не пересилати.
admin-settings-key-egress-probe-url = Адреса перевірки вихідних з’єднань
admin-settings-help-egress-probe-url = Адреса https, що використовується для перевірки вихідного проксі.
admin-settings-key-push-vapid-contact = Контакт для push
admin-settings-help-push-vapid-contact = Контакт (mailto: або https:), що передається push-службам з кожним сповіщенням.
admin-settings-key-push-quiet-hours = Тихі години push
admin-settings-help-push-quiet-hours = Години (UTC), у які push-сповіщення не надсилаються. Залиште порожнім, щоб таких годин не було.
admin-settings-key-updates-check = Перевіряти оновлення
admin-settings-help-updates-check = Чи перевіряє сервер раз на добу наявність нового випуску та чи показує його на панелі.
admin-settings-key-updates-feed-url = Стрічка випусків
admin-settings-help-updates-feed-url = Адреса https, яку читає перевірка оновлень.
admin-settings-key-updates-apply = Застосовувати оновлення
admin-settings-help-updates-apply = Чи можна запустити оновлення з консолі через агент оновлення хоста.
admin-settings-key-backup-retention-count = Збережені резервні копії
admin-settings-help-backup-retention-count = Скільки резервних копій бази даних зберігається.
admin-settings-key-about-security-contact = Контакт із безпеки
admin-settings-help-about-security-contact = Контакт у файлі security.txt (mailto: або https:).
admin-settings-key-about-security-policy-url = Політика безпеки
admin-settings-help-about-security-policy-url = Адреса вашої політики безпеки, указана в security.txt.
admin-settings-key-about-project-links = Посилання розділу «Про програму»
admin-settings-help-about-project-links = Посилання, що показуються на сторінці «Про програму», кожне з підписом і адресою https.
admin-settings-key-config-snapshots-kept = Знімки конфігурації
admin-settings-help-config-snapshots-kept = Скільки знімків конфігурації зберігається.
admin-settings-key-jobs-history-days = Історія завдань
admin-settings-help-jobs-history-days = Скільки днів запусків завдань зберігається.
admin-settings-key-maintenance-announce-minutes-default = Час попередження про обслуговування
admin-settings-help-maintenance-announce-minutes-default = За скільки хвилин до вікна обслуговування користувачів повідомляють про нього.
admin-settings-key-maintenance-default-message = Повідомлення про обслуговування
admin-settings-help-maintenance-default-message = Текст, який користувачі бачать під час обслуговування, окремо для кожної мови.
admin-settings-key-health-disk-free-min-percent = Мінімум вільного місця на диску
admin-settings-help-health-disk-free-min-percent = Перевірка готовності не проходить, коли вільне місце на диску опускається нижче цього відсотка.
admin-settings-key-observability-otlp-metrics = Надсилати метрики через OTLP
admin-settings-help-observability-otlp-metrics = Надсилати метрики сервера до колектора OTLP, заданого під час запуску.
admin-settings-key-metrics-drilldown-top-n = Рядки деталізації метрик
admin-settings-help-metrics-drilldown-top-n = Скільки рядків показує розбивка метрик.
admin-settings-key-metrics-rollup-days = Історія метрик
admin-settings-help-metrics-rollup-days = Скільки днів погодинних зведень метрик зберігається.
admin-settings-key-observability-client-timing-enabled = Час завантаження в браузері
admin-settings-help-observability-client-timing-enabled = Збирати час завантаження сторінок із браузерів користувачів, що увійшли, у вигляді анонімних зведених гістограм. Ніщо не ідентифікує людину, сеанс чи сторінку. Вимкніть, щоб зупинити збирання.
admin-settings-key-observability-client-timing-sample-rate-percent = Частка вибірки часу завантаження в браузері
admin-settings-help-observability-client-timing-sample-rate-percent = Відсоток сеансів браузера, що повідомляють час завантаження (від 1 до 100).
admin-settings-key-observability-client-timing-min-samples = Мінімум вибірок часу завантаження
admin-settings-help-observability-client-timing-min-samples = Медіана та 95-й процентиль часу завантаження приховані, доки в періоді не набереться принаймні стільки вибірок.
admin-settings-key-observability-client-timing-folder-loads = Час завантаження тек
admin-settings-help-observability-client-timing-folder-loads = Вимірює, скільки часу теці потрібно, щоб показати список листів, із розбивкою на дані з кешу браузера та з мережі. Типово ввімкнено. Користувачі можуть вимкнути це для себе, якщо налаштування не заблоковано. Потребує ввімкненого часу завантаження в браузері.
admin-settings-key-observability-client-timing-message-loads = Час завантаження листів
admin-settings-help-observability-client-timing-message-loads = Вимірює, скільки часу листу потрібно, щоб показати заголовки та очищений текст, із розбивкою на дані з кешу чи попередньо завантажені та з мережі. Типово ввімкнено. Користувачі можуть вимкнути це для себе, якщо налаштування не заблоковано. Потребує ввімкненого часу завантаження в браузері.
admin-settings-key-diagnostics-retention-days = Історія діагностики
admin-settings-help-diagnostics-retention-days = Скільки днів діагностичних запусків зберігається.
admin-settings-key-data-export-link-hours = Вікно завантаження експорту даних
admin-settings-help-data-export-link-hours = Скільки годин експорт даних можна завантажувати, перш ніж його буде видалено.
admin-settings-key-dsr-enabled = Запити щодо даних
admin-settings-help-dsr-enabled = Чи можуть люди подавати запити щодо даних (доступ, завантаження, виправлення, видалення, обмеження, заперечення) з налаштувань.
admin-settings-key-dsr-type-access = Приймати запити на доступ
admin-settings-help-dsr-type-access = Чи пропонується користувачам тип запиту на доступ.
admin-settings-key-dsr-type-portability = Приймати запити на завантаження (переносну копію)
admin-settings-help-dsr-type-portability = Чи пропонується користувачам тип запиту на завантаження (переносну копію).
admin-settings-key-dsr-type-rectification = Приймати запити на виправлення
admin-settings-help-dsr-type-rectification = Чи пропонується користувачам тип запиту на виправлення.
admin-settings-key-dsr-type-erasure = Приймати запити на видалення
admin-settings-help-dsr-type-erasure = Чи пропонується користувачам тип запиту на видалення.
admin-settings-key-dsr-type-restriction = Приймати запити на обмеження
admin-settings-help-dsr-type-restriction = Чи пропонується користувачам тип запиту на обмеження.
admin-settings-key-dsr-type-objection = Приймати заперечення
admin-settings-help-dsr-type-objection = Чи пропонується користувачам тип запиту-заперечення.
admin-settings-key-dsr-type-other = Приймати інші запити
admin-settings-help-dsr-type-other = Чи пропонується користувачам загальний тип запиту.
admin-settings-key-dsr-approval-export = Схвалення запитів на доступ і завантаження
admin-settings-help-dsr-approval-export = auto запускає експорт після підтвердження запиту. admin чекає на схвалення адміністратора.
admin-settings-key-dsr-approval-erasure = Схвалення запитів на видалення
admin-settings-help-dsr-approval-erasure = auto планує видалення після підтвердження запиту. admin чекає на схвалення адміністратора.
admin-settings-key-dsr-approval-other = Обробка інших запитів
admin-settings-help-dsr-approval-other = auto позначає запит як виконуваний після підтвердження. admin чекає на схвалення адміністратора.
admin-settings-key-dsr-reauth-required = Повторна автентифікація для запитів
admin-settings-help-dsr-reauth-required = Вимагати свіжий пароль або другий фактор для подання запиту на експорт чи видалення та для створення посилання на завантаження.
admin-settings-key-dsr-confirmation-required = Підтвердне повідомлення
admin-settings-help-dsr-confirmation-required = Надсилати до вхідних заявника підтвердне повідомлення з кодом; запит не продовжується, доки код не введено.
admin-settings-key-dsr-identity-verification-required = Крок перевірки особи
admin-settings-help-dsr-identity-verification-required = Адміністратор має записати, що особу заявника перевірено, перш ніж запит буде схвалено.
admin-settings-key-dsr-erasure-grace-days = Пільговий період видалення
admin-settings-help-dsr-erasure-grace-days = Кількість днів між схваленням і видаленням. Упродовж цього часу заявник може скасувати запит. 0 виконує видалення під час наступного проходу робочого процесу.
admin-settings-key-dsr-sla-days = Днів на відповідь на запит
admin-settings-help-dsr-sla-days = Кількість днів від подання до строку, що показується користувачам і адміністраторам.
admin-settings-key-dsr-sla-max-extension-days = Найбільше продовження
admin-settings-help-dsr-sla-max-extension-days = Найбільша кількість днів, яку адміністратор може додати до строку запиту із записаною причиною.
admin-settings-key-dsr-sla-reminder-days = Позначки наближення строку
admin-settings-help-dsr-sla-reminder-days = За скільки днів до строку запит позначається в черзі як терміновий.
admin-settings-key-dsr-export-mail-allowed = Пошта в експортах
admin-settings-help-dsr-export-mail-allowed = Дозволяє заявнику включати листи до експорту. Включаються лише листи з власного кешу цього сервера; пошта, що залишається на вищому поштовому сервері, не включається.
admin-settings-key-dsr-export-mail-format = Формат пошти в експортах
admin-settings-help-dsr-export-mail-format = mbox записує один файл на теку; eml записує один файл на лист.
admin-settings-key-dsr-export-include-settings = Експорт: налаштування
admin-settings-help-dsr-export-include-settings = Включити налаштування до експорту.
admin-settings-key-dsr-export-include-identities = Експорт: особи
admin-settings-help-dsr-export-include-identities = Включити особи до експорту.
admin-settings-key-dsr-export-include-contacts = Експорт: контакти
admin-settings-help-dsr-export-include-contacts = Включити контакти до експорту.
admin-settings-key-dsr-export-include-calendars = Експорт: календарі
admin-settings-help-dsr-export-include-calendars = Включити календарі до експорту.
admin-settings-key-dsr-export-include-notes = Експорт: нотатки
admin-settings-help-dsr-export-include-notes = Включити нотатки до експорту.
admin-settings-key-dsr-export-include-rules = Експорт: фільтри та правила
admin-settings-help-dsr-export-include-rules = Включити фільтри та правила до експорту.
admin-settings-key-dsr-export-include-sessions = Експорт: сеанси, пристрої та історія входів
admin-settings-help-dsr-export-include-sessions = Включити сеанси, пристрої та історію входів до експорту.
admin-settings-key-dsr-export-include-consent = Експорт: записи згод
admin-settings-help-dsr-export-include-consent = Включити записи згод до експорту.
admin-settings-key-dsr-export-include-audit = Експорт: записи аудиту про людину
admin-settings-help-dsr-export-include-audit = Включити до експорту записи аудиту про цю людину.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-dsr-export-max-megabytes = Межа розміру експорту
admin-settings-help-dsr-export-max-megabytes = Найбільший експорт у мегабайтах. Розділи, що його перевищили б, пропускаються й перелічуються в маніфесті.
admin-settings-key-dsr-export-format = Формати експорту
admin-settings-help-dsr-export-format = json записує файли JSON. jsonCsv додатково записує файли CSV для даних у вигляді списків.
admin-settings-key-dsr-export-encrypt = Шифрувати експорти під час зберігання
admin-settings-help-dsr-export-encrypt = Зберігати створені експорти зашифрованими ключем сервера, доки їх не завантажать або не мине строк.
admin-settings-key-dsr-download-link-minutes = Строк дії посилання для завантаження
admin-settings-help-dsr-download-link-minutes = Скільки хвилин посилання для завантаження працює після створення. Сам експорт зберігається впродовж вікна завантаження експорту даних.
admin-settings-key-dsr-download-single-use = Одноразове посилання для завантаження
admin-settings-help-dsr-download-single-use = Посилання для завантаження перестає працювати після одноразового використання.
admin-settings-key-dsr-retention-completed-days = Зберігання завершених запитів
admin-settings-help-dsr-retention-completed-days = Скільки днів завершений запит і його слід зберігаються до видалення.
admin-settings-key-dsr-notify-admins = Записувати нові запити для адміністраторів
admin-settings-help-dsr-notify-admins = Записувати кожен новий запит як подію безпеки в журналі аудиту, де його бачать адміністратори та приймачі сповіщень.
admin-settings-key-dsr-notify-requester = Повідомлення про перебіг заявнику
admin-settings-help-dsr-notify-requester = Класти повідомлення до вхідних заявника, коли стан запиту змінюється.
admin-settings-key-dsr-contact-name = Ім’я контакту із захисту даних
admin-settings-help-dsr-contact-name = Ім’я, що показується користувачам на сторінці запитів щодо даних.
admin-settings-key-dsr-contact-email = Адреса контакту із захисту даних
admin-settings-help-dsr-contact-email = Адреса електронної пошти, що показується користувачам на сторінці запитів щодо даних.
admin-settings-key-dsr-contact-url = Сторінка контакту із захисту даних
admin-settings-help-dsr-contact-url = Посилання, що показується користувачам на сторінці запитів щодо даних.
admin-settings-key-dsr-on-behalf-allowed = Запити від імені користувача
admin-settings-help-dsr-on-behalf-allowed = Дозволяє адміністратору подати запит за користувача, наприклад отриманий поштою.
admin-settings-key-dsr-rate-limit-per-day = Запитів на людину за день
admin-settings-help-dsr-rate-limit-per-day = Найбільша кількість запитів, яку одна людина може подати за 24 години.
admin-settings-key-dsr-legal-hold-subjects = Юридичні блокування
admin-settings-help-dsr-legal-hold-subjects = Адреси або записи @домен, видалення яких заблоковано. Запит на видалення для них залишається в черзі заблокованим, доки запис не буде видалено.
admin-settings-key-scrub-enabled = Очищення даних людини
admin-settings-help-scrub-enabled = Дозволяє дію «Очистити дані людини» та очищення, що може йти після видалення. Якщо вимкнено, жодне з них не виконується.
admin-settings-key-scrub-pseudonym-algorithm = Довжина псевдоніма
admin-settings-help-scrub-pseudonym-algorithm = Як створюється ключовий псевдонім, що замінює ідентифікатор: HMAC-SHA256 за поточного покоління ключа, обрізаний до 16 або 32 шістнадцяткових символів.
admin-settings-key-scrub-include-logs = Очищати файли журналів
admin-settings-help-scrub-include-logs = Очищення також переписує ідентифікатор у ротованих файлах журналів, зокрема стиснених.
admin-settings-key-scrub-auto-run-on-erasure = Очищення після видалення
admin-settings-help-scrub-auto-run-on-erasure = Завершене видалення також замінює ідентифікатори людини в історії аудиту та журналах псевдонімом. Типово вимкнено.
admin-settings-key-scrub-key-retention-days = Зберігання ключа псевдонімів
admin-settings-help-scrub-key-retention-days = Скільки днів ключ псевдонімів зберігається після виведення з ротації. 0 зберігає його безстроково. Після видалення старі псевдоніми більше не можна перерахувати, а створені цим ключем позначки видалення не можна відтворити.
admin-settings-key-backup-propagate-erasure = Відтворювати видалення під час відновлення
admin-settings-help-backup-propagate-erasure = Відновлення відтворює позначки видалення новіші за резервну копію, тож люди, видалені відтоді, видаляються знову.
admin-settings-key-backup-tombstone-retention-days = Зберігання позначок видалення
admin-settings-help-backup-tombstone-retention-days = Скільки днів зберігається позначка видалення. 0 зберігає їх вічно.
admin-settings-key-dsr-text-intro = Вступ до запитів щодо даних
admin-settings-help-dsr-text-intro = Текст над формою запиту. Одне значення на мову.
admin-settings-key-dsr-text-confirmation = Текст підтвердного повідомлення
admin-settings-help-dsr-text-confirmation = Текст, що додається до підтвердного повідомлення. Одне значення на мову.
admin-settings-key-dsr-text-completion = Текст повідомлення про завершення
admin-settings-help-dsr-text-completion = Текст, що додається до повідомлення, яке надсилається після завершення запиту. Одне значення на мову.
admin-settings-key-server-text-family = Тексти сервера
admin-settings-help-server-text-family = Текст, який сервер вписує в пошту та сповіщення, окремо для мови й домену.
admin-settings-cat-sieve = Поштові фільтри
admin-settings-key-sieve-max-script-bytes = Найбільший розмір скрипта фільтра
admin-settings-help-sieve-max-script-bytes = Найбільша кількість байтів в одному скрипті фільтра Sieve.
admin-settings-key-sieve-max-scripts = Найбільша кількість скриптів фільтрів
admin-settings-help-sieve-max-scripts = Найбільша кількість збережених скриптів на обліковий запис.
admin-settings-key-sieve-max-redirects = Найбільша кількість переспрямувань
admin-settings-help-sieve-max-redirects = Найбільша кількість команд переспрямування в одному скрипті.
admin-settings-key-sieve-vacation-allowed = Дії автовідповіді у фільтрах
admin-settings-help-sieve-vacation-allowed = Дозволяє скриптам надсилати автовідповіді, якщо їх підтримує бекенд.
admin-settings-key-sieve-allowed-extensions = Дозволені розширення фільтрів
admin-settings-help-sieve-allowed-extensions = Необов’язковий список назв розширень Sieve малими літерами. Бекенд також має підтримувати кожне розширення.
admin-settings-key-cap-sieve-raw-edit = Редагування коду скрипта фільтра
admin-settings-help-cap-sieve-raw-edit = Дозволяє редагування вихідних скриптів Sieve для вибраних облікових записів.
admin-settings-key-cap-sieve-forward-external = Пересилання за межі домену облікового запису
admin-settings-help-cap-sieve-forward-external = Дозволяє зовнішнє пересилання в скриптах. Для збереження таких скриптів також потрібна недавня перевірка.
admin-settings-key-session-max-concurrent = Одночасні сеанси
admin-settings-help-session-max-concurrent = Найбільша кількість сеансів на обліковий запис. Найстаріші сеанси відкликаються, коли новий вхід перевищує цей ліміт.
admin-settings-key-session-idle-secs = Тайм-аут бездіяльності сеансу
admin-settings-help-session-idle-secs = Завершувати неактивні сеанси через стільки секунд, але не більше за межу розгортання.
admin-settings-key-session-absolute-secs = Строк життя сеансу
admin-settings-help-session-absolute-secs = Завершувати сеанси через стільки секунд, але не більше за межу розгортання.
admin-settings-key-twofa-remember-days = Строк запам’ятовування пристрою
admin-settings-help-twofa-remember-days = Через скільки днів запам’ятований пристрій має знову підтвердити другий фактор. Нуль вимикає цю функцію.
admin-settings-key-twofa-passkeys-enabled = Дозволити ключі доступу
admin-settings-help-twofa-passkeys-enabled = Дозволяє ключі доступу як фактор автентифікації.
admin-settings-key-twofa-required = Вимагати двофакторну автентифікацію
admin-settings-help-twofa-required = Вимагає зареєстрований другий фактор перед видачею сеансу.
admin-settings-key-twofa-passkey-attachment = Тип автентифікатора ключа доступу
admin-settings-help-twofa-passkey-attachment = any (типово): будь-який автентифікатор. platform: ключі доступу, що зберігаються на пристрої (Windows Hello, Touch ID, блокування екрана Android). cross-platform: ключі безпеки та телефони. Це повідомляє браузер; автентифікатор цього не підписує.
admin-settings-key-twofa-passkey-user-verification = Перевірка користувача ключем доступу
admin-settings-help-twofa-passkey-user-verification = required: ключ доступу має перевіряти PIN або біометрію під час додавання та кожного входу; процедура без цього відхиляється. preferred (типово) і discouraged змінюють лише те, що запитується в браузера.
admin-settings-key-twofa-passkey-resident-key = Виявні ключі доступу
admin-settings-help-twofa-passkey-resident-key = required: облікові дані мають зберігатися на автентифікаторі. discouraged (типово) або preferred. У режимі рушія ключі доступу завжди виявні, бо за їх допомогою можна входити й без пароля.
admin-settings-key-twofa-passkey-legacy = Ключі доступу, додані до чинної політики
admin-settings-help-twofa-passkey-legacy = allow (типово): старіший ключ доступу, що не відповідає політиці, і далі працює, а користувача просять його замінити. refuse: його відхиляють під час входу з повідомленням; користувач входить іншим фактором або кодом відновлення.
admin-settings-key-twofa-last-factor-removal = Видалення останнього другого фактора
admin-settings-help-twofa-last-factor-removal = Коли видалення ключа доступу або застосунку-автентифікатора залишило б обліковий запис без жодного. allow (типово): його видаляють, і для входу потрібен лише пароль. block: користувач спершу має додати інший спосіб входу (коди відновлення не враховуються). Там, де двофакторна автентифікація обов’язкова, останній фактор завжди зберігається.
admin-settings-key-privacy-remote-content = Віддалений вміст
admin-settings-help-privacy-remote-content = Блокувати зображення, спершу питати, використовувати проксі або завантажувати напряму, коли це дозволено.
admin-settings-key-privacy-trackers = Блокувати трекери
admin-settings-help-privacy-trackers = Блокує відомі зображення відстеження в листах.
admin-settings-key-privacy-clean-links = Очищати посилання
admin-settings-help-privacy-clean-links = Видаляє параметри відстеження з посилань у листах.
admin-settings-key-privacy-max-bytes = Ліміт віддаленого зображення
admin-settings-help-privacy-max-bytes = Найбільша кількість байтів, що завантажуються для одного віддаленого зображення.
admin-settings-key-privacy-cache-bytes = Кеш віддалених зображень
admin-settings-help-privacy-cache-bytes = Найбільша кількість байтів, що зберігаються в кеші віддалених зображень.
admin-settings-key-privacy-rate-per-min = Частота запитів віддалених зображень
admin-settings-help-privacy-rate-per-min = Найбільша кількість завантажень віддалених зображень на обліковий запис за хвилину.
admin-settings-privacy-fetch-title = Завантаження віддалених зображень
admin-settings-privacy-fetch-help = Обмеження за часом, переспрямування та паралельні завантаження зображень, які сервер завантажує для читача. Зображення, що перевищило ліміт, показується заглушкою.
admin-settings-key-privacy-fetch-connect-timeout-ms = Тайм-аут з’єднання з віддаленим зображенням
admin-settings-help-privacy-fetch-connect-timeout-ms = Мілісекунди, відведені на відкриття з’єднання з хостом зображення, зокрема TLS-рукостискання. Типово 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Тайм-аут відповіді віддаленого зображення
admin-settings-help-privacy-fetch-headers-timeout-ms = Мілісекунди від початку запиту до відповіді хоста зображення заголовками. Типово 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Тайм-аут зупинки віддаленого зображення
admin-settings-help-privacy-fetch-idle-timeout-ms = Найбільша пауза в мілісекундах між двома частинами завантаження зображення, після якої завантаження припиняється. Типово 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Загальний тайм-аут віддаленого зображення
admin-settings-help-privacy-fetch-total-timeout-ms = Мілісекунди, відведені на завантаження зображення, зокрема переспрямування. Очікування в черзі має окремий бюджет, тож зайняте зображення не витрачає мережевий час до початку. Типово 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Тайм-аут черги віддалених зображень
admin-settings-help-privacy-fetch-queue-timeout-ms = Найбільша кількість мілісекунд, яку зображення можуть чекати на звільнення ресурсів понад бюджет завантаження. Обмежена черга витримує великі сплески автентифікованих запитів, а ліміти активних завантажень і декодування й далі діють. Типово 60000.
admin-settings-key-privacy-max-redirects = Переспрямування віддалених зображень
admin-settings-help-privacy-max-redirects = Скільки переспрямувань виконується для одного зображення. Кожне перевіряється за тими самими правилами адрес, що й перший запит. Типово 3.
admin-settings-key-privacy-max-concurrent-per-message = Завантаження віддалених зображень на лист
admin-settings-help-privacy-max-concurrent-per-message = Скільки зображень одного листа завантажується одночасно. Решта чекають на вільне місце. Діапазон 1–128; типово 30.
admin-settings-key-privacy-max-concurrent-per-account = Завантаження віддалених зображень на обліковий запис
admin-settings-help-privacy-max-concurrent-per-account = Скільки зображень завантажується одночасно для одного облікового запису з усіх його відкритих листів. Діапазон 1–256; типово 128.
admin-settings-key-privacy-allow-direct = Дозволити прямі зображення
admin-settings-key-cap-remote-content-direct = Дозвіл на прямі зображення
admin-settings-help-cap-remote-content-direct = Дозволяє цьому обліковому запису вибирати прямі зображення, якщо сервер це допускає.
admin-settings-help-privacy-allow-direct = Дозволяє пряме завантаження зображень для облікових записів із потрібним дозволом.
admin-settings-key-logs-auth-retention-days = Зберігання історії автентифікації
admin-settings-help-logs-auth-retention-days = Скільки днів зберігаються метадані автентифікації незалежно від постійної ідентичності облікового запису.
admin-settings-key-logs-auth-max-entries = Межа записів історії автентифікації
admin-settings-help-logs-auth-max-entries = Найбільша кількість записів, що зберігаються. Збіжні повторні невдачі в межах однієї хвилини ділять один запис.
admin-settings-key-reader-color-mode = Кольори листів
admin-settings-cat-reader = Показ листів
admin-settings-key-mail-row-actions-mode = Швидкі дії у списку листів
admin-settings-help-mail-row-actions-mode = Як виглядають швидкі дії в кожному рядку листа: open (завжди доступні, показуються під час наведення чи фокуса), hover (три елементи й кнопка «Ще»), click (одна кнопка, що їх відкриває) або off (прибрані з рядків; контекстне меню та панель інструментів зберігають усі дії). На сенсорних екранах hover працює як click. Користувачі можуть обрати своє, якщо адміністратор не заблокує.
admin-settings-key-mail-quick-reply = Швидка відповідь
admin-settings-help-mail-quick-reply = Пропонує компактну вкладку відповіді з уже заповненими одержувачами. Типово вимкнено; користувачі можуть увімкнути, якщо адміністратор не заблокував. Звичайні захисти надсилання й далі діють.
admin-settings-key-mail-request-read-receipt = Запитувати сповіщення про прочитання типово
admin-settings-help-mail-request-read-receipt = Починати підтримувані вихідні листи з вибраним запитом сповіщення. Це типове значення під час створення, а не гарантія, що одержувач надішле сповіщення.
admin-settings-help-reader-color-mode = Автоматично наслідує темні теми. Оберіть початкове, адаптивне або інвертоване відтворення всередині ізольованого фрейму листа.
admin-settings-key-reader-font-size = Розмір шрифту листа
admin-settings-help-reader-font-size = Типовий розмір читання в пікселях, від 12 до 24.
admin-settings-key-reader-line-height = Висота рядка листа
admin-settings-help-reader-line-height = Міжрядковий інтервал під час читання від 1,2 до 2,2.
admin-settings-key-reader-content-width = Ширина читання листа
admin-settings-help-reader-content-width = Вміст листа комфортної, широкої або на всю панель ширини.
admin-settings-key-reader-loading = Вигляд завантаження листа
admin-settings-help-reader-loading = Рядки-заповнювачі, текстовий індикатор або без візуального індикатора. Оголошення для доступності залишаються.
admin-settings-key-reader-loading-delay-ms = Затримка завантаження листа
admin-settings-help-reader-loading-delay-ms = Затримує лише індикатор завантаження, від 0 до 1000 мілісекунд. Доставка листа ніколи не затримується.
admin-settings-key-reader-zoom = Масштаб листа
admin-settings-help-reader-zoom = Типовий масштаб тексту листа у відсотках, від 50 до 200; засіб читання використовує найближчий крок масштабу. Масштабується лише лист, а друк використовує 100%.
admin-settings-key-reader-attachments-collapsed = Панель вкладень згорнуто
admin-settings-help-reader-attachments-collapsed = Чи починається панель вкладень засобу читання згорнутою, показуючи лише кількість і загальний розмір. Останнє відкриття або закриття користувача зберігається тут. Типово вимкнено: вкладення показано.
admin-settings-key-login-show-domain-hint = Показувати бажаний домен під час входу
admin-settings-help-login-show-domain-hint = Показує, який домен приймає ім’я користувача без суфікса електронної пошти. Типово приховано. Змінюється лише підказка; вхід за іменем користувача й далі працює.
admin-settings-key-signin-retry-cooldown-secs = Пауза перед повторною спробою входу
admin-settings-help-signin-retry-cooldown-secs = Мінімум секунд між спробами входу за паролем з однієї клієнтської мережі. Типово 3; 0 вимикає паузу. Наявні блокування після невдач і далі діють.
admin-settings-key-mail-quote-style = Стиль заголовка відповіді та пересилання
admin-settings-key-mail-external-recipient-warning = Попередження про зовнішнього одержувача
admin-settings-help-mail-external-recipient-warning = Вимкнено, попередження або вимога підтвердження у вікні створення. Охоплює «Кому», «Копія» та «Прихована копія». Це захист вікна створення, а не обмеження доставки на сервері.
admin-settings-key-mail-trusted-recipient-domains = Довірені домени одержувачів
admin-settings-help-mail-trusted-recipient-domains = До 100 точних доменів одержувачів. Домен вибраної особи відправника також вважається довіреним. Недійсні записи ніколи не дають довіри. Заблокуйте це налаштування, щоб користувачі не могли його перевизначати.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-quote-style = Використовувати класичні відповіді з цитуванням або заголовки листів у стилі Outlook. Це задає типове значення для користувача; заблокуйте його, щоб примусово застосувати стиль. Наявні чернетки не змінюються.
admin-settings-key-signin-anti-bot-mode = Захист входу від ботів
admin-settings-help-signin-anti-bot-mode = Власна перевірка proof-of-work: off, mailbox, admin або all. Зовнішня служба перевірки не використовується.
admin-settings-key-signin-anti-bot-difficulty = Обсяг роботи перевірки входу
admin-settings-help-signin-anti-bot-difficulty = Найбільший лічильник розв’язання, 100–20000. Вищі значення потребують більше роботи браузера; типово 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Строк дії перевірки входу
admin-settings-help-signin-anti-bot-expiry-secs = Скільки секунд лишається чинною одноразова перевірка, 60–600; типово 180. Після кожної спроби введення облікових даних потрібна нова перевірка.
admin-settings-key-attachments-preview-allowed = Дозволити попередній перегляд вкладень
admin-settings-help-attachments-preview-allowed = Вимкніть, щоб заборонити відтворення попереднього перегляду вкладень; завантаження оригіналів має власне налаштування.
admin-settings-key-attachments-download-allowed = Дозволити видачу оригіналів вкладень
admin-settings-help-attachments-download-allowed = Вимкнення також забороняє попередній перегляд, якому потрібні вихідні байти. Попередній перегляд у браузері не є захистом від копіювання.
admin-settings-key-attachments-max-bytes = Найбільший розмір вкладення в байтах
admin-settings-help-attachments-max-bytes = Нуль означає без обмежень. Застосовується до виданих вихідних байтів, зокрема запитів діапазонів.
admin-settings-key-attachments-allowed-extensions = Дозволені розширення назв файлів
admin-settings-help-attachments-allowed-extensions = Масив JSON розширень малими літерами без крапки, наприклад ["pdf","tar.gz"]. Порожньо дозволяє будь-яке розширення; блокувальні правила мають пріоритет.
admin-settings-key-attachments-blocked-extensions = Заблоковані розширення назв файлів
admin-settings-help-attachments-blocked-extensions = Масив JSON розширень малими літерами. Перевіряються за збереженими назвами файлів MIME, а не за назвою в адресі завантаження.
admin-settings-key-attachments-warn-extensions = Попереджати про розширення назв файлів
admin-settings-help-attachments-warn-extensions = Масив JSON розширень, що потребують явного підтвердження перед видачею оригіналу.
admin-settings-key-attachments-allowed-mime-types = Дозволені типи MIME
admin-settings-help-attachments-allowed-mime-types = Масив JSON типів малими літерами або підстановок верхнього рівня, наприклад ["image/*","application/pdf"]. Порожньо дозволяє будь-який тип.
admin-settings-key-attachments-blocked-mime-types = Заблоковані типи MIME
admin-settings-help-attachments-blocked-mime-types = Масив JSON типів малими літерами або підстановок верхнього рівня. Блокувальні правила мають пріоритет.
admin-settings-key-attachments-warn-mime-types = Попереджати про типи MIME
admin-settings-help-attachments-warn-mime-types = Масив JSON типів малими літерами або підстановок верхнього рівня, що потребують підтвердження. Метадані MIME не є виявленням шкідливого ПЗ.
admin-settings-cat-shortcuts = Комбінації клавіш
admin-settings-cat-time = Часові пояси
admin-settings-key-time-display-zone = Часовий пояс показу
admin-settings-help-time-display-zone = Часовий пояс IANA, наприклад Europe/Lisbon, або local, щоб наслідувати поточний пристрій. Мітки часу листів лишаються збереженими як початкові моменти.
admin-settings-key-time-comparison-zone = Часовий пояс для порівняння
admin-settings-help-time-comparison-zone = Необов’язковий другий часовий пояс IANA для помічників листів і календаря. Залиште порожнім, щоб приховати порівняння; місцезнаходження не визначається за адресою електронної пошти.
admin-settings-key-time-show-helpers = Показувати помічників щодо часових поясів
admin-settings-help-time-show-helpers = Показує порівняння місцевого часу та часу вибраного поясу для міток часу листів і подій календаря. Блокування адміністратора можуть примусово задати цей вибір.
admin-settings-key-shortcuts-enabled = Увімкнути комбінації клавіш
admin-settings-help-shortcuts-enabled = Дозволяє комбінації клавіш застосунку. Блокування можуть вимкнути їх глобально або для домену. Поля введення та модальні вікна зберігають звичайну обробку клавіатури.
admin-settings-key-shortcuts-compose = Комбінація для нового листа
admin-settings-help-shortcuts-compose = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-archive = Комбінація для архівації
admin-settings-help-shortcuts-archive = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-reply = Комбінація для відповіді
admin-settings-help-shortcuts-reply = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-reply-all = Комбінація для відповіді всім
admin-settings-help-shortcuts-reply-all = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-forward = Комбінація для пересилання
admin-settings-help-shortcuts-forward = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-next = Комбінація для наступного листа
admin-settings-help-shortcuts-next = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-previous = Комбінація для попереднього листа
admin-settings-help-shortcuts-previous = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-search = Комбінація для переходу до пошуку
admin-settings-help-shortcuts-search = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-refresh = Комбінація для оновлення листів
admin-settings-help-shortcuts-refresh = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-settings = Комбінація для відкриття налаштувань
admin-settings-help-shortcuts-settings = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-trash = Комбінація для переміщення до кошика
admin-settings-help-shortcuts-trash = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-mark-read = Комбінація для позначення прочитаним
admin-settings-help-shortcuts-mark-read = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-mark-unread = Комбінація для позначення непрочитаним
admin-settings-help-shortcuts-mark-unread = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-spam = Комбінація для позначення як спам
admin-settings-help-shortcuts-spam = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-zoom-in = Комбінація для збільшення листа
admin-settings-help-shortcuts-zoom-in = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-zoom-out = Комбінація для зменшення листа
admin-settings-help-shortcuts-zoom-out = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-zoom-reset = Комбінація для скидання масштабу листа
admin-settings-help-shortcuts-zoom-reset = Використовуйте inherit для вибраного набору, disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-shortcuts-open-in-tab = Комбінація для відкриття листа в окремій вкладці
admin-settings-help-shortcuts-open-in-tab = Використовуйте inherit для вибраного набору (Shift+Enter), disabled, щоб прибрати цю дію, або комбінацію, наприклад Ctrl+Shift+K. Конфліктні чи недійсні комбінації не працюють.
admin-settings-key-notifications-push-allowed = Фонові push-сповіщення дозволено
admin-settings-help-notifications-push-allowed = Дозволяє ввімкненим сповіщенням цього облікового запису реєструватися та отримувати фонові push-повідомлення. Дозвіл браузера й далі потрібен.
admin-settings-key-offline-persistent-storage-allowed = Запити постійного сховища браузера дозволено
admin-settings-help-offline-persistent-storage-allowed = Дозволяє явний запит на захист локальних кешованих даних від автоматичного витіснення браузером. Браузер може відмовити; дозвіл автоматично не надається.
admin-settings-key-security-signin-mailbox-failure-limit = Ліміт невдалих облікових даних скриньки
admin-settings-help-security-signin-mailbox-failure-limit = Відхилені облікові дані з однієї вихідної IP-адреси у вікні підрахунку спричиняють тимчасове блокування входу до скриньки. Збої мережі та конфігурації не враховуються.
admin-settings-key-security-signin-admin-failure-limit = Ліміт невдалих облікових даних адміністратора
admin-settings-help-security-signin-admin-failure-limit = Відхилені облікові дані адміністратора з однієї вихідної IP-адреси спричиняють окреме тимчасове блокування входу адміністратора. Короткий сеанс адміністратора та вимоги свіжого підтвердження не змінюються.
admin-settings-key-security-signin-failure-window-secs = Вікно підрахунку невдалих облікових даних
admin-settings-help-security-signin-failure-window-secs = Рухоме вікно в секундах для відмов за обліковими даними та MFA. Успішно завершений вхід скидає лічильник для цього джерела й поверхні.
admin-settings-key-security-signin-lock-secs = Тривалість тимчасового блокування входу
admin-settings-help-security-signin-lock-secs = Тривалість блокування в секундах. Повторні спроби під час блокування не продовжують його початковий строк. Ручні глобальні блокування IP — окрема функція.
admin-settings-key-security-signin-geo-scope = Область політики країн для входу
admin-settings-help-security-signin-geo-scope = Типово вимкнено. Застосовувати лише до нових входів до скриньки, адміністратора або всіх. Наявні сеанси не відкликаються. Перед увімкненням перевірте діагностику поточного джерела; недоступна локальна база GeoIP призводить до відмови у вході.
admin-settings-key-security-signin-geo-mode = Список дозволених або заборонених країн
admin-settings-help-security-signin-geo-mode = Allow допускає лише перелічені країни; deny відмовляє перелічених країнам. Невідомі адреси мають власну явну політику.
admin-settings-key-security-signin-geo-countries = Коди країн для входу
admin-settings-help-security-signin-geo-countries = Масив JSON двобуквених кодів країн великими літерами, наприклад ["PT", "GB"]. Використовує лише локальну базу країн (Безпека > База країн або MW_GEOIP_DB); адреси ніколи не надсилаються службі геолокації.
admin-settings-key-security-signin-geo-unknown = Політика для невизначених вихідних адрес
admin-settings-help-security-signin-geo-unknown = Дозволяти чи забороняти адреси, яких немає в читабельній локальній базі, зокрема приватні адреси. Відсутня або нечитабельна база завжди відхиляє входи за чинної політики, навіть якщо невідомі адреси дозволено.
admin-settings-key-contacts-avatar-mode = Фото контактів і згенеровані аватари
admin-settings-help-contacts-avatar-mode = Генерувати ініціали локально, приховувати аватари або використовувати вбудовані фото з власних адресних книг кожного облікового запису. Віддалені адреси фото та зовнішня служба аватарів не використовуються. Політику користувача й домену може зафіксувати адміністратор.
admin-settings-cat-contacts = Фото контактів
admin-settings-key-reader-action-style = Стрічка дій із листом
admin-settings-help-reader-action-style = Типово компактні значки. Оберіть підписи, щоб значки мали видимий текст; обидва варіанти зберігають назви дій і тематичні підказки. Підтримує глобальні блокування, блокування домену та користувача.
admin-settings-key-nextcloud-sync-interval-secs = Інтервал автоматичного оновлення Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Секунди між фоновими оновленнями колекцій для підключених облікових записів нативного рушія. Нуль — лише вручну; додатні значення мають мінімум шістдесят секунд. Після збоїв інтервал збільшується. Фоновий вхід не виконується.
admin-settings-key-mail-folder-role-inbox = Зіставлення теки «Вхідні»
admin-settings-help-mail-folder-role-inbox = Null використовує автоматичне виявлення. Використовуйте об’єкт із точним шляхом для спільних типових значень або об’єкт з ідентифікатором скриньки, що належить одному обліковому запису. Відсутні чи конфліктні цілі вимикають цю роль; блокування адміністратора діють.
admin-settings-key-mail-folder-role-drafts = Зіставлення теки «Чернетки»
admin-settings-help-mail-folder-role-drafts = Null використовує автоматичне виявлення. Використовуйте об’єкт із точним шляхом для спільних типових значень або об’єкт з ідентифікатором скриньки, що належить одному обліковому запису. Відсутні чи конфліктні цілі вимикають цю роль; блокування адміністратора діють.
admin-settings-key-mail-folder-role-sent = Зіставлення теки «Надіслані»
admin-settings-help-mail-folder-role-sent = Null використовує автоматичне виявлення. Використовуйте об’єкт із точним шляхом для спільних типових значень або об’єкт з ідентифікатором скриньки, що належить одному обліковому запису. Відсутні чи конфліктні цілі вимикають цю роль; блокування адміністратора діють.
admin-settings-key-mail-folder-role-trash = Зіставлення теки «Кошик»
admin-settings-help-mail-folder-role-trash = Null використовує автоматичне виявлення. Використовуйте об’єкт із точним шляхом для спільних типових значень або об’єкт з ідентифікатором скриньки, що належить одному обліковому запису. Відсутні чи конфліктні цілі вимикають цю роль; блокування адміністратора діють.
admin-settings-key-mail-folder-role-junk = Зіставлення теки «Спам»
admin-settings-help-mail-folder-role-junk = Null використовує автоматичне виявлення. Використовуйте об’єкт із точним шляхом для спільних типових значень або об’єкт з ідентифікатором скриньки, що належить одному обліковому запису. Відсутні чи конфліктні цілі вимикають цю роль; блокування адміністратора діють.
admin-settings-key-mail-folder-role-archive = Зіставлення теки «Архів»
admin-settings-help-mail-folder-role-archive = Null використовує автоматичне виявлення. Використовуйте об’єкт із точним шляхом для спільних типових значень або об’єкт з ідентифікатором скриньки, що належить одному обліковому запису. Відсутні чи конфліктні цілі вимикають цю роль; блокування адміністратора діють.
admin-settings-key-mail-request-delivery-receipt = Запитувати сповіщення про доставку типово
admin-settings-help-mail-request-delivery-receipt = Вибирає сповіщення про доставку для нових листів, якщо обліковий запис підтримує SMTP DSN. Блокування керує цим збереженим типовим значенням; receipts.dsnAllowed окремо визначає, чи дозволені запити. Доставка не доводить прочитання.
admin-settings-key-security-signin-geo-enforce-sessions = Застосовувати політику країн до наявних сеансів
admin-settings-help-security-signin-geo-enforce-sessions = Типово вимкнено. Застосовує налаштовану область країн входу до автентифікованих запитів, нативного доступу bearer і підписаного вмісту засобу читання. Заблоковані клієнти все ще можуть вийти. Уже відкриті потоки перевіряються під час повторного підключення; наявні з’єднання це не розриває.
admin-settings-key-security-protection-rate-mode = Зведені бюджети запитів
admin-settings-help-security-protection-rate-mode = Типово вимкнено. Observe фіксує вичерпання бюджету без відмови; enforce повертає тимчасову відповідь 429. Наявні ліміти паралелізму протоколів лишаються незалежними.
admin-settings-key-security-protection-adaptive-mode = Адаптивний захист запитів
admin-settings-help-security-protection-adaptive-mode = Типово вимкнено. Observe фіксує пояснювані типізовані аномалії та запропоновані дії; enforce застосовує обмежені тимчасові уповільнення або блокування. Це детермінована політика лічильників, а не ШІ-класифікатор.
admin-settings-key-security-protection-client-requests = Запитів клієнта за вікно
admin-settings-help-security-protection-client-requests = Запитів за вікно з однієї адреси, що не увійшла в систему, та кожен запит входу. Запит із чинним сеансом скриньки чи адміністратора використовує власний, більший ліміт. Нуль вимикає цей бюджет.
admin-settings-key-security-protection-account-requests = Запитів облікового запису за вікно
admin-settings-help-security-protection-account-requests = Запитів за вікно для однієї скриньки, що увійшла, з усіх її сеансів, пристроїв і адрес. Вище за анонімний ліміт, щоб у тих, хто увійшов, був більший запас. Нуль вимикає цей бюджет.
admin-settings-key-rate-limit-diagnostics-burst = Одночасні запуски діагностики
admin-settings-help-rate-limit-diagnostics-burst = Запуски діагностики та перевірки бекенда, які сеанс адміністратора може почати поспіль. Запуск перевірки, що вже виконується, повертає цю перевірку, а не рахується повторно.
admin-settings-key-rate-limit-diagnostics-per-minute = Запусків діагностики за хвилину
admin-settings-help-rate-limit-diagnostics-per-minute = Запусків діагностики та перевірок бекенда за хвилину, які відновлює сеанс адміністратора.
admin-settings-key-security-protection-admin-requests = Запитів адміністратора за вікно

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-protection-admin-requests = Запитів за вікно для одного адміністратора, що увійшов. Вище за анонімний ліміт. Нуль вимикає цей бюджет.
admin-settings-key-security-protection-rate-window-secs = Вікно бюджету запитів (секунди)
admin-settings-help-security-protection-rate-window-secs = Фіксований інтервал бюджету запитів. Скидання навантаження за перевантаження та звичайні відповіді «не знайдено» не вважаються адаптивним свідченням.
admin-settings-key-security-protection-adaptive-clients = Спостерігати клієнтів в адаптивному захисті
admin-settings-help-security-protection-adaptive-clients = Вмикає цей вимір суб’єкта, коли адаптивний режим активний. Облікові записи та адміністратори враховуються лише після перевірки їхньої особи.
admin-settings-key-security-protection-adaptive-accounts = Спостерігати облікові записи в адаптивному захисті
admin-settings-help-security-protection-adaptive-accounts = Вмикає цей вимір суб’єкта, коли адаптивний режим активний. Облікові записи та адміністратори враховуються лише після перевірки їхньої особи.
admin-settings-key-security-protection-adaptive-admins = Спостерігати адміністраторів в адаптивному захисті
admin-settings-help-security-protection-adaptive-admins = Вмикає цей вимір суб’єкта, коли адаптивний режим активний. Облікові записи та адміністратори враховуються лише після перевірки їхньої особи.
admin-settings-key-security-protection-short-window-secs = Коротке вікно аномалій (секунди)
admin-settings-help-security-protection-short-window-secs = Рухоме вікно спостереження. Довгі вікна підтримують від годин до тридцяти днів; округлення кошиків може зберегти щонайбільше одну додаткову шістдесят четверту частину вікна.
admin-settings-key-security-protection-short-minimum-samples = Мінімум вибірок короткого відношення
admin-settings-help-security-protection-short-minimum-samples = Мінімум спостережуваних нормальних і аномальних запитів, перш ніж спрацює тригер відношення. Абсолютний поріг аномалій не залежить від цього мінімуму.
admin-settings-key-security-protection-short-absolute-anomalies = Короткий абсолютний бюджет аномалій
admin-settings-help-security-protection-short-absolute-anomalies = Спрацьовує, коли накопичується стільки вибраних аномалій, навіть якщо звичайні запити розбавляють відношення. Нуль вимикає цей тригер.
admin-settings-key-security-protection-short-ratio-percent = Коротке відношення аномалій (відсоток)
admin-settings-help-security-protection-short-ratio-percent = Спрацьовує, коли вибрані аномалії досягають цього відсотка після мінімальної кількості вибірок. Достатньо абсолютного тригера або тригера відношення. Нуль вимикає цей тригер.
admin-settings-key-security-protection-long-window-secs = Довге вікно аномалій (секунди)
admin-settings-help-security-protection-long-window-secs = Рухоме вікно спостереження. Довгі вікна підтримують від годин до тридцяти днів; округлення кошиків може зберегти щонайбільше одну додаткову шістдесят четверту частину вікна.
admin-settings-key-security-protection-long-minimum-samples = Мінімум вибірок довгого відношення
admin-settings-help-security-protection-long-minimum-samples = Мінімум спостережуваних нормальних і аномальних запитів, перш ніж спрацює тригер відношення. Абсолютний поріг аномалій не залежить від цього мінімуму.
admin-settings-key-security-protection-long-absolute-anomalies = Довгий абсолютний бюджет аномалій
admin-settings-help-security-protection-long-absolute-anomalies = Спрацьовує, коли накопичується стільки вибраних аномалій, навіть якщо звичайні запити розбавляють відношення. Нуль вимикає цей тригер.
admin-settings-key-security-protection-long-ratio-percent = Довге відношення аномалій (відсоток)
admin-settings-help-security-protection-long-ratio-percent = Спрацьовує, коли вибрані аномалії досягають цього відсотка після мінімальної кількості вибірок. Достатньо абсолютного тригера або тригера відношення. Нуль вимикає цей тригер.
admin-settings-key-security-protection-normal-cap-per-bucket = Межа нормальної бази на кошик
admin-settings-help-security-protection-normal-cap-per-bucket = Обмежує нормальну базову лінію, яку вивчають у кожному кошику рухомого вікна. Незалежні абсолютні бюджети аномалій не дають необмеженому заповненню дійсними запитами приховати зловживання.
admin-settings-key-security-protection-signals = Сигнали адаптивних свідчень
admin-settings-help-security-protection-signals = Оберіть авторитетні спостереження: некоректні, надто великі, з відхиленими обліковими даними, з відмовою в дозволі, з перевищенням строку тіла або (за бажанням) промахи маршрутів адміністратора. Зондування маршрутів адміністратора виключає відповіді про незнайдену сутність і статичні ресурси; загальні коди стану, відмови через перевантаження та рядки user-agent не доводять зловживання.
admin-settings-key-security-protection-throttle-secs = Початкове тимчасове уповільнення (секунди)
admin-settings-help-security-protection-throttle-secs = Перше перетин порога запускає фіксований строк уповільнення. Запити протягом цього строку його не продовжують.
admin-settings-key-security-protection-block-secs = Посилене тимчасове блокування (секунди)
admin-settings-help-security-protection-block-secs = Пізніше перетин порога до відновлення може перейти до цього обмеженого блокування. Постійне блокування облікового запису не створюється.
admin-settings-key-security-protection-decay-secs = Інтервал спокійного відновлення (секунди)
admin-settings-help-security-protection-decay-secs = Після цього інтервалу без аномалій і завершення поточної дії ескалація повертається до початкової стадії. Рухомі свідчення спливають згідно з налаштованими вікнами.
admin-settings-key-security-protection-exempt-clients = Виключені адреси та мережі клієнтів
admin-settings-help-security-protection-exempt-clients = Точні IP-адреси або мережі CIDR виключають лише вимір клієнта. Перевірені ліміти облікових записів і далі діють. Використовуйте свідомо для спільних мереж NAT; переспрямовані адреси вважаються довіреними лише через налаштовані проксі.
admin-settings-key-security-protection-exempt-accounts = Виключені перевірені ідентифікатори облікових записів
admin-settings-help-security-protection-exempt-accounts = Точні перевірені сервером ідентифікатори облікових записів скриньок виключають лише їхній вимір облікового запису, але не вимір вихідного клієнта. Адреси електронної пошти та параметри облікового запису, передані викликачем, не є доказом особи.
admin-settings-key-security-protection-exempt-admins = Виключені ідентифікатори адміністраторів
admin-settings-help-security-protection-exempt-admins = Точні автентифіковані ідентифікатори адміністраторів виключають лише їхній вимір адміністратора. Наявні блокування після невдалих входів і суворі строки сеансу адміністратора не змінюються.
admin-settings-key-security-protection-storage-failure = Політика збою сховища застосування
admin-settings-help-security-protection-storage-failure = Closed відхиляє захищену роботу, коли увімкнений бюджет застосування неможливо перевірити. Open тимчасово дозволяє її з обмеженим експлуатаційним попередженням. Збої в режимі лише спостереження ніколи не спричиняють дій застосування.
admin-settings-key-security-admin-probe-enabled = Захист від зондування адміністратора
admin-settings-help-security-admin-probe-enabled = Рахує запити, що шукають консоль адміністратора, за мережею клієнта (адреса IPv4, IPv6 /64), уповільнює джерело, яке не зупиняється, а потім блокує його. Одруківки та застарілі закладки вкладаються в пільговий ліміт. Змінюється в розділі «Безпека > Адаптивний захист».
admin-settings-key-security-admin-probe-grace-allowance = Пільговий ліміт (промахів за вікно)
admin-settings-help-security-admin-probe-grace-allowance = Промахи, що ігноруються за вікно. Рахунок безперервно зменшується на стільки балів за вікно.
admin-settings-key-security-admin-probe-window-secs = Пільгове вікно (секунди)
admin-settings-help-security-admin-probe-window-secs = Вікно, до якого належить пільговий ліміт.
admin-settings-key-security-admin-probe-block-after = Промахи понад пільгу до блокування
admin-settings-help-security-admin-probe-block-after = Промахи понад пільговий ліміт, з яких починається блокування. Проміжні промахи обробляються після затримки tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Затримка tarpit (мс)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Затримка, що додається до кожного врахованого промаху понад пільговий ліміт. 0 вимикає tarpit. Інші запити ніколи не затримуються.
admin-settings-key-security-admin-probe-block-secs = Перше блокування (секунди)
admin-settings-help-security-admin-probe-block-secs = Тривалість першого блокування. Заблоковане джерело отримує однакову відповідь для будь-якого шляху.
admin-settings-key-security-admin-probe-max-block-secs = Найдовше блокування (секунди)
admin-settings-help-security-admin-probe-max-block-secs = Повторне блокування подвоює попередню тривалість, але не більше за цю межу.
admin-settings-key-security-admin-probe-repeat-window-secs = Вікно повтору (секунди)
admin-settings-help-security-admin-probe-repeat-window-secs = Блокування, що почалося протягом цього часу після завершення попереднього, вважається повторним.
admin-settings-key-security-admin-probe-block-status = Статус відповіді блокування
admin-settings-help-security-admin-probe-block-status = Статус HTTP, що надсилається заблокованому джерелу, з Retry-After: 429 або 403.
admin-settings-key-security-admin-probe-signals = Враховувані промахи
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (404 на глибині шляху консолі), admin-signin-failed та admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Шляхи зондування
admin-settings-help-security-admin-probe-probe-paths = Шляхи, яких запитують сканери, наприклад /wp-admin або /.env. Шлях, що містить один із них як цілі сегменти, вважається промахом. Власний шлях консолі — ніколи.
admin-settings-key-security-admin-probe-allowlist = Мережі, що ніколи не блокуються
admin-settings-help-security-admin-probe-allowlist = IP-адреси або мережі CIDR (офіс, VPN), які ніколи не враховуються й не блокуються.
admin-settings-key-security-admin-probe-audit-ip = Вихідна адреса в журналі аудиту
admin-settings-help-security-admin-probe-audit-ip = masked показує IPv4 /24 та IPv6 /48; full показує адресу. Рядок fail2ban завжди містить повну адресу.
admin-settings-key-security-admin-probe-fail2ban-log = Рядок журналу fail2ban під час блокування
admin-settings-help-security-admin-probe-fail2ban-log = Записувати один рядок mailwoman[admin-probe] з rhost=<адреса>, коли джерело блокується.
admin-settings-key-appearance-custom-css-enabled = Дозволити особистий CSS вигляду
admin-settings-help-appearance-custom-css-enabled = Дозволяє обмежені візуальні стилі в робочому просторі власника. Типово вимкнено; примусові блокування вигляду скриньки призупиняють ці стилі.
admin-settings-key-appearance-custom-js-enabled = Дозволити особисті скрипти вигляду
admin-settings-help-appearance-custom-js-enabled = Дозволяє ізольовані скрипти, що можуть лише запитувати зміни вигляду. Типово вимкнено; скрипти не мають доступу до пошти, даних облікового запису чи DOM застосунку.
admin-settings-key-mail-modifier-selection = Вибір листів клавішами-модифікаторами
admin-settings-help-mail-modifier-selection = Дозволяє вибір діапазону клавішами Ctrl, Command і Shift. Заблокуйте це значення, щоб застосувати його примусово; видимі елементи вибору лишаються доступними.
admin-settings-key-mail-drag-to-folder = Перетягування листів до тек
admin-settings-help-mail-drag-to-folder = Дозволяє перетягування листів, прив’язаних до облікового запису, до реальних тек. Це не надає дозволу на запис до скриньки; сервер і далі авторизує кожне переміщення.
admin-settings-key-admin-allow-last-superadmin-loss = Дозволити втрату останнього суперадміністратора
admin-settings-help-admin-allow-last-superadmin-loss = Типово вимкнено. Увімкнення дозволяє подальшим змінам прибрати останній доступ суперадміністратора. Повний суперадміністратор має підтвердити наслідки свіжою автентифікацією.
admin-settings-last-superadmin-title = Дозволити втрату всього доступу суперадміністратора?
admin-settings-last-superadmin-body = Подальші зміни облікових записів можуть залишити нікого, хто міг би адмініструвати це розгортання. Відновлення потребуватиме локального адміністрування розгортання. Введіть ALLOW LAST SUPERADMIN LOSS, щоб увімкнути цей виняток.
admin-settings-last-superadmin-confirm = Дозволити втрату доступу
admin-settings-key-mail-folder-counts = Лічильники тек
admin-settings-help-mail-folder-counts = Оберіть приховані, непрочитані, непрочитані лише за наявності непрочитаної пошти, загальні або обидва лічильники біля тек. Задавайте типові значення глобально, для домену чи користувача й блокуйте вибір за потреби. Застосовується до кожної пов’язаної скриньки облікового запису.
admin-settings-key-mail-folder-count-scope = Теки з лічильниками
admin-settings-help-mail-folder-count-scope = Усі теки або лише основні («Вхідні», «Чернетки», «Надіслані», «Архів», «Спам», «Кошик», «Вихідні» та інші теки особливого призначення). За вибору лише основних у користувацьких тек і їхніх підтек лічильник не показується. Застосовується до кожної пов’язаної скриньки облікового запису.
admin-settings-key-mail-compact-folder-badges = Компактні значки решти тек
admin-settings-help-mail-compact-folder-badges = Показує менші значки поза «Вхідними». Користувачі можуть обрати власний розмір, якщо це налаштування не заблоковано.
admin-settings-key-mail-focused-inbox = Важливі вхідні
admin-settings-help-mail-focused-inbox = Ділить «Вхідні» на вкладки «Важливі» та «Інші»; розсилки й автоматичні відправники потрапляють до «Інших». Типово вимкнено. Користувачі можуть увімкнути чи вимкнути це в розділі «Налаштування > Теки та бічна панель», якщо це налаштування не заблоковано.
admin-settings-key-mail-account-unread-badge = Кількість непрочитаних біля назви скриньки
admin-settings-help-mail-account-unread-badge = Показує кількість непрочитаних у «Вхідних» поруч з адресою чи псевдонімом користувача, що увійшов, на бічній панелі. Типово вимкнено. Лічильники інших скриньок у меню скриньок показуються завжди. Застосовується до кожної пов’язаної скриньки облікового запису.
admin-settings-key-privacy-remote-image-downloads = Ручне завантаження віддалених зображень
admin-settings-key-privacy-remote-images-toast = Сповіщення про завантаження зображень
admin-settings-help-privacy-remote-images-toast = Показує одне сповіщення в засобі читання, поки віддалені зображення листа завантажуються через проксі приватності, і повідомляє, коли їх завантажено або скільки не вдалося завантажити. Типово ввімкнено. Змінює лише те, що показує засіб читання; які зображення завантажуються, і далі визначає політика віддаленого вмісту.
admin-settings-help-privacy-remote-image-downloads = Дозволяє явне завантаження зображення, на яке посилається лист, що належить користувачеві, через захищений проксі приватності. Типово вимкнено; зовнішній URL ніколи не відкривається напряму.
admin-settings-key-mail-resume-tabs = Відновлювати вкладки робочої області
admin-settings-help-mail-resume-tabs = Відновлює після перезавантаження посилання на вкладки, що належать обліковому запису. Вимкнення очищує локальний запис робочої області, не видаляючи чернеток.
admin-settings-key-metrics-default-range-seconds = Типовий діапазон історії запитів
admin-settings-help-metrics-default-range-seconds = Секунди, що показуються, коли запит історії не задає діапазон. Обмежений зберіганням і максимальним діапазоном.
admin-settings-key-metrics-max-range-seconds = Максимальний діапазон історії запитів
admin-settings-help-metrics-max-range-seconds = Найбільша кількість секунд на запит історії. Зберігання лишається додатковим обмеженням.
admin-settings-key-contacts-favorites-first = Обрані першими в підказках
admin-settings-help-contacts-favorites-first = Виводить обрані контакти перед іншими збігами в підказках одержувачів. Змінюється лише порядок; жодна підказка не приховується. Перевизначення користувача й домену можна заблокувати.
admin-settings-key-contacts-sender-icons = Вбудовані значки відправників

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-contacts-sender-icons = Показує локально вбудовані значки для впізнаваних доменів відправників, а для решти — згенеровані ініціали. Фото контактів мають пріоритет. Значки не перевіряють особу відправника. Зовнішня служба логотипів не використовується; перевизначення користувача й домену можна заблокувати.
admin-settings-key-mail-brand-avatars = Логотипи брендів біля перевірених відправників
admin-settings-help-mail-brand-avatars = Показує локально вбудований логотип бренду біля відправника в засобі читання, лише якщо лист пройшов DKIM і узгоджену перевірку DMARC для власного домену цього бренду. Листи, що не пройшли ці перевірки або не мають їх, зберігають звичайний аватар. Логотип не завантажується ні від відправника, ні зі служби логотипів.
admin-settings-key-mail-draft-save-mode = Режим збереження чернеток облікового запису
admin-settings-help-mail-draft-save-mode = Зберігати після паузи в наборі, періодично або лише на вимогу. Для збереження потрібна підтримувана тека «Чернетки» на вищому сервері; невизначені записи ніколи не повторюються автоматично.
admin-settings-key-mail-draft-debounce-ms = Пауза в наборі перед збереженням чернетки (мілісекунди)
admin-settings-help-mail-draft-debounce-ms = Чекати від 500 до 10000 мілісекунд після редагування перед збереженням у режимі реального часу.
admin-settings-key-mail-draft-interval-secs = Інтервал збереження чернетки (секунди)
admin-settings-help-mail-draft-interval-secs = Зберігати змінені чернетки кожні 5–3600 секунд у періодичному режимі. Незмінені чернетки повторно не записуються.
admin-settings-key-login-remember-signins = Запам’ятовувати адреси успішних входів
admin-settings-help-login-remember-signins = Зберігати на цьому пристрої до десяти успішно використаних імен користувачів або адрес електронної пошти для випадного списку входу. Паролі та сеанси ніколи не зберігаються. Вимкнення видаляє збережені адреси цього сервера, коли пристрій наступного разу завантажить сторінку входу або завершить вхід; автозаповненням браузера це не керує.
admin-settings-key-login-show-remembered-hint = Показувати пояснення про збережені адреси
admin-settings-help-login-show-remembered-hint = Показує пояснювальний текст у випадному списку попередніх входів. Типово приховано. Діє, лише якщо ввімкнено запам’ятовування успішних входів.
admin-settings-key-mail-show-from = Показувати поле «Від» у вікні створення
admin-settings-help-mail-show-from = Показувати «Від» типово в нових вікнах створення. Користувачі можуть відкрити його для одного листа; надсилати можуть лише авторизовані адреси.
admin-settings-key-accounts-linking-enabled = Дозволити пов’язані облікові записи скриньок
admin-settings-help-accounts-linking-enabled = Типово вимкнено. Глобальне вимкнення блокує весь пов’язаний доступ; обидва облікові записи також мають відповідати політиці свого домену та облікового запису. Для прив’язування потрібні реальна автентифікація дочірньої скриньки та свіже підтвердження основного облікового запису.
admin-settings-key-accounts-use-linked = Використовувати пов’язані скриньки
admin-settings-help-accounts-use-linked = Відмова користувача від пов’язаного доступу. Не може перевизначити адміністративне вимкнення. Дані скриньки зберігаються, коли доступ вимкнено.
admin-settings-key-accounts-max-linked = Найбільша кількість пов’язаних скриньок
admin-settings-help-accounts-max-linked = Найбільша кількість перевірених дочірніх скриньок на основну особу, від 1 до 16. Типово 5.
admin-settings-key-accounts-store-linked-credentials = Зберігати доступ до пов’язаних скриньок
admin-settings-help-accounts-store-linked-credentials = Зберігати пароль кожної пов’язаної скриньки в запечатаному вигляді на сервері, щоб доступ оновлювався сам, між сеансами та перезапусками. Ніколи не зберігається для облікових записів із нульовим доступом і скриньок із двоетапною перевіркою. Вимкнення стирає збережені паролі; зв’язки потім оновлюються повторним входом. Типово ввімкнено.
admin-settings-key-privacy-strip-mailer-headers = Видаляти заголовки поштової програми з вихідних листів
admin-settings-help-privacy-strip-mailer-headers = Видаляє заголовки верхнього рівня X-Mailer, User-Agent і X-MimeOLE з листів нативного SMTP та EWS. Вкладені листи й дані звітів лишаються недоторканими. Надсилання через вищий проксі JMAP не охоплюється.
admin-settings-key-images-trusted-hosts = Довірені хости зображень
admin-settings-help-images-trusted-hosts = До 100 хостів зображень і їхніх піддоменів, віддалені зображення яких завантажуються без дозволу для листа, коли віддалений вміст налаштовано на запит. Ніколи не перевизначає режим блокування; заблокований хост лишається заблокованим.
admin-settings-key-images-blocked-hosts = Заблоковані хости зображень
admin-settings-help-images-blocked-hosts = До 100 хостів зображень і їхніх піддоменів, віддалені зображення яких ніколи не завантажуються в жодному режимі. Має пріоритет над довіреними хостами та будь-яким дозволом.
admin-settings-key-links-trusted-domains = Довірені домени посилань
admin-settings-help-links-trusted-domains = До 100 точних доменів, посилання яких відкриваються з «Перевірки посилань» без кроку призначення. Без підстановних знаків, схем, портів і IP-адрес. Посилання за звичайним HTTP перевіряються завжди. Заблокуйте це налаштування, щоб користувачі не змінювали список.
admin-settings-key-links-trusted-subdomains = Довіряти піддоменам довірених доменів посилань
admin-settings-help-links-trusted-subdomains = Вимкнено: довіряється лише точний домен. Увімкнено: довіряються й імена, що закінчуються на .<домен>. Порівняння йде за цілими мітками, тож example.com ніколи не довіряє evilexample.com.
admin-settings-key-cap-compose-rich-text = Створення форматованого тексту
admin-settings-help-cap-compose-rich-text = Чи можуть користувачі писати форматовані (HTML) листи. Вимкнено надсилає лише простий текст.
admin-settings-key-cap-compose-attachments = Вкладення в нових листах
admin-settings-help-cap-compose-attachments = Чи можуть користувачі прикріплювати файли до листів, що вони пишуть. Зображення в тексті не враховуються.
admin-settings-key-cap-compose-share-links = Посилання спільного доступу для великих файлів
admin-settings-help-cap-compose-share-links = Чи можуть користувачі надіслати великий файл посиланням спільного доступу Nextcloud замість вкладення.
admin-settings-key-cap-compose-scheduled-send = Відкладене надсилання
admin-settings-help-cap-compose-scheduled-send = Чи можуть користувачі запланувати надсилання листа на потім.
admin-settings-key-cap-compose-read-receipts = Запит сповіщень про прочитання
admin-settings-help-cap-compose-read-receipts = Чи можуть користувачі просити одержувачів про сповіщення про прочитання.
admin-settings-key-cap-signatures = Підписи
admin-settings-help-cap-signatures = Чи можуть користувачі створювати й змінювати власні підписи.
admin-settings-key-cap-identities = Особи відправника
admin-settings-help-cap-identities = Чи можуть користувачі додавати й змінювати особи, від імені яких надсилають.
admin-settings-key-cap-keys = Ключі OpenPGP і S/MIME
admin-settings-help-cap-keys = Чи мають користувачі модуль «Ключі» та чи можуть підписувати й шифрувати листи.
admin-settings-key-cap-links-trusted = Редагування довірених доменів посилань
admin-settings-help-cap-links-trusted = Чи можуть користувачі змінювати власний список довірених доменів посилань. Список адміністратора й далі діє.
admin-settings-key-cap-zoom = Масштаб у засобі читання
admin-settings-help-cap-zoom = Чи можуть користувачі масштабувати засіб читання листів.
admin-settings-key-cap-mail-focused-inbox = Важливі вхідні
admin-settings-help-cap-mail-focused-inbox = Чи можуть користувачі ввімкнути вкладки «Важливі» та «Інші» у вхідних.
admin-settings-key-cap-views-events = Подання «Події»
admin-settings-help-cap-views-events = Чи мають користувачі подання «Події» календарної пошти.
admin-settings-key-cap-views-attachments = Подання «Вкладення»
admin-settings-help-cap-views-attachments = Чи мають користувачі подання «Вкладення». Листи лишаються доступними через пошук.
admin-settings-key-cap-search-operators = Оператори пошуку
admin-settings-help-cap-search-operators = Чи можуть користувачі шукати з операторами, як-от from:, subject: або has:attachment.
admin-settings-key-cap-saved-searches = Збережені пошуки
admin-settings-help-cap-saved-searches = Чи можуть користувачі зберігати пошуки.
admin-settings-key-cap-vacation = Автовідповідь на час відпустки
admin-settings-help-cap-vacation = Чи можуть користувачі задавати автоматичну відповідь на час відпустки.
admin-settings-key-cap-labels = Мітки
admin-settings-help-cap-labels = Чи можуть користувачі ставити мітки на листи.
admin-settings-key-cap-snooze = Відкласти
admin-settings-help-cap-snooze = Чи можуть користувачі відкладати листи. Скасування відкладення лишається можливим.
admin-settings-key-cap-follow-up = Нагадування про подальші дії
admin-settings-help-cap-follow-up = Чи можуть користувачі ставити нагадування про подальші дії на листи. Зняття нагадування лишається можливим.
admin-settings-key-cap-masked = Приховані адреси
admin-settings-help-cap-masked = Чи можуть користувачі створювати приховані адреси та керувати ними.
admin-settings-key-cap-directory = Пошук у каталозі
admin-settings-help-cap-directory = Чи можуть користувачі шукати в каталозі організації.
admin-settings-key-cap-mcp = Доступ MCP
admin-settings-help-cap-mcp = Чи можуть ключі та токени користувача використовувати кінцеву точку MCP.
admin-settings-key-cap-appearance-custom = Налаштування вигляду
admin-settings-help-cap-appearance-custom = Чи можуть користувачі змінювати тему, акцент, власні кольори та кольори значків, щільність і шрифт. Вимкнено зберігає вигляд адміністратора.
admin-settings-key-cap-appearance-custom-code = Власний CSS і скрипти
admin-settings-help-cap-appearance-custom-code = Чи можуть користувачі зберігати власний CSS і скрипти вигляду. Перемикачі розгортання в розділі «Вигляд» і далі діють вище за це налаштування.
admin-settings-key-cap-avatars = Аватари скриньок
admin-settings-help-cap-avatars = Чи можуть користувачі змінювати аватари скриньок.
admin-settings-key-cap-shortcuts = Власні комбінації клавіш
admin-settings-help-cap-shortcuts = Чи можуть користувачі змінювати комбінації клавіш. Типові комбінації й далі працюють.
admin-settings-key-cap-analytics = Аналітика продукту
admin-settings-help-cap-analytics = Чи запитується в користувачів згода на аналітику та чи можуть вони надсилати події використання.
admin-settings-key-cap-apps-native = Застосунки для комп’ютера й телефона
admin-settings-help-cap-apps-native = Чи можуть користувачі користуватися застосунками для комп’ютера й телефона.
admin-settings-key-access-protocols = Протоколи доступу до пошти
admin-settings-help-access-protocols = Якими протоколами поштового сервера ці користувачі можуть входити: imap, pop3, ews. Входи через сервер JMAP визначаються цим сервером.
admin-settings-key-limit-linked-mailboxes = Пов’язаних скриньок на користувача
admin-settings-help-limit-linked-mailboxes = Скільки скриньок користувач може пов’язати, не більше за максимум сервера. 0 означає, що діє лише максимум сервера.
admin-settings-key-login-show-account-help-notice = Показувати підказку щодо облікового запису під час входу
admin-settings-help-login-show-account-help-notice = Показує коротке пояснення після невдалого входу. Типово приховано; підказка не розкриває, чи існує обліковий запис або чи вимкнено його.
admin-settings-key-appearance-leetspeak = Стиль тексту leetspeak
admin-settings-help-appearance-leetspeak = Відображає підписи інтерфейсу й текст листів лише для читання у стилі leetspeak. Початковий лист і редагований текст не змінюються. Користувачі можуть обрати звичайний текст, якщо це налаштування не заблоковано.
admin-settings-key-mail-cache-enabled = Увімкнути кеш сеансу
admin-settings-help-mail-cache-enabled = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-mailboxes = Кешувати списки листів тек
admin-settings-help-mail-cache-mailboxes = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-messages = Кешувати відкриті листи
admin-settings-help-mail-cache-messages = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-fresh-seconds = Інтервал свіжості (секунди)
admin-settings-help-mail-cache-fresh-seconds = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-max-age-seconds = Найбільший вік кешу (секунди)
admin-settings-help-mail-cache-max-age-seconds = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-max-entries = Найбільша кількість записів кешу
admin-settings-help-mail-cache-max-entries = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-max-bytes = Бюджет пам’яті (байти)
admin-settings-help-mail-cache-max-bytes = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-prefetch-enabled = Увімкнути випереджальне попереднє завантаження
admin-settings-help-mail-cache-prefetch-enabled = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-hover-enabled = Завантажувати наперед під час наведення на теку
admin-settings-help-mail-cache-hover-enabled = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-focus-enabled = Завантажувати наперед, коли тека отримує фокус клавіатури
admin-settings-help-mail-cache-focus-enabled = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-dwell-ms = Затримка наведення та фокуса (мілісекунди)
admin-settings-help-mail-cache-dwell-ms = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-idle-enabled = Завантажувати теки наперед у простої
admin-settings-help-mail-cache-idle-enabled = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-idle-delay-ms = Затримка простою (мілісекунди)
admin-settings-help-mail-cache-idle-delay-ms = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-max-concurrency = Найбільша кількість одночасних попередніх завантажень
admin-settings-help-mail-cache-max-concurrency = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-max-queued = Найбільша кількість попередніх завантажень у черзі
admin-settings-help-mail-cache-max-queued = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-preload-messages = Дозволити випереджальне завантаження текстів листів
admin-settings-help-mail-cache-preload-messages = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.
admin-settings-key-mail-cache-excluded-folders = Теки, виключені з попереднього завантаження
admin-settings-help-mail-cache-excluded-folders = Зберігає нещодавно відвідані теки й листи в пам’яті цієї вкладки. Кешований вміст оновлюється у фоні. Закриття вкладки або вихід очищує його. Попереднє завантаження ніколи не позначає пошту прочитаною й не завантажує віддалені зображення.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-mail-cache-refresh-on-open = Оновлювати кешований вміст під час відкриття
admin-settings-help-mail-cache-refresh-on-open = Оновлювати кешований вміст під час відкриття
admin-settings-key-mail-templates-enabled = Шаблони відповідей
admin-settings-help-mail-templates-enabled = Увімкнути особисті й спільні шаблони відповідей у вигляді звичайного тексту в редакторі листів. Користувачі можуть відмовитися від них, якщо параметр не заблоковано.
admin-settings-key-mail-templates-allow-personal = Дозволити особисті шаблони
admin-settings-help-mail-templates-allow-personal = Дозволити користувачам створювати власні шаблони відповідей. Спільними шаблонами окремо керують адміністратори.
admin-settings-key-mail-templates-shared = Спільні шаблони відповідей
admin-settings-help-mail-templates-shared = Керування повторно використовуваними відповідями у вигляді звичайного тексту для цієї області. Шаблони ніколи не задають отримувачів і не надсилають листи.
admin-settings-key-mail-templates-personal = Особисті шаблони відповідей
admin-settings-help-mail-templates-personal = Особисті повторно використовувані відповіді у вигляді звичайного тексту. Для створення й редагування мають бути ввімкнені шаблони відповідей та особисті шаблони.
admin-settings-key-mail-compose-window = Вікно редактора
admin-settings-help-mail-compose-window = Відкривати нові чернетки у вкладці робочого простору або в окремому вікні. Якщо спливні вікна заблоковано, використовується вкладка.
admin-settings-key-mail-reply-copy-to-original = Зберігати відповіді в початковій теці
admin-settings-help-mail-reply-copy-to-original = Зберігати копію надісланої відповіді в теці початкового листа, якщо це підтримується.
admin-settings-key-mail-compose-check-invalid-recipients = Перевіряти недійсних отримувачів перед надсиланням
admin-settings-help-mail-compose-check-invalid-recipients = Що відбувається, коли лист надсилається на недійсну адресу: дозволити, попередити, попередити й запам’ятати вибір або відхилити.
admin-settings-key-mail-compose-check-missing-attachment = Перевіряти відсутність вкладення перед надсиланням
admin-settings-help-mail-compose-check-missing-attachment = Що відбувається, коли в тексті згадано вкладення, але нічого не прикріплено: дозволити, попередити, попередити й запам’ятати вибір або відхилити.
admin-settings-key-mail-compose-check-empty-subject = Перевіряти порожню тему перед надсиланням
admin-settings-help-mail-compose-check-empty-subject = Що відбувається, коли в листа немає теми: дозволити, попередити, попередити й запам’ятати вибір або відхилити.
admin-settings-key-mail-compose-auto-chip-recipients = Автоматично завершувати введення отримувачів
admin-settings-help-mail-compose-auto-chip-recipients = Перетворювати введену адресу на мітку отримувача після пробілу, коми або крапки з комою.
admin-settings-key-mail-compose-header-collapse = Заголовок редактора
admin-settings-help-mail-compose-header-collapse = Поведінка рядків «Кому», «Копія», «Прихована копія» й «Тема» в редакторі: розгорнуті, згорнуті в однорядковий підсумок або автоматично (згорнуті під час введення тексту, відкриті, коли вибрано поле). Користувачі можуть самі згортати й розгортати заголовок, якщо параметр не заблоковано.
admin-settings-key-mail-compose-recipient-separator = Роздільник отримувачів
admin-settings-help-mail-compose-recipient-separator = Роздільник, що вставляється після завершеного отримувача: кома або крапка з комою.
admin-settings-key-mail-reply-from-mode = Профіль для відповідей і пересилання
admin-settings-help-mail-reply-from-mode = Від якого профілю надсилається відповідь або пересилання: на який було адресовано початковий лист, типового, з вибором зі списку або поточного вибраного.
admin-settings-key-mail-forward-mode = Типовий формат пересилання
admin-settings-help-mail-forward-mode = Пересилати текст у тілі листа або додавати початковий лист як файл .eml.
admin-settings-key-mail-show-go-to-top = Показувати кнопку «Нагору»
admin-settings-help-mail-show-go-to-top = Показувати в списку листів кнопку «Нагору», яка прокручує до першого листа. Типово вимкнено.
admin-settings-key-mail-show-create-folder-button = Показувати кнопку «Створити теку»
admin-settings-help-mail-show-create-folder-button = Показувати на бічній панелі поруч із кнопкою «Написати» кнопку «Створити теку» лише зі значком. Типово вимкнено.
admin-settings-key-security-protection-user-agent-mode = Режим спостереження за user-agent
admin-settings-help-security-protection-user-agent-mode = Типово — примусовий: вибрані розпізнані заголовки ботів блокуються одразу, якщо ввімкнено блокування ботів під час першого запиту. Спостереження записує рішення без відмови; «вимк.» вимикає цей параметр. Заявлені в заголовках дані не є підтвердженою особою.
admin-settings-key-security-protection-user-agent-block-bots = Блокувати розпізнаних ботів під час першого запиту
admin-settings-help-security-protection-user-agent-block-bots = Типово ввімкнено. Вибрані заголовки пошукових, ШІ-, автоматизованих і власних ботів спричиняють тимчасове блокування HTTP 429 уже під час першого запиту в примусовому режимі. Режим спостереження лише записує це; значення false повертає попередні ліміти запитів. Відсутні або незвичайні заголовки, область дії та винятки працюють як раніше.
admin-settings-key-security-protection-user-agent-categories = Категорії user-agent
admin-settings-help-security-protection-user-agent-categories = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-weak-requests = Поріг запитів зі слабким сигналом
admin-settings-help-security-protection-user-agent-weak-requests = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-crawler-requests = Поріг запитів пошукових роботів
admin-settings-help-security-protection-user-agent-crawler-requests = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-window-secs = Вікно спостереження за user-agent (секунди)
admin-settings-help-security-protection-user-agent-window-secs = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-max-action = Максимальна реакція на user-agent
admin-settings-help-security-protection-user-agent-max-action = Максимальна ескалація за слабких або повторюваних ознак, типово — обмеження швидкості. Цей ліміт не обмежує блокування вибраних розпізнаних заголовків ботів під час першого запиту.
admin-settings-key-security-protection-user-agent-scope = Область дії політики user-agent
admin-settings-help-security-protection-user-agent-scope = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-adaptive = Адаптивні спостереження за user-agent
admin-settings-help-security-protection-user-agent-adaptive = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-patterns = Шаблони user-agent
admin-settings-help-security-protection-user-agent-patterns = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-security-protection-user-agent-exceptions = Винятки user-agent
admin-settings-help-security-protection-user-agent-exceptions = Налаштовує цю частину необов’язкового спостереження за user-agent. Текст user-agent, який передає клієнт, — слабкий сигнал, а не доказ особи чи зловживання.
admin-settings-key-mail-reader-unsafe-html-mode = Потенційно небезпечний HTML листа
admin-settings-help-mail-reader-unsafe-html-mode = Типово «Запитувати»: для кожного листа доступна кнопка очищеного HTML. «Суворий» показує лише безпечний текст; «очищений» автоматично показує очищений HTML. Активний вміст і автоматичні зовнішні ресурси лишаються заблокованими. Заблокуйте, щоб застосовувати цю політику примусово.
admin-settings-key-reader-attachments-density = Щільність вкладень
admin-settings-help-reader-attachments-density = Типово компактні кнопки файлів або зручніші попередні перегляди. Визначає вигляд, коли вкладення листа розгорнуто.
admin-settings-key-cap-compose-open-tracking = Дозволити відстеження відкриття листів
admin-settings-help-cap-compose-open-tracking = Чи може цей обліковий запис фіксувати відкриття листів. Потрібен налаштований публічний URL; для облікових записів із вимкненою функцією не можна створювати й прив’язувати токени відстеження.
admin-settings-key-mail-open-tracking = Типово відстежувати відкриття
admin-settings-help-mail-open-tracking = Типовий вибір відстеження для придатних форматованих листів. Користувачі можуть змінити його, якщо його не заблоковано; вимкнення можливості відстеження завжди має пріоритет.
admin-settings-key-cap-reading-original-html = Дозволити показ початкового HTML
admin-settings-help-cap-reading-original-html = Чи можуть користувачі вмикати перегляд початкового HTML для окремих листів. Типовий очищувальний обробник і захист ізольованим фреймом зберігаються.
admin-settings-key-mail-reader-allow-original-html = Пропонувати перегляд початкового HTML

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-reader-allow-original-html = Показувати для кожного листа варіант використання початкового HTML, якщо це дозволяє можливість читання. Типово вимкнено; типовий обробник листів не змінюється.
admin-settings-key-privacy-tracker-override-allowed = Пропонувати згоду на трекери для листа
admin-settings-help-privacy-tracker-override-allowed = Показувати дію «Дозволити трекери» на панелі конфіденційності листа. Доступно типово, але ніколи не діє без явного дозволу. Заблокована політика блокування трекерів і далі забороняє таке виключення.
admin-settings-key-privacy-max-concurrent-global = Глобальна кількість паралельних завантажень зображень
admin-settings-help-privacy-max-concurrent-global = Максимальна кількість активних мережевих запитів зображень у всьому процесі сервера. Діапазон 1–256; типово 128. Ліміти листів і облікових записів і далі діють.
admin-settings-key-privacy-max-concurrent-pipeline = Паралельні конвеєри обробки зображень
admin-settings-help-privacy-max-concurrent-pipeline = Максимальна кількість зображень, що одночасно допускаються до завантаження, декодування й запису в кеш. Діапазон 1–256; типово 128. Ліміти мережі та декодерів застосовуються окремо.
admin-settings-key-privacy-max-concurrent-decodes = Паралельні декодери зображень
admin-settings-help-privacy-max-concurrent-decodes = Максимальна кількість одночасних декодувань зображень. Діапазон 1–16; типово 10. Більші значення використовують більше процесора й пам’яті.
admin-settings-key-privacy-max-queued-requests = Запити зображень, що очікують
admin-settings-help-privacy-max-queued-requests = Максимальна кількість запитів, які очікують мережевих ресурсів або потужності обробки зображень. Діапазон 16–16384; типово 2048. Переповнення або завершення терміну в черзі можна повторити.
admin-settings-key-privacy-rate-burst = Допустимий сплеск запитів зображень
admin-settings-help-privacy-rate-burst = Початкова й максимальна кількість токенів завантаження на обліковий запис, що поповнюються із заданою швидкістю запитів зображень. Діапазон 1–12000; типово 1024. Влучання в кеш не витрачають токени.
admin-settings-key-privacy-image-rate-mode = Розрахунок швидкості запитів зображень
admin-settings-help-privacy-image-rate-mode = Автоматично масштабує сплеск і поповнення для облікового запису за потужністю реплік, навантаженням черги та кількістю зображень у листі в межах нижніх і верхніх меж нижче. Вручну використовує налаштування допустимого сплеску та швидкості запитів без змін. Типово автоматично. Запити понад ліміт чекають токен до завершення терміну черги завантаження; автентифікація й дозволи перевіряються першими.
admin-settings-key-privacy-image-rate-burst-floor = Нижня межа сплеску (автоматично)
admin-settings-help-privacy-image-rate-burst-floor = Найменший допустимий сплеск на обліковий запис в автоматичному режимі. Діапазон 1–12000; типово 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Верхня межа сплеску (автоматично)
admin-settings-help-privacy-image-rate-burst-ceiling = Найбільший допустимий сплеск на обліковий запис в автоматичному режимі. Діапазон 1–12000; типово 2048.
admin-settings-key-privacy-image-rate-refill-floor = Нижня межа поповнення (автоматично)
admin-settings-help-privacy-image-rate-refill-floor = Найменше поповнення на обліковий запис, у токенах завантаження за хвилину, в автоматичному режимі. Діапазон 10–60000; типово 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Верхня межа поповнення (автоматично)
admin-settings-help-privacy-image-rate-refill-ceiling = Найбільше поповнення на обліковий запис, у токенах завантаження за хвилину, в автоматичному режимі. Діапазон 10–60000; типово 6000.
admin-settings-key-privacy-image-cache-mode = Розрахунок розміру кешу зображень
admin-settings-help-privacy-image-cache-mode = Автоматично підбирає розмір кешу за пам’яттю, що лишилася після резервів декодерів і конвеєрів, і збільшує термін зберігання, поки частка влучань висока, а пам’ять вільна. Вручну використовує налаштування розміру кешу, кількості записів і терміну зберігання без змін. Типово автоматично.
admin-settings-key-privacy-image-cache-bytes-floor = Нижня межа розміру кешу (автоматично)
admin-settings-help-privacy-image-cache-bytes-floor = Найменший кеш зображень у байтах в автоматичному режимі. Діапазон 0–1073741824; типово 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Верхня межа розміру кешу (автоматично)
admin-settings-help-privacy-image-cache-bytes-ceiling = Найбільший кеш зображень у байтах в автоматичному режимі. Діапазон 0–1073741824; типово 268435456.
admin-settings-key-privacy-cache-entries = Ліміт записів кешу зображень
admin-settings-help-privacy-cache-entries = Максимальна кількість зображень у кеші; у ручному режимі — фіксована кількість, в автоматичному — верхня межа. Діапазон 16–65536; типово 4096.
admin-settings-key-privacy-cache-entry-bytes = Найбільше зображення в кеші
admin-settings-help-privacy-cache-entry-bytes = Найбільше окреме перекодоване зображення, що зберігається в кеші, у байтах. Більші зображення віддаються, але не зберігаються. Діапазон 65536–16777216; типово 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Нижня межа терміну зберігання кешу (автоматично)
admin-settings-help-privacy-image-cache-ttl-floor-secs = Найкоротший термін зберігання кешу в секундах в автоматичному режимі, застосовується за низької частки влучань або браку пам’яті. Діапазон 0–86400; типово 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Верхня межа терміну зберігання кешу (автоматично)
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Найдовший термін зберігання кешу в секундах в автоматичному режимі, застосовується за високої частки влучань і вільної пам’яті. Діапазон 0–86400; типово 21600.
admin-settings-key-privacy-cache-ttl-secs = Термін зберігання успішних зображень у кеші
admin-settings-help-privacy-cache-ttl-secs = Кількість секунд, протягом яких декодовані зображення можна повторно використовувати в цьому процесі. Діапазон 0–86400; типово 3600. Нуль вимикає повторне використання успішних зображень; авторизація завжди перевіряється першою.
admin-settings-key-privacy-image-latency-tracking = Відстеження швидкості проксі зображень
admin-settings-help-privacy-image-latency-tracking = Записує в пам’яті час обробки кожного зображення через проксі (медіана, p95, p99 та час очікування, завантаження й декодування) як лічильники у фіксованих інтервалах часу. URL, хости й облікові записи не зберігаються. Типово ввімкнено; історія скидається після перезапуску й стосується одного процесу сервера. Вимкнення припиняє новий запис; розділ «Швидкість» і далі показує вже записане.
admin-settings-key-privacy-negative-cache-ttl-secs = Термін зберігання невдалих зображень у кеші
admin-settings-help-privacy-negative-cache-ttl-secs = Кількість секунд, протягом яких придатні збої джерела запам’ятовуються, щоб уникнути повторних запитів. Діапазон 0–300; типово 10. Нуль вимикає повторне використання збоїв. Явна повторна спроба очищає відповідні збої.
admin-settings-key-privacy-image-retry-attempts = Автоматичні повтори завантаження зображень
admin-settings-help-privacy-image-retry-attempts = Максимальна кількість повторів за тимчасових збоїв зображень під час одного видимого завантаження листа. Діапазон 0–8; типово 3. Нуль вимикає автоматичні повтори, зберігаючи ручний повтор.
admin-settings-key-privacy-image-retry-delay-ms = Затримка автоматичного повтору зображення
admin-settings-help-privacy-image-retry-delay-ms = Кількість мілісекунд між автоматичними повторами завантаження зображення після стабілізації поточного кадру. Діапазон 500–60000; типово 5000. Остаточні відмови автоматично не повторюються.
admin-settings-key-privacy-image-decode-timeout-ms = Тайм-аут декодування зображення
admin-settings-help-privacy-image-decode-timeout-ms = Реальний час у мілісекундах, відведений на одне декодування зображення в пісочниці після його завантаження. Діапазон 1000–60000; типово 10000. Збільшіть для великих зображень; мережеві тайм-аути задаються окремо.
admin-settings-key-privacy-image-concurrency-mode = Профіль паралелізму зображень
admin-settings-help-privacy-image-concurrency-mode = Автоматичний підлаштовується під потужність процесора, пам’ять контейнера та кількість схвалених зображень у кожному листі. Профілі «Мало ресурсів», «Збалансований» і «Висока пропускна здатність» також підлаштовуються, але з різними межами безпеки. Ручний напряму використовує числові межі. Застосування профілю змінює лише цей параметр.
admin-settings-cat-image-proxy = Проксі зображень
admin-settings-key-privacy-image-ram-max-mi-b = Максимум рівня RAM
admin-settings-help-privacy-image-ram-max-mi-b = Верхня межа в МіБ для рівня зображень у пам’яті, що застосовується поверх автоматичного або ручного розміру кешу. Нуль означає відсутність обмеження. Тримайте значення невеликим і дозвольте дисковому рівню зберігати основний обсяг. Діапазон 0–1048576; типово 0.
admin-settings-key-privacy-image-disk-enabled = Дисковий рівень зображень
admin-settings-help-privacy-image-disk-enabled = Зберігає вже перекодовані зображення на диску за рівнем RAM. Типово вимкнено; якщо каталог відсутній, заповнений або доступний лише для читання, кеш мовчки продовжує працювати лише в RAM. Зберігаються лише дані на виході з ізольованого середовища, під хешованими іменами файлів.
admin-settings-key-privacy-image-disk-path = Каталог зображень на диску
admin-settings-help-privacy-image-disk-path = Абсолютний шлях до каталогу дискового рівня. Порожнє значення — тека image-cache поруч із файлом бази даних. Змонтуйте там том із правом запису, якщо коренева файлова система доступна лише для читання. Відносні шляхи й .. відхиляються, рівень лишається вимкненим.
admin-settings-key-privacy-image-disk-mode = Розрахунок розміру диска зображень
admin-settings-help-privacy-image-disk-mode = Автоматично використовує десяту частину місця, яке міг би зайняти цей кеш (вільне місце плюс уже зайняте ним), між нижньою й верхньою межею та ніколи не більше половини цього місця. Вручну використовує вказаний нижче максимальний розмір без змін. Типово автоматично.
admin-settings-key-privacy-image-disk-max-mi-b = Розмір диска зображень (вручну)
admin-settings-help-privacy-image-disk-max-mi-b = Розмір дискового рівня в МіБ у ручному режимі; у показниках відображається фактичний розмір в обох режимах. Діапазон 1–1048576; типово 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Нижня межа розміру диска (автоматично)
admin-settings-help-privacy-image-disk-floor-mi-b = Найменший дисковий рівень у МіБ в автоматичному режимі; також використовується, поки вільне місце невідоме. Діапазон 0–1048576; типово 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Верхня межа розміру диска (автоматично)
admin-settings-help-privacy-image-disk-ceiling-mi-b = Найбільший дисковий рівень у МіБ в автоматичному режимі. Діапазон 0–1048576; типово 4096.
admin-settings-key-privacy-image-disk-entries = Ліміт записів на диску
admin-settings-help-privacy-image-disk-entries = Максимальна кількість зображень на диску. Кожен запис також займає фіксований обсяг RAM в індексі, який враховується в рівні RAM. Діапазон 16–1048576; типово 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Найбільше зображення на диску
admin-settings-help-privacy-image-disk-entry-bytes = Найбільше окреме перекодоване зображення, що записується на диск, у байтах. Зображення, які перевищують ліміт запису в RAM, але вкладаються в цей, одразу потрапляють на диск. Діапазон 65536–67108864; типово 8388608.
admin-settings-key-privacy-image-disk-min-hits = Влучання до запису на диск
admin-settings-help-privacy-image-disk-min-hits = Кількість влучань, потрібна запису в RAM, щоб під час витіснення його було записано на диск, а не відкинуто. Нуль записує кожен витіснений запис, оцінка якого перевищує поріг пониження. Діапазон 0–100; типово 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Черга запису на диск
admin-settings-help-privacy-image-disk-write-queue-mi-b = Обсяг даних зображень у МіБ, які можуть чекати фонового запису на диск. Враховується в рівні RAM; понад нього витіснені зображення відкидаються, а не ставляться в чергу. Діапазон 1–256; типово 8.
admin-settings-key-privacy-image-ttl-step-secs = Приріст терміну зберігання за влучання
admin-settings-help-privacy-image-ttl-step-secs = Кількість секунд, що додається до залишкового терміну зберігання зображення під час влучання, помножена на 1 плюс логарифм його згасаючої кількості влучань, тож часто використовувані зображення живуть довше. Нуль лишає кожне зображення з базовим терміном. Діапазон 0–86400; типово 600.
admin-settings-key-privacy-image-ttl-max-secs = Найдовший подовжений термін зберігання
admin-settings-help-privacy-image-ttl-max-secs = Найдовший залишковий термін зберігання в секундах, який можуть накопичити влучання. Діапазон 60–2592000; типово 86400.
admin-settings-key-privacy-image-max-age-secs = Максимальний вік зображення
admin-settings-help-privacy-image-max-age-secs = Вік у секундах, після якого зображення завантажується знову незалежно від влучань. Шлях завантаження не зберігає ETag і Last-Modified джерела, тож умовної перевірки немає; зображення просто завантажується повторно. Діапазон 3600–7776000; типово 604800.
admin-settings-key-privacy-image-score-half-life-secs = Період піврозпаду оцінки
admin-settings-help-privacy-image-score-half-life-secs = Кількість секунд, за які кількість влучань і свіжість зображення зменшуються вдвічі в оцінці пріоритету, що визначає RAM, диск і витіснення. Діапазон 60–604800; типово 3600.
admin-settings-key-privacy-image-score-demote-milli = Оцінка пониження
admin-settings-help-privacy-image-score-demote-milli = Оцінка в тисячних, нижче якої зображення з RAM переводиться на диск або відкидається. Поріг підвищення — це значення плюс гістерезис. Діапазон 0–10000; типово 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Гістерезис підвищення
admin-settings-help-privacy-image-score-hysteresis-percent = Відсоток, на який поріг підвищення перевищує поріг пониження. Оцінка між ними лишає зображення там, де воно є. Діапазон 0–500; типово 50.
admin-settings-key-privacy-image-context-open-percent = Вага відкритого листа
admin-settings-help-privacy-image-context-open-percent = Вага оцінки у відсотках для зображень, запитаних для листа. Зображення листа вважаються недавніми після десяти хвилин простою. Діапазон 100–1000; типово 200.
admin-settings-key-privacy-image-context-recent-percent = Вага недавнього листа
admin-settings-help-privacy-image-context-recent-percent = Вага оцінки у відсотках для нещодавно використаних зображень. Запити без вказаного листа мають вагу 100. Діапазон 100–1000; типово 150.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-account-share-percent = Частка облікового запису в ОЗП
admin-settings-help-privacy-image-account-share-percent = Найбільша частка рівня ОЗП, яку можуть зайняти вставки одного облікового запису, поки інші облікові записи користуються кешем; понад неї обліковий запис витісняє власні зображення з найнижчою оцінкою. Один обліковий запис може зайняти весь обсяг. Діапазон 1–100; типово 50.
admin-settings-key-privacy-image-sweep-secs = Інтервал очищення кешу
admin-settings-help-privacy-image-sweep-secs = Кількість секунд між фоновими проходами, що видаляють застарілі зображення, перераховують оцінки обмеженої партії, переносять неактивні зображення з ОЗП за браку пам'яті та повторно перевіряють вільне місце на диску. Діапазон 5–3600; типово 30.
admin-settings-key-privacy-image-snapshot-secs = Інтервал знімків індексу
admin-settings-help-privacy-image-snapshot-secs = Кількість секунд між атомарними знімками дискового індексу, які зберігають лічильники влучань і подовжені строки зберігання після перезапуску без запису при кожному влучанні. Нуль вимикає знімки; після перезапуску тоді читається заголовок кожного файлу. Діапазон 0–86400; типово 300.
admin-settings-key-privacy-image-sweep-batch = Розмір партії очищення
admin-settings-help-privacy-image-sweep-batch = Кількість записів, які один прохід обслуговування може видалити за строком, перерахувати або витіснити. Обмежує роботу під блокуванням кешу. Діапазон 16–8192; типово 256.
admin-image-proxy-title = Стан кешу зображень
admin-image-proxy-unavailable = Стан кешу зображень недоступний.
admin-image-proxy-unknown = невідомо
admin-image-proxy-ram = Рівень ОЗП: використано { $used } із { $budget } (налаштовано { $configured }), зображень: { $entries }, гарячих: { $hot }, холодних: { $cold }, частка влучань { $hit }.
admin-image-proxy-disk-off = Дисковий рівень: вимкнено.
admin-image-proxy-disk = Дисковий рівень: використано { $bytes } із чинних { $budget } ({ $mode }; нижня межа { $floor }, верхня { $ceiling }, вручну { $manual }), зображень: { $entries }, гарячих: { $hot }, холодних: { $cold }, частка влучань { $hit }.
admin-image-proxy-disk-inactive = Дисковий рівень: увімкнено, але неактивний. { $reason }
admin-image-proxy-moves = Підвищень за хвилину: { $perMin }. Усього підвищень { $promotions }, пониження { $demotions }, відкидань { $drops }. Записів індексу: { $index }.
admin-image-proxy-refresh = Оновити стан
admin-settings-key-security-trust-enabled = Рівні довіри
admin-settings-help-security-trust-enabled = Присвоює кожному запиту рівень: виконано вхід, відоме джерело (нещодавній успішний вхід із цієї адреси) або невідоме. Змінюються лише ліміти анонімних запитів; облікові дані, другі чинники та блокування ніколи не послаблюються.
admin-settings-key-security-trust-known-ttl-days = Вікно відомого джерела (дні)
admin-settings-help-security-trust-known-ttl-days = Як довго успішний вхід зберігає адресу відомою. Невдалі спроби не враховуються.
admin-settings-key-security-trust-known-capacity = Кількість збережених відомих джерел
admin-settings-help-security-trust-known-capacity = Найбільша кількість відомих джерел у пам'яті (для кожного: хеш із сіллю, країна та кількість облікових записів). Найстарше видаляється першим.
admin-settings-key-security-trust-persist = Зберігати відомі джерела між перезапусками
admin-settings-help-security-trust-persist = Зберігає таблицю відомих джерел (хеші з ключем, країна, час останньої активності, кількість облікових записів) у сховищі налаштувань і завантажує її під час запуску. Записи старші за вікно видаляються.
admin-settings-key-security-trust-persist-interval-secs = Інтервал збереження відомих джерел (секунди)
admin-settings-help-security-trust-persist-interval-secs = Кількість секунд між збереженнями, від 30 до 3600. Збереження виконується лише після зміни таблиці та ще раз під час штатного завершення.
admin-settings-key-security-trust-unknown-percent = Ліміт для невідомого джерела (%)
admin-settings-help-security-trust-unknown-percent = Ліміт анонімних запитів невідомого джерела у відсотках від налаштованих лімітів. 100 зберігає базове значення; менше означає суворіше.
admin-settings-key-security-trust-known-percent = Ліміт для відомого джерела (%)
admin-settings-help-security-trust-known-percent = Ліміт анонімних запитів відомого джерела. 200 подвоює бюджети запитів і вдвічі скорочує паузу після входу та складність завдання. Ніколи не оминає облікові дані, другі чинники та блокування.
admin-settings-key-security-trust-ua-missing = Боти: порожній user-agent
admin-settings-help-security-trust-ua-missing = Дія для анонімних запитів із порожнім user-agent: дозволити, обмежити або заблокувати. Сеанси з виконаним входом не зачіпаються.
admin-settings-key-security-trust-ua-uncommon = Боти: нерозпізнаний user-agent
admin-settings-help-security-trust-ua-uncommon = Дія для анонімних запитів із нерозпізнаним user-agent: дозволити, обмежити або заблокувати.
admin-settings-key-security-trust-ua-automation = Боти: скрипти й автоматизація
admin-settings-help-security-trust-ua-automation = Дія для анонімних запитів, що видають себе за скрипт, HTTP-бібліотеку, headless-браузер або звичайного бота: дозволити, обмежити або заблокувати.
admin-settings-key-security-trust-ua-search = Боти: пошукові роботи
admin-settings-help-security-trust-ua-search = Дія для анонімних запитів, що видають себе за пошуковий робот: дозволити, обмежити або заблокувати.
admin-settings-key-security-trust-ua-ai-training = Боти: роботи для навчання ШІ
admin-settings-help-security-trust-ua-ai-training = Дія для анонімних запитів, що видають себе за GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended або Applebot-Extended: дозволити, обмежити або заблокувати. Заява — це заголовок, а не перевірена особа.
admin-settings-key-security-trust-ua-ai-search = Боти: роботи ШІ-пошуку
admin-settings-help-security-trust-ua-ai-search = Дія для анонімних запитів, що видають себе за OAI-SearchBot, PerplexityBot або Claude-SearchBot: дозволити, обмежити або заблокувати.
admin-settings-key-security-trust-ua-ai-user = Боти: завантаження ШІ на запит користувача
admin-settings-help-security-trust-ua-ai-user = Дія для анонімних запитів, що видають себе за ChatGPT-User або Claude-User: дозволити, обмежити або заблокувати.
admin-settings-key-security-trust-ua-custom = Боти: власні шаблони
admin-settings-help-security-trust-ua-custom = Дія для анонімних запитів, що відповідають власним шаблонам user-agent: дозволити, обмежити або заблокувати.
admin-settings-key-security-trust-admin-geo-enabled = Блокувати пошук консолі адміністратора з невідомих країн
admin-settings-help-security-trust-admin-geo-enabled = Блокує джерело, що шукає консоль адміністратора з країни без нещодавнього успішного входу. Потребує читабельної бази країн і захисту від зондування консолі. MW_ADMIN_GEO_ENABLED=false вимикає це для відновлення доступу.
admin-settings-key-security-trust-admin-geo-countries = Завжди відомі країни (блокування консолі адміністратора)
admin-settings-help-security-trust-admin-geo-countries = Коди країн ISO великими літерами, які завжди вважаються відомими, крім країн нещодавніх успішних входів.
admin-settings-key-security-trust-admin-geo-lookup-failure = Джерело без країни
admin-settings-help-security-trust-admin-geo-lookup-failure = allow не блокує й записує випадок у журнал; unknown вважає джерело без визначеної країни невідомою країною.
admin-settings-key-security-trust-admin-geo-threshold = Зондувань до блокування
admin-settings-help-security-trust-admin-geo-threshold = Кількість зондувань консолі адміністратора з невідомої країни у межах вікна, після якої починається блокування.
admin-settings-key-security-trust-admin-geo-window-secs = Вікно зондування (секунди)
admin-settings-help-security-trust-admin-geo-window-secs = Вікно для порога зондувань.
admin-settings-key-security-trust-admin-geo-block-on-first = Блокувати з першого зондування
admin-settings-help-security-trust-admin-geo-block-on-first = Блокує з першого зондування з невідомої країни. Підвищує ризик блокування адміністратора, що подорожує.
admin-settings-key-security-trust-admin-geo-block-secs = Перше блокування (секунди)
admin-settings-help-security-trust-admin-geo-block-secs = Тривалість першого блокування.
admin-settings-key-security-trust-admin-geo-second-block-secs = Друге блокування (секунди)
admin-settings-help-security-trust-admin-geo-second-block-secs = Тривалість другого блокування того самого джерела в межах періоду повторів.
admin-settings-key-security-trust-admin-geo-max-block-secs = Повторне блокування (секунди)
admin-settings-help-security-trust-admin-geo-max-block-secs = Тривалість третього й кожного наступного блокування в межах періоду повторів.
admin-settings-key-security-trust-admin-geo-repeat-days = Період повторів (дні)
admin-settings-help-security-trust-admin-geo-repeat-days = Блокування протягом цієї кількості днів після попереднього вважається повторним і застосовує наступну, довшу тривалість.
admin-settings-key-security-signin-account-geo-mode = Політика країни входу для облікового запису
admin-settings-help-security-signin-account-geo-mode = Типово вимкнена. Блокує входи з країн, якими обліковий запис раніше не користувався, або з будь-якої країни поза його списком дозволених. IP-адреси та країни зі списку дозволених проходять завжди. Застосовується лише до нових входів у вебпошту, не до наявних сеансів і не до клієнтів протоколів. Визначення країни використовує базу геолокації IP, яка може бути хибною або застарілою; VPN, мобільні оператори, проксі та подорожі можуть заблокувати законний вхід або пропустити іноземний. Додайте довірені IP-адреси або країни до списку дозволених.
admin-settings-key-security-signin-account-geo-allow-countries = Дозволені країни облікового запису
admin-settings-help-security-signin-account-geo-allow-countries = Коди країн ISO великими літерами, завжди дозволені для облікового запису. Визначення країни використовує базу геолокації IP, яка може бути хибною або застарілою; VPN, мобільні оператори, проксі та подорожі можуть заблокувати законний вхід або пропустити іноземний. Додайте довірені IP-адреси або країни до списку дозволених.
admin-settings-key-security-signin-account-geo-allow-ips = Дозволені IP-адреси облікового запису
admin-settings-help-security-signin-account-geo-allow-ips = IP-адреси або мережі CIDR, завжди дозволені для облікового запису. Одна адреса IPv6 означає її мережу /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Дозволені країни для всіх облікових записів
admin-settings-help-security-signin-account-geo-global-allow-countries = Коди країн ISO великими літерами, дозволені для всіх облікових записів; додаються до списку кожного облікового запису.
admin-settings-key-security-signin-account-geo-global-allow-ips = Дозволені IP-адреси для всіх облікових записів
admin-settings-help-security-signin-account-geo-global-allow-ips = IP-адреси або мережі CIDR (офіс чи VPN), дозволені для всіх облікових записів; додаються до списку кожного облікового запису.
admin-settings-key-security-signin-account-geo-lookup-failure = Джерело без країни (політика облікового запису)
admin-settings-help-security-signin-account-geo-lookup-failure = Коли країну джерела не визначено: дозволити вхід і записати це в журнал або відхилити його.
admin-settings-key-security-signin-account-geo-history-days = Історія країн облікового запису (дні)
admin-settings-help-security-signin-account-geo-history-days = Скільки днів країна залишається в історії входів облікового запису для політики незвичних країн. Зберігається окремо від журналу автентифікації, тож скорочення журналу її не скорочує.
admin-settings-key-security-geo-no-database = Немає бази країн: типова поведінка
admin-settings-help-security-geo-no-database = Що роблять функції на основі країни, коли базу не завантажено: дозволяють, вважають джерело невідомою країною або блокують (блокування стосується лише входів). Діє також власне налаштування функції; застосовується суворіше.
admin-settings-key-security-geo-not-found = Адреси немає в базі: типова поведінка
admin-settings-help-security-geo-not-found = Що роблять функції на основі країни, коли базу завантажено, але запису для адреси немає. Приватні, loopback і link-local адреси — локальні мережі й ніколи не зачіпаються.
admin-settings-key-security-signin-geo-no-database = Політика входу екземпляра без бази
admin-settings-help-security-signin-geo-no-database = Коли політику країни входу екземпляра ввімкнено, а базу не завантажено: блокувати входи (закрита відмова) або дозволяти без перевірки. Локальні мережі завжди дозволені.
admin-settings-key-security-geo-session-lookup-failure = Сповіщення про сеанси без країни
admin-settings-help-security-geo-session-lookup-failure = Коли країну запиту сеансу з виконаним входом визначити не вдається: ігнорувати або створювати одне сповіщення на сеанс на добу. Ніколи не блокує й не відкликає сеанс.
admin-settings-key-security-protocol-framing-mode = Захист протоколу: обрамлення запитів
admin-settings-help-security-protocol-framing-mode = Клас request smuggling: Content-Length разом із Transfer-Encoding, Transfer-Encoding не точно chunked (або в HTTP/1.0 і HTTP/2), нечисловий чи суперечливий Content-Length. Режим enforce відповідає 400 і закриває з'єднання; detect лише рахує. Виняток за префіксом маршруту до цього класу не застосовується.
admin-settings-key-security-protocol-host-mode = Захист протоколу: синтаксис Host
admin-settings-help-security-protocol-host-mode = Синтаксис заголовка Host: відсутній у HTTP/1.1, порожній, повторений або з даними користувача, пробілами чи символами шляху. Літерали IPv6, порти й кінцева крапка допускаються. Режим enforce відповідає 400 і закриває з'єднання.
admin-settings-key-security-protocol-host-pinning-mode = Захист протоколу: Host має збігатися з публічним ім'ям
admin-settings-help-security-protocol-host-pinning-mode = Host не збігається з хостом MW_PUBLIC_URL або одним із allowedHosts. Нічого не робить, доки жоден із них не задає хост. Типово detect, оскільки проксі та перевірки стану можуть надсилати внутрішнє ім'я.
admin-settings-key-security-protocol-path-control-mode = Захист протоколу: керівні символи в шляху
admin-settings-help-security-protocol-path-control-mode = NUL, керівні символи або закодовані CR/LF (%00, %0d, %0a) у шляху запиту. Режим enforce відповідає 400. Ніколи не виключається префіксом маршруту.
admin-settings-key-security-protocol-target-mode = Захист протоколу: хитрощі в цілі запиту
admin-settings-help-security-protocol-target-mode = Подвійне або надлишкове відсоткове кодування крапок і скісних рисок, закодовані крапки, сегменти з крапками, порожні сегменти шляху (//), зворотні скісні риски, необроблені байти не ASCII та ціль в абсолютній формі, що вказує інший авторитет, ніж Host.
admin-settings-key-security-protocol-path-length-mode = Захист протоколу: довжина шляху й запиту
admin-settings-help-security-protocol-path-length-mode = Шлях або запит довший за maxPathBytes чи maxQueryBytes. Режим enforce відповідає 414.
admin-settings-key-security-protocol-method-mode = Захист протоколу: методи HTTP
admin-settings-help-security-protocol-method-mode = Метод поза allowedMethods (типово TRACE, TRACK, CONNECT, а також невідомі методи та методи в нижньому регістрі). Режим enforce відповідає 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Захист протоколу: повторені однозначні заголовки
admin-settings-help-security-protocol-duplicate-headers-mode = Заголовок, що має траплятися один раз, але трапляється кілька разів (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, умовні заголовки та заголовки діапазону), повторений Content-Length із рівними значеннями або повторені заголовки Cookie в HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Захист протоколу: заголовки пересилання від недовірених вузлів
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* або X-Real-IP від вузла поза MW_TRUSTED_PROXIES. Ці заголовки в будь-якому разі не вважаються довіреними; параметр фіксує спробу. Враховується в адаптивному захисті лише за налаштованих довірених проксі.
admin-settings-key-security-protocol-header-hygiene-mode = Захист протоколу: гігієна заголовків
admin-settings-help-security-protocol-header-hygiene-mode = Кількість заголовків, одне значення або заголовок Cookie понад налаштовані ліміти; байти не ASCII в заголовках; Upgrade або Connection: upgrade поза маршрутом WebSocket; Connection, що називає заголовки на кшталт Content-Length чи Host. Режим enforce відповідає 431 за перевищення розміру й кількості та 400 в інших випадках.
admin-settings-key-security-protocol-body-mode = Захист протоколу: обрамлення тіла запиту
admin-settings-help-security-protocol-body-mode = GET або HEAD з тілом, Content-Length понад maxBodyBytes (413), Expect, відмінний від 100-continue (417), або без тіла.
admin-settings-key-security-protocol-max-path-bytes = Захист протоколу: найдовший шлях (байти)
admin-settings-help-security-protocol-max-path-bytes = Найдовший шлях запиту, після якого клас довжини шляху повідомляє про порушення.
admin-settings-key-security-protocol-max-query-bytes = Захист протоколу: найдовший рядок запиту (байти)
admin-settings-help-security-protocol-max-query-bytes = Найдовший рядок запиту, після якого клас довжини шляху повідомляє про порушення.
admin-settings-key-security-protocol-max-header-count = Захист протоколу: найбільша кількість заголовків
admin-settings-help-security-protocol-max-header-count = Найбільша кількість полів заголовка в одному запиті, після якої клас гігієни заголовків повідомляє про порушення. Жорсткий ліміт самого слухача (MW_HTTP_MAX_HEADERS) також діє.
admin-settings-key-security-protocol-max-header-value-bytes = Захист протоколу: найдовше значення заголовка (байти)
admin-settings-help-security-protocol-max-header-value-bytes = Найдовше окреме значення заголовка, після якого клас гігієни заголовків повідомляє про порушення.
admin-settings-key-security-protocol-max-cookie-bytes = Захист протоколу: найбільший заголовок Cookie (байти)
admin-settings-help-security-protocol-max-cookie-bytes = Сукупний розмір заголовків Cookie, після якого клас гігієни заголовків повідомляє про порушення.
admin-settings-key-security-protocol-max-body-bytes = Захист протоколу: найбільше заявлене тіло (байти)
admin-settings-help-security-protocol-max-body-bytes = Найбільший Content-Length, який клас тіла приймає без повідомлення. Тримайте його вище за найбільше завантаження чи імпорт, що приймає маршрут; виняток за префіксом маршруту може підняти його для одного маршруту.
admin-settings-key-security-protocol-allowed-methods = Захист протоколу: дозволені методи
admin-settings-help-security-protocol-allowed-methods = Методи, які приймає клас методів. Типовий список містить звичайні дієслова HTTP та дієслова WebDAV, CalDAV і CardDAV; сервер наразі не монтує вхідних маршрутів DAV, тож вони отримують 405 на маршрутизаторі.
admin-settings-key-security-protocol-allowed-hosts = Захист протоколу: додаткові прийняті хости
admin-settings-help-security-protocol-allowed-hosts = Імена хостів, які приймає клас прив'язки хоста на додачу до хоста MW_PUBLIC_URL, наприклад внутрішнє ім'я, яке використовує проксі чи перевірка стану. Порти й кінцева крапка ігноруються.
admin-settings-key-security-protocol-exempt-prefixes = Захист протоколу: виключені префікси маршрутів
admin-settings-help-security-protocol-exempt-prefixes = Префікси маршрутів (кожен починається з / і довший за один символ), для яких евристичні класи не виконуються, для законного клієнта, якому це потрібно, наприклад маршрут великого завантаження. Зіставляються цілі сегменти шляху. Порушення обрамлення та керівних символів у шляху ніколи не виключаються, а / ігнорується.
admin-settings-key-security-health-mode = Доступ до кінцевої точки стану
admin-settings-help-security-health-mode = Хто може звертатися до /healthz і /readyz. internal-only: адреси клієнтів loopback, приватні, link-local і unique-local. allowlist: вони ж і мережі нижче. open: будь-хто. Решта отримує той самий 404, що й для невідомого шляху. Запит із токеном MW_HEALTH_TOKEN допускається завжди. Адреса клієнта — та, що визначена через довірені проксі: за зворотним проксі без MW_FORWARDED_MODE і MW_TRUSTED_PROXIES кожен запит виглядає як приватна адреса проксі, тож допускається весь інтернет.
admin-settings-key-security-health-networks = Мережі кінцевої точки стану
admin-settings-help-security-health-networks = IP-адреси або мережі CIDR, які можуть звертатися до /healthz і /readyz у режимі allowlist, окрім внутрішніх діапазонів, наприклад зовнішній монітор доступності.
admin-settings-key-security-spray-enabled = Виявлення password spray
admin-settings-help-security-spray-enabled = Рахує різні імена користувачів, із якими одне джерело зазнає невдачі. У пам'яті зберігається лише хеш із ключем кожного імені.
admin-settings-key-security-spray-window-secs = Вікно spray (секунди)
admin-settings-help-security-spray-window-secs = Вікно, у якому невдалі входи одного джерела рахуються за різними іменами користувачів. Типово 900.
admin-settings-key-security-spray-challenge-at = Spray: поріг завдання
admin-settings-help-security-spray-challenge-at = Кількість різних імен користувачів із помилками від одного джерела в межах вікна, що вмикає для нього завдання proof-of-work. Успішні входи не враховуються, тож спільна адреса офісу чи кампусу не зачіпається. Типово 10.
admin-settings-key-security-spray-pace-at = Spray: поріг уповільнення
admin-settings-help-security-spray-pace-at = Кількість різних імен користувачів із помилками, що додає додаткове очікування між спробами з джерела. Типово 15.
admin-settings-key-security-spray-lock-at = Spray: поріг блокування
admin-settings-help-security-spray-lock-at = Кількість різних імен користувачів із помилками, що блокує джерело на час блокування через наявне блокування входу. Типово 25.
admin-settings-key-security-spray-pace-secs = Spray: очікування уповільнення (секунди)
admin-settings-help-security-spray-pace-secs = Кількість секунд між спробами з джерела на порозі уповільнення або вище. Типово 10.
admin-settings-key-security-spray-lock-secs = Spray: тривалість блокування (секунди)
admin-settings-help-security-spray-lock-secs = Тривалість тимчасового блокування. Розблокування джерела адміністратором також скидає його лічильник. Типово 900.
admin-settings-key-security-spray-known-multiplier = Spray: множник для відомого джерела
admin-settings-help-security-spray-known-multiplier = Множник трьох порогів для відомого джерела, тобто з нещодавнім успішним входом. Типово 3.
admin-settings-key-security-spray-capacity = Spray: відстежувані джерела
admin-settings-help-security-spray-capacity = Найбільша кількість джерел, що відстежуються в пам'яті; найменш недавно активні видаляються першими. Типово 20000.
admin-settings-key-security-spike-enabled = Виявлення сплесків у всьому екземплярі
admin-settings-help-security-spike-enabled = Вивчає базовий рівень невдалих входів і переходить у підвищений режим, коли весь екземпляр значно його перевищує. Ніколи не блокує екземпляр.
admin-settings-key-security-spike-multiplier-percent = Сплеск: множник (%)
admin-settings-help-security-spike-multiplier-percent = Сигнал враховується, коли він становить не менше цього відсотка вивченого базового рівня (400 = учетверо) і перевищує свою абсолютну нижню межу.
admin-settings-key-security-spike-min-failures-per-min = Сплеск: мінімум невдач за хвилину
admin-settings-help-security-spike-min-failures-per-min = Нижня межа: кількість невдалих входів за хвилину, нижче якої сигнал частоти невдач не враховується. Типово 20.
admin-settings-key-security-spike-min-sources = Сплеск: мінімум джерел із невдачами
admin-settings-help-security-spike-min-sources = Нижня межа: кількість різних джерел із невдачами в п'ятихвилинному інтервалі, нижче якої цей сигнал не враховується. Типово 10.
admin-settings-key-security-spike-min-usernames = Сплеск: мінімум імен користувачів із невдачами
admin-settings-help-security-spike-min-usernames = Нижня межа: кількість різних імен користувачів із невдачами в п'ятихвилинному інтервалі, нижче якої цей сигнал не враховується. Типово 30.
admin-settings-key-security-spike-signals-required = Сплеск: потрібні сигнали
admin-settings-help-security-spike-signals-required = Сигнали, які мають одночасно перевищувати базовий рівень і нижню межу. Розподілена сигнатура spray рахується за два. Типово 2.
admin-settings-key-security-spike-warmup-hours = Сплеск: прогрів (години)
admin-settings-help-security-spike-warmup-hours = Години спостережуваного трафіку, перш ніж може початися підвищений режим. До цього базовий рівень лише навчається. Типово 48.
admin-settings-key-security-spike-adapt-hours = Сплеск: адаптація базового рівня (години)
admin-settings-help-security-spike-adapt-hours = Стала часу базового рівня. Чим більша, тим повільніше він враховує зростання. Сплеск ніколи не оновлює базовий рівень. Типово 168.
admin-settings-key-security-spike-duration-secs = Сплеск: тривалість підвищеного режиму (секунди)
admin-settings-help-security-spike-duration-secs = Як довго триває підвищений режим після останньої ознаки сплеску; він подовжується, поки сигнали лишаються вище половини порога спрацювання. Типово 1800.
admin-settings-key-security-spike-force-challenge = Сплеск: примусове завдання
admin-settings-help-security-spike-force-challenge = У підвищеному режимі вимагає завдання proof-of-work для анонімних входів із невідомих джерел. Автентифіковані користувачі та відомі джерела не зачіпаються.
admin-settings-key-security-spike-pace-factor-percent = Сплеск: очікування під час входу (%)
admin-settings-help-security-spike-pace-factor-percent = У підвищеному режимі очікування під час входу для невідомих джерел у відсотках від звичайного. Типово 300.
admin-settings-key-security-session-binding-enabled = Прив'язка сеансу
admin-settings-help-security-session-binding-enabled = Прив'язує кожен сеанс браузера до хешів із ключем від атрибутів клієнта. Нативні клієнти з токеном bearer, ключі API та клієнти протоколів виключені.
admin-settings-key-security-session-binding-attributes = Прив'язка сеансу: атрибути
admin-settings-help-security-session-binding-attributes = Атрибути прив'язки: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Network типово вимкнено, бо мобільні мережі змінюються. Невідомі імена ігноруються.
admin-settings-key-security-session-binding-strictness = Прив'язка сеансу: суворість
admin-settings-help-security-session-binding-strictness = Tolerant приймає оновлення браузера та зміну одного м'якого атрибута; strict позначає будь-яку м'яку відмінність.
admin-settings-key-security-session-binding-webmail-action = Прив'язка сеансу: дія для вебпошти
admin-settings-help-security-session-binding-webmail-action = Що робить невідповідність сеансу вебпошти: off, detect (лише запис), step-up (повторний запит пароля та другого чинника) або відкликання сеансу. Типово step-up.
admin-settings-key-security-session-binding-admin-action = Прив'язка сеансу: дія для консолі адміністратора
admin-settings-help-security-session-binding-admin-action = Що робить невідповідність сеансу консолі адміністратора: off, detect, step-up або відкликання. Типово відкликання.
admin-settings-key-security-theft-enabled = Виявлення викрадення сеансу
admin-settings-help-security-theft-enabled = Виявляє повторне використання ротованого ідентифікатора сеансу, неможливу подорож і одночасне використання з різних країн.
admin-settings-key-security-theft-rotation-grace-secs = Викрадення: пільговий період ротації (секунди)
admin-settings-help-security-theft-rotation-grace-secs = Після ротації сеансу старий ідентифікатор може пред'являтися повторно протягом цього часу (запит у процесі), перш ніж це вважатимуть викраденням. Типово 30.
admin-settings-key-security-theft-min-travel-secs = Викрадення: мінімальний час подорожі (секунди)
admin-settings-help-security-theft-min-travel-secs = Зміна країни раніше за цей час після попереднього запиту вважається неможливою подорожжю. Визначення країни може бути неточним; VPN і мобільні мережі можуть спричиняти хибні зміни. Типово 3600.
admin-settings-key-security-theft-concurrent-use = Викрадення: одночасне використання
admin-settings-help-security-theft-concurrent-use = Позначає сеанс, що використовується почергово з різних країн (або мереж) протягом п'яти хвилин: off, country або network. Визначення країни може бути неточним; VPN і мобільні мережі можуть спричиняти хибні зміни.
admin-settings-key-security-theft-webmail-action = Викрадення: реакція для вебпошти
admin-settings-help-security-theft-webmail-action = Реакція на неможливу подорож або одночасне використання в сеансі вебпошти: alert, step-up або відкликання. Повторне використання ротованого сеансу завжди його відкликає. Типово step-up.
admin-settings-key-security-theft-admin-action = Викрадення: реакція для консолі адміністратора
admin-settings-help-security-theft-admin-action = Реакція на неможливу подорож або одночасне використання в сеансі консолі адміністратора: alert, step-up або відкликання. Типово відкликання.
admin-settings-key-security-theft-retention-days = Викрадення: зберігання записів (дні)
admin-settings-help-security-theft-retention-days = Як довго зберігаються мітки ротованих сеансів і записи безпеки окремих сеансів; задайте не менше за найдовший строк життя сеансу. Типово 15.
admin-settings-key-security-alerts-country-change = Сповіщення про зміну країни
admin-settings-help-security-alerts-country-change = Попереджає, коли сеанс із виконаним входом використовується з країни, відмінної від тієї, де його розпочато. Визначення країни може бути неточним; VPN і мобільні мережі можуть спричиняти хибні зміни. Користувачі можуть це змінювати, якщо параметр не заблоковано; вимкнення користувачем враховується, лише якщо вимкнення сповіщень дозволено.
admin-settings-key-security-alerts-country-change-enabled = Сповіщення про зміну країни: глобальний перемикач

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-alerts-country-change-enabled = Вимкнення вимикає сповіщення про зміну країни для всіх. Визначення країни може бути неточним; VPN і мобільні мережі можуть спричиняти хибні зміни.
admin-settings-key-security-alerts-country-change-cooldown-secs = Сповіщення про зміну країни: пауза (секунди)
admin-settings-help-security-alerts-country-change-cooldown-secs = Мінімальний час між сповіщеннями про зміну країни в одному сеансі. Кожна нова країна викликає сповіщення один раз за сеанс. Типово 300.
admin-settings-key-security-alerts-country-change-user-mute = Сповіщення про зміну країни: дозволити користувачам вимикати
admin-settings-help-security-alerts-country-change-user-mute = Чи може користувач вимкнути власне сповіщення про зміну країни. Якщо вимкнено, сервер і далі сповіщає, навіть коли власне налаштування користувача вимкнено.
admin-settings-key-security-alerts-channels = Канали сповіщень безпеки
admin-settings-help-security-alerts-channels = Як попередження про безпеку сеансу доходять до користувача: у застосунку, повідомленням у його власній поштовій скриньці або обома способами. Користувачі можуть це змінювати, якщо параметр не заблоковано.
admin-settings-key-security-alerts-email-enabled = Сповіщення безпеки електронною поштою
admin-settings-help-security-alerts-email-enabled = Дозволяє доставляти попередження про безпеку сеансу до власної поштової скриньки користувача. Повідомлення через SMTP не надсилаються; без сеансу доставки попередження лишається в застосунку та журналі аудиту.
admin-settings-key-security-alerts-sensitivity = Чутливість сповіщень безпеки
admin-settings-help-security-alerts-sensitivity = Низька попереджає лише про знахідки щодо місцезнаходження й ніколи не вимагає повторної перевірки та не відкликає; звичайна слідує діям адміністратора; висока подвоює інтервал подорожі й перевіряє також одночасні мережі. Користувачі можуть це змінювати, якщо параметр не заблоковано.
admin-settings-key-security-alerts-notice-retention-days = Повідомлення безпеки: зберігання (дні)
admin-settings-help-security-alerts-notice-retention-days = Як довго повідомлення безпеки в застосунку зберігаються для облікового запису (не більше 20 на обліковий запис). Типово 30.
admin-settings-inline-effective = Діє:
admin-settings-inline-reset = Скасувати зміни
admin-settings-inline-domains = Значення за доменами
admin-attachments-tab-lists = Списки дозволених і блокованих
admin-attachments-tab-limits = Обмеження
admin-attachments-tab-delivery = Завантаження та перегляд
admin-attachments-tab-warnings = Попередження
admin-attachments-tab-test = Перевірка файлу
admin-attachments-tab-denied = У вас немає доступу до цієї вкладки.
admin-attachments-group-lists = Списки дозволених і блокованих
admin-attachments-group-limits = Обмеження розміру
admin-attachments-group-delivery = Завантаження та перегляд
admin-attachments-group-warnings = Попередження
admin-attachments-add-extension = Розширення, наприклад pdf або .tar.gz
admin-attachments-add-mime = Тип MIME, наприклад image/png або image/*
admin-attachments-add = Додати
admin-attachments-remove = Вилучити { $rule }
admin-attachments-filter = Фільтр списку
admin-attachments-list-label = Правила в цьому списку
admin-attachments-count = Правил: { $count } із { $max }
admin-attachments-empty-list = У цьому списку немає правил.
admin-attachments-none-match = Жодне правило не відповідає фільтру.
admin-attachments-bulk = Вставити кілька
admin-attachments-bulk-add = Додати всі
admin-attachments-import = Імпортувати текстовий файл
admin-attachments-export = Експортувати як текст
admin-attachments-err-extension = Це не розширення. Використовуйте літери, цифри, дефіси та підкреслення, до 32 символів; крапки можуть розділяти частини, як у tar.gz.
admin-attachments-err-mime = Це не тип MIME. Використовуйте тип/підтип або тип/* для будь-якого підтипу.
admin-attachments-err-long = Це правило задовге.
admin-attachments-invalid = Не додано, недійсне правило: { $entries }
admin-attachments-duplicates = Уже в списку: { $rules }
admin-attachments-overflow = У списку може бути не більше { $max } правил. Решту не додано.
admin-attachments-conflict-blockedOverAllowed = { $rule } перетинається з { $other } в іншому списку. Блокувальні правила головніші, тому воно блокується.
admin-attachments-conflict-blockedOverWarn = { $rule } перетинається з { $other } в іншому списку. Заблокований файл відхиляється до будь-якого попередження, тому попередження ніколи не з'явиться.
admin-attachments-conflict-allowedAndWarn = { $rule } перетинається з { $other } в іншому списку. Файл дозволено, і він показує попередження.
admin-attachments-bytes-unit = Одиниця
admin-attachments-unlimited = Без обмеження розміру
admin-attachments-bytes-equals = { $size } ({ $bytes } байтів)
admin-attachments-list-allowedExtensions = дозволені розширення
admin-attachments-list-allowedMimeTypes = дозволені типи MIME
admin-attachments-list-blockedExtensions = блоковані розширення
admin-attachments-list-blockedMimeTypes = блоковані типи MIME
admin-attachments-list-warnExtensions = розширення з попередженням
admin-attachments-list-warnMimeTypes = типи MIME з попередженням
admin-attachments-sim-title = Перевірка файлу
admin-attachments-sim-help = Питає сервер, як збережена політика поводиться з файлом із такою назвою, типом і розміром. Нічого не змінюється й не зберігається. Повідомлення та вкладення ніколи не читаються.
admin-attachments-sim-filename = Назва файлу
admin-attachments-sim-mime = Тип MIME
admin-attachments-sim-size = Розмір
admin-attachments-sim-run = Перевірити
admin-attachments-sim-invalid = Введіть назву файлу та розмір, що дорівнює нулю або більший.
admin-attachments-sim-allowed = Дозволено
admin-attachments-sim-warn = Дозволено з попередженням
admin-attachments-sim-blocked = Не доставляється
admin-attachments-sim-preview-yes = Перегляд дозволено.
admin-attachments-sim-preview-no = Перегляд не дозволено.
admin-attachments-sim-checked = Перевірений розмір: { $size }.
admin-attachments-sim-matched = Збіг правил у: { $lists }.
admin-attachments-sim-matched-none = Жодне дозвільне, блокувальне чи попереджувальне правило не збіглося.
admin-attachments-reason-downloadsDisabled = Завантаження вимкнено.
admin-attachments-reason-attachmentTooLarge = Файл більший за обмеження розміру.
admin-attachments-reason-attachmentBlocked = Збігається блокувальне правило.
admin-attachments-reason-attachmentNotAllowed = Існує список дозволених, і нічого в ньому не збігається.
admin-attachments-reason-attachmentContainerRestricted = Вкладені повідомлення та файли TNEF відхиляються за будь-якого ввімкненого обмеження, оскільки їхній вміст не перелічено.
admin-settings-key-mail-reply-images = Зображення у відповідях і пересиланнях
admin-settings-help-mail-reply-images = Чи містить відповідь або пересилання зображення вихідного повідомлення: питати (типово), завжди дозволяти, дозволяти лише для довірених доменів відправників і інакше питати або завжди забороняти. Заблокуйте цей параметр, щоб закріпити його. Зображення читаються лише тоді, коли їх включено.
admin-settings-key-mail-reply-image-trusted-domains = Довірені домени відправників зображень
admin-settings-help-mail-reply-image-trusted-domains = До 200 точних доменів відправників, зображення яких включаються без запиту, коли політика — довірені домени. Користувачі додають до власного списку із запиту про цитату, якщо параметр не заблоковано.
admin-settings-key-mail-send-recall-delay-seconds = Вікно скасування надсилання (типове)
admin-settings-help-mail-send-recall-delay-seconds = Скільки секунд надіслане повідомлення чекає у Вихідних, де відправник може його відкликати, перш ніж його буде доставлено. 0 надсилає одразу. Типово 10, до 1800 (30 хвилин). Користувачі обирають власне значення до наведеного нижче ліміту, якщо ви не заблокуєте параметр.
admin-settings-key-mail-send-recall-delay-max-seconds = Вікно скасування надсилання (ліміт)
admin-settings-help-mail-send-recall-delay-max-seconds = Найдовше вікно скасування надсилання, яке може обрати користувач, у секундах. Сервер скорочує довший запит до цього значення. Типово й максимум 1800.
admin-settings-key-mail-send-allow-recall = Дозволити відкликання надісланих повідомлень
admin-settings-help-mail-send-allow-recall = Якщо вимкнено, повідомлення йдуть одразу, і їх не можна відкликати з Вихідних. Відкладені надсилання не зачіпаються; їх і далі можна скасувати до настання часу.
admin-settings-key-mail-send-max-delayed-seconds = Горизонт планування
admin-settings-help-mail-send-max-delayed-seconds = На скільки наперед можна запланувати повідомлення, у секундах. Типово 7776000 (90 днів), максимум 31536000 (1 рік). 0 вимикає відкладене надсилання.
admin-settings-key-mail-send-max-queued = Повідомлень у черзі на скриньку
admin-settings-help-mail-send-max-queued = Скільки повідомлень одна скринька може одночасно тримати у Вихідних (вікно скасування або відкладені). Типово 200. Подальші надсилання відхиляються з зрозумілим повідомленням і ніколи не губляться.
admin-attachments-sim-pick = Заповнити з файлу (необов'язково)
admin-attachments-mime-toggle = Показати поширені типи MIME
admin-attachments-mime-hint = Виберіть поширений тип або введіть будь-який інший тип/підтип.
admin-attachments-mime-invalid = Це не тип/підтип на кшталт text/plain. Його буде надіслано як введено.
admin-attachments-mime-group-documents = Документи
admin-attachments-mime-group-images = Зображення
admin-attachments-mime-group-archives = Архіви
admin-attachments-mime-group-executables = Виконувані файли та скрипти
admin-attachments-mime-group-media = Аудіо та відео
admin-attachments-mime-group-email = Ел. пошта й календар
admin-attachments-mime-group-other = Інше
admin-settings-key-cookies-enabled = Керування згодою на cookie
admin-settings-help-cookies-enabled = Показувати відвідувачам елемент керування згодою на cookie. Вимкнення зберігає поведінку застосунку без цього елемента.
admin-settings-key-cookies-mode = Модель згоди
admin-settings-help-cookies-mode = optIn: нічого необов'язкового не зберігається, доки відвідувач не прийме. optOut: діють типові значення категорій, доки відвідувач не відхилить. noticeOnly: відвідувача інформують без вибору.
admin-settings-key-cookies-region-mode = Модель згоди в суворих регіонах
admin-settings-help-cookies-region-mode = Модель, яку використовують для відвідувачів із суворих регіонів. same означає, що й там діє модель згоди вище.
admin-settings-key-cookies-strict-regions = Суворі регіони
admin-settings-help-cookies-strict-regions = Коди країн ISO 3166-1, що використовують модель згоди для суворих регіонів. Типово держави ЄС і ЄЕП, Велика Британія та Швейцарія.
admin-settings-key-cookies-region-header = Заголовок країни
admin-settings-help-cookies-region-header = Назва заголовка запиту, що його встановлює проксі чи CDN, у якому передається код країни відвідувача.
admin-settings-key-cookies-unknown-region = Невідома країна
admin-settings-help-cookies-unknown-region = Яка модель застосовується, коли країну визначити не можна: модель суворих регіонів чи звичайна.
admin-settings-key-cookies-functional-enabled = Пропонувати функційні cookie
admin-settings-help-cookies-functional-enabled = Пропонувати в елементі згоди категорію налаштувань і функцій (тема, мова, запам'ятовування розкладки).
admin-settings-key-cookies-functional-default = Функційні типово
admin-settings-help-cookies-functional-default = Початковий стан функційної категорії, коли модель згоди допускає типові значення.
admin-settings-key-cookies-analytics-enabled = Пропонувати аналітичні cookie
admin-settings-help-cookies-analytics-enabled = Пропонувати в елементі згоди аналітичну категорію.
admin-settings-key-cookies-analytics-default = Аналітичні типово
admin-settings-help-cookies-analytics-default = Початковий стан аналітичної категорії, коли модель згоди допускає типові значення.
admin-settings-key-cookies-marketing-enabled = Пропонувати маркетингові cookie
admin-settings-help-cookies-marketing-enabled = Пропонувати в елементі згоди маркетингову категорію.
admin-settings-key-cookies-marketing-default = Маркетингові типово
admin-settings-help-cookies-marketing-default = Початковий стан маркетингової категорії, коли модель згоди допускає типові значення.
admin-settings-key-cookies-layout = Макет
admin-settings-help-cookies-layout = banner — смуга на всю ширину сторінки, card — плаваюча картка, modal — діалог, що утримує сторінку, доки відвідувач не зробить вибір.
admin-settings-key-cookies-position = Розташування
admin-settings-help-cookies-position = Де на сторінці з'являється елемент згоди.
admin-settings-key-cookies-reject-prominent = Відхилення так само помітне, як прийняття
admin-settings-help-cookies-reject-prominent = Показувати «Відхилити все» того самого розміру й стилю, що й «Прийняти все».
admin-settings-key-cookies-policy-url = URL політики cookie
admin-settings-help-cookies-policy-url = Посилання на вашу політику cookie, що показується в елементі згоди.
admin-settings-key-cookies-privacy-url = URL політики конфіденційності
admin-settings-help-cookies-privacy-url = Посилання на вашу політику конфіденційності, що показується в елементі згоди.
admin-settings-key-cookies-policy-version = Версія політики
admin-settings-help-cookies-policy-version = Збільште це число, коли змінюється використання cookie, щоб відвідувачів запитали знову.
admin-settings-key-cookies-lifetime-days = Строк дії згоди
admin-settings-help-cookies-lifetime-days = Кількість днів, протягом яких рішення лишається чинним, перш ніж відвідувача запитають знову.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-cookies-reprompt-on-version = Питати знову за нової версії політики
admin-settings-help-cookies-reprompt-on-version = Питати відвідувачів знову, коли версія політики вища за ту, за якої вони ухвалили рішення.
admin-settings-key-cookies-honour-gpc = Враховувати Global Privacy Control
admin-settings-help-cookies-honour-gpc = Вважати сигнал Global Privacy Control від браузера відмовою від аналітики та маркетингу.
admin-settings-key-cookies-honour-dnt = Враховувати Do Not Track
admin-settings-help-cookies-honour-dnt = Вважати сигнал Do Not Track від браузера відмовою від аналітики та маркетингу.
admin-settings-key-cookies-log-consent = Вести журнал згод
admin-settings-help-cookies-log-consent = Зберігати на сервері запис кожного рішення про згоду для статистики та власного обліку.
admin-settings-key-cookies-log-retention-days = Зберігання журналу згод
admin-settings-help-cookies-log-retention-days = Кількість днів, протягом яких рядки журналу згод зберігаються до видалення.
admin-settings-key-cookies-scan-interval-hours = Інтервал сканування сховища
admin-settings-help-cookies-scan-interval-hours = Години між автоматичними скануваннями cookie та сховища браузера. 0 сканує лише на запит.
admin-settings-key-cookies-unclassified = Некласифіковане сховище
admin-settings-help-cookies-unclassified = report залишає сховище, яке ніщо не класифікує, недоторканим і виводить його на перегляд. Інші значення відносять його до відповідної категорії, доки ви його не класифікуєте.
admin-settings-key-cookies-text-title = Заголовок елемента згоди
admin-settings-help-cookies-text-title = Заголовок елемента згоди для кожної мови. Порожньо — вбудований текст.
admin-settings-key-cookies-text-body = Текст елемента згоди
admin-settings-help-cookies-text-body = Пояснювальний текст елемента згоди для кожної мови. Порожньо — вбудований текст.
admin-settings-key-cookies-label-necessary = Назва категорії необхідних
admin-settings-help-cookies-label-necessary = Назва категорії необхідних для кожної мови. Порожньо — вбудована назва.
admin-settings-key-cookies-description-necessary = Опис категорії необхідних
admin-settings-help-cookies-description-necessary = Опис категорії необхідних для кожної мови. Порожньо — вбудований опис.
admin-settings-key-cookies-label-functional = Назва категорії функційних
admin-settings-help-cookies-label-functional = Назва категорії функційних для кожної мови. Порожньо — вбудована назва.
admin-settings-key-cookies-description-functional = Опис категорії функційних
admin-settings-help-cookies-description-functional = Опис категорії функційних для кожної мови. Порожньо — вбудований опис.
admin-settings-key-cookies-label-analytics = Назва категорії аналітичних
admin-settings-help-cookies-label-analytics = Назва категорії аналітичних для кожної мови. Порожньо — вбудована назва.
admin-settings-key-cookies-description-analytics = Опис категорії аналітичних
admin-settings-help-cookies-description-analytics = Опис категорії аналітичних для кожної мови. Порожньо — вбудований опис.
admin-settings-key-cookies-label-marketing = Назва категорії маркетингових
admin-settings-help-cookies-label-marketing = Назва категорії маркетингових для кожної мови. Порожньо — вбудована назва.
admin-settings-key-cookies-description-marketing = Опис категорії маркетингових
admin-settings-help-cookies-description-marketing = Опис категорії маркетингових для кожної мови. Порожньо — вбудований опис.
admin-settings-key-encryption-rotation-days = Інтервал ротації ключа бази даних (дні)
admin-settings-help-encryption-rotation-days = Кількість днів після останньої зміни ключа, після яких ключ бази даних позначається як такий, що потребує ротації. 0 не задає строку. Сам ключ ніколи не зберігається в налаштуваннях.
admin-settings-key-encryption-warnings-enabled = Повідомляти про знахідки щодо шифрування
admin-settings-help-encryption-warnings-enabled = Показувати сховища у відкритому вигляді та прострочену ротацію ключа як знахідку в оцінці розгортання.
admin-settings-key-retention-sweeper-enabled = Очищення за строками зберігання
admin-settings-help-retention-sweeper-enabled = Запускати щоденне очищення, що видаляє рядки старші за обмеження на цій сторінці. Вимкнення зберігає всі рядки до повторного ввімкнення.
admin-settings-key-retention-sweeper-batch-rows = Рядків у партії
admin-settings-help-retention-sweeper-batch-rows = Скільки рядків видаляє один оператор видалення. Менші партії тримають блокування бази даних менше часу.
admin-settings-key-retention-sweeper-max-batches = Партій на сховище за запуск
admin-settings-help-retention-sweeper-max-batches = Найбільша кількість операторів видалення, яку сховище отримує за запуск. Сховище з більшою кількістю старих рядків продовжить у наступному запуску.
admin-settings-key-retention-sweeper-jitter-minutes = Розкид часу запуску (хвилини)
admin-settings-help-retention-sweeper-jitter-minutes = Наступний запуск починається через випадкову кількість хвилин, до цього значення, після 24 годин, щоб кілька серверів не очищали одночасно.
admin-settings-key-retention-sweeper-vacuum-after-rows = Звільняти місце після (рядків)
admin-settings-help-retention-sweeper-vacuum-after-rows = Якщо один запуск видалив щонайменше стільки рядків, базу даних ущільнюють (VACUUM). 0 ніколи не ущільнює після очищення; щотижневе завдання vacuum окреме.
admin-settings-key-retention-password-change-audit-days = Записи про зміну пароля (дні)
admin-settings-help-retention-password-change-audit-days = Кількість днів зберігання запису про зміну пароля (обліковий запис, спосіб, результат). 0 зберігає їх безстроково.
admin-settings-key-retention-sso-login-audit-days = Записи про входи через єдиний вхід (дні)
admin-settings-help-retention-sso-login-audit-days = Кількість днів зберігання запису про вхід через єдиний вхід (провайдер, хеш суб'єкта, результат). 0 зберігає їх безстроково.
admin-settings-key-retention-dlp-audit-days = Записи запобігання витоку даних (дні)
admin-settings-help-retention-dlp-audit-days = Кількість днів зберігання запису про спрацювання правила запобігання витоку даних. 0 зберігає їх безстроково.
admin-settings-key-retention-assist-audit-days = Записи доступу помічника (дні)
admin-settings-help-retention-assist-audit-days = Кількість днів зберігання запису про використання помічником можливості (суб'єкт, можливість, підсумок області). 0 зберігає їх безстроково.
admin-settings-key-retention-alert-events-days = Записи спрацьованих сповіщень (дні)
admin-settings-help-retention-alert-events-days = Кількість днів зберігання запису про спрацьоване сповіщення адміністратора. 0 зберігає їх безстроково.
admin-settings-key-retention-job-runs-days = Історія фонових завдань (дні)
admin-settings-help-retention-job-runs-days = Кількість днів зберігання завершеного запуску фонового завдання. Запуск, що ще триває, ніколи не видаляється. 0 зберігає історію безстроково.
admin-settings-key-retention-outbox-days = Записи надісланих і скасованих надсилань (дні)
admin-settings-help-retention-outbox-days = Кількість днів зберігання запису про надсилання доставленого або скасованого повідомлення. Повідомлення, що очікує, повторюється, не вдалося або невизначене, ніколи не видаляється. 0 зберігає їх безстроково.
admin-settings-key-retention-expired-credentials-grace-days = Прострочені облікові дані (дні після спливу)
admin-settings-help-retention-expired-credentials-grace-days = Кількість днів зберігання після спливу для запам'ятованого пристрою, завдання входу, токена входу, токена OAuth, nonce вебхука, лічильника збоїв джерела або дескриптора посилання на зображення. 0 видаляє їх під час наступного запуску.
admin-settings-key-retention-ip-addresses-days = Скорочувати збережені IP-адреси через (дні)
admin-settings-help-retention-ip-addresses-days = Через стільки днів IP-адреса в історії автентифікації та записах аудиту обрізається або хешується, а рядок зберігається. 0 зберігає адреси як записано.
admin-settings-key-retention-ip-addresses-mode = Спосіб скорочення IP-адрес
admin-settings-help-retention-ip-addresses-mode = Обрізання зберігає префікс IPv4 /24 або IPv6 /48. Хеш замінює адресу коротким хешем із ключем, який усе ще групує ту саму адресу.
admin-settings-key-retention-user-enabled = Користувачі можуть задавати правила зберігання
admin-settings-help-retention-user-enabled = Користувачі можуть задавати власні правила зберігання для тек і пошти. Якщо вимкнено, наявні користувацькі правила перестають виконуватися; загальні правила організації нижче й далі працюють.
admin-settings-key-retention-user-min-days = Найменша кількість днів до видалення пошти правилом користувача
admin-settings-help-retention-user-min-days = Правило користувача, що видаляє пошту, не може діяти на пошту молодшу за це значення. Менше значення відхиляється під час збереження правила й підвищується під час його виконання.
admin-settings-key-retention-user-max-days = Найбільша кількість днів очікування правила користувача перед видаленням пошти
admin-settings-help-retention-user-max-days = Правило користувача, що видаляє пошту, не може чекати довше за це значення. 0 не задає верхньої межі. Більше значення відхиляється під час збереження правила й знижується під час його виконання.
admin-settings-key-retention-user-max-rules = Найбільша кількість правил зберігання на користувача
admin-settings-help-retention-user-max-rules = Найбільша кількість правил зберігання, яку може мати один користувач.
admin-settings-key-retention-user-allow-permanent-delete = Правила користувачів можуть видаляти пошту безповоротно
admin-settings-help-retention-user-allow-permanent-delete = Якщо вимкнено, правило видалення завжди переміщує пошту до Кошика, і перед остаточним видаленням діє вікно скасування.
admin-settings-key-retention-user-undo-days = Вікно скасування для правил зберігання (дні)
admin-settings-help-retention-user-undo-days = Кількість днів, протягом яких пошта, переміщена правилом зберігання до Кошика, лишається там до остаточного видалення. Користувач може обрати коротше вікно. 0 видаляє під час наступного запуску.
admin-settings-key-retention-user-mandatory = Загальні правила пошти організації
admin-settings-help-retention-user-mandatory = Правила, які отримує кожен обліковий запис, у вигляді записів роль-теки:дні, наприклад trash:30 або junk:30. Пошта в цій теці старша за вказану кількість днів видаляється остаточно. Користувач не може змінювати, призупиняти їх або виключати з них пошту. Пошта облікового запису під юридичним утриманням пропускається.
admin-settings-key-retention-user-protected-folders = Теки, яких правила зберігання ніколи не торкаються
admin-settings-help-retention-user-protected-folders = Теки, з якими не працює жодне правило користувача, за роллю (drafts, sent, inbox, archive) або за назвою теки. Захищена тека головніша за загальне правило організації для тієї самої теки.
admin-settings-key-retention-user-templates = Шаблони зберігання, що пропонуються користувачам
admin-settings-help-retention-user-templates = Шаблони, які користувачі можуть прийняти в Налаштуваннях, за ідентифікатором: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Виконувати правила користувачів для облікових записів з обмеженням
admin-settings-help-retention-user-run-for-restricted = Чи виконуються користувацькі правила зберігання також для облікових записів з обмеженням обробки. Вимкнено — ці облікові записи пропускаються.
admin-settings-key-retention-user-interval-hours = Годин між запусками зберігання користувачів
admin-settings-help-retention-user-interval-hours = Години між плановими запусками завдання зберігання користувачів для одного облікового запису.
admin-settings-key-retention-user-batch-messages = Повідомлень на виклик зберігання
admin-settings-help-retention-user-batch-messages = Скільки повідомлень обробляє один виклик переміщення чи видалення. Використовується не більше 200.
admin-settings-key-retention-user-max-per-run = Найбільша кількість змінюваних повідомлень на обліковий запис за запуск
admin-settings-help-retention-user-max-per-run = Найбільша кількість повідомлень, яку завдання зберігання користувачів змінює для одного облікового запису за один запуск. Решта чекає наступного запуску.
admin-settings-key-retention-user-pause-ms = Пауза між партіями зберігання (мс)
admin-settings-help-retention-user-pause-ms = Пауза між партіями в мілісекундах, щоб велике очищення не навантажувало поштовий сервер.
admin-settings-key-retention-class-short-days = Верхня межа короткого класу (дні)
admin-settings-help-retention-class-short-days = Верхня межа для оперативних даних: сеансів, токенів, лімітів запитів, історії завдань і метрик. Застосовується, коли обмеження класу ввімкнено.
admin-settings-key-retention-class-medium-days = Верхня межа середнього класу (дні)
admin-settings-help-retention-class-medium-days = Верхня межа для історії входів, історії захисту запитів і повідомлень безпеки. Не повинна бути нижчою за короткий клас; менше значення підвищується до нього.
admin-settings-key-retention-class-long-days = Зберігання довгого класу (дні)
admin-settings-help-retention-class-long-days = Кількість днів зберігання довгого класу (аудит безпеки, дії адміністраторів, запити, згоди, надгробки), включно з архівом. 0 зберігає безстроково, і консоль попереджає. Не повинно бути нижчим за середній клас.
admin-settings-key-retention-class-enforce = Утримувати кожне сховище в межі його класу
admin-settings-help-retention-class-enforce = Якщо ввімкнено, власне обмеження кожного сховища обмежується верхньою межею його класу, а 0 (зберігати) вважається верхньою межею. Якщо вимкнено, діють обмеження окремих сховищ.
admin-settings-key-retention-archive-enabled = Архівувати застарілі рядки довгого класу
admin-settings-help-retention-archive-enabled = Переносити застарілі рядки довгого класу в запечатані стиснені файли архіву замість видалення.
admin-settings-key-retention-archive-directory = Каталог архіву
admin-settings-help-retention-archive-directory = Каталог файлів архіву, абсолютний або відносно робочого каталогу сервера. Сегменти батьківського каталогу (..) відхиляються.
admin-settings-key-retention-archive-batch-rows = Рядків у файлі архіву
admin-settings-help-retention-archive-batch-rows = Кількість рядків, що записуються в один файл архіву.
admin-settings-key-retention-archive-max-segments = Файлів архіву на сховище за запуск
admin-settings-help-retention-archive-max-segments = Скільки файлів архіву один запуск може записати для кожного сховища. Решта чекає наступного запуску.
admin-settings-key-retention-archive-key-generation = Покоління ключа запечатування архіву
admin-settings-help-retention-archive-key-generation = Покоління ключа, що запечатує нові файли архіву. Збільште його для ротації ключа; попередні файли й далі перевіряються власним поколінням.
admin-settings-key-retention-archive-verify-days = Днів між перевірками архіву
admin-settings-help-retention-archive-verify-days = Кількість днів між перевірками цілісності файлів архіву.
admin-settings-key-retention-archive-legal-hold = Юридичне утримання архіву
admin-settings-help-retention-archive-legal-hold = Зберігати кожен файл архіву незалежно від обмеження довгого класу.
admin-settings-key-logging-file-enabled = Записувати файли журналу
admin-settings-help-logging-file-enabled = Також записувати журнал сервера в ротовані файли. Вимкнено — журналювання лише в стандартний вивід, що зазвичай і використовує контейнер.
admin-settings-key-logging-file-directory = Каталог журналів
admin-settings-help-logging-file-directory = Каталог файлів журналу, абсолютний або відносно робочого каталогу сервера. Сегменти батьківського каталогу (..) відхиляються.
admin-settings-key-logging-file-max-size-mb = Ротація за розміром (МБ)
admin-settings-help-logging-file-max-size-mb = Поточний файл журналу закривається й починається новий, коли він сягає цього розміру.
admin-settings-key-logging-file-rotate-hours = Ротація за віком (години)
admin-settings-help-logging-file-rotate-hours = Поточний файл журналу ротується, коли йому виповнюється стільки годин. 0 ротує лише за розміром.
admin-settings-key-logging-file-max-age-days = Зберігати ротовані файли (дні)
admin-settings-help-logging-file-max-age-days = Ротовані файли журналу старші за це значення видаляються.
admin-settings-key-logging-file-max-files = Найбільша кількість ротованих файлів
admin-settings-help-logging-file-max-files = Найстарші ротовані файли понад цю кількість видаляються.
admin-settings-key-logging-file-max-total-mb = Найбільший загальний розмір (МБ)
admin-settings-help-logging-file-max-total-mb = Найстарші ротовані файли видаляються, доки каталог журналів не стане меншим за цей розмір.
admin-settings-key-logging-file-compress = Стискати ротовані файли
admin-settings-help-logging-file-compress = Ротовані файли журналу стискаються gzip.
admin-settings-key-logging-level = Рівень журналу
admin-settings-help-logging-level = Рівень для компонентів, що не мають нижче власного рівня. Змінна середовища MW_LOG задає рівень під час запуску, доки цей параметр не прочитано.
admin-settings-key-logging-component-levels = Рівні журналу компонентів
admin-settings-help-logging-component-levels = Один запис компонент=рівень у рядку, наприклад mw_imap=debug. Рівні: error, warn, info, debug і trace.
admin-settings-key-logging-redaction-addresses = Адреси ел. пошти в журналах
admin-settings-help-logging-redaction-addresses = Full залишає адреси як записано. Truncated зберігає першу літеру й домен. Hashed замінює локальну частину коротким хешем. None видаляє адресу.
admin-settings-key-logging-redaction-ips = IP-адреси в журналах
admin-settings-help-logging-redaction-ips = Full залишає адреси як записано. Truncated зберігає префікс IPv4 /24 або IPv6 /48. Hashed замінює адресу коротким хешем. None видаляє її.
admin-settings-key-logging-redaction-tokens = Секрети в журналах
admin-settings-help-logging-redaction-tokens = Вирізає з рядків журналу токени bearer, ключі API, паролі в URL і подібні секрети.
admin-settings-key-retention-sessions-idle-days = Невикористані сеанси (дні)
admin-settings-help-retention-sessions-idle-days = Скільки днів запис сеансу браузера чи нативного клієнта, з її адресою та user-agent, може лишатися невикористаним до видалення. Спочатку діють тайм-аути сеансів; це лише видаляє залишені рядки. 0 зберігає їх.
admin-settings-key-policy-public-pages = Публічні сторінки політик
admin-settings-help-policy-public-pages = Віддавати згенеровану політику конфіденційності за адресою /privacy і політику cookie за адресою /cookies-policy без входу.
admin-settings-key-policy-auto-version = Автоматичні версії політик
admin-settings-help-policy-auto-version = Записувати нову версію політики, коли змінюється налаштування, яке описує політика. Вимкнено: зміни чекають публікації.
admin-settings-key-policy-reconsent-on-change = Знову запитувати згоду на cookie за нової версії політики
admin-settings-help-policy-reconsent-on-change = Підвищувати версію політики cookie під час запису нової версії політики, щоб відвідувачів запитали знову (потрібно cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Назва контролера даних
admin-settings-help-policy-controller-name = Назва організації, що визначає цілі та способи обробки даних, показується в політиці конфіденційності.
admin-settings-key-policy-controller-address = Адреса контролера даних
admin-settings-help-policy-controller-address = Поштова адреса контролера даних, показується в політиці конфіденційності.
admin-settings-key-policy-legal-bases = Правові підстави
admin-settings-help-policy-legal-bases = Правові підстави, на які ви спираєтеся, своїми словами, показуються в політиці конфіденційності.
admin-settings-key-policy-hosting-provider = Хостинг-провайдер
admin-settings-help-policy-hosting-provider = Хто й де розміщує дані, показується в політиці конфіденційності.
admin-settings-key-policy-hosting-transfers = Передавання за межі країни хостингу
admin-settings-help-policy-hosting-transfers = Будь-які передавання даних до інших країн і застосовувані гарантії, показуються в політиці конфіденційності.
admin-settings-key-policy-supervisory-authority = Наглядовий орган
admin-settings-help-policy-supervisory-authority = Орган, до якого користувачі можуть подати скаргу, показується в політиці конфіденційності.
admin-settings-key-dpia-suggest = Пропонувати оцінки впливу
admin-settings-help-dpia-suggest = Показувати запропоновану оцінку впливу, коли ввімкнено помічника, відстеження відкриттів або аналітику, а оцінки немає.
admin-settings-key-dpia-review-months = Інтервал перегляду оцінки впливу
admin-settings-help-dpia-review-months = Кількість місяців до дати перегляду, що задається під час затвердження оцінки.
admin-settings-key-dpia-reminder-days = Вікно нагадування про оцінку впливу
admin-settings-help-dpia-reminder-days = За скільки днів до дати перегляду оцінка виводиться як така, що потребує перегляду.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-restriction-enabled = Обмеження обробки
admin-settings-help-restriction-enabled = Чи може адміністратор або схвалений запит обмежити обробку даних особи.
admin-settings-key-restriction-auto-apply-on-approval = Обмежувати після схвалення запиту
admin-settings-help-restriction-auto-apply-on-approval = Застосовувати обмеження одразу після схвалення запиту на обмеження. Вимкнено: адміністратор застосовує його в розділі Відповідність > Обмеження.
admin-settings-key-restriction-mail-delivery = Нова пошта під час обмеження
admin-settings-help-restriction-mail-delivery = Що адміністратор обрав для пошти, що надходить за чинного обмеження: зберігати, ставити в чергу або відхиляти. Цей сервер не приймає пошти; значення записується й показується, а діє за ним поштовий сервер.
admin-settings-key-restriction-default-expiry-days = Типовий строк обмеження (дні)
admin-settings-help-restriction-default-expiry-days = Кількість днів, після яких нове обмеження закінчується саме. 0 зберігає його, доки хтось не зніме.
admin-settings-key-restriction-notify-user = Сповіщати особу
admin-settings-help-restriction-notify-user = Надсилати особі сповіщення, коли обмеження застосовується або знімається.
admin-settings-key-restriction-scope-indexing = Призупиняти індексування пошуку та ембединги
admin-settings-help-restriction-scope-indexing = Чи призупинено індексування пошуку та ембединги типово в новому обмеженні.
admin-settings-key-restriction-scope-assistant = Призупиняти помічника
admin-settings-help-restriction-scope-assistant = Чи призупинено помічника типово в новому обмеженні.
admin-settings-key-restriction-scope-analytics = Призупиняти аналітику й відстеження відкриттів
admin-settings-help-restriction-scope-analytics = Чи призупинено аналітику й відстеження відкриттів типово в новому обмеженні.
admin-settings-key-restriction-scope-classification = Призупиняти правила й автоматичне сортування
admin-settings-help-restriction-scope-classification = Чи призупинено правила й автоматичне сортування типово в новому обмеженні.
admin-settings-key-restriction-scope-push = Призупиняти push-сповіщення
admin-settings-help-restriction-scope-push = Чи призупинено push-сповіщення типово в новому обмеженні.
admin-settings-key-restriction-scope-third-party = Призупиняти підключені сервіси та плагіни
admin-settings-help-restriction-scope-third-party = Чи призупинено підключені сервіси та плагіни типово в новому обмеженні.
admin-settings-key-consent-enabled = Згода за цілями обробки
admin-settings-help-consent-enabled = Показувати користувачам цілі та варіанти згоди й застосовувати їх під час роботи.
admin-settings-key-consent-receipts-retention-days = Зберігання квитанцій про згоду (дні)
admin-settings-help-consent-receipts-retention-days = Кількість днів зберігання квитанцій про згоду. 0 зберігає їх до видалення даних особи.
admin-settings-key-consent-art22-notice = Повідомлення про автоматизоване оброблення
admin-settings-help-consent-art22-notice = Показувати користувачам просте повідомлення про функції помічника та пошуку з перемикачами для кожної функції.
admin-settings-key-consent-analytics-basis = Правова підстава: лічильники використання
admin-settings-help-consent-analytics-basis = Правова підстава, що показується користувачам для лічильників використання. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-analytics-version = Версія тексту: лічильники використання
admin-settings-help-consent-analytics-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-analytics-text = Власне формулювання: лічильники використання
admin-settings-help-consent-analytics-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-analytics-tag-basis = Правова підстава: скрипт провайдера аналітики
admin-settings-help-consent-analytics-tag-basis = Правова підстава, що показується користувачам для скрипта провайдера аналітики. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-analytics-tag-version = Версія тексту: скрипт провайдера аналітики
admin-settings-help-consent-analytics-tag-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-analytics-tag-text = Власне формулювання: скрипт провайдера аналітики
admin-settings-help-consent-analytics-tag-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-assistant-basis = Правова підстава: помічник
admin-settings-help-consent-assistant-basis = Правова підстава, що показується користувачам для помічника. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-assistant-version = Версія тексту: помічник
admin-settings-help-consent-assistant-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-assistant-text = Власне формулювання: помічник
admin-settings-help-consent-assistant-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-push-basis = Правова підстава: push-сповіщення
admin-settings-help-consent-push-basis = Правова підстава, що показується користувачам для push-сповіщень. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-push-version = Версія тексту: push-сповіщення
admin-settings-help-consent-push-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-push-text = Власне формулювання: push-сповіщення
admin-settings-help-consent-push-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-open-tracking-basis = Правова підстава: відстеження відкриттів
admin-settings-help-consent-open-tracking-basis = Правова підстава, що показується користувачам для відстеження відкриттів. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-open-tracking-version = Версія тексту: відстеження відкриттів
admin-settings-help-consent-open-tracking-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-open-tracking-text = Власне формулювання: відстеження відкриттів
admin-settings-help-consent-open-tracking-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-remote-content-basis = Правова підстава: віддалені зображення
admin-settings-help-consent-remote-content-basis = Правова підстава, що показується користувачам для віддалених зображень. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-remote-content-version = Версія тексту: віддалені зображення
admin-settings-help-consent-remote-content-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-remote-content-text = Власне формулювання: віддалені зображення
admin-settings-help-consent-remote-content-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-integrations-basis = Правова підстава: підключені сервіси
admin-settings-help-consent-integrations-basis = Правова підстава, що показується користувачам для підключених сервісів. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-integrations-version = Версія тексту: підключені сервіси
admin-settings-help-consent-integrations-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-integrations-text = Власне формулювання: підключені сервіси
admin-settings-help-consent-integrations-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-consent-telemetry-basis = Правова підстава: вимірювання часу завантаження
admin-settings-help-consent-telemetry-basis = Правова підстава, що показується користувачам для вимірювань часу завантаження. Лише згода змушує функцію чекати «так»; інші підстави діють, доки особа їх не відкличе. Яка підстава слушна, вирішує адміністратор.
admin-settings-key-consent-telemetry-version = Версія тексту: вимірювання часу завантаження
admin-settings-help-consent-telemetry-version = Збільште це число, коли змінюється формулювання чи обробка; осіб, що погодилися зі старішою версією, запитають знову.
admin-settings-key-consent-telemetry-text = Власне формулювання: вимірювання часу завантаження
admin-settings-help-consent-telemetry-text = Ваш власний текст для цієї цілі для кожної мови. Порожньо — вбудоване формулювання.
admin-settings-key-breach-enabled = Реєстр порушень
admin-settings-help-breach-enabled = Вмикає реєстр і відлік часу на повідомлення.
admin-settings-key-breach-notice-hours = Годин на повідомлення органу
admin-settings-help-breach-notice-hours = Кількість годин від моменту, коли стало відомо про порушення, до повідомлення органу. Зазвичай 72.
admin-settings-key-breach-reminders-hours = Час нагадувань
admin-settings-help-breach-reminders-hours = Кількість годин до строку, за яких створюється нагадування, наприклад 6, 24 і 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Повтор після строку
admin-settings-help-breach-reminders-overdue-repeat-hours = Години між нагадуваннями після спливу строку. 0 нагадує один раз.
admin-settings-key-breach-authority-name = Наглядовий орган
admin-settings-help-breach-authority-name = Назва, що друкується в повідомленні органу.
admin-settings-key-breach-authority-url = Сторінка повідомлень органу
admin-settings-help-breach-authority-url = Адреса сторінки повідомлень органу.
admin-settings-key-breach-authority-contact = Контакт органу
admin-settings-help-breach-authority-contact = Контактні дані органу.
admin-settings-key-breach-dpo-name = Ім'я контакту із захисту даних
admin-settings-help-breach-dpo-name = Ім'я, що зазначається як контактна особа в повідомленнях.
admin-settings-key-breach-dpo-email = Ел. пошта контакту із захисту даних
admin-settings-help-breach-dpo-email = Адреса, що зазначається як контакт у повідомленнях.
admin-settings-key-breach-dpo-phone = Телефон контакту із захисту даних
admin-settings-help-breach-dpo-phone = Номер телефону, що зазначається в повідомленнях.
admin-settings-key-breach-template-subject = Тема повідомлення
admin-settings-help-breach-template-subject = Тема повідомлення постраждалим особам для кожної мови.
admin-settings-key-breach-template-body = Текст повідомлення
admin-settings-help-breach-template-body = Текст повідомлення постраждалим особам для кожної мови.
admin-settings-key-breach-notify-audit = Записувати сповіщення в журнал аудиту
admin-settings-help-breach-notify-audit = Нагадування та виявлення стають подіями безпеки.
admin-settings-key-breach-notify-webhook-urls = Адреси вебхуків
admin-settings-help-breach-notify-webhook-urls = Адреси, яким повідомляється про нагадування та повідомлення. Ставтеся до них як до секретів; кожен виклик підписано.
admin-settings-key-breach-notify-operator-addresses = Скриньки операторів
admin-settings-help-breach-notify-operator-addresses = Адреси локальних скриньок, що отримують нагадування у вхідні.
admin-settings-key-breach-subjects-in-app = Сповіщення в обліковому записі особи
admin-settings-help-breach-subjects-in-app = Показувати повідомлення всередині облікового запису особи.
admin-settings-key-breach-subjects-inbox = Повідомлення до скриньки особи
admin-settings-help-breach-subjects-inbox = Доставляти повідомлення до власної скриньки особи.
admin-settings-key-breach-subjects-webhook = Також надсилати на вебхуки
admin-settings-help-breach-subjects-webhook = Надсилати текст повідомлення й на адреси вебхуків.
admin-settings-key-breach-retention-years = Зберігання реєстру в роках
admin-settings-help-breach-retention-years = Кількість років, протягом яких закритий інцидент лишається в реєстрі.
admin-settings-key-breach-auto-enabled = Автоматизації
admin-settings-help-breach-auto-enabled = Відкривати чернетки інцидентів за виявленнями.
admin-settings-key-breach-auto-start-clock = Відлік для чернетки
admin-settings-help-breach-auto-start-clock = Запускати відлік під час відкриття чернетки або лише після її підтвердження.
admin-settings-key-breach-auto-assignee = Призначати чернетки
admin-settings-help-breach-auto-assignee = Кому призначається чернетка.
admin-settings-key-breach-auto-escalate-after-hours = Ескалювати через годин
admin-settings-help-breach-auto-escalate-after-hours = Скільки годин чернетка чекає до ескалації. 0 ніколи не ескалює.
admin-settings-key-breach-auto-escalate-to = Ескалювати кому
admin-settings-help-breach-auto-escalate-to = Кому призначається ескальована чернетка.
admin-settings-key-breach-auto-window-minutes = Вікно підрахунку в хвилинах
admin-settings-help-breach-auto-window-minutes = Кількість хвилин, за які правило рахує події.
admin-settings-key-breach-auto-theft-threshold = Сповіщення про викрадення сеансу
admin-settings-help-breach-auto-theft-threshold = Кількість сповіщень у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spray або сплеск входів
admin-settings-help-breach-auto-credential-stuffing-threshold = Кількість подій у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-mass-forward-threshold = Масові правила пересилання
admin-settings-help-breach-auto-mass-forward-threshold = Кількість подій у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-role-escalation-threshold = Зміни ролей адміністратора
admin-settings-help-breach-auto-role-escalation-threshold = Кількість змін у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-export-volume-threshold = Незвичні експорти
admin-settings-help-breach-auto-export-volume-threshold = Кількість експортів у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-audit-chain-threshold = Збої ланцюга аудиту
admin-settings-help-breach-auto-audit-chain-threshold = Кількість збоїв, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-backup-anomaly-threshold = Збої резервного копіювання чи відновлення
admin-settings-help-breach-auto-backup-anomaly-threshold = Кількість збоїв у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-breach-auto-dsr-export-threshold = Багато експортів за запитами про дані
admin-settings-help-breach-auto-dsr-export-threshold = Кількість експортів у вікні, що відкриває чернетку. 0 вимикає.
admin-settings-key-ratelimit-queue-ms = Найдовше очікування вільного місця
admin-settings-help-ratelimit-queue-ms = Найдовший час очікування запитом вільного місця (паралелізм JMAP, декодери зображень, обробники перегляду), перш ніж його буде відхилено з кодом 429 і Retry-After. Діапазон 0–30000 мс; типово 3000. Нуль відхиляє одразу.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Одночасних запитів JMAP на обліковий запис
admin-settings-help-ratelimit-jmap-per-account-concurrent = Скільки запитів JMAP один обліковий запис із виконаним входом може виконувати одночасно в усіх вкладках і на всіх пристроях. Не ділиться з іншим обліковим записом чи іншим користувачем за тією самою адресою. Діапазон 4–64; типово 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Одночасних запитів JMAP, усі облікові записи
admin-settings-help-ratelimit-jmap-global-concurrent = Скільки запитів JMAP увесь серверний процес виконує одночасно, для всіх облікових записів разом. Запит понад це чекає час черги. Діапазон 4–4096; типово 256.
admin-settings-key-ratelimit-session-burst = Звітів про час на сеанс поспіль
admin-settings-help-ratelimit-session-burst = Скільки звітів про час клієнта один сеанс може надіслати поспіль. Кінцева точка прив'язана до сеансу, а не до адреси. Діапазон 1–1000; типово 60.
admin-settings-key-ratelimit-session-per-minute = Звітів про час відновлюється за хвилину
admin-settings-help-ratelimit-session-per-minute = Скільки звітів про час клієнта за хвилину сеанс відновлює після серії. Діапазон 1–600; типово 30.
admin-settings-key-ratelimit-anon-burst = Анонімних запитів згоди поспіль
admin-settings-help-ratelimit-anon-burst = Скільки запитів одна адреса може надіслати поспіль на анонімні маршрути згоди на cookie. Розраховано на офіс за однією адресою. Діапазон 1–1000; типово 60.
admin-settings-key-ratelimit-anon-per-minute = Анонімних запитів згоди відновлюється за хвилину
admin-settings-help-ratelimit-anon-per-minute = Скільки запитів за хвилину адреса відновлює на анонімних маршрутах згоди на cookie. Діапазон 1–600; типово 30.
admin-settings-key-ratelimit-trusted-multiplier = Ліміт облікового запису з відомого джерела (%)
admin-settings-help-ratelimit-trusted-multiplier = Відсоток ліміту запитів облікового запису, який отримує скринька з виконаним входом, коли запит надходить із джерела із зафіксованим успішним входом. 100 зберігає базовий ліміт. Ніколи не послаблює бюджети анонімів, адміністратора й user-agent та чинне блокування. Діапазон 100–1000; типово 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Незалежно від того, клацання будь-де на панелі вкладень читача розширює чи згортає її. Коли вимкнено, працює лише кнопка зі стрілкою.

admin-settings-help-attachments-bar-download-all = Чи пропонує панель вкладень кнопку Завантажити все, яка зберігає дозволені вкладення як один файл ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Коли для кожного вкладення на панелі відображається кнопка завантаження: при наведенні або фокусі (за замовчуванням), завжди або ніколи. Завантаження все ще відповідають політиці вкладень і mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Чи зображення, показані в тексті повідомлення, включено до Завантажити все ZIP. За замовчуванням вимкнено.

admin-settings-help-cap-reading-delete-sender = Дозволити користувачам увімкнути дію читання, яка переміщує всі відповідні повідомлення відправника в кошик після підтвердження. Вимкнено за замовчуванням. Профіль функції також має це дозволяти.

admin-settings-help-compose-feature-family = Увімкніть або вимкніть елемент керування композитором за ідентифікатором функції. За замовчуванням увімкнено; заблокуйте його, щоб користувачі не змінювали його.

admin-settings-help-compose-headers-allow-auto-submitted = Дозволяє користувачам установлювати автоматичне надсилання (RFC 3834) для повідомлення. Вимкнено за замовчуванням: це повідомляє іншим системам не відповідати.

admin-settings-help-compose-headers-allow-custom = Дозволяє користувачам додавати власні заголовки (до 20, імена перевіряються, захищені заголовки відхиляються).

admin-settings-help-compose-headers-allow-return-receipt-to = Дозволяє користувачам запитувати квитанції за допомогою застарілого заголовка Return-Receipt-To. За замовчуванням вимкнено.

admin-settings-help-compose-headers-allow-threading-edit = Дозволяє користувачам редагувати In-Reply-To та References або починати новий ланцюжок із відповіді.

admin-settings-help-compose-headers-forced = Одна пара Ім’я=значення на рядок, додається до кожного повідомлення, надісланого з домену, і замінює значення користувача. Захищені заголовки відхилені.

admin-settings-help-compose-insert-image-resize-default = Розмір діалогового вікна «Зображення з файлу» починається: оригінальний, великий (1600 пікселів) або середній (1024 пікселів).

admin-settings-help-compose-insert-max-inline-image-mb = Зображення більшого розміру не розміщуються в тілі повідомлення; їх ще можна прикріпити. 1 до 50.

admin-settings-help-compose-layout-default-font = Сімейство шрифтів, розмір і колір, з якого починається нове повідомлення, як сімейство|розмір|#колір. Empty використовує шрифт теми.

admin-settings-help-compose-reply-to-allow-override = Якщо вимкнено, відповідь на ідентифікатор використовується як є, і користувачі не можуть редагувати або видаляти його.

admin-settings-help-compose-reply-to-show = Чи показує композитор рядок «Відповісти»: лише якщо його має особа або вихідне повідомлення, завжди або ніколи.

admin-settings-help-compose-sent-folder-fixed-id = Роль або ідентифікатор папки використовується, коли правилом є "фіксована папка".

admin-settings-help-compose-sent-folder-mode = Куди зберігається копія надісланого повідомлення: звичайна папка «Надіслані», одна фіксована папка, за доменом одержувача, за ідентифікатором або за правилами теми.

admin-settings-help-compose-sent-folder-per-identity = Одна пара ідентифікатор=папка на рядок. Використовується, коли правило «за ідентичністю».

admin-settings-help-compose-sent-folder-rules = Одне правило на рядок, domain:example.com=папка або subject:text=папка. Перший матч виграє.

admin-settings-help-compose-sent-folder-skip-copy = Ніколи не зберігайте копію надісланого повідомлення. Адміністратор може примусово ввімкнути або вимкнути це.

admin-settings-help-compose-styles-custom = Власні композиторські стилі користувача як JSON, написані галереєю стилів. Користувачі встановлюють його; тут нема чого налаштовувати.

admin-settings-help-compose-styles-default-set = Вигляд галереї стилів композитора: стандартний, сучасний, класичний або компактний.

admin-settings-help-compose-styles-max-custom = Більшість настроюваних стилів користувач може зберегти в композиторі. 0 вимикає створення стилю.

admin-settings-help-contacts-save-from-message-add-note = Додайте примітку «Збережено з повідомлення <дата>» до контактів, збережених із повідомлення. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-contacts-save-from-message-default-book = Ідентифікатор адресної книги, яка отримує контакти, збережені з повідомлення; "default" використовує стандартну адресну книгу. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-contacts-save-from-message-default-scope = Які учасники позначаються першими в діалоговому вікні «Зберегти контакти»: лише відправник, відправник і «Відповісти» або всі. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-contacts-save-from-message-enabled = Пропозиція Зберегти контакти з цього повідомлення в меню повідомлень. Коли вимкнено, елемент приховано. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-contacts-save-from-message-skip-automated = Залиште адреси для невідповідей, сповіщень і відмов у діалоговому вікні збереження контактів, якщо не позначено вручну. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-contacts-save-sender-enabled = Запропонуйте пункт «Зберегти відправника як контакт» одним клацанням миші в меню повідомлень. Коли вимкнено, елемент приховано. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-contacts-save-sender-use-reply-to = Збережіть адресу «Відповісти» замість «Від», якщо відправником є адреса списку розсилки або сповіщення. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-dsr-admin-step-up = Увімкнено (за замовчуванням): адміністратор, який схвалює запит на дані, запускає видалення або схвалює масово, повинен спочатку знову підтвердити, що це він (пароль і другий фактор або ключ доступу). Підтвердження триває п'ять хвилин. Off знімає додаткову перевірку; рішення все ще записується в слід запиту.

admin-settings-help-export-default-format = Формат, з якого починається діалогове вікно експорту.

admin-settings-help-export-default-scope = Область, з якої починається діалогове вікно експорту: повідомлення, розмова, виділення або папка.

admin-settings-help-export-dialog-enabled = Експорт у перегляді повідомлень і меню повідомлень відкриває діалогове вікно на місці. Якщо вимкнено, ці меню завантажують одне повідомлення як .eml.

admin-settings-help-export-folder-enabled = Дозволити користувачам завантажувати цілу папку як ZIP повідомлень. Кожен експорт перевіряється.

admin-settings-help-export-folder-format = Формати, які пропонує експорт папки: ZIP файлів .eml, ZIP, що містить файл mbox, або обидва.

admin-settings-help-export-folder-include-subfolders = Чи включає експорт папки її вкладені папки за замовчуванням, які зберігаються як структура папок у ZIP.

admin-settings-help-export-folder-max-bytes = Максимальна кількість байтів повідомлень, яку може вмістити одна експортована папка. Більшу папку вказаного розміру не можна використовувати.

admin-settings-help-export-folder-max-messages = Максимальна кількість повідомлень, які може вмістити одна експортована папка.

admin-settings-help-export-formats = Формати, які пропонує діалогове вікно експорту: eml, mbox, txt, md, msg, docx, csv, json, pdf. Сервер відмовляється від указаного в списку формату, створеного сервером.

admin-settings-help-export-message-download-enabled = Дозволити користувачам завантажувати повідомлення як .eml, лише його заголовки, або як файл mbox.

admin-settings-help-folders-hide-empty = Приховати порожні папки на бічній панелі. Користувачі можуть змінити значення за замовчуванням або замінити його для окремих папок, якщо це налаштування не заблоковано. Пошта не видаляється.

admin-settings-help-folders-pinned-hide-from-tree = Закріплені папки відображаються в групі «Закріплені» над деревом папок і також зберігають своє місце в дереві. Увімкнено приховує закріплену папку, яка не має вкладених папок, у дереві, тому вона відображається лише в групі «Закріплені». Закріплена папка з вкладеними папками залишається в дереві.

admin-settings-help-mail-badge-auth = Коли в заголовку читача відображається бейдж автентифікації DMARC: завжди, лише коли це помітно або ніколи.

admin-settings-help-mail-badge-density = Розмір значків в шапці читалки: компактний або зручний.

admin-settings-help-mail-badge-folder = Коли заголовок читача показує значок папки: завжди, лише коли це помітно або ніколи.

admin-settings-help-mail-badge-headers = Відображати значок у зчитувачі для заголовків повідомлення Expires, Reply-By, Sensitivity і Oversedes.

admin-settings-help-mail-badge-labels = Чи показують значки заголовка читача піктограму з текстом чи лише піктограму.

admin-settings-help-mail-badge-list = Коли в заголовку читача відображається значок списку розсилки: завжди, лише тоді, коли це помітно, або ніколи. Для цього потрібен заголовок List-Id або List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Коли в заголовку читача відображається очищений бейдж HTML: завжди, лише тоді, коли це помітно, або ніколи.

admin-settings-help-mail-badge-spam = Коли в заголовку програми для читання відображається значок оцінки спаму: завжди, лише тоді, коли це помітно, або ніколи.

admin-settings-help-mail-brand-avatars-require-auth = ніколи: показувати логотип щоразу, коли домен збігається. відома помилка (за замовчуванням): показувати її, якщо результат DKIM або DMARC повідомлення не є помилкою; неперевірені повідомлення зберігають логотип. завжди: показуйте його лише після узгодженого проходу DMARC з DKIM для власного домену бренду. Перевизначення користувача та домену можна заблокувати.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Протягом кількох секунд основний текст має утримувати фокус, перш ніж повний заголовок одержувача та теми згорнеться до свого резюме. Виберіть 0, щоб вимкнути складання за часом, або 1–60 секунд. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-mail-compose-priority-default = Пріоритет, з якого починається нове повідомлення в редакторі повідомлень.

admin-settings-help-mail-compose-priority-enabled = Показати перемикач пріоритетів «Звичайний», «Високий» і «Низький» у групі «Параметри композитора». Якщо вимкнено, нова пошта надсилається без пріоритетних заголовків.

admin-settings-help-mail-expiry-hide-expired = Що робити з повідомленням, термін дії якого минув: нічого, не показувати його затемненим або запропонувати архівувати.

admin-settings-help-mail-list-keep = Відправники та списки розсилки завжди зберігаються, по одному на рядок як sender:address або list:List-Id. Очистіть цього відправника, пропустіть пропозиції щодо скасування підписки та правила збереження.

admin-settings-help-mail-list-page-size = Скільки повідомлень список отримує щоразу, коли користувач прокручує його до кінця. За замовчуванням 50, від 20 до 200. Користувачі вибирають власні значення, якщо ви не заблокуєте це.

admin-settings-help-mail-list-unsubscribe-mode = Як працює скасування підписки, коли список пропонує запит одним клацанням миші: спершу запитайте, надішліть його одразу або просто відкрийте посилання чи чернетку списку.

admin-settings-help-mail-message-list-group = Групуйте розмови за датою отримання або надсилання, відправником, темою, важливістю або розміром. Користувачі можуть перевизначати це, якщо не заблоковано.

admin-settings-help-mail-priority-marks = Який пріоритет позначає список повідомлень, бесіди та програму читання: як високий, так і низький, лише високий чи жодного. Користувачі вибирають у меню «Налаштування» > «Папки» та бічній панелі, якщо вони не заблоковані.

admin-settings-help-mail-priority-marks-style = Як виглядає позначка пріоритету: іконка, іконка з написом High або Low або тонка смужка. Значок і текст зберігають значення без кольору.

admin-settings-help-mail-priority-send-headers = Які заголовки отримує повідомлення з високим або низьким пріоритетом під час надсилання: будь-який із Importance, X-Priority та X-MSMail-Priority.

admin-settings-help-mail-priority-trust-sender-headers = Чи відображається пріоритет, встановлений відправником: завжди, лише коли повідомлення пройшло вирівняну перевірку DMARC, чи ніколи. Використовуйте надійно або ніколи проти відправників, які позначають кожне повідомлення як високий пріоритет.

admin-settings-help-mail-reply-from-header-priority = Заголовки повідомлень читаються в такому порядку, щоб знайти адресу, на яку було надіслано повідомлення під час вибору особи для відповіді. Імена: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc. Пропущений заголовок не читається.

admin-settings-help-mail-search-default-scope = Почніть новий звичайний пошук пошти в поточній папці або в усіх папках. Це керує початковим значенням за замовчуванням, включно з блокуванням; користувачі можуть вибрати інший обсяг для певного пошуку. Збережені запити та запити з явною областю зберігаються.

admin-settings-help-metrics-dimensions-enabled = Зберігайте збережену статистику, розділену за типом клієнта (веб) і класом пристрою (настільний комп’ютер, мобільний). Вимкніть, щоб зберігати один об’єднаний рядок на показник і хвилину.

admin-settings-help-metrics-persist-enabled = Зберігайте статистику продуктивності (швидкість проксі-сервера зображення, час завантаження браузера, кількість обмежень швидкості та захисту запитів) у базі даних кожного інтервалу очищення та коли сервер зупиняється, щоб діаграми зберігали історію після перезапуску. Ніщо не ідентифікує особу, адресу чи повідомлення. Вимкніть, щоб зберегти їх лише в пам’яті.

admin-settings-help-metrics-persist-flush-seconds = Як часто статистика продуктивності записується в базу даних (від 5 до 3600 секунд). Сервер, який вимикається без попередження, втрачає щонайбільше стільки.

admin-settings-help-observability-client-capture-console = Також повідомляйте про виклики console.error, не більше п’яти на хвилину на сторінку. За замовчуванням вимкнено, оскільки бібліотеки реєструють очікувані умови.

admin-settings-help-observability-client-capture-csp = Повідомляти про порушення Content-Security-Policy, які виявляє веб-переглядач, із заблокованою директивою та джерелом лише. Увімкнено за замовчуванням.

admin-settings-help-observability-client-enabled = Збирайте звіти про помилки з веб-програми (межі помилок, нерозпізнані помилки, невдалі фрагменти, помилки робочих елементів, порушення CSP) із очищеними трасуваннями стека, які відображаються в розділі Спостереженість > Помилки клієнта. Увімкнено за замовчуванням. Звіти не містять вмісту повідомлень, адрес, токенів, імен вкладень або запитів URL. Вимкніть, щоб зупинити збір; існуючі звіти закінчуються з їх збереженням.

admin-settings-help-observability-client-forward = Також надсилайте кожен очищений звіт до приймача помилок, налаштованого за допомогою MW_ERROR_FORWARD_URL. За замовчуванням вимкнено. Без налаштованого приймача нічого не надсилається.

admin-settings-help-observability-client-include-account-hash = Додайте ключовий односторонній хеш облікового запису до звітів, надісланих із сеансу, коли ви ввійшли в обліковий запис, щоб звіти з одного облікового запису можна було розрізняти, не називаючи його. За замовчуванням вимкнено. Хеш змінюється під час ротації ключа конфіденційності.

admin-settings-help-observability-client-max-per-session = Більшість звітів про помилки може надіслати одне завантаження сторінки (від 1 до 1000). Повтори тієї самої помилки протягом 60 секунд зараховуються один раз. За замовчуванням 20.

admin-settings-help-observability-client-sample-rate = Частка звітів про помилки, які надсилає браузер, від 0 до 1. За умовчанням 1 (усі). Кожне завантаження сторінки все ще зупиняється на рівні сеансу.

admin-settings-help-observability-client-source-maps = Розв’яжіть позиції пакетів у трасуваннях стека до вихідних файлів, коли збірка надсилає приховані вихідні карти (створено з MW_SOURCEMAP=hidden). Карти читаються лише сервером і ніколи не подаються браузерам. Увімкнено за замовчуванням; без карт відображаються необроблені кадри.

admin-settings-help-privacy-error-reports = Чи надсилає веб-переглядач звіт про помилку, коли частина програми виходить з ладу. Увімкнено за замовчуванням. Глобальна, доменна та користувацька область: установіть тут значення за замовчуванням або заблокуйте його; люди можуть вимкнути його для себе в Налаштуваннях > Конфіденційність, якщо він не заблокований.

admin-settings-help-privacy-image-cache-max-stale-secs = За секунди після свіжості кешоване зображення зберігається, щоб відповісти, коли джерело неможливо отримати. Ніколи не нижче вікна повторної перевірки. Діапазон 0–604800; за замовчуванням 86400.

admin-settings-help-privacy-image-cache-stale-secs = Через кілька секунд після того, як воно було свіжим, кешоване зображення все ще подається відразу, поки воно повторно перевіряється у фоновому режимі. Нуль вимикає. Діапазон 0–86400; за замовчуванням 3600.

admin-settings-help-privacy-image-resize-enabled = Зменшуйте великі віддалені зображення до ширини, яку вимагає читач (320, 640, 960, 1280 або 1600 пікселів). Анімовані зображення подаються без масштабу. За замовчуванням вимкнено.

admin-settings-help-privacy-image-resize-format = Автоматично вибирає найкращий кодер у збірці, яким є PNG. За замовчуванням автоматично.

admin-settings-help-privacy-image-resize-max-pixels = Зображення з більшою кількістю декодованих пікселів, ніж це, подаються без масштабування; розмір зчитується із заголовка перед будь-яким декодуванням. Діапазон 100000–64000000; за замовчуванням 4000000.

admin-settings-help-privacy-image-resize-max-width = Найширше зображення, яке створює зміна розміру. Діапазон 320–1600; за замовчуванням 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Як «Параметри» > «Безпека» > «Сеанси» називає обліковий запис, з якого походить сеанс доступу до зв’язаної поштової скриньки: повна адреса, адреса з маскованою першою частиною імені або нічого. Значення відображається лише власнику сеансів, іншим користувачам ніколи. За замовчуванням: повна адреса.

admin-settings-help-properties-folder-enabled = Властивості пропозиції в меню папок і на ярлику. Якщо вимкнено, діалогове вікно недоступне.

admin-settings-help-properties-folder-notes-enabled = Дозвольте опис або примітку для кожної папки.

admin-settings-help-properties-folder-notes-max-length = Максимальна кількість символів, яку може містити примітка до папки.

admin-settings-help-properties-folder-overrides-allowed = Які параметри користувач може змінити для однієї папки. Перевизначення, якого немає в списку, приховано, а будь-яке збережене значення ігнорується.

admin-settings-help-properties-folder-sections = Розділи діалогового вікна властивостей папки: загальні, віддалені, розмір, офлайн, перевизначення, примітки, дії. Розділ, якого немає в списку, приховано.

admin-settings-help-properties-folder-size-compute = Під час визначення розміру папки на сервері: ніколи, коли відкривається діалогове вікно або у фоновому режимі. Підрахунок великої поштової скриньки коштує серверу сканування.

admin-settings-help-properties-headers-raw = Дозволити діалоговому вікну «Властивості» повідомлення перелічувати необроблені заголовки. Коли вимкнено, відображається лише проаналізований підсумок.

admin-settings-help-properties-message-enabled = Властивості пропозиції в меню повідомлень і на ярлику. Якщо вимкнено, діалогове вікно недоступне.

admin-settings-help-properties-message-permanent-delete = Чи може діалогове вікно повідомлення видалити повідомлення назавжди: дозволити, запитати двічі чи заборонити. Повідомлення, яке затримано на законних підставах, ніколи не видаляється. Видалення таким чином не можна скасувати.

admin-settings-help-properties-message-sections = Розділи діалогового вікна властивостей повідомлення: резюме, оцінка, заголовки, вкладення, дії. Розділ, якого немає в списку, приховано.

admin-settings-help-properties-offline-clear-enabled = Дозволити користувачам очищати офлайн-кеш однієї папки з діалогового вікна «Властивості».

admin-settings-help-properties-remote-location-visibility = Яку частину розташування на стороні сервера бачать користувачі: повне (хост і шлях), замаскований хост (хост частково прихований), лише шлях (шлях до поштової скриньки без хосту) або прихований.

admin-settings-help-properties-scoring-show = Показувати рейтинг спаму, результати автентифікації та сигнали безпеки в діалоговому вікні властивостей повідомлення.

admin-settings-help-properties-shortcut-enabled = Відкрийте діалогове вікно «Властивості» з клавіатури для вибраної папки або вибраного повідомлення.

admin-settings-help-properties-shortcut-key = Комбінація клавіш, яка відкриває Властивості; Alt+Enter за замовчуванням.

admin-settings-help-reader-actions-archive = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-block-sender = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-colors = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-conversation = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-create-filter = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-delete = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-delete-sender = Приховано за умовчанням. Виберіть панель інструментів або меню «Більше», щоб увімкнути його, якщо це дозволяють можливість видалення відправника та профіль функції. Користувачі можуть змінювати розташування, якщо воно не заблоковано.

admin-settings-help-reader-actions-edit-draft = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-export = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-finish-followup = Виберіть панель інструментів, меню «Більше» або приховано для завершення позначених повідомлень і подальших нагадувань.

admin-settings-help-reader-actions-flag = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-focus-focused = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-focus-other = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-forward = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-headers = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-links = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-move = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-not-spam = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-open-draft = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-pin = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-print = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-properties = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-read = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-reply = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-reply-all = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-save-contacts = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-save-sender = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-search = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-security = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-spam = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-sweep = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-view-source = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-reader-actions-zoom = Виберіть панель інструментів, меню Більше або приховано. Обмеження функцій, дозволи та контекст повідомлення все ще застосовуються. Користувачі можуть вибирати, якщо це не заблоковано.

admin-settings-help-retention-client-errors-event-days = Дні зберігаються окремі помилки браузера (його трасування стека та клас браузера). За замовчуванням 30. Група зберігається довше, дивіться наступне налаштування.

admin-settings-help-retention-client-errors-group-days = Кількість днів, протягом яких група помилок браузера (ім’я, повідомлення, кількість) зберігається після того, як вона була останньою. За замовчуванням 180. Ніколи не коротше, ніж збереження входження.

admin-settings-help-retention-metrics-day-buckets-days = Дні щоденної статистики продуктивності зберігаються до їх видалення. 0 зберігає їх без обмежень, що дозволяє таблиці повільно зростати, поки сервер працює.

admin-settings-help-retention-metrics-hour-buckets-days = Дні погодинної статистики продуктивності, які зберігаються до їх об’єднання в щоденні рядки (принаймні 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Години однохвилинної статистики продуктивності, які зберігаються перед об’єднанням у погодинні рядки (принаймні 1).

admin-settings-help-retention-user-allow-strip-attachments = Чи можуть користувачі створювати правила, які видаляють великі вкладення зі старої пошти. За замовчуванням вимкнено. Якщо вимкнено, дія прихована в редакторі правил, правило, яке її використовує, відхиляється під час збереження, а правила, збережені раніше, пропускаються під час виконання завдання.

admin-settings-help-retention-user-calendar-enabled = Чи можуть користувачі вибирати календарі, події з яких видаляються через задану кількість днів після їх завершення. За замовчуванням вимкнено. Якщо параметр вимкнено, він прихований у налаштуваннях, після збереження вибір відхиляється, а збережені раніше параметри не запускаються.

admin-settings-help-retention-user-calendar-max-days = Більшість днів користувач може чекати, перш ніж завершені події будуть видалені. Більше значення відхиляється під час збереження та знижується під час виконання.

admin-settings-help-retention-user-calendar-min-days = Кілька днів після завершення події, перш ніж правило користувача може її видалити. Менше значення відхиляється під час збереження та підвищується під час виконання.

admin-settings-help-retention-user-notes-enabled = Чи можуть користувачі видаляти нотатки, які не змінюються протягом установленої кількості днів. За замовчуванням вимкнено. Видалені нотатки можна відновити, доки не закінчиться вікно скасування для правил збереження.

admin-settings-help-retention-user-notes-max-days = Більшість днів користувач може чекати, перш ніж незаймані нотатки будуть видалені. Більше значення відхиляється під час збереження та знижується під час виконання.

admin-settings-help-retention-user-notes-min-days = Найменшу кількість днів примітка має бути недоторканою, перш ніж правило користувача може її видалити. Менше значення відхиляється під час збереження та підвищується під час виконання.

admin-settings-help-retention-user-strip-keep-inline-images = Чи зберігаються зображення, показані в тілі повідомлення, після видалення вкладень.

admin-settings-help-retention-user-strip-min-age-days = Перш ніж правило може видалити вкладені файли, повідомлення має бути старим за кілька днів. Менше значення відхиляється, коли правило зберігається, і підвищується під час його виконання.

admin-settings-help-retention-user-strip-min-attachment-kb = Видаляються лише вкладення принаймні такого розміру. Менші вкладення залишаються в повідомленні.

admin-settings-help-retention-user-strip-undo-days = Днів, коли вихідне повідомлення залишається в кошику після видалення вкладених файлів. -1 слідує за вікном скасування для правил збереження. 0 одразу видаляє оригінал.

admin-settings-help-server-shutdown-grace-seconds = Скільки часу сервер зупинки дозволяє завершити запити в польоті, перш ніж він закриє з’єднання, у секундах. Змінна MW_HTTP_SHUTDOWN_TIMEOUT_MS замінює її. Застосовується при наступному запуску.

admin-settings-help-session-last-activity-flush-seconds = Час останнього відвідування сеансу записується щонайбільше так часто, у секундах. Zero пише це на кожен запит. Це не змінює межі холостого ходу або абсолютні межі. Застосовується при наступному запуску.

admin-settings-help-session-persist-across-restarts = Коли ввімкнено, сеанси поштової скриньки, програми та консолі залишаються дійсними після перезапуску сервера. Якщо вимкнено, кожен сеанс завершується, коли запускається сервер. Застосовується при наступному запуску.

admin-settings-key-attachments-bar-click-to-toggle = Натисніть панель вкладень, щоб перемкнути

admin-settings-key-attachments-bar-download-all = Кнопка завантажити все

admin-settings-key-attachments-bar-show-download-buttons = Кнопки завантаження вкладених файлів

admin-settings-key-attachments-bar-zip-inline-images = Вбудовані зображення в Завантажити все ZIP

admin-settings-key-cap-reading-delete-sender = Дозволити видаляти всю пошту від відправника

admin-settings-key-compose-feature-family = Особливості композитора

admin-settings-key-compose-headers-allow-auto-submitted = Дозволити автоматичне надсилання

admin-settings-key-compose-headers-allow-custom = Дозволити спеціальні заголовки

admin-settings-key-compose-headers-allow-return-receipt-to = Дозволити Return-Receipt-To

admin-settings-key-compose-headers-allow-threading-edit = Дозволити редагування заголовків потоків

admin-settings-key-compose-headers-forced = Примусові заголовки

admin-settings-key-compose-insert-image-resize-default = Попереднє налаштування розміру зображення

admin-settings-key-compose-insert-max-inline-image-mb = Найбільше вбудоване зображення (МБ)

admin-settings-key-compose-layout-default-font = Типовий шрифт повідомлення

admin-settings-key-compose-reply-to-allow-override = Дозволити зміну відповіді

admin-settings-key-compose-reply-to-show = Відповісти рядку

admin-settings-key-compose-sent-folder-fixed-id = Надіслана копія: фіксована папка

admin-settings-key-compose-sent-folder-mode = Надіслана копія: правило папки

admin-settings-key-compose-sent-folder-per-identity = Надіслана копія: папка для особи

admin-settings-key-compose-sent-folder-rules = Надіслано: правила

admin-settings-key-compose-sent-folder-skip-copy = Надіслана копія: не зберігати

admin-settings-key-compose-styles-custom = Спеціальні стилі

admin-settings-key-compose-styles-default-set = Набір стилів

admin-settings-key-compose-styles-max-custom = Спеціальні стилі для кожного користувача

admin-settings-key-contacts-save-from-message-add-note = Примітка щодо контактів, збережених із повідомлення

admin-settings-key-contacts-save-from-message-default-book = Адресна книга для контактів, збережених із повідомлення

admin-settings-key-contacts-save-from-message-default-scope = Під час збереження з повідомлення контакти відзначаються першими

admin-settings-key-contacts-save-from-message-enabled = Збережіть контакти з повідомлення

admin-settings-key-contacts-save-from-message-skip-automated = Пропускати автоматичних відправників під час збереження контактів

admin-settings-key-contacts-save-sender-enabled = Зберегти відправника як контакт

admin-settings-key-contacts-save-sender-use-reply-to = Зберегти відповідь для списку відправників

admin-settings-key-dsr-admin-step-up = Підтвердьте ще раз перед схваленням

admin-settings-key-export-default-format = Стандартний формат діалогового вікна експорту

admin-settings-key-export-default-scope = Діалогове вікно експорту за замовчуванням

admin-settings-key-export-dialog-enabled = Діалог експорту

admin-settings-key-export-folder-enabled = Експорт папки

admin-settings-key-export-folder-format = Формат експорту папки

admin-settings-key-export-folder-include-subfolders = Включати вкладені папки в експорт

admin-settings-key-export-folder-max-bytes = Обмеження розміру експортованої папки

admin-settings-key-export-folder-max-messages = Ліміт повідомлень експорту папки

admin-settings-key-export-formats = Експорт діалогових форматів

admin-settings-key-export-message-download-enabled = Завантаження повідомлення

admin-settings-key-folders-hide-empty = Приховати порожні папки

admin-settings-key-folders-pinned-hide-from-tree = Приховати закріплені папки з дерева

admin-settings-key-mail-badge-auth = Бейдж автентифікації

admin-settings-key-mail-badge-density = Розмір читацького значка

admin-settings-key-mail-badge-folder = Значок папки

admin-settings-key-mail-badge-headers = Заголовні значки

admin-settings-key-mail-badge-labels = Ярлики читацького значка

admin-settings-key-mail-badge-list = Значок списку розсилки

admin-settings-key-mail-badge-sanitized = Продезінфікований значок HTML

admin-settings-key-mail-badge-spam = Значок оцінки спаму

admin-settings-key-mail-brand-avatars-require-auth = Для логотипів брендів потрібна автентифікація

admin-settings-key-mail-compose-header-auto-collapse-seconds = Затримка автоматичного згортання заголовка Composer

admin-settings-key-mail-compose-priority-default = Пріоритет повідомлень за замовчуванням

admin-settings-key-mail-compose-priority-enabled = Селектор пріоритетів у композиторі

admin-settings-key-mail-expiry-hide-expired = Прострочені повідомлення

admin-settings-key-mail-list-keep = Завжди тримати

admin-settings-key-mail-list-page-size = Розмір сторінки списку повідомлень

admin-settings-key-mail-list-unsubscribe-mode = Список розсилки Відписатися

admin-settings-key-mail-message-list-group = Групування повідомлень

admin-settings-key-mail-priority-marks = Знаки пріоритету

admin-settings-key-mail-priority-marks-style = Стиль позначки пріоритету

admin-settings-key-mail-priority-send-headers = Пріоритетні заголовки, записані під час надсилання

admin-settings-key-mail-priority-trust-sender-headers = Пріоритет довіри відправника

admin-settings-key-mail-reply-from-header-priority = Заголовки, які вибирають ідентифікатор відповіді

admin-settings-key-mail-search-default-scope = Область пошуку пошти за замовчуванням

admin-settings-key-metrics-dimensions-enabled = Розділіть статистику за типом клієнта та класом пристрою

admin-settings-key-metrics-persist-enabled = Ведіть статистику перезапусків

admin-settings-key-metrics-persist-flush-seconds = Інтервал збереження статистики (секунди)

admin-settings-key-observability-client-capture-console = Повідомити про помилки консолі

admin-settings-key-observability-client-capture-csp = Повідомити про порушення CSP

admin-settings-key-observability-client-enabled = Звіти про помилки браузера

admin-settings-key-observability-client-forward = Пересилання звітів про помилки

admin-settings-key-observability-client-include-account-hash = Додайте хеш облікового запису до звітів про помилки

admin-settings-key-observability-client-max-per-session = Звіти про помилки за завантаження сторінки

admin-settings-key-observability-client-sample-rate = Частота вибірки звіту про помилку

admin-settings-key-observability-client-source-maps = Вирішуйте трасування стека за допомогою вихідних карт

admin-settings-key-privacy-error-reports = Надсилати звіти про помилки

admin-settings-key-privacy-image-cache-max-stale-secs = Зберігати застарілим, коли джерело не вдається (секунди)

admin-settings-key-privacy-image-cache-stale-secs = Подавати несвіжі під час повторної перевірки (секунди)

admin-settings-key-privacy-image-resize-enabled = Змінюйте розмір великих зображень

admin-settings-key-privacy-image-resize-format = Змінений формат зображення

admin-settings-key-privacy-image-resize-max-pixels = Найбільше зображення для зміни розміру (пікселі)

admin-settings-key-privacy-image-resize-max-width = Найбільше змінене зображення (пікселі)

admin-settings-key-privacy-sessions-show-linked-origin = Походження пов’язаної поштової скриньки в сеансах

admin-settings-key-properties-folder-enabled = Діалогове вікно властивостей папки

admin-settings-key-properties-folder-notes-enabled = Папка нотаток

admin-settings-key-properties-folder-notes-max-length = Довжина папки

admin-settings-key-properties-folder-overrides-allowed = Дозволено перевизначення для кожної папки

admin-settings-key-properties-folder-sections = Діалогові розділи папок

admin-settings-key-properties-folder-size-compute = Розрахунок розміру папки

admin-settings-key-properties-headers-raw = Необроблені заголовки

admin-settings-key-properties-message-enabled = Діалог властивостей повідомлення

admin-settings-key-properties-message-permanent-delete = Постійне видалення з властивостей

admin-settings-key-properties-message-sections = Розділи діалогу повідомлення

admin-settings-key-properties-offline-clear-enabled = Очистити автономний кеш для кожної папки

admin-settings-key-properties-remote-location-visibility = Деталі віддаленого розташування

admin-settings-key-properties-scoring-show = Деталі скорингу та автентифікації

admin-settings-key-properties-shortcut-enabled = Ярлик властивостей

admin-settings-key-properties-shortcut-key = Клавіша швидкого доступу до властивостей

admin-settings-key-reader-actions-archive = Архівне розміщення дій

admin-settings-key-reader-actions-block-sender = Розміщення дії блокування відправника

admin-settings-key-reader-actions-colors = Кольори повідомлення Розміщення дії

admin-settings-key-reader-actions-conversation = Переглянути розміщення дій у розмові

admin-settings-key-reader-actions-create-filter = Створити розміщення дії фільтра

admin-settings-key-reader-actions-delete = Видалити розміщення дії

admin-settings-key-reader-actions-delete-sender = Видалити всі повідомлення від відправника

admin-settings-key-reader-actions-edit-draft = Редагувати проект розміщення дії

admin-settings-key-reader-actions-export = Експорт розміщення дій

admin-settings-key-reader-actions-finish-followup = Позначити подальшу дію як завершену дію

admin-settings-key-reader-actions-flag = Позначити/зняти позначку розміщення дії

admin-settings-key-reader-actions-focus-focused = Перейти до розміщення зосереджених дій

admin-settings-key-reader-actions-focus-other = Перейти до розміщення інших дій

admin-settings-key-reader-actions-forward = Розміщення передньої дії

admin-settings-key-reader-actions-headers = Необроблені заголовки розміщення дії

admin-settings-key-reader-actions-links = Перегляньте розміщення дії посилань

admin-settings-key-reader-actions-move = Перемістити розміщення дій

admin-settings-key-reader-actions-not-spam = Розміщення дії не спам

admin-settings-key-reader-actions-open-draft = Розміщення дії для чернетки відповіді

admin-settings-key-reader-actions-pin = Закріпити/відкріпити розміщення дії

admin-settings-key-reader-actions-print = Друк розміщення дій

admin-settings-key-reader-actions-properties = Розміщення дії властивостей повідомлення

admin-settings-key-reader-actions-read = Прочитане/непрочитане розташування дії

admin-settings-key-reader-actions-reply = Розміщення дії відповіді

admin-settings-key-reader-actions-reply-all = Відповісти на всі дії розміщення

admin-settings-key-reader-actions-save-contacts = Зберегти розташування контактів повідомлення

admin-settings-key-reader-actions-save-sender = Зберегти розміщення дії відправника

admin-settings-key-reader-actions-search = Розміщення пов’язаної пошукової дії

admin-settings-key-reader-actions-security = Розміщення дій режиму безпеки

admin-settings-key-reader-actions-spam = Позначити як спам розміщення дій

admin-settings-key-reader-actions-sweep = Розміщення дії відправника

admin-settings-key-reader-actions-view-source = Переглянути розміщення дії джерела повідомлення

admin-settings-key-reader-actions-zoom = Розміщення дії масштабування

admin-settings-key-retention-client-errors-event-days = Збережені випадки помилок (днів)

admin-settings-key-retention-client-errors-group-days = Збережені групи помилок (днів)

admin-settings-key-retention-metrics-day-buckets-days = Ведення щоденної статистики (днів)

admin-settings-key-retention-metrics-hour-buckets-days = Ведення погодинної статистики (днів)

admin-settings-key-retention-metrics-minute-buckets-hours = Хвилинна статистика (годин)

admin-settings-key-retention-user-allow-strip-attachments = Користувачі можуть видаляти вкладення зі старих листів

admin-settings-key-retention-user-calendar-enabled = Користувачі можуть видаляти завершені події календаря

admin-settings-key-retention-user-calendar-max-days = Більшість днів до завершення подій видаляється

admin-settings-key-retention-user-calendar-min-days = Події, що завершилися за кілька днів до завершення, видаляються

admin-settings-key-retention-user-notes-enabled = Користувачі можуть видаляти незаймані нотатки

admin-settings-key-retention-user-notes-max-days = За більшість днів до видалення незайманих нотаток

admin-settings-key-retention-user-notes-min-days = Найменше днів до видалення незайманих нотаток

admin-settings-key-retention-user-strip-keep-inline-images = Зберігайте вбудовані зображення

admin-settings-key-retention-user-strip-min-age-days = Найменше днів до видалення вкладень

admin-settings-key-retention-user-strip-min-attachment-kb = Найменший видалений вкладений файл (КБ)

admin-settings-key-retention-user-strip-undo-days = Вікно скасування після видалення вкладень (днів)

admin-settings-key-server-shutdown-grace-seconds = Пільговий період відключення

admin-settings-key-session-last-activity-flush-seconds = Інтервал запису останнього відвідування

admin-settings-key-session-persist-across-restarts = Зберігати сеанси після перезапусків

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = За замовчуванням вимкнено. Зберігайте зашифровані знімки електронної пошти в межах облікового запису під час перезавантаження веб-переглядача без закінчення терміну дії. Існуючі обмеження на введення та кількість байтів все ще застосовуються. Користувачі можуть очистити збережені копії; вимкнення цього параметра видаляє їх у активному браузері. Це не надає веб-переглядачу дозвіл на постійне зберігання чи завантаження вкладень.

admin-settings-help-mail-conversations-allowed = Дозволити користувачам групувати пов’язані відповіді в бесіди. Доступно за умовчанням, але налаштування користувача починаються. Вимкнення доступності перекриває вибір користувача без зміни групування категорій.

admin-settings-help-mail-conversations-enabled = За замовчуванням вимкнено. Групуйте завантажені відповіді за ланцюжком, незалежно від Групувати за. Установіть параметр за замовчуванням або заблокуйте це налаштування. Також потрібно ввімкнути доступність розмови.

admin-settings-key-mail-cache-keep-indefinitely = Зберігати поштовий кеш браузера на невизначений термін

admin-settings-key-mail-conversations-allowed = Зробіть ланцюжок розмов доступним

admin-settings-key-mail-conversations-enabled = Групуйте повідомлення в бесіди

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = До 100 точних доменів відправників у нижньому регістрі, переглянуті посилання на повідомлення яких активуються автоматично. Субдомени окремі. Це не довіряє цільовим веб-сайтам і не дозволяє віддалені зображення. За замовчуванням порожній; користувачі можуть змінити його, якщо він не заблокований.

admin-settings-key-links-trusted-sender-domains = Довірені домени відправників для посилань на повідомлення

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Показувати деталі завантаження та файлу поруч із заголовком попереднього перегляду як значки (за замовчуванням), мітки або значки та мітки. Можна встановити та заблокувати глобально, для кожного домену або для кожного користувача.

admin-settings-help-reader-attachment-thumbnails = За замовчуванням вимкнено. Дозволити видимим вкладенням зображень завантажувати попередній перегляд мініатюр. Правила безпеки вкладених файлів все ще застосовуються. Можна встановити та заблокувати глобально, для кожного домену або для кожного користувача.

admin-settings-key-reader-attachment-action-style = Дії попереднього перегляду вкладених файлів

admin-settings-key-reader-attachment-thumbnails = Мініатюри вкладених зображень

admin-settings-key-mail-compose-auto-add-mention-recipients = Автоматично додавати згаданих людей до одержувачів
admin-settings-help-mail-compose-auto-add-mention-recipients = Після вибору людини з підказок @згадок додавати її в поле Кому, якщо її ще немає в полях Кому, Копія або Прихована копія. Типово вимкнено.

admin-settings-linked-disable-title = Вимкнути збережений доступ до пов’язаних облікових записів?
admin-settings-linked-disable-body = Збереження зміни видаляє облікові дані для поновлення доступу до пов’язаних облікових записів. Користувачам доведеться знову ввійти, коли доступ потребуватиме поновлення. Повторне ввімкнення не відновить ці дані.
admin-settings-linked-disable-confirm = Вимкнути та видалити збережений доступ

admin-settings-key-reader-auto-hide-header = Автоматично приховувати заголовки повідомлень
admin-settings-help-reader-auto-hide-header = Відкривати повідомлення зі згорнутими даними відправника, одержувачів і метаданими. Тема й дії залишаються видимими. Натисніть Показати заголовок для перегляду подробиць. Типово вимкнено.

admin-settings-key-time-region = Регіон
admin-settings-help-time-region = auto або дволітерний код регіону, наприклад GB, US чи PT.

admin-settings-key-time-date-format = Формат дати
admin-settings-help-time-date-format = auto, short, medium, long, full або шаблон, наприклад yyyy-MM-dd чи dd/MM/yyyy. Звичайний текст беріть у 'лапки'.

admin-settings-key-time-time-format = Формат часу
admin-settings-help-time-time-format = auto, short, medium, long, full або HH:mm:ss / h:mm a. H = 24 години, h = 12 годин, a = AM/PM.

admin-settings-key-time-hour-cycle = Годинник
admin-settings-help-time-hour-cycle = Символи H або h у власному шаблоні мають пріоритет.

admin-settings-key-time-first-day-of-week = Перший день тижня
admin-settings-help-time-first-day-of-week = Автоматично за регіональними правилами календаря.

admin-settings-key-compliance-mail-enabled = Увімкнути пошту
admin-settings-help-compliance-mail-enabled = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-host = Сервер SMTP
admin-settings-help-compliance-mail-host = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-port = Порт SMTP
admin-settings-help-compliance-mail-port = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-security = Режим TLS
admin-settings-help-compliance-mail-security = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-from = Адреса відправника
admin-settings-help-compliance-mail-from = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-username = Ім’я користувача SMTP
admin-settings-help-compliance-mail-username = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-allowPrivate = Дозволити приватний сервер
admin-settings-help-compliance-mail-allowPrivate = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-dsr-notify-emailAdmins = Сповіщати операторів про запити даних
admin-settings-help-dsr-notify-emailAdmins = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-dsr-notify-operatorAddresses = Одержувачі оператори
admin-settings-help-dsr-notify-operatorAddresses = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-dsr-notify-emailRequester = Сповіщати заявника поштою
admin-settings-help-dsr-notify-emailRequester = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-breach-notify-email = Сповіщати операторів про витоки
admin-settings-help-breach-notify-email = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-breach-subjects-email = Сповіщати постраждалих осіб поштою
admin-settings-help-breach-subjects-email = Налаштуйте SMTP-сервер із TLS. Пошта використовує маршрут проксі system_mail. За невизначеного прийняття перевірте результат перед повторним надсиланням.

admin-settings-key-compliance-mail-requireProxy = Вимагати маршрут проксі
admin-settings-help-compliance-mail-requireProxy = Вимагати маршрут проксі (system_mail).
admin-settings-key-retention-complianceMail-days = Зберігання історії завершених листів (дні)
admin-settings-help-retention-complianceMail-days = Строк зберігання завершених доставлень. Очікувані та невизначені залишаються до вирішення.

admin-settings-key-incidents-auto-enabled = Автоматично виявляти можливі інциденти
admin-settings-help-incidents-auto-enabled = Збої перевірок і сигнали безпеки сеансів створюють можливі інциденти. Оцініть значущість і перевірте відновлення. Прогалини моніторингу не доводять простою.
admin-settings-key-incidents-auto-failureSamples = Невдалі перевірки до виявлення
admin-settings-help-incidents-auto-failureSamples = Збої перевірок і сигнали безпеки сеансів створюють можливі інциденти. Оцініть значущість і перевірте відновлення. Прогалини моніторингу не доводять простою.
admin-settings-key-incidents-nis2-trustService = Використовувати строк для довірчих послуг
admin-settings-help-incidents-nis2-trustService = Збої перевірок і сигнали безпеки сеансів створюють можливі інциденти. Оцініть значущість і перевірте відновлення. Прогалини моніторингу не доводять простою.
admin-settings-key-incidents-notify-email = Надсилати оновлення й нагадування поштою
admin-settings-help-incidents-notify-email = Збої перевірок і сигнали безпеки сеансів створюють можливі інциденти. Оцініть значущість і перевірте відновлення. Прогалини моніторингу не доводять простою.
admin-settings-key-incidents-notify-operatorAddresses = Адреси операторів
admin-settings-help-incidents-notify-operatorAddresses = Збої перевірок і сигнали безпеки сеансів створюють можливі інциденти. Оцініть значущість і перевірте відновлення. Прогалини моніторингу не доводять простою.
admin-settings-key-incidents-retention-days = Зберігання закритих інцидентів (дні)
admin-settings-help-incidents-retention-days = Збої перевірок і сигнали безпеки сеансів створюють можливі інциденти. Оцініть значущість і перевірте відновлення. Прогалини моніторингу не доводять простою.

admin-settings-key-mail-snooze-presets = Налаштування відкладання
admin-settings-help-mail-snooze-presets = Додавайте, перейменовуйте, упорядковуйте, редагуйте або приховуйте варіанти. У меню завжди можна вибрати дату й час.

admin-settings-key-reader-actions-view-draft = Переглянути чернетку відповіді
admin-settings-help-reader-actions-view-draft = Переглянути чернетку відповіді

admin-settings-key-notifications-view-enabled = Увімкнути розділ сповіщень
admin-settings-help-notifications-view-enabled = Керує центром сповіщень. Попередження безпеки, пошти та браузера зберігають власні налаштування.
admin-settings-key-notifications-view-show-in-sidebar = Показувати сповіщення на бічній панелі
admin-settings-help-notifications-view-show-in-sidebar = Керує центром сповіщень. Попередження безпеки, пошти та браузера зберігають власні налаштування.

admin-settings-key-mail-unified-account = Об’єднаний віртуальний обліковий запис
admin-settings-help-mail-unified-account = Додати окремий віртуальний обліковий запис до перемикача зі спільними папками та лічильниками. Незалежно від об’єднаного перегляду повідомлень.

admin-settings-key-accounts-settings-inherit-groups = Успадковувати налаштування основного облікового запису
admin-settings-help-accounts-settings-inherit-groups = Вибрані групи використовують значення основного облікового запису без локального перевизначення. Локальні налаштування та політика адміністратора пріоритетні. Теми, підписи, облікові дані й ідентифікатори папок не успадковуються.

admin-settings-key-mail-badge-importance = Важливість
admin-settings-help-mail-badge-importance = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-follow-up = До виконання
admin-settings-help-mail-badge-follow-up = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-snooze = Відкласти
admin-settings-help-mail-badge-snooze = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-receipts = Сповіщення про прочитання
admin-settings-help-mail-badge-receipts = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-delivery = Стан доставки
admin-settings-help-mail-badge-delivery = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-dates = Дати повідомлення
admin-settings-help-mail-badge-dates = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-sensitivity = Конфіденційність
admin-settings-help-mail-badge-sensitivity = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-reply-by = Термін відповіді
admin-settings-help-mail-badge-reply-by = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-expires = Термін дії
admin-settings-help-mail-badge-expires = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
admin-settings-key-mail-badge-supersedes = Замінені повідомлення
admin-settings-help-mail-badge-supersedes = Виберіть завжди, лише важливе або ніколи. Виділяє незвичні значення, активні нагадування, підтвердження, доставку або близькі терміни. Дані повідомлення зберігаються.
