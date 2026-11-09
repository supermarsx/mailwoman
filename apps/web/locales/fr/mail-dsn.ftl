# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Rapport d’état de distribution
dsn-unverified = Ce rapport décrit la tentative de distribution de l’expéditeur. Ses affirmations ne sont pas vérifiées de manière indépendante. Une distribution ne signifie pas que le message a été lu.
dsn-reporting-mta = Serveur déclarant :
dsn-original-recipient = Destinataire d’origine :
dsn-remote-mta = Serveur distant :
dsn-last-attempt = Dernière tentative :
dsn-retry-until = L’expéditeur prévoit de réessayer jusqu’au :
dsn-action-failed = Échec signalé
dsn-action-delayed = Retard signalé
dsn-action-delivered = Distribution signalée
dsn-action-relayed = Relais signalé
dsn-action-expanded = Expansion de liste signalée

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Utilisez le diagnostic du serveur lorsque vous demandez de l'aide à votre fournisseur de messagerie.

dsn-advice-1 = Vérifiez les adresses de l’expéditeur et du destinataire. Confirmez l'adresse actuelle du destinataire si nécessaire.

dsn-advice-2 = Contactez le destinataire d'une autre manière si le problème de boîte aux lettres persiste.

dsn-advice-3 = Partagez ce rapport avec l'administrateur du système de messagerie destinataire si le problème persiste.

dsn-advice-4 = Demandez à votre administrateur de messagerie de vérifier la connectivité, DNS et les itinéraires de transfert si le problème persiste.

dsn-advice-5 = Partagez le diagnostic avec votre fournisseur de messagerie afin qu'il puisse vérifier la compatibilité et les limites du SMTP.

dsn-advice-6 = Vérifiez le format du message, les adresses internationales et les pièces jointes par rapport aux exigences du destinataire.

dsn-advice-7 = Partagez le diagnostic avec votre administrateur de messagerie pour identifier la règle exacte de sécurité ou d'authentification.

dsn-advice-success = Vérifiez l'action de remise signalée ci-dessus pour savoir si le message a été remis, relayé ou étendu à une liste.

dsn-category-0 = Le serveur a fourni un code détaillé non reconnu ou spécifique au fournisseur. Son diagnostic ci-dessous peut expliquer la cause exacte.

dsn-category-1 = Il s'agit d'un code lié à l'adresse. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus.

dsn-category-2 = Il s'agit d'un code de boîte aux lettres du destinataire. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus.

dsn-category-3 = Il s'agit d'un code de système de réception de courrier. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus.

dsn-category-4 = Il s'agit d'un code de réseau ou de routage. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus.

dsn-category-5 = Il s'agit d'un code de protocole de messagerie. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus.

dsn-category-6 = Il s'agit d'un contenu de message ou d'un code de format. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus.

dsn-category-7 = Il s'agit d'un code de sécurité ou de politique. Les détails précis sont spécifiques au fournisseur ou ne sont pas reconnus ; cela n'établit pas en soi que le message est du spam.

dsn-class-2 = Succès : le serveur signale une étape de livraison positive. Cela ne prouve pas que le message a été lu.

dsn-class-4 = Échec temporaire : une tentative ultérieure peut réussir. Un rapport de retard peut toujours être réessayé par l'expéditeur ; un rapport d'échec signifie que la tentative est terminée.

dsn-class-5 = Échec permanent : il est peu probable que renvoyer le même message inchangé ne fonctionne pas. Corrigez la cause avant de renvoyer.

dsn-meaning-title = Que signifie ce code

dsn-next-step = Que faire

dsn-provider-barracuda = Barracudas

dsn-provider-detected = Interprétation basée sur le diagnostic du serveur ou le nom d'hôte :

dsn-provider-exchange = Microsoft Exchange/Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmail/Google Workspace

dsn-provider-help = Documentation du fournisseur

dsn-provider-mimecast = Mimecast

dsn-provider-proofpoint = Point de preuve

dsn-provider-relay = Relais SMTP

dsn-provider-yahoo = Yahoo/AOL

