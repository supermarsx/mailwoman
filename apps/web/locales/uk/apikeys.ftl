# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Ключі API
apikeys-heading = Ключі API
apikeys-intro = Створюйте ключі з обмеженими правами для скриптів та інтеграцій. Кожен ключ зберігається у вигляді хешу, показується один раз і може бути окремо відкликаний. Надавайте найменші права, яких достатньо.

apikeys-label = Мітка
apikeys-label-aria = Мітка ключа
apikeys-label-placeholder = напр., скрипт резервного копіювання

apikeys-create = Створити ключ
apikeys-saved = Я його зберіг(-ла)

apikeys-reveal-warning = Скопіюйте цей секрет зараз — він показується один раз, і отримати його знову буде неможливо.
apikeys-copy = Копіювати
apikeys-copy-aria = Копіювати секрет у буфер обміну
apikeys-copied = Скопійовано в буфер обміну

apikeys-error-need-label = дайте ключу мітку, щоб пізніше його впізнати
apikeys-error-create = не вдалося створити ключ
apikeys-error-revoke = не вдалося відкликати ключ

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Наявні ключі
apikeys-existing = Наявні ключі
apikeys-none = Ключів ще немає.
apikeys-created = створено { $date }
apikeys-last-used = востаннє використано { $date }
apikeys-never-used = ніколи не використовувався
apikeys-revoked-at = відкликано { $date }
apikeys-revoke = Відкликати
apikeys-status-active = Активний
apikeys-status-revoked = Відкликаний

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Права доступу
apikeys-capabilities = Можливості
apikeys-cap-read = Читання
apikeys-cap-send = Надсилання
apikeys-cap-delete = Видалення
apikeys-surface = Область
apikeys-surface-mail-label = Пошта
apikeys-surface-pim-label = PIM (календар / завдання / нотатки / контакти)
apikeys-accounts = Облікові записи
apikeys-all-accounts = Усі облікові записи
apikeys-folders = Теки
apikeys-all-folders = Усі теки
apikeys-folder-ids = Ідентифікатори тек (через кому)
apikeys-folder-subset-aria = Вибрані теки
apikeys-constraints = Обмеження
apikeys-expiry = Термін дії (RFC 3339, порожньо = безстроково)
apikeys-expiry-aria = Термін дії
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Обмеження частоти (запитів/хв, порожньо = без обмежень)
apikeys-rate-limit-aria = Обмеження частоти
apikeys-ip-allowlist = Список дозволених IP (CIDR/IP через кому; порожньо = будь-які)
apikeys-ip-allowlist-aria = Список дозволених IP
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = читання
apikeys-verb-send = надсилання
apikeys-verb-delete = видалення
apikeys-summary-mail = пошта
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = без дій
apikeys-summary-nothing = нічого
apikeys-summary-none = немає
apikeys-summary-verbs-on = { $verbs }: { $surfaces }
apikeys-summary-all-accounts = усі облікові записи
apikeys-summary-accounts = облікові записи: { $ids }
apikeys-summary-all-folders = усі теки
apikeys-summary-folders = теки: { $ids }
apikeys-summary-expires = діє до { $date }
apikeys-summary-rate = { $n } запит./хв
apikeys-summary-ips = IP: { $ips }
apikeys-summary-mcp-tools = Інструменти MCP: { $tools }
apikeys-summary-unattended = АВТОМАТИЧНЕ надсилання

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Ключі MCP
apikeys-mcp-heading = Ключі MCP
apikeys-mcp-intro = Надайте ШІ-агенту (через Model Context Protocol) саме ті інструменти, які ви виберете. Тексти листів, що повертаються агенту, позначаються як недовірені вхідні дані. Кожен інструмент надається окремо.
apikeys-mcp-label-aria = Мітка ключа MCP
apikeys-mcp-label-placeholder = напр., агент-помічник
apikeys-mcp-create = Створити ключ MCP
apikeys-mcp-reveal-warning = Скопіюйте цей секрет зараз — він показується один раз.
apikeys-mcp-error-need-label = дайте ключу MCP мітку
apikeys-mcp-error-need-tool = надайте принаймні один інструмент
apikeys-mcp-error-create = не вдалося створити ключ MCP

apikeys-tools = Інструменти
apikeys-tools-group = Інструменти MCP
apikeys-outbox-suffix = (через «Вихідні»)

apikeys-unattended-aria = Автоматичне надсилання
apikeys-unattended-allow = Дозволити автоматичне надсилання (в обхід «Вихідних» — потрібне підтвердження адміністратора)
apikeys-unattended-pending = Автоматичне надсилання залишається неактивним, доки адміністратор не підтвердить цей ключ.
apikeys-unattended-disclosure = Типово надане право надсилання поміщає лист до теки «Вихідні», де він чекає, поки ви підтвердите його в застосунку, — автоматизація не може надсилати самостійно. Автоматичне надсилання ПРИБИРАЄ цей крок підтвердження людиною, тож цей ключ може надсилати пошту без підтвердження. Крім того, для нього потрібне підтвердження ключа адміністратором. Надавайте його лише автоматизації, якій ви повністю довіряєте; скомпрометований ключ з автоматичним надсиланням може надсилати пошту від вашого імені без жодного запиту.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Пошук пошти
apikeys-tool-mail-search-desc = Пошук повідомлень. Результати позначаються як недовірені вхідні дані.
apikeys-tool-mail-read-label = Читання пошти
apikeys-tool-mail-read-desc = Читання тексту повідомлення. Тексти позначаються як недовірені вхідні дані.
apikeys-tool-folders-list-label = Список тек
apikeys-tool-folders-list-desc = Перелік тек поштової скриньки.
apikeys-tool-drafts-create-label = Створення чернеток
apikeys-tool-drafts-create-desc = Створення чернетки повідомлення (ніколи не надсилається автоматично).
apikeys-tool-mail-send-label = Надсилання пошти
apikeys-tool-mail-send-desc = Поміщення повідомлення до «Вихідних» для підтвердження в застосунку.
apikeys-tool-calendar-read-label = Читання календаря
apikeys-tool-calendar-read-desc = Читання подій календаря.
apikeys-tool-calendar-propose-label = Пропонування подій
apikeys-tool-calendar-propose-desc = Пропонування (без збереження) подій календаря.
apikeys-tool-tasks-read-label = Читання завдань
apikeys-tool-tasks-read-desc = Читання завдань.
apikeys-tool-tasks-write-label = Запис завдань
apikeys-tool-tasks-write-desc = Створення або оновлення завдань.
apikeys-tool-contacts-read-label = Читання контактів
apikeys-tool-contacts-read-desc = Читання контактів.
