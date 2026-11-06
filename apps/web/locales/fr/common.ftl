# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Annuler
common-close = Fermer
common-save = Enregistrer
common-delete = Supprimer
common-remove = Retirer
common-edit = Modifier
common-add = Ajouter
common-back = Retour
common-next = Suivant
common-done = Terminé
common-apply = Appliquer
common-confirm = Confirmer
common-retry = Réessayer
common-contact-administrator = Si le problème persiste, contactez votre administrateur.
common-shell-not-configured = Cette application n’a pas été configurée pour un serveur. Contactez votre administrateur.
common-copy = Copier
common-search = Rechercher
common-more = Plus
common-yes = Oui
common-no = Non

# -- Generic states ----------------------------------------------------------
common-loading = Chargement…
common-saving = Enregistrement…
common-empty = Rien pour l’instant
common-error = Une erreur s’est produite
common-error-network = Impossible de joindre le serveur. Vérifiez votre connexion et réessayez.
common-offline = Vous êtes hors ligne

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Connexion…
common-conn-degraded = Mises à jour en temps réel suspendues — reconnexion
common-conn-offline = Vous êtes hors ligne — les modifications seront synchronisées à la reconnexion
common-conn-auth-expired = Votre session a expiré — veuillez vous reconnecter
common-conn-reconnect = Se reconnecter
common-conn-sign-in-again = Se reconnecter
common-conn-reconnected = Reconnecté

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Nouveau message
common-notify-no-subject = (sans objet)

# -- Generic labels ----------------------------------------------------------
common-required = Obligatoire
common-optional = Facultatif

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Ouvrir les dossiers et applications
common-nav-close = Fermer les dossiers et applications

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Pièces jointes
common-attach-search = Rechercher des pièces jointes
common-attach-search-placeholder = filename:rapport type:pdf larger:1mb from:alice
common-attach-filter-type = Filtrer par type
common-attach-loading = Chargement des pièces jointes…
common-attach-empty = Aucune pièce jointe ne correspond.
common-attach-cat-all = toutes
common-attach-cat-image = image
common-attach-cat-pdf = pdf
common-attach-cat-text = texte
common-attach-cat-audio = audio
common-attach-cat-video = vidéo
common-attach-cat-other = autre

