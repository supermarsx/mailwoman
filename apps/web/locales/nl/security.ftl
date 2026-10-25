# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Beveiligingsdetails van het bericht
security-section-auth = Authenticatie
security-section-received = Bezorgroute
security-section-signature = Handtekening
security-section-attachments = Bijlagen
security-section-warnings = Waarschuwingen
security-section-controls = Afzenderopties
security-received-empty = Geen Received-keten beschikbaar.
security-attachments-empty = Geen bijlagen.
security-hop-unknown = onbekend
security-unknown = onbekend

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = geslaagd
security-auth-fail = mislukt
security-auth-none = niet aanwezig
security-auth-neutral = neutraal
security-auth-temperror = tijdelijke fout
security-auth-permerror = permanente fout

# Expert alignment detail field labels + values.
security-detail-domain = domein
security-detail-selector = selector
security-detail-policy = beleid
security-detail-alignment = uitlijning
security-detail-chain-length = ketenlengte
security-aligned = uitgelijnd
security-not-aligned = niet uitgelijnd

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Handtekening geverifieerd
security-sig-unverified-key = Ondertekend — sleutel van ondertekenaar niet geverifieerd
security-sig-invalid = Handtekening is ongeldig
security-sig-none = Niet ondertekend
security-chain-trusted = Vertrouwde keten
security-chain-untrusted = Niet-vertrouwde keten
security-chain-expired = Keten verlopen
security-chain-unknown = Keten onbekend
security-revocation-good = Niet ingetrokken
security-revocation-revoked = Sleutel ingetrokken
security-revocation-unknown = Intrekkingsstatus onbekend
security-key-changed = Sleutel van ondertekenaar gewijzigd sinds de vorige keer
security-fact-signer-key = Sleutel van ondertekenaar
security-fact-algorithm = Algoritme
security-fact-key-created = Sleutel aangemaakt
security-fact-key-expires = Sleutel verloopt
security-fact-chain = Keten
security-fact-revocation = Intrekking
security-fact-key-change = Sleutelwijziging

