# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Avertissement : plugin d’interface non signé
plugins-unsigned-title = Des plugins non signés sont en cours d’exécution
plugins-unsigned-text = Ces plugins ont été admis sans signature vérifiée. Ils s’exécutent dans un bac à sable, mais leur code n’a pas été vérifié par rapport à une clé de confiance.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plugin : { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Les plugins du moteur s’exécutent dans un bac à sable WebAssembly. Un plugin ne s’exécute qu’après avoir été enregistré, approuvé, doté d’au moins une capacité et activé. Il ne peut utiliser que les capacités accordées ici.
plugins-admin-unsigned-banner = Un plugin tiers sans signature est chargé. Il s’exécute parce qu’un administrateur l’a autorisé ; le serveur vérifie seulement son fichier par rapport à l’empreinte approuvée.
plugins-admin-load-error = Impossible de charger les plugins.
plugins-admin-register-heading = Enregistrer un plugin
plugins-admin-register-component = Composant
plugins-admin-register-third-party = Composant tiers
plugins-admin-register-id = Identifiant du plugin
plugins-admin-register-id-hint = Le serveur lit le composant dans le fichier portant cet identifiant dans le répertoire des plugins tiers. Approuvez d’abord l’empreinte de ce fichier dans la liste d’autorisation ci-dessous.
plugins-admin-register-name = Nom
plugins-admin-register-version = Version
plugins-admin-register-capabilities = Capacités déclarées par le plugin
plugins-admin-register-hosts = Hôtes que le plugin peut contacter
plugins-admin-register-hosts-hint-first-party = Noms d’hôte séparés par des virgules. Laisser vide pour utiliser les hôtes par défaut du composant.
plugins-admin-register-hosts-hint-third-party = Noms d’hôte séparés par des virgules. Laisser vide pour n’autoriser aucun accès réseau.
plugins-admin-register-submit = Enregistrer
plugins-admin-trust-first-party = Interne, empreinte intégrée
plugins-admin-trust-third-party = Tiers, empreinte approuvée
plugins-admin-status-loaded = Chargé
plugins-admin-status-not-loaded = Non chargé
plugins-admin-status-restart = Redémarrage requis
plugins-admin-restart-to-load = Redémarrage requis pour charger ce plugin.
plugins-admin-restart-to-apply = Redémarrage requis pour appliquer ces modifications. Certains comptes de cette passerelle ne sont pas servis d’ici là.
plugins-admin-reason-not-approved = Non chargé : aucun administrateur n’a approuvé ce plugin.
plugins-admin-reason-disabled = Non chargé : le plugin n’est pas activé.
plugins-admin-reason-unsigned-not-allowed = Non chargé : le plugin n’a pas de signature et n’a pas été autorisé à s’exécuter sans.
plugins-admin-reason-no-grant = Non chargé : aucune des capacités dont il a besoin n’a été accordée.
plugins-admin-reason-component-unavailable = Non chargé : aucun fichier de composant n’a passé la vérification d’empreinte.
plugins-admin-reason-load-failed = Non chargé : le serveur n’a pas pu charger le composant. Consultez le journal du serveur.
plugins-admin-reason-proxy-mode = Non chargé : ce serveur fonctionne en mode proxy et ne traite pas lui-même le courrier.
plugins-admin-reason-no-account-binding = Non chargé : aucun compte n’est lié à cette passerelle.
plugins-admin-reason-no-host-caller = Non chargé : cette version du serveur n’appelle pas les points d’accroche fournis par ce plugin.
plugins-admin-reason-another-classifier-active = Non chargé : un autre classificateur de spam est chargé. Un seul s’exécute à la fois.
plugins-admin-running-with = En cours d’exécution avec : { $capabilities }
plugins-admin-hosts = Hôtes : { $hosts }
plugins-admin-hosts-none = Hôtes : aucun
plugins-admin-grants-heading = Capacités accordées
plugins-admin-grants-hint = Le plugin ne peut utiliser que les capacités cochées ici. L’enregistrement remplace l’attribution précédente.
plugins-admin-grant-for = Accorder { $capability } à { $name }
plugins-admin-grants-save = Enregistrer les attributions
plugins-admin-endpoint = Adresse du classificateur
plugins-admin-endpoint-for = Adresse du classificateur pour { $name }
plugins-admin-endpoint-hint = Adresse à laquelle le plugin joint son démon. L’hôte doit aussi figurer dans la liste des hôtes du plugin. Laisser vide pour utiliser la valeur par défaut du composant.
plugins-admin-endpoint-save = Enregistrer l’adresse
plugins-admin-test = Tester le classificateur
plugins-admin-test-verdict = Verdict du test : { $verdict }
plugins-admin-uninstall = Désinstaller
plugins-admin-uninstall-confirm = Confirmer la désinstallation
plugins-admin-uninstall-note = La désinstallation supprime l’enregistrement, ses attributions et ses paramètres stockés.
plugins-admin-uninstall-detail = La désinstallation supprime les données stockées du plugin pour tous les comptes, ses épinglages dans la liste d’autorisation, ainsi que son enregistrement et ses attributions. Le fichier du composant sur disque n’est pas supprimé.
plugins-admin-revoke-detail = La révocation retire l’approbation de cette empreinte et désactive le plugin. Si le plugin est en cours d’exécution, il est arrêté immédiatement.
plugins-admin-error-already-registered = Ce plugin est déjà enregistré. Désinstallez-le avant de l’enregistrer à nouveau.
plugins-admin-error-component-unavailable = Aucun fichier de composant de ce plugin ne correspond à l’empreinte intégrée au serveur.
plugins-admin-error-digest-not-approved = Aucun fichier de composant avec une empreinte approuvée n’a été trouvé pour cet identifiant. Approuvez d’abord l’empreinte du fichier dans la liste d’autorisation.
plugins-admin-error-not-approved = Approuvez le plugin avant de l’activer.
plugins-admin-error-unsigned-not-allowed = Ce plugin n’a pas de signature. Autorisez son exécution sans signature avant de l’activer.
plugins-admin-error-not-loaded = Le plugin n’est pas chargé ; il ne peut donc pas être testé.
plugins-admin-error-classifier-error = L’appel au classificateur a échoué : { $detail }
plugins-admin-error-other = Le serveur a refusé la requête : { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = Plugins d’interface
plugins-admin-ui-title = Plugins d’interface
plugins-admin-ui-intro = Les plugins d’interface ajoutent des contrôles au client web. Chacun s’exécute dans un cadre isolé et ne peut utiliser que les capacités accordées ici. Un plugin est proposé aux utilisateurs connectés une fois approuvé et activé.
plugins-admin-ui-empty = Aucun plugin d’interface n’est enregistré.
plugins-admin-ui-load-error = Impossible de charger les plugins d’interface.
plugins-admin-ui-register-heading = Enregistrer un plugin d’interface
plugins-admin-ui-manifest = Manifeste (JSON)
plugins-admin-ui-bundle = Fichier du bundle
plugins-admin-ui-bundle-hint = Requis pour un plugin signé : la signature est vérifiée par rapport à ce fichier.
plugins-admin-ui-allow-unsigned = Autoriser l’enregistrement de ce plugin sans signature
plugins-admin-ui-register = Enregistrer
plugins-admin-ui-manifest-invalid = Le manifeste n’est pas un JSON valide.
plugins-admin-ui-refused = Le serveur a refusé la requête : { $detail }
plugins-admin-ui-approve = Approuver et activer
plugins-admin-ui-extension-points = Points d’extension : { $points }
plugins-admin-ui-grants-heading = Accorder une capacité
plugins-admin-ui-grants-note = Le serveur n’indique pas quelles capacités sont déjà accordées. Accorder à nouveau une capacité remplace son attribution précédente.
plugins-admin-ui-grant = Accorder { $capability }
plugins-admin-ui-grant-for = Accorder { $capability } à { $name }
plugins-admin-ui-grant-hosts = Hôtes pour { $capability } sur { $name }
plugins-admin-ui-grant-hosts-hint = Noms d’hôte, séparés par des virgules, que le plugin peut interroger.
plugins-admin-ui-granted = { $capability } accordée.
plugins-admin-ui-delete = Supprimer
plugins-admin-ui-delete-confirm = Confirmer la suppression
