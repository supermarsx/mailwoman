# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Пользователи. Заголовки столбцов сортируют список.
admin-userlist-search = Поиск пользователей
admin-userlist-search-placeholder = Имя, адрес, псевдоним или домен
admin-userlist-count = { $count ->
    [one] { $count } пользователь
    [few] { $count } пользователя
    [many] { $count } пользователей
   *[other] { $count } пользователя
}
admin-userlist-count-filtered = { $count ->
    [one] Подходит { $count } из { $total } пользователей
    [few] Подходят { $count } из { $total } пользователей
    [many] Подходят { $count } из { $total } пользователей
   *[other] Подходят { $count } из { $total } пользователей
}
admin-userlist-range = { $from }–{ $to } из { $total }
admin-userlist-loading = Загрузка пользователей…
admin-userlist-retry = Повторить
admin-userlist-empty-filtered = Нет пользователей, подходящих под этот поиск и фильтры.
admin-userlist-compact = Компактные строки
admin-userlist-columns = Столбцы
admin-userlist-columns-legend = Показывать столбцы
admin-userlist-actions = Действия
admin-userlist-paging = Страницы пользователей
admin-userlist-page-size = Строк на странице
admin-userlist-prev = Назад
admin-userlist-next = Далее
admin-userlist-sort-by = Сортировать по
admin-userlist-sort-asc = { $column }, по возрастанию
admin-userlist-sort-desc = { $column }, по убыванию
admin-userlist-col-user = Пользователь
admin-userlist-col-domain = Домен
admin-userlist-col-status = Состояние
admin-userlist-col-2fa = Двухфакторная аутентификация
admin-userlist-col-role = Роль в консоли
admin-userlist-col-last-signin = Последний вход
admin-userlist-col-last-seen = Последняя активность
admin-userlist-col-created = Создан
admin-userlist-col-quota = Квота
admin-userlist-col-profile = Профиль функций
admin-userlist-col-linked = Связанные ящики
admin-userlist-state-active = Активен
admin-userlist-state-pending = Ни разу не входил
admin-userlist-state-blocked = Заблокирован
admin-userlist-state-disabled = Отключён
admin-userlist-state-deleted = Удалён
admin-userlist-2fa-passkey = Ключ доступа
admin-userlist-2fa-totp = Приложение-аутентификатор
admin-userlist-2fa-none = Нет
admin-userlist-signin-never = Никогда
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Активность не зафиксирована
admin-userlist-seen-at = Последняя активность: { $relative } ({ $exact })
admin-userlist-quota-unlimited = Без ограничений
admin-userlist-profile-default = По умолчанию
admin-userlist-profile-via-user = Назначен этому пользователю
admin-userlist-profile-via-group = Из группы
admin-userlist-profile-via-domain = Из домена
admin-userlist-more = Дополнительные действия
admin-userlist-more-for = Дополнительные действия для { $account }
admin-userlist-select = Выбрать { $account }
admin-userlist-select-page = Выбрать всех пользователей на этой странице
admin-userlist-filters = Фильтры
admin-userlist-add-filter = Добавить фильтр
admin-userlist-clear-all = Очистить всё
admin-userlist-chip-edit = Изменить фильтр { $label }
admin-userlist-chip-remove = Удалить фильтр { $label }
admin-userlist-filter-domain = Домен
admin-userlist-filter-status = Состояние
admin-userlist-filter-tfa = Двухфакторная аутентификация
admin-userlist-filter-role = Роль в консоли
admin-userlist-filter-profile = Профиль функций
admin-userlist-filter-signin = Последний вход
admin-userlist-filter-created = Создан
admin-userlist-filter-no-choices = Пока нечего выбирать.
admin-userlist-filter-remove = Удалить фильтр
admin-userlist-apply = Применить
admin-userlist-cancel = Отмена
admin-userlist-role-any = Любая роль в консоли
admin-userlist-role-none = Без роли в консоли
admin-userlist-profile-none = Без профиля (по умолчанию)
admin-userlist-signin-older = { $days ->
    [one] Более { $days } дня назад
    [few] Более { $days } дней назад
    [many] Более { $days } дней назад
   *[other] Более { $days } дня назад
}
admin-userlist-signin-within = { $days ->
    [one] За последний { $days } день
    [few] За последние { $days } дня
    [many] За последние { $days } дней
   *[other] За последние { $days } дня
}
admin-userlist-signin-custom = Более указанного числа дней назад:
admin-userlist-signin-days = Дни
admin-userlist-days-invalid = Введите целое число дней от 1 до 36500.
admin-userlist-created-from = С
admin-userlist-created-to = По
admin-userlist-created-between = с { $from } по { $to }
admin-userlist-created-since = с { $date }
admin-userlist-created-until = по { $date }
admin-userlist-dates-invalid = Дата начала позже даты окончания.
admin-userlist-views = Сохранённые представления
admin-userlist-view-save = Сохранить это представление…
admin-userlist-view-name = Название представления
admin-userlist-view-save-confirm = Сохранить представление
admin-userlist-view-delete = Удалить представление «{ $name }»
admin-userlist-bulk = Действия над выбранными пользователями
admin-userlist-selected = { $count ->
    [one] Выбран { $count }
    [few] Выбрано { $count }
    [many] Выбрано { $count }
   *[other] Выбрано { $count }
}
admin-userlist-clear-selection = Снять выбор
admin-userlist-block = Заблокировать вход
admin-userlist-unblock = Разблокировать вход
admin-userlist-sign-out = Выйти везде
admin-userlist-delete = Удалить локальный доступ
admin-userlist-bulk-profile = Назначить профиль функций
admin-userlist-bulk-profile-clear = Удалить собственный профиль пользователя
admin-userlist-confirm-skipped = { $count ->
    [one] Выбранный пользователь ({ $count }) будет пропущен: действие к нему не применимо.
    [few] Выбранные пользователи ({ $count }) будут пропущены: действие к ним не применимо.
    [many] Выбранные пользователи ({ $count }) будут пропущены: действие к ним не применимо.
   *[other] Выбранные пользователи ({ $count }) будут пропущены: действие к ним не применимо.
}
admin-userlist-confirm-block = { $count ->
    [one] Заблокировать вход для { $count } пользователя?
    [few] Заблокировать вход для { $count } пользователей?
    [many] Заблокировать вход для { $count } пользователей?
   *[other] Заблокировать вход для { $count } пользователя?
}
admin-userlist-confirm-block-one = Заблокировать вход для { $account }?
admin-userlist-confirm-block-body = Новые входы отклоняются, а текущие сеансы Mailwoman завершаются. Исходный почтовый ящик не меняется.
admin-userlist-confirm-block-action = Заблокировать вход
admin-userlist-confirm-unblock = { $count ->
    [one] Разблокировать вход для { $count } пользователя?
    [few] Разблокировать вход для { $count } пользователей?
    [many] Разблокировать вход для { $count } пользователей?
   *[other] Разблокировать вход для { $count } пользователя?
}
admin-userlist-confirm-unblock-one = Разблокировать вход для { $account }?
admin-userlist-confirm-unblock-body = Локальная блокировка входа снимается. Остальные ограничения безопасности продолжают действовать.
admin-userlist-confirm-unblock-action = Разблокировать вход
admin-userlist-confirm-signout = { $count ->
    [one] Завершить везде сеансы { $count } пользователя?
    [few] Завершить везде сеансы { $count } пользователей?
    [many] Завершить везде сеансы { $count } пользователей?
   *[other] Завершить везде сеансы { $count } пользователя?
}
admin-userlist-confirm-signout-one = Завершить везде сеансы { $account }?
admin-userlist-confirm-signout-body = Все открытые сеансы Mailwoman этих пользователей завершаются. Они смогут войти снова.
admin-userlist-confirm-signout-action = Выйти
admin-userlist-confirm-delete = { $count ->
    [one] Удалить локальный доступ для { $count } пользователя?
    [few] Удалить локальный доступ для { $count } пользователей?
    [many] Удалить локальный доступ для { $count } пользователей?
   *[other] Удалить локальный доступ для { $count } пользователя?
}
admin-userlist-confirm-delete-one = Удалить локальный доступ для { $account }?
admin-userlist-confirm-delete-body = Запись об удалённом доступе сохраняется, чтобы пользователь не был зарегистрирован повторно автоматически. Настройки Mailwoman, история и исходный почтовый ящик остаются сохранёнными. Введите адрес для подтверждения.
admin-userlist-confirm-delete-action = Удалить локальный доступ
admin-userlist-confirm-profile = { $count ->
    [one] Изменить профиль функций { $count } пользователя?
    [few] Изменить профиль функций { $count } пользователей?
    [many] Изменить профиль функций { $count } пользователей?
   *[other] Изменить профиль функций { $count } пользователя?
}
admin-userlist-confirm-profile-one = Изменить профиль функций пользователя { $account }?
admin-userlist-confirm-profile-body = Профиль назначается каждому пользователю напрямую и действует со следующего запроса. Поддерживаются встроенные, прокси и SSO-идентификации почтовых ящиков.
admin-users-profile-help = Выберите «По умолчанию», чтобы удалить прямое назначение этого пользователя и использовать профили, унаследованные от его групп или домена.
admin-users-profile-ambiguous = У этого пользователя несколько идентификаций для входа. Выберите нужную идентификацию в разделе «Профили функций».
admin-users-profile-unavailable = Нет подтверждённой идентификации почтового ящика. Пользователь должен войти, прежде чем ему можно будет назначить профиль.
admin-userlist-confirm-profile-action = Изменить профиль
admin-userlist-bulk-done-block = { $count ->
    [one] Вход заблокирован для { $count } пользователя.
    [few] Вход заблокирован для { $count } пользователей.
    [many] Вход заблокирован для { $count } пользователей.
   *[other] Вход заблокирован для { $count } пользователя.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Вход разблокирован для { $count } пользователя.
    [few] Вход разблокирован для { $count } пользователей.
    [many] Вход разблокирован для { $count } пользователей.
   *[other] Вход разблокирован для { $count } пользователя.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] Завершены сеансы { $count } пользователя.
    [few] Завершены сеансы { $count } пользователей.
    [many] Завершены сеансы { $count } пользователей.
   *[other] Завершены сеансы { $count } пользователя.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Удалён локальный доступ для { $count } пользователя.
    [few] Удалён локальный доступ для { $count } пользователей.
    [many] Удалён локальный доступ для { $count } пользователей.
   *[other] Удалён локальный доступ для { $count } пользователя.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Изменён профиль функций { $count } пользователя.
    [few] Изменён профиль функций { $count } пользователей.
    [many] Изменён профиль функций { $count } пользователей.
   *[other] Изменён профиль функций { $count } пользователя.
}
admin-userlist-bulk-failed = Выполнено: { $done }, не удалось: { $failed }: { $reason }

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-user-compliance-due = срок оплаты { $date }

admin-user-compliance-empty = Нет запросов данных для этого аккаунта.

admin-user-compliance-help = Запросы отправлены на адреса этого аккаунта. Откройте один из них в запросах данных, чтобы действовать с ним.

admin-user-compliance-open-all = Открыть все запросы данных

admin-user-compliance-title = Запросы данных для этого аккаунта

admin-user-never = Никогда

admin-user-overview-last-signin = Последний вход

admin-user-overview-status = Статус

admin-user-status-active = Активный

admin-user-status-deleted = Удалено

admin-user-status-deleting = Удаление

admin-user-status-disabled = Заблокировано

admin-user-tab-access = Доступ и удаление

admin-user-tab-compliance = Соответствие

admin-user-tab-overview = Обзор

admin-user-tab-profile = Профиль функции

admin-user-tab-security = Безопасность

admin-user-tab-sessions = Сессии

admin-user-tabs-label = Разделы аккаунта
