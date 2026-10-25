# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Панель
admin-shell-group-overview = Огляд
admin-shell-group-people = Люди
admin-shell-group-security = Безпека
admin-shell-group-configuration = Конфігурація
admin-shell-group-operations = Експлуатація
admin-shell-menu = Меню
admin-shell-close-menu = Закрити меню
admin-shell-search = Пошук
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Ви ввійшли як { $name }
admin-shell-language-label = Мова
admin-shell-breadcrumb = Навігаційний ланцюжок
admin-shell-unknown-section = Цього розділу не існує, або ваша роль його не охоплює.
admin-shell-back-to-dashboard = Назад до панелі
admin-shell-cap-missing = Ваша роль не містить дозволу { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Завантаження…
admin-shell-retry = Спробувати ще раз
admin-shell-cancel = Скасувати
admin-shell-save = Зберегти
admin-shell-not-available = Недоступно

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Залишити цю сторінку?
admin-shell-leave-body = Зміни на цій сторінці не збережено. Якщо ви її залишите, їх буде відкинуто.
admin-shell-leave-confirm = Залишити й відкинути зміни

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Введіть { $name }, щоб підтвердити

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Пошук у консолі адміністратора
admin-shell-palette-label = Розділ, користувач, домен або параметр
admin-shell-palette-results = Результати
admin-shell-palette-none = Нічого не знайдено.
admin-shell-palette-searching = Пошук користувачів, доменів і параметрів…
admin-shell-palette-sections-only = Цей сервер поки не вміє шукати користувачів, домени та параметри, тому показано лише розділи.
admin-shell-palette-error = Не вдалося виконати пошук користувачів, доменів і параметрів; показано лише розділи.
admin-shell-palette-kind-section = Розділ
admin-shell-palette-kind-user = Користувач
admin-shell-palette-kind-domain = Домен
admin-shell-palette-kind-setting = Параметр

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Фільтри
admin-shell-table-actions = Дії
admin-shell-table-error = Не вдалося завантажити список.
admin-shell-table-paging = Сторінки таблиці { $table }
admin-shell-table-prev = Попередня сторінка
admin-shell-table-next = Наступна сторінка

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Останнє { $latest }, найбільше { $max }
admin-shell-chart-col-time = Час
admin-shell-chart-col-item = Елемент
admin-shell-chart-col-value = Значення

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Задано через
admin-shell-pinned-sr = Це значення надходить із середовища сервера, і його не можна змінити тут.
admin-shell-lock-text = Заблокувати для користувачів
admin-shell-lock-setting = { $setting }
admin-shell-restore = Відновити попереднє значення
admin-shell-restore-note = Змінив(-ла) { $actor }, { $time }. Попереднє значення: { $value }
admin-shell-restore-done = Попереднє значення відновлено.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Кроки
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Крок { $n } з { $total }: { $title }
admin-shell-wizard-review = Перевірка
admin-shell-wizard-back = Назад
admin-shell-wizard-next = Далі
admin-shell-wizard-saved = Збережено.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Не вдалося зв’язатися із сервером. Перевірте з’єднання і спробуйте ще раз.
admin-shell-err-session = Ваш сеанс адміністратора завершився. Увійдіть знову.
admin-shell-err-forbidden = Ваша роль не дозволяє цю дію. Зверніться до адміністратора з відповідною роллю.
admin-shell-err-out-of-scope = Цей об’єкт належить домену, який ваша роль не охоплює.
admin-shell-err-not-found = Цього вже не існує. Перезавантажте список.
admin-shell-err-conflict = Тим часом хтось це змінив. Перезавантажте і спробуйте ще раз.
admin-shell-err-validation = Сервер відхилив значення. Перевірте поля і спробуйте ще раз.
admin-shell-err-rate-limited = Забагато запитів. Зачекайте хвилину і спробуйте ще раз.
admin-shell-err-server = Сервер відповів помилкою ({ $status }). Спробуйте ще раз; якщо це повториться, перевірте журнал сервера.
admin-shell-err-upstream-dns = Не вдалося розпізнати ім’я хоста. Перевірте написання хоста.
admin-shell-err-upstream-connect = На цьому хості й порту ніхто не відповів. Перевірте порт і брандмауери між ними.
admin-shell-err-upstream-tls = Сервер відповів, але рукостискання TLS не вдалося. Перевірте порт і параметр безпеки з’єднання.
admin-shell-err-upstream-auth = Сервер відхилив облікові дані. Перевірте ім’я користувача і пароль.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Забагато спроб входу. Зачекайте кілька хвилин і спробуйте ще раз.
admin-shell-totp-form = Вхід адміністратора, другий крок
admin-shell-totp-note = Введіть 6-значний код із застосунку-автентифікатора, налаштованого для { $name }.
admin-shell-totp-code = Код автентифікації
admin-shell-totp-verify = Перевірити
admin-shell-totp-start-over = Почати знову
admin-shell-totp-invalid = Цей код недійсний. Введіть поточний код із застосунку-автентифікатора.
admin-shell-totp-expired = Вхід тривав занадто довго. Почніть знову і введіть пароль.
