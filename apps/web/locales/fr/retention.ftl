# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Conservation
retention-intro = Des règles qui déplacent ou suppriment vos propres e-mails après un certain nombre de jours. Une règle peut tenir compte d’un dossier, d’un label, d’un expéditeur, d’une taille, de l’état de lecture ou de la présence d’une pièce jointe. Les e-mails marqués d’un drapeau et ceux portant le label keep sont épargnés, sauf si vous modifiez ce comportement. Les e-mails supprimés vont d’abord dans la corbeille, puis sont purgés à l’issue du délai d’annulation.
retention-disabled = Les règles de conservation sont désactivées sur ce serveur. Vos règles existantes ne s’exécutent pas.
retention-never = jamais
retention-error = Les paramètres de conservation n’ont pas pu être chargés ou enregistrés. Réessayez.
retention-last-run = Dernière exécution : { $when }
retention-next-run = prochaine exécution dans environ { $days } jour(s)
retention-next-run-unknown = prochaine exécution pas encore planifiée
retention-enabled = Exécuter mes règles de conservation
retention-trash-first = Déplacer d’abord les e-mails supprimés dans la corbeille
retention-trash-forced = Ce serveur déplace toujours les e-mails d’abord dans la corbeille et les purge à l’issue du délai d’annulation.
retention-undo = Délai d’annulation en jours
retention-undo-help = Les e-mails qu’une règle a déplacés dans la corbeille y restent pendant ce délai avant d’être purgés. De 0 à { $max } jours.
retention-pause = Suspendre toutes les règles pendant 30 jours
retention-paused-until = Suspendues jusqu’au { $when }
retention-resume = Reprendre
retention-policy = Ce que ce serveur autorise
retention-limits-both = Une règle qui supprime des e-mails attend au moins { $min } jours et au plus { $max } jours.
retention-limits-min = Une règle qui supprime des e-mails attend au moins { $min } jours.
retention-max-rules = Vous pouvez conserver jusqu’à { $count } règles.
retention-permanent-allowed = Une règle peut supprimer définitivement des e-mails si vous désactivez l’étape de la corbeille.
retention-permanent-denied = Les règles ne peuvent pas supprimer définitivement des e-mails. Les e-mails vont dans la corbeille et sont purgés après { $days } jours.
retention-protected = Dossiers que les règles ne touchent jamais : { $folders }.
retention-mandatory = Défini par l’administrateur : les e-mails de { $folder } datant de plus de { $days } jours sont supprimés.
retention-rules = Vos règles
retention-add = Nouvelle règle
retention-no-rules = Vous n’avez encore aucune règle. Adoptez un modèle ci-dessous ou ajoutez-en une.
retention-rule-paused = suspendue
retention-rule-on = Activée
retention-rule-enabled = La règle { $name } est activée
retention-edit-short = Modifier
retention-delete = Supprimer
retention-new = Nouvelle règle
retention-edit = Modifier la règle
retention-name = Nom
retention-action = Action à effectuer
retention-action-keep = Conserver indéfiniment
retention-action-delete = Supprimer
retention-action-trash = Déplacer dans la corbeille
retention-action-archive = Déplacer dans les archives
retention-action-move = Déplacer vers un dossier
retention-action-markRead = Marquer comme lu
retention-days = Après un délai de (jours)
retention-target = Dossier
retention-choose-folder = Choisir un dossier
retention-scope = Quels e-mails
retention-folder = Dossier
retention-any-folder = N’importe quel dossier
retention-subfolders = Inclure les sous-dossiers
retention-labels = Labels (séparés par des virgules)
retention-senders = Adresses d’expéditeur (séparées par des virgules)
retention-domains = Domaines d’expéditeur (séparés par des virgules)
retention-min-size = Taille minimale en Ko
retention-max-size = Taille maximale en Ko
retention-read = État de lecture
retention-read-any = Lu ou non lu
retention-read-read = Lus uniquement
retention-read-unread = Non lus uniquement
retention-attachment = Pièces jointes
retention-attachment-any = Avec ou sans
retention-attachment-with = Avec pièce jointe
retention-attachment-without = Sans pièce jointe
retention-exclude = Épargner
retention-exclude-flagged = E-mails marqués d’un drapeau (favoris)
retention-exclude-unread = E-mails non lus
retention-exclude-tags = E-mails portant ces labels (séparés par des virgules)
retention-exclude-note = Les e-mails soumis à une conservation légale ne sont jamais touchés, quel que soit ce réglage.
retention-role-inbox = Boîte de réception
retention-role-sent = Envoyés
retention-role-archive = Archives
retention-role-junk = Courrier indésirable
retention-role-trash = Corbeille
retention-role-drafts = Brouillons
retention-btn-preview = Aperçu
retention-save = Enregistrer
retention-cancel = Annuler
retention-preview-none = Aucun message ne serait modifié.
retention-preview-count = Cela modifierait { $count } message(s).
retention-preview-truncated = Seule une partie de la boîte aux lettres a été lue pour cet aperçu.
retention-preview-sample = Exemples
retention-no-subject = (sans objet)
retention-do-trash = vers la corbeille
retention-do-move = déplacés
retention-do-delete = supprimés
retention-do-markRead = marqués comme lus
retention-skipped-legalHold = Votre compte est soumis à une conservation légale. Rien n’est modifié.
retention-skipped-restricted = Le traitement de votre compte est limité. Rien n’est modifié.
retention-skipped-noRules = Vous n’avez aucune règle active et l’administrateur n’en a défini aucune.
retention-error-name = Donnez un nom à la règle.
retention-error-days = Le nombre de jours doit être un entier compris entre 0 et 3650.
retention-error-min = Une règle qui supprime des e-mails doit attendre au moins { $days } jours.
retention-error-max = Une règle qui supprime des e-mails peut attendre au plus { $days } jours.
retention-error-target = Choisissez le dossier vers lequel déplacer les e-mails.
retention-summary-keep = conserver indéfiniment
retention-summary-delete = après { $days } jours, vers la corbeille, puis purgés
retention-summary-delete-permanent = après { $days } jours, supprimés définitivement
retention-summary-trash = après { $days } jours, vers la corbeille
retention-summary-archive = après { $days } jours, vers les archives
retention-summary-move = après { $days } jours, vers un dossier
retention-summary-markRead = après { $days } jours, marqués comme lus
retention-templates = Modèles
retention-templates-help = Un modèle ajoute une règle toute prête. Vérifiez ses labels et ses jours avant de vous y fier.
retention-adopt = Ajouter cette règle
retention-run-heading = Exécuter maintenant
retention-preview-all = Aperçu de toutes les règles
retention-btn-run = Exécuter mes règles maintenant
retention-run-result = { $count } message(s) modifié(s).
retention-history = Exécutions récentes
retention-history-when = Quand
retention-history-matched = Correspondances
retention-history-changed = Modifiés
retention-history-errors = Erreurs
retention-context-action = Conservation…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Calendriers

