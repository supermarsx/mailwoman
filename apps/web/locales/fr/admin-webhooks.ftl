# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooks
admin-webhooks-intro = Un webhook sortant envoie un avis signé à une URL lorsqu’un événement se produit dans un compte. Un webhook entrant permet à un autre système d’exécuter une action de règle de messagerie dans un compte.
admin-webhooks-empty = Aucun webhook pour l’instant.
admin-webhooks-unavailable = Ce serveur ne propose pas la gestion des webhooks.
admin-webhooks-load-error = Impossible de charger les webhooks.
admin-webhooks-direction-outbound = Sortant
admin-webhooks-direction-inbound = Entrant
admin-webhooks-col-direction = Sens
admin-webhooks-col-label = Nom
admin-webhooks-col-account = Compte
admin-webhooks-col-target = Envoie vers ou autorise
admin-webhooks-col-created = Créé
admin-webhooks-no-label = (sans nom)
admin-webhooks-created-by = par { $who }
admin-webhooks-last-used = Dernière utilisation { $when }
admin-webhooks-never-used = Pas encore utilisé
admin-webhooks-delete = Supprimer
admin-webhooks-delete-named = Supprimer le webhook { $name }
admin-webhooks-delete-title = Supprimer le webhook { $name } ?
admin-webhooks-delete-body = Les envois vers celui-ci s’arrêtent, ou les appels signés avec son secret sont refusés. Cette action est irréversible ; un nouveau webhook reçoit un nouveau secret.
admin-webhooks-deleted = Webhook supprimé.
admin-webhooks-add = Ajouter un webhook
admin-webhooks-add-direction = Sens
admin-webhooks-add-outbound-help = Mailwoman appelle votre URL.
admin-webhooks-add-inbound-help = Votre système appelle Mailwoman.
admin-webhooks-add-account = Compte
admin-webhooks-add-account-help = Le compte dont les événements sont envoyés, ou le seul compte sur lequel un appel entrant peut agir.
admin-webhooks-add-label = Nom
admin-webhooks-add-label-help = À quoi sert le webhook, afin que les autres le sachent. Facultatif.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Doit pointer vers une adresse Internet publique. Les adresses privées, de bouclage et de lien local sont refusées.
admin-webhooks-add-events = Événements
admin-webhooks-add-actions = Actions autorisées
admin-webhooks-add-submit = Créer le webhook
admin-webhooks-add-saving = Création…
admin-webhooks-add-check = Vérifier
admin-webhooks-add-check-ok = L’URL réussit la vérification. Rien n’a été enregistré.
admin-webhooks-event-message-received = Nouveau message
admin-webhooks-event-message-flags = Message lu, marqué ou étiqueté
admin-webhooks-event-message-moved = Message déplacé
admin-webhooks-event-message-deleted = Message supprimé
admin-webhooks-event-submission-sent = Message envoyé
admin-webhooks-event-submission-failed = Le message n’a pas pu être envoyé
admin-webhooks-event-pim-changed = Agenda, contacts, tâches ou notes modifiés
admin-webhooks-action-run-rules = Exécuter les règles de messagerie du compte sur la description d’un message
admin-webhooks-action-sieve = Appliquer une action de règle à un message
admin-webhooks-secret-title = Secret de { $name }
admin-webhooks-secret-once = Copiez ce secret maintenant. Il ne sera plus affiché.
admin-webhooks-secret-outbound = Chaque envoi comporte X-Mailwoman-Signature: sha256= suivi du HMAC-SHA256 du corps de la requête calculé avec ce secret.
admin-webhooks-secret-inbound = Envoyez X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (secondes Unix), X-Mailwoman-Nonce (16 à 128 lettres, chiffres, - ou _, jamais réutilisé) et X-Mailwoman-Signature: sha256= suivi du HMAC-SHA256 de timestamp.nonce.body calculé avec ce secret. Les appels décalés de plus de cinq minutes, ou réutilisant un nonce, sont refusés.
admin-webhooks-secret-done = Je l’ai copié
admin-webhooks-error-stale = Quelqu’un a modifié les webhooks depuis l’ouverture de cette page. La liste a été rechargée ; réessayez.
admin-webhooks-error-account = Saisissez le compte.
admin-webhooks-error-url = Saisissez une URL commençant par https:// ou http://.
admin-webhooks-error-events = Choisissez au moins un événement.
admin-webhooks-error-actions = Choisissez au moins une action.
