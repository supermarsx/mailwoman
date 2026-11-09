# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Папка перемещена: { $name }
folder-dnd-moved-many = Перемещено папок: { $count }
folder-dnd-undo = Отменить
folder-dnd-cancel = Отмена
folder-dnd-handle = Перетащите «{ $name }», чтобы переместить
folder-dnd-locked-hint = Порядок папок задан администратором.
folder-dnd-forced-hint = Закреплена администратором. Её нельзя переместить или открепить.
folder-dnd-disabled-hint = Перетаскивание папок отключено.
folder-dnd-system-hint = Системные папки нельзя перемещать.
folder-dnd-pinned-group = Закреплённые
folder-dnd-top-level = Переместить на верхний уровень
folder-dnd-problem-disabled = Перемещение папок отключено.
folder-dnd-problem-locked = Порядок папок задан администратором.
folder-dnd-problem-self = Папку нельзя переместить в саму себя.
folder-dnd-problem-descendant = Папку нельзя переместить в одну из её подпапок.
folder-dnd-problem-system = Системные папки нельзя переместить сюда.
folder-dnd-problem-no-children = Эта папка не может содержать подпапки.
folder-dnd-problem-nesting-off = Администратор не разрешает вложенные папки.
folder-dnd-problem-depth = Папки окажутся вложены слишком глубоко.
folder-dnd-problem-forced = Эта папка закреплена администратором.
folder-dnd-problem-unknown = Такой папки больше нет.
folder-dnd-problem-noop = Папка уже находится там.
folder-dnd-problem-collision = Папка «{ $name }» уже существует там. Сначала переименуйте её.
folder-dnd-problem-server = Сервер не переместил папку «{ $name }». Ничего не изменилось.
folder-dnd-announce-up = «{ $name }» перемещена вверх
folder-dnd-announce-down = «{ $name }» перемещена вниз
folder-dnd-announce-indent = «{ $name }» перемещена в папку выше
folder-dnd-announce-outdent = «{ $name }» перемещена на уровень выше
folder-dnd-nest-title = Переместить «{ $name }» сюда?
folder-dnd-nest-title-many = Переместить сюда папки: { $count }?
folder-dnd-nest-body = Выберите, изменится ли папка на почтовом сервере или только то, как она показана в этом приложении.
folder-dnd-nest-legend = Способ перемещения
folder-dnd-nest-local = Только расположить в этом приложении
folder-dnd-nest-local-help = Только вид. Папка остаётся на прежнем месте на сервере и в других почтовых приложениях.
folder-dnd-nest-server = Переместить на сервере
folder-dnd-nest-server-help = Изменяет папку везде, во всех почтовых приложениях, использующих эту учётную запись.
folder-dnd-nest-remember = Запомнить выбор
folder-dnd-nest-confirm = Переместить
folder-dnd-arranged-only = Расположена только в этом приложении
folder-dnd-arranged-only-help = Её место на сервере другое: { $path }
folder-dnd-apply-server = Применить на сервере
folder-dnd-reset = Сбросить порядок по умолчанию
folder-dnd-reset-done = Порядок папок сброшен
folder-dnd-move-to = Переместить «{ $name }» в...
folder-dnd-layout-link = Расположение папок
folder-dnd-reorder-enable = Менять порядок папок перетаскиванием
folder-dnd-nest-mode = При вложении папки
folder-dnd-nest-mode-ask = Спрашивать
folder-dnd-nest-mode-server = Перемещать на сервере
folder-dnd-nest-mode-local = Только располагать в этом приложении
folder-dnd-new-placement = Новые папки появляются
folder-dnd-new-placement-end = После папок того же уровня
folder-dnd-new-placement-alphabetical = В алфавитном порядке
folder-dnd-handles = Показывать маркеры перетаскивания
folder-dnd-handles-hover = При наведении
folder-dnd-handles-always = Всегда
folder-dnd-arrange-title = Расположение папок
folder-dnd-arrange-help = Перетащите папку, чтобы изменить её порядок, опустите на другую папку, чтобы вложить её, или используйте Alt и клавиши со стрелками на выбранной папке.
folder-dnd-admin-locked = Задано администратором
folder-dnd-arrange-menu = Переместить в…
folder-star-add = Отметить папку звёздочкой
folder-star-remove = Снять звёздочку с папки
folder-star-starred = Со звёздочкой
folder-star-toggle = Отметить звёздочкой: { $name }
folder-star-only = Показывать только со звёздочкой
folder-star-only-help = Показывает только папки со звёздочкой и папки, которые их содержат.
folder-dnd-indent = Переместить «{ $name }» в папку выше
folder-dnd-outdent = Переместить «{ $name }» на уровень выше
folder-dnd-more-actions = Расположение
folder-dnd-selected-move = Переместить выбранные в…
folder-dnd-locked-row = Заблокировано
folder-dnd-problem-pin = Закреплённые папки остаются над остальными. Сначала открепите её, чтобы переместить туда.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Пусто…

folder-bar-label = Действия для выбранных папок

folder-bar-mark-read = Отметить все прочитанными

folder-bar-pin = Пин-код

folder-bar-star = Звезда

folder-bar-unpin = Открепить

folder-bar-unstar = Снять пометку

folder-dnd-handle-hint = Перетащите, чтобы переместить эту папку

folder-pinned-group = Закреплено

folder-pinned-unpin-zone = Перетащите сюда, чтобы открепить

