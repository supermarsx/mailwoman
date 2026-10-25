# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Schlüsselverwaltung
keys-title = Schlüssel & Zertifikate
keys-subtitle = OpenPGP- und S/MIME-Schlüssel. Private Schlüssel bleiben auf diesem Gerät und erreichen nie den Server.
keys-generate = Schlüssel erzeugen
keys-import = Schlüssel importieren

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Ihre Schlüssel
keys-empty-own = Noch keine Schlüssel. Erzeugen oder importieren Sie einen, um zu beginnen.
keys-contact-keys = Kontaktschlüssel
keys-empty-contact = Keine Kontaktschlüssel. Suchen Sie unten einen.
keys-loading = Schlüssel werden geladen…
keys-select-prompt = Wählen Sie einen Schlüssel aus, um ihn anzuzeigen und zu verifizieren.
keys-untitled = { $kind }-Schlüssel

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = nicht verifiziert
keys-trust-tofu = TOFU
keys-trust-verified = verifiziert
keys-trust-revoked = widerrufen

# -- Key detail --------------------------------------------------------------
keys-key-card = Schlüssel { $name }
keys-fingerprint = Fingerabdruck
keys-safe-words = Sicherheitswörter
keys-safe-words-help = Lesen Sie diese gemeinsam mit dem Kontakt laut vor, um den Schlüssel auf einem separaten Weg zu bestätigen.
keys-safe-words-list = Sicherheitswörter des Fingerabdrucks
keys-scan-label = Zum Verifizieren scannen
keys-qr-label = QR-Code des Fingerabdrucks
keys-autocrypt = Autocrypt
keys-autocrypt-on = In Autocrypt-Headern angekündigt
keys-autocrypt-off = Nicht über Autocrypt angekündigt
keys-trust-label = Vertrauen
keys-trust-level = Vertrauensstufe
keys-associate-label = Mit einem Kontakt verknüpfen
keys-associate-select = Zu verknüpfender Kontakt
keys-choose-contact = Kontakt auswählen…
keys-associate-button = Verknüpfen
keys-backup-label = Sicherung
keys-backup-help = Exportieren Sie eine Autocrypt Setup Message, um diesen Schlüssel auf ein anderes Gerät zu übertragen.
keys-export-backup = Sicherung exportieren
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Auf diesem Gerät ist kein privater Schlüssel für eine Sicherung vorhanden

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = S/MIME-Zertifikat
keys-smime-self-signed = Dieses Zertifikat ist selbstsigniert: Es wurde mit seinem eigenen Schlüssel signiert, nicht von einer Zertifizierungsstelle. Die E-Mail-Programme anderer Personen vertrauen ihm nur, wenn sie es manuell akzeptieren oder wenn eine Zertifizierungsstelle ein Zertifikat aus der unten stehenden Anforderungsdatei ausstellt.
keys-smime-valid-until = Gültig bis { $date }.
keys-smime-not-in-app = Mailwoman signiert oder entschlüsselt E-Mails noch nicht mit einem S/MIME-Schlüssel. Exportieren Sie den Schlüssel, um ihn in einem anderen E-Mail-Programm zu verwenden.
keys-smime-passphrase = Passphrase für diesen S/MIME-Schlüssel
keys-smime-export-help = Die .p12-Datei enthält das Zertifikat und den privaten Schlüssel. Der Schlüssel wird mit der obigen Passphrase verschlüsselt, und dieselbe Passphrase öffnet die Datei in einem anderen Programm. Die .csr-Datei enthält keinen privaten Schlüssel.
keys-smime-export = Als PKCS#12 (.p12) exportieren
keys-smime-request = Zertifikatsanforderung herunterladen (.csr)
keys-smime-issued-label = Für diesen Schlüssel ausgestelltes Zertifikat (.pem, .crt, .cer)
keys-smime-issued-file = Datei des ausgestellten Zertifikats
keys-smime-issued-help = Ersetzt das für diesen Schlüssel gespeicherte Zertifikat. Ein für einen anderen Schlüssel erstelltes Zertifikat wird abgelehnt.
keys-smime-attach = Ausgestelltes Zertifikat importieren
keys-smime-failed = Das hat nicht funktioniert. Prüfen Sie die Passphrase.
keys-smime-attach-failed = Das Zertifikat wurde nicht importiert. Prüfen Sie die Passphrase und ob die Datei genau ein für diesen Schlüssel ausgestelltes Zertifikat enthält.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Kontaktschlüssel suchen
keys-lookup-heading = Schlüssel suchen
keys-lookup-address = Zu suchende Adresse
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Suchquellen
keys-source = Quelle { $source }
keys-consent-label = Zustimmung zur externen Suche
keys-consent-text = Bei der Suche nach einem Schlüssel werden externe Verzeichnisse (WKD/VKS) kontaktiert. Ich habe die Zustimmung dieser Person dazu.
keys-lookup-button = Suchen
keys-looking-up = Wird gesucht…
keys-lookup-found = { $count } Schlüssel gefunden — zu den Kontaktschlüsseln hinzugefügt.
keys-lookup-none = Kein Schlüssel gefunden.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Schlüssel erzeugen
keys-type = Typ
keys-key-type = Schlüsseltyp
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-generate-smime-note = Erzeugt einen RSA-Schlüssel mit 3072 Bit und ein Zertifikat für die unten angegebene Adresse, zwei Jahre gültig. Das Zertifikat ist selbstsigniert: Die E-Mail-Programme anderer Personen vertrauen ihm nur, wenn sie es manuell akzeptieren oder eine Zertifizierungsstelle eines ausstellt. Mailwoman signiert oder entschlüsselt E-Mails noch nicht damit; danach können Sie es für ein anderes E-Mail-Programm exportieren.
keys-name = Name
keys-email = E-Mail
keys-key-passphrase = Schlüssel-Passphrase
keys-passphrase-help = Die Passphrase schützt den privaten Schlüssel auf diesem Gerät. Sie verlässt den Browser nie.
keys-cancel = Abbrechen
keys-generate-submit = Erzeugen
keys-generating = Wird erzeugt…
keys-generating-smime = Ein RSA-Schlüssel wird in diesem Browser erzeugt. Das kann bis zu einer Minute dauern.
keys-generate-failed = Es wurde kein Schlüssel erzeugt.
keys-generate-failed-smime = Es wurde kein Schlüssel erzeugt. Ein Zertifikat kann nur eine reine ASCII-Adresse (name@domain) und einen Namen mit höchstens 64 Zeichen enthalten.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Schlüssel importieren
keys-import-type = Importtyp
keys-tab-armored = ASCII-Armor (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Schlüssel im ASCII-Armor-Format
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Passphrase (falls der Schlüssel privat ist)
keys-import-passphrase = Import-Passphrase
keys-pkcs12-file-label = PKCS#12-Datei (.p12 / .pfx)
keys-pkcs12-file = PKCS#12-Datei
keys-import-password-label = Import-Passwort
keys-pkcs12-password = PKCS#12-Passwort
keys-import-preview = Importvorschau
keys-preview = Vorschau
keys-preview-type = Typ: { $kind }
keys-preview-fingerprint-aria = Fingerabdruck der Vorschau
keys-preview-fingerprint = Fingerabdruck: { $fp }
keys-preview-has-private = Enthält einen privaten Schlüssel (wird auf diesem Gerät gespeichert)
keys-preview-public-only = Nur öffentlicher Schlüssel
keys-import-submit = Importieren
