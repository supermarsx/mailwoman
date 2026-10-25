# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = Assistant IA
assist-heading = Assistant IA
assist-transcript-label = Conversation avec l’assistant IA
assist-proposed-action = Action proposée
assist-outbox-note = Cela placerait un message dans votre boîte d’envoi. Rien n’est envoyé tant que vous ne l’avez pas confirmé là-bas.
assist-review-outbox = Vérifier dans la boîte d’envoi
assist-review = Vérifier
assist-empty = Demandez à l’assistant IA de résumer, de rédiger ou de trouver des éléments. Il peut proposer des actions, mais c’est toujours vous qui les confirmez.
assist-error = L’assistant IA n’a pas pu répondre pour le moment.
assist-input-label = Écrire à l’assistant IA
assist-input-placeholder = Demandez à l’assistant IA…
assist-thinking = Réflexion…
assist-ask = Demander

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Aide à la rédaction
assist-composer-toolbar = Outils de rédaction
assist-tool-grammar = Corriger la grammaire
assist-tool-rewrite = Reformuler
assist-tool-tone = Ajuster le ton
assist-tool-translate = Traduire
assist-toolarg-tone = Ton
assist-toolarg-translate = Langue
assist-tone-neutral = Neutre
assist-tone-friendly = Amical
assist-tone-formal = Formel
assist-tone-concise = Concis
assist-tone-assertive = Affirmé
assist-busy = { $label }…
assist-composer-empty-error = Écrivez d’abord quelque chose.
assist-composer-error = L’assistant IA n’a pas pu terminer cette opération pour le moment.
assist-suggested-edit = Modification suggérée
assist-apply-draft = Appliquer au brouillon
assist-discard = Ignorer

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Étiquettes suggérées
assist-apply-auto = Appliquer automatiquement
assist-apply-label = Appliquer { $label }
assist-remove-label = Retirer { $label }
assist-apply = Appliquer
assist-undo = Annuler

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Maintenir pour dicter
assist-dictate-stop = Arrêter la dictée
assist-dictate-hold-text = 🎙 Maintenir pour dicter
assist-dictate-listening = ● Écoute…
assist-dictate-endpoint-note = L’audio est transcrit par le point de terminaison de votre assistant IA.
assist-dictate-err = Erreur de dictée.
assist-transcribe-err = Impossible de transcrire l’audio.
assist-mic-err = Microphone indisponible.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Recherche sémantique
assist-semantic-note = Le texte de la requête est envoyé au point de terminaison de votre assistant IA pour classer les résultats par sens.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Ce qui peut quitter cet appareil
assist-left-1 = le texte du message sélectionné (objet + corps) pour la fonction choisie
assist-left-2 = l’hôte du point de terminaison auquel il a été envoyé
assist-left-3 = jamais : le contenu chiffré de bout en bout (exclu par défaut)
assist-left-4 = jamais : les pièces jointes (exclues par défaut)
assist-left-5 = jamais : vos identifiants ou vos autres comptes
assist-disclosure-off = L’assistant IA est désactivé. Aucun contenu de message ne quitte cet appareil.
assist-disclosure-sentence = Lorsque vous utilisez un outil de l’assistant IA, le texte du message sélectionné est relayé vers { $host }. { $excl } L’envoi n’est jamais automatisé — vous confirmez toujours avant que quoi que ce soit ne quitte votre boîte d’envoi.
assist-disclosure-excl-one = Il n’envoie jamais { $a }.
assist-disclosure-excl-two = Il n’envoie jamais { $a } ni { $b }.
assist-disclosure-admin-allowed = Votre administrateur a autorisé l’envoi du contenu chiffré et des pièces jointes.
assist-excluded-e2ee = le contenu chiffré de bout en bout
assist-excluded-attachments = les pièces jointes

