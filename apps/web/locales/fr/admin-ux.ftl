# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Fermer

# -- Page states (UX11) ---------------------------------------------------------
admin-ux-state-unavailable = Non disponible — { $reason }
admin-ux-unavailable-default = ce serveur ne le propose pas encore.
admin-ux-state-code = Code d’erreur : { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Ouvrir
admin-ux-list-back = Retour à la liste

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Aucune modification non enregistrée
admin-ux-form-changes =
    { $count ->
        [one] { $count } modification non enregistrée
        [many] { $count } de modifications non enregistrées
       *[other] { $count } modifications non enregistrées
    }
admin-ux-form-discard = Abandonner les modifications
admin-ux-form-saving = Enregistrement…
admin-ux-form-saved =
    { $count ->
        [one] { $count } modification enregistrée.
        [many] { $count } de modifications enregistrées.
       *[other] { $count } modifications enregistrées.
    }
admin-ux-form-fix =
    { $count ->
        [one] Rien n’a été enregistré. Corrigez le champ signalé ci-dessous.
        [many] Rien n’a été enregistré. Corrigez les { $count } de champs signalés ci-dessous.
       *[other] Rien n’a été enregistré. Corrigez les { $count } champs signalés ci-dessous.
    }
admin-ux-learn-more = En savoir plus
admin-ux-learn-more-field = sur { $label }
admin-ux-propagation-hot = S’applique en moins de 30 secondes sur chaque processus serveur.
admin-ux-propagation-restart = S’applique après le redémarrage du serveur.
admin-ux-secret-set = Une valeur est enregistrée. Saisissez-en une nouvelle pour la remplacer.
admin-ux-secret-unset = Aucune valeur n’est enregistrée.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Saisissez une valeur.
admin-ux-err-number = Saisissez un nombre.
admin-ux-err-whole = Saisissez un nombre entier.
admin-ux-err-min = Saisissez { $min } ou plus.
admin-ux-err-max = Saisissez { $max } ou moins.
admin-ux-err-max-length = Utilisez au plus { $max } caractères.
admin-ux-err-url = Saisissez une adresse complète commençant par https:// ou http://.
admin-ux-err-email = Saisissez une adresse e-mail comme name@example.org.
admin-ux-err-pattern = Cette valeur n’a pas la forme attendue.
admin-ux-err-option = Choisissez l’une des options proposées.
admin-ux-err-out-of-range = Le serveur a refusé cette valeur, car elle est en dehors de la plage autorisée.
admin-ux-err-too-long = Le serveur a refusé cette valeur, car elle est trop longue.
admin-ux-err-invalid = Le serveur a refusé cette valeur. Vérifiez-la et enregistrez à nouveau.
admin-ux-err-pinned = Cette valeur est définie dans l’environnement du serveur et ne peut pas être modifiée ici.
admin-ux-err-locked = Cette valeur est verrouillée à une portée plus large et ne peut pas être modifiée ici.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Par défaut
admin-ux-source-stored = Enregistré
admin-ux-source-env-pin = Défini par { $name }
admin-ux-source-env-fallback = Repli depuis { $name }
admin-ux-source-build = Défini à la compilation
admin-ux-source-locked = Verrouillé par { $scope }
admin-ux-source-read-only = Lecture seule
admin-ux-source-restart = Redémarrage requis
admin-ux-source-env-pin-reason = L’environnement du serveur définit cette valeur avec { $name }. Modifiez-la à cet endroit et redémarrez le serveur.
admin-ux-source-build-reason = Cette valeur est fixée lors de la compilation du serveur.
admin-ux-source-locked-reason = Un verrou à la portée { $scope } fixe cette valeur. Retirez le verrou à cet endroit pour la modifier.
admin-ux-source-read-only-reason = La console affiche cette valeur mais ne peut pas la modifier. La documentation indique où elle est définie.

# -- History (UX8) ------------------------------------------------------------
admin-ux-history-changed-by-at = Dernière modification par { $who } le { $time }
admin-ux-history-changed-by = Dernière modification par { $who }
admin-ux-history-changed-at = Dernière modification le { $time }
admin-ux-history-never = Non modifié depuis la console
admin-ux-history-link = Historique

# -- Revert to default (UX7) --------------------------------------------------
admin-ux-revert-default-is = Par défaut : { $value }
admin-ux-revert = Rétablir la valeur par défaut
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Valeur par défaut rétablie ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } a modifié ceci le { $time } pendant que vous le modifiiez.
admin-ux-stale-by = { $who } a modifié ceci pendant que vous le modifiiez.
admin-ux-stale-unknown = Quelqu’un d’autre a modifié ceci pendant que vous le modifiiez.
admin-ux-stale-choice = Rechargez pour voir sa version et perdre vos modifications, ou conservez les vôtres et enregistrez-les par-dessus.
admin-ux-stale-reload = Recharger
admin-ux-stale-keep = Garder les miennes

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Vérification de ce que cette modification affecte…
admin-ux-impact-heading = Cette modification affecte
admin-ux-impact-count = { $label } : { $count }
admin-ux-impact-more =
    { $count ->
        [one] et { $count } autre
        [many] et { $count } d’autres
       *[other] et { $count } autres
    }
admin-ux-impact-none = Rien d’autre ne change.
admin-ux-impact-unavailable = Aperçu de l’impact non disponible — le serveur n’a pas indiqué ce que cette modification affecte.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filtrer
admin-ux-stream-pause = Pause
admin-ux-stream-resume = Reprendre
admin-ux-stream-clear = Effacer
admin-ux-stream-announce = Lire les nouvelles entrées à voix haute
admin-ux-stream-connecting = Connexion…
admin-ux-stream-live = En direct
admin-ux-stream-ended = Terminé
admin-ux-stream-lost = Connexion perdue
admin-ux-stream-reconnect = Se reconnecter
admin-ux-stream-paused =
    { $count ->
        [0] En pause
        [one] En pause — { $count } nouvelle entrée retenue
        [many] En pause — { $count } de nouvelles entrées retenues
       *[other] En pause — { $count } nouvelles entrées retenues
    }
admin-ux-stream-no-match = Aucune entrée ne correspond au filtre.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Champ

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Apparence
admin-ux-appearance-title = Apparence de la console
admin-ux-appearance-note = Ces choix ne modifient que la console d’administration, dans ce navigateur.
admin-ux-appearance-following = La console suit le thème du webmail de ce navigateur tant que vous ne faites pas de choix ici. Ces choix ne modifient que la console d’administration, dans ce navigateur.
admin-ux-appearance-mode = Mode
admin-ux-appearance-mode-light = Clair
admin-ux-appearance-mode-dark = Sombre
admin-ux-appearance-mode-system = Suivre le système
admin-ux-appearance-theme = Thème
admin-ux-appearance-accent = Couleur d’accentuation
admin-ux-appearance-light = Clair
admin-ux-appearance-dark = Sombre
admin-ux-appearance-reset = Utiliser le thème du webmail
