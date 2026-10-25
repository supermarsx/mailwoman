# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Détails de sécurité du message
security-section-auth = Authentification
security-section-received = Chemin de remise
security-section-signature = Signature
security-section-attachments = Pièces jointes
security-section-warnings = Avertissements
security-section-controls = Contrôles de l’expéditeur
security-received-empty = Aucune chaîne Received disponible.
security-attachments-empty = Aucune pièce jointe.
security-hop-unknown = inconnu
security-unknown = inconnu

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = réussi
security-auth-fail = échoué
security-auth-none = absent
security-auth-neutral = neutre
security-auth-temperror = erreur temporaire
security-auth-permerror = erreur permanente

security-detail-domain = domaine
security-detail-selector = sélecteur
security-detail-policy = politique
security-detail-alignment = alignement
security-detail-chain-length = longueur de la chaîne
security-aligned = aligné
security-not-aligned = non aligné

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Signature vérifiée
security-sig-unverified-key = Signé — clé du signataire non vérifiée
security-sig-invalid = La signature n’est pas valide
security-sig-none = Non signé
security-chain-trusted = Chaîne de confiance
security-chain-untrusted = Chaîne non fiable
security-chain-expired = Chaîne expirée
security-chain-unknown = Chaîne inconnue
security-revocation-good = Non révoquée
security-revocation-revoked = Clé révoquée
security-revocation-unknown = Révocation inconnue
security-key-changed = La clé du signataire a changé depuis la dernière fois
security-fact-signer-key = Clé du signataire
security-fact-algorithm = Algorithme
security-fact-key-created = Clé créée le
security-fact-key-expires = Clé expire le
security-fact-chain = Chaîne
security-fact-revocation = Révocation
security-fact-key-change = Changement de clé

