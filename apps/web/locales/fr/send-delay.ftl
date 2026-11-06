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