dsn-server-diagnostic = Diagnostic du serveur

dsn-status-0-0 = Le serveur n'a pas donné de raison plus précise.

dsn-status-1-0 = Il y a un problème avec une adresse e-mail.

dsn-status-1-1 = La boîte aux lettres du destinataire n'existe pas. Confirmez l'adresse avec le destinataire.

dsn-status-1-10 = Le domaine destinataire déclare explicitement qu'il n'accepte aucun email (null MX). Utilisez une autre adresse.

dsn-status-1-2 = Le domaine du destinataire ne peut pas recevoir ce courrier. Vérifiez la partie après @.

dsn-status-1-3 = L'adresse du destinataire a un format non valide. Vérifiez les fautes de frappe ou les espaces supplémentaires.

dsn-status-1-4 = Le serveur ne peut pas identifier un destinataire unique pour cette adresse.

dsn-status-1-5 = L'adresse du destinataire a été acceptée comme valide.

dsn-status-1-6 = Cette boîte aux lettres a été déplacée et aucune adresse de transfert n'est disponible. Demandez la nouvelle adresse.

dsn-status-1-7 = L'adresse de l'expéditeur a un format invalide. Vérifiez l'adresse d'envoi du compte.

dsn-status-1-8 = Le domaine de l'expéditeur n'est pas valide ou ne peut pas recevoir de courrier en retour.

dsn-status-1-9 = Le message a été transmis à un système qui ne peut pas fournir d'autres rapports de livraison.

dsn-status-2-0 = La boîte aux lettres du destinataire a un problème non spécifié.

dsn-status-2-1 = La boîte aux lettres du destinataire est désactivée ou n'accepte pas les messages. Contactez le destinataire d'une autre manière.

dsn-status-2-2 = La boîte aux lettres du destinataire est pleine. Le destinataire doit libérer de l'espace ou augmenter son quota.

dsn-status-2-3 = Le message dépasse la taille limite de ce destinataire. Réduisez les pièces jointes ou utilisez un lien de partage.

dsn-status-2-4 = Le serveur n'a pas pu résoudre les membres d'une liste de diffusion. Contactez le propriétaire de la liste.

dsn-status-3-0 = Le système de réception du courrier a signalé un problème non spécifié.

dsn-status-3-1 = Le serveur de messagerie de réception n'a plus de stockage. Son administrateur doit libérer de l'espace.

dsn-status-3-2 = Le serveur de réception n'accepte pas le courrier, par exemple lors d'une maintenance ou d'une surcharge.

dsn-status-3-3 = Le système de réception ne prend pas en charge une fonctionnalité requise par ce message.

dsn-status-3-4 = Le message dépasse la taille limite du serveur de réception. Réduisez les pièces jointes ou utilisez un lien de partage.

dsn-status-3-5 = Un problème de configuration du serveur de messagerie empêche la livraison. Demandez à l'administrateur de messagerie d'enquêter.

dsn-status-3-6 = Le serveur a accepté le message mais a modifié la priorité de livraison demandée.

dsn-status-4-0 = Un problème de réseau ou de routage de courrier a été signalé sans plus de détails.

dsn-status-4-1 = Le serveur de réception n'a pas répondu à la tentative de connexion.

dsn-status-4-2 = La connexion a été établie mais interrompue avant la fin du transfert de courrier.

dsn-status-4-3 = Une recherche DNS ou dans le répertoire a échoué. Un administrateur de messagerie devra peut-être vérifier la résolution de nom.

dsn-status-4-4 = Le serveur n'a pas pu trouver un itinéraire vers le système de messagerie du destinataire.

dsn-status-4-5 = Les embouteillages du courrier ont empêché la livraison.

dsn-status-4-6 = Une boucle de transfert a été détectée. Demandez à l'administrateur de vérifier les règles de transfert et de routage.

dsn-status-4-7 = Le délai de livraison a expiré. Vérifiez le diagnostic du problème sous-jacent avant de renvoyer.

dsn-status-5-0 = Les serveurs de messagerie n'ont pas pu terminer leur échange de protocole de livraison.

