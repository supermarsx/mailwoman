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
mail-inbox-filter = Filtre de la boîte de réception
mail-inbox-focused = Prioritaire
mail-inbox-other = Autres

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
mail-encrypted-note = Ce message est chiffré de bout en bout. Déverrouillez-le sur cet appareil pour le lire.
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
mail-toast-pinned = Message « { $target } » épinglé
mail-toast-unpinned = Message « { $target } » détaché
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
mail-feature-off = Désactivé par vos paramètres ou par votre administrateur
mail-send-disabled = L’envoi est désactivé par vos paramètres ou par votre administrateur. Vous pouvez toujours enregistrer un brouillon.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Événements
mail-events-upcoming = À venir uniquement
mail-events-filter = Filtre d’événements
mail-loading-more = Chargement des messages plus anciens…
mail-event-state-invitation = Invitation
mail-event-state-updated = Mis à jour
mail-event-state-cancelled = Annulé
mail-event-state-reply = Réponse
mail-event-state-counter = Nouvel horaire proposé
mail-event-state-event = Événement
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Afficher plus d’actions
mail-remote-moved = Ce message a été déplacé vers { $folder } depuis un autre appareil ou une autre application. Vous lisez toujours la copie que vous avez ouverte.
mail-remote-moved-unknown = Ce message a été déplacé depuis un autre appareil ou une autre application. Vous lisez toujours la copie que vous avez ouverte.
mail-remote-deleted = Ce message a été supprimé depuis un autre appareil ou une autre application. Vous pouvez continuer à lire cette copie jusqu’à sa fermeture.
mail-remote-dismiss = Ignorer
mail-reply-draft = Réponse en cours
mail-reply-draft-open = Ouvrir le brouillon
mail-draft-load-failed = Le brouillon n’a pas pu être ouvert. Réessayez.
mail-reply-images-omitted =
    { $count ->
        [one] Une image n’a pas pu être incluse dans la réponse.
       *[other] { $count } images n’ont pas pu être incluses dans la réponse.
    }
