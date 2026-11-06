# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Користувачі. Заголовки стовпців сортують список.
admin-userlist-search = Пошук користувачів
admin-userlist-search-placeholder = Ім'я, адреса, псевдонім або домен
admin-userlist-count = { $count ->
    [one] { $count } користувач
    [few] { $count } користувачі
    [many] { $count } користувачів
   *[other] { $count } користувача
}
admin-userlist-count-filtered = { $count ->
    [one] Підходить { $count } із { $total } користувачів
    [few] Підходять { $count } із { $total } користувачів
    [many] Підходять { $count } із { $total } користувачів
   *[other] Підходять { $count } із { $total } користувачів
}
admin-userlist-range = { $from }–{ $to } із { $total }
admin-userlist-loading = Завантаження користувачів…
admin-userlist-retry = Повторити
admin-userlist-empty-filtered = Немає користувачів, що відповідають цьому пошуку та фільтрам.
admin-userlist-compact = Компактні рядки
admin-userlist-columns = Стовпці
admin-userlist-columns-legend = Показувати стовпці
admin-userlist-actions = Дії
admin-userlist-paging = Сторінки користувачів
admin-userlist-page-size = Рядків на сторінці
admin-userlist-prev = Назад
admin-userlist-next = Далі
admin-userlist-sort-by = Сортувати за
admin-userlist-sort-asc = { $column }, за зростанням
admin-userlist-sort-desc = { $column }, за спаданням
admin-userlist-col-user = Користувач
admin-userlist-col-domain = Домен
admin-userlist-col-status = Стан
admin-userlist-col-2fa = Двофакторна автентифікація
admin-userlist-col-role = Роль у консолі
admin-userlist-col-last-signin = Останній вхід
admin-userlist-col-last-seen = Остання активність
admin-userlist-col-created = Створено
admin-userlist-col-quota = Квота
admin-userlist-col-profile = Профіль функцій
admin-userlist-col-linked = Пов'язані скриньки
admin-userlist-state-active = Активний
admin-userlist-state-pending = Ніколи не входив
admin-userlist-state-blocked = Заблокований
admin-userlist-state-disabled = Вимкнений
admin-userlist-state-deleted = Видалений
admin-userlist-2fa-passkey = Ключ доступу
admin-userlist-2fa-totp = Застосунок-автентифікатор
admin-userlist-2fa-none = Немає
admin-userlist-signin-never = Ніколи
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Активність не зафіксовано
admin-userlist-seen-at = Остання активність: { $relative } ({ $exact })
admin-userlist-quota-unlimited = Без обмежень
admin-userlist-profile-default = Типовий
admin-userlist-profile-via-user = Призначено цьому користувачу
admin-userlist-profile-via-group = Із групи
admin-userlist-profile-via-domain = Із домену
admin-userlist-more = Додаткові дії
admin-userlist-more-for = Додаткові дії для { $account }
admin-userlist-select = Вибрати { $account }
admin-userlist-select-page = Вибрати всіх користувачів на цій сторінці
admin-userlist-filters = Фільтри
admin-userlist-add-filter = Додати фільтр
admin-userlist-clear-all = Очистити все
admin-userlist-chip-edit = Змінити фільтр { $label }
admin-userlist-chip-remove = Вилучити фільтр { $label }
admin-userlist-filter-domain = Домен
admin-userlist-filter-status = Стан
admin-userlist-filter-tfa = Двофакторна автентифікація
admin-userlist-filter-role = Роль у консолі
admin-userlist-filter-profile = Профіль функцій
admin-userlist-filter-signin = Останній вхід
admin-userlist-filter-created = Створено
admin-userlist-filter-no-choices = Поки немає з чого вибирати.
admin-userlist-filter-remove = Вилучити фільтр
admin-userlist-apply = Застосувати
admin-userlist-cancel = Скасувати
admin-userlist-role-any = Будь-яка роль у консолі
admin-userlist-role-none = Без ролі в консолі
admin-userlist-profile-none = Без профілю (типовий)
admin-userlist-signin-older = { $days ->
    [one] Понад { $days } день тому
    [few] Понад { $days } дні тому
    [many] Понад { $days } днів тому
   *[other] Понад { $days } дня тому
}
admin-userlist-signin-within = { $days ->
    [one] За останній { $days } день
    [few] За останні { $days } дні
    [many] За останні { $days } днів
   *[other] За останні { $days } дня
}
admin-userlist-signin-custom = Понад указану кількість днів тому:
admin-userlist-signin-days = Дні
admin-userlist-days-invalid = Введіть ціле число днів від 1 до 36500.
admin-userlist-created-from = З
admin-userlist-created-to = По
admin-userlist-created-between = з { $from } по { $to }
admin-userlist-created-since = з { $date }
admin-userlist-created-until = по { $date }
admin-userlist-dates-invalid = Дата початку пізніша за дату закінчення.
admin-userlist-views = Збережені подання
admin-userlist-view-save = Зберегти це подання…
admin-userlist-view-name = Назва подання
admin-userlist-view-save-confirm = Зберегти подання
admin-userlist-view-delete = Видалити подання «{ $name }»
admin-userlist-bulk = Дії над вибраними користувачами
admin-userlist-selected = { $count ->
    [one] Вибрано { $count }
    [few] Вибрано { $count }
    [many] Вибрано { $count }
   *[other] Вибрано { $count }
}
admin-userlist-clear-selection = Скасувати вибір
admin-userlist-block = Заблокувати вхід
admin-userlist-unblock = Розблокувати вхід
admin-userlist-sign-out = Вийти скрізь
admin-userlist-delete = Видалити локальний доступ
admin-userlist-bulk-profile = Призначити профіль функцій
admin-userlist-bulk-profile-clear = Вилучити власний профіль користувача
admin-userlist-confirm-skipped = { $count ->
    [one] Вибраного користувача ({ $count }) буде пропущено: дія до нього не застосовна.
    [few] Вибраних користувачів ({ $count }) буде пропущено: дія до них не застосовна.
    [many] Вибраних користувачів ({ $count }) буде пропущено: дія до них не застосовна.
   *[other] Вибраних користувачів ({ $count }) буде пропущено: дія до них не застосовна.
}
admin-userlist-confirm-block = { $count ->
    [one] Заблокувати вхід для { $count } користувача?
    [few] Заблокувати вхід для { $count } користувачів?
    [many] Заблокувати вхід для { $count } користувачів?
   *[other] Заблокувати вхід для { $count } користувача?
}
admin-userlist-confirm-block-one = Заблокувати вхід для { $account }?
admin-userlist-confirm-block-body = Нові входи відхиляються, а поточні сеанси Mailwoman завершуються. Вихідну поштову скриньку не змінено.
admin-userlist-confirm-block-action = Заблокувати вхід
admin-userlist-confirm-unblock = { $count ->
    [one] Розблокувати вхід для { $count } користувача?
    [few] Розблокувати вхід для { $count } користувачів?
    [many] Розблокувати вхід для { $count } користувачів?
   *[other] Розблокувати вхід для { $count } користувача?
}
admin-userlist-confirm-unblock-one = Розблокувати вхід для { $account }?
admin-userlist-confirm-unblock-body = Локальне блокування входу знімається. Інші обмеження безпеки продовжують діяти.
admin-userlist-confirm-unblock-action = Розблокувати вхід
admin-userlist-confirm-signout = { $count ->
    [one] Завершити скрізь сеанси { $count } користувача?
    [few] Завершити скрізь сеанси { $count } користувачів?
    [many] Завершити скрізь сеанси { $count } користувачів?
   *[other] Завершити скрізь сеанси { $count } користувача?
}
admin-userlist-confirm-signout-one = Завершити скрізь сеанси { $account }?
admin-userlist-confirm-signout-body = Усі відкриті сеанси Mailwoman цих користувачів завершуються. Вони зможуть увійти знову.
admin-userlist-confirm-signout-action = Вийти
admin-userlist-confirm-delete = { $count ->
    [one] Видалити локальний доступ для { $count } користувача?
    [few] Видалити локальний доступ для { $count } користувачів?
    [many] Видалити локальний доступ для { $count } користувачів?
   *[other] Видалити локальний доступ для { $count } користувача?
}
admin-userlist-confirm-delete-one = Видалити локальний доступ для { $account }?
admin-userlist-confirm-delete-body = Запис про видалений доступ зберігається, щоб користувача не було зареєстровано повторно автоматично. Налаштування Mailwoman, історія та вихідна поштова скринька залишаються збереженими. Введіть адресу для підтвердження.
admin-userlist-confirm-delete-action = Видалити локальний доступ
admin-userlist-confirm-profile = { $count ->
    [one] Змінити профіль функцій { $count } користувача?
    [few] Змінити профіль функцій { $count } користувачів?
    [many] Змінити профіль функцій { $count } користувачів?
   *[other] Змінити профіль функцій { $count } користувача?
}
admin-userlist-confirm-profile-one = Змінити профіль функцій користувача { $account }?
admin-userlist-confirm-profile-body = Профіль призначається кожному користувачу напряму й діє з його наступного запиту. Підтримуються вбудовані, проксі та SSO-ідентифікації поштових скриньок.
admin-users-profile-help = Оберіть «Типовий», щоб вилучити пряме призначення цього користувача й використовувати профілі, успадковані від його груп або домену.
admin-users-profile-ambiguous = Цей користувач має кілька ідентифікацій для входу. Оберіть потрібну ідентифікацію в розділі «Профілі функцій».
admin-users-profile-unavailable = Немає підтвердженої ідентифікації поштової скриньки. Користувач має увійти, перш ніж йому можна буде призначити профіль.
admin-userlist-confirm-profile-action = Змінити профіль
admin-userlist-bulk-done-block = { $count ->
    [one] Вхід заблоковано для { $count } користувача.
    [few] Вхід заблоковано для { $count } користувачів.
    [many] Вхід заблоковано для { $count } користувачів.
   *[other] Вхід заблоковано для { $count } користувача.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Вхід розблоковано для { $count } користувача.
    [few] Вхід розблоковано для { $count } користувачів.
    [many] Вхід розблоковано для { $count } користувачів.
   *[other] Вхід розблоковано для { $count } користувача.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] Завершено сеанси { $count } користувача.
    [few] Завершено сеанси { $count } користувачів.
    [many] Завершено сеанси { $count } користувачів.
   *[other] Завершено сеанси { $count } користувача.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Видалено локальний доступ для { $count } користувача.
    [few] Видалено локальний доступ для { $count } користувачів.
    [many] Видалено локальний доступ для { $count } користувачів.
   *[other] Видалено локальний доступ для { $count } користувача.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Змінено профіль функцій { $count } користувача.
    [few] Змінено профіль функцій { $count } користувачів.
    [many] Змінено профіль функцій { $count } користувачів.
   *[other] Змінено профіль функцій { $count } користувача.
}
admin-userlist-bulk-failed = Виконано: { $done }, не вдалося: { $failed }: { $reason }