# -- Attachment risk --------------------------------------------------------
security-attach-none = Aucun risque connu
security-attach-macro = Contient des macros
security-attach-executable = Fichier exécutable
security-attach-encrypted-archive = Archive chiffrée
security-attach-double-extension = Double extension de fichier
security-attach-mismatch = type incohérent (déclaré { $declared }, détecté { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = L’adresse Reply-To diffère de celle de l’expéditeur
security-anomaly-envelopeFromDivergence = L’expéditeur de l’enveloppe diffère de l’adresse From
security-anomaly-messageIdDomainAnomaly = Le domaine du Message-ID ne correspond pas à l’expéditeur
security-anomaly-dateSkew = La date d’envoi semble décalée
security-anomaly-punycodeSender = L’expéditeur utilise du punycode (domaine sosie possible)

# -- Sender controls --------------------------------------------------------
security-control-block = Bloquer l’expéditeur
security-control-silence = Mettre l’expéditeur en sourdine
security-control-ignore-conversation = Ignorer la conversation
security-control-report-phishing = Signaler un hameçonnage
security-control-report-junk = Signaler comme indésirable
security-control-done-block = Expéditeur bloqué
security-control-done-silence = Expéditeur mis en sourdine
security-control-done-ignore-conversation = Conversation ignorée
security-control-done-report-phishing = Signalé comme hameçonnage
security-control-done-report-junk = Signalé comme indésirable

security-za-title = Stockage à accès zéro
security-za-on = Activé
security-za-off = Désactivé
security-za-protects = L’accès zéro chiffre votre courrier, vos notes et vos données PIM stockés, de sorte que le serveur d’hébergement ne conserve qu’un texte chiffré qu’il ne peut pas lire. Il protège le contenu de vos données stockées contre un opérateur curieux ou une intrusion sur l’hôte de stockage : quiconque lit la base de données, les fichiers sur disque ou une sauvegarde volée ne voit que du texte chiffré XChaCha20-Poly1305 et ne peut pas récupérer le corps des messages, les objets, les pièces jointes, le texte des notes ni l’index de recherche.

security-za-server-sees-title = Ce que le serveur voit encore
security-za-sees-1 = Les blocs chiffrés — les lignes chiffrées elles-mêmes.
security-za-sees-2 = Les identifiants de ligne opaques — les identifiants utilisés pour stocker et récupérer les lignes.
security-za-sees-3 = Les tailles — la longueur de chaque texte chiffré (taille approximative des messages et pièces jointes).
security-za-sees-4 = Les horodatages — le moment où les lignes sont écrites et mises à jour.
security-za-sees-5 = Les métadonnées de routage de l’enveloppe nécessaires pour relayer IMAP/SMTP — le serveur se connecte toujours à votre fournisseur de messagerie en votre nom ; les métadonnées de connexion et l’enveloppe de routage nécessaires pour envoyer et recevoir du courrier passent donc par lui.

security-za-active-server-caveat = L’accès zéro protège les données AU REPOS contre un hôte curieux ou compromis. Un serveur actif entièrement malveillant qui relaie votre trafic IMAP/SMTP en direct est un adversaire plus puissant, et l’accès zéro ne protège PAS contre lui : un tel serveur se trouve sur la connexion en direct avec votre fournisseur de messagerie et pourrait observer ou altérer le courrier qui transite, quelle que soit la façon dont la copie stockée est chiffrée. Choisissez l’accès zéro si votre préoccupation est « Je ne veux pas que celui qui gère (ou compromet) l’hôte de stockage lise mon courrier stocké. » Ce n’est pas une protection contre un opérateur qui détourne le chemin du courrier en direct.

security-za-no-search-claim = Il n’y a pas ici de chiffrement interrogeable côté serveur, et aucune affirmation de ce type n’est faite. La recherche s’exécute entièrement sur le client, sur le contenu qu’il a déchiffré localement ; le serveur ne détient jamais de forme interrogeable de votre texte en clair.

security-za-recovery-tradeoff = Vos clés sont dérivées et conservées uniquement sur vos appareils — le serveur ne reçoit jamais de clé en clair. Si vous perdez votre phrase secrète (ou votre clé d’accès) ET tous les appareils associés ET votre phrase de récupération, les données chiffrées ne peuvent pas être récupérées. Conservez la phrase de récupération hors ligne et considérez-la comme équivalente au secret maître de votre compte.

security-za-setup-key = Configurer votre clé
security-za-key-source = Source de la clé
security-za-passphrase = Phrase secrète
security-za-passkey = Clé d’accès (sans mot de passe)
security-za-passkey-unavailable = les clés d’accès ne sont pas disponibles dans ce navigateur
security-za-passphrase-label = Phrase secrète d’accès zéro
security-za-enable = Activer l’accès zéro
security-za-recovery-title = Phrase de récupération
security-za-recovery-heading = Phrase de récupération — conservez-la hors ligne maintenant
security-za-recovery-note = C’est la seule copie. Quiconque la possède peut lire vos données ; sans elle (et sans appareil associé), vos données ne peuvent pas être récupérées.
security-za-disable-section = Désactiver l’accès zéro
security-za-disable-heading = Désactiver l’accès zéro
security-za-disable-note = Les nouvelles données seront de nouveau stockées sans chiffrement. Les données chiffrées existantes restent lisibles uniquement tant que vous pouvez encore dériver votre clé.
security-za-disable-btn = Désactiver l’accès zéro
security-za-err-passphrase-len = utilisez une phrase secrète d’au moins 8 caractères
security-za-err-enable = impossible d’activer l’accès zéro
security-za-err-disable = impossible de désactiver l’accès zéro

security-pair-title = Association d’appareils
security-pair-heading = Associer un appareil
security-pair-intro = Transférez vos clés vers un autre appareil sans les faire passer par le serveur — il ne relaie qu’une enveloppe scellée opaque. Comparez les six mots sur les deux écrans avant de faire confiance à l’association.
security-pair-role = Rôle dans l’association
security-pair-new-role = Ceci est le nouvel appareil
security-pair-existing-role = Associer un autre appareil depuis celui-ci
security-pair-show-qr = Afficher le QR d’association
security-pair-scan = Scannez-le sur votre appareil existant
security-pair-qr-label = QR code d’association d’appareil
security-pair-qr-default = QR code d’association
security-pair-copy-code = Ou copiez ce code d’association
security-pair-paste-envelope = Collez l’enveloppe scellée provenant de l’autre appareil
security-pair-envelope-placeholder = enveloppe…
security-pair-envelope-label = Enveloppe scellée
security-pair-complete = Terminer l’association
security-pair-code-from-new = Code d’association du nouvel appareil
security-pair-code-placeholder = code d’association…
security-pair-code-label = Code d’association
security-pair-seal = Sceller mes clés pour cet appareil
security-pair-envelope-out = Enveloppe scellée — collez-la sur le nouvel appareil
security-pair-compare = Comparez ces mots sur les deux appareils
security-pair-sas-label = Chaîne d’authentification courte
security-pair-match = Les mots correspondent
security-pair-abort = Ils diffèrent — annuler
security-pair-confirmed = Association confirmée. Ce canal est authentifié.
security-pair-err-start = impossible de démarrer l’association
security-pair-err-complete = impossible de terminer l’association
security-pair-err-unlock = déverrouillez d’abord l’accès zéro pour associer un autre appareil
security-pair-err-seal = impossible de sceller la clé racine
