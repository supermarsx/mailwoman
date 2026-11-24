# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Зараз діє: { $connections } з’єднань ({ $connectionsSource ->
        [console] задано в консолі
        [explicit] задано оточенням
       *[other] Авто
    }), { $requests } одночасних запитів ({ $requestsSource ->
        [console] задано в консолі
        [explicit] задано оточенням
       *[other] Авто
    }), { $streams } потоків HTTP/2 на з’єднання. Запит на межі чекає до { $queueWaitMs } мс, чекати можуть не більше { $queueMax }.
admin-http-limits-warning = Попередження: { $text }
