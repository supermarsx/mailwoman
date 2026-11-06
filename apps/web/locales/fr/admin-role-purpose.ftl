# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Administration complète de toutes les fonctionnalités, y compris les administrateurs, les rôles, la sécurité, l’acheminement du courrier et la configuration du service. Réservez ce rôle aux opérateurs de confiance responsables de l’ensemble du déploiement.
admin-role-purpose-domain_admin = Administration quotidienne des utilisateurs et des domaines : créer, mettre à jour, désactiver ou supprimer des utilisateurs ; gérer les quotas, les fonctionnalités des utilisateurs, les règles de connexion du domaine, les paramètres et l’image de marque ; révoquer des sessions et gérer les bannissements. Inclut aussi le diagnostic du courrier et les exports d’utilisation et de données. N’inclut ni la gestion des rôles d’administrateur ni les modifications des serveurs en amont.
admin-role-purpose-helpdesk = Assistance aux comptes et tri des incidents : consulter les utilisateurs, l’activité d’audit et l’utilisation ; modifier les indicateurs des utilisateurs, désactiver des comptes et révoquer des sessions ; consulter les bannissements et lancer des simulations de connexion. N’inclut ni la création ou la suppression d’utilisateurs, ni la modification des quotas, ni l’écriture des paramètres généraux.
admin-role-purpose-auditor = Inspection et rapports : accorde toutes les capacités se terminant par .read, ainsi que l’export d’audit et d’utilisation. N’inclut aucune modification de configuration ni de compte. Les rapports exportés peuvent contenir des informations d’administration sensibles.
admin-role-purpose-security_officer = Opérations de sécurité : gérer les politiques de sécurité et d’authentification multifacteur, la politique de connexion et les bannissements ; désactiver des utilisateurs, révoquer des sessions ou des clés d’API ; consulter et exporter l’activité d’audit ; lire les journaux d’authentification et lancer des diagnostics. La configuration SSO est consultable, mais ce rôle n’inclut ni la modification du SSO ni la gestion des rôles d’administrateur.
admin-role-purpose-custom = Un rôle personnalisé avec { $count } autorisations sélectionnées. Son pouvoir provient de la liste exacte des autorisations ci-dessous, et non de son nom. Examinez chaque action avant de l’attribuer.
admin-role-purpose-scope = Le périmètre de domaine est défini lorsque ce rôle est attribué à un administrateur. Les exceptions d’autorisation ou de refus au niveau du compte peuvent modifier l’accès effectif. Une description n’accorde aucune autorisation supplémentaire.
admin-role-purpose-editor = Choisissez les tâches que cet administrateur doit effectuer, puis sélectionnez les autorisations correspondantes. L’accès en lecture, l’export de données et la modification d’enregistrements sont des actions distinctes. Vérifiez les identifiants exacts des capacités avant d’enregistrer.
admin-role-purpose-selection = Ces descriptions expliquent les actions sélectionnées. Elles n’élargissent pas les autorisations envoyées au serveur.
admin-role-purpose-read = Consulter les informations de cette zone.
admin-role-purpose-export = Exporter les informations de cette zone.
admin-role-purpose-revoke = Révoquer les sessions ou les identifiants visés par cette autorisation.
admin-role-purpose-delete = Supprimer les enregistrements couverts par cette autorisation.
admin-role-purpose-disable = Désactiver les comptes couverts par cette autorisation.
admin-role-purpose-create = Créer les enregistrements couverts par cette autorisation.
admin-role-purpose-specific = Effectuer l’action précise désignée par cette capacité ; vérifiez son identifiant avant de l’accorder.
admin-role-purpose-description = Objectif et responsabilités
admin-role-purpose-description-help = Expliquez qui doit recevoir ce rôle personnalisé et quelles tâches cette personne assume (jusqu’à 1024 caractères). Ce texte ne modifie pas les autorisations.
