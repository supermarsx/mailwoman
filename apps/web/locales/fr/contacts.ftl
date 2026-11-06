# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Contacts
contacts-search = Rechercher des contacts
contacts-new = Nouveau contact
contacts-find-duplicates = Trouver les doublons
contacts-loading = Chargement…
contacts-none = Aucun contact.
contacts-list = Liste des contacts
contacts-favorite = Mettre { $name } en favori
contacts-export-vcard = Exporter en vCard
contacts-export-csv = Exporter en CSV
contacts-select-hint = Sélectionnez un contact pour afficher sa fiche.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Carnets d’adresses et groupes
contacts-address-books = Carnets d’adresses
contacts-all = Tous les contacts
contacts-favorites = Favoris
contacts-groups = Groupes
contacts-new-group = Nouveau groupe
contacts-new-group-name = Nom du nouveau groupe
contacts-group-name-placeholder = Nom du groupe
contacts-create = Créer
contacts-data = Données
contacts-import = Importer…

# -- Business card -----------------------------------------------------------
contacts-card = Contact { $name }
contacts-favorited = En favori
contacts-favorite-action = Favori
contacts-email = E-mail
contacts-phone = Téléphone
contacts-dates = Dates
contacts-groups-label = Groupes
contacts-notes = Notes
contacts-security-key = Clé de sécurité
contacts-key-on-file = Une clé ou un certificat est enregistré (affichage seul tant que PGP/S-MIME n’est pas pris en charge).
contacts-no-key = Aucune clé enregistrée.
contacts-directory-security = Sécurité de l’annuaire
contacts-member-of = Membre de
contacts-group-membership = Appartenance à { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Nouveau contact
contacts-edit-contact = Modifier le contact
contacts-full-name = Nom complet
contacts-given = Prénom
contacts-given-placeholder = Prénom
contacts-surname = Nom
contacts-surname-placeholder = Nom
contacts-organization = Organisation
contacts-job-title = Fonction
contacts-job-title-placeholder = Fonction
contacts-email-n = E-mail { $n }
contacts-email-label = Libellé de l’e-mail { $n }
contacts-email-label-placeholder = travail
contacts-remove-email = Supprimer l’e-mail { $n }
contacts-add-email = Ajouter un e-mail
contacts-phone-n = Téléphone { $n }
contacts-remove-phone = Supprimer le téléphone { $n }
contacts-add-phone = Ajouter un téléphone
contacts-security-key-label = Clé de sécurité (valeur opaque)
contacts-security-key-placeholder = Clé PGP / certificat S-MIME (affichage seul)
contacts-favorite-word = Favori

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Importer des contacts
contacts-imported = { $count ->
    [one] { $count } contact importé
   *[other] { $count } contacts importés
}.
contacts-import-hint = Collez du vCard (.vcf) ou du CSV, ou choisissez un fichier. Le format est détecté automatiquement.
contacts-import-file = Importer un fichier
contacts-paste = Coller du vCard ou du CSV
contacts-preview = Aperçu
contacts-map-columns = Associer les colonnes
contacts-col = Colonne
contacts-maps-to = Correspond à
contacts-sample = Exemple
contacts-map-column = Associer la colonne { $header }
contacts-preview-count = Aperçu ({ $count })
contacts-import-preview = Aperçu de l’import
contacts-no-name = (sans nom)
contacts-no-email = aucun e-mail
contacts-import-n = Importer { $count ->
    [one] { $count } contact
   *[other] { $count } contacts
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Fusionner les doublons
contacts-merge-review-hint = Vérifiez la fiche fusionnée. Les autres fiches sont marquées comme supprimées (réversible).
contacts-merged-preview = Aperçu de la fusion
contacts-emails = E-mails
contacts-phones = Téléphones
contacts-merge-confirm = Fusionner les contacts
contacts-no-duplicates = Aucun doublon trouvé.
contacts-possible-duplicates = { $count } doublons possibles
contacts-review-merge = Vérifier la fusion

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Coordonnées du contact
contacts-address-recipients = Afficher les destinataires
contacts-address-save = Enregistrer le contact
contacts-address-compose = Envoyer un e-mail
contacts-address-unsaved = Cette adresse n’est pas enregistrée dans vos contacts.
contacts-address-book = Carnet d’adresses
contacts-address-lookup-failed = Les informations du contact n’ont pas pu être chargées. Réessayez.
contacts-address-save-failed = Le contact n’a pas pu être enregistré. Réessayez.
contacts-address-saved = Contact enregistré.
contacts-address-loading = Chargement du contact…
contacts-editor-person = Nom et travail
contacts-editor-reach = E-mail et téléphone
contacts-editor-other = Notes et préférences
contacts-editor-save-error = Le contact n’a pas pu être enregistré. Vos modifications sont conservées. Réessayez.
contacts-group-editor-help = Créez un groupe dans un carnet d’adresses, puis ajoutez des membres depuis leurs fiches de contact.
contacts-group-details = Détails du groupe
contacts-group-save-error = Le groupe n’a pas pu être enregistré. Vos modifications sont conservées. Réessayez.
contacts-edit-group = Modifier le groupe de contacts
contacts-edit-named-group = Modifier le groupe { $name }
contacts-group-name = Nom du groupe
contacts-group-edit-help = Modifiez le nom de ce groupe. Son carnet d’adresses et ses membres restent inchangés.
contacts-back-to-list = Retour aux contacts
contacts-membership-title = Appartenance au groupe
contacts-membership-help = Choisissez un groupe, puis ajoutez ou retirez ses membres. Les modifications sont enregistrées au fur et à mesure.
contacts-membership-no-groups = Aucun groupe pour l’instant. Créez-en un pour commencer à ajouter des membres.
contacts-membership-pick = Sélectionnez un groupe pour voir ses membres.
contacts-membership-of = Membres de { $name }
contacts-membership-members = Membres
contacts-membership-add = Saisissez un nom ou une adresse pour ajouter un membre
contacts-membership-add-group = Saisissez un nom de groupe pour y ajouter ce contact
contacts-membership-count = { $count ->
    [one] 1 membre
   *[other] { $count } membres
    }
contacts-membership-failed = Certaines modifications d’appartenance n’ont pas pu être enregistrées. La liste affiche l’état enregistré.
contacts-membership-manage = Gérer l’appartenance au groupe
contacts-import-source = Source
contacts-import-book = Carnet d’adresses
contacts-import-review = Vérifier
contacts-import-empty = Choisissez un fichier ou collez du texte vCard ou CSV, puis sélectionnez Aperçu. Les contacts trouvés sont listés ici et vous pouvez en exclure.
contacts-merge-sets = Doublons possibles
contacts-merge-set = Doublons de { $name }
contacts-merge-keep = Fiche à conserver
contacts-merge-done = Fusionné avec { $name }.
contacts-address-actions = Actions du contact
contacts-address-save-to = Enregistrer dans les contacts
contacts-address-goto = Aller au contact
contacts-address-copy = Copier les informations du contact
contacts-address-copied = Informations du contact copiées.
contacts-address-copy-failed = Impossible de copier les informations du contact.
contacts-share = Partager le contact
contacts-share-email = Envoyer le contact par e-mail
contacts-share-copy = Copier en vCard
contacts-share-download = Télécharger le .vcf
contacts-share-native = Partager…
contacts-share-copied = vCard copiée.
contacts-share-copy-failed = Impossible de copier la vCard.
contacts-share-downloaded = vCard téléchargée.
contacts-share-failed = Le contact n’a pas pu être partagé.
contacts-share-attach-failed = Le fichier .vcf n’a pas pu être joint ; la vCard figure donc dans le texte du message.
contacts-address-saved-favorite = Contact enregistré et ajouté aux favoris.
contacts-favorite-add = Ajouter aux favoris
contacts-favorite-remove = Retirer des favoris
contacts-favorite-save = Enregistrer et ajouter aux favoris
contacts-favorite-added = { $name } ajouté aux favoris.
contacts-favorite-removed = { $name } retiré des favoris.
contacts-favorite-failed = Les favoris n’ont pas pu être mis à jour.
contacts-select-row = Sélectionner { $name }
contacts-selected-count = { $count } sélectionné(s)
contacts-select-clear = Effacer la sélection
contacts-bulk-actions = Actions pour les contacts sélectionnés
contacts-bulk-favorites-added = { $count } ajouté(s) aux favoris.
contacts-bulk-favorites-removed = { $count } retiré(s) des favoris.
contacts-row-menu = Actions pour { $name }
contacts-favorites-empty = Aucun favori pour l’instant. Utilisez l’étoile d’un contact pour en ajouter.
