# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = Introuvable (404) par intervalle
admin-history-nf-total = Introuvable (404) sur cette période
admin-history-nf-share = { $share } des requêtes
admin-history-nf-not-tracked-window = Les erreurs 404 n’ont pas été suivies sur cette période.
admin-history-nf-not-tracked-before = Les erreurs 404 n’étaient pas suivies avant { $time }. Les périodes antérieures apparaissent comme non disponibles, et non comme zéro.
admin-history-nf-paths-title = Chemins introuvables les plus fréquents
admin-history-nf-paths-help = Jusqu’à { $limit } chemins par heure, de { $from } à { $to }, arrondis aux heures entières. Les chaînes de requête sont supprimées, et les adresses e-mail, nombres, identifiants, jetons et longues valeurs sont remplacés par des espaces réservés comme {"{"}token{"}"}. Les échecs attendus de l’application elle-même, comme les requêtes d’icônes du navigateur, ne sont pas comptés.
admin-history-nf-paths-approximate = Plus de chemins distincts sont arrivés que ce qui est conservé ; certains nombres sont donc une plage et non une valeur exacte.
admin-history-nf-paths-empty = Aucune réponse « introuvable » sur cette période.
admin-history-nf-paths-error = Impossible de charger les chemins introuvables. Actualisez pour réessayer.
admin-history-nf-paths-method = Méthode
admin-history-nf-paths-path = Chemin
admin-history-nf-paths-count = Nombre
admin-history-nf-paths-range = { $min } à { $max }
admin-history-nf-paths-client = Des clients propres à l’application
admin-history-nf-paths-client-help = Chemins demandés par l’application web, son service worker ou les applications de bureau et mobiles : fichiers de build, script du worker, routes API, JMAP, DAV et console. Un échec ici signifie généralement qu’un client est obsolète ou qu’une route est erronée.
admin-history-nf-paths-external = Autre trafic
admin-history-nf-paths-external-help = Chemins que l’application ne demande jamais, par exemple des scanners qui recherchent des fichiers PHP, des fichiers cachés ou d’autres logiciels, et des liens venant d’ailleurs.
admin-history-nf-paths-all = Chemins
admin-history-nf-paths-all-help = Ce serveur ne distingue pas les chemins des clients des autres trafics.
