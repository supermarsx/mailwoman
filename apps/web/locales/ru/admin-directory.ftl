# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-directory. Source: ../en/admin-directory.ftl


# -- Machine translation (2026-11): added to match en ---------------------
directory-title = Конфигурация каталога LDAP
directory-help = Настройте серверы каталога, сопоставление атрибутов и запечатанные учётные данные службы. Изменения вступают в силу после перезапуска сервера. Доступность сервера при этом не проверяется.
directory-reload = Перезагрузить сохранённую конфигурацию
directory-saved = Конфигурация сохранена. Перезапустите сервер, чтобы применить её.
directory-restart = Сохранённая конфигурация отличается от работающего каталога. Требуется перезапуск сервера.
directory-proxy-blocked = Подключения LDAP блокируются, пока активен исходящий прокси. Этот транспорт LDAP не поддерживает прокси-туннели и не будет обходить ваш прокси.
directory-env-pin = Окружение закрепляет учётные данные службы для этого DN привязки:
directory-url = URL сервера
directory-base = DN базы поиска
directory-bind = DN привязки службы
directory-tls = Защита транспорта
directory-plaintext = Открытый текст (только анонимно)
directory-priority = Приоритет
directory-connection = Политика подключений
directory-blocked = Блокировать подключения
directory-direct = Разрешить защищённые прямые подключения
directory-enabled = Включать в поиск по каталогу
directory-private = Разрешить частные адреса локальной сети и loopback
directory-consent = Подключения заблокированы. Явно выберите защищённый прямой доступ, чтобы разрешить эту конечную точку; для частных адресов требуется отдельное согласие.
directory-password-set = Запечатанный пароль службы сохранён.
directory-password-unset = Пароль службы не сохранён. Поиск использует анонимный доступ, если не применяются учётные данные из окружения.
directory-secret-action = Действие с сохранённым паролем
directory-keep = Оставить без изменений
directory-replace = Задать или заменить пароль
directory-clear = Удалить сохранённый пароль
directory-password = Новый пароль службы
directory-attr-display-name = Атрибут отображаемого имени
directory-attr-mail = Атрибут электронной почты
directory-attr-member = Атрибут участника группы
directory-attr-user-cert = Атрибут сертификата
directory-attr-photo = Атрибут фотографии
directory-remove = Удалить из конфигурации
directory-add = Добавить каталог
directory-save = Сохранить конфигурацию каталога
