# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Télémétrie opérationnelle
admin-telemetry-help = Exportez les traces et les métriques opérationnelles vers un collecteur compatible OpenTelemetry, par exemple un OpenTelemetry Collector auto-hébergé. L’export est désactivé par défaut. Les modifications enregistrées s’appliquent après le redémarrage du serveur.
admin-telemetry-back = Retour à l’observabilité
admin-telemetry-saved = Configuration de la télémétrie enregistrée. Redémarrez le serveur pour appliquer les modifications.
admin-telemetry-restart = La configuration enregistrée diffère du serveur en cours d’exécution. Un redémarrage est nécessaire.
admin-telemetry-reload = Recharger la configuration
admin-telemetry-save = Enregistrer la configuration de la télémétrie
admin-telemetry-collector = Collecteur et signaux
admin-telemetry-collector-help = Seul un administrateur peut choisir cette destination valable pour tout le serveur. Utilisez un collecteur privé ou HTTPS ; le HTTP transmet la télémétrie sans chiffrement.
admin-telemetry-enabled = Activer l’export OTLP
admin-telemetry-traces = Exporter les traces
admin-telemetry-metrics = Exporter les métriques
admin-telemetry-endpoint = Point de terminaison OTLP gRPC
admin-telemetry-endpoint-help = Adresse absolue HTTP ou HTTPS du collecteur, généralement le port 4317. N’incluez ni identifiants, ni requête, ni fragment. Il s’agit d’un point de terminaison gRPC, et non du port OTLP HTTP 4318.
admin-telemetry-serviceName = Nom du service
admin-telemetry-serviceName-help = Libellé de service stable servant à identifier ce déploiement de Mailwoman dans votre collecteur. N’utilisez pas d’adresse e-mail ni d’autres informations personnelles.
admin-telemetry-environment = Environnement
admin-telemetry-environment-help = Libellé de déploiement facultatif, par exemple production ou staging. Il est envoyé avec la télémétrie exportée.
admin-telemetry-pinned = Les champs désactivés sont contrôlés par des variables d’environnement du déploiement. Modifiez ces variables pour remplacer leurs valeurs.
admin-telemetry-limits = Échantillonnage et limites d’export
admin-telemetry-limits-help = Limitent la mémoire et le trafic vers le collecteur. La taille de lot ne peut pas dépasser la taille de la file. Lorsque les files sont pleines, des spans peuvent être abandonnés plutôt que de bloquer les requêtes de messagerie.
admin-telemetry-sampleRatio = Taux d’échantillonnage des traces
admin-telemetry-sampleRatio-help = Fraction des nouvelles traces racines à conserver, de 0 à 1. Les spans enfants suivent la décision d’échantillonnage du parent.
admin-telemetry-exportTimeoutMs = Délai d’export (millisecondes)
admin-telemetry-exportTimeoutMs-help = Durée maximale d’une requête d’export vers le collecteur, de 100 à 30000 millisecondes.
admin-telemetry-metricsIntervalSecs = Intervalle des métriques (secondes)
admin-telemetry-metricsIntervalSecs-help = Délai entre deux exports de métriques, de 10 à 3600 secondes.
admin-telemetry-spanQueueSize = Taille de la file de spans
admin-telemetry-spanQueueSize-help = Nombre maximal de spans en attente d’export, de 128 à 32768.
admin-telemetry-spanBatchSize = Taille des lots de spans
admin-telemetry-spanBatchSize-help = Nombre maximal de spans dans un export, de 1 à 4096 et sans dépasser la taille de la file.
admin-telemetry-spanExportDelayMs = Délai des lots de spans (millisecondes)
admin-telemetry-spanExportDelayMs-help = Attente normale entre deux lots de spans, de 100 à 60000 millisecondes.
admin-telemetry-prometheus-help = Un point de terminaison authentifié distinct pour un collecteur compatible Prometheus. L’accès exige toujours le jeton porteur MW_METRICS_TOKEN du déploiement ; sa valeur n’est jamais renvoyée à cette page.
admin-telemetry-prometheusEnabled = Activer la collecte Prometheus
admin-telemetry-token-missing = Aucun jeton de collecte n’est configuré. Le point de terminaison des métriques reste inaccessible tant que l’opérateur n’a pas défini MW_METRICS_TOKEN.
