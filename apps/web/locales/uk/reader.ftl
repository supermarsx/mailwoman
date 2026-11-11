# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Відображення повідомлень
reader-settings-help = Виберіть, як відображаються повідомлення. Значення адміністратора за замовчуванням діють, доки ви не виберете власні. Вміст повідомлень залишається ізольованим від застосунку.
reader-colorMode = Кольори повідомлень
reader-colorMode-help = «Автоматично» підлаштовує кольори повідомлень у темних темах. «Початкові» зберігають кольори відправника; «адаптивні» використовують активну тему; «інверсія» змінює кольори на протилежні, зберігаючи кольори зображень.
reader-fontSize = Розмір шрифту для читання (px)
reader-fontSize-help = Базовий розмір шрифту тексту повідомлення, від 12 до 24 пікселів. Форматування відправника все одно може задавати розміри заголовків.
reader-lineHeight = Міжрядковий інтервал під час читання
reader-lineHeight-help = Відстань між рядками тексту повідомлення, від 1,2 до 2,2 розміру шрифту.
reader-contentWidth = Ширина під час читання
reader-contentWidth-help = Обмежте довжину рядків для зручного читання або використовуйте всю область повідомлення.
reader-loading = Вигляд індикатора завантаження
reader-loading-help = Виберіть спокійні рядки-заповнювачі, текстовий індикатор або відсутність візуального індикатора. Програми зчитування з екрана все одно отримують відомості про завантаження.
reader-loadingDelayMs = Затримка індикатора завантаження (мс)
reader-loadingDelayMs-help = Не показувати індикатор, що блимає, під час швидких запитів. Затримується лише індикатор, але не саме повідомлення.
reader-option-auto = Автоматично за темою
reader-option-original = Початкові кольори повідомлення
reader-option-adaptive = Підлаштовувати під тему
reader-option-invert = Інвертувати кольори повідомлення
reader-option-comfortable = Зручна
reader-option-wide = Широка
reader-option-full = Уся область
reader-option-skeleton = Рядки-заповнювачі
reader-option-indicator = Текстовий індикатор
reader-option-none = Без візуального індикатора
reader-inherit = Використовувати значення адміністратора за замовчуванням
reader-actionStyle = Стрічка дій із повідомленням
reader-actionStyle-help = Компактні значки залишають більше місця для повідомлення. Кожна дія зберігає назву для клавіатури та пояснювальну підказку. Виберіть видимі підписи, якщо надаєте перевагу тексту поруч зі значками.
reader-option-icons = Компактні значки
reader-option-labels = Значки та підписи
reader-zoom-group = Масштаб повідомлення
reader-zoom-in = Збільшити
reader-zoom-in-help = Збільшує повідомлення. Змінюється лише розмір повідомлення.
reader-zoom-out = Зменшити
reader-zoom-out-help = Зменшує повідомлення. Змінюється лише розмір повідомлення.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Масштаб повідомлення { $level }%. Скинути до 100%
reader-zoom-reset-help = Скинути масштаб повідомлення до 100%
reader-zoom-reset-menu = Скинути масштаб ({ $level }%)
reader-zoom = Масштаб повідомлення (%)
reader-zoom-help = Розмір, у якому відкриваються повідомлення, від 50% до 200%. Масштабується лише текст повідомлення, а не застосунок, а друк завжди виконується в масштабі 100%. Кнопки масштабу поруч із діями з повідомленням теж змінюють його.
reader-attachments-summary = { $count ->
    [one] { $count } вкладення
    [few] { $count } вкладення
    [many] { $count } вкладень
   *[other] { $count } вкладення
  } · { $size }
