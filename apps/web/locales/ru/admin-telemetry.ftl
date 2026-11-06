# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Операционная телеметрия
admin-telemetry-help = Экспортируйте операционные трассировки и метрики в коллектор, совместимый с OpenTelemetry, например в собственный OpenTelemetry Collector. Экспорт по умолчанию выключен. Сохранённые изменения применяются после перезапуска сервера.
admin-telemetry-back = Назад к наблюдаемости
admin-telemetry-saved = Конфигурация телеметрии сохранена. Перезапустите сервер, чтобы применить изменения.
admin-telemetry-restart = Сохранённая конфигурация отличается от работающего сервера. Требуется перезапуск.
admin-telemetry-reload = Перезагрузить конфигурацию
admin-telemetry-save = Сохранить конфигурацию телеметрии
admin-telemetry-collector = Коллектор и сигналы
admin-telemetry-collector-help = Выбрать это общесерверное назначение может только администратор. Используйте приватный коллектор или HTTPS; HTTP передаёт телеметрию без шифрования.
admin-telemetry-enabled = Включить экспорт OTLP
admin-telemetry-traces = Экспортировать трассировки
admin-telemetry-metrics = Экспортировать метрики
admin-telemetry-endpoint = Конечная точка OTLP gRPC
admin-telemetry-endpoint-help = Абсолютный адрес коллектора по HTTP или HTTPS, обычно порт 4317. Не указывайте учётные данные, запрос или фрагмент. Это конечная точка gRPC, а не OTLP HTTP на порту 4318.
admin-telemetry-serviceName = Имя службы
admin-telemetry-serviceName-help = Постоянная метка службы, по которой коллектор определяет это развёртывание Mailwoman. Не используйте адрес электронной почты и другие персональные данные.
admin-telemetry-environment = Окружение
admin-telemetry-environment-help = Необязательная метка развёртывания, например production или staging. Отправляется вместе с экспортируемой телеметрией.
admin-telemetry-pinned = Отключённые поля управляются переменными окружения развёртывания. Чтобы переопределить их значения, измените эти переменные.
admin-telemetry-limits = Выборка и ограничения экспорта
admin-telemetry-limits-help = Ограничивают использование памяти и трафик к коллектору. Размер пакета не может превышать размер очереди. При переполнении очереди спаны могут отбрасываться, а запросы почты не блокируются.
admin-telemetry-sampleRatio = Доля выборки трассировок
admin-telemetry-sampleRatio-help = Доля сохраняемых новых корневых трассировок, от 0 до 1. Дочерние спаны следуют решению о выборке родительского.
admin-telemetry-exportTimeoutMs = Тайм-аут экспорта (миллисекунды)
admin-telemetry-exportTimeoutMs-help = Максимальная длительность запроса экспорта к коллектору, от 100 до 30000 миллисекунд.
admin-telemetry-metricsIntervalSecs = Интервал метрик (секунды)
admin-telemetry-metricsIntervalSecs-help = Время между экспортами метрик, от 10 до 3600 секунд.
admin-telemetry-spanQueueSize = Размер очереди спанов
admin-telemetry-spanQueueSize-help = Максимальное число спанов, ожидающих экспорта, от 128 до 32768.
admin-telemetry-spanBatchSize = Размер пакета спанов
admin-telemetry-spanBatchSize-help = Максимальное число спанов в одном экспорте, от 1 до 4096 и не больше размера очереди.
admin-telemetry-spanExportDelayMs = Задержка пакета спанов (миллисекунды)
admin-telemetry-spanExportDelayMs-help = Обычное ожидание между пакетами спанов, от 100 до 60000 миллисекунд.
admin-telemetry-prometheus-help = Отдельная защищённая конечная точка для сборщика, совместимого с Prometheus. Доступ всегда требует токена-носителя MW_METRICS_TOKEN развёртывания; его значение никогда не возвращается на эту страницу.
admin-telemetry-prometheusEnabled = Включить сбор данных Prometheus
admin-telemetry-token-missing = Токен сбора не настроен. Конечная точка метрик остаётся недоступной, пока оператор не задаст MW_METRICS_TOKEN.