# -- Attachment risk --------------------------------------------------------
security-attach-none = Geen bekend risico
security-attach-macro = Bevat macro's
security-attach-executable = Uitvoerbaar bestand
security-attach-encrypted-archive = Versleuteld archief
security-attach-double-extension = Dubbele bestandsextensie
security-attach-mismatch = type komt niet overeen (opgegeven { $declared }, gedetecteerd { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Reply-To-adres wijkt af van de afzender
security-anomaly-envelopeFromDivergence = Envelop-afzender wijkt af van het From-adres
security-anomaly-messageIdDomainAnomaly = Domein van Message-ID komt niet overeen met de afzender
security-anomaly-dateSkew = Verzenddatum lijkt niet te kloppen
security-anomaly-punycodeSender = Afzender gebruikt punycode (mogelijk een domein dat op een ander lijkt)

# -- Sender controls --------------------------------------------------------
security-control-block = Afzender blokkeren
security-control-silence = Afzender dempen
security-control-ignore-conversation = Gesprek negeren
security-control-report-phishing = Phishing melden
security-control-report-junk = Als ongewenst melden
security-control-done-block = Afzender geblokkeerd
security-control-done-silence = Afzender gedempt
security-control-done-ignore-conversation = Gesprek genegeerd
security-control-done-report-phishing = Gemeld als phishing
security-control-done-report-junk = Gemeld als ongewenst

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Zero-access-opslag
security-za-on = Aan
security-za-off = Uit
security-za-protects = Zero-access versleutelt je opgeslagen mail, notities en PIM-gegevens, zodat de hostingserver alleen versleutelde gegevens bewaart die hij niet kan lezen. Het beschermt de inhoud van je opgeslagen gegevens tegen een nieuwsgierige beheerder of een inbraak op de opslaghost: wie de database, de bestanden op schijf of een gestolen back-up leest, ziet XChaCha20-Poly1305-versleutelde gegevens en kan berichtteksten, onderwerpen, bijlagen, notitieteksten en de zoekindex niet terughalen.

security-za-server-sees-title = Wat de server nog wel ziet
security-za-sees-1 = Versleutelde blobs — de versleutelde rijen zelf.
security-za-sees-2 = Ondoorzichtige rij-ID's — de identificatoren waarmee rijen worden opgeslagen en opgehaald.
security-za-sees-3 = Groottes — de lengte van elk versleuteld item (bij benadering de grootte van berichten/bijlagen).
security-za-sees-4 = Tijdstempels — wanneer rijen worden geschreven en bijgewerkt.
security-za-sees-5 = Envelop- en routeringsmetadata die nodig zijn om IMAP/SMTP door te sturen — de server maakt nog steeds namens jou verbinding met je upstream-mailprovider, dus verbindingsmetadata en de routeringsenvelop die nodig zijn om mail te verzenden en te ontvangen, gaan via de server.

security-za-active-server-caveat = Zero-access beschermt OPGESLAGEN gegevens tegen een nieuwsgierige of gehackte host. Een volledig kwaadwillende actieve server die je live IMAP/SMTP-verkeer doorstuurt, is een sterkere tegenstander, en daartegen beschermt zero-access NIET: zo'n server zit op de live verbinding met je mailprovider en kan mail bekijken of wijzigen terwijl die erdoorheen gaat, ongeacht hoe de opgeslagen kopie is versleuteld. Kies zero-access als je zorg is: "Ik wil niet dat wie de opslaghost beheert (of hackt) mijn opgeslagen mail kan lezen." Het is geen bescherming tegen een beheerder die het live mailpad ondermijnt.

security-za-no-search-claim = Er is hier geen doorzoekbare versleuteling aan de serverkant, en dat wordt ook niet beweerd. Zoeken gebeurt volledig in de client, op inhoud die lokaal is ontsleuteld; de server heeft nooit een doorzoekbare vorm van je onversleutelde tekst.

security-za-recovery-tradeoff = Je sleutels worden alleen op je apparaten afgeleid en bewaard — de server ontvangt nooit een onversleutelde sleutel. Als je je wachtwoordzin (of passkey) EN al je gekoppelde apparaten EN je herstelzin kwijtraakt, kunnen de versleutelde gegevens niet worden hersteld. Bewaar de herstelzin offline en behandel hem als gelijkwaardig aan het hoofdgeheim van je account.

security-za-setup-key = Je sleutel instellen
security-za-key-source = Sleutelbron
security-za-passphrase = Wachtwoordzin
security-za-passkey = Passkey (zonder wachtwoord)
security-za-passkey-unavailable = passkeys zijn niet beschikbaar in deze browser
security-za-passphrase-label = Wachtwoordzin voor zero-access
security-za-enable = Zero-access inschakelen
security-za-recovery-title = Herstelzin
security-za-recovery-heading = Herstelzin — bewaar deze nu offline
security-za-recovery-note = Dit is de enige kopie. Wie hem heeft, kan je gegevens lezen; zonder deze zin (en zonder gekoppeld apparaat) kunnen je gegevens niet worden hersteld.
security-za-disable-section = Zero-access uitschakelen
security-za-disable-heading = Zero-access uitzetten
security-za-disable-note = Nieuwe gegevens worden weer onversleuteld opgeslagen. Bestaande versleutelde gegevens blijven alleen leesbaar zolang je je sleutel nog kunt afleiden.
security-za-disable-btn = Zero-access uitschakelen
security-za-err-passphrase-len = gebruik een wachtwoordzin van minstens 8 tekens
security-za-err-enable = kan zero-access niet inschakelen
security-za-err-disable = kan zero-access niet uitschakelen

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Apparaat koppelen
security-pair-heading = Een apparaat koppelen
security-pair-intro = Zet je sleutels over naar een ander apparaat zonder ze via de server te sturen — die geeft alleen een ondoorzichtige verzegelde envelop door. Vergelijk de zes woorden op beide schermen voordat je de koppeling vertrouwt.
security-pair-role = Rol bij koppelen
security-pair-new-role = Dit is het nieuwe apparaat
security-pair-existing-role = Vanaf hier een ander apparaat koppelen
security-pair-show-qr = Koppel-QR-code tonen
security-pair-scan = Scan deze op je bestaande apparaat
security-pair-qr-label = QR-code voor apparaatkoppeling
security-pair-qr-default = Koppel-QR-code
security-pair-copy-code = Of kopieer deze koppelcode
security-pair-paste-envelope = Plak de verzegelde envelop van het andere apparaat
security-pair-envelope-placeholder = envelop…
security-pair-envelope-label = Verzegelde envelop
security-pair-complete = Koppeling voltooien
security-pair-code-from-new = Koppelcode van het nieuwe apparaat
security-pair-code-placeholder = koppelcode…
security-pair-code-label = Koppelcode
security-pair-seal = Mijn sleutels verzegelen voor dat apparaat
security-pair-envelope-out = Verzegelde envelop — plak deze op het nieuwe apparaat
security-pair-compare = Vergelijk deze woorden op beide apparaten
security-pair-sas-label = Korte verificatiereeks
security-pair-match = De woorden komen overeen
security-pair-abort = Ze verschillen — afbreken
security-pair-confirmed = Koppeling bevestigd. Dit kanaal is geverifieerd.
security-pair-err-start = kan het koppelen niet starten
security-pair-err-complete = kan het koppelen niet voltooien
security-pair-err-unlock = ontgrendel eerst zero-access om een ander apparaat te koppelen
security-pair-err-seal = kan de hoofdsleutel niet verzegelen
