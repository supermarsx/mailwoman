# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Profils de fonctionnalités
admin-features-intro = Un profil est un ensemble nommé d'interrupteurs de fonctionnalités. Attribuez-le à un domaine, à un groupe ou à un utilisateur. Les attributions individuelles prennent en charge les boîtes aux lettres natives et les identités proxy ou SSO vérifiées ; elles contrôlent l'accès aux fonctionnalités de Mailwoman.
admin-features-new = Nouveau profil
admin-features-search = Rechercher des profils
admin-features-empty = Aucun profil de fonctionnalités pour l'instant.
admin-features-no-match = Aucun profil ne correspond à la recherche.
admin-features-load-error = Le catalogue des fonctionnalités n'a pas pu être chargé.
admin-features-not-loaded = Le catalogue des fonctionnalités n'est pas encore chargé. Rechargez la page et réessayez.
admin-features-stale = Un autre administrateur a modifié le catalogue des fonctionnalités. Il a été rechargé ; vérifiez les valeurs et enregistrez à nouveau.
admin-features-not-found = Aucun profil avec l'identifiant { $id }.
admin-features-col-name = Nom
admin-features-col-values = Interrupteurs
admin-features-col-assigned = Attribué à
admin-features-values-count = { $count ->
    [one] { $count } interrupteur défini
   *[other] { $count } interrupteurs définis
}
admin-features-assigned-count = { $count ->
    [0] Non attribué
    [one] { $count } emplacement
   *[other] { $count } emplacements
}
admin-features-editor = Paramètres du profil
admin-features-name = Nom du profil
admin-features-name-help = Visible uniquement des administrateurs, 128 caractères au maximum.
admin-features-name-invalid = Saisissez un nom de 1 à 128 caractères.
admin-features-switches = Interrupteurs de fonctionnalités
admin-features-switches-help = « Non défini » laisse la fonctionnalité au profil suivant, au domaine ou à la valeur par défaut du serveur. Une valeur verrouillée ne peut être modifiée ni par l'utilisateur ni par un niveau inférieur.
admin-features-col-feature = Fonctionnalité
admin-features-col-value = Valeur
admin-features-col-lock = Verrou
admin-features-value-for = Valeur pour { $feature }
admin-features-unset = Non défini
admin-features-on = Autorisé
admin-features-off = Non autorisé
admin-features-lock = Verrouillé
admin-features-save = Enregistrer le profil
admin-features-create = Créer le profil
admin-features-delete = Supprimer le profil
admin-features-delete-title = Supprimer le profil { $name } ?
admin-features-delete-body = { $count ->
    [0] Le profil n'est attribué nulle part. Ses interrupteurs cessent de s'appliquer immédiatement.
    [one] Le profil est retiré de l'{ $count } emplacement auquel il est attribué, et ses interrupteurs cessent de s'appliquer immédiatement.
   *[other] Le profil est retiré des { $count } emplacements auxquels il est attribué, et ses interrupteurs cessent de s'appliquer immédiatement.
}
admin-features-saved = Profil { $name } enregistré.
admin-features-deleted = Profil { $name } supprimé.
admin-features-assigned = Profil { $name } attribué.
admin-features-unassigned = Attribution de { $name } retirée.
admin-features-assignments = Attribué à
admin-features-assignments-help = Les profils s'appliquent par ordre de priorité, les plus élevés d'abord. À priorité égale, une attribution à un utilisateur s'applique avant celle d'un groupe, et celle d'un groupe avant celle d'un domaine.
admin-features-assignments-empty = Ce profil n'est pas encore attribué.
admin-features-assign-form = Attribuer ce profil
admin-features-col-scope = S'applique à
admin-features-col-target = Nom
admin-features-col-priority = Priorité
admin-features-col-actions = Actions
admin-features-scope-domain = Domaine
admin-features-scope-group = Groupe
admin-features-scope-user = Utilisateur
admin-features-assign = Attribuer
admin-features-unassign = Retirer
admin-features-no-domain = Aucun domaine n'est configuré.
admin-features-no-group = Aucun groupe d'utilisateurs n'existe.
admin-features-no-user = Aucun compte de boîte aux lettres ne s'est encore connecté.
admin-features-tools = Outils de profil
admin-features-preset = Partir d'un préréglage
admin-features-preset-choose = Choisir un préréglage
admin-features-preset-apply = Appliquer le préréglage
admin-features-preset-applied = Préréglage { $name } appliqué. Vérifiez les valeurs et enregistrez.
admin-features-preset-full = Complet
admin-features-preset-standard = Standard
admin-features-preset-mail-only = Courrier uniquement
admin-features-preset-minimal = Minimal / borne
admin-features-import = Importer du JSON
admin-features-export = Exporter en JSON
admin-features-exported = Profil exporté sous forme de fichier JSON.
admin-features-imported = { $count ->
    [one] { $count } valeur importée. Vérifiez-la et enregistrez.
   *[other] { $count } valeurs importées. Vérifiez-les et enregistrez.
}
admin-features-import-json = Le fichier n'est pas du JSON valide.
admin-features-import-format = Le fichier n'est pas un profil de fonctionnalités Mailwoman.
admin-features-import-version = Ce fichier de profil de fonctionnalités provient d'une version plus récente et ne peut pas être importé.
admin-features-clone = Cloner
admin-features-clone-name = Copie de { $name }
admin-features-filter = Rechercher une fonctionnalité
admin-features-filter-placeholder = Nom, paramètre ou clé
admin-features-filter-none = Aucune fonctionnalité ne correspond à la recherche.
admin-features-diff-summary = { $differs ->
    [one] { $differs } valeur diffère de la valeur par défaut du serveur
   *[other] { $differs } valeurs diffèrent de la valeur par défaut du serveur
} · { $same } la répètent
admin-features-differs = Diffère de la valeur par défaut
admin-features-default-is = Par défaut : { $value }
admin-features-fix-values = Corrigez les valeurs marquées comme invalides avant d'enregistrer.
admin-features-allow-all = Tout autoriser
admin-features-deny-all = Tout interdire
admin-features-clear-all = Effacer
admin-features-group-allow = Autoriser toutes les fonctionnalités de { $group }
admin-features-group-deny = Interdire toutes les fonctionnalités de { $group }
admin-features-group-clear = Effacer toutes les valeurs de { $group }
admin-features-enforcement-ui = Masqué dans l'application uniquement
admin-features-enforcement-partial = Certaines limites ne sont pas encore comptabilisées
admin-features-other = Autres paramètres qu'un profil peut contenir
admin-features-other-help = Tous les autres paramètres qu'un administrateur peut définir par profil. Laissez un champ vide pour conserver la valeur du serveur.
admin-features-cat-compose = Courrier et rédaction
admin-features-cat-reading = Lecture et confidentialité
admin-features-cat-views = Vues et recherche
admin-features-cat-organise = Organisation
admin-features-cat-pim = Agenda, contacts, tâches, notes et clés
admin-features-cat-data = Données
admin-features-cat-accounts = Comptes et accès
admin-features-cat-integrations = Intégrations
admin-features-cat-developer = Accès développeur
admin-features-cat-personal = Personnalisation
admin-features-feature-compose-sending = Limites d'envoi
admin-features-feature-compose-sending-help = Le nombre de destinataires par message est refusé par le serveur au-delà de la limite. Les messages par heure et par jour sont enregistrés pour le profil mais pas encore comptabilisés.
admin-features-feature-storage = Quota de stockage
admin-features-feature-storage-help = La limite de stockage est enregistrée pour le profil ; le quota propre au serveur de messagerie continue de s'appliquer.
admin-features-feature-access-protocols = Protocoles d'accès au courrier
admin-features-feature-access-protocols-help = Avec quels protocoles de serveur de messagerie ces utilisateurs peuvent se connecter, séparés par des virgules : imap, pop3, ews.
admin-features-feature-accounts-linked = Boîtes aux lettres liées
admin-features-feature-calendar = Agenda
admin-features-feature-contacts = Contacts
admin-features-feature-tasks = Tâches
admin-features-feature-notes = Notes
admin-features-preview = Valeurs effectives
admin-features-preview-help = Ce que devient chaque fonctionnalité pour un utilisateur ou pour un utilisateur d'un domaine, après chaque profil, groupe, valeur de domaine et verrou.
admin-features-preview-kind = Aperçu pour
admin-features-preview-target = Qui
admin-features-preview-show = Afficher
admin-features-preview-loading = Chargement…
admin-features-preview-state = Effectif
admin-features-preview-from = Décidé par
admin-features-preview-profiles = Profils qui s'appliquent, le premier l'emporte : { $names }
admin-features-preview-no-profiles = Aucun profil ne s'applique ; les valeurs du domaine et du serveur décident.
admin-features-source-default = Valeur intégrée par défaut
admin-features-source-env-fallback = Valeur par défaut de l'environnement
admin-features-source-env-pin = Environnement (imposé)
admin-features-source-global = Valeur du serveur
admin-features-source-domain = Valeur du domaine
admin-features-source-profile = Profil
admin-features-source-profile-named = Profil { $name }
admin-features-source-group = Valeur du groupe
admin-features-source-admin-user = Valeur définie pour cet utilisateur
admin-features-source-user = Choix de l'utilisateur
admin-features-source-locked = { $source } (verrouillé)
admin-retention-col-category = Catégorie
admin-retention-col-kept = Conservé
admin-retention-col-change = Modifier
admin-retention-cat-default = Par défaut
admin-retention-cat-admin = Actions d'administrateur
admin-retention-cat-security = Événements de sécurité
admin-retention-cat-alert = Alertes utilisateur
admin-retention-forever = Toutes les entrées
admin-retention-days = { $days ->
    [one] { $days } jour
   *[other] { $days } jours
}
admin-retention-follows-default = Par défaut : { $kept }
admin-retention-hold-on = Conservation légale
admin-retention-read-only = Modifier la conservation nécessite l'autorisation settings.write.
admin-retention-edit-for = Conservation pour { $category }
admin-retention-keep = Conserver
admin-retention-mode-inherit = Suivre la valeur par défaut
admin-retention-mode-forever = Toutes les entrées
admin-retention-mode-days = Pendant un nombre de jours
admin-retention-days-label = Jours
admin-retention-hold = Conservation légale
admin-retention-save = Enregistrer
admin-retention-bounds = Saisissez un nombre entier de { $min } à { $max }.
admin-retention-saved = Conservation pour { $category } enregistrée.
admin-retention-task-none = Aucune tâche de nettoyage pour l'instant. Elle est créée, chaque jour à 03:15 UTC, lorsqu'un nombre de jours est enregistré pour la première fois.
admin-retention-task-disabled = La tâche de nettoyage planifiée est désactivée ; les entrées ne sont nettoyées qu'avec Nettoyer maintenant.
admin-retention-task-next = Prochain nettoyage planifié : { $when }.
admin-retention-task-daily = Nettoyage quotidien à 03:15 UTC.
admin-retention-last = Dernier nettoyage ayant supprimé des entrées : { $when }.
admin-retention-prune = Nettoyer maintenant
admin-retention-prune-title = Nettoyer le journal d'audit maintenant ?
admin-retention-prune-body = Les entrées plus anciennes que la durée de conservation de leur catégorie sont supprimées. Les catégories qui conservent toutes les entrées ou sont sous conservation légale ne sont pas modifiées. Cette action est irréversible.
admin-retention-pruned = { $count ->
    [0] Aucune entrée n'était assez ancienne pour être supprimée.
    [one] { $count } entrée supprimée.
   *[other] { $count } entrées supprimées.
}
admin-auth-attempted = Tentative en tant que { $name }
admin-features-feature-compose-open-tracking = Suivi de l'ouverture des messages
admin-features-feature-compose-open-tracking-help = Autorise l'enregistrement des ouvertures des messages mis en forme éligibles à l'aide de l'image de suivi du serveur. Nécessite une URL publique configurée.
admin-features-feature-reading-original-html = Affichage du HTML d'origine
admin-features-feature-reading-original-html-help = Autorise les utilisateurs à activer le HTML d'origine pour des messages individuels, dans le cadre de lecture isolé. Le HTML assaini reste la valeur par défaut.
admin-features-inherit = Hériter
admin-features-override = Définir une valeur
admin-features-setting-mode = Mode de valeur pour { $setting }
admin-features-inherit-help = Hériter supprime la valeur et le verrou de ce profil. Le profil applicable suivant, le domaine ou la valeur du serveur décide ; l'aperçu des valeurs effectives montre le résultat et sa source.
admin-features-domain-alias = { $alias } (alias de { $domain })
admin-features-domain-aliases = { $domain } (alias : { $aliases })
admin-audit-retention-title = Conservation du journal d’audit
admin-audit-retention-intro = Durée de conservation des entrées d’audit, par catégorie. Une catégorie sans valeur propre suit la valeur par défaut ; sans valeur par défaut, toutes les entrées sont conservées. Une conservation légale suspend la purge de sa catégorie, et une conservation sur la valeur par défaut la suspend pour toutes. Les modifications et les purges sont consignées dans le journal d’audit.
admin-audit-retention-loading = Chargement de la politique de conservation…
admin-audit-retention-unavailable = La conservation de l’audit n’est pas disponible sur ce serveur.