dsn-status-5-1 = Un serveur de messagerie a rejeté une commande non prise en charge ou une commande envoyée dans le mauvais ordre.

dsn-status-5-2 = Un serveur de messagerie n'a pas pu comprendre la syntaxe d'une commande.

dsn-status-5-3 = Le message a trop de destinataires. Envoyer à des groupes plus petits.

dsn-status-5-4 = Un serveur de messagerie a rejeté les paramètres d'une commande.

dsn-status-5-5 = Les serveurs utilisent des versions incompatibles du protocole de messagerie.

dsn-status-5-6 = La réponse d'authentification était trop longue pour le serveur. Demandez à l'administrateur de messagerie de vérifier les paramètres d'authentification.

dsn-status-6-0 = Le serveur a signalé un problème non spécifié concernant le contenu ou le format du message.

dsn-status-6-1 = Le système de réception ne prend pas en charge le type de contenu de ce message.

dsn-status-6-10 = Ce code obsolète signifie qu'une réponse internationale (UTF-8) est requise mais non prise en charge, comme avec le code x.6.8.

dsn-status-6-2 = La livraison nécessite une conversion de contenu interdite par la politique de l'expéditeur ou du serveur.

dsn-status-6-3 = La livraison nécessite une conversion de contenu que le serveur ne peut pas effectuer.

dsn-status-6-4 = Une conversion de contenu a perdu certaines informations. Vérifiez le message d'origine et le diagnostic du serveur.

dsn-status-6-5 = Le serveur a échoué lors de la conversion du contenu du message.

dsn-status-6-6 = Le serveur n'a pas pu récupérer le contenu du message depuis un autre système.

dsn-status-6-7 = Une adresse e-mail internationale n'est pas prise en charge pour cet expéditeur ou destinataire. Demandez une autre adresse.

dsn-status-6-8 = Le serveur a besoin d'une réponse internationale (UTF-8) que le système expéditeur ne peut pas accepter.

dsn-status-6-9 = Le message contient des en-têtes internationaux (UTF-8) qui ne peuvent pas être remis à au moins un destinataire.

dsn-status-7-0 = Une règle de sécurité ou une politique de messagerie a empêché la livraison normale. Vérifiez le diagnostic du serveur pour connaître la règle exacte.

dsn-status-7-1 = Le serveur a refusé l'autorisation de livraison, probablement en raison de restrictions de filtrage ou de relais. Demandez à l'administrateur de messagerie de vérifier le diagnostic.

dsn-status-7-10 = Le serveur nécessite une connexion cryptée ou une protection plus forte avant l'authentification.

dsn-status-7-11 = Cette méthode d'authentification nécessite une connexion cryptée SMTP (TLS).

dsn-status-7-12 = Le compte nécessite une méthode d'authentification ou une migration de mot de passe. Contactez le fournisseur de compte.

dsn-status-7-13 = Le compte d'envoi est désactivé. Contactez son administrateur pour restaurer l'accès.

dsn-status-7-14 = Le serveur a besoin d'une relation de confiance configurée pour récupérer le message. Contactez son administrateur.

dsn-status-7-15 = La priorité du message est inférieure au minimum actuel du serveur de réception.

dsn-status-7-16 = Le message est trop volumineux pour la priorité demandée. Réduisez sa taille ou demandez à l'administrateur.

dsn-status-7-17 = La boîte aux lettres du destinataire a changé de propriétaire depuis la date requise. Confirmez le destinataire avant de renvoyer.

dsn-status-7-18 = Le domaine destinataire a changé de propriétaire depuis la date requise. Confirmez la destination avant d'envoyer à nouveau.

dsn-status-7-19 = Le serveur n'a pas pu vérifier la propriété continue de l'adresse du destinataire. Renseignez-vous auprès de l'administrateur sur le contrôle de validité du destinataire.

dsn-status-7-2 = L'expéditeur n'est pas autorisé à publier sur cette liste de diffusion. Contactez le propriétaire de la liste.

