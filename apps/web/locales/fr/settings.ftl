# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: settings. Source: ../en/settings.ftl

settings-title = Paramètres
settings-appearance = Apparence
settings-close = Fermer les paramètres

settings-theme = Thème
settings-density = Densité
settings-accent = Couleur d’accent
settings-font = Police de l’interface
settings-layout = Disposition

settings-density-compact = Compacte
settings-density-cozy = Confortable
settings-density-relaxed = Aérée

settings-font-default = Par défaut
settings-font-system = Système
settings-font-serif = Serif
settings-font-mono = Mono

settings-layout-default = Par défaut
settings-layout-ribbon = Ruban

settings-cancel = Annuler
settings-edit = Modifier
settings-delete = Supprimer
settings-save = Enregistrer
settings-saved = Enregistré

settings-2fa-title = Authentification à deux facteurs
settings-2fa-intro = Ajoutez un second facteur pour exiger plus qu’un mot de passe à la connexion.
settings-2fa-policy-required = Votre organisation exige un second facteur pour ce compte.
settings-2fa-enabled = Activée
settings-2fa-recovery-title = Codes de récupération
settings-2fa-recovery-once = Enregistrez-les maintenant. Ils ne sont affichés qu’une fois et ne peuvent plus être récupérés. Chaque code ne fonctionne qu’une seule fois.
settings-2fa-recovery-ack = J’ai enregistré ces codes
settings-2fa-totp-title = Application d’authentification
settings-2fa-totp-enrol = Configurer une application d’authentification
settings-2fa-totp-scan = Ajoutez ce secret à votre application d’authentification, puis saisissez le code qu’elle affiche.
settings-2fa-totp-uri-link = Ouvrir dans une application d’authentification
settings-2fa-totp-code-label = Code d’authentification
settings-2fa-totp-confirm = Confirmer
settings-2fa-totp-disable = Supprimer l’application d’authentification
settings-2fa-passkey-title = Clés d’accès
settings-2fa-passkey-remove = Supprimer
settings-2fa-passkey-unsupported = Ce navigateur ne prend pas en charge les clés d’accès.
settings-2fa-passkey-label-placeholder = Nom de la clé d’accès (facultatif)
settings-2fa-passkey-add = Ajouter une clé d’accès
settings-2fa-recovery-remaining = { $count ->
    [one] { $count } code de récupération restant
   *[other] { $count } codes de récupération restants
}
settings-2fa-recovery-regenerate = Générer de nouveaux codes de récupération
settings-2fa-recovery-code-label = Code de récupération
settings-2fa-error-generic = Cela n’a pas fonctionné. Réessayez.
settings-2fa-error-code = Ce code n’a pas pu être vérifié.
settings-2fa-error-passkey = La clé d’accès n’a pas pu être enregistrée.

settings-2fa-challenge-title = Authentification à deux facteurs
settings-2fa-challenge-intro = Confirmez votre second facteur pour terminer la connexion.
settings-2fa-challenge-failed = La vérification a échoué. Réessayez.
settings-2fa-challenge-use-passkey = Utiliser une clé d’accès
settings-2fa-challenge-method = Choisir une méthode
settings-2fa-challenge-totp-tab = Code d’authentification
settings-2fa-challenge-recovery-tab = Code de récupération
settings-2fa-challenge-verify = Vérifier

settings-sessions-title = Sessions actives
settings-sessions-intro = Sessions actuellement connectées à ce compte.
settings-sessions-last-seen = Dernière activité { $when }
settings-sessions-revoke = Déconnecter
settings-sessions-current = Cette session
settings-sessions-revoke-others = { $count ->
    [one] Déconnecter { $count } autre session
   *[other] Déconnecter { $count } autres sessions
}
settings-sessions-error = La session n’a pas pu être modifiée.

settings-sig-title = Signatures
settings-sig-intro = Texte ajouté à la fin des messages que vous envoyez. Une signature peut être définie par défaut.
settings-sig-default = Par défaut
settings-sig-new = Nouvelle signature
settings-sig-name-label = Nom
settings-sig-body-label = Signature
settings-sig-default-label = Utiliser par défaut
settings-sig-error-name = Donnez un nom à la signature.
settings-sig-error-generic = La signature n’a pas pu être enregistrée.

settings-ident-title = Identités
settings-ident-intro = Le nom et l’adresse sous lesquels vous envoyez. Chaque identité peut utiliser sa propre signature.
settings-ident-new = Nouvelle identité
settings-ident-name-label = Nom affiché
settings-ident-email-label = Adresse e-mail
settings-ident-replyto-label = Adresse de réponse (facultatif)
settings-ident-signature-label = Signature par défaut
settings-ident-signature-none = Aucune
settings-ident-error-fields = Saisissez un nom et une adresse e-mail valide.
settings-ident-error-generic = L’identité n’a pas pu être enregistrée.

settings-notif-title = Notifications
settings-notif-intro = Choisissez quels nouveaux messages vous sont notifiés, et quand rester silencieux.
settings-notif-enabled-label = Afficher des notifications pour le nouveau courrier
settings-notif-quiet-title = Heures calmes
settings-notif-quiet-enabled-label = Désactiver les notifications pendant une plage horaire
settings-notif-quiet-start = De
settings-notif-quiet-end = À
settings-notif-rules-title = Règles
settings-notif-rule-match-placeholder = L’expéditeur, le dossier ou l’objet contient…
settings-notif-rule-action-label = Action
settings-notif-rule-notify = Notifier
settings-notif-rule-mute = Ignorer
settings-notif-rule-add = Ajouter une règle
settings-notif-error = Les paramètres de notification n’ont pas pu être enregistrés.

settings-search-title = Recherches enregistrées
settings-search-intro = Afficher une recherche enregistrée comme dossier dans la liste de vos dossiers.
settings-search-empty = Vous n’avez encore aucune recherche enregistrée.
settings-search-as-folder = Afficher { $name } comme dossier
settings-search-as-folder-label = Afficher comme dossier
settings-search-error = La recherche enregistrée n’a pas pu être modifiée.

settings-kbd-title = Raccourcis clavier
settings-kbd-intro = Choisissez un jeu de raccourcis. Les modifications s’appliquent sur cet appareil.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Désactivés (aucun raccourci)
settings-kbd-action-compose = Nouveau message
settings-kbd-action-archive = Archiver
settings-kbd-action-reply = Répondre
settings-kbd-action-next = Message suivant
settings-kbd-action-previous = Message précédent
settings-kbd-action-search = Rechercher

settings-offline-title = Cache hors ligne
settings-offline-intro = Quelle quantité de courrier conserver sur cet appareil pour une utilisation hors ligne, et comment libérer de l’espace.
settings-offline-budget-label = Taille du cache (Mo)
settings-offline-retention-label = Conserver pendant (jours)
settings-offline-strategy-label = Lorsque la taille est atteinte
settings-offline-lru = Supprimer les moins récemment utilisés
settings-offline-oldest = Supprimer les plus anciens d’abord
settings-offline-manual = Uniquement quand je le vide
settings-offline-purge = Vider le cache hors ligne maintenant