mail-export-forbidden = Votre administrateur a désactivé l’export de messages pour ce compte.
mail-export-open-failed = Les options d’export n’ont pas pu être ouvertes. Fermez un onglet de l’espace de travail et réessayez.
mail-headers-title = En-têtes bruts du message
mail-headers-action-help = Afficher les en-têtes d’origine du message, y compris les détails de remise et d’authentification.
mail-headers-help = En-têtes d’origine du message. Les lignes repliées sont conservées ; seuls les en-têtes sont affichés ici.
mail-headers-loading = Chargement des en-têtes du message…
mail-headers-failed = Les en-têtes du message n’ont pas pu être chargés. Vérifiez votre connexion et réessayez.
mail-headers-copy = Copier les en-têtes
mail-headers-copied = En-têtes copiés.
mail-headers-copy-failed = Les en-têtes n’ont pas pu être copiés. Sélectionnez le texte pour le copier manuellement.
mail-attachment-failed = Cette pièce jointe n’a pas pu être téléchargée. Réessayez lorsque la connexion sera disponible.
mail-compose-rt-rule = Ligne horizontale
mail-compose-signature-new = Nouveaux messages
mail-compose-signature-replies = Réponses
mail-editor-body-size = { $size } Kio
mail-editor-body-size-label = Taille du corps
mail-editor-body-size-help = Taille du corps actuel en UTF-8, balisage HTML et images intégrées compris. Les en-têtes du message et les pièces jointes séparées ne sont pas inclus.
mail-outbox-load-failed = La boîte d’envoi n’a pas pu être vérifiée. Vérifiez votre connexion, puis essayez Actualiser.
mail-outbox-unavailable = Boîte d’envoi indisponible. Utilisez Actualiser pour réessayer.
mail-maxsec-view = Vue : { $mode }
mail-maxsec-menu = Mode d’affichage
mail-toast-tab-pinned = Onglet « { $target } » épinglé
mail-toast-tab-unpinned = Onglet « { $target } » désépinglé
mail-workspace-message = Message
mail-opened-announcement = Ouvert : { $subject }
mail-workspace-layout = Disposition de l’espace de travail
mail-workspace-tabs = Onglets
mail-workspace-side-by-side = Côte à côte
mail-workspace-stack = Empilé
mail-workspace-refresh = Actualiser le dossier
mail-workspace-load-error = Ce message n’a pas pu être chargé. Fermez l’onglet et réessayez.
mail-resize-folders = Redimensionner la barre latérale des dossiers
mail-resize-list = Redimensionner la liste des messages
mail-resize-tiles = Redimensionner les volets de l’espace de travail
mail-show-folders = Afficher la barre latérale des dossiers
mail-hide-folders = Masquer la barre latérale des dossiers
mail-compact-folders = Icônes de dossier compactes
mail-toggle-list = Afficher ou masquer la liste des messages
mail-toggle-reader = Afficher ou masquer le volet de lecture
mail-folder-counts = { $unread } non lus, { $total } messages
mail-print = Imprimer / Enregistrer en PDF
mail-go-to-top = Haut de page
mail-print-help = Imprimez ce message ou choisissez Enregistrer en PDF dans la boîte de dialogue d’impression. Les images externes restent bloquées.
mail-file-details = Détails du fichier et empreinte
mail-file-type = Type de fichier
mail-file-extension = Extension
mail-file-detected = Signature détectée
mail-file-unknown = Aucune signature reconnue
mail-file-size = Taille
mail-file-size-bytes = { $count } octets
mail-file-hash-unavailable = Empreinte indisponible (nécessite un navigateur sécurisé et un fichier d’au plus 64 Mio).
mail-file-fingerprint-help = Une empreinte identifie exactement ces octets. Elle ne prouve ni que le fichier est sûr ni qui l’a créé.
mail-file-warning-executable = Ce fichier peut exécuter des programmes ou modifier les paramètres système. Ne le téléchargez que si vous faites confiance à sa source.
mail-file-warning-macro = Ce document peut contenir des macros. N’activez pas les macros sans avoir vérifié l’expéditeur et l’objet.
mail-file-warning-disguisedName = Le nom du fichier peut masquer sa véritable extension. Vérifiez les détails du fichier avant de le télécharger.
mail-file-warning-typeMismatch = La signature détectée ne correspond pas au type déclaré. L’aperçu automatique a été bloqué.
mail-file-warning-activeDocument = Ce format peut contenir du contenu actif. Son aperçu est isolé ; l’ouverture d’une copie téléchargée peut se comporter différemment.
mail-file-preview-blocked = L’aperçu automatique est désactivé pour ce type de fichier ou en cas de type incohérent.
mail-quote-header-from = De :
mail-quote-header-sent = Envoyé :
mail-quote-header-to = À :
mail-quote-header-cc = Cc :
mail-quote-header-subject = Objet :
mail-spam-score = Analyse anti-spam
mail-spam-score-loading = Vérification des en-têtes anti-spam…
mail-spam-score-unavailable = Score de spam indisponible
mail-spam-score-unverified = Indiqué par les en-têtes du message, sans vérification indépendante. Les en-têtes fournis par l’expéditeur peuvent être falsifiés.
mail-spam-score-positive = L’en-tête signale un spam.
mail-spam-score-negative = L’en-tête signale un résultat non spam.
mail-spam-score-unknown = Aucun verdict n’a été fourni avec ce score.
mail-external-recipients = Destinataires externes
mail-external-count = { $count } destinataire(s) hors des domaines de confiance
mail-external-help = Vérifiez les destinataires de À, Cc et Cci avant d’envoyer des informations confidentielles. La confiance repose sur l’identité d’envoi sélectionnée et votre politique de domaine effective.
mail-external-reviewed = J’ai vérifié ces destinataires externes.
mail-external-confirm-required = Vérifiez et confirmez les destinataires externes avant l’envoi.
mail-quick-reply = Réponse rapide
mail-quick-reply-help = Proposer un onglet de réponse compact avec les destinataires déjà renseignés. Désactivé par défaut. Les contrôles de sécurité et options d’envoi habituels s’appliquent toujours.
mail-quick-reply-expand = Modifier les destinataires et l’objet
mail-attachment-policy-loading = Vérification de la politique des pièces jointes…
mail-attachment-policy-unavailable = La politique des pièces jointes n’a pas pu être vérifiée. Réessayez avant d’ouvrir ce fichier.
mail-attachment-policy-blocked = Votre administrateur a bloqué cette pièce jointe ou son téléchargement d’origine.
mail-attachment-policy-warning = Votre administrateur recommande la prudence avec cette pièce jointe. Ne continuez que si vous faites confiance à sa source.
mail-attachment-policy-acknowledge = J’ai compris, continuer
mail-attachment-policy-no-preview = Les aperçus de pièces jointes sont désactivés par votre administrateur. Les téléchargements d’origine peuvent rester disponibles.
mail-time-sent = Envoyé
mail-time-received = Reçu
mail-time-unknown = Non indiqué
mail-time-comparison = Heure en { $zone }
mail-time-privacy = Le fuseau de comparaison est votre choix. Les horodatages des messages n’indiquent la position de personne.
mail-time-event-comparison = Comparaison de l’heure de l’événement
mail-time-ambiguous = Cette heure locale se produit deux fois lors du changement d’heure. Les deux instants possibles sont affichés ; vérifiez le décalage prévu de l’événement avant d’enregistrer.
mail-bulk-add-label = Ajouter un label
mail-select-messages = Sélectionner des messages
mail-bulk-remove-label = Retirer le label
mail-mark-spam = Marquer comme spam
mail-bulk-forward = Transférer les messages sélectionnés en pièces jointes
mail-bulk-download = Télécharger les messages sélectionnés au format mbox
mail-bulk-forward-subject = Messages transférés ({ $count })
mail-bulk-transfer-failed = Les messages sélectionnés n’ont pas pu être préparés. Sélectionnez jusqu’à 100 messages totalisant au plus 32 Mio et vérifiez les autorisations d’export de votre compte.
mail-toast-labels-added = Label { $label } ajouté à { $count } messages
mail-toast-labels-removed = Label { $label } retiré de { $count } messages
mail-action-help-reply = Répondre à l’expéditeur dans un nouvel onglet de rédaction.
mail-action-help-reply-all = Répondre à l’expéditeur et aux autres destinataires dans un nouvel onglet de rédaction.
mail-action-help-forward = Vérifier ce message et le transférer aux destinataires de votre choix.
mail-action-help-pin = Épingler ce message pour y accéder facilement, ou retirer l’épingle.
mail-action-help-read = Modifier l’état lu de ce message. Vous pouvez annuler la modification.
mail-action-help-flag = Ajouter ou retirer le drapeau de suivi du message.
mail-action-help-archive = Déplacer ce message vers le dossier d’archive configuré.
mail-action-help-delete = Déplacer ce message vers la corbeille. Vous pouvez annuler le déplacement.
mail-action-help-spam = Marquer ce message comme indésirable et le déplacer vers le dossier de spam configuré.
mail-action-help-export = Ouvrir la page d’export pour télécharger ce message dans le format de votre choix.
mail-action-help-sweep = Vérifier les actions applicables aux autres messages de cet expéditeur avant de les appliquer.
mail-empty-trash = Vider la corbeille
mail-trash-help = Vérifiez puis supprimez définitivement les messages actuellement dans la corbeille. Cette action est irréversible.
mail-trash-preparing = Préparation d’un instantané des messages de la corbeille…
mail-trash-prepare-failed = L’aperçu de la corbeille n’a pas pu être préparé. Vérifiez votre connexion et rouvrez-le. Aucun message n’a été supprimé.
mail-trash-confirm = Supprimer définitivement { $count } messages de la corbeille ? Cette action est irréversible. Les messages arrivés après cet aperçu sont exclus ; les messages sortis de la corbeille sont ignorés.
mail-trash-limit = Cette opération inclut jusqu’à { $count } messages. D’autres restent dans la corbeille ; vérifiez une autre opération ensuite pour les supprimer.
mail-trash-permanent = Supprimer définitivement
mail-trash-stop = Arrêter après la requête en cours
mail-trash-progress = { $deleted } supprimés ; { $skipped } messages ignorés car ils ne sont plus dans la corbeille ; { $remaining } restants dans cet aperçu.
mail-trash-stopped = Suppression arrêtée. Suppressions confirmées : { $deleted } ; ignorés : { $skipped } ; restants : { $remaining } ; résultat inconnu : { $uncertain }. Vérifiez la corbeille avant de lancer une autre opération.
mail-links-title = Liens du message
mail-links-review = Vérifier les liens ({ $count })
mail-links-help = Les liens du message ne s’ouvrent qu’après vérification de leur destination ici. Aucune destination n’est récupérée automatiquement.
mail-links-inert = Utilisez Vérifier les liens au-dessus du message pour examiner et ouvrir cette destination.
mail-links-limit = Les 100 premières destinations prises en charge distinctes sont listées. Les autres liens restent inactifs dans le message.
mail-links-destination = Destination à ouvrir
mail-links-external = Cette destination provient de l’expéditeur du message. L’ouvrir vous fait quitter Mailwoman ; vérifiez l’adresse avant de continuer.
mail-links-open = Ouvrir dans un nouvel onglet
mail-links-compose = Vérifier un nouveau message
mail-links-warning-http = Cette adresse utilise HTTP non chiffré.
mail-links-warning-address = Cette destination utilise une adresse IP au lieu d’un nom de domaine.
mail-links-warning-international = Ce domaine contient un nom internationalisé. Vérifiez attentivement son orthographe ASCII.
mail-links-warning-mismatch = L’adresse web visible désigne un hôte différent de la destination réelle.
mail-links-warning-direction = Le texte du lien contient des caractères de changement de direction pouvant masquer son adresse.
mail-outbox-uncertain = État de remise inconnu
mail-outbox-uncertain-note = L’envoi a commencé, mais la remise n’a pas pu être confirmée. Ce message ne sera pas renvoyé automatiquement. Consultez Envoyés ou confirmez auprès du destinataire avant de renvoyer.
mail-conversation-title = Afficher la conversation
mail-conversation-help = Parcourir les messages disponibles de cette conversation dans ce compte, tous dossiers confondus. L’ouverture d’un message ne modifie pas votre sélection de dossier.
mail-conversation-count = { $loaded } messages sur { $total } de la conversation affichés.
mail-conversation-other-folder = Autre dossier
mail-conversation-loading = Chargement de la conversation…
mail-conversation-missing = { $count } message(s) ne sont plus disponibles.
mail-conversation-error = La conversation n’a pas pu être chargée. Réessayez.
mail-conversation-more = Charger plus de messages

