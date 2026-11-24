# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Jetzt in Kraft: { $connections } Verbindungen ({ $connectionsSource ->
        [console] in der Konsole gesetzt
        [explicit] durch die Umgebung gesetzt
       *[other] Auto
    }), { $requests } gleichzeitige Anfragen ({ $requestsSource ->
        [console] in der Konsole gesetzt
        [explicit] durch die Umgebung gesetzt
       *[other] Auto
    }), { $streams } HTTP/2-Streams pro Verbindung. Eine Anfrage am Limit wartet bis zu { $queueWaitMs } ms, höchstens { $queueMax } warten gleichzeitig.
admin-http-limits-warning = Warnung: { $text }
