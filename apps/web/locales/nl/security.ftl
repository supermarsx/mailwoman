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
security-za-protects = Zero-access versleutelt uw opgeslagen mail, notities en PIM-gegevens, zodat de hostingserver alleen versleutelde gegevens bewaart die hij niet kan lezen. Het beschermt de inhoud van uw opgeslagen gegevens tegen een nieuwsgierige beheerder of een inbraak op de opslaghost: wie de database, de bestanden op schijf of een gestolen back-up leest, ziet XChaCha20-Poly1305-versleutelde gegevens en kan berichtteksten, onderwerpen, bijlagen, notitieteksten en de zoekindex niet terughalen.

security-za-server-sees-title = Wat de server nog wel ziet
security-za-sees-1 = Versleutelde blobs — de versleutelde rijen zelf.
security-za-sees-2 = Ondoorzichtige rij-ID's — de identificatoren waarmee rijen worden opgeslagen en opgehaald.
security-za-sees-3 = Groottes — de lengte van elk versleuteld item (bij benadering de grootte van berichten/bijlagen).
security-za-sees-4 = Tijdstempels — wanneer rijen worden geschreven en bijgewerkt.
security-za-sees-5 = Envelop- en routeringsmetadata die nodig zijn om IMAP/SMTP door te sturen — de server maakt nog steeds namens u verbinding met uw upstream-mailprovider, dus verbindingsmetadata en de routeringsenvelop die nodig zijn om mail te verzenden en te ontvangen, gaan via de server.

security-za-active-server-caveat = Zero-access beschermt OPGESLAGEN gegevens tegen een nieuwsgierige of gehackte host. Een volledig kwaadwillende actieve server die uw live IMAP/SMTP-verkeer doorstuurt, is een sterkere tegenstander, en daartegen beschermt zero-access NIET: zo'n server zit op de live verbinding met uw mailprovider en kan mail bekijken of wijzigen terwijl die erdoorheen gaat, ongeacht hoe de opgeslagen kopie is versleuteld. Kies zero-access als uw zorg is: "Ik wil niet dat wie de opslaghost beheert (of hackt) mijn opgeslagen mail kan lezen." Het is geen bescherming tegen een beheerder die het live mailpad ondermijnt.

security-za-no-search-claim = Er is hier geen doorzoekbare versleuteling aan de serverkant, en dat wordt ook niet beweerd. Zoeken gebeurt volledig in de client, op inhoud die lokaal is ontsleuteld; de server heeft nooit een doorzoekbare vorm van uw onversleutelde tekst.

security-za-recovery-tradeoff = Uw sleutels worden alleen op uw apparaten afgeleid en bewaard — de server ontvangt nooit een onversleutelde sleutel. Als u uw wachtwoordzin (of passkey) EN al uw gekoppelde apparaten EN uw herstelzin kwijtraakt, kunnen de versleutelde gegevens niet worden hersteld. Bewaar de herstelzin offline en behandel hem als gelijkwaardig aan het hoofdgeheim van uw account.

security-za-setup-key = Uw sleutel instellen
security-za-key-source = Sleutelbron
security-za-passphrase = Wachtwoordzin
security-za-passkey = Passkey (zonder wachtwoord)
security-za-passkey-unavailable = passkeys zijn niet beschikbaar in deze browser
security-za-passphrase-label = Wachtwoordzin voor zero-access
security-za-enable = Zero-access inschakelen
security-za-recovery-title = Herstelzin
security-za-recovery-heading = Herstelzin — bewaar deze nu offline
security-za-recovery-note = Dit is de enige kopie. Wie hem heeft, kan uw gegevens lezen; zonder deze zin (en zonder gekoppeld apparaat) kunnen uw gegevens niet worden hersteld.
security-za-disable-section = Zero-access uitschakelen
security-za-disable-heading = Zero-access uitzetten
security-za-disable-note = Nieuwe gegevens worden weer onversleuteld opgeslagen. Bestaande versleutelde gegevens blijven alleen leesbaar zolang u uw sleutel nog kunt afleiden.
security-za-disable-btn = Zero-access uitschakelen
security-za-err-passphrase-len = gebruik een wachtwoordzin van minstens 8 tekens
security-za-err-enable = kan zero-access niet inschakelen
security-za-err-disable = kan zero-access niet uitschakelen

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Apparaat koppelen
security-pair-heading = Een apparaat koppelen
security-pair-intro = Zet uw sleutels over naar een ander apparaat zonder ze via de server te sturen — die geeft alleen een ondoorzichtige verzegelde envelop door. Vergelijk de zes woorden op beide schermen voordat u de koppeling vertrouwt.
security-pair-role = Rol bij koppelen
security-pair-new-role = Dit is het nieuwe apparaat
security-pair-existing-role = Vanaf hier een ander apparaat koppelen
security-pair-show-qr = Koppel-QR-code tonen
security-pair-scan = Scan deze op uw bestaande apparaat
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