settings-dir-title = Sens de l’interface
settings-dir-intro = Suivre la langue, ou imposer un sens. De droite à gauche inverse la disposition.
settings-dir-auto = Automatique
settings-dir-ltr = De gauche à droite
settings-dir-rtl = De droite à gauche
settings-dir-preview-title = Aperçu
settings-dir-preview-body = Ce texte s’inverse lorsque le sens est de droite à gauche.
settings-mail-features = Fonctions de messagerie
settings-mail-feature-allowSend = Autoriser l’envoi de messages
settings-mail-feature-allowSend-help = Si désactivé, Envoyer et son raccourci sont désactivés. Les brouillons peuvent toujours être enregistrés.
settings-mail-feature-allowReply = Autoriser Répondre et Répondre à tous
settings-mail-feature-allowReply-help = Si désactivé, les actions Répondre et Répondre à tous et leurs raccourcis ne sont pas proposées.
settings-mail-feature-allowForward = Autoriser le transfert
settings-mail-feature-allowForward-help = Si désactivé, les actions Transférer et leur raccourci ne sont pas proposées.
settings-mail-feature-allowDownload = Autoriser le téléchargement des messages et des pièces jointes
settings-mail-feature-allowDownload-help = Si désactivé, le téléchargement des messages et des pièces jointes n’est pas proposé.
settings-mail-feature-allowPrint = Autoriser l’impression
settings-mail-feature-allowPrint-help = Si désactivé, l’action Imprimer n’est pas proposée.
settings-mail-feature-showSpam = Afficher le dossier Spam
settings-mail-feature-showSpam-help = Si désactivé, le dossier Spam et les messages marqués comme spam sont masqués dans la liste des dossiers et dans les résultats de recherche. Rien n’est supprimé.
settings-mail-feature-showAuthStatus = Afficher les résultats d’authentification de l’expéditeur
settings-mail-feature-showAuthStatus-help = Affiche dans le lecteur les résultats SPF, DKIM et DMARC d’un message.
settings-mail-feature-closeOnDelete = Fermer un message lorsqu’il est supprimé
settings-mail-feature-closeOnDelete-help = Ferme le message, l’onglet ou la fenêtre ouverts lorsque ce message est supprimé ou déplacé dans la corbeille.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Chemin des paramètres
settings-crumb-loading = Chargement du nom
settings-crumb-new = Nouveau
settings-crumb-edit = Modifier
settings-badge-color = Couleur du badge de non-lus
settings-badge-color-accent = Identique à l’accent
settings-badge-color-accent-lighter = Accent plus clair
settings-badge-color-accent-darker = Accent plus foncé
settings-badge-color-option = Badges { $color }
settings-appearance-theme-card = Thème et basculement
settings-appearance-theme-card-help = Choisissez un thème et s’il suit le système ou l’heure de la journée.
settings-appearance-colours-card = Couleurs
settings-appearance-colours-card-help = L’accent utilisé pour les actions et la sélection, et la couleur des compteurs de non-lus.
settings-appearance-text-card = Texte et mise en page
settings-appearance-text-card-help = La densité de l’interface, la police utilisée et la disposition des e-mails.
settings-sig-create-variant = Créer une variante
settings-sig-error-existing = Une signature portant ce nom existe déjà. Choisissez un autre nom.
settings-sig-save-failed = La signature n’a pas pu être enregistrée. { $reason }
settings-ident-signature-new = Nouveaux messages
settings-ident-signature-account = Utiliser la signature du compte
settings-ident-signature-replies = Réponses
settings-ident-signature-inherit = Utiliser la signature des nouveaux messages
settings-ident-signature-unavailable = Signature indisponible : { $name }
settings-ident-signature-help = Créez des variantes de signature dans Signatures, puis attribuez-en une ici. La même variante peut servir pour les deux.
settings-ident-save-failed = L’identité n’a pas pu être enregistrée. { $reason }
settings-search-save-failed = La recherche enregistrée n’a pas pu être enregistrée. { $reason }
settings-kbd-preset-title = Préréglage de raccourcis
settings-editor-discard-title = Abandonner les modifications non enregistrées ?
settings-editor-discard-detail = Vos modifications n’ont pas été enregistrées. Continuez à modifier ou abandonnez ce brouillon pour quitter l’éditeur.
settings-editor-keep = Continuer la modification
settings-editor-discard = Abandonner les modifications
settings-editor-missing = Cet élément n’existe plus. Revenez à la liste et choisissez un autre élément.
settings-editor-back = Retour à la liste
settings-sig-edit-title = Modifier la signature
settings-ident-edit-title = Modifier l’identité
settings-sections = Sections des paramètres
settings-category-account = Compte
settings-category-mail = Messagerie
settings-category-interface = Interface
settings-category-integrations = Intégrations
settings-category-about = À propos
settings-nav-preferences = Clavier et affichage
settings-nav-rules = Règles de messagerie
settings-nav-access = Accès à l’API
settings-nav-metadata = Informations du serveur
settings-folders-title = Dossiers et barre latérale
settings-folders-help = Épinglez des dossiers au-dessus des autres, choisissez une icône, puis faites glisser ou utilisez les boutons de déplacement pour organiser les dossiers dans chaque groupe. Ces choix sont synchronisés avec ce compte et ne déplacent aucun message ni ne renomment aucun dossier.
settings-folders-pin = Épingler { $name }
settings-folders-icon = Icône de { $name }
settings-folders-automatic = Rôle de dossier automatique
settings-folders-drag = Faire glisser { $name } pour réorganiser
settings-folders-up = Monter { $name }
settings-folders-down = Descendre { $name }
settings-folders-sync-idle = Connectez-vous pour synchroniser les préférences de dossiers.
settings-folders-sync-loading = Chargement des préférences du compte…
settings-folders-sync-saving = Enregistrement des préférences du compte…
settings-folders-sync-synced = Préférences synchronisées avec ce compte.
settings-folders-sync-error = Les préférences n’ont pas pu être synchronisées. Vos modifications restent dans cette session et seront réessayées au retour de la connexion.
settings-folder-icon-folder = Dossier
settings-folder-icon-star = Étoile
settings-folder-icon-flag = Drapeau
settings-folder-icon-tag = Label
settings-folder-icon-clock = Horloge
settings-folder-icon-calendar = Calendrier
settings-folder-icon-file = Document
settings-folder-icon-image = Image
settings-folder-icon-lock = Cadenas
settings-folder-icon-users = Personnes
settings-folder-icon-archive = Archive
settings-folder-icon-mail = Enveloppe
settings-folder-icon-attachment = Pièce jointe
settings-folder-icon-check = Coche
settings-folder-icon-alert = Alerte
settings-search-new = Nouveau dossier de recherche
settings-search-editor = Dossier de recherche
settings-search-editor-help = Les messages doivent satisfaire toutes les conditions. Laissez une condition vide pour l’ignorer. Les dossiers de recherche affichent les messages correspondants sans les déplacer ni les copier.
settings-search-name = Nom du dossier
settings-search-name-required = Saisissez un nom pour ce dossier de recherche.
settings-search-invalid = Cette recherche enregistrée contient un filtre invalide. Elle ne peut pas être modifiée en toute sécurité.
settings-search-saved = Dossier de recherche « { $name } » enregistré.
settings-search-field-text = Rechercher dans tout le texte des messages
settings-search-field-from = L’expéditeur contient
settings-search-field-to = Le destinataire contient
settings-search-field-subject = L’objet contient
settings-search-field-body = Le corps du message contient
settings-search-field-filename = Le nom de fichier de la pièce jointe contient
settings-search-field-inMailbox = Limiter à l’identifiant de boîte aux lettres (facultatif)
settings-search-field-after = Reçu après (date et heure UTC)
settings-search-field-before = Reçu avant (date et heure UTC)
settings-search-attachment = Pièces jointes
settings-search-any = N’importe quel message
settings-search-with-attachments = Avec pièces jointes
settings-search-without-attachments = Sans pièces jointes
settings-search-unread = Messages non lus uniquement
settings-search-flagged = Messages marqués uniquement
settings-sessions-search = Rechercher un compte, un appareil ou une IP
settings-sessions-account = Compte
settings-sessions-device = Appareil et client
settings-sessions-location = IP et dernier pays connu
settings-sessions-location-unavailable = Pays indisponible
settings-sessions-activity = Activité
settings-sessions-actions = Actions
settings-sessions-revoke-device = Déconnecter { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Rédaction et réponses
settings-quote-style = Style d’en-tête des réponses et transferts
settings-quote-classic = Réponse citée classique
settings-quote-outlook = En-tête de style Outlook
settings-quote-help = Le style Outlook ajoute De, Envoyé, À, Cc et Objet au-dessus du message d’origine, en texte enrichi comme en texte brut. Les brouillons existants ne changent pas.
settings-external-warning = Avertissements pour destinataires externes
settings-external-off = Désactivé
settings-external-warn = Afficher un avertissement
settings-external-confirm = Exiger une confirmation avant l’envoi
settings-external-help = Compare À, Cc et Cci au domaine de l’identité sélectionnée et à la liste des domaines de confiance. Cela protège les envois depuis ce rédacteur ; ce n’est pas une règle de remise côté serveur.
settings-trusted-domains = Domaines de destinataires de confiance
settings-trusted-domains-help = Saisissez jusqu’à 100 domaines exacts, un par ligne. Les sous-domaines doivent être listés séparément. Le domaine de l’identité d’envoi est toujours de confiance. Les URL, adresses e-mail et caractères génériques ne sont pas acceptés.
settings-trusted-domains-invalid = Saisissez jusqu’à 100 noms de domaine valides, sans URL, adresse e-mail ni caractère générique.
settings-trusted-domains-save = Enregistrer les domaines de confiance
settings-shortcuts-enabled = Activer les raccourcis de l’application
settings-shortcuts-action = Action
settings-shortcuts-binding = Raccourci
settings-shortcuts-edit = Modifier le raccourci
settings-shortcuts-edit-action = Modifier le raccourci de { $action }
settings-shortcuts-help = Utilisez inherit pour votre préréglage sélectionné, disabled pour aucun raccourci, ou une combinaison telle que Ctrl+Shift+K. Ctrl signifie Cmd sous macOS. Les raccourcis en conflit sont inactifs. Les champs de saisie et les boîtes de dialogue conservent leurs touches habituelles ; les combinaisons réservées par le navigateur peuvent être indisponibles.
settings-shortcuts-invalid = Saisissez inherit, disabled ou une touche unique valide avec les modificateurs facultatifs Ctrl, Alt et Shift.
settings-shortcuts-conflict = Deux actions utilisent le même raccourci. Modifiez l’un d’eux ; les raccourcis en conflit ne s’exécuteront pas.

# -- Machine translation (2026-11): added to match en ---------------------
settings-shortcuts-discard = Abandonner la modification non enregistrée du raccourci ?
settings-shortcuts-save = Enregistrer le raccourci
settings-shortcuts-back = Retour aux raccourcis
settings-shortcuts-off = Votre préréglage Désactivé désactive tous les raccourcis, y compris les raccourcis personnalisés. Choisissez un autre préréglage dans Préférences pour utiliser ces raccourcis.
settings-kbd-action-replyAll = Répondre à tous
settings-kbd-action-forward = Transférer
settings-kbd-action-refresh = Actualiser les messages
settings-kbd-action-settings = Ouvrir les paramètres
settings-kbd-action-trash = Déplacer vers la corbeille
settings-kbd-action-markRead = Marquer comme lu
settings-kbd-action-markUnread = Marquer comme non lu
settings-kbd-action-spam = Marquer comme spam
settings-kbd-action-zoomIn = Agrandir le message
settings-kbd-action-zoomOut = Réduire le message
settings-kbd-action-zoomReset = Réinitialiser le zoom du message
settings-kbd-action-renameFolder = Renommer le dossier
settings-kbd-action-openInTab = Ouvrir le message dans son propre onglet
settings-notif-device-title = Autorisation sur cet appareil
settings-notif-device-help = Vos préférences de notification suivent votre compte. L’autorisation du navigateur appartient à cet appareil ; Mailwoman ne la demande que lorsque vous choisissez de l’accorder.
settings-notif-ask-permission = Proposer l’autorisation de notification sur les nouveaux appareils
settings-notif-ask-permission-help = Afficher une demande d’autorisation lorsque cet appareil n’a ni autorisé ni bloqué les notifications. Activé par défaut ; cette préférence est enregistrée immédiatement.
settings-notif-permission-default = L’autorisation du navigateur n’a pas été demandée.
settings-notif-permission-granted = Les notifications du navigateur sont autorisées sur cet appareil.
settings-notif-permission-denied = Les notifications sont bloquées. Modifiez l’autorisation de ce site dans les paramètres de votre navigateur pour les autoriser.
settings-notif-permission-unsupported = Ce navigateur ne prend pas en charge les notifications système ici.
settings-notif-permission-request = Autoriser les notifications sur cet appareil
settings-notif-permission-error = Le navigateur n’a pas pu demander l’autorisation de notification.
settings-notif-timezone = Fuseau horaire des heures calmes (IANA)
settings-notif-timezone-device = Utiliser le fuseau horaire de cet appareil
settings-permissions-background-title = Notifications en arrière-plan sur ce navigateur
settings-permissions-background-help = Enregistrez d’abord les préférences de notification et autorisez les notifications du navigateur. Enregistrez ce navigateur pour recevoir l’activité du compte lorsque l’application est fermée.
settings-permissions-push-unsupported = Le Web Push en arrière-plan n’est pas disponible dans ce navigateur.
settings-permissions-push-registered = Ce navigateur dispose d’un abonnement push. Enregistrez-le pour le lier au compte actuel.
settings-permissions-push-unregistered = Ce navigateur n’a aucun abonnement push.
settings-permissions-push-register = Enregistrer les notifications en arrière-plan
settings-permissions-push-remove = Supprimer l’abonnement du navigateur
settings-permissions-push-policy = Votre administrateur n’autorise pas les notifications push en arrière-plan.
settings-permissions-storage-title = Stockage du navigateur
settings-permissions-storage-help = Le stockage persistant protège les données locales de ce site contre l’éviction automatique lorsque le navigateur l’accorde. Il est propre à cet appareil et ne modifie pas la conservation côté serveur. Vous pouvez effacer les données du site dans les paramètres du navigateur ; les modifications hors ligne non envoyées peuvent être perdues.
settings-permissions-storage-unsupported = Les demandes de stockage persistant ne sont pas disponibles dans ce navigateur.
settings-permissions-storage-persistent = Le navigateur a accordé le stockage persistant à ce site.
settings-permissions-storage-evictable = Le navigateur peut évincer automatiquement les données locales de ce site.
settings-permissions-storage-usage = Stockage actuel du site : { $mib } Mio.
settings-permissions-storage-request = Demander le stockage persistant
settings-permissions-storage-policy = Votre administrateur n’autorise pas les demandes de stockage persistant.
settings-permissions-error = L’action du navigateur n’a pas pu être terminée. Vérifiez les paramètres enregistrés du compte et les autorisations du navigateur pour ce site.
settings-notif-background-rules = Les règles de contenu sont évaluées pendant que Mailwoman est ouvert. Les notifications système en arrière-plan sont supprimées tant que des règles de contenu sont configurées, car un réveil opaque en arrière-plan ne contient pas les détails du message.
settings-nav-regional = Paramètres régionaux
settings-time-title = Fuseaux horaires
settings-time-help = Choisissez comment s’affichent les heures des messages et un fuseau de comparaison facultatif pour la planification. Les lieux ne sont jamais déduits des adresses e-mail.
settings-time-displayZone = Fuseau horaire d’affichage
settings-time-displayZone-help = Utilisez local pour cet appareil, ou un nom IANA tel que Europe/Lisbon.
settings-time-comparisonZone = Fuseau horaire de comparaison
settings-time-comparisonZone-help = Saisissez un nom IANA tel que America/New_York, ou laissez vide pour masquer les comparaisons.
settings-time-helpers = Afficher les aides sur les fuseaux horaires
settings-time-save = Enregistrer le fuseau horaire
settings-time-invalid = Saisissez un fuseau horaire IANA valide. N’utilisez local que pour le fuseau d’affichage, ou laissez le fuseau de comparaison vide.
settings-time-discard = Abandonner les modifications non enregistrées du fuseau horaire ?
settings-folders-name = Dossier
settings-folders-actions = Actions
settings-folders-edit = Modifier l’apparence de { $name }
settings-folders-icon-help = Choisissez l’une des cinquante icônes thématiques, ou conservez la sélection automatique selon la fonction du dossier. Cela suit votre compte et ne renomme pas la boîte aux lettres.
settings-folders-save = Enregistrer l’apparence du dossier
settings-folders-back = Retour aux dossiers
settings-folders-discard = Abandonner les modifications non enregistrées de l’apparence du dossier ?
settings-folder-labels = Labels du dossier
settings-folder-labels-help = Organisez ce dossier avec jusqu’à 32 labels de compte. Cela n’applique pas de label aux messages qu’il contient et ne les déplace pas. Gérez les noms et couleurs des labels dans Labels.
settings-folders-leave-pending = Ces modifications attendent encore leur synchronisation. Quitter cet éditeur ?
settings-folders-retry = Réessayer l’enregistrement
settings-folder-role-title = Dossiers spéciaux
settings-folder-role-help = Choisissez le dossier réel utilisé pour chaque rôle de messagerie. Automatique utilise la découverte par le serveur. Utiliser la valeur par défaut de l’administrateur supprime votre remplacement. Les destinations en conflit ou manquantes désactivent ce rôle au lieu de choisir un autre dossier.
settings-folder-role-auto = Découverte automatique
settings-folder-role-inherit = Utiliser la valeur par défaut de l’administrateur
settings-folder-role-path = Chemin configuré : { $path }
settings-folder-role-missing = Le dossier configuré est indisponible
settings-folder-role-conflict = Cette destination est manquante ou en conflit avec un autre rôle. Ce rôle est indisponible tant que l’association n’est pas corrigée.
settings-folder-role-refresh-error = Enregistré, mais la liste des dossiers n’a pas pu être actualisée. Actualisez les dossiers avant d’utiliser la nouvelle association.
settings-folder-role-unsupported = Ces remplacements nécessitent des dossiers de messagerie natifs. Les comptes JMAP en amont conservent les rôles de dossier de leur serveur.
settings-folder-role-inbox = Boîte de réception
settings-folder-role-drafts = Brouillons
settings-folder-role-sent = Messages envoyés
settings-folder-role-trash = Corbeille
settings-folder-role-junk = Indésirables et spam
settings-folder-role-archive = Archive
settings-sessions-select = Sélectionner
settings-sessions-select-device = Sélectionner la session de { $device }
settings-sessions-select-visible = Sélectionner toutes les autres sessions visibles
settings-sessions-current-protected = Cette session reste connectée et ne peut pas être sélectionnée ici. Utilisez Se déconnecter pour la terminer.
settings-sessions-selected-title = Déconnecter les sessions sélectionnées ({ $count })
settings-sessions-selected-confirm = Ne terminer que les autres sessions sélectionnées. Cette session reste connectée. Les requêtes en échec ou ignorées seront signalées sans nouvelle tentative automatique.
settings-sessions-selected-action = Déconnecter les sessions sélectionnées
settings-sessions-cancel = Annuler
settings-sessions-selected-result = Déconnectées : { $revoked }. Déjà terminées : { $unchanged }. Non confirmées : { $failed }. Non tentées : { $skipped }.
settings-sessions-refresh-failed = La liste des sessions n’a pas pu être actualisée. Rechargez-la avant de sélectionner d’autres sessions.
settings-sig-format = Format de la signature
settings-sig-format-plain = Texte brut
settings-sig-format-html = Texte enrichi (HTML)
settings-sig-html-help = Le HTML est limité à 2 Mio et nettoyé avant l’enregistrement et l’envoi. Les images distantes et le contenu actif sont supprimés.
settings-sig-html-source = Modifier la source HTML
settings-sig-editor-loading = Chargement de l’éditeur de signature…
settings-sig-rich-body = Signature enrichie
settings-sig-plain-alternative = Alternative en texte brut
settings-sig-plain-help = Les messages en texte brut utilisent ce texte. Les modifications enrichies le mettent à jour tant que vous ne l’avez pas personnalisé ; les modifications de la source HTML conservent votre alternative.
settings-sig-account-changed = Le compte connecté a changé. Rouvrez cette signature dans le bon compte.
settings-sig-image-type = { $name } n’est pas une image PNG, JPEG, GIF ou WebP.
settings-sig-image-size = { $name } ne rentre pas : le HTML d’une signature, images comprises, est limité à 2 Mio.
settings-sig-too-large = Cette signature dépasse 2 Mio de HTML. Retirez une image ou une partie de la mise en forme.
settings-sig-images-help = Les images sont stockées dans la signature et envoyées comme images intégrées de chaque message.
settings-folder-icon-inbox = Boîte de réception
settings-folder-icon-sent = Envoyés
settings-folder-icon-edit = Modifier
settings-folder-icon-delete = Supprimer
settings-folder-icon-spam = Spam
settings-folder-icon-search = Recherche
settings-folder-icon-pin = Épingle
settings-folder-icon-link = Lien
settings-folder-icon-shield = Bouclier
settings-folder-icon-unlock = Déverrouiller
settings-folder-icon-fileAudio = Audio
settings-folder-icon-fileVideo = Vidéo
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Code
settings-folder-icon-print = Imprimer
settings-folder-icon-download = Télécharger
settings-folder-icon-preferences = Préférences
settings-folder-icon-dashboard = Tableau de bord
settings-folder-icon-analytics = Analyses
settings-folder-icon-nextcloud = Fichiers cloud
settings-folder-icon-domains = Domaines
settings-folder-icon-twofactor = Authentification
settings-folder-icon-assistant = Assistant
settings-folder-icon-integrations = Intégrations
settings-folder-icon-logs = Journaux
settings-folder-icon-home = Accueil
settings-folder-icon-briefcase = Mallette
settings-folder-icon-heart = Cœur
settings-folder-icon-shopping = Achats
settings-folder-icon-education = Éducation
settings-folder-icon-travel = Voyage
settings-folder-icon-coffee = Café
settings-folder-icon-gift = Cadeau

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-icon-bank = Banque
settings-folder-icon-book = Livre
settings-nav-keyboard-display = Clavier et affichage
settings-2fa-passkey-policy-disabled = Votre administrateur a désactivé la connexion et l’enregistrement par clé d’accès pour ce compte. Les identifiants existants restent listés afin que vous puissiez les gérer ou les supprimer.
settings-mail-interactions = Sélection et déplacement des messages
settings-mail-modifier-selection = Sélectionner les messages avec les touches de modification
settings-mail-modifier-selection-help = Ctrl ou Commande bascule des messages séparés. Maj sélectionne une plage, et Maj avec les flèches l’étend. Le bouton de sélection visible reste disponible lorsque cette option est désactivée.
settings-mail-drag-folder = Faire glisser les messages vers des dossiers
settings-mail-drag-folder-help = Faites glisser un message sélectionné pour déplacer la sélection, ou un message non sélectionné pour ne déplacer que celui-ci. Seuls les dossiers de ce compte sont acceptés. Les utilisateurs de l’écran tactile et du clavier peuvent utiliser la barre d’outils de sélection.
settings-search-sections = Rechercher dans les paramètres
settings-search-no-sections = Aucune section de paramètres correspondante.
settings-time-local = Fuseau horaire local de l’appareil
settings-time-none = Aucun fuseau horaire de comparaison
settings-folder-counts = Compteurs de dossiers
settings-folder-counts-none = Masquer les compteurs
settings-folder-counts-unread = Messages non lus
settings-folder-counts-total = Total des messages
settings-folder-counts-both = Messages non lus et total
settings-folder-counts-unreadIfAny = Messages non lus, seulement s’il y en a
settings-folder-count-scope = Dossiers avec compteurs
settings-folder-count-scope-all = Tous les dossiers
settings-folder-count-scope-main = Dossiers principaux uniquement
settings-folder-count-scope-help = Les dossiers principaux sont Boîte de réception, Brouillons, Envoyés, Archive, Indésirables, Corbeille, Boîte d’envoi et les autres dossiers spéciaux. Les dossiers personnalisés et leurs sous-dossiers n’affichent aucun compteur.
settings-account-unread-badge = Afficher le nombre de non-lus à côté du nom de la boîte aux lettres
settings-folder-counts-compact = Badges plus petits hors boîte de réception
settings-focused-inbox = Boîte de réception prioritaire
settings-focused-inbox-help = Diviser la boîte de réception en onglets Prioritaire et Autres au-dessus de la liste. Les messages des newsletters et des expéditeurs automatiques vont dans Autres. Les autres dossiers et les résultats de recherche affichent tous les messages.
settings-focused-senders-title = Gérer les expéditeurs prioritaires
settings-focused-senders-help = Expéditeurs que vous avez choisi de déplacer toujours vers Prioritaire ou Autres. Retirez-en un pour laisser de nouveau le tri automatique décider.
settings-focused-senders-empty = Aucun expéditeur pour le moment. Utilisez Toujours déplacer vers Prioritaire ou Toujours déplacer vers Autres sur un message.
settings-focused-senders-to-focused = Toujours Prioritaire
settings-focused-senders-to-other = Toujours Autres
settings-focused-senders-remove = Retirer
settings-focused-senders-remove-label = Retirer { $sender }
settings-focused-senders-remove-failed = L’expéditeur n’a pas pu être retiré. Réessayez.
settings-folder-counts-reset = Hériter des compteurs de dossiers
settings-folder-counts-reset-size = Hériter de la taille des badges
settings-mail-resume-tabs = Reprendre mes onglets ouverts
settings-mail-resume-tabs-help = Restaurer les références d’onglets propres au compte après la réouverture de Mailwoman. Les corps de messages et les URL de pièces jointes ne sont pas enregistrés dans l’enregistrement de l’espace de travail.
settings-row-actions-mode = Actions rapides de la liste des messages
settings-row-actions-mode-open = Toujours ouvertes
settings-row-actions-mode-hover = Afficher au survol
settings-row-actions-mode-click = Afficher au clic
settings-row-actions-mode-off = Désactivé
settings-row-actions-mode-help = Comment apparaissent les actions rapides sur chaque ligne de message. Au survol, trois commandes et un bouton Plus s’affichent ; au clic, un seul bouton les ouvre ; sur écran tactile, le survol se comporte comme un clic. Désactivé les retire des lignes (le menu contextuel et la barre d’outils conservent toutes les actions).
settings-mail-feature-showGoToTop = Afficher un bouton Haut de page dans la liste des messages
settings-mail-feature-showGoToTop-help = Ajoute un bouton qui fait remonter la liste des messages jusqu’au premier message. Désactivé par défaut.
settings-mail-feature-showCreateFolderButton = Afficher un bouton Créer un dossier à côté de Rédiger
settings-mail-feature-showCreateFolderButton-help = Ajoute dans la barre latérale un bouton avec icône seule qui crée un dossier. Désactivé par défaut.
settings-draft-save = Brouillons du compte
settings-draft-save-help = Enregistrer dans le dossier Brouillons de votre compte pour y accéder depuis un autre appareil. L’enregistrement est discret ; seuls les échecs affichent une notification. Les comptes non pris en charge et le contenu déchiffré protégé ne sont jamais envoyés silencieusement.
settings-draft-save-mode = Enregistrer automatiquement
settings-draft-realtime = Après une pause de frappe
settings-draft-periodic = À intervalles réguliers
settings-draft-manual = Seulement quand je choisis Enregistrer
settings-draft-debounce = Pause de frappe (millisecondes)
settings-draft-interval = Intervalle d’enregistrement (secondes)
settings-permissions-push-other-registration = Ce navigateur a un abonnement qui n’est pas enregistré pour ce compte. Supprimez-le avant de vous réinscrire.
settings-show-from = Afficher De lors de la rédaction
settings-show-from-help = Commencer avec le champ De visible. Vous pouvez aussi l’afficher pour un seul message depuis le ruban. Seules les adresses d’envoi autorisées sont acceptées.
settings-sessions-table = Tableau des sessions actives
settings-folders-table = Liste des dossiers
settings-kbd-reference = Afficher les raccourcis du préréglage
settings-kbd-reference-help = Référence du préréglage uniquement. Les raccourcis personnalisés et la politique de l’administrateur peuvent modifier les raccourcis actifs.
settings-sessions-client-details = Détails du client
settings-2fa-passkey-rules = Votre organisation n’accepte que les clés d’accès qui :
settings-2fa-passkey-rule-platform = sont stockées sur cet appareil (Windows Hello, Touch ID, verrouillage d’écran Android)
settings-2fa-passkey-rule-cross-platform = se trouvent sur une clé de sécurité ou un téléphone
settings-2fa-passkey-rule-user-verification = vérifient votre code PIN, votre empreinte ou votre visage
settings-2fa-passkey-rule-resident-key = sont stockées sur l’authentificateur (détectables)
settings-2fa-passkey-gap-allowed = Ajoutée avant la politique actuelle et ne la respecte pas. Elle fonctionne toujours ; ajoutez une nouvelle clé d’accès et supprimez celle-ci.
settings-2fa-passkey-gap-refused = Ne respecte pas la politique actuelle et n’est plus acceptée pour la connexion. Ajoutez une nouvelle clé d’accès et supprimez celle-ci.
settings-2fa-passkey-browser-refused-policy = Votre navigateur n’a trouvé aucun authentificateur conforme à la politique de clés d’accès de votre organisation, ou la demande a été annulée.
settings-2fa-policy-user-verification = Votre organisation exige une clé d’accès qui vérifie votre code PIN, votre empreinte ou votre visage. Cet authentificateur ne vous a pas vérifié.
settings-2fa-policy-platform = Votre organisation n’accepte que les clés d’accès stockées sur cet appareil (comme Windows Hello, Touch ID ou le verrouillage d’écran Android). Les clés de sécurité et les téléphones ne sont pas acceptés.
settings-2fa-policy-cross-platform = Votre organisation n’accepte que les clés de sécurité ou les clés d’accès de téléphone. Les clés d’accès stockées sur cet appareil ne sont pas acceptées.
settings-2fa-policy-attachment-unknown = Votre navigateur n’a pas indiqué de quel type d’authentificateur il s’agit ; il ne peut donc pas être vérifié selon la politique de clés d’accès de votre organisation. Essayez un autre navigateur.
settings-2fa-policy-resident-key = Votre organisation exige une clé d’accès stockée sur l’authentificateur. Cet authentificateur n’en a stocké aucune.
settings-2fa-policy-legacy = Cette clé d’accès a été ajoutée avant la politique actuelle de votre organisation et ne la respecte plus. Connectez-vous avec un autre facteur ou un code de récupération, puis ajoutez une nouvelle clé d’accès.
settings-2fa-policy-last-factor-blocked = Votre organisation n’autorise pas la suppression de votre dernier second facteur. Ajoutez d’abord une autre méthode de connexion.
settings-2fa-policy-last-factor-required = Votre organisation exige l’authentification à deux facteurs. Ajoutez une autre méthode de connexion avant de supprimer celle-ci.
settings-2fa-last-factor-hint = Ajoutez d’abord une autre méthode de connexion.
settings-2fa-last-factor-tooltip = Ajoutez d’abord une autre méthode de connexion. Votre organisation exige au moins une clé d’accès ou une application d’authentification sur votre compte.
settings-2fa-challenge-passkey-not-used = La clé d’accès n’a pas été utilisée : la demande a été annulée ou a expiré, ou cet authentificateur ne peut pas satisfaire à ce que votre organisation exige (comme une vérification par PIN, empreinte ou visage). Réessayez ou utilisez une autre méthode.
settings-folders-hidden-help = Les dossiers masqués et leurs sous-dossiers restent hors de la barre latérale. Ils ne sont ni supprimés ni soumis à une restriction d’accès. Afficher un sous-dossier ici affiche aussi ses parents masqués. La boîte de réception reste toujours disponible.
settings-folders-collapse-user = Réduire les dossiers utilisateur dans la barre latérale
settings-folders-show = Afficher { $name } dans la barre latérale
settings-folders-hide = Masquer { $name } de la barre latérale
settings-folders-show-short = Afficher
settings-folders-hide-short = Masquer
settings-folders-search = Rechercher des dossiers
settings-folders-select = Sélectionner le dossier : { $name }
settings-folders-selected = Dossiers sélectionnés : { $count }
settings-folders-select-visible = Sélectionner les visibles
settings-folders-select-all = Sélectionner tous les dossiers
settings-folders-clear = Effacer la sélection
settings-folders-pin-selected = Épingler la sélection
settings-folders-unpin-selected = Désépingler la sélection
settings-folders-show-selected = Afficher la sélection
settings-folders-hide-selected = Masquer la sélection
settings-folders-visible = Affiché
settings-folders-hidden = Masqué
settings-folders-pinned = Épinglé
settings-folders-location = Emplacement
settings-folders-counts = { $unread } non lus / { $total } au total
settings-folders-more = Plus d’actions pour { $name }
settings-folders-empty = Aucun dossier ne correspond à votre recherche.
settings-folders-root = Niveau supérieur
settings-folders-order-help = Faites glisser les dossiers pour les réorganiser dans la barre latérale. Les dossiers épinglés restent ensemble.
cache-title = Cache de messagerie et préchargement
cache-help = Conserver les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache s’actualise en arrière-plan. La fermeture de l’onglet ou la déconnexion le vide. Le préchargement ne marque jamais les messages comme lus et ne charge pas d’images distantes.
cache-enabled = Activer le cache de session
cache-mailboxes = Mettre en cache les listes de messages des dossiers
cache-messages = Mettre en cache les messages ouverts
cache-freshSeconds = Intervalle de fraîcheur (secondes)
cache-maxAgeSeconds = Âge maximal du cache (secondes)
cache-maxEntries = Nombre maximal d’entrées en cache
cache-maxBytes = Budget mémoire (octets)
cache-prefetchEnabled = Activer le préchargement spéculatif
cache-hoverEnabled = Précharger au pointage d’un dossier
cache-focusEnabled = Précharger lorsqu’un dossier reçoit le focus clavier
cache-dwellMs = Délai du pointeur et du focus (millisecondes)
cache-idleEnabled = Précharger les dossiers en période d’inactivité
cache-idleDelayMs = Délai d’inactivité (millisecondes)
cache-maxConcurrency = Nombre maximal de préchargements simultanés
cache-maxQueued = Nombre maximal de préchargements en file d’attente
cache-preloadMessages = Autoriser les corps de messages spéculatifs
cache-excludedFolders = Dossiers exclus du préchargement
cache-exclusions-help = Une règle par ligne : role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive, ou id: suivi de l’ID exact d’un dossier.
cache-refreshOnOpen = Actualiser le contenu en cache à l’ouverture
cache-clear = Vider maintenant le cache de messagerie
settings-compose-window = Ouvrir les nouveaux messages dans
settings-compose-window-tab = Un onglet de l’espace de travail
settings-compose-window-new = Une nouvelle fenêtre
settings-compose-window-help = Si votre navigateur bloque une nouvelle fenêtre, le rédacteur s’ouvre dans un onglet de l’espace de travail.

# -- Machine translation (2026-11): added to match en ---------------------
settings-compose-read-receipt = Toujours demander un accusé de lecture
settings-compose-delivery-receipt = Toujours demander un accusé de réception
settings-compose-reply-copy = Enregistrer une copie des réponses dans le dossier d’origine
settings-compose-receipts-help = Les accusés sont des demandes. Le destinataire ou son serveur de messagerie peut les refuser ou les ignorer.
settings-compose-forward = Format de transfert par défaut
settings-compose-forward-inline = Texte du message dans le corps
settings-compose-forward-attachment = Message d’origine en pièce jointe .eml
settings-compose-saving = Enregistrement de la préférence de rédaction…
settings-compose-saved = Préférence de rédaction enregistrée
settings-ident-organization = Organisation
settings-ident-auto-bcc = Cci automatique
settings-ident-auto-bcc-help = Ajoutez jusqu’à 20 adresses en Cci lors de l’envoi depuis cette identité. Séparez les adresses par des virgules.
settings-ident-pgp-key = Clé OpenPGP
settings-ident-pgp-none = Utiliser la clé par défaut
settings-ident-pgp-unavailable = Clé enregistrée indisponible : choisissez une autre clé
settings-ident-pgp-help = Choisissez l’une de vos clés de signature OpenPGP existantes. Le matériel de clé privée reste dans votre coffre de clés.
settings-ident-error-optional = Vérifiez l’adresse de réponse, les adresses Cci automatiques, l’organisation et la clé sélectionnée.
settings-compose-open-tracking = Enregistrer par défaut l’ouverture des messages
settings-compose-open-tracking-help = Ajoute une image de suivi aux messages mis en forme éligibles. Les ouvertures dépendent du chargement des images par le destinataire et ne prouvent pas que le message a été lu. Votre administrateur peut désactiver le suivi ou verrouiller ce réglage par défaut.
settings-compose-open-tracking-inherit = Utiliser le réglage de suivi hérité
settings-reply-images = Images dans les réponses et les transferts
settings-reply-images-ask = Demander à chaque fois
settings-reply-images-allow = Toujours inclure
settings-reply-images-trusted = Inclure pour les domaines d’expéditeur de confiance, sinon demander
settings-reply-images-deny = Ne jamais inclure
settings-reply-images-help = Choisissez si une réponse ou un transfert conserve les images du message d’origine. Les images incluses sont copiées dans le brouillon comme pièces jointes.
settings-reply-image-domains = Domaines d’expéditeur de confiance pour les images
settings-reply-image-domains-help = Saisissez jusqu’à 200 domaines d’expéditeur exacts, un par ligne. Utilisé lorsque le choix ci-dessus porte sur les domaines de confiance.
settings-reply-image-domains-save = Enregistrer les domaines d’images de confiance
settings-ident-group-basics = Informations de base
settings-ident-group-replies = Gestion des réponses
settings-ident-group-signature = Signature
settings-ident-group-sending = Options d’envoi
settings-ident-name-required = Saisissez un nom affiché.
settings-ident-email-invalid = Saisissez une adresse e-mail valide, par exemple nom@example.org.
settings-ident-email-duplicate = Vous avez déjà une identité avec cette adresse.
settings-ident-replyto-invalid = Saisissez une adresse e-mail valide ou laissez ce champ vide.
settings-ident-replyto-help = Les réponses aux messages envoyés depuis cette identité sont adressées à cette adresse plutôt qu’à celle de l’expéditeur.
settings-ident-bcc-invalid = Vérifiez la liste Cci : utilisez des adresses valides séparées par des virgules, 20 au maximum.
settings-ident-delete-title = Supprimer l’identité { $email } ?
settings-ident-delete-body = Vous ne pourrez plus envoyer de messages depuis cette identité. Les brouillons et les messages envoyés ne sont pas affectés.
settings-ident-empty = Vous n’avez encore aucune identité. Ajoutez-en une pour choisir son nom, son adresse et sa signature lors de la rédaction.
settings-sig-delete-title = Supprimer la signature { $name } ?
settings-sig-delete-body = Les identités qui l’utilisent reviennent à la signature du compte. Les messages déjà envoyés ne sont pas affectés.
settings-sig-deleted = Signature supprimée.
settings-reply-from = Répondre depuis
settings-reply-from-match = L’adresse à laquelle le message a été envoyé
settings-reply-from-default = Toujours mon identité par défaut
settings-reply-from-ask = Me demander lorsque plusieurs identités correspondent
settings-reply-from-keep = Conserver l’identité déjà sélectionnée
settings-reply-from-help = Choisissez l’identité utilisée lorsque vous répondez ou transférez. Si aucune identité ne correspond à l’adresse à laquelle le message a été envoyé, votre identité par défaut est utilisée. Les adresses avec plus, comme nom+tag@example.org, correspondent à nom@example.org.
settings-ident-default-badge = Par défaut
settings-ident-make-default = Définir par défaut
settings-ident-default-set = { $email } est maintenant votre identité par défaut.
settings-ident-create-variant = Créer une variante
settings-ident-variant-count = { $count } { $count ->
        [one] variante
       *[other] variantes
    }
settings-ident-inherits = Utilise { $base } pour : { $fields }
settings-ident-field-replyTo = adresse de réponse
settings-ident-field-autoBcc = Cci automatique
settings-ident-field-organization = organisation
settings-ident-field-pgpKeyId = clé OpenPGP
settings-ident-field-signatureName = signature
settings-ident-delete-default-body = Il s’agit de votre identité par défaut. Votre plus ancienne identité restante devient l’identité par défaut. Vous ne pourrez plus envoyer de messages depuis cette identité. Les brouillons et les messages envoyés ne sont pas affectés.
settings-ident-variants-choice-title = Supprimer { $email } ?
settings-ident-variants-choice-body = Cette identité a { $count } { $count ->
        [one] variante
       *[other] variantes
    }. Supprimez-{ $count ->
        [one] la
       *[other] les
    } également, ou conservez-{ $count ->
        [one] la
       *[other] les
    } comme identités distinctes. Une variante conservée copie les réglages hérités de cette identité.
settings-ident-delete-with-variants = Supprimer l’identité et les variantes
settings-ident-delete-keep-variants = Supprimer l’identité, conserver les variantes
settings-ident-new-variant = Nouvelle variante
settings-ident-edit-variant-title = Modifier la variante
settings-ident-variant-intro = Une variante de { $base }. Laissez un champ vide pour utiliser la valeur de cette identité.
settings-ident-label-label = Libellé de la variante
settings-ident-label-required = Saisissez un libellé pour cette variante, par exemple Newsletters.
settings-ident-inherit-hint = Vide : utilise { $value } de l’identité de base.
settings-ident-inherit-none = rien
settings-check-policy-allow = Envoyer sans demander
settings-check-policy-warn = Demander à chaque fois
settings-check-policy-warn-remember = Demander, avec possibilité de ne plus demander
settings-check-policy-deny = Bloquer l’envoi
settings-check-invalid = Adresses de destinataires non valides
settings-check-attachment = Pièce jointe mentionnée mais absente
settings-check-subject = Objet vide
settings-check-help = Ce qui se passe lorsque la vérification détecte un problème. Avec « Demander », l’option Envoyer quand même est toujours proposée ; seul Bloquer l’envoi empêche l’envoi d’un message.
settings-external-warn-remember = Demander, avec possibilité de ne plus demander
settings-external-deny = Bloquer l’envoi
settings-auto-chip = Finaliser un destinataire lorsque je tape un espace, une virgule ou un point-virgule
settings-auto-chip-help = Une adresse valide devient un destinataire finalisé et le séparateur est ajouté pour vous.
settings-recipient-separator = Séparateur de destinataires
settings-recipient-separator-comma = Virgule
settings-recipient-separator-semicolon = Point-virgule
settings-header-collapse = En-tête du message (À, Cc, Objet)
settings-header-collapse-expanded = Toujours afficher
settings-header-collapse-collapsed = Réduire en une ligne de résumé
settings-header-collapse-auto = Réduire pendant que je rédige le corps
settings-header-collapse-help = Réduit, l’en-tête affiche une ligne comme À : Ana +2 et s’ouvre lorsque vous cliquez dessus ou qu’un champ demande votre attention.
settings-retention-title = Conservation
settings-folder-icon-camera = Appareil photo
settings-folder-icon-music = Musique
settings-folder-icon-film = Film
settings-folder-icon-lightbulb = Idée
settings-folder-icon-rocket = Fusée
settings-folder-icon-trophy = Trophée
settings-folder-icon-leaf = Feuille
settings-folder-icon-globe = Globe
settings-folder-icon-cloud = Nuage
settings-folder-icon-bookmark = Signet
settings-folder-icon-wallet = Portefeuille
settings-folder-icon-medical = Médical
settings-folder-icon-bell = Cloche
settings-folder-icon-key = Clé
settings-folder-icon-receipt = Reçu
settings-folder-icon-cart = Panier
settings-folder-icon-package = Colis
settings-folder-icon-wrench = Outils
settings-folder-icon-chart = Graphique
settings-folder-icon-team = Équipe
settings-folder-icon-store = Boutique
settings-folder-icon-office = Bureau
settings-folder-icon-laptop = Ordinateur portable
settings-folder-icon-crown = Couronne
settings-folder-icon-family = Famille
settings-folder-icon-group-general = Général
settings-folder-icon-group-mail = Messagerie
settings-folder-icon-group-work = Travail
settings-folder-icon-group-finance = Finances et achats
settings-folder-icon-group-travel = Voyages et maison
settings-folder-icon-group-personal = Personnel et loisirs
settings-folder-icon-group-tech = Technologie et outils
settings-folder-icon-search-label = Rechercher des icônes
settings-folder-icon-no-match = Aucune icône correspondante

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Authentification (DMARC)

settings-badge-auth-help = Le résultat DMARC ; il ouvre tous les détails de sécurité.

settings-badge-density = Taille du badge

settings-badge-density-comfortable = Confortable

settings-badge-density-compact = Compacte

settings-badge-density-help = Hauteur et espacement compacts ou confortables pour chaque badge.

settings-badge-folder = Dossier

settings-badge-folder-help = Où le message est stocké. Seulement lorsque cela est notable, il l'affiche pour les déchets et les déchets.

settings-badge-labels = Étiquettes de badge

settings-badge-labels-help = Icône avec texte, ou icône seule (le texte intégral reste dans l'info-bulle et pour les lecteurs d'écran).

settings-badge-labels-icon = Icône uniquement

settings-badge-labels-text = Icône et texte

settings-badge-list = Liste de diffusion

settings-badge-list-help = La liste de diffusion d'où provient le message ; il ouvre les détails de l'abonnement et les actions. Uniquement lorsqu'il est notable, il l'affiche lorsque le courrier continue d'arriver après votre désabonnement.

settings-badge-mode-always = Toujours

settings-badge-mode-never = Jamais

settings-badge-mode-notable = Seulement quand c'est notable

settings-badge-sanitized = Désinfecté HTML

settings-badge-sanitized-help = Ce que le désinfectant HTML a fait au message. Seulement lorsque notable le montre quand quelque chose a été bloqué.

settings-badge-spam = Taux de spam

settings-badge-spam-help = Le score que le serveur de réception a écrit dans les en-têtes du message.

settings-folders-hide-empty = Masquer les dossiers vides dans la barre latérale

settings-folders-hide-empty-help = Les dossiers réapparaissent lorsqu'ils contiennent des messages. La boîte de réception, les dossiers de recherche, les dossiers dont le nombre est inconnu et les parents des dossiers visibles restent affichés. Remplacez les dossiers individuels sous Propriétés, Affichage ; chaque dossier reste dans cette liste.

settings-folders-list-card = Liste et ordre des dossiers

settings-header-auto-collapse = Pliez l'en-tête après l'écriture (secondes)

settings-header-auto-collapse-help = Lorsque le corps a eu le focus pendant autant de secondes et que l'en-tête a un destinataire et un sujet, il se replie sur la ligne récapitulative. 0 désactive cela.

settings-message-badges = Insignes à messages

settings-message-badges-help = Choisissez les badges qui apparaissent dans l'en-tête d'un message ouvert. Les marques de priorité sont définies sous Dossiers et barre latérale. Vous pouvez également cliquer avec le bouton droit sur un badge pour le masquer.

settings-reply-from-headers = En-têtes qui choisissent l'identité

settings-reply-from-headers-help = Un nom d'en-tête par ligne, essayé dans cet ordre : Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc. Un en-tête laissé de côté n'est pas lu.

settings-reply-from-headers-invalid = Utilisez uniquement ces noms, un par ligne : Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc.

settings-reply-from-headers-save = Enregistrer l'ordre d'en-tête

settings-sessions-linked-created = Lien créé

settings-sessions-linked-details = Détails du lien

settings-sessions-linked-kind = Type de lien

settings-sessions-linked-kind-linked = Boîte aux lettres liée

settings-sessions-linked-last-used = Dernière utilisation

settings-sessions-linked-line = Accès à la boîte aux lettres liée · à partir de { $origin }

settings-sessions-linked-line-hidden = Accès à la boîte aux lettres liée

settings-sessions-linked-origin = Vient de

settings-sessions-linked-removed = Inconnu (compte supprimé)

settings-sessions-linked-revoke = Révoquer l'accès au lien

settings-sessions-linked-revoke-label = Révoquer l'accès au lien pour { $mailbox }

settings-sessions-linked-target = Boîte aux lettres ouverte

settings-sessions-origin-full = Adresse complète

settings-sessions-origin-help = Contrôle la manière dont cette table nomme le compte d'où provient une session de boîte aux lettres liée. Vous seul le voyez, dans vos propres séances.

settings-sessions-origin-hidden = Caché

settings-sessions-origin-masked-local-part = Adresse avec la première partie masquée

settings-sessions-origin-setting = Afficher d'où vient l'accès à la boîte aux lettres liée

settings-shortcuts-search = Raccourcis de recherche

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = Gardez les dossiers et les messages récemment visités prêts à être ouverts. Par défaut, les copies restent dans la mémoire de cet onglet. Activez le stockage indéfini pour conserver des copies chiffrées lors des rechargements. Le préchargement ne marque jamais le courrier lu ni ne charge les images distantes.

cache-keepIndefinitely = Conserver indéfiniment le courrier en cache

cache-keepIndefinitely-help = Désactivé par défaut. Conservez des copies cryptées dans ce navigateur lors des recharges sans expiration de l'âge. Les messages en ligne sont toujours actualisés. Les limites d'entrée et d'octets s'appliquent toujours. La désactivation de cette option ou la suppression du courrier en cache supprime les copies enregistrées. La suppression des données du navigateur peut également les supprimer. Les pièces jointes ne sont pas téléchargées pour une utilisation hors ligne.
