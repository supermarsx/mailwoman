# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Clés d’API
apikeys-heading = Clés d’API
apikeys-intro = Créez des clés à portée limitée pour vos scripts et intégrations. Chaque clé est stockée sous forme hachée, affichée une seule fois et révocable individuellement. Accordez la portée minimale qui suffit.

apikeys-label = Libellé
apikeys-label-aria = Libellé de la clé
apikeys-label-placeholder = p. ex. script de sauvegarde

apikeys-create = Créer une clé
apikeys-saved = Je l’ai enregistrée

apikeys-reveal-warning = Copiez ce secret maintenant — il n’est affiché qu’une fois et ne peut plus être récupéré.
apikeys-copy = Copier
apikeys-copy-aria = Copier le secret dans le presse-papiers
apikeys-copied = Copié dans le presse-papiers

apikeys-error-need-label = donnez un libellé à la clé pour pouvoir la reconnaître plus tard
apikeys-error-create = impossible de créer la clé
apikeys-error-revoke = impossible de révoquer la clé

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Clés existantes
apikeys-existing = Clés existantes
apikeys-none = Aucune clé pour l’instant.
apikeys-created = créée le { $date }
apikeys-last-used = dernière utilisation le { $date }
apikeys-never-used = jamais utilisée
apikeys-revoked-at = révoquée le { $date }
apikeys-revoke = Révoquer
apikeys-status-active = Active
apikeys-status-revoked = Révoquée

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Portée
apikeys-capabilities = Autorisations
apikeys-cap-read = Lire
apikeys-cap-send = Envoyer
apikeys-cap-delete = Supprimer
apikeys-surface = Domaine
apikeys-surface-mail-label = Courrier
apikeys-surface-pim-label = PIM (agenda / tâches / notes / contacts)
apikeys-accounts = Comptes
apikeys-all-accounts = Tous les comptes
apikeys-folders = Dossiers
apikeys-all-folders = Tous les dossiers
apikeys-folder-ids = Identifiants de dossiers (séparés par des virgules)
apikeys-folder-subset-aria = Sous-ensemble de dossiers
apikeys-constraints = Contraintes
apikeys-expiry = Expiration (RFC 3339, vide = aucune expiration)
apikeys-expiry-aria = Expiration
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Limite de débit (requêtes/min, vide = illimité)
apikeys-rate-limit-aria = Limite de débit
apikeys-ip-allowlist = Liste d’IP autorisées (CIDR/IP, séparées par des virgules ; vide = toutes)
apikeys-ip-allowlist-aria = Liste d’IP autorisées
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = lire
apikeys-verb-send = envoyer
apikeys-verb-delete = supprimer
apikeys-summary-mail = courrier
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = aucune action
apikeys-summary-nothing = rien
apikeys-summary-none = aucun
apikeys-summary-verbs-on = { $verbs } sur { $surfaces }
apikeys-summary-all-accounts = tous les comptes
apikeys-summary-accounts = comptes : { $ids }
apikeys-summary-all-folders = tous les dossiers
apikeys-summary-folders = dossiers : { $ids }
apikeys-summary-expires = expire le { $date }
apikeys-summary-rate = { $n } req/min
apikeys-summary-ips = IP : { $ips }
apikeys-summary-mcp-tools = Outils MCP : { $tools }
apikeys-summary-unattended = Envoi SANS SURVEILLANCE

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Clés MCP
apikeys-mcp-heading = Clés MCP
apikeys-mcp-intro = Accordez à un agent d’IA (via le Model Context Protocol) exactement les outils que vous choisissez. Le corps des messages renvoyé à l’agent est marqué comme entrée non fiable. Chaque outil est accordé individuellement.
apikeys-mcp-label-aria = Libellé de la clé MCP
apikeys-mcp-label-placeholder = p. ex. agent assistant
apikeys-mcp-create = Créer une clé MCP
apikeys-mcp-reveal-warning = Copiez ce secret maintenant — il n’est affiché qu’une fois.
apikeys-mcp-error-need-label = donnez un libellé à la clé MCP
apikeys-mcp-error-need-tool = accordez au moins un outil
apikeys-mcp-error-create = impossible de créer la clé MCP

apikeys-tools = Outils
apikeys-tools-group = Outils MCP
apikeys-outbox-suffix = (soumis à la boîte d’envoi)

apikeys-unattended-aria = Envoi sans surveillance
apikeys-unattended-allow = Autoriser l’envoi sans surveillance (contourne la boîte d’envoi — nécessite la contre-signature d’un administrateur)
apikeys-unattended-pending = L’envoi sans surveillance reste inactif jusqu’à ce qu’un administrateur contresigne cette clé.
apikeys-unattended-disclosure = Par défaut, un envoi autorisé arrive dans votre boîte d’envoi et attend que vous le confirmiez dans l’application — l’automatisation ne peut pas envoyer seule. L’envoi sans surveillance SUPPRIME cette étape de validation humaine, de sorte que cette clé peut envoyer du courrier sans confirmation. Il nécessite en outre qu’un administrateur contresigne la clé. Ne l’accordez qu’à une automatisation en laquelle vous avez entièrement confiance ; une clé d’envoi sans surveillance compromise peut envoyer du courrier en votre nom sans aucune demande.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Rechercher dans le courrier
apikeys-tool-mail-search-desc = Rechercher des messages. Les résultats sont marqués comme entrée non fiable.
apikeys-tool-mail-read-label = Lire le courrier
apikeys-tool-mail-read-desc = Lire le corps d’un message. Les corps sont marqués comme entrée non fiable.
apikeys-tool-folders-list-label = Lister les dossiers
apikeys-tool-folders-list-desc = Lister les dossiers de la boîte aux lettres.
apikeys-tool-drafts-create-label = Créer des brouillons
apikeys-tool-drafts-create-desc = Créer un brouillon (jamais envoyé automatiquement).
apikeys-tool-mail-send-label = Envoyer du courrier
apikeys-tool-mail-send-desc = Placer un message dans la boîte d’envoi pour confirmation dans l’application.
apikeys-tool-calendar-read-label = Lire l’agenda
apikeys-tool-calendar-read-desc = Lire les événements de l’agenda.
apikeys-tool-calendar-propose-label = Proposer des événements
apikeys-tool-calendar-propose-desc = Proposer (sans enregistrer) des événements d’agenda.
apikeys-tool-tasks-read-label = Lire les tâches
apikeys-tool-tasks-read-desc = Lire les tâches.
apikeys-tool-tasks-write-label = Écrire des tâches
apikeys-tool-tasks-write-desc = Créer ou modifier des tâches.
apikeys-tool-contacts-read-label = Lire les contacts
apikeys-tool-contacts-read-desc = Lire les contacts.