dsn-status-7-20 = Aucune signature DKIM n'a réussi la vérification. Demandez à l'administrateur de messagerie de l'expéditeur de vérifier la signature et DNS.

dsn-status-7-21 = Une signature DKIM a été transmise, mais la politique du serveur de réception ne l'a pas acceptée.

dsn-status-7-22 = Aucune signature DKIM acceptée ne correspond à l'adresse d'expédition. Demandez à l'administrateur de messagerie de vérifier l'alignement du domaine de signature.

dsn-status-7-23 = SPF a échoué : le serveur expéditeur n'a pas été autorisé par la politique SPF du domaine expéditeur.

dsn-status-7-24 = Le serveur a rencontré une erreur lors de la vérification de SPF. Demandez à l'administrateur de messagerie de vérifier les enregistrements SPF et DNS.

dsn-status-7-25 = Le serveur d'envoi a échoué à la vérification inversée DNS du destinataire. Son administrateur doit vérifier les enregistrements DNS de l'adresse IP.

dsn-status-7-26 = Plusieurs vérifications d’authentification des messages ont échoué. Le diagnostic peut identifier des problèmes de politique SPF, DKIM ou DMARC.

dsn-status-7-27 = Le domaine de l'expéditeur déclare qu'il n'accepte aucun email de retour (null MX), ce que le destinataire rejette.

dsn-status-7-28 = Le système de réception a détecté un éventuel flot de messages abusifs.

dsn-status-7-29 = La vérification ARC a échoué, le destinataire n'a donc pas pu accepter la chaîne d'authentification enregistrée du message.

dsn-status-7-3 = La livraison nécessite une conversion au format de sécurité que le serveur ne peut pas effectuer.

dsn-status-7-30 = Le message nécessite REQUIRETLS, mais le serveur de messagerie suivant ne le prend pas en charge. Demandez à l'administrateur de vérifier la prise en charge de la livraison sécurisée.

dsn-status-7-4 = Le système de livraison ne prend pas en charge une fonctionnalité de sécurité requise.

dsn-status-7-5 = Le serveur n'a pas pu valider ou déchiffrer le message, par exemple parce qu'une clé était manquante ou invalide.

dsn-status-7-6 = Le serveur ne prend pas en charge l'algorithme cryptographique requis par le message.

dsn-status-7-7 = Un contrôle d'intégrité indique que le message a été modifié ou endommagé.

dsn-status-7-8 = Le serveur d'envoi a rejeté les informations de connexion. Vérifiez les paramètres d'authentification du compte.

dsn-status-7-9 = La méthode d'authentification est plus faible que ce que le serveur permet. Une méthode plus forte est nécessaire.

dsn-vendor-advice-account = Demandez à l'administrateur expéditeur d'enquêter sur un éventuel abus de compte, de sécuriser le compte et de demander la suppression de la restriction.

dsn-vendor-advice-admin = Partagez ce diagnostic avec l'administrateur de messagerie de l'organisation pour examiner la configuration ou la stratégie correspondante.

dsn-vendor-advice-auth = Vérifiez la méthode d'authentification et l'autorisation SMTP prises en charge par le compte. Complétez toute connexion requise via la page du compte officiel du fournisseur.

dsn-vendor-advice-authentication = Demandez à l'administrateur de l'expéditeur de vérifier SPF, DKIM, DMARC et l'alignement du domaine à l'aide du diagnostic exact.

dsn-vendor-advice-connector = Demandez à l'administrateur de vérifier l'autorisation de relais, la correspondance du connecteur IP/certificat, les domaines acceptés et l'itinéraire réel du courrier.

dsn-vendor-advice-content = Examinez les journaux de diagnostic et de passerelle avec l'administrateur du destinataire pour identifier le contenu bloqué et la stratégie applicable.

dsn-vendor-advice-dns = Demandez à l'administrateur de courrier expéditeur ou au fournisseur IP de corriger DNS, d'inverser DNS et l'identité du serveur SMTP.

dsn-vendor-advice-domain = Demandez à l'administrateur Microsoft 365 d'enregistrer et de vérifier le domaine d'envoi et de revoir les limites d'envoi du locataire.