pim-cal-days = Jours après la fin d'un événement

pim-cal-enabled = Supprimer les événements terminés

pim-cal-help = Supprimez les événements des calendriers que vous choisissez un nombre défini de jours après leur fin. Les événements récurrents ne sont jamais supprimés.

pim-cal-title = Événements du calendrier

pim-days-range = Choisissez entre { $min } et { $max } jours.

pim-held-title = Notes supprimées que vous pouvez restaurer

pim-held-until = Conservé jusqu'à { $when }

pim-limits = Entre { $min } et { $max } jours.

pim-notes-days = Jours depuis la dernière modification d'une note

pim-notes-enabled = Supprimer les notes intactes

pim-notes-help = Supprimez les notes que vous n'avez pas modifiées depuis un nombre de jours défini. Les notes épinglées sont conservées. Une note supprimée peut être restaurée pendant { $days } jours.

pim-notes-title = Remarques

pim-off = Désactivé par votre administrateur.

pim-preview = Aperçu

pim-restore = Restaurer

pim-result = { $events } événement(s) et { $notes } note(s).

pim-run = Courez maintenant

pim-run-title = Courez maintenant

pim-save = Enregistrer

pim-skipped = { $count } choix(s) ont été ignorés car votre administrateur les a désactivés.

pim-untitled = Note sans titre

retention-action-stripAttachments = Supprimer les pièces jointes volumineuses

retention-do-stripAttachments = pièces jointes supprimées

retention-error-strip-disabled = La suppression des pièces jointes est désactivée par votre administrateur.

retention-error-strip-min = Une règle qui supprime les pièces jointes doit attendre au moins { $days } jours.

retention-fd-action = Qu'arrive-t-il au courrier en { $folder }

retention-fd-days = Jours avant le traitement de { $folder }

retention-fd-help = Définissez la durée pendant laquelle le courrier reste dans un dossier avant qu'il ne soit déplacé ou supprimé. Un dossier défini sur « Hériter » n'a pas de valeur par défaut propre ; les règles ci-dessus et les limites de l'administrateur s'appliquent toujours. Les sous-dossiers suivent leur dossier.

retention-fd-inherit = Hériter

retention-fd-mandatory = L'administrateur conserve ce dossier pendant { $days } jours

retention-fd-protected = Protégé par l'administrateur

retention-fd-save = Enregistrer les valeurs par défaut du dossier

retention-fd-title = Valeurs par défaut du dossier

retention-rule-strip-off = Désactivé par votre administrateur

retention-strip-allowed = Une règle peut supprimer les pièces jointes d'au moins { $kb } Ko d'un courrier datant d'au moins { $days } jours. Une courte note remplace chaque pièce jointe.

retention-strip-denied = La suppression des pièces jointes est désactivée par votre administrateur.

retention-strip-done = Pièces jointes supprimées du(des) message(s) { $count }, libérant { $size }.

retention-strip-inline-kept = Les images présentées à l'intérieur du message sont conservées.

retention-strip-inline-removed = Les images affichées à l’intérieur du message sont également supprimées.

retention-strip-limits = Supprime les pièces jointes d'au moins { $kb } Ko du courrier datant d'au moins { $days } jours. L'original reste dans la corbeille pendant { $undo } jours.

retention-strip-preview = Les messages { $count } perdraient leurs pièces jointes, libérant ainsi { $size }.

retention-strip-preview-estimate = Environ { $count } message(s) perdrait les pièces jointes, libérant jusqu'à { $size }.

retention-strip-skipped = { $count } règle(s) qui suppriment les pièces jointes ont été ignorées car votre administrateur a désactivé cette fonction.

retention-summary-stripAttachments = après { $days } jours, pièces jointes volumineuses supprimées
