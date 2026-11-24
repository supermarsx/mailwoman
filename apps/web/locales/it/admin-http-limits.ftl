# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = In vigore ora: { $connections } connessioni ({ $connectionsSource ->
        [console] impostato nella console
        [explicit] impostato dall'ambiente
       *[other] Auto
    }), { $requests } richieste simultanee ({ $requestsSource ->
        [console] impostato nella console
        [explicit] impostato dall'ambiente
       *[other] Auto
    }), { $streams } flussi HTTP/2 per connessione. Una richiesta al limite attende fino a { $queueWaitMs } ms, con al massimo { $queueMax } in attesa.
admin-http-limits-warning = Avviso: { $text }
