# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Fournisseur d’identité
admin-identity-description = Permettez aux applications approuvées de se connecter avec Mailwoman. Cette fonction est distincte des fournisseurs de connexion externes et de l’accès à l’API de messagerie. Le code d’autorisation avec PKCE est obligatoire ; les autres types d’octroi ne sont pas disponibles.
admin-identity-configure = Configurer l’émetteur
admin-identity-new = Ajouter un client d’identité
admin-identity-edit = Modifier le client
admin-identity-enabled = Activé
admin-identity-disabled = Désactivé
admin-identity-all = Tous
admin-identity-discovery = Ouvrir le document de découverte
admin-identity-search = Rechercher des clients
admin-identity-status = État
admin-identity-clients = Clients d’identité
admin-identity-client = Identité du client
admin-identity-client-id = ID client
admin-identity-name = Nom
admin-identity-method = Authentification du client
admin-identity-public = Client public (PKCE, sans secret)
admin-identity-pkce = Chaque client doit utiliser PKCE S256. Les secrets des clients confidentiels sont générés ici et affichés une seule fois.
admin-identity-rotate-secret = Remplacer le secret client existant
admin-identity-redirects = Destinations enregistrées
admin-identity-redirect-help = Un URI HTTPS exact par ligne ; le HTTP de bouclage numérique est autorisé pour les applications locales. Les caractères génériques et les fragments sont refusés.
admin-identity-callbacks = URI de rappel d’autorisation
admin-identity-logout-uris = URI de redirection après déconnexion
admin-identity-scopes = Portées d’identité autorisées
admin-identity-actions = Actions
admin-identity-delete = Supprimer le client
admin-identity-delete-confirm = Supprimer l’accès d’identité de { $name } ? Les autorisations d’identité existantes cessent de fonctionner et l’enregistrement du client réservé à l’identité est supprimé.
admin-identity-empty = Aucun client d’identité correspondant.
admin-identity-cancel = Annuler
admin-identity-save = Enregistrer
admin-identity-done = Terminé
admin-identity-secret = Secret client généré
admin-identity-secret-once = Copiez ce secret dans l’application dépendante maintenant. Il ne pourra plus être affiché. Ne placez jamais un secret confidentiel dans du code de navigateur.
admin-identity-issuer = URL HTTPS publique de l’émetteur
admin-identity-issuer-help = Utilisez l’origine HTTPS exacte de ce déploiement et le chemin /oidc, y compris tout préfixe de déploiement. La modification de l’émetteur ou de la configuration révoque les autorisations d’identité existantes.
admin-identity-rotate-key = Renouveler la clé de signature
admin-identity-key-help = Le renouvellement nécessite une nouvelle vérification de l’administrateur. Les clés publiques précédentes restent disponibles pendant 15 minutes afin que les jetons émis récemment puissent terminer leur courte durée de vie.
