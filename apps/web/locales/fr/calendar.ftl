# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Agenda
calendar-prev = Précédent
calendar-next = Suivant
calendar-today = Aujourd’hui
calendar-views = Vues de l’agenda
calendar-new-event = Nouvel événement
calendar-import = Importer
calendar-export = Exporter
calendar-import-file = Importer un fichier d’agenda

# -- View names --------------------------------------------------------------
calendar-view-day = Jour
calendar-view-3day = 3 jours
calendar-view-work-week = Semaine de travail
calendar-view-week = Semaine
calendar-view-month = Mois
calendar-view-tri-month = Trimestre
calendar-view-schedule = Planning
calendar-view-agenda = Liste
calendar-view-year = Année

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Agendas
calendar-toggle = Afficher ou masquer { $name }
calendar-color-for = Couleur de { $name }
calendar-add-calendar = Ajouter un agenda
calendar-new-calendar-name = Nouvel agenda
calendar-holidays-heading = Jours fériés
calendar-subscribe-holidays = S’abonner aux jours fériés
calendar-add-region = Ajouter une région…
calendar-synced = Agenda synchronisé

# -- Grid / views ------------------------------------------------------------
calendar-all-day = journée
calendar-all-day-full = Toute la journée
calendar-more = { $count } de plus
calendar-conflict = conflit
calendar-no-events-30 = Aucun événement dans les 30 prochains jours.
calendar-events-count = { $count ->
    [zero] Aucun événement
    [one] { $count } événement
   *[other] { $count } événements
}
calendar-month-grid = Mois de { $month }
calendar-time-grid = Grille horaire
calendar-cell = { $date }, { $events }
calendar-announce = { $date }, { $events }
calendar-event-at = { $time } { $title }
calendar-event-allday = Toute la journée : { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Nouvel événement
calendar-editor-edit = Modifier l’événement
calendar-invitation = Invitation
calendar-invited = Vous êtes invité ({ $status }).
calendar-accept = Accepter
calendar-tentative = Provisoire
calendar-decline = Refuser
calendar-counter = Contre-proposition…
calendar-propose-start = Proposer un nouveau début
calendar-send-counter = Envoyer la contre-proposition
calendar-overlaps = Cet événement chevauche { $count ->
    [one] { $count } autre événement
   *[other] { $count } autres événements
}.
calendar-field-title = Titre
calendar-untitled = (sans titre)
calendar-field-calendar = Agenda
calendar-field-all-day = Toute la journée
calendar-field-start = Début
calendar-field-duration = Durée (min)
calendar-field-location = Lieu
calendar-field-shows-as = Afficher comme
calendar-shows-busy = Occupé
calendar-shows-free = Libre
calendar-field-status = Statut
calendar-status-confirmed = Confirmé
calendar-status-tentative = Provisoire
calendar-status-cancelled = Annulé
calendar-repeats = Répétition
calendar-frequency = Fréquence
calendar-freq-daily = Quotidienne
calendar-freq-weekly = Hebdomadaire
calendar-freq-monthly = Mensuelle
calendar-freq-yearly = Annuelle
calendar-every = tous les
calendar-interval = Intervalle
calendar-ends = Fin
calendar-end-mode = Type de fin
calendar-end-never = Jamais
calendar-end-count = Après N fois
calendar-end-until = À une date
calendar-occurrences = Occurrences
calendar-until-date = Jusqu’au
calendar-reminders = Rappels
calendar-reminder-before = { $min } min avant
calendar-remove-reminder = Supprimer le rappel { $min }
calendar-add-reminder = Ajouter un rappel
calendar-reminder-none = + rappel
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 heure
calendar-reminder-1440 = 1 jour
calendar-attendees = Participants
calendar-add-attendee = Ajouter un participant
calendar-attendee-invalid = Ce n’est pas une adresse e-mail. Saisissez une adresse, comme name@example.com.
calendar-save-failed = L’événement n’a pas été enregistré. Le serveur l’a refusé.
calendar-remove-attendee = Retirer { $email }
calendar-notes = Notes

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Rôle de { $email }
calendar-attendee-cutype-for = Type de { $email }
calendar-role-chair = Organisateur
calendar-role-required = Obligatoire
calendar-role-optional = Facultatif
calendar-role-non-participant = Non participant
calendar-cutype-individual = Personne
calendar-cutype-group = Groupe
calendar-cutype-resource = Ressource
calendar-cutype-room = Salle
calendar-partstat-needs-action = Sans réponse
calendar-partstat-accepted = Accepté
calendar-partstat-declined = Refusé
calendar-partstat-tentative = Provisoire

# -- Schedule view (distinct from Agenda) ------------------------------------
calendar-schedule-row = { $start } à { $end } : { $title }
calendar-schedule-free = { $gap } libre
calendar-gap-hm = { $h } h { $m } min
calendar-gap-h = { $h } h
calendar-gap-m = { $m } min

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Résoudre { $count ->
    [one] { $count } conflit
   *[other] { $count } conflits
}
calendar-resolver-title = Résoudre les conflits
calendar-resolver-none = Aucun conflit à résoudre.
calendar-resolver-pick = Conflit
calendar-resolver-pair-n = { $n } sur { $total } : { $a } contre { $b }
calendar-resolver-time = Heure
calendar-resolver-overlap = Chevauchement { $start } – { $end }
calendar-resolver-reschedule = Décaler l’événement suivant
calendar-resolver-shorten = Raccourcir l’événement précédent
calendar-resolver-tentative = Marquer le suivant comme provisoire
calendar-resolver-double-book = Double réservation (marquer libre)
calendar-resolver-keep = Garder les deux
calendar-events-count-people = { $count ->
    [zero] Aucun participant
    [one] { $count } participant
   *[other] { $count } participants
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Agendas
calendar-fb-own = Les vôtres
calendar-fb-unavailable = Impossible de charger vos heures occupées.
calendar-fb-cell = { $principal } à { $hour } : { $status }
calendar-fb-busy = Occupé
calendar-fb-tentative = Provisoire
calendar-fb-free = Libre

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Ajout rapide d’événement
calendar-quick-add-placeholder = Ajout rapide, ex. Déjeuner vendredi 13h
calendar-quick-add-btn = Ajouter
calendar-quick-add-do = Ajout rapide

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Catégories
calendar-add-category = Ajouter une catégorie
calendar-category-placeholder = Ajouter une catégorie
calendar-remove-category = Supprimer la catégorie { $name }
calendar-filter-heading = Filtre
calendar-filter-category = Filtrer par catégorie
calendar-filter-category-placeholder = Catégorie à afficher

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Pièces jointes
calendar-attachment-title = Nom de la pièce jointe
calendar-attachment-title-placeholder = Nom (facultatif)
calendar-attachment-uri = Lien de la pièce jointe
calendar-attachment-add = Ajouter une pièce jointe
calendar-remove-attachment = Supprimer la pièce jointe { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = S’abonner par URL
calendar-subscribe-url = URL de l’agenda
calendar-subscribe-add = S’abonner
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = L’agenda à cette adresse n’a pas pu être récupéré ; il n’a donc pas été ajouté.
calendar-subscribe-hol-many-locations = Ce fichier de jours fériés contient plusieurs lieux. Téléchargez-le, puis utilisez Importer et choisissez les lieux voulus.
calendar-subscribe-refresh = Actualiser { $name }
calendar-subscribe-refresh-btn = Actualiser
calendar-subscribe-refresh-failed = Le flux de l’agenda n’a pas pu être récupéré. Ses événements sont restés inchangés.
calendar-import-failed = Ce fichier n’a pas pu être lu comme un agenda. Rien n’a été importé.
calendar-import-done = { $count ->
    [one] { $count } événement importé.
   *[other] { $count } événements importés.
}
calendar-import-locations-title = Choisir les lieux à importer
calendar-import-locations-intro = Ce fichier contient des jours fériés pour { $count } lieux. Seuls les lieux sélectionnés sont importés.
calendar-import-locations-filter = Filtrer les lieux
calendar-import-locations-list = Lieux du fichier
calendar-import-locations-none = Aucun lieu ne correspond au filtre.
calendar-import-locations-picked = { $count ->
    [one] { $count } lieu sélectionné.
   *[other] { $count } lieux sélectionnés.
}
calendar-import-locations-import = Importer la sélection
calendar-export-failed = L’agenda n’a pas pu être exporté.
calendar-quick-add-failed = Cet événement n’a pas pu être ajouté.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Partager
calendar-share-for = Partager { $name }
calendar-share-title = Partager { $name }
calendar-share-intro = Les personnes avec qui vous partagez peuvent voir cet agenda. Donnez un accès en lecture-écriture pour leur permettre d’ajouter et de modifier des événements.
calendar-share-people = Partagé avec
calendar-share-empty = Cet agenda n’est partagé avec personne.
calendar-share-add = Ajouter une personne
calendar-share-new-access = Accès de la nouvelle personne
calendar-share-access-for = Accès de { $principal }
calendar-share-remove = Retirer { $principal }
calendar-share-read = Lecture seule
calendar-share-readwrite = Lecture et écriture

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Navigation par date
calendar-event-range = { $title }, du { $start } au { $end }
calendar-busy-free = Apparaît comme disponible
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = { $count ->
    [one] { $count } autre événement
   *[other] { $count } autres événements
  } le { $date }
calendar-fb-caption-people = Plages occupées le jour du conflit, heure par heure, pour vous et chaque personne invitée.
calendar-fb-unknown = Les disponibilités de { $principal } n’ont pas pu être récupérées.
calendar-fb-loading = Chargement des disponibilités…
calendar-import-itip-message = Ce fichier est une réponse ou une annulation envoyée par e-mail. Ouvrez le message correspondant pour l’appliquer. Rien n’a été importé.
calendar-import-skipped = { $count ->
    [one] { $count } événement figurait déjà dans l’agenda et a été laissé tel quel.
   *[other] { $count } événements figuraient déjà dans l’agenda et ont été laissés tels quels.
    }
calendar-invite-title = Invitation d’agenda
calendar-invite-sender = De { $sender }
calendar-invite-add = Ajouter à l’agenda
calendar-invite-update = Appliquer la mise à jour de l’événement
calendar-invite-cancel = Appliquer l’annulation
calendar-invite-reply = Enregistrer la réponse du participant
calendar-invite-new = Cette invitation peut être ajoutée à votre agenda.
calendar-invite-newer = L’organisateur a envoyé un événement mis à jour.
calendar-invite-current = Votre agenda contient déjà cette version.
calendar-invite-stale = Ce message est plus ancien que l’événement de votre agenda.
calendar-invite-cancelled = L’organisateur a annulé cet événement.
calendar-invite-response = Un participant a envoyé une réponse.
calendar-invite-not-organizer = L’expéditeur n’est pas l’organisateur enregistré. Rien ne sera modifié.
calendar-invite-not-attendee = L’expéditeur n’est pas un participant correspondant. Rien ne sera modifié.
calendar-invite-unknown = Ce message fait référence à un événement absent de votre agenda.
calendar-invite-unsupported = Cette proposition ne peut pas être appliquée automatiquement.
calendar-invite-no-reply = Cela met à jour votre agenda sans envoyer de réponse.
calendar-invite-applied = Votre agenda a été mis à jour.
calendar-invite-unchanged = L’événement n’a pas été modifié. Vérifiez l’état actuel de l’invitation.
calendar-invite-apply-failed = L’invitation n’a pas pu être appliquée. Réessayez.
calendar-invite-load-failed = L’invitation n’a pas pu être vérifiée.
calendar-invite-retry = Réessayer
calendar-field-time-zone = Fuseau horaire de l’événement
calendar-field-time-zone-help = Utilisez un nom IANA tel que Europe/Lisbon. L’événement conserve ce fuseau lorsqu’il est consulté ailleurs.
calendar-time-invalid = Saisissez un fuseau horaire et une heure locale valides. Certaines heures locales n’existent pas lors du passage à l’heure d’été.
calendar-mini-prev = Mois précédent
calendar-mini-next = Mois suivant
calendar-card-region-event = Événement d’agenda
calendar-card-kind-request = Invitation à une réunion
calendar-card-kind-update = Invitation mise à jour
calendar-card-kind-cancel = Réunion annulée
calendar-card-kind-reply = Réponse à la réunion
calendar-card-kind-counter = Nouvel horaire proposé
calendar-card-kind-publish = { $count ->
    [one] Événement d’agenda
   *[other] { $count } événements d’agenda
    }
calendar-card-when = Quand
calendar-card-where = Où
calendar-card-organizer = Organisateur
calendar-card-attendees = Participants
calendar-card-all-day = { $when } (journée entière)
calendar-card-organizer-time = Heure de l’organisateur : { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Tous les jours
   *[other] Tous les { $interval } jours
    }
calendar-card-repeat-weekly = { $interval ->
    [one] Toutes les semaines
   *[other] Toutes les { $interval } semaines
    }
calendar-card-repeat-weekly-on = { $interval ->
    [one] Toutes les semaines : { $days }
   *[other] Toutes les { $interval } semaines : { $days }
    }
calendar-card-repeat-monthly = { $interval ->
    [one] Tous les mois
   *[other] Tous les { $interval } mois
    }
calendar-card-repeat-monthly-day = { $interval ->
    [one] Tous les mois, le { $day }
   *[other] Tous les { $interval } mois, le { $day }
    }
calendar-card-repeat-yearly = { $interval ->
    [one] Tous les ans
   *[other] Tous les { $interval } ans
    }
calendar-card-repeat-until = { $rule } jusqu’au { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] une fois
   *[other] { $count } fois
    }
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 date ignorée
   *[other] { $count } dates ignorées
    })