# -- Machine translation (2026-11): added to match en ---------------------
security-stepup-title = Bevestig dat u het bent
security-stepup-body = Bevestig uw identiteit om deze actie voort te zetten. De bevestiging geldt vijf minuten in deze sessie.
security-stepup-password = Wachtwoord
security-stepup-code = Authenticatorcode
security-stepup-confirm = Bevestigen en doorgaan
security-stepup-passkey = Een passkey gebruiken
security-stepup-cancel = Annuleren
security-stepup-failed = Bevestiging mislukt. Probeer het opnieuw.
security-stepup-unavailable = Deze sessie heeft geen beschikbare bevestigingsmethode.
security-center-title = Beveiligingscentrum
security-center-history = Inloggeschiedenis
security-center-result = Resultaat
security-center-all = Alle resultaten
security-center-success = Geslaagd
security-center-failure = Mislukt
security-center-blocked = Geblokkeerd
security-center-refresh = Vernieuwen
security-center-export = Geschiedenis exporteren
security-center-devices = Onthouden apparaten
security-center-remember-help = Als uw beheerder het toestaat, kunt u dit apparaat onthouden nadat uw identiteit is geverifieerd. Gevoelige wijzigingen vereisen nog steeds een nieuwe verificatie.
security-center-label = Apparaatnaam
security-center-remember = Dit apparaat onthouden
security-center-revoke = Intrekken
security-center-activity = Accountactiviteit
security-center-error = De beveiligingsgegevens konden niet worden geladen of gewijzigd. Uw beheerder heeft deze actie mogelijk uitgeschakeld.
security-center-first-seen = Voor het eerst gezien: { $when }
security-center-passkey-mode = In engine-modus kunt u met passkeys zonder wachtwoord inloggen. In proxymodus verifiëren ze een tweede factor na uw mailwachtwoord.
security-stepup-recovery = Herstelcode (met uw wachtwoord, in plaats van een authenticatorcode)
security-center-signin = Inloggen en passkeys
security-center-sessions = Sessies en toegang
security-center-encryption = Versleuteling
signin-geo-title = Landenbeleid voor inloggen
signin-geo-help = Weiger desgewenst nieuwe logins op uw account vanuit landen die u nog niet eerder hebt gebruikt, of vanuit elk land buiten uw lijst met toegestane landen. Standaard uit en heeft nooit invloed op sessies die al zijn ingelogd. Toegestane IP's en landen komen altijd door deze controle; uw wachtwoord en tweede factor blijven vereist.
signin-geo-accuracy = Landdetectie gebruikt een IP-geolocatiedatabase die onjuist of verouderd kan zijn. VPN's, mobiele providers, proxy's en reizen kunnen een legitieme login blokkeren of een buitenlandse laten doorgaan. Voeg vertrouwde IP-adressen of landen toe aan de toegestanelijst.
signin-geo-locked = Een beheerder beheert deze instelling voor uw account.
signin-geo-mode = Beleid
signin-geo-mode-off = Uit
signin-geo-mode-block-unusual-countries = Landen blokkeren waarvan ik nog niet heb ingelogd
signin-geo-mode-block-foreign-countries = Landen buiten mijn lijst met toegestane landen blokkeren
signin-geo-countries = Toegestane landen (codes van twee letters, gescheiden door komma's)
signin-geo-ips = Toegestane IP-adressen of netwerken (één per regel)
signin-geo-current = Uw huidige adres is { $ip } (gedetecteerd land: { $country }).
signin-geo-no-context = Uw huidige adres kon niet worden vastgesteld.
signin-geo-unknown-country = onbekend
signin-geo-add-ip = Mijn huidige IP toevoegen
signin-geo-add-country = Mijn huidige land toevoegen
signin-geo-save = Opslaan
signin-geo-saved = Opgeslagen.
signin-geo-invalid = Controleer de landcodes en IP-adressen.
signin-geo-self-lockout = Hiermee zou inloggen vanaf uw huidige adres of land worden geblokkeerd. Voeg eerst uw huidige IP-adres of land toe aan de toegestanelijst.
signin-geo-error = Het landenbeleid voor inloggen kon niet worden opgeslagen.
session-alerts-title = Beveiligingsmeldingen over sessies
session-alerts-help = Waarschuwingen over ongebruikelijk gebruik van uw ingelogde sessies. Elke melding toont wat de server zag en wat hij deed.
session-alerts-none = Geen beveiligingsmeldingen over sessies.
session-alerts-banner = { $count ->
    [one] U hebt 1 ongelezen beveiligingsmelding over een sessie.
   *[other] U hebt { $count } ongelezen beveiligingsmeldingen over sessies.
}
session-alerts-banner-review = Bekijken
session-alerts-kind-countryChange = Een sessie is vanuit een ander land gebruikt ({ $from } naar { $to }).
session-alerts-kind-impossibleTravel = Een sessie verscheen in een ander land ({ $from } naar { $to }) sneller dan reizen toelaat.
session-alerts-kind-concurrentUse = Eén sessie is tegelijkertijd vanuit verschillende landen of netwerken gebruikt ({ $from } en { $to }).
session-alerts-kind-fingerprintMismatch = Een sessie is gebruikt vanuit een browser of apparaat dat niet overeenkomt met degene waarmee is ingelogd.
session-alerts-kind-rotationReuse = Een oude sessie-ID werd opnieuw aangeboden nadat de sessie was vernieuwd.
session-alerts-action-alerted = Actie: u bent gewaarschuwd.
session-alerts-action-step-up = Actie: de sessie is gevraagd uw wachtwoord en tweede factor te bevestigen.
session-alerts-action-revoked = Actie: de sessie is afgemeld.
session-alerts-countries = Landen: { $from } naar { $to }
session-alerts-unknown-country = onbekend
session-alerts-session = Sessie { $id }
session-alerts-caveat = Landdetectie kan onnauwkeurig zijn; VPN's en mobiele netwerken kunnen valse wijzigingen veroorzaken.
session-alerts-not-me = Dit was ik niet
session-alerts-dismiss = Sluiten
session-alerts-dismiss-all = Alles sluiten
session-alerts-revoked = Uw andere sessies zijn afgemeld. Wijzig nu uw wachtwoord in het wachtwoordonderdeel van Inloggen en passkeys.
session-alerts-error = De beveiligingsmeldingen over sessies konden niet worden bijgewerkt.
session-alerts-prefs = Voorkeuren voor waarschuwingen
session-alerts-country = Waarschuw mij wanneer een sessie vanuit een nieuw land wordt gebruikt
session-alerts-country-help = Als de beheerder dempen niet toestaat, blijft de server waarschuwen, ook wanneer dit uit staat.
session-alerts-channels = Waar waarschuwingen worden ontvangen
session-alerts-channel-app = In de app
session-alerts-channel-email = In mijn mailbox
session-alerts-channel-both = In de app en in mijn mailbox
session-alerts-sensitivity = Gevoeligheid
session-alerts-sensitivity-low = Laag: alleen locatiebevindingen, geen gedwongen afmelding
session-alerts-sensitivity-normal = Normaal
session-alerts-sensitivity-high = Hoog: strengere reiscontrole, vergelijkt ook netwerken
session-alerts-reset = Standaardwaarden van de beheerder gebruiken
session-alerts-pref-error = De voorkeur kon niet worden opgeslagen.
signin-geo-history-title = Geziene landen
signin-geo-history-help = De landen waarvanuit u met succes hebt ingelogd, { $days } dagen bewaard. De modus "Ongebruikelijke landen" gebruikt deze lijst. Alleen landcodes, datums en aantallen worden opgeslagen, nooit adressen.
signin-geo-history-empty = Er zijn nog geen landen geregistreerd.
signin-geo-history-entry = { $country }: eerste { $first }, laatste { $last }, { $count } logins
signin-geo-history-forget = Geziene landen vergeten
security-severity-info = Informatief
security-severity-caution = Let op
security-severity-suspicious = Verdacht
security-explain-toggle = Wat betekent dit?
security-explain-label-checked = Wat is gecontroleerd
security-explain-label-legit = Veelvoorkomende onschuldige oorzaken
security-explain-label-risk = Wat het kan aanduiden
security-explain-label-action = Voorgestelde actie
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = Het domein achter de @ in de Message-ID-kop is vergeleken met het domein in het Van-adres. Ze verschillen.
security-explain-messageIdDomainAnomaly-legit = Mailinglijsten, marketing- en nieuwsbriefplatforms, doorstuurdiensten, bedrijfsgateways en webmailhosts genereren de Message-ID vaak met hun eigen servernaam.
security-explain-messageIdDomainAnomaly-risk = Een vervalste afzender, of een bericht dat is gemaakt door software die niet het eigen mailsysteem van de afzender is. Op zichzelf is dit een zwak signaal.
security-explain-messageIdDomainAnomaly-action = Bekijk de resultaten van SPF, DKIM en DMARC hierboven. Als ze voor het domein van de afzender zijn geslaagd, is dit hoogstwaarschijnlijk routine. Als ze zijn mislukt of ontbreken, vermijd dan links en bijlagen en bevestig het bij de afzender op een andere manier.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = Het Antwoord-aan-adres is vergeleken met het Van-adres. Antwoorden zouden naar een ander adres gaan.
security-explain-replyToMismatch-legit = Mailinglijsten, helpdesks, assistenten die namens iemand schrijven en contactformulieren stellen met opzet een ander Antwoord-aan-adres in.
security-explain-replyToMismatch-risk = Antwoorden die worden omgeleid naar een adres dat de aanvaller beheert, een veelvoorkomend patroon bij imitatie en factuurfraude.
security-explain-replyToMismatch-action = Controleer het Antwoord-aan-adres voordat u antwoordt. Als het bericht om geld, inloggegevens of een wijziging van gegevens vraagt, bevestig dit dan via een kanaal dat u al vertrouwt.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = De enveloppe-afzender (Return-Path), waar onbestelbaar-meldingen naartoe gaan, is vergeleken met het Van-adres. Ze verschillen.
security-explain-envelopeFromDivergence-legit = Bulkmaildiensten, ticketsystemen en mailinglijsten gebruiken hun eigen retouradres terwijl ze het adres van uw contact in Van tonen.
security-explain-envelopeFromDivergence-risk = Afzenderspoofing, wanneer SPF of DMARC ook mislukt. Op zichzelf komt het vaak voor en is het meestal onschuldig.
security-explain-envelopeFromDivergence-action = Controleer of DMARC met alignment is geslaagd. Zo ja, dan is geen actie nodig.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = De Date-kop is vergeleken met het tijdstip waarop het bericht is ontvangen. Ze liggen ver uit elkaar.
security-explain-dateSkew-legit = Een computer van de afzender met een verkeerde klok, een bericht dat lang in een wachtrij heeft gestaan of opnieuw is geprobeerd, of een geplande verzending.
security-explain-dateSkew-risk = Een herhaald of teruggedateerd bericht. Dit is ongebruikelijk en op zichzelf zelden schadelijk.
security-explain-dateSkew-action = Behandel tijdgevoelige verzoeken in het bericht met zorg en bevestig ze als ze belangrijk zijn.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = Het domein van de afzender bevat punycode (een xn--label), de manier waarop niet-Latijnse tekens in domeinnamen worden geschreven.
security-explain-punycodeSender-legit = Echte domeinen in talen zoals Arabisch, Chinees, Cyrillisch of Portugees met accenten worden zo gecodeerd.
security-explain-punycodeSender-risk = Een domein dat erop lijkt en tekens uit andere alfabetten mengt om op een bekend merk of een bekende persoon te lijken.
security-explain-punycodeSender-action = Vergelijk het domein van de afzender teken voor teken met het domein dat u verwacht. Klik niet op links en open geen bijlagen totdat u de afzender op een andere manier hebt bevestigd, en meld het als het niet echt is.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = De ontvangende server heeft deze methode gecontroleerd en het resultaat kwam niet overeen met wat het domein van de afzender publiceert.
security-explain-auth-fail-legit = Doorsturen en mailinglijsten kunnen SPF of DKIM breken, en een verkeerd ingestelde verzenddienst kan voor beide mislukken.
security-explain-auth-fail-risk = Het bericht komt mogelijk niet van wie het beweert, of is onderweg gewijzigd.
security-explain-auth-fail-action = Vertrouw de afzender niet op basis van dit bewijs. Vermijd links en bijlagen en verifieer het via een ander kanaal.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = De controle kon niet worden voltooid omdat het gepubliceerde record van het domein ongeldig of te complex is.
security-explain-auth-permerror-legit = Een fout in de DNS-records van de afzender.
security-explain-auth-permerror-risk = Niets over het bericht zelf, maar het domein van de afzender kan niet worden geverifieerd.
security-explain-auth-permerror-action = Behandel het als een ontbrekend resultaat en controleer de andere methoden.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = Er is geen resultaat voor deze methode gevonden in de berichtkoppen.
security-explain-auth-none-legit = Het domein van de afzender publiceert deze methode niet, of de ontvangende server registreert haar niet.
security-explain-auth-none-risk = Het domein biedt geen manier om de afzender te verifiëren, dus spoofing kan niet worden uitgesloten.
security-explain-auth-none-action = Baseer u op de andere methoden en op de vraag of u dit bericht verwacht.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = Het domein heeft een record voor deze methode gepubliceerd, maar doet geen uitspraak over dit bericht.
security-explain-auth-neutral-legit = Een domein dat zijn beleid nog aan het instellen is.
security-explain-auth-neutral-risk = Weinig. De afzender kon niet worden bevestigd of afgewezen.
security-explain-auth-neutral-action = Baseer u op de andere methoden en op de vraag of u dit bericht verwacht.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = De controle kon niet worden afgerond vanwege een tijdelijk probleem, meestal een mislukte DNS-opzoeking.
security-explain-auth-temperror-legit = Een korte storing in de DNS van de afzender of de ontvanger.
security-explain-auth-temperror-risk = Niets over het bericht zelf.
security-explain-auth-temperror-action = Meestal is geen actie nodig. Controleer later opnieuw als het bericht belangrijk is.
breach-notice-heading = Kennisgeving over een inbreuk op persoonsgegevens
breach-notice-reference = Referentie { $reference }
breach-notice-ack = Bevestigen

session-evidence-ip = IP-adres
session-evidence-location = Locatie
session-evidence-device = Apparaattype
session-evidence-recognized = Apparaatherkenning
session-evidence-browser = Browser
session-evidence-engine = Browserengine
session-evidence-os = Besturingssysteem
session-evidence-client = Clienttype
session-evidence-version = Clientversie
session-evidence-authentication = Authenticatie
session-evidence-fingerprint = Apparaatvingerafdruk
session-evidence-agent = User-agent
session-evidence-unavailable = Niet beschikbaar
session-evidence-known = Herkend apparaat
session-evidence-new = Niet-herkend apparaat
session-evidence-technical = Technische details
session-evidence-legacy = Apparaatgegevens zijn niet vastgelegd voor deze oudere melding.
session-evidence-help = De client rapporteert apparaat- en browsergegevens. Herkenning is informatief en bewijst geen veiligheid. De vingerafdruk is een accountspecifieke referentie, geen hardware-ID.
session-evidence-device-desktop = Computer
session-evidence-device-phone = Telefoon
session-evidence-device-tablet = Tablet
session-evidence-device-unknown = Onbekend apparaat
session-evidence-client-browser = Webbrowser
session-alerts-kind-countryUnavailable = Het land van een sessieverzoek kon niet worden bepaald.
session-alerts-kind-unknown = Controleer ongebruikelijke sessieactiviteit.

notification-center-help = Bekijk belangrijke accountactiviteit en updates. Negeren en archiveren ordenen de lijst, maar lossen beveiligingsproblemen niet op.
notification-center-active = Actief
notification-center-archived = Gearchiveerd
notification-center-dismissed = Genegeerd
notification-center-type = Type
notification-center-type-all = Alle typen
notification-center-type-security = Beveiliging
notification-center-type-session = Sessiewaarschuwingen
notification-center-type-login = Aanmeldingen
notification-center-type-update = App-updates
notification-center-login = Aanmelding geslaagd
notification-center-security = Accountbeveiligingsactiviteit
notification-center-newDevice = Aanmelding vanaf een niet-herkend apparaat
notification-center-newCountry = Aanmelding vanuit een nieuw land
notification-center-twoFactorChanged = Tweestapsverificatie gewijzigd
notification-center-delegateAdded = Postvaktoegang verleend
notification-center-forwardingAdded = Doorstuurregel toegevoegd
notification-center-apiKeyCreated = API-sleutel aangemaakt
notification-center-error = Meldingen konden niet worden geladen of bijgewerkt. Vernieuw de weergave.
notification-center-empty = Geen meldingen in deze weergave.
notification-center-open = Bijbehorende actie openen
notification-center-dismiss = Negeren
notification-center-archive = Archiveren
notification-center-restore = Herstellen
