# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Changer le mot de passe
passwd-heading = Changer le mot de passe

# -- Forced change -----------------------------------------------------------
passwd-force-change = Votre administrateur exige que vous changiez votre mot de passe avant de continuer.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Mot de passe actuel
passwd-new-label = Nouveau mot de passe
passwd-confirm-label = Confirmer le nouveau mot de passe

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Les nouveaux mots de passe correspondent.
passwd-match-no = Les nouveaux mots de passe ne correspondent pas encore.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = { $count ->
    [one] au moins { $count } caractère
   *[other] au moins { $count } caractères
}
passwd-rule-uppercase = une lettre majuscule
passwd-rule-lowercase = une lettre minuscule
passwd-rule-digit = un chiffre
passwd-rule-symbol = un symbole

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Ce compte est à accès zéro. Avant que la modification ne soit appliquée, une phrase de récupération vous sera présentée — conservez-la afin de pouvoir encore accéder à vos données en cas de problème.

# -- Actions -----------------------------------------------------------------
passwd-continue = Continuer
passwd-submit = Changer le mot de passe

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Conservez votre phrase de récupération
passwd-recovery-prose = Notez cette phrase et gardez-la en lieu sûr. Elle est affichée avant le changement de mot de passe afin que vous puissiez récupérer vos données même si le nouveau mot de passe est perdu. Elle n’est pas stockée sur le serveur.
passwd-recovery-ack-label = J’ai conservé ma phrase de récupération
passwd-recovery-ack-text = J’ai conservé ma phrase de récupération en lieu sûr.

# -- Done phase --------------------------------------------------------------
passwd-done = Votre mot de passe a été changé.
passwd-done-reencrypt = Vos identifiants de serveur stockés ont été rechiffrés avec le nouveau mot de passe.
passwd-done-rewrap = Vos clés d’accès zéro ont été réencapsulées avec le nouveau mot de passe.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = saisissez votre mot de passe actuel
passwd-error-mismatch = le nouveau mot de passe et sa confirmation ne correspondent pas
passwd-error-policy = le nouveau mot de passe doit contenir { $rules }
passwd-error-ack-first = confirmez d’abord que vous avez conservé la phrase de récupération
passwd-error-prepare-recovery = impossible de préparer la phrase de récupération
passwd-error-change = impossible de changer le mot de passe
passwd-error-session-ended = Votre session a pris fin. Reconnectez-vous.
passwd-error-not-configured = Ce serveur n’est pas configuré pour changer les mots de passe. Contactez votre administrateur.
passwd-error-contact-admin = Le serveur n’a pas pu effectuer la modification. Contactez votre administrateur.

# -- Forced change screen ----------------------------------------------------
passwd-forced-title = Changez votre mot de passe pour continuer
passwd-forced-signed-in-as = Connecté en tant que { $username }.
passwd-forced-explain = Un administrateur a exigé un changement de mot de passe pour ce compte. Le courrier, l’agenda et les contacts ne sont pas disponibles tant que le changement n’a pas réussi.
passwd-forced-backend-note = Si le changement est refusé alors que votre mot de passe actuel est correct, ce serveur n’est peut-être pas en mesure de changer votre mot de passe. Contactez votre administrateur : il peut configurer le changement de mot de passe ou supprimer cette exigence.
passwd-forced-sign-out = Se déconnecter
passwd-forced-still-required = Le serveur a accepté le changement mais exige toujours un changement de mot de passe pour ce compte. Contactez votre administrateur.
passwd-forced-reload-failed = Le mot de passe a été changé, mais votre compte n’a pas pu être chargé. Vérifiez votre connexion et réessayez.
passwd-forced-retry = Réessayer
