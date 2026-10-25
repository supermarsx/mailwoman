# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: rules. Source: ../en/rules.ftl

rules-title = Règles et filtres
rules-intro = Trier, étiqueter ou traiter automatiquement le courrier entrant. Les règles qui n’utilisent que des tests standard s’exécutent sur le serveur de messagerie en Sieve ; les autres s’exécutent dans le moteur à la distribution.
rules-empty = Aucune règle pour l’instant.
rules-unavailable = Ce serveur de messagerie ne prend pas en charge les règles.
rules-load-failed = Impossible de charger les règles.
rules-new = Nouvelle règle
rules-list-label = Règles de courrier
rules-enabled = Activée
rules-delete-named = Supprimer la règle { $name }

rules-builder-title = Éditeur de règle
rules-builder-hint = Sélectionnez une règle à modifier ou créez-en une.
rules-name-label = Nom de la règle
rules-match-legend = Quand l’exécuter
rules-match-all = Toutes les conditions
rules-match-any = Au moins une condition
rules-conditions-legend = Conditions
rules-actions-legend = Actions
rules-add-condition = Ajouter une condition
rules-add-action = Ajouter une action
rules-remove-condition = Supprimer la condition
rules-remove-action = Supprimer l’action
rules-cond-type-label = Champ
rules-cond-op-label = Opérateur
rules-cond-value-label = Valeur recherchée
rules-act-type-label = Action
rules-act-value-label = Valeur de l’action
rules-save = Enregistrer la règle
rules-editing = Modification de { $name }
rules-save-refused = La règle n’a pas été enregistrée : { $reason }

rules-cond-from = De
rules-cond-to = À
rules-cond-subject = Objet
rules-cond-thread-unsupported = Fil { $op } { $value } : les conditions sur les fils ne sont pas prises en charge ; la règle ne peut pas être enregistrée tant que cette condition n’est pas supprimée.
rules-op-contains = contient
rules-op-is = est

rules-act-move = Déplacer vers le dossier
rules-act-tag = Ajouter une étiquette
rules-act-archive = Archiver
rules-act-suppress = Supprimer la notification
rules-act-stop = Arrêter le traitement

rules-runs-server = Serveur (Sieve)
rules-runs-server-detail = Envoyée au serveur de messagerie et appliquée sur celui-ci.
rules-runs-engine = Moteur
rules-runs-engine-detail = Appliquée par le moteur à la distribution.

rules-panes-label = Outils de règles
rules-tab-builder = Éditeur
rules-tab-raw = Sieve brut
rules-tab-dryrun = Simulation

rules-raw-label = Source Sieve brute
rules-raw-reset = Rétablir le code généré
rules-lint-clean = Aucun problème détecté.

rules-dryrun-title = Simulation
rules-dryrun-help = Saisissez un message d’exemple pour voir quelles règles se déclencheraient.
rules-dryrun-results = Résultats de la simulation
rules-dryrun-none = Aucune règle activée à évaluer.
rules-dryrun-nomatch = Aucune correspondance
rules-dryrun-stopped = Non atteinte (une règle précédente a arrêté le traitement)
