# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Autoriser l’envoi
admin-settings-help-mail-allow-send = Permettre aux utilisateurs d’envoyer des messages. Si désactivé, l’éditeur désactive Envoyer et son raccourci et le serveur refuse les envois ; les brouillons peuvent toujours être enregistrés.
admin-settings-key-mail-allow-reply = Autoriser la réponse
admin-settings-help-mail-allow-reply = Permettre aux utilisateurs de répondre et de répondre à tous depuis le lecteur et avec des raccourcis. Si désactivé, ces actions ne sont pas proposées.
admin-settings-key-mail-allow-forward = Autoriser le transfert
admin-settings-help-mail-allow-forward = Permettre aux utilisateurs de transférer des messages dans le corps ou en pièce jointe. Si désactivé, ces actions ne sont pas proposées.
admin-settings-key-mail-allow-download = Autoriser les téléchargements
admin-settings-help-mail-allow-download = Permettre aux utilisateurs de télécharger des messages et des pièces jointes. Si désactivé, l’application web ne propose aucun téléchargement et le serveur refuse l’export des messages.
admin-settings-key-mail-allow-print = Autoriser l’impression
admin-settings-help-mail-allow-print = Permettre aux utilisateurs d’imprimer un message depuis le lecteur. Si désactivé, l’action Imprimer n’est pas proposée.
admin-settings-key-mail-show-spam = Afficher le dossier Spam
admin-settings-help-mail-show-spam = Afficher le dossier Spam ou Junk et les messages marqués comme spam. Si désactivé, ils sont masqués dans la liste des dossiers et dans les résultats de recherche ; rien n’est supprimé.
admin-settings-key-mail-show-auth-status = Afficher les résultats d’authentification de l’expéditeur
admin-settings-help-mail-show-auth-status = Afficher dans le lecteur les résultats SPF, DKIM et DMARC de l’en-tête Authentication-Results.
admin-settings-key-mail-close-on-delete = Fermer un message lorsqu’il est supprimé
admin-settings-help-mail-close-on-delete = Fermer le message, l’onglet ou la fenêtre détachée ouverts lorsque ce message est supprimé ou déplacé dans la corbeille. Si désactivé, le lecteur conserve son comportement précédent.
admin-settings-key-links-trusted-senders = Expéditeurs de liens de confiance
admin-settings-help-links-trusted-senders = Boîtes aux lettres d’expéditeurs exactes dont les liens de messages examinés peuvent s’ouvrir sans consentement par message. Une adresse par entrée.
admin-settings-key-appearance-font-size = Taille de police de l’interface
admin-settings-help-appearance-font-size = Taille de police de l’interface en pixels, de 12 à 24.
admin-settings-key-appearance-font-weight = Graisse de police de l’interface
admin-settings-help-appearance-font-weight = Graisse de base du texte de l’interface : 300, 400, 500 ou 600. Le texte mis en valeur conserve sa hiérarchie.
admin-settings-key-appearance-font-color = Couleur du texte de l’interface
admin-settings-help-appearance-font-color = Couleur du texte de l’interface au format #RRGGBB. Laisser vide pour suivre le thème.
admin-settings-key-appearance-low-contrast = Atténuer le contraste du texte
admin-settings-help-appearance-low-contrast = Atténue le contraste du texte de l’interface.
admin-settings-key-mail-message-list-order = Ordre de la liste des messages
admin-settings-help-mail-message-list-order = Ordre de la liste des messages : plus récents, plus anciens, expéditeur ou objet.
admin-settings-key-mail-message-list-style = Style de la liste des messages
admin-settings-help-mail-message-list-style = Afficher la liste des messages sous forme de cartes ou de lignes avec en-têtes de colonnes.
admin-settings-key-mail-message-list-preview = Aperçus de la liste des messages
admin-settings-help-mail-message-list-preview = Afficher des extraits du corps dans la liste des messages.
admin-settings-key-mail-after-delete = Après la suppression d’un message
admin-settings-help-mail-after-delete = Message que le lecteur affiche après la suppression du message ouvert : suivant, précédent ou aucun.
admin-settings-key-reader-attachments-at-end = Pièces jointes après le corps
admin-settings-help-reader-attachments-at-end = Afficher les pièces jointes après le corps du message plutôt qu’avant.
admin-settings-key-notifications-ask-permission = Proposer l’autorisation de notifications
admin-settings-help-notifications-ask-permission = Proposer l’invite d’autorisation de notifications du navigateur aux utilisateurs dont l’appareil n’a ni autorisé ni bloqué les notifications.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Paramètres
admin-settings-description = Valeurs pour l'ensemble du serveur, par domaine et par utilisateur. Une valeur verrouillée ne peut pas être modifiée par les utilisateurs ni par des portées plus étroites.
admin-settings-list-caption = Paramètres
admin-settings-search = Rechercher des paramètres
admin-settings-empty = Aucun paramètre ne correspond à la recherche.
admin-settings-unavailable = Ce serveur ne propose pas encore le modèle de paramètres.
admin-settings-col-setting = Paramètre
admin-settings-col-value = Valeur
admin-settings-col-source = Source
admin-settings-col-overrides = Valeurs plus précises
admin-settings-open = Ouvrir
admin-settings-overrides-none = Aucune
admin-settings-overrides = { $domains } domaines, { $users } utilisateurs
admin-settings-not-found = Il n'existe aucun paramètre nommé { $key }.
admin-settings-value-on = Activé
admin-settings-value-off = Désactivé
admin-settings-value-empty = Non défini
admin-settings-value-list = { $count ->
    [one] { $count } élément
   *[other] { $count } éléments
}
admin-settings-detail-key = Clé : { $key }
admin-settings-global = Pour l'ensemble du serveur
admin-settings-global-help = Cette valeur s'applique à tous les utilisateurs, sauf si un domaine ou un utilisateur a une valeur plus précise, ou si l'utilisateur choisit la sienne là où c'est autorisé.
admin-settings-value-label = Valeur
admin-settings-json-hint = JSON. Le serveur vérifie la valeur et indique la partie qu'il refuse.
admin-settings-json-invalid = Ce JSON n'est pas valide.
admin-settings-user-override = Les utilisateurs peuvent la modifier
admin-settings-user-override-help = Lorsque c'est activé, les utilisateurs peuvent choisir leur propre valeur dans leurs paramètres. Lorsque c'est désactivé, la valeur définie ici ou pour leur domaine s'applique.
admin-settings-lock-help = Lorsqu'elle est verrouillée, les valeurs de domaine et les choix des utilisateurs sont ignorés et cette valeur s'applique à tous.
admin-settings-save = Enregistrer
admin-settings-saved = Enregistré. La valeur s'applique sous 30 secondes sur chaque processus du serveur.
admin-settings-saved-restart = Enregistré. La valeur s'applique après un redémarrage.
admin-settings-dedicated = Ce paramètre se modifie dans { $section }, qui en vérifie d'abord l'effet.
admin-settings-dedicated-link = Ouvrir { $section }
admin-settings-pinned = Défini par la variable d'environnement { $name }. La valeur enregistrée ici s'applique lorsque la variable n'est pas définie.
admin-settings-lock-title = Verrouiller { $setting } ?
admin-settings-lock-body = Le verrouillage ignore les valeurs plus précises listées ci-dessous. Elles sont conservées et s'appliquent de nouveau au déverrouillage.
admin-settings-lock-confirm = Verrouiller
admin-settings-impact-domain = { $count ->
    [one] { $count } valeur de domaine est ignorée
   *[other] { $count } valeurs de domaine sont ignorées
}
admin-settings-impact-user = { $count ->
    [one] { $count } choix propre d'un utilisateur est ignoré
   *[other] { $count } choix propres d'utilisateurs sont ignorés
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } valeur par utilisateur est ignorée
   *[other] { $count } valeurs par utilisateur sont ignorées
}
admin-settings-impact-profile = { $count ->
    [one] { $count } valeur de profil de fonctionnalités est ignorée
   *[other] { $count } valeurs de profils de fonctionnalités sont ignorées
}
admin-settings-impact-group = { $count ->
    [one] { $count } valeur de groupe est ignorée
   *[other] { $count } valeurs de groupe sont ignorées
}
admin-settings-domains = Valeurs de domaine
admin-settings-domains-help = Une valeur de domaine s'applique aux utilisateurs de ce domaine à la place de la valeur de l'ensemble du serveur.
admin-settings-domains-caption = Valeurs de domaine de { $setting }
admin-settings-domains-empty = Aucun domaine n'a de valeur propre.
admin-settings-domain = Domaine
admin-settings-domain-value = Valeur pour ce domaine
admin-settings-domain-locked = Verrouillée pour les utilisateurs du domaine
admin-settings-domain-add = Ajouter une valeur de domaine
admin-settings-domain-save = Enregistrer la valeur de domaine
admin-settings-domain-remove = Supprimer
admin-settings-domain-remove-title = Supprimer la valeur de { $domain } ?
admin-settings-domain-remove-body = Les utilisateurs de { $domain } reprennent la valeur de l'ensemble du serveur.
admin-settings-domain-required = Saisissez un nom de domaine.
admin-settings-locked-yes = Verrouillée
admin-settings-locked-no = Non verrouillée
admin-settings-users = Valeurs par utilisateur
admin-settings-users-help = Valeur définie par un administrateur pour un utilisateur. Pour les limites, l'utilisateur est désigné par son adresse.
admin-settings-users-caption = Valeurs par utilisateur de { $setting }
admin-settings-users-empty = Aucun utilisateur n'a de valeur définie par un administrateur.
admin-settings-user = Utilisateur
admin-settings-user-hint = Identifiant du compte ou, pour les limites, adresse.
admin-settings-user-value = Valeur pour cet utilisateur
admin-settings-user-add = Ajouter une valeur par utilisateur
admin-settings-user-save = Enregistrer la valeur de l'utilisateur
admin-settings-user-required = Saisissez un utilisateur.
admin-settings-user-remove-title = Supprimer la valeur de { $user } ?
admin-settings-user-remove-body = L'utilisateur reprend la valeur de son domaine, ou celle de l'ensemble du serveur.
admin-settings-source-default = Valeur par défaut intégrée
admin-settings-source-envFallback = Variable d'environnement (aucune valeur enregistrée)
admin-settings-source-envPin = Variable d'environnement
admin-settings-source-global = Enregistrée pour l'ensemble du serveur
admin-settings-cat-interface = Interface
admin-settings-cat-appearance = Apparence
admin-settings-cat-mail = Messagerie
admin-settings-cat-folders = Disposition des dossiers
admin-settings-cat-notifications = Notifications
admin-settings-cat-offline = Utilisation hors ligne
admin-settings-cat-assistant = Assistant IA
admin-settings-cat-upstream = Serveur de messagerie
admin-settings-cat-signin = Connexion
admin-settings-cat-admin = Console d'administration
admin-settings-cat-passwords = Mots de passe
admin-settings-cat-limits = Limites et utilisation
admin-settings-cat-plugins = Modules
admin-settings-cat-features = Autorisations de fonctionnalités
admin-settings-cat-integrations = Autorisations d'intégrations
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Autorise les fichiers Nextcloud liés au compte, le partage et les opérations sur les calendriers, contacts et tâches.
admin-settings-key-cap-integration-google = Passerelle Google
admin-settings-help-cap-integration-google = Autorise les opérations de la passerelle Google, y compris l'obtention et l'actualisation des jetons.
admin-settings-key-cap-integration-microsoft = Passerelle Microsoft
admin-settings-help-cap-integration-microsoft = Autorise les opérations de la passerelle Microsoft, y compris l'obtention et l'actualisation des jetons.
admin-settings-key-cap-integration-family = Intégration individuelle
admin-settings-help-cap-integration-family = Autorisation pour l'identifiant d'une intégration individuelle ; la couverture à l'exécution figure dans le catalogue des fonctionnalités.
admin-settings-key-cap-ui-plugin-family = Module d'interface individuel
admin-settings-help-cap-ui-plugin-family = Autorisation pour un module d'interface. Utilisez la clé de politique exacte du catalogue des intégrations.
admin-settings-key-cap-ai-assistant = Assistant IA
admin-settings-help-cap-ai-assistant = Autorise la configuration et l'appel de l'assistant.
admin-settings-key-cap-plugins = Modules
admin-settings-help-cap-plugins = Autorise les opérations des modules d'interface et de moteur.
admin-settings-key-cap-pim-calendar = Calendrier
admin-settings-help-cap-pim-calendar = Autorise les méthodes de calendrier et les abonnements.
admin-settings-key-cap-pim-contacts = Contacts
admin-settings-help-cap-pim-contacts = Autorise les méthodes de contacts et de carnets d'adresses.
admin-settings-key-cap-pim-tasks = Tâches
admin-settings-help-cap-pim-tasks = Autorise les méthodes de tâches.
admin-settings-key-cap-pim-notes = Notes
admin-settings-help-cap-pim-notes = Autorise les méthodes de notes.
admin-settings-key-cap-export = Export
admin-settings-help-cap-export = Autorise les exports de courrier et de données PIM.
admin-settings-key-cap-import = Import
admin-settings-help-cap-import = Autorise les imports de courrier et de données PIM.
admin-settings-key-cap-api-keys = Clés d'API
admin-settings-help-cap-api-keys = Politique des clés d'API. Consultez le catalogue des fonctionnalités pour la couverture de l'application.
admin-settings-key-cap-oauth-apps = Applications OAuth
admin-settings-help-cap-oauth-apps = Politique des applications OAuth. Consultez le catalogue des fonctionnalités pour la couverture de l'application.
admin-settings-key-cap-rules = Règles de messagerie
admin-settings-help-cap-rules = Autorise les méthodes gérées de règles de messagerie et de réponse d'absence.
admin-settings-key-cap-rules-forward = Transfert par règle
admin-settings-help-cap-rules-forward = Politique de transfert. Consultez le catalogue des fonctionnalités pour la couverture de l'application.
admin-settings-key-cap-remote-images = Images distantes
admin-settings-help-cap-remote-images = Autorisation des images distantes. Les paramètres de confidentialité existants continuent de régir les requêtes ; vérifiez la couverture de la fonctionnalité.
admin-settings-key-cap-user-background = Arrière-plan personnalisé
admin-settings-help-cap-user-background = Autorisation de l'arrière-plan personnalisé, désactivée par défaut. Vérifiez la couverture de la fonctionnalité.
admin-settings-key-cap-user-css = CSS personnalisé
admin-settings-help-cap-user-css = Autorisation du CSS personnalisé, désactivée par défaut. Vérifiez la couverture de la fonctionnalité.
admin-settings-key-limit-attachment-bytes = Limite de taille des pièces jointes
admin-settings-help-limit-attachment-bytes = Politique des pièces jointes en octets, 50000000 par défaut. L'application à l'exécution reste à faire.
admin-settings-key-limit-send-per-hour = Limite d'envoi par heure
admin-settings-help-limit-send-per-hour = Politique de soumission par heure ; zéro signifie illimité. L'application à l'exécution reste à faire.
admin-settings-key-limit-recipients-per-message = Destinataires par message
admin-settings-help-limit-recipients-per-message = Politique du nombre de destinataires ; zéro signifie illimité. L'application à l'exécution reste à faire.
admin-settings-cat-import = Import
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Mises à jour en direct
admin-settings-cat-privacy = Confidentialité
admin-settings-cat-observability = Observabilité
admin-settings-cat-egress = Connexions sortantes
admin-settings-cat-updates = Mises à jour
admin-settings-cat-backups = Sauvegardes
admin-settings-cat-about = À propos
admin-settings-cat-config = Configuration
admin-settings-cat-jobs = Tâches planifiées
admin-settings-cat-maintenance = Maintenance
admin-settings-cat-health = Contrôles d'état
admin-settings-cat-diagnostics = Diagnostics
admin-settings-cat-users = Utilisateurs
admin-settings-cat-server-text = Texte du serveur
admin-settings-key-ui-language = Langue
admin-settings-help-ui-language = La langue du client de messagerie pour les utilisateurs qui n'en ont pas choisi. « Identique au navigateur » suit les langues de chaque navigateur.
admin-settings-key-ui-notify-new-version = Avis de nouvelle version
admin-settings-help-ui-notify-new-version = Activé (par défaut) affiche aux utilisateurs connectés une notification avec « Recharger maintenant » et « Plus tard » lorsqu'une version plus récente de l'application web a été déployée depuis le chargement de leur page. Désactivé ne l'affiche jamais ; les utilisateurs obtiennent alors le nouveau code à leur prochain rechargement.
admin-settings-key-ui-undo-history-size = Taille de l'historique d'annulation et de rétablissement
admin-settings-help-ui-undo-history-size = Nombre d'actions récentes de messagerie (archivage, suppression, déplacement, marquage comme lu, drapeau, étiquette et similaires) dont les boutons Annuler et Rétablir se souviennent, de 0 à 50. 0 désactive les boutons et le raccourci Ctrl+Z. La valeur par défaut est 50. Les utilisateurs peuvent choisir leur propre taille, jamais au-dessus du maximum ci-dessous, sauf si un administrateur la verrouille.
admin-settings-key-folders-reorder-enabled = Glisser-déposer des dossiers
admin-settings-help-folders-reorder-enabled = Permet de réorganiser ses dossiers en les faisant glisser dans la barre latérale et dans les paramètres. Désactivé masque les poignées de glissement et les commandes Déplacer. Les utilisateurs peuvent le désactiver pour eux-mêmes, sauf si c'est verrouillé.
admin-settings-key-folders-reorder-allow-nesting = Imbrication des dossiers
admin-settings-help-folders-reorder-allow-nesting = Permet de déposer un dossier sur un autre pour l'imbriquer, ou de le ramener au premier niveau. Désactivé laisse chaque dossier sous son parent actuel et n'autorise que la réorganisation entre dossiers de même niveau.
admin-settings-key-folders-reorder-allow-server-move = Déplacer les dossiers sur le serveur
admin-settings-help-folders-reorder-allow-server-move = Permet à une imbrication ou à une désimbrication de modifier le véritable parent du dossier sur le serveur de messagerie, de sorte que le changement vaut partout. Désactivé conserve ces déplacements comme simple organisation dans cette application.
admin-settings-key-folders-reorder-allow-system-folders = Déplacer les dossiers système
admin-settings-help-folders-reorder-allow-system-folders = Permet de faire glisser Boîte de réception, Brouillons, Envoyés, Courrier indésirable, Corbeille et Archives pour les réorganiser. Désactivé les garde fixes en haut. Ils ne sont réorganisés que dans cette application, jamais renommés ni déplacés sur le serveur.
admin-settings-key-folders-reorder-max-depth = Profondeur d'imbrication des dossiers
admin-settings-help-folders-reorder-max-depth = Le niveau d'imbrication le plus profond qu'un glissement peut créer, de 1 à 16. Un déplacement qui placerait un dossier ou l'un de ses sous-dossiers plus profondément est refusé.
admin-settings-key-folders-order-locked = Figer l'ordre des dossiers
admin-settings-help-folders-order-locked = Fige l'ordre des dossiers pour tous les utilisateurs de la portée. Le glissement et les commandes Déplacer sont désactivés, et les utilisateurs voient une indication de verrouillage. Les épinglages et l'ordre défini par l'administrateur sont conservés.
admin-settings-key-folders-pinned-admin = Dossiers épinglés de force
admin-settings-help-folders-pinned-admin = Dossiers affichés dans un groupe Épinglés en haut de la barre latérale de chaque utilisateur. Une entrée par ligne, sous la forme role:inbox ou un chemin de dossier tel que Shared/Support. Les utilisateurs ne peuvent ni les désépingler ni les déplacer. Une entrée sans dossier correspondant est ignorée.
admin-settings-key-folders-order-default = Ordre par défaut des dossiers
admin-settings-help-folders-order-default = L'ordre dans lequel les dossiers apparaissent pour les utilisateurs qui ne les ont pas organisés, du premier au dernier. Une entrée par ligne, sous la forme role:<role> ou un chemin de dossier. Les dossiers non listés suivent dans l'ordre du serveur. L'ordre propre à un utilisateur remplace celui-ci.
admin-settings-key-folders-new-folder-placement = Emplacement des nouveaux dossiers
admin-settings-help-folders-new-folder-placement = L'endroit où un dossier qui apparaît sur le serveur est placé parmi ses voisins : à la fin, ou par ordre alphabétique.
admin-settings-key-ui-undo-history-max = Maximum de l'historique d'annulation et de rétablissement
admin-settings-help-ui-undo-history-max = Le plus grand nombre d'actions qu'un utilisateur peut conserver dans l'historique d'annulation et de rétablissement, de 0 à 50. La taille propre à un utilisateur est abaissée à cette valeur. 0 désactive l'historique pour tous. L'historique réside dans l'onglet du navigateur et est effacé à la déconnexion.
admin-settings-key-ui-bar-mail-top-bar = Barre d'outils de messagerie
admin-settings-help-ui-bar-mail-top-bar = « pinned » garde toujours affichée la barre d'outils de messagerie avec les commandes de disposition. « auto » la masque derrière une fine poignée qui l'ouvre lorsqu'un utilisateur la pointe, la touche ou y accède par tabulation.
admin-settings-key-ui-bar-workspace-tabs = Onglets de l'espace de travail
admin-settings-help-ui-bar-workspace-tabs = « pinned » garde toujours affichée la barre d'onglets de l'espace de travail. « auto » n'affiche qu'une fine poignée portant le nom de l'onglet actif, jusqu'à ce qu'un utilisateur l'ouvre.
admin-settings-key-ui-bar-mail-ribbon = Ruban de messagerie
admin-settings-help-ui-bar-mail-ribbon = Pour la disposition en ruban. « pinned » garde le ruban ouvert. « auto » n'affiche que les noms d'onglets ; un onglet ouvre le ruban au-dessus de la liste des messages.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-ui-bar-composer-ribbon = Ruban de l'éditeur
admin-settings-help-ui-bar-composer-ribbon = Pour chaque éditeur de message. « pinned » garde le ruban de l'éditeur ouvert. « auto » n'affiche que les noms d'onglets ; l'ouverture d'un onglet repousse le corps du message vers le bas au lieu de le recouvrir.
admin-settings-key-appearance-theme = Thème
admin-settings-help-appearance-theme = Le thème de couleurs que voient les utilisateurs tant qu'ils n'en choisissent pas un. Le verrouiller impose ce thème à tous les utilisateurs.
admin-settings-key-appearance-accent = Couleur d'accent
admin-settings-help-appearance-accent = La couleur d'accent du thème. Vide signifie l'accent propre au thème.
admin-settings-key-appearance-badge-color = Couleur du badge de non-lus
admin-settings-help-appearance-badge-color = La couleur des badges de compteur de non-lus. Défaut conserve la couleur du thème ; Accent suit la couleur d'accent ; accent-lighter et accent-darker utilisent l'accent 20 % plus clair ou plus foncé, avec un chiffre noir ou blanc. Un verrou l'emporte sur le choix du compte.
admin-settings-key-appearance-density = Densité
admin-settings-help-appearance-density = L'espace que prennent les listes et les commandes : compact, confortable ou aéré.
admin-settings-key-appearance-font = Police de l'interface
admin-settings-key-packages-allowed = Suivi des colis autorisé
admin-settings-help-packages-allowed = Autorise le suivi des colis pour les comptes concernés. Le fournisseur doit aussi être activé dans Suivi des colis, et chaque utilisateur doit y consentir. Désactiver ce paramètre empêche immédiatement tout nouvel appel au fournisseur.
admin-settings-help-appearance-font = Choisissez une police fournie pour les comptes sans préférence de police, ou verrouillez le choix pour tous les comptes concernés. Les polices sont chargées depuis ce serveur ; aucune requête n'est envoyée à Google Fonts. Défaut utilise la police du thème.
admin-settings-key-appearance-brand-name = Nom du produit
admin-settings-help-appearance-brand-name = Le nom affiché sur la page de connexion et dans l'onglet du navigateur.
admin-settings-key-login-notice = Avis de connexion
admin-settings-help-login-notice = Un court texte affiché sur la page de connexion, par exemple un contact d'assistance. Laissez vide pour n'en afficher aucun.
admin-settings-key-mail-read-receipts = Accusés de lecture
admin-settings-key-receipts-dsn-allowed = Demandes d'état de livraison
admin-settings-help-receipts-dsn-allowed = Autorise les comptes à demander des notifications d'état de livraison SMTP. Cela ne demande ni ne prouve qu'un message a été lu.
admin-settings-help-mail-read-receipts = Ce qui se passe lorsqu'un message ouvert demande un accusé de lecture : demander à l'utilisateur, ne jamais envoyer, ou toujours envoyer.
admin-settings-key-notifications-enabled = Notifications de nouveaux messages
admin-settings-help-notifications-enabled = Indique si les utilisateurs reçoivent des notifications pour les nouveaux messages. Cela ne modifie pas leurs règles de notification.
admin-settings-key-offline-retention-days = Ancienneté du cache hors ligne
admin-settings-help-offline-retention-days = Pendant combien de jours un appareil conserve les messages pour une utilisation hors ligne. 0 signifie aucune limite d'ancienneté.
admin-settings-key-offline-budget-mb = Taille du cache hors ligne
admin-settings-help-offline-budget-mb = Combien de mégaoctets un appareil peut utiliser pour le courrier hors ligne. 0 signifie aucun cache hors ligne.
admin-settings-key-assistant-enabled = Assistant IA
admin-settings-help-assistant-enabled = Indique si les utilisateurs voient l'assistant IA. Le commutateur de déploiement sous Assistant IA s'applique toujours au-dessus.
admin-settings-key-assist-profile = Profil de fournisseur IA
admin-settings-help-assist-profile = L'identifiant du profil de fournisseur qui sert ces utilisateurs (Assistant IA > Profils de fournisseur). Vide utilise le point de terminaison du déploiement. Un identifiant de profil inexistant désactive l'assistant IA pour ces utilisateurs.
admin-settings-key-upstream-default = Serveur de messagerie par défaut
admin-settings-help-upstream-default = Le serveur de messagerie auquel une adresse se connecte lorsqu'aucun domaine n'en désigne un. La table des domaines l'emporte.
admin-settings-key-upstream-proxy-timeout-secs = Délai d'attente du serveur de messagerie
admin-settings-help-upstream-proxy-timeout-secs = La durée maximale d'une requête vers le serveur de messagerie, en secondes.
admin-settings-key-signin-new-users = Nouveaux utilisateurs
admin-settings-help-signin-new-users = Qui peut créer un compte à la première connexion : tout le monde (ouvert), seulement les adresses ajoutées à l'avance (préprovisionné), celles-ci plus les utilisateurs importés (provisionné), ou personne de nouveau (fermé). Les comptes existants ne sont jamais affectés. Modifié dans la Politique de connexion.
admin-settings-key-signin-new-users-mail = Connexion par mot de passe : nouveaux utilisateurs
admin-settings-help-signin-new-users-mail = Règle pour un nouvel utilisateur de boîte aux lettres qui se connecte avec un mot de passe (IMAP, POP3, Exchange). Hériter utilise Nouveaux utilisateurs.
admin-settings-key-signin-new-users-sso = Authentification unique : nouveaux utilisateurs
admin-settings-help-signin-new-users-sso = Règle pour un nouvel utilisateur qui se connecte par authentification unique ou par un en-tête de confiance. Hériter utilise Nouveaux utilisateurs.
admin-settings-key-signin-new-users-linked = Boîtes aux lettres liées : nouveaux utilisateurs
admin-settings-help-signin-new-users-linked = Règle pour lier une boîte aux lettres qui n'a pas encore de compte ici, jugée d'après le domaine propre de la boîte liée. Hériter utilise Nouveaux utilisateurs.
admin-settings-key-signin-new-users-linked-imap = Boîtes aux lettres IMAP liées : nouveaux utilisateurs
admin-settings-help-signin-new-users-linked-imap = Règle pour les seules boîtes aux lettres IMAP liées. Hériter utilise Boîtes aux lettres liées.
admin-settings-key-signin-new-users-linked-pop3 = Boîtes aux lettres POP3 liées : nouveaux utilisateurs
admin-settings-help-signin-new-users-linked-pop3 = Règle pour les seules boîtes aux lettres POP3 liées. Hériter utilise Boîtes aux lettres liées.
admin-settings-key-signin-new-users-linked-ews = Boîtes aux lettres Exchange liées : nouveaux utilisateurs
admin-settings-help-signin-new-users-linked-ews = Règle pour les seules boîtes aux lettres Exchange (EWS) liées. Hériter utilise Boîtes aux lettres liées.
admin-settings-key-signin-preprovision-ttl-days = Durée de vie des utilisateurs préajoutés (jours)
admin-settings-help-signin-preprovision-ttl-days = Combien de temps un utilisateur en attente ajouté à l'avance reste valide lorsqu'aucune autre durée n'est donnée.
admin-settings-key-signin-preprovision-max-ttl-days = Durée de vie maximale des utilisateurs préajoutés (jours)
admin-settings-help-signin-preprovision-max-ttl-days = La durée de vie la plus longue qu'un administrateur peut donner à un utilisateur en attente ajouté à l'avance.
admin-settings-key-signin-preprovision-domain-wide = Autoriser les entrées préajoutées pour tout un domaine
admin-settings-help-signin-preprovision-domain-wide = Autorise les entrées qui admettent toute personne d'un domaine entier, jusqu'à une limite d'utilisation. Désactivé par défaut.
admin-settings-key-signin-preprovision-retention-days = Conserver les utilisateurs en attente expirés (jours)
admin-settings-help-signin-preprovision-retention-days = Combien de temps un utilisateur en attente expiré est conservé avant d'être supprimé.
admin-settings-key-signin-domain-mode = Domaines de connexion
admin-settings-help-signin-domain-mode = Indique si n'importe quel domaine peut se connecter ou seulement les domaines listés. Modifié dans la Politique de connexion.
admin-settings-key-signin-refusal-detail = Messages de refus
admin-settings-help-signin-refusal-detail = Indique si une connexion refusée en donne la raison (compte désactivé, adresse non autorisée) ou donne la même réponse qu'un mot de passe erroné. Modifié dans la Politique de connexion.
admin-settings-key-signin-ldap-bind-precheck = Vérification LDAP avant la connexion
admin-settings-help-signin-ldap-bind-precheck = Vérifie le mot de passe auprès de l'annuaire avant le serveur de messagerie. Modifié dans Intégrations, après un test de liaison.
admin-settings-key-twofa-pending-ttl-secs = Délai pour le second facteur
admin-settings-help-twofa-pending-ttl-secs = Le temps dont dispose un utilisateur pour saisir le second facteur après le mot de passe, en secondes.
admin-settings-key-twofa-max-attempts = Tentatives du second facteur
admin-settings-help-twofa-max-attempts = Combien de codes de second facteur erronés sont autorisés avant que la connexion ne reprenne depuis le début.
admin-settings-key-admin-session-idle-secs = Délai d'inactivité de la console
admin-settings-help-admin-session-idle-secs = La durée pendant laquelle une session de la console d'administration dure sans requête, en secondes.
admin-settings-key-admin-session-absolute-secs = Limite de session de la console
admin-settings-help-admin-session-absolute-secs = La durée maximale d'une session de la console d'administration, en secondes, active ou non.
admin-settings-key-admin-session-rotate-secs = Rotation de l'identifiant de session de la console
admin-settings-help-admin-session-rotate-secs = À quelle fréquence, en secondes, la console d'administration remplace l'identifiant de session en cours d'utilisation (0 = seulement après une confirmation renforcée). La rotation ne prolonge jamais les limites d'inactivité ou absolue.
admin-settings-key-passwd-policy-min-length = Longueur minimale du mot de passe
admin-settings-help-passwd-policy-min-length = Le plus petit nombre de caractères d'un nouveau mot de passe.
admin-settings-key-passwd-policy-require-upper = Exiger une majuscule
admin-settings-help-passwd-policy-require-upper = Un nouveau mot de passe doit contenir une majuscule.
admin-settings-key-passwd-policy-require-lower = Exiger une minuscule
admin-settings-help-passwd-policy-require-lower = Un nouveau mot de passe doit contenir une minuscule.
admin-settings-key-passwd-policy-require-digit = Exiger un chiffre
admin-settings-help-passwd-policy-require-digit = Un nouveau mot de passe doit contenir un chiffre.
admin-settings-key-passwd-policy-require-symbol = Exiger un symbole
admin-settings-help-passwd-policy-require-symbol = Un nouveau mot de passe doit contenir un caractère qui n'est ni une lettre ni un chiffre.
admin-settings-key-limit-storage-bytes = Limite de stockage
admin-settings-help-limit-storage-bytes = Le plus grand nombre d'octets qu'un utilisateur peut stocker sur ce serveur. 0 signifie aucune limite.
admin-settings-key-limit-send-per-day = Messages envoyés par jour
admin-settings-help-limit-send-per-day = Le plus grand nombre de messages qu'un utilisateur peut envoyer par jour. 0 signifie aucune limite.
admin-settings-key-limit-ai-calls-per-day = Appels à l'assistant IA par jour
admin-settings-help-limit-ai-calls-per-day = Le plus grand nombre de requêtes à l'assistant IA qu'un utilisateur peut faire par jour. 0 signifie aucune limite.
admin-settings-key-quota-warn-thresholds = Niveaux d'alerte d'utilisation
admin-settings-help-quota-warn-thresholds = Les pourcentages d'une limite auxquels les utilisateurs sont avertis, du plus bas au plus haut.
admin-settings-key-quota-warn-enabled = Alertes d'utilisation
admin-settings-help-quota-warn-enabled = Indique si les utilisateurs sont avertis lorsqu'ils approchent d'une limite.
admin-settings-key-usage-retention-days = Historique d'utilisation
admin-settings-help-usage-retention-days = Combien de jours de compteurs d'utilisation quotidiens sont conservés.
admin-settings-key-usage-meter-api = Compter l'utilisation de l'API
admin-settings-help-usage-meter-api = Indique si les requêtes d'API sont comptées dans les chiffres d'utilisation.
admin-settings-key-usage-measure-interval-hours = Intervalle de mesure du stockage
admin-settings-help-usage-measure-interval-hours = À quelle fréquence les données stockées de chaque utilisateur sont mesurées, en heures.
admin-settings-key-assistant-rate-limit-per-min = Limite de débit de l'assistant IA
admin-settings-help-assistant-rate-limit-per-min = Le plus grand nombre de requêtes à l'assistant IA par minute pour l'ensemble du serveur. 0 signifie aucune limite.
admin-settings-key-plugins-kv-max-key-bytes = Taille de clé des modules
admin-settings-help-plugins-kv-max-key-bytes = La clé la plus longue qu'un module peut stocker, en octets.
admin-settings-key-plugins-kv-max-value-bytes = Taille de valeur des modules
admin-settings-help-plugins-kv-max-value-bytes = La plus grande valeur qu'un module peut stocker sous une clé, en octets.
admin-settings-key-plugins-kv-max-total-bytes = Stockage des modules
admin-settings-help-plugins-kv-max-total-bytes = Le plus grand nombre d'octets qu'un module peut stocker au total.
admin-settings-key-plugins-kv-max-keys = Clés des modules
admin-settings-help-plugins-kv-max-keys = Le plus grand nombre de clés qu'un module peut stocker.
admin-settings-key-import-max-messages = Nombre de messages à l'import
admin-settings-help-import-max-messages = Le plus grand nombre de messages qu'un import peut apporter.
admin-settings-key-import-max-message-bytes = Taille des messages à l'import
admin-settings-help-import-max-message-bytes = Le plus gros message qu'un import accepte, en octets.
admin-settings-key-webhooks-max-attempts = Tentatives de webhook
admin-settings-help-webhooks-max-attempts = Combien de fois une livraison de webhook est tentée avant d'être abandonnée.
admin-settings-key-webhooks-timeout-secs = Délai d'attente des webhooks
admin-settings-help-webhooks-timeout-secs = La durée maximale d'une livraison de webhook, en secondes.
admin-settings-key-outbox-max-attempts = Tentatives d'envoi
admin-settings-help-outbox-max-attempts = Combien de fois un message est remis au serveur de messagerie avant d'être signalé comme non envoyé.
admin-settings-key-realtime-recheck-secs = Revérification des mises à jour en direct
admin-settings-help-realtime-recheck-secs = À quelle fréquence une connexion de mises à jour en direct ouverte vérifie que sa session est toujours valide, en secondes.
admin-settings-key-privacy-watermark-enabled = Filigrane du lecteur
admin-settings-help-privacy-watermark-enabled = Affiche un filigrane avec l'adresse du lecteur sur les messages ouverts.
admin-settings-key-privacy-watermark-opacity = Intensité du filigrane
admin-settings-help-privacy-watermark-opacity = La visibilité du filigrane, de 0,02 (discret) à 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Historique du suivi d'ouverture
admin-settings-help-privacy-open-tracking-retention-days = Combien de jours les enregistrements de suivi de lecture sont conservés.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Jetons de suivi non envoyés
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Combien d'heures un jeton de suivi attend l'envoi de son message avant d'être supprimé.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-masked-email-domain = Domaine des adresses masquées
admin-settings-help-masked-email-domain = Le domaine sous lequel les adresses masquées sont créées. Laissez vide pour désactiver les adresses masquées.
admin-settings-key-abuse-report-address = Adresse des signalements d'abus
admin-settings-help-abuse-report-address = Où sont envoyés les signalements de spam et d'abus.
admin-settings-key-dlp-admin-address = Avis de perte de données
admin-settings-help-dlp-admin-address = Où sont envoyés les avis de prévention des pertes de données.
admin-settings-key-observability-error-forward-url = Adresse de transfert des erreurs
admin-settings-help-observability-error-forward-url = Une adresse https vers laquelle les rapports d'erreurs des clients sont transférés. Laissez vide pour aucune.
admin-settings-key-egress-probe-url = Adresse de test sortante
admin-settings-help-egress-probe-url = L'adresse https utilisée pour tester un proxy sortant.
admin-settings-key-push-vapid-contact = Contact push
admin-settings-help-push-vapid-contact = Le contact (mailto: ou https:) transmis aux services push avec chaque notification.
admin-settings-key-push-quiet-hours = Heures calmes des notifications push
admin-settings-help-push-quiet-hours = Heures (UTC) pendant lesquelles aucune notification push n'est envoyée. Laissez vide pour aucune.
admin-settings-key-updates-check = Rechercher les mises à jour
admin-settings-help-updates-check = Indique si le serveur recherche une fois par jour une version plus récente et l'affiche sur le tableau de bord.
admin-settings-key-updates-feed-url = Flux des versions
admin-settings-help-updates-feed-url = L'adresse https que lit la recherche de mises à jour.
admin-settings-key-updates-apply = Appliquer les mises à jour
admin-settings-help-updates-apply = Indique si une mise à jour peut être lancée depuis la console par l'agent de mise à jour de l'hôte.
admin-settings-key-backup-retention-count = Sauvegardes conservées
admin-settings-help-backup-retention-count = Combien de sauvegardes de la base de données sont conservées.
admin-settings-key-about-security-contact = Contact sécurité
admin-settings-help-about-security-contact = Le contact indiqué dans security.txt (mailto: ou https:).
admin-settings-key-about-security-policy-url = Politique de sécurité
admin-settings-help-about-security-policy-url = L'adresse de votre politique de sécurité, indiquée dans security.txt.
admin-settings-key-about-project-links = Liens À propos
admin-settings-help-about-project-links = Liens affichés sur la page À propos, chacun avec un libellé et une adresse https.
admin-settings-key-config-snapshots-kept = Instantanés de configuration
admin-settings-help-config-snapshots-kept = Combien d'instantanés de configuration sont conservés.
admin-settings-key-jobs-history-days = Historique des tâches
admin-settings-help-jobs-history-days = Combien de jours d'exécutions de tâches sont conservés.
admin-settings-key-maintenance-announce-minutes-default = Préavis d'annonce de maintenance
admin-settings-help-maintenance-announce-minutes-default = Combien de minutes avant une fenêtre de maintenance les utilisateurs en sont informés.
admin-settings-key-maintenance-default-message = Message de maintenance
admin-settings-help-maintenance-default-message = Le texte que voient les utilisateurs pendant la maintenance, par langue.
admin-settings-key-health-disk-free-min-percent = Seuil d'espace disque libre
admin-settings-help-health-disk-free-min-percent = Le contrôle de disponibilité échoue lorsque l'espace disque libre passe sous ce pourcentage.
admin-settings-key-observability-otlp-metrics = Envoyer les métriques par OTLP
admin-settings-help-observability-otlp-metrics = Envoie les métriques du serveur au collecteur OTLP défini au démarrage.
admin-settings-key-metrics-drilldown-top-n = Lignes de détail des métriques
admin-settings-help-metrics-drilldown-top-n = Combien de lignes affiche une ventilation des métriques.
admin-settings-key-metrics-rollup-days = Historique des métriques
admin-settings-help-metrics-rollup-days = Combien de jours de synthèses horaires des métriques sont conservés.
admin-settings-key-observability-client-timing-enabled = Temps de chargement du navigateur
admin-settings-help-observability-client-timing-enabled = Collecte les temps de chargement des pages depuis les navigateurs connectés sous forme d'histogrammes agrégés anonymes. Rien n'identifie une personne, une session ou une page. Désactivez pour arrêter la collecte.
admin-settings-key-observability-client-timing-sample-rate-percent = Taux d'échantillonnage des temps de chargement du navigateur
admin-settings-help-observability-client-timing-sample-rate-percent = Pourcentage des sessions de navigateur qui transmettent des temps de chargement (de 1 à 100).
admin-settings-key-observability-client-timing-min-samples = Échantillons minimaux des temps de chargement
admin-settings-help-observability-client-timing-min-samples = La médiane et le 95e centile des temps de chargement restent masqués tant qu'une période n'a pas au moins ce nombre d'échantillons.
admin-settings-key-observability-client-timing-folder-loads = Temps de chargement des dossiers
admin-settings-help-observability-client-timing-folder-loads = Mesure le temps qu'un dossier met à afficher sa liste de messages, selon qu'elle provient du cache du navigateur ou du réseau. Activé par défaut. Les utilisateurs peuvent le désactiver pour eux-mêmes, sauf si ce paramètre est verrouillé. Nécessite que les temps de chargement du navigateur soient activés.
admin-settings-key-observability-client-timing-message-loads = Temps de chargement des messages
admin-settings-help-observability-client-timing-message-loads = Mesure le temps qu'un message met à afficher ses en-têtes et son corps nettoyé, selon qu'il était en cache ou préchargé, ou qu'il provient du réseau. Activé par défaut. Les utilisateurs peuvent le désactiver pour eux-mêmes, sauf si ce paramètre est verrouillé. Nécessite que les temps de chargement du navigateur soient activés.
admin-settings-key-diagnostics-retention-days = Historique des diagnostics
admin-settings-help-diagnostics-retention-days = Combien de jours d'exécutions de diagnostic sont conservés.
admin-settings-key-data-export-link-hours = Fenêtre de téléchargement des exports de données
admin-settings-help-data-export-link-hours = Pendant combien d'heures un export de données peut être téléchargé avant d'être supprimé.
admin-settings-key-dsr-enabled = Demandes relatives aux données
admin-settings-help-dsr-enabled = Indique si les personnes peuvent déposer des demandes relatives aux données (accès, téléchargement, rectification, effacement, limitation, opposition) depuis les paramètres.
admin-settings-key-dsr-type-access = Accepter les demandes d'accès
admin-settings-help-dsr-type-access = Indique si le type de demande d'accès est proposé aux utilisateurs.
admin-settings-key-dsr-type-portability = Accepter les demandes de téléchargement (copie portable)
admin-settings-help-dsr-type-portability = Indique si le type de demande de téléchargement (copie portable) est proposé aux utilisateurs.
admin-settings-key-dsr-type-rectification = Accepter les demandes de rectification
admin-settings-help-dsr-type-rectification = Indique si le type de demande de rectification est proposé aux utilisateurs.
admin-settings-key-dsr-type-erasure = Accepter les demandes d'effacement
admin-settings-help-dsr-type-erasure = Indique si le type de demande d'effacement est proposé aux utilisateurs.
admin-settings-key-dsr-type-restriction = Accepter les demandes de limitation
admin-settings-help-dsr-type-restriction = Indique si le type de demande de limitation est proposé aux utilisateurs.
admin-settings-key-dsr-type-objection = Accepter les demandes d'opposition
admin-settings-help-dsr-type-objection = Indique si le type de demande d'opposition est proposé aux utilisateurs.
admin-settings-key-dsr-type-other = Accepter les autres demandes
admin-settings-help-dsr-type-other = Indique si le type de demande générique est proposé aux utilisateurs.
admin-settings-key-dsr-approval-export = Approbation des demandes d'accès et de téléchargement
admin-settings-help-dsr-approval-export = auto lance l'export dès que la demande est confirmée. admin attend l'approbation d'un administrateur.
admin-settings-key-dsr-approval-erasure = Approbation des demandes d'effacement
admin-settings-help-dsr-approval-erasure = auto programme l'effacement dès que la demande est confirmée. admin attend l'approbation d'un administrateur.
admin-settings-key-dsr-approval-other = Traitement des autres demandes
admin-settings-help-dsr-approval-other = auto marque la demande comme en cours une fois confirmée. admin attend l'approbation d'un administrateur.
admin-settings-key-dsr-reauth-required = Nouvelle authentification pour les demandes
admin-settings-help-dsr-reauth-required = Exige un mot de passe ou un second facteur récent pour déposer une demande d'export ou d'effacement et pour créer un lien de téléchargement.
admin-settings-key-dsr-confirmation-required = Message de confirmation
admin-settings-help-dsr-confirmation-required = Envoie un message de confirmation avec un code dans la boîte de réception du demandeur ; la demande ne se poursuit pas tant que le code n'est pas saisi.
admin-settings-key-dsr-identity-verification-required = Étape de vérification d'identité
admin-settings-help-dsr-identity-verification-required = Un administrateur doit consigner que l'identité du demandeur a été vérifiée avant l'approbation de la demande.
admin-settings-key-dsr-erasure-grace-days = Délai de grâce avant effacement
admin-settings-help-dsr-erasure-grace-days = Jours entre l'approbation et l'effacement. Le demandeur peut annuler pendant ce délai. 0 exécute l'effacement au prochain passage du processus de traitement.
admin-settings-key-dsr-sla-days = Jours pour répondre à une demande
admin-settings-help-dsr-sla-days = Le nombre de jours entre le dépôt et la date d'échéance affichée aux utilisateurs et aux administrateurs.
admin-settings-key-dsr-sla-max-extension-days = Prolongation maximale
admin-settings-help-dsr-sla-max-extension-days = Le plus grand nombre de jours qu'un administrateur peut ajouter à l'échéance d'une demande, avec un motif consigné.
admin-settings-key-dsr-sla-reminder-days = Signalements d'échéance
admin-settings-help-dsr-sla-reminder-days = Jours avant l'échéance à partir desquels une demande est signalée comme bientôt due dans la file.
admin-settings-key-dsr-export-mail-allowed = Courrier dans les exports
admin-settings-help-dsr-export-mail-allowed = Permet à un demandeur d'inclure des messages dans un export. Seuls les messages conservés dans le cache propre à ce serveur sont inclus ; le courrier qui reste sur le serveur de messagerie amont ne l'est pas.
admin-settings-key-dsr-export-mail-format = Format du courrier dans les exports
admin-settings-help-dsr-export-mail-format = mbox écrit un fichier par dossier ; eml écrit un fichier par message.
admin-settings-key-dsr-export-include-settings = Export : paramètres
admin-settings-help-dsr-export-include-settings = Inclure les paramètres dans un export.
admin-settings-key-dsr-export-include-identities = Export : identités
admin-settings-help-dsr-export-include-identities = Inclure les identités dans un export.
admin-settings-key-dsr-export-include-contacts = Export : contacts
admin-settings-help-dsr-export-include-contacts = Inclure les contacts dans un export.
admin-settings-key-dsr-export-include-calendars = Export : calendriers
admin-settings-help-dsr-export-include-calendars = Inclure les calendriers dans un export.
admin-settings-key-dsr-export-include-notes = Export : notes
admin-settings-help-dsr-export-include-notes = Inclure les notes dans un export.
admin-settings-key-dsr-export-include-rules = Export : filtres et règles
admin-settings-help-dsr-export-include-rules = Inclure les filtres et les règles dans un export.
admin-settings-key-dsr-export-include-sessions = Export : sessions, appareils et historique de connexion
admin-settings-help-dsr-export-include-sessions = Inclure les sessions, les appareils et l'historique de connexion dans un export.
admin-settings-key-dsr-export-include-consent = Export : enregistrements de consentement
admin-settings-help-dsr-export-include-consent = Inclure les enregistrements de consentement dans un export.
admin-settings-key-dsr-export-include-audit = Export : entrées d'audit concernant la personne
admin-settings-help-dsr-export-include-audit = Inclure dans un export les entrées d'audit concernant la personne.
admin-settings-key-dsr-export-max-megabytes = Taille maximale de l'export
admin-settings-help-dsr-export-max-megabytes = Plus grand export en mégaoctets. Les sections qui le dépasseraient sont omises et listées dans le manifeste.
admin-settings-key-dsr-export-format = Formats d'export
admin-settings-help-dsr-export-format = json écrit des fichiers JSON. jsonCsv écrit aussi des fichiers CSV pour les données de type liste.
admin-settings-key-dsr-export-encrypt = Chiffrer les exports au repos
admin-settings-help-dsr-export-encrypt = Stocke les exports générés chiffrés avec la clé du serveur jusqu'à leur téléchargement ou leur expiration.
admin-settings-key-dsr-download-link-minutes = Durée de vie du lien de téléchargement
admin-settings-help-dsr-download-link-minutes = Minutes pendant lesquelles un lien de téléchargement fonctionne après sa création. L'export lui-même est conservé pendant la fenêtre de téléchargement des exports de données.
admin-settings-key-dsr-download-single-use = Lien de téléchargement à usage unique
admin-settings-help-dsr-download-single-use = Un lien de téléchargement cesse de fonctionner après avoir été utilisé une fois.
admin-settings-key-dsr-retention-completed-days = Conservation des demandes terminées
admin-settings-help-dsr-retention-completed-days = Jours pendant lesquels une demande terminée et son historique sont conservés avant d'être supprimés.
admin-settings-key-dsr-notify-admins = Consigner les nouvelles demandes pour les administrateurs
admin-settings-help-dsr-notify-admins = Consigne chaque nouvelle demande comme événement de sécurité dans le journal d'audit, où les administrateurs et les destinations d'alerte la voient.
admin-settings-key-dsr-notify-requester = Messages d'avancement au demandeur
admin-settings-help-dsr-notify-requester = Dépose un message dans la boîte de réception du demandeur lorsqu'une demande change d'état.
admin-settings-key-dsr-contact-name = Nom du contact protection des données
admin-settings-help-dsr-contact-name = Nom affiché aux utilisateurs sur la page des demandes relatives aux données.
admin-settings-key-dsr-contact-email = Adresse du contact protection des données
admin-settings-help-dsr-contact-email = Adresse e-mail affichée aux utilisateurs sur la page des demandes relatives aux données.
admin-settings-key-dsr-contact-url = Page du contact protection des données
admin-settings-help-dsr-contact-url = Lien affiché aux utilisateurs sur la page des demandes relatives aux données.
admin-settings-key-dsr-on-behalf-allowed = Demandes au nom d'un utilisateur
admin-settings-help-dsr-on-behalf-allowed = Permet à un administrateur de déposer une demande pour un utilisateur, par exemple une demande reçue par courrier postal.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-dsr-rate-limit-per-day = Demandes par personne et par jour
admin-settings-help-dsr-rate-limit-per-day = Le plus grand nombre de demandes qu'une personne peut déposer en 24 heures.
admin-settings-key-dsr-legal-hold-subjects = Conservations légales
admin-settings-help-dsr-legal-hold-subjects = Adresses, ou entrées @domaine, dont l'effacement est bloqué. Une demande d'effacement les concernant reste dans la file comme bloquée jusqu'à la suppression de l'entrée.
admin-settings-key-scrub-enabled = Épuration d'une personne
admin-settings-help-scrub-enabled = Autorise l'action Épurer la personne et l'épuration qui peut suivre un effacement. Désactivé, ni l'une ni l'autre ne s'exécute.
admin-settings-key-scrub-pseudonym-algorithm = Longueur du pseudonyme
admin-settings-help-scrub-pseudonym-algorithm = Comment est produit le pseudonyme à clé qui remplace un identifiant : HMAC-SHA256 sous la génération de clé actuelle, tronqué à 16 ou 32 caractères hexadécimaux.
admin-settings-key-scrub-include-logs = Épurer les fichiers journaux
admin-settings-help-scrub-include-logs = Une épuration réécrit aussi l'identifiant dans les fichiers journaux archivés, y compris compressés.
admin-settings-key-scrub-auto-run-on-erasure = Épuration après effacement
admin-settings-help-scrub-auto-run-on-erasure = Un effacement terminé remplace aussi les identifiants de la personne dans l'historique d'audit et les journaux par le pseudonyme. Désactivé par défaut.
admin-settings-key-scrub-key-retention-days = Conservation des clés de pseudonyme
admin-settings-help-scrub-key-retention-days = Jours pendant lesquels une clé de pseudonyme est conservée après sa rotation. 0 la conserve. Après suppression, les anciens pseudonymes ne peuvent plus être recalculés et les marqueurs créés avec la clé ne peuvent plus être rejoués.
admin-settings-key-backup-propagate-erasure = Rejouer les effacements à la restauration
admin-settings-help-backup-propagate-erasure = Une restauration rejoue les marqueurs d'effacement plus récents que la sauvegarde, de sorte que les personnes effacées depuis sont de nouveau supprimées.
admin-settings-key-backup-tombstone-retention-days = Conservation des marqueurs d'effacement
admin-settings-help-backup-tombstone-retention-days = Jours pendant lesquels un marqueur d'effacement est conservé. 0 les conserve indéfiniment.
admin-settings-key-dsr-text-intro = Introduction des demandes relatives aux données
admin-settings-help-dsr-text-intro = Texte affiché au-dessus du formulaire de demande. Une valeur par langue.
admin-settings-key-dsr-text-confirmation = Texte du message de confirmation
admin-settings-help-dsr-text-confirmation = Texte ajouté au message de confirmation. Une valeur par langue.
admin-settings-key-dsr-text-completion = Texte du message de clôture
admin-settings-help-dsr-text-completion = Texte ajouté au message envoyé lorsqu'une demande est terminée. Une valeur par langue.
admin-settings-key-server-text-family = Texte du serveur
admin-settings-help-server-text-family = Texte que le serveur écrit dans les e-mails et les notifications, par langue et par domaine.
admin-settings-cat-sieve = Filtres de messagerie
admin-settings-key-sieve-max-script-bytes = Taille maximale d'un script de filtre
admin-settings-help-sieve-max-script-bytes = Nombre maximal d'octets dans un script de filtre Sieve.
admin-settings-key-sieve-max-scripts = Nombre maximal de scripts de filtre
admin-settings-help-sieve-max-scripts = Nombre maximal de scripts enregistrés par compte.
admin-settings-key-sieve-max-redirects = Nombre maximal de redirections
admin-settings-help-sieve-max-redirects = Nombre maximal de commandes de redirection dans un script.
admin-settings-key-sieve-vacation-allowed = Actions d'absence dans les filtres
admin-settings-help-sieve-vacation-allowed = Autorise les scripts à envoyer des réponses d'absence lorsque le backend les prend en charge.
admin-settings-key-sieve-allowed-extensions = Extensions de filtre autorisées
admin-settings-help-sieve-allowed-extensions = Une liste facultative de noms d'extensions Sieve en minuscules. Le backend doit aussi prendre en charge chaque extension.
admin-settings-key-cap-sieve-raw-edit = Modifier la source du script de filtre
admin-settings-help-cap-sieve-raw-edit = Autorise la modification directe des scripts Sieve pour les comptes sélectionnés.
admin-settings-key-cap-sieve-forward-external = Transférer hors du domaine du compte
admin-settings-help-cap-sieve-forward-external = Autorise le transfert externe dans les scripts. L'enregistrement de ces scripts exige aussi une vérification récente.
admin-settings-key-session-max-concurrent = Sessions simultanées
admin-settings-help-session-max-concurrent = Nombre maximal de sessions par compte. Les sessions les plus anciennes sont révoquées lorsqu'une nouvelle connexion dépasse cette limite.
admin-settings-key-session-idle-secs = Délai d'inactivité de la session
admin-settings-help-session-idle-secs = Termine les sessions inactives après ce nombre de secondes, dans la limite du déploiement.
admin-settings-key-session-absolute-secs = Durée de vie de la session
admin-settings-help-session-absolute-secs = Termine les sessions après ce nombre de secondes, dans la limite du déploiement.
admin-settings-key-twofa-remember-days = Durée de vie des appareils mémorisés
admin-settings-help-twofa-remember-days = Jours avant qu'un appareil mémorisé doive de nouveau vérifier un second facteur. Zéro désactive cette fonctionnalité.
admin-settings-key-twofa-passkeys-enabled = Autoriser les clés d'accès
admin-settings-help-twofa-passkeys-enabled = Autorise les clés d'accès comme facteur d'authentification.
admin-settings-key-twofa-required = Exiger l'authentification à deux facteurs
admin-settings-help-twofa-required = Exige un second facteur enregistré avant d'émettre une session.
admin-settings-key-twofa-passkey-attachment = Type d'authentificateur de clé d'accès
admin-settings-help-twofa-passkey-attachment = any (par défaut) : tous les authentificateurs. platform : clés d'accès stockées sur l'appareil (Windows Hello, Touch ID, verrouillage d'écran Android). cross-platform : clés de sécurité et téléphones. Le navigateur le signale ; ce n'est pas signé par l'authentificateur.
admin-settings-key-twofa-passkey-user-verification = Vérification de l'utilisateur pour les clés d'accès
admin-settings-help-twofa-passkey-user-verification = required : la clé d'accès doit vérifier un code PIN ou une donnée biométrique à son ajout et à chaque connexion ; une opération sans vérification est refusée. preferred (par défaut) et discouraged ne changent que ce qui est demandé au navigateur.
admin-settings-key-twofa-passkey-resident-key = Clés d'accès détectables
admin-settings-help-twofa-passkey-resident-key = required : l'identifiant doit être stocké sur l'authentificateur. discouraged (par défaut) ou preferred. En mode moteur, les clés d'accès sont toujours détectables, car elles permettent aussi de se connecter sans mot de passe.
admin-settings-key-twofa-passkey-legacy = Clés d'accès ajoutées avant la politique actuelle
admin-settings-help-twofa-passkey-legacy = allow (par défaut) : une ancienne clé d'accès qui ne respecte pas la politique continue de fonctionner et l'utilisateur est invité à la remplacer. refuse : elle est refusée à la connexion avec un message ; l'utilisateur se connecte avec un autre facteur ou un code de récupération.
admin-settings-key-twofa-last-factor-removal = Suppression du dernier second facteur
admin-settings-help-twofa-last-factor-removal = Lorsque la suppression d'une clé d'accès ou d'une application d'authentification laisserait le compte sans aucun second facteur. allow (par défaut) : il est supprimé et la connexion n'exige plus que le mot de passe. block : l'utilisateur doit d'abord ajouter une autre méthode de connexion (les codes de récupération ne comptent pas). Là où l'authentification à deux facteurs est exigée, le dernier facteur est toujours conservé.
admin-settings-key-privacy-remote-content = Contenu distant
admin-settings-help-privacy-remote-content = Bloquer les images, demander d'abord, utiliser le proxy, ou charger directement lorsque c'est autorisé.
admin-settings-key-privacy-trackers = Bloquer les traqueurs
admin-settings-help-privacy-trackers = Bloque les images de suivi d'e-mails connues.
admin-settings-key-privacy-clean-links = Nettoyer les liens
admin-settings-help-privacy-clean-links = Supprime les paramètres de suivi des liens des messages.
admin-settings-key-privacy-max-bytes = Limite des images distantes
admin-settings-help-privacy-max-bytes = Nombre maximal d'octets récupérés pour une image distante.
admin-settings-key-privacy-cache-bytes = Cache des images distantes
admin-settings-help-privacy-cache-bytes = Nombre maximal d'octets conservés dans le cache des images distantes.
admin-settings-key-privacy-rate-per-min = Débit des requêtes d'images distantes
admin-settings-help-privacy-rate-per-min = Nombre maximal de récupérations d'images distantes par compte et par minute.
admin-settings-privacy-fetch-title = Récupération des images distantes
admin-settings-privacy-fetch-help = Délais, redirections et téléchargements parallèles pour les images que le serveur récupère pour le lecteur. Une image qui dépasse une limite affiche un espace réservé.
admin-settings-key-privacy-fetch-connect-timeout-ms = Délai de connexion des images distantes
admin-settings-help-privacy-fetch-connect-timeout-ms = Millisecondes accordées pour ouvrir la connexion à un hôte d'images, poignée de main TLS comprise. Par défaut 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Délai de réponse des images distantes
admin-settings-help-privacy-fetch-headers-timeout-ms = Millisecondes accordées entre le début d'une requête et la réponse de l'hôte d'images avec ses en-têtes. Par défaut 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Délai de blocage des images distantes
admin-settings-help-privacy-fetch-idle-timeout-ms = Plus longue pause, en millisecondes, entre deux parties d'un téléchargement d'image avant son interruption. Par défaut 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Délai total des images distantes
admin-settings-help-privacy-fetch-total-timeout-ms = Millisecondes accordées pour récupérer une image, redirections comprises. L'attente en file a son propre budget, de sorte qu'une image très sollicitée n'épuise pas son temps réseau avant de commencer. Par défaut 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Délai de file d'attente des images distantes
admin-settings-help-privacy-fetch-queue-timeout-ms = Nombre maximal de millisecondes pendant lesquelles les images peuvent attendre de la capacité, en plus de leur budget de récupération. La file bornée absorbe de grands pics authentifiés tandis que les limites de récupération et de décodage actifs restent appliquées. Par défaut 60000.
admin-settings-key-privacy-max-redirects = Redirections des images distantes
admin-settings-help-privacy-max-redirects = Redirections suivies pour une image. Chacune est vérifiée selon les mêmes règles d'adresse que la première requête. Par défaut 3.
admin-settings-key-privacy-max-concurrent-per-message = Téléchargements d'images distantes par message
admin-settings-help-privacy-max-concurrent-per-message = Images d'un message récupérées en même temps. Les autres attendent une place libre. Plage 1–128 ; par défaut 30.
admin-settings-key-privacy-max-concurrent-per-account = Téléchargements d'images distantes par compte
admin-settings-help-privacy-max-concurrent-per-account = Images récupérées en même temps pour un compte sur l'ensemble de ses messages ouverts. Plage 1–256 ; par défaut 128.
admin-settings-key-privacy-allow-direct = Autoriser les images directes
admin-settings-key-cap-remote-content-direct = Autorisation des images directes
admin-settings-help-cap-remote-content-direct = Autorise ce compte à choisir les images directes lorsque le serveur le permet.
admin-settings-help-privacy-allow-direct = Autorise le chargement direct des images pour les comptes ayant l'autorisation requise.
admin-settings-key-logs-auth-retention-days = Conservation de l'historique d'authentification
admin-settings-help-logs-auth-retention-days = Jours pendant lesquels les métadonnées d'authentification sont conservées, indépendamment de l'identité durable du compte.
admin-settings-key-logs-auth-max-entries = Limite d'entrées de l'historique d'authentification
admin-settings-help-logs-auth-max-entries = Nombre maximal d'entrées conservées. Les échecs répétés identiques en une minute partagent une entrée.
admin-settings-key-reader-color-mode = Couleurs des messages
admin-settings-cat-reader = Affichage des messages
admin-settings-key-mail-row-actions-mode = Actions rapides de la liste des messages
admin-settings-help-mail-row-actions-mode = Comment les actions rapides de chaque ligne de message apparaissent : open (toujours disponibles, révélées au survol ou au focus), hover (trois commandes et un bouton Plus), click (un bouton qui les ouvre) ou off (retirées des lignes ; le menu contextuel et la barre d'outils conservent toutes les actions). Hover se comporte comme click sur les écrans tactiles. Les utilisateurs peuvent choisir la leur, sauf si un administrateur la verrouille.
admin-settings-key-mail-quick-reply = Réponse rapide
admin-settings-help-mail-quick-reply = Propose un onglet de réponse compact avec les destinataires déjà renseignés. Désactivé par défaut ; les utilisateurs peuvent l'activer, sauf si un administrateur le verrouille. Les protections d'envoi habituelles s'appliquent toujours.
admin-settings-key-mail-request-read-receipt = Demander des accusés de lecture par défaut
admin-settings-help-mail-request-read-receipt = Démarre les messages sortants pris en charge avec une demande d'accusé sélectionnée. C'est une valeur par défaut de rédaction, pas une garantie que le destinataire enverra un accusé.
admin-settings-help-reader-color-mode = Automatique suit les thèmes sombres. Choisissez un rendu original, adaptatif ou inversé dans le cadre isolé du message.
admin-settings-key-reader-font-size = Taille de police des messages
admin-settings-help-reader-font-size = Taille de lecture par défaut en pixels, de 12 à 24.
admin-settings-key-reader-line-height = Hauteur de ligne des messages
admin-settings-help-reader-line-height = Interligne de lecture de 1,2 à 2,2.
admin-settings-key-reader-content-width = Largeur de lecture des messages
admin-settings-help-reader-content-width = Contenu des messages confortable, large ou sur toute la largeur du volet.
admin-settings-key-reader-loading = Apparence du chargement des messages
admin-settings-help-reader-loading = Lignes d'espace réservé, indicateur textuel ou aucun indicateur visuel. Les annonces d'accessibilité restent disponibles.
admin-settings-key-reader-loading-delay-ms = Délai de chargement des messages
admin-settings-help-reader-loading-delay-ms = Retarde uniquement l'indicateur de chargement, entre 0 et 1000 millisecondes. La remise du message n'est jamais retardée.
admin-settings-key-reader-zoom = Zoom des messages
admin-settings-help-reader-zoom = Zoom par défaut du corps du message en pourcentage, de 50 à 200 ; le lecteur utilise le palier de zoom le plus proche. Seul le message est mis à l'échelle, et l'impression utilise 100 %.
admin-settings-key-reader-attachments-collapsed = Barre des pièces jointes réduite
admin-settings-help-reader-attachments-collapsed = Indique si la barre des pièces jointes du lecteur démarre réduite, n'affichant que le nombre et la taille totale. La dernière ouverture ou fermeture de l'utilisateur est enregistrée ici. Désactivé par défaut : les pièces jointes sont affichées.
admin-settings-key-login-show-domain-hint = Afficher le domaine préféré à la connexion
admin-settings-help-login-show-domain-hint = Indique quel domaine accepte un nom d'utilisateur sans suffixe d'e-mail. Masqué par défaut. Cela ne change que l'indication ; la connexion avec un nom d'utilisateur fonctionne toujours.
admin-settings-key-signin-retry-cooldown-secs = Délai entre les tentatives de connexion
admin-settings-help-signin-retry-cooldown-secs = Nombre minimal de secondes entre les tentatives de connexion par mot de passe depuis un même réseau client. Par défaut 3 ; 0 désactive le délai. Les bannissements existants après échecs s'appliquent toujours.
admin-settings-key-mail-quote-style = Style d'en-tête des réponses et transferts
admin-settings-key-mail-external-recipient-warning = Avertissement de destinataire externe
admin-settings-help-mail-external-recipient-warning = Désactivé, avertir, ou exiger une confirmation dans l'éditeur. Couvre À, Cc et Cci. C'est une protection de l'éditeur, pas une restriction de livraison du serveur.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-mail-trusted-recipient-domains = Domaines de destinataires de confiance
admin-settings-help-mail-trusted-recipient-domains = Jusqu'à 100 domaines de destinataires exacts. Le domaine de l'identité d'envoi sélectionnée est aussi de confiance. Les entrées non valides n'accordent jamais la confiance. Verrouillez ce paramètre pour empêcher les remplacements par les utilisateurs.
admin-settings-help-mail-quote-style = Utilisez des réponses citées classiques ou des en-têtes de message de style Outlook. Cela définit la valeur par défaut des utilisateurs ; verrouillez-la pour imposer le style. Les brouillons existants ne changent pas.
admin-settings-key-signin-anti-bot-mode = Protection anti-robots à la connexion
admin-settings-help-signin-anti-bot-mode = Vérification par preuve de travail auto-hébergée : off, mailbox, admin ou all. Aucun service de vérification externe n'est contacté.
admin-settings-key-signin-anti-bot-difficulty = Travail de vérification à la connexion
admin-settings-help-signin-anti-bot-difficulty = Compteur maximal du résolveur, 100–20000. Des valeurs plus élevées demandent plus de travail au navigateur ; par défaut 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Durée de validité de la vérification à la connexion
admin-settings-help-signin-anti-bot-expiry-secs = Secondes pendant lesquelles une vérification à usage unique reste valide, 60–600 ; par défaut 180. Une nouvelle vérification est nécessaire après chaque tentative d'authentification.
admin-settings-key-attachments-preview-allowed = Autoriser les aperçus de pièces jointes
admin-settings-help-attachments-preview-allowed = Désactivez pour empêcher le rendu des aperçus de pièces jointes ; les téléchargements d'originaux ont leur propre contrôle.
admin-settings-key-attachments-download-allowed = Autoriser la remise de la pièce jointe d'origine
admin-settings-help-attachments-download-allowed = La désactivation empêche aussi les aperçus qui exigent les octets d'origine. Les aperçus du navigateur ne protègent pas contre la copie.
admin-settings-key-attachments-max-bytes = Taille maximale des pièces jointes en octets
admin-settings-help-attachments-max-bytes = Zéro signifie illimité. S'applique aux octets d'origine remis, y compris les requêtes de plage.
admin-settings-key-attachments-allowed-extensions = Extensions de nom de fichier autorisées
admin-settings-help-attachments-allowed-extensions = Tableau JSON d'extensions en minuscules sans point, comme ["pdf","tar.gz"]. Vide autorise toute extension ; les règles de blocage l'emportent.
admin-settings-key-attachments-blocked-extensions = Extensions de nom de fichier bloquées
admin-settings-help-attachments-blocked-extensions = Tableau JSON d'extensions en minuscules. Évalué d'après les noms de fichier MIME enregistrés, jamais d'après le nom dans l'URL de téléchargement.
admin-settings-key-attachments-warn-extensions = Avertir pour les extensions de nom de fichier
admin-settings-help-attachments-warn-extensions = Tableau JSON d'extensions exigeant une confirmation explicite avant la remise de l'original.
admin-settings-key-attachments-allowed-mime-types = Types MIME autorisés
admin-settings-help-attachments-allowed-mime-types = Tableau JSON de types en minuscules ou de jokers de premier niveau, comme ["image/*","application/pdf"]. Vide autorise tout type.
admin-settings-key-attachments-blocked-mime-types = Types MIME bloqués
admin-settings-help-attachments-blocked-mime-types = Tableau JSON de types en minuscules ou de jokers de premier niveau. Les règles de blocage ont la priorité.
admin-settings-key-attachments-warn-mime-types = Avertir pour les types MIME
admin-settings-help-attachments-warn-mime-types = Tableau JSON de types en minuscules ou de jokers de premier niveau exigeant une confirmation. Les métadonnées MIME ne constituent pas une détection de logiciels malveillants.
admin-settings-cat-shortcuts = Raccourcis clavier
admin-settings-cat-time = Fuseaux horaires
admin-settings-key-time-display-zone = Fuseau horaire d'affichage
admin-settings-help-time-display-zone = Fuseau horaire IANA comme Europe/Lisbon, ou local pour suivre l'appareil actuel. Les horodatages des messages restent enregistrés comme leurs instants d'origine.
admin-settings-key-time-comparison-zone = Fuseau horaire de comparaison
admin-settings-help-time-comparison-zone = Second fuseau horaire IANA facultatif pour les aides des messages et du calendrier. Laissez vide pour masquer la comparaison ; aucun lieu n'est déduit d'une adresse e-mail.
admin-settings-key-time-show-helpers = Afficher les aides de fuseau horaire
admin-settings-help-time-show-helpers = Affiche des comparaisons entre l'heure locale et le fuseau choisi pour les horodatages des messages et les événements du calendrier. Les verrous d'administrateur peuvent imposer ce choix.
admin-settings-key-shortcuts-enabled = Activer les raccourcis clavier
admin-settings-help-shortcuts-enabled = Autorise les raccourcis de l'application. Les verrous peuvent les désactiver globalement ou pour un domaine. Les champs de saisie et les boîtes de dialogue modales conservent leur gestion normale du clavier.
admin-settings-key-shortcuts-compose = Raccourci Rédiger
admin-settings-help-shortcuts-compose = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-archive = Raccourci Archiver
admin-settings-help-shortcuts-archive = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-reply = Raccourci Répondre
admin-settings-help-shortcuts-reply = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-reply-all = Raccourci Répondre à tous
admin-settings-help-shortcuts-reply-all = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-forward = Raccourci Transférer
admin-settings-help-shortcuts-forward = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-next = Raccourci Message suivant
admin-settings-help-shortcuts-next = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-previous = Raccourci Message précédent
admin-settings-help-shortcuts-previous = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-search = Raccourci Placer le focus sur la recherche
admin-settings-help-shortcuts-search = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-refresh = Raccourci Actualiser les messages
admin-settings-help-shortcuts-refresh = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-settings = Raccourci Ouvrir les paramètres
admin-settings-help-shortcuts-settings = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-trash = Raccourci Déplacer vers la corbeille
admin-settings-help-shortcuts-trash = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-mark-read = Raccourci Marquer comme lu
admin-settings-help-shortcuts-mark-read = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-mark-unread = Raccourci Marquer comme non lu
admin-settings-help-shortcuts-mark-unread = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-spam = Raccourci Marquer comme indésirable
admin-settings-help-shortcuts-spam = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-zoom-in = Raccourci Agrandir le message
admin-settings-help-shortcuts-zoom-in = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-zoom-out = Raccourci Réduire le message
admin-settings-help-shortcuts-zoom-out = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-zoom-reset = Raccourci Réinitialiser le zoom du message
admin-settings-help-shortcuts-zoom-reset = Utilisez inherit pour le préréglage sélectionné, disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-shortcuts-open-in-tab = Raccourci Ouvrir le message dans son propre onglet
admin-settings-help-shortcuts-open-in-tab = Utilisez inherit pour le préréglage sélectionné (Shift+Enter), disabled pour supprimer cette action, ou une combinaison de touches comme Ctrl+Shift+K. Les combinaisons en conflit ou non valides ne s'exécutent pas.
admin-settings-key-notifications-push-allowed = Notifications push en arrière-plan autorisées
admin-settings-help-notifications-push-allowed = Autorise les notifications activées de ce compte à s'enregistrer pour recevoir des notifications push en arrière-plan. L'autorisation du navigateur reste requise.
admin-settings-key-offline-persistent-storage-allowed = Demandes de stockage persistant du navigateur autorisées
admin-settings-help-offline-persistent-storage-allowed = Autorise une demande explicite visant à protéger les données mises en cache localement contre l'éviction automatique par le navigateur. Le navigateur peut refuser ; cela n'accorde pas l'autorisation automatiquement.
admin-settings-key-security-signin-mailbox-failure-limit = Limite d'identifiants de boîte aux lettres refusés
admin-settings-help-security-signin-mailbox-failure-limit = Des identifiants refusés depuis une même adresse IP source dans la fenêtre de comptage déclenchent un verrouillage temporaire de la connexion à la boîte aux lettres. Les échecs de réseau et de configuration ne comptent pas.
admin-settings-key-security-signin-admin-failure-limit = Limite d'identifiants d'administrateur refusés
admin-settings-help-security-signin-admin-failure-limit = Des identifiants d'administrateur refusés depuis une même adresse IP source déclenchent un verrouillage temporaire distinct de la connexion administrateur. La session administrateur courte et les exigences de preuve récente restent inchangées.
admin-settings-key-security-signin-failure-window-secs = Fenêtre de comptage des identifiants refusés
admin-settings-help-security-signin-failure-window-secs = Fenêtre glissante en secondes pour les refus d'identifiants et de MFA. Une connexion terminée avec succès réinitialise cette source et cette surface.
admin-settings-key-security-signin-lock-secs = Durée du verrouillage temporaire de connexion
admin-settings-help-security-signin-lock-secs = Durée du verrouillage en secondes. Les tentatives répétées pendant le verrouillage ne prolongent pas son expiration d'origine. Les bannissements d'IP globaux manuels sont distincts.
admin-settings-key-security-signin-geo-scope = Portée de la politique de pays à la connexion
admin-settings-help-security-signin-geo-scope = Désactivé par défaut. S'applique uniquement aux nouvelles connexions de boîte aux lettres, d'administrateur ou à toutes. Les sessions existantes ne sont pas révoquées. Consultez le diagnostic de la source actuelle avant d'activer ; une base GeoIP locale indisponible refuse la connexion.
admin-settings-key-security-signin-geo-mode = Liste d'autorisation ou de refus de pays
admin-settings-help-security-signin-geo-mode = allow n'autorise que les pays listés ; deny refuse les pays listés. Les adresses inconnues suivent leur propre politique explicite.
admin-settings-key-security-signin-geo-countries = Codes de pays pour la connexion
admin-settings-help-security-signin-geo-countries = Tableau JSON de codes de pays à deux lettres en majuscules, par exemple ["PT", "GB"]. Utilise uniquement la base de pays locale (Sécurité > Base de pays, ou MW_GEOIP_DB) ; les adresses ne sont jamais envoyées à un service de localisation.
admin-settings-key-security-signin-geo-unknown = Politique des adresses sources non associées
admin-settings-help-security-signin-geo-unknown = Autorise ou refuse les adresses absentes d'une base locale lisible, y compris les adresses privées. Les bases absentes ou illisibles refusent toujours les connexions soumises à une politique active, même si les adresses inconnues sont autorisées.
admin-settings-key-contacts-avatar-mode = Photos de contacts et avatars générés
admin-settings-help-contacts-avatar-mode = Générer des initiales localement, masquer les avatars, ou utiliser les photos intégrées des carnets d'adresses propres à chaque compte. Aucune URL de photo distante ni service d'avatars externe n'est contacté. La politique par utilisateur et par domaine peut être fixée par un administrateur.
admin-settings-cat-contacts = Photos de contacts
admin-settings-key-reader-action-style = Ruban d'actions des messages
admin-settings-help-reader-action-style = Les icônes compactes sont le choix par défaut. Choisissez labels pour des icônes avec texte visible ; les deux conservent les noms d'actions et des infobulles thématisées. Prend en charge les verrous globaux, de domaine et d'utilisateur.
admin-settings-key-nextcloud-sync-interval-secs = Intervalle d'actualisation automatique de Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Secondes entre les actualisations en arrière-plan des collections pour les comptes connectés au moteur natif. Zéro signifie manuel uniquement ; les valeurs positives ont un minimum de soixante secondes. Les échecs espacent les tentatives. Aucune connexion en arrière-plan n'est tentée.
admin-settings-key-mail-folder-role-inbox = Association du dossier Boîte de réception
admin-settings-help-mail-folder-role-inbox = Null utilise la détection automatique. Utilisez un objet de chemin exact pour des valeurs partagées par défaut, ou un objet d'identifiant de boîte aux lettres propre à un compte. Les cibles manquantes ou en conflit désactivent ce rôle ; les verrous d'administrateur s'appliquent.
admin-settings-key-mail-folder-role-drafts = Association du dossier Brouillons
admin-settings-help-mail-folder-role-drafts = Null utilise la détection automatique. Utilisez un objet de chemin exact pour des valeurs partagées par défaut, ou un objet d'identifiant de boîte aux lettres propre à un compte. Les cibles manquantes ou en conflit désactivent ce rôle ; les verrous d'administrateur s'appliquent.
admin-settings-key-mail-folder-role-sent = Association du dossier Envoyés
admin-settings-help-mail-folder-role-sent = Null utilise la détection automatique. Utilisez un objet de chemin exact pour des valeurs partagées par défaut, ou un objet d'identifiant de boîte aux lettres propre à un compte. Les cibles manquantes ou en conflit désactivent ce rôle ; les verrous d'administrateur s'appliquent.
admin-settings-key-mail-folder-role-trash = Association de la corbeille
admin-settings-help-mail-folder-role-trash = Null utilise la détection automatique. Utilisez un objet de chemin exact pour des valeurs partagées par défaut, ou un objet d'identifiant de boîte aux lettres propre à un compte. Les cibles manquantes ou en conflit désactivent ce rôle ; les verrous d'administrateur s'appliquent.
admin-settings-key-mail-folder-role-junk = Association du dossier Courrier indésirable
admin-settings-help-mail-folder-role-junk = Null utilise la détection automatique. Utilisez un objet de chemin exact pour des valeurs partagées par défaut, ou un objet d'identifiant de boîte aux lettres propre à un compte. Les cibles manquantes ou en conflit désactivent ce rôle ; les verrous d'administrateur s'appliquent.
admin-settings-key-mail-folder-role-archive = Association du dossier Archives
admin-settings-help-mail-folder-role-archive = Null utilise la détection automatique. Utilisez un objet de chemin exact pour des valeurs partagées par défaut, ou un objet d'identifiant de boîte aux lettres propre à un compte. Les cibles manquantes ou en conflit désactivent ce rôle ; les verrous d'administrateur s'appliquent.
admin-settings-key-mail-request-delivery-receipt = Demander des notifications de livraison par défaut
admin-settings-help-mail-request-delivery-receipt = Sélectionne les notifications de livraison pour les nouveaux messages lorsque le compte prend en charge les DSN SMTP. Le verrouillage contrôle cette valeur par défaut enregistrée ; receipts.dsnAllowed détermine séparément si les demandes sont permises. La livraison ne prouve pas la lecture.
admin-settings-key-security-signin-geo-enforce-sessions = Appliquer la politique de pays aux sessions existantes
admin-settings-help-security-signin-geo-enforce-sessions = Désactivé par défaut. Applique la portée de pays de connexion configurée aux requêtes authentifiées, à l'accès bearer natif et au contenu signé du lecteur. Les clients bloqués peuvent toujours se déconnecter. Les flux déjà ouverts sont vérifiés à leur reconnexion ; cela ne met pas fin aux connexions existantes.
admin-settings-key-security-protection-rate-mode = Budgets de requêtes agrégés
admin-settings-help-security-protection-rate-mode = Désactivé par défaut. observe consigne l'épuisement du budget sans refus ; enforce renvoie un 429 temporaire. Les limites de concurrence de protocole existantes restent indépendantes.
admin-settings-key-security-protection-adaptive-mode = Protection adaptative des requêtes
admin-settings-help-security-protection-adaptive-mode = Désactivé par défaut. observe consigne des anomalies typées explicables et des actions proposées ; enforce applique des ralentissements ou blocages temporaires bornés. C'est une politique de compteurs déterministe, pas un classificateur d'IA.
admin-settings-key-security-protection-client-requests = Requêtes de client par fenêtre
admin-settings-help-security-protection-client-requests = Requêtes par fenêtre depuis une adresse non connectée, et chaque requête de connexion. Une requête avec une session de boîte aux lettres ou d'administrateur valide utilise à la place son propre quota, plus large. Zéro désactive ce budget.
admin-settings-key-security-protection-account-requests = Requêtes de compte par fenêtre
admin-settings-help-security-protection-account-requests = Requêtes par fenêtre pour une boîte aux lettres connectée, sur l'ensemble de ses sessions, appareils et adresses. Maintenu au-dessus du quota anonyme pour que l'usage connecté ait plus de marge. Zéro désactive ce budget.
admin-settings-key-rate-limit-diagnostics-burst = Exécutions de diagnostic simultanées
admin-settings-help-rate-limit-diagnostics-burst = Exécutions de diagnostic et vérifications de backend qu'une session d'administrateur peut lancer à la suite. Lancer une vérification déjà en cours renvoie cette vérification au lieu de compter à nouveau.
admin-settings-key-rate-limit-diagnostics-per-minute = Exécutions de diagnostic par minute
admin-settings-help-rate-limit-diagnostics-per-minute = Exécutions de diagnostic et vérifications de backend par minute que récupère une session d'administrateur.
admin-settings-key-security-protection-admin-requests = Requêtes d'administrateur par fenêtre
admin-settings-help-security-protection-admin-requests = Requêtes par fenêtre pour un administrateur connecté. Maintenu au-dessus du quota anonyme. Zéro désactive ce budget.
admin-settings-key-security-protection-rate-window-secs = Fenêtre du budget de requêtes (secondes)
admin-settings-help-security-protection-rate-window-secs = Intervalle fixe du budget de requêtes. Le délestage de charge par capacité et les réponses ordinaires « introuvable » ne sont pas des preuves adaptatives.
admin-settings-key-security-protection-adaptive-clients = Observer les clients pour la protection adaptative
admin-settings-help-security-protection-adaptive-clients = Active cette dimension de sujet lorsque le mode adaptatif est actif. Les comptes et les administrateurs ne sont comptés qu'après vérification de leur identité.
admin-settings-key-security-protection-adaptive-accounts = Observer les comptes pour la protection adaptative
admin-settings-help-security-protection-adaptive-accounts = Active cette dimension de sujet lorsque le mode adaptatif est actif. Les comptes et les administrateurs ne sont comptés qu'après vérification de leur identité.
admin-settings-key-security-protection-adaptive-admins = Observer les administrateurs pour la protection adaptative
admin-settings-help-security-protection-adaptive-admins = Active cette dimension de sujet lorsque le mode adaptatif est actif. Les comptes et les administrateurs ne sont comptés qu'après vérification de leur identité.
admin-settings-key-security-protection-short-window-secs = Fenêtre d'anomalie courte (secondes)
admin-settings-help-security-protection-short-window-secs = Fenêtre d'observation glissante. Les longues fenêtres couvrent de quelques heures à trente jours ; l'arrondi des intervalles peut conserver au plus un soixante-quatrième de fenêtre supplémentaire.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protection-short-minimum-samples = Échantillons minimaux du ratio court
admin-settings-help-security-protection-short-minimum-samples = Nombre minimal de requêtes normales plus anormales observées avant l'application du déclencheur par ratio. Le seuil d'anomalies absolu est indépendant de ce minimum.
admin-settings-key-security-protection-short-absolute-anomalies = Budget d'anomalies absolu court
admin-settings-help-security-protection-short-absolute-anomalies = Se déclenche lorsque ce nombre d'anomalies sélectionnées s'accumule, même si des requêtes normales diluent le ratio. Zéro désactive ce déclencheur.
admin-settings-key-security-protection-short-ratio-percent = Ratio d'anomalies court (pourcentage)
admin-settings-help-security-protection-short-ratio-percent = Se déclenche lorsque les anomalies sélectionnées atteignent ce pourcentage après le nombre minimal d'échantillons. Le déclencheur absolu ou par ratio suffit. Zéro désactive ce déclencheur.
admin-settings-key-security-protection-long-window-secs = Fenêtre d'anomalie longue (secondes)
admin-settings-help-security-protection-long-window-secs = Fenêtre d'observation glissante. Les longues fenêtres couvrent de quelques heures à trente jours ; l'arrondi des intervalles peut conserver au plus un soixante-quatrième de fenêtre supplémentaire.
admin-settings-key-security-protection-long-minimum-samples = Échantillons minimaux du ratio long
admin-settings-help-security-protection-long-minimum-samples = Nombre minimal de requêtes normales plus anormales observées avant l'application du déclencheur par ratio. Le seuil d'anomalies absolu est indépendant de ce minimum.
admin-settings-key-security-protection-long-absolute-anomalies = Budget d'anomalies absolu long
admin-settings-help-security-protection-long-absolute-anomalies = Se déclenche lorsque ce nombre d'anomalies sélectionnées s'accumule, même si des requêtes normales diluent le ratio. Zéro désactive ce déclencheur.
admin-settings-key-security-protection-long-ratio-percent = Ratio d'anomalies long (pourcentage)
admin-settings-help-security-protection-long-ratio-percent = Se déclenche lorsque les anomalies sélectionnées atteignent ce pourcentage après le nombre minimal d'échantillons. Le déclencheur absolu ou par ratio suffit. Zéro désactive ce déclencheur.
admin-settings-key-security-protection-normal-cap-per-bucket = Plafond de référence normale par intervalle
admin-settings-help-security-protection-normal-cap-per-bucket = Borne la référence normale apprise dans chaque intervalle de la fenêtre glissante. Des budgets d'anomalies absolus indépendants empêchent un remplissage illimité de requêtes valides de masquer un abus.
admin-settings-key-security-protection-signals = Signaux de preuve adaptative
admin-settings-help-security-protection-signals = Sélectionnez les observations faisant foi : requêtes mal formées, trop volumineuses, identifiants refusés, autorisation refusée, délai de corps dépassé ou, en option, accès manqués aux routes d'administration. La détection de sondes sur les routes d'administration exclut les réponses « entité introuvable » et les ressources statiques ; les codes d'état génériques, les refus de capacité et les chaînes user-agent n'établissent pas un abus.
admin-settings-key-security-protection-throttle-secs = Ralentissement temporaire initial (secondes)
admin-settings-help-security-protection-throttle-secs = Le premier franchissement du seuil démarre une échéance de ralentissement fixe. Les requêtes pendant cette échéance ne la prolongent pas.
admin-settings-key-security-protection-block-secs = Blocage temporaire renforcé (secondes)
admin-settings-help-security-protection-block-secs = Un franchissement ultérieur du seuil avant la récupération peut escalader vers ce blocage borné. Aucun verrouillage permanent de compte n'est créé.
admin-settings-key-security-protection-decay-secs = Intervalle de récupération au calme (secondes)
admin-settings-help-security-protection-decay-secs = Après cet intervalle sans anomalie et l'expiration de l'action en cours, l'escalade revient à son stade initial. Les preuves glissantes expirent selon leurs fenêtres configurées.
admin-settings-key-security-protection-exempt-clients = Adresses et réseaux de clients exemptés
admin-settings-help-security-protection-exempt-clients = Des adresses IP exactes ou des réseaux CIDR n'exemptent que la dimension client. Les limites de compte vérifiées s'appliquent toujours. À utiliser délibérément pour les réseaux NAT partagés ; les adresses transmises ne sont fiables que via les proxys configurés.
admin-settings-key-security-protection-exempt-accounts = Identifiants de compte vérifiés exemptés
admin-settings-help-security-protection-exempt-accounts = Des identifiants exacts de comptes de boîte aux lettres vérifiés par le serveur n'exemptent que leur dimension de compte, jamais la dimension du client source. Les adresses e-mail et les paramètres de compte fournis par l'appelant ne sont pas des preuves d'identité.
admin-settings-key-security-protection-exempt-admins = Identifiants d'administrateur exemptés
admin-settings-help-security-protection-exempt-admins = Des identifiants exacts d'administrateurs authentifiés n'exemptent que leur dimension d'administrateur. Les verrouillages existants après échecs de connexion et les délais stricts de session d'administrateur restent inchangés.
admin-settings-key-security-protection-storage-failure = Politique en cas d'échec du stockage d'application
admin-settings-help-security-protection-storage-failure = Closed refuse le travail protégé lorsqu'un budget d'application activé ne peut pas être vérifié. Open l'autorise temporairement avec un avertissement opérationnel borné. Les échecs en mode observation seule n'imposent jamais d'action.
admin-settings-key-security-admin-probe-enabled = Protection contre les sondes d'administration
admin-settings-help-security-admin-probe-enabled = Compte les requêtes qui cherchent la console d'administration par réseau client (adresse IPv4, IPv6 /64), ralentit une source qui insiste, puis la bloque. Les fautes de frappe et les favoris périmés restent dans la tolérance. Modifié dans Sécurité > Protection adaptative.
admin-settings-key-security-admin-probe-grace-allowance = Tolérance (échecs par fenêtre)
admin-settings-help-security-admin-probe-grace-allowance = Échecs ignorés par fenêtre. Le score diminue en continu de ce nombre de points par fenêtre.
admin-settings-key-security-admin-probe-window-secs = Fenêtre de tolérance (secondes)
admin-settings-help-security-admin-probe-window-secs = La fenêtre à laquelle s'applique la tolérance.
admin-settings-key-security-admin-probe-block-after = Échecs au-delà de la tolérance avant blocage
admin-settings-help-security-admin-probe-block-after = Échecs au-dessus de la tolérance qui déclenchent un blocage. Les échecs intermédiaires reçoivent une réponse après le délai de tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Délai de tarpit (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Délai ajouté à chaque échec compté au-dessus de la tolérance. 0 désactive le tarpit. Les autres requêtes ne sont jamais retardées.
admin-settings-key-security-admin-probe-block-secs = Premier blocage (secondes)
admin-settings-help-security-admin-probe-block-secs = Durée du premier blocage. Une source bloquée reçoit la même réponse pour chaque chemin.
admin-settings-key-security-admin-probe-max-block-secs = Blocage le plus long (secondes)
admin-settings-help-security-admin-probe-max-block-secs = Un blocage répété double la durée précédente, jusqu'à ce plafond.
admin-settings-key-security-admin-probe-repeat-window-secs = Fenêtre de répétition (secondes)
admin-settings-help-security-admin-probe-repeat-window-secs = Un blocage qui commence dans ce délai après la fin du précédent compte comme une répétition.
admin-settings-key-security-admin-probe-block-status = État de la réponse de blocage
admin-settings-help-security-admin-probe-block-status = Statut HTTP envoyé à une source bloquée, avec Retry-After : 429 ou 403.
admin-settings-key-security-admin-probe-signals = Échecs comptés
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (un 404 à la profondeur de chemin de la console), admin-signin-failed et admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Chemins de sonde
admin-settings-help-security-admin-probe-probe-paths = Chemins que demandent les scanners, comme /wp-admin ou /.env. Un chemin qui en contient un en segments entiers compte comme un échec. Le propre chemin de la console jamais.
admin-settings-key-security-admin-probe-allowlist = Réseaux jamais bloqués
admin-settings-help-security-admin-probe-allowlist = Adresses IP ou réseaux CIDR (bureau, VPN) qui ne sont jamais comptés ni bloqués.
admin-settings-key-security-admin-probe-audit-ip = Adresse source dans le journal d'audit
admin-settings-help-security-admin-probe-audit-ip = masked affiche IPv4 /24 et IPv6 /48 ; full affiche l'adresse. La ligne fail2ban contient toujours l'adresse complète.
admin-settings-key-security-admin-probe-fail2ban-log = Ligne de journal fail2ban lors d'un blocage
admin-settings-help-security-admin-probe-fail2ban-log = Écrit une ligne mailwoman[admin-probe] avec rhost=<address> lorsqu'une source est bloquée.
admin-settings-key-appearance-custom-css-enabled = Autoriser le CSS d'apparence personnel
admin-settings-help-appearance-custom-css-enabled = Autorise des styles visuels restreints dans l'espace de travail du propriétaire. Désactivé par défaut ; les verrous d'apparence imposés de la boîte aux lettres suspendent ces styles.
admin-settings-key-appearance-custom-js-enabled = Autoriser les scripts d'apparence personnels
admin-settings-help-appearance-custom-js-enabled = Autorise des scripts isolés qui ne peuvent que demander des changements d'apparence. Désactivé par défaut ; les scripts ne peuvent accéder ni au courrier, ni aux données du compte, ni au DOM de l'application.
admin-settings-key-mail-modifier-selection = Sélection des messages avec les touches de modification
admin-settings-help-mail-modifier-selection = Autorise la sélection de plage avec Ctrl, Commande et Maj. Verrouillez cette valeur pour l'imposer ; les commandes de sélection visibles restent disponibles.
admin-settings-key-mail-drag-to-folder = Glisser les messages vers des dossiers
admin-settings-help-mail-drag-to-folder = Autorise le glissement de messages, lié au compte, vers de vrais dossiers. Cela n'accorde pas l'autorisation d'écriture sur la boîte aux lettres ; le serveur autorise toujours chaque déplacement.
admin-settings-key-admin-allow-last-superadmin-loss = Autoriser la perte du dernier super-administrateur
admin-settings-help-admin-allow-last-superadmin-loss = Désactivé par défaut. L'activer permet à des modifications ultérieures de retirer le dernier accès de super-administrateur. Un super-administrateur complet doit confirmer la conséquence par une authentification récente.
admin-settings-last-superadmin-title = Autoriser la perte de tout accès super-administrateur ?
admin-settings-last-superadmin-body = Des modifications de compte ultérieures peuvent faire que plus personne ne puisse administrer ce déploiement. La récupération exigerait une administration locale du déploiement. Saisissez ALLOW LAST SUPERADMIN LOSS pour activer cette exception.
admin-settings-last-superadmin-confirm = Autoriser la perte d'accès
admin-settings-key-mail-folder-counts = Compteurs de dossiers
admin-settings-help-mail-folder-counts = Choisissez à côté des dossiers : masqués, non lus, non lus seulement tant qu'il y a du courrier non lu, total, ou les deux compteurs. Définissez des valeurs par défaut globales, de domaine ou d'utilisateur, et verrouillez le choix si nécessaire. S'applique à chaque boîte aux lettres liée d'un compte.
admin-settings-key-mail-folder-count-scope = Dossiers avec compteurs
admin-settings-help-mail-folder-count-scope = Tous les dossiers, ou seulement les dossiers principaux (Boîte de réception, Brouillons, Envoyés, Archives, Courrier indésirable, Corbeille, Boîte d'envoi et autres dossiers à usage spécial). Avec les seuls dossiers principaux, les dossiers personnalisés et leurs sous-dossiers n'affichent aucun compteur. S'applique à chaque boîte aux lettres liée d'un compte.
admin-settings-key-mail-compact-folder-badges = Badges secondaires de dossiers compacts
admin-settings-help-mail-compact-folder-badges = Affiche des badges plus petits en dehors de la boîte de réception. Les utilisateurs peuvent choisir leur propre taille, sauf si ce paramètre est verrouillé.
admin-settings-key-mail-focused-inbox = Boîte de réception prioritaire
admin-settings-help-mail-focused-inbox = Divise la boîte de réception en onglets Prioritaires et Autres ; les lettres d'information et les expéditeurs automatiques vont dans Autres. Désactivé par défaut. Les utilisateurs peuvent l'activer ou le désactiver dans Paramètres > Dossiers et barre latérale, sauf si ce paramètre est verrouillé.
admin-settings-key-mail-account-unread-badge = Compteur de non-lus à côté du nom de la boîte aux lettres
admin-settings-help-mail-account-unread-badge = Affiche le nombre de messages non lus de la boîte de réception à côté de l'adresse ou du pseudonyme connecté dans la barre latérale. Désactivé par défaut. Les compteurs des autres boîtes aux lettres dans le menu des boîtes aux lettres sont toujours affichés. S'applique à chaque boîte aux lettres liée d'un compte.
admin-settings-key-privacy-remote-image-downloads = Téléchargements manuels d'images distantes
admin-settings-key-privacy-remote-images-toast = Notification de chargement des images
admin-settings-help-privacy-remote-images-toast = Affiche une notification dans le lecteur pendant que les images distantes d'un message se chargent via le proxy de confidentialité, et indique quand elles sont chargées ou combien n'ont pas pu l'être. Activé par défaut. Ne change que ce que le lecteur affiche ; les images chargées restent décidées par la politique de contenu distant.
admin-settings-help-privacy-remote-image-downloads = Autorise le téléchargement explicite d'une image référencée par le message du propriétaire via le proxy de confidentialité protégé. Désactivé par défaut ; cela n'ouvre jamais directement l'URL externe.
admin-settings-key-mail-resume-tabs = Reprendre les onglets de l'espace de travail
admin-settings-help-mail-resume-tabs = Restaure les références d'onglets propres au compte après un rechargement. La désactivation efface l'enregistrement local de l'espace de travail sans supprimer les brouillons.
admin-settings-key-metrics-default-range-seconds = Plage par défaut de l'historique des requêtes
admin-settings-help-metrics-default-range-seconds = Secondes affichées lorsqu'une requête d'historique ne précise pas de plage. Limité par la conservation et la plage maximale.
admin-settings-key-metrics-max-range-seconds = Plage maximale de l'historique des requêtes
admin-settings-help-metrics-max-range-seconds = Nombre maximal de secondes par requête d'historique. La conservation reste une limite supplémentaire.
admin-settings-key-contacts-favorites-first = Favoris en premier dans les suggestions
admin-settings-help-contacts-favorites-first = Liste les contacts favoris avant les autres correspondances dans les suggestions de destinataires. Seul l'ordre change ; aucune suggestion n'est masquée. Les remplacements par utilisateur et par domaine peuvent être verrouillés.
admin-settings-key-contacts-sender-icons = Icônes d'expéditeurs intégrées
admin-settings-help-contacts-sender-icons = Affiche des icônes intégrées localement pour les domaines d'expéditeurs reconnus, avec des initiales générées pour les autres. Les photos de contacts ont la priorité. Les icônes ne vérifient pas l'identité d'un expéditeur. Aucun service de logos externe n'est contacté ; les remplacements par utilisateur et par domaine peuvent être verrouillés.
admin-settings-key-mail-brand-avatars = Logos de marque à côté des expéditeurs vérifiés
admin-settings-help-mail-brand-avatars = Affiche un logo de marque intégré localement à côté de l'expéditeur dans le lecteur, uniquement lorsque le message a réussi DKIM et un contrôle DMARC aligné pour le domaine propre de cette marque. Les messages qui échouent à ces contrôles ou en sont dépourvus gardent l'avatar normal. Aucun logo n'est récupéré auprès de l'expéditeur ni d'un service de logos.
admin-settings-key-mail-draft-save-mode = Mode d'enregistrement des brouillons du compte
admin-settings-help-mail-draft-save-mode = Enregistrer après une pause de saisie, périodiquement, ou seulement à la demande. L'enregistrement exige un dossier Brouillons amont pris en charge ; les écritures incertaines ne sont jamais réessayées automatiquement.
admin-settings-key-mail-draft-debounce-ms = Pause de saisie des brouillons (millisecondes)
admin-settings-help-mail-draft-debounce-ms = Attend de 500 à 10000 millisecondes après la modification avant d'enregistrer en mode temps réel.
admin-settings-key-mail-draft-interval-secs = Intervalle d'enregistrement des brouillons (secondes)
admin-settings-help-mail-draft-interval-secs = Enregistre les brouillons modifiés toutes les 5 à 3600 secondes en mode périodique. Les brouillons inchangés ne sont pas réécrits.
admin-settings-key-login-remember-signins = Mémoriser les adresses de connexion réussies
admin-settings-help-login-remember-signins = Conserve sur cet appareil jusqu'à dix noms d'utilisateur ou adresses e-mail utilisés avec succès pour la liste déroulante de connexion. N'enregistre jamais de mots de passe ni de sessions. La désactivation supprime les adresses enregistrées de ce serveur lorsqu'un appareil charge ensuite la page de connexion ou termine une connexion ; elle ne contrôle pas le remplissage automatique du navigateur.
admin-settings-key-login-show-remembered-hint = Afficher l'explication des adresses enregistrées
admin-settings-help-login-show-remembered-hint = Affiche un texte explicatif dans la liste déroulante des connexions précédentes. Masqué par défaut. S'applique uniquement lorsque la mémorisation des connexions réussies est activée.
admin-settings-key-mail-show-from = Afficher le champ De de l'éditeur
admin-settings-help-mail-show-from = Affiche De par défaut dans les nouveaux éditeurs. Les utilisateurs peuvent l'afficher pour un message ; seules les adresses autorisées peuvent envoyer.
admin-settings-key-accounts-linking-enabled = Autoriser les comptes de boîtes aux lettres liées
admin-settings-help-accounts-linking-enabled = Désactivé par défaut. Une désactivation globale bloque tout accès lié ; les deux comptes doivent aussi respecter leur politique de domaine et de compte. La liaison exige l'authentification réelle du compte enfant et une preuve récente du compte principal.
admin-settings-key-accounts-use-linked = Utiliser les boîtes aux lettres liées
admin-settings-help-accounts-use-linked = Désactivation par l'utilisateur de l'accès lié. Ne peut pas annuler une désactivation administrative. Les données de la boîte aux lettres sont conservées lorsque l'accès est désactivé.
admin-settings-key-accounts-max-linked = Nombre maximal de boîtes aux lettres liées
admin-settings-help-accounts-max-linked = Nombre maximal de boîtes aux lettres enfants vérifiées par identité principale, de 1 à 16. Par défaut 5.
admin-settings-key-accounts-store-linked-credentials = Conserver l'accès aux boîtes aux lettres liées
admin-settings-help-accounts-store-linked-credentials = Conserve le mot de passe de chaque boîte aux lettres liée scellé sur le serveur afin que son accès se renouvelle de lui-même, d'une session et d'un redémarrage à l'autre. Jamais conservé pour les comptes à accès nul ou les boîtes aux lettres avec vérification en deux étapes. La désactivation efface les mots de passe enregistrés ; les liaisons se renouvellent alors en se reconnectant. Activé par défaut.
admin-settings-key-privacy-strip-mailer-headers = Supprimer les en-têtes d'agent d'envoi sortants
admin-settings-help-privacy-strip-mailer-headers = Supprime les en-têtes X-Mailer, User-Agent et X-MimeOLE de premier niveau des e-mails SMTP et EWS natifs. Les messages joints et les contenus de rapport restent intacts. Les envois via le proxy JMAP amont ne sont pas couverts.
admin-settings-key-images-trusted-hosts = Hôtes d'images de confiance
admin-settings-help-images-trusted-hosts = Jusqu'à 100 hôtes d'images, et leurs sous-domaines, dont les images distantes se chargent sans autorisation par message lorsque le contenu distant est réglé sur demander. Ne l'emporte jamais sur le mode blocage ; un hôte bloqué reste bloqué.
admin-settings-key-images-blocked-hosts = Hôtes d'images bloqués
admin-settings-help-images-blocked-hosts = Jusqu'à 100 hôtes d'images, et leurs sous-domaines, dont les images distantes ne se chargent dans aucun mode. L'emporte sur les hôtes de confiance et sur toute autorisation.
admin-settings-key-links-trusted-domains = Domaines de liens de confiance
admin-settings-help-links-trusted-domains = Jusqu'à 100 domaines exacts dont les liens s'ouvrent depuis Vérifier les liens sans l'étape de destination. Pas de jokers, de schémas, de ports ni d'adresses IP. Les liens en HTTP simple sont toujours vérifiés. Verrouillez ce paramètre pour empêcher les utilisateurs de modifier la liste.
admin-settings-key-links-trusted-subdomains = Faire confiance aux sous-domaines des domaines de liens de confiance
admin-settings-help-links-trusted-subdomains = Désactivé : seul le domaine exact est de confiance. Activé : les noms se terminant par .<domain> le sont aussi. La correspondance se fait sur des étiquettes entières, de sorte que example.com ne fait jamais confiance à evilexample.com.
admin-settings-key-cap-compose-rich-text = Rédaction en texte enrichi
admin-settings-help-cap-compose-rich-text = Indique si les utilisateurs peuvent écrire des messages mis en forme (HTML). Désactivé n'envoie que du texte brut.
admin-settings-key-cap-compose-attachments = Pièces jointes dans les nouveaux messages
admin-settings-help-cap-compose-attachments = Indique si les utilisateurs peuvent joindre des fichiers aux messages qu'ils écrivent. Les images intégrées au corps ne sont pas comptées.
admin-settings-key-cap-compose-share-links = Liens de partage de gros fichiers
admin-settings-help-cap-compose-share-links = Indique si les utilisateurs peuvent envoyer un gros fichier sous forme de lien de partage Nextcloud plutôt que de pièce jointe.
admin-settings-key-cap-compose-scheduled-send = Envoi programmé
admin-settings-help-cap-compose-scheduled-send = Indique si les utilisateurs peuvent programmer l'envoi d'un message pour plus tard.
admin-settings-key-cap-compose-read-receipts = Demander des accusés de lecture
admin-settings-help-cap-compose-read-receipts = Indique si les utilisateurs peuvent demander un accusé de lecture aux destinataires.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-cap-signatures = Signatures
admin-settings-help-cap-signatures = Indique si les utilisateurs peuvent créer et modifier leurs propres signatures.
admin-settings-key-cap-identities = Identités d'envoi
admin-settings-help-cap-identities = Indique si les utilisateurs peuvent ajouter et modifier les identités sous lesquelles ils envoient.
admin-settings-key-cap-keys = Clés OpenPGP et S/MIME
admin-settings-help-cap-keys = Indique si les utilisateurs disposent du module Clés et peuvent signer et chiffrer des messages.
admin-settings-key-cap-links-trusted = Modifier les domaines de liens de confiance
admin-settings-help-cap-links-trusted = Indique si les utilisateurs peuvent modifier leur propre liste de domaines de liens de confiance. La liste de l'administrateur s'applique toujours.
admin-settings-key-cap-zoom = Zoom du lecteur
admin-settings-help-cap-zoom = Indique si les utilisateurs peuvent zoomer le lecteur de messages.
admin-settings-key-cap-mail-focused-inbox = Boîte de réception prioritaire
admin-settings-help-cap-mail-focused-inbox = Indique si les utilisateurs peuvent activer les onglets Prioritaires et Autres de la boîte de réception.
admin-settings-key-cap-views-events = Vue Événements
admin-settings-help-cap-views-events = Indique si les utilisateurs disposent de la vue Événements pour le courrier de calendrier.
admin-settings-key-cap-views-attachments = Vue Pièces jointes
admin-settings-help-cap-views-attachments = Indique si les utilisateurs disposent de la vue Pièces jointes. Les messages restent accessibles par la recherche.
admin-settings-key-cap-search-operators = Opérateurs de recherche
admin-settings-help-cap-search-operators = Indique si les utilisateurs peuvent rechercher avec des opérateurs tels que from:, subject: ou has:attachment.
admin-settings-key-cap-saved-searches = Recherches enregistrées
admin-settings-help-cap-saved-searches = Indique si les utilisateurs peuvent enregistrer des recherches.
admin-settings-key-cap-vacation = Réponse automatique d'absence
admin-settings-help-cap-vacation = Indique si les utilisateurs peuvent définir une réponse d'absence automatique.
admin-settings-key-cap-labels = Étiquettes
admin-settings-help-cap-labels = Indique si les utilisateurs peuvent placer des étiquettes sur les messages.
admin-settings-key-cap-snooze = Report
admin-settings-help-cap-snooze = Indique si les utilisateurs peuvent reporter des messages. Mettre fin à un report reste possible.
admin-settings-key-cap-follow-up = Rappels de relance
admin-settings-help-cap-follow-up = Indique si les utilisateurs peuvent définir des rappels de relance sur les messages. En supprimer un reste possible.
admin-settings-key-cap-masked = Adresses masquées
admin-settings-help-cap-masked = Indique si les utilisateurs peuvent créer et gérer des adresses masquées.
admin-settings-key-cap-directory = Recherche dans l'annuaire
admin-settings-help-cap-directory = Indique si les utilisateurs peuvent rechercher dans l'annuaire de l'organisation.
admin-settings-key-cap-mcp = Accès MCP
admin-settings-help-cap-mcp = Indique si les clés et les jetons de l'utilisateur peuvent utiliser le point de terminaison MCP.
admin-settings-key-cap-appearance-custom = Personnalisation de l'apparence
admin-settings-help-cap-appearance-custom = Indique si les utilisateurs peuvent modifier leur thème, leur accent, leurs couleurs personnalisées et de badge, la densité et la police. Désactivé conserve l'apparence de l'administrateur.
admin-settings-key-cap-appearance-custom-code = CSS et scripts personnalisés
admin-settings-help-cap-appearance-custom-code = Indique si les utilisateurs peuvent enregistrer leur propre CSS et leurs scripts d'apparence. Les commutateurs de déploiement sous Apparence s'appliquent toujours au-dessus.
admin-settings-key-cap-avatars = Avatars de boîte aux lettres
admin-settings-help-cap-avatars = Indique si les utilisateurs peuvent modifier les avatars de boîte aux lettres.
admin-settings-key-cap-shortcuts = Raccourcis clavier personnalisés
admin-settings-help-cap-shortcuts = Indique si les utilisateurs peuvent modifier les raccourcis clavier. Les raccourcis par défaut continuent de fonctionner.
admin-settings-key-cap-analytics = Analyse du produit
admin-settings-help-cap-analytics = Indique si le consentement aux analyses est demandé aux utilisateurs et s'ils peuvent envoyer des événements d'utilisation.
admin-settings-key-cap-apps-native = Applications de bureau et mobiles
admin-settings-help-cap-apps-native = Indique si les utilisateurs peuvent utiliser les applications de bureau et mobiles.
admin-settings-key-access-protocols = Protocoles d'accès à la messagerie
admin-settings-help-access-protocols = Avec quels protocoles de serveur de messagerie ces utilisateurs peuvent se connecter : imap, pop3, ews. Les connexions via un serveur JMAP suivent ce serveur.
admin-settings-key-limit-linked-mailboxes = Boîtes aux lettres liées par utilisateur
admin-settings-help-limit-linked-mailboxes = Combien de boîtes aux lettres un utilisateur peut lier, en dessous du maximum du serveur. 0 signifie que seul le maximum du serveur s'applique.
admin-settings-key-login-show-account-help-notice = Afficher l'aide sur le compte à la connexion
admin-settings-help-login-show-account-help-notice = Affiche une courte explication après une connexion échouée. Masqué par défaut ; l'avis ne révèle pas si un compte existe ou est désactivé.
admin-settings-key-appearance-leetspeak = Style de texte leetspeak
admin-settings-help-appearance-leetspeak = Affiche les libellés de l'interface et le texte des messages en lecture seule en leetspeak. Le message d'origine et le texte modifiable restent inchangés. Les utilisateurs peuvent choisir le texte normal, sauf si ce paramètre est verrouillé.
admin-settings-key-mail-cache-enabled = Activer le cache de session
admin-settings-help-mail-cache-enabled = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-mailboxes = Mettre en cache les listes de messages des dossiers
admin-settings-help-mail-cache-mailboxes = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-messages = Mettre en cache les messages ouverts
admin-settings-help-mail-cache-messages = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-fresh-seconds = Intervalle de fraîcheur (secondes)
admin-settings-help-mail-cache-fresh-seconds = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-max-age-seconds = Ancienneté maximale du cache (secondes)
admin-settings-help-mail-cache-max-age-seconds = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-max-entries = Nombre maximal d'entrées en cache
admin-settings-help-mail-cache-max-entries = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-max-bytes = Budget mémoire (octets)
admin-settings-help-mail-cache-max-bytes = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-prefetch-enabled = Activer le préchargement spéculatif
admin-settings-help-mail-cache-prefetch-enabled = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-hover-enabled = Précharger au survol d'un dossier
admin-settings-help-mail-cache-hover-enabled = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-focus-enabled = Précharger lorsqu'un dossier reçoit le focus clavier
admin-settings-help-mail-cache-focus-enabled = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-dwell-ms = Délai du pointeur et du focus (millisecondes)
admin-settings-help-mail-cache-dwell-ms = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-idle-enabled = Précharger les dossiers pendant l'inactivité
admin-settings-help-mail-cache-idle-enabled = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-idle-delay-ms = Délai d'inactivité (millisecondes)
admin-settings-help-mail-cache-idle-delay-ms = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-max-concurrency = Nombre maximal de préchargements simultanés
admin-settings-help-mail-cache-max-concurrency = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-max-queued = Nombre maximal de préchargements en file
admin-settings-help-mail-cache-max-queued = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-preload-messages = Autoriser le préchargement spéculatif des corps de messages
admin-settings-help-mail-cache-preload-messages = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-excluded-folders = Dossiers exclus du préchargement
admin-settings-help-mail-cache-excluded-folders = Conserve les dossiers et messages récemment consultés dans la mémoire de cet onglet. Le contenu en cache est actualisé en arrière-plan. La fermeture de l'onglet ou la déconnexion l'efface. Le préchargement ne marque jamais le courrier comme lu et ne charge pas d'images distantes.
admin-settings-key-mail-cache-refresh-on-open = Actualiser le contenu en cache à l'ouverture
admin-settings-help-mail-cache-refresh-on-open = Actualiser le contenu en cache à l'ouverture
admin-settings-key-mail-templates-enabled = Modèles de réponse
admin-settings-help-mail-templates-enabled = Active les modèles de réponse en texte brut, personnels et partagés, dans l'éditeur. Les utilisateurs peuvent s'en désinscrire, sauf si ce paramètre est verrouillé.
admin-settings-key-mail-templates-allow-personal = Autoriser les modèles personnels
admin-settings-help-mail-templates-allow-personal = Permet aux utilisateurs de créer leurs propres modèles de réponse. Les modèles partagés sont gérés séparément par les administrateurs.
admin-settings-key-mail-templates-shared = Modèles de réponse partagés
admin-settings-help-mail-templates-shared = Gère des réponses réutilisables en texte brut pour cette portée. Les modèles ne définissent jamais de destinataires et n'envoient pas de messages.
admin-settings-key-mail-templates-personal = Modèles de réponse personnels
admin-settings-help-mail-templates-personal = Réponses réutilisables personnelles en texte brut. La création ou la modification exige que les modèles de réponse et les modèles personnels soient activés.
admin-settings-key-mail-compose-window = Fenêtre de l'éditeur
admin-settings-help-mail-compose-window = Ouvre les nouveaux brouillons dans un onglet de l'espace de travail ou une fenêtre séparée. Les fenêtres contextuelles bloquées se rabattent sur un onglet.
admin-settings-key-mail-reply-copy-to-original = Garder les réponses dans leur dossier d'origine
admin-settings-help-mail-reply-copy-to-original = Enregistre une copie de la réponse envoyée dans le dossier du message d'origine lorsque c'est pris en charge.
admin-settings-key-mail-compose-check-invalid-recipients = Vérifier les destinataires non valides avant l'envoi
admin-settings-help-mail-compose-check-invalid-recipients = Ce qui se passe lorsqu'un message est envoyé à une adresse non valide : autoriser, avertir, avertir et mémoriser le choix, ou refuser.
admin-settings-key-mail-compose-check-missing-attachment = Vérifier l'absence de pièce jointe avant l'envoi
admin-settings-help-mail-compose-check-missing-attachment = Ce qui se passe lorsque le texte mentionne une pièce jointe mais qu'aucune n'est jointe : autoriser, avertir, avertir et mémoriser le choix, ou refuser.
admin-settings-key-mail-compose-check-empty-subject = Vérifier l'absence d'objet avant l'envoi
admin-settings-help-mail-compose-check-empty-subject = Ce qui se passe lorsqu'un message n'a pas d'objet : autoriser, avertir, avertir et mémoriser le choix, ou refuser.
admin-settings-key-mail-compose-auto-chip-recipients = Terminer automatiquement les destinataires saisis
admin-settings-help-mail-compose-auto-chip-recipients = Transforme une adresse saisie en pastille de destinataire lorsqu'un espace, une virgule ou un point-virgule la suit.
admin-settings-key-mail-compose-header-collapse = En-tête de l'éditeur
admin-settings-help-mail-compose-header-collapse = Comportement des lignes À, Cc, Cci et Objet de l'éditeur : développées, repliées en un résumé d'une ligne, ou auto (repliées pendant la rédaction du corps, ouvertes lorsqu'un champ a le focus). Les utilisateurs peuvent les replier ou les déplier eux-mêmes, sauf si c'est verrouillé.
admin-settings-key-mail-compose-recipient-separator = Séparateur de destinataires
admin-settings-help-mail-compose-recipient-separator = Le séparateur inséré après un destinataire terminé : une virgule ou un point-virgule.
admin-settings-key-mail-reply-from-mode = Identité utilisée pour les réponses et les transferts
admin-settings-help-mail-reply-from-mode = L'identité depuis laquelle une réponse ou un transfert est envoyé : celle à laquelle l'original était adressé, l'identité par défaut, un sélecteur, ou celle actuellement sélectionnée.
admin-settings-key-mail-forward-mode = Format de transfert par défaut
admin-settings-help-mail-forward-mode = Transférer le texte dans le corps ou joindre le message d'origine sous forme de fichier .eml.
admin-settings-key-mail-show-go-to-top = Afficher le bouton Haut de page
admin-settings-help-mail-show-go-to-top = Affiche dans la liste des messages un bouton Haut de page qui remonte jusqu'au premier message. Désactivé par défaut.
admin-settings-key-mail-show-create-folder-button = Afficher le bouton Créer un dossier
admin-settings-help-mail-show-create-folder-button = Affiche dans la barre latérale, à côté de Rédiger, un bouton Créer un dossier avec icône seule. Désactivé par défaut.
admin-settings-key-security-protection-user-agent-mode = Mode d'observation du user-agent
admin-settings-help-security-protection-user-agent-mode = enforce par défaut : les en-têtes de robots reconnus sélectionnés sont bloqués immédiatement lorsque le blocage des robots dès la première requête est activé. observe consigne les décisions sans refus ; off désactive cette dimension. Les déclarations d'en-tête ne sont pas des identités vérifiées.
admin-settings-key-security-protection-user-agent-block-bots = Bloquer les robots reconnus dès la première requête
admin-settings-help-security-protection-user-agent-block-bots = true par défaut. Les en-têtes sélectionnés de robots de recherche, d'IA, d'automatisation et personnalisés déclenchent un blocage HTTP 429 temporaire dès la première requête en mode enforce. observe se contente de le consigner ; false rétablit les budgets de requêtes précédents. Les en-têtes absents ou inhabituels, la portée et les exceptions conservent leur comportement actuel.
admin-settings-key-security-protection-user-agent-categories = Catégories de user-agent
admin-settings-help-security-protection-user-agent-categories = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-security-protection-user-agent-weak-requests = Seuil de requêtes à signal faible
admin-settings-help-security-protection-user-agent-weak-requests = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protection-user-agent-crawler-requests = Seuil de requêtes de robots d'exploration
admin-settings-help-security-protection-user-agent-crawler-requests = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-security-protection-user-agent-window-secs = Fenêtre d'observation du user-agent (secondes)
admin-settings-help-security-protection-user-agent-window-secs = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-security-protection-user-agent-max-action = Réponse maximale au user-agent
admin-settings-help-security-protection-user-agent-max-action = Escalade maximale pour des preuves faibles ou de requêtes répétées, ralentissement par défaut. Ce plafond ne limite pas le blocage dès la première requête des en-têtes de robots reconnus sélectionnés.
admin-settings-key-security-protection-user-agent-scope = Portée de la politique user-agent
admin-settings-help-security-protection-user-agent-scope = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-security-protection-user-agent-adaptive = Observations adaptatives du user-agent
admin-settings-help-security-protection-user-agent-adaptive = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-security-protection-user-agent-patterns = Modèles de user-agent
admin-settings-help-security-protection-user-agent-patterns = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-security-protection-user-agent-exceptions = Exceptions de user-agent
admin-settings-help-security-protection-user-agent-exceptions = Configure cette partie de l'observation facultative du user-agent. Le texte user-agent fourni par le client est un signal faible, pas une preuve d'identité ou d'abus.
admin-settings-key-mail-reader-unsafe-html-mode = HTML de message potentiellement dangereux
admin-settings-help-mail-reader-unsafe-html-mode = ask propose par défaut un bouton HTML nettoyé par message. strict n'affiche que du texte sûr ; sanitized affiche automatiquement le HTML nettoyé. Le contenu actif et les ressources distantes automatiques restent bloqués. Verrouillez pour imposer cette politique.
admin-settings-key-reader-attachments-density = Densité des pièces jointes
admin-settings-help-reader-attachments-density = Boutons de fichier compacts par défaut, ou aperçus confortables. Contrôle la présentation lorsque les pièces jointes d'un message sont développées.
admin-settings-key-cap-compose-open-tracking = Autoriser le suivi d'ouverture des messages
admin-settings-help-cap-compose-open-tracking = Indique si ce compte peut enregistrer les ouvertures de messages. Exige une URL publique configurée ; les comptes désactivés ne peuvent ni créer ni lier de jetons de suivi.
admin-settings-key-mail-open-tracking = Suivre les ouvertures par défaut
admin-settings-help-mail-open-tracking = Choix de suivi par message par défaut pour les messages mis en forme éligibles. Les utilisateurs peuvent le modifier, sauf s'il est verrouillé ; la désactivation de l'autorisation de suivi l'emporte toujours.
admin-settings-key-cap-reading-original-html = Autoriser l'affichage du HTML d'origine
admin-settings-help-cap-reading-original-html = Indique si les utilisateurs peuvent choisir la vue HTML d'origine pour des messages individuels. Le moteur de rendu nettoyé par défaut et la sécurité du cadre isolé restent en place.
admin-settings-key-mail-reader-allow-original-html = Proposer la vue HTML d'origine
admin-settings-help-mail-reader-allow-original-html = Affiche l'option par message d'utiliser le HTML d'origine lorsque l'autorisation de lecture le permet. Désactivé par défaut ; cela ne change pas le moteur de rendu des messages par défaut.
admin-settings-key-privacy-tracker-override-allowed = Proposer le consentement aux traqueurs par message
admin-settings-help-privacy-tracker-override-allowed = Affiche l'action Autoriser les traqueurs dans la barre de confidentialité du message. Disponible par défaut, mais jamais active sans autorisation explicite. Une politique verrouillée de blocage des traqueurs interdit toujours cette dérogation.
admin-settings-key-privacy-max-concurrent-global = Récupérations d'images parallèles globales
admin-settings-help-privacy-max-concurrent-global = Nombre maximal de requêtes réseau d'images actives sur ce processus serveur. Plage 1–256 ; par défaut 128. Les limites par message et par compte s'appliquent toujours.
admin-settings-key-privacy-max-concurrent-pipeline = Pipelines de traitement d'images simultanés
admin-settings-help-privacy-max-concurrent-pipeline = Nombre maximal d'images admises ensemble à la récupération, au décodage et à la publication en cache. Plage 1–256 ; par défaut 128. Les plafonds réseau et de décodeur sont appliqués séparément.
admin-settings-key-privacy-max-concurrent-decodes = Décodeurs d'images parallèles
admin-settings-help-privacy-max-concurrent-decodes = Nombre maximal de décodages d'images exécutés en même temps. Plage 1–16 ; par défaut 10. Des valeurs plus élevées consomment plus de CPU et de mémoire.
admin-settings-key-privacy-max-queued-requests = Requêtes d'images en attente
admin-settings-help-privacy-max-queued-requests = Nombre maximal de requêtes en attente de capacité réseau ou de traitement pour les images. Plage 16–16384 ; par défaut 2048. Un débordement ou l'expiration en file reste réessayable.
admin-settings-key-privacy-rate-burst = Rafale autorisée de requêtes d'images
admin-settings-help-privacy-rate-burst = Jetons de récupération par compte initiaux et maximaux, réapprovisionnés au débit de requêtes d'images configuré. Plage 1–12000 ; par défaut 1024. Les succès en cache ne consomment pas de jetons.
admin-settings-key-privacy-image-rate-mode = Dimensionnement du débit des requêtes d'images
admin-settings-help-privacy-image-rate-mode = Automatique adapte la rafale et le réapprovisionnement par compte d'après la capacité des réplicas, la charge de la file et le nombre d'images du message, entre les planchers et plafonds ci-dessous. Manuel utilise exactement les paramètres de rafale autorisée et de débit de requêtes. Automatique par défaut. Les requêtes au-delà du quota attendent un jeton jusqu'à l'échéance de la file de récupération ; l'authentification et les autorisations sont vérifiées d'abord.
admin-settings-key-privacy-image-rate-burst-floor = Plancher de la rafale automatique
admin-settings-help-privacy-image-rate-burst-floor = Rafale la plus basse par compte en mode de débit automatique. Plage 1–12000 ; par défaut 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Plafond de la rafale automatique
admin-settings-help-privacy-image-rate-burst-ceiling = Rafale la plus haute par compte en mode de débit automatique. Plage 1–12000 ; par défaut 2048.
admin-settings-key-privacy-image-rate-refill-floor = Plancher du réapprovisionnement automatique
admin-settings-help-privacy-image-rate-refill-floor = Réapprovisionnement le plus bas par compte, en jetons de récupération par minute, en mode de débit automatique. Plage 10–60000 ; par défaut 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Plafond du réapprovisionnement automatique
admin-settings-help-privacy-image-rate-refill-ceiling = Réapprovisionnement le plus haut par compte, en jetons de récupération par minute, en mode de débit automatique. Plage 10–60000 ; par défaut 6000.
admin-settings-key-privacy-image-cache-mode = Dimensionnement du cache d'images
admin-settings-help-privacy-image-cache-mode = Automatique dimensionne le cache d'après la mémoire restante après les réservations du décodeur et du pipeline, et allonge la durée de vie tant que le taux de succès est élevé et que la mémoire est libre. Manuel utilise exactement les paramètres de taille du cache, de nombre d'entrées et de durée de vie. Automatique par défaut.
admin-settings-key-privacy-image-cache-bytes-floor = Plancher de taille du cache automatique
admin-settings-help-privacy-image-cache-bytes-floor = Plus petit cache d'images en octets en mode automatique. Plage 0–1073741824 ; par défaut 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Plafond de taille du cache automatique
admin-settings-help-privacy-image-cache-bytes-ceiling = Plus grand cache d'images en octets en mode automatique. Plage 0–1073741824 ; par défaut 268435456.
admin-settings-key-privacy-cache-entries = Limite d'entrées du cache d'images
admin-settings-help-privacy-cache-entries = Nombre maximal d'images en cache ; le nombre fixe en mode manuel et le plafond en mode automatique. Plage 16–65536 ; par défaut 4096.
admin-settings-key-privacy-cache-entry-bytes = Plus grande image en cache
admin-settings-help-privacy-cache-entry-bytes = Plus grande image réencodée conservée dans le cache, en octets. Les images plus grandes sont servies mais non conservées. Plage 65536–16777216 ; par défaut 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Plancher de durée de vie du cache automatique
admin-settings-help-privacy-image-cache-ttl-floor-secs = Plus courte durée de vie du cache en secondes en mode automatique, utilisée en cas de faible taux de succès ou de pression mémoire. Plage 0–86400 ; par défaut 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Plafond de durée de vie du cache automatique
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Plus longue durée de vie du cache en secondes en mode automatique, utilisée en cas de taux de succès élevé avec mémoire libre. Plage 0–86400 ; par défaut 21600.
admin-settings-key-privacy-cache-ttl-secs = Durée de vie des images réussies en cache
admin-settings-help-privacy-cache-ttl-secs = Secondes pendant lesquelles les images décodées restent réutilisables dans ce processus. Plage 0–86400 ; par défaut 3600. Zéro désactive la réutilisation des images réussies ; l'autorisation est toujours vérifiée d'abord.
admin-settings-key-privacy-image-latency-tracking = Suivi de la vitesse du proxy d'images
admin-settings-help-privacy-image-latency-tracking = Enregistre le temps que prend chaque image passée par le proxy (médiane, p95, p99 et le temps passé à attendre, récupérer et décoder) sous forme de compteurs dans des intervalles de temps fixes en mémoire. Aucune URL, aucun hôte ni compte n'est stocké. Activé par défaut ; l'historique est réinitialisé au redémarrage et propre à chaque processus serveur. Le désactiver arrête les nouveaux enregistrements ; la section Vitesse continue d'afficher ce qui a été enregistré.
admin-settings-key-privacy-negative-cache-ttl-secs = Durée de vie des échecs d'images en cache
admin-settings-help-privacy-negative-cache-ttl-secs = Secondes pendant lesquelles les échecs amont éligibles sont mémorisés pour éviter des requêtes répétées. Plage 0–300 ; par défaut 10. Zéro désactive la réutilisation des échecs. Une nouvelle tentative explicite efface les échecs concernés.
admin-settings-key-privacy-image-retry-attempts = Nouvelles tentatives automatiques d'images
admin-settings-help-privacy-image-retry-attempts = Nombre maximal de nouvelles tentatives pour des échecs temporaires d'images lors d'un chargement de message visible. Plage 0–8 ; par défaut 3. Zéro désactive les nouvelles tentatives automatiques tout en conservant la nouvelle tentative manuelle.
admin-settings-key-privacy-image-retry-delay-ms = Délai des nouvelles tentatives automatiques d'images
admin-settings-help-privacy-image-retry-delay-ms = Millisecondes entre les nouvelles tentatives automatiques d'images, une fois la trame actuelle stabilisée. Plage 500–60000 ; par défaut 5000. Les refus permanents ne sont pas réessayés automatiquement.
admin-settings-key-privacy-image-decode-timeout-ms = Délai de décodage d'image
admin-settings-help-privacy-image-decode-timeout-ms = Millisecondes en temps réel accordées à un décodage d'image en bac à sable après sa récupération. Plage 1000–60000 ; par défaut 10000. À augmenter pour les grandes images ; les délais réseau restent distincts.
admin-settings-key-privacy-image-concurrency-mode = Profil de concurrence des images
admin-settings-help-privacy-image-concurrency-mode = Automatique s'adapte à la capacité du CPU, à la mémoire du conteneur et au nombre d'images approuvées de chaque message. Faibles ressources, Équilibré et Débit élevé s'adaptent aussi, avec des plafonds de sécurité différents. Manuel utilise directement les plafonds numériques. L'application d'un profil ne change que ce paramètre.
admin-settings-cat-image-proxy = Proxy d'images
admin-settings-key-privacy-image-ram-max-mi-b = Maximum du niveau RAM
admin-settings-help-privacy-image-ram-max-mi-b = Borne supérieure en Mio pour le niveau d'images en mémoire, appliquée en plus de la taille de cache automatique ou manuelle. Zéro n'ajoute aucun plafond. Gardez-la petite et laissez le niveau disque porter l'essentiel. Plage 0–1048576 ; par défaut 0.
admin-settings-key-privacy-image-disk-enabled = Niveau disque des images
admin-settings-help-privacy-image-disk-enabled = Conserve sur disque, derrière le niveau RAM, les images déjà réencodées. Désactivé par défaut ; si le répertoire est absent, plein ou en lecture seule, le cache continue silencieusement en RAM seule. Seule la sortie du jail est stockée, sous des noms de fichier hachés.
admin-settings-key-privacy-image-disk-path = Répertoire disque des images
admin-settings-help-privacy-image-disk-path = Répertoire absolu utilisé pour le niveau disque. Vide utilise un dossier image-cache à côté du fichier de base de données. Montez-y un volume accessible en écriture lorsque le système de fichiers racine est en lecture seule. Les chemins relatifs et .. sont refusés et laissent le niveau désactivé.
admin-settings-key-privacy-image-disk-mode = Dimensionnement du disque des images
admin-settings-help-privacy-image-disk-mode = Automatique utilise un dixième de l'espace que ce cache pourrait utiliser (l'espace libre plus ce qu'il contient déjà), entre le plancher et le plafond et jamais plus de la moitié de cet espace. Manuel utilise exactement la taille maximale ci-dessous. Automatique par défaut.
admin-settings-key-privacy-image-disk-max-mi-b = Taille du disque des images (manuel)
admin-settings-help-privacy-image-disk-max-mi-b = Taille du niveau disque en Mio en mode manuel ; l'affichage montre la taille effective dans les deux cas. Plage 1–1048576 ; par défaut 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Plancher de taille du disque automatique
admin-settings-help-privacy-image-disk-floor-mi-b = Plus petit niveau disque en Mio en mode automatique ; également utilisé tant que l'espace libre est inconnu. Plage 0–1048576 ; par défaut 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Plafond de taille du disque automatique
admin-settings-help-privacy-image-disk-ceiling-mi-b = Plus grand niveau disque en Mio en mode automatique. Plage 0–1048576 ; par défaut 4096.
admin-settings-key-privacy-image-disk-entries = Limite d'entrées du disque
admin-settings-help-privacy-image-disk-entries = Nombre maximal d'images conservées sur disque. Chaque entrée coûte aussi une quantité fixe de RAM dans l'index, qui compte dans le niveau RAM. Plage 16–1048576 ; par défaut 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Plus grande image sur disque
admin-settings-help-privacy-image-disk-entry-bytes = Plus grande image réencodée écrite sur disque, en octets. Les images au-dessus de la limite d'entrée RAM mais dans celle-ci vont directement sur disque. Plage 65536–67108864 ; par défaut 8388608.
admin-settings-key-privacy-image-disk-min-hits = Succès avant une écriture sur disque
admin-settings-help-privacy-image-disk-min-hits = Succès dont une entrée RAM a besoin avant que son éviction ne l'écrive sur disque au lieu de l'abandonner. Zéro écrit chaque entrée évincée dont le score dépasse le seuil de rétrogradation. Plage 0–100 ; par défaut 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = File d'écriture sur disque
admin-settings-help-privacy-image-disk-write-queue-mi-b = Données d'image en Mio pouvant attendre l'écrivain disque en arrière-plan. Elles comptent dans le niveau RAM ; au-delà, les images évincées sont abandonnées au lieu d'être mises en file. Plage 1–256 ; par défaut 8.
admin-settings-key-privacy-image-ttl-step-secs = Pas de durée de vie par succès
admin-settings-help-privacy-image-ttl-step-secs = Secondes ajoutées à la durée de vie restante d'une image lors d'un succès, multipliées par 1 plus le logarithme de son nombre de succès amorti, de sorte que les images souvent utilisées vivent plus longtemps. Zéro maintient chaque image à la durée de vie de base. Plage 0–86400 ; par défaut 600.
admin-settings-key-privacy-image-ttl-max-secs = Durée de vie étendue maximale
admin-settings-help-privacy-image-ttl-max-secs = Plus longue durée de vie restante en secondes que les succès peuvent accumuler. Plage 60–2592000 ; par défaut 86400.
admin-settings-key-privacy-image-max-age-secs = Âge maximal des images
admin-settings-help-privacy-image-max-age-secs = Âge en secondes après lequel une image est récupérée de nouveau quels que soient ses succès. Le chemin de récupération ne conserve ni ETag ni Last-Modified amont, donc il n'y a pas de revalidation conditionnelle ; l'image est simplement récupérée de nouveau. Plage 3600–7776000 ; par défaut 604800.
admin-settings-key-privacy-image-score-half-life-secs = Demi-vie du score
admin-settings-help-privacy-image-score-half-life-secs = Secondes pour que le nombre de succès et la récence d'une image soient divisés par deux dans le score de priorité qui décide de la RAM, du disque et de l'éviction. Plage 60–604800 ; par défaut 3600.
admin-settings-key-privacy-image-score-demote-milli = Score de rétrogradation
admin-settings-help-privacy-image-score-demote-milli = Score, en millièmes, en dessous duquel une image en RAM est rétrogradée sur disque ou abandonnée. Le seuil de promotion est ce score plus l'hystérésis. Plage 0–10000 ; par défaut 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Hystérésis de promotion
admin-settings-help-privacy-image-score-hysteresis-percent = Pourcentage dont le seuil de promotion se situe au-dessus du seuil de rétrogradation. Un score entre les deux laisse l'image où elle est. Plage 0–500 ; par défaut 50.
admin-settings-key-privacy-image-context-open-percent = Poids du message ouvert
admin-settings-help-privacy-image-context-open-percent = Poids du score en pourcentage pour les images demandées pour un message. Les images d'un message comptent comme Récentes après dix minutes d'inactivité. Plage 100–1000 ; par défaut 200.
admin-settings-key-privacy-image-context-recent-percent = Poids du message récent
admin-settings-help-privacy-image-context-recent-percent = Poids du score en pourcentage pour les images utilisées récemment. Les requêtes qui ne nomment aucun message pèsent 100. Plage 100–1000 ; par défaut 150.
admin-settings-key-privacy-image-account-share-percent = Part de RAM par compte
admin-settings-help-privacy-image-account-share-percent = Plus grande part du niveau RAM que les insertions d'un compte peuvent remplir pendant que d'autres comptes utilisent le cache ; au-delà, le compte évince d'abord ses propres images au score le plus bas. Un compte seul peut tout utiliser. Plage 1–100 ; par défaut 50.
admin-settings-key-privacy-image-sweep-secs = Intervalle de balayage du cache
admin-settings-help-privacy-image-sweep-secs = Secondes entre les balayages en arrière-plan qui font expirer les images, rescorent un lot borné, rétrogradent les images froides en RAM sous pression et revérifient l'espace disque libre. Plage 5–3600 ; par défaut 30.
admin-settings-key-privacy-image-snapshot-secs = Intervalle des instantanés de l'index
admin-settings-help-privacy-image-snapshot-secs = Secondes entre les instantanés atomiques de l'index disque, qui conservent les nombres de succès et les durées de vie étendues d'un redémarrage à l'autre sans écriture à chaque succès. Zéro désactive les instantanés ; un redémarrage lit alors l'en-tête de chaque fichier. Plage 0–86400 ; par défaut 300.
admin-settings-key-privacy-image-sweep-batch = Taille du lot de balayage
admin-settings-help-privacy-image-sweep-batch = Entrées qu'un balayage de maintenance peut faire expirer, rescorer ou évincer. Borne le travail effectué sous le verrou du cache. Plage 16–8192 ; par défaut 256.
admin-image-proxy-title = Relevé du cache d'images
admin-image-proxy-unavailable = Le relevé du cache d'images n'est pas disponible.
admin-image-proxy-unknown = inconnu
admin-image-proxy-ram = Niveau RAM : { $used } utilisés sur { $budget } (configuré { $configured }), { $entries } images, { $hot } chaudes, { $cold } froides, taux de succès { $hit }.
admin-image-proxy-disk-off = Niveau disque : désactivé.
admin-image-proxy-disk = Niveau disque : { $bytes } utilisés sur { $budget } effectifs ({ $mode } ; plancher { $floor }, plafond { $ceiling }, manuel { $manual }), { $entries } images, { $hot } chaudes, { $cold } froides, taux de succès { $hit }.
admin-image-proxy-disk-inactive = Niveau disque : activé mais inactif. { $reason }
admin-image-proxy-moves = Promotions par minute : { $perMin }. Promotions totales { $promotions }, rétrogradations { $demotions }, abandons { $drops }. Entrées d'index : { $index }.

# -- Machine translation (2026-11): added to match en ---------------------
admin-image-proxy-refresh = Actualiser le relevé
admin-settings-key-security-trust-enabled = Niveaux de confiance
admin-settings-help-security-trust-enabled = Attribue un niveau à chaque requête : connecté, source connue (une connexion réussie récente depuis cette adresse) ou inconnue. Seuls les quotas de requêtes anonymes changent ; les identifiants, les seconds facteurs et les verrouillages ne sont jamais assouplis.
admin-settings-key-security-trust-known-ttl-days = Fenêtre de source connue (jours)
admin-settings-help-security-trust-known-ttl-days = Combien de temps une connexion réussie garde son adresse connue. Les tentatives échouées ne comptent jamais.
admin-settings-key-security-trust-known-capacity = Sources connues conservées
admin-settings-help-security-trust-known-capacity = Nombre maximal de sources connues conservées en mémoire (un hachage salé, un pays et un nombre de comptes pour chacune). La plus ancienne est supprimée en premier.
admin-settings-key-security-trust-persist = Enregistrer les sources connues entre les redémarrages
admin-settings-help-security-trust-persist = Enregistre la table des sources connues (hachages à clé, pays, dernière observation, nombre de comptes) dans le stockage des paramètres et la charge au démarrage. Les entrées plus anciennes que la fenêtre sont supprimées.
admin-settings-key-security-trust-persist-interval-secs = Intervalle d'enregistrement des sources connues (secondes)
admin-settings-help-security-trust-persist-interval-secs = Secondes entre les enregistrements, de 30 à 3600. Un enregistrement n'a lieu que si la table a changé, et une fois de plus à l'arrêt propre.
admin-settings-key-security-trust-unknown-percent = Quota des sources inconnues (%)
admin-settings-help-security-trust-unknown-percent = Quota de requêtes anonymes d'une source inconnue, en pourcentage des limites configurées. 100 conserve la référence ; plus bas est plus strict.
admin-settings-key-security-trust-known-percent = Quota des sources connues (%)
admin-settings-help-security-trust-known-percent = Quota de requêtes anonymes d'une source connue. 200 double les budgets de requêtes et divise par deux le délai de connexion et la difficulté du défi. Ne contourne jamais les identifiants, les seconds facteurs ni les verrouillages.
admin-settings-key-security-trust-ua-missing = Robots : user-agent vide
admin-settings-help-security-trust-ua-missing = Action pour les requêtes anonymes avec un user-agent vide : allow, limit ou block. Les sessions connectées ne sont jamais affectées.
admin-settings-key-security-trust-ua-uncommon = Robots : user-agent non reconnu
admin-settings-help-security-trust-ua-uncommon = Action pour les requêtes anonymes avec un user-agent non reconnu : allow, limit ou block.
admin-settings-key-security-trust-ua-automation = Robots : scripts et automatisation
admin-settings-help-security-trust-ua-automation = Action pour les requêtes anonymes se réclamant d'un script, d'une bibliothèque HTTP, d'un navigateur sans interface ou d'un robot générique : allow, limit ou block.
admin-settings-key-security-trust-ua-search = Robots : robots d'exploration de moteurs de recherche
admin-settings-help-security-trust-ua-search = Action pour les requêtes anonymes se réclamant d'un robot d'exploration de moteur de recherche : allow, limit ou block.
admin-settings-key-security-trust-ua-ai-training = Robots : robots d'entraînement d'IA
admin-settings-help-security-trust-ua-ai-training = Action pour les requêtes anonymes se réclamant de GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended ou Applebot-Extended : allow, limit ou block. Une déclaration est un en-tête, pas une identité vérifiée.
admin-settings-key-security-trust-ua-ai-search = Robots : robots de recherche d'IA
admin-settings-help-security-trust-ua-ai-search = Action pour les requêtes anonymes se réclamant d'OAI-SearchBot, PerplexityBot ou Claude-SearchBot : allow, limit ou block.
admin-settings-key-security-trust-ua-ai-user = Robots : récupérations d'IA déclenchées par un utilisateur
admin-settings-help-security-trust-ua-ai-user = Action pour les requêtes anonymes se réclamant de ChatGPT-User ou Claude-User : allow, limit ou block.
admin-settings-key-security-trust-ua-custom = Robots : modèles personnalisés
admin-settings-help-security-trust-ua-custom = Action pour les requêtes anonymes correspondant aux modèles de user-agent personnalisés : allow, limit ou block.
admin-settings-key-security-trust-admin-geo-enabled = Bloquer la découverte de l'administration depuis des pays inconnus
admin-settings-help-security-trust-admin-geo-enabled = Bloque une source qui cherche la console d'administration depuis un pays sans connexion réussie récente. Exige une base de pays lisible et la protection contre les sondes d'administration. MW_ADMIN_GEO_ENABLED=false la désactive pour la récupération.
admin-settings-key-security-trust-admin-geo-countries = Pays toujours connus (blocage admin)
admin-settings-help-security-trust-admin-geo-countries = Codes pays ISO en majuscules toujours considérés comme connus, en plus des pays des connexions réussies récentes.
admin-settings-key-security-trust-admin-geo-lookup-failure = Source sans pays
admin-settings-help-security-trust-admin-geo-lookup-failure = allow ne bloque pas et consigne le cas ; unknown traite une source sans pays détecté comme un pays inconnu.
admin-settings-key-security-trust-admin-geo-threshold = Sondes avant un blocage
admin-settings-help-security-trust-admin-geo-threshold = Sondes de découverte de l'administration depuis un pays inconnu, dans la fenêtre, qui déclenchent un blocage.
admin-settings-key-security-trust-admin-geo-window-secs = Fenêtre de sondes (secondes)
admin-settings-help-security-trust-admin-geo-window-secs = Fenêtre du seuil de sondes.
admin-settings-key-security-trust-admin-geo-block-on-first = Bloquer dès la première sonde
admin-settings-help-security-trust-admin-geo-block-on-first = Bloque dès la première sonde depuis un pays inconnu. Augmente le risque de verrouiller un administrateur en déplacement.
admin-settings-key-security-trust-admin-geo-block-secs = Premier blocage (secondes)
admin-settings-help-security-trust-admin-geo-block-secs = Durée du premier blocage.
admin-settings-key-security-trust-admin-geo-second-block-secs = Deuxième blocage (secondes)
admin-settings-help-security-trust-admin-geo-second-block-secs = Durée du deuxième blocage de la même source dans la période de répétition.
admin-settings-key-security-trust-admin-geo-max-block-secs = Blocage répété (secondes)
admin-settings-help-security-trust-admin-geo-max-block-secs = Durée du troisième blocage et de chaque blocage suivant dans la période de répétition.
admin-settings-key-security-trust-admin-geo-repeat-days = Période de répétition (jours)
admin-settings-help-security-trust-admin-geo-repeat-days = Un blocage survenant dans ce nombre de jours après le précédent compte comme une répétition et utilise la durée suivante, plus longue.
admin-settings-key-security-signin-account-geo-mode = Politique de pays de connexion par compte
admin-settings-help-security-signin-account-geo-mode = Désactivé par défaut. Bloque les connexions depuis des pays que le compte n'a pas utilisés auparavant, ou depuis tout pays hors de sa liste autorisée. Les IP et pays de la liste d'autorisation passent toujours. S'applique uniquement aux nouvelles connexions webmail, jamais aux sessions existantes ni aux clients de protocole. La détection du pays repose sur une base de géolocalisation d'IP qui peut être inexacte ou périmée ; les VPN, les opérateurs mobiles, les proxys et les déplacements peuvent bloquer une connexion légitime ou laisser passer une connexion étrangère. Ajoutez les IP ou pays de confiance à la liste d'autorisation.
admin-settings-key-security-signin-account-geo-allow-countries = Pays autorisés du compte
admin-settings-help-security-signin-account-geo-allow-countries = Codes pays ISO en majuscules toujours autorisés pour le compte. La détection du pays repose sur une base de géolocalisation d'IP qui peut être inexacte ou périmée ; les VPN, les opérateurs mobiles, les proxys et les déplacements peuvent bloquer une connexion légitime ou laisser passer une connexion étrangère. Ajoutez les IP ou pays de confiance à la liste d'autorisation.
admin-settings-key-security-signin-account-geo-allow-ips = IP autorisées du compte
admin-settings-help-security-signin-account-geo-allow-ips = Adresses IP ou réseaux CIDR toujours autorisés pour le compte. Une adresse IPv6 seule désigne son /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Pays autorisés pour tous les comptes
admin-settings-help-security-signin-account-geo-global-allow-countries = Codes pays ISO en majuscules autorisés pour tous les comptes, ajoutés à la liste propre à chaque compte.
admin-settings-key-security-signin-account-geo-global-allow-ips = IP autorisées pour tous les comptes
admin-settings-help-security-signin-account-geo-global-allow-ips = Adresses IP ou réseaux CIDR (un bureau ou un VPN) autorisés pour tous les comptes, ajoutés à la liste propre à chaque compte.
admin-settings-key-security-signin-account-geo-lookup-failure = Source sans pays (politique de compte)
admin-settings-help-security-signin-account-geo-lookup-failure = Lorsque la source n'a pas de pays détecté : autoriser la connexion et la consigner, ou la refuser.
admin-settings-key-security-signin-account-geo-history-days = Historique des pays du compte (jours)
admin-settings-help-security-signin-account-geo-history-days = Combien de jours un pays reste dans l'historique de connexion propre à un compte pour la politique des pays inhabituels. Il est conservé à part du journal d'authentification, donc raccourcir le journal ne le raccourcit pas.
admin-settings-key-security-geo-no-database = Aucune base de pays : comportement par défaut
admin-settings-help-security-geo-no-database = Ce que font les fonctions liées aux pays lorsqu'aucune base n'est chargée : autoriser, traiter la source comme un pays inconnu, ou bloquer (block ne s'applique qu'aux connexions). Le paramètre propre à une fonction s'applique aussi et le plus strict l'emporte.
admin-settings-key-security-geo-not-found = Adresse absente de la base : comportement par défaut
admin-settings-help-security-geo-not-found = Ce que font les fonctions liées aux pays lorsqu'une base est chargée mais n'a aucune entrée pour l'adresse. Les adresses privées, de bouclage et de lien local sont des réseaux locaux et ne sont jamais concernées.
admin-settings-key-security-signin-geo-no-database = Politique de connexion de l'instance sans base
admin-settings-help-security-signin-geo-no-database = Lorsque la politique de pays de connexion de l'instance est active et qu'aucune base n'est chargée : bloquer les connexions (fail closed) ou les autoriser sans contrôle. Les réseaux locaux sont toujours autorisés.
admin-settings-key-security-geo-session-lookup-failure = Alertes de session sans pays
admin-settings-help-security-geo-session-lookup-failure = Lorsque le pays d'une requête connectée ne peut pas être déterminé : l'ignorer, ou émettre une alerte de session par session et par jour. Cela ne bloque ni ne révoque jamais une session.
admin-settings-key-security-protocol-framing-mode = Garde de protocole : délimitation des requêtes
admin-settings-help-security-protocol-framing-mode = Classe de request smuggling : Content-Length avec Transfer-Encoding, Transfer-Encoding autre que exactement chunked (ou sur HTTP/1.0 et HTTP/2), Content-Length non numérique ou contradictoire. enforce répond 400 et ferme la connexion ; detect ne fait que compter. Une exemption par préfixe de route ne s'applique jamais à cette classe.
admin-settings-key-security-protocol-host-mode = Garde de protocole : syntaxe de Host
admin-settings-help-security-protocol-host-mode = Syntaxe de l'en-tête Host : absent en HTTP/1.1, vide, répété, ou contenant des informations d'utilisateur, des espaces ou des caractères de chemin. Les littéraux IPv6, les ports et un point final sont acceptés. enforce répond 400 et ferme la connexion.
admin-settings-key-security-protocol-host-pinning-mode = Garde de protocole : Host doit correspondre au nom public
admin-settings-help-security-protocol-host-pinning-mode = Host ne correspondant ni à l'hôte de MW_PUBLIC_URL ni à un élément de allowedHosts. Sans effet tant que ni l'un ni l'autre ne désigne un hôte. detect par défaut car des proxys et des contrôles de santé peuvent envoyer un nom interne.
admin-settings-key-security-protocol-path-control-mode = Garde de protocole : caractères de contrôle dans le chemin
admin-settings-help-security-protocol-path-control-mode = NUL, caractères de contrôle ou CR/LF encodés (%00, %0d, %0a) dans le chemin de la requête. enforce répond 400. Jamais exempté par un préfixe de route.
admin-settings-key-security-protocol-target-mode = Garde de protocole : ruses de cible de requête
admin-settings-help-security-protocol-target-mode = Encodage en pourcentage double ou excessif des points et des barres obliques, points encodés, segments de points, segments de chemin vides (//), barres obliques inverses, octets non ASCII bruts, et cible de forme absolue désignant une autre autorité que Host.
admin-settings-key-security-protocol-path-length-mode = Garde de protocole : longueur du chemin et de la requête
admin-settings-help-security-protocol-path-length-mode = Chemin ou requête plus longs que maxPathBytes ou maxQueryBytes. enforce répond 414.
admin-settings-key-security-protocol-method-mode = Garde de protocole : méthodes HTTP
admin-settings-help-security-protocol-method-mode = Méthode hors de allowedMethods (TRACE, TRACK, CONNECT et méthodes inconnues ou en minuscules par défaut). enforce répond 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Garde de protocole : en-têtes à valeur unique répétés
admin-settings-help-security-protocol-duplicate-headers-mode = Un en-tête qui ne devrait apparaître qu'une fois apparaît plusieurs fois (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, en-têtes conditionnels et de plage), un Content-Length répété avec des valeurs égales, ou des en-têtes Cookie répétés en HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Garde de protocole : en-têtes de transfert de pairs non fiables
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* ou X-Real-IP envoyés par un pair hors de MW_TRUSTED_PROXIES. Les en-têtes ne sont jamais crus dans les deux cas ; cela consigne la tentative. Elle ne compte pour la protection adaptative que si des proxys de confiance sont configurés.
admin-settings-key-security-protocol-header-hygiene-mode = Garde de protocole : hygiène des en-têtes
admin-settings-help-security-protocol-header-hygiene-mode = Nombre d'en-têtes, une valeur unique ou l'en-tête Cookie au-delà des plafonds configurés ; octets non ASCII dans les en-têtes ; Upgrade ou Connection: upgrade hors de la route WebSocket ; Connection nommant des en-têtes tels que Content-Length ou Host. enforce répond 431 pour les constats de taille et de nombre et 400 pour les autres.
admin-settings-key-security-protocol-body-mode = Garde de protocole : délimitation du corps de la requête
admin-settings-help-security-protocol-body-mode = GET ou HEAD portant un corps, Content-Length supérieur à maxBodyBytes (413), Expect autre que 100-continue (417) ou sans corps.
admin-settings-key-security-protocol-max-path-bytes = Garde de protocole : chemin le plus long (octets)
admin-settings-help-security-protocol-max-path-bytes = Chemin de requête le plus long avant que la classe de longueur de chemin ne le signale.
admin-settings-key-security-protocol-max-query-bytes = Garde de protocole : requête la plus longue (octets)
admin-settings-help-security-protocol-max-query-bytes = Chaîne de requête la plus longue avant que la classe de longueur de chemin ne la signale.
admin-settings-key-security-protocol-max-header-count = Garde de protocole : nombre maximal d'en-têtes
admin-settings-help-security-protocol-max-header-count = Nombre maximal de champs d'en-tête dans une requête avant que la classe d'hygiène des en-têtes ne le signale. La limite stricte propre à l'écouteur (MW_HTTP_MAX_HEADERS) s'applique toujours.
admin-settings-key-security-protocol-max-header-value-bytes = Garde de protocole : valeur d'en-tête la plus longue (octets)
admin-settings-help-security-protocol-max-header-value-bytes = Valeur d'en-tête unique la plus longue avant que la classe d'hygiène des en-têtes ne la signale.
admin-settings-key-security-protocol-max-cookie-bytes = Garde de protocole : plus grand en-tête Cookie (octets)
admin-settings-help-security-protocol-max-cookie-bytes = Taille combinée des en-têtes Cookie avant que la classe d'hygiène des en-têtes ne la signale.
admin-settings-key-security-protocol-max-body-bytes = Garde de protocole : plus grand corps déclaré (octets)
admin-settings-help-security-protocol-max-body-bytes = Plus grand Content-Length que la classe de corps accepte sans le signaler. Gardez-le au-dessus du plus gros envoi ou import qu'une route accepte ; une exemption par préfixe de route peut le relever pour une route.
admin-settings-key-security-protocol-allowed-methods = Garde de protocole : méthodes autorisées
admin-settings-help-security-protocol-allowed-methods = Méthodes qu'accepte la classe de méthode. La valeur par défaut liste les verbes HTTP ordinaires et ceux de WebDAV, CalDAV et CardDAV ; le serveur ne monte aujourd'hui aucune route DAV entrante, ils aboutissent donc à un 405 au routeur.
admin-settings-key-security-protocol-allowed-hosts = Garde de protocole : hôtes acceptés supplémentaires
admin-settings-help-security-protocol-allowed-hosts = Noms d'hôte acceptés par la classe d'épinglage de Host en plus de l'hôte de MW_PUBLIC_URL, par exemple un nom interne qu'utilise un proxy ou un contrôle de santé. Les ports et un point final sont ignorés.
admin-settings-key-security-protocol-exempt-prefixes = Garde de protocole : préfixes de route exemptés
admin-settings-help-security-protocol-exempt-prefixes = Préfixes de route (chacun commençant par / et de plus d'un caractère) pour lesquels les classes heuristiques ne s'exécutent pas, pour un client légitime qui en a besoin, par exemple une route d'envoi volumineux. Correspond à des segments de chemin entiers. Les constats de délimitation et de contrôle du chemin ne sont jamais exemptés, et / est ignoré.
admin-settings-key-security-health-mode = Accès aux points de terminaison de santé
admin-settings-help-security-health-mode = Qui peut atteindre /healthz et /readyz. internal-only : adresses client de bouclage, privées, de lien local et locales uniques. allowlist : celles-ci plus les réseaux ci-dessous. open : tout le monde. Tout autre reçoit le même 404 qu'un chemin inconnu. Une requête avec le jeton MW_HEALTH_TOKEN est toujours admise. L'adresse client est celle résolue via les proxys de confiance : derrière un proxy inverse sans MW_FORWARDED_MODE ni MW_TRUSTED_PROXIES, chaque requête semble provenir de l'adresse privée du proxy, si bien que tout Internet est admis.
admin-settings-key-security-health-networks = Réseaux des points de terminaison de santé
admin-settings-help-security-health-networks = Adresses IP ou réseaux CIDR pouvant atteindre /healthz et /readyz en mode allowlist, en plus des plages internes, par exemple un moniteur de disponibilité externe.
admin-settings-key-security-spray-enabled = Détection du password spraying
admin-settings-help-security-spray-enabled = Compte les noms d'utilisateur distincts avec lesquels une source échoue. Seul un hachage à clé de chaque nom d'utilisateur est conservé, en mémoire.
admin-settings-key-security-spray-window-secs = Fenêtre de spraying (secondes)
admin-settings-help-security-spray-window-secs = Fenêtre dans laquelle les échecs de connexion d'une source sont comptés par nom d'utilisateur distinct. Par défaut 900.
admin-settings-key-security-spray-challenge-at = Spraying : seuil de défi
admin-settings-help-security-spray-challenge-at = Noms d'utilisateur distincts en échec depuis une source, dans la fenêtre, qui imposent le défi de preuve de travail pour cette source. Les connexions réussies ne comptent jamais, de sorte qu'une adresse partagée de bureau ou de campus n'est pas affectée. Par défaut 10.
admin-settings-key-security-spray-pace-at = Spraying : seuil de ralentissement
admin-settings-help-security-spray-pace-at = Noms d'utilisateur distincts en échec qui ajoutent une attente supplémentaire entre les tentatives de la source. Par défaut 15.
admin-settings-key-security-spray-lock-at = Spraying : seuil de verrouillage
admin-settings-help-security-spray-lock-at = Noms d'utilisateur distincts en échec qui verrouillent la source pour la durée de verrouillage, via le verrouillage de connexion existant. Par défaut 25.
admin-settings-key-security-spray-pace-secs = Spraying : attente de ralentissement (secondes)
admin-settings-help-security-spray-pace-secs = Secondes entre les tentatives d'une source au seuil de ralentissement ou au-dessus. Par défaut 10.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-spray-lock-secs = Spraying : durée de verrouillage (secondes)
admin-settings-help-security-spray-lock-secs = Durée du verrouillage temporaire. Lorsqu'un administrateur déverrouille la source, son compteur est aussi réinitialisé. Par défaut 900.
admin-settings-key-security-spray-known-multiplier = Spraying : multiplicateur des sources connues
admin-settings-help-security-spray-known-multiplier = Multiplicateur appliqué aux trois seuils pour une source connue, c'est-à-dire avec une connexion réussie récente. Par défaut 3.
admin-settings-key-security-spray-capacity = Spraying : sources suivies
admin-settings-help-security-spray-capacity = Nombre maximal de sources suivies en mémoire ; les moins récemment actives sont supprimées en premier. Par défaut 20000.
admin-settings-key-security-spike-enabled = Détection de pics à l'échelle de l'instance
admin-settings-help-security-spike-enabled = Apprend une référence des connexions échouées et passe en mode renforcé lorsque l'ensemble de l'instance la dépasse largement. Ne verrouille jamais l'instance.
admin-settings-key-security-spike-multiplier-percent = Pic : multiplicateur (%)
admin-settings-help-security-spike-multiplier-percent = Un signal compte lorsqu'il atteint au moins ce pourcentage de sa référence apprise (400 = quatre fois) et dépasse son plancher absolu.
admin-settings-key-security-spike-min-failures-per-min = Pic : échecs minimaux par minute
admin-settings-help-security-spike-min-failures-per-min = Plancher : connexions échouées par minute en dessous desquelles le signal du taux d'échec ne compte jamais. Par défaut 20.
admin-settings-key-security-spike-min-sources = Pic : sources en échec minimales
admin-settings-help-security-spike-min-sources = Plancher : sources en échec distinctes dans un intervalle de cinq minutes en dessous desquelles ce signal ne compte jamais. Par défaut 10.
admin-settings-key-security-spike-min-usernames = Pic : noms d'utilisateur en échec minimaux
admin-settings-help-security-spike-min-usernames = Plancher : noms d'utilisateur en échec distincts dans un intervalle de cinq minutes en dessous desquels ce signal ne compte jamais. Par défaut 30.
admin-settings-key-security-spike-signals-required = Pic : signaux requis
admin-settings-help-security-spike-signals-required = Signaux qui doivent dépasser ensemble leur référence et leur plancher. La signature d'un spraying distribué compte pour deux. Par défaut 2.
admin-settings-key-security-spike-warmup-hours = Pic : préchauffage (heures)
admin-settings-help-security-spike-warmup-hours = Heures de trafic observé avant que le mode renforcé puisse démarrer. D'ici là, la référence ne fait qu'apprendre. Par défaut 48.
admin-settings-key-security-spike-adapt-hours = Pic : adaptation de la référence (heures)
admin-settings-help-security-spike-adapt-hours = Constante de temps de la référence. Plus long absorbe la croissance plus lentement. Un pic ne met jamais à jour la référence. Par défaut 168.
admin-settings-key-security-spike-duration-secs = Pic : durée du mode renforcé (secondes)
admin-settings-help-security-spike-duration-secs = Durée du mode renforcé après la dernière preuve d'un pic ; il est prolongé tant que les signaux restent au-dessus de la moitié de leur niveau de déclenchement. Par défaut 1800.
admin-settings-key-security-spike-force-challenge = Pic : imposer le défi
admin-settings-help-security-spike-force-challenge = En mode renforcé, exige le défi de preuve de travail pour les connexions anonymes depuis des sources inconnues. Les utilisateurs authentifiés et les sources connues ne sont pas affectés.
admin-settings-key-security-spike-pace-factor-percent = Pic : attente de connexion (%)
admin-settings-help-security-spike-pace-factor-percent = En mode renforcé, l'attente de connexion pour les sources inconnues en pourcentage de l'attente habituelle. Par défaut 300.
admin-settings-key-security-session-binding-enabled = Liaison de session
admin-settings-help-security-session-binding-enabled = Lie chaque session de navigateur à des hachages à clé de ses attributs de client. Les clients natifs avec jeton bearer, les clés d'API et les clients de protocole sont exemptés.
admin-settings-key-security-session-binding-attributes = Liaison de session : attributs
admin-settings-help-security-session-binding-attributes = Attributs liés : browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). network est désactivé par défaut car les réseaux mobiles changent. Les noms inconnus sont ignorés.
admin-settings-key-security-session-binding-strictness = Liaison de session : rigueur
admin-settings-help-security-session-binding-strictness = tolerant accepte les mises à jour du navigateur et un changement isolé d'un attribut souple ; strict signale toute différence souple.
admin-settings-key-security-session-binding-webmail-action = Liaison de session : action webmail
admin-settings-help-security-session-binding-webmail-action = Ce que fait une discordance sur une session webmail : off, detect (consigner seulement), step-up (redemander le mot de passe et le second facteur) ou révoquer la session. Par défaut step-up.
admin-settings-key-security-session-binding-admin-action = Liaison de session : action de la console d'administration
admin-settings-help-security-session-binding-admin-action = Ce que fait une discordance sur une session de la console d'administration : off, detect, step-up ou revoke. Par défaut revoke.
admin-settings-key-security-theft-enabled = Détection de vol de session
admin-settings-help-security-theft-enabled = Détecte la réutilisation d'un identifiant de session renouvelé, les déplacements impossibles et l'utilisation simultanée depuis différents pays.
admin-settings-key-security-theft-rotation-grace-secs = Vol : tolérance de rotation (secondes)
admin-settings-help-security-theft-rotation-grace-secs = Après la rotation d'une session, l'ancien identifiant peut être présenté de nouveau pendant ce délai (une requête en cours) avant de compter comme un vol. Par défaut 30.
admin-settings-key-security-theft-min-travel-secs = Vol : durée de trajet minimale (secondes)
admin-settings-help-security-theft-min-travel-secs = Un changement de pays plus rapide que cela après la requête précédente est traité comme un déplacement impossible. La détection du pays peut être inexacte ; les VPN et les réseaux mobiles peuvent provoquer de faux changements. Par défaut 3600.
admin-settings-key-security-theft-concurrent-use = Vol : utilisation simultanée
admin-settings-help-security-theft-concurrent-use = Signale une session utilisée alternativement depuis différents pays (ou réseaux) en cinq minutes : off, country ou network. La détection du pays peut être inexacte ; les VPN et les réseaux mobiles peuvent provoquer de faux changements.
admin-settings-key-security-theft-webmail-action = Vol : réponse pour le webmail
admin-settings-help-security-theft-webmail-action = Réponse à un déplacement impossible ou à une utilisation simultanée sur une session webmail : alert, step-up ou revoke. La réutilisation d'une session renouvelée révoque toujours. Par défaut step-up.
admin-settings-key-security-theft-admin-action = Vol : réponse pour la console d'administration
admin-settings-help-security-theft-admin-action = Réponse à un déplacement impossible ou à une utilisation simultanée sur une session de la console d'administration : alert, step-up ou revoke. Par défaut revoke.
admin-settings-key-security-theft-retention-days = Vol : conservation des enregistrements (jours)
admin-settings-help-security-theft-retention-days = Combien de temps les marqueurs de sessions renouvelées et les enregistrements de sécurité par session sont conservés ; réglez au moins la durée de vie de session la plus longue. Par défaut 15.
admin-settings-key-security-alerts-country-change = Alerte de changement de pays
admin-settings-help-security-alerts-country-change = Avertit lorsqu'une session connectée est utilisée depuis un pays autre que celui où elle a démarré. La détection du pays peut être inexacte ; les VPN et les réseaux mobiles peuvent provoquer de faux changements. Les utilisateurs peuvent la modifier, sauf si elle est verrouillée ; la désactivation par un utilisateur n'est respectée que si la mise en sourdine est autorisée.
admin-settings-key-security-alerts-country-change-enabled = Alerte de changement de pays : interrupteur global
admin-settings-help-security-alerts-country-change-enabled = Désactivé désactive l'alerte de changement de pays pour tout le monde. La détection du pays peut être inexacte ; les VPN et les réseaux mobiles peuvent provoquer de faux changements.
admin-settings-key-security-alerts-country-change-cooldown-secs = Alerte de changement de pays : délai d'attente (secondes)
admin-settings-help-security-alerts-country-change-cooldown-secs = Durée minimale entre les alertes de changement de pays d'une même session. Chaque nouveau pays déclenche une alerte une fois par session. Par défaut 300.
admin-settings-key-security-alerts-country-change-user-mute = Alerte de changement de pays : autoriser la mise en sourdine par les utilisateurs
admin-settings-help-security-alerts-country-change-user-mute = Indique si un utilisateur peut désactiver sa propre alerte de changement de pays. Désactivé, le serveur continue d'alerter même si le paramètre propre de l'utilisateur est désactivé.
admin-settings-key-security-alerts-channels = Canaux d'alerte de sécurité
admin-settings-help-security-alerts-channels = Comment les avertissements de sécurité de session parviennent à l'utilisateur : dans l'application, sous forme de message dans sa propre boîte aux lettres, ou les deux. Les utilisateurs peuvent le modifier, sauf si c'est verrouillé.
admin-settings-key-security-alerts-email-enabled = Alertes de sécurité par e-mail
admin-settings-help-security-alerts-email-enabled = Autorise la remise des avertissements de sécurité de session dans la propre boîte aux lettres de l'utilisateur. Aucun message n'est envoyé par SMTP ; sans session de remise, l'avertissement reste dans l'application et le journal d'audit.
admin-settings-key-security-alerts-sensitivity = Sensibilité des alertes de sécurité
admin-settings-help-security-alerts-sensitivity = low n'avertit que pour les constats de localisation et ne déclenche jamais de step-up ni de révocation ; normal suit les actions de l'administrateur ; high double l'intervalle de déplacement et vérifie aussi les réseaux simultanés. Les utilisateurs peuvent le modifier, sauf si c'est verrouillé.
admin-settings-key-security-alerts-notice-retention-days = Avis de sécurité : conservation (jours)
admin-settings-help-security-alerts-notice-retention-days = Combien de temps les avis de sécurité dans l'application sont conservés par compte (20 au plus par compte). Par défaut 30.
admin-settings-inline-effective = En vigueur :
admin-settings-inline-reset = Annuler les modifications
admin-settings-inline-domains = Valeurs par domaine
admin-attachments-tab-lists = Listes d'autorisation et de blocage
admin-attachments-tab-limits = Limites
admin-attachments-tab-delivery = Téléchargement et aperçu
admin-attachments-tab-warnings = Avertissements
admin-attachments-tab-test = Tester un fichier
admin-attachments-tab-denied = Vous n'avez pas accès à cet onglet.
admin-attachments-group-lists = Listes d'autorisation et de blocage
admin-attachments-group-limits = Limite de taille
admin-attachments-group-delivery = Téléchargement et aperçu
admin-attachments-group-warnings = Avertissements
admin-attachments-add-extension = Extension, par exemple pdf ou .tar.gz
admin-attachments-add-mime = Type MIME, par exemple image/png ou image/*
admin-attachments-add = Ajouter
admin-attachments-remove = Supprimer { $rule }
admin-attachments-filter = Filtrer la liste
admin-attachments-list-label = Règles de cette liste
admin-attachments-count = { $count } règles sur { $max }
admin-attachments-empty-list = Aucune règle dans cette liste.
admin-attachments-none-match = Aucune règle ne correspond au filtre.
admin-attachments-bulk = Coller plusieurs règles
admin-attachments-bulk-add = Tout ajouter
admin-attachments-import = Importer un fichier texte
admin-attachments-export = Exporter en texte
admin-attachments-err-extension = Ce n'est pas une extension. Utilisez des lettres, des chiffres, des tirets et des traits de soulignement, jusqu'à 32 caractères ; des points peuvent séparer les parties, comme dans tar.gz.
admin-attachments-err-mime = Ce n'est pas un type MIME. Utilisez type/sous-type, ou type/* pour tous les sous-types.
admin-attachments-err-long = Cette règle est trop longue.
admin-attachments-invalid = Non ajouté, règle non valide : { $entries }
admin-attachments-duplicates = Déjà dans la liste : { $rules }
admin-attachments-overflow = Une liste contient au plus { $max } règles. Les autres n'ont pas été ajoutées.
admin-attachments-conflict-blockedOverAllowed = { $rule } chevauche { $other } dans l'autre liste. Les règles de blocage l'emportent, elle est donc bloquée.
admin-attachments-conflict-blockedOverWarn = { $rule } chevauche { $other } dans l'autre liste. Un fichier bloqué est refusé avant tout avertissement, l'avertissement ne s'affiche donc jamais.
admin-attachments-conflict-allowedAndWarn = { $rule } chevauche { $other } dans l'autre liste. Le fichier est autorisé et affiche un avertissement.
admin-attachments-bytes-unit = Unité
admin-attachments-unlimited = Aucune limite de taille
admin-attachments-bytes-equals = { $size } ({ $bytes } octets)
admin-attachments-list-allowedExtensions = extensions autorisées
admin-attachments-list-allowedMimeTypes = types MIME autorisés
admin-attachments-list-blockedExtensions = extensions bloquées
admin-attachments-list-blockedMimeTypes = types MIME bloqués
admin-attachments-list-warnExtensions = extensions avec avertissement
admin-attachments-list-warnMimeTypes = types MIME avec avertissement
admin-attachments-sim-title = Tester un fichier
admin-attachments-sim-help = Demande au serveur comment la politique enregistrée traite un fichier de ce nom, de ce type et de cette taille. Rien n'est modifié ni stocké. Les messages et les pièces jointes ne sont jamais lus.
admin-attachments-sim-filename = Nom du fichier
admin-attachments-sim-mime = Type MIME
admin-attachments-sim-size = Taille
admin-attachments-sim-run = Tester
admin-attachments-sim-invalid = Saisissez un nom de fichier et une taille de zéro ou plus.
admin-attachments-sim-allowed = Autorisé
admin-attachments-sim-warn = Autorisé avec un avertissement
admin-attachments-sim-blocked = Non remis
admin-attachments-sim-preview-yes = L'aperçu est autorisé.
admin-attachments-sim-preview-no = L'aperçu n'est pas autorisé.
admin-attachments-sim-checked = Taille vérifiée : { $size }.
admin-attachments-sim-matched = Règles correspondantes dans : { $lists }.
admin-attachments-sim-matched-none = Aucune règle d'autorisation, de blocage ou d'avertissement ne correspond.
admin-attachments-reason-downloadsDisabled = Les téléchargements sont désactivés.
admin-attachments-reason-attachmentTooLarge = Le fichier dépasse la limite de taille.
admin-attachments-reason-attachmentBlocked = Une règle de blocage correspond.
admin-attachments-reason-attachmentNotAllowed = Une liste d'autorisation existe et rien n'y correspond.

# -- Machine translation (2026-11): added to match en ---------------------
admin-attachments-reason-attachmentContainerRestricted = Les messages joints et les fichiers TNEF sont refusés tant qu'une restriction est active, car leur contenu n'est pas listé.
admin-settings-key-mail-reply-images = Images dans les réponses et les transferts
admin-settings-help-mail-reply-images = Indique si une réponse ou un transfert inclut les images du message d'origine : ask (par défaut), toujours autoriser, autoriser seulement pour les domaines d'expéditeurs de confiance et demander sinon, ou toujours refuser. Verrouillez ce paramètre pour l'imposer. Les images ne sont lues que lorsqu'elles sont incluses.
admin-settings-key-mail-reply-image-trusted-domains = Domaines d'expéditeurs d'images de confiance
admin-settings-help-mail-reply-image-trusted-domains = Jusqu'à 200 domaines d'expéditeurs exacts dont les images sont incluses sans demander lorsque la politique est domaines de confiance. Les utilisateurs complètent leur propre liste depuis l'invite de citation, sauf si ce paramètre est verrouillé.
admin-settings-key-mail-send-recall-delay-seconds = Fenêtre d'annulation d'envoi (par défaut)
admin-settings-help-mail-send-recall-delay-seconds = Combien de secondes un message envoyé attend dans la boîte d'envoi, où l'expéditeur peut le rappeler, avant d'être livré. 0 envoie immédiatement. Par défaut 10, jusqu'à 1800 (30 minutes). Les utilisateurs choisissent leur propre valeur jusqu'à la limite ci-dessous, sauf si vous le verrouillez.
admin-settings-key-mail-send-recall-delay-max-seconds = Fenêtre d'annulation d'envoi (limite)
admin-settings-help-mail-send-recall-delay-max-seconds = La plus longue fenêtre d'annulation d'envoi qu'un utilisateur peut choisir, en secondes. Le serveur ramène une demande plus longue à cette valeur. Par défaut et maximum 1800.
admin-settings-key-mail-send-allow-recall = Autoriser le rappel des messages envoyés
admin-settings-help-mail-send-allow-recall = Désactivé, les messages partent immédiatement et ne peuvent pas être rappelés depuis la boîte d'envoi. Les envois programmés ne sont pas concernés ; ils peuvent toujours être annulés jusqu'à leur heure.
admin-settings-key-mail-send-max-delayed-seconds = Horizon de programmation
admin-settings-help-mail-send-max-delayed-seconds = Jusqu'à quelle échéance un message peut être programmé, en secondes. Par défaut 7776000 (90 jours), maximum 31536000 (1 an). 0 désactive l'envoi programmé.
admin-settings-key-mail-send-max-queued = Messages en file par boîte aux lettres
admin-settings-help-mail-send-max-queued = Combien de messages une boîte aux lettres peut avoir en attente dans la boîte d'envoi (fenêtre d'annulation ou programmés) à la fois. Par défaut 200. Les envois supplémentaires sont refusés avec un message clair et ne sont jamais perdus.
admin-attachments-sim-pick = Remplir à partir d'un fichier (facultatif)
admin-attachments-mime-toggle = Afficher les types MIME courants
admin-attachments-mime-hint = Choisissez un type courant ou saisissez tout autre type/sous-type.
admin-attachments-mime-invalid = Ce n'est pas un type/sous-type comme text/plain. Il sera envoyé tel que saisi.
admin-attachments-mime-group-documents = Documents
admin-attachments-mime-group-images = Images
admin-attachments-mime-group-archives = Archives
admin-attachments-mime-group-executables = Exécutables et scripts
admin-attachments-mime-group-media = Audio et vidéo
admin-attachments-mime-group-email = E-mail et calendrier
admin-attachments-mime-group-other = Autres
admin-settings-key-cookies-enabled = Contrôle du consentement aux cookies
admin-settings-help-cookies-enabled = Affiche le contrôle du consentement aux cookies aux visiteurs. Désactivé laisse l'application se comporter comme sans le contrôle.
admin-settings-key-cookies-mode = Modèle de consentement
admin-settings-help-cookies-mode = optIn : rien de facultatif n'est stocké tant que le visiteur n'accepte pas. optOut : les valeurs par défaut des catégories s'appliquent tant que le visiteur ne refuse pas. noticeOnly : le visiteur est informé, sans choix.
admin-settings-key-cookies-region-mode = Modèle de consentement dans les régions strictes
admin-settings-help-cookies-region-mode = Le modèle utilisé pour les visiteurs des régions strictes. same y utilise aussi le modèle de consentement ci-dessus.
admin-settings-key-cookies-strict-regions = Régions strictes
admin-settings-help-cookies-strict-regions = Codes pays ISO 3166-1 qui utilisent le modèle de consentement des régions strictes. Par défaut les États de l'UE et de l'EEE, le Royaume-Uni et la Suisse.
admin-settings-key-cookies-region-header = En-tête de pays
admin-settings-help-cookies-region-header = Nom de l'en-tête de requête, défini par un proxy ou un CDN, qui contient le code pays du visiteur.
admin-settings-key-cookies-unknown-region = Pays inconnu
admin-settings-help-cookies-unknown-region = Quel modèle s'applique lorsqu'aucun pays ne peut être déterminé : le modèle des régions strictes ou le modèle normal.
admin-settings-key-cookies-functional-enabled = Proposer les cookies fonctionnels
admin-settings-help-cookies-functional-enabled = Propose la catégorie des préférences et des fonctionnalités (thème, langue, mémorisation de la disposition) dans le contrôle de consentement.
admin-settings-key-cookies-functional-default = Valeur par défaut des cookies fonctionnels
admin-settings-help-cookies-functional-default = État initial de la catégorie fonctionnelle lorsque le modèle de consentement autorise des valeurs par défaut.
admin-settings-key-cookies-analytics-enabled = Proposer les cookies d'analyse
admin-settings-help-cookies-analytics-enabled = Propose la catégorie d'analyse dans le contrôle de consentement.
admin-settings-key-cookies-analytics-default = Valeur par défaut de l'analyse
admin-settings-help-cookies-analytics-default = État initial de la catégorie d'analyse lorsque le modèle de consentement autorise des valeurs par défaut.
admin-settings-key-cookies-marketing-enabled = Proposer les cookies de marketing
admin-settings-help-cookies-marketing-enabled = Propose la catégorie de marketing dans le contrôle de consentement.
admin-settings-key-cookies-marketing-default = Valeur par défaut du marketing
admin-settings-help-cookies-marketing-default = État initial de la catégorie de marketing lorsque le modèle de consentement autorise des valeurs par défaut.
admin-settings-key-cookies-layout = Disposition
admin-settings-help-cookies-layout = banner est une barre sur toute la page, card une carte flottante, modal une boîte de dialogue qui bloque la page jusqu'au choix du visiteur.
admin-settings-key-cookies-position = Position
admin-settings-help-cookies-position = Où le contrôle de consentement apparaît sur la page.
admin-settings-key-cookies-reject-prominent = Refuser aussi visible qu'accepter
admin-settings-help-cookies-reject-prominent = Affiche Tout refuser avec la même taille et le même style que Tout accepter.
admin-settings-key-cookies-policy-url = URL de la politique de cookies
admin-settings-help-cookies-policy-url = Lien vers votre politique de cookies, affiché dans le contrôle de consentement.
admin-settings-key-cookies-privacy-url = URL de la politique de confidentialité
admin-settings-help-cookies-privacy-url = Lien vers votre politique de confidentialité, affiché dans le contrôle de consentement.
admin-settings-key-cookies-policy-version = Version de la politique
admin-settings-help-cookies-policy-version = Augmentez ce nombre lorsque votre utilisation des cookies change, afin que les visiteurs soient interrogés de nouveau.
admin-settings-key-cookies-lifetime-days = Durée de validité du consentement
admin-settings-help-cookies-lifetime-days = Jours pendant lesquels une décision reste valide avant que le visiteur ne soit interrogé de nouveau.
admin-settings-key-cookies-reprompt-on-version = Redemander lors d'une nouvelle version de la politique
admin-settings-help-cookies-reprompt-on-version = Redemande aux visiteurs lorsque la version de la politique est supérieure à celle sur laquelle ils ont décidé.
admin-settings-key-cookies-honour-gpc = Respecter Global Privacy Control
admin-settings-help-cookies-honour-gpc = Traite un signal Global Privacy Control du navigateur comme un refus de l'analyse et du marketing.
admin-settings-key-cookies-honour-dnt = Respecter Do Not Track
admin-settings-help-cookies-honour-dnt = Traite un signal Do Not Track du navigateur comme un refus de l'analyse et du marketing.
admin-settings-key-cookies-log-consent = Tenir un journal des consentements
admin-settings-help-cookies-log-consent = Conserve côté serveur une trace de chaque décision de consentement, pour les statistiques et vos propres registres.
admin-settings-key-cookies-log-retention-days = Conservation du journal des consentements
admin-settings-help-cookies-log-retention-days = Jours pendant lesquels les lignes du journal des consentements sont conservées avant d'être supprimées.
admin-settings-key-cookies-scan-interval-hours = Intervalle d'analyse du stockage
admin-settings-help-cookies-scan-interval-hours = Heures entre les analyses automatiques des cookies et du stockage du navigateur. 0 n'analyse que sur demande.
admin-settings-key-cookies-unclassified = Stockage non classé
admin-settings-help-cookies-unclassified = report laisse en l'état le stockage que rien ne classe et le liste pour examen. Les autres valeurs le traitent comme cette catégorie jusqu'à ce que vous le classiez.
admin-settings-key-cookies-text-title = Titre du contrôle de consentement
admin-settings-help-cookies-text-title = Titre du contrôle de consentement par langue. Vide utilise le texte intégré.
admin-settings-key-cookies-text-body = Texte du contrôle de consentement
admin-settings-help-cookies-text-body = Texte explicatif du contrôle de consentement par langue. Vide utilise le texte intégré.
admin-settings-key-cookies-label-necessary = Nom de la catégorie nécessaire
admin-settings-help-cookies-label-necessary = Nom de la catégorie nécessaire par langue. Vide utilise le nom intégré.
admin-settings-key-cookies-description-necessary = Description de la catégorie nécessaire
admin-settings-help-cookies-description-necessary = Description de la catégorie nécessaire par langue. Vide utilise la description intégrée.
admin-settings-key-cookies-label-functional = Nom de la catégorie fonctionnelle
admin-settings-help-cookies-label-functional = Nom de la catégorie fonctionnelle par langue. Vide utilise le nom intégré.
admin-settings-key-cookies-description-functional = Description de la catégorie fonctionnelle
admin-settings-help-cookies-description-functional = Description de la catégorie fonctionnelle par langue. Vide utilise la description intégrée.
admin-settings-key-cookies-label-analytics = Nom de la catégorie d'analyse
admin-settings-help-cookies-label-analytics = Nom de la catégorie d'analyse par langue. Vide utilise le nom intégré.
admin-settings-key-cookies-description-analytics = Description de la catégorie d'analyse
admin-settings-help-cookies-description-analytics = Description de la catégorie d'analyse par langue. Vide utilise la description intégrée.
admin-settings-key-cookies-label-marketing = Nom de la catégorie de marketing
admin-settings-help-cookies-label-marketing = Nom de la catégorie de marketing par langue. Vide utilise le nom intégré.
admin-settings-key-cookies-description-marketing = Description de la catégorie de marketing
admin-settings-help-cookies-description-marketing = Description de la catégorie de marketing par langue. Vide utilise la description intégrée.
admin-settings-key-encryption-rotation-days = Intervalle de rotation de la clé de base de données (jours)
admin-settings-help-encryption-rotation-days = Jours après le dernier changement de clé au bout desquels la clé de base de données est signalée comme à renouveler. 0 ne fixe aucune échéance. La clé elle-même n'est jamais stockée dans les paramètres.
admin-settings-key-encryption-warnings-enabled = Signaler les constats de chiffrement
admin-settings-help-encryption-warnings-enabled = Affiche les stockages en clair et une rotation de clé en retard comme constat dans l'évaluation du déploiement.
admin-settings-key-retention-sweeper-enabled = Balayage de conservation
admin-settings-help-retention-sweeper-enabled = Exécute le balayage quotidien qui supprime les lignes plus anciennes que les limites de cette page. Le désactiver conserve toutes les lignes jusqu'à sa réactivation.
admin-settings-key-retention-sweeper-batch-rows = Lignes par lot
admin-settings-help-retention-sweeper-batch-rows = Lignes qu'une instruction de suppression retire. Des lots plus petits maintiennent les verrous de base de données moins longtemps.
admin-settings-key-retention-sweeper-max-batches = Lots par stockage et par exécution
admin-settings-help-retention-sweeper-max-batches = Nombre maximal d'instructions de suppression qu'un stockage reçoit lors d'une exécution. Un stockage avec davantage d'anciennes lignes continue à l'exécution suivante.
admin-settings-key-retention-sweeper-jitter-minutes = Dispersion de l'heure de démarrage (minutes)
admin-settings-help-retention-sweeper-jitter-minutes = La prochaine exécution démarre un nombre aléatoire de minutes, jusqu'à cette valeur, après 24 heures, afin que plusieurs serveurs ne balayent pas en même temps.
admin-settings-key-retention-sweeper-vacuum-after-rows = Récupérer l'espace après (lignes)
admin-settings-help-retention-sweeper-vacuum-after-rows = Lorsqu'une exécution supprime au moins ce nombre de lignes, la base de données est compactée (VACUUM). 0 ne compacte jamais après un balayage ; la tâche de vacuum hebdomadaire est distincte.
admin-settings-key-retention-password-change-audit-days = Enregistrements de changement de mot de passe (jours)
admin-settings-help-retention-password-change-audit-days = Jours pendant lesquels un enregistrement de changement de mot de passe (compte, méthode, résultat) est conservé. 0 les conserve indéfiniment.
admin-settings-key-retention-sso-login-audit-days = Enregistrements de connexion par authentification unique (jours)
admin-settings-help-retention-sso-login-audit-days = Jours pendant lesquels un enregistrement de connexion par authentification unique (fournisseur, sujet haché, résultat) est conservé. 0 les conserve indéfiniment.
admin-settings-key-retention-dlp-audit-days = Enregistrements de prévention des pertes de données (jours)
admin-settings-help-retention-dlp-audit-days = Jours pendant lesquels un enregistrement de correspondance avec une règle de prévention des pertes de données est conservé. 0 les conserve indéfiniment.
admin-settings-key-retention-assist-audit-days = Enregistrements d'accès de l'assistant (jours)
admin-settings-help-retention-assist-audit-days = Jours pendant lesquels un enregistrement de l'utilisation d'une capacité par un assistant (acteur, capacité, résumé de la portée) est conservé. 0 les conserve indéfiniment.
admin-settings-key-retention-alert-events-days = Enregistrements d'alertes déclenchées (jours)
admin-settings-help-retention-alert-events-days = Jours pendant lesquels un enregistrement d'alerte d'administrateur déclenchée est conservé. 0 les conserve indéfiniment.
admin-settings-key-retention-job-runs-days = Historique des tâches en arrière-plan (jours)
admin-settings-help-retention-job-runs-days = Jours pendant lesquels une exécution terminée de tâche en arrière-plan est conservée. Une exécution encore en cours n'est jamais supprimée. 0 conserve l'historique indéfiniment.
admin-settings-key-retention-outbox-days = Enregistrements d'envois terminés et annulés (jours)
admin-settings-help-retention-outbox-days = Jours pendant lesquels l'enregistrement d'envoi d'un message livré ou annulé est conservé. Un message en attente, en nouvelle tentative, échoué ou incertain n'est jamais supprimé. 0 les conserve indéfiniment.
admin-settings-key-retention-expired-credentials-grace-days = Identifiants expirés (jours après expiration)
admin-settings-help-retention-expired-credentials-grace-days = Jours pendant lesquels un appareil mémorisé, un défi de connexion, un jeton de connexion, un jeton OAuth, un nonce de webhook, un compteur d'échecs de source ou un identifiant de lien d'image expiré est conservé après son expiration. 0 le supprime à l'exécution suivante.
admin-settings-key-retention-ip-addresses-days = Réduire les adresses IP stockées après (jours)
admin-settings-help-retention-ip-addresses-days = Après ce nombre de jours, l'adresse IP de l'historique d'authentification et des entrées d'audit est tronquée ou hachée et la ligne est conservée. 0 conserve les adresses telles qu'enregistrées.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-retention-ip-addresses-mode = Méthode de réduction des IP
admin-settings-help-retention-ip-addresses-mode = Truncate conserve le préfixe IPv4 /24 ou IPv6 /48. Hash remplace l'adresse par un court hachage à clé qui regroupe toujours la même adresse.
admin-settings-key-retention-user-enabled = Les utilisateurs peuvent définir des règles de conservation
admin-settings-help-retention-user-enabled = Les utilisateurs peuvent définir leurs propres règles de conservation pour les dossiers et le courrier. Désactivé, les règles utilisateur existantes cessent de s'exécuter ; les règles à l'échelle de l'organisation ci-dessous continuent de s'exécuter.
admin-settings-key-retention-user-min-days = Nombre minimal de jours avant qu'une règle utilisateur supprime du courrier
admin-settings-help-retention-user-min-days = Une règle utilisateur qui supprime du courrier ne peut pas agir sur du courrier plus récent que cela. Une valeur plus petite est refusée à l'enregistrement de la règle et relevée à son exécution.
admin-settings-key-retention-user-max-days = Nombre maximal de jours qu'une règle utilisateur peut attendre avant de supprimer du courrier
admin-settings-help-retention-user-max-days = Une règle utilisateur qui supprime du courrier ne peut pas attendre plus longtemps. 0 ne fixe aucun plafond. Une valeur plus grande est refusée à l'enregistrement de la règle et abaissée à son exécution.
admin-settings-key-retention-user-max-rules = Nombre maximal de règles de conservation par utilisateur
admin-settings-help-retention-user-max-rules = Le plus grand nombre de règles de conservation qu'un utilisateur peut garder.
admin-settings-key-retention-user-allow-permanent-delete = Les règles utilisateur peuvent supprimer du courrier définitivement
admin-settings-help-retention-user-allow-permanent-delete = Désactivé, une règle de suppression déplace toujours le courrier vers la corbeille et la fenêtre d'annulation s'applique avant la purge.
admin-settings-key-retention-user-undo-days = Fenêtre d'annulation des règles de conservation (jours)
admin-settings-help-retention-user-undo-days = Jours pendant lesquels le courrier déplacé vers la corbeille par une règle de conservation y reste avant d'être purgé. Un utilisateur peut choisir une fenêtre plus courte. 0 purge à l'exécution suivante.
admin-settings-key-retention-user-mandatory = Règles de courrier à l'échelle de l'organisation
admin-settings-help-retention-user-mandatory = Règles que reçoit chaque compte, sous forme d'entrées rôle-de-dossier:jours, par exemple trash:30 ou junk:30. Le courrier de ce dossier plus ancien que ce nombre de jours est purgé. Un utilisateur ne peut ni les modifier, ni les suspendre, ni en exclure du courrier. Le courrier d'un compte sous conservation légale est ignoré.
admin-settings-key-retention-user-protected-folders = Dossiers que les règles de conservation ne touchent jamais
admin-settings-help-retention-user-protected-folders = Dossiers sur lesquels aucune règle utilisateur n'agit, par rôle (drafts, sent, inbox, archive) ou par nom de dossier. Un dossier protégé l'emporte sur une règle d'organisation pour le même dossier.
admin-settings-key-retention-user-templates = Modèles de conservation proposés aux utilisateurs
admin-settings-help-retention-user-templates = Modèles que les utilisateurs peuvent adopter dans les paramètres, par identifiant : newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Exécuter les règles utilisateur pour les comptes restreints
admin-settings-help-retention-user-run-for-restricted = Indique si les règles de conservation des utilisateurs s'exécutent aussi pour les comptes sous une limitation du traitement. Désactivé ignore ces comptes.
admin-settings-key-retention-user-interval-hours = Heures entre les exécutions de conservation des utilisateurs
admin-settings-help-retention-user-interval-hours = Heures entre les exécutions planifiées de la tâche de conservation des utilisateurs pour un compte.
admin-settings-key-retention-user-batch-messages = Messages par appel de conservation
admin-settings-help-retention-user-batch-messages = Messages qu'un appel de déplacement ou de suppression traite. 200 au plus sont utilisés.
admin-settings-key-retention-user-max-per-run = Nombre maximal de messages modifiés par compte et par exécution
admin-settings-help-retention-user-max-per-run = Le plus grand nombre de messages que la tâche de conservation des utilisateurs modifie pour un compte lors d'une exécution. Le reste attend l'exécution suivante.
admin-settings-key-retention-user-pause-ms = Pause entre les lots de conservation (ms)
admin-settings-help-retention-user-pause-ms = Pause entre les lots, en millisecondes, pour qu'un grand nettoyage ne surcharge pas le serveur de messagerie.
admin-settings-key-retention-class-short-days = Plafond de la classe courte (jours)
admin-settings-help-retention-class-short-days = Plafond pour les données opérationnelles : sessions, jetons, limites de débit, historique des tâches et métriques. S'applique lorsque la limite de classe est activée.
admin-settings-key-retention-class-medium-days = Plafond de la classe moyenne (jours)
admin-settings-help-retention-class-medium-days = Plafond pour l'historique de connexion, l'historique de protection des requêtes et les avis de sécurité. Ne doit pas être inférieur à la classe courte ; une valeur plus basse est relevée à celle-ci.
admin-settings-key-retention-class-long-days = Conservation de la classe longue (jours)
admin-settings-help-retention-class-long-days = Jours pendant lesquels la classe longue (audit de sécurité, actions d'administrateur, demandes, consentement, marqueurs) est conservée, archive comprise. 0 la conserve indéfiniment et la console avertit. Ne doit pas être inférieure à la classe moyenne.
admin-settings-key-retention-class-enforce = Appliquer à chaque stockage le plafond de sa classe
admin-settings-help-retention-class-enforce = Activé, la limite propre de chaque stockage est ramenée au plafond de sa classe et 0 (conserver) compte comme le plafond. Désactivé, les limites par stockage s'appliquent.
admin-settings-key-retention-archive-enabled = Archiver les lignes anciennes de la classe longue
admin-settings-help-retention-archive-enabled = Déplace les lignes anciennes de la classe longue vers des fichiers d'archive scellés et compressés au lieu de les supprimer.
admin-settings-key-retention-archive-directory = Répertoire d'archive
admin-settings-help-retention-archive-directory = Répertoire des fichiers d'archive, absolu ou relatif au répertoire de travail du serveur. Les segments de répertoire parent (..) sont refusés.
admin-settings-key-retention-archive-batch-rows = Lignes par fichier d'archive
admin-settings-help-retention-archive-batch-rows = Lignes écrites dans un fichier d'archive.
admin-settings-key-retention-archive-max-segments = Fichiers d'archive par stockage et par exécution
admin-settings-help-retention-archive-max-segments = Fichiers d'archive qu'une exécution peut écrire pour chaque stockage. Le reste attend l'exécution suivante.
admin-settings-key-retention-archive-key-generation = Génération de la clé de scellement des archives
admin-settings-help-retention-archive-key-generation = Génération de la clé qui scelle les nouveaux fichiers d'archive. Augmentez-la pour renouveler la clé ; les fichiers précédents se vérifient toujours avec leur propre génération.
admin-settings-key-retention-archive-verify-days = Jours entre les vérifications d'archive
admin-settings-help-retention-archive-verify-days = Jours entre les vérifications d'intégrité des fichiers d'archive.
admin-settings-key-retention-archive-legal-hold = Conservation légale de l'archive
admin-settings-help-retention-archive-legal-hold = Conserve chaque fichier d'archive, quelle que soit la limite de la classe longue.
admin-settings-key-logging-file-enabled = Écrire des fichiers journaux
admin-settings-help-logging-file-enabled = Écrit aussi le journal du serveur dans des fichiers tournants. Désactivé limite la journalisation à la sortie standard, ce qu'un conteneur utilise normalement.
admin-settings-key-logging-file-directory = Répertoire des journaux
admin-settings-help-logging-file-directory = Répertoire des fichiers journaux, absolu ou relatif au répertoire de travail du serveur. Les segments de répertoire parent (..) sont refusés.
admin-settings-key-logging-file-max-size-mb = Rotation à la taille (Mo)
admin-settings-help-logging-file-max-size-mb = Le fichier journal courant est fermé et un nouveau est commencé lorsqu'il atteint cette taille.
admin-settings-key-logging-file-rotate-hours = Rotation à l'âge (heures)
admin-settings-help-logging-file-rotate-hours = Le fichier journal courant est archivé en rotation dès qu'il a cet âge. 0 fait tourner uniquement selon la taille.
admin-settings-key-logging-file-max-age-days = Conserver les fichiers archivés (jours)
admin-settings-help-logging-file-max-age-days = Les fichiers journaux archivés plus anciens que cela sont supprimés.
admin-settings-key-logging-file-max-files = Nombre maximal de fichiers archivés
admin-settings-help-logging-file-max-files = Les plus anciens fichiers archivés au-delà de ce nombre sont supprimés.
admin-settings-key-logging-file-max-total-mb = Taille totale maximale (Mo)
admin-settings-help-logging-file-max-total-mb = Les plus anciens fichiers archivés sont supprimés jusqu'à ce que le répertoire des journaux passe sous cette taille.
admin-settings-key-logging-file-compress = Compresser les fichiers archivés
admin-settings-help-logging-file-compress = Les fichiers journaux archivés sont compressés avec gzip.
admin-settings-key-logging-level = Niveau de journalisation
admin-settings-help-logging-level = Le niveau des composants qui n'ont pas de niveau propre ci-dessous. La variable d'environnement MW_LOG définit le niveau au démarrage jusqu'à la lecture de ce paramètre.
admin-settings-key-logging-component-levels = Niveaux de journalisation par composant
admin-settings-help-logging-component-levels = Une entrée composant=niveau par ligne, par exemple mw_imap=debug. Les niveaux sont error, warn, info, debug et trace.
admin-settings-key-logging-redaction-addresses = Adresses e-mail dans les journaux
admin-settings-help-logging-redaction-addresses = Full laisse les adresses telles que journalisées. Truncated conserve la première lettre et le domaine. Hashed remplace la partie locale par un court hachage. None supprime l'adresse.
admin-settings-key-logging-redaction-ips = Adresses IP dans les journaux
admin-settings-help-logging-redaction-ips = Full laisse les adresses telles que journalisées. Truncated conserve le préfixe IPv4 /24 ou IPv6 /48. Hashed remplace l'adresse par un court hachage. None la supprime.
admin-settings-key-logging-redaction-tokens = Secrets dans les journaux
admin-settings-help-logging-redaction-tokens = Retire des lignes de journal les jetons bearer, les clés d'API, les mots de passe dans les URL et les secrets similaires.
admin-settings-key-retention-sessions-idle-days = Sessions inutilisées (jours)
admin-settings-help-retention-sessions-idle-days = Jours pendant lesquels l'enregistrement d'une session de navigateur ou de client natif, avec son adresse et son user-agent, peut rester inutilisé avant d'être supprimé. Les délais de session s'appliquent d'abord ; ceci ne supprime que les lignes restantes. 0 les conserve.
admin-settings-key-policy-public-pages = Pages de politique publiques
admin-settings-help-policy-public-pages = Sert la politique de confidentialité générée à /privacy et la politique de cookies à /cookies-policy sans connexion.
admin-settings-key-policy-auto-version = Versions de politique automatiques
admin-settings-help-policy-auto-version = Enregistre une nouvelle version de la politique lorsqu'un paramètre décrit par la politique change. Désactivé : les modifications attendent la publication.
admin-settings-key-policy-reconsent-on-change = Redemander le consentement aux cookies lors d'une nouvelle version de la politique
admin-settings-help-policy-reconsent-on-change = Augmente la version de la politique de cookies lorsqu'une nouvelle version de la politique est enregistrée, afin que les visiteurs soient interrogés de nouveau (exige cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Nom du responsable du traitement
admin-settings-help-policy-controller-name = Nom de l'organisation qui décide pourquoi et comment les données sont traitées, affiché dans la politique de confidentialité.
admin-settings-key-policy-controller-address = Adresse du responsable du traitement
admin-settings-help-policy-controller-address = Adresse postale du responsable du traitement, affichée dans la politique de confidentialité.
admin-settings-key-policy-legal-bases = Bases juridiques
admin-settings-help-policy-legal-bases = Les bases juridiques sur lesquelles vous vous appuyez, avec vos propres mots, affichées dans la politique de confidentialité.
admin-settings-key-policy-hosting-provider = Hébergeur
admin-settings-help-policy-hosting-provider = Qui héberge les données et où, affiché dans la politique de confidentialité.
admin-settings-key-policy-hosting-transfers = Transferts hors du pays d'hébergement
admin-settings-help-policy-hosting-transfers = Tout transfert de données vers d'autres pays et les garanties que vous utilisez, affichés dans la politique de confidentialité.
admin-settings-key-policy-supervisory-authority = Autorité de contrôle
admin-settings-help-policy-supervisory-authority = L'autorité auprès de laquelle les utilisateurs peuvent déposer une réclamation, affichée dans la politique de confidentialité.
admin-settings-key-dpia-suggest = Suggérer des analyses d'impact
admin-settings-help-dpia-suggest = Liste une analyse d'impact suggérée lorsque l'assistant, le suivi d'ouverture ou l'analyse est activé et qu'aucune n'existe.
admin-settings-key-dpia-review-months = Intervalle de révision des analyses d'impact
admin-settings-help-dpia-review-months = Mois jusqu'à la date de révision fixée lorsqu'une analyse est approuvée.
admin-settings-key-dpia-reminder-days = Fenêtre de rappel des analyses d'impact
admin-settings-help-dpia-reminder-days = Jours avant la date de révision à partir desquels une analyse est listée comme à échéance.
admin-settings-key-restriction-enabled = Limitations du traitement
admin-settings-help-restriction-enabled = Indique si un administrateur ou une demande approuvée peut limiter le traitement des données d'une personne.
admin-settings-key-restriction-auto-apply-on-approval = Limiter lorsqu'une demande est approuvée
admin-settings-help-restriction-auto-apply-on-approval = Applique la limitation dès qu'une demande de limitation est approuvée. Désactivé : un administrateur l'applique depuis Conformité > Limitations.
admin-settings-key-restriction-mail-delivery = Nouveau courrier pendant une limitation
admin-settings-help-restriction-mail-delivery = Ce que l'administrateur a choisi pour le courrier qui arrive pendant qu'une limitation s'applique : le conserver, le mettre en file ou le rejeter. Ce serveur ne reçoit pas de courrier ; la valeur est enregistrée et affichée, et le serveur de messagerie agit en conséquence.
admin-settings-key-restriction-default-expiry-days = Durée par défaut d'une limitation (jours)
admin-settings-help-restriction-default-expiry-days = Jours jusqu'à ce qu'une nouvelle limitation se termine d'elle-même. 0 la conserve jusqu'à ce que quelqu'un la lève.
admin-settings-key-restriction-notify-user = Informer la personne
admin-settings-help-restriction-notify-user = Envoie un avis à la personne lorsqu'une limitation est appliquée ou levée.
admin-settings-key-restriction-scope-indexing = Suspendre l'indexation de recherche et les embeddings
admin-settings-help-restriction-scope-indexing = Indique si l'indexation de recherche et les embeddings sont suspendus par défaut dans une nouvelle limitation.
admin-settings-key-restriction-scope-assistant = Suspendre l'assistant
admin-settings-help-restriction-scope-assistant = Indique si l'assistant est suspendu par défaut dans une nouvelle limitation.
admin-settings-key-restriction-scope-analytics = Suspendre l'analyse et le suivi d'ouverture
admin-settings-help-restriction-scope-analytics = Indique si l'analyse et le suivi d'ouverture sont suspendus par défaut dans une nouvelle limitation.
admin-settings-key-restriction-scope-classification = Suspendre les règles et le classement automatique
admin-settings-help-restriction-scope-classification = Indique si les règles et le classement automatique sont suspendus par défaut dans une nouvelle limitation.
admin-settings-key-restriction-scope-push = Suspendre les notifications push
admin-settings-help-restriction-scope-push = Indique si les notifications push sont suspendues par défaut dans une nouvelle limitation.
admin-settings-key-restriction-scope-third-party = Suspendre les services connectés et les modules
admin-settings-help-restriction-scope-third-party = Indique si les services connectés et les modules sont suspendus par défaut dans une nouvelle limitation.
admin-settings-key-consent-enabled = Consentement par finalité
admin-settings-help-consent-enabled = Affiche les finalités et les choix de consentement aux utilisateurs et les applique à l'exécution.
admin-settings-key-consent-receipts-retention-days = Reçus de consentement conservés (jours)
admin-settings-help-consent-receipts-retention-days = Jours pendant lesquels les reçus de consentement sont conservés. 0 les conserve jusqu'à l'effacement de la personne.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-consent-art22-notice = Avis de traitement automatisé
admin-settings-help-consent-art22-notice = Affiche aux utilisateurs l'avis en langage clair sur les fonctions d'assistant et de recherche, avec les interrupteurs par fonction.
admin-settings-key-consent-analytics-basis = Base juridique : compteurs d'utilisation
admin-settings-help-consent-analytics-basis = La base juridique affichée aux utilisateurs pour les compteurs d'utilisation. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-analytics-version = Version du texte : compteurs d'utilisation
admin-settings-help-consent-analytics-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-analytics-text = Formulation propre : compteurs d'utilisation
admin-settings-help-consent-analytics-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-analytics-tag-basis = Base juridique : script du fournisseur d'analyse
admin-settings-help-consent-analytics-tag-basis = La base juridique affichée aux utilisateurs pour le script du fournisseur d'analyse. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-analytics-tag-version = Version du texte : script du fournisseur d'analyse
admin-settings-help-consent-analytics-tag-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-analytics-tag-text = Formulation propre : script du fournisseur d'analyse
admin-settings-help-consent-analytics-tag-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-assistant-basis = Base juridique : assistant
admin-settings-help-consent-assistant-basis = La base juridique affichée aux utilisateurs pour l'assistant. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-assistant-version = Version du texte : assistant
admin-settings-help-consent-assistant-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-assistant-text = Formulation propre : assistant
admin-settings-help-consent-assistant-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-push-basis = Base juridique : notifications push
admin-settings-help-consent-push-basis = La base juridique affichée aux utilisateurs pour les notifications push. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-push-version = Version du texte : notifications push
admin-settings-help-consent-push-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-push-text = Formulation propre : notifications push
admin-settings-help-consent-push-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-open-tracking-basis = Base juridique : suivi d'ouverture
admin-settings-help-consent-open-tracking-basis = La base juridique affichée aux utilisateurs pour le suivi d'ouverture. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-open-tracking-version = Version du texte : suivi d'ouverture
admin-settings-help-consent-open-tracking-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-open-tracking-text = Formulation propre : suivi d'ouverture
admin-settings-help-consent-open-tracking-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-remote-content-basis = Base juridique : images distantes
admin-settings-help-consent-remote-content-basis = La base juridique affichée aux utilisateurs pour les images distantes. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-remote-content-version = Version du texte : images distantes
admin-settings-help-consent-remote-content-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-remote-content-text = Formulation propre : images distantes
admin-settings-help-consent-remote-content-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-integrations-basis = Base juridique : services connectés
admin-settings-help-consent-integrations-basis = La base juridique affichée aux utilisateurs pour les services connectés. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-integrations-version = Version du texte : services connectés
admin-settings-help-consent-integrations-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-integrations-text = Formulation propre : services connectés
admin-settings-help-consent-integrations-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-consent-telemetry-basis = Base juridique : mesures des temps de chargement
admin-settings-help-consent-telemetry-basis = La base juridique affichée aux utilisateurs pour les mesures des temps de chargement. Seul le consentement fait attendre un oui à la fonction ; les autres bases s'appliquent tant que la personne ne se rétracte pas. L'administrateur décide quelle base convient.
admin-settings-key-consent-telemetry-version = Version du texte : mesures des temps de chargement
admin-settings-help-consent-telemetry-version = Augmentez ce nombre lorsque la formulation ou le traitement change ; les personnes qui ont accepté une version plus ancienne sont interrogées de nouveau.
admin-settings-key-consent-telemetry-text = Formulation propre : mesures des temps de chargement
admin-settings-help-consent-telemetry-text = Votre propre texte pour cette finalité, par langue. Vide utilise la formulation intégrée.
admin-settings-key-breach-enabled = Registre des violations de données
admin-settings-help-breach-enabled = Active le registre et le compte à rebours de notification.
admin-settings-key-breach-notice-hours = Heures pour notifier l'autorité
admin-settings-help-breach-notice-hours = Heures entre la prise de connaissance et la notification à l'autorité. Le délai habituel est de 72.
admin-settings-key-breach-reminders-hours = Moments de rappel
admin-settings-help-breach-reminders-hours = Heures avant l'échéance qui déclenchent un rappel, par exemple 6, 24 et 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Répétition en cas de retard
admin-settings-help-breach-reminders-overdue-repeat-hours = Heures entre les rappels une fois l'échéance dépassée. 0 ne rappelle qu'une fois.
admin-settings-key-breach-authority-name = Autorité de contrôle
admin-settings-help-breach-authority-name = Nom imprimé sur la notification à l'autorité.
admin-settings-key-breach-authority-url = Page de notification de l'autorité
admin-settings-help-breach-authority-url = Adresse de la page de notification de l'autorité.
admin-settings-key-breach-authority-contact = Contact de l'autorité
admin-settings-help-breach-authority-contact = Coordonnées de l'autorité.
admin-settings-key-breach-dpo-name = Nom du contact protection des données
admin-settings-help-breach-dpo-name = Nom indiqué comme point de contact dans les notifications.
admin-settings-key-breach-dpo-email = E-mail du contact protection des données
admin-settings-help-breach-dpo-email = Adresse indiquée comme point de contact dans les notifications.
admin-settings-key-breach-dpo-phone = Téléphone du contact protection des données
admin-settings-help-breach-dpo-phone = Numéro de téléphone indiqué dans les notifications.
admin-settings-key-breach-template-subject = Objet du message
admin-settings-help-breach-template-subject = Objet du message aux personnes concernées, par langue.
admin-settings-key-breach-template-body = Texte du message
admin-settings-help-breach-template-body = Texte du message aux personnes concernées, par langue.
admin-settings-key-breach-notify-audit = Consigner les notifications dans le journal d'audit
admin-settings-help-breach-notify-audit = Les rappels et les détections deviennent des événements de sécurité.
admin-settings-key-breach-notify-webhook-urls = Adresses de webhook
admin-settings-help-breach-notify-webhook-urls = Adresses informées des rappels et des messages. Traitez-les comme des secrets ; chaque appel est signé.
admin-settings-key-breach-notify-operator-addresses = Boîtes aux lettres des opérateurs
admin-settings-help-breach-notify-operator-addresses = Adresses de boîtes aux lettres locales qui reçoivent les rappels dans leur propre boîte de réception.
admin-settings-key-breach-subjects-in-app = Avis dans le compte de la personne
admin-settings-help-breach-subjects-in-app = Affiche le message dans le compte de la personne.
admin-settings-key-breach-subjects-inbox = Message dans la boîte aux lettres de la personne
admin-settings-help-breach-subjects-inbox = Remet le message dans la propre boîte aux lettres de la personne.
admin-settings-key-breach-subjects-webhook = Envoyer aussi aux webhooks
admin-settings-help-breach-subjects-webhook = Envoie aussi le texte du message aux adresses de webhook.
admin-settings-key-breach-retention-years = Conservation du registre en années
admin-settings-help-breach-retention-years = Années pendant lesquelles un incident clos reste dans le registre.
admin-settings-key-breach-auto-enabled = Automatisations
admin-settings-help-breach-auto-enabled = Ouvre des incidents brouillons à partir des détections.
admin-settings-key-breach-auto-start-clock = Compte à rebours des brouillons
admin-settings-help-breach-auto-start-clock = Démarre le compte à rebours à l'ouverture du brouillon, ou seulement à sa confirmation.
admin-settings-key-breach-auto-assignee = Attribuer les brouillons à
admin-settings-help-breach-auto-assignee = La personne à qui un brouillon est attribué.
admin-settings-key-breach-auto-escalate-after-hours = Escalader après (heures)
admin-settings-help-breach-auto-escalate-after-hours = Heures qu'un brouillon attend avant d'être escaladé. 0 n'escalade jamais.
admin-settings-key-breach-auto-escalate-to = Escalader vers
admin-settings-help-breach-auto-escalate-to = La personne à qui un brouillon escaladé est attribué.
admin-settings-key-breach-auto-window-minutes = Fenêtre de comptage en minutes
admin-settings-help-breach-auto-window-minutes = Minutes pendant lesquelles une règle compte les événements.
admin-settings-key-breach-auto-theft-threshold = Alertes de vol de session
admin-settings-help-breach-auto-theft-threshold = Alertes dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spraying ou pic de connexions
admin-settings-help-breach-auto-credential-stuffing-threshold = Événements dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-mass-forward-threshold = Règles de transfert en masse
admin-settings-help-breach-auto-mass-forward-threshold = Événements dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-role-escalation-threshold = Changements de rôle d'administrateur
admin-settings-help-breach-auto-role-escalation-threshold = Changements dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-export-volume-threshold = Exports inhabituels
admin-settings-help-breach-auto-export-volume-threshold = Exports dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-audit-chain-threshold = Échecs de la chaîne d'audit
admin-settings-help-breach-auto-audit-chain-threshold = Échecs qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-backup-anomaly-threshold = Échecs de sauvegarde ou de restauration
admin-settings-help-breach-auto-backup-anomaly-threshold = Échecs dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-breach-auto-dsr-export-threshold = Nombreux exports de demandes relatives aux données
admin-settings-help-breach-auto-dsr-export-threshold = Exports dans la fenêtre qui ouvrent un brouillon. 0 désactive.
admin-settings-key-ratelimit-queue-ms = Attente maximale d'une place libre
admin-settings-help-ratelimit-queue-ms = Durée maximale pendant laquelle une requête attend une place libre (concurrence JMAP, décodeurs d'images, workers d'aperçu) avant d'être refusée avec 429 et Retry-After. Plage 0-30000 ms ; par défaut 3000. Zéro refuse immédiatement.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Requêtes JMAP simultanées par compte
admin-settings-help-ratelimit-jmap-per-account-concurrent = Requêtes JMAP qu'un compte connecté peut exécuter en même temps, sur l'ensemble de ses onglets et appareils. Elles ne sont jamais partagées avec un autre compte ni un autre utilisateur derrière la même adresse. Plage 4-64 ; par défaut 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Requêtes JMAP simultanées, tous comptes
admin-settings-help-ratelimit-jmap-global-concurrent = Requêtes JMAP que l'ensemble du processus serveur exécute en même temps, tous comptes confondus. Une requête au-delà attend le temps de file. Plage 4-4096 ; par défaut 256.
admin-settings-key-ratelimit-session-burst = Rapports de chronométrage par session d'affilée
admin-settings-help-ratelimit-session-burst = Rapports de chronométrage client qu'une session peut envoyer d'affilée. Le point de terminaison est indexé par session, jamais par adresse. Plage 1-1000 ; par défaut 60.
admin-settings-key-ratelimit-session-per-minute = Rapports de chronométrage récupérés par minute
admin-settings-help-ratelimit-session-per-minute = Rapports de chronométrage client par minute qu'une session récupère après sa rafale. Plage 1-600 ; par défaut 30.
admin-settings-key-ratelimit-anon-burst = Requêtes de consentement anonymes d'affilée
admin-settings-help-ratelimit-anon-burst = Requêtes qu'une adresse peut faire d'affilée vers les routes anonymes de consentement aux cookies. Dimensionné pour un bureau derrière une même adresse. Plage 1-1000 ; par défaut 60.
admin-settings-key-ratelimit-anon-per-minute = Requêtes de consentement anonymes récupérées par minute
admin-settings-help-ratelimit-anon-per-minute = Requêtes par minute qu'une adresse récupère sur les routes anonymes de consentement aux cookies. Plage 1-600 ; par défaut 30.
admin-settings-key-ratelimit-trusted-multiplier = Quota du compte depuis une source connue (%)
admin-settings-help-ratelimit-trusted-multiplier = Pourcentage du quota de requêtes du compte dont bénéficie une boîte aux lettres connectée lorsque la requête provient d'une source ayant une connexion réussie enregistrée. 100 conserve le quota de base. N'assouplit jamais les budgets anonymes, d'administrateur ou de user-agent ni un blocage actif. Plage 100-1000 ; par défaut 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Que le fait de cliquer n'importe où sur la barre des pièces jointes du lecteur l'agrandisse ou la réduise. Lorsqu'il est désactivé, seul le bouton fléché le fait.

admin-settings-help-attachments-bar-download-all = Si la barre des pièces jointes propose un bouton Télécharger tout qui enregistre les pièces jointes autorisées dans un seul fichier ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Lorsque chaque pièce jointe de la barre affiche son bouton de téléchargement : en survol ou en focus (par défaut), toujours ou jamais. Les téléchargements suivent toujours la politique de pièces jointes et mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Indique si les images affichées dans le texte du message sont incluses dans Télécharger tout ZIP. Désactivé par défaut.

admin-settings-help-cap-reading-delete-sender = Autorisez les utilisateurs à activer l'action du lecteur qui déplace tous les messages de l'expéditeur correspondant vers la corbeille après confirmation. Désactivé par défaut. Le profil de fonctionnalités doit également le permettre.

admin-settings-help-compose-feature-family = Activez ou désactivez un contrôle Composer par identifiant de fonctionnalité. Activé par défaut ; verrouillez-le pour empêcher les utilisateurs de le modifier.

admin-settings-help-compose-headers-allow-auto-submitted = Permet aux utilisateurs de définir la soumission automatique (RFC 3834) sur un message. Désactivé par défaut : il indique aux autres systèmes de ne pas répondre.

admin-settings-help-compose-headers-allow-custom = Permet aux utilisateurs d'ajouter leurs propres en-têtes (jusqu'à 20, noms vérifiés, en-têtes protégés refusés).

admin-settings-help-compose-headers-allow-return-receipt-to = Permet aux utilisateurs de demander des reçus avec l’ancien en-tête Return-Receipt-To. Désactivé par défaut.

admin-settings-help-compose-headers-allow-threading-edit = Permet aux utilisateurs de modifier les réponses aux réponses et les références, ou de démarrer un nouveau fil de discussion à partir d'une réponse.

admin-settings-help-compose-headers-forced = Une paire Nom=valeur par ligne, ajoutée à chaque message envoyé depuis le domaine et remplaçant la valeur de l'utilisateur. Les en-têtes protégés sont refusés.

admin-settings-help-compose-insert-image-resize-default = La taille par laquelle commence la boîte de dialogue Image à partir du fichier : originale, grande (1 600 px) ou moyenne (1 024 px).

admin-settings-help-compose-insert-max-inline-image-mb = Les images plus grandes ne sont pas placées dans le corps du message ; ils peuvent toujours être attachés. 1 à 50.

admin-settings-help-compose-layout-default-font = Famille de police, taille et couleur avec lesquelles un nouveau message commence, sous la forme famille|taille|#couleur. Empty utilise la police du thème.

admin-settings-help-compose-reply-to-allow-override = Lorsqu'elle est désactivée, la réponse à l'identité est utilisée telle quelle et les utilisateurs ne peuvent pas la modifier ou la supprimer.

admin-settings-help-compose-reply-to-show = Indique si le compositeur affiche la ligne Répondre à : uniquement lorsqu'une identité ou le message d'origine en a une, toujours ou jamais.

admin-settings-help-compose-sent-folder-fixed-id = Rôle ou identifiant de dossier utilisé lorsque la règle est « dossier fixe ».

admin-settings-help-compose-sent-folder-mode = Où va la copie d'un message envoyé : le dossier Envoyé normal, un dossier fixe, par domaine du destinataire, par identité ou par règles de sujet.

admin-settings-help-compose-sent-folder-per-identity = Une paire identité=dossier par ligne. Utilisé lorsque la règle est « par identité ».

admin-settings-help-compose-sent-folder-rules = Une règle par ligne, domain:example.com=dossier ou subject:text=dossier. Le premier match gagne.

admin-settings-help-compose-sent-folder-skip-copy = Ne conservez jamais de copie d’un message envoyé. Un administrateur peut forcer cette activation ou désactivation.

admin-settings-help-compose-styles-custom = Les propres styles de compositeur d'un utilisateur sous la forme JSON, écrits par la galerie Styles. Les utilisateurs le définissent ; il n'y a rien à configurer ici.

admin-settings-help-compose-styles-default-set = L'apparence de la galerie de styles du compositeur : par défaut, moderne, classique ou compacte.

admin-settings-help-compose-styles-max-custom = La plupart des styles personnalisés qu'un utilisateur peut conserver dans le compositeur. 0 désactive la création de style.

admin-settings-help-contacts-save-from-message-add-note = Ajoutez la note « Enregistré à partir d'un message le <date> » aux contacts enregistrés à partir d'un message. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-contacts-save-from-message-default-book = L'identifiant du carnet d'adresses qui reçoit les contacts enregistrés à partir d'un message ; "default" utilise le carnet d'adresses par défaut. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-contacts-save-from-message-default-scope = Quels participants sont cochés en premier dans la boîte de dialogue Enregistrer les contacts : l'expéditeur uniquement, l'expéditeur et Répondre à, ou tout le monde. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-contacts-save-from-message-enabled = Offre Enregistrer les contacts de ce message dans les menus de message. Lorsqu'il est éteint, l'élément est masqué. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-contacts-save-from-message-skip-automated = Laissez les adresses de non-réponse, de notification et de rebond en dehors de la boîte de dialogue Enregistrer les contacts, à moins qu'elles ne soient cochées manuellement. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-contacts-save-sender-enabled = Proposez l'élément Enregistrer l'expéditeur en tant que contact en un clic dans les menus de message. Lorsqu'il est éteint, l'élément est masqué. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-contacts-save-sender-use-reply-to = Enregistrez l'adresse de réponse au lieu de De lorsque l'expéditeur est une liste de diffusion ou une adresse de notification. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-dsr-admin-step-up = Activé (par défaut) : un administrateur qui approuve une demande de données, effectue un effacement ou approuve en masse doit d'abord confirmer à nouveau qu'il s'agit bien de lui (mot de passe et deuxième facteur, ou clé d'accès). La confirmation dure cinq minutes. Désactivé supprime la vérification supplémentaire ; la décision est toujours enregistrée dans le journal des demandes.

admin-settings-help-export-default-format = Le format avec lequel commence la boîte de dialogue d'exportation.

admin-settings-help-export-default-scope = La portée par laquelle commence la boîte de dialogue d'exportation : message, conversation, sélection ou dossier.

admin-settings-help-export-dialog-enabled = Exporter dans la vue des messages et les menus des messages ouvre une boîte de dialogue en place. Lorsqu'ils sont désactivés, ces menus téléchargent le message au format .eml.

admin-settings-help-export-folder-enabled = Autoriser les utilisateurs à télécharger un dossier entier sous forme de ZIP de messages. Chaque export est audité.

admin-settings-help-export-folder-format = Les formats proposés par un dossier d'exportation : un ZIP de fichiers .eml, un ZIP contenant un fichier mbox, ou les deux.

admin-settings-help-export-folder-include-subfolders = Si une exportation de dossiers inclut ses sous-dossiers par défaut, conservés sous forme de structure de dossiers dans le ZIP.

admin-settings-help-export-folder-max-bytes = Le plus grand nombre d’octets de messages qu’une exportation de dossier peut contenir. Un dossier plus grand est refusé avec la taille indiquée.

admin-settings-help-export-folder-max-messages = Le plus grand nombre de messages qu’une exportation de dossier peut contenir.

admin-settings-help-export-formats = Les formats proposés par la boîte de dialogue d'exportation : eml, mbox, txt, md, msg, docx, csv, json, pdf. Le serveur refuse un format créé par le serveur et répertorié.

admin-settings-help-export-message-download-enabled = Autorisez les utilisateurs à télécharger un message au format .eml, ses en-têtes uniquement ou sous forme de fichier mbox.

admin-settings-help-folders-hide-empty = Masquez les dossiers vides dans la barre latérale. Les utilisateurs peuvent modifier la valeur par défaut ou la remplacer pour des dossiers individuels, sauf si ce paramètre est verrouillé. Aucun courrier n'est supprimé.

admin-settings-help-folders-pinned-hide-from-tree = Les dossiers épinglés s'affichent dans un groupe Épinglé au-dessus de l'arborescence des dossiers et conservent également leur place dans l'arborescence. Activé masque un dossier épinglé qui n’a aucun sous-dossier dans l’arborescence, il s’affiche donc uniquement dans le groupe Épinglé. Un dossier épinglé avec des sous-dossiers reste dans l'arborescence.

admin-settings-help-mail-badge-auth = Lorsque l'en-tête du lecteur affiche le badge d'authentification DMARC : toujours, uniquement lorsque cela est notable, ou jamais.

admin-settings-help-mail-badge-density = Taille des badges dans l'en-tête du lecteur : compact ou confortable.

admin-settings-help-mail-badge-folder = Lorsque l'en-tête du lecteur affiche le badge du dossier : toujours, uniquement lorsque cela est notable, ou jamais.

admin-settings-help-mail-badge-headers = Afficher un badge dans le lecteur pour les en-têtes Expire, Répondre, Sensibilité et Remplacer d'un message.

admin-settings-help-mail-badge-labels = Indique si les badges d'en-tête du lecteur affichent une icône avec du texte ou uniquement l'icône.

admin-settings-help-mail-badge-list = Lorsque l'en-tête du lecteur affiche le badge de la liste de diffusion : toujours, uniquement lorsque cela est notable, ou jamais. Il a besoin d’un en-tête List-Id ou List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Lorsque l'en-tête du lecteur affiche le badge HTML aseptisé : toujours, uniquement lorsque cela est notable, ou jamais.

admin-settings-help-mail-badge-spam = Lorsque l'en-tête du lecteur affiche le badge de score de spam : toujours, uniquement lorsque cela est notable, ou jamais.

admin-settings-help-mail-brand-avatars-require-auth = jamais : afficher un logo chaque fois que le domaine correspond. known-failure (par défaut) : l'afficher à moins que le résultat DKIM ou DMARC mis en cache du message soit un échec ; les messages non cochés conservent le logo. toujours : ne l'afficher qu'après un passage DMARC aligné avec DKIM pour le domaine propre de la marque. Les remplacements d'utilisateur et de domaine peuvent être verrouillés.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Quelques secondes pendant lesquelles le corps doit rester concentré avant qu'un destinataire complet et un en-tête de sujet ne se replient sur son résumé. Choisissez 0 pour désactiver le pliage chronométré, ou 1 à 60 secondes. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-mail-compose-priority-default = La priorité avec laquelle commence un nouveau message dans le compositeur.

admin-settings-help-mail-compose-priority-enabled = Affichez le sélecteur de priorité Normal, Élevé et Faible dans le groupe Options du compositeur. Lorsqu'elle est désactivée, les nouveaux messages sont envoyés sans en-têtes prioritaires.

admin-settings-help-mail-expiry-hide-expired = Que faire d'un message dont la date d'expiration est dépassée : rien, l'afficher en grisé ou proposer de l'archiver.

admin-settings-help-mail-list-keep = Les expéditeurs et les listes de diffusion sont toujours conservés, un par ligne sous la forme sender:address ou list:List-Id. Nettoyez cet expéditeur, désabonnez-vous des suggestions et ignorez-les des règles de conservation.

admin-settings-help-mail-list-page-size = Combien de messages la liste récupère chaque fois qu'un utilisateur fait défiler jusqu'à la fin. Par défaut 50, de 20 à 200. Les utilisateurs choisissent leur propre valeur, sauf si vous la verrouillez.

admin-settings-help-mail-list-unsubscribe-mode = Comment fonctionne la désinscription lorsqu'une liste propose une demande en un clic : demandez d'abord, envoyez-la immédiatement ou ouvrez uniquement le lien ou le brouillon de la liste.

admin-settings-help-mail-message-list-group = Regroupez les conversations par date de réception ou d'envoi, expéditeur, sujet, importance ou taille. Les utilisateurs peuvent ignorer cela à moins qu'ils ne soient verrouillés.

admin-settings-help-mail-priority-marks = Quelle priorité marque la liste des messages, les conversations et l'affichage du lecteur : à la fois élevée et faible, élevée uniquement ou aucune. Les utilisateurs choisissent sous Paramètres > Dossiers et barre latérale, sauf s'ils sont verrouillés.

admin-settings-help-mail-priority-marks-style = À quoi ressemble une marque de priorité : une icône, une icône avec les mots High ou Low, ou une fine barre. L'icône et le texte gardent leur sens sans couleur.

admin-settings-help-mail-priority-send-headers = Quels en-têtes un message de priorité élevée ou faible obtient lorsqu'il est envoyé : Importance, X-Priority et X-MSMail-Priority.

admin-settings-help-mail-priority-trust-sender-headers = Indique si la priorité définie par l'expéditeur est affichée : toujours, uniquement lorsque le message a réussi une vérification alignée DMARC, ou jamais. Utilisez Trusted or Never contre les expéditeurs qui marquent chaque message comme étant hautement prioritaire.

admin-settings-help-mail-reply-from-header-priority = Les en-têtes de message sont lus, dans cet ordre, pour trouver l'adresse à laquelle un message a été envoyé lors du choix de l'identité pour une réponse. Noms : Remis à, X-Original-À, Enveloppe-À, X-Transféré-À, Renvoyé à, À, Cc. Un en-tête laissé de côté n'est pas lu.

admin-settings-help-mail-search-default-scope = Démarrez de nouvelles recherches de courrier normales dans le dossier actuel ou dans tous les dossiers. Ceci contrôle la valeur par défaut initiale, y compris lorsqu'elle est verrouillée ; les utilisateurs peuvent toujours choisir une portée différente pour une recherche particulière. Les requêtes enregistrées et explicitement étendues sont conservées.

admin-settings-help-metrics-dimensions-enabled = Stockez les statistiques enregistrées réparties par type de client (Web) et classe d'appareil (ordinateur de bureau, mobile). Désactivez-la pour stocker une ligne combinée par métrique et par minute.

admin-settings-help-metrics-persist-enabled = Enregistrez les statistiques de performances (vitesse du proxy d'image, temps de chargement du navigateur, nombre de limites de débit et de protection des demandes) dans la base de données à chaque intervalle de vidage et à l'arrêt du serveur, afin que les graphiques conservent leur historique après un redémarrage. Rien n'identifie une personne, une adresse ou un message. Désactivez-les pour les conserver uniquement en mémoire.

admin-settings-help-metrics-persist-flush-seconds = À quelle fréquence les statistiques de performances sont écrites dans la base de données (5 à 3 600 secondes). Un serveur tué sans avertissement perd au maximum cette somme.

admin-settings-help-observability-client-capture-console = Signalez également les appels console.error, au maximum cinq par minute et par page. Désactivé par défaut car les bibliothèques enregistrent les conditions attendues.

admin-settings-help-observability-client-capture-csp = Signalez les violations de la politique de sécurité du contenu détectées par le navigateur, avec la directive et l'origine bloquées uniquement. Activé par défaut.

admin-settings-help-observability-client-enabled = Collectez les rapports d'erreurs de l'application Web (limites d'erreurs, erreurs non détectées, morceaux ayant échoué, erreurs de travail, violations CSP) avec des traces de pile nettoyées, affichées sous Observabilité > Erreurs client. Activé par défaut. Les rapports ne contiennent aucun contenu de message, adresses, jetons, noms de pièces jointes ou requêtes URL. Éteignez pour arrêter la collecte ; les rapports existants expirent avec leur conservation.

admin-settings-help-observability-client-forward = Envoyez également chaque rapport nettoyé au récepteur d’erreurs configuré avec MW_ERROR_FORWARD_URL. Désactivé par défaut. Sans récepteur configuré, rien n'est envoyé.

admin-settings-help-observability-client-include-account-hash = Ajoutez un hachage unidirectionnel du compte aux rapports envoyés à partir d'une session connectée, afin que les rapports d'un compte puissent être distingués sans le nommer. Désactivé par défaut. Le hachage change lorsque la clé de confidentialité tourne.

admin-settings-help-observability-client-max-per-session = La plupart des rapports d'erreurs qu'un chargement de page peut envoyer (1 à 1 000). Les répétitions de la même erreur dans les 60 secondes comptent une fois. Par défaut 20.

admin-settings-help-observability-client-sample-rate = Fraction des rapports d'erreur envoyés par un navigateur, de 0 à 1. Par défaut 1 (tous). Chaque chargement de page s'arrête toujours à la limite par session.

admin-settings-help-observability-client-source-maps = Résolvez les positions du bundle dans les traces de pile vers les fichiers source lorsque la build a expédié des cartes sources cachées (construites avec MW_SOURCEMAP=hidden). Les cartes sont lues uniquement par le serveur et ne sont jamais transmises aux navigateurs. Activé par défaut ; sans cartes, les images brutes sont affichées.

admin-settings-help-privacy-error-reports = Indique si un navigateur envoie un rapport d'erreur lorsqu'une partie de l'application échoue. Activé par défaut. Portée globale, domaine et utilisateur : définissez la valeur par défaut ici ou verrouillez-la ; les gens peuvent le désactiver eux-mêmes dans Paramètres > Confidentialité, à moins qu'il ne soit verrouillé.

admin-settings-help-privacy-image-cache-max-stale-secs = Quelques secondes après sa fraîcheur, une image en cache est conservée pour répondre lorsque l'origine ne peut pas être atteinte. Jamais en dessous de la fenêtre de revalidation. Plage de 0 à 604 800 ; par défaut 86400.

admin-settings-help-privacy-image-cache-stale-secs = Quelques secondes après sa fraîcheur, une image mise en cache est toujours diffusée immédiatement pendant qu'elle est revalidée en arrière-plan. Zero l'éteint. Plage de 0 à 86 400 ; par défaut 3600.

admin-settings-help-privacy-image-resize-enabled = Réduisez les images distantes surdimensionnées à la largeur demandée par le lecteur (320, 640, 960, 1 280 ou 1 600 pixels). Les images animées sont diffusées sans mise à l'échelle. Désactivé par défaut.

admin-settings-help-privacy-image-resize-format = Sélectionne automatiquement le meilleur encodeur de la construction, à savoir PNG. Automatique par défaut.

admin-settings-help-privacy-image-resize-max-pixels = Les images avec plus de pixels décodés que cela sont diffusées sans mise à l'échelle ; la taille est lue dans l'en-tête avant tout décodage. Plage 100 000 à 6 400 000 ; par défaut 4 000 000.

admin-settings-help-privacy-image-resize-max-width = L’image la plus large produite par le redimensionnement. Plage de 320 à 1 600 ; par défaut 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Comment Paramètres > Sécurité > Sessions nomme le compte d'où provient une session d'accès à la boîte aux lettres liée : l'adresse complète, l'adresse avec la première partie du nom masquée ou rien. La valeur est affichée uniquement au propriétaire des sessions, jamais aux autres utilisateurs. Par défaut : adresse complète.

admin-settings-help-properties-folder-enabled = Proposez des propriétés dans les menus des dossiers et sur le raccourci. Lorsqu'elle est désactivée, la boîte de dialogue n'est pas disponible.

admin-settings-help-properties-folder-notes-enabled = Autoriser une description ou une note sur chaque dossier.

admin-settings-help-properties-folder-notes-max-length = Nombre maximal de caractères qu'une note de dossier peut contenir.

admin-settings-help-properties-folder-overrides-allowed = Quels paramètres un utilisateur peut remplacer pour un seul dossier. Un remplacement non répertorié est masqué et toute valeur stockée est ignorée.

admin-settings-help-properties-folder-sections = Les sections de la boîte de dialogue Propriétés du dossier : général, distant, taille, hors ligne, remplacements, notes, actions. Une section non répertoriée est masquée.

admin-settings-help-properties-folder-size-compute = Lorsque la taille d'un dossier sur le serveur est déterminée : jamais, à l'ouverture de la boîte de dialogue ou en arrière-plan. Compter une grande boîte aux lettres coûte une analyse au serveur.

admin-settings-help-properties-headers-raw = Autorisez la boîte de dialogue Propriétés du message à répertorier les en-têtes bruts. Lorsqu'elle est désactivée, seul le résumé analysé s'affiche.

admin-settings-help-properties-message-enabled = Propriétés de l'offre dans les menus des messages et sur le raccourci. Lorsqu'elle est désactivée, la boîte de dialogue n'est pas disponible.

admin-settings-help-properties-message-permanent-delete = Indique si la boîte de dialogue de message peut supprimer définitivement un message : autoriser, demander deux fois ou refuser. Un message sous conservation légale n’est jamais supprimé. La suppression de cette manière ne peut pas être annulée.

admin-settings-help-properties-message-sections = Les sections de la boîte de dialogue Propriétés du message : résumé, score, en-têtes, pièces jointes, actions. Une section non répertoriée est masquée.

admin-settings-help-properties-offline-clear-enabled = Autoriser les utilisateurs à vider le cache hors ligne d'un seul dossier à partir de sa boîte de dialogue Propriétés.

admin-settings-help-properties-remote-location-visibility = Quelle quantité d'emplacement côté serveur les utilisateurs voient : complet (hôte et chemin), hôte masqué (hôte partiellement caché), chemin uniquement (chemin de la boîte aux lettres sans l'hôte) ou masqué.

admin-settings-help-properties-scoring-show = Afficher le score de spam, les résultats d'authentification et les signaux de sécurité dans la boîte de dialogue Propriétés du message.

admin-settings-help-properties-shortcut-enabled = Ouvrez la boîte de dialogue Propriétés à partir du clavier sur un dossier ciblé ou un message sélectionné.

admin-settings-help-properties-shortcut-key = La combinaison de touches qui ouvre les propriétés ; Alt+Entrée par défaut.

admin-settings-help-reader-actions-archive = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-block-sender = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-colors = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-conversation = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-create-filter = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-delete = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-delete-sender = Masqué par défaut. Choisissez la barre d'outils ou le menu Plus pour l'activer lorsque la fonctionnalité de suppression de l'expéditeur et le profil de fonctionnalité le permettent. Les utilisateurs peuvent remplacer le placement à moins qu'ils ne soient verrouillés.

admin-settings-help-reader-actions-edit-draft = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-export = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-finish-followup = Choisissez la barre d'outils, le menu Plus ou masqué pour compléter les messages signalés et les rappels de suivi.

admin-settings-help-reader-actions-flag = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-focus-focused = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-focus-other = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-forward = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-headers = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-links = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-move = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-not-spam = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-open-draft = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-pin = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-print = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-properties = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-read = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-reply = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-reply-all = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-save-contacts = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-save-sender = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-search = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-security = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-spam = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-sweep = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-view-source = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-reader-actions-zoom = Choisissez la barre d'outils, le menu Plus ou masqué. Les restrictions de fonctionnalités, les autorisations et le contexte des messages s'appliquent toujours. Les utilisateurs peuvent choisir à moins que cela ne soit verrouillé.

admin-settings-help-retention-client-errors-event-days = Jours pendant lesquels une occurrence d'erreur de navigateur individuelle (sa trace de pile et sa classe de navigateur) est conservée. Par défaut 30. Un groupe est conservé plus longtemps, voir le paramètre suivant.

admin-settings-help-retention-client-errors-group-days = Nombre de jours pendant lesquels un groupe d'erreurs de navigateur (nom, message, nombre) est conservé après sa dernière visualisation. Par défaut 180. Jamais plus court que la rétention d'occurrence.

admin-settings-help-retention-metrics-day-buckets-days = Jours de statistiques de performances quotidiennes conservés avant leur suppression. 0 les conserve sans limite, ce qui permet à la table de croître lentement tant que le serveur fonctionne.

admin-settings-help-retention-metrics-hour-buckets-days = Jours de statistiques de performances horaires conservés avant d'être fusionnés en lignes quotidiennes (au moins 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Heures de statistiques de performances d'une minute conservées avant d'être fusionnées en lignes horaires (au moins 1).

admin-settings-help-retention-user-allow-strip-attachments = Indique si les utilisateurs peuvent créer des règles permettant de supprimer les pièces jointes volumineuses des anciens messages. Désactivé par défaut. Lorsqu'elle est désactivée, l'action est masquée dans l'éditeur de règles, une règle qui l'utilise est refusée lors de son enregistrement et les règles enregistrées précédemment sont ignorées lors de l'exécution du travail.

admin-settings-help-retention-user-calendar-enabled = Indique si les utilisateurs peuvent choisir des calendriers dont les événements sont supprimés un nombre défini de jours après leur fin. Désactivé par défaut. Lorsqu'elle est désactivée, l'option est masquée dans Paramètres, un choix est refusé une fois enregistré et les choix enregistrés précédemment ne sont pas exécutés.

admin-settings-help-retention-user-calendar-max-days = La plupart du temps, un utilisateur peut attendre avant que les événements terminés soient supprimés. Une valeur plus élevée est refusée lors de l'enregistrement et réduite lors de son exécution.

admin-settings-help-retention-user-calendar-min-days = Nombre de jours après la fin d'un événement avant qu'une règle utilisateur puisse le supprimer. Une valeur plus petite est refusée lors de l'enregistrement et augmentée lors de son exécution.

admin-settings-help-retention-user-notes-enabled = La possibilité pour les utilisateurs de supprimer les notes n'a pas changé pendant un nombre de jours défini. Désactivé par défaut. Les notes supprimées peuvent être restaurées jusqu'à la fin de la fenêtre d'annulation des règles de conservation.

admin-settings-help-retention-user-notes-max-days = La plupart du temps, un utilisateur peut attendre avant que les notes intactes ne soient supprimées. Une valeur plus élevée est refusée lors de l'enregistrement et réduite lors de son exécution.

admin-settings-help-retention-user-notes-min-days = Nombre de jours pendant lequel une note doit rester intacte avant qu'une règle utilisateur puisse la supprimer. Une valeur plus petite est refusée lors de l'enregistrement et augmentée lors de son exécution.

admin-settings-help-retention-user-strip-keep-inline-images = Indique si les images affichées dans le corps du message sont conservées lorsque les pièces jointes sont supprimées.

admin-settings-help-retention-user-strip-min-age-days = Nombre de jours minimum pendant lequel un message doit être ancien avant qu'une règle puisse supprimer ses pièces jointes. Une valeur inférieure est refusée lorsque la règle est enregistrée et levée lors de son exécution.

admin-settings-help-retention-user-strip-min-attachment-kb = Seules les pièces jointes d'au moins cette taille sont supprimées. Les pièces jointes plus petites restent avec le message.

admin-settings-help-retention-user-strip-undo-days = Nombre de jours pendant lesquels le message d'origine reste dans la corbeille après la suppression de ses pièces jointes. -1 suit la fenêtre d'annulation pour les règles de conservation. 0 supprime l'original d'un coup.

admin-settings-help-server-shutdown-grace-seconds = Durée pendant laquelle un serveur en arrêt laisse les requêtes en cours se terminer avant de fermer les connexions, en secondes. La variable MW_HTTP_SHUTDOWN_TIMEOUT_MS la remplace. S'applique au prochain départ.

admin-settings-help-session-last-activity-flush-seconds = L'heure de dernière visite d'une session est écrite au plus souvent, en secondes. Zero l'écrit à chaque demande. Cela ne modifie pas les limites de ralenti ou absolues. S'applique au prochain départ.

admin-settings-help-session-persist-across-restarts = Lorsqu'elle est activée, les sessions de boîte aux lettres, d'application et de console restent valides après le redémarrage du serveur. Lorsqu'elle est désactivée, chaque session se termine au démarrage du serveur. S'applique au prochain départ.

admin-settings-key-attachments-bar-click-to-toggle = Cliquez sur la barre des pièces jointes pour basculer

admin-settings-key-attachments-bar-download-all = Bouton Télécharger tout

admin-settings-key-attachments-bar-show-download-buttons = Boutons de téléchargement des pièces jointes

admin-settings-key-attachments-bar-zip-inline-images = Images en ligne dans Télécharger tout ZIP

admin-settings-key-cap-reading-delete-sender = Autoriser la suppression de tous les messages d'un expéditeur

admin-settings-key-compose-feature-family = Fonctionnalités du compositeur

admin-settings-key-compose-headers-allow-auto-submitted = Autoriser la soumission automatique

admin-settings-key-compose-headers-allow-custom = Autoriser les en-têtes personnalisés

admin-settings-key-compose-headers-allow-return-receipt-to = Autoriser l'accusé de réception

admin-settings-key-compose-headers-allow-threading-edit = Autoriser la modification des en-têtes de thread

admin-settings-key-compose-headers-forced = En-têtes forcés

admin-settings-key-compose-insert-image-resize-default = Taille d'image prédéfinie

admin-settings-key-compose-insert-max-inline-image-mb = Plus grande image en ligne (Mo)

admin-settings-key-compose-layout-default-font = Police des messages par défaut

admin-settings-key-compose-reply-to-allow-override = Autoriser la modification de la réponse à

admin-settings-key-compose-reply-to-show = Ligne de réponse

admin-settings-key-compose-sent-folder-fixed-id = Copie envoyée : dossier corrigé

admin-settings-key-compose-sent-folder-mode = Copie envoyée : règle de dossier

admin-settings-key-compose-sent-folder-per-identity = Copie envoyée : dossier par identité

admin-settings-key-compose-sent-folder-rules = Copie envoyée : règles

admin-settings-key-compose-sent-folder-skip-copy = Copie envoyée : ne pas enregistrer

admin-settings-key-compose-styles-custom = Styles personnalisés

admin-settings-key-compose-styles-default-set = Ensemble de styles

admin-settings-key-compose-styles-max-custom = Styles personnalisés par utilisateur

admin-settings-key-contacts-save-from-message-add-note = Note sur les contacts enregistrés à partir d'un message

admin-settings-key-contacts-save-from-message-default-book = Carnet d'adresses pour les contacts enregistrés à partir d'un message

admin-settings-key-contacts-save-from-message-default-scope = Contacts cochés en premier lors de l'enregistrement d'un message

admin-settings-key-contacts-save-from-message-enabled = Enregistrer les contacts d'un message

admin-settings-key-contacts-save-from-message-skip-automated = Ignorer les expéditeurs automatisés lors de l'enregistrement des contacts

admin-settings-key-contacts-save-sender-enabled = Enregistrer l'expéditeur comme contact

admin-settings-key-contacts-save-sender-use-reply-to = Enregistrer la réponse pour les expéditeurs de la liste

admin-settings-key-dsr-admin-step-up = Confirmez à nouveau avant d'approuver

admin-settings-key-export-default-format = Format par défaut de la boîte de dialogue d'exportation

admin-settings-key-export-default-scope = Portée par défaut de la boîte de dialogue Exporter

admin-settings-key-export-dialog-enabled = Boîte de dialogue Exporter

admin-settings-key-export-folder-enabled = Exportation de dossiers

admin-settings-key-export-folder-format = Format d'exportation de dossier

admin-settings-key-export-folder-include-subfolders = Inclure les sous-dossiers dans l'exportation

admin-settings-key-export-folder-max-bytes = Limite de taille d'exportation de dossier

admin-settings-key-export-folder-max-messages = Limite des messages d'exportation de dossiers

admin-settings-key-export-formats = Exporter les formats de dialogue

admin-settings-key-export-message-download-enabled = Téléchargement des messages

admin-settings-key-folders-hide-empty = Masquer les dossiers vides

admin-settings-key-folders-pinned-hide-from-tree = Masquer les dossiers épinglés de l'arborescence

admin-settings-key-mail-badge-auth = Badge d'authentification

admin-settings-key-mail-badge-density = Taille du badge du lecteur

admin-settings-key-mail-badge-folder = Insigne de dossier

admin-settings-key-mail-badge-headers = Insignes d'en-tête

admin-settings-key-mail-badge-labels = Etiquettes badge lecteur

admin-settings-key-mail-badge-list = Insigne de liste de diffusion

admin-settings-key-mail-badge-sanitized = Badge HTML aseptisé

admin-settings-key-mail-badge-spam = Badge de score de spam

admin-settings-key-mail-brand-avatars-require-auth = Authentification requise pour les logos de marque

admin-settings-key-mail-compose-header-auto-collapse-seconds = Délai de réduction automatique de l'en-tête du compositeur

admin-settings-key-mail-compose-priority-default = Priorité des messages par défaut

admin-settings-key-mail-compose-priority-enabled = Sélecteur de priorité dans le compositeur

admin-settings-key-mail-expiry-hide-expired = Messages expirés

admin-settings-key-mail-list-keep = Gardez toujours

admin-settings-key-mail-list-page-size = Taille de la page de la liste de messages

admin-settings-key-mail-list-unsubscribe-mode = Liste de diffusion Se désabonner

admin-settings-key-mail-message-list-group = Regroupement des messages

admin-settings-key-mail-priority-marks = Marques de priorité

admin-settings-key-mail-priority-marks-style = Style de marque de priorité

admin-settings-key-mail-priority-send-headers = En-têtes prioritaires écrits lors de l'envoi

admin-settings-key-mail-priority-trust-sender-headers = Priorité définie par l'expéditeur de confiance

admin-settings-key-mail-reply-from-header-priority = En-têtes qui choisissent l'identité de la réponse

admin-settings-key-mail-search-default-scope = Portée de recherche de courrier par défaut

admin-settings-key-metrics-dimensions-enabled = Répartir les statistiques par type de client et classe d'appareil

admin-settings-key-metrics-persist-enabled = Conserver des statistiques lors des redémarrages

admin-settings-key-metrics-persist-flush-seconds = Intervalle de sauvegarde des statistiques (secondes)

admin-settings-key-observability-client-capture-console = Signaler les erreurs de la console

admin-settings-key-observability-client-capture-csp = Signaler les violations CSP

admin-settings-key-observability-client-enabled = Rapports d'erreurs du navigateur

admin-settings-key-observability-client-forward = Transférer les rapports d'erreurs

admin-settings-key-observability-client-include-account-hash = Ajouter le hachage du compte aux rapports d'erreurs

admin-settings-key-observability-client-max-per-session = Rapports d'erreurs par chargement de page

admin-settings-key-observability-client-sample-rate = Taux d'échantillonnage du rapport d'erreur

admin-settings-key-observability-client-source-maps = Résoudre les traces de pile avec des mappages sources

admin-settings-key-privacy-error-reports = Envoyer des rapports d'erreurs

admin-settings-key-privacy-image-cache-max-stale-secs = Rester obsolète lorsque l'origine échoue (secondes)

admin-settings-key-privacy-image-cache-stale-secs = Servir rassis tout en revalidant (secondes)

admin-settings-key-privacy-image-resize-enabled = Redimensionner les grandes images

admin-settings-key-privacy-image-resize-format = Format d'image redimensionné

admin-settings-key-privacy-image-resize-max-pixels = Plus grande image à redimensionner (pixels)

admin-settings-key-privacy-image-resize-max-width = Image redimensionnée la plus large (pixels)

admin-settings-key-privacy-sessions-show-linked-origin = Origine de la boîte aux lettres liée dans les sessions

admin-settings-key-properties-folder-enabled = Boîte de dialogue Propriétés du dossier

admin-settings-key-properties-folder-notes-enabled = Notes de dossier

admin-settings-key-properties-folder-notes-max-length = Longueur des notes du dossier

admin-settings-key-properties-folder-overrides-allowed = Remplacements par dossier autorisés

admin-settings-key-properties-folder-sections = Sections de la boîte de dialogue des dossiers

admin-settings-key-properties-folder-size-compute = Calcul de la taille du dossier

admin-settings-key-properties-headers-raw = En-têtes bruts

admin-settings-key-properties-message-enabled = Boîte de dialogue Propriétés du message

admin-settings-key-properties-message-permanent-delete = Suppression définitive des propriétés

admin-settings-key-properties-message-sections = Sections de la boîte de dialogue des messages

admin-settings-key-properties-offline-clear-enabled = Effacer le cache hors ligne par dossier

admin-settings-key-properties-remote-location-visibility = Détails de l'emplacement distant

admin-settings-key-properties-scoring-show = Détails de notation et d'authentification

admin-settings-key-properties-shortcut-enabled = Raccourci des propriétés

admin-settings-key-properties-shortcut-key = Touche de raccourci Propriétés

admin-settings-key-reader-actions-archive = Placement des actions d'archivage

admin-settings-key-reader-actions-block-sender = Bloquer le placement de l'action de l'expéditeur

admin-settings-key-reader-actions-colors = Placement des actions sur les couleurs des messages

admin-settings-key-reader-actions-conversation = Afficher l'emplacement des actions de conversation

admin-settings-key-reader-actions-create-filter = Créer un placement d'action de filtre

admin-settings-key-reader-actions-delete = Supprimer l'emplacement de l'action

admin-settings-key-reader-actions-delete-sender = Supprimer tous les messages de l'action de l'expéditeur

admin-settings-key-reader-actions-edit-draft = Modifier le placement d'une action brouillon

admin-settings-key-reader-actions-export = Emplacement de l'action d'exportation

admin-settings-key-reader-actions-finish-followup = Marquer le suivi comme une action terminée

admin-settings-key-reader-actions-flag = Marquer/démarquer le placement d'une action

admin-settings-key-reader-actions-focus-focused = Passer au placement d'action ciblée

admin-settings-key-reader-actions-focus-other = Déplacer vers un autre emplacement d'action

admin-settings-key-reader-actions-forward = Placement d'action avant

admin-settings-key-reader-actions-headers = Placement des actions des en-têtes bruts

admin-settings-key-reader-actions-links = Examiner le placement des actions des liens

admin-settings-key-reader-actions-move = Déplacer le placement de l'action

admin-settings-key-reader-actions-not-spam = Pas de placement d'action de spam

admin-settings-key-reader-actions-open-draft = Placement d'action de brouillon de réponse ouverte

admin-settings-key-reader-actions-pin = Épingler/désépingler le placement de l'action

admin-settings-key-reader-actions-print = Placement des actions d'impression

admin-settings-key-reader-actions-properties = Placement des actions sur les propriétés du message

admin-settings-key-reader-actions-read = Placement des actions lues/non lues

admin-settings-key-reader-actions-reply = Placement de l'action de réponse

admin-settings-key-reader-actions-reply-all = Répondre à tous les placements d'actions

admin-settings-key-reader-actions-save-contacts = Enregistrer le placement de l'action des contacts du message

admin-settings-key-reader-actions-save-sender = Enregistrer le placement de l'action de l'expéditeur

admin-settings-key-reader-actions-search = Emplacement des actions de recherche associées

admin-settings-key-reader-actions-security = Placement des actions en mode sécurité

admin-settings-key-reader-actions-spam = Marquer comme placement d'action spam

admin-settings-key-reader-actions-sweep = Placement de l'action de l'expéditeur par balayage

admin-settings-key-reader-actions-view-source = Afficher le placement de l'action de la source du message

admin-settings-key-reader-actions-zoom = Placement des actions de zoom

admin-settings-key-retention-client-errors-event-days = Occurrences d'erreur conservées (jours)

admin-settings-key-retention-client-errors-group-days = Groupes d'erreurs conservés (jours)

admin-settings-key-retention-metrics-day-buckets-days = Statistiques quotidiennes conservées (jours)

admin-settings-key-retention-metrics-hour-buckets-days = Statistiques horaires conservées (jours)

admin-settings-key-retention-metrics-minute-buckets-hours = Statistiques minutes conservées (heures)

admin-settings-key-retention-user-allow-strip-attachments = Les utilisateurs peuvent supprimer les pièces jointes des anciens messages

admin-settings-key-retention-user-calendar-enabled = Les utilisateurs peuvent supprimer les événements de calendrier terminés

admin-settings-key-retention-user-calendar-max-days = La plupart des jours précédant la fin des événements sont supprimés

admin-settings-key-retention-user-calendar-min-days = Quelques jours avant la fin des événements sont supprimés

admin-settings-key-retention-user-notes-enabled = Les utilisateurs peuvent supprimer les notes intactes

admin-settings-key-retention-user-notes-max-days = La plupart du temps avant que les notes intactes ne soient supprimées

admin-settings-key-retention-user-notes-min-days = Quelques jours avant la suppression des notes intactes

admin-settings-key-retention-user-strip-keep-inline-images = Conserver les images en ligne

admin-settings-key-retention-user-strip-min-age-days = Quelques jours avant la suppression des pièces jointes

admin-settings-key-retention-user-strip-min-attachment-kb = Plus petite pièce jointe supprimée (Ko)

admin-settings-key-retention-user-strip-undo-days = Fenêtre d'annulation après la suppression des pièces jointes (jours)

admin-settings-key-server-shutdown-grace-seconds = Délai de grâce d'arrêt

admin-settings-key-session-last-activity-flush-seconds = Intervalle d'écriture pour la dernière fois

admin-settings-key-session-persist-across-restarts = Conserver les sessions lors des redémarrages

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = Désactivé par défaut. Enregistrez des instantanés de courrier chiffrés au niveau du compte lors des recharges du navigateur sans expiration de l'âge. Les limites d’entrées et d’octets existantes s’appliquent toujours. Les utilisateurs peuvent effacer les copies enregistrées ; la désactivation de cette préférence les supprime du navigateur actif. Cela n’accorde pas l’autorisation de stockage persistant au navigateur ni le téléchargement de pièces jointes.

admin-settings-help-mail-conversations-allowed = Autorisez les utilisateurs à regrouper les réponses associées dans des conversations. Disponible par défaut, mais la préférence de l'utilisateur démarre. La désactivation de la disponibilité remplace le choix de l'utilisateur sans modifier le regroupement de catégories.

admin-settings-help-mail-conversations-enabled = Désactivé par défaut. Regroupez les réponses chargées par fil de discussion, indépendamment de Grouper par. Définissez la valeur par défaut ou verrouillez cette préférence. La disponibilité des conversations doit également être activée.

admin-settings-key-mail-cache-keep-indefinitely = Conserver le cache de messagerie du navigateur indéfiniment

admin-settings-key-mail-conversations-allowed = Rendre le fil de conversation disponible

admin-settings-key-mail-conversations-enabled = Regrouper les messages en conversations

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = Jusqu'à 100 domaines d'expéditeurs en minuscules exacts dont les liens de messages examinés sont activés automatiquement. Les sous-domaines sont distincts. Cela ne fait pas confiance aux sites Web de destination et n'autorise pas les images distantes. Vide par défaut ; les utilisateurs peuvent le modifier à moins qu'il ne soit verrouillé.

admin-settings-key-links-trusted-sender-domains = Domaines d'expéditeur de confiance pour les liens de messages

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Afficher les détails du téléchargement et du fichier à côté du titre de l'aperçu sous forme d'icônes (par défaut), d'étiquettes ou d'icônes et d'étiquettes. Peut être défini et verrouillé globalement, par domaine ou par utilisateur.

admin-settings-help-reader-attachment-thumbnails = Désactivé par défaut. Autorisez les pièces jointes d’images visibles à charger des aperçus miniatures. Les règles de sécurité des pièces jointes s'appliquent toujours. Peut être défini et verrouillé globalement, par domaine ou par utilisateur.

admin-settings-key-reader-attachment-action-style = Actions d’aperçu des pièces jointes

admin-settings-key-reader-attachment-thumbnails = Vignettes des images jointes

admin-settings-key-mail-compose-auto-add-mention-recipients = Ajouter automatiquement les personnes mentionnées aux destinataires
admin-settings-help-mail-compose-auto-add-mention-recipients = Lorsque vous sélectionnez une personne dans les suggestions de @mentions, elle est ajoutée à À si elle ne figure pas déjà dans À, Cc ou Cci. Désactivé par défaut.

admin-settings-linked-disable-title = Désactiver l’accès enregistré aux comptes liés ?
admin-settings-linked-disable-body = Enregistrer cette modification supprime les identifiants servant à renouveler l’accès aux comptes liés. Les utilisateurs devront se reconnecter lorsque cet accès doit être renouvelé. Réactiver cette option ne restaure pas ces identifiants.
admin-settings-linked-disable-confirm = Désactiver et effacer l’accès enregistré

admin-settings-key-reader-auto-hide-header = Masquer automatiquement les en-têtes des messages
admin-settings-help-reader-auto-hide-header = Ouvrir chaque message avec les détails de l’expéditeur, des destinataires et les métadonnées repliés. L’objet et les actions restent visibles. Utilisez Afficher l’en-tête pour voir les détails. Désactivé par défaut.
