# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Разрешить отправку
admin-settings-help-mail-allow-send = Позволяет пользователям отправлять сообщения. Если выключено, редактор отключает «Отправить» и её сочетание клавиш, а сервер отклоняет отправку; черновики по-прежнему можно сохранять.
admin-settings-key-mail-allow-reply = Разрешить ответы
admin-settings-help-mail-allow-reply = Позволяет пользователям отвечать и отвечать всем из окна чтения и с помощью сочетаний клавиш. Если выключено, эти действия недоступны.
admin-settings-key-mail-allow-forward = Разрешить пересылку
admin-settings-help-mail-allow-forward = Позволяет пользователям пересылать сообщения в тексте письма или вложением. Если выключено, эти действия недоступны.
admin-settings-key-mail-allow-download = Разрешить скачивание
admin-settings-help-mail-allow-download = Позволяет пользователям скачивать сообщения и вложения. Если выключено, веб-приложение не предлагает скачивание, а сервер отклоняет экспорт сообщений.
admin-settings-key-mail-allow-print = Разрешить печать
admin-settings-help-mail-allow-print = Позволяет пользователям печатать сообщение из окна чтения. Если выключено, действие «Печать» недоступно.
admin-settings-key-mail-show-spam = Показывать папку «Спам»
admin-settings-help-mail-show-spam = Показывает папку «Спам» или «Junk» и сообщения, помеченные как спам. Если выключено, они скрыты в списке папок и в результатах поиска; ничего не удаляется.
admin-settings-key-mail-show-auth-status = Показывать результаты проверки подлинности отправителя
admin-settings-help-mail-show-auth-status = Показывает в окне чтения результаты SPF, DKIM и DMARC из заголовка Authentication-Results.
admin-settings-key-mail-close-on-delete = Закрывать сообщение при удалении
admin-settings-help-mail-close-on-delete = Закрывает открытое сообщение, вкладку или отдельное окно, когда это сообщение удалено или перемещено в корзину. Если выключено, окно чтения ведёт себя как раньше.
admin-settings-key-links-trusted-senders = Доверенные отправители ссылок
admin-settings-help-links-trusted-senders = Точные почтовые ящики отправителей, ссылки в проверенных сообщениях которых можно открывать без согласия для каждого сообщения. Один адрес на запись.
admin-settings-key-appearance-font-size = Размер шрифта интерфейса
admin-settings-help-appearance-font-size = Размер шрифта интерфейса в пикселях, от 12 до 24.
admin-settings-key-appearance-font-weight = Насыщенность шрифта интерфейса
admin-settings-help-appearance-font-weight = Базовая насыщенность текста интерфейса: 300, 400, 500 или 600. Выделенный текст сохраняет свою иерархию.
admin-settings-key-appearance-font-color = Цвет текста интерфейса
admin-settings-help-appearance-font-color = Цвет текста интерфейса в формате #RRGGBB. Оставьте пустым, чтобы следовать теме.
admin-settings-key-appearance-low-contrast = Смягчить контраст текста
admin-settings-help-appearance-low-contrast = Смягчает контраст текста интерфейса.
admin-settings-key-mail-message-list-order = Порядок списка сообщений
admin-settings-help-mail-message-list-order = Порядок списка сообщений: новые, старые, отправитель или тема.
admin-settings-key-mail-message-list-style = Стиль списка сообщений
admin-settings-help-mail-message-list-style = Показывать список сообщений карточками или строками с заголовками столбцов.
admin-settings-key-mail-message-list-preview = Предпросмотр в списке сообщений
admin-settings-help-mail-message-list-preview = Показывать фрагменты текста в списке сообщений.
admin-settings-key-mail-after-delete = После удаления сообщения
admin-settings-help-mail-after-delete = Какое сообщение показывается после удаления открытого: следующее, предыдущее или никакое.
admin-settings-key-reader-attachments-at-end = Вложения после текста
admin-settings-help-reader-attachments-at-end = Показывать вложения после текста сообщения, а не перед ним.
admin-settings-key-notifications-ask-permission = Предлагать разрешение на уведомления
admin-settings-help-notifications-ask-permission = Предлагать запрос браузера на разрешение уведомлений пользователям на устройствах, где уведомления не разрешены и не заблокированы.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Настройки
admin-settings-description = Значения для всего сервера, для домена и для пользователя. Заблокированное значение не могут изменить пользователи и более узкие области.
admin-settings-list-caption = Настройки
admin-settings-search = Поиск настроек
admin-settings-empty = Нет настроек, подходящих под запрос.
admin-settings-unavailable = Этот сервер пока не поддерживает модель настроек.
admin-settings-col-setting = Настройка
admin-settings-col-value = Значение
admin-settings-col-source = Источник
admin-settings-col-overrides = Более узкие значения
admin-settings-open = Открыть
admin-settings-overrides-none = Нет
admin-settings-overrides = Доменов: { $domains }, пользователей: { $users }
admin-settings-not-found = Настройки с именем { $key } нет.
admin-settings-value-on = Включено
admin-settings-value-off = Выключено
admin-settings-value-empty = Не задано
admin-settings-value-list = { $count ->
    [one] { $count } элемент
    [few] { $count } элемента
    [many] { $count } элементов
   *[other] { $count } элемента
}
admin-settings-detail-key = Ключ: { $key }
admin-settings-global = Для всего сервера
admin-settings-global-help = Это значение действует для всех пользователей, если только у домена или пользователя нет более узкого значения или пользователь не выбрал своё там, где это разрешено.
admin-settings-value-label = Значение
admin-settings-json-hint = JSON. Сервер проверяет значение и указывает часть, которую он не принимает.
admin-settings-json-invalid = Это некорректный JSON.
admin-settings-user-override = Пользователи могут изменять
admin-settings-user-override-help = Если включено, пользователи могут выбрать своё значение в своих настройках. Если выключено, действует значение, заданное здесь или для их домена.
admin-settings-lock-help = При блокировке значения доменов и выбор пользователей игнорируются, и это значение действует для всех.
admin-settings-save = Сохранить
admin-settings-saved = Сохранено. Изменение вступит в силу в течение 30 секунд в каждом процессе сервера.
admin-settings-saved-restart = Сохранено. Изменение вступит в силу после перезапуска.
admin-settings-dedicated = Эта настройка меняется в разделе { $section }, который сначала проверяет её последствия.
admin-settings-dedicated-link = Открыть { $section }
admin-settings-pinned = Задано переменной окружения { $name }. Сохранённое здесь значение действует, если переменная не задана.
admin-settings-lock-title = Заблокировать { $setting }?
admin-settings-lock-body = При блокировке более узкие значения из списка ниже игнорируются. Они сохраняются и снова действуют после разблокировки.
admin-settings-lock-confirm = Заблокировать
admin-settings-impact-domain = { $count ->
    [one] { $count } значение домена игнорируется
    [few] { $count } значения доменов игнорируются
    [many] { $count } значений доменов игнорируются
   *[other] { $count } значения доменов игнорируются
}
admin-settings-impact-user = { $count ->
    [one] { $count } собственный выбор пользователя игнорируется
    [few] { $count } собственных выбора пользователей игнорируются
    [many] { $count } собственных выборов пользователей игнорируются
   *[other] { $count } собственного выбора пользователей игнорируются
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } значение для пользователя игнорируется
    [few] { $count } значения для пользователей игнорируются
    [many] { $count } значений для пользователей игнорируются
   *[other] { $count } значения для пользователей игнорируются
}
admin-settings-impact-profile = { $count ->
    [one] { $count } значение профиля функций игнорируется
    [few] { $count } значения профилей функций игнорируются
    [many] { $count } значений профилей функций игнорируются
   *[other] { $count } значения профилей функций игнорируются
}
admin-settings-impact-group = { $count ->
    [one] { $count } значение группы игнорируется
    [few] { $count } значения групп игнорируются
    [many] { $count } значений групп игнорируются
   *[other] { $count } значения групп игнорируются
}
admin-settings-domains = Значения доменов
admin-settings-domains-help = Значение домена действует для пользователей этого домена вместо значения для всего сервера.
admin-settings-domains-caption = Значения доменов для { $setting }
admin-settings-domains-empty = Ни у одного домена нет своего значения.
admin-settings-domain = Домен
admin-settings-domain-value = Значение для этого домена
admin-settings-domain-locked = Заблокировано для пользователей домена
admin-settings-domain-add = Добавить значение домена
admin-settings-domain-save = Сохранить значение домена
admin-settings-domain-remove = Удалить
admin-settings-domain-remove-title = Удалить значение домена { $domain }?
admin-settings-domain-remove-body = Пользователи домена { $domain } снова получат значение для всего сервера.
admin-settings-domain-required = Введите имя домена.
admin-settings-locked-yes = Заблокировано
admin-settings-locked-no = Не заблокировано
admin-settings-users = Значения для пользователей
admin-settings-users-help = Значение, заданное администратором для одного пользователя. Для лимитов пользователь указывается адресом.
admin-settings-users-caption = Значения для пользователей: { $setting }
admin-settings-users-empty = Ни у одного пользователя нет значения, заданного администратором.
admin-settings-user = Пользователь
admin-settings-user-hint = Идентификатор учётной записи или адрес для лимитов.
admin-settings-user-value = Значение для этого пользователя
admin-settings-user-add = Добавить значение для пользователя
admin-settings-user-save = Сохранить значение пользователя
admin-settings-user-required = Введите пользователя.
admin-settings-user-remove-title = Удалить значение пользователя { $user }?
admin-settings-user-remove-body = Пользователь получит значение своего домена или значение для всего сервера.
admin-settings-source-default = Встроенное значение по умолчанию
admin-settings-source-envFallback = Переменная окружения (сохранённого значения нет)
admin-settings-source-envPin = Переменная окружения
admin-settings-source-global = Сохранено для всего сервера
admin-settings-cat-interface = Интерфейс
admin-settings-cat-appearance = Оформление
admin-settings-cat-mail = Почта
admin-settings-cat-notifications = Уведомления
admin-settings-cat-offline = Работа офлайн
admin-settings-cat-assistant = ИИ-ассистент
admin-settings-cat-upstream = Почтовый сервер
admin-settings-cat-signin = Вход
admin-settings-cat-admin = Консоль администратора
admin-settings-cat-passwords = Пароли
admin-settings-cat-limits = Лимиты и использование
admin-settings-cat-plugins = Плагины
admin-settings-cat-features = Разрешения функций
admin-settings-cat-integrations = Разрешения интеграций
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Разрешает операции с файлами, общим доступом, а также с календарём, контактами и задачами Nextcloud, связанными с учётной записью.
admin-settings-key-cap-integration-google = Мост Google
admin-settings-help-cap-integration-google = Разрешает операции моста Google, включая получение и обновление токенов.
admin-settings-key-cap-integration-microsoft = Мост Microsoft
admin-settings-help-cap-integration-microsoft = Разрешает операции моста Microsoft, включая получение и обновление токенов.
admin-settings-key-cap-integration-family = Отдельная интеграция
admin-settings-help-cap-integration-family = Разрешение для идентификатора отдельной интеграции; охват во время работы указан в каталоге функций.
admin-settings-key-cap-ui-plugin-family = Отдельный плагин интерфейса
admin-settings-help-cap-ui-plugin-family = Разрешение для одного плагина интерфейса. Используйте точный ключ политики из каталога интеграций.
admin-settings-key-cap-ai-assistant = ИИ-ассистент
admin-settings-help-cap-ai-assistant = Разрешает настройку и вызов ассистента.
admin-settings-key-cap-plugins = Плагины
admin-settings-help-cap-plugins = Разрешает операции плагинов интерфейса и движка.
admin-settings-key-cap-pim-calendar = Календарь
admin-settings-help-cap-pim-calendar = Разрешает методы календаря и подписки.
admin-settings-key-cap-pim-contacts = Контакты
admin-settings-help-cap-pim-contacts = Разрешает методы контактов и адресных книг.
admin-settings-key-cap-pim-tasks = Задачи
admin-settings-help-cap-pim-tasks = Разрешает методы задач.
admin-settings-key-cap-pim-notes = Заметки
admin-settings-help-cap-pim-notes = Разрешает методы заметок.
admin-settings-key-cap-export = Экспорт
admin-settings-help-cap-export = Разрешает экспорт почты и данных PIM.
admin-settings-key-cap-import = Импорт
admin-settings-help-cap-import = Разрешает импорт почты и данных PIM.
admin-settings-key-cap-api-keys = Ключи API
admin-settings-help-cap-api-keys = Политика ключей API. Охват применения смотрите в каталоге функций.
admin-settings-key-cap-oauth-apps = Приложения OAuth
admin-settings-help-cap-oauth-apps = Политика приложений OAuth. Охват применения смотрите в каталоге функций.
admin-settings-key-cap-rules = Правила почты
admin-settings-help-cap-rules = Разрешает управляемые методы правил почты и автоответа.
admin-settings-key-cap-rules-forward = Пересылка правилами
admin-settings-help-cap-rules-forward = Политика пересылки. Охват применения смотрите в каталоге функций.
admin-settings-key-cap-remote-images = Удалённые изображения
admin-settings-help-cap-remote-images = Разрешение для удалённых изображений. Запросами по-прежнему управляют существующие настройки приватности; проверьте охват функций.
admin-settings-key-cap-user-background = Собственный фон
admin-settings-help-cap-user-background = Разрешение для собственного фона, по умолчанию выключено. Проверьте охват функций.
admin-settings-key-cap-user-css = Собственный CSS
admin-settings-help-cap-user-css = Разрешение для собственного CSS, по умолчанию выключено. Проверьте охват функций.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-limit-attachment-bytes = Лимит размера вложения
admin-settings-help-limit-attachment-bytes = Политика вложений в байтах, по умолчанию 50000000. Применение во время работы пока не реализовано.
admin-settings-key-limit-send-per-hour = Лимит отправки в час
admin-settings-help-limit-send-per-hour = Политика отправки в час; ноль означает без ограничений. Применение во время работы пока не реализовано.
admin-settings-key-limit-recipients-per-message = Получателей в письме
admin-settings-help-limit-recipients-per-message = Политика числа получателей; ноль означает без ограничений. Применение во время работы пока не реализовано.
admin-settings-cat-import = Импорт
admin-settings-cat-webhooks = Вебхуки
admin-settings-cat-realtime = Живые обновления
admin-settings-cat-privacy = Приватность
admin-settings-cat-observability = Наблюдаемость
admin-settings-cat-egress = Исходящие соединения
admin-settings-cat-updates = Обновления
admin-settings-cat-backups = Резервные копии
admin-settings-cat-about = О программе
admin-settings-cat-config = Конфигурация
admin-settings-cat-jobs = Задания
admin-settings-cat-maintenance = Обслуживание
admin-settings-cat-health = Проверки состояния
admin-settings-cat-diagnostics = Диагностика
admin-settings-cat-users = Пользователи
admin-settings-cat-server-text = Тексты сервера
admin-settings-key-ui-language = Язык
admin-settings-help-ui-language = Язык почтового клиента для пользователей, которые не выбрали свой. «Как в браузере» следует языкам каждого браузера.
admin-settings-key-ui-notify-new-version = Уведомление о новой версии
admin-settings-help-ui-notify-new-version = Включено (по умолчанию): вошедшим пользователям показывается уведомление с кнопками «Обновить сейчас» и «Позже», если после загрузки их страницы была развёрнута более новая версия веб-приложения. Выключено: уведомление не показывается, и пользователи получают новый код при следующей перезагрузке.
admin-settings-key-ui-undo-history-size = Размер истории отмены и повтора
admin-settings-help-ui-undo-history-size = Сколько последних действий с почтой (архивация, удаление, перемещение, отметка прочитанным, флаг, метка и подобные) запоминают кнопки «Отменить» и «Повторить», от 0 до 50. 0 отключает кнопки и сочетание Ctrl+Z. По умолчанию 50. Пользователи могут выбрать свой размер, но не больше максимума ниже, если администратор не заблокирует его.
admin-settings-key-folders-reorder-enabled = Перетаскивание папок
admin-settings-help-folders-reorder-enabled = Позволяет менять порядок папок перетаскиванием на боковой панели и в настройках. Если выключено, маркеры перетаскивания и команды «Переместить» скрыты. Пользователи могут отключить это для себя, если настройка не заблокирована.
admin-settings-key-folders-reorder-allow-nesting = Вложение папок
admin-settings-help-folders-reorder-allow-nesting = Позволяет перетащить папку на другую папку, чтобы вложить её, или обратно на верхний уровень. Если выключено, каждая папка остаётся в текущей родительской, а порядок можно менять только среди папок одного уровня.
admin-settings-key-folders-reorder-allow-server-move = Перемещать папки на сервере
admin-settings-help-folders-reorder-allow-server-move = Позволяет вложению или извлечению менять настоящую родительскую папку на почтовом сервере, так что изменение видно везде. Если выключено, такие перемещения остаются расположением только в этом приложении.
admin-settings-key-folders-reorder-allow-system-folders = Перемещать системные папки
admin-settings-help-folders-reorder-allow-system-folders = Позволяет перетаскивать «Входящие», «Черновики», «Отправленные», «Спам», «Корзину» и «Архив», меняя их порядок. Если выключено, они остаются закреплёнными вверху. Порядок меняется только в этом приложении; на сервере они никогда не переименовываются и не перемещаются.
admin-settings-key-folders-reorder-max-depth = Глубина вложения папок
admin-settings-help-folders-reorder-max-depth = Наибольший уровень вложения папок, который может создать перетаскивание, от 1 до 16. Перемещение, при котором папка или её подпапка окажется глубже, отклоняется.
admin-settings-key-folders-order-locked = Зафиксировать порядок папок
admin-settings-help-folders-order-locked = Фиксирует порядок папок для всех в этой области. Перетаскивание и команды «Переместить» отключены, пользователи видят подсказку о блокировке. Закрепления и порядок, заданный администратором, сохраняются.
admin-settings-key-folders-pinned-admin = Принудительно закреплённые папки
admin-settings-help-folders-pinned-admin = Папки, показываемые в группе «Закреплённые» вверху боковой панели каждого пользователя. Одна запись в строке: role:inbox или путь папки, например Shared/Support. Пользователи не могут их открепить или переместить. Запись без подходящей папки пропускается.
admin-settings-key-folders-order-default = Порядок папок по умолчанию
admin-settings-help-folders-order-default = Порядок, в котором папки показываются пользователям, не настроившим его, от первой к последней. Одна запись в строке: role:<role> или путь папки. Папки, не вошедшие в список, идут следом в порядке сервера. Собственный порядок пользователя заменяет этот.
admin-settings-key-folders-new-folder-placement = Размещение новой папки
admin-settings-help-folders-new-folder-placement = Куда среди папок одного уровня попадает папка, появившаяся на сервере: в конец или по алфавиту.
admin-settings-key-ui-undo-history-max = Максимум истории отмены и повтора
admin-settings-help-ui-undo-history-max = Наибольшее число действий, которое любой пользователь может хранить в истории отмены и повтора, от 0 до 50. Собственный размер пользователя снижается до этого значения. 0 отключает историю для всех. История хранится во вкладке браузера и очищается при выходе.
admin-settings-key-ui-bar-mail-top-bar = Панель инструментов почты
admin-settings-help-ui-bar-mail-top-bar = «pinned» оставляет панель инструментов почты с элементами раскладки всегда видимой. «auto» прячет её за узким маркером, который открывает её при наведении, касании или переходе к нему клавишей Tab.
admin-settings-key-ui-bar-workspace-tabs = Вкладки рабочей области
admin-settings-help-ui-bar-workspace-tabs = «pinned» оставляет полосу вкладок рабочей области всегда видимой. «auto» показывает только узкий маркер с названием активной вкладки, пока пользователь не откроет её.
admin-settings-key-ui-bar-mail-ribbon = Лента почты
admin-settings-help-ui-bar-mail-ribbon = Для раскладки с лентой. «pinned» оставляет ленту открытой. «auto» показывает только названия вкладок; вкладка открывает ленту поверх списка писем.
admin-settings-key-ui-bar-composer-ribbon = Лента редактора
admin-settings-help-ui-bar-composer-ribbon = Для каждого окна составления письма. «pinned» оставляет ленту редактора открытой. «auto» показывает только названия вкладок; открытие вкладки сдвигает текст письма вниз, а не закрывает его.
admin-settings-key-appearance-theme = Тема
admin-settings-help-appearance-theme = Цветовая тема, которую видят пользователи, пока не выберут свою. Блокировка закрепляет эту тему за всеми пользователями.
admin-settings-key-appearance-accent = Акцентный цвет
admin-settings-help-appearance-accent = Акцентный цвет темы. Пусто означает собственный акцент темы.
admin-settings-key-appearance-badge-color = Цвет значка непрочитанных
admin-settings-help-appearance-badge-color = Цвет значков с числом непрочитанных. Default сохраняет цвет темы; Accent следует акцентному цвету; accent-lighter и accent-darker используют акцент светлее или темнее на 20% с чёрным или белым текстом числа. Блокировка переопределяет выбор учётной записи.
admin-settings-key-appearance-density = Плотность
admin-settings-help-appearance-density = Сколько места занимают списки и элементы управления: compact, cozy или relaxed.
admin-settings-key-appearance-font = Шрифт интерфейса
admin-settings-key-packages-allowed = Отслеживание посылок разрешено
admin-settings-help-packages-allowed = Разрешает отслеживание посылок для затронутых учётных записей. Поставщик также должен быть включён в разделе «Отслеживание посылок», а каждый пользователь должен дать согласие. Отключение этой настройки немедленно прекращает новые обращения к поставщику.
admin-settings-help-appearance-font = Выберите встроенный шрифт для учётных записей без собственного выбора шрифта или заблокируйте выбор для всех затронутых учётных записей. Шрифты загружаются с этого сервера; запрос к Google Fonts не выполняется. Default использует шрифт темы.
admin-settings-key-appearance-brand-name = Название продукта
admin-settings-help-appearance-brand-name = Название, показываемое на странице входа и во вкладке браузера.
admin-settings-key-login-notice = Сообщение на странице входа
admin-settings-help-login-notice = Короткий текст на странице входа, например контакт службы поддержки. Оставьте пустым, чтобы его не было.
admin-settings-key-mail-read-receipts = Уведомления о прочтении
admin-settings-key-receipts-dsn-allowed = Запросы статуса доставки
admin-settings-help-receipts-dsn-allowed = Разрешает учётным записям запрашивать уведомления SMTP о статусе доставки. Это не запрашивает и не доказывает, что письмо прочитано.
admin-settings-help-mail-read-receipts = Что происходит, когда открытое письмо запрашивает уведомление о прочтении: спросить пользователя, никогда не отправлять или всегда отправлять.
admin-settings-key-notifications-enabled = Уведомления о новой почте
admin-settings-help-notifications-enabled = Получают ли пользователи уведомления о новой почте. Не меняет их правила уведомлений.
admin-settings-key-offline-retention-days = Возраст офлайн-кэша
admin-settings-help-offline-retention-days = Сколько дней устройство хранит письма для работы офлайн. 0 означает отсутствие ограничения по возрасту.
admin-settings-key-offline-budget-mb = Размер офлайн-кэша
admin-settings-help-offline-budget-mb = Сколько мегабайт устройство может занять под почту офлайн. 0 означает отсутствие офлайн-кэша.
admin-settings-key-assistant-enabled = ИИ-ассистент
admin-settings-help-assistant-enabled = Видят ли пользователи ИИ-ассистента. Переключатель развёртывания в разделе «ИИ-ассистент» по-прежнему действует выше этой настройки.
admin-settings-key-assist-profile = Профиль поставщика ИИ
admin-settings-help-assist-profile = Идентификатор профиля поставщика, обслуживающего этих пользователей (ИИ-ассистент > Профили поставщиков). Пусто — используется конечная точка развёртывания. Идентификатор несуществующего профиля отключает ИИ-ассистента для этих пользователей.
admin-settings-key-upstream-default = Почтовый сервер по умолчанию
admin-settings-help-upstream-default = Почтовый сервер, на который входит адрес, если ни один домен не указывает другой. Таблица доменов имеет приоритет над этой настройкой.
admin-settings-key-upstream-proxy-timeout-secs = Тайм-аут почтового сервера
admin-settings-help-upstream-proxy-timeout-secs = Сколько секунд может занимать один запрос к почтовому серверу.
admin-settings-key-signin-new-users = Новые пользователи
admin-settings-help-signin-new-users = Кто может создать учётную запись при первом входе: любой (open), только заранее добавленные адреса (preprovisioned), они же и импортированные пользователи (provisioned) или никто новый (closed). Существующие учётные записи это не затрагивает. Меняется в политике входа.
admin-settings-key-signin-new-users-mail = Вход по паролю: новые пользователи
admin-settings-help-signin-new-users-mail = Правило для нового пользователя почтового ящика, входящего по паролю (IMAP, POP3, Exchange). Inherit использует «Новые пользователи».
admin-settings-key-signin-new-users-sso = Единый вход: новые пользователи
admin-settings-help-signin-new-users-sso = Правило для нового пользователя, входящего через единый вход или доверенный заголовок. Inherit использует «Новые пользователи».
admin-settings-key-signin-new-users-linked = Связанные ящики: новые пользователи
admin-settings-help-signin-new-users-linked = Правило привязки ящика, для которого здесь ещё нет учётной записи, по домену самого привязываемого ящика. Inherit использует «Новые пользователи».
admin-settings-key-signin-new-users-linked-imap = Связанные ящики IMAP: новые пользователи
admin-settings-help-signin-new-users-linked-imap = Правило только для связанных ящиков IMAP. Inherit использует «Связанные ящики».
admin-settings-key-signin-new-users-linked-pop3 = Связанные ящики POP3: новые пользователи
admin-settings-help-signin-new-users-linked-pop3 = Правило только для связанных ящиков POP3. Inherit использует «Связанные ящики».
admin-settings-key-signin-new-users-linked-ews = Связанные ящики Exchange: новые пользователи
admin-settings-help-signin-new-users-linked-ews = Правило только для связанных ящиков Exchange (EWS). Inherit использует «Связанные ящики».
admin-settings-key-signin-preprovision-ttl-days = Срок действия заранее добавленного пользователя (дней)
admin-settings-help-signin-preprovision-ttl-days = Как долго остаётся действительным ожидающий пользователь, добавленный заранее, если не задан другой срок.
admin-settings-key-signin-preprovision-max-ttl-days = Наибольший срок заранее добавленного пользователя (дней)
admin-settings-help-signin-preprovision-max-ttl-days = Наибольший срок, который администратор может задать ожидающему пользователю, добавленному заранее.
admin-settings-key-signin-preprovision-domain-wide = Разрешить записи для всего домена
admin-settings-help-signin-preprovision-domain-wide = Разрешает записи, допускающие любого пользователя целого домена, вплоть до лимита использований. По умолчанию выключено.
admin-settings-key-signin-preprovision-retention-days = Хранить истёкших ожидающих пользователей (дней)
admin-settings-help-signin-preprovision-retention-days = Как долго хранится истёкший ожидающий пользователь, прежде чем будет удалён.
admin-settings-key-signin-domain-mode = Домены для входа
admin-settings-help-signin-domain-mode = Может ли входить любой домен или только перечисленные. Меняется в политике входа.
admin-settings-key-signin-refusal-detail = Сообщения об отказе
admin-settings-help-signin-refusal-detail = Указывает ли отказ во входе причину (учётная запись отключена, адрес не разрешён) или даёт тот же ответ, что и при неверном пароле. Меняется в политике входа.
admin-settings-key-signin-ldap-bind-precheck = Проверка LDAP перед входом
admin-settings-help-signin-ldap-bind-precheck = Проверять пароль в каталоге до почтового сервера. Меняется в разделе «Интеграции» после пробной привязки.
admin-settings-key-twofa-pending-ttl-secs = Лимит времени второго фактора
admin-settings-help-twofa-pending-ttl-secs = Сколько секунд у пользователя есть на ввод второго фактора после пароля.
admin-settings-key-twofa-max-attempts = Попытки второго фактора
admin-settings-help-twofa-max-attempts = Сколько неверных кодов второго фактора допускается, прежде чем вход начнётся заново.
admin-settings-key-admin-session-idle-secs = Тайм-аут бездействия консоли
admin-settings-help-admin-session-idle-secs = Сколько секунд сеанс консоли администратора длится без запросов.
admin-settings-key-admin-session-absolute-secs = Предел сеанса консоли
admin-settings-help-admin-session-absolute-secs = Наибольшая длительность сеанса консоли администратора в секундах, активного или нет.
admin-settings-key-admin-session-rotate-secs = Смена идентификатора сеанса консоли
admin-settings-help-admin-session-rotate-secs = Как часто, в секундах, консоль администратора заменяет идентификатор сеанса во время работы (0 = только после подтверждения повышения прав). Смена никогда не продлевает пределы бездействия и абсолютный предел.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-cat-folders = Раскладка папок
admin-settings-key-passwd-policy-min-length = Минимальная длина пароля
admin-settings-help-passwd-policy-min-length = Наименьшее число символов в новом пароле.
admin-settings-key-passwd-policy-require-upper = Требовать заглавную букву
admin-settings-help-passwd-policy-require-upper = Новый пароль должен содержать заглавную букву.
admin-settings-key-passwd-policy-require-lower = Требовать строчную букву
admin-settings-help-passwd-policy-require-lower = Новый пароль должен содержать строчную букву.
admin-settings-key-passwd-policy-require-digit = Требовать цифру
admin-settings-help-passwd-policy-require-digit = Новый пароль должен содержать цифру.
admin-settings-key-passwd-policy-require-symbol = Требовать символ
admin-settings-help-passwd-policy-require-symbol = Новый пароль должен содержать символ, не являющийся буквой или цифрой.
admin-settings-key-limit-storage-bytes = Лимит хранилища
admin-settings-help-limit-storage-bytes = Наибольшее число байт, которое один пользователь может хранить на этом сервере. 0 означает без ограничений.
admin-settings-key-limit-send-per-day = Отправляемых писем в день
admin-settings-help-limit-send-per-day = Наибольшее число писем, которое один пользователь может отправить за день. 0 означает без ограничений.
admin-settings-key-limit-ai-calls-per-day = Обращений к ИИ-ассистенту в день
admin-settings-help-limit-ai-calls-per-day = Наибольшее число запросов к ИИ-ассистенту, которое один пользователь может сделать за день. 0 означает без ограничений.
admin-settings-key-quota-warn-thresholds = Уровни предупреждения об использовании
admin-settings-help-quota-warn-thresholds = Проценты лимита, при которых пользователи получают предупреждение, от наименьшего.
admin-settings-key-quota-warn-enabled = Предупреждения об использовании
admin-settings-help-quota-warn-enabled = Предупреждать ли пользователей при приближении к лимиту.
admin-settings-key-usage-retention-days = История использования
admin-settings-help-usage-retention-days = Сколько дней хранятся ежедневные счётчики использования.
admin-settings-key-usage-meter-api = Учитывать использование API
admin-settings-help-usage-meter-api = Учитываются ли запросы API в показателях использования.
admin-settings-key-usage-measure-interval-hours = Интервал измерения хранилища
admin-settings-help-usage-measure-interval-hours = Как часто, в часах, измеряется объём данных каждого пользователя.
admin-settings-key-assistant-rate-limit-per-min = Ограничение частоты запросов к ИИ-ассистенту
admin-settings-help-assistant-rate-limit-per-min = Наибольшее число запросов к ИИ-ассистенту в минуту для всего сервера. 0 означает без ограничений.
admin-settings-key-plugins-kv-max-key-bytes = Размер ключа плагина
admin-settings-help-plugins-kv-max-key-bytes = Самый длинный ключ, который плагин может сохранить, в байтах.
admin-settings-key-plugins-kv-max-value-bytes = Размер значения плагина
admin-settings-help-plugins-kv-max-value-bytes = Наибольшее значение, которое плагин может сохранить под одним ключом, в байтах.
admin-settings-key-plugins-kv-max-total-bytes = Хранилище плагина
admin-settings-help-plugins-kv-max-total-bytes = Наибольшее суммарное число байт, которое может сохранить один плагин.
admin-settings-key-plugins-kv-max-keys = Ключи плагина
admin-settings-help-plugins-kv-max-keys = Наибольшее число ключей, которое может сохранить один плагин.
admin-settings-key-import-max-messages = Число писем в импорте
admin-settings-help-import-max-messages = Наибольшее число писем, которое может принести один импорт.
admin-settings-key-import-max-message-bytes = Размер письма в импорте
admin-settings-help-import-max-message-bytes = Наибольшее письмо, которое принимает импорт, в байтах.
admin-settings-key-webhooks-max-attempts = Попытки вебхука
admin-settings-help-webhooks-max-attempts = Сколько раз пытаются доставить вебхук, прежде чем отказаться.
admin-settings-key-webhooks-timeout-secs = Тайм-аут вебхука
admin-settings-help-webhooks-timeout-secs = Сколько секунд может длиться одна доставка вебхука.
admin-settings-key-outbox-max-attempts = Попытки отправки
admin-settings-help-outbox-max-attempts = Сколько раз письмо передаётся почтовому серверу, прежде чем будет сообщено, что оно не отправлено.
admin-settings-key-realtime-recheck-secs = Повторная проверка живых обновлений
admin-settings-help-realtime-recheck-secs = Как часто, в секундах, открытое соединение живых обновлений проверяет, что его сеанс всё ещё действителен.
admin-settings-key-privacy-watermark-enabled = Водяной знак читателя
admin-settings-help-privacy-watermark-enabled = Показывать водяной знак с адресом читателя поверх открытых писем.
admin-settings-key-privacy-watermark-opacity = Интенсивность водяного знака
admin-settings-help-privacy-watermark-opacity = Насколько заметен водяной знак, от 0,02 (едва заметный) до 0,5.
admin-settings-key-privacy-open-tracking-retention-days = История отслеживания открытий
admin-settings-help-privacy-open-tracking-retention-days = Сколько дней хранятся записи отслеживания прочтения.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Токены отслеживания неотправленных писем
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Сколько часов токен отслеживания ждёт отправки письма, прежде чем будет удалён.
admin-settings-key-masked-email-domain = Домен скрытых адресов
admin-settings-help-masked-email-domain = Домен, в котором создаются скрытые адреса. Оставьте пустым, чтобы отключить скрытые адреса.
admin-settings-key-abuse-report-address = Адрес для жалоб на злоупотребления
admin-settings-help-abuse-report-address = Куда отправляются жалобы на спам и злоупотребления.
admin-settings-key-dlp-admin-address = Уведомления об утечке данных
admin-settings-help-dlp-admin-address = Куда отправляются уведомления системы предотвращения утечек данных.
admin-settings-key-observability-error-forward-url = Адрес пересылки ошибок
admin-settings-help-observability-error-forward-url = Адрес https, на который пересылаются отчёты об ошибках клиентов. Оставьте пустым, чтобы не пересылать.
admin-settings-key-egress-probe-url = Адрес проверки исходящих соединений
admin-settings-help-egress-probe-url = Адрес https, используемый для проверки исходящего прокси.
admin-settings-key-push-vapid-contact = Контакт для push
admin-settings-help-push-vapid-contact = Контакт (mailto: или https:), передаваемый push-службам с каждым уведомлением.
admin-settings-key-push-quiet-hours = Тихие часы push
admin-settings-help-push-quiet-hours = Часы (UTC), в которые push-уведомления не отправляются. Оставьте пустым, чтобы таких часов не было.
admin-settings-key-updates-check = Проверять обновления
admin-settings-help-updates-check = Проверяет ли сервер раз в сутки наличие нового выпуска и показывает ли его на панели.
admin-settings-key-updates-feed-url = Лента выпусков
admin-settings-help-updates-feed-url = Адрес https, который читает проверка обновлений.
admin-settings-key-updates-apply = Применять обновления
admin-settings-help-updates-apply = Можно ли запустить обновление из консоли через агент обновления хоста.
admin-settings-key-backup-retention-count = Хранимые резервные копии
admin-settings-help-backup-retention-count = Сколько резервных копий базы данных хранится.
admin-settings-key-about-security-contact = Контакт по безопасности
admin-settings-help-about-security-contact = Контакт в файле security.txt (mailto: или https:).
admin-settings-key-about-security-policy-url = Политика безопасности
admin-settings-help-about-security-policy-url = Адрес вашей политики безопасности, указанный в security.txt.
admin-settings-key-about-project-links = Ссылки раздела «О программе»
admin-settings-help-about-project-links = Ссылки, показываемые на странице «О программе», каждая с подписью и адресом https.
admin-settings-key-config-snapshots-kept = Снимки конфигурации
admin-settings-help-config-snapshots-kept = Сколько снимков конфигурации хранится.
admin-settings-key-jobs-history-days = История заданий
admin-settings-help-jobs-history-days = Сколько дней запусков заданий хранится.
admin-settings-key-maintenance-announce-minutes-default = Время предупреждения об обслуживании
admin-settings-help-maintenance-announce-minutes-default = За сколько минут до окна обслуживания пользователям сообщают о нём.
admin-settings-key-maintenance-default-message = Сообщение об обслуживании
admin-settings-help-maintenance-default-message = Текст, который пользователи видят во время обслуживания, отдельно для каждого языка.
admin-settings-key-health-disk-free-min-percent = Минимум свободного места на диске
admin-settings-help-health-disk-free-min-percent = Проверка готовности не проходит, когда свободное место на диске опускается ниже этого процента.
admin-settings-key-observability-otlp-metrics = Отправлять метрики по OTLP
admin-settings-help-observability-otlp-metrics = Отправлять метрики сервера в коллектор OTLP, заданный при запуске.
admin-settings-key-metrics-drilldown-top-n = Строки детализации метрик
admin-settings-help-metrics-drilldown-top-n = Сколько строк показывает разбивка метрик.
admin-settings-key-metrics-rollup-days = История метрик
admin-settings-help-metrics-rollup-days = Сколько дней почасовых сводок метрик хранится.
admin-settings-key-observability-client-timing-enabled = Время загрузки в браузере
admin-settings-help-observability-client-timing-enabled = Собирать время загрузки страниц из браузеров вошедших пользователей в виде анонимных сводных гистограмм. Ничто не идентифицирует человека, сеанс или страницу. Отключите, чтобы остановить сбор.
admin-settings-key-observability-client-timing-sample-rate-percent = Доля выборки времени загрузки в браузере
admin-settings-help-observability-client-timing-sample-rate-percent = Процент сеансов браузера, сообщающих время загрузки (от 1 до 100).
admin-settings-key-observability-client-timing-min-samples = Минимум выборок времени загрузки
admin-settings-help-observability-client-timing-min-samples = Медиана и 95-й процентиль времени загрузки скрыты, пока в периоде не наберётся хотя бы столько выборок.
admin-settings-key-observability-client-timing-folder-loads = Время загрузки папок
admin-settings-help-observability-client-timing-folder-loads = Измеряет, сколько времени папке требуется, чтобы показать список писем, с разбивкой на данные из кэша браузера и из сети. Включено по умолчанию. Пользователи могут отключить это для себя, если настройка не заблокирована. Требует включённого времени загрузки в браузере.
admin-settings-key-observability-client-timing-message-loads = Время загрузки писем
admin-settings-help-observability-client-timing-message-loads = Измеряет, сколько времени письму требуется, чтобы показать заголовки и очищенный текст, с разбивкой на данные из кэша или предзагруженные и из сети. Включено по умолчанию. Пользователи могут отключить это для себя, если настройка не заблокирована. Требует включённого времени загрузки в браузере.
admin-settings-key-diagnostics-retention-days = История диагностики
admin-settings-help-diagnostics-retention-days = Сколько дней диагностических запусков хранится.
admin-settings-key-data-export-link-hours = Окно скачивания экспорта данных
admin-settings-help-data-export-link-hours = Сколько часов экспорт данных можно скачивать, прежде чем он будет удалён.
admin-settings-key-dsr-enabled = Запросы на данные
admin-settings-help-dsr-enabled = Могут ли люди подавать запросы на данные (доступ, скачивание, исправление, удаление, ограничение, возражение) из настроек.
admin-settings-key-dsr-type-access = Принимать запросы на доступ
admin-settings-help-dsr-type-access = Предлагается ли пользователям тип запроса на доступ.
admin-settings-key-dsr-type-portability = Принимать запросы на скачивание (переносимую копию)
admin-settings-help-dsr-type-portability = Предлагается ли пользователям тип запроса на скачивание (переносимую копию).
admin-settings-key-dsr-type-rectification = Принимать запросы на исправление
admin-settings-help-dsr-type-rectification = Предлагается ли пользователям тип запроса на исправление.
admin-settings-key-dsr-type-erasure = Принимать запросы на удаление
admin-settings-help-dsr-type-erasure = Предлагается ли пользователям тип запроса на удаление.
admin-settings-key-dsr-type-restriction = Принимать запросы на ограничение
admin-settings-help-dsr-type-restriction = Предлагается ли пользователям тип запроса на ограничение.
admin-settings-key-dsr-type-objection = Принимать возражения
admin-settings-help-dsr-type-objection = Предлагается ли пользователям тип запроса-возражения.
admin-settings-key-dsr-type-other = Принимать прочие запросы
admin-settings-help-dsr-type-other = Предлагается ли пользователям общий тип запроса.
admin-settings-key-dsr-approval-export = Одобрение запросов на доступ и скачивание
admin-settings-help-dsr-approval-export = auto запускает экспорт после подтверждения запроса. admin ждёт одобрения администратора.
admin-settings-key-dsr-approval-erasure = Одобрение запросов на удаление
admin-settings-help-dsr-approval-erasure = auto планирует удаление после подтверждения запроса. admin ждёт одобрения администратора.
admin-settings-key-dsr-approval-other = Обработка прочих запросов
admin-settings-help-dsr-approval-other = auto помечает запрос как выполняемый после подтверждения. admin ждёт одобрения администратора.
admin-settings-key-dsr-reauth-required = Повторная аутентификация для запросов
admin-settings-help-dsr-reauth-required = Требовать свежий пароль или второй фактор для подачи запроса на экспорт или удаление и для создания ссылки на скачивание.
admin-settings-key-dsr-confirmation-required = Подтверждающее сообщение
admin-settings-help-dsr-confirmation-required = Отправлять во входящие заявителя подтверждающее сообщение с кодом; запрос не продолжается, пока код не введён.
admin-settings-key-dsr-identity-verification-required = Шаг проверки личности
admin-settings-help-dsr-identity-verification-required = Администратор должен записать, что личность заявителя проверена, прежде чем запрос будет одобрен.
admin-settings-key-dsr-erasure-grace-days = Льготный период удаления
admin-settings-help-dsr-erasure-grace-days = Число дней между одобрением и удалением. В это время заявитель может отменить запрос. 0 выполняет удаление при следующем проходе рабочего процесса.
admin-settings-key-dsr-sla-days = Дней на ответ на запрос
admin-settings-help-dsr-sla-days = Число дней от подачи до срока, показываемого пользователям и администраторам.
admin-settings-key-dsr-sla-max-extension-days = Наибольшее продление
admin-settings-help-dsr-sla-max-extension-days = Наибольшее число дней, которое администратор может добавить к сроку запроса с записанной причиной.
admin-settings-key-dsr-sla-reminder-days = Отметки приближающегося срока
admin-settings-help-dsr-sla-reminder-days = За сколько дней до срока запрос отмечается в очереди как срочный.
admin-settings-key-dsr-export-mail-allowed = Почта в экспортах
admin-settings-help-dsr-export-mail-allowed = Разрешает заявителю включать письма в экспорт. Включаются только письма из собственного кэша этого сервера; почта, остающаяся на вышестоящем почтовом сервере, не включается.
admin-settings-key-dsr-export-mail-format = Формат почты в экспортах
admin-settings-help-dsr-export-mail-format = mbox записывает один файл на папку; eml записывает один файл на письмо.
admin-settings-key-dsr-export-include-settings = Экспорт: настройки
admin-settings-help-dsr-export-include-settings = Включить настройки в экспорт.
admin-settings-key-dsr-export-include-identities = Экспорт: личности
admin-settings-help-dsr-export-include-identities = Включить личности в экспорт.
admin-settings-key-dsr-export-include-contacts = Экспорт: контакты
admin-settings-help-dsr-export-include-contacts = Включить контакты в экспорт.
admin-settings-key-dsr-export-include-calendars = Экспорт: календари
admin-settings-help-dsr-export-include-calendars = Включить календари в экспорт.
admin-settings-key-dsr-export-include-notes = Экспорт: заметки
admin-settings-help-dsr-export-include-notes = Включить заметки в экспорт.
admin-settings-key-dsr-export-include-rules = Экспорт: фильтры и правила
admin-settings-help-dsr-export-include-rules = Включить фильтры и правила в экспорт.
admin-settings-key-dsr-export-include-sessions = Экспорт: сеансы, устройства и история входов
admin-settings-help-dsr-export-include-sessions = Включить сеансы, устройства и историю входов в экспорт.
admin-settings-key-dsr-export-include-consent = Экспорт: записи согласий
admin-settings-help-dsr-export-include-consent = Включить записи согласий в экспорт.
admin-settings-key-dsr-export-include-audit = Экспорт: записи аудита о человеке
admin-settings-help-dsr-export-include-audit = Включить в экспорт записи аудита об этом человеке.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-dsr-export-max-megabytes = Предел размера экспорта
admin-settings-help-dsr-export-max-megabytes = Наибольший экспорт в мегабайтах. Разделы, которые его превысили бы, пропускаются и перечисляются в манифесте.
admin-settings-key-dsr-export-format = Форматы экспорта
admin-settings-help-dsr-export-format = json записывает файлы JSON. jsonCsv дополнительно записывает файлы CSV для данных в виде списков.
admin-settings-key-dsr-export-encrypt = Шифровать экспорты при хранении
admin-settings-help-dsr-export-encrypt = Хранить созданные экспорты зашифрованными ключом сервера, пока их не скачают или не истечёт срок.
admin-settings-key-dsr-download-link-minutes = Срок действия ссылки для скачивания
admin-settings-help-dsr-download-link-minutes = Сколько минут ссылка для скачивания работает после создания. Сам экспорт хранится в течение окна скачивания экспорта данных.
admin-settings-key-dsr-download-single-use = Одноразовая ссылка для скачивания
admin-settings-help-dsr-download-single-use = Ссылка для скачивания перестаёт работать после однократного использования.
admin-settings-key-dsr-retention-completed-days = Хранение завершённых запросов
admin-settings-help-dsr-retention-completed-days = Сколько дней завершённый запрос и его след хранятся до удаления.
admin-settings-key-dsr-notify-admins = Записывать новые запросы для администраторов
admin-settings-help-dsr-notify-admins = Записывать каждый новый запрос как событие безопасности в журнале аудита, где его видят администраторы и приёмники оповещений.
admin-settings-key-dsr-notify-requester = Сообщения о ходе выполнения заявителю
admin-settings-help-dsr-notify-requester = Класть сообщение во входящие заявителя, когда состояние запроса меняется.
admin-settings-key-dsr-contact-name = Имя контакта по защите данных
admin-settings-help-dsr-contact-name = Имя, показываемое пользователям на странице запросов на данные.
admin-settings-key-dsr-contact-email = Адрес контакта по защите данных
admin-settings-help-dsr-contact-email = Адрес электронной почты, показываемый пользователям на странице запросов на данные.
admin-settings-key-dsr-contact-url = Страница контакта по защите данных
admin-settings-help-dsr-contact-url = Ссылка, показываемая пользователям на странице запросов на данные.
admin-settings-key-dsr-on-behalf-allowed = Запросы от имени пользователя
admin-settings-help-dsr-on-behalf-allowed = Разрешает администратору подать запрос за пользователя, например полученный по почте.
admin-settings-key-dsr-rate-limit-per-day = Запросов на человека в день
admin-settings-help-dsr-rate-limit-per-day = Наибольшее число запросов, которое один человек может подать за 24 часа.
admin-settings-key-dsr-legal-hold-subjects = Юридические блокировки
admin-settings-help-dsr-legal-hold-subjects = Адреса или записи @домен, удаление которых заблокировано. Запрос на удаление для них остаётся в очереди заблокированным, пока запись не будет удалена.
admin-settings-key-scrub-enabled = Очистка данных человека
admin-settings-help-scrub-enabled = Разрешает действие «Очистить данные человека» и очистку, которая может следовать за удалением. Если выключено, ни то, ни другое не выполняется.
admin-settings-key-scrub-pseudonym-algorithm = Длина псевдонима
admin-settings-help-scrub-pseudonym-algorithm = Как создаётся ключевой псевдоним, заменяющий идентификатор: HMAC-SHA256 при текущем поколении ключа, обрезанный до 16 или 32 шестнадцатеричных символов.
admin-settings-key-scrub-include-logs = Очищать файлы журналов
admin-settings-help-scrub-include-logs = Очистка также переписывает идентификатор в ротированных файлах журналов, включая сжатые.
admin-settings-key-scrub-auto-run-on-erasure = Очистка после удаления
admin-settings-help-scrub-auto-run-on-erasure = Завершённое удаление также заменяет идентификаторы человека в истории аудита и журналах псевдонимом. По умолчанию выключено.
admin-settings-key-scrub-key-retention-days = Хранение ключа псевдонимов
admin-settings-help-scrub-key-retention-days = Сколько дней ключ псевдонимов хранится после вывода из ротации. 0 хранит его бессрочно. После удаления старые псевдонимы больше нельзя пересчитать, а созданные этим ключом метки удаления нельзя воспроизвести.
admin-settings-key-backup-propagate-erasure = Воспроизводить удаления при восстановлении
admin-settings-help-backup-propagate-erasure = Восстановление воспроизводит метки удаления новее резервной копии, так что удалённые с тех пор люди удаляются снова.
admin-settings-key-backup-tombstone-retention-days = Хранение меток удаления
admin-settings-help-backup-tombstone-retention-days = Сколько дней хранится метка удаления. 0 хранит их вечно.
admin-settings-key-dsr-text-intro = Введение к запросам на данные
admin-settings-help-dsr-text-intro = Текст над формой запроса. Одно значение на язык.
admin-settings-key-dsr-text-confirmation = Текст подтверждающего сообщения
admin-settings-help-dsr-text-confirmation = Текст, добавляемый в подтверждающее сообщение. Одно значение на язык.
admin-settings-key-dsr-text-completion = Текст сообщения о завершении
admin-settings-help-dsr-text-completion = Текст, добавляемый в сообщение, отправляемое по завершении запроса. Одно значение на язык.
admin-settings-key-server-text-family = Тексты сервера
admin-settings-help-server-text-family = Текст, который сервер вписывает в почту и уведомления, отдельно для языка и домена.
admin-settings-cat-sieve = Почтовые фильтры
admin-settings-key-sieve-max-script-bytes = Наибольший размер скрипта фильтра
admin-settings-help-sieve-max-script-bytes = Наибольшее число байт в одном скрипте фильтра Sieve.
admin-settings-key-sieve-max-scripts = Наибольшее число скриптов фильтров
admin-settings-help-sieve-max-scripts = Наибольшее число сохранённых скриптов на учётную запись.
admin-settings-key-sieve-max-redirects = Наибольшее число перенаправлений
admin-settings-help-sieve-max-redirects = Наибольшее число команд перенаправления в одном скрипте.
admin-settings-key-sieve-vacation-allowed = Действия автоответа в фильтрах
admin-settings-help-sieve-vacation-allowed = Разрешает скриптам отправлять автоответы, если их поддерживает бэкенд.
admin-settings-key-sieve-allowed-extensions = Разрешённые расширения фильтров
admin-settings-help-sieve-allowed-extensions = Необязательный список имён расширений Sieve в нижнем регистре. Бэкенд также должен поддерживать каждое расширение.
admin-settings-key-cap-sieve-raw-edit = Правка исходного кода скрипта фильтра
admin-settings-help-cap-sieve-raw-edit = Разрешает правку исходных скриптов Sieve для выбранных учётных записей.
admin-settings-key-cap-sieve-forward-external = Пересылка за пределы домена учётной записи
admin-settings-help-cap-sieve-forward-external = Разрешает внешнюю пересылку в скриптах. Для сохранения таких скриптов также требуется недавняя проверка.
admin-settings-key-session-max-concurrent = Одновременные сеансы
admin-settings-help-session-max-concurrent = Наибольшее число сеансов на учётную запись. Самые старые сеансы отзываются, когда новый вход превышает этот предел.
admin-settings-key-session-idle-secs = Тайм-аут бездействия сеанса
admin-settings-help-session-idle-secs = Завершать неактивные сеансы через столько секунд, но не больше предела развёртывания.
admin-settings-key-session-absolute-secs = Срок жизни сеанса
admin-settings-help-session-absolute-secs = Завершать сеансы через столько секунд, но не больше предела развёртывания.
admin-settings-key-twofa-remember-days = Срок запоминания устройства
admin-settings-help-twofa-remember-days = Через сколько дней запомненное устройство должно снова подтвердить второй фактор. Ноль отключает эту функцию.
admin-settings-key-twofa-passkeys-enabled = Разрешить ключи доступа
admin-settings-help-twofa-passkeys-enabled = Разрешает ключи доступа как фактор аутентификации.
admin-settings-key-twofa-required = Требовать двухфакторную аутентификацию
admin-settings-help-twofa-required = Требует зарегистрированный второй фактор перед выдачей сеанса.
admin-settings-key-twofa-passkey-attachment = Тип аутентификатора ключа доступа
admin-settings-help-twofa-passkey-attachment = any (по умолчанию): любой аутентификатор. platform: ключи доступа, хранящиеся на устройстве (Windows Hello, Touch ID, блокировка экрана Android). cross-platform: ключи безопасности и телефоны. Это сообщает браузер; аутентификатор это не подписывает.
admin-settings-key-twofa-passkey-user-verification = Проверка пользователя ключом доступа
admin-settings-help-twofa-passkey-user-verification = required: ключ доступа должен проверять PIN или биометрию при добавлении и при каждом входе; процедура без этого отклоняется. preferred (по умолчанию) и discouraged меняют только то, что запрашивается у браузера.
admin-settings-key-twofa-passkey-resident-key = Обнаруживаемые ключи доступа
admin-settings-help-twofa-passkey-resident-key = required: учётные данные должны храниться на аутентификаторе. discouraged (по умолчанию) или preferred. В режиме движка ключи доступа всегда обнаруживаемы, так как с их помощью можно входить и без пароля.
admin-settings-key-twofa-passkey-legacy = Ключи доступа, добавленные до текущей политики
admin-settings-help-twofa-passkey-legacy = allow (по умолчанию): более старый ключ доступа, не отвечающий политике, продолжает работать, а пользователя просят заменить его. refuse: он отклоняется при входе с сообщением; пользователь входит другим фактором или кодом восстановления.
admin-settings-key-twofa-last-factor-removal = Удаление последнего второго фактора
admin-settings-help-twofa-last-factor-removal = Когда удаление ключа доступа или приложения-аутентификатора оставило бы учётную запись без таковых. allow (по умолчанию): он удаляется, и для входа нужен только пароль. block: пользователь сначала должен добавить другой способ входа (коды восстановления не считаются). Там, где двухфакторная аутентификация обязательна, последний фактор всегда сохраняется.
admin-settings-key-privacy-remote-content = Удалённое содержимое
admin-settings-help-privacy-remote-content = Блокировать изображения, спрашивать сначала, использовать прокси или загружать напрямую, если это разрешено.
admin-settings-key-privacy-trackers = Блокировать трекеры
admin-settings-help-privacy-trackers = Блокирует известные отслеживающие изображения в письмах.
admin-settings-key-privacy-clean-links = Очищать ссылки
admin-settings-help-privacy-clean-links = Удаляет параметры отслеживания из ссылок в письмах.
admin-settings-key-privacy-max-bytes = Лимит удалённого изображения
admin-settings-help-privacy-max-bytes = Наибольшее число байт, загружаемых для одного удалённого изображения.
admin-settings-key-privacy-cache-bytes = Кэш удалённых изображений
admin-settings-help-privacy-cache-bytes = Наибольшее число байт, хранимых в кэше удалённых изображений.
admin-settings-key-privacy-rate-per-min = Частота запросов удалённых изображений
admin-settings-help-privacy-rate-per-min = Наибольшее число загрузок удалённых изображений на учётную запись в минуту.
admin-settings-privacy-fetch-title = Загрузка удалённых изображений
admin-settings-privacy-fetch-help = Ограничения по времени, перенаправления и параллельные загрузки изображений, которые сервер загружает для читателя. Изображение, превысившее лимит, показывается заглушкой.
admin-settings-key-privacy-fetch-connect-timeout-ms = Тайм-аут соединения с удалённым изображением
admin-settings-help-privacy-fetch-connect-timeout-ms = Миллисекунды, отведённые на открытие соединения с хостом изображения, включая подтверждение связи TLS. По умолчанию 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Тайм-аут ответа удалённого изображения
admin-settings-help-privacy-fetch-headers-timeout-ms = Миллисекунды от начала запроса до ответа хоста изображения заголовками. По умолчанию 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Тайм-аут остановки удалённого изображения
admin-settings-help-privacy-fetch-idle-timeout-ms = Наибольшая пауза в миллисекундах между двумя частями загрузки изображения, после которой загрузка прекращается. По умолчанию 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Общий тайм-аут удалённого изображения
admin-settings-help-privacy-fetch-total-timeout-ms = Миллисекунды, отведённые на загрузку изображения, включая перенаправления. У ожидания в очереди отдельный бюджет, поэтому занятое изображение не расходует сетевое время до начала. По умолчанию 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Тайм-аут очереди удалённых изображений
admin-settings-help-privacy-fetch-queue-timeout-ms = Наибольшее число миллисекунд, которое изображения могут ждать освобождения ресурсов сверх бюджета загрузки. Ограниченная очередь выдерживает крупные всплески аутентифицированных запросов, а лимиты активных загрузок и декодирования продолжают действовать. По умолчанию 60000.
admin-settings-key-privacy-max-redirects = Перенаправления удалённых изображений
admin-settings-help-privacy-max-redirects = Сколько перенаправлений выполняется для одного изображения. Каждое проверяется по тем же правилам адресов, что и первый запрос. По умолчанию 3.
admin-settings-key-privacy-max-concurrent-per-message = Загрузки удалённых изображений на письмо
admin-settings-help-privacy-max-concurrent-per-message = Сколько изображений одного письма загружается одновременно. Остальные ждут свободного места. Диапазон 1–128; по умолчанию 30.
admin-settings-key-privacy-max-concurrent-per-account = Загрузки удалённых изображений на учётную запись
admin-settings-help-privacy-max-concurrent-per-account = Сколько изображений загружается одновременно для одной учётной записи по всем её открытым письмам. Диапазон 1–256; по умолчанию 128.
admin-settings-key-privacy-allow-direct = Разрешить прямые изображения
admin-settings-key-cap-remote-content-direct = Разрешение на прямые изображения
admin-settings-help-cap-remote-content-direct = Разрешает этой учётной записи выбирать прямые изображения, если сервер это допускает.
admin-settings-help-privacy-allow-direct = Разрешает прямую загрузку изображений для учётных записей с нужным разрешением.
admin-settings-key-logs-auth-retention-days = Хранение истории аутентификации
admin-settings-help-logs-auth-retention-days = Сколько дней хранятся метаданные аутентификации независимо от постоянной идентичности учётной записи.
admin-settings-key-logs-auth-max-entries = Предел записей истории аутентификации
admin-settings-help-logs-auth-max-entries = Наибольшее число хранимых записей. Совпадающие повторные неудачи в пределах одной минуты делят одну запись.
admin-settings-key-reader-color-mode = Цвета писем
admin-settings-cat-reader = Отображение писем
admin-settings-key-mail-row-actions-mode = Быстрые действия в списке писем
admin-settings-help-mail-row-actions-mode = Как выглядят быстрые действия в каждой строке письма: open (всегда доступны, показываются при наведении или фокусе), hover (три элемента и кнопка «Ещё»), click (одна кнопка, открывающая их) или off (убраны из строк; контекстное меню и панель инструментов сохраняют все действия). На сенсорных экранах hover работает как click. Пользователи могут выбрать своё, если администратор не заблокирует.
admin-settings-key-mail-quick-reply = Быстрый ответ
admin-settings-help-mail-quick-reply = Предлагает компактную вкладку ответа с уже заполненными получателями. По умолчанию выключено; пользователи могут включить, если администратор не заблокировал. Обычные защиты отправки продолжают действовать.
admin-settings-key-mail-request-read-receipt = Запрашивать уведомления о прочтении по умолчанию
admin-settings-help-mail-request-read-receipt = Начинать поддерживаемые исходящие письма с выбранным запросом уведомления. Это значение по умолчанию при составлении, а не гарантия, что получатель отправит уведомление.
admin-settings-help-reader-color-mode = Автоматически следует тёмным темам. Выберите исходное, адаптивное или инвертированное отображение внутри изолированного фрейма письма.
admin-settings-key-reader-font-size = Размер шрифта письма
admin-settings-help-reader-font-size = Размер чтения по умолчанию в пикселях, от 12 до 24.
admin-settings-key-reader-line-height = Высота строки письма
admin-settings-help-reader-line-height = Межстрочный интервал при чтении от 1,2 до 2,2.
admin-settings-key-reader-content-width = Ширина чтения письма
admin-settings-help-reader-content-width = Содержимое письма комфортной, широкой или на всю панель ширины.
admin-settings-key-reader-loading = Вид загрузки письма
admin-settings-help-reader-loading = Строки-заполнители, текстовый индикатор или без визуального индикатора. Объявления для специальных возможностей остаются доступными.
admin-settings-key-reader-loading-delay-ms = Задержка загрузки письма
admin-settings-help-reader-loading-delay-ms = Задерживает только индикатор загрузки, от 0 до 1000 миллисекунд. Доставка письма никогда не задерживается.
admin-settings-key-reader-zoom = Масштаб письма
admin-settings-help-reader-zoom = Масштаб текста письма по умолчанию в процентах, от 50 до 200; средство чтения использует ближайший шаг масштаба. Масштабируется только письмо, при печати используется 100%.
admin-settings-key-reader-attachments-collapsed = Панель вложений свёрнута
admin-settings-help-reader-attachments-collapsed = Начинается ли панель вложений средства чтения свёрнутой, показывая только число и общий размер. Последнее открытие или закрытие пользователя сохраняется здесь. По умолчанию выключено: вложения показаны.
admin-settings-key-login-show-domain-hint = Показывать предпочтительный домен при входе
admin-settings-help-login-show-domain-hint = Показывает, какой домен принимает имя пользователя без суффикса электронной почты. По умолчанию скрыто. Меняется только подсказка; вход по имени пользователя продолжает работать.
admin-settings-key-signin-retry-cooldown-secs = Пауза перед повторной попыткой входа
admin-settings-help-signin-retry-cooldown-secs = Минимум секунд между попытками входа по паролю из одной клиентской сети. По умолчанию 3; 0 отключает паузу. Существующие блокировки после неудач продолжают действовать.
admin-settings-key-mail-quote-style = Стиль заголовка ответа и пересылки
admin-settings-key-mail-external-recipient-warning = Предупреждение о внешнем получателе
admin-settings-help-mail-external-recipient-warning = Выключено, предупреждение или требование подтверждения в окне составления. Охватывает «Кому», «Копия» и «Скрытая копия». Это защита окна составления, а не ограничение доставки на сервере.
admin-settings-key-mail-trusted-recipient-domains = Доверенные домены получателей
admin-settings-help-mail-trusted-recipient-domains = До 100 точных доменов получателей. Домен выбранной личности отправителя также считается доверенным. Недопустимые записи никогда не дают доверия. Заблокируйте эту настройку, чтобы пользователи не могли её переопределять.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-quote-style = Использовать классические ответы с цитированием или заголовки писем в стиле Outlook. Это задаёт значение по умолчанию для пользователя; заблокируйте его, чтобы принудительно применить стиль. Существующие черновики не меняются.
admin-settings-key-signin-anti-bot-mode = Защита входа от ботов
admin-settings-help-signin-anti-bot-mode = Собственная проверка proof-of-work: off, mailbox, admin или all. Внешняя служба проверки не используется.
admin-settings-key-signin-anti-bot-difficulty = Объём работы проверки входа
admin-settings-help-signin-anti-bot-difficulty = Наибольший счётчик решения, 100–20000. Более высокие значения требуют больше работы браузера; по умолчанию 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Срок действия проверки входа
admin-settings-help-signin-anti-bot-expiry-secs = Сколько секунд остаётся действительной одноразовая проверка, 60–600; по умолчанию 180. После каждой попытки ввода учётных данных нужна новая проверка.
admin-settings-key-attachments-preview-allowed = Разрешить предпросмотр вложений
admin-settings-help-attachments-preview-allowed = Отключите, чтобы запретить отображение предпросмотра вложений; скачивание оригиналов имеет собственную настройку.
admin-settings-key-attachments-download-allowed = Разрешить выдачу оригиналов вложений
admin-settings-help-attachments-download-allowed = Отключение также запрещает предпросмотр, которому нужны исходные байты. Предпросмотр в браузере не является защитой от копирования.
admin-settings-key-attachments-max-bytes = Наибольший размер вложения в байтах
admin-settings-help-attachments-max-bytes = Ноль означает без ограничений. Применяется к выданным исходным байтам, включая запросы диапазонов.
admin-settings-key-attachments-allowed-extensions = Разрешённые расширения имён файлов
admin-settings-help-attachments-allowed-extensions = Массив JSON расширений в нижнем регистре без точки, например ["pdf","tar.gz"]. Пусто разрешает любое расширение; блокирующие правила имеют приоритет.
admin-settings-key-attachments-blocked-extensions = Заблокированные расширения имён файлов
admin-settings-help-attachments-blocked-extensions = Массив JSON расширений в нижнем регистре. Проверяются по сохранённым именам файлов MIME, а не по имени в адресе скачивания.
admin-settings-key-attachments-warn-extensions = Предупреждать о расширениях имён файлов
admin-settings-help-attachments-warn-extensions = Массив JSON расширений, требующих явного подтверждения перед выдачей оригинала.
admin-settings-key-attachments-allowed-mime-types = Разрешённые типы MIME
admin-settings-help-attachments-allowed-mime-types = Массив JSON типов в нижнем регистре или подстановок верхнего уровня, например ["image/*","application/pdf"]. Пусто разрешает любой тип.
admin-settings-key-attachments-blocked-mime-types = Заблокированные типы MIME
admin-settings-help-attachments-blocked-mime-types = Массив JSON типов в нижнем регистре или подстановок верхнего уровня. Блокирующие правила имеют приоритет.
admin-settings-key-attachments-warn-mime-types = Предупреждать о типах MIME
admin-settings-help-attachments-warn-mime-types = Массив JSON типов в нижнем регистре или подстановок верхнего уровня, требующих подтверждения. Метаданные MIME не являются обнаружением вредоносного ПО.
admin-settings-cat-shortcuts = Сочетания клавиш
admin-settings-cat-time = Часовые пояса
admin-settings-key-time-display-zone = Часовой пояс отображения
admin-settings-help-time-display-zone = Часовой пояс IANA, например Europe/Lisbon, или local, чтобы следовать текущему устройству. Метки времени писем остаются сохранёнными как исходные моменты.
admin-settings-key-time-comparison-zone = Часовой пояс для сравнения
admin-settings-help-time-comparison-zone = Необязательный второй часовой пояс IANA для помощников писем и календаря. Оставьте пустым, чтобы скрыть сравнение; местоположение не определяется по адресу электронной почты.
admin-settings-key-time-show-helpers = Показывать помощников по часовым поясам
admin-settings-help-time-show-helpers = Показывает сравнение местного времени и времени выбранного пояса для меток времени писем и событий календаря. Блокировки администратора могут принудительно задать этот выбор.
admin-settings-key-shortcuts-enabled = Включить сочетания клавиш
admin-settings-help-shortcuts-enabled = Разрешает сочетания клавиш приложения. Блокировки могут отключить их глобально или для домена. Поля ввода и модальные окна сохраняют обычную обработку клавиатуры.
admin-settings-key-shortcuts-compose = Сочетание для нового письма
admin-settings-help-shortcuts-compose = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-archive = Сочетание для архивации
admin-settings-help-shortcuts-archive = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-reply = Сочетание для ответа
admin-settings-help-shortcuts-reply = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-reply-all = Сочетание для ответа всем
admin-settings-help-shortcuts-reply-all = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-forward = Сочетание для пересылки
admin-settings-help-shortcuts-forward = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-next = Сочетание для следующего письма
admin-settings-help-shortcuts-next = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-previous = Сочетание для предыдущего письма
admin-settings-help-shortcuts-previous = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-search = Сочетание для перехода к поиску
admin-settings-help-shortcuts-search = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-refresh = Сочетание для обновления писем
admin-settings-help-shortcuts-refresh = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-settings = Сочетание для открытия настроек
admin-settings-help-shortcuts-settings = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-trash = Сочетание для перемещения в корзину
admin-settings-help-shortcuts-trash = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-mark-read = Сочетание для отметки прочитанным
admin-settings-help-shortcuts-mark-read = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-mark-unread = Сочетание для отметки непрочитанным
admin-settings-help-shortcuts-mark-unread = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-spam = Сочетание для отметки как спам
admin-settings-help-shortcuts-spam = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-zoom-in = Сочетание для увеличения письма
admin-settings-help-shortcuts-zoom-in = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-zoom-out = Сочетание для уменьшения письма
admin-settings-help-shortcuts-zoom-out = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-zoom-reset = Сочетание для сброса масштаба письма
admin-settings-help-shortcuts-zoom-reset = Используйте inherit для выбранного набора, disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-shortcuts-open-in-tab = Сочетание для открытия письма в отдельной вкладке
admin-settings-help-shortcuts-open-in-tab = Используйте inherit для выбранного набора (Shift+Enter), disabled, чтобы убрать это действие, или сочетание, например Ctrl+Shift+K. Конфликтующие или недопустимые сочетания не работают.
admin-settings-key-notifications-push-allowed = Фоновые push-уведомления разрешены
admin-settings-help-notifications-push-allowed = Разрешает включённым уведомлениям этой учётной записи регистрироваться и получать фоновые push-сообщения. Разрешение браузера по-прежнему требуется.
admin-settings-key-offline-persistent-storage-allowed = Запросы постоянного хранилища браузера разрешены
admin-settings-help-offline-persistent-storage-allowed = Разрешает явный запрос на защиту локальных кэшированных данных от автоматического вытеснения браузером. Браузер может отказать; разрешение автоматически не выдаётся.
admin-settings-key-security-signin-mailbox-failure-limit = Лимит неудачных учётных данных ящика
admin-settings-help-security-signin-mailbox-failure-limit = Отклонённые учётные данные с одного исходного IP-адреса в окне подсчёта вызывают временную блокировку входа в ящик. Сбои сети и конфигурации не учитываются.
admin-settings-key-security-signin-admin-failure-limit = Лимит неудачных учётных данных администратора
admin-settings-help-security-signin-admin-failure-limit = Отклонённые учётные данные администратора с одного исходного IP-адреса вызывают отдельную временную блокировку входа администратора. Короткий сеанс администратора и требования свежего подтверждения не меняются.
admin-settings-key-security-signin-failure-window-secs = Окно подсчёта неудачных учётных данных
admin-settings-help-security-signin-failure-window-secs = Скользящее окно в секундах для отказов по учётным данным и MFA. Успешно завершённый вход сбрасывает счётчик для этого источника и поверхности.
admin-settings-key-security-signin-lock-secs = Длительность временной блокировки входа
admin-settings-help-security-signin-lock-secs = Длительность блокировки в секундах. Повторные попытки во время блокировки не продлевают её исходный срок. Ручные глобальные блокировки IP — отдельная функция.
admin-settings-key-security-signin-geo-scope = Область политики стран для входа
admin-settings-help-security-signin-geo-scope = По умолчанию выключено. Применять только к новым входам в ящик, администратора или ко всем. Существующие сеансы не отзываются. Перед включением проверьте диагностику текущего источника; недоступная локальная база GeoIP приводит к отказу во входе.
admin-settings-key-security-signin-geo-mode = Список разрешённых или запрещённых стран
admin-settings-help-security-signin-geo-mode = Allow допускает только перечисленные страны; deny отказывает перечисленным странам. У неизвестных адресов своя явная политика.
admin-settings-key-security-signin-geo-countries = Коды стран для входа
admin-settings-help-security-signin-geo-countries = Массив JSON двухбуквенных кодов стран в верхнем регистре, например ["PT", "GB"]. Использует только локальную базу стран (Безопасность > База стран или MW_GEOIP_DB); адреса никогда не отправляются службе геолокации.
admin-settings-key-security-signin-geo-unknown = Политика для неопределённых исходных адресов
admin-settings-help-security-signin-geo-unknown = Разрешать или запрещать адреса, отсутствующие в читаемой локальной базе, включая частные адреса. Отсутствующая или нечитаемая база всегда отклоняет входы при действующей политике, даже если неизвестные адреса разрешены.
admin-settings-key-contacts-avatar-mode = Фото контактов и сгенерированные аватары
admin-settings-help-contacts-avatar-mode = Генерировать инициалы локально, скрывать аватары или использовать встроенные фото из собственных адресных книг каждой учётной записи. Удалённые адреса фото и внешняя служба аватаров не используются. Политику пользователя и домена может зафиксировать администратор.
admin-settings-cat-contacts = Фото контактов
admin-settings-key-reader-action-style = Лента действий с письмом
admin-settings-help-reader-action-style = По умолчанию компактные значки. Выберите подписи, чтобы у значков был видимый текст; оба варианта сохраняют названия действий и тематические подсказки. Поддерживает глобальные блокировки, блокировки домена и пользователя.
admin-settings-key-nextcloud-sync-interval-secs = Интервал автоматического обновления Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Секунды между фоновыми обновлениями коллекций для подключённых учётных записей нативного движка. Ноль — только вручную; положительные значения имеют минимум шестьдесят секунд. После сбоев интервал увеличивается. Фоновый вход не выполняется.
admin-settings-key-mail-folder-role-inbox = Сопоставление папки «Входящие»
admin-settings-help-mail-folder-role-inbox = Null использует автоматическое обнаружение. Используйте объект с точным путём для общих значений по умолчанию или объект с идентификатором принадлежащего ящика для одной учётной записи. Отсутствующие или конфликтующие цели отключают эту роль; блокировки администратора действуют.
admin-settings-key-mail-folder-role-drafts = Сопоставление папки «Черновики»
admin-settings-help-mail-folder-role-drafts = Null использует автоматическое обнаружение. Используйте объект с точным путём для общих значений по умолчанию или объект с идентификатором принадлежащего ящика для одной учётной записи. Отсутствующие или конфликтующие цели отключают эту роль; блокировки администратора действуют.
admin-settings-key-mail-folder-role-sent = Сопоставление папки «Отправленные»
admin-settings-help-mail-folder-role-sent = Null использует автоматическое обнаружение. Используйте объект с точным путём для общих значений по умолчанию или объект с идентификатором принадлежащего ящика для одной учётной записи. Отсутствующие или конфликтующие цели отключают эту роль; блокировки администратора действуют.
admin-settings-key-mail-folder-role-trash = Сопоставление папки «Корзина»
admin-settings-help-mail-folder-role-trash = Null использует автоматическое обнаружение. Используйте объект с точным путём для общих значений по умолчанию или объект с идентификатором принадлежащего ящика для одной учётной записи. Отсутствующие или конфликтующие цели отключают эту роль; блокировки администратора действуют.
admin-settings-key-mail-folder-role-junk = Сопоставление папки «Спам»
admin-settings-help-mail-folder-role-junk = Null использует автоматическое обнаружение. Используйте объект с точным путём для общих значений по умолчанию или объект с идентификатором принадлежащего ящика для одной учётной записи. Отсутствующие или конфликтующие цели отключают эту роль; блокировки администратора действуют.
admin-settings-key-mail-folder-role-archive = Сопоставление папки «Архив»
admin-settings-help-mail-folder-role-archive = Null использует автоматическое обнаружение. Используйте объект с точным путём для общих значений по умолчанию или объект с идентификатором принадлежащего ящика для одной учётной записи. Отсутствующие или конфликтующие цели отключают эту роль; блокировки администратора действуют.
admin-settings-key-mail-request-delivery-receipt = Запрашивать уведомления о доставке по умолчанию
admin-settings-help-mail-request-delivery-receipt = Выбирает уведомления о доставке для новых писем, если учётная запись поддерживает SMTP DSN. Блокировка управляет этим сохранённым значением по умолчанию; receipts.dsnAllowed отдельно определяет, разрешены ли запросы. Доставка не доказывает прочтение.
admin-settings-key-security-signin-geo-enforce-sessions = Применять политику стран к существующим сеансам
admin-settings-help-security-signin-geo-enforce-sessions = По умолчанию выключено. Применяет настроенную область стран входа к аутентифицированным запросам, нативному доступу bearer и подписанному содержимому средства чтения. Заблокированные клиенты по-прежнему могут выйти. Уже открытые потоки проверяются при повторном подключении; существующие соединения это не разрывает.
admin-settings-key-security-protection-rate-mode = Сводные бюджеты запросов
admin-settings-help-security-protection-rate-mode = По умолчанию выключено. Observe фиксирует исчерпание бюджета без отказа; enforce возвращает временный ответ 429. Существующие лимиты параллелизма протоколов остаются независимыми.
admin-settings-key-security-protection-adaptive-mode = Адаптивная защита запросов
admin-settings-help-security-protection-adaptive-mode = По умолчанию выключено. Observe фиксирует объяснимые типизированные аномалии и предлагаемые действия; enforce применяет ограниченные временные замедления или блокировки. Это детерминированная политика счётчиков, а не ИИ-классификатор.
admin-settings-key-security-protection-client-requests = Запросов клиента за окно
admin-settings-help-security-protection-client-requests = Запросов за окно с одного адреса, не вошедшего в систему, и каждый запрос входа. Запрос с действительным сеансом ящика или администратора использует собственный, больший лимит. Ноль отключает этот бюджет.
admin-settings-key-security-protection-account-requests = Запросов учётной записи за окно
admin-settings-help-security-protection-account-requests = Запросов за окно для одного вошедшего ящика по всем его сеансам, устройствам и адресам. Выше анонимного лимита, чтобы у вошедших был больший запас. Ноль отключает этот бюджет.
admin-settings-key-rate-limit-diagnostics-burst = Одновременные запуски диагностики
admin-settings-help-rate-limit-diagnostics-burst = Запуски диагностики и проверки бэкенда, которые сеанс администратора может начать подряд. Запуск уже выполняющейся проверки возвращает эту проверку, а не считается повторно.
admin-settings-key-rate-limit-diagnostics-per-minute = Запусков диагностики в минуту
admin-settings-help-rate-limit-diagnostics-per-minute = Запусков диагностики и проверок бэкенда в минуту, которые восстанавливает сеанс администратора.
admin-settings-key-security-protection-admin-requests = Запросов администратора за окно

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-protection-admin-requests = Запросов за окно для одного вошедшего администратора. Выше анонимного лимита. Ноль отключает этот бюджет.
admin-settings-key-security-protection-rate-window-secs = Окно бюджета запросов (секунды)
admin-settings-help-security-protection-rate-window-secs = Фиксированный интервал бюджета запросов. Сброс нагрузки при перегрузке и обычные ответы «не найдено» не считаются адаптивным свидетельством.
admin-settings-key-security-protection-adaptive-clients = Наблюдать клиентов в адаптивной защите
admin-settings-help-security-protection-adaptive-clients = Включает это измерение субъекта, когда адаптивный режим активен. Учётные записи и администраторы учитываются только после проверки их личности.
admin-settings-key-security-protection-adaptive-accounts = Наблюдать учётные записи в адаптивной защите
admin-settings-help-security-protection-adaptive-accounts = Включает это измерение субъекта, когда адаптивный режим активен. Учётные записи и администраторы учитываются только после проверки их личности.
admin-settings-key-security-protection-adaptive-admins = Наблюдать администраторов в адаптивной защите
admin-settings-help-security-protection-adaptive-admins = Включает это измерение субъекта, когда адаптивный режим активен. Учётные записи и администраторы учитываются только после проверки их личности.
admin-settings-key-security-protection-short-window-secs = Короткое окно аномалий (секунды)
admin-settings-help-security-protection-short-window-secs = Скользящее окно наблюдения. Длинные окна поддерживают от часов до тридцати дней; округление корзин может сохранить не более одной дополнительной шестьдесят четвёртой части окна.
admin-settings-key-security-protection-short-minimum-samples = Минимум выборок короткого отношения
admin-settings-help-security-protection-short-minimum-samples = Минимум наблюдаемых нормальных и аномальных запросов, прежде чем сработает триггер отношения. Абсолютный порог аномалий не зависит от этого минимума.
admin-settings-key-security-protection-short-absolute-anomalies = Короткий абсолютный бюджет аномалий
admin-settings-help-security-protection-short-absolute-anomalies = Срабатывает, когда накапливается столько выбранных аномалий, даже если обычные запросы разбавляют отношение. Ноль отключает этот триггер.
admin-settings-key-security-protection-short-ratio-percent = Короткое отношение аномалий (процент)
admin-settings-help-security-protection-short-ratio-percent = Срабатывает, когда выбранные аномалии достигают этого процента после минимального числа выборок. Достаточно абсолютного триггера или триггера отношения. Ноль отключает этот триггер.
admin-settings-key-security-protection-long-window-secs = Длинное окно аномалий (секунды)
admin-settings-help-security-protection-long-window-secs = Скользящее окно наблюдения. Длинные окна поддерживают от часов до тридцати дней; округление корзин может сохранить не более одной дополнительной шестьдесят четвёртой части окна.
admin-settings-key-security-protection-long-minimum-samples = Минимум выборок длинного отношения
admin-settings-help-security-protection-long-minimum-samples = Минимум наблюдаемых нормальных и аномальных запросов, прежде чем сработает триггер отношения. Абсолютный порог аномалий не зависит от этого минимума.
admin-settings-key-security-protection-long-absolute-anomalies = Длинный абсолютный бюджет аномалий
admin-settings-help-security-protection-long-absolute-anomalies = Срабатывает, когда накапливается столько выбранных аномалий, даже если обычные запросы разбавляют отношение. Ноль отключает этот триггер.
admin-settings-key-security-protection-long-ratio-percent = Длинное отношение аномалий (процент)
admin-settings-help-security-protection-long-ratio-percent = Срабатывает, когда выбранные аномалии достигают этого процента после минимального числа выборок. Достаточно абсолютного триггера или триггера отношения. Ноль отключает этот триггер.
admin-settings-key-security-protection-normal-cap-per-bucket = Предел нормальной базы на корзину
admin-settings-help-security-protection-normal-cap-per-bucket = Ограничивает нормальную базовую линию, изучаемую в каждой корзине скользящего окна. Независимые абсолютные бюджеты аномалий не дают неограниченному заполнению допустимыми запросами скрыть злоупотребление.
admin-settings-key-security-protection-signals = Сигналы адаптивных свидетельств
admin-settings-help-security-protection-signals = Выберите авторитетные наблюдения: некорректные, слишком большие, с отклонёнными учётными данными, с отказом в доступе, с превышением срока тела или (по желанию) промахи маршрутов администратора. Зондирование маршрутов администратора исключает ответы о ненайденной сущности и статические ресурсы; общие коды состояния, отказы из-за перегрузки и строки user-agent не доказывают злоупотребление.
admin-settings-key-security-protection-throttle-secs = Начальное временное замедление (секунды)
admin-settings-help-security-protection-throttle-secs = Первое пересечение порога запускает фиксированный срок замедления. Запросы в течение этого срока его не продлевают.
admin-settings-key-security-protection-block-secs = Повышенная временная блокировка (секунды)
admin-settings-help-security-protection-block-secs = Более позднее пересечение порога до восстановления может перейти к этой ограниченной блокировке. Постоянная блокировка учётной записи не создаётся.
admin-settings-key-security-protection-decay-secs = Интервал спокойного восстановления (секунды)
admin-settings-help-security-protection-decay-secs = После этого интервала без аномалий и истечения текущего действия эскалация возвращается к начальной стадии. Скользящие свидетельства истекают согласно настроенным окнам.
admin-settings-key-security-protection-exempt-clients = Исключённые адреса и сети клиентов
admin-settings-help-security-protection-exempt-clients = Точные IP-адреса или сети CIDR исключают только измерение клиента. Проверенные лимиты учётных записей продолжают действовать. Используйте осознанно для общих сетей NAT; пересылаемые адреса считаются доверенными только через настроенные прокси.
admin-settings-key-security-protection-exempt-accounts = Исключённые проверенные идентификаторы учётных записей
admin-settings-help-security-protection-exempt-accounts = Точные проверенные сервером идентификаторы учётных записей ящиков исключают только их измерение учётной записи, но не измерение исходного клиента. Адреса электронной почты и параметры учётной записи, переданные вызывающей стороной, не являются доказательством личности.
admin-settings-key-security-protection-exempt-admins = Исключённые идентификаторы администраторов
admin-settings-help-security-protection-exempt-admins = Точные аутентифицированные идентификаторы администраторов исключают только их измерение администратора. Существующие блокировки после неудачных входов и строгие сроки сеанса администратора не меняются.
admin-settings-key-security-protection-storage-failure = Политика сбоя хранилища применения
admin-settings-help-security-protection-storage-failure = Closed отклоняет защищаемую работу, когда включённый бюджет применения нельзя проверить. Open временно разрешает её с ограниченным эксплуатационным предупреждением. Сбои в режиме только наблюдения никогда не вызывают действий применения.
admin-settings-key-security-admin-probe-enabled = Защита от зондирования администратора
admin-settings-help-security-admin-probe-enabled = Считает запросы, ищущие консоль администратора, по сети клиента (адрес IPv4, IPv6 /64), замедляет источник, который не останавливается, затем блокирует его. Опечатки и устаревшие закладки укладываются в льготный лимит. Меняется в разделе «Безопасность > Адаптивная защита».
admin-settings-key-security-admin-probe-grace-allowance = Льготный лимит (промахов за окно)
admin-settings-help-security-admin-probe-grace-allowance = Промахи, игнорируемые за окно. Счёт непрерывно уменьшается на столько баллов за окно.
admin-settings-key-security-admin-probe-window-secs = Льготное окно (секунды)
admin-settings-help-security-admin-probe-window-secs = Окно, к которому относится льготный лимит.
admin-settings-key-security-admin-probe-block-after = Промахи сверх льготы до блокировки
admin-settings-help-security-admin-probe-block-after = Промахи сверх льготного лимита, с которых начинается блокировка. Промежуточные промахи обрабатываются после задержки tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Задержка tarpit (мс)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Задержка, добавляемая к каждому учтённому промаху сверх льготного лимита. 0 отключает tarpit. Другие запросы никогда не задерживаются.
admin-settings-key-security-admin-probe-block-secs = Первая блокировка (секунды)
admin-settings-help-security-admin-probe-block-secs = Длительность первой блокировки. Заблокированный источник получает один и тот же ответ для любого пути.
admin-settings-key-security-admin-probe-max-block-secs = Самая длинная блокировка (секунды)
admin-settings-help-security-admin-probe-max-block-secs = Повторная блокировка удваивает предыдущую длительность, но не больше этого предела.
admin-settings-key-security-admin-probe-repeat-window-secs = Окно повтора (секунды)
admin-settings-help-security-admin-probe-repeat-window-secs = Блокировка, начавшаяся в течение этого времени после окончания предыдущей, считается повторной.
admin-settings-key-security-admin-probe-block-status = Статус ответа блокировки
admin-settings-help-security-admin-probe-block-status = Статус HTTP, отправляемый заблокированному источнику, с Retry-After: 429 или 403.
admin-settings-key-security-admin-probe-signals = Учитываемые промахи
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (404 на глубине пути консоли), admin-signin-failed и admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Пути зондирования
admin-settings-help-security-admin-probe-probe-paths = Пути, которые запрашивают сканеры, например /wp-admin или /.env. Путь, содержащий один из них как целые сегменты, считается промахом. Собственный путь консоли — никогда.
admin-settings-key-security-admin-probe-allowlist = Сети, которые никогда не блокируются
admin-settings-help-security-admin-probe-allowlist = IP-адреса или сети CIDR (офис, VPN), которые никогда не учитываются и не блокируются.
admin-settings-key-security-admin-probe-audit-ip = Исходный адрес в журнале аудита
admin-settings-help-security-admin-probe-audit-ip = masked показывает IPv4 /24 и IPv6 /48; full показывает адрес. Строка fail2ban всегда содержит полный адрес.
admin-settings-key-security-admin-probe-fail2ban-log = Строка журнала fail2ban при блокировке
admin-settings-help-security-admin-probe-fail2ban-log = Записывать одну строку mailwoman[admin-probe] с rhost=<адрес>, когда источник блокируется.
admin-settings-key-appearance-custom-css-enabled = Разрешить личный CSS оформления
admin-settings-help-appearance-custom-css-enabled = Разрешает ограниченные визуальные стили в рабочем пространстве владельца. По умолчанию выключено; принудительные блокировки оформления ящика приостанавливают эти стили.
admin-settings-key-appearance-custom-js-enabled = Разрешить личные скрипты оформления
admin-settings-help-appearance-custom-js-enabled = Разрешает изолированные скрипты, которые могут только запрашивать изменения оформления. По умолчанию выключено; скрипты не имеют доступа к почте, данным учётной записи или DOM приложения.
admin-settings-key-mail-modifier-selection = Выбор писем клавишами-модификаторами
admin-settings-help-mail-modifier-selection = Разрешает выбор диапазона клавишами Ctrl, Command и Shift. Заблокируйте это значение, чтобы применить его принудительно; видимые элементы выбора остаются доступными.
admin-settings-key-mail-drag-to-folder = Перетаскивание писем в папки
admin-settings-help-mail-drag-to-folder = Разрешает перетаскивание писем, привязанных к учётной записи, в реальные папки. Это не выдаёт разрешение на запись в ящик; сервер по-прежнему авторизует каждое перемещение.
admin-settings-key-admin-allow-last-superadmin-loss = Разрешить потерю последнего суперадминистратора
admin-settings-help-admin-allow-last-superadmin-loss = По умолчанию выключено. Включение позволяет последующим изменениям убрать последний доступ суперадминистратора. Полный суперадминистратор должен подтвердить последствия свежей аутентификацией.
admin-settings-last-superadmin-title = Разрешить потерю всего доступа суперадминистратора?
admin-settings-last-superadmin-body = Последующие изменения учётных записей могут оставить некому администрировать это развёртывание. Восстановление потребует локального администрирования развёртывания. Введите ALLOW LAST SUPERADMIN LOSS, чтобы включить это исключение.
admin-settings-last-superadmin-confirm = Разрешить потерю доступа
admin-settings-key-mail-folder-counts = Счётчики папок
admin-settings-help-mail-folder-counts = Выберите скрытые, непрочитанные, непрочитанные только при наличии непрочитанной почты, общие или оба счётчика рядом с папками. Задавайте значения по умолчанию глобально, для домена или пользователя и блокируйте выбор при необходимости. Применяется к каждому связанному ящику учётной записи.
admin-settings-key-mail-folder-count-scope = Папки со счётчиками
admin-settings-help-mail-folder-count-scope = Все папки или только основные («Входящие», «Черновики», «Отправленные», «Архив», «Спам», «Корзина», «Исходящие» и другие папки особого назначения). При выборе только основных у пользовательских папок и их подпапок счётчик не показывается. Применяется к каждому связанному ящику учётной записи.
admin-settings-key-mail-compact-folder-badges = Компактные значки остальных папок
admin-settings-help-mail-compact-folder-badges = Показывает меньшие значки вне «Входящих». Пользователи могут выбрать свой размер, если эта настройка не заблокирована.
admin-settings-key-mail-focused-inbox = Важные входящие
admin-settings-help-mail-focused-inbox = Делит «Входящие» на вкладки «Важные» и «Прочие»; рассылки и автоматические отправители попадают в «Прочие». По умолчанию выключено. Пользователи могут включить или выключить это в разделе «Настройки > Папки и боковая панель», если эта настройка не заблокирована.
admin-settings-key-mail-account-unread-badge = Число непрочитанных рядом с именем ящика
admin-settings-help-mail-account-unread-badge = Показывает число непрочитанных во «Входящих» рядом с адресом или псевдонимом вошедшего пользователя на боковой панели. По умолчанию выключено. Счётчики других ящиков в меню ящиков показываются всегда. Применяется к каждому связанному ящику учётной записи.
admin-settings-key-privacy-remote-image-downloads = Ручная загрузка удалённых изображений
admin-settings-key-privacy-remote-images-toast = Уведомление о загрузке изображений
admin-settings-help-privacy-remote-images-toast = Показывает одно уведомление в средстве чтения, пока удалённые изображения письма загружаются через прокси приватности, и сообщает, когда они загружены или сколько не удалось загрузить. По умолчанию включено. Меняет только то, что показывает средство чтения; какие изображения загружаются, по-прежнему определяет политика удалённого содержимого.
admin-settings-help-privacy-remote-image-downloads = Разрешает явную загрузку изображения, на которое ссылается принадлежащее пользователю письмо, через защищённый прокси приватности. По умолчанию выключено; внешний URL никогда не открывается напрямую.
admin-settings-key-mail-resume-tabs = Возобновлять вкладки рабочей области
admin-settings-help-mail-resume-tabs = Восстанавливает после перезагрузки ссылки на вкладки, принадлежащие учётной записи. Отключение очищает локальную запись рабочей области, не удаляя черновики.
admin-settings-key-metrics-default-range-seconds = Диапазон истории запросов по умолчанию
admin-settings-help-metrics-default-range-seconds = Секунды, показываемые, когда запрос истории не задаёт диапазон. Ограничен хранением и максимальным диапазоном.
admin-settings-key-metrics-max-range-seconds = Максимальный диапазон истории запросов
admin-settings-help-metrics-max-range-seconds = Наибольшее число секунд на запрос истории. Хранение остаётся дополнительным ограничением.
admin-settings-key-contacts-favorites-first = Избранные первыми в подсказках
admin-settings-help-contacts-favorites-first = Выводит избранные контакты перед остальными совпадениями в подсказках получателей. Меняется только порядок; ни одна подсказка не скрывается. Переопределения пользователя и домена можно заблокировать.
admin-settings-key-contacts-sender-icons = Встроенные значки отправителей

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-contacts-sender-icons = Показывает локально встроенные значки для узнаваемых доменов отправителей, а для остальных — сгенерированные инициалы. Фото контактов имеют приоритет. Значки не проверяют личность отправителя. Внешняя служба логотипов не используется; переопределения пользователя и домена можно заблокировать.
admin-settings-key-mail-brand-avatars = Логотипы брендов рядом с проверенными отправителями
admin-settings-help-mail-brand-avatars = Показывает локально встроенный логотип бренда рядом с отправителем в средстве чтения, только если письмо прошло DKIM и согласованную проверку DMARC для собственного домена этого бренда. Письма, не прошедшие эти проверки или не имеющие их, сохраняют обычный аватар. Логотип не загружается ни у отправителя, ни из службы логотипов.
admin-settings-key-mail-draft-save-mode = Режим сохранения черновиков учётной записи
admin-settings-help-mail-draft-save-mode = Сохранять после паузы в наборе, периодически или только по запросу. Для сохранения нужна поддерживаемая папка «Черновики» на вышестоящем сервере; неопределённые записи никогда не повторяются автоматически.
admin-settings-key-mail-draft-debounce-ms = Пауза в наборе перед сохранением черновика (миллисекунды)
admin-settings-help-mail-draft-debounce-ms = Ждать от 500 до 10000 миллисекунд после правки перед сохранением в режиме реального времени.
admin-settings-key-mail-draft-interval-secs = Интервал сохранения черновика (секунды)
admin-settings-help-mail-draft-interval-secs = Сохранять изменённые черновики каждые 5–3600 секунд в периодическом режиме. Неизменённые черновики повторно не записываются.
admin-settings-key-login-remember-signins = Запоминать адреса успешных входов
admin-settings-help-login-remember-signins = Хранить на этом устройстве до десяти успешно использованных имён пользователей или адресов электронной почты для выпадающего списка входа. Пароли и сеансы никогда не сохраняются. Отключение удаляет сохранённые адреса этого сервера, когда устройство в следующий раз загрузит страницу входа или завершит вход; автозаполнением браузера это не управляет.
admin-settings-key-login-show-remembered-hint = Показывать пояснение о сохранённых адресах
admin-settings-help-login-show-remembered-hint = Показывает пояснительный текст в выпадающем списке прежних входов. По умолчанию скрыто. Действует, только если включено запоминание успешных входов.
admin-settings-key-mail-show-from = Показывать поле «От» в окне составления
admin-settings-help-mail-show-from = Показывать «От» по умолчанию в новых окнах составления. Пользователи могут открыть его для одного письма; отправлять могут только авторизованные адреса.
admin-settings-key-accounts-linking-enabled = Разрешить связанные учётные записи ящиков
admin-settings-help-accounts-linking-enabled = По умолчанию выключено. Глобальное отключение блокирует весь связанный доступ; обе учётные записи также должны соответствовать политике своего домена и учётной записи. Для привязки требуются реальная аутентификация дочернего ящика и свежее подтверждение основной учётной записи.
admin-settings-key-accounts-use-linked = Использовать связанные ящики
admin-settings-help-accounts-use-linked = Отказ пользователя от связанного доступа. Не может переопределить административное отключение. Данные ящика сохраняются при отключении доступа.
admin-settings-key-accounts-max-linked = Наибольшее число связанных ящиков
admin-settings-help-accounts-max-linked = Наибольшее число проверенных дочерних ящиков на основную личность, от 1 до 16. По умолчанию 5.
admin-settings-key-accounts-store-linked-credentials = Сохранять доступ к связанным ящикам
admin-settings-help-accounts-store-linked-credentials = Хранить пароль каждого связанного ящика в запечатанном виде на сервере, чтобы доступ обновлялся сам, между сеансами и перезапусками. Никогда не хранится для учётных записей с нулевым доступом и ящиков с двухэтапной проверкой. Отключение стирает сохранённые пароли; связи затем обновляются повторным входом. По умолчанию включено.
admin-settings-key-privacy-strip-mailer-headers = Удалять заголовки почтовой программы из исходящих писем
admin-settings-help-privacy-strip-mailer-headers = Удаляет заголовки верхнего уровня X-Mailer, User-Agent и X-MimeOLE из писем нативного SMTP и EWS. Вложенные письма и данные отчётов остаются нетронутыми. Отправки через вышестоящий прокси JMAP не затрагиваются.
admin-settings-key-images-trusted-hosts = Доверенные хосты изображений
admin-settings-help-images-trusted-hosts = До 100 хостов изображений и их поддоменов, удалённые изображения которых загружаются без разрешения для письма, когда удалённое содержимое настроено на запрос. Никогда не переопределяет режим блокировки; заблокированный хост остаётся заблокированным.
admin-settings-key-images-blocked-hosts = Заблокированные хосты изображений
admin-settings-help-images-blocked-hosts = До 100 хостов изображений и их поддоменов, удалённые изображения которых никогда не загружаются ни в одном режиме. Имеет приоритет над доверенными хостами и любым разрешением.
admin-settings-key-links-trusted-domains = Доверенные домены ссылок
admin-settings-help-links-trusted-domains = До 100 точных доменов, ссылки которых открываются из «Проверки ссылок» без шага назначения. Без подстановочных знаков, схем, портов и IP-адресов. Ссылки по обычному HTTP проверяются всегда. Заблокируйте эту настройку, чтобы пользователи не меняли список.
admin-settings-key-links-trusted-subdomains = Доверять поддоменам доверенных доменов ссылок
admin-settings-help-links-trusted-subdomains = Выключено: доверяется только точный домен. Включено: доверяются и имена, оканчивающиеся на .<домен>. Сравнение идёт по целым меткам, поэтому example.com никогда не доверяет evilexample.com.
admin-settings-key-cap-compose-rich-text = Составление форматированного текста
admin-settings-help-cap-compose-rich-text = Могут ли пользователи писать форматированные (HTML) письма. Выключено отправляет только простой текст.
admin-settings-key-cap-compose-attachments = Вложения в новых письмах
admin-settings-help-cap-compose-attachments = Могут ли пользователи прикреплять файлы к составляемым письмам. Изображения в тексте не учитываются.
admin-settings-key-cap-compose-share-links = Ссылки общего доступа для больших файлов
admin-settings-help-cap-compose-share-links = Могут ли пользователи отправить большой файл ссылкой общего доступа Nextcloud вместо вложения.
admin-settings-key-cap-compose-scheduled-send = Отложенная отправка
admin-settings-help-cap-compose-scheduled-send = Могут ли пользователи запланировать отправку письма на потом.
admin-settings-key-cap-compose-read-receipts = Запрос уведомлений о прочтении
admin-settings-help-cap-compose-read-receipts = Могут ли пользователи просить получателей об уведомлении о прочтении.
admin-settings-key-cap-signatures = Подписи
admin-settings-help-cap-signatures = Могут ли пользователи создавать и изменять собственные подписи.
admin-settings-key-cap-identities = Личности отправителя
admin-settings-help-cap-identities = Могут ли пользователи добавлять и изменять личности, от имени которых отправляют.
admin-settings-key-cap-keys = Ключи OpenPGP и S/MIME
admin-settings-help-cap-keys = Есть ли у пользователей модуль «Ключи» и могут ли они подписывать и шифровать письма.
admin-settings-key-cap-links-trusted = Правка доверенных доменов ссылок
admin-settings-help-cap-links-trusted = Могут ли пользователи менять собственный список доверенных доменов ссылок. Список администратора продолжает действовать.
admin-settings-key-cap-zoom = Масштаб в средстве чтения
admin-settings-help-cap-zoom = Могут ли пользователи масштабировать средство чтения писем.
admin-settings-key-cap-mail-focused-inbox = Важные входящие
admin-settings-help-cap-mail-focused-inbox = Могут ли пользователи включить вкладки «Важные» и «Прочие» во входящих.
admin-settings-key-cap-views-events = Представление «События»
admin-settings-help-cap-views-events = Есть ли у пользователей представление «События» календарной почты.
admin-settings-key-cap-views-attachments = Представление «Вложения»
admin-settings-help-cap-views-attachments = Есть ли у пользователей представление «Вложения». Письма остаются доступны через поиск.
admin-settings-key-cap-search-operators = Операторы поиска
admin-settings-help-cap-search-operators = Могут ли пользователи искать с операторами, такими как from:, subject: или has:attachment.
admin-settings-key-cap-saved-searches = Сохранённые поиски
admin-settings-help-cap-saved-searches = Могут ли пользователи сохранять поиски.
admin-settings-key-cap-vacation = Автоответ на время отпуска
admin-settings-help-cap-vacation = Могут ли пользователи задавать автоматический ответ на время отпуска.
admin-settings-key-cap-labels = Метки
admin-settings-help-cap-labels = Могут ли пользователи ставить метки на письма.
admin-settings-key-cap-snooze = Отложить
admin-settings-help-cap-snooze = Могут ли пользователи откладывать письма. Отмена откладывания остаётся возможной.
admin-settings-key-cap-follow-up = Напоминания о последующих действиях
admin-settings-help-cap-follow-up = Могут ли пользователи ставить напоминания о последующих действиях на письма. Снятие напоминания остаётся возможным.
admin-settings-key-cap-masked = Скрытые адреса
admin-settings-help-cap-masked = Могут ли пользователи создавать скрытые адреса и управлять ими.
admin-settings-key-cap-directory = Поиск в каталоге
admin-settings-help-cap-directory = Могут ли пользователи искать в каталоге организации.
admin-settings-key-cap-mcp = Доступ MCP
admin-settings-help-cap-mcp = Могут ли ключи и токены пользователя использовать конечную точку MCP.
admin-settings-key-cap-appearance-custom = Настройка оформления
admin-settings-help-cap-appearance-custom = Могут ли пользователи менять тему, акцент, собственные цвета и цвета значков, плотность и шрифт. Выключено сохраняет оформление администратора.
admin-settings-key-cap-appearance-custom-code = Собственный CSS и скрипты
admin-settings-help-cap-appearance-custom-code = Могут ли пользователи сохранять собственный CSS и скрипты оформления. Переключатели развёртывания в разделе «Оформление» по-прежнему действуют выше этой настройки.
admin-settings-key-cap-avatars = Аватары ящиков
admin-settings-help-cap-avatars = Могут ли пользователи менять аватары ящиков.
admin-settings-key-cap-shortcuts = Собственные сочетания клавиш
admin-settings-help-cap-shortcuts = Могут ли пользователи менять сочетания клавиш. Сочетания по умолчанию продолжают работать.
admin-settings-key-cap-analytics = Аналитика продукта
admin-settings-help-cap-analytics = Запрашивается ли у пользователей согласие на аналитику и могут ли они отправлять события использования.
admin-settings-key-cap-apps-native = Приложения для компьютера и телефона
admin-settings-help-cap-apps-native = Могут ли пользователи пользоваться приложениями для компьютера и телефона.
admin-settings-key-access-protocols = Протоколы доступа к почте
admin-settings-help-access-protocols = Какими протоколами почтового сервера эти пользователи могут входить: imap, pop3, ews. Входы через сервер JMAP определяются этим сервером.
admin-settings-key-limit-linked-mailboxes = Связанных ящиков на пользователя
admin-settings-help-limit-linked-mailboxes = Сколько ящиков пользователь может связать, не выше максимума сервера. 0 означает, что действует только максимум сервера.
admin-settings-key-login-show-account-help-notice = Показывать подсказку по учётной записи при входе
admin-settings-help-login-show-account-help-notice = Показывает краткое пояснение после неудачного входа. По умолчанию скрыто; подсказка не раскрывает, существует ли учётная запись или отключена ли она.
admin-settings-key-appearance-leetspeak = Стиль текста leetspeak
admin-settings-help-appearance-leetspeak = Отображает подписи интерфейса и текст писем только для чтения в стиле leetspeak. Исходное письмо и редактируемый текст не меняются. Пользователи могут выбрать обычный текст, если эта настройка не заблокирована.
admin-settings-key-mail-cache-enabled = Включить кэш сеанса
admin-settings-help-mail-cache-enabled = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-mailboxes = Кэшировать списки писем папок
admin-settings-help-mail-cache-mailboxes = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-messages = Кэшировать открытые письма
admin-settings-help-mail-cache-messages = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-fresh-seconds = Интервал свежести (секунды)
admin-settings-help-mail-cache-fresh-seconds = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-max-age-seconds = Наибольший возраст кэша (секунды)
admin-settings-help-mail-cache-max-age-seconds = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-max-entries = Наибольшее число записей кэша
admin-settings-help-mail-cache-max-entries = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-max-bytes = Бюджет памяти (байты)
admin-settings-help-mail-cache-max-bytes = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-prefetch-enabled = Включить упреждающую предзагрузку
admin-settings-help-mail-cache-prefetch-enabled = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-hover-enabled = Предзагружать при наведении на папку
admin-settings-help-mail-cache-hover-enabled = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-focus-enabled = Предзагружать, когда папка получает фокус клавиатуры
admin-settings-help-mail-cache-focus-enabled = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-dwell-ms = Задержка наведения и фокуса (миллисекунды)
admin-settings-help-mail-cache-dwell-ms = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-idle-enabled = Предзагружать папки в простое
admin-settings-help-mail-cache-idle-enabled = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-idle-delay-ms = Задержка простоя (миллисекунды)
admin-settings-help-mail-cache-idle-delay-ms = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-max-concurrency = Наибольшее число одновременных предзагрузок
admin-settings-help-mail-cache-max-concurrency = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-max-queued = Наибольшее число предзагрузок в очереди
admin-settings-help-mail-cache-max-queued = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-preload-messages = Разрешить упреждающую загрузку текстов писем
admin-settings-help-mail-cache-preload-messages = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.
admin-settings-key-mail-cache-excluded-folders = Папки, исключённые из предзагрузки
admin-settings-help-mail-cache-excluded-folders = Хранит недавно посещённые папки и письма в памяти этой вкладки. Кэшированное содержимое обновляется в фоне. Закрытие вкладки или выход очищает его. Предзагрузка никогда не отмечает почту прочитанной и не загружает удалённые изображения.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-mail-cache-refresh-on-open = Обновлять кэшированное содержимое при открытии
admin-settings-help-mail-cache-refresh-on-open = Обновлять кэшированное содержимое при открытии
admin-settings-key-mail-templates-enabled = Шаблоны ответов
admin-settings-help-mail-templates-enabled = Включить личные и общие шаблоны ответов в виде обычного текста в редакторе писем. Пользователи могут отказаться от них, если параметр не заблокирован.
admin-settings-key-mail-templates-allow-personal = Разрешить личные шаблоны
admin-settings-help-mail-templates-allow-personal = Разрешить пользователям создавать собственные шаблоны ответов. Общими шаблонами отдельно управляют администраторы.
admin-settings-key-mail-templates-shared = Общие шаблоны ответов
admin-settings-help-mail-templates-shared = Управление повторно используемыми ответами в виде обычного текста для этой области. Шаблоны никогда не задают получателей и не отправляют письма.
admin-settings-key-mail-templates-personal = Личные шаблоны ответов
admin-settings-help-mail-templates-personal = Личные повторно используемые ответы в виде обычного текста. Для создания и изменения должны быть включены шаблоны ответов и личные шаблоны.
admin-settings-key-mail-compose-window = Окно редактора
admin-settings-help-mail-compose-window = Открывать новые черновики во вкладке рабочего пространства или в отдельном окне. Если всплывающие окна заблокированы, используется вкладка.
admin-settings-key-mail-reply-copy-to-original = Хранить ответы в исходной папке
admin-settings-help-mail-reply-copy-to-original = Сохранять копию отправленного ответа в папке исходного письма, если это поддерживается.
admin-settings-key-mail-compose-check-invalid-recipients = Проверять недопустимых получателей перед отправкой
admin-settings-help-mail-compose-check-invalid-recipients = Что происходит при отправке письма на недопустимый адрес: разрешить, предупредить, предупредить и запомнить выбор или отклонить.
admin-settings-key-mail-compose-check-missing-attachment = Проверять отсутствие вложения перед отправкой
admin-settings-help-mail-compose-check-missing-attachment = Что происходит, если в тексте упомянуто вложение, но ничего не прикреплено: разрешить, предупредить, предупредить и запомнить выбор или отклонить.
admin-settings-key-mail-compose-check-empty-subject = Проверять пустую тему перед отправкой
admin-settings-help-mail-compose-check-empty-subject = Что происходит, если у письма нет темы: разрешить, предупредить, предупредить и запомнить выбор или отклонить.
admin-settings-key-mail-compose-auto-chip-recipients = Автоматически завершать ввод получателей
admin-settings-help-mail-compose-auto-chip-recipients = Преобразовывать введённый адрес в метку получателя после пробела, запятой или точки с запятой.
admin-settings-key-mail-compose-header-collapse = Заголовок редактора
admin-settings-help-mail-compose-header-collapse = Поведение строк «Кому», «Копия», «Скрытая копия» и «Тема» в редакторе: развёрнуты, свёрнуты в однострочную сводку или автоматически (свёрнуты при вводе текста, открыты при выборе поля). Пользователи могут сворачивать и разворачивать заголовок сами, если параметр не заблокирован.
admin-settings-key-mail-compose-recipient-separator = Разделитель получателей
admin-settings-help-mail-compose-recipient-separator = Разделитель, вставляемый после завершённого получателя: запятая или точка с запятой.
admin-settings-key-mail-reply-from-mode = Профиль для ответов и пересылки
admin-settings-help-mail-reply-from-mode = От какого профиля отправляется ответ или пересылка: на который было адресовано исходное письмо, по умолчанию, с выбором из списка или текущий выбранный.
admin-settings-key-mail-forward-mode = Формат пересылки по умолчанию
admin-settings-help-mail-forward-mode = Пересылать текст в теле письма или прикреплять исходное письмо как файл .eml.
admin-settings-key-mail-show-go-to-top = Показывать кнопку «Наверх»
admin-settings-help-mail-show-go-to-top = Показывать в списке писем кнопку «Наверх», которая прокручивает к первому письму. По умолчанию выключено.
admin-settings-key-mail-show-create-folder-button = Показывать кнопку «Создать папку»
admin-settings-help-mail-show-create-folder-button = Показывать в боковой панели рядом с кнопкой «Написать» кнопку «Создать папку» только со значком. По умолчанию выключено.
admin-settings-key-security-protection-user-agent-mode = Режим наблюдения за user-agent
admin-settings-help-security-protection-user-agent-mode = По умолчанию — принудительный: выбранные распознанные заголовки ботов блокируются сразу, если включена блокировка ботов при первом запросе. Наблюдение записывает решения без отказа; «выкл.» отключает этот параметр. Заявленные в заголовках данные не являются подтверждённой личностью.
admin-settings-key-security-protection-user-agent-block-bots = Блокировать распознанных ботов при первом запросе
admin-settings-help-security-protection-user-agent-block-bots = По умолчанию включено. Выбранные заголовки поисковых, ИИ-, автоматизированных и пользовательских ботов вызывают временную блокировку HTTP 429 уже при первом запросе в принудительном режиме. Режим наблюдения только записывает это; значение false возвращает прежние лимиты запросов. Отсутствующие или необычные заголовки, область действия и исключения работают как раньше.
admin-settings-key-security-protection-user-agent-categories = Категории user-agent
admin-settings-help-security-protection-user-agent-categories = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-weak-requests = Порог запросов со слабым сигналом
admin-settings-help-security-protection-user-agent-weak-requests = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-crawler-requests = Порог запросов поисковых роботов
admin-settings-help-security-protection-user-agent-crawler-requests = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-window-secs = Окно наблюдения за user-agent (секунды)
admin-settings-help-security-protection-user-agent-window-secs = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-max-action = Максимальная реакция на user-agent
admin-settings-help-security-protection-user-agent-max-action = Максимальная эскалация при слабых или повторяющихся признаках, по умолчанию — ограничение скорости. Этот предел не ограничивает блокировку выбранных распознанных заголовков ботов при первом запросе.
admin-settings-key-security-protection-user-agent-scope = Область действия политики user-agent
admin-settings-help-security-protection-user-agent-scope = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-adaptive = Адаптивные наблюдения за user-agent
admin-settings-help-security-protection-user-agent-adaptive = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-patterns = Шаблоны user-agent
admin-settings-help-security-protection-user-agent-patterns = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-security-protection-user-agent-exceptions = Исключения user-agent
admin-settings-help-security-protection-user-agent-exceptions = Настраивает эту часть необязательного наблюдения за user-agent. Текст user-agent, передаваемый клиентом, — слабый сигнал, а не доказательство личности или злоупотребления.
admin-settings-key-mail-reader-unsafe-html-mode = Потенциально небезопасный HTML письма
admin-settings-help-mail-reader-unsafe-html-mode = По умолчанию «Спрашивать»: для каждого письма доступна кнопка очищенного HTML. «Строгий» показывает только безопасный текст; «очищенный» автоматически показывает очищенный HTML. Активное содержимое и автоматические внешние ресурсы остаются заблокированными. Заблокируйте, чтобы применять эту политику принудительно.
admin-settings-key-reader-attachments-density = Плотность вложений
admin-settings-help-reader-attachments-density = По умолчанию компактные кнопки файлов или более удобные превью. Определяет отображение при развёрнутых вложениях письма.
admin-settings-key-cap-compose-open-tracking = Разрешить отслеживание открытия писем
admin-settings-help-cap-compose-open-tracking = Может ли эта учётная запись фиксировать открытия писем. Требуется настроенный публичный URL; для учётных записей с отключённой функцией нельзя создавать и привязывать токены отслеживания.
admin-settings-key-mail-open-tracking = Отслеживать открытия по умолчанию
admin-settings-help-mail-open-tracking = Выбор отслеживания по умолчанию для подходящих форматированных писем. Пользователи могут изменить его, если он не заблокирован; отключение возможности отслеживания всегда имеет приоритет.
admin-settings-key-cap-reading-original-html = Разрешить отображение исходного HTML
admin-settings-help-cap-reading-original-html = Могут ли пользователи включать просмотр исходного HTML для отдельных писем. Очищающий обработчик по умолчанию и защита изолированным фреймом сохраняются.
admin-settings-key-mail-reader-allow-original-html = Предлагать просмотр исходного HTML

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-reader-allow-original-html = Показывать для каждого письма вариант использования исходного HTML, если это разрешает возможность чтения. По умолчанию выключено; обработчик писем по умолчанию не меняется.
admin-settings-key-privacy-tracker-override-allowed = Предлагать согласие на трекеры для письма
admin-settings-help-privacy-tracker-override-allowed = Показывать действие «Разрешить трекеры» на панели конфиденциальности письма. Доступно по умолчанию, но никогда не действует без явного разрешения. Заблокированная политика блокировки трекеров по-прежнему запрещает такое исключение.
admin-settings-key-privacy-max-concurrent-global = Глобальное число параллельных загрузок изображений
admin-settings-help-privacy-max-concurrent-global = Максимальное число активных сетевых запросов изображений во всём процессе сервера. Диапазон 1–256; по умолчанию 128. Лимиты писем и учётных записей продолжают действовать.
admin-settings-key-privacy-max-concurrent-pipeline = Параллельные конвейеры обработки изображений
admin-settings-help-privacy-max-concurrent-pipeline = Максимальное число изображений, одновременно допускаемых к загрузке, декодированию и записи в кэш. Диапазон 1–256; по умолчанию 128. Лимиты сети и декодеров применяются отдельно.
admin-settings-key-privacy-max-concurrent-decodes = Параллельные декодеры изображений
admin-settings-help-privacy-max-concurrent-decodes = Максимальное число одновременных декодирований изображений. Диапазон 1–16; по умолчанию 10. Большие значения расходуют больше процессора и памяти.
admin-settings-key-privacy-max-queued-requests = Ожидающие запросы изображений
admin-settings-help-privacy-max-queued-requests = Максимальное число запросов, ожидающих сетевых ресурсов или мощности обработки изображений. Диапазон 16–16384; по умолчанию 2048. Переполнение или истечение срока в очереди можно повторить.
admin-settings-key-privacy-rate-burst = Допустимый всплеск запросов изображений
admin-settings-help-privacy-rate-burst = Начальное и максимальное число токенов загрузки на учётную запись, пополняемых с заданной скоростью запросов изображений. Диапазон 1–12000; по умолчанию 1024. Попадания в кэш не расходуют токены.
admin-settings-key-privacy-image-rate-mode = Расчёт скорости запросов изображений
admin-settings-help-privacy-image-rate-mode = Автоматически масштабирует всплеск и пополнение для учётной записи по мощности реплик, нагрузке очереди и числу изображений в письме в пределах нижних и верхних границ ниже. Вручную использует настройки допустимого всплеска и скорости запросов без изменений. По умолчанию автоматически. Запросы сверх лимита ждут токен до истечения срока очереди загрузки; аутентификация и разрешения проверяются в первую очередь.
admin-settings-key-privacy-image-rate-burst-floor = Нижняя граница всплеска (автоматически)
admin-settings-help-privacy-image-rate-burst-floor = Наименьший допустимый всплеск на учётную запись в автоматическом режиме. Диапазон 1–12000; по умолчанию 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Верхняя граница всплеска (автоматически)
admin-settings-help-privacy-image-rate-burst-ceiling = Наибольший допустимый всплеск на учётную запись в автоматическом режиме. Диапазон 1–12000; по умолчанию 2048.
admin-settings-key-privacy-image-rate-refill-floor = Нижняя граница пополнения (автоматически)
admin-settings-help-privacy-image-rate-refill-floor = Наименьшее пополнение на учётную запись, в токенах загрузки в минуту, в автоматическом режиме. Диапазон 10–60000; по умолчанию 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Верхняя граница пополнения (автоматически)
admin-settings-help-privacy-image-rate-refill-ceiling = Наибольшее пополнение на учётную запись, в токенах загрузки в минуту, в автоматическом режиме. Диапазон 10–60000; по умолчанию 6000.
admin-settings-key-privacy-image-cache-mode = Расчёт размера кэша изображений
admin-settings-help-privacy-image-cache-mode = Автоматически подбирает размер кэша по памяти, оставшейся после резервов декодеров и конвейеров, и увеличивает срок хранения, пока доля попаданий высока, а память свободна. Вручную использует настройки размера кэша, числа записей и срока хранения без изменений. По умолчанию автоматически.
admin-settings-key-privacy-image-cache-bytes-floor = Нижняя граница размера кэша (автоматически)
admin-settings-help-privacy-image-cache-bytes-floor = Наименьший кэш изображений в байтах в автоматическом режиме. Диапазон 0–1073741824; по умолчанию 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Верхняя граница размера кэша (автоматически)
admin-settings-help-privacy-image-cache-bytes-ceiling = Наибольший кэш изображений в байтах в автоматическом режиме. Диапазон 0–1073741824; по умолчанию 268435456.
admin-settings-key-privacy-cache-entries = Лимит записей кэша изображений
admin-settings-help-privacy-cache-entries = Максимальное число изображений в кэше; в ручном режиме — фиксированное число, в автоматическом — верхняя граница. Диапазон 16–65536; по умолчанию 4096.
admin-settings-key-privacy-cache-entry-bytes = Наибольшее изображение в кэше
admin-settings-help-privacy-cache-entry-bytes = Наибольшее отдельное перекодированное изображение, хранимое в кэше, в байтах. Более крупные изображения отдаются, но не сохраняются. Диапазон 65536–16777216; по умолчанию 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Нижняя граница срока хранения кэша (автоматически)
admin-settings-help-privacy-image-cache-ttl-floor-secs = Наименьший срок хранения кэша в секундах в автоматическом режиме, применяется при низкой доле попаданий или нехватке памяти. Диапазон 0–86400; по умолчанию 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Верхняя граница срока хранения кэша (автоматически)
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Наибольший срок хранения кэша в секундах в автоматическом режиме, применяется при высокой доле попаданий и свободной памяти. Диапазон 0–86400; по умолчанию 21600.
admin-settings-key-privacy-cache-ttl-secs = Срок хранения успешных изображений в кэше
admin-settings-help-privacy-cache-ttl-secs = Число секунд, в течение которых декодированные изображения можно повторно использовать в этом процессе. Диапазон 0–86400; по умолчанию 3600. Ноль отключает повторное использование успешных изображений; авторизация всегда проверяется первой.
admin-settings-key-privacy-image-latency-tracking = Отслеживание скорости прокси изображений
admin-settings-help-privacy-image-latency-tracking = Записывает в памяти время обработки каждого изображения через прокси (медиана, p95, p99 и время ожидания, загрузки и декодирования) в виде счётчиков по фиксированным интервалам времени. URL, хосты и учётные записи не сохраняются. По умолчанию включено; история сбрасывается при перезапуске и относится к одному процессу сервера. Выключение прекращает новую запись; раздел «Скорость» продолжает показывать уже записанное.
admin-settings-key-privacy-negative-cache-ttl-secs = Срок хранения неудачных изображений в кэше
admin-settings-help-privacy-negative-cache-ttl-secs = Число секунд, в течение которых подходящие сбои источника запоминаются, чтобы избежать повторных запросов. Диапазон 0–300; по умолчанию 10. Ноль отключает повторное использование сбоев. Явная повторная попытка очищает затронутые сбои.
admin-settings-key-privacy-image-retry-attempts = Автоматические повторы загрузки изображений
admin-settings-help-privacy-image-retry-attempts = Максимальное число повторов при временных сбоях изображений за одну видимую загрузку письма. Диапазон 0–8; по умолчанию 3. Ноль отключает автоматические повторы, сохраняя ручной повтор.
admin-settings-key-privacy-image-retry-delay-ms = Задержка автоматического повтора изображения
admin-settings-help-privacy-image-retry-delay-ms = Число миллисекунд между автоматическими повторами загрузки изображения после стабилизации текущего кадра. Диапазон 500–60000; по умолчанию 5000. Окончательные отказы автоматически не повторяются.
admin-settings-key-privacy-image-decode-timeout-ms = Тайм-аут декодирования изображения
admin-settings-help-privacy-image-decode-timeout-ms = Реальное время в миллисекундах, отведённое на одно декодирование изображения в песочнице после его загрузки. Диапазон 1000–60000; по умолчанию 10000. Увеличьте для крупных изображений; сетевые тайм-ауты задаются отдельно.
admin-settings-key-privacy-image-concurrency-mode = Профиль параллелизма изображений
admin-settings-help-privacy-image-concurrency-mode = Автоматический подстраивается под мощность процессора, память контейнера и число одобренных изображений в каждом письме. Профили «Мало ресурсов», «Сбалансированный» и «Высокая пропускная способность» тоже подстраиваются, но с разными предельными значениями безопасности. Ручной напрямую использует числовые пределы. Применение профиля меняет только этот параметр.
admin-settings-cat-image-proxy = Прокси изображений
admin-settings-key-privacy-image-ram-max-mi-b = Максимум уровня RAM
admin-settings-help-privacy-image-ram-max-mi-b = Верхняя граница в МиБ для уровня изображений в памяти, применяемая поверх автоматического или ручного размера кэша. Ноль означает отсутствие ограничения. Держите значение небольшим и позвольте дисковому уровню хранить основной объём. Диапазон 0–1048576; по умолчанию 0.
admin-settings-key-privacy-image-disk-enabled = Дисковый уровень изображений
admin-settings-help-privacy-image-disk-enabled = Хранит уже перекодированные изображения на диске за уровнем RAM. По умолчанию выключено; если каталог отсутствует, заполнен или доступен только для чтения, кэш молча продолжает работать только в RAM. Сохраняются только данные на выходе из изолированной среды, под хешированными именами файлов.
admin-settings-key-privacy-image-disk-path = Каталог изображений на диске
admin-settings-help-privacy-image-disk-path = Абсолютный путь к каталогу дискового уровня. Пустое значение — папка image-cache рядом с файлом базы данных. Смонтируйте там том с правом записи, если корневая файловая система доступна только для чтения. Относительные пути и .. отклоняются, уровень остаётся выключенным.
admin-settings-key-privacy-image-disk-mode = Расчёт размера диска изображений
admin-settings-help-privacy-image-disk-mode = Автоматически использует десятую часть места, которое мог бы занять этот кэш (свободное место плюс уже занятое им), между нижней и верхней границей и никогда не более половины этого места. Вручную использует указанный ниже максимальный размер без изменений. По умолчанию автоматически.
admin-settings-key-privacy-image-disk-max-mi-b = Размер диска изображений (вручную)
admin-settings-help-privacy-image-disk-max-mi-b = Размер дискового уровня в МиБ в ручном режиме; в показаниях отображается фактический размер в обоих режимах. Диапазон 1–1048576; по умолчанию 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Нижняя граница размера диска (автоматически)
admin-settings-help-privacy-image-disk-floor-mi-b = Наименьший дисковый уровень в МиБ в автоматическом режиме; также используется, пока свободное место неизвестно. Диапазон 0–1048576; по умолчанию 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Верхняя граница размера диска (автоматически)
admin-settings-help-privacy-image-disk-ceiling-mi-b = Наибольший дисковый уровень в МиБ в автоматическом режиме. Диапазон 0–1048576; по умолчанию 4096.
admin-settings-key-privacy-image-disk-entries = Лимит записей на диске
admin-settings-help-privacy-image-disk-entries = Максимальное число изображений на диске. Каждая запись также занимает фиксированный объём RAM в индексе, который учитывается в уровне RAM. Диапазон 16–1048576; по умолчанию 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Наибольшее изображение на диске
admin-settings-help-privacy-image-disk-entry-bytes = Наибольшее отдельное перекодированное изображение, записываемое на диск, в байтах. Изображения, превышающие лимит записи в RAM, но укладывающиеся в этот, сразу попадают на диск. Диапазон 65536–67108864; по умолчанию 8388608.
admin-settings-key-privacy-image-disk-min-hits = Попадания до записи на диск
admin-settings-help-privacy-image-disk-min-hits = Число попаданий, необходимое записи в RAM, чтобы при вытеснении она была записана на диск, а не отброшена. Ноль записывает каждую вытесняемую запись, оценка которой превышает порог понижения. Диапазон 0–100; по умолчанию 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Очередь записи на диск
admin-settings-help-privacy-image-disk-write-queue-mi-b = Объём данных изображений в МиБ, которые могут ждать фоновой записи на диск. Учитывается в уровне RAM; сверх него вытесняемые изображения отбрасываются, а не ставятся в очередь. Диапазон 1–256; по умолчанию 8.
admin-settings-key-privacy-image-ttl-step-secs = Прирост срока хранения за попадание
admin-settings-help-privacy-image-ttl-step-secs = Число секунд, добавляемых к оставшемуся сроку хранения изображения при попадании, умноженное на 1 плюс логарифм его затухающего числа попаданий, поэтому часто используемые изображения живут дольше. Ноль оставляет каждое изображение с базовым сроком. Диапазон 0–86400; по умолчанию 600.
admin-settings-key-privacy-image-ttl-max-secs = Наибольший продлённый срок хранения
admin-settings-help-privacy-image-ttl-max-secs = Наибольший оставшийся срок хранения в секундах, который могут накопить попадания. Диапазон 60–2592000; по умолчанию 86400.
admin-settings-key-privacy-image-max-age-secs = Максимальный возраст изображения
admin-settings-help-privacy-image-max-age-secs = Возраст в секундах, по достижении которого изображение загружается заново независимо от попаданий. Путь загрузки не хранит ETag и Last-Modified источника, поэтому условной перепроверки нет; изображение просто загружается повторно. Диапазон 3600–7776000; по умолчанию 604800.
admin-settings-key-privacy-image-score-half-life-secs = Период полураспада оценки
admin-settings-help-privacy-image-score-half-life-secs = Число секунд, за которое число попаданий и свежесть изображения уменьшаются вдвое в оценке приоритета, определяющей RAM, диск и вытеснение. Диапазон 60–604800; по умолчанию 3600.
admin-settings-key-privacy-image-score-demote-milli = Оценка понижения
admin-settings-help-privacy-image-score-demote-milli = Оценка в тысячных, ниже которой изображение из RAM переводится на диск или отбрасывается. Порог повышения — это значение плюс гистерезис. Диапазон 0–10000; по умолчанию 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Гистерезис повышения
admin-settings-help-privacy-image-score-hysteresis-percent = Процент, на который порог повышения превышает порог понижения. Оценка между ними оставляет изображение там, где оно есть. Диапазон 0–500; по умолчанию 50.
admin-settings-key-privacy-image-context-open-percent = Вес открытого письма
admin-settings-help-privacy-image-context-open-percent = Вес оценки в процентах для изображений, запрошенных для письма. Изображения письма считаются недавними после десяти минут простоя. Диапазон 100–1000; по умолчанию 200.
admin-settings-key-privacy-image-context-recent-percent = Вес недавнего письма
admin-settings-help-privacy-image-context-recent-percent = Вес оценки в процентах для недавно использованных изображений. Запросы без указания письма имеют вес 100. Диапазон 100–1000; по умолчанию 150.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-account-share-percent = Доля аккаунта в ОЗУ
admin-settings-help-privacy-image-account-share-percent = Наибольшая доля уровня ОЗУ, которую могут занять вставки одного аккаунта, пока другие аккаунты пользуются кэшем; сверх неё аккаунт вытесняет собственные изображения с наименьшей оценкой. Один аккаунт может занять весь объём. Диапазон 1–100; по умолчанию 50.
admin-settings-key-privacy-image-sweep-secs = Интервал очистки кэша
admin-settings-help-privacy-image-sweep-secs = Число секунд между фоновыми проходами, которые удаляют устаревшие изображения, пересчитывают оценки ограниченной партии, переносят неактивные изображения из ОЗУ при нехватке памяти и повторно проверяют свободное место на диске. Диапазон 5–3600; по умолчанию 30.
admin-settings-key-privacy-image-snapshot-secs = Интервал снимков индекса
admin-settings-help-privacy-image-snapshot-secs = Число секунд между атомарными снимками дискового индекса, которые сохраняют счётчики попаданий и продлённые сроки хранения после перезапуска без записи при каждом попадании. Ноль отключает снимки; после перезапуска тогда читается заголовок каждого файла. Диапазон 0–86400; по умолчанию 300.
admin-settings-key-privacy-image-sweep-batch = Размер партии очистки
admin-settings-help-privacy-image-sweep-batch = Число записей, которые один проход обслуживания может удалить по сроку, пересчитать или вытеснить. Ограничивает работу под блокировкой кэша. Диапазон 16–8192; по умолчанию 256.
admin-image-proxy-title = Состояние кэша изображений
admin-image-proxy-unavailable = Состояние кэша изображений недоступно.
admin-image-proxy-unknown = неизвестно
admin-image-proxy-ram = Уровень ОЗУ: занято { $used } из { $budget } (настроено { $configured }), изображений: { $entries }, горячих: { $hot }, холодных: { $cold }, доля попаданий { $hit }.
admin-image-proxy-disk-off = Дисковый уровень: выключен.
admin-image-proxy-disk = Дисковый уровень: занято { $bytes } из действующих { $budget } ({ $mode }; нижняя граница { $floor }, верхняя { $ceiling }, вручную { $manual }), изображений: { $entries }, горячих: { $hot }, холодных: { $cold }, доля попаданий { $hit }.
admin-image-proxy-disk-inactive = Дисковый уровень: включён, но не активен. { $reason }
admin-image-proxy-moves = Повышений в минуту: { $perMin }. Всего повышений { $promotions }, понижений { $demotions }, отбрасываний { $drops }. Записей индекса: { $index }.
admin-image-proxy-refresh = Обновить состояние
admin-settings-key-security-trust-enabled = Уровни доверия
admin-settings-help-security-trust-enabled = Присваивает каждому запросу уровень: выполнен вход, известный источник (недавний успешный вход с этого адреса) или неизвестный. Меняются только лимиты анонимных запросов; учётные данные, вторые факторы и блокировки никогда не ослабляются.
admin-settings-key-security-trust-known-ttl-days = Окно известного источника (дни)
admin-settings-help-security-trust-known-ttl-days = Как долго успешный вход сохраняет адрес известным. Неудачные попытки не учитываются.
admin-settings-key-security-trust-known-capacity = Число хранимых известных источников
admin-settings-help-security-trust-known-capacity = Наибольшее число известных источников в памяти (для каждого: хеш с солью, страна и число аккаунтов). Самый старый удаляется первым.
admin-settings-key-security-trust-persist = Сохранять известные источники между перезапусками
admin-settings-help-security-trust-persist = Сохраняет таблицу известных источников (хеши с ключом, страна, время последней активности, число аккаунтов) в хранилище настроек и загружает её при запуске. Записи старше окна удаляются.
admin-settings-key-security-trust-persist-interval-secs = Интервал сохранения известных источников (секунды)
admin-settings-help-security-trust-persist-interval-secs = Число секунд между сохранениями, от 30 до 3600. Сохранение выполняется только при изменении таблицы и ещё раз при штатном завершении.
admin-settings-key-security-trust-unknown-percent = Лимит для неизвестного источника (%)
admin-settings-help-security-trust-unknown-percent = Лимит анонимных запросов неизвестного источника в процентах от настроенных лимитов. 100 сохраняет базовое значение; меньше означает строже.
admin-settings-key-security-trust-known-percent = Лимит для известного источника (%)
admin-settings-help-security-trust-known-percent = Лимит анонимных запросов известного источника. 200 удваивает бюджеты запросов и вдвое сокращает паузу после входа и сложность задачи. Никогда не обходит учётные данные, вторые факторы и блокировки.
admin-settings-key-security-trust-ua-missing = Боты: пустой user-agent
admin-settings-help-security-trust-ua-missing = Действие для анонимных запросов с пустым user-agent: разрешить, ограничить или заблокировать. Сеансы с выполненным входом не затрагиваются.
admin-settings-key-security-trust-ua-uncommon = Боты: нераспознанный user-agent
admin-settings-help-security-trust-ua-uncommon = Действие для анонимных запросов с нераспознанным user-agent: разрешить, ограничить или заблокировать.
admin-settings-key-security-trust-ua-automation = Боты: скрипты и автоматизация
admin-settings-help-security-trust-ua-automation = Действие для анонимных запросов, заявляющих себя скриптом, HTTP-библиотекой, headless-браузером или обычным ботом: разрешить, ограничить или заблокировать.
admin-settings-key-security-trust-ua-search = Боты: поисковые роботы
admin-settings-help-security-trust-ua-search = Действие для анонимных запросов, заявляющих себя поисковым роботом: разрешить, ограничить или заблокировать.
admin-settings-key-security-trust-ua-ai-training = Боты: роботы для обучения ИИ
admin-settings-help-security-trust-ua-ai-training = Действие для анонимных запросов, заявляющих себя GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended или Applebot-Extended: разрешить, ограничить или заблокировать. Заявление — это заголовок, а не проверенная личность.
admin-settings-key-security-trust-ua-ai-search = Боты: роботы ИИ-поиска
admin-settings-help-security-trust-ua-ai-search = Действие для анонимных запросов, заявляющих себя OAI-SearchBot, PerplexityBot или Claude-SearchBot: разрешить, ограничить или заблокировать.
admin-settings-key-security-trust-ua-ai-user = Боты: загрузка ИИ по запросу пользователя
admin-settings-help-security-trust-ua-ai-user = Действие для анонимных запросов, заявляющих себя ChatGPT-User или Claude-User: разрешить, ограничить или заблокировать.
admin-settings-key-security-trust-ua-custom = Боты: собственные шаблоны
admin-settings-help-security-trust-ua-custom = Действие для анонимных запросов, соответствующих собственным шаблонам user-agent: разрешить, ограничить или заблокировать.
admin-settings-key-security-trust-admin-geo-enabled = Блокировать поиск консоли администратора из неизвестных стран
admin-settings-help-security-trust-admin-geo-enabled = Блокирует источник, который ищет консоль администратора из страны без недавнего успешного входа. Требует читаемой базы стран и защиты от зондирования консоли. MW_ADMIN_GEO_ENABLED=false отключает это для восстановления доступа.
admin-settings-key-security-trust-admin-geo-countries = Всегда известные страны (блокировка консоли администратора)
admin-settings-help-security-trust-admin-geo-countries = Коды стран ISO заглавными буквами, всегда считающиеся известными, помимо стран недавних успешных входов.
admin-settings-key-security-trust-admin-geo-lookup-failure = Источник без страны
admin-settings-help-security-trust-admin-geo-lookup-failure = allow не блокирует и записывает случай в журнал; unknown считает источник без определённой страны неизвестной страной.
admin-settings-key-security-trust-admin-geo-threshold = Зондирований до блокировки
admin-settings-help-security-trust-admin-geo-threshold = Число зондирований консоли администратора из неизвестной страны в пределах окна, после которого начинается блокировка.
admin-settings-key-security-trust-admin-geo-window-secs = Окно зондирования (секунды)
admin-settings-help-security-trust-admin-geo-window-secs = Окно для порога зондирований.
admin-settings-key-security-trust-admin-geo-block-on-first = Блокировать с первого зондирования
admin-settings-help-security-trust-admin-geo-block-on-first = Блокирует с первого зондирования из неизвестной страны. Повышает риск блокировки путешествующего администратора.
admin-settings-key-security-trust-admin-geo-block-secs = Первая блокировка (секунды)
admin-settings-help-security-trust-admin-geo-block-secs = Длительность первой блокировки.
admin-settings-key-security-trust-admin-geo-second-block-secs = Вторая блокировка (секунды)
admin-settings-help-security-trust-admin-geo-second-block-secs = Длительность второй блокировки того же источника в пределах периода повторов.
admin-settings-key-security-trust-admin-geo-max-block-secs = Повторная блокировка (секунды)
admin-settings-help-security-trust-admin-geo-max-block-secs = Длительность третьей и каждой последующей блокировки в пределах периода повторов.
admin-settings-key-security-trust-admin-geo-repeat-days = Период повторов (дни)
admin-settings-help-security-trust-admin-geo-repeat-days = Блокировка в течение этого числа дней после предыдущей считается повторной и применяет следующую, более длительную.
admin-settings-key-security-signin-account-geo-mode = Политика страны входа для аккаунта
admin-settings-help-security-signin-account-geo-mode = По умолчанию выключена. Блокирует входы из стран, которыми аккаунт ещё не пользовался, или из любой страны вне его списка разрешённых. IP-адреса и страны из списка разрешённых проходят всегда. Применяется только к новым входам в веб-почту, не к существующим сеансам и не к клиентам протоколов. Определение страны использует базу геолокации IP, которая может быть неверной или устаревшей; VPN, мобильные операторы, прокси и поездки могут заблокировать законный вход или пропустить иностранный. Добавьте доверенные IP-адреса или страны в список разрешённых.
admin-settings-key-security-signin-account-geo-allow-countries = Разрешённые страны аккаунта
admin-settings-help-security-signin-account-geo-allow-countries = Коды стран ISO заглавными буквами, всегда разрешённые для аккаунта. Определение страны использует базу геолокации IP, которая может быть неверной или устаревшей; VPN, мобильные операторы, прокси и поездки могут заблокировать законный вход или пропустить иностранный. Добавьте доверенные IP-адреса или страны в список разрешённых.
admin-settings-key-security-signin-account-geo-allow-ips = Разрешённые IP-адреса аккаунта
admin-settings-help-security-signin-account-geo-allow-ips = IP-адреса или сети CIDR, всегда разрешённые для аккаунта. Один адрес IPv6 означает его сеть /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Разрешённые страны для всех аккаунтов
admin-settings-help-security-signin-account-geo-global-allow-countries = Коды стран ISO заглавными буквами, разрешённые для всех аккаунтов; добавляются к списку каждого аккаунта.
admin-settings-key-security-signin-account-geo-global-allow-ips = Разрешённые IP-адреса для всех аккаунтов
admin-settings-help-security-signin-account-geo-global-allow-ips = IP-адреса или сети CIDR (офис или VPN), разрешённые для всех аккаунтов; добавляются к списку каждого аккаунта.
admin-settings-key-security-signin-account-geo-lookup-failure = Источник без страны (политика аккаунта)
admin-settings-help-security-signin-account-geo-lookup-failure = Когда страна источника не определена: разрешить вход и записать это в журнал или отклонить его.
admin-settings-key-security-signin-account-geo-history-days = История стран аккаунта (дни)
admin-settings-help-security-signin-account-geo-history-days = Сколько дней страна остаётся в истории входов аккаунта для политики необычных стран. Хранится отдельно от журнала аутентификации, поэтому сокращение журнала её не сокращает.
admin-settings-key-security-geo-no-database = Нет базы стран: поведение по умолчанию
admin-settings-help-security-geo-no-database = Что делают функции на основе страны, когда база не загружена: разрешают, считают источник неизвестной страной или блокируют (блокировка касается только входов). Действует и собственная настройка функции; применяется более строгая.
admin-settings-key-security-geo-not-found = Адреса нет в базе: поведение по умолчанию
admin-settings-help-security-geo-not-found = Что делают функции на основе страны, когда база загружена, но записи для адреса нет. Частные, loopback и link-local адреса — локальные сети и никогда не затрагиваются.
admin-settings-key-security-signin-geo-no-database = Политика входа экземпляра без базы
admin-settings-help-security-signin-geo-no-database = Когда политика страны входа экземпляра включена, а база не загружена: блокировать входы (закрытый отказ) или разрешать без проверки. Локальные сети всегда разрешены.
admin-settings-key-security-geo-session-lookup-failure = Оповещения о сеансах без страны
admin-settings-help-security-geo-session-lookup-failure = Когда страну запроса сеанса с выполненным входом определить нельзя: игнорировать или создавать одно оповещение на сеанс в сутки. Никогда не блокирует и не отзывает сеанс.
admin-settings-key-security-protocol-framing-mode = Защита протокола: разбиение запросов
admin-settings-help-security-protocol-framing-mode = Класс request smuggling: Content-Length вместе с Transfer-Encoding, Transfer-Encoding не точно chunked (или в HTTP/1.0 и HTTP/2), нечисловой или конфликтующий Content-Length. Режим enforce отвечает 400 и закрывает соединение; detect только считает. Исключение по префиксу маршрута к этому классу не применяется.
admin-settings-key-security-protocol-host-mode = Защита протокола: синтаксис Host
admin-settings-help-security-protocol-host-mode = Синтаксис заголовка Host: отсутствует в HTTP/1.1, пустой, повторённый или содержащий данные пользователя, пробелы либо символы пути. Литералы IPv6, порты и конечная точка допускаются. Режим enforce отвечает 400 и закрывает соединение.
admin-settings-key-security-protocol-host-pinning-mode = Защита протокола: Host должен совпадать с публичным именем
admin-settings-help-security-protocol-host-pinning-mode = Host не совпадает с хостом MW_PUBLIC_URL или одним из allowedHosts. Ничего не делает, пока ни то, ни другое не задаёт хост. По умолчанию detect, поскольку прокси и проверки состояния могут присылать внутреннее имя.
admin-settings-key-security-protocol-path-control-mode = Защита протокола: управляющие символы в пути
admin-settings-help-security-protocol-path-control-mode = NUL, управляющие символы или закодированные CR/LF (%00, %0d, %0a) в пути запроса. Режим enforce отвечает 400. Никогда не исключается префиксом маршрута.
admin-settings-key-security-protocol-target-mode = Защита протокола: уловки в цели запроса
admin-settings-help-security-protocol-target-mode = Двойное или избыточное процентное кодирование точек и косых черт, закодированные точки, сегменты с точками, пустые сегменты пути (//), обратные косые черты, необработанные байты не ASCII и цель в абсолютной форме, указывающая иной адресат, чем Host.
admin-settings-key-security-protocol-path-length-mode = Защита протокола: длина пути и запроса
admin-settings-help-security-protocol-path-length-mode = Путь или запрос длиннее maxPathBytes или maxQueryBytes. Режим enforce отвечает 414.
admin-settings-key-security-protocol-method-mode = Защита протокола: методы HTTP
admin-settings-help-security-protocol-method-mode = Метод вне allowedMethods (по умолчанию TRACE, TRACK, CONNECT, а также неизвестные методы и методы в нижнем регистре). Режим enforce отвечает 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Защита протокола: повторённые однозначные заголовки
admin-settings-help-security-protocol-duplicate-headers-mode = Заголовок, который должен встречаться один раз, но встречается несколько раз (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, условные заголовки и заголовки диапазона), повторённый Content-Length с равными значениями или повторные заголовки Cookie в HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Защита протокола: заголовки пересылки от недоверенных узлов
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* или X-Real-IP от узла вне MW_TRUSTED_PROXIES. Эти заголовки в любом случае не считаются доверенными; параметр фиксирует попытку. Учитывается в адаптивной защите только при настроенных доверенных прокси.
admin-settings-key-security-protocol-header-hygiene-mode = Защита протокола: гигиена заголовков
admin-settings-help-security-protocol-header-hygiene-mode = Число заголовков, одно значение или заголовок Cookie сверх настроенных лимитов; байты не ASCII в заголовках; Upgrade или Connection: upgrade вне маршрута WebSocket; Connection, называющий заголовки вроде Content-Length или Host. Режим enforce отвечает 431 при превышении размера и числа и 400 в остальных случаях.
admin-settings-key-security-protocol-body-mode = Защита протокола: разбиение тела запроса
admin-settings-help-security-protocol-body-mode = GET или HEAD с телом, Content-Length больше maxBodyBytes (413), Expect, отличный от 100-continue (417), или без тела.
admin-settings-key-security-protocol-max-path-bytes = Защита протокола: наибольший путь (байты)
admin-settings-help-security-protocol-max-path-bytes = Наибольший путь запроса, после которого класс длины пути сообщает о нарушении.
admin-settings-key-security-protocol-max-query-bytes = Защита протокола: наибольшая строка запроса (байты)
admin-settings-help-security-protocol-max-query-bytes = Наибольшая строка запроса, после которой класс длины пути сообщает о нарушении.
admin-settings-key-security-protocol-max-header-count = Защита протокола: наибольшее число заголовков
admin-settings-help-security-protocol-max-header-count = Наибольшее число полей заголовка в одном запросе, после которого класс гигиены заголовков сообщает о нарушении. Жёсткий предел самого слушателя (MW_HTTP_MAX_HEADERS) также действует.
admin-settings-key-security-protocol-max-header-value-bytes = Защита протокола: наибольшее значение заголовка (байты)
admin-settings-help-security-protocol-max-header-value-bytes = Наибольшее отдельное значение заголовка, после которого класс гигиены заголовков сообщает о нарушении.
admin-settings-key-security-protocol-max-cookie-bytes = Защита протокола: наибольший заголовок Cookie (байты)
admin-settings-help-security-protocol-max-cookie-bytes = Суммарный размер заголовков Cookie, после которого класс гигиены заголовков сообщает о нарушении.
admin-settings-key-security-protocol-max-body-bytes = Защита протокола: наибольшее объявленное тело (байты)
admin-settings-help-security-protocol-max-body-bytes = Наибольший Content-Length, который класс тела принимает без сообщения. Держите его выше самой большой загрузки или импорта, принимаемых маршрутом; исключение по префиксу маршрута может поднять его для одного маршрута.
admin-settings-key-security-protocol-allowed-methods = Защита протокола: разрешённые методы
admin-settings-help-security-protocol-allowed-methods = Методы, которые принимает класс методов. По умолчанию в списке обычные глаголы HTTP и глаголы WebDAV, CalDAV и CardDAV; сервер сейчас не монтирует входящие маршруты DAV, поэтому они получают 405 на маршрутизаторе.
admin-settings-key-security-protocol-allowed-hosts = Защита протокола: дополнительные принимаемые хосты
admin-settings-help-security-protocol-allowed-hosts = Имена хостов, принимаемые классом привязки хоста помимо хоста MW_PUBLIC_URL, например внутреннее имя, которое использует прокси или проверка состояния. Порты и конечная точка игнорируются.
admin-settings-key-security-protocol-exempt-prefixes = Защита протокола: исключённые префиксы маршрутов
admin-settings-help-security-protocol-exempt-prefixes = Префиксы маршрутов (каждый начинается с / и длиннее одного символа), для которых эвристические классы не выполняются, для законного клиента, которому это нужно, например маршрут большой загрузки. Сопоставляются целые сегменты пути. Нарушения разбиения и управляющих символов в пути никогда не исключаются, а / игнорируется.
admin-settings-key-security-health-mode = Доступ к конечной точке состояния
admin-settings-help-security-health-mode = Кто может обращаться к /healthz и /readyz. internal-only: адреса клиентов loopback, частные, link-local и unique-local. allowlist: они же и сети ниже. open: любой. Остальные получают тот же 404, что и для неизвестного пути. Запрос с токеном MW_HEALTH_TOKEN допускается всегда. Адрес клиента — тот, что определён через доверенные прокси: за обратным прокси без MW_FORWARDED_MODE и MW_TRUSTED_PROXIES каждый запрос выглядит как частный адрес прокси, поэтому допускается весь интернет.
admin-settings-key-security-health-networks = Сети конечной точки состояния
admin-settings-help-security-health-networks = IP-адреса или сети CIDR, которые могут обращаться к /healthz и /readyz в режиме allowlist, помимо внутренних диапазонов, например внешний монитор доступности.
admin-settings-key-security-spray-enabled = Обнаружение password spray
admin-settings-help-security-spray-enabled = Считает различные имена пользователей, с которыми один источник терпит неудачу. В памяти хранится только хеш с ключом каждого имени.
admin-settings-key-security-spray-window-secs = Окно spray (секунды)
admin-settings-help-security-spray-window-secs = Окно, в котором неудачные входы одного источника считаются по различным именам пользователей. По умолчанию 900.
admin-settings-key-security-spray-challenge-at = Spray: порог задачи
admin-settings-help-security-spray-challenge-at = Число различных имён пользователей с ошибками от одного источника в пределах окна, которое включает для него задачу proof-of-work. Успешные входы не учитываются, поэтому общий адрес офиса или кампуса не затрагивается. По умолчанию 10.
admin-settings-key-security-spray-pace-at = Spray: порог замедления
admin-settings-help-security-spray-pace-at = Число различных имён пользователей с ошибками, которое добавляет дополнительное ожидание между попытками с источника. По умолчанию 15.
admin-settings-key-security-spray-lock-at = Spray: порог блокировки
admin-settings-help-security-spray-lock-at = Число различных имён пользователей с ошибками, которое блокирует источник на время блокировки через существующую блокировку входа. По умолчанию 25.
admin-settings-key-security-spray-pace-secs = Spray: ожидание замедления (секунды)
admin-settings-help-security-spray-pace-secs = Число секунд между попытками с источника на пороге замедления или выше. По умолчанию 10.
admin-settings-key-security-spray-lock-secs = Spray: длительность блокировки (секунды)
admin-settings-help-security-spray-lock-secs = Длительность временной блокировки. Разблокировка источника администратором также обнуляет его счётчик. По умолчанию 900.
admin-settings-key-security-spray-known-multiplier = Spray: множитель для известного источника
admin-settings-help-security-spray-known-multiplier = Множитель трёх порогов для известного источника, то есть с недавним успешным входом. По умолчанию 3.
admin-settings-key-security-spray-capacity = Spray: отслеживаемые источники
admin-settings-help-security-spray-capacity = Наибольшее число источников, отслеживаемых в памяти; наименее недавно активные удаляются первыми. По умолчанию 20000.
admin-settings-key-security-spike-enabled = Обнаружение всплесков по всему экземпляру
admin-settings-help-security-spike-enabled = Изучает базовый уровень неудачных входов и переходит в повышенный режим, когда весь экземпляр заметно его превышает. Никогда не блокирует экземпляр.
admin-settings-key-security-spike-multiplier-percent = Всплеск: множитель (%)
admin-settings-help-security-spike-multiplier-percent = Сигнал учитывается, когда он составляет не менее этого процента изученного базового уровня (400 = в четыре раза) и превышает свою абсолютную нижнюю границу.
admin-settings-key-security-spike-min-failures-per-min = Всплеск: минимум неудач в минуту
admin-settings-help-security-spike-min-failures-per-min = Нижняя граница: число неудачных входов в минуту, ниже которого сигнал частоты неудач не учитывается. По умолчанию 20.
admin-settings-key-security-spike-min-sources = Всплеск: минимум источников с неудачами
admin-settings-help-security-spike-min-sources = Нижняя граница: число различных источников с неудачами в пятиминутном интервале, ниже которого этот сигнал не учитывается. По умолчанию 10.
admin-settings-key-security-spike-min-usernames = Всплеск: минимум имён пользователей с неудачами
admin-settings-help-security-spike-min-usernames = Нижняя граница: число различных имён пользователей с неудачами в пятиминутном интервале, ниже которого этот сигнал не учитывается. По умолчанию 30.
admin-settings-key-security-spike-signals-required = Всплеск: требуемые сигналы
admin-settings-help-security-spike-signals-required = Сигналы, которые должны одновременно превышать базовый уровень и нижнюю границу. Распределённая сигнатура spray считается за два. По умолчанию 2.
admin-settings-key-security-spike-warmup-hours = Всплеск: прогрев (часы)
admin-settings-help-security-spike-warmup-hours = Часы наблюдаемого трафика, прежде чем может начаться повышенный режим. До этого базовый уровень только обучается. По умолчанию 48.
admin-settings-key-security-spike-adapt-hours = Всплеск: адаптация базового уровня (часы)
admin-settings-help-security-spike-adapt-hours = Постоянная времени базового уровня. Чем больше, тем медленнее он учитывает рост. Всплеск никогда не обновляет базовый уровень. По умолчанию 168.
admin-settings-key-security-spike-duration-secs = Всплеск: длительность повышенного режима (секунды)
admin-settings-help-security-spike-duration-secs = Как долго длится повышенный режим после последнего признака всплеска; он продлевается, пока сигналы остаются выше половины порога срабатывания. По умолчанию 1800.
admin-settings-key-security-spike-force-challenge = Всплеск: принудительная задача
admin-settings-help-security-spike-force-challenge = В повышенном режиме требует задачу proof-of-work для анонимных входов с неизвестных источников. Аутентифицированные пользователи и известные источники не затрагиваются.
admin-settings-key-security-spike-pace-factor-percent = Всплеск: ожидание при входе (%)
admin-settings-help-security-spike-pace-factor-percent = В повышенном режиме ожидание при входе для неизвестных источников в процентах от обычного. По умолчанию 300.
admin-settings-key-security-session-binding-enabled = Привязка сеанса
admin-settings-help-security-session-binding-enabled = Привязывает каждый сеанс браузера к хешам с ключом от атрибутов клиента. Нативные клиенты с токеном bearer, ключи API и клиенты протоколов исключены.
admin-settings-key-security-session-binding-attributes = Привязка сеанса: атрибуты
admin-settings-help-security-session-binding-attributes = Привязываемые атрибуты: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Network по умолчанию выключен, так как мобильные сети меняются. Неизвестные имена игнорируются.
admin-settings-key-security-session-binding-strictness = Привязка сеанса: строгость
admin-settings-help-security-session-binding-strictness = Tolerant принимает обновления браузера и изменение одного мягкого атрибута; strict отмечает любое мягкое отличие.
admin-settings-key-security-session-binding-webmail-action = Привязка сеанса: действие для веб-почты
admin-settings-help-security-session-binding-webmail-action = Что делает несовпадение сеанса веб-почты: off, detect (только запись), step-up (повторный запрос пароля и второго фактора) или отзыв сеанса. По умолчанию step-up.
admin-settings-key-security-session-binding-admin-action = Привязка сеанса: действие для консоли администратора
admin-settings-help-security-session-binding-admin-action = Что делает несовпадение сеанса консоли администратора: off, detect, step-up или отзыв. По умолчанию отзыв.
admin-settings-key-security-theft-enabled = Обнаружение кражи сеанса
admin-settings-help-security-theft-enabled = Обнаруживает повторное использование ротированного идентификатора сеанса, невозможное перемещение и одновременное использование из разных стран.
admin-settings-key-security-theft-rotation-grace-secs = Кража: льготный период ротации (секунды)
admin-settings-help-security-theft-rotation-grace-secs = После ротации сеанса старый идентификатор может предъявляться повторно в течение этого времени (запрос в процессе), прежде чем это сочтут кражей. По умолчанию 30.
admin-settings-key-security-theft-min-travel-secs = Кража: минимальное время перемещения (секунды)
admin-settings-help-security-theft-min-travel-secs = Смена страны раньше этого времени после предыдущего запроса считается невозможным перемещением. Определение страны может быть неточным; VPN и мобильные сети могут вызывать ложные смены. По умолчанию 3600.
admin-settings-key-security-theft-concurrent-use = Кража: одновременное использование
admin-settings-help-security-theft-concurrent-use = Отмечает сеанс, используемый попеременно из разных стран (или сетей) в течение пяти минут: off, country или network. Определение страны может быть неточным; VPN и мобильные сети могут вызывать ложные смены.
admin-settings-key-security-theft-webmail-action = Кража: реакция для веб-почты
admin-settings-help-security-theft-webmail-action = Реакция на невозможное перемещение или одновременное использование в сеансе веб-почты: alert, step-up или отзыв. Повторное использование ротированного сеанса всегда его отзывает. По умолчанию step-up.
admin-settings-key-security-theft-admin-action = Кража: реакция для консоли администратора
admin-settings-help-security-theft-admin-action = Реакция на невозможное перемещение или одновременное использование в сеансе консоли администратора: alert, step-up или отзыв. По умолчанию отзыв.
admin-settings-key-security-theft-retention-days = Кража: хранение записей (дни)
admin-settings-help-security-theft-retention-days = Как долго хранятся метки ротированных сеансов и записи безопасности отдельных сеансов; задайте не меньше самого длительного срока жизни сеанса. По умолчанию 15.
admin-settings-key-security-alerts-country-change = Оповещение о смене страны
admin-settings-help-security-alerts-country-change = Предупреждает, когда сеанс с выполненным входом используется из страны, отличной от той, где он начат. Определение страны может быть неточным; VPN и мобильные сети могут вызывать ложные смены. Пользователи могут менять это, если параметр не заблокирован; отключение пользователем учитывается, только если отключение оповещений разрешено.
admin-settings-key-security-alerts-country-change-enabled = Оповещение о смене страны: глобальный переключатель

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-security-alerts-country-change-enabled = Выключение отключает оповещение о смене страны для всех. Определение страны может быть неточным; VPN и мобильные сети могут вызывать ложные смены.
admin-settings-key-security-alerts-country-change-cooldown-secs = Оповещение о смене страны: пауза (секунды)
admin-settings-help-security-alerts-country-change-cooldown-secs = Минимальное время между оповещениями о смене страны в одном сеансе. Каждая новая страна вызывает оповещение один раз за сеанс. По умолчанию 300.
admin-settings-key-security-alerts-country-change-user-mute = Оповещение о смене страны: разрешить пользователям отключать
admin-settings-help-security-alerts-country-change-user-mute = Может ли пользователь отключить собственное оповещение о смене страны. Если выключено, сервер продолжает оповещать, даже когда собственная настройка пользователя выключена.
admin-settings-key-security-alerts-channels = Каналы оповещений безопасности
admin-settings-help-security-alerts-channels = Как предупреждения о безопасности сеанса доходят до пользователя: в приложении, сообщением в его собственном почтовом ящике или обоими способами. Пользователи могут менять это, если параметр не заблокирован.
admin-settings-key-security-alerts-email-enabled = Оповещения безопасности по эл. почте
admin-settings-help-security-alerts-email-enabled = Позволяет доставлять предупреждения о безопасности сеанса в собственный почтовый ящик пользователя. Сообщения по SMTP не отправляются; без сеанса доставки предупреждение остаётся в приложении и журнале аудита.
admin-settings-key-security-alerts-sensitivity = Чувствительность оповещений безопасности
admin-settings-help-security-alerts-sensitivity = Низкая предупреждает только о находках по местоположению и никогда не требует повторной проверки и не отзывает; обычная следует действиям администратора; высокая удваивает интервал перемещения и проверяет также одновременные сети. Пользователи могут менять это, если параметр не заблокирован.
admin-settings-key-security-alerts-notice-retention-days = Уведомления безопасности: хранение (дни)
admin-settings-help-security-alerts-notice-retention-days = Как долго уведомления безопасности в приложении хранятся для аккаунта (не более 20 на аккаунт). По умолчанию 30.
admin-settings-inline-effective = Действует:
admin-settings-inline-reset = Отменить изменения
admin-settings-inline-domains = Значения по доменам
admin-attachments-tab-lists = Списки разрешённых и блокируемых
admin-attachments-tab-limits = Ограничения
admin-attachments-tab-delivery = Скачивание и просмотр
admin-attachments-tab-warnings = Предупреждения
admin-attachments-tab-test = Проверка файла
admin-attachments-tab-denied = У вас нет доступа к этой вкладке.
admin-attachments-group-lists = Списки разрешённых и блокируемых
admin-attachments-group-limits = Ограничение размера
admin-attachments-group-delivery = Скачивание и просмотр
admin-attachments-group-warnings = Предупреждения
admin-attachments-add-extension = Расширение, например pdf или .tar.gz
admin-attachments-add-mime = Тип MIME, например image/png или image/*
admin-attachments-add = Добавить
admin-attachments-remove = Удалить { $rule }
admin-attachments-filter = Фильтр списка
admin-attachments-list-label = Правила в этом списке
admin-attachments-count = Правил: { $count } из { $max }
admin-attachments-empty-list = В этом списке нет правил.
admin-attachments-none-match = Ни одно правило не соответствует фильтру.
admin-attachments-bulk = Вставить несколько
admin-attachments-bulk-add = Добавить все
admin-attachments-import = Импортировать текстовый файл
admin-attachments-export = Экспортировать как текст
admin-attachments-err-extension = Это не расширение. Используйте буквы, цифры, дефисы и подчёркивания, до 32 символов; точки могут разделять части, как в tar.gz.
admin-attachments-err-mime = Это не тип MIME. Используйте тип/подтип или тип/* для любого подтипа.
admin-attachments-err-long = Это правило слишком длинное.
admin-attachments-invalid = Не добавлено, недопустимое правило: { $entries }
admin-attachments-duplicates = Уже в списке: { $rules }
admin-attachments-overflow = В списке может быть не более { $max } правил. Остальные не добавлены.
admin-attachments-conflict-blockedOverAllowed = { $rule } пересекается с { $other } в другом списке. Блокирующие правила главнее, поэтому оно блокируется.
admin-attachments-conflict-blockedOverWarn = { $rule } пересекается с { $other } в другом списке. Заблокированный файл отклоняется до любого предупреждения, поэтому предупреждение никогда не появится.
admin-attachments-conflict-allowedAndWarn = { $rule } пересекается с { $other } в другом списке. Файл разрешён и показывает предупреждение.
admin-attachments-bytes-unit = Единица
admin-attachments-unlimited = Без ограничения размера
admin-attachments-bytes-equals = { $size } ({ $bytes } байт)
admin-attachments-list-allowedExtensions = разрешённые расширения
admin-attachments-list-allowedMimeTypes = разрешённые типы MIME
admin-attachments-list-blockedExtensions = блокируемые расширения
admin-attachments-list-blockedMimeTypes = блокируемые типы MIME
admin-attachments-list-warnExtensions = расширения с предупреждением
admin-attachments-list-warnMimeTypes = типы MIME с предупреждением
admin-attachments-sim-title = Проверка файла
admin-attachments-sim-help = Спрашивает сервер, как сохранённая политика обращается с файлом с таким именем, типом и размером. Ничего не изменяется и не сохраняется. Сообщения и вложения никогда не читаются.
admin-attachments-sim-filename = Имя файла
admin-attachments-sim-mime = Тип MIME
admin-attachments-sim-size = Размер
admin-attachments-sim-run = Проверить
admin-attachments-sim-invalid = Введите имя файла и размер, равный нулю или больше.
admin-attachments-sim-allowed = Разрешён
admin-attachments-sim-warn = Разрешён с предупреждением
admin-attachments-sim-blocked = Не доставляется
admin-attachments-sim-preview-yes = Просмотр разрешён.
admin-attachments-sim-preview-no = Просмотр не разрешён.
admin-attachments-sim-checked = Проверенный размер: { $size }.
admin-attachments-sim-matched = Совпавшие правила в: { $lists }.
admin-attachments-sim-matched-none = Ни одно разрешающее, блокирующее или предупреждающее правило не совпало.
admin-attachments-reason-downloadsDisabled = Скачивание отключено.
admin-attachments-reason-attachmentTooLarge = Файл больше ограничения размера.
admin-attachments-reason-attachmentBlocked = Совпадает блокирующее правило.
admin-attachments-reason-attachmentNotAllowed = Существует список разрешённых, и ничто в нём не совпадает.
admin-attachments-reason-attachmentContainerRestricted = Вложенные сообщения и файлы TNEF отклоняются при любом включённом ограничении, поскольку их содержимое не перечислено.
admin-settings-key-mail-reply-images = Изображения в ответах и пересылках
admin-settings-help-mail-reply-images = Включает ли ответ или пересылка изображения исходного сообщения: спрашивать (по умолчанию), всегда разрешать, разрешать только для доверенных доменов отправителей и иначе спрашивать или всегда запрещать. Заблокируйте этот параметр, чтобы закрепить его. Изображения читаются только при включении.
admin-settings-key-mail-reply-image-trusted-domains = Доверенные домены отправителей изображений
admin-settings-help-mail-reply-image-trusted-domains = До 200 точных доменов отправителей, чьи изображения включаются без вопроса, когда политика — доверенные домены. Пользователи добавляют в свой список из запроса о цитате, если параметр не заблокирован.
admin-settings-key-mail-send-recall-delay-seconds = Окно отмены отправки (по умолчанию)
admin-settings-help-mail-send-recall-delay-seconds = Сколько секунд отправленное сообщение ждёт в Исходящих, где отправитель может его отозвать, прежде чем будет доставлено. 0 отправляет сразу. По умолчанию 10, до 1800 (30 минут). Пользователи выбирают своё значение до приведённого ниже предела, если вы не заблокируете параметр.
admin-settings-key-mail-send-recall-delay-max-seconds = Окно отмены отправки (предел)
admin-settings-help-mail-send-recall-delay-max-seconds = Самое длинное окно отмены отправки, которое может выбрать пользователь, в секундах. Сервер сокращает более длинный запрос до этого значения. По умолчанию и максимум 1800.
admin-settings-key-mail-send-allow-recall = Разрешить отзыв отправленных сообщений
admin-settings-help-mail-send-allow-recall = Если выключено, сообщения уходят сразу и их нельзя отозвать из Исходящих. Отложенные отправки не затрагиваются; их по-прежнему можно отменить до наступления времени.
admin-settings-key-mail-send-max-delayed-seconds = Горизонт планирования
admin-settings-help-mail-send-max-delayed-seconds = На сколько вперёд можно запланировать сообщение, в секундах. По умолчанию 7776000 (90 дней), максимум 31536000 (1 год). 0 отключает отложенную отправку.
admin-settings-key-mail-send-max-queued = Сообщений в очереди на ящик
admin-settings-help-mail-send-max-queued = Сколько сообщений один ящик может одновременно держать в Исходящих (окно отмены или отложенные). По умолчанию 200. Дальнейшие отправки отклоняются с понятным сообщением и никогда не теряются.
admin-attachments-sim-pick = Заполнить из файла (необязательно)
admin-attachments-mime-toggle = Показать распространённые типы MIME
admin-attachments-mime-hint = Выберите распространённый тип или введите любой другой тип/подтип.
admin-attachments-mime-invalid = Это не тип/подтип вроде text/plain. Он будет отправлен как введён.
admin-attachments-mime-group-documents = Документы
admin-attachments-mime-group-images = Изображения
admin-attachments-mime-group-archives = Архивы
admin-attachments-mime-group-executables = Исполняемые файлы и скрипты
admin-attachments-mime-group-media = Аудио и видео
admin-attachments-mime-group-email = Эл. почта и календарь
admin-attachments-mime-group-other = Другое
admin-settings-key-cookies-enabled = Управление согласием на cookie
admin-settings-help-cookies-enabled = Показывать посетителям элемент управления согласием на cookie. Выключение сохраняет поведение приложения без этого элемента.
admin-settings-key-cookies-mode = Модель согласия
admin-settings-help-cookies-mode = optIn: ничего необязательного не сохраняется, пока посетитель не примет. optOut: действуют значения категорий по умолчанию, пока посетитель не отклонит. noticeOnly: посетитель информируется без выбора.
admin-settings-key-cookies-region-mode = Модель согласия в строгих регионах
admin-settings-help-cookies-region-mode = Модель, используемая для посетителей из строгих регионов. same означает, что и там действует модель согласия выше.
admin-settings-key-cookies-strict-regions = Строгие регионы
admin-settings-help-cookies-strict-regions = Коды стран ISO 3166-1, использующие модель согласия для строгих регионов. По умолчанию государства ЕС и ЕЭЗ, Великобритания и Швейцария.
admin-settings-key-cookies-region-header = Заголовок страны
admin-settings-help-cookies-region-header = Имя заголовка запроса, устанавливаемого прокси или CDN, в котором передаётся код страны посетителя.
admin-settings-key-cookies-unknown-region = Неизвестная страна
admin-settings-help-cookies-unknown-region = Какая модель применяется, когда страну определить нельзя: модель строгих регионов или обычная.
admin-settings-key-cookies-functional-enabled = Предлагать функциональные cookie
admin-settings-help-cookies-functional-enabled = Предлагать в элементе согласия категорию настроек и функций (тема, язык, запоминание раскладки).
admin-settings-key-cookies-functional-default = Функциональные по умолчанию
admin-settings-help-cookies-functional-default = Начальное состояние функциональной категории, когда модель согласия допускает значения по умолчанию.
admin-settings-key-cookies-analytics-enabled = Предлагать аналитические cookie
admin-settings-help-cookies-analytics-enabled = Предлагать в элементе согласия аналитическую категорию.
admin-settings-key-cookies-analytics-default = Аналитические по умолчанию
admin-settings-help-cookies-analytics-default = Начальное состояние аналитической категории, когда модель согласия допускает значения по умолчанию.
admin-settings-key-cookies-marketing-enabled = Предлагать маркетинговые cookie
admin-settings-help-cookies-marketing-enabled = Предлагать в элементе согласия маркетинговую категорию.
admin-settings-key-cookies-marketing-default = Маркетинговые по умолчанию
admin-settings-help-cookies-marketing-default = Начальное состояние маркетинговой категории, когда модель согласия допускает значения по умолчанию.
admin-settings-key-cookies-layout = Макет
admin-settings-help-cookies-layout = banner — полоса во всю ширину страницы, card — плавающая карточка, modal — диалог, который удерживает страницу, пока посетитель не сделает выбор.
admin-settings-key-cookies-position = Положение
admin-settings-help-cookies-position = Где на странице появляется элемент согласия.
admin-settings-key-cookies-reject-prominent = Отклонение так же заметно, как принятие
admin-settings-help-cookies-reject-prominent = Показывать «Отклонить все» того же размера и стиля, что и «Принять все».
admin-settings-key-cookies-policy-url = URL политики cookie
admin-settings-help-cookies-policy-url = Ссылка на вашу политику cookie, показываемая в элементе согласия.
admin-settings-key-cookies-privacy-url = URL политики конфиденциальности
admin-settings-help-cookies-privacy-url = Ссылка на вашу политику конфиденциальности, показываемая в элементе согласия.
admin-settings-key-cookies-policy-version = Версия политики
admin-settings-help-cookies-policy-version = Увеличьте это число, когда меняется использование cookie, чтобы посетителей спросили снова.
admin-settings-key-cookies-lifetime-days = Срок действия согласия
admin-settings-help-cookies-lifetime-days = Число дней, в течение которых решение остаётся действительным, прежде чем посетителя спросят снова.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-cookies-reprompt-on-version = Спрашивать снова при новой версии политики
admin-settings-help-cookies-reprompt-on-version = Спрашивать посетителей снова, когда версия политики выше той, при которой они приняли решение.
admin-settings-key-cookies-honour-gpc = Учитывать Global Privacy Control
admin-settings-help-cookies-honour-gpc = Считать сигнал Global Privacy Control от браузера отказом от аналитики и маркетинга.
admin-settings-key-cookies-honour-dnt = Учитывать Do Not Track
admin-settings-help-cookies-honour-dnt = Считать сигнал Do Not Track от браузера отказом от аналитики и маркетинга.
admin-settings-key-cookies-log-consent = Вести журнал согласий
admin-settings-help-cookies-log-consent = Хранить на сервере запись каждого решения о согласии для статистики и собственного учёта.
admin-settings-key-cookies-log-retention-days = Хранение журнала согласий
admin-settings-help-cookies-log-retention-days = Число дней, в течение которых строки журнала согласий хранятся до удаления.
admin-settings-key-cookies-scan-interval-hours = Интервал сканирования хранилища
admin-settings-help-cookies-scan-interval-hours = Часы между автоматическими сканированиями cookie и хранилища браузера. 0 сканирует только по запросу.
admin-settings-key-cookies-unclassified = Неклассифицированное хранилище
admin-settings-help-cookies-unclassified = report оставляет хранилище, которое ничто не классифицирует, нетронутым и выводит его на проверку. Остальные значения относят его к соответствующей категории, пока вы его не классифицируете.
admin-settings-key-cookies-text-title = Заголовок элемента согласия
admin-settings-help-cookies-text-title = Заголовок элемента согласия для каждого языка. Пусто — встроенный текст.
admin-settings-key-cookies-text-body = Текст элемента согласия
admin-settings-help-cookies-text-body = Поясняющий текст элемента согласия для каждого языка. Пусто — встроенный текст.
admin-settings-key-cookies-label-necessary = Название категории необходимых
admin-settings-help-cookies-label-necessary = Название категории необходимых для каждого языка. Пусто — встроенное название.
admin-settings-key-cookies-description-necessary = Описание категории необходимых
admin-settings-help-cookies-description-necessary = Описание категории необходимых для каждого языка. Пусто — встроенное описание.
admin-settings-key-cookies-label-functional = Название категории функциональных
admin-settings-help-cookies-label-functional = Название категории функциональных для каждого языка. Пусто — встроенное название.
admin-settings-key-cookies-description-functional = Описание категории функциональных
admin-settings-help-cookies-description-functional = Описание категории функциональных для каждого языка. Пусто — встроенное описание.
admin-settings-key-cookies-label-analytics = Название категории аналитических
admin-settings-help-cookies-label-analytics = Название категории аналитических для каждого языка. Пусто — встроенное название.
admin-settings-key-cookies-description-analytics = Описание категории аналитических
admin-settings-help-cookies-description-analytics = Описание категории аналитических для каждого языка. Пусто — встроенное описание.
admin-settings-key-cookies-label-marketing = Название категории маркетинговых
admin-settings-help-cookies-label-marketing = Название категории маркетинговых для каждого языка. Пусто — встроенное название.
admin-settings-key-cookies-description-marketing = Описание категории маркетинговых
admin-settings-help-cookies-description-marketing = Описание категории маркетинговых для каждого языка. Пусто — встроенное описание.
admin-settings-key-encryption-rotation-days = Интервал ротации ключа базы данных (дни)
admin-settings-help-encryption-rotation-days = Число дней после последней смены ключа, по истечении которых ключ базы данных отмечается как требующий ротации. 0 не задаёт срок. Сам ключ никогда не хранится в настройках.
admin-settings-key-encryption-warnings-enabled = Сообщать о находках по шифрованию
admin-settings-help-encryption-warnings-enabled = Показывать хранилища в открытом виде и просроченную ротацию ключа как находку в оценке развёртывания.
admin-settings-key-retention-sweeper-enabled = Очистка по срокам хранения
admin-settings-help-retention-sweeper-enabled = Запускать ежедневную очистку, удаляющую строки старше ограничений на этой странице. Выключение сохраняет все строки до повторного включения.
admin-settings-key-retention-sweeper-batch-rows = Строк в партии
admin-settings-help-retention-sweeper-batch-rows = Сколько строк удаляет один оператор удаления. Меньшие партии держат блокировки базы данных меньше времени.
admin-settings-key-retention-sweeper-max-batches = Партий на хранилище за запуск
admin-settings-help-retention-sweeper-max-batches = Наибольшее число операторов удаления, которое хранилище получает за запуск. Хранилище с большим числом старых строк продолжит в следующем запуске.
admin-settings-key-retention-sweeper-jitter-minutes = Разброс времени запуска (минуты)
admin-settings-help-retention-sweeper-jitter-minutes = Следующий запуск начинается через случайное число минут, до этого значения, после 24 часов, чтобы несколько серверов не очищали одновременно.
admin-settings-key-retention-sweeper-vacuum-after-rows = Освобождать место после (строк)
admin-settings-help-retention-sweeper-vacuum-after-rows = Если один запуск удалил не менее стольких строк, база данных уплотняется (VACUUM). 0 никогда не уплотняет после очистки; еженедельное задание vacuum отдельное.
admin-settings-key-retention-password-change-audit-days = Записи о смене пароля (дни)
admin-settings-help-retention-password-change-audit-days = Число дней хранения записи о смене пароля (аккаунт, способ, результат). 0 хранит их бессрочно.
admin-settings-key-retention-sso-login-audit-days = Записи о входах через единый вход (дни)
admin-settings-help-retention-sso-login-audit-days = Число дней хранения записи о входе через единый вход (провайдер, хеш субъекта, результат). 0 хранит их бессрочно.
admin-settings-key-retention-dlp-audit-days = Записи предотвращения утечек данных (дни)
admin-settings-help-retention-dlp-audit-days = Число дней хранения записи о срабатывании правила предотвращения утечек данных. 0 хранит их бессрочно.
admin-settings-key-retention-assist-audit-days = Записи доступа помощника (дни)
admin-settings-help-retention-assist-audit-days = Число дней хранения записи об использовании помощником возможности (субъект, возможность, сводка области). 0 хранит их бессрочно.
admin-settings-key-retention-alert-events-days = Записи сработавших оповещений (дни)
admin-settings-help-retention-alert-events-days = Число дней хранения записи о сработавшем оповещении администратора. 0 хранит их бессрочно.
admin-settings-key-retention-job-runs-days = История фоновых заданий (дни)
admin-settings-help-retention-job-runs-days = Число дней хранения завершённого запуска фонового задания. Выполняющийся запуск никогда не удаляется. 0 хранит историю бессрочно.
admin-settings-key-retention-outbox-days = Записи отправленных и отменённых отправок (дни)
admin-settings-help-retention-outbox-days = Число дней хранения записи об отправке доставленного или отменённого сообщения. Ожидающее, повторяемое, неудавшееся или неопределённое сообщение никогда не удаляется. 0 хранит их бессрочно.
admin-settings-key-retention-expired-credentials-grace-days = Истёкшие учётные данные (дни после истечения)
admin-settings-help-retention-expired-credentials-grace-days = Число дней хранения после истечения для запомненного устройства, задачи входа, токена входа, токена OAuth, nonce вебхука, счётчика сбоев источника или дескриптора ссылки на изображение. 0 удаляет их при следующем запуске.
admin-settings-key-retention-ip-addresses-days = Сокращать хранимые IP-адреса через (дни)
admin-settings-help-retention-ip-addresses-days = Через столько дней IP-адрес в истории аутентификации и записях аудита усекается или хешируется, а строка сохраняется. 0 сохраняет адреса как записаны.
admin-settings-key-retention-ip-addresses-mode = Способ сокращения IP-адресов
admin-settings-help-retention-ip-addresses-mode = Усечение сохраняет префикс IPv4 /24 или IPv6 /48. Хеш заменяет адрес коротким хешем с ключом, который всё ещё группирует один и тот же адрес.
admin-settings-key-retention-user-enabled = Пользователи могут задавать правила хранения
admin-settings-help-retention-user-enabled = Пользователи могут задавать собственные правила хранения для папок и почты. Если выключено, существующие пользовательские правила перестают выполняться; общие правила организации ниже продолжают работать.
admin-settings-key-retention-user-min-days = Наименьшее число дней до удаления почты правилом пользователя
admin-settings-help-retention-user-min-days = Правило пользователя, удаляющее почту, не может действовать на почту моложе этого значения. Меньшее значение отклоняется при сохранении правила и повышается при его выполнении.
admin-settings-key-retention-user-max-days = Наибольшее число дней ожидания правила пользователя перед удалением почты
admin-settings-help-retention-user-max-days = Правило пользователя, удаляющее почту, не может ждать дольше этого значения. 0 не задаёт верхнего предела. Большее значение отклоняется при сохранении правила и понижается при его выполнении.
admin-settings-key-retention-user-max-rules = Наибольшее число правил хранения на пользователя
admin-settings-help-retention-user-max-rules = Наибольшее число правил хранения, которое может иметь один пользователь.
admin-settings-key-retention-user-allow-permanent-delete = Правила пользователей могут удалять почту безвозвратно
admin-settings-help-retention-user-allow-permanent-delete = Если выключено, правило удаления всегда перемещает почту в Корзину, и перед окончательным удалением действует окно отмены.
admin-settings-key-retention-user-undo-days = Окно отмены для правил хранения (дни)
admin-settings-help-retention-user-undo-days = Число дней, в течение которых почта, перемещённая правилом хранения в Корзину, остаётся там до окончательного удаления. Пользователь может выбрать более короткое окно. 0 удаляет при следующем запуске.
admin-settings-key-retention-user-mandatory = Общие правила почты организации
admin-settings-help-retention-user-mandatory = Правила, которые получает каждый аккаунт, в виде записей роль-папки:дни, например trash:30 или junk:30. Почта в этой папке старше указанного числа дней удаляется окончательно. Пользователь не может изменять, приостанавливать их или исключать из них почту. Почта аккаунта под юридическим удержанием пропускается.
admin-settings-key-retention-user-protected-folders = Папки, которых правила хранения никогда не касаются
admin-settings-help-retention-user-protected-folders = Папки, с которыми не работает ни одно пользовательское правило, по роли (drafts, sent, inbox, archive) или по имени папки. Защищённая папка главнее общего правила организации для той же папки.
admin-settings-key-retention-user-templates = Шаблоны хранения, предлагаемые пользователям
admin-settings-help-retention-user-templates = Шаблоны, которые пользователи могут принять в Настройках, по идентификатору: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Выполнять правила пользователей для аккаунтов с ограничением
admin-settings-help-retention-user-run-for-restricted = Выполняются ли пользовательские правила хранения также для аккаунтов с ограничением обработки. Выключено — эти аккаунты пропускаются.
admin-settings-key-retention-user-interval-hours = Часов между запусками хранения пользователей
admin-settings-help-retention-user-interval-hours = Часы между плановыми запусками задания хранения пользователей для одного аккаунта.
admin-settings-key-retention-user-batch-messages = Сообщений на вызов хранения
admin-settings-help-retention-user-batch-messages = Сколько сообщений обрабатывает один вызов перемещения или удаления. Используется не более 200.
admin-settings-key-retention-user-max-per-run = Наибольшее число изменяемых сообщений на аккаунт за запуск
admin-settings-help-retention-user-max-per-run = Наибольшее число сообщений, которое задание хранения пользователей изменяет для одного аккаунта за один запуск. Остальное ждёт следующего запуска.
admin-settings-key-retention-user-pause-ms = Пауза между партиями хранения (мс)
admin-settings-help-retention-user-pause-ms = Пауза между партиями в миллисекундах, чтобы большая очистка не нагружала почтовый сервер.
admin-settings-key-retention-class-short-days = Верхний предел короткого класса (дни)
admin-settings-help-retention-class-short-days = Верхний предел для оперативных данных: сеансов, токенов, лимитов запросов, истории заданий и метрик. Применяется, когда ограничение класса включено.
admin-settings-key-retention-class-medium-days = Верхний предел среднего класса (дни)
admin-settings-help-retention-class-medium-days = Верхний предел для истории входов, истории защиты запросов и уведомлений безопасности. Не должен быть ниже короткого класса; меньшее значение повышается до него.
admin-settings-key-retention-class-long-days = Хранение длинного класса (дни)
admin-settings-help-retention-class-long-days = Число дней хранения длинного класса (аудит безопасности, действия администраторов, запросы, согласия, надгробия), включая архив. 0 хранит бессрочно, и консоль предупреждает. Не должно быть ниже среднего класса.
admin-settings-key-retention-class-enforce = Удерживать каждое хранилище в пределе его класса
admin-settings-help-retention-class-enforce = Если включено, собственное ограничение каждого хранилища ограничивается верхним пределом его класса, а 0 (хранить) считается верхним пределом. Если выключено, действуют ограничения отдельных хранилищ.
admin-settings-key-retention-archive-enabled = Архивировать устаревшие строки длинного класса
admin-settings-help-retention-archive-enabled = Переносить устаревшие строки длинного класса в запечатанные сжатые файлы архива вместо удаления.
admin-settings-key-retention-archive-directory = Каталог архива
admin-settings-help-retention-archive-directory = Каталог файлов архива, абсолютный или относительно рабочего каталога сервера. Сегменты родительского каталога (..) отклоняются.
admin-settings-key-retention-archive-batch-rows = Строк в файле архива
admin-settings-help-retention-archive-batch-rows = Число строк, записываемых в один файл архива.
admin-settings-key-retention-archive-max-segments = Файлов архива на хранилище за запуск
admin-settings-help-retention-archive-max-segments = Сколько файлов архива один запуск может записать для каждого хранилища. Остальное ждёт следующего запуска.
admin-settings-key-retention-archive-key-generation = Поколение ключа запечатывания архива
admin-settings-help-retention-archive-key-generation = Поколение ключа, запечатывающего новые файлы архива. Увеличьте его для ротации ключа; прежние файлы по-прежнему проверяются своим поколением.
admin-settings-key-retention-archive-verify-days = Дней между проверками архива
admin-settings-help-retention-archive-verify-days = Число дней между проверками целостности файлов архива.
admin-settings-key-retention-archive-legal-hold = Юридическое удержание архива
admin-settings-help-retention-archive-legal-hold = Сохранять каждый файл архива независимо от ограничения длинного класса.
admin-settings-key-logging-file-enabled = Записывать файлы журнала
admin-settings-help-logging-file-enabled = Также записывать журнал сервера в ротируемые файлы. Выключено — журналирование только в стандартный вывод, что обычно и использует контейнер.
admin-settings-key-logging-file-directory = Каталог журналов
admin-settings-help-logging-file-directory = Каталог файлов журнала, абсолютный или относительно рабочего каталога сервера. Сегменты родительского каталога (..) отклоняются.
admin-settings-key-logging-file-max-size-mb = Ротация по размеру (МБ)
admin-settings-help-logging-file-max-size-mb = Текущий файл журнала закрывается и начинается новый, когда он достигает этого размера.
admin-settings-key-logging-file-rotate-hours = Ротация по возрасту (часы)
admin-settings-help-logging-file-rotate-hours = Текущий файл журнала ротируется, когда ему исполняется столько часов. 0 ротирует только по размеру.
admin-settings-key-logging-file-max-age-days = Хранить ротированные файлы (дни)
admin-settings-help-logging-file-max-age-days = Ротированные файлы журнала старше этого значения удаляются.
admin-settings-key-logging-file-max-files = Наибольшее число ротированных файлов
admin-settings-help-logging-file-max-files = Самые старые ротированные файлы сверх этого числа удаляются.
admin-settings-key-logging-file-max-total-mb = Наибольший общий размер (МБ)
admin-settings-help-logging-file-max-total-mb = Самые старые ротированные файлы удаляются, пока каталог журналов не станет меньше этого размера.
admin-settings-key-logging-file-compress = Сжимать ротированные файлы
admin-settings-help-logging-file-compress = Ротированные файлы журнала сжимаются gzip.
admin-settings-key-logging-level = Уровень журнала
admin-settings-help-logging-level = Уровень для компонентов, у которых ниже нет собственного уровня. Переменная окружения MW_LOG задаёт уровень при запуске, пока этот параметр не прочитан.
admin-settings-key-logging-component-levels = Уровни журнала компонентов
admin-settings-help-logging-component-levels = Одна запись компонент=уровень в строке, например mw_imap=debug. Уровни: error, warn, info, debug и trace.
admin-settings-key-logging-redaction-addresses = Адреса эл. почты в журналах
admin-settings-help-logging-redaction-addresses = Full оставляет адреса как записаны. Truncated сохраняет первую букву и домен. Hashed заменяет локальную часть коротким хешем. None удаляет адрес.
admin-settings-key-logging-redaction-ips = IP-адреса в журналах
admin-settings-help-logging-redaction-ips = Full оставляет адреса как записаны. Truncated сохраняет префикс IPv4 /24 или IPv6 /48. Hashed заменяет адрес коротким хешем. None удаляет его.
admin-settings-key-logging-redaction-tokens = Секреты в журналах
admin-settings-help-logging-redaction-tokens = Вырезает из строк журнала токены bearer, ключи API, пароли в URL и подобные секреты.
admin-settings-key-retention-sessions-idle-days = Неиспользуемые сеансы (дни)
admin-settings-help-retention-sessions-idle-days = Сколько дней запись сеанса браузера или нативного клиента, с её адресом и user-agent, может оставаться неиспользуемой до удаления. Сначала действуют тайм-ауты сеансов; это лишь удаляет оставшиеся строки. 0 сохраняет их.
admin-settings-key-policy-public-pages = Публичные страницы политик
admin-settings-help-policy-public-pages = Отдавать сгенерированную политику конфиденциальности по адресу /privacy и политику cookie по адресу /cookies-policy без входа.
admin-settings-key-policy-auto-version = Автоматические версии политик
admin-settings-help-policy-auto-version = Записывать новую версию политики при изменении настройки, которую описывает политика. Выключено: изменения ждут публикации.
admin-settings-key-policy-reconsent-on-change = Снова запрашивать согласие на cookie при новой версии политики
admin-settings-help-policy-reconsent-on-change = Повышать версию политики cookie при записи новой версии политики, чтобы посетителей спросили снова (нужно cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Название оператора данных
admin-settings-help-policy-controller-name = Название организации, определяющей цели и способы обработки данных, показываемое в политике конфиденциальности.
admin-settings-key-policy-controller-address = Адрес оператора данных
admin-settings-help-policy-controller-address = Почтовый адрес оператора данных, показываемый в политике конфиденциальности.
admin-settings-key-policy-legal-bases = Правовые основания
admin-settings-help-policy-legal-bases = Правовые основания, на которые вы опираетесь, своими словами, показываемые в политике конфиденциальности.
admin-settings-key-policy-hosting-provider = Хостинг-провайдер
admin-settings-help-policy-hosting-provider = Кто и где размещает данные, показывается в политике конфиденциальности.
admin-settings-key-policy-hosting-transfers = Передача за пределы страны хостинга
admin-settings-help-policy-hosting-transfers = Любые передачи данных в другие страны и применяемые гарантии, показываются в политике конфиденциальности.
admin-settings-key-policy-supervisory-authority = Надзорный орган
admin-settings-help-policy-supervisory-authority = Орган, в который пользователи могут подать жалобу, показывается в политике конфиденциальности.
admin-settings-key-dpia-suggest = Предлагать оценки воздействия
admin-settings-help-dpia-suggest = Показывать предлагаемую оценку воздействия, когда включены помощник, отслеживание открытий или аналитика, а оценки нет.
admin-settings-key-dpia-review-months = Интервал пересмотра оценки воздействия
admin-settings-help-dpia-review-months = Число месяцев до даты пересмотра, задаваемой при утверждении оценки.
admin-settings-key-dpia-reminder-days = Окно напоминания об оценке воздействия
admin-settings-help-dpia-reminder-days = За сколько дней до даты пересмотра оценка выводится как требующая пересмотра.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-restriction-enabled = Ограничения обработки
admin-settings-help-restriction-enabled = Может ли администратор или одобренный запрос ограничить обработку данных человека.
admin-settings-key-restriction-auto-apply-on-approval = Ограничивать при одобрении запроса
admin-settings-help-restriction-auto-apply-on-approval = Применять ограничение сразу после одобрения запроса на ограничение. Выключено: администратор применяет его в разделе Соответствие > Ограничения.
admin-settings-key-restriction-mail-delivery = Новая почта при ограничении
admin-settings-help-restriction-mail-delivery = Что администратор выбрал для почты, приходящей при действующем ограничении: сохранять, ставить в очередь или отклонять. Этот сервер не принимает почту; значение записывается и показывается, а действует по нему почтовый сервер.
admin-settings-key-restriction-default-expiry-days = Срок ограничения по умолчанию (дни)
admin-settings-help-restriction-default-expiry-days = Число дней, через которое новое ограничение заканчивается само. 0 сохраняет его, пока кто-то не снимет.
admin-settings-key-restriction-notify-user = Уведомлять человека
admin-settings-help-restriction-notify-user = Отправлять человеку уведомление при применении или снятии ограничения.
admin-settings-key-restriction-scope-indexing = Приостанавливать индексирование поиска и эмбеддинги
admin-settings-help-restriction-scope-indexing = Приостановлены ли индексирование поиска и эмбеддинги по умолчанию в новом ограничении.
admin-settings-key-restriction-scope-assistant = Приостанавливать помощника
admin-settings-help-restriction-scope-assistant = Приостановлен ли помощник по умолчанию в новом ограничении.
admin-settings-key-restriction-scope-analytics = Приостанавливать аналитику и отслеживание открытий
admin-settings-help-restriction-scope-analytics = Приостановлены ли аналитика и отслеживание открытий по умолчанию в новом ограничении.
admin-settings-key-restriction-scope-classification = Приостанавливать правила и автоматическую сортировку
admin-settings-help-restriction-scope-classification = Приостановлены ли правила и автоматическая сортировка по умолчанию в новом ограничении.
admin-settings-key-restriction-scope-push = Приостанавливать push-уведомления
admin-settings-help-restriction-scope-push = Приостановлены ли push-уведомления по умолчанию в новом ограничении.
admin-settings-key-restriction-scope-third-party = Приостанавливать подключённые сервисы и плагины
admin-settings-help-restriction-scope-third-party = Приостановлены ли подключённые сервисы и плагины по умолчанию в новом ограничении.
admin-settings-key-consent-enabled = Согласие по целям обработки
admin-settings-help-consent-enabled = Показывать пользователям цели и варианты согласия и применять их во время работы.
admin-settings-key-consent-receipts-retention-days = Хранение квитанций о согласии (дни)
admin-settings-help-consent-receipts-retention-days = Число дней хранения квитанций о согласии. 0 хранит их до удаления данных человека.
admin-settings-key-consent-art22-notice = Уведомление об автоматизированной обработке
admin-settings-help-consent-art22-notice = Показывать пользователям простое уведомление о функциях помощника и поиска с переключателями для каждой функции.
admin-settings-key-consent-analytics-basis = Правовое основание: счётчики использования
admin-settings-help-consent-analytics-basis = Правовое основание, показываемое пользователям для счётчиков использования. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-analytics-version = Версия текста: счётчики использования
admin-settings-help-consent-analytics-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-analytics-text = Собственная формулировка: счётчики использования
admin-settings-help-consent-analytics-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-analytics-tag-basis = Правовое основание: скрипт провайдера аналитики
admin-settings-help-consent-analytics-tag-basis = Правовое основание, показываемое пользователям для скрипта провайдера аналитики. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-analytics-tag-version = Версия текста: скрипт провайдера аналитики
admin-settings-help-consent-analytics-tag-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-analytics-tag-text = Собственная формулировка: скрипт провайдера аналитики
admin-settings-help-consent-analytics-tag-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-assistant-basis = Правовое основание: помощник
admin-settings-help-consent-assistant-basis = Правовое основание, показываемое пользователям для помощника. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-assistant-version = Версия текста: помощник
admin-settings-help-consent-assistant-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-assistant-text = Собственная формулировка: помощник
admin-settings-help-consent-assistant-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-push-basis = Правовое основание: push-уведомления
admin-settings-help-consent-push-basis = Правовое основание, показываемое пользователям для push-уведомлений. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-push-version = Версия текста: push-уведомления
admin-settings-help-consent-push-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-push-text = Собственная формулировка: push-уведомления
admin-settings-help-consent-push-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-open-tracking-basis = Правовое основание: отслеживание открытий
admin-settings-help-consent-open-tracking-basis = Правовое основание, показываемое пользователям для отслеживания открытий. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-open-tracking-version = Версия текста: отслеживание открытий
admin-settings-help-consent-open-tracking-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-open-tracking-text = Собственная формулировка: отслеживание открытий
admin-settings-help-consent-open-tracking-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-remote-content-basis = Правовое основание: удалённые изображения
admin-settings-help-consent-remote-content-basis = Правовое основание, показываемое пользователям для удалённых изображений. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-remote-content-version = Версия текста: удалённые изображения
admin-settings-help-consent-remote-content-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-remote-content-text = Собственная формулировка: удалённые изображения
admin-settings-help-consent-remote-content-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-integrations-basis = Правовое основание: подключённые сервисы
admin-settings-help-consent-integrations-basis = Правовое основание, показываемое пользователям для подключённых сервисов. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-integrations-version = Версия текста: подключённые сервисы
admin-settings-help-consent-integrations-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-integrations-text = Собственная формулировка: подключённые сервисы
admin-settings-help-consent-integrations-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-consent-telemetry-basis = Правовое основание: измерения времени загрузки
admin-settings-help-consent-telemetry-basis = Правовое основание, показываемое пользователям для измерений времени загрузки. Только согласие заставляет функцию ждать «да»; остальные основания действуют, пока человек не отзовёт их. Какое основание подходит, решает администратор.
admin-settings-key-consent-telemetry-version = Версия текста: измерения времени загрузки
admin-settings-help-consent-telemetry-version = Увеличьте это число при изменении формулировки или обработки; людей, согласившихся с прежней версией, спросят снова.
admin-settings-key-consent-telemetry-text = Собственная формулировка: измерения времени загрузки
admin-settings-help-consent-telemetry-text = Ваш собственный текст для этой цели для каждого языка. Пусто — встроенная формулировка.
admin-settings-key-breach-enabled = Реестр нарушений
admin-settings-help-breach-enabled = Включает реестр и отсчёт времени на уведомление.
admin-settings-key-breach-notice-hours = Часов на уведомление органа
admin-settings-help-breach-notice-hours = Число часов с момента, когда стало известно о нарушении, до уведомления органа. Обычно 72.
admin-settings-key-breach-reminders-hours = Время напоминаний
admin-settings-help-breach-reminders-hours = Число часов до срока, при которых создаётся напоминание, например 6, 24 и 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Повтор после срока
admin-settings-help-breach-reminders-overdue-repeat-hours = Часы между напоминаниями после истечения срока. 0 напоминает один раз.
admin-settings-key-breach-authority-name = Надзорный орган
admin-settings-help-breach-authority-name = Название, указываемое в уведомлении органа.
admin-settings-key-breach-authority-url = Страница уведомлений органа
admin-settings-help-breach-authority-url = Адрес страницы уведомлений органа.
admin-settings-key-breach-authority-contact = Контакт органа
admin-settings-help-breach-authority-contact = Контактные данные органа.
admin-settings-key-breach-dpo-name = Имя контакта по защите данных
admin-settings-help-breach-dpo-name = Имя, указываемое как контактное лицо в уведомлениях.
admin-settings-key-breach-dpo-email = Эл. почта контакта по защите данных
admin-settings-help-breach-dpo-email = Адрес, указываемый как контакт в уведомлениях.
admin-settings-key-breach-dpo-phone = Телефон контакта по защите данных
admin-settings-help-breach-dpo-phone = Номер телефона, указываемый в уведомлениях.
admin-settings-key-breach-template-subject = Тема сообщения
admin-settings-help-breach-template-subject = Тема сообщения затронутым людям для каждого языка.
admin-settings-key-breach-template-body = Текст сообщения
admin-settings-help-breach-template-body = Текст сообщения затронутым людям для каждого языка.
admin-settings-key-breach-notify-audit = Записывать уведомления в журнал аудита
admin-settings-help-breach-notify-audit = Напоминания и обнаружения становятся событиями безопасности.
admin-settings-key-breach-notify-webhook-urls = Адреса вебхуков
admin-settings-help-breach-notify-webhook-urls = Адреса, которым сообщается о напоминаниях и сообщениях. Относитесь к ним как к секретам; каждый вызов подписан.
admin-settings-key-breach-notify-operator-addresses = Ящики операторов
admin-settings-help-breach-notify-operator-addresses = Адреса локальных ящиков, получающих напоминания во входящие.
admin-settings-key-breach-subjects-in-app = Уведомление в аккаунте человека
admin-settings-help-breach-subjects-in-app = Показывать сообщение внутри аккаунта человека.
admin-settings-key-breach-subjects-inbox = Сообщение в ящик человека
admin-settings-help-breach-subjects-inbox = Доставлять сообщение в собственный ящик человека.
admin-settings-key-breach-subjects-webhook = Также отправлять на вебхуки
admin-settings-help-breach-subjects-webhook = Отправлять текст сообщения и на адреса вебхуков.
admin-settings-key-breach-retention-years = Хранение реестра в годах
admin-settings-help-breach-retention-years = Число лет, в течение которых закрытый инцидент остаётся в реестре.
admin-settings-key-breach-auto-enabled = Автоматизации
admin-settings-help-breach-auto-enabled = Открывать черновики инцидентов по обнаружениям.
admin-settings-key-breach-auto-start-clock = Отсчёт для черновика
admin-settings-help-breach-auto-start-clock = Запускать отсчёт при открытии черновика или только после его подтверждения.
admin-settings-key-breach-auto-assignee = Назначать черновики
admin-settings-help-breach-auto-assignee = Кому назначается черновик.
admin-settings-key-breach-auto-escalate-after-hours = Эскалировать через часов
admin-settings-help-breach-auto-escalate-after-hours = Сколько часов черновик ждёт до эскалации. 0 никогда не эскалирует.
admin-settings-key-breach-auto-escalate-to = Эскалировать кому
admin-settings-help-breach-auto-escalate-to = Кому назначается эскалированный черновик.
admin-settings-key-breach-auto-window-minutes = Окно подсчёта в минутах
admin-settings-help-breach-auto-window-minutes = Число минут, за которые правило считает события.
admin-settings-key-breach-auto-theft-threshold = Оповещения о краже сеанса
admin-settings-help-breach-auto-theft-threshold = Число оповещений в окне, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spray или всплеск входов
admin-settings-help-breach-auto-credential-stuffing-threshold = Число событий в окне, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-mass-forward-threshold = Массовые правила пересылки
admin-settings-help-breach-auto-mass-forward-threshold = Число событий в окне, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-role-escalation-threshold = Изменения ролей администратора
admin-settings-help-breach-auto-role-escalation-threshold = Число изменений в окне, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-export-volume-threshold = Необычные экспорты
admin-settings-help-breach-auto-export-volume-threshold = Число экспортов в окне, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-audit-chain-threshold = Сбои цепочки аудита
admin-settings-help-breach-auto-audit-chain-threshold = Число сбоев, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-backup-anomaly-threshold = Сбои резервного копирования или восстановления
admin-settings-help-breach-auto-backup-anomaly-threshold = Число сбоев в окне, открывающее черновик. 0 отключает.
admin-settings-key-breach-auto-dsr-export-threshold = Много экспортов по запросам о данных
admin-settings-help-breach-auto-dsr-export-threshold = Число экспортов в окне, открывающее черновик. 0 отключает.
admin-settings-key-ratelimit-queue-ms = Наибольшее ожидание свободного места
admin-settings-help-ratelimit-queue-ms = Наибольшее время ожидания запросом свободного места (параллелизм JMAP, декодеры изображений, обработчики просмотра), прежде чем он будет отклонён с кодом 429 и Retry-After. Диапазон 0–30000 мс; по умолчанию 3000. Ноль отклоняет сразу.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Одновременных запросов JMAP на аккаунт
admin-settings-help-ratelimit-jmap-per-account-concurrent = Сколько запросов JMAP один аккаунт с выполненным входом может выполнять одновременно во всех вкладках и на всех устройствах. Не делится с другим аккаунтом или другим пользователем за тем же адресом. Диапазон 4–64; по умолчанию 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Одновременных запросов JMAP, все аккаунты
admin-settings-help-ratelimit-jmap-global-concurrent = Сколько запросов JMAP весь серверный процесс выполняет одновременно, для всех аккаунтов вместе. Запрос сверх этого ждёт время очереди. Диапазон 4–4096; по умолчанию 256.
admin-settings-key-ratelimit-session-burst = Отчётов о времени на сеанс подряд
admin-settings-help-ratelimit-session-burst = Сколько отчётов о времени клиента один сеанс может отправить подряд. Конечная точка привязана к сеансу, а не к адресу. Диапазон 1–1000; по умолчанию 60.
admin-settings-key-ratelimit-session-per-minute = Отчётов о времени восстанавливается в минуту
admin-settings-help-ratelimit-session-per-minute = Сколько отчётов о времени клиента в минуту сеанс восстанавливает после серии. Диапазон 1–600; по умолчанию 30.
admin-settings-key-ratelimit-anon-burst = Анонимных запросов согласия подряд
admin-settings-help-ratelimit-anon-burst = Сколько запросов один адрес может отправить подряд на анонимные маршруты согласия на cookie. Рассчитано на офис за одним адресом. Диапазон 1–1000; по умолчанию 60.
admin-settings-key-ratelimit-anon-per-minute = Анонимных запросов согласия восстанавливается в минуту
admin-settings-help-ratelimit-anon-per-minute = Сколько запросов в минуту адрес восстанавливает на анонимных маршрутах согласия на cookie. Диапазон 1–600; по умолчанию 30.
admin-settings-key-ratelimit-trusted-multiplier = Лимит аккаунта с известного источника (%)
admin-settings-help-ratelimit-trusted-multiplier = Процент лимита запросов аккаунта, который получает ящик с выполненным входом, когда запрос приходит с источника с зафиксированным успешным входом. 100 сохраняет базовый лимит. Никогда не ослабляет бюджеты анонимов, администратора и user-agent и действующую блокировку. Диапазон 100–1000; по умолчанию 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Щелчок в любом месте панели вложений читателя разворачивает или сворачивает ее. В выключенном состоянии работает только кнопка со стрелкой.

admin-settings-help-attachments-bar-download-all = Имеется ли на панели вложений кнопка «Загрузить все», которая сохраняет разрешенные вложения в один файл ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Когда для каждого вложения на панели отображается кнопка загрузки: при наведении или фокусе (по умолчанию), всегда или никогда. Загрузки по-прежнему подчиняются политике вложений и mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Включены ли изображения, показанные внутри текста сообщения, в «Загрузить все» ZIP. По умолчанию выключено.

admin-settings-help-cap-reading-delete-sender = Разрешить пользователям включать действие чтения, которое после подтверждения перемещает все соответствующие сообщения отправителя в корзину. По умолчанию отключено. Профиль функции также должен разрешать это.

admin-settings-help-compose-feature-family = Включите или выключите элемент управления композитором по идентификатору функции. По умолчанию включено; заблокируйте его, чтобы пользователи не могли его изменить.

admin-settings-help-compose-headers-allow-auto-submitted = Позволяет пользователям устанавливать автоматическую отправку (RFC 3834) для сообщения. По умолчанию отключено: другие системы не отвечают.

admin-settings-help-compose-headers-allow-custom = Позволяет пользователям добавлять свои собственные заголовки (до 20, имена проверяются, защищенные заголовки отклоняются).

admin-settings-help-compose-headers-allow-return-receipt-to = Позволяет пользователям запрашивать квитанции с помощью устаревшего заголовка Return-Receipt-To. По умолчанию выключено.

admin-settings-help-compose-headers-allow-threading-edit = Позволяет пользователям редактировать In-Reply-To и ссылки или начинать новую цепочку из ответа.

admin-settings-help-compose-headers-forced = Одна пара Имя=значение в каждой строке добавляется к каждому сообщению, отправленному из домена, и заменяет значение пользователя. Защищенные заголовки отклоняются.

admin-settings-help-compose-insert-image-resize-default = Размер, с которого начинается диалоговое окно «Картинка из файла»: исходный, большой (1600 пикселей) или средний (1024 пикселей).

admin-settings-help-compose-insert-max-inline-image-mb = Изображения большего размера не помещаются в тело сообщения; их все еще можно прикрепить. от 1 до 50.

admin-settings-help-compose-layout-default-font = Семейство шрифтов, размер и цвет, с которого начинается новое сообщение, например, Family|size|#color. Empty использует шрифт темы.

admin-settings-help-compose-reply-to-allow-override = Если этот параметр отключен, ответ для удостоверения используется в том виде, в каком он есть, и пользователи не могут редактировать или удалять его.

admin-settings-help-compose-reply-to-show = Показывает ли композитор строку «Ответить»: только если она есть в идентификаторе или исходном сообщении, всегда или никогда.

admin-settings-help-compose-sent-folder-fixed-id = Роль или идентификатор папки, используемый, когда правилом является «фиксированная папка».

admin-settings-help-compose-sent-folder-mode = Куда попадает копия отправленного сообщения: обычная папка «Отправленные», одна фиксированная папка, по домену получателя, по удостоверению или по правилам темы.

admin-settings-help-compose-sent-folder-per-identity = Одна пара идентификатор=папка в каждой строке. Используется, когда правило установлено «по идентификатору».

admin-settings-help-compose-sent-folder-rules = Одно правило в строке, domain:example.com=папка или subject:text=папка. Первый матч выигрывает.

admin-settings-help-compose-sent-folder-skip-copy = Никогда не сохраняйте копию отправленного сообщения. Администратор может принудительно включить или выключить эту функцию.

admin-settings-help-compose-styles-custom = Собственные композиторские стили пользователя как JSON, записанные в галерее стилей. Пользователи устанавливают его; здесь нечего настраивать.

admin-settings-help-compose-styles-default-set = Внешний вид галереи стилей композитора: по умолчанию, современный, классический или компактный.

admin-settings-help-compose-styles-max-custom = Большинство пользовательских стилей пользователь может сохранить в композиторе. 0 отключает создание стиля.

admin-settings-help-contacts-save-from-message-add-note = Добавьте пометку «Сохранено из сообщения <дата>» к контактам, сохраненным из сообщения. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-contacts-save-from-message-default-book = Идентификатор адресной книги, в которую попадают контакты, сохраненные из сообщения; «по умолчанию» использует адресную книгу по умолчанию. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-contacts-save-from-message-default-scope = Какие участники отмечаются первыми в диалоговом окне «Сохранить контакты»: только отправитель, отправитель и ответчик или все. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-contacts-save-from-message-enabled = Предложение Сохранить контакты из этого сообщения в меню сообщений. При выключении элемент скрыт. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-contacts-save-from-message-skip-automated = Оставьте адреса отсутствия ответа, уведомлений и отказов в диалоговом окне «Сохранить контакты», если они не отмечены галочкой вручную. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-contacts-save-sender-enabled = Предложите пункт «Сохранить отправителя как контакт» одним щелчком мыши в меню сообщений. При выключении элемент скрыт. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-contacts-save-sender-use-reply-to = Сохраните адрес для ответа вместо адреса «От», если отправителем является список рассылки или адрес уведомления. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-dsr-admin-step-up = Включено (по умолчанию): администратор, который утверждает запрос данных, запускает удаление или осуществляет массовое утверждение, должен сначала подтвердить это снова (пароль и второй фактор или ключ доступа). Подтверждение длится пять минут. Off удаляет дополнительную проверку; решение по-прежнему записывается в журнале запросов.

admin-settings-help-export-default-format = Формат, с которого начинается диалоговое окно экспорта.

admin-settings-help-export-default-scope = Область, с которой начинается диалоговое окно экспорта: сообщение, беседа, выделение или папка.

admin-settings-help-export-dialog-enabled = Экспорт в режиме просмотра сообщений и меню сообщений открывает диалоговое окно на месте. Если этот параметр отключен, эти меню загружают одно сообщение в формате .eml.

admin-settings-help-export-folder-enabled = Разрешить пользователям загружать целую папку сообщений в формате ZIP. Каждый экспорт проверяется.

admin-settings-help-export-folder-format = Форматы, которые предлагает экспорт папки: ZIP файлов .eml, ZIP, содержащий файл mbox, или оба.

admin-settings-help-export-folder-include-subfolders = Включает ли экспорт папки ее подпапки по умолчанию, которые сохраняются в виде структуры папок в ZIP.

admin-settings-help-export-folder-max-bytes = Максимальное количество байтов сообщений, которое может содержать одна экспортированная папка. Папка большего размера с указанным размером отклоняется.

admin-settings-help-export-folder-max-messages = Максимальное количество сообщений, которое может содержать одна экспортированная папка.

admin-settings-help-export-formats = Форматы, предлагаемые диалоговым окном экспорта: eml, mbox, txt, md, msg, docx, csv, json, pdf. Сервер отказывается от указанного в списке формата, созданного сервером.

admin-settings-help-export-message-download-enabled = Разрешить пользователям загружать сообщение в формате .eml, только его заголовки или в виде файла mbox.

admin-settings-help-folders-hide-empty = Скрыть пустые папки на боковой панели. Пользователи могут изменить значение по умолчанию или переопределить его для отдельных папок, если этот параметр не заблокирован. Ни одна почта не удаляется.

admin-settings-help-folders-pinned-hide-from-tree = Закрепленные папки отображаются в группе «Закрепленные» над деревом папок, а также сохраняют свое место в дереве. Вкл. скрывает закрепленную папку, не имеющую подпапок в дереве, поэтому она отображается только в группе Закрепленные. Закрепленная папка с подпапками останется в дереве.

admin-settings-help-mail-badge-auth = Когда в заголовке считывателя отображается значок аутентификации DMARC: всегда, только когда это заметно или никогда.

admin-settings-help-mail-badge-density = Размер бейджей в шапке считывателя: компактный или удобный.

admin-settings-help-mail-badge-folder = Когда в заголовке программы чтения отображается значок папки: всегда, только когда это заметно или никогда.

admin-settings-help-mail-badge-headers = Отображение значка в программе чтения для заголовков сообщения Expires, Reply-By, Sensitivity и Supersedes.

admin-settings-help-mail-badge-labels = Отображается ли в значках заголовка читателя значок с текстом или только значок.

admin-settings-help-mail-badge-list = Когда в заголовке читателя отображается значок списка рассылки: всегда, только когда это заметно или никогда. Ему нужен заголовок List-Id или List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Когда в заголовке программы чтения отображается очищенный значок HTML: всегда, только когда это заметно или никогда.

admin-settings-help-mail-badge-spam = Когда в заголовке программы чтения отображается значок оценки спама: всегда, только если заметно или никогда.

admin-settings-help-mail-brand-avatars-require-auth = никогда: показывать логотип всякий раз, когда домен совпадает. известный-сбой (по умолчанию): показывать его, если результат кэширования сообщения DKIM или DMARC не является сбоем; непроверенные сообщения сохраняют логотип. всегда: показывать его только после согласования DMARC с DKIM для собственного домена бренда. Переопределения пользователя и домена можно заблокировать.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Секунды, в течение которых тело должно удерживать фокус, прежде чем полный заголовок получателя и темы превратится в сводную информацию. Выберите 0, чтобы отключить складывание по времени, или 1–60 секунд. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-mail-compose-priority-default = Приоритет, с которым начинается новое сообщение в композиторе.

admin-settings-help-mail-compose-priority-enabled = Покажите переключатель «Нормальный», «Высокий» и «Низкий приоритет» в группе «Параметры композитора». Если этот параметр отключен, новая почта отправляется без приоритетных заголовков.

admin-settings-help-mail-expiry-hide-expired = Что делать с сообщением, срок действия которого истек: ничего, показать его затемненным или предложить заархивировать.

admin-settings-help-mail-list-keep = Отправители и списки рассылки всегда сохраняются, по одному на строку как sender:address или list:List-Id. Очистите этого отправителя, отмените подписку и правила хранения пропустите их.

admin-settings-help-mail-list-page-size = Сколько сообщений список получает каждый раз, когда пользователь прокручивает список до конца. По умолчанию 50, от 20 до 200. Пользователи выбирают свое собственное значение, если вы не заблокируете его.

admin-settings-help-mail-list-unsubscribe-mode = Как работает отмена подписки, когда список предлагает запрос в один клик: сначала спросите, отправьте его сразу или откройте только ссылку или черновик списка.

admin-settings-help-mail-message-list-group = Группируйте беседы по дате получения или отправки, отправителю, теме, важности или размеру. Пользователи могут отменить это, если оно не заблокировано.

admin-settings-help-mail-priority-marks = Какой приоритет отмечается в списке сообщений, разговорах и программе чтения: высокий и низкий, только высокий или нет. Пользователи выбирают в «Настройки» > «Папки и боковая панель», если они не заблокированы.

admin-settings-help-mail-priority-marks-style = Как выглядит отметка приоритета: значок, значок с надписью «Высокий» или «Низкий» или тонкая полоска. Значок и текст сохраняют смысл без цвета.

admin-settings-help-mail-priority-send-headers = Какие заголовки получает сообщение с высоким или низким приоритетом при отправке: любой из Важности, X-Priority и X-MSMail-Priority.

admin-settings-help-mail-priority-trust-sender-headers = Отображается ли приоритет, установленный отправителем: всегда, только если сообщение прошло согласованную проверку DMARC или никогда. Используйте параметр «Доверено или никогда» для отправителей, которые отмечают каждое сообщение с высоким приоритетом.

admin-settings-help-mail-reply-from-header-priority = Заголовки сообщений читаются в этом порядке, чтобы найти адрес, на который было отправлено сообщение, при выборе удостоверения для ответа. Имена: Кому-доставлено, Кому-оригиналу, Кому-конверту, Кому-пересланному, Кому-отправленному, Кому, Копия. Оставленный заголовок не читается.

admin-settings-help-mail-search-default-scope = Начать новый обычный поиск почты в текущей папке или во всех папках. Это контролирует начальное значение по умолчанию, в том числе при блокировке; пользователи по-прежнему могут выбрать другую область для конкретного поиска. Сохраненные и явно ограниченные запросы сохраняются.

admin-settings-help-metrics-dimensions-enabled = Храните сохраненную статистику, разделенную по типу клиента (веб) и классу устройства (настольный, мобильный). Выключите, чтобы хранить одну объединенную строку для каждого показателя и минуты.

admin-settings-help-metrics-persist-enabled = Сохраняйте статистику производительности (скорость прокси-сервера изображений, время загрузки браузера, количество ограничений скорости и защиты запросов) в базе данных при каждом интервале очистки и при остановке сервера, чтобы диаграммы сохраняли свою историю после перезапуска. Ничто не идентифицирует человека, адрес или сообщение. Выключите, чтобы они оставались только в памяти.

admin-settings-help-metrics-persist-flush-seconds = Как часто статистика производительности записывается в базу данных (от 5 до 3600 секунд). Сервер, убитый без предупреждения, теряет максимум столько же.

admin-settings-help-observability-client-capture-console = Также сообщайте о вызовах console.error, не более пяти минут на страницу. По умолчанию отключено, поскольку библиотеки регистрируют ожидаемые условия.

admin-settings-help-observability-client-capture-csp = Сообщайте о нарушениях Content-Security-Policy, обнаруженных браузером, используя только заблокированную директиву и источник. Включено по умолчанию.

admin-settings-help-observability-client-enabled = Собирайте отчеты об ошибках из веб-приложения (границы ошибок, необнаруженные ошибки, неудачные фрагменты, рабочие ошибки, нарушения CSP) с очищенными трассировками стека, которые отображаются в разделе «Наблюдаемость» > «Ошибки клиента». Включено по умолчанию. Отчеты не содержат содержания сообщений, адресов, токенов, имен вложений или запросов URL. Выключите, чтобы остановить сбор; Срок хранения существующих отчетов истекает.

admin-settings-help-observability-client-forward = Также отправляйте каждый очищенный отчет в приемник ошибок, настроенный с помощью MW_ERROR_FORWARD_URL. По умолчанию выключено. Без настроенного приемника ничего не отправляется.

admin-settings-help-observability-client-include-account-hash = Добавляйте односторонний хэш учетной записи с ключом к отчетам, отправленным из сеанса входа в систему, чтобы отчеты из одной учетной записи можно было отличить друг от друга, не называя ее. По умолчанию выключено. Хэш меняется при смене секретного ключа.

admin-settings-help-observability-client-max-per-session = Большинство отчетов об ошибках может отправлять одна загрузка страницы (от 1 до 1000). Повторение одной и той же ошибки в течение 60 секунд засчитывается один раз. По умолчанию 20.

admin-settings-help-observability-client-sample-rate = Доля отчетов об ошибках, отправляемых браузером, от 0 до 1. По умолчанию 1 (все). Загрузка каждой страницы по-прежнему останавливается на пределе для каждого сеанса.

admin-settings-help-observability-client-source-maps = Разрешить позиции пакетов в трассировках стека для исходных файлов, когда сборка отправляла скрытые исходные карты (собранные с MW_SOURCEMAP=hidden). Карты читаются только сервером и никогда не передаются браузерам. Включено по умолчанию; без карт показаны необработанные кадры.

admin-settings-help-privacy-error-reports = Отправляет ли браузер отчет об ошибке при сбое части приложения. Включено по умолчанию. Глобальная, доменная и пользовательская область: установите здесь значение по умолчанию или заблокируйте его; люди могут отключить его для себя в «Настройки» > «Конфиденциальность», если он не заблокирован.

admin-settings-help-privacy-image-cache-max-stale-secs = Через несколько секунд после обновления кэшированное изображение сохраняется, чтобы ответить, когда источник не может быть достигнут. Никогда не ниже окна повторной проверки. Диапазон 0–604800; по умолчанию 86400.

admin-settings-help-privacy-image-cache-stale-secs = Спустя несколько секунд после актуальности кэшированное изображение по-прежнему обслуживается сразу, пока оно проходит повторную проверку в фоновом режиме. Зеро выключает его. Диапазон 0–86400; по умолчанию 3600.

admin-settings-help-privacy-image-resize-enabled = Уменьшите увеличенные удаленные изображения до ширины, которую запрашивает считыватель (320, 640, 960, 1280 или 1600 пикселей). Анимированные изображения подаются в немасштабированном виде. По умолчанию выключено.

admin-settings-help-privacy-image-resize-format = Автоматически выбирает лучший кодировщик в сборке — PNG. По умолчанию авто.

admin-settings-help-privacy-image-resize-max-pixels = Изображения с большим количеством декодированных пикселей подаются в немасштабированном виде; размер считывается из заголовка перед любым декодированием. Диапазон 100000–64000000; по умолчанию 4000000.

admin-settings-help-privacy-image-resize-max-width = Самое широкое изображение, которое можно получить при изменении размера. Диапазон 320–1600; по умолчанию 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Как в «Настройки» > «Безопасность» > «Сеансы» именуется учетная запись, из которой происходит сеанс доступа к связанному почтовому ящику: полный адрес, адрес с замаскированной первой частью имени или ничего. Значение отображается только владельцу сеансов, но не другим пользователям. По умолчанию: полный адрес.

admin-settings-help-properties-folder-enabled = Предлагайте свойства в меню папок и на ярлыке. Если параметр выключен, диалоговое окно недоступно.

admin-settings-help-properties-folder-notes-enabled = Разрешите описание или примечание для каждой папки.

admin-settings-help-properties-folder-notes-max-length = Максимальное количество символов, которое может содержать заметка к папке.

admin-settings-help-properties-folder-overrides-allowed = Какие настройки пользователь может переопределить для одной папки. Переопределение, не указанное в списке, скрыто, и любое сохраненное значение игнорируется.

admin-settings-help-properties-folder-sections = Разделы диалога свойств папки: общие, удаленные, размер, автономный, переопределения, заметки, действия. Раздел, которого нет в списке, скрыт.

admin-settings-help-properties-folder-size-compute = Когда определяется размер папки на сервере: никогда, при открытии диалога или в фоновом режиме. Подсчет большого почтового ящика требует от сервера сканирования.

admin-settings-help-properties-headers-raw = Разрешить в диалоговом окне «Свойства сообщения» отображать необработанные заголовки. Если этот параметр отключен, отображается только проанализированная сводка.

admin-settings-help-properties-message-enabled = Предлагайте свойства в меню сообщений и на ярлыке. Если параметр выключен, диалоговое окно недоступно.

admin-settings-help-properties-message-permanent-delete = Может ли диалоговое окно сообщения удалить сообщение навсегда: разрешить, спросить дважды или запретить. Сообщение, находящееся на законном хранении, никогда не удаляется. Удаление таким способом невозможно отменить.

admin-settings-help-properties-message-sections = Разделы диалога свойств сообщения: сводка, оценка, заголовки, вложения, действия. Раздел, которого нет в списке, скрыт.

admin-settings-help-properties-offline-clear-enabled = Разрешить пользователям очищать автономный кэш отдельной папки из диалогового окна свойств.

admin-settings-help-properties-remote-location-visibility = Какую часть расположения на стороне сервера видят пользователи: полное (хост и путь), замаскированный хост (хост частично скрыт), только путь (путь к почтовому ящику без хоста) или скрытый.

admin-settings-help-properties-scoring-show = Покажите оценку спама, результаты аутентификации и сигналы безопасности в диалоговом окне свойств сообщения.

admin-settings-help-properties-shortcut-enabled = Откройте диалоговое окно «Свойства» с клавиатуры для выбранной папки или выбранного сообщения.

admin-settings-help-properties-shortcut-key = Комбинация клавиш, открывающая свойства; Alt+Ввод по умолчанию.

admin-settings-help-reader-actions-archive = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-block-sender = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-colors = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-conversation = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-create-filter = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-delete = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-delete-sender = Скрыто по умолчанию. Выберите панель инструментов или меню «Дополнительно», чтобы включить эту функцию, если это разрешено возможностью удаления отправителя и профилем функции. Пользователи могут переопределить размещение, если оно не заблокировано.

admin-settings-help-reader-actions-edit-draft = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-export = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-finish-followup = Выберите панель инструментов, меню «Дополнительно» или «Скрыть», чтобы завершить помеченные сообщения и последующие напоминания.

admin-settings-help-reader-actions-flag = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-focus-focused = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-focus-other = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-forward = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-headers = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-links = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-move = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-not-spam = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-open-draft = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-pin = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-print = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-properties = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-read = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-reply = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-reply-all = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-save-contacts = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-save-sender = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-search = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-security = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-spam = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-sweep = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-view-source = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-reader-actions-zoom = Выберите панель инструментов, меню «Дополнительно» или «Скрыть». Ограничения функций, разрешения и контекст сообщения по-прежнему применяются. Пользователи могут выбирать, если это не заблокировано.

admin-settings-help-retention-client-errors-event-days = Дней, в течение которых сохраняется отдельное возникновение ошибки браузера (трассировка стека и класс браузера). По умолчанию 30. Группа хранится дольше, см. следующую настройку.

admin-settings-help-retention-client-errors-group-days = Дни, в течение которых группа ошибок браузера (имя, сообщение, количество) сохраняется после ее последнего просмотра. По умолчанию 180. Никогда не короче срока хранения вхождения.

admin-settings-help-retention-metrics-day-buckets-days = Ежедневная статистика производительности сохраняется до ее удаления. 0 сохраняет их без ограничений, что позволяет таблице медленно расти, пока работает сервер.

admin-settings-help-retention-metrics-hour-buckets-days = Статистика почасовой производительности за дни сохраняется до того, как она будет объединена в дневные строки (не менее 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Статистика производительности за одну минуту сохраняется до того, как она будет объединена в почасовые строки (не менее 1).

admin-settings-help-retention-user-allow-strip-attachments = Могут ли пользователи создавать правила, удаляющие большие вложения из старых писем. По умолчанию выключено. Если этот параметр отключен, действие скрыто в редакторе правил, правило, использующее его, отклоняется при сохранении, а сохраненные ранее правила пропускаются при выполнении задания.

admin-settings-help-retention-user-calendar-enabled = Могут ли пользователи выбирать календари, события которых удаляются через определенное количество дней после их завершения. По умолчанию выключено. Если параметр отключен, параметр скрыт в настройках, выбор отклоняется при сохранении, а сохраненные ранее варианты не запускаются.

admin-settings-help-retention-user-calendar-max-days = В большинстве дней пользователь может ждать, прежде чем завершенные события будут удалены. Большее значение отклоняется при сохранении и уменьшается при запуске.

admin-settings-help-retention-user-calendar-min-days = Наименьшее количество дней после завершения события, прежде чем правило пользователя сможет его удалить. Меньшее значение отклоняется при сохранении и увеличивается при запуске.

admin-settings-help-retention-user-notes-enabled = Могут ли пользователи удалять заметки, которые не менялись в течение определенного количества дней. По умолчанию выключено. Удаленные заметки можно восстановить до тех пор, пока не закончится окно отмены правил хранения.

admin-settings-help-retention-user-notes-max-days = В большинстве случаев пользователь может ждать, прежде чем нетронутые заметки будут удалены. Большее значение отклоняется при сохранении и уменьшается при запуске.

admin-settings-help-retention-user-notes-min-days = Заметка должна оставаться нетронутой в течение наименьшего количества дней, прежде чем правило пользователя сможет ее удалить. Меньшее значение отклоняется при сохранении и увеличивается при запуске.

admin-settings-help-retention-user-strip-keep-inline-images = Сохраняются ли изображения, показанные внутри тела сообщения, при удалении вложений.

admin-settings-help-retention-user-strip-min-age-days = Сообщение должно быть устаревшим наименьшее количество дней, прежде чем правило сможет удалить вложения. Меньшее значение отклоняется при сохранении правила и возникает при его запуске.

admin-settings-help-retention-user-strip-min-attachment-kb = Удалены только вложения хотя бы этого размера. Вложения меньшего размера остаются с сообщением.

admin-settings-help-retention-user-strip-undo-days = Сколько дней исходное сообщение остается в корзине после удаления вложений. -1 следует за окном отмены правил хранения. 0 удаляет оригинал сразу.

admin-settings-help-server-shutdown-grace-seconds = Как долго остановленный сервер позволяет завершить текущие запросы до закрытия соединений (в секундах). Переменная MW_HTTP_SHUTDOWN_TIMEOUT_MS переопределяет ее. Применяется при следующем запуске.

admin-settings-help-session-last-activity-flush-seconds = Время последнего посещения сеанса записывается не чаще, чем в секундах. Зеро пишет его при каждом запросе. Это не изменяет холостые или абсолютные пределы. Применяется при следующем запуске.

admin-settings-help-session-persist-across-restarts = Если этот параметр включен, сеансы почтового ящика, приложения и консоли остаются действительными после перезапуска сервера. Если этот параметр отключен, каждый сеанс завершается при запуске сервера. Применяется при следующем запуске.

admin-settings-key-attachments-bar-click-to-toggle = Нажмите на панель вложений, чтобы переключить

admin-settings-key-attachments-bar-download-all = Кнопка «Загрузить все»

admin-settings-key-attachments-bar-show-download-buttons = Кнопки загрузки вложений

admin-settings-key-attachments-bar-zip-inline-images = Встроенные изображения в разделе «Загрузить все» ZIP

admin-settings-key-cap-reading-delete-sender = Разрешить удаление всей почты от отправителя

admin-settings-key-compose-feature-family = Возможности композитора

admin-settings-key-compose-headers-allow-auto-submitted = Разрешить автоматическую отправку

admin-settings-key-compose-headers-allow-custom = Разрешить пользовательские заголовки

admin-settings-key-compose-headers-allow-return-receipt-to = Разрешить возврат квитанции

admin-settings-key-compose-headers-allow-threading-edit = Разрешить редактирование заголовков тредов

admin-settings-key-compose-headers-forced = Принудительные заголовки

admin-settings-key-compose-insert-image-resize-default = Предустановленный размер изображения

admin-settings-key-compose-insert-max-inline-image-mb = Самое большое встроенное изображение (МБ)

admin-settings-key-compose-layout-default-font = Шрифт сообщения по умолчанию

admin-settings-key-compose-reply-to-allow-override = Разрешить изменение поля ответа

admin-settings-key-compose-reply-to-show = Строка «Ответить»

admin-settings-key-compose-sent-folder-fixed-id = Отправленная копия: фиксированная папка

admin-settings-key-compose-sent-folder-mode = Отправленная копия: правило папки

admin-settings-key-compose-sent-folder-per-identity = Отправленная копия: папка для каждого удостоверения.

admin-settings-key-compose-sent-folder-rules = Отправленная копия: правила

admin-settings-key-compose-sent-folder-skip-copy = Отправленная копия: не сохранять

admin-settings-key-compose-styles-custom = Пользовательские стили

admin-settings-key-compose-styles-default-set = Набор стилей

admin-settings-key-compose-styles-max-custom = Пользовательские стили для каждого пользователя

admin-settings-key-contacts-save-from-message-add-note = Примечание о контактах, сохраненных из сообщения

admin-settings-key-contacts-save-from-message-default-book = Адресная книга для контактов, сохраненных из сообщения

admin-settings-key-contacts-save-from-message-default-scope = Контакты отмечаются первыми при сохранении из сообщения

admin-settings-key-contacts-save-from-message-enabled = Сохранение контактов из сообщения

admin-settings-key-contacts-save-from-message-skip-automated = Пропускать автоматических отправителей при сохранении контактов

admin-settings-key-contacts-save-sender-enabled = Сохранить отправителя как контакт

admin-settings-key-contacts-save-sender-use-reply-to = Сохранить ответ для отправителей списка

admin-settings-key-dsr-admin-step-up = Подтвердите еще раз, прежде чем одобрить

admin-settings-key-export-default-format = Формат диалогового окна экспорта по умолчанию

admin-settings-key-export-default-scope = Область действия диалогового окна экспорта по умолчанию

admin-settings-key-export-dialog-enabled = Диалоговое окно экспорта

admin-settings-key-export-folder-enabled = Экспорт папок

admin-settings-key-export-folder-format = Формат экспорта папки

admin-settings-key-export-folder-include-subfolders = Включить подпапки в экспорт

admin-settings-key-export-folder-max-bytes = Ограничение размера экспорта папки

admin-settings-key-export-folder-max-messages = Ограничение количества сообщений экспорта папок

admin-settings-key-export-formats = Экспорт форматов диалогов

admin-settings-key-export-message-download-enabled = Загрузка сообщения

admin-settings-key-folders-hide-empty = Скрыть пустые папки

admin-settings-key-folders-pinned-hide-from-tree = Скрыть закрепленные папки из дерева

admin-settings-key-mail-badge-auth = Значок аутентификации

admin-settings-key-mail-badge-density = Размер значка читателя

admin-settings-key-mail-badge-folder = Значок папки

admin-settings-key-mail-badge-headers = Значки заголовка

admin-settings-key-mail-badge-labels = Этикетки для значков читателей

admin-settings-key-mail-badge-list = Значок списка рассылки

admin-settings-key-mail-badge-sanitized = Продезинфицированный значок HTML

admin-settings-key-mail-badge-spam = Значок оценки спама

admin-settings-key-mail-brand-avatars-require-auth = Для логотипов брендов необходима аутентификация

admin-settings-key-mail-compose-header-auto-collapse-seconds = Задержка автоматического свертывания заголовка Composer

admin-settings-key-mail-compose-priority-default = Приоритет сообщения по умолчанию

admin-settings-key-mail-compose-priority-enabled = Селектор приоритета в композиторе

admin-settings-key-mail-expiry-hide-expired = Просроченные сообщения

admin-settings-key-mail-list-keep = Всегда держите

admin-settings-key-mail-list-page-size = Размер страницы списка сообщений

admin-settings-key-mail-list-unsubscribe-mode = Список рассылки Отписаться

admin-settings-key-mail-message-list-group = Группировка сообщений

admin-settings-key-mail-priority-marks = Знаки приоритета

admin-settings-key-mail-priority-marks-style = Стиль знака приоритета

admin-settings-key-mail-priority-send-headers = Приоритетные заголовки, записываемые при отправке

admin-settings-key-mail-priority-trust-sender-headers = Доверять приоритет, установленный отправителем

admin-settings-key-mail-reply-from-header-priority = Заголовки, которые выбирают идентификатор ответа

admin-settings-key-mail-search-default-scope = Область поиска почты по умолчанию

admin-settings-key-metrics-dimensions-enabled = Разделение статистики по типу клиента и классу устройства

admin-settings-key-metrics-persist-enabled = Сохраняйте статистику между перезапусками

admin-settings-key-metrics-persist-flush-seconds = Интервал сохранения статистики (секунды)

admin-settings-key-observability-client-capture-console = Сообщить об ошибках консоли

admin-settings-key-observability-client-capture-csp = Сообщить о нарушениях CSP

admin-settings-key-observability-client-enabled = Отчеты об ошибках браузера

admin-settings-key-observability-client-forward = Пересылать отчеты об ошибках

admin-settings-key-observability-client-include-account-hash = Добавить хеш аккаунта в отчеты об ошибках

admin-settings-key-observability-client-max-per-session = Отчеты об ошибках при загрузке страницы

admin-settings-key-observability-client-sample-rate = Частота выборки отчета об ошибках

admin-settings-key-observability-client-source-maps = Разрешение трассировок стека с помощью исходных карт

admin-settings-key-privacy-error-reports = Отправлять отчеты об ошибках

admin-settings-key-privacy-image-cache-max-stale-secs = Сохранять устаревшие данные при сбое источника (секунды)

admin-settings-key-privacy-image-cache-stale-secs = Подавать устаревшие данные при повторной проверке (секунды)

admin-settings-key-privacy-image-resize-enabled = Изменение размера больших изображений

admin-settings-key-privacy-image-resize-format = Формат изображения с измененным размером

admin-settings-key-privacy-image-resize-max-pixels = Самое большое изображение, размер которого можно изменить (в пикселях)

admin-settings-key-privacy-image-resize-max-width = Максимально измененное изображение (в пикселях)

admin-settings-key-privacy-sessions-show-linked-origin = Источник связанного почтового ящика в сеансах

admin-settings-key-properties-folder-enabled = Диалог свойств папки

admin-settings-key-properties-folder-notes-enabled = Примечания к папке

admin-settings-key-properties-folder-notes-max-length = Длина заметки в папке

admin-settings-key-properties-folder-overrides-allowed = Разрешено переопределение для каждой папки

admin-settings-key-properties-folder-sections = Разделы диалогового окна папки

admin-settings-key-properties-folder-size-compute = Вычисление размера папки

admin-settings-key-properties-headers-raw = Необработанные заголовки

admin-settings-key-properties-message-enabled = Диалог свойств сообщения

admin-settings-key-properties-message-permanent-delete = Безвозвратное удаление из свойств

admin-settings-key-properties-message-sections = Разделы диалога сообщений

admin-settings-key-properties-offline-clear-enabled = Очистить автономный кеш для каждой папки

admin-settings-key-properties-remote-location-visibility = Детали удаленного местоположения

admin-settings-key-properties-scoring-show = Детали подсчета очков и аутентификации

admin-settings-key-properties-shortcut-enabled = Ярлык свойств

admin-settings-key-properties-shortcut-key = Сочетание клавиш свойств

admin-settings-key-reader-actions-archive = Размещение архивного действия

admin-settings-key-reader-actions-block-sender = Заблокировать размещение действия отправителя

admin-settings-key-reader-actions-colors = Расположение действий цвета сообщения

admin-settings-key-reader-actions-conversation = Посмотреть размещение действия в диалоге

admin-settings-key-reader-actions-create-filter = Создать размещение действия фильтра

admin-settings-key-reader-actions-delete = Удалить место действия

admin-settings-key-reader-actions-delete-sender = Удалить все сообщения из действия отправителя

admin-settings-key-reader-actions-edit-draft = Изменить место размещения черновика действия

admin-settings-key-reader-actions-export = Размещение экспортного действия

admin-settings-key-reader-actions-finish-followup = Отметить продолжение как завершенное действие

admin-settings-key-reader-actions-flag = Пометить/снять пометку с места действия

admin-settings-key-reader-actions-focus-focused = Перейти к размещению целенаправленного действия

admin-settings-key-reader-actions-focus-other = Перейти к другому месту действия

admin-settings-key-reader-actions-forward = Размещение действия вперед

admin-settings-key-reader-actions-headers = Размещение действий в необработанных заголовках

admin-settings-key-reader-actions-links = Обзор размещения действий со ссылками

admin-settings-key-reader-actions-move = Размещение действия перемещения

admin-settings-key-reader-actions-not-spam = Размещение не спам-действия

admin-settings-key-reader-actions-open-draft = Место действия «Открыть ответ»

admin-settings-key-reader-actions-pin = Закрепить/открепить размещение действия

admin-settings-key-reader-actions-print = Размещение действия печати

admin-settings-key-reader-actions-properties = Размещение действия свойств сообщения

admin-settings-key-reader-actions-read = Прочитанное/непрочитанное размещение действия

admin-settings-key-reader-actions-reply = Размещение ответного действия

admin-settings-key-reader-actions-reply-all = Ответить на все размещения действий

admin-settings-key-reader-actions-save-contacts = Сохранить размещение контактов в сообщении

admin-settings-key-reader-actions-save-sender = Сохранить место действия отправителя

admin-settings-key-reader-actions-search = Место размещения связанного поискового действия

admin-settings-key-reader-actions-security = Размещение действия в режиме безопасности

admin-settings-key-reader-actions-spam = Пометить как спам-место размещения

admin-settings-key-reader-actions-sweep = Размещение действия отправителя

admin-settings-key-reader-actions-view-source = Просмотреть размещение действия источника сообщения

admin-settings-key-reader-actions-zoom = Размещение действия масштабирования

admin-settings-key-retention-client-errors-event-days = Сохранение ошибок (дней)

admin-settings-key-retention-client-errors-group-days = Группы ошибок сохраняются (дни)

admin-settings-key-retention-metrics-day-buckets-days = Ведение ежедневной статистики (дней)

admin-settings-key-retention-metrics-hour-buckets-days = Хранится почасовая статистика (дни)

admin-settings-key-retention-metrics-minute-buckets-hours = Хранится минутная статистика (часы)

admin-settings-key-retention-user-allow-strip-attachments = Пользователи могут удалять вложения из старой почты

admin-settings-key-retention-user-calendar-enabled = Пользователи могут удалять завершенные события календаря.

admin-settings-key-retention-user-calendar-max-days = Большинство дней до окончания мероприятий удаляются.

admin-settings-key-retention-user-calendar-min-days = Мероприятия за меньшее количество дней до окончания будут удалены.

admin-settings-key-retention-user-notes-enabled = Пользователи могут удалять нетронутые заметки

admin-settings-key-retention-user-notes-max-days = За большинство дней до удаления нетронутых заметок

admin-settings-key-retention-user-notes-min-days = Минимум дней до удаления нетронутых заметок.

admin-settings-key-retention-user-strip-keep-inline-images = Сохраняйте встроенные изображения

admin-settings-key-retention-user-strip-min-age-days = Минимум дней до удаления вложений

admin-settings-key-retention-user-strip-min-attachment-kb = Удалено наименьшее вложение (КБ)

admin-settings-key-retention-user-strip-undo-days = Окно отмены после удаления вложений (дни)

admin-settings-key-server-shutdown-grace-seconds = Льготный период отключения

admin-settings-key-session-last-activity-flush-seconds = Интервал последней записи

admin-settings-key-session-persist-across-restarts = Сохраняйте сеансы после перезапуска

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = По умолчанию выключено. Сохраняйте зашифрованные снимки почты на уровне учетной записи при перезагрузке браузера без истечения срока действия. Существующие ограничения на количество записей и байтов по-прежнему применяются. Пользователи могут удалять сохраненные копии; отключение этого параметра удаляет их в активном браузере. Это не дает браузеру разрешения на постоянное хранение или загрузку вложений.

admin-settings-help-mail-conversations-allowed = Разрешить пользователям группировать связанные ответы в беседы. Доступно по умолчанию, но настройки пользователя сначала отключены. Отключение доступности отменяет выбор пользователя без изменения группировки категорий.

admin-settings-help-mail-conversations-enabled = По умолчанию выключено. Группировать ответы по цепочкам, независимо от функции «Группировать по». Установите настройку по умолчанию или заблокируйте эту настройку. Доступность разговора также должна быть включена.

admin-settings-key-mail-cache-keep-indefinitely = Хранить почтовый кеш браузера на неопределенный срок

admin-settings-key-mail-conversations-allowed = Сделайте цепочку разговоров доступной

admin-settings-key-mail-conversations-enabled = Группируйте сообщения в беседы

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = До 100 доменов отправителей, написанных строчными буквами, ссылки на проверенные сообщения которых включаются автоматически. Субдомены отдельные. Это не доверяет целевым веб-сайтам и не разрешает удаленные изображения. По умолчанию пусто; пользователи могут изменить его, если он не заблокирован.

admin-settings-key-links-trusted-sender-domains = Доверенные домены отправителей для ссылок на сообщения

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Показывать сведения о загрузке и файле рядом с заголовком предварительного просмотра в виде значков (по умолчанию), меток или значков и меток. Может быть установлен и заблокирован глобально, для каждого домена или для каждого пользователя.

admin-settings-help-reader-attachment-thumbnails = По умолчанию выключено. Разрешить видимым вложениям изображений загружать миниатюры предварительного просмотра. Правила безопасности вложений по-прежнему применяются. Может быть установлен и заблокирован глобально, для каждого домена или для каждого пользователя.

admin-settings-key-reader-attachment-action-style = Действия предварительного просмотра вложений

admin-settings-key-reader-attachment-thumbnails = Миниатюры прикрепленных изображений

admin-settings-key-mail-compose-auto-add-mention-recipients = Автоматически добавлять упомянутых людей в получатели
admin-settings-help-mail-compose-auto-add-mention-recipients = При выборе человека из подсказок @упоминаний добавлять его в поле Кому, если его ещё нет в полях Кому, Копия или Скрытая копия. По умолчанию отключено.

admin-settings-linked-disable-title = Отключить сохранённый доступ к связанным аккаунтам?
admin-settings-linked-disable-body = При сохранении удаляются учётные данные для продления доступа к связанным аккаунтам. Пользователям потребуется войти снова, когда доступ нужно будет продлить. Повторное включение не восстановит эти данные.
admin-settings-linked-disable-confirm = Отключить и удалить сохранённый доступ

admin-settings-key-reader-auto-hide-header = Автоматически скрывать заголовки сообщений
admin-settings-help-reader-auto-hide-header = Открывать сообщения со свёрнутыми данными отправителя, получателей и метаданными. Тема и действия остаются видимыми. Нажмите Показать заголовок для просмотра подробностей. По умолчанию отключено.

admin-settings-key-time-region = Регион
admin-settings-help-time-region = auto или двухбуквенный код региона, например GB, US или PT.

admin-settings-key-time-date-format = Формат даты
admin-settings-help-time-date-format = auto, short, medium, long, full или шаблон, например yyyy-MM-dd либо dd/MM/yyyy. Обычный текст заключайте в 'кавычки'.

admin-settings-key-time-time-format = Формат времени
admin-settings-help-time-time-format = auto, short, medium, long, full или HH:mm:ss / h:mm a. H = 24 часа, h = 12 часов, a = AM/PM.

admin-settings-key-time-hour-cycle = Часы
admin-settings-help-time-hour-cycle = Символы H или h в своём шаблоне имеют приоритет.

admin-settings-key-time-first-day-of-week = Первый день недели
admin-settings-help-time-first-day-of-week = Автоматически по региональным правилам календаря.

admin-settings-key-compliance-mail-enabled = Включить почту
admin-settings-help-compliance-mail-enabled = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-host = Сервер SMTP
admin-settings-help-compliance-mail-host = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-port = Порт SMTP
admin-settings-help-compliance-mail-port = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-security = Режим TLS
admin-settings-help-compliance-mail-security = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-from = Адрес отправителя
admin-settings-help-compliance-mail-from = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-username = Имя пользователя SMTP
admin-settings-help-compliance-mail-username = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-allowPrivate = Разрешить частный сервер
admin-settings-help-compliance-mail-allowPrivate = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-dsr-notify-emailAdmins = Уведомлять операторов о запросах данных
admin-settings-help-dsr-notify-emailAdmins = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-dsr-notify-operatorAddresses = Адресаты операторы
admin-settings-help-dsr-notify-operatorAddresses = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-dsr-notify-emailRequester = Уведомлять заявителя по почте
admin-settings-help-dsr-notify-emailRequester = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-breach-notify-email = Уведомлять операторов об утечках
admin-settings-help-breach-notify-email = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-breach-subjects-email = Уведомлять затронутых лиц по почте
admin-settings-help-breach-subjects-email = Настройте SMTP-сервер с TLS. Почта использует маршрут прокси system_mail. При неясном принятии проверьте результат перед повторной отправкой.

admin-settings-key-compliance-mail-requireProxy = Требовать маршрут прокси
admin-settings-help-compliance-mail-requireProxy = Требовать маршрут прокси (system_mail).
admin-settings-key-retention-complianceMail-days = Хранение истории завершённых писем (дни)
admin-settings-help-retention-complianceMail-days = Срок хранения завершённых доставок. Ожидающие и неопределённые остаются до разрешения.

admin-settings-key-incidents-auto-enabled = Автоматически выявлять возможные инциденты
admin-settings-help-incidents-auto-enabled = Сбои проверок и сигналы безопасности сеансов создают возможные инциденты. Оцените значимость и проверьте восстановление. Пробелы мониторинга не доказывают простой.
admin-settings-key-incidents-auto-failureSamples = Неудачные проверки до обнаружения
admin-settings-help-incidents-auto-failureSamples = Сбои проверок и сигналы безопасности сеансов создают возможные инциденты. Оцените значимость и проверьте восстановление. Пробелы мониторинга не доказывают простой.
admin-settings-key-incidents-nis2-trustService = Использовать срок для доверительных услуг
admin-settings-help-incidents-nis2-trustService = Сбои проверок и сигналы безопасности сеансов создают возможные инциденты. Оцените значимость и проверьте восстановление. Пробелы мониторинга не доказывают простой.
admin-settings-key-incidents-notify-email = Отправлять обновления и напоминания по почте
admin-settings-help-incidents-notify-email = Сбои проверок и сигналы безопасности сеансов создают возможные инциденты. Оцените значимость и проверьте восстановление. Пробелы мониторинга не доказывают простой.
admin-settings-key-incidents-notify-operatorAddresses = Адреса операторов
admin-settings-help-incidents-notify-operatorAddresses = Сбои проверок и сигналы безопасности сеансов создают возможные инциденты. Оцените значимость и проверьте восстановление. Пробелы мониторинга не доказывают простой.
admin-settings-key-incidents-retention-days = Хранение закрытых инцидентов (дни)
admin-settings-help-incidents-retention-days = Сбои проверок и сигналы безопасности сеансов создают возможные инциденты. Оцените значимость и проверьте восстановление. Пробелы мониторинга не доказывают простой.

admin-settings-key-mail-snooze-presets = Настройки откладывания
admin-settings-help-mail-snooze-presets = Добавляйте, переименовывайте, упорядочивайте, изменяйте или скрывайте варианты. В меню всегда доступен выбор даты и времени.

admin-settings-key-notifications-view-enabled = Включить раздел уведомлений
admin-settings-help-notifications-view-enabled = Управляет центром уведомлений. Оповещения безопасности, почты и браузера сохраняют собственные настройки.
admin-settings-key-notifications-view-show-in-sidebar = Показывать уведомления на боковой панели
admin-settings-help-notifications-view-show-in-sidebar = Управляет центром уведомлений. Оповещения безопасности, почты и браузера сохраняют собственные настройки.

admin-settings-key-mail-unified-account = Общая виртуальная учётная запись
admin-settings-help-mail-unified-account = Добавить отдельную виртуальную учётную запись в список аккаунтов с общими папками и счётчиками. Независимо от общего вида сообщений.

admin-settings-key-accounts-settings-inherit-groups = Наследовать настройки основного аккаунта
admin-settings-help-accounts-settings-inherit-groups = Выбранные группы используют значения основного аккаунта при отсутствии локального переопределения. Локальные настройки и политика администратора приоритетны. Темы, подписи, учётные данные и идентификаторы папок не наследуются.