# -- Machine translation (2026-11): added to match en ---------------------
mail-conversation-limit = Cette vue est limitée à { $count } messages.
mail-compose-request-delivery = Demander des accusés de remise
mail-compose-request-delivery-hint = Demander aux serveurs de messagerie destinataires de signaler la remise, l’échec ou le retard. Cela ne demande pas et ne prouve pas que quelqu’un a lu le message. Les rapports renvoyés contiennent les en-têtes, pas le corps complet.
mail-compose-delivery-unavailable = Les accusés de remise sont indisponibles ou désactivés par votre administrateur pour ce compte. Un accusé explicitement demandé ne sera pas retiré silencieusement.
mail-compose-account-changed = Cette rédaction appartient à une autre connexion. Connectez-vous au bon compte et rouvrez le brouillon.
mail-context-open-message = Ouvrir le message
mail-context-open-folder = Ouvrir le dossier
mail-context-download = Télécharger le message
mail-context-refresh = Actualiser le dossier
mail-context-folder-settings = Paramètres du dossier
mail-context-failed = L’action n’a pas pu être terminée. Réessayez.
mail-sharing-unavailable = Le partage de dossier est indisponible ou vous n’avez plus l’autorisation de le gérer.
mail-trash-cancelling = Arrêt après la requête en cours. Suppressions confirmées : { $deleted } ; ignorés : { $skipped } ; restants : { $remaining }. Les suppressions déjà acceptées ne peuvent pas être annulées.
mail-trash-already-running = La suppression de la corbeille est déjà en cours. Utilisez sa notification de progression pour arrêter les lots suivants.
mail-context-search-sender = Rechercher cet expéditeur
mail-context-search-subject = Rechercher le même objet
mail-drag-too-many = Faites glisser au plus { $count } messages à la fois. Sélectionnez un groupe plus petit avant de déplacer.
mail-subtab-reorder-help = Faites glisser pour réorganiser les onglets, ou appuyez sur Ctrl+Maj+Gauche ou Droite sur l’onglet actif.
mail-compose-ribbon = Ruban de l’éditeur
mail-compose-ribbon-format = Mettre en forme le texte
mail-compose-ribbon-insert = Insérer
mail-compose-font = Police
mail-compose-font-size = Taille de police
mail-compose-font-inherit = Par défaut
mail-compose-undo = Annuler
mail-compose-redo = Rétablir
mail-compose-undo-tip = Annuler ({ $shortcut })
mail-compose-redo-tip = Rétablir ({ $shortcut })
mail-workspace-limit = Fermez un onglet avant d’en ouvrir un autre. Jusqu’à 32 onglets peuvent être conservés.
mail-compose-private-storage-unavailable = La récupération locale des brouillons est indisponible pour cette session. Gardez cet éditeur ouvert jusqu’à l’envoi ou la copie de votre message.
mail-offline-storage-unavailable = Les modifications hors ligne sont indisponibles tant que cette session n’a pas vérifié l’autorité de stockage. Reconnectez-vous puis connectez-vous à nouveau.
mail-draft-status-idle = Pas encore enregistré dans le compte
mail-draft-status-pending = Modifications non enregistrées
mail-draft-status-saving = Enregistrement dans Brouillons…
mail-draft-status-saved = Enregistré dans les brouillons du compte
mail-draft-status-error = Le brouillon n’a pas été enregistré. Votre copie de récupération locale reste disponible.
mail-draft-status-uncertain = La confirmation d’enregistrement est absente. Vérifiez Brouillons avant de créer une autre copie ; l’enregistrement automatique est suspendu.
mail-draft-status-conflict = Ce brouillon a été modifié sur un autre appareil. Rouvrez sa copie du compte avant de continuer.
mail-draft-status-unsupported = Ce compte ne peut actuellement pas confirmer les enregistrements dans son dossier Brouillons.
mail-draft-protected = Le contenu cité chiffré ou déchiffré n’est pas enregistré en clair. Tout brouillon antérieur du compte reste en place jusqu’à ce que vous l’abandonniez explicitement.
mail-draft-save-now = Enregistrer dans Brouillons
mail-draft-save-failed = Ce brouillon n’a pas pu être enregistré dans votre compte. Vérifiez l’éditeur avant de le fermer.
mail-draft-uncertain = Le brouillon a peut-être été enregistré. Les nouvelles tentatives automatiques sont suspendues ; vérifiez votre dossier Brouillons.
mail-draft-conflict = Un autre appareil a modifié ce brouillon. Vos modifications n’ont pas écrasé sa copie.
mail-draft-cleanup-pending = Le nouveau brouillon a été enregistré, mais sa copie précédente n’a pas pu être supprimée. Vérifiez Brouillons avant d’enregistrer à nouveau.
mail-draft-account-list = Brouillons du compte
mail-draft-loading = Chargement des brouillons du compte…
mail-draft-close-recovery = Fermer et ne conserver que la récupération locale
mail-draft-discard-account = Abandonner
mail-draft-discard-pending = La suppression du brouillon n’a pas pu être confirmée. Son enregistrement dans le compte demeure ; réessayez Abandonner après avoir vérifié le dossier Brouillons.
mail-draft-limit = Ce compte contient déjà 100 brouillons enregistrés. Abandonnez un brouillon du compte inutile avant d’en enregistrer un nouveau.
mail-draft-quota-help = Jusqu’à 100 brouillons de compte sont conservés. Abandonner supprime la copie confirmée en amont avant de libérer son emplacement.
mail-draft-discard-confirm = Confirmer la suppression du brouillon
mail-draft-discard-confirm-detail = Supprimer définitivement « { $subject } » des brouillons du compte ? Fermer l’éditeur à la place conserve le brouillon.
mail-draft-discard-title = Abandonner ce brouillon ?
mail-draft-keep-editing = Continuer la modification
mail-draft-discard-action = Abandonner
mail-draft-close-title = Abandonner les modifications non enregistrées ?
mail-draft-close-protected = Ce message contient du contenu chiffré ou déchiffré ; il n’est donc pas enregistré comme brouillon. La fermeture abandonne ces modifications. Un brouillon antérieur du compte, le cas échéant, reste dans Brouillons.
mail-draft-close-unstored = Les brouillons ne peuvent pas être enregistrés dans cette session ; fermer ce message l’abandonne donc.
mail-workspace-view-mode = Disposition des vues
mail-workspace-tabs-help = Afficher une seule vue active en gardant les autres onglets ouverts.
mail-workspace-side-by-side-help = Afficher deux vues sélectionnées côte à côte. Faites glisser un onglet sur une vue pour la remplacer.
mail-workspace-stack-help = Afficher deux vues sélectionnées l’une au-dessus de l’autre. Faites glisser un onglet sur une vue pour la remplacer.
mail-workspace-masonry-side-by-side = Mosaïque côte à côte
mail-workspace-masonry-side-by-side-help = Disposer toutes les vues ouvertes en colonnes décalées. Les petits écrans affichent l’onglet actif.
mail-workspace-masonry-stack = Mosaïque empilée
mail-workspace-masonry-stack-help = Disposer toutes les vues ouvertes en lignes décalées. Les petits écrans affichent l’onglet actif.
mail-workspace-view-first = Première vue
mail-workspace-view-second = Deuxième vue
mail-workspace-view-empty = Faites glisser un onglet ouvert ici pour choisir cette vue.
mail-popup-blocked = Autorisez les fenêtres contextuelles pour Mailwoman afin de détacher cet onglet. Votre onglet reste ici.
mail-popup-unavailable = La fenêtre détachée n’a pas pu être chargée. Votre onglet et vos modifications non enregistrées restent dans la fenêtre principale. Réessayez après avoir actualisé l’application.
mail-popup-reattach = Rattacher à la fenêtre principale
mail-popup-action-reattached = Retour à la fenêtre principale pour terminer cette action.
mail-popup-detached = Cet onglet est ouvert dans une fenêtre séparée. La fermeture de cette fenêtre le ramène ici.
mail-compose-from-custom = Autre adresse autorisée…
mail-compose-from-alias = Adresse d’expéditeur pour ce message
mail-compose-from-alias-help = Utilisez un alias depuis lequel vous êtes autorisé à envoyer. Cela ne modifie que ce brouillon ; l’autorisation est vérifiée avant l’envoi.
mail-compose-from-refused = Cette adresse n’est pas autorisée à envoyer depuis ce compte.
mail-compose-from-unavailable = Cette adresse d’expéditeur n’a pas pu être vérifiée. Votre message n’a pas été envoyé.
mail-compose-from-invalid = Saisissez une seule adresse e-mail valide pour l’expéditeur.
mail-compose-from-toggle = Afficher ou masquer l’expéditeur de ce message
mail-compose-ribbon-security = Sécurité
mail-compose-ribbon-delivery = Remise
mail-compose-ribbon-recipients = Destinataires
mail-compose-ribbon-message-type = Type de message
mail-compose-recall-help = Annuler les messages encore retenus pour annulation ou envoi programmé. Les messages déjà remis ne peuvent pas être rappelés.
mail-compose-recall-loading = Vérification des messages en file d’attente…
mail-compose-recall-empty = Aucun message en file d’attente n’est disponible à annuler.
mail-compose-recall-failed = L’état de la file d’attente n’a pas pu être confirmé. Fermez et rouvrez ce panneau pour l’actualiser avant une autre action.
mail-compose-importance = Importance
mail-compose-importance-high = Haute
mail-compose-importance-normal = Normale
mail-compose-importance-low = Basse
mail-insert-account-title = Insérer un élément du compte
mail-insert-account-help = Joindre un export de calendrier, une carte de visite de contact ou un message d’origine de ce compte.
mail-insert-account-close = Fermer le sélecteur d’éléments du compte
mail-insert-account-kind = Type d’élément
mail-insert-account-event = Événement de calendrier (.ics)
mail-insert-account-calendar = Calendrier (.ics)
mail-insert-account-contact = Carte de visite du contact (.vcf)
mail-insert-account-message = Message d’origine (.eml)
mail-insert-account-search = Filtrer les éléments listés
mail-insert-account-selection = Sélectionner un élément
mail-insert-account-scope = Seuls les éléments du compte actuel sont listés. Les messages d’origine suivent la politique d’export du serveur.
mail-insert-account-limited = Affichage de 100 éléments au plus. Le filtre ne recherche que dans cette liste.
mail-insert-account-unavailable = Aucun export de compte pris en charge n’est disponible.
mail-insert-account-empty = Aucun élément correspondant.
mail-insert-account-attach = Joindre l’élément sélectionné
mail-insert-account-loading = Chargement de l’élément du compte…
mail-insert-account-error = L’élément n’a pas pu être joint. Vérifiez votre accès et la politique d’export, puis réessayez. Les grands calendriers peuvent nécessiter des exports d’événements individuels.
mail-compose-html-source = Source HTML
mail-editor-clear-format = Effacer la mise en forme
mail-compose-format-html = HTML
mail-compose-html-visual = Éditeur visuel
mail-compose-html-help = Modifier le HTML directement. Seule la mise en forme d’e-mail prise en charge est enregistrée et envoyée ; les scripts, images externes et balisages non sûrs sont supprimés.
mail-compose-html-processing = Préparation d’un HTML sûr…
mail-compose-html-failed = Le HTML n’a pas pu être préparé. Modifiez-le ou réessayez avant d’enregistrer ou d’envoyer.
mail-compose-check-names = Vérifier les noms
mail-compose-check-names-help = Résoudre les noms exacts à partir de vos contacts chargés et vérifier le format des adresses. Cela ne vérifie pas la remise dans la boîte aux lettres.
mail-compose-names-unresolved = Choisissez une adresse e-mail complète pour ces noms non résolus ou ambigus : { $names }
mail-compose-names-checked = Format des destinataires vérifié ; { $count } nom(s) de contact résolu(s). La remise dans la boîte aux lettres n’a pas été vérifiée.
mail-compose-ribbon-table = Tableau
mail-compose-group-clipboard = Presse-papiers
mail-compose-group-paragraph = Paragraphe
mail-compose-group-editing = Édition
mail-compose-group-view = Affichage
mail-compose-dark-background = Fond sombre
mail-compose-dark-background-help = Afficher le message sur fond sombre pendant la rédaction. Seul votre écran change : le message est envoyé sans couleur de fond.
mail-compose-group-assist = Assistant de rédaction
mail-editor-counts = Statistiques de l’éditeur
mail-editor-words = { $count ->
    [one] { $count } mot
   *[other] { $count } mots
}
mail-editor-paragraphs = { $count ->
    [one] { $count } paragraphe
   *[other] { $count } paragraphes
}
mail-editor-characters = { $count ->
    [one] { $count } caractère
   *[other] { $count } caractères
}
mail-editor-selection-required = Sélectionnez du texte ou des cellules de tableau adaptés avant d’utiliser cette action.
mail-editor-clipboard-unavailable = L’accès au presse-papiers était indisponible. Utilisez les raccourcis Copier, Couper ou Coller de votre navigateur.
mail-editor-selection-changed = La sélection a changé. Sélectionnez le texte voulu et réessayez.
mail-editor-clipboard = Presse-papiers et mise en forme
mail-editor-copy = Copier
mail-editor-cut = Couper