dsn-vendor-advice-format = Demandez à l'application d'envoi ou à l'administrateur de messagerie de corriger la génération du message, puis créez un nouveau message.

dsn-vendor-advice-rate = Réduisez le volume d’envoi et attendez que le quota ou la restriction soit supprimé. Vérifiez l'action signalée avant de supposer que cette tentative est toujours en file d'attente.

dsn-vendor-advice-recipient-admin = Contactez le destinataire ou le propriétaire du groupe d'une autre manière et demandez à son administrateur de consulter l'autorisation ou la politique appropriée.

dsn-vendor-advice-reputation = Demandez à l'administrateur expéditeur d'enquêter sur le trafic indésirable et la réputation, puis suivez le processus officiel d'examen ou de suppression du fournisseur.

dsn-vendor-advice-route = Demandez à l'administrateur de messagerie de vérifier la destination, les règles de transfert, le service de messagerie et le chemin pare-feu/réseau.

dsn-vendor-advice-tls = Demandez aux administrateurs de messagerie de réparer les enregistrements de sécurité TLS, certificats ou DNS afin que la route sécurisée requise puisse réussir.

dsn-vendor-barracuda-reputation = Le service de réputation de Barracuda répertorie ou bloque l'adresse d'envoi IP.

dsn-vendor-certificate-chain = Le certificat TLS distant n'a pas pu être validé par rapport à la confiance requise ou à la stratégie MTA-STS.

dsn-vendor-certificate-expired = Le serveur de messagerie de destination a présenté un certificat TLS expiré.

dsn-vendor-certificate-host = Le certificat de destination TLS ne correspond pas au nom d'hôte du serveur de messagerie requis.

dsn-vendor-connection-refused = Le serveur de destination a refusé la connexion réseau. Son service de messagerie, son pare-feu ou son itinéraire réseau doivent être vérifiés.

dsn-vendor-content-security = La passerelle a rejeté le contenu du message ou une pièce jointe selon ses règles de filtrage de sécurité. Consultez son diagnostic ou ses journaux pour l'élément spécifique.

dsn-vendor-credentials = Le service SMTP a rejeté les informations de connexion fournies.

dsn-vendor-dane = Le certificat TLS ou la configuration TLSA de la destination n'a pas satisfait à la validation DANE.

dsn-vendor-dkim = Le destinataire n'a pas accepté l'authentification DKIM ou la configuration de signature du message.

dsn-vendor-dmarc = La politique DMARC du destinataire n'a pas été respectée. Vérifiez l'authentification SPF ou DKIM et l'alignement avec le domaine visible From.

dsn-vendor-dnssec = Les enregistrements DNSSEC du domaine de destination n'ont pas pu être validés.

dsn-vendor-exchange-auth = L’application ou l’appareil ne s’est pas authentifié correctement pour la soumission Microsoft 365 SMTP.

dsn-vendor-exchange-connector = Microsoft n'a pas pu autoriser le relais via le connecteur locataire configuré. Son certificat, envoyant IP, ou son domaine accepté peut ne plus correspondre.

dsn-vendor-exchange-domain-blocked = Microsoft a restreint ce domaine d'envoi après avoir détecté une activité suspectée de spam.

dsn-vendor-exchange-forward-relay = Un système en aval a rejeté le courrier transféré ou relayé. L’authentification peut avoir été affectée par le transfert ou par une passerelle non Microsoft.

dsn-vendor-exchange-forwarding = La stratégie Microsoft 365 bloque le transfert automatique vers des destinataires externes. L'administrateur de l'organisation doit revoir la politique de transfert.

dsn-vendor-exchange-generic = Exchange a signalé un refus général de livraison. Le diagnostic identifie le problème réel, qui peut impliquer le formatage du message ou la stratégie des pièces jointes.

dsn-vendor-exchange-group-allow = L’expéditeur ne figure pas dans la liste des expéditeurs autorisés de ce groupe Exchange.

