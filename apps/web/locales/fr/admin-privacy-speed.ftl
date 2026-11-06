# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Rapidité
privacy-speed-unavailable = Les chiffres de rapidité ne sont pas disponibles sur ce serveur.
privacy-speed-empty = Aucune image n’a encore été servie sur cette période.
privacy-speed-off = Le suivi de la rapidité est désactivé (paramètre : suivi de la rapidité du proxy d’images). Rien de nouveau n’est enregistré.
privacy-speed-verdict-healthy = Satisfaisant
privacy-speed-verdict-slow = Lent
privacy-speed-verdict-degraded = Dégradé
privacy-speed-verdict-noData = Données insuffisantes
privacy-speed-reason-ok = Les images récupérées et les accès au cache respectent les limites ci-dessous, et peu de requêtes ont échoué.
privacy-speed-reason-failureRate = { $failurePct } % des requêtes se sont terminées par « occupé », un délai dépassé ou une erreur en amont.
privacy-speed-reason-missP95 = Les images qu’il a fallu récupérer sont lentes : leur 95e centile dépasse la limite ci-dessous.
privacy-speed-reason-hitP95 = Les images en cache sont lentes à répondre : leur 95e centile dépasse la limite ci-dessous.
privacy-speed-reason-fewRequests = Moins de { $min } requêtes sur cette période, trop peu pour juger.
privacy-speed-thresholds = Lent à partir de : 95e centile des images récupérées { $slowMiss }, 95e centile des images en cache { $slowHit }, ou { $slowFail } % occupé, délai dépassé ou échec. Dégradé à partir de : 95e centile des images récupérées { $degradedMiss } ou { $degradedFail } % d’échecs. Limites fixes, pas des paramètres.
privacy-speed-median = Médiane
privacy-speed-p95 = 95e centile
privacy-speed-p99 = 99e centile
privacy-speed-max = Plus lent
privacy-speed-requests = Requêtes mesurées
privacy-speed-busy = Réponses « occupé »
privacy-speed-timeouts = Délais dépassés
privacy-speed-upstream-errors = Erreurs en amont
privacy-speed-hit = Accès au cache réussis
privacy-speed-miss = Images récupérées (échecs de cache)
privacy-speed-split-count = { $count } requêtes
privacy-speed-split-times = Médiane { $p50 }, 95e centile { $p95 }, plus lent { $max }.
privacy-speed-chart = Temps pour servir une image
privacy-speed-chart-miss = Temps pour servir une image récupérée
privacy-speed-chart-unit = De la requête reçue à la réponse prête
privacy-speed-phases = Où passe le temps
privacy-speed-phases-help = Pour les requêtes qui n’ont pas trouvé l’image en cache : la durée de chaque étape et sa part du temps total.
privacy-speed-phase = Étape
privacy-speed-share = Part du temps
privacy-speed-phase-queue = Attente de capacité
privacy-speed-phase-cache = Recherche dans le cache
privacy-speed-phase-upstream = Récupération en amont
privacy-speed-phase-decode = Décodage et réencodage
privacy-speed-phase-other = Tout le reste
privacy-speed-note = Mesuré uniquement sur ce processus serveur, conservé en mémoire et vide après un redémarrage. Les centiles sont précis à { $error } % près. Aucune URL, aucun hôte ni aucun compte n’est enregistré.