# -- Machine translation (2026-11): added to match en ---------------------
mail-privacy-images-failed = { $loaded }/{ $total } images affichées · { $failed } en échec
mail-privacy-images-loaded = { $loaded }/{ $total } images affichées
mail-privacy-images-loading-failed = { $loaded }/{ $total } images affichées · { $pending } en cours de chargement · { $failed } en échec
mail-reader-folder-title = Dossier : { $folder }
mail-reader-move-to = Déplacer vers…
mail-action-help-move = Déplacer ce message vers un autre dossier.
folder-ops-new-subfolder = Nouveau sous-dossier
folder-ops-mark-read = Tout marquer comme lu
folder-ops-this-folder = Ce dossier
folder-ops-with-subfolders = Inclure les sous-dossiers
folder-ops-no-unread = Aucun message non lu.
folder-ops-read-unavailable = Vous ne pouvez pas modifier les messages de ce dossier.
folder-ops-more = Plus
folder-ops-system-folder = Un dossier système ne peut pas être modifié ainsi.
folder-ops-sort-subfolders = Trier les sous-dossiers par
folder-ops-sort-folders = Trier les dossiers par
folder-ops-sort-name = Nom
folder-ops-sort-unread = Nombre de non lus
folder-ops-sort-total = Nombre total
folder-ops-sort-date = Dernière activité
folder-ops-sort-custom = Ordre personnalisé
folder-ops-sort-asc = Croissant
folder-ops-sort-desc = Décroissant
folder-ops-delete-with-subfolders = Supprimer le dossier et ses sous-dossiers
folder-ops-delete-unavailable = Ce dossier ne peut pas être supprimé.
folder-ops-delete-tree-body = Cela supprime définitivement { $folders } dossiers et les { $count } messages qu’ils contiennent. Cette action est irréversible.
folder-ops-copy-all-menu = Copier tous les éléments vers…
folder-ops-move-all-menu = Déplacer tous les éléments vers…
folder-ops-move-folder-menu = Déplacer le dossier vers…
folder-ops-move-up = Monter
folder-ops-move-down = Descendre
folder-ops-move-unavailable = Vous ne pouvez pas déplacer ce dossier ni ses éléments.
folder-ops-move-duplicate = La destination contient déjà un dossier portant ce nom.
folder-ops-move-folder-title = Déplacer { $name }
folder-ops-move-folder-body = Le dossier est déplacé avec ses sous-dossiers et ses messages. Choisissez le dossier qui le contiendra, ou le niveau supérieur.
folder-ops-move-here = Déplacer ici
folder-ops-moved-folder = { $name } déplacé.
folder-ops-choose-folder = Choisissez un dossier de destination.
folder-ops-destination = Dossier de destination
folder-ops-include-subfolders = Inclure les messages de { $count } sous-dossiers
folder-ops-copy-all-title = Copier tous les éléments de { $name }
folder-ops-move-all-title = Déplacer tous les éléments de { $name }
folder-ops-copy-all-body = Ajoute { $count } messages au dossier de destination. Ils restent à leur place.
folder-ops-move-all-body = Déplace { $count } messages vers le dossier de destination. Vous pourrez l’annuler ensuite.
folder-ops-copy-all = Copier les éléments
folder-ops-move-all = Déplacer les éléments
folder-ops-undo = Annuler
folder-ops-stop = Arrêter
folder-ops-marking = Marquage de { $name } comme lu…
folder-ops-marking-progress = Marquage de { $name } comme lu : { $done } sur { $total }
folder-ops-marked = { $count } messages de { $name } marqués comme lus.
folder-ops-copying = Copie de { $name }…
folder-ops-copying-progress = Copie de { $name } : { $done } sur { $total }
folder-ops-copied = { $count } messages copiés depuis { $name }.
folder-ops-moving = Déplacement de { $name }…
folder-ops-moving-progress = Déplacement de { $name } : { $done } sur { $total }
folder-ops-moved = { $count } messages déplacés depuis { $name }.
folder-ops-stopped = { $name } arrêté après { $count } messages sur { $total }.
folder-ops-partial = { $count } messages sur { $total } dans { $name } ont été traités ; { $refused } ont été refusés.
folder-ops-failed = { $name } n’a pas pu être terminé. Actualisez et réessayez.
folder-ops-error-forbidden = Vous n’avez pas les droits nécessaires pour cette modification.
folder-ops-error-exists = Un dossier portant ce nom existe déjà à cet endroit.
folder-ops-error-mailboxHasEmail = Le dossier contient encore des messages.
folder-ops-error-notFound = Le dossier n’existe plus.
folder-ops-error-failed = La modification n’a pas pu être effectuée. Actualisez et réessayez.
folder-ops-expand = Afficher les sous-dossiers de { $name }
folder-ops-collapse = Masquer les sous-dossiers de { $name }
mail-context-reply = Répondre
mail-context-reply-all = Répondre à tous
mail-context-forward = Transférer
mail-context-quick-print = Impression rapide
mail-context-copy = Copier
mail-context-copy-to-folder = Copier le message vers un dossier…
mail-context-copy-sender = Copier l’adresse de l’expéditeur
mail-context-copy-subject = Copier l’objet
mail-context-copy-link = Copier le lien du message
mail-context-copied = Copié.
mail-context-copy-failed = Copie dans le presse-papiers impossible.
mail-context-print-failed = Le message n’a pas pu être imprimé.
mail-reply-images-title = Images du message d’origine
mail-reply-images-ask = Inclure les images du message d’origine dans cette réponse ?
mail-reply-images-include = Inclure les images
mail-reply-images-not-now = Pas maintenant
mail-reply-images-trust = Toujours faire confiance à { $domain }
mail-reply-images-loading = Ajout des images…
mail-reply-images-cancelled = Chargement annulé. Les images n’ont pas été ajoutées.
mail-reply-images-failed = Les images n’ont pas pu être ajoutées.
mail-reply-images-retry = Recharger
mail-image-cancel = Annuler
mail-compose-from-why-match = Envoi depuis { $identity } car le message a été envoyé à { $address }.
mail-compose-from-why-match-plus = Envoi depuis { $identity } car le message a été envoyé à { $address }, une adresse plus de celle-ci.
mail-compose-from-why-ambiguous = Plusieurs identités correspondent à { $address }. Envoi depuis { $identity } ; choisissez-en une autre :
mail-compose-from-why-default = Envoi depuis votre identité par défaut, { $identity }, car aucune ne correspond aux destinataires d’origine.
mail-compose-from-why-forced-default = Envoi depuis votre identité par défaut, { $identity }, comme défini dans vos paramètres.
mail-compose-from-why-keep = Envoi depuis { $identity }.
mail-compose-from-change = Modifier
reader-attachments-reason-encrypted = Ce message est chiffré. Ses pièces jointes apparaissent après son déverrouillage.
reader-attachments-reason-list-missing = La liste des pièces jointes de ce message n’a pas pu être chargée ; la barre des pièces jointes est donc masquée.
reader-attachments-reason-inline-only = Les pièces jointes de ce message ({ $count }) sont des images intégrées affichées dans le message lui-même.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] Une pièce jointe de ce message n’a aucun contenu téléchargeable ; la barre des pièces jointes est donc masquée.
   *[other] { $count } pièces jointes de ce message n’ont aucun contenu téléchargeable ; la barre des pièces jointes est donc masquée.
  }
