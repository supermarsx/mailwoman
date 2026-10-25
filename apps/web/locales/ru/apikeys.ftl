# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Ключи API
apikeys-heading = Ключи API
apikeys-intro = Создавайте ключи с ограниченной областью действия для скриптов и интеграций. Каждый ключ хранится в виде хеша, показывается один раз и может быть отозван отдельно. Выдавайте минимальную область действия, которой достаточно.

apikeys-label = Метка
apikeys-label-aria = Метка ключа
apikeys-label-placeholder = например, скрипт резервного копирования

apikeys-create = Создать ключ
apikeys-saved = Секрет сохранён

apikeys-reveal-warning = Скопируйте этот секрет сейчас — он показывается один раз, и получить его повторно невозможно.
apikeys-copy = Копировать
apikeys-copy-aria = Скопировать секрет в буфер обмена
apikeys-copied = Скопировано в буфер обмена

apikeys-error-need-label = задайте ключу метку, чтобы потом его узнать
apikeys-error-create = не удалось создать ключ
apikeys-error-revoke = не удалось отозвать ключ

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Существующие ключи
apikeys-existing = Существующие ключи
apikeys-none = Ключей пока нет.
apikeys-created = создан { $date }
apikeys-last-used = последнее использование { $date }
apikeys-never-used = не использовался
apikeys-revoked-at = отозван { $date }
apikeys-revoke = Отозвать
apikeys-status-active = Активен
apikeys-status-revoked = Отозван

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Область действия
apikeys-capabilities = Возможности
apikeys-cap-read = Чтение
apikeys-cap-send = Отправка
apikeys-cap-delete = Удаление
apikeys-surface = Раздел
apikeys-surface-mail-label = Почта
apikeys-surface-pim-label = PIM (календарь / задачи / заметки / контакты)
apikeys-accounts = Учётные записи
apikeys-all-accounts = Все учётные записи
apikeys-folders = Папки
apikeys-all-folders = Все папки
apikeys-folder-ids = Идентификаторы папок (через запятую)
apikeys-folder-subset-aria = Подмножество папок
apikeys-constraints = Ограничения
apikeys-expiry = Срок действия (RFC 3339, пусто = бессрочно)
apikeys-expiry-aria = Срок действия
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Ограничение частоты (запросов/мин, пусто = без ограничений)
apikeys-rate-limit-aria = Ограничение частоты
apikeys-ip-allowlist = Разрешённые IP (CIDR/IP через запятую; пусто = любые)
apikeys-ip-allowlist-aria = Разрешённые IP
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = чтение
apikeys-verb-send = отправка
apikeys-verb-delete = удаление
apikeys-summary-mail = почта
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = нет действий
apikeys-summary-nothing = ничего
apikeys-summary-none = нет
apikeys-summary-verbs-on = { $verbs }: { $surfaces }
apikeys-summary-all-accounts = все учётные записи
apikeys-summary-accounts = учётные записи: { $ids }
apikeys-summary-all-folders = все папки
apikeys-summary-folders = папки: { $ids }
apikeys-summary-expires = истекает { $date }
apikeys-summary-rate = { $n } запр./мин
apikeys-summary-ips = IP: { $ips }
apikeys-summary-mcp-tools = Инструменты MCP: { $tools }
apikeys-summary-unattended = Отправка БЕЗ ПОДТВЕРЖДЕНИЯ

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Ключи MCP
apikeys-mcp-heading = Ключи MCP
apikeys-mcp-intro = Предоставьте ИИ-агенту (по протоколу Model Context Protocol) ровно те инструменты, которые вы выберете. Тексты писем, возвращаемые агенту, помечаются как недоверенные входные данные. Каждый инструмент выдаётся отдельно.
apikeys-mcp-label-aria = Метка ключа MCP
apikeys-mcp-label-placeholder = например, агент-ассистент
apikeys-mcp-create = Создать ключ MCP
apikeys-mcp-reveal-warning = Скопируйте этот секрет сейчас — он показывается один раз.
apikeys-mcp-error-need-label = задайте ключу MCP метку
apikeys-mcp-error-need-tool = выдайте хотя бы один инструмент
apikeys-mcp-error-create = не удалось создать ключ MCP

apikeys-tools = Инструменты
apikeys-tools-group = Инструменты MCP
apikeys-outbox-suffix = (через «Исходящие»)

apikeys-unattended-aria = Отправка без подтверждения
apikeys-unattended-allow = Разрешить отправку без подтверждения (в обход «Исходящих» — требуется подпись администратора)
apikeys-unattended-pending = Отправка без подтверждения остаётся неактивной, пока администратор не подпишет этот ключ.
apikeys-unattended-disclosure = По умолчанию разрешённая отправка попадает в ваши «Исходящие» и ждёт, пока вы подтвердите её в приложении, — автоматизация не может отправлять самостоятельно. Отправка без подтверждения УБИРАЕТ этот шаг с участием человека, и этот ключ сможет отправлять почту без подтверждения. Дополнительно требуется, чтобы администратор подписал ключ. Выдавайте её только той автоматизации, которой вы полностью доверяете: скомпрометированный ключ с отправкой без подтверждения может отправлять почту от вашего имени без каких-либо запросов.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Поиск почты
apikeys-tool-mail-search-desc = Поиск сообщений. Результаты помечаются как недоверенные входные данные.
apikeys-tool-mail-read-label = Чтение почты
apikeys-tool-mail-read-desc = Чтение текста сообщения. Тексты помечаются как недоверенные входные данные.
apikeys-tool-folders-list-label = Список папок
apikeys-tool-folders-list-desc = Получение списка папок почтового ящика.
apikeys-tool-drafts-create-label = Создание черновиков
apikeys-tool-drafts-create-desc = Создание черновика сообщения (никогда не отправляется автоматически).
apikeys-tool-mail-send-label = Отправка почты
apikeys-tool-mail-send-desc = Помещение сообщения в «Исходящие» для подтверждения в приложении.
apikeys-tool-calendar-read-label = Чтение календаря
apikeys-tool-calendar-read-desc = Чтение событий календаря.
apikeys-tool-calendar-propose-label = Предложение событий
apikeys-tool-calendar-propose-desc = Предложение (без сохранения) событий календаря.
apikeys-tool-tasks-read-label = Чтение задач
apikeys-tool-tasks-read-desc = Чтение задач.
apikeys-tool-tasks-write-label = Запись задач
apikeys-tool-tasks-write-desc = Создание или изменение задач.
apikeys-tool-contacts-read-label = Чтение контактов
apikeys-tool-contacts-read-desc = Чтение контактов.
apikeys-surface-scripts-label = Управление скриптами Sieve
