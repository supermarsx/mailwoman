# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Utilisateurs. Les en-têtes de colonne trient la liste.
admin-userlist-search = Rechercher des utilisateurs
admin-userlist-search-placeholder = Nom, adresse, alias ou domaine
admin-userlist-count = { $count ->
    [one] { $count } utilisateur
   *[other] { $count } utilisateurs
}
admin-userlist-count-filtered = { $count ->
    [one] { $count } utilisateur sur { $total } correspond
   *[other] { $count } utilisateurs sur { $total } correspondent
}
admin-userlist-range = { $from }–{ $to } sur { $total }
admin-userlist-loading = Chargement des utilisateurs…
admin-userlist-retry = Réessayer
admin-userlist-empty-filtered = Aucun utilisateur ne correspond à cette recherche et à ces filtres.
admin-userlist-compact = Lignes compactes
admin-userlist-columns = Colonnes
admin-userlist-columns-legend = Afficher les colonnes
admin-userlist-actions = Actions
admin-userlist-paging = Pages d'utilisateurs
admin-userlist-page-size = Lignes par page
admin-userlist-prev = Précédent
admin-userlist-next = Suivant
admin-userlist-sort-by = Trier par
admin-userlist-sort-asc = { $column }, croissant
admin-userlist-sort-desc = { $column }, décroissant
admin-userlist-col-user = Utilisateur
admin-userlist-col-domain = Domaine
admin-userlist-col-status = Statut
admin-userlist-col-2fa = Double authentification
admin-userlist-col-role = Rôle de console
admin-userlist-col-last-signin = Dernière connexion
admin-userlist-col-last-seen = Dernière activité
admin-userlist-col-created = Créé
admin-userlist-col-quota = Quota
admin-userlist-col-profile = Profil de fonctionnalités
admin-userlist-col-linked = Boîtes aux lettres liées
admin-userlist-state-active = Actif
admin-userlist-state-pending = Jamais connecté
admin-userlist-state-blocked = Bloqué
admin-userlist-state-disabled = Désactivé
admin-userlist-state-deleted = Supprimé
admin-userlist-2fa-passkey = Clé d'accès
admin-userlist-2fa-totp = Application d'authentification
admin-userlist-2fa-none = Aucune
admin-userlist-signin-never = Jamais
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Aucune activité enregistrée
admin-userlist-seen-at = Dernière activité { $relative } ({ $exact })
admin-userlist-quota-unlimited = Illimité
admin-userlist-profile-default = Par défaut
admin-userlist-profile-via-user = Attribué à cet utilisateur
admin-userlist-profile-via-group = Issu d'un groupe
admin-userlist-profile-via-domain = Issu du domaine
admin-userlist-more = Autres actions
admin-userlist-more-for = Autres actions pour { $account }
admin-userlist-select = Sélectionner { $account }
admin-userlist-select-page = Sélectionner tous les utilisateurs de cette page
admin-userlist-filters = Filtres
admin-userlist-add-filter = Ajouter un filtre
admin-userlist-clear-all = Tout effacer
admin-userlist-chip-edit = Modifier le filtre { $label }
admin-userlist-chip-remove = Supprimer le filtre { $label }
admin-userlist-filter-domain = Domaine
admin-userlist-filter-status = Statut
admin-userlist-filter-tfa = Double authentification
admin-userlist-filter-role = Rôle de console
admin-userlist-filter-profile = Profil de fonctionnalités
admin-userlist-filter-signin = Dernière connexion
admin-userlist-filter-created = Créé
admin-userlist-filter-no-choices = Rien à choisir pour l'instant.
admin-userlist-filter-remove = Supprimer le filtre
admin-userlist-apply = Appliquer
admin-userlist-cancel = Annuler
admin-userlist-role-any = N'importe quel rôle de console
admin-userlist-role-none = Aucun rôle de console
admin-userlist-profile-none = Aucun profil (par défaut)
admin-userlist-signin-older = { $days ->
    [one] Il y a plus de { $days } jour
   *[other] Il y a plus de { $days } jours
}
admin-userlist-signin-within = { $days ->
    [one] Au cours du dernier { $days } jour
   *[other] Au cours des { $days } derniers jours
}
admin-userlist-signin-custom = Il y a plus de ce nombre de jours :
admin-userlist-signin-days = Jours
admin-userlist-days-invalid = Saisissez un nombre entier de jours compris entre 1 et 36500.
admin-userlist-created-from = Du
admin-userlist-created-to = Au
admin-userlist-created-between = du { $from } au { $to }
admin-userlist-created-since = depuis le { $date }
admin-userlist-created-until = jusqu'au { $date }
admin-userlist-dates-invalid = La date de début est postérieure à la date de fin.
admin-userlist-views = Vues enregistrées
admin-userlist-view-save = Enregistrer cette vue…
admin-userlist-view-name = Nom de la vue
admin-userlist-view-save-confirm = Enregistrer la vue
admin-userlist-view-delete = Supprimer la vue « { $name } »
admin-userlist-bulk = Actions sur les utilisateurs sélectionnés
admin-userlist-selected = { $count ->
    [one] { $count } sélectionné
   *[other] { $count } sélectionnés
}
admin-userlist-clear-selection = Effacer la sélection
admin-userlist-block = Bloquer la connexion
admin-userlist-unblock = Débloquer la connexion
admin-userlist-sign-out = Déconnecter partout
admin-userlist-delete = Supprimer l'accès local
admin-userlist-bulk-profile = Attribuer un profil de fonctionnalités
admin-userlist-bulk-profile-clear = Retirer le profil propre à l'utilisateur
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } utilisateur sélectionné est ignoré : l'action ne s'applique pas à lui.
   *[other] { $count } utilisateurs sélectionnés sont ignorés : l'action ne s'applique pas à eux.
}
admin-userlist-confirm-block = { $count ->
    [one] Bloquer la connexion de { $count } utilisateur ?
   *[other] Bloquer la connexion de { $count } utilisateurs ?
}
admin-userlist-confirm-block-one = Bloquer la connexion de { $account } ?
admin-userlist-confirm-block-body = Les nouvelles connexions sont refusées et les sessions Mailwoman en cours prennent fin. La boîte aux lettres en amont n'est pas modifiée.
admin-userlist-confirm-block-action = Bloquer la connexion
admin-userlist-confirm-unblock = { $count ->
    [one] Débloquer la connexion de { $count } utilisateur ?
   *[other] Débloquer la connexion de { $count } utilisateurs ?
}
admin-userlist-confirm-unblock-one = Débloquer la connexion de { $account } ?
admin-userlist-confirm-unblock-body = Le blocage local de la connexion est levé. Les autres restrictions de sécurité restent en vigueur.
admin-userlist-confirm-unblock-action = Débloquer la connexion
admin-userlist-confirm-signout = { $count ->
    [one] Déconnecter { $count } utilisateur partout ?
   *[other] Déconnecter { $count } utilisateurs partout ?
}
admin-userlist-confirm-signout-one = Déconnecter { $account } partout ?
admin-userlist-confirm-signout-body = Toutes les sessions Mailwoman ouvertes de ces utilisateurs prennent fin. Ils peuvent se reconnecter.
admin-userlist-confirm-signout-action = Déconnecter
admin-userlist-confirm-delete = { $count ->
    [one] Supprimer l'accès local de { $count } utilisateur ?
   *[other] Supprimer l'accès local de { $count } utilisateurs ?
}
admin-userlist-confirm-delete-one = Supprimer l'accès local de { $account } ?
admin-userlist-confirm-delete-body = Un enregistrement d'accès supprimé est conservé afin que l'utilisateur ne soit pas réinscrit automatiquement. Les paramètres Mailwoman, l'historique et la boîte aux lettres en amont restent stockés. Saisissez l'adresse pour confirmer.
admin-userlist-confirm-delete-action = Supprimer l'accès local
admin-userlist-confirm-profile = { $count ->
    [one] Modifier le profil de fonctionnalités de { $count } utilisateur ?
   *[other] Modifier le profil de fonctionnalités de { $count } utilisateurs ?
}
admin-userlist-confirm-profile-one = Modifier le profil de fonctionnalités de { $account } ?
admin-userlist-confirm-profile-body = Le profil est attribué directement à chaque utilisateur et s'applique dès sa prochaine requête. Les identités de boîte aux lettres natives, proxy et SSO sont prises en charge.
admin-users-profile-help = Sélectionnez Par défaut pour supprimer l'attribution directe de cet utilisateur et utiliser les profils hérités de ses groupes ou de son domaine.
admin-users-profile-ambiguous = Cet utilisateur a plusieurs identités de connexion. Choisissez l'identité voulue dans Profils de fonctionnalités.
admin-users-profile-unavailable = Aucune identité de boîte aux lettres vérifiée n'est disponible. Cet utilisateur doit se connecter avant qu'un profil puisse lui être attribué.
admin-userlist-confirm-profile-action = Modifier le profil
admin-userlist-bulk-done-block = { $count ->
    [one] Connexion bloquée pour { $count } utilisateur.
   *[other] Connexion bloquée pour { $count } utilisateurs.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Connexion débloquée pour { $count } utilisateur.
   *[other] Connexion débloquée pour { $count } utilisateurs.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] { $count } utilisateur déconnecté.
   *[other] { $count } utilisateurs déconnectés.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Accès local supprimé pour { $count } utilisateur.
   *[other] Accès local supprimé pour { $count } utilisateurs.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Profil de fonctionnalités modifié pour { $count } utilisateur.
   *[other] Profil de fonctionnalités modifié pour { $count } utilisateurs.
}
admin-userlist-bulk-failed = { $done } terminées, { $failed } en échec : { $reason }