reader-attachments-reason-policy-blocked = Votre politique de pièces jointes bloque le téléchargement { $count ->
    [one] de cette pièce jointe.
   *[other] de ces pièces jointes.
  }
reader-attachments-reason-policy-unavailable = La politique des pièces jointes n’a pas pu être vérifiée ; les pièces jointes ne peuvent donc pas encore être ouvertes.
mail-drafts-popup-count = { $count ->
    [one] 1 brouillon
   *[other] { $count } brouillons
  }
mail-drafts-popup-search = Rechercher par objet ou destinataire
mail-drafts-popup-tab-local = Brouillons locaux
mail-drafts-popup-local-hint = Les brouillons locaux sont conservés uniquement dans ce navigateur.
mail-drafts-popup-badge-account = Compte
mail-drafts-popup-badge-local = Local
mail-drafts-popup-current = Ouvert actuellement
mail-drafts-popup-select = Sélectionner
mail-drafts-popup-select-all = Tout sélectionner
mail-drafts-popup-clear = Effacer
mail-drafts-popup-done = Terminé
mail-drafts-popup-selected = { $count } sélectionnés
mail-drafts-popup-select-row = Sélectionner { $subject }
mail-drafts-popup-open = Ouvrir
mail-drafts-popup-discard = Abandonner
mail-drafts-popup-discard-selected = Abandonner la sélection
mail-drafts-popup-more = Plus d’actions pour { $subject }
mail-drafts-popup-retry = Réessayer
mail-drafts-popup-load-failed = Les brouillons du compte n’ont pas pu être chargés.
mail-drafts-popup-empty-account = Aucun brouillon de compte
mail-drafts-popup-no-match = Aucun brouillon ne correspond à « { $query } »
mail-drafts-popup-discarding = Suppression…
mail-drafts-popup-row-failed = Suppression impossible. Sa copie dans le compte demeure.
mail-drafts-popup-confirm-title = { $count ->
    [one] Abandonner ce brouillon ?
   *[other] Abandonner { $count } brouillons ?
  }
