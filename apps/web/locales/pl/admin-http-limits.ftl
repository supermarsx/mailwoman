# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Obowiązuje teraz: { $connections } połączeń ({ $connectionsSource ->
        [console] ustawione w konsoli
        [explicit] ustawione przez środowisko
       *[other] Auto
    }), { $requests } jednoczesnych żądań ({ $requestsSource ->
        [console] ustawione w konsoli
        [explicit] ustawione przez środowisko
       *[other] Auto
    }), { $streams } strumieni HTTP/2 na połączenie. Żądanie na limicie czeka do { $queueWaitMs } ms, a czekać może najwyżej { $queueMax }.
admin-http-limits-warning = Ostrzeżenie: { $text }
