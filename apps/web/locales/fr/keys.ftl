# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Gestion des clés
keys-title = Clés et certificats
keys-subtitle = Clés OpenPGP et S/MIME. Les clés privées restent sur cet appareil et ne parviennent jamais au serveur.
keys-generate = Générer une clé
keys-import = Importer une clé

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Vos clés
keys-empty-own = Aucune clé pour l’instant. Générez-en ou importez-en une pour commencer.
keys-contact-keys = Clés des contacts
keys-empty-contact = Aucune clé de contact. Recherchez-en une ci-dessous.
keys-loading = Chargement des clés…
keys-select-prompt = Sélectionnez une clé pour l’afficher et la vérifier.
keys-untitled = Clé { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = non vérifiée
keys-trust-tofu = tofu
keys-trust-verified = vérifiée
keys-trust-revoked = révoquée

# -- Key detail --------------------------------------------------------------
keys-key-card = Clé { $name }
keys-fingerprint = Empreinte
keys-safe-words = Mots de contrôle
keys-safe-words-help = Lisez-les à voix haute avec le contact pour confirmer la clé par un autre canal.
keys-safe-words-list = Mots de contrôle de l’empreinte
keys-scan-label = Scanner pour vérifier
keys-qr-label = QR code de l’empreinte
keys-autocrypt = Autocrypt
keys-autocrypt-on = Annoncée dans les en-têtes Autocrypt
keys-autocrypt-off = Non annoncée via Autocrypt
keys-trust-label = Confiance
keys-trust-level = Niveau de confiance
keys-associate-label = Associer à un contact
keys-associate-select = Contact à associer
keys-choose-contact = Choisir un contact…
keys-associate-button = Associer
keys-backup-label = Sauvegarde
keys-backup-help = Exportez un Autocrypt Setup Message pour transférer cette clé vers un autre appareil.
keys-export-backup = Exporter la sauvegarde
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Aucune clé privée détenue sur cet appareil à sauvegarder

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Certificat S/MIME
keys-smime-self-signed = Ce certificat est auto-signé : il a été signé par sa propre clé, et non par une autorité de certification. Les logiciels de messagerie des autres personnes ne lui feront pas confiance à moins qu’elles ne l’acceptent manuellement, ou jusqu’à ce qu’une autorité de certification émette un certificat à partir du fichier de demande ci-dessous.
keys-smime-valid-until = Valide jusqu’au { $date }.
keys-smime-not-in-app = Mailwoman ne signe ni ne déchiffre encore le courrier avec une clé S/MIME. Exportez la clé pour l’utiliser dans un autre logiciel de messagerie.
keys-smime-passphrase = Phrase secrète de cette clé S/MIME
keys-smime-export-help = Le fichier .p12 contient le certificat et la clé privée. La clé est chiffrée avec la phrase secrète ci-dessus, et la même phrase secrète ouvre le fichier dans un autre logiciel. Le fichier .csr ne contient aucune clé privée.
keys-smime-export = Exporter au format PKCS#12 (.p12)
keys-smime-request = Télécharger la demande de certificat (.csr)
keys-smime-issued-label = Certificat émis pour cette clé (.pem, .crt, .cer)
keys-smime-issued-file = Fichier du certificat émis
keys-smime-issued-help = Remplace le certificat détenu pour cette clé. Un certificat établi pour une autre clé est refusé.
keys-smime-attach = Importer le certificat émis
keys-smime-failed = Cela n’a pas fonctionné. Vérifiez la phrase secrète.
keys-smime-attach-failed = Le certificat n’a pas été importé. Vérifiez la phrase secrète, et que le fichier contient un seul certificat émis pour cette clé.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Rechercher la clé d’un contact
keys-lookup-heading = Rechercher une clé
keys-lookup-address = Adresse à rechercher
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Sources de recherche
keys-source = Source { $source }
keys-consent-label = Consentement à la recherche externe
keys-consent-text = La recherche d’une clé contacte des annuaires externes (WKD/VKS). J’ai le consentement de cette personne pour le faire.
keys-lookup-button = Rechercher
keys-looking-up = Recherche…
keys-lookup-found = { $count ->
    [one] { $count } clé trouvée — ajoutée aux clés des contacts.
   *[other] { $count } clés trouvées — ajoutées aux clés des contacts.
}
keys-lookup-none = Aucune clé trouvée.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Générer une clé
keys-type = Type
keys-key-type = Type de clé
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-generate-smime-note = Crée une clé RSA de 3072 bits et un certificat pour l’adresse ci-dessous, valable deux ans. Le certificat est auto-signé : les logiciels de messagerie des autres personnes ne lui feront pas confiance à moins qu’elles ne l’acceptent manuellement, ou jusqu’à ce qu’une autorité de certification en émette un. Mailwoman ne signe ni ne déchiffre encore le courrier avec ; vous pourrez ensuite l’exporter vers un autre logiciel de messagerie.
keys-name = Nom
keys-email = E-mail
keys-key-passphrase = Phrase secrète de la clé
keys-passphrase-help = La phrase secrète protège la clé privée sur cet appareil. Elle ne quitte jamais le navigateur.
keys-cancel = Annuler
keys-generate-submit = Générer
keys-generating = Génération…
keys-generating-smime = Génération d’une clé RSA dans ce navigateur. Cela peut prendre jusqu’à une minute.
keys-generate-failed = Aucune clé n’a été générée.
keys-generate-failed-smime = Aucune clé n’a été générée. Un certificat ne peut contenir qu’une adresse ASCII simple (nom@domaine) et un nom de 64 caractères au maximum.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Importer une clé
keys-import-type = Type d’importation
keys-tab-armored = Armure ASCII (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Clé en armure ASCII
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Phrase secrète (si la clé est privée)
keys-import-passphrase = Phrase secrète d’importation
keys-pkcs12-file-label = Fichier PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = Fichier PKCS#12
keys-import-password-label = Mot de passe d’importation
keys-pkcs12-password = Mot de passe PKCS#12
keys-import-preview = Aperçu de l’importation
keys-preview = Aperçu
keys-preview-type = Type : { $kind }
keys-preview-fingerprint-aria = Empreinte de l’aperçu
keys-preview-fingerprint = Empreinte : { $fp }
keys-preview-has-private = Inclut une clé privée (sera stockée sur cet appareil)
keys-preview-public-only = Clé publique uniquement
keys-import-submit = Importer