mail-drafts-popup-confirm-account = Les brouillons du compte sont d’abord supprimés du dossier Brouillons de votre compte, et leur emplacement n’est libéré qu’une fois cette suppression confirmée.
mail-drafts-popup-confirm-local = Les brouillons locaux sont supprimés de ce navigateur.
mail-drafts-popup-announce-done = { $count ->
    [one] 1 brouillon abandonné
   *[other] { $count } brouillons abandonnés
  }
mail-drafts-popup-announce-failed = { $done } abandonnés, { $failed } n’ont pas pu l’être
mail-check-title = Vérifier avant l’envoi
mail-check-intro = Ces points méritent peut-être un second regard :
mail-check-invalid = Certaines adresses de destinataires semblent invalides : { $list }
mail-check-subject = L’objet est vide.
mail-check-attachment = Le message mentionne une pièce jointe, mais rien n’est joint.

# -- Machine translation (2026-11): added to match en ---------------------
mail-editor-paste = Coller
mail-editor-copy-format = Copier la mise en forme
mail-editor-apply-format = Appliquer la mise en forme
mail-editor-style = Style de paragraphe
mail-editor-text-color = Couleur de police
mail-editor-clear-color = Couleur par défaut
mail-editor-highlight = Surlignage
mail-editor-clear-highlight = Supprimer le surlignage
mail-editor-paragraph = Options de paragraphe
mail-editor-alignment = Alignement
mail-editor-line-height = Hauteur de ligne
mail-editor-space-before = Espacement avant
mail-editor-space-after = Espacement après
mail-editor-indent = Retrait
mail-editor-shading = Trame du paragraphe
mail-editor-clear-shading = Supprimer la trame
mail-editor-case = Changer la casse
mail-editor-sort = Trier les paragraphes sélectionnés
mail-editor-sort-alpha = Alphabétique
mail-editor-sort-numeric = Ordre numérique
mail-editor-table-rows = Lignes du tableau
mail-editor-table-columns = Colonnes du tableau
mail-editor-table-insert = Insérer un tableau
mail-editor-table-limits = Jusqu’à 20 lignes, 10 colonnes et 200 cellules par tableau ; 10 tableaux au plus. Les tableaux imbriqués ne sont pas pris en charge.
mail-editor-table-row-before = Ligne au-dessus
mail-editor-table-row-after = Ligne en dessous
mail-editor-table-row-delete = Supprimer la ligne
mail-editor-table-column-before = Colonne avant
mail-editor-table-column-after = Colonne après
mail-editor-table-column-delete = Supprimer la colonne
mail-editor-table-merge = Fusionner les cellules
mail-editor-table-split = Fractionner la cellule
mail-editor-table-header = Activer/désactiver la ligne d’en-tête
mail-editor-table-delete = Supprimer le tableau
mail-editor-table-shading = Trame de cellule
mail-editor-find-replace = Rechercher et remplacer
mail-editor-find = Rechercher du texte
mail-editor-replacement = Remplacer par
mail-editor-match-case = Respecter la casse
mail-editor-previous = Occurrence précédente
mail-editor-next = Occurrence suivante
mail-editor-replace = Remplacer l’occurrence sélectionnée
mail-editor-replace-all = Remplacer toutes les occurrences
mail-editor-matches = { $count } occurrences (jusqu’à 1 000)
mail-editor-select-all = Tout sélectionner
mail-editor-clear-selection = Effacer la sélection
mail-editor-history = Historique des annulations
mail-editor-undo-groups = Annuler les { $count } derniers groupes de modifications
mail-editor-zoom = Zoom de l’éditeur
mail-editor-markers = Marques de paragraphe
mail-editor-style-normal = Normal
mail-editor-style-no-spacing = Sans espacement
mail-editor-style-title = Titre
mail-editor-style-subtitle = Sous-titre
mail-editor-style-heading1 = Titre 1
mail-editor-style-heading2 = Titre 2
mail-editor-style-heading3 = Titre 3
mail-editor-style-heading4 = Titre 4
mail-editor-style-heading5 = Titre 5
mail-editor-style-heading6 = Titre 6
mail-editor-align-left = Gauche
mail-editor-align-center = Centré
mail-editor-align-right = Droite
mail-editor-align-justify = Justifié
mail-editor-case-lower = minuscules
mail-editor-case-upper = MAJUSCULES
mail-editor-case-title = Initiales En Majuscules
mail-editor-case-sentence = Majuscule en début de phrase
mail-editor-action-history = Historique : { $action }
mail-editor-history-empty = Aucune action disponible
mail-editor-redo-groups = Rétablir { $count } modifications
mail-compose-identities-failed = Les identités d’envoi n’ont pas pu être chargées. Fermez et rouvrez ce brouillon pour réessayer.
mail-folders-loading = Chargement des dossiers…
mail-initial-sync-error = Vos messages n’ont pas encore pu être synchronisés.
mail-initial-sync-error-help = La connexion à votre serveur de messagerie a été interrompue. Réessayez, ou demandez à votre administrateur de vérifier les journaux de connexion si cela se reproduit.
mail-links-trusted = Domaine de confiance : s’ouvre sans vérification
mail-links-trust-domain = Faire confiance à { $domain }
mail-links-trust-failed = Le domaine de confiance n’a pas pu être enregistré. Réessayez.
mail-focus-always-focused = Toujours déplacer vers Prioritaire
mail-focus-always-other = Toujours déplacer vers Autres
mail-focus-moved-focused = Les messages de { $sender } vont désormais dans Prioritaire.
mail-focus-moved-other = Les messages de { $sender } vont désormais dans Autres.
mail-focus-save-failed = Le choix de boîte de réception Prioritaire pour cet expéditeur n’a pas pu être enregistré. Réessayez.
mail-context-search-domain = Rechercher les messages de ce domaine
mail-loading-mailbox = Chargement de { $name }…
mail-user-folders = Dossiers de l’utilisateur
mail-context-hide-folder = Masquer de la barre latérale
mail-folder-fallback = dossier
mail-toast-moving = { $count ->
    [one] Déplacement d’un message vers { $folder }…
   *[other] Déplacement de { $count } messages vers { $folder }…
    }
