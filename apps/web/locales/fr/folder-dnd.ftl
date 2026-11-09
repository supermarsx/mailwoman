# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Dossier déplacé : { $name }
folder-dnd-moved-many = { $count } dossiers déplacés
folder-dnd-undo = Annuler
folder-dnd-cancel = Annuler
folder-dnd-handle = Faites glisser { $name } pour le déplacer
folder-dnd-locked-hint = L’ordre des dossiers est fixé par votre administrateur.
folder-dnd-forced-hint = Épinglé par votre administrateur. Il ne peut être ni déplacé ni désépinglé.
folder-dnd-disabled-hint = Le glisser-déposer des dossiers est désactivé.
folder-dnd-system-hint = Les dossiers système ne peuvent pas être déplacés.
folder-dnd-pinned-group = Épinglés
folder-dnd-top-level = Déplacer au niveau supérieur
folder-dnd-problem-disabled = Le déplacement des dossiers est désactivé.
folder-dnd-problem-locked = L’ordre des dossiers est fixé par votre administrateur.
folder-dnd-problem-self = Un dossier ne peut pas être déplacé dans lui-même.
folder-dnd-problem-descendant = Un dossier ne peut pas être déplacé dans l’un de ses propres sous-dossiers.
folder-dnd-problem-system = Les dossiers système ne peuvent pas être déplacés ici.
folder-dnd-problem-no-children = Ce dossier ne peut pas contenir de sous-dossiers.
folder-dnd-problem-nesting-off = Votre administrateur n’autorise pas l’imbrication des dossiers.
folder-dnd-problem-depth = Cela imbriquerait les dossiers trop profondément.
folder-dnd-problem-forced = Ce dossier est épinglé par votre administrateur.
folder-dnd-problem-unknown = Ce dossier n’existe plus.
folder-dnd-problem-noop = Le dossier s’y trouve déjà.
folder-dnd-problem-collision = Un dossier nommé { $name } existe déjà à cet endroit. Renommez-le d’abord.
folder-dnd-problem-server = Le serveur n’a pas déplacé { $name }. Rien n’a été modifié.
folder-dnd-announce-up = { $name } déplacé vers le haut
folder-dnd-announce-down = { $name } déplacé vers le bas
folder-dnd-announce-indent = { $name } déplacé dans le dossier au-dessus
folder-dnd-announce-outdent = { $name } remonté d’un niveau
folder-dnd-nest-title = Déplacer { $name } ici ?
folder-dnd-nest-title-many = Déplacer { $count } dossiers ici ?
folder-dnd-nest-body = Choisissez si cela modifie le dossier sur le serveur de messagerie ou seulement la façon dont cette application l’affiche.
folder-dnd-nest-legend = Comment le déplacer
folder-dnd-nest-local = Organiser uniquement dans cette application
folder-dnd-nest-local-help = Une simple vue. Le dossier reste à sa place sur le serveur et dans les autres applications de messagerie.
folder-dnd-nest-server = Déplacer sur le serveur
folder-dnd-nest-server-help = Modifie le dossier partout, dans toutes les applications de messagerie qui utilisent ce compte.
folder-dnd-nest-remember = Mémoriser mon choix
folder-dnd-nest-confirm = Déplacer
folder-dnd-arranged-only = Organisé uniquement dans cette application
folder-dnd-arranged-only-help = Sa place sur le serveur est différente : { $path }
folder-dnd-apply-server = Appliquer sur le serveur
folder-dnd-reset = Rétablir l’ordre par défaut
folder-dnd-reset-done = Ordre des dossiers rétabli
folder-dnd-move-to = Déplacer { $name } vers…
folder-dnd-layout-link = Disposition des dossiers
folder-dnd-reorder-enable = Réorganiser les dossiers par glisser-déposer
folder-dnd-nest-mode = Lorsque j’imbrique un dossier
folder-dnd-nest-mode-ask = Me demander
folder-dnd-nest-mode-server = Déplacer sur le serveur
folder-dnd-nest-mode-local = Organiser uniquement dans cette application
folder-dnd-new-placement = Les nouveaux dossiers apparaissent
folder-dnd-new-placement-end = Après les dossiers du même niveau
folder-dnd-new-placement-alphabetical = Par ordre alphabétique
folder-dnd-handles = Afficher les poignées de glissement
folder-dnd-handles-hover = Au survol
folder-dnd-handles-always = Toujours
folder-dnd-arrange-title = Organiser les dossiers
folder-dnd-arrange-help = Faites glisser un dossier pour le réordonner, déposez-le sur un autre dossier pour l’imbriquer, ou utilisez Alt et les touches fléchées sur un dossier ayant le focus.
folder-dnd-admin-locked = Défini par votre administrateur
folder-dnd-arrange-menu = Déplacer vers…
folder-star-add = Ajouter le dossier aux favoris
folder-star-remove = Retirer le dossier des favoris
folder-star-starred = Favori
folder-star-toggle = Mettre { $name } en favori
folder-star-only = Afficher uniquement les favoris
folder-star-only-help = Affiche uniquement les dossiers favoris et les dossiers qui les contiennent.
folder-dnd-indent = Déplacer { $name } dans le dossier au-dessus
folder-dnd-outdent = Remonter { $name } d’un niveau
folder-dnd-more-actions = Organiser
folder-dnd-selected-move = Déplacer la sélection vers…
folder-dnd-locked-row = Verrouillé
folder-dnd-problem-pin = Les dossiers épinglés restent au-dessus des autres. Désépinglez-le d’abord pour le déplacer à cet endroit.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Vide…

