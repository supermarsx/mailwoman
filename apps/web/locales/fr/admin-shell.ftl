# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Tableau de bord
admin-shell-group-overview = Vue d’ensemble
admin-shell-group-people = Personnes
admin-shell-group-security = Sécurité
admin-shell-group-configuration = Configuration
admin-shell-group-operations = Exploitation
admin-shell-menu = Menu
admin-shell-close-menu = Fermer le menu
admin-shell-search = Rechercher
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Connecté en tant que { $name }
admin-shell-language-label = Langue
admin-shell-breadcrumb = Fil d’Ariane
admin-shell-unknown-section = Cette section n’existe pas, ou votre rôle ne l’inclut pas.
admin-shell-back-to-dashboard = Retour au tableau de bord
admin-shell-cap-missing = Votre rôle n’inclut pas { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Chargement…
admin-shell-retry = Réessayer
admin-shell-cancel = Annuler
admin-shell-save = Enregistrer
admin-shell-not-available = Non disponible

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Quitter cette page ?
admin-shell-leave-body = Vos modifications sur cette page n’ont pas été enregistrées. Quitter la page les abandonne.
admin-shell-leave-confirm = Quitter et abandonner les modifications

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Saisissez { $name } pour confirmer

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Rechercher dans la console d’administration
admin-shell-palette-label = Section, utilisateur, domaine ou paramètre
admin-shell-palette-results = Résultats
admin-shell-palette-none = Aucun résultat.
admin-shell-palette-searching = Recherche des utilisateurs, domaines et paramètres…
admin-shell-palette-sections-only = Ce serveur ne permet pas encore de rechercher les utilisateurs, domaines et paramètres : seules les sections sont listées.
admin-shell-palette-error = La recherche des utilisateurs, domaines et paramètres a échoué ; seules les sections sont listées.
admin-shell-palette-kind-section = Section
admin-shell-palette-kind-user = Utilisateur
admin-shell-palette-kind-domain = Domaine
admin-shell-palette-kind-setting = Paramètre

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filtres
admin-shell-table-actions = Actions
admin-shell-table-error = La liste n’a pas pu être chargée.
admin-shell-table-paging = Pages de { $table }
admin-shell-table-prev = Page précédente
admin-shell-table-next = Page suivante

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time } : { $value }
admin-shell-chart-summary = Dernière valeur { $latest }, maximum { $max }
admin-shell-chart-col-time = Heure
admin-shell-chart-col-item = Élément
admin-shell-chart-col-value = Valeur

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Défini par
admin-shell-pinned-sr = Cette valeur provient de l’environnement du serveur et ne peut pas être modifiée ici.
admin-shell-lock-text = Verrouiller pour les utilisateurs
admin-shell-lock-setting = { $setting }
admin-shell-restore = Restaurer la valeur précédente
admin-shell-restore-note = Modifié par { $actor } le { $time }. Valeur précédente : { $value }
admin-shell-restore-done = La valeur précédente est restaurée.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Étapes
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Étape { $n } sur { $total } : { $title }
admin-shell-wizard-review = Vérification
admin-shell-wizard-back = Retour
admin-shell-wizard-next = Suivant
admin-shell-wizard-saved = Enregistré.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = Impossible de joindre le serveur. Vérifiez la connexion et réessayez.
admin-shell-err-session = Votre session d’administration est terminée. Reconnectez-vous.
admin-shell-err-forbidden = Votre rôle ne permet pas cette action. Demandez à un administrateur disposant du rôle adéquat.
admin-shell-err-out-of-scope = Cet objet appartient à un domaine que votre rôle ne couvre pas.
admin-shell-err-not-found = Cet élément n’existe plus. Rechargez la liste.
admin-shell-err-conflict = Quelqu’un l’a modifié entre-temps. Rechargez et réessayez.
admin-shell-err-validation = Le serveur a refusé une valeur. Vérifiez les champs et réessayez.
admin-shell-err-rate-limited = Trop de requêtes. Attendez une minute et réessayez.
admin-shell-err-server = Le serveur a répondu par une erreur ({ $status }). Réessayez ; si cela se reproduit, consultez le journal du serveur.
admin-shell-err-upstream-dns = Le nom d’hôte n’a pas pu être résolu. Vérifiez l’orthographe de l’hôte.
admin-shell-err-upstream-connect = Rien n’a répondu sur cet hôte et ce port. Vérifiez le port et tout pare-feu intermédiaire.
admin-shell-err-upstream-tls = Le serveur a répondu mais la négociation TLS a échoué. Vérifiez le port et le paramètre de sécurité de la connexion.
admin-shell-err-upstream-auth = Le serveur a refusé les identifiants. Vérifiez le nom d’utilisateur et le mot de passe.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Trop de tentatives de connexion. Attendez quelques minutes et réessayez.
admin-shell-totp-form = Connexion administrateur, seconde étape
admin-shell-totp-note = Saisissez le code à 6 chiffres de l’application d’authentification configurée pour { $name }.
admin-shell-totp-code = Code d’authentification
admin-shell-totp-verify = Vérifier
admin-shell-totp-start-over = Recommencer
admin-shell-totp-invalid = Ce code n’est pas valide. Saisissez le code actuel de votre application d’authentification.
admin-shell-totp-expired = La connexion a pris trop de temps. Recommencez et saisissez votre mot de passe.
