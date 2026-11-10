# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: privacy. Source: ../en/privacy.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-title = Confidentialité
privacy-help = Les images distantes peuvent révéler quand vous ouvrez un message. Choisissez quand elles se chargent et si le serveur les récupère pour vous.
privacy-remote-mode = Contenu distant
privacy-mode-block = Bloquer le contenu distant
privacy-mode-ask = Demander avant de charger via le serveur
privacy-mode-proxy = Charger via le serveur
privacy-mode-direct = Charger directement depuis les serveurs de l’expéditeur
privacy-trackers = Bloquer les images et balises de suivi
privacy-clean-links = Supprimer les paramètres de suivi des liens
privacy-direct-unavailable = Le chargement direct nécessite l’autorisation de votre administrateur. Il expose votre adresse réseau à l’hôte de l’image.
privacy-direct-native = Le chargement direct n’est pas disponible dans cette application native. Les images peuvent se charger via le serveur.
privacy-inherit = Utiliser le réglage hérité
privacy-content-failed = Certaines images n’ont pas pu être autorisées. Le message reste lisible, ces images restant bloquées.
privacy-frame-failed = Le visualiseur d’images isolé n’a pas pu se charger. Rechargez le message pour réessayer.
privacy-download-file = Télécharger via le serveur
privacy-file-failed = Ce fichier n’a pas pu être téléchargé via le serveur.
privacy-admin-title = Confidentialité et contenu distant
privacy-admin-help = Définissez les valeurs par défaut, les verrous et les limites via la politique des paramètres. Les compteurs de diagnostic ne concernent que ce processus serveur.
privacy-cache-entries = Images en cache
privacy-cache-bytes = Taille du cache (octets)
privacy-replaced-trackers = Pixels espions remplacés
privacy-edit-setting = Modifier la politique
privacy-outbound-title = Connexions sortantes
privacy-fail-closed = Une route configurée en échec bloque la requête. Il n’y a jamais de repli sur une connexion directe.
privacy-linked-files = Fichiers liés dans ce message
privacy-allow-direct = Autoriser le chargement direct des images
privacy-max-bytes = Taille maximale d’une image (octets)
privacy-cache-limit = Taille maximale du cache (octets)
privacy-rate = Requêtes par minute
privacy-on = Activé
privacy-off = Désactivé
privacy-locked = Verrouillé par la politique de l’administrateur
privacy-process-stats = Depuis le démarrage de ce processus serveur
privacy-refresh = Actualiser
privacy-traffic-class = Classe de trafic
privacy-allowed = Autorisé
privacy-refused = Refusé par la politique
privacy-failed = Échec
privacy-policy-values = Politique de contenu distant
privacy-cache-summary = Cache et protection contre le suivi
privacy-traffic-summary = Traitement du trafic sortant
privacy-metric = Mesure
privacy-value = Valeur
privacy-total-duration = Durée totale
privacy-no-traffic = Aucun trafic sortant n’a été enregistré depuis le démarrage de ce processus.
privacy-analytics-title = Compteurs d’utilisation facultatifs
privacy-analytics-help = Désactivé par défaut. Si votre administrateur active un collecteur, vous pouvez partager des noms d’écran fixes comme Courrier ou Calendrier. Les messages, adresses e-mail, recherches, détails de route et identifiants de navigateur ne sont jamais envoyés. Il s’agit de compteurs d’événements, non de statistiques de visiteurs uniques.
privacy-analytics-consent = Autoriser les compteurs facultatifs d’utilisation des écrans pour ce compte
privacy-analytics-signal = Le signal de confidentialité de votre navigateur empêche les statistiques, même si le consentement est enregistré.
privacy-load-time-title = Temps de chargement
privacy-load-time-help = Aidez votre administrateur à voir la rapidité de Mailwoman. Seuls le nombre de millisecondes et l’origine des données (cache ou réseau) sont transmis, de façon anonyme et agrégée. Aucun dossier, message, expéditeur ni objet n’est inclus.
privacy-load-time-folder = Mesurer le temps d’ouverture des dossiers
privacy-load-time-folder-help = Activé par défaut. Transmet le temps écoulé entre le choix d’un dossier et l’affichage de sa liste de messages.
privacy-load-time-message = Mesurer le temps d’ouverture des messages
privacy-load-time-message-help = Activé par défaut. Transmet le temps écoulé entre l’ouverture d’un message et l’affichage de ses en-têtes et de son texte.
privacy-analytics-save-error = Le consentement n’a pas pu être synchronisé. Ce navigateur suspend les nouvelles requêtes de statistiques jusqu’à l’enregistrement de la modification.
privacy-analytics-saving = Enregistrement du consentement…
privacy-back-policies = Retour aux politiques de confidentialité
privacy-image-toast = Afficher une notification pendant le chargement des images distantes
privacy-image-downloads = Téléchargements manuels d’images distantes
privacy-image-download-help = Si une image ne s’affiche pas, téléchargez une copie vérifiée via ce serveur. Le serveur vérifie qu’elle appartient à ce message et applique toujours ses règles de confidentialité et de sécurité. Les images déchiffrées côté client ne sont pas disponibles ici.
privacy-image-download-ready = L’image est prête et son téléchargement a commencé.
privacy-image-download-failed = L’image n’a pas pu être téléchargée. Elle est peut-être indisponible ou bloquée par votre politique de confidentialité.
privacy-strip-mailer-headers = Supprimer les en-têtes de logiciel des messages sortants
privacy-strip-mailer-help = Les envois SMTP et EWS natifs suppriment les en-têtes d’empreinte du client au niveau supérieur. Les corps de message, les messages joints et les données de rapport restent intacts. Cela ne couvre pas les envois via le proxy JMAP amont ni les en-têtes ajoutés ensuite par un relais. Les messages pré-signés couvrant un en-tête supprimé exigent de désactiver ce réglage avant l’envoi.
privacy-frame-compatible = Visualiseur compatible : les images chargées directement depuis d’autres sites restent bloquées pour préserver la confidentialité. Les images intégrées et les images chargées via le serveur sont prises en charge.
privacy-analytics-tag-custom = un script de statistiques personnalisé
privacy-analytics-tag-title = Statistiques du navigateur : { $provider }
privacy-analytics-tag-help = Votre administrateur propose également { $provider }, qui s’exécute dans cette page, dans votre navigateur. Contrairement aux compteurs ci-dessus, son script peut voir les pages que vous ouvrez dans cette application, y compris les messages et adresses affichés à l’écran, votre navigateur et votre adresse IP, et il peut définir ses propres identifiants. Le fournisseur peut mettre à jour son script sans vous redemander. Désactivé tant que vous ne l’activez pas ici ; le désactiver recharge la page sans lui, dans tous les onglets.
privacy-analytics-tag-consent = Autoriser { $provider } à s’exécuter dans cette application pour ce compte
privacy-analytics-tag-changed = Le fournisseur de statistiques, ou la façon dont ce serveur le charge, a changé depuis votre dernier accord. Il reste désactivé tant que vous n’avez pas de nouveau donné votre accord.
privacy-trusted-links-title = Domaines de liens de confiance
privacy-trusted-links-help = Les liens vers ces domaines s’ouvrent directement depuis Vérifier les liens, sans l’étape de destination. Les autres liens, y compris les domaines trompeurs, restent vérifiés. Les liens en HTTP simple ou avec une adresse IP sont toujours vérifiés.
privacy-trusted-links-domains = Domaines
privacy-trusted-links-domains-help = Un domaine par ligne, par exemple example.com. La correspondance est exacte : example.com ne fait confiance ni à examples.com ni à example.com.other.net. Utilisez la forme ASCII (xn--) pour les noms internationaux.
privacy-trusted-links-invalid = Saisissez jusqu’à 100 noms de domaine comportant au moins deux libellés, sans schéma, chemin, port ni caractère générique. Non acceptés : { $entries }
privacy-trusted-links-save = Enregistrer les domaines de confiance
privacy-trusted-links-subdomains = Faire aussi confiance aux sous-domaines (mail.example.com pour example.com)
privacy-trusted-links-subdomains-help = Désactivé : seul le domaine exact est de confiance. Activé : tout nom se terminant par .example.com l’est aussi.
privacy-linked-files-count = { $count ->
    [one] Un fichier lié dans ce message
   *[other] { $count } fichiers liés dans ce message
}
privacy-linked-links = Liens dans ce message
privacy-linked-files-show = Afficher les fichiers
privacy-linked-links-show = Télécharger un lien
privacy-linked-files-help = Les téléchargements passent par ce serveur, de sorte que l’hôte du fichier ne voit pas votre adresse. La taille d’un fichier n’est connue qu’une fois téléchargé.
privacy-linked-links-help = Un lien de ce message peut être téléchargé comme fichier via ce serveur, de sorte que son hôte ne voit pas votre adresse.
privacy-linked-file-item = Télécharger { $name } depuis { $host }
privacy-linked-other-item = Télécharger { $link }
privacy-links-enable = Activer les liens
privacy-links-options = Options des liens
privacy-links-enable-once = Activer une fois
privacy-links-trust-sender = Toujours autoriser les liens de cet expéditeur
privacy-links-revoke-sender = Ne plus toujours autoriser les liens de cet expéditeur
privacy-links-trust-failed = Les préférences de liens n’ont pas pu être enregistrées.
privacy-trusted-link-senders-title = Expéditeurs de liens de confiance
privacy-trusted-link-senders-help = Autoriser automatiquement les liens des messages provenant de ces adresses d’expéditeur exactes. Les destinations des liens restent validées.
privacy-trusted-link-senders-list = Adresses d’expéditeur
privacy-trusted-link-senders-list-help = Une adresse e-mail complète par ligne. La partie avant @ est comparée exactement, casse comprise. Supprimez une adresse pour retirer la confiance.
privacy-trusted-link-senders-save = Enregistrer les expéditeurs de confiance
privacy-trusted-link-senders-remove = Supprimer { $sender }
privacy-trusted-link-senders-invalid = Saisissez des adresses e-mail individuelles valides : { $entries }
privacy-links-disable = Désactiver les liens
privacy-links-enable-help = Permet d’ouvrir les liens de ce message dans un nouvel onglet. Vous pouvez les désactiver de nouveau.
privacy-links-loading = Préparation des liens…
privacy-body-blocked = HTML du message bloqué par mesure de sécurité
privacy-body-blocked-detail = Ce corps de message contient un contenu potentiellement dangereux ou n’a pas pu être vérifié. Le contenu actif reste bloqué. Le message d’origine n’est pas modifié.
privacy-body-show-sanitized = Afficher le HTML nettoyé
privacy-body-hide-sanitized = Afficher le texte sûr
privacy-body-sanitized-shown = HTML nettoyé affiché
privacy-unsafe-html = HTML de message potentiellement dangereux
privacy-unsafe-html-ask = Demander avant d’afficher le HTML nettoyé
privacy-unsafe-html-strict = Toujours afficher le texte sûr
privacy-unsafe-html-sanitized = Toujours afficher le HTML nettoyé
privacy-unsafe-html-help = Le HTML nettoyé supprime le contenu actif. Les ressources distantes, les liens et les accusés automatiques restent bloqués pour ces messages. Les originaux ne sont pas modifiés.
privacy-original-html-allow = Autoriser le HTML d’origine message par message
privacy-original-html-help = Propose une vue explicite du HTML d’origine pour chaque message. Les scripts et les formulaires restent désactivés, et les images et les liens suivent toujours vos réglages de confidentialité. Le HTML nettoyé reste la vue par défaut.
privacy-original-html = HTML d’origine
privacy-original-html-show = Afficher le HTML d’origine
privacy-original-html-hide = Retour à la vue nettoyée
privacy-original-html-active = HTML d’origine affiché : scripts désactivés, ressources contrôlées
privacy-original-html-offer = Conserver la mise en page d’origine du message pour ce message uniquement.
privacy-tracker-choice = Autoriser un choix de traceurs message par message
privacy-tracker-choice-help = Propose une action explicite pour charger les traceurs bloqués pour un seul message. Les traceurs restent bloqués tant que vous ne le choisissez pas, et un administrateur peut imposer la protection.
privacy-image-retry-attempts = Tentatives automatiques de rechargement des images
privacy-image-retry-help = Ne réessaie que les échecs temporaires d’images, une fois le chargement des images en cours terminé. Zéro désactive les nouvelles tentatives automatiques ; vous pouvez toujours réessayer manuellement. Les verrous de l’administrateur prévalent.
privacy-image-retry-delay = Délai avant nouvelle tentative pour les images (secondes)
privacy-concurrency-title = Capacité d’images adaptative
privacy-concurrency-mode-auto = Automatique
privacy-concurrency-mode-conservative = Faible consommation
privacy-concurrency-mode-balanced = Équilibré
privacy-concurrency-mode-throughput = Débit élevé
privacy-concurrency-mode-manual = Manuel
privacy-concurrency-apply = Appliquer le profil
privacy-concurrency-saved = Profil de concurrence des images enregistré.
privacy-concurrency-unavailable = Indisponible
privacy-concurrency-sizing = Cœurs de processeur : { $cpu } · Limite de mémoire : { $limit } · Mémoire disponible : { $available }
privacy-concurrency-running = Profil actif : { $mode } · Dimensionnement adaptatif : { $state }
privacy-concurrency-cap-perMessage = Par message
privacy-concurrency-cap-perAccount = Par compte
privacy-concurrency-cap-global = Réseau global
privacy-concurrency-cap-pipeline = Pipeline de traitement
privacy-concurrency-cap-decoders = Décodeurs
privacy-concurrency-effective-ceiling = { $effective } effectifs / { $ceiling } plafond
privacy-concurrency-message-bound = La valeur par message est une borne supérieure de l’instance. Chaque message utilise au plus cette borne ou son nombre d’images approuvées, selon la plus petite des deux.
privacy-concurrency-active = Actifs : { $network } réseau · { $pipeline } pipelines · { $decoders } décodeurs · { $queued } en file d’attente
privacy-image-rate-readout = Quota de récupération par compte ({ $mode }) : rafale { $burst } (configuré { $configuredBurst }), recharge { $perMin } par minute (configuré { $configuredPerMin })
privacy-image-cache-readout = Cache d’images ({ $mode }) : jusqu’à { $bytes } et { $entries } entrées, durée de vie { $ttl } s, taux de réussite { $hit }
privacy-image-hit-unknown = pas encore assez de recherches
privacy-tab-overview = Vue d’ensemble
privacy-tab-remote = Contenu distant et traceurs
privacy-tab-image-proxy = Proxy d’images
privacy-tab-fetch-limits = Limites de récupération
privacy-tab-traffic = Trafic
privacy-tab-cookies = Cookies
privacy-tab-denied = Vous n’avez pas accès à cet onglet.
privacy-tab-remote-help = Valeurs par défaut, verrous et valeurs par domaine pour le contenu distant, la protection contre le suivi et le nettoyage des liens. Chaque réglage s’enregistre séparément.
privacy-settings-denied = La lecture des paramètres nécessite l’autorisation settings.read, que vous n’avez pas. Les statistiques des autres onglets restent disponibles.
privacy-overview-status = État
privacy-overview-figures = Chiffres clés
privacy-status-ok = Le proxy d’images transmet ses données.
privacy-status-unavailable = Les statistiques du proxy d’images n’ont pas pu être lues.
privacy-history-title = Proxy d’images dans le temps
privacy-history-worker-title = Historique des workers
privacy-history-cache-title = Cache et limites
privacy-history-traffic-title = Trafic sortant
privacy-history-empty = Aucun échantillon sur cette période pour le moment.
privacy-history-unavailable = L’historique n’est pas disponible sur ce serveur.
privacy-history-summary = { $series } : dernière valeur { $latest }, maximum { $max }.
privacy-history-range = Période
privacy-history-range-15m = 15 minutes
privacy-history-range-1h = 1 heure
privacy-history-range-6h = 6 heures
privacy-history-range-24h = 24 heures
privacy-history-updated = Actualisé { $time }
privacy-history-not-updated = Pas encore actualisé
privacy-history-memory-note = Le serveur conserve un échantillon toutes les { $interval } secondes pendant 24 heures, en mémoire uniquement. L’historique est vide après un redémarrage et n’est pas partagé entre les processus serveur.
privacy-chart-requests = Images servies
privacy-chart-requests-unit = Requêtes par seconde
privacy-chart-bytes = Données servies
privacy-chart-bytes-unit = Octets par seconde
privacy-chart-hit = Taux de réussite du cache
privacy-chart-hit-unit = Pourcentage des recherches
privacy-chart-ram = Mémoire
privacy-chart-disk = Disque
privacy-chart-queue = Récupérations en cours et en attente
privacy-chart-queue-unit = Nombre
privacy-chart-active = En cours
privacy-chart-queued = En attente
privacy-chart-busy = Réponses « occupé »
privacy-chart-busy-unit = Réponses par intervalle
privacy-chart-trackers = Pixels espions remplacés
privacy-chart-trackers-unit = Nombre par intervalle
privacy-chart-size = Taille du cache
privacy-chart-size-unit = Octets conservés
privacy-chart-entries = Entrées du cache
privacy-chart-entries-unit = Nombre
privacy-chart-wait = Attente de limitation de débit
privacy-chart-wait-unit = Millisecondes moyennes par requête en attente
privacy-chart-rate = Budget de limitation de débit
privacy-chart-rate-unit = Quota de rafale et comptes actifs
privacy-chart-burst = Quota de rafale
privacy-chart-accounts = Comptes actifs
privacy-chart-allowed = Requêtes autorisées
privacy-chart-refused = Requêtes refusées et en échec
privacy-chart-traffic-unit = Nombre par intervalle
privacy-image-group-rate = Débit des requêtes
privacy-image-group-cache = Cache
privacy-image-group-ram = Niveau mémoire
privacy-image-group-disk = Niveau disque
privacy-image-group-tiering = Hiérarchisation et entretien

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-error-reports-help = Lorsqu'une partie de Mailwoman tombe en panne, le navigateur peut envoyer à votre administrateur un bref rapport technique afin que le problème puisse être corrigé. Il contient le type d'erreur, l'endroit où cela s'est produit dans l'application, la version de l'application et le type de navigateur. Il ne contient jamais le contenu des messages, les adresses, les noms des pièces jointes ou ce que vous avez tapé.

