# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Попередження про непідписаний плагін інтерфейсу
plugins-unsigned-title = Працюють непідписані плагіни
plugins-unsigned-text = Ці плагіни допущено без перевіреного підпису. Вони працюють в ізольованому середовищі, але їхній код не перевірено за довіреним ключем.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Плагін: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Плагіни рушія працюють в ізольованому середовищі WebAssembly. Плагін запускається лише після того, як його зареєстровано, схвалено, надано принаймні одну можливість і ввімкнено. Він може використовувати лише можливості, надані тут.
plugins-admin-unsigned-banner = Завантажено сторонній плагін без підпису. Він працює, бо адміністратор це дозволив; сервер перевіряє його файл лише за схваленим дайджестом.
plugins-admin-load-error = Не вдалося завантажити плагіни.
plugins-admin-register-heading = Реєстрація плагіна
plugins-admin-register-component = Компонент
plugins-admin-register-third-party = Сторонній компонент
plugins-admin-register-id = Ідентифікатор плагіна
plugins-admin-register-id-hint = Сервер читає компонент із файлу, названого за цим ідентифікатором, у теці сторонніх плагінів. Спершу схваліть дайджест цього файлу в списку дозволених нижче.
plugins-admin-register-name = Назва
plugins-admin-register-version = Версія
plugins-admin-register-capabilities = Можливості, які оголошує плагін
plugins-admin-register-hosts = Хости, до яких плагін може звертатися
plugins-admin-register-hosts-hint-first-party = Імена хостів через кому. Залиште порожнім, щоб використати типові хости компонента.
plugins-admin-register-hosts-hint-third-party = Імена хостів через кому. Залиште порожнім, щоб заборонити доступ до мережі.
plugins-admin-register-submit = Зареєструвати
plugins-admin-trust-first-party = Власний, дайджест вбудовано
plugins-admin-trust-third-party = Сторонній, дайджест схвалено
plugins-admin-status-loaded = Завантажено
plugins-admin-status-not-loaded = Не завантажено
plugins-admin-status-restart = Потрібен перезапуск
plugins-admin-restart-to-load = Щоб завантажити цей плагін, потрібен перезапуск.
plugins-admin-restart-to-apply = Щоб застосувати ці зміни, потрібен перезапуск. До того часу частина облікових записів цього моста не обслуговується.
plugins-admin-reason-not-approved = Не завантажено: адміністратор не схвалив цей плагін.
plugins-admin-reason-disabled = Не завантажено: плагін не ввімкнено.
plugins-admin-reason-unsigned-not-allowed = Не завантажено: плагін не має підпису, і йому не дозволено працювати без нього.
plugins-admin-reason-no-grant = Не завантажено: не надано жодної потрібної йому можливості.
plugins-admin-reason-component-unavailable = Не завантажено: жоден файл компонента не пройшов перевірку дайджесту.
plugins-admin-reason-load-failed = Не завантажено: сервер не зміг завантажити компонент. Див. журнал сервера.
plugins-admin-reason-proxy-mode = Не завантажено: цей сервер працює в режимі проксі й сам не обробляє пошту.
plugins-admin-reason-no-account-binding = Не завантажено: до цього моста не прив'язано жодного облікового запису.
plugins-admin-reason-no-host-caller = Не завантажено: ця версія сервера не викликає хуки, які надає цей плагін.
plugins-admin-reason-another-classifier-active = Не завантажено: завантажено інший класифікатор спаму. Одночасно працює лише один.
plugins-admin-running-with = Працює з можливостями: { $capabilities }
plugins-admin-hosts = Хости: { $hosts }
plugins-admin-hosts-none = Хости: немає
plugins-admin-grants-heading = Надані можливості
plugins-admin-grants-hint = Плагін може використовувати лише позначені тут можливості. Збереження замінює попередній набір.
plugins-admin-grant-for = Надати { $capability } для { $name }
plugins-admin-grants-save = Зберегти дозволи
plugins-admin-endpoint = Адреса класифікатора
plugins-admin-endpoint-for = Адреса класифікатора для { $name }
plugins-admin-endpoint-hint = Адреса, за якою плагін звертається до своєї служби. Хост також має бути у списку хостів плагіна. Залиште порожнім, щоб використати типову адресу компонента.
plugins-admin-endpoint-save = Зберегти адресу
plugins-admin-test = Перевірити класифікатор
plugins-admin-test-verdict = Результат перевірки: { $verdict }
plugins-admin-uninstall = Видалити
plugins-admin-uninstall-confirm = Підтвердити видалення
plugins-admin-uninstall-note = Видалення вилучає реєстрацію, надані можливості та збережені налаштування.
plugins-admin-uninstall-detail = Видалення стирає збережені дані плагіна для всіх облікових записів, його закріплення в списку дозволених, реєстрацію та надані можливості. Файл компонента на диску не видаляється.
plugins-admin-revoke-detail = Відкликання скасовує схвалення цього дайджесту й вимикає плагін. Якщо плагін працює, його буде негайно зупинено.
plugins-admin-error-already-registered = Цей плагін уже зареєстровано. Видаліть його, перш ніж реєструвати знову.
plugins-admin-error-component-unavailable = Жоден файл компонента цього плагіна не відповідає дайджесту, вбудованому в сервер.
plugins-admin-error-digest-not-approved = Для цього ідентифікатора не знайдено файлу компонента зі схваленим дайджестом. Спершу схваліть дайджест файлу в списку дозволених.
plugins-admin-error-not-approved = Схваліть плагін, перш ніж вмикати його.
plugins-admin-error-unsigned-not-allowed = Цей плагін не має підпису. Дозвольте йому працювати без підпису, перш ніж вмикати його.
plugins-admin-error-not-loaded = Плагін не завантажено, тому його не можна перевірити.
plugins-admin-error-classifier-error = Помилка виклику класифікатора: { $detail }
plugins-admin-error-other = Сервер відхилив запит: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = Плагіни інтерфейсу
plugins-admin-ui-title = Плагіни інтерфейсу
plugins-admin-ui-intro = Плагіни інтерфейсу додають елементи керування до вебклієнта. Кожен працює в ізольованому фреймі й може використовувати лише можливості, надані тут. Плагін стає доступним користувачам, що увійшли, після схвалення та ввімкнення.
plugins-admin-ui-empty = Плагінів інтерфейсу не зареєстровано.
plugins-admin-ui-load-error = Не вдалося завантажити плагіни інтерфейсу.
plugins-admin-ui-register-heading = Реєстрація плагіна інтерфейсу
plugins-admin-ui-manifest = Маніфест (JSON)
plugins-admin-ui-bundle = Файл пакета
plugins-admin-ui-bundle-hint = Обов'язковий для підписаного плагіна: підпис перевіряється за цим файлом.
plugins-admin-ui-allow-unsigned = Дозволити реєстрацію цього плагіна без підпису
plugins-admin-ui-register = Зареєструвати
plugins-admin-ui-manifest-invalid = Маніфест не є коректним JSON.
plugins-admin-ui-refused = Сервер відхилив запит: { $detail }
plugins-admin-ui-approve = Схвалити й увімкнути
plugins-admin-ui-extension-points = Точки розширення: { $points }
plugins-admin-ui-grants-heading = Надати можливість
plugins-admin-ui-grants-note = Сервер не повідомляє, які можливості вже надано. Повторне надання можливості замінює попереднє.
plugins-admin-ui-grant = Надати { $capability }
plugins-admin-ui-grant-for = Надати { $capability } для { $name }
plugins-admin-ui-grant-hosts = Хости для { $capability } у { $name }
plugins-admin-ui-grant-hosts-hint = Імена хостів через кому, з яких плагін може отримувати дані.
plugins-admin-ui-granted = Надано { $capability }.
plugins-admin-ui-delete = Видалити
plugins-admin-ui-delete-confirm = Підтвердити видалення