# -- Admin → AI assistant (deployment configuration) ------------------------------
assist-admin-title = Assistant IA
assist-admin-intro = L’assistant IA envoie le texte des messages sélectionné par les utilisateurs au point de terminaison IA configuré ici. Il ne peut ni envoyer, ni supprimer, ni accepter du courrier. Le contenu chiffré de bout en bout et les pièces jointes sont retenus, sauf si vous les autorisez ci-dessous.
assist-admin-load-unreadable = La configuration enregistrée de l’assistant IA ne peut pas être lue : sa clé d’API ne s’ouvre pas avec la clé de ce serveur (MW_SERVER_KEY). Elle a été enregistrée avec une autre clé ou a été modifiée. L’assistant IA est désactivé. Remplissez le formulaire, y compris la clé d’API, et enregistrez pour la remplacer.
assist-admin-load-error = Impossible de charger la configuration de l’assistant IA. L’enregistrement est désactivé afin de ne pas écraser la configuration enregistrée.
assist-admin-status-running = L’assistant IA est en cours d’exécution. Les requêtes sont envoyées à { $host }.
assist-admin-status-running-pending = L’assistant IA fonctionne avec la configuration chargée au démarrage du serveur. Les modifications enregistrées prendront effet au redémarrage du serveur.
assist-admin-status-off = L’assistant IA est désactivé. Toutes les requêtes à l’assistant IA sont refusées.
assist-admin-status-pending = L’assistant IA n’est pas en cours d’exécution. La configuration enregistrée prendra effet au redémarrage du serveur.
assist-admin-status-local-program-not-allowed = L’assistant IA n’est pas en cours d’exécution. Le programme configuré { $program } ne figure pas dans MW_ASSIST_LOCAL_PROGRAMS sur le serveur ; le serveur ne l’a donc pas lancé.
assist-admin-status-no-endpoint = L’assistant IA est activé mais ne fonctionne pas, car aucun point de terminaison utilisable n’est configuré.
assist-admin-stop = Arrêter l’assistant IA maintenant
assist-admin-stop-note = L’arrêt prend effet immédiatement pour tous les utilisateurs et est conservé après un redémarrage.
assist-admin-resume = Activer l’assistant IA
assist-admin-resume-note = Si le serveur a démarré avec l’assistant IA désactivé, l’activation prendra effet au redémarrage du serveur.
assist-admin-kill-error = Impossible de modifier l’activation de l’assistant IA.
assist-admin-endpoint = Point de terminaison
assist-admin-endpoint-note = Le serveur contacte ce point de terminaison ; les navigateurs ne le font jamais. Les modifications prennent effet au redémarrage du serveur.
assist-admin-kind = Type de point de terminaison
assist-admin-kind-none = Aucun
assist-admin-kind-open-ai-compatible = Compatible OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Programme local
assist-admin-local-program-enable = Pour que l’assistant IA exécute un programme sur ce serveur, définissez MW_ASSIST_LOCAL_PROGRAMS avec le chemin absolu du programme là où le serveur est lancé (séparez plusieurs chemins par des virgules), puis redémarrez le serveur.
assist-admin-local-program-note = Le serveur lance ce programme pour chaque requête à l’assistant IA et lui transmet la requête sur l’entrée standard, y compris le texte du message sélectionné par l’utilisateur. Il lit la réponse sur la sortie standard du programme. Seuls les programmes listés dans MW_ASSIST_LOCAL_PROGRAMS sur le serveur peuvent être choisis.
assist-admin-local-program = Programme
assist-admin-local-program-unlisted = { $program } (non listé)
assist-admin-local-program-unlisted-note = Ce programme ne figure pas dans MW_ASSIST_LOCAL_PROGRAMS ; le serveur ne l’exécute donc pas et ne l’acceptera pas à l’enregistrement. Choisissez un programme listé ou un autre type de point de terminaison.
assist-admin-local-program-required = Choisissez le programme à exécuter.
assist-admin-local-args = Arguments, un par ligne
assist-admin-local-args-note = Chaque ligne est transmise au programme comme un argument. Aucun shell n’intervient.
assist-admin-base-url = URL de base
assist-admin-base-url-required = Saisissez l’URL de base du point de terminaison, commençant par http:// ou https://.
assist-admin-api-key = Clé d’API
assist-admin-api-key-stored = Une clé est enregistrée. Le serveur ne l’affiche plus.
assist-admin-api-key-replace = Remplacer la clé d’API
assist-admin-api-key-replace-placeholder = Laisser vide pour conserver la clé enregistrée
assist-admin-api-key-remove = Supprimer la clé enregistrée
assist-admin-api-key-keep = Conserver la clé enregistrée
assist-admin-api-key-remove-pending = La clé enregistrée sera supprimée lors de l’enregistrement.
assist-admin-chat-model = Modèle de conversation
assist-admin-embed-model = Modèle d’embeddings
assist-admin-stt-model = Modèle de reconnaissance vocale
assist-admin-model = Modèle
assist-admin-anthropic-version = Version de l’API
assist-admin-max-tokens = Nombre maximal de jetons par réponse
assist-admin-default-placeholder = Valeur par défaut du serveur
assist-admin-grants = Fonctions
assist-admin-grants-note = Seules les fonctions cochées ici peuvent être utilisées.
assist-admin-cap-summarize = Résumer
assist-admin-cap-draft = Rédiger et reformuler
assist-admin-cap-grammar = Grammaire
assist-admin-cap-dictation = Dictée
assist-admin-cap-search-semantic = Recherche sémantique
assist-admin-cap-auto-tag = Étiquetage automatique
assist-admin-cap-recap = Récapitulatif
assist-admin-cap-assistant = Conversation avec l’assistant IA
assist-admin-ceilings = Limites de données
assist-admin-ceilings-note = Ces limites s’appliquent à chaque requête, quelle que soit la fonction.
assist-admin-accounts = Identifiants des comptes dont le courrier peut être envoyé, un par ligne
assist-admin-accounts-note = Si aucun compte n’est listé, aucun contenu de courrier n’est envoyé au point de terminaison.
assist-admin-folders = Identifiants des dossiers auxquels limiter l’envoi, un par ligne
assist-admin-folders-note = Si aucun dossier n’est listé, tous les dossiers des comptes listés sont autorisés.
assist-admin-allow-e2ee = Autoriser l’envoi du contenu chiffré de bout en bout
assist-admin-allow-attachments = Autoriser l’envoi des pièces jointes
assist-admin-save = Enregistrer la configuration
assist-admin-saved = Enregistré.
assist-admin-saved-pending = Enregistré. La modification prendra effet au redémarrage du serveur.
assist-admin-save-error = La configuration n’a pas été enregistrée.
assist-admin-save-not-allowed = La configuration n’a pas été enregistrée car le serveur n’autorise pas ce programme. Seuls les programmes listés dans MW_ASSIST_LOCAL_PROGRAMS peuvent être utilisés.