folder-bar-label = Actions pour les dossiers sélectionnés

folder-bar-mark-read = Marquer tout comme lu

folder-bar-pin = Épingle

folder-bar-star = Étoile

folder-bar-unpin = Désépingler

folder-bar-unstar = Supprimer la star

folder-dnd-handle-hint = Faites glisser pour déplacer ce dossier

folder-pinned-group = Épinglé

folder-pinned-unpin-zone = Déposez ici pour désépingler

settings-folders-bulk-delete-body = { $folders ->
        [0] { $folders } dossiers (avec leurs sous-dossiers) et les messages { $messages } qu'ils contiennent seront supprimés. Cela ne peut pas être annulé.
        [one] { $folders } dossier et les messages { $messages } qu'il contient seront supprimés. Cela ne peut pas être annulé.
        [many] { $folders } dossiers (avec leurs sous-dossiers) et les messages { $messages } qu'ils contiennent seront supprimés. Cela ne peut pas être annulé.
       *[other] { $folders } dossiers (avec leurs sous-dossiers) et les messages { $messages } qu'ils contiennent seront supprimés. Cela ne peut pas être annulé.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [0] Supprimer { $count } dossiers
        [one] Supprimer { $count } dossier
        [many] Supprimer { $count } dossiers
       *[other] Supprimer { $count } dossiers
    }

settings-folders-bulk-delete-done = { $count ->
        [0] Supprimé { $count } dossiers
        [one] { $count } dossier supprimé
        [many] Supprimé { $count } dossiers
       *[other] Supprimé { $count } dossiers
    }

settings-folders-bulk-delete-failed = Rien n'a été supprimé. Non supprimé : { $names }

settings-folders-bulk-delete-list = Dossiers à supprimer

settings-folders-bulk-delete-none = Aucun des dossiers sélectionnés ne peut être supprimé (les dossiers système et les dossiers que vous ne pouvez pas supprimer sont ignorés).

settings-folders-bulk-delete-others = Non supprimé non plus : { $names }

settings-folders-bulk-delete-partial = Dossiers { $done } supprimés. Non supprimé : { $names }

settings-folders-bulk-delete-partial-done = Dossiers { $done } supprimés.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [0] Non supprimé car le compte est sous conservation légale ({ $count } dossiers) : { $names }
        [one] Non supprimé car le compte est sous rétention légale : { $names }
        [many] Non supprimé car le compte est sous conservation légale ({ $count } dossiers) : { $names }
       *[other] Non supprimé car le compte est sous conservation légale ({ $count } dossiers) : { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [0] Non supprimé car le traitement du compte est restreint ({ $count } dossiers) : { $names }
        [one] Non supprimé car le traitement du compte est restreint : { $names }
        [many] Non supprimé car le traitement du compte est restreint ({ $count } dossiers) : { $names }
       *[other] Non supprimé car le traitement du compte est restreint ({ $count } dossiers) : { $names }
    }

