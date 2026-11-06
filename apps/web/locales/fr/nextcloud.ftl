# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Dossier parent
nextcloud-loading = Chargement…
nextcloud-list-error = Impossible de lister ce dossier.
nextcloud-empty = Ce dossier est vide.
nextcloud-file-list = Fichiers Nextcloud
nextcloud-open-folder = Ouvrir le dossier { $name }
nextcloud-select-file = Sélectionner { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Joindre depuis Nextcloud
nextcloud-attach-action = { $count ->
    [0] Joindre
    [one] Joindre { $count } fichier
   *[other] Joindre { $count } fichiers
  }
nextcloud-error-select-file = sélectionnez au moins un fichier à joindre
nextcloud-error-attach-failed = impossible de joindre les fichiers sélectionnés

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Enregistrer dans Nextcloud
nextcloud-save-title = Enregistrer « { $name } » dans Nextcloud
nextcloud-save-action = Enregistrer ici ({ $dir })
nextcloud-saved = Enregistré dans { $path }
nextcloud-error-save-failed = impossible d’enregistrer dans Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Créer un lien de partage
nextcloud-share-title = Lien de partage
nextcloud-share-intro = Créer un lien public vers { $path } au lieu de joindre le fichier.
nextcloud-protect-password = Protéger par mot de passe
nextcloud-password-label = Mot de passe
nextcloud-share-password = Mot de passe du partage
nextcloud-set-expiry = Définir une date d’expiration
nextcloud-expires-on = Expire le
nextcloud-expiry-date = Date d’expiration
nextcloud-create-link = Créer le lien
nextcloud-password-protected = Protégé par mot de passe
nextcloud-no-password = Sans mot de passe
nextcloud-expires = expire le { $date }
nextcloud-no-expiry = sans expiration
nextcloud-error-need-password = saisissez un mot de passe ou désactivez la protection par mot de passe
nextcloud-error-need-expiry = choisissez une date d’expiration ou désactivez l’expiration
nextcloud-error-share-failed = impossible de créer le lien de partage

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } dépasse la limite des pièces jointes. Téléversez-le dans le dossier « Mail attachments » de votre Nextcloud et insérez plutôt un lien de partage dans le message.
nextcloud-upload-and-link = Téléverser et insérer le lien
nextcloud-share-not-now = Pas maintenant
nextcloud-upload-progress = Téléversement : { $percent } %
nextcloud-upload-progress-label = Progression du téléversement
nextcloud-share-inserted = Le lien de partage de { $name } a été inséré dans le message.
nextcloud-share-body = { $name } ({ $size }) : { $url }
nextcloud-share-body-expires = Le lien fonctionne jusqu'au { $date }.
nextcloud-share-body-password = Le lien est protégé par un mot de passe.
nextcloud-account-title = Comptes Nextcloud
nextcloud-account-help = Associez votre propre compte à une instance approuvée par votre administrateur. Les identifiants sont stockés chiffrés pour ce seul compte de messagerie.
nextcloud-account-load-error = Impossible de charger les liaisons Nextcloud. Rechargez la page pour réessayer.
nextcloud-account-unlink-error = Impossible de supprimer la liaison. Rechargez la page avant de réessayer.
nextcloud-account-reload = Recharger
nextcloud-account-no-instances = Aucune instance Nextcloud activée n'est disponible. Demandez à votre administrateur d'en configurer une.
nextcloud-account-ready = Associé et prêt
nextcloud-account-stale = La configuration de l'instance a changé. Vérifiez à nouveau la liaison.
nextcloud-account-unlinked = Non associé
nextcloud-account-relink = Vérifier à nouveau la liaison
nextcloud-account-link = Associer le compte
nextcloud-account-unlink = Dissocier le compte
nextcloud-account-password-help = Dans les paramètres de sécurité personnels de Nextcloud, créez un mot de passe d'application et copiez ici son nom d'utilisateur et son mot de passe. La vérification effectue une seule requête bornée ; les identifiants refusés ne sont pas réessayés.
nextcloud-account-username = Nom d'utilisateur du mot de passe d'application Nextcloud
nextcloud-account-password = Mot de passe d'application Nextcloud
nextcloud-account-verify = Vérifier et associer
nextcloud-account-verify-error = La vérification ou l'enregistrement a échoué. Contrôlez l'instance, le nom d'utilisateur et le mot de passe d'application. Le champ du mot de passe a été vidé ; une liaison existante qui fonctionne n'a pas été remplacée.
nextcloud-admin-title = Instances Nextcloud
nextcloud-admin-help = Approuvez les instances HTTPS que les utilisateurs de messagerie peuvent associer individuellement. Aucun mot de passe de service partagé n'est donné aux comptes de messagerie. Les opérations sur les fichiers sont limitées à 8 Mio. Les utilisateurs du moteur natif découvrent et connectent explicitement les collections d'agendas, de tâches et de carnets d'adresses après l'association.
nextcloud-admin-add = Ajouter une instance Nextcloud
nextcloud-admin-edit = Modifier l'instance Nextcloud
nextcloud-admin-enabled = Activée
nextcloud-admin-disabled = Désactivée
nextcloud-admin-missing = Cette instance n'existe plus. Retournez à la liste des instances.
nextcloud-admin-private-help = Les adresses privées exigent une autorisation de sortie Nextcloud explicite. Les routes de proxy configurées restent prioritaires ; une requête refusée par le proxy ne bascule jamais vers une connexion directe.
nextcloud-admin-revalidation = Toute modification d'une instance oblige les utilisateurs à vérifier à nouveau leurs liaisons. Désactiver une instance arrête immédiatement les nouvelles opérations.
nextcloud-admin-authority = Autorité de l'instance
nextcloud-admin-name = Nom d'affichage
nextcloud-admin-name-help = Un nom court affiché aux utilisateurs lorsqu'ils choisissent une instance.
nextcloud-admin-url = URL d'installation HTTPS
nextcloud-admin-url-help = Utilisez la racine de l'installation, par exemple https://cloud.example.org/ ou https://example.org/nextcloud/. Le certificat doit être approuvé et correspondre au nom d'hôte. Les identifiants, paramètres de requête et fragments sont refusés.
nextcloud-admin-enable = Autoriser les utilisateurs à associer cette instance
nextcloud-admin-ca = Autorités de certification supplémentaires (PEM)
nextcloud-admin-ca-help = Certificats d'AC privées facultatifs pour ce serveur Nextcloud. La confiance publique reste disponible et la vérification du nom d'hôte reste obligatoire. Les modifications obligent les utilisateurs à vérifier à nouveau leurs liaisons ; la confiance envers les certificats du proxy est distincte.
nextcloud-instance-select = Instance Nextcloud
nextcloud-instance-placeholder = Choisir une instance associée
nextcloud-instances-empty = Aucune liaison Nextcloud vérifiée. Ajoutez ou vérifiez une liaison dans Paramètres utilisateur → Nextcloud.
nextcloud-instances-error = Impossible de charger les instances associées. Rouvrez le sélecteur pour réessayer.
nextcloud-save-warning = Votre administrateur exige une confirmation avant que cette pièce jointe puisse quitter la boîte aux lettres.
nextcloud-save-acknowledge = J'ai compris l'avertissement sur la pièce jointe et je souhaite enregistrer ce fichier dans l'instance Nextcloud sélectionnée.
nextcloud-save-attachment = Enregistrer { $name } dans Nextcloud
nextcloud-save-close = Fermer la destination Nextcloud
nextcloud-save-options = Enregistrer les pièces jointes dans Nextcloud
nextcloud-pim-title = Contacts et agendas
nextcloud-pim-help = Découvrez les collections de ce compte Nextcloud vérifié, puis connectez explicitement celles que vous souhaitez. Synchronisez pour récupérer les données actuelles ; les modifications des collections connectées s'appliquent à la collection distante et échouent si sa liaison ou ses autorisations sont révoquées.
nextcloud-pim-category = Catégorie de collection
nextcloud-pim-calendar = Agendas
nextcloud-pim-contacts = Carnets d'adresses
nextcloud-pim-tasks = Listes de tâches
nextcloud-pim-discover = Découvrir les collections
nextcloud-pim-discovered = Collections disponibles
nextcloud-pim-empty = Aucune collection correspondante n'a été renvoyée.
nextcloud-pim-connect = Connecter
nextcloud-pim-connected = Collections connectées
nextcloud-pim-sync = Synchroniser maintenant
nextcloud-pim-sync-needed = Collection connectée. Synchronisez maintenant pour charger son contenu.
nextcloud-pim-synced = Synchronisation terminée.
nextcloud-pim-error = L'opération a échoué. Vérifiez la liaison du compte et les autorisations, puis réessayez explicitement. Les données déjà en cache n'ont pas été supprimées.
nextcloud-pim-native-required = La synchronisation des contacts et des agendas nécessite le moteur de messagerie natif. L'accès aux fichiers reste disponible via la liaison de votre compte.
nextcloud-pim-disconnect = Déconnecter
nextcloud-pim-disconnect-help = La déconnexion conserve les données en cache et arrête les écritures distantes. Les comptes associés à nouveau nécessitent une nouvelle découverte et une connexion explicite.
nextcloud-pim-ready = Prêt
nextcloud-pim-disconnected = Déconnecté ou liaison modifiée
nextcloud-pim-unnamed = Collection sans nom
nextcloud-pim-back = Retour aux comptes Nextcloud
nextcloud-pim-create-title = Nouvelle collection dans Nextcloud
nextcloud-pim-create-help = Crée un agenda ou une liste de tâches vide dans ce compte Nextcloud et le connecte ici.
nextcloud-pim-create-name = Nom de la nouvelle collection
nextcloud-pim-create = Créer dans Nextcloud
nextcloud-pim-created = { $name } a été créé dans Nextcloud et connecté. Synchronisez maintenant pour le charger.
nextcloud-pim-interval = Intervalle d'actualisation automatique (secondes)
nextcloud-pim-interval-help = Zéro n'utilise que l'actualisation manuelle. Les intervalles positifs ont un minimum de 60 secondes. Les comptes connectés s'actualisent en arrière-plan ; en cas d'échec le délai augmente, et les identifiants refusés interrompent les requêtes jusqu'à une nouvelle association.
nextcloud-pim-interval-reset = Utiliser la valeur par défaut de l'administrateur
nextcloud-admin-empty = Aucune instance approuvée pour l'instant. Ajoutez une instance pour la rendre disponible à l'association de comptes.
nextcloud-admin-status = Statut
nextcloud-admin-open = Ouvrir { $name }
