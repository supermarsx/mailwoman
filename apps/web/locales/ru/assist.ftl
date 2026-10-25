# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = ИИ-ассистент
assist-heading = ИИ-ассистент
assist-transcript-label = Беседа с ИИ-ассистентом
assist-proposed-action = Предлагаемое действие
assist-outbox-note = Это поместит сообщение в ваши «Исходящие». Ничего не будет отправлено, пока вы не подтвердите отправку там.
assist-review-outbox = Проверить в «Исходящих»
assist-review = Проверить
assist-empty = Попросите ИИ-ассистента кратко изложить, составить черновик или что-нибудь найти. Он может предлагать действия, но подтверждаете их всегда вы сами.
assist-error = ИИ-ассистент сейчас не может ответить.
assist-input-label = Сообщение для ИИ-ассистента
assist-input-placeholder = Спросите ИИ-ассистента…
assist-thinking = Обдумывает…
assist-ask = Спросить

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Помощь в написании
assist-composer-toolbar = Инструменты редактора
assist-tool-grammar = Исправить грамматику
assist-tool-rewrite = Переписать
assist-tool-tone = Изменить тон
assist-tool-translate = Перевести
assist-toolarg-tone = Тон
assist-toolarg-translate = Язык
assist-tone-neutral = Нейтральный
assist-tone-friendly = Дружелюбный
assist-tone-formal = Официальный
assist-tone-concise = Краткий
assist-tone-assertive = Уверенный
assist-busy = { $label }…
assist-composer-empty-error = Сначала напишите что-нибудь.
assist-composer-error = ИИ-ассистент сейчас не может это выполнить.
assist-suggested-edit = Предлагаемая правка
assist-apply-draft = Применить к черновику
assist-discard = Отклонить

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Предлагаемые метки
assist-apply-auto = Применять автоматически
assist-apply-label = Применить { $label }
assist-remove-label = Убрать { $label }
assist-apply = Применить
assist-undo = Отменить

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Удерживайте для диктовки
assist-dictate-stop = Остановить диктовку
assist-dictate-hold-text = 🎙 Удерживайте для диктовки
assist-dictate-listening = ● Слушаю…
assist-dictate-endpoint-note = Аудио расшифровывается конечной точкой вашего ИИ-ассистента.
assist-dictate-err = Ошибка диктовки.
assist-transcribe-err = Не удалось расшифровать аудио.
assist-mic-err = Микрофон недоступен.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Смысловой поиск
assist-semantic-note = Текст запроса отправляется конечной точке вашего ИИ-ассистента для ранжирования по смыслу.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Что может покинуть это устройство
assist-left-1 = выбранный текст сообщения (тема + текст) для выбранной функции
assist-left-2 = хост конечной точки, куда он был отправлен
assist-left-3 = никогда: содержимое со сквозным шифрованием (исключено по умолчанию)
assist-left-4 = никогда: вложения (исключены по умолчанию)
assist-left-5 = никогда: ваши учётные данные или другие учётные записи
assist-disclosure-off = ИИ-ассистент выключен. Никакое содержимое сообщений не покидает это устройство.
assist-disclosure-sentence = Когда вы используете инструмент ИИ-ассистента, выбранный текст сообщения передаётся через прокси на { $host }. { $excl } Отправка никогда не выполняется автоматически — вы всегда подтверждаете её, прежде чем что-либо покинет «Исходящие».
assist-disclosure-excl-one = Он никогда не отправляет { $a }.
assist-disclosure-excl-two = Он никогда не отправляет { $a } и { $b }.
assist-disclosure-admin-allowed = Ваш администратор разрешил отправку зашифрованного содержимого и вложений.
assist-excluded-e2ee = содержимое со сквозным шифрованием
assist-excluded-attachments = вложения

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = ИИ-ассистент
assist-admin-intro = ИИ-ассистент отправляет выбранный пользователями текст сообщений на настроенную здесь конечную точку ИИ. Он не может отправлять, удалять или принимать почту. Содержимое со сквозным шифрованием и вложения не передаются, если вы не разрешите это ниже.
assist-admin-load-unreadable = Сохранённую конфигурацию ИИ-ассистента невозможно прочитать: её ключ API не расшифровывается ключом этого сервера (MW_SERVER_KEY). Она была сохранена с другим ключом или была изменена. ИИ-ассистент выключен. Заполните форму, включая ключ API, и сохраните, чтобы заменить её.
assist-admin-load-error = Не удалось загрузить конфигурацию ИИ-ассистента. Сохранение отключено, чтобы не перезаписать сохранённую конфигурацию.
assist-admin-status-running = ИИ-ассистент работает. Запросы отправляются на { $host }.
assist-admin-status-running-pending = ИИ-ассистент работает с конфигурацией, с которой был запущен сервер. Сохранённые изменения вступят в силу после перезапуска сервера.
assist-admin-status-off = ИИ-ассистент выключен. Все запросы к ИИ-ассистенту отклоняются.
assist-admin-status-pending = ИИ-ассистент не работает. Сохранённая конфигурация вступит в силу после перезапуска сервера.
assist-admin-status-local-program-not-allowed = ИИ-ассистент не работает. Настроенная программа { $program } не указана в MW_ASSIST_LOCAL_PROGRAMS на сервере, поэтому сервер её не запустил.
assist-admin-status-no-endpoint = ИИ-ассистент включён, но не работает, так как не настроена пригодная конечная точка.
assist-admin-stop = Остановить ИИ-ассистента сейчас
assist-admin-stop-note = Остановка действует немедленно для всех пользователей и сохраняется после перезапусков.
assist-admin-resume = Включить ИИ-ассистента
assist-admin-resume-note = Если сервер был запущен с выключенным ИИ-ассистентом, включение вступит в силу после перезапуска сервера.
assist-admin-kill-error = Не удалось изменить состояние ИИ-ассистента.
assist-admin-endpoint = Конечная точка
assist-admin-endpoint-note = К этой конечной точке обращается сервер, браузеры — никогда. Изменения вступают в силу после перезапуска сервера.
assist-admin-kind = Тип конечной точки
assist-admin-kind-none = Нет
assist-admin-kind-open-ai-compatible = Совместимая с OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Локальная программа
assist-admin-local-program-enable = Чтобы ИИ-ассистент запускал программу на этом сервере, задайте в MW_ASSIST_LOCAL_PROGRAMS абсолютный путь к программе там, где запускается сервер (несколько путей разделяйте запятыми), затем перезапустите сервер.
assist-admin-local-program-note = Сервер запускает эту программу для каждого запроса к ИИ-ассистенту и передаёт ей запрос через стандартный ввод, включая выбранный пользователем текст сообщения. Ответ считывается из стандартного вывода программы. Можно выбрать только программы, указанные в MW_ASSIST_LOCAL_PROGRAMS на сервере.
assist-admin-local-program = Программа
assist-admin-local-program-unlisted = { $program } (не указана)
assist-admin-local-program-unlisted-note = Эта программа не указана в MW_ASSIST_LOCAL_PROGRAMS, поэтому сервер её не запускает и не примет при сохранении. Выберите указанную программу или другой тип конечной точки.
assist-admin-local-program-required = Выберите программу для запуска.
assist-admin-local-args = Аргументы, по одному на строку
assist-admin-local-args-note = Каждая строка передаётся программе как отдельный аргумент. Командная оболочка не используется.
assist-admin-base-url = Базовый URL
assist-admin-base-url-required = Введите базовый URL конечной точки, начинающийся с http:// или https://.
assist-admin-api-key = Ключ API
assist-admin-api-key-stored = Ключ сохранён. Сервер больше его не показывает.
assist-admin-api-key-replace = Заменить ключ API
assist-admin-api-key-replace-placeholder = Оставьте пустым, чтобы сохранить текущий ключ
assist-admin-api-key-remove = Удалить сохранённый ключ
assist-admin-api-key-keep = Оставить сохранённый ключ
assist-admin-api-key-remove-pending = Сохранённый ключ будет удалён при сохранении.
assist-admin-chat-model = Модель чата
assist-admin-embed-model = Модель эмбеддингов
assist-admin-stt-model = Модель распознавания речи
assist-admin-model = Модель
assist-admin-anthropic-version = Версия API
assist-admin-max-tokens = Максимум токенов в ответе
assist-admin-default-placeholder = По умолчанию сервера
assist-admin-grants = Возможности
assist-admin-grants-note = Можно использовать только отмеченные здесь возможности.
assist-admin-cap-summarize = Краткое изложение
assist-admin-cap-draft = Черновики и переписывание
assist-admin-cap-grammar = Грамматика
assist-admin-cap-dictation = Диктовка
assist-admin-cap-search-semantic = Смысловой поиск
assist-admin-cap-auto-tag = Автоматические метки
assist-admin-cap-recap = Сводка
assist-admin-cap-assistant = Чат с ИИ-ассистентом
assist-admin-ceilings = Ограничения данных
assist-admin-ceilings-note = Эти ограничения применяются ко всем запросам, независимо от возможности.
assist-admin-accounts = Идентификаторы учётных записей, почту которых можно отправлять, по одному на строку
assist-admin-accounts-note = Если ни одна учётная запись не указана, содержимое почты на конечную точку не отправляется.
assist-admin-folders = Идентификаторы папок, которыми ограничена отправка, по одному на строку
assist-admin-folders-note = Если ни одна папка не указана, разрешены все папки указанных учётных записей.
assist-admin-allow-e2ee = Разрешить отправку содержимого со сквозным шифрованием
assist-admin-allow-attachments = Разрешить отправку вложений
assist-admin-save = Сохранить конфигурацию
assist-admin-saved = Сохранено.
assist-admin-saved-pending = Сохранено. Изменение вступит в силу после перезапуска сервера.
assist-admin-save-error = Конфигурация не сохранена.
assist-admin-save-not-allowed = Конфигурация не сохранена, так как сервер не разрешает эту программу. Можно использовать только программы, указанные в MW_ASSIST_LOCAL_PROGRAMS.