settings-folders-bulk-delete-skipped = { $count } les dossiers sélectionnés sont ignorés car ils ne peuvent pas être supprimés : { $names }

settings-folders-bulk-delete-subfolders = Supprimez également les sous-dossiers de chaque dossier

settings-folders-bulk-delete-subfolders-off = Seuls les dossiers sélectionnés sont supprimés. Un dossier comportant encore un sous-dossier en dehors de la sélection est conservé.

settings-folders-bulk-delete-title = { $count ->
        [0] Supprimer { $count } dossiers ?
        [one] Supprimer { $count } dossier ?
        [many] Supprimer { $count } dossiers ?
       *[other] Supprimer { $count } dossiers ?
    }

settings-folders-bulk-empty-body = { $total } les messages dans les dossiers { $folders } seront supprimés : { $toTrash } seront déplacés vers la corbeille et pourront être annulés, { $permanent } seront définitivement supprimés et ne pourront pas être récupérés. Les sous-dossiers ne sont pas touchés.

settings-folders-bulk-empty-confirm = Vider les dossiers

settings-folders-bulk-empty-counting = Compter les messages dans chaque dossier…

settings-folders-bulk-empty-done = { $count } dossiers vidés, { $messages } messages

settings-folders-bulk-empty-fail-refused = le serveur en a refusé une partie ; { $remaining } messages restent

settings-folders-bulk-empty-fail-stopped = arrêté; { $remaining } messages restent

settings-folders-bulk-empty-fail-uncertain = le résultat du dernier lot est inconnu ; vérifie le dossier

settings-folders-bulk-empty-list = Dossiers à vider

settings-folders-bulk-empty-partial = Dossiers { $done } vidés ; { $failed } a échoué. Voir la liste.

settings-folders-bulk-empty-permanent = { $count } les messages sont définitivement supprimés

settings-folders-bulk-empty-progress = { $done } de { $of } dossiers terminés, { $messages } messages traités. Maintenant : { $name }

settings-folders-bulk-empty-skip-count-failed = ne pouvait pas être compté, ignoré

settings-folders-bulk-empty-skip-empty = déjà vide, ignoré

settings-folders-bulk-empty-skip-permanent-denied = serait définitivement supprimé, ce que l'administrateur a désactivé, ignoré

settings-folders-bulk-empty-skip-search = une recherche enregistrée, ignorée

settings-folders-bulk-empty-skip-unsupported = ce compte ne peut pas le vider, ignoré

settings-folders-bulk-empty-title = { $count ->
        [0] Vider { $count } dossiers ?
        [one] Vider { $count } dossier ?
        [many] Vider { $count } dossiers ?
       *[other] Vider { $count } dossiers ?
    }

settings-folders-bulk-empty-to-trash = { $count } les messages sont déplacés vers la corbeille

settings-folders-bulk-empty-undo-label = Dossiers { $count } vidés (messages { $messages })

settings-folders-delete-selected = Supprimer…

settings-folders-select-user = Sélectionnez tous les dossiers utilisateur

settings-folders-subscribe-failed = Les abonnements n'ont pas pu être modifiés.

settings-folders-subscribe-partial = Dossiers { $done } modifiés ; { $failed } n'a pas pu être modifié.

settings-folders-subscribe-selected = Abonnez-vous

settings-folders-subscribed = { $count ->
        [0] Abonné à { $count } dossiers
        [one] Abonné à { $count } dossier
        [many] Abonné à { $count } dossiers
       *[other] Abonné à { $count } dossiers
    }

settings-folders-unsubscribe-selected = Se désabonner

settings-folders-unsubscribed = { $count ->
        [0] Désabonné de { $count } dossiers
        [one] Désabonné de { $count } dossier
        [many] Désabonné de { $count } dossiers
       *[other] Désabonné de { $count } dossiers
    }
