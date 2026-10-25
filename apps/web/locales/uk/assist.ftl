# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = ШІ-помічник
assist-heading = ШІ-помічник
assist-transcript-label = Розмова з ШІ-помічником
assist-proposed-action = Запропонована дія
assist-outbox-note = Це помістить повідомлення до теки «Вихідні». Нічого не буде надіслано, доки ви не підтвердите це там.
assist-review-outbox = Переглянути у «Вихідних»
assist-review = Переглянути
assist-empty = Попросіть ШІ-помічника підсумувати, написати чернетку або щось знайти. Він може пропонувати дії, але ви завжди підтверджуєте їх самі.
assist-error = ШІ-помічник зараз не може відповісти.
assist-input-label = Повідомлення для ШІ-помічника
assist-input-placeholder = Запитайте ШІ-помічника…
assist-thinking = Обмірковування…
assist-ask = Запитати

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Допомога з текстом
assist-composer-toolbar = Інструменти редактора
assist-tool-grammar = Виправити граматику
assist-tool-rewrite = Переписати
assist-tool-tone = Змінити тон
assist-tool-translate = Перекласти
assist-toolarg-tone = Тон
assist-toolarg-translate = Мова
assist-tone-neutral = Нейтральний
assist-tone-friendly = Дружній
assist-tone-formal = Офіційний
assist-tone-concise = Стислий
assist-tone-assertive = Наполегливий
assist-busy = { $label }…
assist-composer-empty-error = Спочатку щось напишіть.
assist-composer-error = ШІ-помічник зараз не може цього виконати.
assist-suggested-edit = Запропонована правка
assist-apply-draft = Застосувати до чернетки
assist-discard = Відкинути

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Запропоновані мітки
assist-apply-auto = Застосовувати автоматично
assist-apply-label = Застосувати { $label }
assist-remove-label = Прибрати { $label }
assist-apply = Застосувати
assist-undo = Скасувати

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Утримуйте, щоб диктувати
assist-dictate-stop = Зупинити диктування
assist-dictate-hold-text = 🎙 Утримуйте, щоб диктувати
assist-dictate-listening = ● Слухаю…
assist-dictate-endpoint-note = Аудіо розшифровує кінцева точка вашого ШІ-помічника.
assist-dictate-err = Помилка диктування.
assist-transcribe-err = Не вдалося розшифрувати аудіо.
assist-mic-err = Мікрофон недоступний.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Семантичний пошук
assist-semantic-note = Текст запиту надсилається до кінцевої точки вашого ШІ-помічника для впорядкування за змістом.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Що може залишити цей пристрій
assist-left-1 = текст вибраного повідомлення (тема + вміст) для вибраної можливості
assist-left-2 = хост кінцевої точки, до якого його надіслано
assist-left-3 = ніколи: наскрізно зашифрований вміст (типово виключено)
assist-left-4 = ніколи: вкладення (типово виключено)
assist-left-5 = ніколи: ваші облікові дані чи інші облікові записи
assist-disclosure-off = ШІ-помічник вимкнений. Жоден вміст повідомлень не залишає цей пристрій.
assist-disclosure-sentence = Коли ви користуєтеся інструментом ШІ-помічника, текст вибраного повідомлення передається через проксі до { $host }. { $excl } Надсилання ніколи не автоматизоване — ви завжди підтверджуєте, перш ніж щось залишить теку «Вихідні».
assist-disclosure-excl-one = Він ніколи не надсилає { $a }.
assist-disclosure-excl-two = Він ніколи не надсилає { $a } або { $b }.
assist-disclosure-admin-allowed = Ваш адміністратор дозволив надсилати зашифрований вміст і вкладення.
assist-excluded-e2ee = наскрізно зашифрований вміст
assist-excluded-attachments = вкладення

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = ШІ-помічник
assist-admin-intro = ШІ-помічник надсилає текст повідомлень, вибраний користувачами, до кінцевої точки ШІ, налаштованої тут. Він не може надсилати, видаляти чи приймати пошту. Наскрізно зашифрований вміст і вкладення не передаються, якщо ви не дозволите їх нижче.
assist-admin-load-unreadable = Не вдається прочитати збережену конфігурацію ШІ-помічника: її ключ API не відкривається ключем цього сервера (MW_SERVER_KEY). Її збережено з іншим ключем або змінено. ШІ-помічник вимкнений. Заповніть форму, зокрема ключ API, і збережіть, щоб замінити її.
assist-admin-load-error = Не вдалося завантажити конфігурацію ШІ-помічника. Збереження вимкнено, щоб не перезаписати збережену конфігурацію.
assist-admin-status-running = ШІ-помічник працює. Запити надходять до { $host }.
assist-admin-status-running-pending = ШІ-помічник працює з конфігурацією, з якою запустився сервер. Збережені зміни наберуть чинності після перезапуску сервера.
assist-admin-status-off = ШІ-помічник вимкнений. Усі запити до ШІ-помічника відхиляються.
assist-admin-status-pending = ШІ-помічник не працює. Збережена конфігурація набере чинності після перезапуску сервера.
assist-admin-status-local-program-not-allowed = ШІ-помічник не працює. Налаштованої програми { $program } немає в MW_ASSIST_LOCAL_PROGRAMS на сервері, тому сервер її не запустив.
assist-admin-status-no-endpoint = ШІ-помічник увімкнений, але не працює, бо не налаштовано придатної кінцевої точки.
assist-admin-stop = Зупинити ШІ-помічника зараз
assist-admin-stop-note = Зупинка діє негайно для всіх користувачів і зберігається після перезапусків.
assist-admin-resume = Увімкнути ШІ-помічника
assist-admin-resume-note = Якщо сервер запустився з вимкненим ШІ-помічником, увімкнення набере чинності після перезапуску сервера.
assist-admin-kill-error = Не вдалося змінити стан увімкнення ШІ-помічника.
assist-admin-endpoint = Кінцева точка
assist-admin-endpoint-note = До цієї кінцевої точки звертається сервер; браузери — ніколи. Зміни наберуть чинності після перезапуску сервера.
assist-admin-kind = Тип кінцевої точки
assist-admin-kind-none = Немає
assist-admin-kind-open-ai-compatible = Сумісна з OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Локальна програма
assist-admin-local-program-enable = Щоб ШІ-помічник запускав програму на цьому сервері, задайте в MW_ASSIST_LOCAL_PROGRAMS абсолютний шлях до програми там, де запускається сервер (кілька шляхів розділяйте комами), потім перезапустіть сервер.
assist-admin-local-program-note = Сервер запускає цю програму для кожного запиту до ШІ-помічника і передає їй запит через стандартний вхід, зокрема текст повідомлення, вибраний користувачем. Відповідь він читає зі стандартного виходу програми. Можна вибрати лише програми, перелічені в MW_ASSIST_LOCAL_PROGRAMS на сервері.
assist-admin-local-program = Програма
assist-admin-local-program-unlisted = { $program } (не в списку)
assist-admin-local-program-unlisted-note = Цієї програми немає в MW_ASSIST_LOCAL_PROGRAMS, тому сервер її не запускає і не прийме її під час збереження. Виберіть програму зі списку або інший тип кінцевої точки.
assist-admin-local-program-required = Виберіть програму для запуску.
assist-admin-local-args = Аргументи, по одному в рядку
assist-admin-local-args-note = Кожен рядок передається програмі як один аргумент. Оболонка не використовується.
assist-admin-base-url = Базовий URL
assist-admin-base-url-required = Введіть базовий URL кінцевої точки, що починається з http:// або https://.
assist-admin-api-key = Ключ API
assist-admin-api-key-stored = Ключ збережено. Сервер більше його не показує.
assist-admin-api-key-replace = Замінити ключ API
assist-admin-api-key-replace-placeholder = Залиште порожнім, щоб зберегти наявний ключ
assist-admin-api-key-remove = Вилучити збережений ключ
assist-admin-api-key-keep = Зберегти наявний ключ
assist-admin-api-key-remove-pending = Збережений ключ буде вилучено під час збереження.
assist-admin-chat-model = Модель для чату
assist-admin-embed-model = Модель вбудовувань
assist-admin-stt-model = Модель розпізнавання мовлення
assist-admin-model = Модель
assist-admin-anthropic-version = Версія API
assist-admin-max-tokens = Максимум токенів у відповіді
assist-admin-default-placeholder = Типове значення сервера
assist-admin-grants = Можливості
assist-admin-grants-note = Можна використовувати лише позначені тут можливості.
assist-admin-cap-summarize = Підсумовування
assist-admin-cap-draft = Чернетки та переписування
assist-admin-cap-grammar = Граматика
assist-admin-cap-dictation = Диктування
assist-admin-cap-search-semantic = Семантичний пошук
assist-admin-cap-auto-tag = Автоматичні мітки
assist-admin-cap-recap = Огляд
assist-admin-cap-assistant = Чат із ШІ-помічником
assist-admin-ceilings = Обмеження даних
assist-admin-ceilings-note = Ці обмеження застосовуються до кожного запиту незалежно від можливості.
assist-admin-accounts = Ідентифікатори облікових записів, пошту яких можна надсилати, по одному в рядку
assist-admin-accounts-note = Якщо жодного облікового запису не вказано, вміст пошти до кінцевої точки не надсилається.
assist-admin-folders = Ідентифікатори тек, якими обмежити надсилання, по одному в рядку
assist-admin-folders-note = Якщо жодної теки не вказано, дозволено всі теки перелічених облікових записів.
assist-admin-allow-e2ee = Дозволити надсилання наскрізно зашифрованого вмісту
assist-admin-allow-attachments = Дозволити надсилання вкладень
assist-admin-save = Зберегти конфігурацію
assist-admin-saved = Збережено.
assist-admin-saved-pending = Збережено. Зміна набере чинності після перезапуску сервера.
assist-admin-save-error = Конфігурацію не збережено.
assist-admin-save-not-allowed = Конфігурацію не збережено, бо сервер не дозволяє цю програму. Можна використовувати лише програми, перелічені в MW_ASSIST_LOCAL_PROGRAMS.
