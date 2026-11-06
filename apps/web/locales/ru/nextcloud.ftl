# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Вверх
nextcloud-loading = Загрузка…
nextcloud-list-error = Не удалось получить содержимое этой папки.
nextcloud-empty = Эта папка пуста.
nextcloud-file-list = Файлы Nextcloud
nextcloud-open-folder = Открыть папку { $name }
nextcloud-select-file = Выбрать { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Вложить из Nextcloud
nextcloud-attach-action = { $count ->
    [0] Вложить
    [one] Вложить { $count } файл
    [few] Вложить { $count } файла
    [many] Вложить { $count } файлов
   *[other] Вложить { $count } файла
  }
nextcloud-error-select-file = выберите хотя бы один файл для вложения
nextcloud-error-attach-failed = не удалось вложить выбранные файлы

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Сохранить в Nextcloud
nextcloud-save-title = Сохранить «{ $name }» в Nextcloud
nextcloud-save-action = Сохранить сюда ({ $dir })
nextcloud-saved = Сохранено в { $path }
nextcloud-error-save-failed = не удалось сохранить в Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Создать ссылку для общего доступа
nextcloud-share-title = Ссылка для общего доступа
nextcloud-share-intro = Создайте общедоступную ссылку на { $path } вместо того, чтобы вкладывать файл.
nextcloud-protect-password = Защитить паролем
nextcloud-password-label = Пароль
nextcloud-share-password = Пароль для общего доступа
nextcloud-set-expiry = Задать срок действия
nextcloud-expires-on = Действует до
nextcloud-expiry-date = Дата окончания срока действия
nextcloud-create-link = Создать ссылку
nextcloud-password-protected = Защищено паролем
nextcloud-no-password = Без пароля
nextcloud-expires = действует до { $date }
nextcloud-no-expiry = без срока действия
nextcloud-error-need-password = введите пароль или отключите защиту паролем
nextcloud-error-need-expiry = выберите дату окончания или отключите срок действия
nextcloud-error-share-failed = не удалось создать ссылку для общего доступа

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = Файл { $path } превышает ограничение на вложения. Загрузите его в папку «Mail attachments» в вашем Nextcloud и вставьте в сообщение ссылку для доступа.
nextcloud-upload-and-link = Загрузить и вставить ссылку
nextcloud-share-not-now = Не сейчас
nextcloud-upload-progress = Загрузка: { $percent }%
nextcloud-upload-progress-label = Ход загрузки
nextcloud-share-inserted = Ссылка для доступа к файлу { $name } вставлена в сообщение.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = Ссылка действует до { $date }.
nextcloud-share-body-password = Ссылка защищена паролем.
nextcloud-account-title = Учётные записи Nextcloud
nextcloud-account-help = Привяжите свою учётную запись к экземпляру, одобренному администратором. Учётные данные хранятся в зашифрованном виде только для этой почтовой учётной записи.
nextcloud-account-load-error = Не удалось загрузить связи с Nextcloud. Перезагрузите страницу, чтобы повторить попытку.
nextcloud-account-unlink-error = Не удалось удалить связь. Перезагрузите страницу перед повторной попыткой.
nextcloud-account-reload = Перезагрузить
nextcloud-account-no-instances = Нет доступных включённых экземпляров Nextcloud. Попросите администратора настроить хотя бы один.
nextcloud-account-ready = Привязано и готово
nextcloud-account-stale = Конфигурация экземпляра изменилась. Проверьте привязку снова.
nextcloud-account-unlinked = Не привязано
nextcloud-account-relink = Проверить привязку снова
nextcloud-account-link = Привязать учётную запись
nextcloud-account-unlink = Отвязать учётную запись
nextcloud-account-password-help = В личных настройках безопасности Nextcloud создайте пароль приложения и скопируйте сюда его имя пользователя и пароль. Проверка выполняет один ограниченный запрос; отклонённые учётные данные не повторяются.
nextcloud-account-username = Имя пользователя пароля приложения Nextcloud
nextcloud-account-password = Пароль приложения Nextcloud
nextcloud-account-verify = Проверить и привязать
nextcloud-account-verify-error = Проверка или сохранение не удались. Проверьте экземпляр, имя пользователя и пароль приложения. Поле пароля очищено; существующая рабочая привязка не заменена.
nextcloud-admin-title = Экземпляры Nextcloud
nextcloud-admin-help = Одобрите экземпляры HTTPS, которые пользователи почты могут привязывать индивидуально. Почтовым учётным записям не выдаётся общий пароль службы. Операции с файлами ограничены 8 МиБ. Пользователи нативного движка после привязки явно обнаруживают и подключают коллекции календарей, задач и адресных книг.
nextcloud-admin-add = Добавить экземпляр Nextcloud
nextcloud-admin-edit = Изменить экземпляр Nextcloud
nextcloud-admin-enabled = Включён
nextcloud-admin-disabled = Отключён
nextcloud-admin-missing = Этого экземпляра больше нет. Вернитесь к списку экземпляров.
nextcloud-admin-private-help = Для частных адресов требуется явное разрешение на исходящий трафик Nextcloud. Настроенные маршруты прокси остаются приоритетными; отклонённый запрос через прокси никогда не переходит на прямое соединение.
nextcloud-admin-revalidation = Сохранение любого изменения экземпляра требует от пользователей повторной проверки их привязок. Отключение экземпляра немедленно останавливает новые операции.
nextcloud-admin-authority = Полномочия экземпляра
nextcloud-admin-name = Отображаемое имя
nextcloud-admin-name-help = Короткое название, которое видят пользователи при выборе экземпляра.
nextcloud-admin-url = URL установки HTTPS
nextcloud-admin-url-help = Используйте корень установки, например https://cloud.example.org/ или https://example.org/nextcloud/. Сертификат должен быть доверенным и соответствовать имени хоста. Учётные данные, параметры запроса и фрагменты отклоняются.
nextcloud-admin-enable = Разрешить пользователям привязывать этот экземпляр
nextcloud-admin-ca = Дополнительные центры сертификации (PEM)
nextcloud-admin-ca-help = Необязательные сертификаты частных центров сертификации для этого сервера Nextcloud. Публичное доверие остаётся доступным, проверка имени хоста по-прежнему обязательна. Изменения требуют от пользователей повторной проверки привязок; доверие к сертификатам прокси задаётся отдельно.
nextcloud-instance-select = Экземпляр Nextcloud
nextcloud-instance-placeholder = Выберите привязанный экземпляр
nextcloud-instances-empty = Нет проверенных привязок Nextcloud. Добавьте или проверьте привязку в разделе «Настройки пользователя» → Nextcloud.
nextcloud-instances-error = Не удалось загрузить привязанные экземпляры. Откройте окно выбора снова, чтобы повторить попытку.
nextcloud-save-warning = Администратор требует подтверждения, прежде чем это вложение сможет покинуть почтовый ящик.
nextcloud-save-acknowledge = Я понимаю предупреждение о вложении и хочу сохранить этот файл в выбранном экземпляре Nextcloud.
nextcloud-save-attachment = Сохранить { $name } в Nextcloud
nextcloud-save-close = Закрыть выбор места назначения Nextcloud
nextcloud-save-options = Сохранить вложения в Nextcloud
nextcloud-pim-title = Контакты и календари
nextcloud-pim-help = Обнаружьте коллекции из этой проверенной учётной записи Nextcloud, затем явно подключите нужные. Синхронизируйте, чтобы получить актуальные данные; изменения в подключённых коллекциях применяются к удалённой коллекции и завершаются ошибкой, если её привязка или права отозваны.
nextcloud-pim-category = Категория коллекции
nextcloud-pim-calendar = Календари
nextcloud-pim-contacts = Адресные книги
nextcloud-pim-tasks = Списки задач
nextcloud-pim-discover = Обнаружить коллекции
nextcloud-pim-discovered = Доступные коллекции
nextcloud-pim-empty = Подходящих коллекций не получено.
nextcloud-pim-connect = Подключить
nextcloud-pim-connected = Подключённые коллекции
nextcloud-pim-sync = Синхронизировать сейчас
nextcloud-pim-sync-needed = Коллекция подключена. Синхронизируйте сейчас, чтобы загрузить её содержимое.
nextcloud-pim-synced = Синхронизация завершена.
nextcloud-pim-error = Операция не удалась. Проверьте привязку учётной записи и права, затем повторите попытку явно. Существующие кэшированные данные не удалены.
nextcloud-pim-native-required = Для синхронизации контактов и календаря нужен нативный почтовый движок. Доступ к файлам остаётся доступным через привязку вашей учётной записи.
nextcloud-pim-disconnect = Отключить
nextcloud-pim-disconnect-help = Отключение сохраняет кэшированные данные и останавливает запись на удалённую сторону. Для заново привязанных учётных записей нужно снова выполнить обнаружение и явное подключение.
nextcloud-pim-ready = Готово
nextcloud-pim-disconnected = Отключено или привязка изменена
nextcloud-pim-unnamed = Коллекция без названия
nextcloud-pim-back = Назад к учётным записям Nextcloud
nextcloud-pim-create-title = Новая коллекция в Nextcloud
nextcloud-pim-create-help = Создаёт пустой календарь или список задач в этой учётной записи Nextcloud и подключает его здесь.
nextcloud-pim-create-name = Название новой коллекции
nextcloud-pim-create = Создать в Nextcloud
nextcloud-pim-created = { $name } создана в Nextcloud и подключена. Синхронизируйте сейчас, чтобы загрузить её.
nextcloud-pim-interval = Интервал автоматического обновления (секунды)
nextcloud-pim-interval-help = Ноль означает только ручное обновление. Положительные интервалы не менее 60 секунд. Подключённые учётные записи обновляются в фоне; при сбоях интервалы увеличиваются, а при отклонённых учётных данных дальнейшие запросы прекращаются до повторной привязки.
nextcloud-pim-interval-reset = Использовать значение администратора по умолчанию
nextcloud-admin-empty = Одобренных экземпляров пока нет. Добавьте экземпляр, чтобы сделать его доступным для привязки учётных записей.
nextcloud-admin-status = Состояние
nextcloud-admin-open = Открыть { $name }
