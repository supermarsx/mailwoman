# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Адміністрування Mailwoman
admin-nav = Розділи адміністрування
admin-sign-out = Вийти
admin-nav-domains = Домени
admin-nav-users = Користувачі
admin-nav-integrations = Інтеграції
admin-nav-observability = Моніторинг
admin-nav-plugins = Плагіни
admin-nav-assist = ШІ-помічник
admin-nav-sso = Єдиний вхід (SSO)
admin-nav-servermeta = Метадані сервера
admin-nav-rethread = Обслуговування
admin-nav-2fa = Обов’язкова двофакторна автентифікація

# Shared admin actions / states
admin-delete = Видалити
admin-revoke = Відкликати
admin-remove = Прибрати
admin-saved = Збережено.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Вхід адміністратора
admin-login-note = Ця панель працює в окремому сеансі адміністратора. Сеанс завершується після 30 хвилин бездіяльності і не пізніше ніж через 12 годин після входу.
admin-login-session-ended = Ваш сеанс адміністратора завершився. Увійдіть знову.
admin-login-username = Ім’я користувача адміністратора
admin-login-password = Пароль
admin-login-sign-in = Увійти
admin-login-signing-in = Вхід…
admin-login-invalid = Неправильні облікові дані адміністратора
admin-login-unreachable = Не вдалося зв’язатися із сервером