reader-attachments-show = Показати вкладення
reader-attachments-hide = Сховати вкладення
reader-attachmentsDensity = Щільність вкладень
reader-attachmentsDensity-help = «Компактна» показує невеликі кнопки файлів; «зручна» після розгортання показує більші мініатюри.
reader-option-compact = Компактна
reader-attachmentsCollapsed = Згортати вкладення за замовчуванням
reader-attachmentsCollapsed-help = Починати кожне повідомлення з компактного підсумку вкладень. Конкретне повідомлення можна розгорнути, не змінюючи це значення за замовчуванням.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Текст і кольори
reader-card-text-help = Розмір, інтервали, ширина та кольори тексту повідомлення.
reader-card-actions = Дії та вкладення
reader-card-actions-help = Стрічка дій із повідомленням і спосіб початку роботи з вкладеннями.
reader-card-loading = Завантаження
reader-card-loading-help = Що ви бачите, поки повідомлення відкривається.
settings-mail-card-list = Список повідомлень
settings-mail-card-list-help = Дії в рядках і поведінка списку після дії з повідомленням.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Завантажити { $name }

reader-attachments-download-all = Завантажити все

reader-attachments-download-all-blocked = Політика вкладень блокує всі файли тут

reader-attachments-download-all-busy = Підготовка файлу ZIP

reader-attachments-download-all-label = Завантажте всі вкладення як файл ZIP

reader-attachments-download-all-none = Немає вкладень для завантаження

reader-attachments-download-all-warned = { $count ->
        [one] Один файл, який потребує підтвердження, пропущено. Завантажте його самостійно.
        [few] { $count } файли, які потребують підтвердження, були пропущені. Завантажуйте їх один за одним.
        [many] { $count } файлів, які потребують підтвердження, було пропущено. Завантажуйте їх один за одним.
       *[other] { $count } файли, які потребують підтвердження, були пропущені. Завантажуйте їх один за одним.
    }

reader-attachments-download-blocked = { $name } неможливо завантажити: { $reason }

reader-attachments-download-label = Завантажити { $name } ({ $size })

reader-attachments-download-off = Завантаження вимкнено

reader-attachments-inline-group = { $count ->
        [one] { $count } вбудоване зображення
        [few] { $count } вбудовані зображення
        [many] { $count } вбудованих зображень
       *[other] { $count } вбудованих зображень
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } вбудоване зображення
        [few] { $count } вбудовані зображення
        [many] { $count } вбудованих зображень
       *[other] { $count } вбудованих зображень
    } · { $size }

reader-attachmentsBarClickToToggle = Натисніть панель, щоб розгорнути або згорнути

reader-attachmentsBarClickToToggle-help = Клацніть будь-де на панелі вкладень, щоб перемкнути її. Коли вимкнено, працює лише кнопка зі стрілкою.

reader-attachmentsBarDownloadAll = Кнопка завантажити все

reader-attachmentsBarDownloadAll-help = Показати кнопку на панелі, яка завантажує вкладення як один файл ZIP.

reader-attachmentsBarShowDownloadButtons = Кнопки завантаження вкладень

reader-attachmentsBarShowDownloadButtons-help = Показувати кнопку завантаження кожного вкладення при наведенні або фокусі, завжди або ніколи.

reader-attachmentsBarZipInlineImages = Включіть вбудовані зображення в ZIP

reader-attachmentsBarZipInlineImages-help = Зображення, які з’являються всередині тексту повідомлення, залишаються поза Завантажити все, якщо цей параметр не ввімкнено.

reader-card-attachments-bar = Панель вкладень

reader-card-attachments-bar-help = Як реагує панель над вкладеннями повідомлення та що вона пропонує. Рядки, позначені як заблоковані, встановлює ваш адміністратор.

reader-option-always = Завжди

reader-option-hover = При наведенні або фокусі

reader-option-never = Ніколи

reader-card-header = Заголовок повідомлення
reader-autoHideHeader = Автоматично приховувати заголовки повідомлень
reader-autoHideHeader-help = Відкривати повідомлення зі згорнутими даними відправника, одержувачів і метаданими. Тема й дії залишаються видимими. Натисніть Показати заголовок для перегляду подробиць. Типово вимкнено.
reader-header-show = Показати заголовок
reader-header-hide = Приховати заголовок
