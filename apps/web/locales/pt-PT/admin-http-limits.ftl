# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Em vigor agora: { $connections } ligações ({ $connectionsSource ->
        [console] definido na consola
        [explicit] definido pelo ambiente
       *[other] Auto
    }), { $requests } pedidos em simultâneo ({ $requestsSource ->
        [console] definido na consola
        [explicit] definido pelo ambiente
       *[other] Auto
    }), { $streams } streams HTTP/2 por ligação. Um pedido no limite espera até { $queueWaitMs } ms, com no máximo { $queueMax } em espera.
admin-http-limits-warning = Aviso: { $text }