mail-toast-move-queueing = { $count ->
    [one] Mise en file d’attente d’un message à déplacer vers { $folder }…
   *[other] Mise en file d’attente de { $count } messages à déplacer vers { $folder }…
    }
mail-toast-move-queued = { $count ->
    [one] Un message en file d’attente pour déplacement vers { $folder }
   *[other] { $count } messages en file d’attente pour déplacement vers { $folder }
    }
mail-toast-move-partial = { $count } messages sur { $total } déplacés vers { $folder } ; { $failed } n’ont pas pu être déplacés
mail-send-copy-folder-unavailable = Le dossier sélectionné pour la copie des messages envoyés est indisponible. Choisissez un autre dossier avant d’envoyer.
mail-send-copy-failed = Le message a été accepté, mais sa copie envoyée n’a pas pu être classée. Ne l’envoyez pas à nouveau.
mail-compose-attach-public-key = Joindre ma clé publique ou mon certificat
mail-compose-include-hashes = Inclure les empreintes SHA-256 des fichiers joints
mail-compose-hashes-pending = Calcul des empreintes des pièces jointes. Envoyez quand c’est prêt.
mail-compose-hashes-failed = Les octets des pièces jointes n’ont pas pu être vérifiés. Désactivez les empreintes ou réessayez en les réactivant.
mail-compose-crypto-pending = Le chiffrement est en cours de mise à jour. Patientez avant d’envoyer.
mail-compose-public-key-unavailable = Aucune clé publique ou certificat actuel ne correspond à l’adresse d’envoi.
mail-compose-sent-copy = Enregistrer le message envoyé dans
mail-compose-sent-copy-default = Valeur par défaut du compte
folder-create = Créer un dossier
folder-name = Nom du dossier
folder-parent = Dossier parent
folder-top-level = Niveau supérieur
folder-created = Dossier créé.
folder-create-unavailable = La création de dossier est indisponible pour ce compte ou ce dossier parent.
folder-create-invalid = Choisissez un nom de dossier valide et unique, sans séparateurs de hiérarchie ni caractères de contrôle.
folder-create-failed = Le dossier n’a pas pu être créé. Actualisez et réessayez.
folder-cleanup = Nettoyer le dossier…
folder-cleanup-title = Nettoyer { $name }
folder-cleanup-help = Rechercher les messages plus anciens que l’âge choisi. Vérifiez le nombre avant de supprimer. Les sous-dossiers sont exclus.
folder-cleanup-age = Plus ancien que
folder-age-days = Jours
folder-age-weeks = Semaines
folder-age-months = Mois
folder-age-years = Ans
folder-cleanup-preview = Aperçu du nettoyage
folder-cleanup-confirm = Supprimer les messages correspondants
folder-cleanup-permanent = Supprimer définitivement { $count } messages correspondants. Cette action est irréversible.
folder-cleanup-trash = Déplacer { $count } messages correspondants vers la corbeille.
folder-cleanup-before = Reçu avant le { $date }
folder-cleanup-done = { $count } messages nettoyés.
mail-from = Expéditeur
mail-subject = Objet
mail-date = Date
mail-cancel = Annuler
mail-bulk-selected = { $count } conversations sélectionnées
mail-bulk-select-folder = Tout sélectionner dans { $folder }
mail-list-order = Ordre des messages

