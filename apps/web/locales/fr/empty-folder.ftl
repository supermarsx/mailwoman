# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Vider le dossier…
empty-folder-title = Vider « { $name } » ?
empty-folder-body-trash = { $count ->
    [one] Le message de { $name } sera déplacé dans la corbeille, d’où vous pourrez le restaurer.
   *[other] Les { $count } messages de { $name } seront déplacés dans la corbeille, d’où vous pourrez les restaurer.
    }
empty-folder-body-destroy-trash = { $count ->
    [one] Le message de la corbeille sera supprimé définitivement. Cette action est irréversible.
   *[other] Les { $count } messages de la corbeille seront supprimés définitivement. Cette action est irréversible.
    }
empty-folder-body-destroy-junk = { $count ->
    [one] Le message de { $name } sera supprimé définitivement. Cette action est irréversible.
   *[other] Les { $count } messages de { $name } seront supprimés définitivement. Cette action est irréversible.
    }
empty-folder-body-destroy-no-trash = Ce compte n’a pas de corbeille, donc { $count ->
    [one] le message de { $name } sera supprimé définitivement.
   *[other] les { $count } messages de { $name } seront supprimés définitivement.
    }
empty-folder-confirm-trash = Déplacer dans la corbeille
empty-folder-confirm-destroy = Supprimer définitivement
empty-folder-cancel = Annuler
empty-folder-already-empty = « { $name } » est déjà vide.
empty-folder-prepare-failed = « { $name } » n’a pas pu être dénombré, donc rien n’a été modifié. Vérifiez votre connexion et réessayez.
empty-folder-syncing = « { $name } » est encore en cours de synchronisation et ne peut pas encore être dénombré. Rien n’a été modifié ; réessayez dans un instant.
empty-folder-running = Un dossier est déjà en cours de vidage. Arrêtez-le depuis sa notification avant d’en démarrer un autre.
empty-folder-progress-trash = Déplacement des messages de { $name } vers la corbeille : { $done } sur { $total }.
empty-folder-progress-destroy = Suppression des messages de { $name } : { $done } sur { $total }.
empty-folder-stop = Arrêter
empty-folder-stopping = Arrêt après le lot en cours. { $done } sur { $total } traités jusqu’ici.
empty-folder-done-trash = { $done ->
    [one] 1 message de { $name } déplacé dans la corbeille.
   *[other] { $done } messages de { $name } déplacés dans la corbeille.
    }
empty-folder-done-destroy = { $done ->
    [one] 1 message de { $name } supprimé définitivement.
   *[other] { $done } messages de { $name } supprimés définitivement.
    }
empty-folder-stopped = Arrêté. { $done } messages traités sur { $total } ; { $remaining } restent dans { $name }.
empty-folder-refused = Arrêté. { $done } messages traités sur { $total } ; le serveur de messagerie en a refusé { $refused }. { $remaining } restent dans { $name }.
empty-folder-failed = Le vidage de { $name } s’est arrêté sur une erreur après { $done } messages. Le dernier lot a pu être terminé ou non ; vérifiez le dossier avant de réessayer.
