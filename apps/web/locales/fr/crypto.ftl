# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Chiffré de bout en bout
crypto-banner-e2ee-detail = Chaque destinataire possède une clé — le corps du message est chiffré sur cet appareil.
crypto-banner-tls-label = Chiffrement du transport (TLS)
crypto-banner-tls-detail = Aucune clé de chiffrement de destinataire n’a été trouvée ; la remise n’est protégée qu’en transit.
crypto-banner-mixed-label = Protection mixte
crypto-banner-mixed-detail = Certains destinataires peuvent recevoir un chiffrement de bout en bout ; les autres n’ont que TLS.
crypto-banner-checking-label = Vérification des clés des destinataires
crypto-banner-checking = · vérification…
crypto-banner-tls-only = TLS uniquement pour :

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Sécurité du message
crypto-encrypt-label = Chiffrer (de bout en bout)
crypto-encrypt-hint-default = Chiffrer le corps sur cet appareil avant l’envoi.
crypto-encrypt-drafted = Brouillon chiffré sur cet appareil.
crypto-encrypt-no-key = Aucune clé de chiffrement de destinataire disponible.
crypto-sign-label = Signer (prouver que c’est vous)
crypto-protect-subject = Chiffrer aussi l’objet
crypto-reason-add-recipient = Ajoutez un destinataire pour rechercher des clés de chiffrement.
crypto-reason-tls = Aucune clé de chiffrement de destinataire disponible — envoi via TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Avertissements de prévention des pertes de données
crypto-dlp-block = Envoi bloqué
crypto-dlp-require = Chiffrement requis
crypto-dlp-warn = Attention
crypto-dlp-notify = L’administrateur sera averti
crypto-dlp-matched = Correspondance : { $list }
