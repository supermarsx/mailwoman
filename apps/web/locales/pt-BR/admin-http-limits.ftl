# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Em vigor agora: { $connections } conexões ({ $connectionsSource ->
        [console] definido no console
        [explicit] definido pelo ambiente
       *[other] Auto
    }), { $requests } solicitações simultâneas ({ $requestsSource ->
        [console] definido no console
        [explicit] definido pelo ambiente
       *[other] Auto
    }), { $streams } streams HTTP/2 por conexão. Uma solicitação no limite espera até { $queueWaitMs } ms, com no máximo { $queueMax } aguardando.
admin-http-limits-warning = Aviso: { $text }
