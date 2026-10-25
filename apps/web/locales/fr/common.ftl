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