privacy-error-reports-send = Envoyer des rapports d'erreurs depuis ce navigateur

privacy-error-reports-send-help = Activé par défaut. Désactivez-la pour conserver ces rapports sur votre appareil. Le panneau d'erreur affiche toujours une référence que vous pouvez copier.

privacy-error-reports-title = Rapports d'erreurs

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-links-revoke-domain = Arrêtez de toujours autoriser les liens provenant de ce domaine d'expéditeur

privacy-links-trust-domain = Toujours autoriser les liens provenant de ce domaine d'expéditeur

privacy-trusted-link-sender-domains = Domaines d'expéditeur

privacy-trusted-link-sender-domains-help = Autoriser les liens examinés dans les messages provenant de ces domaines d'expéditeur précis. Un domaine par ligne ; les sous-domaines sont séparés. Cela ne fait pas confiance aux sites Web de destination et ne charge pas les images. Une liste vide demande chaque message.

privacy-trusted-link-sender-domains-save = Enregistrer les domaines d'expéditeurs approuvés

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-feedback-image-host = Préférence d’hôte d’image enregistrée.

privacy-feedback-images-off = Autorisation d'image supprimée.

privacy-feedback-images-on = Autorisation d'image enregistrée. Les images autorisées peuvent désormais se charger.

privacy-feedback-link-domain-off = Approbation du lien de domaine supprimée pour { $address }.

privacy-feedback-link-domain-on = Les liens seront autorisés dans les messages du domaine { $address }.

privacy-feedback-link-sender-off = Confiance du lien de l'expéditeur supprimée pour { $address }.

privacy-feedback-link-sender-on = Les liens seront autorisés dans les messages de { $address }.

privacy-feedback-links-off = Liens désactivés pour ce message.

privacy-feedback-links-on = Liens activés pour ce message.