settings-folders-bulk-delete-body = { $folders ->
        [one] { $folders } и сообщения { $messages } в ней будут удалены. Это невозможно отменить.
        [few] { $folders } папки (с их подпапками) и сообщения { $messages } в них будут удалены. Это невозможно отменить.
        [many] { $folders } (с их подпапками) и сообщения { $messages } в них будут удалены. Это невозможно отменить.
       *[other] { $folders } (с их подпапками) и сообщения { $messages } в них будут удалены. Это невозможно отменить.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [one] Удалить { $count } папку
        [few] Удалить { $count } папки
        [many] Удалить { $count } папок
       *[other] Удалить { $count } папки
    }

settings-folders-bulk-delete-done = { $count ->
        [one] Удалена { $count } папка
        [few] Удалил { $count } папки
        [many] Удалено { $count } папок
       *[other] Удалил { $count } папки
    }

settings-folders-bulk-delete-failed = Ничего не было удалено. Не удалено: { $names }

settings-folders-bulk-delete-list = Папки для удаления

settings-folders-bulk-delete-none = Ни одну из выбранных папок удалить невозможно (системные папки и папки, которые вы не можете удалить, пропускаются).

settings-folders-bulk-delete-others = Также не удалено: { $names }

settings-folders-bulk-delete-partial = Удалены папки { $done }. Не удалено: { $names }

settings-folders-bulk-delete-partial-done = Удалены папки { $done }.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [one] Не удалено, поскольку учетная запись находится под юридическим запретом: { $names }
        [few] Не удалено, поскольку учетная запись находится под юридическим запретом ({ $count } папки): { $names }
        [many] Не удалено, поскольку учетная запись находится под юридическим запретом ({ $count } папок): { $names }
       *[other] Не удалено, поскольку учетная запись находится под юридическим запретом ({ $count } папки): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [one] Не удалено, поскольку обработка учетной записи ограничена: { $names }
        [few] Не удалено, поскольку обработка учетной записи ограничена ({ $count } папки): { $names }
        [many] Не удалено, поскольку обработка учетной записи ограничена ({ $count } папок): { $names }
       *[other] Не удалено, поскольку обработка учетной записи ограничена ({ $count } папки): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } выбранные папки пропускаются, поскольку их невозможно удалить: { $names }

settings-folders-bulk-delete-subfolders = Также удалите подпапки каждой папки

settings-folders-bulk-delete-subfolders-off = Будут удалены только выбранные папки. Папка, в которой все еще есть подпапка вне выделенной области, сохраняется.

settings-folders-bulk-delete-title = { $count ->
        [one] Удалить { $count } папку?
        [few] Удалить { $count } папки?
        [many] Удалить { $count } папок?
       *[other] Удалить { $count } папки?
    }

settings-folders-bulk-empty-body = Сообщения { $total } в папках { $folders } будут удалены: { $toTrash } перемещаются в корзину и их можно отменить, { $permanent } удаляются навсегда и их нельзя вернуть обратно. Подпапки не трогаются.

settings-folders-bulk-empty-confirm = Очистите папки

settings-folders-bulk-empty-counting = Подсчет сообщений в каждой папке…

settings-folders-bulk-empty-done = Опустошены папки { $count }, сообщения { $messages }.

settings-folders-bulk-empty-fail-refused = сервер отказался от части; { $remaining } сообщения остаются

settings-folders-bulk-empty-fail-stopped = остановился; { $remaining } сообщения остаются

settings-folders-bulk-empty-fail-uncertain = результат последней партии неизвестен; проверьте папку

settings-folders-bulk-empty-list = Папки, которые нужно очистить

settings-folders-bulk-empty-partial = Опустошены папки { $done }; { $failed } не удалось. Посмотрите список.

settings-folders-bulk-empty-permanent = Сообщения { $count } удаляются навсегда

settings-folders-bulk-empty-progress = { $done } из { $of } папок выполнено, { $messages } сообщений обработано. Сейчас: { $name }

settings-folders-bulk-empty-skip-count-failed = невозможно было посчитать, пропустили

settings-folders-bulk-empty-skip-empty = уже пусто, пропущено

settings-folders-bulk-empty-skip-permanent-denied = будут удалены навсегда, которые администратор отключил, пропустил

settings-folders-bulk-empty-skip-search = сохраненный поиск, пропущен

settings-folders-bulk-empty-skip-unsupported = этот аккаунт не может очистить его, пропущено

settings-folders-bulk-empty-title = { $count ->
        [one] Очистить { $count } папку?
        [few] Очистить { $count } папки?
        [many] Очистить { $count } папок?
       *[other] Очистить выбранные папки ({ $count })?
    }

settings-folders-bulk-empty-to-trash = Сообщения { $count } перемещаются в корзину.

settings-folders-bulk-empty-undo-label = Опустошенные папки { $count } (сообщения { $messages })

settings-folders-delete-selected = Удалить…

settings-folders-select-user = Выбрать все папки пользователя

settings-folders-subscribe-failed = Подписки не удалось изменить.

settings-folders-subscribe-partial = Изменены папки { $done }; { $failed } изменить невозможно.

settings-folders-subscribe-selected = Подписаться

settings-folders-subscribed = { $count ->
        [one] Подписан на { $count } папку
        [few] Подписан на { $count } папки
        [many] Подписан на { $count } папок
       *[other] Подписан на { $count } папки
    }

settings-folders-unsubscribe-selected = Отписаться

settings-folders-unsubscribed = { $count ->
        [one] Отписался от { $count } папки
        [few] Отписался от { $count } папок
        [many] Отписался от { $count } папок
       *[other] Отписался от { $count } папок
    }
