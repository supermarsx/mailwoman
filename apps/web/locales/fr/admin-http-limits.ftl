# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = En vigueur : { $connections } connexions ({ $connectionsSource ->
        [console] défini dans la console
        [explicit] défini par l'environnement
       *[other] Auto
    }), { $requests } requêtes simultanées ({ $requestsSource ->
        [console] défini dans la console
        [explicit] défini par l'environnement
       *[other] Auto
    }), { $streams } flux HTTP/2 par connexion. Une requête à la limite attend jusqu'à { $queueWaitMs } ms, avec au plus { $queueMax } en attente.
admin-http-limits-warning = Avertissement : { $text }