calendar-card-repeat-custom = Se répète selon un calendrier personnalisé
calendar-card-join = Rejoindre { $provider }
calendar-card-organizer-is = Organisé par { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 participant
   *[other] { $count } participants
    }
calendar-card-people-all = Afficher les { $count }
calendar-card-people-fewer = Afficher moins
calendar-card-rsvp-accepted = Accepté
calendar-card-rsvp-tentative = Provisoire
calendar-card-rsvp-declined = Refusé
calendar-card-rsvp-none = Pas encore de réponse
calendar-card-in-calendar = Dans votre agenda
calendar-card-cancelled = Annulé
calendar-card-open = Ouvrir dans l’agenda
calendar-card-agenda = Votre agenda le { $day }
calendar-card-agenda-title = Votre journée
calendar-card-agenda-this-event = Cet événement
calendar-card-agenda-unavailable = Votre agenda pour ce jour n’a pas pu être chargé.
calendar-card-conflicts = { $count ->
    [one] En conflit avec { $titles }
   *[other] En conflit avec { $count } événements : { $titles }
    }
calendar-card-no-conflicts = Aucun conflit dans votre agenda
calendar-card-invited-as = Vous avez été invité avec l’adresse { $address }.
calendar-card-not-invited = Cette invitation ne mentionne aucune de vos adresses. Vous pouvez l’ajouter à votre agenda, mais vous ne pouvez pas y répondre.
calendar-card-not-invited-short = Cette invitation ne mentionne aucune de vos adresses.
calendar-card-out-of-date = Cette invitation est obsolète. Une version plus récente figure déjà dans votre agenda.
calendar-card-replied = { $who } { $answer ->
    [accepted] a accepté.
    [tentative] a accepté provisoirement.
    [declined] a refusé.
   *[other] n’a pas encore répondu.
    }
