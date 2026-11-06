# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Вгору
nextcloud-loading = Завантаження…
nextcloud-list-error = Не вдалося отримати вміст цієї теки.
nextcloud-empty = Ця тека порожня.
nextcloud-file-list = Файли Nextcloud
nextcloud-open-folder = Відкрити теку { $name }
nextcloud-select-file = Вибрати { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Вкласти з Nextcloud
nextcloud-attach-action = { $count ->
    [0] Вкласти
    [one] Вкласти { $count } файл
    [few] Вкласти { $count } файли
    [many] Вкласти { $count } файлів
   *[other] Вкласти { $count } файлу
  }
nextcloud-error-select-file = виберіть принаймні один файл для вкладення
nextcloud-error-attach-failed = не вдалося вкласти вибрані файли

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Зберегти в Nextcloud
nextcloud-save-title = Зберегти «{ $name }» в Nextcloud
nextcloud-save-action = Зберегти тут ({ $dir })
nextcloud-saved = Збережено в { $path }
nextcloud-error-save-failed = не вдалося зберегти в Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Створити посилання для спільного доступу
nextcloud-share-title = Посилання для спільного доступу
nextcloud-share-intro = Створити публічне посилання на { $path } замість вкладення файлу.
nextcloud-protect-password = Захистити паролем
nextcloud-password-label = Пароль
nextcloud-share-password = Пароль доступу
nextcloud-set-expiry = Встановити термін дії
nextcloud-expires-on = Діє до
nextcloud-expiry-date = Дата закінчення терміну дії
nextcloud-create-link = Створити посилання
nextcloud-password-protected = Захищено паролем
nextcloud-no-password = Без пароля
nextcloud-expires = діє до { $date }
nextcloud-no-expiry = безстроково
nextcloud-error-need-password = введіть пароль або вимкніть захист паролем
nextcloud-error-need-expiry = виберіть дату закінчення терміну дії або вимкніть його
nextcloud-error-share-failed = не вдалося створити посилання для спільного доступу

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = Файл { $path } перевищує обмеження на вкладення. Завантажте його до теки «Mail attachments» у вашому Nextcloud і вставте в повідомлення посилання для доступу.
nextcloud-upload-and-link = Завантажити й вставити посилання
nextcloud-share-not-now = Не зараз
nextcloud-upload-progress = Завантаження: { $percent }%
nextcloud-upload-progress-label = Перебіг завантаження
nextcloud-share-inserted = Посилання для доступу до файлу { $name } вставлено в повідомлення.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = Посилання діє до { $date }.
nextcloud-share-body-password = Посилання захищено паролем.
nextcloud-account-title = Облікові записи Nextcloud
nextcloud-account-help = Прив’яжіть свій обліковий запис до екземпляра, схваленого адміністратором. Облікові дані зберігаються в зашифрованому вигляді лише для цього поштового облікового запису.
nextcloud-account-load-error = Не вдалося завантажити зв’язки з Nextcloud. Перезавантажте сторінку, щоб спробувати ще раз.
nextcloud-account-unlink-error = Не вдалося вилучити зв’язок. Перезавантажте сторінку перед повторною спробою.
nextcloud-account-reload = Перезавантажити
nextcloud-account-no-instances = Немає доступних увімкнених екземплярів Nextcloud. Попросіть адміністратора налаштувати хоча б один.
nextcloud-account-ready = Прив’язано й готово
nextcloud-account-stale = Конфігурація екземпляра змінилася. Перевірте прив’язку знову.
nextcloud-account-unlinked = Не прив’язано
nextcloud-account-relink = Перевірити прив’язку знову
nextcloud-account-link = Прив’язати обліковий запис
nextcloud-account-unlink = Відв’язати обліковий запис
nextcloud-account-password-help = В особистих налаштуваннях безпеки Nextcloud створіть пароль застосунку й скопіюйте сюди його ім’я користувача та пароль. Перевірка виконує один обмежений запит; відхилені облікові дані не повторюються.
nextcloud-account-username = Ім’я користувача пароля застосунку Nextcloud
nextcloud-account-password = Пароль застосунку Nextcloud
nextcloud-account-verify = Перевірити й прив’язати
nextcloud-account-verify-error = Перевірка або збереження не вдалися. Перевірте екземпляр, ім’я користувача та пароль застосунку. Поле пароля очищено; наявну робочу прив’язку не замінено.
nextcloud-admin-title = Екземпляри Nextcloud
nextcloud-admin-help = Схваліть екземпляри HTTPS, які користувачі пошти можуть прив’язувати індивідуально. Поштовим обліковим записам не видається спільний пароль служби. Операції з файлами обмежено 8 МіБ. Користувачі нативного рушія після прив’язки явно виявляють і підключають колекції календарів, завдань і адресних книг.
nextcloud-admin-add = Додати екземпляр Nextcloud
nextcloud-admin-edit = Редагувати екземпляр Nextcloud
nextcloud-admin-enabled = Увімкнено
nextcloud-admin-disabled = Вимкнено
nextcloud-admin-missing = Цього екземпляра більше немає. Поверніться до списку екземплярів.
nextcloud-admin-private-help = Для приватних адрес потрібен явний дозвіл на вихідний трафік Nextcloud. Налаштовані маршрути проксі залишаються пріоритетними; відхилений запит через проксі ніколи не переходить на пряме з’єднання.
nextcloud-admin-revalidation = Збереження будь-якої зміни екземпляра вимагає від користувачів повторної перевірки їхніх прив’язок. Вимкнення екземпляра негайно зупиняє нові операції.
nextcloud-admin-authority = Повноваження екземпляра
nextcloud-admin-name = Відображуване ім’я
nextcloud-admin-name-help = Коротка назва, яку бачать користувачі під час вибору екземпляра.
nextcloud-admin-url = URL встановлення HTTPS
nextcloud-admin-url-help = Використовуйте корінь встановлення, наприклад https://cloud.example.org/ або https://example.org/nextcloud/. Сертифікат має бути довіреним і відповідати імені хоста. Облікові дані, параметри запиту та фрагменти відхиляються.
nextcloud-admin-enable = Дозволити користувачам прив’язувати цей екземпляр
nextcloud-admin-ca = Додаткові центри сертифікації (PEM)
nextcloud-admin-ca-help = Необов’язкові сертифікати приватних центрів сертифікації для цього сервера Nextcloud. Публічна довіра залишається доступною, перевірка імені хоста, як і раніше, обов’язкова. Зміни вимагають від користувачів повторної перевірки прив’язок; довіра до сертифікатів проксі задається окремо.
nextcloud-instance-select = Екземпляр Nextcloud
nextcloud-instance-placeholder = Виберіть прив’язаний екземпляр
nextcloud-instances-empty = Немає перевірених прив’язок Nextcloud. Додайте або перевірте прив’язку в розділі «Налаштування користувача» → Nextcloud.
nextcloud-instances-error = Не вдалося завантажити прив’язані екземпляри. Відкрийте вікно вибору знову, щоб спробувати ще раз.
nextcloud-save-warning = Адміністратор вимагає підтвердження, перш ніж це вкладення зможе залишити поштову скриньку.
nextcloud-save-acknowledge = Я розумію попередження про вкладення й хочу зберегти цей файл у вибраному екземплярі Nextcloud.
nextcloud-save-attachment = Зберегти { $name } до Nextcloud
nextcloud-save-close = Закрити вибір місця призначення Nextcloud
nextcloud-save-options = Зберегти вкладення до Nextcloud
nextcloud-pim-title = Контакти та календарі
nextcloud-pim-help = Виявіть колекції з цього перевіреного облікового запису Nextcloud, потім явно підключіть потрібні. Синхронізуйте, щоб отримати актуальні дані; зміни в підключених колекціях застосовуються до віддаленої колекції та завершуються помилкою, якщо її прив’язку чи права відкликано.
nextcloud-pim-category = Категорія колекції
nextcloud-pim-calendar = Календарі
nextcloud-pim-contacts = Адресні книги
nextcloud-pim-tasks = Списки завдань
nextcloud-pim-discover = Виявити колекції
nextcloud-pim-discovered = Доступні колекції
nextcloud-pim-empty = Відповідних колекцій не отримано.
nextcloud-pim-connect = Підключити
nextcloud-pim-connected = Підключені колекції
nextcloud-pim-sync = Синхронізувати зараз
nextcloud-pim-sync-needed = Колекцію підключено. Синхронізуйте зараз, щоб завантажити її вміст.
nextcloud-pim-synced = Синхронізацію завершено.
nextcloud-pim-error = Операція не вдалася. Перевірте прив’язку облікового запису та права, потім повторіть спробу явно. Наявні кешовані дані не видалено.
nextcloud-pim-native-required = Для синхронізації контактів і календаря потрібен нативний поштовий рушій. Доступ до файлів залишається доступним через прив’язку вашого облікового запису.
nextcloud-pim-disconnect = Відключити
nextcloud-pim-disconnect-help = Відключення зберігає кешовані дані та зупиняє запис на віддалений бік. Для заново прив’язаних облікових записів потрібно знову виконати виявлення та явне підключення.
nextcloud-pim-ready = Готово
nextcloud-pim-disconnected = Відключено або прив’язку змінено
nextcloud-pim-unnamed = Колекція без назви
nextcloud-pim-back = Назад до облікових записів Nextcloud
nextcloud-pim-create-title = Нова колекція в Nextcloud
nextcloud-pim-create-help = Створює порожній календар або список завдань у цьому обліковому записі Nextcloud і підключає його тут.
nextcloud-pim-create-name = Назва нової колекції
nextcloud-pim-create = Створити в Nextcloud
nextcloud-pim-created = { $name } створено в Nextcloud і підключено. Синхронізуйте зараз, щоб завантажити її.
nextcloud-pim-interval = Інтервал автоматичного оновлення (секунди)
nextcloud-pim-interval-help = Нуль означає лише ручне оновлення. Додатні інтервали не менші за 60 секунд. Підключені облікові записи оновлюються у фоні; у разі збоїв інтервали збільшуються, а у разі відхилених облікових даних подальші запити припиняються до повторної прив’язки.
nextcloud-pim-interval-reset = Використовувати типове значення адміністратора
nextcloud-admin-empty = Схвалених екземплярів поки немає. Додайте екземпляр, щоб зробити його доступним для прив’язки облікових записів.
nextcloud-admin-status = Стан
nextcloud-admin-open = Відкрити { $name }
