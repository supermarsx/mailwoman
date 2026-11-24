# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Сейчас действует: { $connections } соединений ({ $connectionsSource ->
        [console] задано в консоли
        [explicit] задано окружением
       *[other] Авто
    }), { $requests } одновременных запросов ({ $requestsSource ->
        [console] задано в консоли
        [explicit] задано окружением
       *[other] Авто
    }), { $streams } потоков HTTP/2 на соединение. Запрос на пределе ждёт до { $queueWaitMs } мс, ждать могут не более { $queueMax }.
admin-http-limits-warning = Предупреждение: { $text }