calendar-card-counter = { $who } a proposé un nouvel horaire. Les propositions ne sont pas appliquées automatiquement.
calendar-card-todos-skipped = { $count ->
    [one] Le fichier contient aussi 1 tâche, qui n’est pas ajoutée.
   *[other] Le fichier contient aussi { $count } tâches, qui ne sont pas ajoutées.
    }
calendar-card-respond = Répondre à l’invitation
calendar-card-accept = Accepter
calendar-card-tentative = Provisoire
calendar-card-decline = Refuser
calendar-card-propose = Proposer un autre horaire
calendar-card-propose-start = Nouveau début
calendar-card-propose-send = Envoyer la proposition
calendar-card-propose-invalid = Saisissez une heure de début valide.
calendar-card-send-response = Envoyer une réponse à { $organizer }
calendar-card-comment = Commentaire (facultatif)
calendar-card-calendar = Agenda
calendar-card-events = Événements de ce fichier
calendar-card-add-one = Ajouter
calendar-card-add-all = Tout ajouter ({ $count })
calendar-card-added = { $count ->
    [one] Ajouté à votre agenda.
   *[other] { $count } événements ajoutés à votre agenda.
    }
calendar-card-remove = Retirer de l’agenda
calendar-card-removed = Retiré de votre agenda.
calendar-card-record-reply = Mettre à jour le statut du participant
calendar-card-answered-sent = { $answer }. Votre réponse a été envoyée à { $organizer }.
calendar-card-answered-not-sent = { $answer }. Votre agenda a été mis à jour, mais la réponse n’a pas pu être envoyée.
calendar-card-answered-silently = { $answer }. Aucune réponse n’a été envoyée.
calendar-card-proposal-sent = Votre proposition a été envoyée à { $organizer }.
calendar-card-not-sent = Le message n’a pas pu être envoyé.
calendar-card-more = { $count ->
    [one] 1 autre
   *[other] { $count } autres
    }
