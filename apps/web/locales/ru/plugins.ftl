# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Предупреждение о неподписанном UI-плагине
plugins-unsigned-title = Запущены неподписанные плагины
plugins-unsigned-text = Эти плагины допущены без проверенной подписи. Они работают в изолированной среде, но их код не сверен с доверенным ключом.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Плагин: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Плагины движка работают в изолированной среде WebAssembly. Плагин запускается только после того, как он зарегистрирован, одобрен, получил хотя бы одну возможность и включён. Он может использовать только возможности, предоставленные здесь.
plugins-admin-unsigned-banner = Загружен сторонний плагин без подписи. Он работает, потому что это разрешил администратор; сервер лишь сверяет его файл с одобренным хешем.
plugins-admin-load-error = Не удалось загрузить плагины.
plugins-admin-register-heading = Регистрация плагина
plugins-admin-register-component = Компонент
plugins-admin-register-third-party = Сторонний компонент
plugins-admin-register-id = Идентификатор плагина
plugins-admin-register-id-hint = Сервер читает компонент из файла с именем, соответствующим этому идентификатору, в каталоге сторонних плагинов. Сначала одобрите хеш этого файла в списке разрешённых ниже.
plugins-admin-register-name = Название
plugins-admin-register-version = Версия
plugins-admin-register-capabilities = Возможности, заявленные плагином
plugins-admin-register-hosts = Узлы, к которым плагин может обращаться
plugins-admin-register-hosts-hint-first-party = Имена узлов через запятую. Оставьте пустым, чтобы использовать узлы компонента по умолчанию.
plugins-admin-register-hosts-hint-third-party = Имена узлов через запятую. Оставьте пустым, чтобы запретить доступ к сети.
plugins-admin-register-submit = Зарегистрировать
plugins-admin-trust-first-party = Собственный, хеш встроен
plugins-admin-trust-third-party = Сторонний, хеш одобрен
plugins-admin-status-loaded = Загружен
plugins-admin-status-not-loaded = Не загружен
plugins-admin-status-restart = Требуется перезапуск
plugins-admin-restart-to-load = Для загрузки этого плагина требуется перезапуск.
plugins-admin-restart-to-apply = Для применения этих изменений требуется перезапуск. До этого некоторые учётные записи этого моста не обслуживаются.
plugins-admin-reason-not-approved = Не загружен: администратор не одобрил этот плагин.
plugins-admin-reason-disabled = Не загружен: плагин не включён.
plugins-admin-reason-unsigned-not-allowed = Не загружен: у плагина нет подписи, и запуск без неё не разрешён.
plugins-admin-reason-no-grant = Не загружен: не предоставлена ни одна из нужных ему возможностей.
plugins-admin-reason-component-unavailable = Не загружен: ни один файл компонента не прошёл проверку хеша.
plugins-admin-reason-load-failed = Не загружен: серверу не удалось загрузить компонент. См. журнал сервера.
plugins-admin-reason-proxy-mode = Не загружен: этот сервер работает в режиме прокси и сам не обрабатывает почту.
plugins-admin-reason-no-account-binding = Не загружен: к этому мосту не привязана ни одна учётная запись.
plugins-admin-reason-no-host-caller = Не загружен: эта версия сервера не вызывает хуки, которые предоставляет плагин.
plugins-admin-reason-another-classifier-active = Не загружен: загружен другой классификатор спама. Одновременно может работать только один.
plugins-admin-running-with = Работает с возможностями: { $capabilities }
plugins-admin-hosts = Узлы: { $hosts }
plugins-admin-hosts-none = Узлы: нет
plugins-admin-grants-heading = Предоставленные возможности
plugins-admin-grants-hint = Плагин может использовать только отмеченные здесь возможности. При сохранении предыдущий набор заменяется.
plugins-admin-grant-for = Предоставить { $capability } плагину { $name }
plugins-admin-grants-save = Сохранить возможности
plugins-admin-endpoint = Адрес классификатора
plugins-admin-endpoint-for = Адрес классификатора для { $name }
plugins-admin-endpoint-hint = Адрес, по которому плагин обращается к своей службе. Узел также должен быть в списке узлов плагина. Оставьте пустым, чтобы использовать значение компонента по умолчанию.
plugins-admin-endpoint-save = Сохранить адрес
plugins-admin-test = Проверить классификатор
plugins-admin-test-verdict = Результат проверки: { $verdict }
plugins-admin-uninstall = Удалить
plugins-admin-uninstall-confirm = Подтвердить удаление
plugins-admin-uninstall-note = При удалении удаляются регистрация, предоставленные возможности и сохранённые настройки.
plugins-admin-uninstall-detail = При удалении удаляются сохранённые данные плагина для всех учётных записей, его закрепления в списке разрешённых, а также регистрация и предоставленные возможности. Файл компонента на диске не удаляется.
plugins-admin-revoke-detail = Отзыв снимает одобрение этого хеша и отключает плагин. Если плагин работает, он сразу останавливается.
plugins-admin-error-already-registered = Этот плагин уже зарегистрирован. Удалите его, прежде чем регистрировать снова.
plugins-admin-error-component-unavailable = Ни один файл компонента этого плагина не соответствует хешу, встроенному в сервер.
plugins-admin-error-digest-not-approved = Для этого идентификатора не найден файл компонента с одобренным хешем. Сначала одобрите хеш файла в списке разрешённых.
plugins-admin-error-not-approved = Одобрите плагин, прежде чем включать его.
plugins-admin-error-unsigned-not-allowed = У этого плагина нет подписи. Разрешите запуск без подписи, прежде чем включать его.
plugins-admin-error-not-loaded = Плагин не загружен, поэтому его нельзя проверить.
plugins-admin-error-classifier-error = Ошибка вызова классификатора: { $detail }
plugins-admin-error-other = Сервер отклонил запрос: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = UI-плагины
plugins-admin-ui-title = UI-плагины
plugins-admin-ui-intro = UI-плагины добавляют элементы управления в веб-клиент. Каждый работает в изолированном фрейме и может использовать только предоставленные здесь возможности. Плагин становится доступен вошедшим пользователям после одобрения и включения.
plugins-admin-ui-empty = Нет зарегистрированных UI-плагинов.
plugins-admin-ui-load-error = Не удалось загрузить UI-плагины.
plugins-admin-ui-register-heading = Регистрация UI-плагина
plugins-admin-ui-manifest = Манифест (JSON)
plugins-admin-ui-bundle = Файл пакета
plugins-admin-ui-bundle-hint = Обязателен для подписанного плагина: подпись проверяется по этому файлу.
plugins-admin-ui-allow-unsigned = Разрешить регистрацию этого плагина без подписи
plugins-admin-ui-register = Зарегистрировать
plugins-admin-ui-manifest-invalid = Манифест не является корректным JSON.
plugins-admin-ui-refused = Сервер отклонил запрос: { $detail }
plugins-admin-ui-approve = Одобрить и включить
plugins-admin-ui-extension-points = Точки расширения: { $points }
plugins-admin-ui-grants-heading = Предоставление возможности
plugins-admin-ui-grants-note = Сервер не сообщает, какие возможности уже предоставлены. Повторное предоставление возможности заменяет предыдущее.
plugins-admin-ui-grant = Предоставить { $capability }
plugins-admin-ui-grant-for = Предоставить { $capability } плагину { $name }
plugins-admin-ui-grant-hosts = Узлы для { $capability } в { $name }
plugins-admin-ui-grant-hosts-hint = Имена узлов через запятую, с которых плагин может загружать данные.
plugins-admin-ui-granted = Предоставлено: { $capability }.
plugins-admin-ui-delete = Удалить
plugins-admin-ui-delete-confirm = Подтвердить удаление
