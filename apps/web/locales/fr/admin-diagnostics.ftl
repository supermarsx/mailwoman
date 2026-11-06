# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Pages de diagnostic
admin-diagx-tab-overview = Vue d'ensemble
admin-diagx-tab-server = Serveur
admin-diagx-tab-connectivity = Connectivité
admin-diagx-tab-backends = Backends de messagerie
admin-diagx-tab-posture = Posture de sécurité
admin-diagx-tab-tools = Outils
admin-diagx-tab-history = Historique
admin-diagx-tab-report = Rapport
admin-diagx-tab-server-intro = Vérifications des composants propres à ce serveur : la base de données, ses migrations et la clé du serveur. Elles s'exécutent avec les vérifications du serveur.
admin-diagx-tab-connectivity-intro = Indique si ce serveur est joignable sous son nom public, s'il peut joindre les backends enregistrés et s'il garde les adresses privées hors de portée. Les outils de domaine ci-dessous recherchent des enregistrements DNS, testent TLS et découvrent les paramètres d'un domaine.
admin-diagx-tab-backends-intro = Une connexion et une négociation de sécurité avec chaque point de terminaison de messagerie, d'envoi, de règles et DAV enregistré. Aucun identifiant n'est envoyé et aucun courrier n'est remis.
admin-diagx-tab-posture-intro = Paramètres qui protègent les connexions et les sessions, et façon dont fail2ban verrait l'adresse d'une requête.
admin-diagx-tab-tools-intro = Autres endroits où vérifier le service, et limites qui s'appliquent aux exécutions de diagnostic.
admin-diagx-tab-history-intro = Exécutions enregistrées, les plus récentes d'abord. Ouvrez une exécution pour voir ses résultats et ce qui a changé depuis l'exécution précédente.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Avertissement
admin-diagx-status-fail = Échec
admin-diagx-status-skipped = Ignoré
admin-diagx-status-not-run = Non exécuté
admin-diagx-status-running = En cours
admin-diagx-run = Exécuter
admin-diagx-run-again = Exécuter à nouveau
admin-diagx-running = Exécution en cours…
admin-diagx-run-server-note = Exécute de nouveau les vérifications du serveur. Toutes les cartes de cette exécution sont mises à jour.
admin-diagx-run-backends-note = Vérifie de nouveau tous les backends enregistrés. Toutes les cartes de backend sont mises à jour.
admin-diagx-how-to-fix = Comment corriger
admin-diagx-details = Détails
admin-diagx-copy-details = Copier les détails
admin-diagx-copy-value = Copier { $label }
admin-diagx-copied = Copié.
admin-diagx-copy-failed = Le navigateur n'a pas autorisé la copie. Sélectionnez le texte et copiez-le à la place.
admin-diagx-not-run-yet = Pas encore exécuté.
admin-diagx-not-reported = La dernière exécution du serveur n'a pas signalé cette vérification ; elle ne s'applique pas à ce déploiement.
admin-diagx-posture-met = Satisfait
admin-diagx-posture-not-met = Non satisfait
admin-diagx-loading = Chargement…
admin-diagx-none = Aucun
admin-diagx-yes = Oui
admin-diagx-no = Non
admin-diagx-open = Ouvrir
admin-diagx-row-result = Résultat
admin-diagx-row-duration = Durée
admin-diagx-row-checked = Vérifié
admin-diagx-row-id = ID de la vérification
admin-diagx-row-domain = Domaine
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protocole
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Capacités
admin-diagx-row-credentials = Identifiants envoyés
admin-diagx-row-source = Trouvé dans
admin-diagx-row-oauth = Nécessite OAuth
admin-diagx-overview-intro = Le dernier résultat de chaque vérification, par groupe. Tout exécuter lance les vérifications du serveur, puis vérifie chaque backend enregistré.
admin-diagx-run-all = Exécuter toutes les vérifications
admin-diagx-run-all-step = Étape { $step } sur { $steps } : { $suite ->
    [backends] backends enregistrés
   *[server] vérifications du serveur
}.
admin-diagx-run-all-done = Toutes les vérifications sont terminées. Les résultats sont enregistrés dans l'historique.
admin-diagx-backends-progress = { $completed } backends vérifiés sur { $total }.
admin-diagx-counts = { $ok } OK · { $warn } avertissements · { $fail } échecs · { $notRun } non exécutées
admin-diagx-attention = Requiert votre attention
admin-diagx-attention-none = Aucune vérification n'a signalé d'avertissement ni d'échec lors de sa dernière exécution.
admin-diagx-never-run = Aucune vérification n'a encore été exécutée. Exécutez toutes les vérifications pour obtenir un premier résultat.
admin-diagx-last-server = Dernières vérifications du serveur
admin-diagx-last-backends = Dernières vérifications des backends
admin-diagx-never = Jamais
admin-diagx-open-health = État du service
admin-diagx-check-database-connectivity = Connexion à la base de données
admin-diagx-check-database-connectivity-purpose = La base de données répond à une requête simple.
admin-diagx-check-database-connectivity-fix = Vérifiez que la base de données fonctionne et que MW_DB_PATH ou MW_DATABASE_URL pointe vers elle. Le journal du serveur indique l'erreur de connexion.
admin-diagx-check-database-write = Écritures dans la base de données
admin-diagx-check-database-write-purpose = Une ligne de test peut être écrite, puis elle est annulée.
admin-diagx-check-database-write-fix = Vérifiez l'espace disque libre et que le fichier de base de données ou l'utilisateur peut écrire. Un volume en lecture seule ou un disque plein empêche les écritures.
admin-diagx-check-database-migrations = Migrations de la base de données
admin-diagx-check-database-migrations-purpose = La base de données contient toutes les migrations attendues par cette version du serveur.
admin-diagx-check-database-migrations-fix = Redémarrez le serveur pour qu'il applique les migrations en attente, et consultez son journal pour repérer une migration qui a échoué. N'exécutez pas un serveur plus ancien sur une base de données plus récente.
admin-diagx-check-server-key-roundtrip = Clé du serveur
admin-diagx-check-server-key-roundtrip-purpose = La clé du serveur scelle et ouvre une valeur de test.
admin-diagx-check-server-key-roundtrip-fix = Vérifiez que MW_SERVER_KEY (ou le fichier de clé) est la clé avec laquelle cette base de données a été créée. Les secrets stockés ne peuvent pas être ouverts avec une autre clé.
admin-diagx-check-other-purpose = Une vérification signalée par ce serveur.
admin-diagx-check-other-fix = Ouvrez les détails pour voir le résultat brut et consultez le journal du serveur au moment de l'exécution.
admin-diagx-check-network-public-dns = Nom public
admin-diagx-check-network-public-dns-purpose = Le nom d'hôte de MW_PUBLIC_URL se résout depuis ce serveur.
admin-diagx-check-network-public-dns-fix = Vérifiez l'enregistrement DNS du nom public, ou définissez MW_PUBLIC_URL sur l'adresse que les utilisateurs emploient pour joindre ce serveur.
admin-diagx-check-network-public-port = Port public
admin-diagx-check-network-public-port-purpose = Le port de l'URL publique accepte une connexion depuis ce serveur.
admin-diagx-check-network-public-port-fix = Vérifiez le pare-feu, le proxy inverse et la redirection de port. Certains routeurs ne peuvent pas se connecter à leur propre adresse publique (NAT hairpinning) ; testez aussi depuis l'extérieur de votre réseau.
admin-diagx-check-network-backend-ports = Ports des backends
admin-diagx-check-network-backend-ports-purpose = L'hôte et le port de chaque backend enregistré acceptent une connexion directe.
admin-diagx-check-network-backend-ports-fix = Vérifiez que les hôtes des backends sont en marche et qu'un pare-feu autorise ce serveur à les joindre. Backends de messagerie indique quel point de terminaison échoue à la négociation.
admin-diagx-check-egress-private-policy = Protection des adresses privées
admin-diagx-check-egress-private-policy-purpose = Par défaut, les connexions sortantes vers les adresses de bouclage et privées sont refusées.
admin-diagx-check-egress-private-policy-fix = Examinez les paramètres de sortie sous Confidentialité. N'autorisez les adresses privées que pour les backends nommés qui en ont besoin.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Les requêtes atteignent ce serveur via HTTPS.
admin-diagx-check-security-https-fix = Servez la console en HTTPS, ou définissez MW_PUBLIC_URL sur https et MW_TRUSTED_PROXIES lorsqu'un proxy inverse termine TLS.
admin-diagx-check-security-secure-cookies = Cookies sécurisés
admin-diagx-check-security-secure-cookies-purpose = Les cookies de session ne sont envoyés que via HTTPS.
admin-diagx-check-security-secure-cookies-fix = Définissez MW_COOKIE_SECURE sur true dès que le serveur est joint en HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Les navigateurs reçoivent l'instruction d'utiliser HTTPS pour ce site.
admin-diagx-check-security-hsts-fix = Activez Strict-Transport-Security dans les paramètres de sécurité, ou envoyez-le depuis le proxy inverse.
admin-diagx-check-security-csrf = Protection CSRF
admin-diagx-check-security-csrf-purpose = Les requêtes qui modifient des données doivent porter le jeton CSRF.
admin-diagx-check-security-csrf-fix = Définissez MW_CSRF_STRICT sur true.
admin-diagx-check-security-admin-factors = Second facteur des administrateurs
admin-diagx-check-security-admin-factors-purpose = Chaque administrateur dispose d'un second facteur de connexion.
admin-diagx-check-security-admin-factors-fix = Demandez à chaque administrateur qui n'en a pas d'ajouter une clé d'accès ou une application d'authentification dans Mes préférences, ou exigez-le dans le centre de sécurité.
admin-diagx-check-security-signin-pacing = Délai de nouvelle tentative de connexion
admin-diagx-check-security-signin-pacing-purpose = Une courte attente suit une connexion échouée.
admin-diagx-check-security-signin-pacing-fix = Activez le cadencement des connexions dans la politique de sécurité.
admin-diagx-check-security-credential-locks = Verrouillages d'identifiants
admin-diagx-check-security-credential-locks-purpose = Des échecs de connexion répétés verrouillent l'identifiant pendant un temps.
admin-diagx-check-security-credential-locks-fix = Activez les verrouillages automatiques dans le centre de sécurité.
admin-diagx-check-security-geography = Données de géographie des connexions
admin-diagx-check-security-geography-purpose = La politique de géographie des connexions dispose des données de pays dont elle a besoin.
admin-diagx-check-security-geography-fix = Installez ou mettez à jour la base de données des pays, ou désactivez la politique de géographie.
admin-diagx-check-security-render-isolation = Rendu isolé des messages
admin-diagx-check-security-render-isolation-purpose = Les messages sont rendus dans le worker distinct et isolé.
admin-diagx-check-security-render-isolation-fix = Vérifiez que MW_RENDER_BIN pointe vers le programme mw-render et qu'il démarre. L'état du service dans le tableau de bord indique l'état du worker.
admin-diagx-check-security-deployment = Paramètres de déploiement
admin-diagx-check-security-deployment-purpose = Les paramètres de sécurité du déploiement peuvent être lus.
admin-diagx-check-security-deployment-fix = Vérifiez la connexion à la base de données, puis ouvrez l'évaluation de sécurité du déploiement.
admin-diagx-check-security-plaintext-backends = Mots de passe sur des connexions non chiffrées
admin-diagx-check-security-plaintext-backends-purpose = Aucun backend enregistré n'accepte de mots de passe sans TLS.
admin-diagx-check-security-plaintext-backends-fix = Modifiez les connexions du domaine et choisissez TLS ou STARTTLS. N'autorisez les connexions non chiffrées que sur un réseau privé que vous contrôlez.
admin-diagx-check-fail2ban-client-address = Adresse client pour fail2ban
admin-diagx-check-fail2ban-client-address-purpose = L'adresse que nommerait une ligne de journal fail2ban est l'adresse réelle du client.
admin-diagx-check-fail2ban-client-address-fix = Derrière un proxy inverse, définissez MW_TRUSTED_PROXIES sur l'adresse du proxy, sinon fail2ban bannit le proxy au lieu du client. Une adresse de bouclage est normale lorsque vous ouvrez la console sur le serveur lui-même.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Point de terminaison { $protocol } : connexion et négociation de sécurité, sans authentification.
admin-diagx-backend-other-purpose = Une remarque de l'exécution des backends.
admin-diagx-backend-fix = Vérifiez le nom d'hôte, le port et le mode TLS enregistrés pour ce backend, que le DNS se résout et que le certificat est valide pour le nom d'hôte. Exécutez ensuite de nouveau.
admin-diagx-backend-config-fix = Ouvrez les connexions du domaine et enregistrez-les de nouveau ; la liste des backends enregistrés n'a pas pu être lue.
admin-diagx-backend-results = Vérifications des backends
admin-diagx-backends-not-run = Les backends enregistrés n'ont pas encore été vérifiés.
admin-diagx-backends-run-at = Dernière vérification { $date } : { $count } résultats.
admin-diagx-failover-title = Basculement
admin-diagx-failover-purpose = Indique si un domaine peut passer à un autre backend de messagerie lorsque son premier choix est injoignable.
admin-diagx-failover-state = Basculement { $state ->
    [on] activé
   *[off] désactivé
} · { $backends } backends de messagerie{ $enabled ->
    [no] {" "}· domaine désactivé
   *[yes] {""}
}
admin-diagx-no-domains = Aucun domaine enregistré.
admin-diagx-domain-tools = Outils de domaine
admin-diagx-domain-tools-purpose = Vérifications ponctuelles pour un domaine enregistré. Elles ne se connectent pas et n'envoient pas de courrier.
admin-diagx-domain = Domaine
admin-diagx-tool-dns = Enregistrements DNS
admin-diagx-tool-dns-purpose = Recherche les enregistrements MX, SPF, DMARC et de service (SRV) du domaine.
admin-diagx-tool-dns-fix = Ajoutez les enregistrements manquants chez votre fournisseur DNS. Sans MX, le domaine ne reçoit aucun courrier ; sans SPF et DMARC, d'autres serveurs peuvent rejeter son courrier.
admin-diagx-srv-found = { $found } enregistrements de service trouvés sur { $total }
admin-diagx-tool-tls = TLS et négociation
admin-diagx-tool-tls-purpose = Se connecte à chaque point de terminaison enregistré du domaine et indique la version de TLS, l'expiration du certificat et le temps écoulé.
admin-diagx-tool-tls-fix = Pour un point de terminaison injoignable, vérifiez l'hôte, le port et le pare-feu. Renouvelez un certificat qui expire dans les 14 jours.
admin-diagx-probe-none = Le domaine n'a aucun point de terminaison enregistré à tester.
admin-diagx-probe-line = { $host } · joignable : { $reachable ->
    [yes] oui
   *[no] non
} · TLS { $tls } · certificat { $days ->
    [none] non signalé
   *[other] expire dans { $days } jours
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = Découvrir les paramètres
admin-diagx-tool-detect-purpose = Recherche les paramètres de messagerie publiés (autoconfig, autodiscover, SRV) pour un nom de domaine. Rien n'est enregistré.
admin-diagx-tool-detect-fix = Publiez des enregistrements autoconfig ou SRV pour le domaine, ou saisissez les paramètres manuellement sous Domaines.
admin-diagx-detect-name = Nom de domaine à rechercher
admin-diagx-tool-health = État du service
admin-diagx-tool-health-purpose = Synchronisation du courrier, boîte d'envoi, index de recherche, worker de rendu et plugins, dans le tableau de bord.
admin-diagx-tool-deployment-purpose = Chaque paramètre de sécurité du déploiement que ce serveur peut observer, avec le paramètre qui le contrôle.
admin-diagx-tool-ssh = Tests de tunnel SSH
admin-diagx-tool-ssh-purpose = Testez les tunnels SSH utilisés pour joindre des backends situés sur un autre réseau.
admin-diagx-tool-geo = Géographie des connexions
admin-diagx-tool-geo-purpose = D'où viennent les connexions, et la base de données des pays qui sous-tend la politique de géographie.
admin-diagx-catalogue-purpose = Ce que couvre chaque suite de diagnostic, et les limites que chaque exécution respecte.
admin-diagx-suite-unsupported = Non disponible : { $reason }
admin-diagx-suite-server = Vérifications du serveur
admin-diagx-suite-backends = Backends enregistrés
admin-diagx-run-title = { $suite ->
    [backends] Backends enregistrés
   *[server] Vérifications du serveur
}, { $date }
admin-diagx-changes = Modifications
admin-diagx-changes-first = Aucune exécution antérieure de ce type à comparer.
admin-diagx-changes-none = Rien n'a changé depuis l'exécution du { $date }.
admin-diagx-changes-since = Modifié depuis l'exécution du { $date } :
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Rapport de dépannage
admin-diagx-report-purpose = Un rapport JSON pour l'assistance : indicateurs d'exécution, derniers résultats et exécution de votre choix. Les mots de passe, jetons, cookies, clés, configuration brute et courriers n'y figurent jamais.
admin-diagx-report-run = Exécution à inclure
admin-diagx-report-preview = Aperçu
admin-crumb-server = Serveur
admin-crumb-connectivity = Connectivité
admin-crumb-posture = Posture de sécurité
admin-crumb-tools = Outils
admin-crumb-history = Historique
admin-crumb-report = Rapport
admin-diagx-table-actions = Actions
admin-diagx-check-pdfjs-viewer-assets = Fichiers de la visionneuse PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = Les fichiers de la visionneuse PDF sont tous présents, correctement typés et correspondent à ce build du serveur.
admin-diagx-check-pdfjs-viewer-assets-fix = Redéployez le build web, ou supprimez le dossier pdfjs de MW_WEB_DIR afin d'utiliser la copie intégrée au serveur. Si la vérification échoue au lieu de simplement avertir, reconstruisez l'image du serveur.
