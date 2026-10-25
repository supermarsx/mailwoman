# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman — администрирование
admin-nav = Разделы администрирования
admin-sign-out = Выйти
admin-nav-domains = Домены
admin-nav-users = Пользователи
admin-nav-integrations = Интеграции
admin-nav-observability = Мониторинг
admin-nav-plugins = Плагины
admin-nav-assist = ИИ-ассистент
admin-nav-sso = Единый вход
admin-nav-servermeta = Метаданные сервера
admin-nav-rethread = Обслуживание
admin-nav-2fa = Обязательная двухфакторная аутентификация

# Shared admin actions / states
admin-delete = Удалить
admin-revoke = Отозвать
admin-remove = Убрать
admin-saved = Сохранено.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Вход администратора
admin-login-note = Эта панель работает в отдельном сеансе администратора. Сеанс завершается после 30 минут бездействия и не позднее чем через 12 часов после входа.
admin-login-session-ended = Сеанс администратора завершён. Войдите снова.
admin-login-username = Имя администратора
admin-login-password = Пароль
admin-login-sign-in = Войти
admin-login-signing-in = Выполняется вход…
admin-login-invalid = Неверные учётные данные администратора
admin-login-unreachable = Не удалось связаться с сервером

# -- Domains -----------------------------------------------------------------
admin-domains-title = Домены
admin-domains-load-error = Не удалось загрузить домены
admin-domains-save-error = Не удалось сохранить домен
admin-domains-delete-error = Не удалось удалить домен
admin-domains-add = Добавить домен
admin-domains-name = Имя домена
admin-domains-name-placeholder = example.com
admin-domains-note = Домен регистрируется по имени. Зарегистрированные имена предлагаются на экране «Обязательная двухфакторная аутентификация», когда правило относится к одному домену. Регистрация домена не влияет на маршрутизацию и фильтрацию почты.
admin-domains-save = Сохранить домен
admin-domains-empty = Доменов пока нет.
admin-domains-delete-for = Удалить { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Пользователи
admin-users-load-error = Не удалось загрузить пользователей
admin-users-provision-error = Не удалось создать пользователя
admin-users-flag-error = Не удалось обновить флаг
admin-users-revoke-error = Не удалось отозвать сеансы
admin-users-provision = Создать пользователя
admin-users-username = Имя пользователя
admin-users-domain = Домен
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Квота в байтах (0 = без ограничений)
admin-users-quota-msgs = Квота в сообщениях (0 = без ограничений)
admin-users-empty = Пользователей пока нет.
admin-users-col-account = Учётная запись
admin-users-col-quota = Квота (байты/сообщения)
admin-users-col-zeroaccess = Нулевой доступ
admin-users-col-flags = Флаги
admin-users-col-sessions = Сеансы
admin-users-zeroaccess-for = Нулевой доступ для { $account }
admin-users-zeroaccess-help = Только отметка. Хранилище с нулевым доступом включает и выключает сам пользователь в своих настройках, своим ключом. Установка этого флажка не шифрует сохранённую почту учётной записи, а снятие не расшифровывает её.
admin-users-disable-for = Отключить { $account }
admin-users-disabled = отключена
admin-users-disabled-help = Запрещает вход и завершает все открытые сеансы этой учётной записи. Её ключи API и токены отклоняются. Почтовый ящик на почтовом сервере при этом не отключается: другие почтовые клиенты могут обращаться к нему, пока вы не отключите его и там.
admin-users-force-change-for = Требовать смену пароля для { $account }
admin-users-force-change = смена пароля
admin-users-force-change-help = Пользователь остаётся на экране смены пароля — при следующем входе и в уже открытых сеансах — и не получит доступ к почте, пока смена не пройдёт успешно. Для этого нужен работающий механизм смены пароля (MW_PASSWD_BACKEND). Без него смена невозможна, и пользователь остаётся на этом экране, пока вы не снимете флажок.
admin-users-revoke-for = Отозвать сеансы для { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Интеграции
admin-integrations-load-error = Не удалось загрузить интеграции
admin-integrations-revoke-error = Не удалось отозвать ключ
admin-integrations-ldap = Каталог LDAP / GAL
admin-integrations-nextcloud = Мост Nextcloud
admin-integrations-active = Активно
admin-integrations-configured = Настроено
admin-integrations-not-configured = Не настроено
admin-integrations-unknown = Состояние неизвестно
admin-integrations-config-note = «Настроено» означает, что в этом развёртывании есть параметры интеграции: включённая запись каталога LDAP в базе данных или три переменные окружения MW_NEXTCLOUD_*. И то и другое считывается при запуске сервера. Это не означает, что с удалённой службой было установлено соединение или что она доступна.
admin-integrations-webhooks = Веб-хуки
admin-integrations-webhooks-empty = Веб-хуки не зарегистрированы.
admin-integrations-keys = Ключи API и MCP
admin-integrations-keys-empty = Ключи не выданы.
admin-integrations-col-prefix = Префикс
admin-integrations-col-account = Учётная запись
admin-integrations-col-scopes = Области действия
admin-integrations-col-status = Статус
admin-integrations-status-revoked = отозван
admin-integrations-status-active = активен
admin-integrations-revoke-key = Отозвать ключ { $prefix }
admin-integrations-col-unattended = Отправка без подтверждения
admin-integrations-unattended-not-requested = Не запрошена
admin-integrations-unattended-requested = Запрошена, не одобрена
admin-integrations-unattended-approved = Одобрена
admin-integrations-unattended-approve = Одобрить
admin-integrations-unattended-withdraw = Отозвать
admin-integrations-unattended-approve-key = Одобрить отправку без подтверждения для ключа { $prefix }
admin-integrations-unattended-withdraw-key = Отозвать одобрение отправки без подтверждения для ключа { $prefix }
admin-integrations-unattended-approve-title = Одобрить отправку без подтверждения для этого ключа?
admin-integrations-unattended-withdraw-title = Отозвать одобрение для этого ключа?
admin-integrations-unattended-owner = Владелец: { $account }
admin-integrations-unattended-key = Ключ: { $prefix }
admin-integrations-unattended-scope = Разрешения: { $permissions }. Инструменты MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = Не удалось прочитать область действия этого ключа.
admin-integrations-scope-read = чтение
admin-integrations-scope-send = отправка
admin-integrations-scope-delete = удаление
admin-integrations-scope-mail = почта
admin-integrations-scope-pim = календарь, задачи, заметки и контакты
admin-integrations-scope-none = нет
admin-integrations-unattended-approve-meaning = Сообщения, отправленные с этим ключом через MCP, передаются без того, чтобы человек их выпускал.
admin-integrations-unattended-withdraw-meaning = Отправка с этим ключом через MCP отклоняется, пока ключ не будет одобрен снова. Сообщение не передаётся и не помещается в «Исходящие» владельца.
admin-integrations-unattended-approve-effect = Одобрение вступает в силу немедленно, со следующей отправки ключа. Перезапуск не нужен.
admin-integrations-unattended-withdraw-effect = Отзыв вступает в силу немедленно: со следующей отправки ключ перестаёт отправлять без подтверждения. Перезапуск не нужен.
admin-integrations-unattended-cancel = Отмена
admin-integrations-unattended-saving = Сохранение…
admin-integrations-unattended-approved-done = Отправка без подтверждения одобрена для ключа { $prefix }. Она уже действует.
admin-integrations-unattended-withdrawn-done = Одобрение для ключа { $prefix } отозвано. Этот ключ больше не отправляет без подтверждения.
admin-integrations-unattended-error-gone = Ключ { $prefix } больше не существует или был отозван. Ничего не изменено.
admin-integrations-unattended-error-conflict = Ключ { $prefix } отозван, или его владелец не запрашивал отправку без подтверждения. Ничего не одобрено.
admin-integrations-unattended-error = Не удалось изменить одобрение для ключа { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Мониторинг
admin-obs-load-error = Не удалось загрузить данные мониторинга
admin-obs-export-error = Не удалось экспортировать журнал аудита
admin-obs-ban-add-error = Не удалось добавить блокировку
admin-obs-unban-error = Не удалось снять блокировку
admin-obs-telemetry-note = Журналирование и телеметрия здесь не настраиваются. Фильтр журнала (MW_LOG), сборщик OTLP (MW_OTLP_ENDPOINT) и конечная точка метрик (MW_METRICS_TOKEN) считываются из окружения сервера при его запуске.
admin-obs-audit = Журнал аудита
admin-obs-export = Экспорт в JSONL
admin-obs-audit-empty = Записей аудита нет.
admin-obs-col-time = Время
admin-obs-col-actor = Кто
admin-obs-col-action = Действие
admin-obs-col-target = Объект
admin-obs-bans = Мониторинг входов / список блокировок
admin-obs-bans-note = Этот список — только запись. Mailwoman не отклоняет подключения с указанных в нём адресов, и запись никак не влияет на вход. Каждая неудачная попытка входа администратора записывает в журнал сервера строку, на которую может реагировать правило fail2ban, а пять неудач с одного адреса добавляют этот адрес сюда. Блокировку выполняет это правило или ваш межсетевой экран.
admin-obs-ban-add = Добавить блокировку
admin-obs-ban-ip = IP-адрес
admin-obs-ban-reason = Причина
admin-obs-ban-btn = Заблокировать IP
admin-obs-bans-empty = Активных блокировок нет.
admin-obs-unban-for = Разблокировать { $ip }
admin-obs-unban-btn = Разблокировать

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Плагины
admin-plugins-intro = Плагины движка работают в песочнице WebAssembly с контролем возможностей. Одобрите плагин, прежде чем его можно будет включить, и предоставляйте только те возможности, которые ему нужны.
admin-plugins-empty = Плагины не зарегистрированы.
admin-plugins-signed = Подписан
admin-plugins-unsigned = Не подписан
admin-plugins-approved = Одобрен
admin-plugins-enabled = Включён
admin-plugins-approve = Одобрить
admin-plugins-enable = Включить
admin-plugins-disable = Отключить
admin-plugins-net = сеть: { $hosts }
admin-plugins-limits = ограничения: { $memory } МиБ · { $deadline } мс
admin-plugins-limits-fuel = ограничения: { $memory } МиБ · { $deadline } мс · { $fuel } ед. топлива
admin-plugins-allow-unsigned-for = Разрешить неподписанный плагин { $name }
admin-plugins-allow-unsigned = Разрешить запуск этого неподписанного плагина
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = Список разрешённых сторонних плагинов
admin-allowlist-intro = Сторонний компонент загружается только после того, как администратор одобрит его точный хеш SHA-256. Хеш ниже вычисляется сервером по байтам компонента на диске; одобрение закрепляет именно эти байты. Собственные компоненты закреплены при сборке и здесь не управляются.
admin-allowlist-load-error = Не удалось загрузить список разрешённых плагинов.
admin-allowlist-present-heading = Компоненты на диске
admin-allowlist-present-empty = Сторонних компонентов нет. Поместите компонент в каталог сторонних плагинов, чтобы он появился здесь.
admin-allowlist-digest-label = Вычисленный SHA-256
admin-allowlist-status-approved = Одобрен
admin-allowlist-status-pending = Не одобрен
admin-allowlist-status-firstparty = Собственный
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Допущен по закреплённому хешу. У этого компонента нет подписи; одобрение доверяет именно тем байтам, хеш которых показан, — для компонента, одобренного таким способом, это ожидаемо.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = Расширенные возможности ({ $caps }) нельзя предоставить стороннему плагину. Сервер отклоняет их независимо от одобрения; они доступны только собственным компонентам.
admin-allowlist-firstparty-note = Этот идентификатор совпадает с собственным компонентом. Закрепление собственного компонента всегда имеет приоритет, поэтому этот идентификатор нельзя одобрить как сторонний.
admin-allowlist-approve = Одобрить хеш
admin-allowlist-approve-for = Одобрить хеш для { $id }
admin-allowlist-revoke = Отозвать
admin-allowlist-revoke-for = Отозвать закрепление для { $id }
admin-allowlist-uninstall = Удалить
admin-allowlist-uninstall-for = Удалить { $id }
admin-allowlist-pins-heading = Одобренные и отозванные закрепления
admin-allowlist-pins-empty = Закреплений нет.
admin-allowlist-pin-approved-by = Одобрено: { $by }, { $at }
admin-allowlist-pin-revoked = Отозвано
admin-allowlist-cancel = Отмена

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Разрешить загрузку этого компонента?
admin-allowlist-approve-detail = Одобрение закрепляет именно те байты, SHA-256 которых показан ниже. После одобрения для этого идентификатора будут загружаться только байты с этим хешем; любое изменение компонента даёт другой хеш, и он не загрузится до повторного одобрения. Само по себе одобрение не предоставляет никаких возможностей.
admin-allowlist-approve-confirm = Одобрить хеш

# Revoke confirmation.
admin-allowlist-revoke-title = Отозвать это закрепление?
admin-allowlist-revoke-detail = Отзыв снимает одобрение этого хеша и отключает плагин. Это вступает в силу при следующей загрузке; уже работающий экземпляр до тех пор не останавливается.
admin-allowlist-revoke-confirm = Отозвать закрепление

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Удалить этот плагин?
admin-allowlist-uninstall-detail = Удаление стирает сохранённые данные ключ/значение плагина для всех учётных записей, убирает его закрепления в списке разрешённых и отключает его. Файл компонента на диске не удаляется; позже его можно одобрить снова.
admin-allowlist-uninstall-confirm = Удалить плагин

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Единый вход
admin-sso-intro = Настройте механизмы входа OIDC и SAML 2.0. Включённые механизмы отображаются кнопками «Войти через…» на странице входа в почтовый ящик — для всего развёртывания или для одного домена.
admin-sso-add = Добавить механизм входа
admin-sso-edit = Изменить
admin-sso-create = Добавить механизм
admin-sso-update = Сохранить изменения
admin-sso-cancel = Отмена
admin-sso-empty = Механизмы SSO не настроены.
admin-sso-load-error = Не удалось загрузить механизмы SSO.
admin-sso-save-error = Не удалось сохранить механизм.
admin-sso-delete-error = Не удалось удалить механизм.

# Common fields
admin-sso-id = Идентификатор механизма
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Отображаемое имя
admin-sso-display-name-placeholder = Войти через Acme SSO
admin-sso-kind = Протокол
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Область действия
admin-sso-scope-deployment = Всё развёртывание
admin-sso-scope-domain = Один домен
admin-sso-domain = Домен
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Включён
admin-sso-first-login = Политика первого входа
admin-sso-policy-allowlist = Список разрешённых (неизвестным пользователям отказывать)
admin-sso-policy-autocreate = Автоматически создавать учётные записи при первом входе

# OIDC fields
admin-sso-issuer = URL издателя
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = Идентификатор клиента
admin-sso-client-secret = Секрет клиента
admin-sso-secret-unchanged = Оставьте пустым, чтобы сохранить текущий секрет
admin-sso-redirect = URL перенаправления
admin-sso-scopes = Области (scopes)
admin-sso-metadata = Метаданные SP

# SAML fields
admin-sso-sp-entity-id = Идентификатор сущности SP
admin-sso-acs-url = URL ACS
admin-sso-idp-metadata-url = URL метаданных IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML метаданных IdP
admin-sso-idp-metadata-xml-placeholder = Вставьте XML метаданных IdP или используйте URL выше
admin-sso-idp-sso-url = URL SSO для IdP
admin-sso-idp-slo-url = URL выхода (SLO) для IdP
admin-sso-idp-certs = Сертификаты подписи IdP (PEM)
admin-sso-idp-certs-placeholder = Один сертификат PEM на блок, блоки разделяются пустой строкой
admin-sso-nameid-format = Формат NameID
admin-sso-want-signed = Требовать подписанные утверждения
admin-sso-want-encrypted = Требовать зашифрованные утверждения

# Claim map
admin-sso-claims = Сопоставление утверждений (claims)
admin-sso-claim-email = Утверждение с адресом электронной почты
admin-sso-claim-username = Утверждение с именем пользователя
admin-sso-claim-display = Утверждение с отображаемым именем
admin-sso-claim-groups = Утверждение с группами

# List row
admin-sso-badge-enabled = Включён
admin-sso-badge-disabled = Отключён
admin-sso-enable = Включить
admin-sso-disable = Отключить
admin-sso-delete = Удалить
# `name` is the backend's admin-set display name.
admin-sso-enable-for = Включить { $name }
admin-sso-disable-for = Отключить { $name }
admin-sso-delete-for = Удалить { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = Метаданные сервера
admin-servermeta-intro = Просмотр и изменение серверных аннотаций RFC 5464 для созданной учётной записи. Изменения записываются прямо на почтовый сервер, который решает, может ли учётная запись их задавать.
admin-servermeta-account = Учётная запись
admin-servermeta-select-option = Выберите учётную запись…
admin-servermeta-select-prompt = Выберите учётную запись, чтобы просмотреть и изменить её серверные аннотации.
admin-servermeta-load-error = Не удалось загрузить список учётных записей.
admin-servermeta-no-accounts = Учётные записи не созданы.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = Перестроить цепочки почтового ящика
admin-rethread-intro = Повторно выполняет группировку в цепочки (JWZ) для сохранённой почты созданной учётной записи и заново назначает ключи группировки цепочек. Это разовое действие по обслуживанию, оно не выполняется автоматически.
admin-rethread-account = Учётная запись
admin-rethread-select-option = Выберите учётную запись…
admin-rethread-no-accounts = Учётные записи не созданы.
admin-rethread-load-error = Не удалось загрузить список учётных записей.
admin-rethread-run = Перестроить цепочки
admin-rethread-confirm-title = Перестроить цепочки этого почтового ящика?
admin-rethread-confirm-warning = Перестроение заново назначает ключи группировки цепочек для этой учётной записи. Существующие цепочки могут объединиться или разделиться, и пользователи могут увидеть перегруппированные цепочки.
admin-rethread-confirm-detail = Это разовое действие по обслуживанию. Оно выполняется один раз сейчас; его можно безопасно повторить, и оно не удаляет почту.
admin-rethread-confirm = Перестроить цепочки
admin-rethread-running = Перестроение…
admin-rethread-cancel = Отмена
admin-rethread-summary = Сообщений распределено: { $messages }, цепочек: { $threads }, учётных записей: { $accounts }; перенесено в другую цепочку сообщений: { $reassigned }.
admin-rethread-error = Перестроение цепочек не удалось. Если сервер отклонил запрос, группировка цепочек не изменилась; проверьте журналы сервера и повторите попытку.

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = Поисковый индекс
admin-searchindex-intro = Поиск использует индекс, построенный по сохранённой почте. Сервер перестраивает его при запуске, если он не соответствует сохранённой почте. Почта учётных записей с нулевым доступом не индексируется, поэтому поиск по таким учётным записям ничего не находит.
admin-searchindex-count = Проиндексировано { $documents } из { $messages } сохранённых сообщений.
admin-searchindex-on-disk = Индекс хранится на диске.
admin-searchindex-in-memory = Индекс хранится в памяти и перестраивается при каждом запуске сервера.
admin-searchindex-progress = Выполняется перестроение: обработано { $done } из { $total } сообщений.
admin-searchindex-run = Перестроить поисковый индекс
admin-searchindex-running = Перестроение…
admin-searchindex-summary = Прочитано сохранённых сообщений: { $messages }, учётных записей: { $accounts }. Проиндексировано: { $indexed }, удалено из индекса: { $removed }, не удалось прочитать: { $failed }. Учётных записей с нулевым доступом не проиндексировано: { $zeroAccess }.
admin-searchindex-busy = Перестроение уже выполняется. Дождитесь его завершения.
admin-searchindex-error = Перестроение не удалось. Индекс сохраняет прежнее содержимое; проверьте журналы сервера и повторите попытку.
admin-searchindex-status-error = Не удалось получить состояние поискового индекса.
admin-searchindex-unavailable = Этот сервер работает в режиме прокси, и поискового индекса у него нет.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
admin-2fa-title = Обязательная двухфакторная аутентификация
admin-2fa-intro = Требовать второй фактор (ключ доступа или приложение-аутентификатор) при входе. Любой пользователь может сам подключить второй фактор; требование здесь обязывает учётные записи в области действия подключить его при следующем входе.
admin-2fa-load-error = Не удалось загрузить политику двухфакторной аутентификации.
admin-2fa-save-error = Не удалось сохранить политику двухфакторной аутентификации.
admin-2fa-global = Требовать двухфакторную аутентификацию для всего развёртывания
admin-2fa-global-label = Требовать двухфакторную аутентификацию для всего развёртывания
admin-2fa-global-note = Если включено, у каждой учётной записи должен быть второй фактор. Правило для домена ниже может также требовать его для одного домена, не требуя везде.
admin-2fa-domains-heading = Требования для доменов
admin-2fa-domains-empty = Требования для доменов не заданы.
admin-2fa-col-domain = Домен
admin-2fa-col-require = Обязательно
admin-2fa-require-for = Требовать двухфакторную аутентификацию для { $domain }
admin-2fa-add-domain = Домен
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Требовать двухфакторную аутентификацию для этого домена
admin-2fa-add-require-label = Требовать двухфакторную аутентификацию для этого домена
admin-2fa-add-save = Добавить правило для домена

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Исходящий трафик
admin-egress-heading = Маршруты исходящего прокси
admin-egress-intro = Направляйте исходящие запросы сервера через прокси HTTP CONNECT или SOCKS5. Mailwoman по-прежнему применяет собственную политику адресов — прокси никогда не просят разрешить имя.
admin-egress-empty = Маршруты не настроены. Исходящие запросы идут напрямую.
admin-egress-none-active = Ни один маршрут не используется. Исходящие запросы идут напрямую. Сохранённый маршрут начинает передавать трафик только после того, как вы выберете «Использовать этот маршрут».
admin-egress-in-use = Исходящие запросы используют маршрут «{ $id }».
admin-egress-deactivate = Прекратить использовать прокси
admin-egress-activate = Использовать этот маршрут
admin-egress-activate-for = Использовать маршрут { $id }
admin-egress-activate-error = Не удалось переключиться на маршрут
admin-egress-deactivate-error = Не удалось прекратить использование маршрута
admin-egress-col-state = Состояние
admin-egress-state-active = Используется
admin-egress-state-inactive = Не используется
admin-egress-test-scope = Проверить можно только используемый маршрут. Проверка загружает один фиксированный URL через маршрут и сообщает, на каком этапе остановилась; она ничего не меняет.
admin-egress-probe-url = Загружено: { $url }
admin-egress-load-error = Не удалось загрузить маршруты исходящего трафика
admin-egress-save-error = Не удалось сохранить маршрут
admin-egress-delete-error = Не удалось удалить маршрут
admin-egress-invalid = Требуются идентификатор, хост и порт больше нуля.
admin-egress-username-required = Для пароля нужно имя пользователя; без него маршрут проходил бы аутентификацию без пользователя.
admin-egress-col-id = Идентификатор
admin-egress-col-route = Маршрут
admin-egress-col-username = Имя пользователя
admin-egress-col-credentials = Пароль
admin-egress-col-plaintext = Без шифрования
admin-egress-col-actions = Действия
admin-egress-cred-set = Задан
admin-egress-cred-none = Нет
admin-egress-plaintext-on = Разрешено
admin-egress-plaintext-off = Запрещено
admin-egress-add-heading = Добавить маршрут
admin-egress-edit-heading = Изменить маршрут
admin-egress-id = Идентификатор
admin-egress-scheme = Схема
admin-egress-host = Хост
admin-egress-port = Порт
admin-egress-username = Имя пользователя
admin-egress-password = Пароль
admin-egress-password-unchanged = Оставьте пустым, чтобы сохранить текущий пароль
admin-egress-allow-plaintext = Разрешить нешифрованные источники http через этот маршрут
admin-egress-allow-plaintext-note = По умолчанию выключено. Источник https не позволяет прокси читать передаваемый трафик; разрешение http означает, что прокси его видит.
admin-egress-test = Проверить
admin-egress-testing = Проверка…
admin-egress-test-failed = Не удалось выполнить проверку, поэтому о маршруте ничего не известно.
admin-egress-outcome-connected = Подключено
admin-egress-outcome-auth-rejected = Аутентификация отклонена
admin-egress-outcome-refused-by-policy = Отклонено политикой
admin-egress-outcome-dns-failed = Имя не разрешилось
admin-egress-outcome-unreachable = Недоступно
admin-egress-outcome-origin-tls-failed = Ошибка TLS на источнике
admin-egress-outcome-route-invalid = Маршрут недействителен
admin-egress-outcome-unknown = Нераспознанный результат
admin-egress-stage-dns = остановлено на разрешении имени
admin-egress-stage-connect = остановлено при подключении к прокси
admin-egress-stage-tunnel = остановлено при открытии туннеля
admin-egress-stage-origin = остановлено на источнике
admin-egress-stage-unknown = остановлено на нераспознанном этапе
admin-egress-proxied-yes = Прошло через прокси
admin-egress-proxied-no = Не прошло через прокси
