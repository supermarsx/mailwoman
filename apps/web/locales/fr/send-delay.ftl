# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Annuler l’envoi
send-delay-help = Après avoir appuyé sur Envoyer, le message attend dans votre boîte d’envoi pendant ce délai. Jusque-là, vous pouvez le rappeler et continuer à le modifier. Le serveur le conserve : fermer le navigateur ne l’envoie pas trop tôt et ne le perd pas.
send-delay-label = Retenir les messages envoyés pendant
send-delay-off = Désactivé (envoi immédiat)
send-delay-seconds = { $n } secondes
send-delay-minutes = { $n ->
    [one] 1 minute
   *[other] { $n } minutes
}
send-delay-custom = Personnalisé…
send-delay-custom-label = Délai personnalisé en secondes
send-delay-custom-help = Secondes entières, de 0 à { $max }.
send-delay-capped = Votre administrateur limite cette valeur à { $max }.
send-delay-disabled = Le rappel des messages envoyés est désactivé par votre administrateur. Les messages sont envoyés immédiatement.
send-delay-invalid = Saisissez un nombre entier de secondes de 0 à { $max }.
send-later-button = Envoyer plus tard
send-later-menu = Options d’envoi différé
send-later-preset-hour = Dans 1 heure
send-later-preset-today = Plus tard aujourd’hui, 17:00
send-later-preset-tomorrow = Demain matin, 09:00
send-later-preset-monday = Lundi matin, 09:00
send-later-custom = Choisir la date et l’heure…
send-later-custom-label = Envoyer le
send-later-zone = Fuseau horaire : { $zone }
send-later-schedule = Programmer l’envoi
send-later-cancel = Annuler
send-later-err-invalid = Saisissez une date et une heure valides.
send-later-err-past = Choisissez un moment dans le futur.
send-later-err-horizon = C’est trop loin. Les messages peuvent être programmés jusqu’à { $days } jours à l’avance.
send-later-err-off = L’envoi programmé est désactivé par votre administrateur.
send-queued = Message en file d’attente — envoi dans
send-queued-undo = Annuler
send-queued-now = Envoyer maintenant
send-queued-recalled = Message rappelé. Il est de retour dans vos brouillons.
send-queued-too-late = Trop tard pour le rappeler : le message est déjà en cours d’envoi.
send-queued-sent-now = Envoi en cours.
send-queued-scheduled = Programmé pour le { $when } ({ $zone }).
outbox-scheduled-badge = Programmé
outbox-sends-in = Envoi dans { $time }
outbox-sends-at = Envoi le { $when }
outbox-recall-edit = Rappeler et modifier
outbox-recall-too-late = Déjà en cours d’envoi ; il ne peut plus être rappelé.
outbox-failed-drafts = N’a pas pu être envoyé et a été conservé dans vos brouillons.
outbox-queued-count = { $count ->
    [one] 1 message en attente d’envoi
   *[other] { $count } messages en attente d’envoi
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = Prochain envoi : { $when }

outbox-chip-title = Messages en attente d'envoi

outbox-chip-view-all = Ouvrir la boîte d'envoi

outbox-delete = Supprimer

outbox-delete-confirm = Supprimer le message ?

outbox-delete-failed = Le message n'a pas pu être supprimé. { $error }

outbox-deleted = Le message en file d'attente a été supprimé.

outbox-edit-time = Modifier l'heure d'envoi

outbox-edit-time-conflict = L'heure d'envoi a changé ou le message a déjà été envoyé. La boîte d'envoi a été rechargée.

outbox-edit-time-current = Actuellement { $when }

outbox-edit-time-refused = L'heure d'envoi n'a pas pu être modifiée. { $error }

outbox-edit-time-save = Gagnez du temps d'envoi

outbox-failed-toast = Un message n'a pas pu être envoyé. { $error }

outbox-failed-uncertain-toast = Un message peut avoir été livré ou non. Cochez Envoyé avant de le renvoyer.

outbox-open = Ouvert

outbox-reader-actions = Actions des messages en file d'attente

outbox-reader-cc = Cc : { $recipients }

outbox-reader-label = Message en file d'attente

outbox-reader-loading = Chargement du message…

outbox-reader-to = À : { $recipients }

outbox-reader-unavailable = Le message n'a pas pu être chargé.

outbox-recall-to-drafts = Rappel aux brouillons

outbox-send-again = Envoyer à nouveau

outbox-send-again-failed = Impossible de remettre le message en file d'attente. { $error }

outbox-send-again-queued = Le message est mis en file d'attente pour être renvoyé.

outbox-sent-copy-folder = Sera enregistré en { $folder }

outbox-sent-copy-missing = Sera enregistré dans Envoyés (le dossier choisi n'est pas disponible)

outbox-sent-copy-skip = Aucune copie ne sera enregistrée

outbox-status-attempts = { $count ->
        [0] { $count } tentatives pour l'instant
        [one] { $count } tentative jusqu'à présent
        [many] { $count } tentatives pour l'instant
       *[other] { $count } tentatives pour l'instant
    }

outbox-status-canceled = Annulé

outbox-status-failed = Non envoyé

outbox-status-held = Conservé jusqu'à ce que vous le relâchiez

outbox-status-last-error = Dernière erreur : { $error }

outbox-status-next-attempt = Prochaine tentative { $when }

outbox-status-queued = En file d'attente

outbox-status-retrying = Réessayer

outbox-status-scheduled = Programmé

outbox-status-sending = Envoi

outbox-status-sends-at-in = Envoie { $when } (en { $time })

outbox-status-sent = Envoyé

outbox-status-uncertain = Livraison inconnue
