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
