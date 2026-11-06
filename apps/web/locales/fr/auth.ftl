# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-email = Adresse e-mail
auth-password = Mot de passe
auth-sign-in = Se connecter
auth-signing-in = Connexion…
auth-invalid-credentials = Identifiants non valides
auth-refused-note = Si l’adresse e-mail et le mot de passe sont corrects, le compte a peut-être été désactivé par un administrateur. Un compte désactivé ne peut pas se connecter tant qu’un administrateur ne l’a pas réactivé.
auth-unreachable = Impossible de joindre le serveur
auth-address-not-configured = Cette adresse n’est pas configurée sur ce serveur. Contactez votre administrateur.

# -- Single sign-on (t9) -----------------------------------------------------
auth-sso-heading = Authentification unique
auth-sso-divider = ou continuer avec
auth-sso-button = Se connecter avec { $name }
auth-sso-error = L’authentification unique n’a pas abouti. Réessayez ou connectez-vous avec votre mot de passe.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Autoriser l’application
auth-consent-title = Autoriser l’accès
auth-consent-loading = Chargement de la demande…
auth-consent-intro = souhaite accéder à votre compte.
auth-consent-approved = Client approuvé par l’administrateur
auth-consent-unapproved = Client non reconnu — non approuvé par l’administrateur
auth-consent-requesting = Il demande
auth-consent-redirects-to = Redirige vers
auth-consent-for-resource = Pour la ressource
auth-consent-deny = Refuser
auth-consent-allow = Autoriser
auth-consent-error = impossible d’enregistrer votre décision

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Trop de tentatives de connexion. Veuillez patienter avant de réessayer.
auth-retry-countdown = Réessayez dans { $seconds } s
auth-upstream-protocol = Connexion interrompue : le serveur de messagerie a envoyé une réponse que ce serveur n’a pas pu lire. Réessayez plus tard ou prévenez votre administrateur.
auth-account-disabled = Ce compte a été désactivé par un administrateur. Il ne peut pas se connecter tant qu’un administrateur ne l’a pas réactivé.
auth-signin-not-allowed = Cette adresse n’est pas autorisée à se connecter sur ce serveur. Contactez votre administrateur.
auth-signin-not-provisioned = Aucun compte n’existe encore pour cette adresse, et les nouveaux comptes ne sont pas créés automatiquement. Demandez à votre administrateur de vous ajouter.
auth-reference = Référence : { $id }
auth-domain-hint = Pour { $domain }, vous pouvez vous connecter avec votre seul nom d’utilisateur.
auth-passkey-sign-in = Se connecter avec une clé d’accès
auth-passkey-failed = La clé d’accès n’a pas permis de vous connecter. Réessayez ou utilisez votre mot de passe.
auth-remembered = Connexions précédentes
auth-remembered-help = Adresses utilisées avec succès sur cet appareil. Le choix d’une adresse ne remplit que l’adresse.
auth-forget-address = Oublier { $address } sur cet appareil
auth-login-timeout = La connexion a expiré. Aucune nouvelle tentative automatique n’a été faite. Vérifiez la connexion et réessayez quand vous êtes prêt.
auth-verification-title = Vérification de connexion
auth-verification-loading = Chargement de la vérification de connexion…
auth-verification-error = La vérification n’a pas pu être terminée. Relancez la vérification avant de vous connecter.
auth-verification-required = Effectuez une nouvelle vérification de connexion avant de réessayer.
auth-identity-title = Partager votre identité
auth-identity-sign-in = Reconnectez-vous pour poursuivre cette demande d’identité.
auth-identity-sign-out = Se déconnecter de Mailwoman
auth-identity-expired = Cette demande d’identité a expiré ou a été modifiée. Retournez à l’application demandeuse et recommencez.
auth-identity-retry = Réessayer
auth-identity-disclosure = Cette application reçoit les informations d’identité ci-dessous. Cette autorisation ne donne pas accès à vos e-mails.
auth-identity-scope-openid = Un identifiant d’identité stable
auth-identity-scope-profile = Les informations de profil disponibles
auth-identity-scope-email = Votre adresse e-mail de connexion, sans garantie qu’elle ait été vérifiée de façon indépendante
auth-identity-scope-offline_access = Renouveler l’accès à l’identité tant que cette session Mailwoman reste valide, jusqu’à 15 jours après la connexion
auth-identity-deny = Annuler
auth-identity-approve = Autoriser l’accès à l’identité
auth-identity-grants = Applications utilisant votre identité
auth-identity-grants-help = Révoquez l’accès à l’identité pour une application. Cela ne supprime pas les autorisations distinctes de l’API de messagerie.
auth-identity-no-grants = Aucune application ne dispose actuellement d’un consentement d’identité.
auth-identity-revoke = Révoquer l’accès à l’identité
auth-maintenance = Le webmail est temporairement suspendu pour maintenance. Veuillez réessayer plus tard.
