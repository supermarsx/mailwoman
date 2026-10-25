# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Paramètres
mail-compose = Nouveau message
mail-nav-mailboxes = Dossiers
mail-nav-attachments = Pièces jointes
mail-nav-outbox = Boîte d’envoi
mail-nav-sharing = Partager le dossier
mail-nav-apps = Applications
mail-logout = Se déconnecter
mail-offline = Hors ligne
mail-module-loading = Chargement de { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Rechercher
mail-search-label = Rechercher dans le courrier
mail-search-placeholder = Rechercher — from:alice subject:facture larger:1mb
mail-search-clear = Effacer

# -- Message list ------------------------------------------------------------
mail-list-label = Messages
mail-loading = Chargement des messages…
mail-empty = Aucun message
mail-unknown-sender = (expéditeur inconnu)
mail-no-subject = (sans objet)
mail-pinned = Épinglé
mail-has-attachment = Contient une pièce jointe
mail-answered = Répondu
mail-forwarded = Transféré
mail-unread = Non lu
mail-flagged = Suivi
mail-selected = Sélectionné
mail-row-position = Message { $pos } sur { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } message
   *[other] { $count } messages
}
mail-thread-expand = Développer la conversation, { $count ->
    [one] { $count } message
   *[other] { $count } messages
}
mail-thread-collapse = Réduire la conversation, { $count ->
    [one] { $count } message
   *[other] { $count } messages
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Options d’affichage
mail-reading-pane = Volet de lecture
mail-reading-pane-right = À droite
mail-reading-pane-bottom = En bas
mail-reading-pane-off = Désactivé

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Messages sélectionnés
mail-bulk-count = { $count ->
    [one] { $count } sélectionné
   *[other] { $count } sélectionnés
}
mail-mark-read = Marquer comme lu
mail-mark-unread = Marquer comme non lu
mail-flag-message = Marquer
mail-unflag-message = Retirer l’indicateur
mail-move-to = Déplacer vers un dossier
mail-move-to-placeholder = Déplacer vers…
mail-select-all = Tout sélectionner
mail-clear-selection = Désélectionner

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = Boîte de réception Prioritaire
mail-inbox-filter = Filtre de la boîte de réception
mail-inbox-focused = Prioritaire
mail-inbox-other = Autres
mail-inbox-turn-off = Désactiver

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Onglets ouverts
mail-subtab-pin = Épingler { $title }
mail-subtab-unpin = Détacher { $title }
mail-subtab-tearoff = Ouvrir { $title } dans une nouvelle fenêtre
mail-subtab-close = Fermer { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Épingler
mail-unpin = Détacher
mail-snooze = Reporter
mail-snooze-menu = Reporter jusqu’à
mail-snooze-later = Plus tard aujourd’hui
mail-snooze-tomorrow = Demain
mail-snooze-next-week = La semaine prochaine
mail-unsnooze = Annuler le report
mail-label = Étiquette
mail-labels-menu = Étiquettes
mail-flag = Marquer pour suivi
mail-clear-flag = Effacer le suivi
mail-archive = Archiver
mail-delete = Supprimer
mail-more-actions = Plus d’actions

# -- Reader ------------------------------------------------------------------
mail-reader-label = Message
mail-reader-empty = Sélectionnez un message à lire
mail-back = Retour
mail-reader-from = De : { $addr }
mail-reader-to = À : { $addr }
mail-reader-cc = Cc : { $addr }
mail-reader-actions = Actions sur le message
mail-reply = Répondre
mail-reply-all = Répondre à tous
mail-forward = Transférer
mail-reply-failed = Le message n’a pas pu être préparé. Réessayez.
mail-reply-unthreaded = L’identifiant du message d’origine n’a pas pu être lu. Cette réponse ne sera pas rattachée à sa conversation.
mail-spam = Spam
mail-export = Exporter
mail-sweep-sender = Nettoyer l’expéditeur
mail-message-body = Corps du message
mail-no-content = Aucun contenu
mail-sanitizing = Nettoyage…
mail-attachments = Pièces jointes
mail-attachment-unnamed = (sans nom)
mail-attachment-open = Pièce jointe { $name }
mail-attachment-close = Fermer la pièce jointe
mail-attachment-loading = Chargement de la pièce jointe…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Message chiffré
mail-encrypted-note = 🔒 Ce message est chiffré de bout en bout. Déverrouillez-le sur cet appareil pour le lire.
mail-key-passphrase = Phrase secrète de la clé
mail-decrypt = Déchiffrer
mail-decrypting = Déchiffrement…
mail-decrypt-no-key = Aucune clé privée n’est disponible pour déchiffrer ce message.
mail-decrypt-failed = Échec du déchiffrement

# -- Reply and forward: text written into the outgoing message --------------
mail-quote-attribution = Le { $date }, { $name } a écrit :
mail-quote-attribution-undated = { $name } a écrit :
mail-forward-heading = ---------- Message transféré ----------
mail-forward-from = De : { $addr }
mail-forward-date = Date : { $date }
mail-forward-subject = Objet : { $subject }
mail-forward-to = À : { $addr }
mail-forward-cc = Cc : { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Rédiger un message
mail-compose-title = Nouveau message
mail-compose-title-reply = Répondre
mail-compose-title-reply-all = Répondre à tous
mail-compose-title-forward = Transférer
mail-compose-close = Fermer
mail-compose-from = De
mail-compose-from-default = Par défaut
mail-compose-to = À
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Suggestions de contacts
mail-compose-cc = Cc
mail-compose-bcc = Cci
mail-compose-show-cc-bcc = Ajouter Cc ou Cci
mail-compose-bcc-encrypted = Un message chiffré indique la clé de chaque destinataire ; un destinataire en Cci serait donc visible des autres. Retirez les destinataires en Cci ou désactivez le chiffrement.
mail-compose-plaintext-confirm-title = Envoyer du texte déchiffré sans chiffrement ?
mail-compose-plaintext-confirm-body = Ce message cite le texte d’un message chiffré. Il n’est pas configuré pour être chiffré : le texte cité serait donc envoyé sans protection.
mail-compose-plaintext-confirm-send = Envoyer sans chiffrement
mail-compose-plaintext-confirm-cancel = Revenir
mail-compose-subject = Objet
mail-compose-body = Corps
mail-compose-attachments = Pièces jointes
mail-compose-remove-attachment = Retirer { $name }
mail-compose-attach-nextcloud = Joindre depuis Nextcloud
mail-compose-close-nextcloud = Fermer Nextcloud
mail-compose-attach-file = Joindre un fichier
mail-compose-uploading = Envoi du fichier…
mail-compose-upload-unavailable = L’envoi de fichiers n’est pas disponible pour le moment.
mail-compose-upload-failed = Impossible d’envoyer { $name }.
mail-compose-upload-too-large = { $name } fait { $size } Mo. La taille maximale d’envoi est de { $max } Mo.
mail-compose-send-later = Envoyer plus tard
mail-compose-cancel = Annuler
mail-compose-send = Envoyer
mail-compose-schedule = Programmer
mail-compose-sending = Envoi…
mail-compose-encrypted-subject = Message chiffré
mail-compose-dlp-blocked = L’envoi est bloqué par une règle de prévention des pertes de données (voir l’avertissement ci-dessus).
mail-compose-send-failed = Échec de l’envoi
mail-compose-send-later-past = Choisissez une heure d’envoi dans le futur, ou videz le champ pour envoyer maintenant.
mail-compose-options = Options d’envoi
mail-compose-request-receipt = Demander un accusé de lecture
mail-compose-request-receipt-hint = Le logiciel de messagerie du destinataire peut lui demander son accord, en envoyer un sans demander ou ignorer la demande.
mail-compose-track-opens = Enregistrer quand ce message est ouvert
mail-compose-track-opens-hint = Une image de 1×1 chargée depuis ce serveur est ajoutée au message. Beaucoup de logiciels de messagerie bloquent les images ou les chargent à l’avance : une ouverture enregistrée ne prouve donc pas que le message a été lu, et l’absence d’enregistrement ne prouve pas qu’il ne l’a pas été. Le destinataire n’en est pas informé.
mail-compose-image-type = { $name } ne peut pas être placée dans le texte. Utilisez une image PNG, JPEG, GIF ou WebP.
mail-send-not-signed-in = Vous n’êtes pas connecté.
mail-send-no-draft-folder = Ce compte n’a aucun dossier pour enregistrer le brouillon.
mail-send-no-recipient = Ajoutez au moins un destinataire.
mail-send-bad-recipient = { $count ->
    [one] Ceci n’est pas une adresse e-mail : { $addresses }
   *[other] Ce ne sont pas des adresses e-mail : { $addresses }
}
mail-send-later-past = L’heure d’envoi n’est pas dans le futur. Rien n’a été envoyé.
mail-send-tracking-failed = Les ouvertures ne peuvent pas être enregistrées pour le moment, donc rien n’a été envoyé. Décochez « Enregistrer quand ce message est ouvert » pour envoyer sans cette option.
mail-send-draft-refused = Le serveur a refusé ce message : { $reason }. Rien n’a été envoyé.
mail-send-submission-refused = Le serveur a refusé d’envoyer ce message : { $reason }. Rien n’a été envoyé.
mail-compose-sign-unlock-title = Déverrouiller la clé de signature
mail-compose-sign-unlock-note = Saisissez la phrase secrète de votre clé pour signer ce message. Elle reste déverrouillée pour cette fenêtre de rédaction uniquement.
mail-compose-sign-unlock = Déverrouiller
mail-compose-sign-unlocking = Déverrouillage…
mail-compose-sign-no-key = Aucune clé de signature n’est disponible sur cet appareil.
mail-compose-sign-unlock-failed = Impossible de déverrouiller la clé de signature.
mail-compose-sign-unlock-required = Déverrouillez votre clé de signature pour envoyer ce message signé.

mail-compose-format-plain = Texte brut
mail-compose-format-rich = Texte enrichi
mail-compose-rt-toolbar = Mise en forme
mail-compose-rt-bold = Gras
mail-compose-rt-italic = Italique
mail-compose-rt-underline = Souligné
mail-compose-rt-strike = Barré
mail-compose-rt-bullet = Liste à puces
mail-compose-rt-ordered = Liste numérotée
mail-compose-rt-quote = Citation
mail-compose-rt-link = Lien
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Appliquer
mail-compose-rt-image = Insérer une image

mail-compose-signature = Signature
mail-compose-signature-none = Ajouter une signature…

mail-compose-recall = Rappeler
mail-compose-recall-action = Rappeler
mail-compose-recall-holding = En attente d’envoi
mail-compose-recall-scheduled = Programmé pour { $when }

mail-compose-drafts = Brouillons
mail-compose-drafts-empty = Aucun brouillon enregistré
mail-compose-drafts-delete = Supprimer le brouillon
mail-compose-drafts-no-recipient = (aucun destinataire)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Boîte d’envoi
mail-outbox-refresh = Actualiser
mail-outbox-empty = Aucun message en attente d’envoi
mail-outbox-send-now = Envoyer maintenant
mail-outbox-cancel = Annuler
mail-outbox-scheduled = Programmé
mail-outbox-holding = Envoi imminent
mail-outbox-sent = Envoyé
mail-outbox-canceled = Annulé
mail-outbox-held = Retenu
mail-outbox-failed = Non envoyé
mail-outbox-release = Libérer
mail-outbox-discard = Abandonner
mail-outbox-held-note = Retenu. Non envoyé tant qu’il n’est pas libéré.
mail-outbox-held-note-api-key = Retenu — créé par la clé d’API { $name }. Non envoyé tant qu’il n’est pas libéré.
mail-outbox-held-note-oauth = Retenu — créé par l’application connectée { $name }. Non envoyé tant qu’il n’est pas libéré.
mail-outbox-origin-api-key = Créé par la clé d’API { $name }
mail-outbox-origin-oauth = Créé par l’application connectée { $name }
mail-outbox-to = À { $recipients }
mail-outbox-no-subject = (sans objet)
mail-outbox-attempts = { $count ->
    [one] { $count } tentative. Dernière erreur : { $error }
   *[other] { $count } tentatives. Dernière erreur : { $error }
}
mail-outbox-error = Erreur : { $error }
mail-outbox-toast-canceled = Envoi annulé
mail-outbox-toast-discarded = Abandonné. Le message n’a pas été envoyé.
mail-outbox-toast-sent = Envoyé
mail-outbox-toast-retrying = Pas encore envoyé : { $error }. Une nouvelle tentative sera faite.
mail-outbox-toast-failed = Non envoyé : { $error }
mail-outbox-toast-not-released = Envoi impossible : { $error }
mail-outbox-toast-not-canceled = Annulation impossible : { $error }

# -- Network status and the offline queue ------------------------------------
mail-net-lost = Connexion perdue — nouvelle tentative…
mail-net-back = De nouveau en ligne
mail-queue-sent = { $count ->
    [one] { $count } modification en file d’attente envoyée
   *[other] { $count } modifications en file d’attente envoyées
}
mail-queue-failed = { $count ->
    [one] Échec de { $count } modification en file d’attente
   *[other] Échec de { $count } modifications en file d’attente
}
mail-queue-notice-label = Modifications qui n’ont pas pu être appliquées
mail-queue-notice-title = { $count ->
    [one] { $count } modification faite hors ligne n’a pas pu être appliquée
   *[other] { $count } modifications faites hors ligne n’ont pas pu être appliquées
}
mail-queue-kind-send = Envoyer « { $subject } »
mail-queue-kind-send-untitled = Envoyer un message sans objet
mail-queue-kind-move = Déplacer un message
mail-queue-kind-flag = Modifier une étiquette ou un indicateur
mail-queue-kind-draft = Enregistrer un brouillon
mail-queue-kind-pim = Modification d’agenda, de tâche, de note ou de contact
mail-queue-reason = Motif : { $reason }
mail-queue-reason-unknown = Le serveur n’a pas indiqué de motif.
mail-queue-retry = Réessayer
mail-queue-discard = Abandonner
mail-queue-retry-item = Réessayer : { $what }
mail-queue-discard-item = Abandonner : { $what }
mail-queue-discarded = Abandonnée. La modification n’a pas été appliquée.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Nettoyer les messages
mail-sweep-title = Nettoyer { $sender }
mail-sweep-all = Supprimer tout ce qui vient de cet expéditeur
mail-sweep-keep-latest = Garder le plus récent, supprimer le reste
mail-sweep-older-than = Supprimer les messages de plus de N jours
mail-sweep-block = Tout supprimer et bloquer cet expéditeur
mail-sweep-days = Jours
mail-sweep-count = { $count ->
    [one] { $count } message sera déplacé dans la corbeille
   *[other] { $count } messages seront déplacés dans la corbeille
}
mail-sweep-cancel = Annuler
mail-sweep-run = Nettoyer { $count }
mail-sweeping = Nettoyage…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Fermer
mail-undo-action = Annuler
mail-undo-send-cancel = Annuler
mail-undo-failed = Annulation impossible

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Étiquette ajoutée
mail-toast-label-removed = Étiquette retirée
mail-toast-pinned = Épinglé
mail-toast-unpinned = Détaché
mail-toast-snoozed = Reporté
mail-toast-follow-up-set = Suivi défini
mail-toast-follow-up-cleared = Suivi effacé
mail-toast-archived = Archivé
mail-toast-trashed = Déplacé dans la corbeille
mail-toast-marked-spam = Marqué comme spam
mail-toast-moved-to = Déplacé vers { $folder }
mail-toast-moved = Déplacé dans le dossier
mail-toast-archived-count = { $count ->
    [one] { $count } message archivé
   *[other] { $count } messages archivés
}
mail-toast-trashed-count = { $count ->
    [one] { $count } message déplacé dans la corbeille
   *[other] { $count } messages déplacés dans la corbeille
}
mail-toast-marked-spam-count = { $count ->
    [one] { $count } message marqué comme spam
   *[other] { $count } messages marqués comme spam
}
mail-toast-moved-count = { $count ->
    [one] { $count } message déplacé
   *[other] { $count } messages déplacés
}
mail-toast-moved-to-count = { $count ->
    [one] { $count } message déplacé vers { $folder }
   *[other] { $count } messages déplacés vers { $folder }
}
mail-toast-marked-read = { $count ->
    [one] Marqué comme lu
   *[other] { $count } messages marqués comme lus
}
mail-toast-marked-unread = { $count ->
    [one] Marqué comme non lu
   *[other] { $count } messages marqués comme non lus
}
mail-toast-flagged = { $count ->
    [one] Marqué
   *[other] { $count } messages marqués
}
mail-toast-unflagged = { $count ->
    [one] Indicateur retiré
   *[other] Indicateur retiré de { $count } messages
}
mail-toast-move-failed = Impossible de déplacer les messages. Aucun message n’a été déplacé.
mail-toast-update-failed = Impossible de modifier les messages. Rien n’a été modifié.
mail-toast-update-refused = { $count ->
    [one] Le serveur a refusé la modification pour un message
   *[other] Le serveur a refusé la modification pour { $count } messages
}
mail-toast-no-archive-folder = Aucun dossier Archives
mail-toast-no-trash-folder = Aucun dossier Corbeille
mail-toast-no-spam-folder = Aucun dossier Spam
mail-toast-sweep-nothing = Rien à nettoyer
mail-toast-swept = { $count ->
    [one] { $count } message nettoyé
   *[other] { $count } messages nettoyés
}
mail-toast-load-more-failed = Impossible de charger plus de messages
mail-toast-export-nothing = Rien à exporter
mail-toast-exported = .eml exporté
mail-toast-export-failed = Échec de l’exportation
mail-toast-send-queued = En file d’attente — envoi au retour de la connexion
mail-toast-tracking-not-bound = Le message a été envoyé, mais ses ouvertures ne seront pas enregistrées.
mail-toast-send-scheduled = Envoi programmé
mail-toast-sent = Message envoyé
mail-toast-send-canceled = Envoi annulé

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Ruban
mail-ribbon-tab-home = Accueil
mail-ribbon-tab-view = Affichage
mail-ribbon-tab-folder = Dossier
mail-ribbon-expand = Développer
mail-ribbon-collapse = Réduire
mail-ribbon-group-new = Nouveau
mail-ribbon-compose = Nouveau message
mail-ribbon-group-session = Session
mail-ribbon-logout = Se déconnecter
mail-ribbon-group-theme = Thème
mail-ribbon-group-density = Densité
mail-ribbon-group-settings = Paramètres
mail-ribbon-more = Plus…
mail-ribbon-group-folders = Dossiers
mail-density-compact = compacte
mail-density-cozy = confortable
mail-density-relaxed = aérée

# -- Message security level: the three-position switch in the reader toolbar
mail-maxsec-label = Niveau de sécurité du message
mail-maxsec-full = Complet
mail-maxsec-full-hint = HTML nettoyé avec images et mise en forme.
mail-maxsec-no-media = Sans médias
mail-maxsec-no-media-hint = HTML nettoyé, sans images ni médias.
mail-maxsec-plain-text = Texte brut
mail-maxsec-plain-text-hint = Aucun rendu HTML — message affiché en texte brut uniquement.
