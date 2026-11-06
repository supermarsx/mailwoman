# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Chiffres clés
admin-stats-charts = Graphiques
admin-stats-tables = Tableaux
admin-stats-group-traffic = Trafic sur cette période
admin-stats-group-storage = Stockage et distribution, actuellement
admin-stats-refresh = Actualiser les statistiques
admin-stats-loading = Chargement des statistiques…
admin-stats-requests-total = Requêtes
admin-stats-errors-total = Erreurs serveur (5xx)
admin-stats-p95-peak = 95e centile le plus lent
admin-stats-requests = Requêtes
admin-stats-errors = Erreurs serveur (5xx)
admin-stats-not-found = Introuvable (404)
admin-stats-latency = Temps de réponse
admin-stats-series-p50 = Médiane
admin-stats-series-p95 = 95e centile
admin-stats-unit-count = Nombre par { $bucket }
admin-stats-unit-duration = Durée par { $bucket }, dans l’unité affichée
admin-stats-unit-accounts = Comptes bloqués par domaine
admin-stats-chart-about = Courbe de { $from } à { $to }.
admin-stats-bars-about = { $count ->
    [one] Diagramme à barres avec un domaine.
   *[other] Diagramme à barres avec { $count } domaines.
}
admin-stats-view-table = Afficher sous forme de tableau
admin-stats-legend = Légende
admin-stats-empty-chart = Rien n’a encore été mesuré sur cette période. Choisissez une période plus longue ou revenez une fois que le serveur aura traité quelques requêtes.
admin-stats-empty-blocked = Aucun compte n’est bloqué.
admin-stats-delta-vs = par rapport à la période précédente
admin-stats-delta-same = Identique à la période précédente
admin-stats-delta-sr-up = En hausse de { $change } par rapport à la période précédente.
admin-stats-delta-sr-down = En baisse de { $change } par rapport à la période précédente.
admin-stats-delta-sr-same = Identique à la période précédente.
admin-ct-group = Temps de chargement côté client
admin-ct-metric-timeToUsable = Temps avant utilisation
admin-ct-metric-ttfb = Premier octet
admin-ct-metric-dcl = DOM prêt
admin-ct-metric-load = Chargement de la page
admin-ct-metric-fcp = Premier affichage de contenu
admin-ct-metric-lcp = Plus grand affichage de contenu
admin-ct-metric-folderOpen = Ouverture du dossier
admin-ct-metric-folderLoadCold = Chargement du dossier, depuis le réseau
admin-ct-metric-folderLoadCached = Chargement du dossier, depuis le cache
admin-ct-metric-messageLoadCold = Chargement du message, depuis le réseau
admin-ct-metric-messageLoadCached = Chargement du message, depuis le cache ou le préchargement
admin-ct-device-desktop = Ordinateur
admin-ct-device-mobile = Mobile
admin-ct-device-unknown = Inconnu
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } échantillon
       *[other] { $count } échantillons
    }
admin-ct-no-samples = Aucun échantillon sur cette période.
admin-ct-few-samples = Seulement { $count } échantillons ; au moins { $min } sont nécessaires.
admin-ct-chart-usable = Temps avant utilisation
admin-ct-group-folder = Temps de chargement des dossiers
admin-ct-group-message = Temps de chargement des messages
admin-ct-chart-folder-cold = Chargement du dossier depuis le réseau
admin-ct-chart-folder-cached = Chargement du dossier depuis le cache
admin-ct-chart-message-cold = Chargement du message depuis le réseau
admin-ct-chart-message-cached = Chargement du message depuis le cache ou le préchargement
admin-ct-folder-off = Les temps de chargement des dossiers sont désactivés par défaut. Seules les personnes qui les ont activés sont mesurées.
admin-ct-message-off = Les temps de chargement des messages sont désactivés par défaut. Seules les personnes qui les ont activés sont mesurées.
admin-ct-chart-note = Les centiles ne s’affichent que pour les intervalles comptant au moins { $min } échantillons.
admin-ct-empty-chart = Aucun intervalle ne compte les { $min } échantillons nécessaires pour afficher une médiane. Choisissez une période plus longue.
admin-ct-col-samples = Échantillons
admin-ct-col-metric = Mesure
admin-ct-col-device = Appareil
admin-ct-table-title = Temps de chargement côté client par appareil
admin-ct-table-caption = Échantillons, médiane et 95e centile pour chaque mesure et chaque type d’appareil.
admin-ct-empty-table = Aucun temps de chargement de navigateur n’a été signalé sur cette période.
admin-ct-failed = Impossible de charger les temps de chargement côté client.
admin-ct-disabled = La collecte des temps de chargement côté client est désactivée. Les chiffres ci-dessous datent d’avant sa désactivation.
admin-ct-note = Uniquement depuis des navigateurs connectés, { $rate } % des sessions échantillonnées, anonymes et agrégées. Le temps avant utilisation court du début de la navigation jusqu’à l’affichage des premières lignes de la liste des messages. Les chiffres sont des estimations, à environ { $error } % de la valeur réelle.