dsn-vendor-exchange-group-internal = Ce groupe de distribution Exchange rejette les expéditeurs en dehors de son organisation, sauf si son propriétaire ou son administrateur les autorise.

dsn-vendor-exchange-internal-only = Ce destinataire Exchange accepte les expéditeurs authentifiés de sa propre organisation ; ce message a été traité comme externe ou non authentifié.

dsn-vendor-exchange-ipv6 = La destination n'accepte pas cette livraison au-dessus de IPv6. L'administrateur expéditeur doit vérifier l'itinéraire ou utiliser un moyen de transport accepté.

dsn-vendor-exchange-journaling = L'organisation tente de journaliser les messages sur site dans Microsoft 365 sans la configuration d'archive requise.

dsn-vendor-exchange-onmicrosoft-limit = L'organisation a atteint la limite de destinataires externes pour son domaine d'envoi onmicrosoft.com. L'envoi régulier doit utiliser un domaine personnalisé vérifié.

dsn-vendor-exchange-public-folder = Ce dossier public Exchange n’accepte pas cet expéditeur externe ou non authentifié.

dsn-vendor-exchange-receive-rate = Ce destinataire Exchange a atteint la limite horaire de réception pour tous les expéditeurs.

dsn-vendor-exchange-recipient = Exchange n'a pas pu résoudre ce destinataire dans son annuaire. Vérifiez l'adresse et supprimez toute entrée de saisie semi-automatique obsolète avant de créer un nouveau message.

dsn-vendor-exchange-recipient-access = Exchange a refusé l’accès à ce destinataire ou domaine de destination. Vérifiez l'adresse du destinataire et la configuration du domaine accepté de l'organisation.

dsn-vendor-exchange-recipient-rate = Exchange a reçu trop de messages de cet expéditeur à ce destinataire au cours de sa fenêtre horaire.

dsn-vendor-exchange-sender-blocked = Microsoft a restreint ce compte d'envoi après avoir détecté des soupçons de spam ou d'abus. Cela peut nécessiter qu'un administrateur sécurise et débloque le compte.

dsn-vendor-exchange-temporary-restriction = Microsoft restreint temporairement cet envoi de IP ou de trafic tout en évaluant les activités suspectes.

dsn-vendor-exchange-tenant-blocked = Microsoft a restreint l'envoi pour cette organisation car son trafic était considéré comme suspect.

dsn-vendor-exchange-tenant-blocklist = Une adresse ou un domaine de destinataire est bloqué par la liste des locataires autorisés/bloqués de l’organisation émettrice. Une destination bloquée peut rejeter l'intégralité du message.

dsn-vendor-exchange-tenant-limit = L'organisation Microsoft 365 a atteint sa limite quotidienne glissante pour les destinataires externes.

dsn-vendor-exchange-unregistered = Microsoft a rejeté l'envoi à partir d'un domaine qui n'est pas correctement enregistré et vérifié dans le locataire.

dsn-vendor-expired = Le service d'envoi a abandonné après l'expiration de son délai de livraison. Vérifiez le diagnostic de l'échec de connexion ou de routage à l'origine du retard.

dsn-vendor-fortimail-relay = FortiMail a refusé cette livraison en vertu de sa politique de contrôle d'accès ou de relais. L'administrateur de la passerelle doit vérifier la règle de réception correspondante.

dsn-vendor-gmail-app-password = Google nécessite un mot de passe d'application ou une méthode d'autorisation de compte prise en charge pour cette connexion SMTP.

dsn-vendor-gmail-auth-method = Google a rejeté une méthode d'authentification non prise en charge, telle que la méthode retirée XOAUTH.

dsn-vendor-gmail-authentication = Google n'a pas accepté l'authentification de l'expéditeur. Le diagnostic distingue les échecs matériels SPF/DKIM manquants, SPF et les rejets DMARC.

dsn-vendor-gmail-browser-login = Google nécessite une connexion au compte ou un contrôle de sécurité dans le navigateur avant d'autoriser cette connexion SMTP.