# -- Domains -----------------------------------------------------------------
admin-domains-title = Домени
admin-domains-load-error = Не вдалося завантажити домени
admin-domains-save-error = Не вдалося зберегти домен
admin-domains-delete-error = Не вдалося видалити домен
admin-domains-add = Додати домен
admin-domains-name = Ім’я домену
admin-domains-name-placeholder = example.com
admin-domains-note = Домен реєструється за іменем. Зареєстровані імена пропонуються на екрані «Обов’язкова двофакторна автентифікація», коли правило стосується одного домену. Реєстрація домену не маршрутизує і не фільтрує пошту.
admin-domains-save = Зберегти домен
admin-domains-empty = Доменів ще немає.
admin-domains-delete-for = Видалити { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Користувачі
admin-users-load-error = Не вдалося завантажити користувачів
admin-users-provision-error = Не вдалося створити користувача
admin-users-flag-error = Не вдалося оновити позначку
admin-users-revoke-error = Не вдалося відкликати сеанси
admin-users-provision = Створити користувача
admin-users-username = Ім’я користувача
admin-users-domain = Домен
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Квота в байтах (0 = без обмежень)
admin-users-quota-msgs = Квота повідомлень (0 = без обмежень)
admin-users-empty = Користувачів ще немає.
admin-users-col-account = Обліковий запис
admin-users-col-quota = Квота (байти/повідомлення)
admin-users-col-zeroaccess = Нульовий доступ
admin-users-col-flags = Позначки
admin-users-col-sessions = Сеанси
admin-users-zeroaccess-for = Нульовий доступ для { $account }
admin-users-zeroaccess-help = Лише запис. Сховище з нульовим доступом вмикає і вимикає сам користувач у власних налаштуваннях своїм ключем. Позначення цього поля не шифрує збережену пошту облікового запису, а зняття позначки не розшифровує її.
admin-users-disable-for = Вимкнути { $account }
admin-users-disabled = вимкнено
admin-users-disabled-help = Блокує вхід і завершує всі відкриті сеанси цього облікового запису. Його ключі API і токени відхиляються. Це не вимикає поштову скриньку на поштовому сервері: інші поштові клієнти можуть і далі до неї звертатися, доки ви не вимкнете її і там.
admin-users-force-change-for = Примусова зміна пароля для { $account }
admin-users-force-change = примусова зміна
admin-users-force-change-help = Користувача буде затримано на екрані зміни пароля — під час наступного входу і в уже відкритих сеансах, — і він не матиме доступу до пошти, доки зміна не вдасться. Для цього потрібен робочий механізм зміни пароля (MW_PASSWD_BACKEND). Без нього зміна не може вдатися, і користувач залишатиметься на цьому екрані, доки ви не знімете цю позначку.
admin-users-revoke-for = Відкликати сеанси для { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Інтеграції
admin-integrations-load-error = Не вдалося завантажити інтеграції
admin-integrations-revoke-error = Не вдалося відкликати ключ
admin-integrations-ldap = Каталог LDAP / GAL
admin-integrations-nextcloud = Міст Nextcloud
admin-integrations-active = Активно
admin-integrations-configured = Налаштовано
admin-integrations-not-configured = Не налаштовано
admin-integrations-unknown = Стан невідомий
admin-integrations-config-note = «Налаштовано» означає, що це розгортання має параметри для інтеграції: увімкнений запис каталогу LDAP у базі даних або три змінні середовища MW_NEXTCLOUD_*. Обидва зчитуються під час запуску сервера. Це не означає, що з віддаленою службою було встановлено зв’язок або що вона доступна.
admin-integrations-webhooks = Вебхуки
admin-integrations-webhooks-empty = Вебхуків не зареєстровано.
admin-integrations-keys = Ключі API і MCP
admin-integrations-keys-empty = Ключів не видано.
admin-integrations-col-prefix = Префікс
admin-integrations-col-account = Обліковий запис
admin-integrations-col-scopes = Області доступу
admin-integrations-col-status = Стан
admin-integrations-status-revoked = відкликано
admin-integrations-status-active = активний
admin-integrations-revoke-key = Відкликати ключ { $prefix }
admin-integrations-col-unattended = Автоматичне надсилання
admin-integrations-unattended-not-requested = Не запитано
admin-integrations-unattended-requested = Запитано, не схвалено
admin-integrations-unattended-approved = Схвалено
admin-integrations-unattended-approve = Схвалити
admin-integrations-unattended-withdraw = Відкликати
admin-integrations-unattended-approve-key = Схвалити автоматичне надсилання для ключа { $prefix }
admin-integrations-unattended-withdraw-key = Відкликати схвалення автоматичного надсилання для ключа { $prefix }
admin-integrations-unattended-approve-title = Схвалити автоматичне надсилання для цього ключа?
admin-integrations-unattended-withdraw-title = Відкликати схвалення для цього ключа?
admin-integrations-unattended-owner = Власник: { $account }
admin-integrations-unattended-key = Ключ: { $prefix }
admin-integrations-unattended-scope = Дозволи: { $permissions }. Інструменти MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = Не вдалося прочитати області доступу цього ключа.
admin-integrations-scope-read = читання
admin-integrations-scope-send = надсилання
admin-integrations-scope-delete = видалення
admin-integrations-scope-mail = пошта
admin-integrations-scope-pim = календар, завдання, нотатки і контакти
admin-integrations-scope-none = немає
admin-integrations-unattended-approve-meaning = Повідомлення, надіслані з цим ключем через MCP, передаються без того, щоб людина їх підтверджувала.
admin-integrations-unattended-withdraw-meaning = Надсилання з цим ключем через MCP відхиляється, доки ключ не буде схвалено знову. Повідомлення не передається і не потрапляє до теки «Вихідні» власника.
admin-integrations-unattended-approve-effect = Схвалення діє одразу, починаючи з наступного надсилання цим ключем. Перезапуск не потрібен.
admin-integrations-unattended-withdraw-effect = Відкликання діє одразу: ключ перестає надсилати автоматично з наступного надсилання. Перезапуск не потрібен.
admin-integrations-unattended-cancel = Скасувати
admin-integrations-unattended-saving = Збереження…
admin-integrations-unattended-approved-done = Автоматичне надсилання для ключа { $prefix } схвалено. Воно вже діє.
admin-integrations-unattended-withdrawn-done = Схвалення для ключа { $prefix } відкликано. Цей ключ більше не надсилає автоматично.
admin-integrations-unattended-error-gone = Ключ { $prefix } більше не існує або його відкликано. Нічого не змінено.
admin-integrations-unattended-error-conflict = Ключ { $prefix } відкликано або його власник не запитував автоматичне надсилання. Нічого не схвалено.
admin-integrations-unattended-error = Не вдалося змінити схвалення для ключа { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Моніторинг
admin-obs-load-error = Не вдалося завантажити дані моніторингу
admin-obs-export-error = Не вдалося експортувати журнал аудиту
admin-obs-ban-add-error = Не вдалося додати блокування
admin-obs-unban-error = Не вдалося зняти блокування
admin-obs-telemetry-note = Журналювання і телеметрія тут не налаштовуються. Фільтр журналу (MW_LOG), збирач OTLP (MW_OTLP_ENDPOINT) і кінцева точка метрик (MW_METRICS_TOKEN) зчитуються із середовища сервера під час його запуску.
admin-obs-audit = Журнал аудиту
admin-obs-export = Експортувати JSONL
admin-obs-audit-empty = Записів аудиту немає.
admin-obs-col-time = Час
admin-obs-col-actor = Виконавець
admin-obs-col-action = Дія
admin-obs-col-target = Об’єкт
admin-obs-bans = Монітор входів / список блокувань
admin-obs-bans-note = Цей список — лише запис. Mailwoman не відхиляє з’єднання з адрес у ньому, і запис ніяк не впливає на вхід. Кожна невдала спроба входу адміністратора записує рядок у журнал сервера, на який може реагувати в’язниця fail2ban, а п’ять невдач з однієї адреси додають цю адресу сюди. Блокування виконує ця в’язниця або ваш брандмауер.
admin-obs-ban-add = Додати блокування
admin-obs-ban-ip = IP-адреса
admin-obs-ban-reason = Причина
admin-obs-ban-btn = Заблокувати IP
admin-obs-bans-empty = Активних блокувань немає.
admin-obs-unban-for = Розблокувати { $ip }
admin-obs-unban-btn = Розблокувати

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Плагіни
admin-plugins-intro = Плагіни рушія працюють у пісочниці WebAssembly з контролем можливостей. Схваліть плагін, перш ніж його можна буде ввімкнути, і надавайте лише ті можливості, які йому потрібні.
admin-plugins-empty = Плагінів не зареєстровано.
admin-plugins-signed = Підписаний
admin-plugins-unsigned = Непідписаний
admin-plugins-approved = Схвалений
admin-plugins-enabled = Увімкнений
admin-plugins-approve = Схвалити
admin-plugins-enable = Увімкнути
admin-plugins-disable = Вимкнути
admin-plugins-net = мережа: { $hosts }
admin-plugins-limits = обмеження: { $memory } МіБ · { $deadline } мс
admin-plugins-limits-fuel = обмеження: { $memory } МіБ · { $deadline } мс · { $fuel } пального
admin-plugins-allow-unsigned-for = Дозволити непідписаний плагін { $name }
admin-plugins-allow-unsigned = Дозволити запуск цього непідписаного плагіна
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = Список дозволених сторонніх плагінів
admin-allowlist-intro = Сторонній компонент завантажується лише після того, як адміністратор схвалить його точний дайджест SHA-256. Дайджест нижче обчислює сервер за байтами компонента на диску; схвалення закріплює саме ці байти. Власні компоненти закріплені під час збирання і тут не керуються.
admin-allowlist-load-error = Не вдалося завантажити список дозволених плагінів.
admin-allowlist-present-heading = Компоненти на диску
admin-allowlist-present-empty = Сторонніх компонентів немає. Помістіть компонент у теку сторонніх плагінів, щоб він з’явився тут.
admin-allowlist-digest-label = Обчислений SHA-256
admin-allowlist-status-approved = Схвалено
admin-allowlist-status-pending = Не схвалено
admin-allowlist-status-firstparty = Власний
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Допущено за закріпленим дайджестом. Цей компонент не має підпису; схвалення довіряє саме тим байтам, дайджест яких показано, — це очікуваний стан для компонента, схваленого таким способом.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = Можливості підвищеного рівня ({ $caps }) не можна надати сторонньому плагіну. Сервер відхиляє їх незалежно від схвалення; вони доступні лише власним компонентам.
admin-allowlist-firstparty-note = Цей ідентифікатор збігається з власним компонентом. Закріплення власного компонента завжди має пріоритет, тому цей ідентифікатор не можна схвалити як сторонній.
admin-allowlist-approve = Схвалити дайджест
admin-allowlist-approve-for = Схвалити дайджест для { $id }
admin-allowlist-revoke = Відкликати
admin-allowlist-revoke-for = Відкликати закріплення для { $id }
admin-allowlist-uninstall = Видалити
admin-allowlist-uninstall-for = Видалити { $id }
admin-allowlist-pins-heading = Схвалені та відкликані закріплення
admin-allowlist-pins-empty = Закріплень не записано.
admin-allowlist-pin-approved-by = Схвалив(-ла) { $by }, { $at }
admin-allowlist-pin-revoked = Відкликано
admin-allowlist-cancel = Скасувати

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Схвалити завантаження цього компонента?
admin-allowlist-approve-detail = Схвалення закріплює саме ті байти, SHA-256 яких показано нижче. Після схвалення для цього ідентифікатора завантажуватимуться лише байти з таким дайджестом; будь-яка зміна компонента дає інший дайджест і не завантажиться до повторного схвалення. Саме схвалення не надає жодних можливостей.
admin-allowlist-approve-confirm = Схвалити дайджест

# Revoke confirmation.
admin-allowlist-revoke-title = Відкликати це закріплення?
admin-allowlist-revoke-detail = Відкликання знімає схвалення цього дайджесту і вимикає плагін. Воно набуває чинності під час наступного завантаження; уже запущений екземпляр до того часу не зупиняється.
admin-allowlist-revoke-confirm = Відкликати закріплення

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Видалити цей плагін?
admin-allowlist-uninstall-detail = Видалення стирає збережені дані ключ/значення плагіна для всіх облікових записів, прибирає його закріплення зі списку дозволених і вимикає його. Файл компонента на диску не видаляється; його можна схвалити знову пізніше.
admin-allowlist-uninstall-confirm = Видалити плагін

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Єдиний вхід (SSO)
admin-sso-intro = Налаштуйте механізми входу OIDC і SAML 2.0. Увімкнені механізми з’являються як кнопки «Увійти через…» на сторінці входу до поштової скриньки — для всього розгортання або для одного домену.
admin-sso-add = Додати механізм входу
admin-sso-edit = Редагувати
admin-sso-create = Додати механізм
admin-sso-update = Зберегти зміни
admin-sso-cancel = Скасувати
admin-sso-empty = Механізмів SSO не налаштовано.
admin-sso-load-error = Не вдалося завантажити механізми SSO.
admin-sso-save-error = Не вдалося зберегти механізм.
admin-sso-delete-error = Не вдалося видалити механізм.

# Common fields
admin-sso-id = Ідентифікатор механізму
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Відображувана назва
admin-sso-display-name-placeholder = Увійти через Acme SSO
admin-sso-kind = Протокол
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Область дії
admin-sso-scope-deployment = Усе розгортання
admin-sso-scope-domain = Один домен
admin-sso-domain = Домен
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Увімкнено
admin-sso-first-login = Політика першого входу
admin-sso-policy-allowlist = Список дозволених (відмовляти невідомим користувачам)
admin-sso-policy-autocreate = Автоматично створювати облікові записи під час першого входу

# OIDC fields
admin-sso-issuer = URL видавця
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = Ідентифікатор клієнта
admin-sso-client-secret = Секрет клієнта
admin-sso-secret-unchanged = Залиште порожнім, щоб зберегти наявний секрет
admin-sso-redirect = URL перенаправлення
admin-sso-scopes = Області доступу
admin-sso-metadata = Метадані SP

# SAML fields
admin-sso-sp-entity-id = Ідентифікатор сутності SP
admin-sso-acs-url = URL ACS
admin-sso-idp-metadata-url = URL метаданих IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML метаданих IdP
admin-sso-idp-metadata-xml-placeholder = Вставте XML метаданих IdP або скористайтеся URL вище
admin-sso-idp-sso-url = URL SSO IdP
admin-sso-idp-slo-url = URL виходу IdP (SLO)
admin-sso-idp-certs = Сертифікати підпису IdP (PEM)
admin-sso-idp-certs-placeholder = Один сертифікат PEM на блок, блоки розділено порожнім рядком
admin-sso-nameid-format = Формат NameID
admin-sso-want-signed = Вимагати підписані твердження
admin-sso-want-encrypted = Вимагати зашифровані твердження

# Claim map
admin-sso-claims = Зіставлення атрибутів
admin-sso-claim-email = Атрибут електронної пошти
admin-sso-claim-username = Атрибут імені користувача
admin-sso-claim-display = Атрибут відображуваного імені
admin-sso-claim-groups = Атрибут груп

# List row
admin-sso-badge-enabled = Увімкнено
admin-sso-badge-disabled = Вимкнено
admin-sso-enable = Увімкнути
admin-sso-disable = Вимкнути
admin-sso-delete = Видалити
# `name` is the backend's admin-set display name.
admin-sso-enable-for = Увімкнути { $name }
admin-sso-disable-for = Вимкнути { $name }
admin-sso-delete-for = Видалити { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = Метадані сервера
admin-servermeta-intro = Перегляд і редагування серверних анотацій RFC 5464 для створеного облікового запису. Зміни записуються безпосередньо на поштовий сервер, який вирішує, чи може обліковий запис їх задавати.
admin-servermeta-account = Обліковий запис
admin-servermeta-select-option = Виберіть обліковий запис…
admin-servermeta-select-prompt = Виберіть обліковий запис, щоб переглянути та змінити його серверні анотації.
admin-servermeta-load-error = Не вдалося завантажити список облікових записів.
admin-servermeta-no-accounts = Немає створених облікових записів.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = Перегрупувати ланцюжки скриньки
admin-rethread-intro = Повторно запускає групування в ланцюжки (JWZ) для збереженої пошти створеного облікового запису і перебудовує ключі групування ланцюжків. Це одноразова дія з обслуговування, а не автоматичний процес.
admin-rethread-account = Обліковий запис
admin-rethread-select-option = Виберіть обліковий запис…
admin-rethread-no-accounts = Немає створених облікових записів.
admin-rethread-load-error = Не вдалося завантажити список облікових записів.
admin-rethread-run = Перегрупувати ланцюжки скриньки
admin-rethread-confirm-title = Перегрупувати ланцюжки цієї скриньки?
admin-rethread-confirm-warning = Перегрупування змінює ключі групування розмов для цього облікового запису. Наявні ланцюжки можуть об’єднатися або розділитися, і користувачі можуть побачити розмови згрупованими по-новому.
admin-rethread-confirm-detail = Це одноразова дія з обслуговування. Вона виконується один раз зараз; її безпечно запускати повторно, і вона не видаляє пошту.
admin-rethread-confirm = Перегрупувати ланцюжки скриньки
admin-rethread-running = Перегрупування…
admin-rethread-cancel = Скасувати
admin-rethread-summary = Повідомлень перегруповано: { $messages }, ланцюжків: { $threads }, облікових записів: { $accounts }; повідомлень перенесено до іншого ланцюжка: { $reassigned }.
admin-rethread-error = Не вдалося виконати перегрупування. Якщо сервер відхилив запит, групування ланцюжків не змінено; перевірте журнали сервера і спробуйте ще раз.

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = Пошуковий індекс
admin-searchindex-intro = Пошук використовує індекс, побудований зі збереженої пошти. Сервер перебудовує його під час запуску, якщо він не відповідає збереженій пошті. Пошта облікових записів із нульовим доступом не індексується, тому пошук нічого не знаходить для таких облікових записів.
admin-searchindex-count = Проіндексовано { $documents } з { $messages } збережених повідомлень.
admin-searchindex-on-disk = Індекс зберігається на диску.
admin-searchindex-in-memory = Індекс зберігається в пам’яті й перебудовується під час кожного запуску сервера.
admin-searchindex-progress = Триває перебудова: оброблено { $done } з { $total } повідомлень.
admin-searchindex-run = Перебудувати пошуковий індекс
admin-searchindex-running = Перебудова…
admin-searchindex-summary = Прочитано збережених повідомлень: { $messages }, облікових записів: { $accounts }. Проіндексовано: { $indexed }, вилучено з індексу: { $removed }, не вдалося прочитати: { $failed }. Облікових записів із нульовим доступом залишено без індексу: { $zeroAccess }.
admin-searchindex-busy = Перебудова вже триває. Дочекайтеся її завершення.
admin-searchindex-error = Не вдалося перебудувати індекс. Індекс зберігає попередній вміст; перевірте журнали сервера і спробуйте ще раз.
admin-searchindex-status-error = Не вдалося отримати стан пошукового індексу.
admin-searchindex-unavailable = Цей сервер працює в режимі проксі й не має пошукового індексу.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
admin-2fa-title = Обов’язкова двофакторна автентифікація
admin-2fa-intro = Вимагати другий фактор (ключ доступу або застосунок-автентифікатор) для входу. Будь-який користувач може налаштувати фактор самостійно; вимога тут змушує облікові записи в її межах налаштувати його під час наступного входу.
admin-2fa-load-error = Не вдалося завантажити політику двофакторної автентифікації.
admin-2fa-save-error = Не вдалося зберегти політику двофакторної автентифікації.
admin-2fa-global = Вимагати двофакторну автентифікацію для всього розгортання
admin-2fa-global-label = Вимагати двофакторну автентифікацію для всього розгортання
admin-2fa-global-note = Коли ввімкнено, кожен обліковий запис повинен мати другий фактор. Правило для домену нижче може також вимагати його для одного домену, не вимагаючи всюди.
admin-2fa-domains-heading = Вимоги для доменів
admin-2fa-domains-empty = Вимог для доменів не задано.
admin-2fa-col-domain = Домен
admin-2fa-col-require = Обов’язково
admin-2fa-require-for = Вимагати двофакторну автентифікацію для { $domain }
admin-2fa-add-domain = Домен
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Вимагати двофакторну автентифікацію для цього домену
admin-2fa-add-require-label = Вимагати двофакторну автентифікацію для цього домену
admin-2fa-add-save = Додати правило для домену

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Вихідний трафік
admin-egress-heading = Маршрути вихідного проксі
admin-egress-intro = Спрямовуйте вихідні запити сервера через проксі HTTP CONNECT або SOCKS5. Mailwoman і далі застосовує власну політику адрес — проксі ніколи не просять розпізнавати імена.
admin-egress-empty = Маршрутів не налаштовано. Вихідні запити йдуть напряму.
admin-egress-none-active = Жоден маршрут не використовується. Вихідні запити йдуть напряму. Збережений маршрут передає трафік лише після того, як ви виберете «Використовувати цей маршрут».
admin-egress-in-use = Вихідні запити використовують маршрут «{ $id }».
admin-egress-deactivate = Припинити використання проксі
admin-egress-activate = Використовувати цей маршрут
admin-egress-activate-for = Використовувати маршрут { $id }
admin-egress-activate-error = Не вдалося перейти на маршрут
admin-egress-deactivate-error = Не вдалося припинити використання маршруту
admin-egress-col-state = Стан
admin-egress-state-active = Використовується
admin-egress-state-inactive = Не використовується
admin-egress-test-scope = Перевірити можна лише маршрут, що використовується. Перевірка завантажує одну фіксовану URL-адресу через маршрут і повідомляє, на якому етапі вона зупинилася; вона нічого не змінює.
admin-egress-probe-url = Завантажено: { $url }
admin-egress-load-error = Не вдалося завантажити маршрути вихідного трафіку
admin-egress-save-error = Не вдалося зберегти маршрут
admin-egress-delete-error = Не вдалося видалити маршрут
admin-egress-invalid = Потрібні ідентифікатор, хост і порт більший за нуль.
admin-egress-username-required = Для пароля потрібне ім’я користувача; без нього маршрут автентифікувався б як ніхто.
admin-egress-col-id = Ідентифікатор
admin-egress-col-route = Маршрут
admin-egress-col-username = Ім’я користувача
admin-egress-col-credentials = Пароль
admin-egress-col-plaintext = Незашифрований трафік
admin-egress-col-actions = Дії
admin-egress-cred-set = Задано
admin-egress-cred-none = Немає
admin-egress-plaintext-on = Дозволено
admin-egress-plaintext-off = Заборонено
admin-egress-add-heading = Додати маршрут
admin-egress-edit-heading = Редагувати маршрут
admin-egress-id = Ідентифікатор
admin-egress-scheme = Схема
admin-egress-host = Хост
admin-egress-port = Порт
admin-egress-username = Ім’я користувача
admin-egress-password = Пароль
admin-egress-password-unchanged = Залиште порожнім, щоб зберегти наявний пароль
admin-egress-allow-plaintext = Дозволити незашифровані джерела http через цей маршрут
admin-egress-allow-plaintext-note = Типово вимкнено. Саме джерело https не дає проксі читати трафік, який він передає; дозвіл http означає, що проксі його бачить.
admin-egress-test = Перевірити
admin-egress-testing = Перевірка…
admin-egress-test-failed = Не вдалося виконати перевірку, тому про цей маршрут нічого не відомо.
admin-egress-outcome-connected = З’єднано
admin-egress-outcome-auth-rejected = Автентифікацію відхилено
admin-egress-outcome-refused-by-policy = Відхилено політикою
admin-egress-outcome-dns-failed = Ім’я не розпізнано
admin-egress-outcome-unreachable = Недосяжно
admin-egress-outcome-origin-tls-failed = Помилка TLS джерела
admin-egress-outcome-route-invalid = Маршрут недійсний
admin-egress-outcome-unknown = Нерозпізнаний результат
admin-egress-stage-dns = зупинилося на розпізнаванні імені
admin-egress-stage-connect = зупинилося під час з’єднання з проксі
admin-egress-stage-tunnel = зупинилося під час відкриття тунелю
admin-egress-stage-origin = зупинилося на джерелі
admin-egress-stage-unknown = зупинилося на нерозпізнаному етапі
admin-egress-proxied-yes = Пройшло через проксі
admin-egress-proxied-no = Не пройшло через проксі
