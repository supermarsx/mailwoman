# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Actualiser
admin-dash-unavailable = Les chiffres du tableau de bord ne sont pas encore disponibles.
admin-dash-unavailable-detail = Ce serveur ne fournit pas les données du tableau de bord (GET /admin/dashboard répond 404). Elles arriveront avec une mise à jour ultérieure du serveur. Les autres sections fonctionnent comme avant.
admin-dash-error-network = Le tableau de bord n’a pas pu être chargé : le serveur n’a pas répondu. Réessayez.
admin-dash-error-status = Le tableau de bord n’a pas pu être chargé : le serveur a répondu { $status }. Réessayez ; si cela se reproduit, consultez le journal du serveur.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = À traiter
admin-dash-warn-fix = Ouvrir la section qui corrige ce problème
admin-dash-warn-blocked-accounts = { $count ->
    [one] { $count } compte ne peut pas se connecter car son domaine n’a aucune configuration.
   *[other] { $count } comptes ne peuvent pas se connecter car leurs domaines n’ont aucune configuration.
}
admin-dash-warn-no-trusted-proxy = Des règles IP sont actives mais aucun proxy de confiance n’est configuré : tous les clients peuvent donc sembler provenir de l’adresse du proxy.
admin-dash-warn-allowlist-bypass = Une variable d’environnement désactive la liste d’autorisation IP.
admin-dash-warn-metrics-token = Aucun jeton n’est défini pour le point de terminaison des métriques.
admin-dash-warn-pending-migrations = { $count ->
    [one] { $count } migration de base de données n’a pas été appliquée. Redémarrez le serveur pour l’appliquer.
   *[other] { $count } migrations de base de données n’ont pas été appliquées. Redémarrez le serveur pour les appliquer.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] { $count } certificat amont expire bientôt (d’après le dernier test de connexion).
   *[other] { $count } certificats amont expirent bientôt (d’après le dernier test de connexion).
}
admin-dash-warn-unknown = Avertissement du serveur { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] { $count } compte bloqué : aucune configuration de domaine
   *[other] { $count } comptes bloqués : aucune configuration de domaine
}
admin-dash-blocked-note = Ces comptes se connectaient avant cette mise à jour, mais leur domaine n’a aucune configuration dans la console : ils ne peuvent donc plus se connecter. Configurer le domaine leur permet de se connecter à nouveau.
admin-dash-blocked-chart = Comptes bloqués par domaine
admin-dash-blocked-row = { $count ->
    [one] { $count } compte
   *[other] { $count } comptes
}
admin-dash-blocked-seen = (dernier serveur de messagerie utilisé : { $host })
admin-dash-configure-domain = Configurer { $domain }
admin-dash-blocked-no-access = Votre rôle ne permet pas de configurer les domaines.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Liste de mise en route : rien en attente
    [one] Liste de mise en route : un élément en attente
   *[other] Liste de mise en route : { $open } éléments en attente
}
admin-dash-check-done = Fait
admin-dash-check-open = En attente
admin-dash-check-optional = (facultatif)
admin-dash-check-go = Ouvrir
admin-dash-check-hide = Masquer
admin-dash-check-unhide = Réafficher
admin-dash-check-hidden = { $count ->
    [one] { $count } élément est masqué. Le masquer ne le marque pas comme fait.
   *[other] { $count } éléments sont masqués. Les masquer ne les marque pas comme faits.
}
admin-dash-check-show-hidden = Afficher les éléments masqués
admin-dash-check-hide-hidden = Ne pas afficher les éléments masqués
admin-dash-check-domain-exists = Ajouter un domaine de messagerie
admin-dash-check-domain-upstream = Attribuer un serveur de messagerie au domaine, ou définir un serveur amont par défaut
admin-dash-check-test-connection = Réussir un test de connexion (au cours des 7 derniers jours)
admin-dash-check-preferred-domain = Choisir un domaine préféré, pour que les utilisateurs puissent se connecter sans le saisir
admin-dash-check-second-admin = Créer un second compte administrateur, ou désactiver la connexion par environnement
admin-dash-check-trusted-proxies = Configurer les proxys de confiance placés devant ce serveur
admin-dash-check-security-policy = Enregistrer la politique de sécurité
admin-dash-check-metrics-token = Définir un jeton pour le point de terminaison des métriques
admin-dash-check-branding = Vérifier l’habillage

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Comptes et sessions
admin-dash-users = Utilisateurs créés
admin-dash-signed-in-24h = Comptes connectés, dernières 24 heures
admin-dash-signed-in-7d = Comptes connectés, 7 derniers jours
admin-dash-domains = Domaines
admin-dash-domains-detail = { $upstream } avec un serveur de messagerie, { $disabled } désactivé(s)
admin-dash-sessions = Sessions webmail actives
admin-dash-admin-sessions = Sessions d’administration actives
admin-dash-security = Sécurité des connexions
admin-dash-failed-webmail = Connexions webmail échouées, dernières 24 heures
admin-dash-failed-admin = Connexions administrateur échouées, dernières 24 heures
admin-dash-active-blocks = Adresses bloquées
admin-dash-allowlists = Listes d’autorisation IP
admin-dash-allowlists-value = Admin { $admin }, webmail { $webmail }
admin-dash-on = activée
admin-dash-off = désactivée
admin-dash-trusted-proxies = Proxys de confiance
admin-dash-configured = Configurés
admin-dash-not-configured = Non configurés
admin-dash-untrusted-forwarded = { $count ->
    [one] { $count } requête en 24 heures portait des en-têtes de transfert provenant d’un pair non fiable
   *[other] { $count } requêtes en 24 heures portaient des en-têtes de transfert provenant d’un pair non fiable
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Trafic, { $mins } dernières minutes
admin-dash-traffic-note = Compté par ce processus serveur depuis son démarrage. Les autres processus ou réplicas ne sont pas inclus.
admin-dash-traffic-empty = Aucune requête n’a été comptée sur cette période.
admin-dash-requests = Requêtes par minute
admin-dash-errors = Erreurs serveur (5xx) par minute
admin-dash-p95 = Temps de réponse, 95e centile

# -- Health ----------------------------------------------------------------------------
admin-dash-health = État de santé
admin-dash-h-ok = OK
admin-dash-h-attention = À traiter
admin-dash-h-db = Base de données
admin-dash-h-migrations = Migrations
admin-dash-h-engine = Synchronisation du courrier
admin-dash-h-outbox = Boîte d’envoi
admin-dash-h-index = Index de recherche
admin-dash-h-sandbox = Bac à sable de rendu
admin-dash-h-plugins = Environnement d’exécution des plugins

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Stockage
admin-dash-db-size = Fichier de base de données
admin-dash-wal-size = Journal d’écriture anticipée
admin-dash-index-size = Index de recherche
admin-dash-volume-free = Espace libre sur le volume de données
admin-dash-volume-of = sur { $total }
admin-dash-na-wal = Cette base de données n’utilise pas de fichier de journal d’écriture anticipée, ou le serveur ne l’a pas indiqué.
admin-dash-na-volume = Le serveur ne peut pas lire l’espace libre sur cette plateforme.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Actions d’administration récentes
admin-dash-audit-empty = Aucune action d’administration n’est encore enregistrée.
admin-dash-audit-when = Quand
admin-dash-audit-actor = Qui
admin-dash-audit-action = Action
admin-dash-audit-target = Cible
admin-dash-audit-all = Ouvrir le journal d’audit complet
admin-dash-server = Serveur
admin-dash-version = Version
admin-dash-build = Build
admin-dash-mode = Mode
admin-dash-mode-engine = Moteur : ce serveur synchronise et stocke le courrier
admin-dash-mode-proxy = Proxy : ce serveur transmet les requêtes au serveur de messagerie
admin-dash-uptime = En service depuis
admin-dash-base-path = Servi à
admin-dash-base-root = la racine du site
admin-dash-duration-dh = { $d } j { $h } h
admin-dash-duration-hm = { $h } h { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
