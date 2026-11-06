# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Постачальник посвідчень
admin-identity-description = Дозвольте схваленим застосункам входити через Mailwoman. Це окрема функція від зовнішніх постачальників входу та доступу до API пошти. Потрібен код авторизації з PKCE; інші типи дозволів недоступні.
admin-identity-configure = Налаштувати емітента
admin-identity-new = Додати клієнт посвідчень
admin-identity-edit = Редагувати клієнт
admin-identity-enabled = Увімкнено
admin-identity-disabled = Вимкнено
admin-identity-all = Усі
admin-identity-discovery = Відкрити документ виявлення
admin-identity-search = Пошук клієнтів
admin-identity-status = Стан
admin-identity-clients = Клієнти посвідчень
admin-identity-client = Посвідчення клієнта
admin-identity-client-id = ID клієнта
admin-identity-name = Назва
admin-identity-method = Автентифікація клієнта
admin-identity-public = Публічний клієнт (PKCE, без секрету)
admin-identity-pkce = Кожен клієнт має використовувати PKCE S256. Секрети конфіденційних клієнтів створюються тут і показуються один раз.
admin-identity-rotate-secret = Замінити наявний секрет клієнта
admin-identity-redirects = Зареєстровані адреси призначення
admin-identity-redirect-help = Один точний HTTPS URI в рядку; для локальних застосунків дозволено числовий loopback HTTP. Символи підстановки та фрагменти відхиляються.
admin-identity-callbacks = URI зворотного виклику авторизації
admin-identity-logout-uris = URI перенаправлення після виходу
admin-identity-scopes = Дозволені області посвідчень
admin-identity-actions = Дії
admin-identity-delete = Видалити клієнт
admin-identity-delete-confirm = Видалити доступ посвідчень для { $name }? Наявні дозволи посвідчень перестануть працювати, а реєстрацію клієнта лише для посвідчень буде видалено.
admin-identity-empty = Відповідних клієнтів посвідчень немає.
admin-identity-cancel = Скасувати
admin-identity-save = Зберегти
admin-identity-done = Готово
admin-identity-secret = Створений секрет клієнта
admin-identity-secret-once = Скопіюйте цей секрет до довіреного застосунку зараз. Повторно показати його не можна. Ніколи не розміщуйте конфіденційний секрет у коді браузера.
admin-identity-issuer = Публічний HTTPS URL емітента
admin-identity-issuer-help = Використайте точне HTTPS-джерело цього розгортання та шлях /oidc, разом із префіксом розгортання, якщо він є. Зміна емітента чи конфігурації відкликає наявні дозволи посвідчень.
admin-identity-rotate-key = Змінити ключ підпису
admin-identity-key-help = Зміна потребує свіжого підтвердження адміністратора. Попередні відкриті ключі залишаються доступними 15 хвилин, щоб нещодавно видані токени дожили до кінця свого короткого терміну.
