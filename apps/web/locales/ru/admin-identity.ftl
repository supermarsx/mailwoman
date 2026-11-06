# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Поставщик удостоверений
admin-identity-description = Позвольте одобренным приложениям входить через Mailwoman. Это отдельная функция от внешних поставщиков входа и доступа к API почты. Требуется код авторизации с PKCE; другие типы разрешений недоступны.
admin-identity-configure = Настроить эмитента
admin-identity-new = Добавить клиент удостоверений
admin-identity-edit = Изменить клиент
admin-identity-enabled = Включён
admin-identity-disabled = Отключён
admin-identity-all = Все
admin-identity-discovery = Открыть документ обнаружения
admin-identity-search = Поиск клиентов
admin-identity-status = Состояние
admin-identity-clients = Клиенты удостоверений
admin-identity-client = Удостоверение клиента
admin-identity-client-id = ID клиента
admin-identity-name = Название
admin-identity-method = Аутентификация клиента
admin-identity-public = Публичный клиент (PKCE, без секрета)
admin-identity-pkce = Каждый клиент должен использовать PKCE S256. Секреты конфиденциальных клиентов создаются здесь и показываются один раз.
admin-identity-rotate-secret = Заменить существующий секрет клиента
admin-identity-redirects = Зарегистрированные адреса назначения
admin-identity-redirect-help = Один точный HTTPS URI в строке; для локальных приложений допускается числовой loopback HTTP. Подстановочные знаки и фрагменты отклоняются.
admin-identity-callbacks = URI обратного вызова авторизации
admin-identity-logout-uris = URI перенаправления после выхода
admin-identity-scopes = Разрешённые области удостоверений
admin-identity-actions = Действия
admin-identity-delete = Удалить клиент
admin-identity-delete-confirm = Удалить доступ удостоверений для { $name }? Существующие разрешения удостоверений перестанут работать, а регистрация клиента только для удостоверений будет удалена.
admin-identity-empty = Подходящих клиентов удостоверений нет.
admin-identity-cancel = Отмена
admin-identity-save = Сохранить
admin-identity-done = Готово
admin-identity-secret = Созданный секрет клиента
admin-identity-secret-once = Скопируйте этот секрет в доверяющее приложение сейчас. Повторно показать его нельзя. Никогда не помещайте конфиденциальный секрет в код браузера.
admin-identity-issuer = Публичный HTTPS URL эмитента
admin-identity-issuer-help = Используйте точный HTTPS-источник этого развёртывания и путь /oidc, включая префикс развёртывания, если он есть. Изменение эмитента или конфигурации отзывает существующие разрешения удостоверений.
admin-identity-rotate-key = Сменить ключ подписи
admin-identity-key-help = Смена требует свежего подтверждения администратора. Прежние открытые ключи остаются доступными 15 минут, чтобы недавно выданные токены дожили до конца своего короткого срока.
