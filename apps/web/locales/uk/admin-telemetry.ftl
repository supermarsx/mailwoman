# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Операційна телеметрія
admin-telemetry-help = Експортуйте операційні трасування та метрики до колектора, сумісного з OpenTelemetry, наприклад до власного OpenTelemetry Collector. Експорт типово вимкнений. Збережені зміни застосовуються після перезапуску сервера.
admin-telemetry-back = Назад до спостережуваності
admin-telemetry-saved = Конфігурацію телеметрії збережено. Перезапустіть сервер, щоб застосувати зміни.
admin-telemetry-restart = Збережена конфігурація відрізняється від працюючого сервера. Потрібен перезапуск.
admin-telemetry-reload = Перезавантажити конфігурацію
admin-telemetry-save = Зберегти конфігурацію телеметрії
admin-telemetry-collector = Колектор і сигнали
admin-telemetry-collector-help = Обрати це загальносерверне призначення може лише адміністратор. Використовуйте приватний колектор або HTTPS; HTTP передає телеметрію без шифрування.
admin-telemetry-enabled = Увімкнути експорт OTLP
admin-telemetry-traces = Експортувати трасування
admin-telemetry-metrics = Експортувати метрики
admin-telemetry-endpoint = Кінцева точка OTLP gRPC
admin-telemetry-endpoint-help = Абсолютна адреса колектора за HTTP або HTTPS, зазвичай порт 4317. Не вказуйте облікові дані, запит або фрагмент. Це кінцева точка gRPC, а не OTLP HTTP на порту 4318.
admin-telemetry-serviceName = Назва служби
admin-telemetry-serviceName-help = Стала мітка служби, за якою колектор визначає це розгортання Mailwoman. Не використовуйте адресу електронної пошти чи інші персональні дані.
admin-telemetry-environment = Середовище
admin-telemetry-environment-help = Необов'язкова мітка розгортання, наприклад production або staging. Надсилається разом з експортованою телеметрією.
admin-telemetry-pinned = Вимкнені поля керуються змінними середовища розгортання. Щоб перевизначити їхні значення, змініть ці змінні.
admin-telemetry-limits = Вибірка та обмеження експорту
admin-telemetry-limits-help = Обмежують використання пам'яті та трафік до колектора. Розмір пакета не може перевищувати розмір черги. Повні черги можуть відкидати спани замість блокування поштових запитів.
admin-telemetry-sampleRatio = Частка вибірки трасувань
admin-telemetry-sampleRatio-help = Частка нових кореневих трасувань, що зберігаються, від 0 до 1. Дочірні спани дотримуються рішення про вибірку батьківського.
admin-telemetry-exportTimeoutMs = Тайм-аут експорту (мілісекунди)
admin-telemetry-exportTimeoutMs-help = Максимальна тривалість запиту експорту до колектора, від 100 до 30000 мілісекунд.
admin-telemetry-metricsIntervalSecs = Інтервал метрик (секунди)
admin-telemetry-metricsIntervalSecs-help = Час між експортами метрик, від 10 до 3600 секунд.
admin-telemetry-spanQueueSize = Розмір черги спанів
admin-telemetry-spanQueueSize-help = Максимальна кількість спанів, що чекають на експорт, від 128 до 32768.
admin-telemetry-spanBatchSize = Розмір пакета спанів
admin-telemetry-spanBatchSize-help = Максимальна кількість спанів в одному експорті, від 1 до 4096 і не більше за розмір черги.
admin-telemetry-spanExportDelayMs = Затримка пакета спанів (мілісекунди)
admin-telemetry-spanExportDelayMs-help = Звичайне очікування між пакетами спанів, від 100 до 60000 мілісекунд.
admin-telemetry-prometheus-help = Окрема захищена кінцева точка для збирача, сумісного з Prometheus. Доступ завжди потребує токена-носія MW_METRICS_TOKEN розгортання; його значення ніколи не повертається на цю сторінку.
admin-telemetry-prometheusEnabled = Увімкнути збирання даних Prometheus
admin-telemetry-token-missing = Токен збирання не налаштовано. Кінцева точка метрик лишається недоступною, доки оператор не задасть MW_METRICS_TOKEN.
