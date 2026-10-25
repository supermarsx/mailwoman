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