# -- Password fields (components/PasswordInput.tsx): the button at the end of
common-password-show = Afficher le mot de passe
common-password-hide = Masquer le mot de passe

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
common-module-mail = Courrier
common-module-calendar = Agenda
common-module-tasks = Tâches
common-module-notes = Notes
common-module-contacts = Contacts
common-module-keys = Clés

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
common-viewer-loading = Chargement de la visionneuse…
common-viewer-loading-audio = Chargement de l’audio…
common-viewer-loading-image = Chargement de l’image…
common-viewer-loading-video = Chargement de la vidéo…
common-viewer-rendering-pdf = Rendu du PDF…
common-viewer-pdf-failed = Impossible d’afficher ce PDF.
common-viewer-no-preview = Aucun aperçu intégré pour ce type de fichier.
common-viewer-unknown-type = type inconnu
common-viewer-download = Télécharger
common-viewer-attachments = Pièces jointes

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Nouvelle tentative…
common-waiting-online = En attente d’une connexion…
common-failure-offline = Vous êtes hors ligne. Reconnectez-vous, puis réessayez.
common-failure-network = Le serveur est injoignable.
common-failure-server = Le serveur n’a pas répondu à temps.
common-failure-rate = Trop de requêtes en peu de temps. Patientez un instant, puis réessayez.
common-failure-conflict = Cet élément a été modifié ailleurs en même temps. Rechargez, puis réessayez.
common-failure-account = Le compte connecté a changé. Rechargez la page.
common-failure-linked = Indisponible tant qu’une boîte aux lettres liée est ouverte.
common-failure-refused = Le serveur a refusé la modification.
common-new-version = Une nouvelle version de Mailwoman est disponible. Si vous fermez ce message, il vous sera rappelé dans une heure.
common-new-version-reload = Recharger maintenant
common-toast-close = Fermer la notification
common-toast-seconds = { $seconds } s
common-attach-refresh = Actualiser les pièces jointes
common-attach-open-message = Ouvrir le message contenant cette pièce jointe
common-attach-page-filter-note = Les filtres s’appliquent à cette page. Utilisez Suivant pour parcourir les messages plus anciens avec pièces jointes.
common-attach-pages = Pages de pièces jointes
common-attach-previous = Précédent
common-attach-next = Suivant
common-attach-page = Page { $page }
common-attach-column-name = Fichier
common-attach-column-type = Type
common-attach-column-size = Taille
common-attach-column-sender = Expéditeur
common-attach-column-subject = Objet
common-attach-column-date = Reçu
common-pdf-preview = Aperçu PDF
common-pdf-link-title = Ouvrir le lien de ce PDF ?
common-pdf-link-help = La destination s’ouvre dans un nouvel onglet, en dehors de Mailwoman.
common-pdf-link-open = Ouvrir le lien
common-open-event = Ouvrir l’événement
common-message-preview-too-large = L’aperçu des pièces jointes de type message est limité à 4 Mio. Vous pouvez toujours télécharger la pièce jointe d’origine.
common-message-preview-failed = L’aperçu de cette pièce jointe de type message est impossible. Le fichier n’est peut-être pas pris en charge ou est endommagé, ou le service d’aperçu est indisponible. Vous pouvez toujours télécharger l’original.
common-message-preview-attachments = Fichiers contenus dans ce message
common-message-preview-download = Télécharger le fichier
common-message-preview-size-bytes = { $count } octets
common-attach-cat-message = Messages / pièces jointes Outlook
common-error-retry-help = Cette section est temporairement indisponible. Réessayez pour la recharger.
common-error-network-help = Vérifiez votre connexion, puis réessayez. Le serveur est peut-être temporairement indisponible.
common-error-page = Cette page n’a pas pu se charger
common-error-page-help = Une ressource de la page n’a pas pu être téléchargée. Réessayez. Si le problème persiste, rechargez l’application pour obtenir la dernière version.
common-reload-app = Recharger l’application
common-new-version-reloading = Une nouvelle version est disponible, rechargement…
common-attach-account-changed = Le compte a changé pendant le chargement des pièces jointes. Rouvrez les pièces jointes pour le compte actuel.
common-attach-search-account = Rechercher dans tout le compte
common-attach-scanning = Recherche en cours… { $pages } pages de messages vérifiées · { $count } fichiers trouvés
common-attach-scan-cancelled = Recherche annulée · { $count } fichiers trouvés sur { $pages } pages. Les résultats sont incomplets.
common-attach-scan-incomplete = Recherche interrompue · { $count } fichiers trouvés. Relancez la recherche pour parcourir tout le compte.
common-attach-scan-limited = Affichage des 1 000 premiers fichiers correspondants. Affinez vos filtres pour trouver des fichiers plus précis.
common-attach-scan-complete = Recherche terminée · { $count } fichiers trouvés sur { $pages } pages de messages
common-attach-browse-pages = Parcourir les pages
common-attach-cancel-search = Annuler la recherche
multiselect-selected = Sélectionné : { $label }
multiselect-remove = Supprimer { $label }
multiselect-no-match = Aucune option correspondante
common-notification-permission-title = Autoriser les notifications ?
common-notification-permission-help = Soyez averti des nouveaux messages sur cet appareil. Vous pouvez modifier ce réglage dans les paramètres.
common-notification-permission-allow = Autoriser les notifications
common-notification-permission-later = Pas maintenant
common-notification-permission-error = Les notifications n’ont pas pu être activées. Vérifiez les autorisations et les paramètres de notification, puis réessayez.
common-notifications-enabled = Notifications activées sur cet appareil.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [one] 1 sélectionné
   *[other] { $count } sélectionnés
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = Le serveur est occupé. Patientez quelques secondes, puis réessayez.
