# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Sleutelbeheer
keys-title = Sleutels en certificaten
keys-subtitle = OpenPGP- en S/MIME-sleutels. Privésleutels blijven op dit apparaat en komen nooit op de server.
keys-generate = Sleutel aanmaken
keys-import = Sleutel importeren

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Jouw sleutels
keys-empty-own = Nog geen sleutels. Maak er een aan of importeer er een om te beginnen.
keys-contact-keys = Sleutels van contacten
keys-empty-contact = Geen sleutels van contacten. Zoek er hieronder een op.
keys-loading = Sleutels laden…
keys-select-prompt = Selecteer een sleutel om deze te bekijken en te verifiëren.
keys-untitled = { $kind }-sleutel

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = niet geverifieerd
keys-trust-tofu = tofu
keys-trust-verified = geverifieerd
keys-trust-revoked = ingetrokken

# -- Key detail --------------------------------------------------------------
keys-key-card = Sleutel { $name }
keys-fingerprint = Vingerafdruk
keys-safe-words = Controlewoorden
keys-safe-words-help = Lees deze samen met het contact hardop voor om de sleutel via een ander kanaal te bevestigen.
keys-safe-words-list = Controlewoorden van de vingerafdruk
keys-scan-label = Scan om te verifiëren
keys-qr-label = QR-code van de vingerafdruk
keys-autocrypt = Autocrypt
keys-autocrypt-on = Aangeboden in Autocrypt-headers
keys-autocrypt-off = Niet aangeboden via Autocrypt
keys-trust-label = Vertrouwen
keys-trust-level = Vertrouwensniveau
keys-associate-label = Koppelen aan een contact
keys-associate-select = Contact om te koppelen
keys-choose-contact = Kies een contact…
keys-associate-button = Koppelen
keys-backup-label = Back-up
keys-backup-help = Exporteer een Autocrypt Setup Message om deze sleutel naar een ander apparaat over te zetten.
keys-export-backup = Back-up exporteren
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Er staat op dit apparaat geen privésleutel om een back-up van te maken

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = S/MIME-certificaat
# Shown for a key generated here (not for an imported or CA-issued certificate).
keys-smime-self-signed = Dit certificaat is zelfondertekend: het is ondertekend met de eigen sleutel, niet door een certificeringsinstantie. De mailprogramma's van anderen vertrouwen het pas als ze het handmatig accepteren, of nadat een certificeringsinstantie een certificaat uitgeeft op basis van het aanvraagbestand hieronder.
# $date is YYYY-MM-DD, taken from the certificate.
keys-smime-valid-until = Geldig tot { $date }.
keys-smime-not-in-app = Mailwoman ondertekent en ontsleutelt nog geen mail met een S/MIME-sleutel. Exporteer de sleutel om hem in een ander mailprogramma te gebruiken.
keys-smime-passphrase = Wachtwoordzin voor deze S/MIME-sleutel
keys-smime-export-help = Het .p12-bestand bevat het certificaat en de privésleutel. De sleutel wordt versleuteld met de wachtwoordzin hierboven, en dezelfde wachtwoordzin opent het bestand in een ander programma. Het .csr-bestand bevat geen privésleutel.
keys-smime-export = Exporteren als PKCS#12 (.p12)
keys-smime-request = Certificaataanvraag downloaden (.csr)
keys-smime-issued-label = Certificaat uitgegeven voor deze sleutel (.pem, .crt, .cer)
keys-smime-issued-file = Bestand met uitgegeven certificaat
keys-smime-issued-help = Vervangt het certificaat dat voor deze sleutel is opgeslagen. Een certificaat dat voor een andere sleutel is gemaakt, wordt geweigerd.
keys-smime-attach = Uitgegeven certificaat importeren
keys-smime-failed = Dat is niet gelukt. Controleer de wachtwoordzin.
keys-smime-attach-failed = Het certificaat is niet geïmporteerd. Controleer de wachtwoordzin en of het bestand één certificaat bevat dat voor deze sleutel is uitgegeven.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Sleutel van een contact opzoeken
keys-lookup-heading = Sleutel opzoeken
keys-lookup-address = Adres om op te zoeken
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Bronnen om te doorzoeken
keys-source = Bron { $source }
keys-consent-label = Toestemming voor extern opzoeken
keys-consent-text = Bij het opzoeken van een sleutel worden externe directory's (WKD/VKS) geraadpleegd. Ik heb hiervoor toestemming van deze persoon.
keys-lookup-button = Opzoeken
keys-looking-up = Opzoeken…
keys-lookup-found = { $count } sleutel(s) gevonden — toegevoegd aan Sleutels van contacten.
keys-lookup-none = Geen sleutel gevonden.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Sleutel aanmaken
keys-type = Type
keys-key-type = Sleuteltype
keys-openpgp = OpenPGP
keys-smime = S/MIME
# Under the type list while S/MIME is chosen: what will be made, and its limits.
keys-generate-smime-note = Maakt een RSA-sleutel van 3072 bits en een certificaat voor het adres hieronder, twee jaar geldig. Het certificaat is zelfondertekend: de mailprogramma's van anderen vertrouwen het pas als ze het handmatig accepteren, of nadat een certificeringsinstantie er een uitgeeft. Mailwoman ondertekent en ontsleutelt er nog geen mail mee; daarna kun je het exporteren voor een ander mailprogramma.
keys-name = Naam
keys-email = E-mail
keys-key-passphrase = Wachtwoordzin voor de sleutel
keys-passphrase-help = De wachtwoordzin beschermt de privésleutel op dit apparaat. Hij verlaat de browser nooit.
keys-cancel = Annuleren
keys-generate-submit = Aanmaken
keys-generating = Aanmaken…
keys-generating-smime = Er wordt een RSA-sleutel aangemaakt in deze browser. Dit kan tot een minuut duren.
keys-generate-failed = Er is geen sleutel aangemaakt.
keys-generate-failed-smime = Er is geen sleutel aangemaakt. Een certificaat kan alleen een gewoon ASCII-adres (naam@domein) bevatten, en een naam van maximaal 64 tekens.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Sleutel importeren
keys-import-type = Importtype
keys-tab-armored = ASCII-armored (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = ASCII-armored sleutel
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Wachtwoordzin (als het een privésleutel is)
keys-import-passphrase = Wachtwoordzin voor importeren
keys-pkcs12-file-label = PKCS#12-bestand (.p12 / .pfx)
keys-pkcs12-file = PKCS#12-bestand
keys-import-password-label = Importwachtwoord
keys-pkcs12-password = PKCS#12-wachtwoord
keys-import-preview = Importvoorbeeld
keys-preview = Voorbeeld
keys-preview-type = Type: { $kind }
keys-preview-fingerprint-aria = Vingerafdruk in voorbeeld
keys-preview-fingerprint = Vingerafdruk: { $fp }
keys-preview-has-private = Bevat een privésleutel (wordt op dit apparaat opgeslagen)
keys-preview-public-only = Alleen openbare sleutel
keys-import-submit = Importeren
