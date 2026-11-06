# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Accès entrant
admin-inbound-help = Restreignez séparément les adresses IP sources de confiance pour la console d’administration et pour le webmail. Les deux politiques sont d’abord désactivées. Ces règles ne modifient ni les proxys de confiance, ni les portées d’API, ni l’accès sortant.
admin-inbound-unscoped = Cette politique valable pour tout le déploiement nécessite un administrateur disposant d’un accès de sécurité sans restriction de périmètre.
admin-inbound-admin = Console d’administration et API
admin-inbound-admin-help = Comprend la connexion et cet éditeur. L’authentification existante d’un administrateur ne contourne pas la politique réseau.
admin-inbound-webmail = Webmail et API de compte
admin-inbound-webmail-help = Couvre le courrier, DAV, le contenu signé, le SSO, OIDC et les routes d’API. Les connexions déjà mises à niveau conservent la décision prise lors de leur négociation.
admin-inbound-enabled = Restreindre cette surface à la liste d’autorisation
admin-inbound-cidrs = IP et CIDR de secours
admin-inbound-cidrs-help = Une IP littérale ou un CIDR par ligne, jusqu’à 128. Au moins un est requis lorsque la politique est activée. Incluez une source de confiance utilisable pour la récupération.
admin-inbound-hostnames = Noms d’hôte DNS autorisés
admin-inbound-hostnames-help = Un nom DNS ASCII en minuscules par ligne. Utilisez l’ASCII IDNA pour les noms internationaux. Pas d’URL, de ports, de caractères génériques, d’IP littérales ni de points finaux. Les noms n’accordent que les adresses actuellement résolues.
admin-inbound-dns = Limites de résolution DNS
admin-inbound-dns-help = Les noms nouveaux, expirés et en échec n’accordent aucun accès. Les réponses réussies respectent leur TTL DNS et le plafond de cache configuré. Les requêtes n’attendent jamais la résolution DNS.
admin-inbound-servers = Serveurs DNS récursifs
admin-inbound-servers-help = Laissez vide pour utiliser les résolveurs du système, ou saisissez jusqu’à quatre sockets IP:port, un par ligne (par exemple 10.0.0.53:53 ou [fd00::53]:53). Les serveurs explicites remplacent les résolveurs du système, sans repli public. La confiance accordée au résolveur relève de votre responsabilité ; une validation DNSSEC indépendante n’est pas garantie.
admin-inbound-dns-timeoutMs = Délai d’expiration de la recherche (millisecondes)
admin-inbound-dns-cacheSeconds = Durée de vie maximale du cache (secondes)
admin-inbound-dns-maxHosts = Nombre maximal de noms d’hôte distincts
admin-inbound-dns-maxAddresses = Nombre maximal d’adresses par nom d’hôte
admin-inbound-dns-concurrency = Recherches simultanées
admin-inbound-bounds = Plage autorisée : { $min }–{ $max }.
admin-inbound-lockout = Cette politique peut bloquer immédiatement votre propre accès, y compris à cet éditeur. Vérifiez que les IP de secours couvrent votre source réelle telle que la déterminent les proxys de confiance configurés. Il n’existe aucune exception automatique pour le bouclage ou pour un administrateur connecté. La reprise après un verrouillage nécessite une restauration côté déploiement ; une connexion ordinaire ne peut pas le contourner.
admin-inbound-confirm-title = Appliquer la politique d’accès entrant ?
admin-inbound-confirm-word = APPLIQUER
admin-inbound-save = Appliquer la politique confirmée
admin-inbound-review = Vérifier les modifications
admin-inbound-refresh = Actualiser la politique enregistrée et l’état DNS
admin-inbound-reset = Rétablir le brouillon à la politique chargée
admin-inbound-saved = Le serveur a accepté la politique d’accès entrant.
admin-inbound-conflict = La politique enregistrée a changé pendant votre modification. Réinitialisez le brouillon et actualisez pour charger sa révision actuelle avant de modifier à nouveau.
admin-inbound-no-revision = Le serveur n’a pas renvoyé de révision de la politique. Actualisez avant d’enregistrer.
admin-inbound-status = Autorisations DNS actuelles
admin-inbound-status-help = Il s’agit d’un instantané de la dernière actualisation de l’état, et non d’un compte à rebours ni d’un test de connectivité. Actualisez pour voir les autorisations actuelles. Une erreur peut coexister avec une réponse antérieure encore valide jusqu’à son expiration d’origine.
admin-inbound-refreshing = Une actualisation DNS limitée est en cours.
admin-inbound-idle = Aucune actualisation DNS n’était en cours lors de la dernière lecture de l’état.
admin-inbound-no-hosts = Aucun nom d’hôte n’est actif dans les politiques activées.
admin-inbound-hostname = Nom d’hôte
admin-inbound-state = Autorisation d’accès
admin-inbound-addresses = Adresses utilisables
admin-inbound-validity = Validité à la dernière actualisation
admin-inbound-error = Code de résultat DNS
admin-inbound-usable = Utilisable
admin-inbound-unavailable = Aucune autorisation actuelle
admin-inbound-seconds = { $seconds } secondes
admin-inbound-on = Activé
admin-inbound-off = Désactivé
admin-inbound-invalid-dns = Les limites DNS ou le nombre de résolveurs sont en dehors des bornes autorisées.
admin-inbound-rescue-required = Chaque surface activée nécessite au moins une IP ou un CIDR de secours.
admin-inbound-too-many = La politique dépasse la limite configurée de noms d’hôte ou d’adresses littérales.
admin-inbound-invalid-host = Utilisez des noms DNS ASCII en minuscules, sans URL, ports, caractères génériques, IP littérales ni points finaux.