dsn-vendor-gmail-custom-policy = La règle de messagerie personnalisée d'un administrateur Google Workspace a rejeté le message. Cela ne signifie pas nécessairement que le filtre anti-spam de Google l'a rejeté.

dsn-vendor-gmail-dmarc-record = Google exige un enregistrement DMARC avec une politique déclarée pour ce domaine d'envoi.

dsn-vendor-gmail-rate = Google a appliqué une limite d'envoi ou de relais, ou détecté un débit de messages inhabituel. Le diagnostic identifie le compte concerné, IP, ou le domaine.

dsn-vendor-gmail-relay = Google Workspace n'a pas autorisé l'envoi de IP, de domaine ou de compte pour son service de relais SMTP.

dsn-vendor-gmail-reputation = Le filtrage de Google a rejeté le message car il considérait le courrier non sollicité ou la réputation d'envoi IP/domaine trop faible.

dsn-vendor-gmail-spf-record = Google considère la configuration SPF du domaine de l'expéditeur comme invalide ou suspecte.

dsn-vendor-greylist = La passerelle a temporairement différé cette livraison en utilisant une liste grise. Un serveur d'envoi correctement configuré réessaye normalement plus tard.

dsn-vendor-headers = Le service de réception a rejeté les en-têtes de message non valides ou manquants. L'application d'envoi doit générer un courrier correctement formaté.

dsn-vendor-helo = Le serveur d'envoi a utilisé une identité HELO/EHLO inacceptable. Son message d'accueil doit identifier correctement le serveur expéditeur.

dsn-vendor-ip-block = Le service de réception a rejeté l'envoi IP en raison de sa réputation ou d'une entrée dans la liste de blocage.

dsn-vendor-mimecast-block = Mimecast a rejeté l'expéditeur ou le domaine en vertu d'une politique d'expéditeur bloqué ou d'administration.

dsn-vendor-mimecast-content = Une politique d'examen du contenu Mimecast a rejeté le message.

dsn-vendor-mimecast-spoof = La politique anti-usurpation d'identité de Mimecast a rejeté l'identité de l'expéditeur.

dsn-vendor-mta-sts-host = Le serveur de messagerie de destination n'est pas autorisé par la politique MTA-STS du domaine.

dsn-vendor-proofpoint-block = Proofpoint a bloqué l'envoi du IP ou du domaine après avoir détecté du spam ou une activité malveillante.

dsn-vendor-proofpoint-throttle = Proofpoint a temporairement différé cet envoi IP car son modèle d'envoi était considéré comme inhabituel.

dsn-vendor-ptr = L'envoi IP n'a pas d'inverse acceptable DNS, ou son nom PTR ne revient pas au même IP.

dsn-vendor-recipient-ip-block = L'organisation destinataire bloque explicitement l'adresse d'envoi IP. Son administrateur contrôle ce bloc local.

dsn-vendor-relay = Le serveur de messagerie ne relayera pas ce message vers la destination. Vérifiez l'authentification SMTP, les adresses IP d'envoi approuvées, les connecteurs et les domaines acceptés.

dsn-vendor-routing-loop = Le message est entré dans une boucle de transfert ou de routage et a dépassé le nombre de sauts autorisé.

dsn-vendor-sending-limit = Le compte d'envoi a dépassé un quota de messages ou de destinataires. Les renvois immédiats répétés ne supprimeront pas la limite.

dsn-vendor-spf = Le destinataire n'a pas accepté l'authentification SPF pour le domaine d'envoi IP et l'expéditeur de l'enveloppe.

dsn-vendor-starttls = La connexion cryptée SMTP requise n'a pas pu être établie ou n'a pas été prise en charge par le serveur suivant.

dsn-vendor-yahoo-deferred = Yahoo a temporairement différé l'envoi de ce message, généralement lors de l'évaluation du volume, de la réputation ou des plaintes des destinataires. Vérifiez le diagnostic pour la cause indiquée.

dsn-vendor-yahoo-policy = Yahoo a rejeté le message conformément à sa politique de messagerie. L'authentification, l'alignement DMARC ou le contenu du message peuvent être impliqués ; le diagnostic donne la raison précise.
