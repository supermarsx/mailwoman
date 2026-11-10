# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: settings. Source: ../en/settings.ftl

settings-title = Налаштування
settings-appearance = Вигляд
settings-close = Закрити налаштування

settings-theme = Тема
settings-density = Щільність
settings-accent = Акцентний колір
settings-font = Шрифт інтерфейсу
settings-layout = Компонування

# Density options
settings-density-compact = Компактна
settings-density-cozy = Зручна
settings-density-relaxed = Вільна

# Interface-font options
settings-font-default = Типовий
settings-font-system = Системний
settings-font-serif = Із засічками
settings-font-mono = Моноширинний

# Layout options
settings-layout-default = Типове
settings-layout-ribbon = Стрічка

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Скасувати
settings-edit = Редагувати
settings-delete = Видалити
settings-save = Зберегти
settings-saved = Збережено

# Two-factor authentication (S1)
settings-2fa-title = Двофакторна автентифікація
settings-2fa-intro = Додайте другий фактор, щоб для входу був потрібен не лише пароль.
settings-2fa-policy-required = Ваша організація вимагає другий фактор для цього облікового запису.
settings-2fa-enabled = Увімкнено
settings-2fa-recovery-title = Коди відновлення
settings-2fa-recovery-once = Збережіть їх зараз. Їх показано один раз, і отримати їх знову неможливо. Кожен код можна використати лише один раз.
settings-2fa-recovery-ack = Я зберіг(-ла) ці коди
settings-2fa-totp-title = Застосунок-автентифікатор
settings-2fa-totp-enrol = Налаштувати застосунок-автентифікатор
settings-2fa-totp-scan = Додайте цей секрет до застосунку-автентифікатора, а потім введіть код, який він покаже.
settings-2fa-totp-uri-link = Відкрити в застосунку-автентифікаторі
settings-2fa-totp-code-label = Код автентифікатора
settings-2fa-totp-confirm = Підтвердити
settings-2fa-totp-disable = Вилучити автентифікатор
settings-2fa-passkey-title = Ключі доступу
settings-2fa-passkey-remove = Вилучити
settings-2fa-passkey-unsupported = Цей браузер не підтримує ключі доступу.
settings-2fa-passkey-label-placeholder = Назва ключа доступу (необов'язково)
settings-2fa-passkey-add = Додати ключ доступу
settings-2fa-recovery-remaining = { $count ->
        [one] Залишився { $count } код відновлення
        [few] Залишилося { $count } коди відновлення
        [many] Залишилося { $count } кодів відновлення
       *[other] Залишилося { $count } коду відновлення
    }
settings-2fa-recovery-regenerate = Згенерувати нові коди відновлення
settings-2fa-recovery-code-label = Код відновлення
settings-2fa-error-generic = Не вдалося. Спробуйте знову.
settings-2fa-error-code = Код не пройшов перевірку.
settings-2fa-error-passkey = Не вдалося зареєструвати ключ доступу.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Двофакторна автентифікація
settings-2fa-challenge-intro = Підтвердьте другий фактор, щоб завершити вхід.
settings-2fa-challenge-failed = Перевірку не пройдено. Спробуйте знову.
settings-2fa-challenge-use-passkey = Використати ключ доступу
settings-2fa-challenge-method = Виберіть спосіб
settings-2fa-challenge-totp-tab = Код автентифікатора
settings-2fa-challenge-recovery-tab = Код відновлення
settings-2fa-challenge-verify = Перевірити

# Active sessions (S11)
settings-sessions-title = Активні сеанси
settings-sessions-intro = Сеанси, у яких зараз виконано вхід у цей обліковий запис.
settings-sessions-last-seen = Остання активність: { $when }
settings-sessions-revoke = Вийти
settings-sessions-current = Цей сеанс
settings-sessions-revoke-others = { $count ->
        [one] Завершити { $count } інший сеанс
        [few] Завершити { $count } інші сеанси
        [many] Завершити { $count } інших сеансів
       *[other] Завершити { $count } іншого сеансу
    }
settings-sessions-error = Не вдалося змінити сеанс.

# Signatures (W12)
settings-sig-title = Підписи
settings-sig-intro = Текст, що додається до надісланих вами повідомлень. Один підпис можна зробити типовим.
settings-sig-default = Типовий
settings-sig-new = Новий підпис
settings-sig-name-label = Назва
settings-sig-body-label = Підпис
settings-sig-default-label = Використовувати як типовий
settings-sig-error-name = Дайте підпису назву.
settings-sig-error-generic = Не вдалося зберегти підпис.

# Identities
settings-ident-title = Профілі відправника
settings-ident-intro = Ім'я та адреса, від яких ви надсилаєте пошту. Кожен профіль відправника може мати власний підпис.
settings-ident-new = Новий профіль відправника
settings-ident-name-label = Відображуване ім'я
settings-ident-email-label = Адреса електронної пошти
settings-ident-replyto-label = Адреса для відповіді (необов'язково)
settings-ident-signature-label = Типовий підпис
settings-ident-signature-none = Немає
settings-ident-error-fields = Введіть ім'я та дійсну адресу електронної пошти.
settings-ident-error-generic = Не вдалося зберегти профіль відправника.

# Notification rules + quiet hours (W15)
settings-notif-title = Сповіщення
settings-notif-intro = Виберіть, про які нові повідомлення сповіщати і коли не турбувати.
settings-notif-enabled-label = Показувати сповіщення про нову пошту
settings-notif-quiet-title = Тихі години
settings-notif-quiet-enabled-label = Вимикати сповіщення у визначений час
settings-notif-quiet-start = З
settings-notif-quiet-end = До
settings-notif-rules-title = Правила
settings-notif-rule-match-placeholder = Відправник, тека або тема містить…
settings-notif-rule-action-label = Дія
settings-notif-rule-notify = Сповіщати
settings-notif-rule-mute = Без звуку
settings-notif-rule-add = Додати правило
settings-notif-error = Не вдалося зберегти налаштування сповіщень.

# Saved searches → search folders (W13)
settings-search-title = Збережені пошуки
settings-search-intro = Показувати збережений пошук як теку у списку тек.
settings-search-empty = У вас ще немає збережених пошуків.
settings-search-as-folder = Показувати { $name } як теку
settings-search-as-folder-label = Показувати як теку
settings-search-error = Не вдалося змінити збережений пошук.

# Keyboard shortcuts (W14)
settings-kbd-title = Комбінації клавіш
settings-kbd-intro = Виберіть набір комбінацій клавіш. Зміни застосовуються на цьому пристрої.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Вимкнено (без комбінацій)
settings-kbd-action-compose = Написати
settings-kbd-action-archive = Архівувати
settings-kbd-action-reply = Відповісти
settings-kbd-action-next = Наступне повідомлення
settings-kbd-action-previous = Попереднє повідомлення
settings-kbd-action-search = Пошук

# Offline cache (W16)
settings-offline-title = Офлайн-кеш
settings-offline-intro = Скільки пошти зберігати на цьому пристрої для роботи офлайн і як звільняти місце.
settings-offline-budget-label = Обсяг кешу (МБ)
settings-offline-retention-label = Зберігати (днів)
settings-offline-strategy-label = Коли обсяг вичерпано
settings-offline-lru = Видаляти найдовше не використовувані
settings-offline-oldest = Видаляти спершу найстаріші
settings-offline-manual = Лише коли я очищу вручну
settings-offline-purge = Очистити офлайн-кеш зараз

# Interface direction (W20)
settings-dir-title = Напрямок інтерфейсу
settings-dir-intro = Відповідно до мови або примусово заданий напрямок. Справа наліво дзеркально відображає компонування.
settings-dir-auto = Автоматично
settings-dir-ltr = Зліва направо
settings-dir-rtl = Справа наліво
settings-dir-preview-title = Попередній перегляд
settings-dir-preview-body = Цей текст дзеркально відображається, коли напрямок — справа наліво.
settings-mail-features = Функції пошти
settings-mail-feature-allowSend = Дозволити надсилання повідомлень
settings-mail-feature-allowSend-help = Якщо вимкнено, кнопка «Надіслати» та її скорочення вимкнені. Чернетки все одно можна зберігати.
settings-mail-feature-allowReply = Дозволити «Відповісти» і «Відповісти всім»
settings-mail-feature-allowReply-help = Якщо вимкнено, дії «Відповісти» і «Відповісти всім» та їхні скорочення недоступні.
settings-mail-feature-allowForward = Дозволити пересилання
settings-mail-feature-allowForward-help = Якщо вимкнено, дії «Переслати» та їхнє скорочення недоступні.
settings-mail-feature-allowDownload = Дозволити завантаження повідомлень і вкладень
settings-mail-feature-allowDownload-help = Якщо вимкнено, завантаження повідомлень і вкладень недоступне.
settings-mail-feature-allowPrint = Дозволити друк
settings-mail-feature-allowPrint-help = Якщо вимкнено, дія «Друк» недоступна.
settings-mail-feature-showSpam = Показувати теку «Спам»
settings-mail-feature-showSpam-help = Якщо вимкнено, тека «Спам» і повідомлення, позначені як спам, приховані зі списку тек і з результатів пошуку. Нічого не видаляється.
settings-mail-feature-showAuthStatus = Показувати результати автентифікації відправника
settings-mail-feature-showAuthStatus-help = Показує в засобі читання результати SPF, DKIM і DMARC для повідомлення.
settings-mail-feature-closeOnDelete = Закривати повідомлення під час видалення
settings-mail-feature-closeOnDelete-help = Закриває відкрите повідомлення, вкладку або вікно, коли це повідомлення видалено або переміщено до кошика.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Шлях у налаштуваннях
settings-crumb-loading = Завантаження назви
settings-crumb-new = Новий
settings-crumb-edit = Змінити
settings-badge-color = Колір значка непрочитаних
settings-badge-color-accent = Як акцентний колір
settings-badge-color-accent-lighter = Світліший акцент
settings-badge-color-accent-darker = Темніший акцент
settings-badge-color-option = Значки: { $color }
settings-appearance-theme-card = Тема та перемикання
settings-appearance-theme-card-help = Виберіть тему та те, чи слідує вона за системою або часом доби.
settings-appearance-colours-card = Кольори
settings-appearance-colours-card-help = Акцент для дій і виділення та колір лічильників непрочитаних.
settings-appearance-text-card = Текст і макет
settings-appearance-text-card-help = Щільність інтерфейсу, шрифт, що використовується, і компонування пошти.
settings-sig-create-variant = Створити варіант
settings-sig-error-existing = Підпис із такою назвою вже існує. Виберіть іншу назву.
settings-sig-save-failed = Не вдалося зберегти підпис. { $reason }
settings-ident-signature-new = Нові листи
settings-ident-signature-account = Використовувати підпис облікового запису
settings-ident-signature-replies = Відповіді
settings-ident-signature-inherit = Використовувати підпис для нових листів
settings-ident-signature-unavailable = Недоступний підпис: { $name }
settings-ident-signature-help = Створіть варіанти підпису в розділі «Підписи», потім призначте один тут. Той самий варіант можна використати для обох випадків.
settings-ident-save-failed = Не вдалося зберегти профіль відправника. { $reason }
settings-search-save-failed = Не вдалося зберегти збережений пошук. { $reason }
settings-kbd-preset-title = Набір скорочень
settings-editor-discard-title = Скасувати незбережені зміни?
settings-editor-discard-detail = Ваші зміни не збережено. Продовжте редагування або скасуйте цю чернетку, щоб вийти з редактора.
settings-editor-keep = Продовжити редагування
settings-editor-discard = Скасувати зміни
settings-editor-missing = Цього елемента більше немає. Поверніться до списку й виберіть інший елемент.
settings-editor-back = Назад до списку
settings-sig-edit-title = Змінити підпис
settings-ident-edit-title = Змінити профіль відправника
settings-sections = Розділи налаштувань
settings-category-account = Обліковий запис
settings-category-mail = Пошта
settings-category-interface = Інтерфейс
settings-category-integrations = Інтеграції
settings-category-about = Про програму
settings-nav-preferences = Клавіатура й відображення
settings-nav-rules = Правила пошти
settings-nav-access = Доступ до API
settings-nav-metadata = Відомості про сервер
settings-folders-title = Теки та бічна панель
settings-folders-help = Закріплюйте теки над рештою, вибирайте значок і перетягуйте теки або використовуйте кнопки переміщення, щоб упорядкувати їх у кожній групі. Ці вибори синхронізуються з цим обліковим записом і не переміщують листи та не перейменовують теки.
settings-folders-pin = Закріпити { $name }
settings-folders-icon = Значок для { $name }
settings-folders-automatic = Автоматична роль теки
settings-folders-drag = Перетягніть { $name }, щоб змінити порядок
settings-folders-up = Перемістити { $name } угору
settings-folders-down = Перемістити { $name } донизу
settings-folders-sync-idle = Увійдіть, щоб синхронізувати налаштування тек.
settings-folders-sync-loading = Завантаження налаштувань облікового запису…
settings-folders-sync-saving = Збереження налаштувань облікового запису…
settings-folders-sync-synced = Налаштування синхронізовано з цим обліковим записом.
settings-folders-sync-error = Не вдалося синхронізувати налаштування. Ваші зміни залишаються в цьому сеансі й будуть повторно надіслані після відновлення з’єднання.
settings-folder-icon-folder = Тека
settings-folder-icon-star = Зірка
settings-folder-icon-flag = Прапорець
settings-folder-icon-tag = Мітка
settings-folder-icon-clock = Годинник
settings-folder-icon-calendar = Календар
settings-folder-icon-file = Документ
settings-folder-icon-image = Зображення
settings-folder-icon-lock = Замок
settings-folder-icon-users = Люди
settings-folder-icon-archive = Архів
settings-folder-icon-mail = Конверт
settings-folder-icon-attachment = Вкладення
settings-folder-icon-check = Галочка
settings-folder-icon-alert = Попередження
settings-search-new = Нова тека пошуку
settings-search-editor = Тека пошуку
settings-search-editor-help = Листи мають відповідати всім умовам. Залиште умову порожньою, щоб пропустити її. Теки пошуку показують відповідні листи, не переміщуючи й не копіюючи їх.
settings-search-name = Назва теки
settings-search-name-required = Введіть назву цієї теки пошуку.
settings-search-invalid = У цього збереженого пошуку недійсний фільтр. Його не можна безпечно змінити.
settings-search-saved = Теку пошуку «{ $name }» збережено.
settings-search-field-text = Шукати в усьому тексті листа
settings-search-field-from = Відправник містить
settings-search-field-to = Одержувач містить
settings-search-field-subject = Тема містить
settings-search-field-body = Текст листа містить
settings-search-field-filename = Ім’я файлу вкладення містить
settings-search-field-inMailbox = Обмежити ідентифікатором скриньки (необов’язково)
settings-search-field-after = Отримано після (дата й час UTC)
settings-search-field-before = Отримано до (дата й час UTC)
settings-search-attachment = Вкладення
settings-search-any = Будь-який лист
settings-search-with-attachments = З вкладеннями
settings-search-without-attachments = Без вкладень
settings-search-unread = Лише непрочитані листи
settings-search-flagged = Лише листи з прапорцем
settings-sessions-search = Пошук за обліковим записом, пристроєм або IP
settings-sessions-account = Обліковий запис
settings-sessions-device = Пристрій і клієнт
settings-sessions-location = IP і остання відома країна
settings-sessions-location-unavailable = Країна недоступна
settings-sessions-activity = Активність
settings-sessions-actions = Дії
settings-sessions-revoke-device = Завершити сеанс: { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Написання листів і відповіді
settings-quote-style = Стиль заголовка відповіді та пересилання
settings-quote-classic = Класична відповідь із цитатою
settings-quote-outlook = Заголовок у стилі Outlook
settings-quote-help = Стиль Outlook додає поля «Від», «Надіслано», «Кому», «Копія» і «Тема» над початковим листом у форматованому та простому тексті. Наявні чернетки не змінюються.
settings-external-warning = Попередження про зовнішніх одержувачів
settings-external-off = Вимк.
settings-external-warn = Показувати попередження
settings-external-confirm = Вимагати підтвердження перед надсиланням
settings-external-help = Перевіряти поля «Кому», «Копія» та «Прихована копія» за доменом вибраного профілю відправника та списком довірених доменів. Це захищає надсилання з цього редактора; це не правило доставки на сервері.
settings-trusted-domains = Довірені домени одержувачів
settings-trusted-domains-help = Введіть до 100 точних доменів, по одному в рядку. Піддомени потрібно вказувати окремо. Власний домен профілю відправника завжди довірений. URL, адреси електронної пошти та символи-замінники не приймаються.
settings-trusted-domains-invalid = Введіть до 100 дійсних доменних імен без URL, адрес електронної пошти та символів-замінників.
settings-trusted-domains-save = Зберегти довірені домени
settings-shortcuts-enabled = Увімкнути скорочення застосунку
settings-shortcuts-action = Дія
settings-shortcuts-binding = Призначення
settings-shortcuts-edit = Змінити скорочення
settings-shortcuts-edit-action = Змінити скорочення: { $action }
settings-shortcuts-help = Використовуйте inherit для вибраного набору, disabled — щоб прибрати скорочення, або комбінацію на кшталт Ctrl+Shift+K. У macOS Ctrl означає Cmd. Конфліктні призначення неактивні. У полях введення та діалогах звичні клавіші зберігаються; комбінації, зарезервовані браузером, можуть бути недоступні.
settings-shortcuts-invalid = Введіть inherit, disabled або дійсну одну клавішу з необов’язковими модифікаторами Ctrl, Alt і Shift.
settings-shortcuts-conflict = Дві дії використовують те саме скорочення. Змініть одне призначення; конфліктні скорочення не працюватимуть.
settings-shortcuts-discard = Скасувати незбережену зміну скорочення?
settings-shortcuts-save = Зберегти скорочення
settings-shortcuts-back = Назад до скорочень
settings-shortcuts-off = Набір «Вимк.» вимикає всі скорочення, зокрема власні призначення. Виберіть інший набір у розділі «Налаштування», щоб використовувати ці призначення.
settings-kbd-action-replyAll = Відповісти всім
settings-kbd-action-forward = Переслати
settings-kbd-action-refresh = Оновити листи
settings-kbd-action-settings = Відкрити налаштування
settings-kbd-action-trash = Перемістити до кошика
settings-kbd-action-markRead = Позначити прочитаним
settings-kbd-action-markUnread = Позначити непрочитаним
settings-kbd-action-spam = Позначити як спам
settings-kbd-action-zoomIn = Збільшити лист
settings-kbd-action-zoomOut = Зменшити лист
settings-kbd-action-zoomReset = Скинути масштаб листа
settings-kbd-action-renameFolder = Перейменувати теку
settings-kbd-action-openInTab = Відкрити лист в окремій вкладці
settings-notif-device-title = Дозвіл на цьому пристрої
settings-notif-device-help = Налаштування сповіщень прив’язані до вашого облікового запису. Дозвіл браузера стосується цього пристрою; Mailwoman запитує лише тоді, коли ви вирішите його надати.
settings-notif-ask-permission = Пропонувати дозвіл на сповіщення на нових пристроях
settings-notif-ask-permission-help = Показувати запит на дозвіл, якщо цей пристрій не дозволяв і не блокував сповіщення. Типово ввімкнено; цей параметр зберігається одразу.
settings-notif-permission-default = Дозвіл браузера ще не запитувався.
settings-notif-permission-granted = Сповіщення браузера дозволено на цьому пристрої.
settings-notif-permission-denied = Сповіщення заблоковано. Змініть дозвіл для цього сайту в налаштуваннях браузера, щоб дозволити їх.
settings-notif-permission-unsupported = Цей браузер не підтримує тут системних сповіщень.
settings-notif-permission-request = Дозволити сповіщення на цьому пристрої
settings-notif-permission-error = Браузеру не вдалося запросити дозвіл на сповіщення.
settings-notif-timezone = Часовий пояс тихих годин (IANA)
settings-notif-timezone-device = Використовувати часовий пояс цього пристрою
settings-permissions-background-title = Фонові сповіщення в цьому браузері
settings-permissions-background-help = Спершу збережіть налаштування сповіщень і дозвольте сповіщення браузера. Зареєструйте цей браузер, щоб отримувати відомості про активність облікового запису, коли застосунок закрито.
settings-permissions-push-unsupported = Фонові Web Push недоступні в цьому браузері.
settings-permissions-push-registered = Цей браузер має push-підписку. Зареєструйте її, щоб прив’язати до поточного облікового запису.
settings-permissions-push-unregistered = Цей браузер не має push-підписки.
settings-permissions-push-register = Зареєструвати фонові сповіщення
settings-permissions-push-remove = Видалити підписку браузера
settings-permissions-push-policy = Адміністратор не дозволяє фонові push-сповіщення.
settings-permissions-storage-title = Сховище браузера
settings-permissions-storage-help = Постійне сховище захищає локальні дані цього сайту від автоматичного видалення, якщо браузер його надасть. Воно діє лише на цьому пристрої й не змінює строк зберігання на сервері. Дані сайту можна очистити в налаштуваннях браузера; ненадіслані офлайн-зміни можуть бути втрачені.
settings-permissions-storage-unsupported = Запити постійного сховища недоступні в цьому браузері.
settings-permissions-storage-persistent = Браузер надав цьому сайту постійне сховище.
settings-permissions-storage-evictable = Браузер може автоматично видалити локальні дані цього сайту.
settings-permissions-storage-usage = Поточний обсяг сховища сайту: { $mib } МіБ.
settings-permissions-storage-request = Запросити постійне сховище
settings-permissions-storage-policy = Адміністратор не дозволяє запити постійного сховища.
settings-permissions-error = Не вдалося виконати дію браузера. Перевірте збережені налаштування облікового запису та дозволи браузера для цього сайту.
settings-notif-background-rules = Правила за вмістом оцінюються, поки Mailwoman відкрито. Фонові системні сповіщення пригнічуються, доки налаштовано будь-які правила за вмістом, оскільки «глухе» фонове пробудження не містить відомостей про лист.
settings-time-title = Часові пояси
settings-time-help = Виберіть, як відображається час листів, і необов’язковий часовий пояс для порівняння під час планування. Місцезнаходження ніколи не визначається за адресами електронної пошти.
settings-time-displayZone = Часовий пояс для відображення
settings-time-displayZone-help = Використовуйте local для цього пристрою або назву IANA, наприклад Europe/Lisbon.
settings-time-comparisonZone = Часовий пояс для порівняння
settings-time-comparisonZone-help = Введіть назву IANA, наприклад America/New_York, або залиште порожнім, щоб приховати порівняння.
settings-time-helpers = Показувати підказки щодо часових поясів
settings-time-save = Зберегти часовий пояс
settings-time-invalid = Введіть дійсний часовий пояс IANA. Значення local використовуйте лише для часового поясу відображення або залиште часовий пояс для порівняння порожнім.
settings-time-discard = Скасувати незбережені зміни часових поясів?
settings-folders-name = Тека
settings-folders-actions = Дії
settings-folders-edit = Змінити вигляд: { $name }
settings-folders-icon-help = Виберіть один із п’ятдесяти тематичних значків або залиште автоматичний вибір за призначенням теки. Це стосується вашого облікового запису й не перейменовує скриньку.
settings-folders-save = Зберегти вигляд теки
settings-folders-back = Назад до тек
settings-folders-discard = Скасувати незбережені зміни вигляду теки?
settings-folder-labels = Мітки теки
settings-folder-labels-help = Впорядкуйте цю теку за допомогою до 32 міток облікового запису. Мітки не призначаються листам усередині неї, і листи не переміщуються. Назви й кольори міток налаштовуються в розділі «Мітки».
settings-folders-leave-pending = Ці зміни все ще очікують синхронізації. Вийти з цього редактора?
settings-folders-retry = Повторити збереження
settings-folder-role-title = Спеціальні теки
settings-folder-role-help = Виберіть справжню теку для кожної поштової ролі. «Автоматично» використовує виявлення сервером. «Використовувати значення адміністратора» прибирає ваше перевизначення. Конфліктні або відсутні призначення вимикають цю роль, а не обирають іншу теку.
settings-folder-role-auto = Автоматичне виявлення
settings-folder-role-inherit = Використовувати значення адміністратора
settings-folder-role-path = Налаштований шлях: { $path }
settings-folder-role-missing = Налаштована тека недоступна
settings-folder-role-conflict = Це призначення відсутнє або конфліктує з іншою роллю. Ця роль недоступна, доки відповідність не буде виправлено.
settings-folder-role-refresh-error = Збережено, але список тек не вдалося оновити. Оновіть теки перед використанням нової відповідності.
settings-folder-role-unsupported = Для цих перевизначень потрібні нативні поштові теки. Облікові записи JMAP на вищестоящому сервері зберігають ролі тек свого сервера.
settings-folder-role-inbox = Вхідні
settings-folder-role-drafts = Чернетки
settings-folder-role-sent = Надіслані
settings-folder-role-trash = Кошик
settings-folder-role-junk = Спам
settings-folder-role-archive = Архів
settings-sessions-select = Вибрати
settings-sessions-select-device = Вибрати сеанс для { $device }
settings-sessions-select-visible = Вибрати всі видимі інші сеанси
settings-sessions-current-protected = Цей сеанс залишається активним, і тут його вибрати не можна. Щоб завершити його, скористайтеся «Вийти».
settings-sessions-selected-title = Завершити вибрані сеанси ({ $count })
settings-sessions-selected-confirm = Завершити лише вибрані інші сеанси. Цей сеанс залишиться активним. Про невдалі або пропущені запити буде повідомлено без автоматичних повторів.
settings-sessions-selected-action = Завершити вибрані сеанси
settings-sessions-cancel = Скасувати
settings-sessions-selected-result = Завершено: { $revoked }. Уже завершені: { $unchanged }. Не вдалося підтвердити: { $failed }. Не виконувалося: { $skipped }.
settings-sessions-refresh-failed = Не вдалося оновити список сеансів. Завантажте його знову, перш ніж вибирати інші сеанси.
settings-sig-format = Формат підпису
settings-sig-format-plain = Простий текст
settings-sig-format-html = Форматований текст (HTML)
settings-sig-html-help = HTML обмежено 2 МіБ, і він очищується перед збереженням і надсиланням. Віддалені зображення й активний вміст вилучаються.
settings-sig-html-source = Змінити вихідний HTML
settings-sig-editor-loading = Завантаження редактора підпису…
settings-sig-rich-body = Форматований підпис
settings-sig-plain-alternative = Варіант у вигляді простого тексту
settings-sig-plain-help = Листи у вигляді простого тексту використовують цей текст. Правки у форматованому вигляді оновлюють його, доки ви не змінили його вручну; правки вихідного HTML зберігають ваш варіант.
settings-sig-account-changed = Обліковий запис, до якого ви ввійшли, змінився. Відкрийте цей підпис знову в потрібному обліковому записі.
settings-sig-image-type = { $name } — не зображення PNG, JPEG, GIF або WebP.
settings-sig-image-size = { $name } не вміщується: HTML підпису разом із зображеннями обмежено 2 МіБ.
settings-sig-too-large = HTML цього підпису перевищує 2 МіБ. Вилучіть зображення або частину форматування.
settings-sig-images-help = Зображення зберігаються в підписі та надсилаються як вбудовані зображення в кожному листі.
settings-folder-icon-inbox = Вхідні
settings-folder-icon-sent = Надіслані
settings-folder-icon-edit = Змінити
settings-folder-icon-delete = Видалити
settings-folder-icon-spam = Спам
settings-folder-icon-search = Пошук
settings-folder-icon-pin = Кнопка
settings-folder-icon-link = Посилання
settings-folder-icon-shield = Щит
settings-folder-icon-unlock = Відкритий замок
settings-folder-icon-fileAudio = Аудіо
settings-folder-icon-fileVideo = Відео
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Код
settings-folder-icon-print = Друк
settings-folder-icon-download = Завантажити
settings-folder-icon-preferences = Налаштування
settings-folder-icon-dashboard = Панель
settings-folder-icon-analytics = Аналітика
settings-folder-icon-nextcloud = Хмарні файли
settings-folder-icon-domains = Домени
settings-folder-icon-twofactor = Автентифікація
settings-folder-icon-assistant = Помічник
settings-folder-icon-integrations = Інтеграції
settings-folder-icon-logs = Журнали
settings-folder-icon-home = Дім
settings-folder-icon-briefcase = Портфель
settings-folder-icon-heart = Серце
settings-folder-icon-shopping = Покупки
settings-folder-icon-education = Освіта
settings-folder-icon-travel = Подорожі
settings-folder-icon-coffee = Кава
settings-folder-icon-gift = Подарунок
settings-folder-icon-bank = Банк
settings-folder-icon-book = Книга
settings-nav-keyboard-display = Клавіатура й відображення
settings-2fa-passkey-policy-disabled = Адміністратор вимкнув вхід за допомогою ключа доступу та його реєстрацію для цього облікового запису. Наявні облікові дані залишаються в списку, щоб ви могли керувати ними або видалити їх.
settings-mail-interactions = Вибір і переміщення листів
settings-mail-modifier-selection = Вибирати листи клавішами-модифікаторами
settings-mail-modifier-selection-help = Ctrl або Command перемикає окремі листи. Shift виділяє діапазон, а Shift зі стрілками розширює його. Видима кнопка вибору лишається доступною, коли параметр вимкнено.
settings-mail-drag-folder = Перетягувати листи до тек
settings-mail-drag-folder-help = Перетягніть вибраний лист, щоб перемістити виділення, або невибраний, щоб перемістити лише його. Приймаються лише теки цього облікового запису. Користувачі сенсорних екранів і клавіатури можуть скористатися панеллю інструментів вибору.
settings-search-sections = Пошук у налаштуваннях

# -- Machine translation (2026-11): added to match en ---------------------
settings-search-no-sections = Відповідних розділів налаштувань немає.
settings-time-local = Часовий пояс пристрою
settings-time-none = Без часового поясу для порівняння
settings-folder-counts = Лічильники тек
settings-folder-counts-none = Сховати лічильники
settings-folder-counts-unread = Непрочитані повідомлення
settings-folder-counts-total = Усі повідомлення
settings-folder-counts-both = Непрочитані й усі повідомлення
settings-folder-counts-unreadIfAny = Непрочитані повідомлення, лише коли вони є
settings-folder-count-scope = Теки з лічильниками
settings-folder-count-scope-all = Усі теки
settings-folder-count-scope-main = Лише основні теки
settings-folder-count-scope-help = Основні теки: «Вхідні», «Чернетки», «Надіслані», «Архів», «Спам», «Кошик», «Вихідні» та інші службові теки. Власні теки та їхні підтеки лічильник не показують.
settings-account-unread-badge = Показувати кількість непрочитаних поруч із назвою поштової скриньки
settings-folder-counts-compact = Менші значки поза «Вхідними»
settings-focused-inbox = Важливі вхідні
settings-focused-inbox-help = Поділити «Вхідні» над списком на вкладки «Важливі» та «Інші». Листи з розсилок і від автоматичних відправників потрапляють в «Інші». Інші теки та результати пошуку показують усі повідомлення.
settings-focused-senders-title = Керування важливими відправниками
settings-focused-senders-help = Відправники, яких ви завжди переміщуєте до «Важливих» або «Інших». Видаліть відправника, щоб сортування знову виконувала автоматика.
settings-focused-senders-empty = Відправників поки немає. Виберіть у повідомленні «Завжди до Важливих» або «Завжди до Інших».
settings-focused-senders-to-focused = Завжди важливі
settings-focused-senders-to-other = Завжди інші
settings-focused-senders-remove = Видалити
settings-focused-senders-remove-label = Видалити: { $sender }
settings-focused-senders-remove-failed = Не вдалося видалити відправника. Спробуйте ще раз.
settings-folder-counts-reset = Успадковувати лічильники тек
settings-folder-counts-reset-size = Успадковувати розмір значка
settings-mail-resume-tabs = Відновлювати відкриті вкладки
settings-mail-resume-tabs-help = Відновлювати посилання на вкладки облікового запису після повторного відкриття Mailwoman. Тексти повідомлень і URL-адреси вкладень у записі робочої області не зберігаються.
settings-row-actions-mode = Швидкі дії в списку повідомлень
settings-row-actions-mode-open = Завжди видимі
settings-row-actions-mode-hover = Показувати при наведенні
settings-row-actions-mode-click = Показувати після клацання
settings-row-actions-mode-off = Вимкнено
settings-row-actions-mode-help = Як показуються швидкі дії в рядку повідомлення. Під час наведення з'являються три елементи й кнопка «Ще»; після клацання — одна кнопка, що їх відкриває; на сенсорних екранах наведення діє як клацання. «Вимкнено» прибирає їх із рядків (контекстне меню й панель інструментів і далі містять усі дії).
settings-mail-feature-showGoToTop = Показувати кнопку «Догори» в списку повідомлень
settings-mail-feature-showGoToTop-help = Додає кнопку, що прокручує список до першого повідомлення. За замовчуванням вимкнено.
settings-mail-feature-showCreateFolderButton = Показувати кнопку «Створити теку» поруч із «Написати»
settings-mail-feature-showCreateFolderButton-help = Додає на бічну панель кнопку лише зі значком для створення теки. За замовчуванням вимкнено.
settings-draft-save = Чернетки облікового запису
settings-draft-save-help = Зберігати в теку «Чернетки» вашого облікового запису для доступу з іншого пристрою. Збереження тихе; сповіщення з'являється лише в разі помилки. Непідтримувані облікові записи та захищений розшифрований вміст ніколи не надсилаються непомітно.
settings-draft-save-mode = Зберігати автоматично
settings-draft-realtime = Після паузи в наборі
settings-draft-periodic = Через рівні проміжки
settings-draft-manual = Лише коли я виберу «Зберегти»
settings-draft-debounce = Пауза в наборі (мілісекунди)
settings-draft-interval = Інтервал збереження (секунди)
settings-permissions-push-other-registration = У цьому браузері є підписка, не зареєстрована для цього облікового запису. Видаліть її, перш ніж реєструвати знову.
settings-show-from = Показувати «Від» під час написання
settings-show-from-help = Починати з видимим полем «Від». Його також можна показати для одного повідомлення на стрічці. Приймаються лише дозволені адреси відправника.
settings-sessions-table = Таблиця активних сеансів
settings-folders-table = Список тек
settings-kbd-reference = Переглянути попередньо визначені сполучення
settings-kbd-reference-help = Лише довідка щодо попередніх налаштувань. Власні сполучення й політика адміністратора можуть змінювати чинні скорочення.
settings-sessions-client-details = Відомості про клієнт
settings-2fa-passkey-rules = Ваша організація приймає лише ключі доступу, які:
settings-2fa-passkey-rule-platform = зберігаються на цьому пристрої (Windows Hello, Touch ID, блокування екрана Android)
settings-2fa-passkey-rule-cross-platform = містяться на ключі безпеки або телефоні
settings-2fa-passkey-rule-user-verification = перевіряють PIN-код, відбиток пальця або обличчя
settings-2fa-passkey-rule-resident-key = зберігаються на автентифікаторі (виявні)
settings-2fa-passkey-gap-allowed = Додано до чинної політики й не відповідає їй. Він і далі працює; додайте новий ключ доступу й видаліть цей.
settings-2fa-passkey-gap-refused = Не відповідає чинній політиці й більше не приймається для входу. Додайте новий ключ доступу й видаліть цей.
settings-2fa-passkey-browser-refused-policy = Браузер не знайшов автентифікатора, що відповідає політиці ключів доступу вашої організації, або запит було скасовано.
settings-2fa-policy-user-verification = Ваша організація вимагає ключ доступу, який перевіряє PIN-код, відбиток пальця або обличчя. Цей автентифікатор вас не перевірив.
settings-2fa-policy-platform = Ваша організація приймає лише ключі доступу, що зберігаються на цьому пристрої (наприклад, Windows Hello, Touch ID або блокування екрана Android). Ключі безпеки й телефони не приймаються.
settings-2fa-policy-cross-platform = Ваша організація приймає лише ключі безпеки або ключі доступу на телефоні. Ключі доступу, що зберігаються на цьому пристрої, не приймаються.
settings-2fa-policy-attachment-unknown = Браузер не повідомив тип цього автентифікатора, тому його не можна перевірити на відповідність політиці ключів доступу вашої організації. Спробуйте інший браузер.
settings-2fa-policy-resident-key = Ваша організація вимагає ключ доступу, що зберігається на автентифікаторі. Цей автентифікатор його не зберіг.
settings-2fa-policy-legacy = Цей ключ доступу додано до чинної політики вашої організації, і він їй більше не відповідає. Увійдіть іншим чинником або кодом відновлення, потім додайте новий ключ доступу.
settings-2fa-policy-last-factor-blocked = Ваша організація не дозволяє видаляти останній другий чинник. Спершу додайте інший спосіб входу.
settings-2fa-policy-last-factor-required = Ваша організація вимагає двофакторну автентифікацію. Додайте інший спосіб входу, перш ніж видаляти цей.
settings-2fa-last-factor-hint = Спершу додайте інший спосіб входу.
settings-2fa-last-factor-tooltip = Спершу додайте інший спосіб входу. Ваша організація вимагає принаймні один ключ доступу або застосунок-автентифікатор в обліковому записі.
settings-2fa-challenge-passkey-not-used = Ключ доступу не використано: запит скасовано або час вичерпано, або цей автентифікатор не виконує вимог вашої організації (наприклад, перевірку PIN-коду, відбитка пальця чи обличчя). Спробуйте ще раз або виберіть інший спосіб.
settings-folders-hidden-help = Приховані теки та їхні підтеки не показуються на бічній панелі. Їх не видалено, і доступ до них не обмежено. Показавши підтеку тут, ви покажете й її приховані батьківські теки. «Вхідні» завжди залишаються доступними.
settings-folders-collapse-user = Згортати користувацькі теки на бічній панелі
settings-folders-show = Показати «{ $name }» на бічній панелі
settings-folders-hide = Сховати «{ $name }» з бічної панелі
settings-folders-show-short = Показати
settings-folders-hide-short = Сховати
settings-folders-search = Пошук тек
settings-folders-select = Вибрати теку: { $name }
settings-folders-selected = Вибрано тек: { $count }
settings-folders-select-visible = Вибрати видимі
settings-folders-select-all = Вибрати всі теки
settings-folders-clear = Зняти виділення
settings-folders-pin-selected = Закріпити вибрані
settings-folders-unpin-selected = Відкріпити вибрані
settings-folders-show-selected = Показати вибрані
settings-folders-hide-selected = Сховати вибрані
settings-folders-visible = Показана
settings-folders-hidden = Прихована
settings-folders-pinned = Закріплена
settings-folders-location = Розташування
settings-folders-counts = { $unread } непрочитаних / { $total } усього
settings-folders-more = Інші дії для «{ $name }»
settings-folders-empty = Немає тек, що відповідають пошуку.
settings-folders-root = Верхній рівень
settings-folders-order-help = Перетягуйте теки, щоб змінити їхній порядок на бічній панелі. Закріплені теки лишаються разом.
cache-title = Кеш пошти та попереднє завантаження
cache-help = Зберігати нещодавно відкриті теки й повідомлення в пам'яті цієї вкладки. Кешований вміст оновлюється у фоновому режимі. Закриття вкладки або вихід із системи очищає кеш. Попереднє завантаження ніколи не позначає пошту прочитаною і не завантажує віддалені зображення.
cache-enabled = Увімкнути кеш сеансу
cache-mailboxes = Кешувати списки повідомлень тек
cache-messages = Кешувати відкриті повідомлення
cache-freshSeconds = Інтервал актуальності (секунди)
cache-maxAgeSeconds = Максимальний вік кешу (секунди)
cache-maxEntries = Максимум записів у кеші
cache-maxBytes = Ліміт пам'яті (байти)
cache-prefetchEnabled = Увімкнути випереджальне попереднє завантаження
cache-hoverEnabled = Завантажувати наперед при наведенні на теку
cache-focusEnabled = Завантажувати наперед, коли тека отримує фокус клавіатури
cache-dwellMs = Затримка вказівника та фокуса (мілісекунди)
cache-idleEnabled = Завантажувати теки наперед у простої
cache-idleDelayMs = Затримка простою (мілісекунди)
cache-maxConcurrency = Максимум одночасних попередніх завантажень
cache-maxQueued = Максимум попередніх завантажень у черзі
cache-preloadMessages = Дозволити випереджальне завантаження текстів повідомлень
cache-excludedFolders = Теки, виключені з попереднього завантаження
cache-exclusions-help = По одному правилу в рядку: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive або id: і точний ідентифікатор теки.
cache-refreshOnOpen = Оновлювати кешований вміст під час відкриття
cache-clear = Очистити кеш пошти
settings-compose-window = Відкривати нові повідомлення
settings-compose-window-tab = На вкладці робочої області
settings-compose-window-new = У новому вікні
settings-compose-window-help = Якщо браузер блокує нове вікно, редактор відкриється на вкладці робочої області.
settings-compose-read-receipt = Завжди запитувати сповіщення про прочитання
settings-compose-delivery-receipt = Завжди запитувати сповіщення про доставлення
settings-compose-reply-copy = Зберігати копію відповідей у вихідній теці
settings-compose-receipts-help = Сповіщення — це лише запити. Отримувач або його поштовий сервер може їх відхилити чи проігнорувати.
settings-compose-forward = Формат пересилання за замовчуванням
settings-compose-forward-inline = Текст повідомлення в тілі листа
settings-compose-forward-attachment = Вихідне повідомлення вкладенням .eml
settings-compose-saving = Збереження налаштування написання…
settings-compose-saved = Налаштування написання збережено
settings-ident-organization = Організація
settings-ident-auto-bcc = Автоматична прихована копія
settings-ident-auto-bcc-help = Додавати до 20 адрес у приховану копію під час надсилання з цього профілю. Розділяйте адреси комами.
settings-ident-pgp-key = Ключ OpenPGP
settings-ident-pgp-none = Використовувати ключ за замовчуванням
settings-ident-pgp-unavailable = Збережений ключ недоступний — виберіть інший ключ
settings-ident-pgp-help = Виберіть один зі своїх наявних ключів підпису OpenPGP. Закритий ключ залишається у вашому сховищі ключів.
settings-ident-error-optional = Перевірте адреси для відповіді та автоматичної прихованої копії, організацію й вибраний ключ.
settings-compose-open-tracking = За замовчуванням записувати відкриття повідомлень
settings-compose-open-tracking-help = Додає зображення відстеження до відповідних форматованих повідомлень. Відкриття залежать від того, чи завантажує отримувач зображення, і не доводять прочитання. Адміністратор може вимкнути відстеження або заблокувати це значення за замовчуванням.
settings-compose-open-tracking-inherit = Використовувати успадковане значення відстеження
settings-reply-images = Зображення у відповідях і пересиланнях
settings-reply-images-ask = Питати щоразу
settings-reply-images-allow = Завжди включати
settings-reply-images-trusted = Включати для довірених доменів відправників, інакше питати
settings-reply-images-deny = Ніколи не включати
settings-reply-images-help = Виберіть, чи міститиме відповідь або пересилання зображення вихідного повідомлення. Включені зображення копіюються в чернетку як вкладення.
settings-reply-image-domains = Довірені домени відправників зображень
settings-reply-image-domains-help = Вкажіть до 200 точних доменів відправників, по одному в рядку. Використовується, якщо вище вибрано довірені домени.
settings-reply-image-domains-save = Зберегти довірені домени зображень
settings-ident-group-basics = Основне
settings-ident-group-replies = Обробка відповідей
settings-ident-group-signature = Підпис
settings-ident-group-sending = Параметри надсилання
settings-ident-name-required = Введіть ім'я для відображення.
settings-ident-email-invalid = Введіть дійсну адресу електронної пошти, наприклад name@example.org.
settings-ident-email-duplicate = Профіль із такою адресою у вас уже є.
settings-ident-replyto-invalid = Введіть дійсну адресу електронної пошти або залиште поле порожнім.
settings-ident-replyto-help = Відповіді на повідомлення, надіслані з цього профілю, надходитимуть на цю адресу, а не на адресу відправника.
settings-ident-bcc-invalid = Перевірте список прихованої копії: дійсні адреси через кому, не більше 20.
settings-ident-delete-title = Видалити профіль { $email }?
settings-ident-delete-body = Надсилати пошту з цього профілю буде неможливо. Чернетки й надіслані повідомлення не зміняться.
settings-ident-empty = Профілів поки немає. Додайте профіль, щоб під час написання вибирати ім'я, адресу та підпис.
settings-sig-delete-title = Видалити підпис «{ $name }»?
settings-sig-delete-body = Профілі, що його використовують, перейдуть на підпис облікового запису. Уже надіслані повідомлення не зміняться.
settings-sig-deleted = Підпис видалено.
settings-reply-from = Відповідати від імені
settings-reply-from-match = Адреси, на яку надіслано повідомлення
settings-reply-from-default = Завжди мого профілю за замовчуванням
settings-reply-from-ask = Питати, якщо підходить кілька профілів
settings-reply-from-keep = Залишати вже вибраний профіль
settings-reply-from-help = Під час відповіді чи пересилання виберіть, який профіль використовувати. Якщо жоден профіль не збігається з адресою, на яку надіслано повідомлення, використовується профіль за замовчуванням. Адреси з плюсом, наприклад name+tag@example.org, відповідають name@example.org.
settings-ident-default-badge = За замовчуванням
settings-ident-make-default = Зробити основним
settings-ident-default-set = { $email } тепер ваш профіль за замовчуванням.
settings-ident-create-variant = Створити варіант
settings-ident-variant-count = { $count } { $count ->
        [one] варіант
        [few] варіанти
        [many] варіантів
       *[other] варіанта
    }
settings-ident-inherits = Використовує { $base } для: { $fields }
settings-ident-field-replyTo = адреси для відповіді
settings-ident-field-autoBcc = автоматичної прихованої копії
settings-ident-field-organization = організації
settings-ident-field-pgpKeyId = ключа OpenPGP
settings-ident-field-signatureName = підпису
settings-ident-delete-default-body = Це ваш профіль за замовчуванням. Профілем за замовчуванням стане найстарший із решти. Надсилати пошту з цього профілю буде неможливо. Чернетки й надіслані повідомлення не зміняться.
settings-ident-variants-choice-title = Видалити { $email }?
settings-ident-variants-choice-body = У цього профілю { $count } { $count ->
        [one] варіант
        [few] варіанти
        [many] варіантів
       *[other] варіанта
    }. Видаліть { $count ->
        [one] його
       *[other] їх
    } також або залиште { $count ->
        [one] його
       *[other] їх
    } як окремі профілі. Залишений варіант копіює налаштування, успадковані від цього профілю.
settings-ident-delete-with-variants = Видалити профіль і варіанти
settings-ident-delete-keep-variants = Видалити профіль, залишити варіанти
settings-ident-new-variant = Новий варіант
settings-ident-edit-variant-title = Змінити варіант
settings-ident-variant-intro = Варіант профілю { $base }. Залиште поле порожнім, щоб використати значення з цього профілю.
settings-ident-label-label = Назва варіанта
settings-ident-label-required = Введіть назву варіанта, наприклад «Розсилки».
settings-ident-inherit-hint = Порожньо: використовується { $value } з базового профілю.
settings-ident-inherit-none = нічого
settings-check-policy-allow = Надсилати без запитань
settings-check-policy-warn = Питати щоразу
settings-check-policy-warn-remember = Питати, з можливістю більше не питати
settings-check-policy-deny = Блокувати надсилання
settings-check-invalid = Недійсні адреси отримувачів
settings-check-attachment = Вкладення згадано, але його немає
settings-check-subject = Порожня тема
settings-check-help = Що робити, коли перевірка знаходить проблему. Під час запиту завжди є «Усе одно надіслати»; повідомлення зупиняє лише «Блокувати надсилання».
settings-external-warn-remember = Питати, з можливістю більше не питати
settings-external-deny = Блокувати надсилання
settings-auto-chip = Завершувати отримувача після введення пробілу, коми чи крапки з комою
settings-auto-chip-help = Дійсна адреса стає готовим отримувачем, а роздільник додається автоматично.
settings-recipient-separator = Роздільник отримувачів
settings-recipient-separator-comma = Кома
settings-recipient-separator-semicolon = Крапка з комою
settings-header-collapse = Заголовок повідомлення (Кому, Копія, Тема)
settings-header-collapse-expanded = Завжди показувати
settings-header-collapse-collapsed = Згортати в один рядок
settings-header-collapse-auto = Згортати, поки я пишу текст
settings-header-collapse-help = У згорнутому вигляді заголовок займає один рядок, наприклад «Кому: Ана +2», і розгортається після клацання або коли поле потребує уваги.
settings-retention-title = Зберігання
settings-folder-icon-camera = Камера
settings-folder-icon-music = Музика
settings-folder-icon-film = Фільм
settings-folder-icon-lightbulb = Ідея
settings-folder-icon-rocket = Ракета
settings-folder-icon-trophy = Кубок
settings-folder-icon-leaf = Листок
settings-folder-icon-globe = Глобус
settings-folder-icon-cloud = Хмара
settings-folder-icon-bookmark = Закладка
settings-folder-icon-wallet = Гаманець
settings-folder-icon-medical = Медицина
settings-folder-icon-bell = Дзвоник
settings-folder-icon-key = Ключ
settings-folder-icon-receipt = Чек
settings-folder-icon-cart = Кошик покупок
settings-folder-icon-package = Посилка
settings-folder-icon-wrench = Інструменти
settings-folder-icon-chart = Графік
settings-folder-icon-team = Команда
settings-folder-icon-store = Магазин
settings-folder-icon-office = Офіс
settings-folder-icon-laptop = Ноутбук
settings-folder-icon-crown = Корона
settings-folder-icon-family = Сім'я
settings-folder-icon-group-general = Загальні
settings-folder-icon-group-mail = Пошта
settings-folder-icon-group-work = Робота
settings-folder-icon-group-finance = Фінанси та покупки
settings-folder-icon-group-travel = Подорожі та дім
settings-folder-icon-group-personal = Особисте та розваги
settings-folder-icon-group-tech = Техніка та інструменти
settings-folder-icon-search-label = Пошук значків
settings-folder-icon-no-match = Відповідних значків немає

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Автентифікація (DMARC)

settings-badge-auth-help = Результат DMARC; він відкриває повну інформацію про безпеку.

settings-badge-density = Розмір значка

settings-badge-density-comfortable = Комфортно

settings-badge-density-compact = Компактний

settings-badge-density-help = Компактна або зручна висота та відстань для кожного значка.

settings-badge-folder = Папка

settings-badge-folder-help = Де зберігається повідомлення. Лише тоді, коли помітно, показує це для сміття та сміття.

settings-badge-labels = Ярлики значків

settings-badge-labels-help = Піктограма з текстом або одна піктограма (повний текст залишається у підказці та для програм зчитування з екрана).

settings-badge-labels-icon = Лише значок

settings-badge-labels-text = Значок і текст

settings-badge-list = Список розсилки

settings-badge-list-help = Список розсилки, з якого надійшло повідомлення; він відкриває деталі підписки та дії. Лише тоді, коли notable показує це, коли листи надходять після того, як ви скасували підписку.

settings-badge-mode-always = Завжди

settings-badge-mode-never = Ніколи

settings-badge-mode-notable = Тільки коли помітно

settings-badge-sanitized = Продезінфіковано HTML

settings-badge-sanitized-help = Що дезінфікуючий засіб HTML зробив із повідомленням. Лише тоді, коли notable показує, коли щось було заблоковано.

settings-badge-spam = Рейтинг спаму

settings-badge-spam-help = Оцінка, яку сервер-одержувач записав у заголовки повідомлення.

settings-folders-hide-empty = Приховати порожні папки на бічній панелі

settings-folders-hide-empty-help = Папки з’являються знову, коли містять повідомлення. Вхідні, папки пошуку, папки з невідомою кількістю та батьківські папки видимих ​​папок залишаються показаними. Перевизначити окремі папки в розділі «Властивості», «Відображення»; кожна папка залишається в цьому списку.

settings-folders-list-card = Список і порядок папок

settings-header-auto-collapse = Згорнути заголовок після написання (секунди)

settings-header-auto-collapse-help = Коли основний текст перебуває у фокусі стільки секунд, а заголовок має одержувача та тему, він згортається до підсумкового рядка. 0 вимикає це.

settings-message-badges = Значки повідомлень

settings-message-badges-help = Виберіть, які значки відображатимуться в заголовку відкритого повідомлення. Позначки пріоритету встановлюються в папках і бічній панелі. Ви також можете клацнути значок правою кнопкою миші, щоб приховати його.

settings-reply-from-headers = Заголовки, які вибирають ідентифікатор

settings-reply-from-headers-help = Одне ім’я заголовка на рядок у такому порядку: «Доставлено», «Кому», «Кому», «Кому», «Кому», «Переслано», «Кому», «Кому», «Копія». Пропущений заголовок не читається.

settings-reply-from-headers-invalid = Використовуйте лише ці назви, по одному на рядок: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc.

settings-reply-from-headers-save = Зберегти порядок заголовків

settings-sessions-linked-created = Посилання створено

settings-sessions-linked-details = Деталі посилання

settings-sessions-linked-kind = Тип посилання

settings-sessions-linked-kind-linked = Пов'язана поштова скринька

settings-sessions-linked-last-used = Останнє використання

settings-sessions-linked-line = Доступ до пов’язаної поштової скриньки · від { $origin }

settings-sessions-linked-line-hidden = Доступ до пов’язаної поштової скриньки

settings-sessions-linked-origin = Походить з

settings-sessions-linked-removed = Невідомо (вилучений обліковий запис)

settings-sessions-linked-revoke = Скасувати доступ за посиланням

settings-sessions-linked-revoke-label = Скасувати доступ за посиланням для { $mailbox }

settings-sessions-linked-target = Поштову скриньку відкрито

settings-sessions-origin-full = Повна адреса

settings-sessions-origin-help = Керує тим, як ця таблиця називає обліковий запис, з якого походить пов’язаний сеанс поштової скриньки. Тільки ви бачите це на своїх власних сесіях.

settings-sessions-origin-hidden = Прихований

settings-sessions-origin-masked-local-part = Адреса з маскованою першою частиною

settings-sessions-origin-setting = Показати, звідки надходить доступ до пов’язаної поштової скриньки

settings-shortcuts-search = Швидкі клавіші пошуку

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = Тримайте нещодавно відвідані папки та повідомлення готовими до відкриття. За замовчуванням копії залишаються в пам'яті цієї вкладки. Увімкніть необмежений термін зберігання, щоб зберігати зашифровані копії під час перезавантаження. Попереднє завантаження ніколи не позначає пошту як прочитану та не завантажує віддалені зображення.

cache-keepIndefinitely = Зберігати кешовану пошту необмежений час

cache-keepIndefinitely-help = За замовчуванням вимкнено. Зберігайте зашифровані копії в цьому браузері під час перезавантаження без закінчення терміну дії. Онлайн-повідомлення все ще оновлюються. Обмеження на введення та кількість байтів все ще застосовуються. Якщо вимкнути цю функцію або очистити кешовану пошту, збережені копії буде видалено. Очищення даних браузера також може видалити їх. Вкладені файли не завантажуються для використання в автономному режимі.
