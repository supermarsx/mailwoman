# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = En vigor ahora: { $connections } conexiones ({ $connectionsSource ->
        [console] definido en la consola
        [explicit] definido por el entorno
       *[other] Auto
    }), { $requests } solicitudes a la vez ({ $requestsSource ->
        [console] definido en la consola
        [explicit] definido por el entorno
       *[other] Auto
    }), { $streams } flujos HTTP/2 por conexión. Una solicitud en el límite espera hasta { $queueWaitMs } ms, con { $queueMax } en espera como máximo.
admin-http-limits-warning = Aviso: { $text }
