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
