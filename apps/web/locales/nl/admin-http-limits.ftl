# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = Nu van kracht: { $connections } verbindingen ({ $connectionsSource ->
        [console] ingesteld in de console
        [explicit] ingesteld door de omgeving
       *[other] Auto
    }), { $requests } gelijktijdige verzoeken ({ $requestsSource ->
        [console] ingesteld in de console
        [explicit] ingesteld door de omgeving
       *[other] Auto
    }), { $streams } HTTP/2-streams per verbinding. Een verzoek op de limiet wacht tot { $queueWaitMs } ms, met hoogstens { $queueMax } wachtend.
admin-http-limits-warning = Waarschuwing: { $text }
