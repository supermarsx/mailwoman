# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Administration Mailwoman
admin-nav = Sections d’administration
admin-sign-out = Se déconnecter
admin-nav-domains = Domaines
admin-nav-users = Utilisateurs
admin-nav-integrations = Intégrations
admin-nav-observability = Observabilité
admin-nav-plugins = Plugins
admin-nav-assist = Assistant IA
admin-nav-sso = Authentification unique
admin-nav-servermeta = Métadonnées du serveur
admin-nav-rethread = Maintenance
admin-nav-2fa = Exiger la double authentification

admin-delete = Supprimer
admin-revoke = Révoquer
admin-remove = Retirer
admin-saved = Enregistré.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Connexion administrateur
admin-login-note = Ce panneau fonctionne dans une session d’administration distincte. La session se termine après 30 minutes d’inactivité, et au plus tard 12 heures après la connexion.
admin-login-session-ended = Votre session d’administration est terminée. Reconnectez-vous.
admin-login-username = Nom d’utilisateur administrateur
admin-login-password = Mot de passe
admin-login-sign-in = Se connecter
admin-login-signing-in = Connexion…
admin-login-invalid = Identifiants d’administrateur non valides
admin-login-unreachable = Impossible de joindre le serveur

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domaines
admin-domains-load-error = Impossible de charger les domaines
admin-domains-save-error = Impossible d’enregistrer le domaine
admin-domains-delete-error = Impossible de supprimer le domaine
admin-domains-add = Ajouter un domaine
admin-domains-name = Nom de domaine
admin-domains-name-placeholder = example.com
admin-domains-note = Un domaine est enregistré par son nom. Les noms enregistrés sont proposés sur l’écran Exiger la double authentification lorsqu’une règle s’applique à un seul domaine. Enregistrer un domaine n’achemine ni ne filtre le courrier.
admin-domains-save = Enregistrer le domaine
admin-domains-empty = Aucun domaine pour l’instant.
admin-domains-delete-for = Supprimer { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Utilisateurs
admin-users-load-error = Impossible de charger les utilisateurs
admin-users-provision-error = Impossible de créer l’utilisateur
admin-users-flag-error = Impossible de mettre à jour l’indicateur
admin-users-revoke-error = Impossible de révoquer les sessions
admin-users-provision = Créer un utilisateur
admin-users-username = Nom d’utilisateur
admin-users-domain = Domaine
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Quota en octets (0 = illimité)
admin-users-quota-msgs = Quota en messages (0 = illimité)
admin-users-empty = Aucun utilisateur pour l’instant.
admin-users-col-account = Compte
admin-users-col-quota = Quota (octets/msgs)
admin-users-col-zeroaccess = Accès zéro
admin-users-col-flags = Indicateurs
admin-users-col-sessions = Sessions
admin-users-zeroaccess-for = Accès zéro pour { $account }
admin-users-zeroaccess-help = Simple mention. Le stockage à accès zéro est activé et désactivé par l’utilisateur, dans ses propres paramètres, avec sa propre clé. Cocher cette case ne chiffre pas le courrier stocké du compte, et la décocher ne le déchiffre pas.
admin-users-disable-for = Désactiver { $account }
admin-users-disabled = désactivé
admin-users-disabled-help = Bloque la connexion et met fin à toutes les sessions ouvertes de ce compte. Ses clés et jetons d’API sont refusés. Cela ne désactive pas la boîte aux lettres sur le serveur de messagerie : d’autres clients de messagerie peuvent encore y accéder tant que vous ne la désactivez pas là aussi.
admin-users-force-change-for = Forcer le changement de mot de passe pour { $account }
admin-users-force-change = forcer le changement
admin-users-force-change-help = L’utilisateur est retenu sur un écran de changement de mot de passe, à la prochaine connexion et dans les sessions déjà ouvertes, et ne peut pas accéder au courrier tant que le changement n’a pas réussi. Cela nécessite un backend de changement de mot de passe fonctionnel (MW_PASSWD_BACKEND). Sans lui, le changement ne peut pas réussir et l’utilisateur reste sur cet écran jusqu’à ce que vous décochiez cette case.
admin-users-revoke-for = Révoquer les sessions de { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Intégrations
admin-integrations-load-error = Impossible de charger les intégrations
admin-integrations-revoke-error = Impossible de révoquer la clé
admin-integrations-ldap = Annuaire LDAP / GAL
admin-integrations-nextcloud = Passerelle Nextcloud
admin-integrations-active = Actif
admin-integrations-configured = Configuré
admin-integrations-not-configured = Non configuré
admin-integrations-unknown = État inconnu
admin-integrations-config-note = « Configuré » signifie que ce déploiement dispose de paramètres pour l’intégration : une entrée d’annuaire LDAP activée dans la base de données, ou les trois variables d’environnement MW_NEXTCLOUD_*. Les deux sont lus au démarrage du serveur. Cela ne signifie pas que le service distant a été contacté ni qu’il est joignable.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = Aucun webhook enregistré.
admin-integrations-keys = Clés API et MCP
admin-integrations-keys-empty = Aucune clé émise.
admin-integrations-col-prefix = Préfixe
admin-integrations-col-account = Compte
admin-integrations-col-scopes = Portées
admin-integrations-col-status = État
admin-integrations-status-revoked = révoquée
admin-integrations-status-active = active
admin-integrations-revoke-key = Révoquer la clé { $prefix }
admin-integrations-col-unattended = Envoi sans intervention
admin-integrations-unattended-not-requested = Non demandé
admin-integrations-unattended-requested = Demandé, non approuvé
admin-integrations-unattended-approved = Approuvé
admin-integrations-unattended-approve = Approuver
admin-integrations-unattended-withdraw = Retirer
admin-integrations-unattended-approve-key = Approuver l’envoi sans intervention pour la clé { $prefix }
admin-integrations-unattended-withdraw-key = Retirer l’approbation de l’envoi sans intervention pour la clé { $prefix }
admin-integrations-unattended-approve-title = Approuver l’envoi sans intervention pour cette clé ?
admin-integrations-unattended-withdraw-title = Retirer l’approbation pour cette clé ?
admin-integrations-unattended-owner = Propriétaire : { $account }
admin-integrations-unattended-key = Clé : { $prefix }
admin-integrations-unattended-scope = Autorisations : { $permissions }. Outils MCP : { $tools }.
admin-integrations-unattended-scope-unreadable = La portée de cette clé n’a pas pu être lue.
admin-integrations-scope-read = lecture
admin-integrations-scope-send = envoi
admin-integrations-scope-delete = suppression
admin-integrations-scope-mail = courrier
admin-integrations-scope-pim = agenda, tâches, notes et contacts
admin-integrations-scope-none = aucune
admin-integrations-unattended-approve-meaning = Les messages envoyés avec cette clé via MCP sont transmis sans qu’une personne les libère.
admin-integrations-unattended-withdraw-meaning = Un envoi effectué avec cette clé via MCP est refusé tant que la clé n’est pas de nouveau approuvée. Le message n’est pas transmis et n’est pas placé dans la boîte d’envoi du propriétaire.
admin-integrations-unattended-approve-effect = L’approbation prend effet immédiatement, dès le prochain envoi de la clé. Aucun redémarrage n’est nécessaire.
admin-integrations-unattended-withdraw-effect = Le retrait prend effet immédiatement : la clé cesse d’envoyer sans intervention dès son prochain envoi. Aucun redémarrage n’est nécessaire.
admin-integrations-unattended-cancel = Annuler
admin-integrations-unattended-saving = Enregistrement…
admin-integrations-unattended-approved-done = Envoi sans intervention approuvé pour la clé { $prefix }. Il est en vigueur dès maintenant.
admin-integrations-unattended-withdrawn-done = Approbation retirée pour la clé { $prefix }. Cette clé n’envoie plus sans intervention.
admin-integrations-unattended-error-gone = La clé { $prefix } n’existe plus ou a été révoquée. Rien n’a été modifié.
admin-integrations-unattended-error-conflict = La clé { $prefix } est révoquée ou son propriétaire n’a pas demandé l’envoi sans intervention. Rien n’a été approuvé.
admin-integrations-unattended-error = Impossible de modifier l’approbation pour la clé { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Observabilité
admin-obs-load-error = Impossible de charger les données d’observabilité
admin-obs-export-error = Impossible d’exporter le journal d’audit
admin-obs-ban-add-error = Impossible d’ajouter le bannissement
admin-obs-unban-error = Impossible de retirer le bannissement
admin-obs-telemetry-note = La journalisation et la télémétrie ne se règlent pas ici. Le filtre de journal (MW_LOG), le collecteur OTLP (MW_OTLP_ENDPOINT) et le point de terminaison des métriques (MW_METRICS_TOKEN) sont lus dans l’environnement du serveur à son démarrage.
admin-obs-audit = Journal d’audit
admin-obs-export = Exporter en JSONL
admin-obs-audit-empty = Aucune entrée d’audit.
admin-obs-col-time = Heure
admin-obs-col-actor = Acteur
admin-obs-col-action = Action
admin-obs-col-target = Cible
admin-obs-bans = Surveillance des connexions / liste des bannissements
admin-obs-bans-note = Cette liste est un registre. Mailwoman ne refuse pas les connexions provenant des adresses qui y figurent, et une entrée n’a aucun effet sur la connexion. Chaque échec de connexion administrateur écrit une ligne dans le journal du serveur, exploitable par une prison fail2ban, et cinq échecs depuis une même adresse ajoutent cette adresse ici. Le blocage est assuré par cette prison ou par votre pare-feu.
admin-obs-ban-add = Ajouter un bannissement
admin-obs-ban-ip = Adresse IP
admin-obs-ban-reason = Motif
admin-obs-ban-btn = Bannir l’IP
admin-obs-bans-empty = Aucun bannissement actif.
admin-obs-unban-for = Débannir { $ip }
admin-obs-unban-btn = Débannir

# -- Plugins (§22) -----------------------------------------------------------
admin-plugins-title = Plugins
admin-plugins-intro = Les plugins du moteur s’exécutent dans un bac à sable WebAssembly contrôlé par capacités. Approuvez un plugin avant de pouvoir l’activer, et n’accordez que les capacités dont il a besoin.
admin-plugins-empty = Aucun plugin n’est enregistré.
admin-plugins-signed = Signé
admin-plugins-unsigned = Non signé
admin-plugins-approved = Approuvé
admin-plugins-enabled = Activé
admin-plugins-approve = Approuver
admin-plugins-enable = Activer
admin-plugins-disable = Désactiver
admin-plugins-net = réseau : { $hosts }
admin-plugins-limits = limites : { $memory } Mio · { $deadline } ms
admin-plugins-limits-fuel = limites : { $memory } Mio · { $deadline } ms · { $fuel } unités de carburant
admin-plugins-allow-unsigned-for = Autoriser le plugin non signé { $name }
admin-plugins-allow-unsigned = Autoriser l’exécution de ce plugin non signé
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
admin-allowlist-title = Liste d’autorisation des plugins tiers
admin-allowlist-intro = Un composant tiers (non interne) ne se charge qu’après qu’un administrateur a approuvé son empreinte SHA-256 exacte. L’empreinte ci-dessous est calculée par le serveur sur les octets du composant sur disque ; l’approuver épingle exactement ces octets. Les composants internes sont épinglés dans la build et ne sont pas gérés ici.
admin-allowlist-load-error = Impossible de charger la liste d’autorisation des plugins.
admin-allowlist-present-heading = Composants sur disque
admin-allowlist-present-empty = Aucun composant tiers n’est présent. Placez un composant dans le répertoire des plugins tiers pour qu’il apparaisse ici.
admin-allowlist-digest-label = SHA-256 calculé
admin-allowlist-status-approved = Approuvé
admin-allowlist-status-pending = Non approuvé
admin-allowlist-status-firstparty = Interne
admin-allowlist-unsigned-note = Admis par épinglage d’empreinte. Ce composant ne porte aucune signature ; l’approbation fait confiance aux octets exacts dont l’empreinte est affichée, ce qui est la situation attendue pour un composant approuvé de cette façon.
admin-allowlist-highpower-note = Les capacités à privilèges élevés ({ $caps }) ne peuvent pas être accordées à un plugin tiers. Le serveur les refuse quelle que soit l’approbation ; elles sont réservées aux composants internes.
admin-allowlist-firstparty-note = Cet identifiant correspond à un composant interne. L’épinglage interne est toujours prioritaire, donc cet identifiant ne peut pas être approuvé comme tiers.
admin-allowlist-approve = Approuver l’empreinte
admin-allowlist-approve-for = Approuver l’empreinte de { $id }
admin-allowlist-revoke = Révoquer
admin-allowlist-revoke-for = Révoquer l’épinglage de { $id }
admin-allowlist-uninstall = Désinstaller
admin-allowlist-uninstall-for = Désinstaller { $id }
admin-allowlist-pins-heading = Épinglages approuvés et révoqués
admin-allowlist-pins-empty = Aucun épinglage enregistré.
admin-allowlist-pin-approved-by = Approuvé par { $by } le { $at }
admin-allowlist-pin-revoked = Révoqué
admin-allowlist-cancel = Annuler

admin-allowlist-approve-title = Autoriser le chargement de ce composant ?
admin-allowlist-approve-detail = L’approbation épingle les octets exacts dont le SHA-256 est affiché ci-dessous. Après approbation, seuls des octets correspondant à cette empreinte seront chargés pour cet identifiant ; toute modification du composant produit une autre empreinte et ne sera pas chargée avant une nouvelle approbation. L’approbation n’accorde à elle seule aucune capacité.
admin-allowlist-approve-confirm = Approuver l’empreinte

admin-allowlist-revoke-title = Révoquer cet épinglage ?
admin-allowlist-revoke-detail = La révocation retire l’approbation de cette empreinte et désactive le plugin. Elle prend effet au prochain chargement ; une instance déjà en cours d’exécution n’est pas arrêtée d’ici là.
admin-allowlist-revoke-confirm = Révoquer l’épinglage

admin-allowlist-uninstall-title = Désinstaller ce plugin ?
admin-allowlist-uninstall-detail = La désinstallation supprime les données clé/valeur stockées par le plugin pour tous les comptes, retire ses épinglages de la liste d’autorisation et le désactive. Le fichier du composant sur disque n’est pas supprimé ; il pourra être réapprouvé plus tard.
admin-allowlist-uninstall-confirm = Désinstaller le plugin

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Authentification unique
admin-sso-intro = Configurez des backends de connexion OIDC et SAML 2.0. Les backends activés apparaissent sous forme de boutons « Se connecter avec… » sur la page de connexion à la messagerie, pour tout le déploiement ou pour un seul domaine.
admin-sso-add = Ajouter un backend de connexion
admin-sso-edit = Modifier
admin-sso-create = Ajouter le backend
admin-sso-update = Enregistrer les modifications
admin-sso-cancel = Annuler
admin-sso-empty = Aucun backend SSO configuré.
admin-sso-load-error = Impossible de charger les backends SSO.
admin-sso-save-error = Impossible d’enregistrer le backend.
admin-sso-delete-error = Impossible de supprimer le backend.

admin-sso-id = ID du backend
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Nom affiché
admin-sso-display-name-placeholder = Se connecter avec Acme SSO
admin-sso-kind = Protocole
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Portée
admin-sso-scope-deployment = Tout le déploiement
admin-sso-scope-domain = Un seul domaine
admin-sso-domain = Domaine
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Activé
admin-sso-first-login = Règle de première connexion
admin-sso-policy-allowlist = Liste d’autorisation (refuser les utilisateurs inconnus)
admin-sso-policy-autocreate = Créer automatiquement les comptes à la première connexion

admin-sso-issuer = URL de l’émetteur
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = ID client
admin-sso-client-secret = Secret client
admin-sso-secret-unchanged = Laisser vide pour conserver le secret enregistré
admin-sso-redirect = URL de redirection
admin-sso-scopes = Portées
admin-sso-metadata = Métadonnées du SP

admin-sso-sp-entity-id = ID d’entité du SP
admin-sso-acs-url = URL ACS
admin-sso-idp-metadata-url = URL des métadonnées de l’IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML des métadonnées de l’IdP
admin-sso-idp-metadata-xml-placeholder = Collez le XML des métadonnées de l’IdP, ou utilisez l’URL ci-dessus
admin-sso-idp-sso-url = URL SSO de l’IdP
admin-sso-idp-slo-url = URL de déconnexion (SLO) de l’IdP
admin-sso-idp-certs = Certificats de signature de l’IdP (PEM)
admin-sso-idp-certs-placeholder = Un certificat PEM par bloc, séparés par une ligne vide
admin-sso-nameid-format = Format du NameID
admin-sso-want-signed = Exiger des assertions signées
admin-sso-want-encrypted = Exiger des assertions chiffrées

admin-sso-claims = Correspondance des claims
admin-sso-claim-email = Claim de l’adresse e-mail
admin-sso-claim-username = Claim du nom d’utilisateur
admin-sso-claim-display = Claim du nom affiché
admin-sso-claim-groups = Claim des groupes

admin-sso-badge-enabled = Activé
admin-sso-badge-disabled = Désactivé
admin-sso-enable = Activer
admin-sso-disable = Désactiver
admin-sso-delete = Supprimer
admin-sso-enable-for = Activer { $name }
admin-sso-disable-for = Désactiver { $name }
admin-sso-delete-for = Supprimer { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = Métadonnées du serveur
admin-servermeta-intro = Consultez et modifiez les annotations serveur RFC 5464 d’un compte créé. Les modifications sont écrites directement sur le serveur de messagerie, qui décide si le compte peut les définir.
admin-servermeta-account = Compte
admin-servermeta-select-option = Sélectionnez un compte…
admin-servermeta-select-prompt = Sélectionnez un compte pour consulter et modifier ses annotations serveur.
admin-servermeta-load-error = Impossible de charger la liste des comptes.
admin-servermeta-no-accounts = Aucun compte n’est créé.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = Refaire les fils de la boîte
admin-rethread-intro = Relance le regroupement en conversations (JWZ) sur le courrier stocké d’un compte créé et recalcule les clés de ses fils. Il s’agit d’une opération de maintenance ponctuelle, et non d’une tâche automatique.
admin-rethread-account = Compte
admin-rethread-select-option = Sélectionnez un compte…
admin-rethread-no-accounts = Aucun compte n’est créé.
admin-rethread-load-error = Impossible de charger la liste des comptes.
admin-rethread-run = Refaire les fils de la boîte
admin-rethread-confirm-title = Refaire les fils de cette boîte ?
admin-rethread-confirm-warning = Refaire les fils recalcule le regroupement des conversations de ce compte. Des fils existants peuvent fusionner ou se scinder, et les utilisateurs peuvent voir leurs conversations regroupées autrement.
admin-rethread-confirm-detail = Il s’agit d’une opération de maintenance ponctuelle. Elle s’exécute une fois maintenant ; elle peut être relancée sans risque et ne supprime aucun message.
admin-rethread-confirm = Refaire les fils de la boîte
admin-rethread-running = Reconstruction des fils…
admin-rethread-cancel = Annuler
admin-rethread-summary = { $messages } message(s) regroupé(s) en { $threads } fil(s) sur { $accounts } compte(s) ; { $reassigned } message(s) déplacé(s) vers un autre fil.
admin-rethread-error = L’opération a échoué. Si le serveur a rejeté la requête, aucun regroupement n’a été modifié ; consultez les journaux du serveur et réessayez.

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = Index de recherche
admin-searchindex-intro = La recherche lit un index construit à partir du courrier stocké. Le serveur le reconstruit au démarrage s’il ne correspond pas au courrier stocké. Le courrier des comptes à accès zéro n’est pas indexé : la recherche ne trouve donc rien pour ces comptes.
admin-searchindex-count = { $documents } message(s) stocké(s) sur { $messages } sont indexés.
admin-searchindex-on-disk = L’index est conservé sur disque.
admin-searchindex-in-memory = L’index est conservé en mémoire et reconstruit à chaque démarrage du serveur.
admin-searchindex-progress = Une reconstruction est en cours : { $done } message(s) traité(s) sur { $total }.
admin-searchindex-run = Reconstruire l’index de recherche
admin-searchindex-running = Reconstruction…
admin-searchindex-summary = { $messages } message(s) stocké(s) lu(s) dans { $accounts } compte(s) : { $indexed } indexé(s), { $removed } retiré(s) de l’index, { $failed } illisible(s). { $zeroAccess } compte(s) à accès zéro non indexé(s).
admin-searchindex-busy = Une reconstruction est déjà en cours. Attendez qu’elle se termine.
admin-searchindex-error = La reconstruction a échoué. L’index conserve son contenu ; consultez les journaux du serveur et réessayez.
admin-searchindex-status-error = Impossible de lire l’état de l’index de recherche.
admin-searchindex-unavailable = Ce serveur fonctionne en mode proxy et n’a pas d’index de recherche.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = Exiger la double authentification
admin-2fa-intro = Exigez un second facteur (clé d’accès ou application d’authentification) pour la connexion. Tout utilisateur peut enregistrer un facteur de lui-même ; l’exiger ici oblige les comptes concernés à en enregistrer un à leur prochaine connexion.
admin-2fa-load-error = Impossible de charger la règle de double authentification.
admin-2fa-save-error = Impossible d’enregistrer la règle de double authentification.
admin-2fa-global = Exiger la double authentification pour tout le déploiement
admin-2fa-global-label = Exiger la double authentification pour tout le déploiement
admin-2fa-global-note = Lorsque cette option est activée, chaque compte doit disposer d’un second facteur. Une règle par domaine ci-dessous peut aussi l’exiger pour un seul domaine sans l’exiger partout.
admin-2fa-domains-heading = Exigences par domaine
admin-2fa-domains-empty = Aucune exigence par domaine définie.
admin-2fa-col-domain = Domaine
admin-2fa-col-require = Exigée
admin-2fa-require-for = Exiger la double authentification pour { $domain }
admin-2fa-add-domain = Domaine
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Exiger la double authentification pour ce domaine
admin-2fa-add-require-label = Exiger la double authentification pour ce domaine
admin-2fa-add-save = Ajouter la règle de domaine

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Sortie
admin-egress-heading = Routes de proxy sortant
admin-egress-intro = Faites passer les requêtes sortantes du serveur par un proxy HTTP CONNECT ou SOCKS5. Mailwoman applique toujours sa propre règle d’adresses — le proxy n’est jamais chargé de résoudre un nom.
admin-egress-empty = Aucune route configurée. Les requêtes sortantes sont directes.
admin-egress-none-active = Aucune route n’est utilisée. Les requêtes sortantes sont directes. Une route enregistrée ne transporte du trafic qu’après avoir choisi « Utiliser cette route ».
admin-egress-in-use = Les requêtes sortantes utilisent la route « { $id } ».
admin-egress-deactivate = Ne plus utiliser de proxy
admin-egress-activate = Utiliser cette route
admin-egress-activate-for = Utiliser la route { $id }
admin-egress-activate-error = Impossible de basculer vers la route
admin-egress-deactivate-error = Impossible d’arrêter d’utiliser la route
admin-egress-col-state = État
admin-egress-state-active = Utilisée
admin-egress-state-inactive = Non utilisée
admin-egress-test-scope = Seule la route utilisée peut être testée. Un test récupère une URL fixe via la route et indique jusqu’où il est allé ; il ne modifie rien.
admin-egress-probe-url = Récupéré : { $url }
admin-egress-load-error = Impossible de charger les routes de sortie
admin-egress-save-error = Impossible d’enregistrer la route
admin-egress-delete-error = Impossible de supprimer la route
admin-egress-invalid = Un identifiant, un hôte et un port supérieur à zéro sont requis.
admin-egress-username-required = Un mot de passe nécessite un nom d’utilisateur ; sans lui, la route s’authentifierait sans identité.
admin-egress-col-id = Id
admin-egress-col-route = Route
admin-egress-col-username = Nom d’utilisateur
admin-egress-col-credentials = Mot de passe
admin-egress-col-plaintext = En clair
admin-egress-col-actions = Actions
admin-egress-cred-set = Défini
admin-egress-cred-none = Aucun
admin-egress-plaintext-on = Autorisé
admin-egress-plaintext-off = Refusé
admin-egress-add-heading = Ajouter une route
admin-egress-edit-heading = Modifier la route
admin-egress-id = Id
admin-egress-scheme = Schéma
admin-egress-host = Hôte
admin-egress-port = Port
admin-egress-username = Nom d’utilisateur
admin-egress-password = Mot de passe
admin-egress-password-unchanged = Laisser vide pour conserver le mot de passe enregistré
admin-egress-allow-plaintext = Autoriser les origines http en clair via cette route
admin-egress-allow-plaintext-note = Désactivé par défaut. Une origine https empêche le proxy de lire le trafic qu’il transporte ; autoriser http signifie que le proxy le voit.
admin-egress-test = Tester
admin-egress-testing = Test en cours…
admin-egress-test-failed = Le test n’a pas pu être exécuté ; on ne sait donc rien de cette route.
admin-egress-outcome-connected = Connecté
admin-egress-outcome-auth-rejected = Authentification refusée
admin-egress-outcome-refused-by-policy = Refusé par la règle
admin-egress-outcome-dns-failed = Le nom n’a pas été résolu
admin-egress-outcome-unreachable = Injoignable
admin-egress-outcome-origin-tls-failed = Échec TLS vers l’origine
admin-egress-outcome-route-invalid = La route n’est pas valide
admin-egress-outcome-unknown = Résultat non reconnu
admin-egress-stage-dns = arrêté à la résolution de nom
admin-egress-stage-connect = arrêté pendant la connexion au proxy
admin-egress-stage-tunnel = arrêté pendant l’ouverture du tunnel
admin-egress-stage-origin = arrêté à l’origine
admin-egress-stage-unknown = arrêté à une étape non reconnue
admin-egress-proxied-yes = Passé par le proxy
admin-egress-proxied-no = Non passé par le proxy