# -- Machine translation (2026-11): added to match en ---------------------
mail-list-order-newest = Plus récents d’abord
mail-list-order-oldest = Plus anciens d’abord
mail-list-order-sender = Expéditeur
mail-list-order-subject = Objet
mail-list-order-error = L’ordre des messages n’a pas pu être enregistré.
mail-bulk-selection-load-failed = Le dossier entier n’a pas pu être sélectionné. Vérifiez votre connexion et réessayez.
mail-privacy-bar = Confidentialité du message
mail-privacy-trackers-allow = Autoriser les traceurs pour ce message
mail-privacy-trackers-block = Bloquer de nouveau les traceurs
mail-privacy-images-loading = { $loaded }/{ $total } images affichées · { $pending } en cours de chargement
mail-check-external = Destinataires hors des domaines de confiance : { $list }
mail-check-send-anyway = Envoyer quand même
mail-check-review = Vérifier
mail-check-remember = Ne plus demander pour ceux-ci
mail-check-denied-invalid = L’envoi est bloqué par vos paramètres. Corrigez les adresses de destinataires invalides : { $list }
mail-check-denied-subject = L’envoi est bloqué par vos paramètres. Ajoutez un objet.
mail-check-denied-attachment = L’envoi est bloqué par vos paramètres. Joignez le fichier mentionné ou reformulez le message.
mail-check-denied-external = L’envoi est bloqué par vos paramètres. Hors des domaines de confiance : { $list }
mail-alerts-label = Problèmes du message
mail-alert-invalid = Adresse invalide : { $address }
mail-alert-subject = Aucun objet
mail-recipient-chip-remove = Retirer { $address }
mail-recipient-chips = Destinataires dans { $field }
mail-recipient-invalid-title = Adresse e-mail invalide
mail-recipient-chip-name = { $name }, { $address }. Entrée pour les détails du contact, Retour arrière ou F2 pour modifier, Suppr pour retirer
mail-recipient-chip-name-invalid = { $address }, adresse invalide. Entrée pour modifier, Suppr pour retirer
mail-recipient-added = { $name } ajouté
mail-recipient-removed = { $name } retiré
mail-recipient-removed-many = { $count ->
    [one] { $count } destinataire retiré
   *[other] { $count } destinataires retirés
}
mail-recipient-editing = Modification de { $name }
mail-recipient-all-selected = { $count ->
    [one] { $count } destinataire sélectionné
   *[other] { $count } destinataires sélectionnés
}
mail-external-line = { $count } hors des domaines de confiance : { $domains }
mail-external-dismiss = Ignorer
mail-status-label = État de l’éditeur
mail-status-more = { $count } autres avis
mail-status-more-label = Autres avis
mail-status-no-recipients = Aucun destinataire
mail-status-no-attachment = Aucune pièce jointe
mail-status-external = Externes : { $count }
mail-status-external-reviewed = Destinataires externes vérifiés
mail-status-external-review = Marquer comme vérifié
mail-status-from-match = De { $identity }
mail-status-from-match-plus = De { $identity }
mail-status-from-ambiguous = De { $identity } (plusieurs correspondent)
mail-status-from-default = De l’identité par défaut
mail-status-from-forced-default = De l’identité par défaut
mail-status-from-keep = De { $identity }
mail-status-held = { $count } en file d’attente
mail-status-storage = Pas de récupération locale
mail-status-protected = Non enregistré en clair
mail-status-saving = Enregistrement…
mail-status-saved = Enregistré { $time }
mail-status-saved-full = Enregistré dans les brouillons du compte · { $time }
mail-status-not-saved = Brouillon non enregistré
mail-status-retry = Réessayer
mail-status-conflict = Modifié sur un autre appareil
mail-status-uncertain = Enregistrement non confirmé
mail-status-close-recovery = Fermer, garder la récupération
mail-status-unsupported = Enregistrement du brouillon non confirmé
mail-compose-header = En-tête du message
mail-header-label = En-tête du message
mail-header-collapse = Réduire l’en-tête du message
mail-header-expand = Développer l’en-tête du message : { $summary }
mail-header-from-chip = De { $address }. Changer l’expéditeur
mail-compose-group-timing = Moment d’envoi
mail-compose-group-priority = Priorité
mail-compose-group-receipts = Accusés et suivi
mail-compose-group-keys-files = Clés et fichiers
mail-compose-group-sent-copy = Copie envoyée
mail-editor-style-quote = Citation
mail-editor-style-code = Code
mail-compose-ribbon-table-layout = Disposition du tableau
mail-compose-ribbon-picture = Image
mail-editor-table-group-rows = Lignes
mail-editor-table-group-columns = Colonnes
mail-editor-table-group-cells = Cellules
mail-editor-table-row-height = Hauteur de ligne
mail-editor-table-column-width = Largeur de colonne
mail-editor-table-size-auto = Auto
mail-editor-table-padding = Marge interne des cellules
mail-editor-table-padding-compact = Compacte
mail-editor-table-padding-normal = Normale
mail-editor-table-padding-roomy = Aérée
mail-editor-table-align-left = Aligner la cellule à gauche
mail-editor-table-align-center = Centrer la cellule
mail-editor-table-align-right = Aligner la cellule à droite
mail-picture-group-size = Taille
mail-picture-group-arrange = Organiser
mail-picture-group-text = Texte
mail-picture-group-style = Style
mail-picture-group-edit = Modifier
mail-picture-width = Largeur
mail-picture-height = Hauteur
mail-picture-lock = Conserver les proportions
mail-picture-reset-size = Taille d’origine
mail-picture-align-inline = Dans le texte
mail-picture-align-left = Aligner à gauche
mail-picture-align-center = Centrer
mail-picture-align-right = Aligner à droite
mail-picture-alt = Texte alternatif
mail-picture-title = Titre
mail-picture-border = Bordure
mail-picture-radius = Coins
mail-picture-none = Aucun
mail-picture-round = Arrondis
mail-picture-rotate-ccw = Pivoter à gauche
mail-picture-rotate-cw = Pivoter à droite
mail-picture-flip-h = Retourner horizontalement
mail-picture-flip-v = Retourner verticalement
mail-picture-crop = Rogner
mail-picture-crop-left = Gauche (%)
mail-picture-crop-top = Haut (%)
mail-picture-crop-right = Droite (%)
mail-picture-crop-bottom = Bas (%)
mail-picture-crop-apply = Rogner
mail-picture-replace = Remplacer
mail-picture-remove = Retirer
mail-picture-failed-no-image = Sélectionnez d’abord une image.
mail-picture-failed-no-source = Cette image n’a pas de copie locale à modifier. Remplacez-la plutôt.
mail-picture-failed-failed = L’image n’a pas pu être modifiée.
mail-context-download-attachments = Télécharger les pièces jointes
mail-context-download-named = Télécharger { $name }
mail-context-download-all = Tout télécharger en ZIP
mail-context-loading-attachments = Chargement des pièces jointes…
mail-context-attachments-failed = Les pièces jointes n’ont pas pu être chargées.
mail-context-searching = Recherche du message…
mail-context-go-reply = Aller à la réponse
mail-context-go-forward = Aller au transfert
mail-context-reply-not-found = Réponse introuvable
mail-context-forward-not-found = Transfert introuvable
mail-no-recipients = (aucun destinataire)
mail-attachment-downloaded = { $name } téléchargé
mail-attachment-download-failed = Le téléchargement a échoué.
mail-attachments-zip-done = { $count } pièces jointes téléchargées en ZIP.
mail-attachments-zip-capped = Le ZIP contient les { $count } premières pièces jointes ; la limite est de { $max } fichiers ou { $size }.
mail-focus-not-found = Message introuvable dans { $folder }
mail-focus-go-to = Aller à { $folder }
mail-nav-views-queues = Vues et files d’attente
mail-badge-unread = { $unread ->
    [one] { $unread } message non lu
   *[other] { $unread } messages non lus
  }
mail-badge-total = { $total ->
    [one] { $total } message dans ce dossier
   *[other] { $total } messages dans ce dossier
  }
mail-badge-both = { $unread ->
    [one] { $unread } message non lu
   *[other] { $unread } messages non lus
  } sur { $total ->
    [one] { $total } dans ce dossier
   *[other] { $total } dans ce dossier
  }
mail-badge-drafts = { $total ->
    [one] Brouillons : { $total } message
   *[other] Brouillons : { $total } messages
  }
mail-badge-compact-hint = Les dossiers autres que la boîte de réception utilisent un badge plus petit. Modifiez cela dans Paramètres, Dossiers et barre latérale.
mail-badge-outbox = { $count ->
    [one] Boîte d’envoi : { $count } message en attente d’envoi
   *[other] Boîte d’envoi : { $count } messages en attente d’envoi
  }
