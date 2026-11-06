# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Diagnostiekpagina's
admin-diagx-tab-overview = Overzicht
admin-diagx-tab-server = Server
admin-diagx-tab-connectivity = Connectiviteit
admin-diagx-tab-backends = Mailbackends
admin-diagx-tab-posture = Beveiligingsniveau
admin-diagx-tab-tools = Hulpmiddelen
admin-diagx-tab-history = Geschiedenis
admin-diagx-tab-report = Rapport
admin-diagx-tab-server-intro = Controles van de eigen onderdelen van deze server: de database, de migraties en de serversleutel. Ze draaien samen met de servercontroles.
admin-diagx-tab-connectivity-intro = Of deze server bereikbaar is onder zijn openbare naam, de opgeslagen backends kan bereiken en privéadressen buiten bereik houdt. De domeinhulpmiddelen hieronder zoeken DNS-records op, testen TLS en ontdekken instellingen voor één domein.
admin-diagx-tab-backends-intro = Een verbinding en beveiligingshandshake met elk opgeslagen mail-, verzend-, regel- en DAV-endpoint. Er worden geen inloggegevens verstuurd en er wordt geen e-mail bezorgd.
admin-diagx-tab-posture-intro = Instellingen die aanmeldingen en sessies beschermen, en hoe fail2ban het adres van een verzoek zou zien.
admin-diagx-tab-tools-intro = Andere plekken om de dienst te controleren, en de limieten die voor diagnostiekruns gelden.
admin-diagx-tab-history-intro = Opgeslagen uitvoeringen, nieuwste eerst. Open een uitvoering om de resultaten te zien en wat er is veranderd sinds de vorige uitvoering.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Waarschuwing
admin-diagx-status-fail = Mislukt
admin-diagx-status-skipped = Overgeslagen
admin-diagx-status-not-run = Niet uitgevoerd
admin-diagx-status-running = Bezig
admin-diagx-run = Uitvoeren
admin-diagx-run-again = Opnieuw uitvoeren
admin-diagx-running = Bezig…
admin-diagx-run-server-note = Voert de servercontroles opnieuw uit. Alle kaarten van die uitvoering worden bijgewerkt.
admin-diagx-run-backends-note = Controleert alle opgeslagen backends opnieuw. Alle backendkaarten worden bijgewerkt.
admin-diagx-how-to-fix = Hoe op te lossen
admin-diagx-details = Details
admin-diagx-copy-details = Details kopiëren
admin-diagx-copy-value = { $label } kopiëren
admin-diagx-copied = Gekopieerd.
admin-diagx-copy-failed = De browser stond kopiëren niet toe. Selecteer de tekst en kopieer die handmatig.
admin-diagx-not-run-yet = Nog niet uitgevoerd.
admin-diagx-not-reported = De laatste serveruitvoering meldde deze controle niet; ze is niet van toepassing op deze implementatie.
admin-diagx-posture-met = Voldaan
admin-diagx-posture-not-met = Niet voldaan
admin-diagx-loading = Laden…
admin-diagx-none = Geen
admin-diagx-yes = Ja
admin-diagx-no = Nee
admin-diagx-open = Openen
admin-diagx-row-result = Resultaat
admin-diagx-row-duration = Duur
admin-diagx-row-checked = Gecontroleerd
admin-diagx-row-id = Controle-ID
admin-diagx-row-domain = Domein
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protocol
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Mogelijkheden
admin-diagx-row-credentials = Inloggegevens verstuurd
admin-diagx-row-source = Gevonden in
admin-diagx-row-oauth = Vereist OAuth
admin-diagx-overview-intro = Het nieuwste resultaat van elke controle, per groep. Alles uitvoeren voert de servercontroles uit en controleert daarna elke opgeslagen backend.
admin-diagx-run-all = Alle controles uitvoeren
admin-diagx-run-all-step = Stap { $step } van { $steps }: { $suite ->
    [backends] opgeslagen backends
   *[server] servercontroles
}.
admin-diagx-run-all-done = Alle controles zijn klaar. De resultaten staan in Geschiedenis.
admin-diagx-backends-progress = { $completed } van { $total } backends gecontroleerd.
admin-diagx-counts = { $ok } OK · { $warn } waarschuwingen · { $fail } mislukt · { $notRun } niet uitgevoerd
admin-diagx-attention = Aandacht nodig
admin-diagx-attention-none = Geen enkele controle meldde een waarschuwing of fout in de laatste uitvoering.
admin-diagx-never-run = Er zijn nog geen controles uitgevoerd. Voer alle controles uit voor een eerste resultaat.
admin-diagx-last-server = Laatste servercontroles
admin-diagx-last-backends = Laatste backendcontroles
admin-diagx-never = Nooit
admin-diagx-open-health = Servicestatus
admin-diagx-check-database-connectivity = Databaseverbinding
admin-diagx-check-database-connectivity-purpose = De database beantwoordt een eenvoudige query.
admin-diagx-check-database-connectivity-fix = Controleer of de database draait en of MW_DB_PATH of MW_DATABASE_URL ernaar verwijst. Het serverlogboek noemt de verbindingsfout.
admin-diagx-check-database-write = Databaseschrijfacties
admin-diagx-check-database-write-purpose = Een testrij kan worden geschreven en wordt daarna teruggedraaid.
admin-diagx-check-database-write-fix = Controleer de vrije schijfruimte en of het databasebestand of de databasegebruiker mag schrijven. Een alleen-lezen volume of een volle schijf stopt schrijfacties.
admin-diagx-check-database-migrations = Databasemigraties
admin-diagx-check-database-migrations-purpose = De database heeft elke migratie die deze serverversie verwacht.
admin-diagx-check-database-migrations-fix = Start de server opnieuw zodat hij openstaande migraties toepast, en controleer het logboek op een mislukte migratie. Draai geen oudere server op een nieuwere database.
admin-diagx-check-server-key-roundtrip = Serversleutel
admin-diagx-check-server-key-roundtrip-purpose = De serversleutel verzegelt en opent een testwaarde.
admin-diagx-check-server-key-roundtrip-fix = Controleer of MW_SERVER_KEY (of het sleutelbestand) de sleutel is waarmee deze database is gemaakt. Opgeslagen geheimen kunnen niet met een andere sleutel worden geopend.
admin-diagx-check-other-purpose = Een controle gemeld door deze server.
admin-diagx-check-other-fix = Open Details voor het ruwe resultaat en controleer het serverlogboek op het moment van de uitvoering.
admin-diagx-check-network-public-dns = Openbare naam
admin-diagx-check-network-public-dns-purpose = De hostnaam in MW_PUBLIC_URL wordt vanaf deze server omgezet.
admin-diagx-check-network-public-dns-fix = Controleer het DNS-record van de openbare naam, of stel MW_PUBLIC_URL in op het adres waarmee mensen deze server bereiken.
admin-diagx-check-network-public-port = Openbare poort
admin-diagx-check-network-public-port-purpose = De poort van de openbare URL accepteert een verbinding vanaf deze server.
admin-diagx-check-network-public-port-fix = Controleer de firewall, de reverse proxy en de poortdoorsturing. Sommige routers kunnen geen verbinding maken met hun eigen openbare adres (NAT hairpinning); test ook van buiten uw netwerk.
admin-diagx-check-network-backend-ports = Backendpoorten
admin-diagx-check-network-backend-ports-purpose = De host en poort van elke opgeslagen backend accepteren een directe verbinding.
admin-diagx-check-network-backend-ports-fix = Controleer of de backendhosts draaien en of een firewall deze server toegang geeft. Mailbackends toont welk endpoint de handshake niet haalt.
admin-diagx-check-egress-private-policy = Bescherming van privéadressen
admin-diagx-check-egress-private-policy-purpose = Uitgaande verbindingen naar loopback- en privéadressen worden standaard geweigerd.
admin-diagx-check-egress-private-policy-fix = Bekijk de uitgaande instellingen onder Privacy. Sta privéadressen alleen toe voor genoemde backends die ze nodig hebben.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Verzoeken bereiken deze server via HTTPS.
admin-diagx-check-security-https-fix = Lever de console via HTTPS, of stel MW_PUBLIC_URL in op https en MW_TRUSTED_PROXIES wanneer een reverse proxy TLS afsluit.
admin-diagx-check-security-secure-cookies = Beveiligde cookies
admin-diagx-check-security-secure-cookies-purpose = Sessiecookies worden alleen via HTTPS verstuurd.
admin-diagx-check-security-secure-cookies-fix = Stel MW_COOKIE_SECURE in op true zodra de server via HTTPS wordt bereikt.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Browsers krijgen de opdracht HTTPS voor deze site te gebruiken.
admin-diagx-check-security-hsts-fix = Schakel Strict-Transport-Security in bij de beveiligingsinstellingen, of stuur het mee vanuit de reverse proxy.
admin-diagx-check-security-csrf = CSRF-bescherming
admin-diagx-check-security-csrf-purpose = Verzoeken die gegevens wijzigen moeten het CSRF-token meesturen.
admin-diagx-check-security-csrf-fix = Stel MW_CSRF_STRICT in op true.
admin-diagx-check-security-admin-factors = Tweede factor van beheerders
admin-diagx-check-security-admin-factors-purpose = Elke beheerder heeft een tweede aanmeldfactor.
admin-diagx-check-security-admin-factors-fix = Vraag elke beheerder zonder tweede factor een passkey of authenticator-app toe te voegen in Mijn voorkeuren, of eis dit in het Beveiligingscentrum.
admin-diagx-check-security-signin-pacing = Wachttijd voor nieuwe aanmeldpoging
admin-diagx-check-security-signin-pacing-purpose = Na een mislukte aanmelding volgt een korte wachttijd.
admin-diagx-check-security-signin-pacing-fix = Schakel aanmeldtempo in bij het beveiligingsbeleid.
admin-diagx-check-security-credential-locks = Vergrendeling van inloggegevens
admin-diagx-check-security-credential-locks-purpose = Herhaalde mislukte aanmeldingen vergrendelen de inloggegevens een tijd.
admin-diagx-check-security-credential-locks-fix = Schakel automatische vergrendeling in bij het Beveiligingscentrum.
admin-diagx-check-security-geography = Geografiegegevens voor aanmelding
admin-diagx-check-security-geography-purpose = Het geografiebeleid voor aanmelding heeft de landgegevens die het nodig heeft.
admin-diagx-check-security-geography-fix = Installeer of werk de landendatabase bij, of schakel het geografiebeleid uit.
admin-diagx-check-security-render-isolation = Geïsoleerde berichtweergave
admin-diagx-check-security-render-isolation-purpose = Berichten worden weergegeven in de aparte, geïsoleerde worker.
admin-diagx-check-security-render-isolation-fix = Controleer of MW_RENDER_BIN naar het programma mw-render wijst en of het start. De servicestatus op het Dashboard toont de status van de worker.
admin-diagx-check-security-deployment = Implementatie-instellingen
admin-diagx-check-security-deployment-purpose = De beveiligingsinstellingen van de implementatie kunnen worden gelezen.
admin-diagx-check-security-deployment-fix = Controleer de databaseverbinding en open daarna de beveiligingsbeoordeling van de implementatie.
admin-diagx-check-security-plaintext-backends = Wachtwoorden via onversleutelde verbindingen
admin-diagx-check-security-plaintext-backends-purpose = Geen enkele opgeslagen backend accepteert wachtwoorden zonder TLS.
admin-diagx-check-security-plaintext-backends-fix = Bewerk de verbindingen van het domein en kies TLS of STARTTLS. Sta gewone verbindingen alleen toe op een privénetwerk dat u beheert.
admin-diagx-check-fail2ban-client-address = Clientadres voor fail2ban
admin-diagx-check-fail2ban-client-address-purpose = Het adres dat een fail2ban-logregel zou noemen is het echte adres van de client.
admin-diagx-check-fail2ban-client-address-fix = Stel achter een reverse proxy MW_TRUSTED_PROXIES in op het adres van de proxy, anders bant fail2ban de proxy in plaats van de client. Een loopbackadres is normaal wanneer u de console op de server zelf opent.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = { $protocol }-endpoint: verbinding en beveiligingshandshake, zonder aan te melden.
admin-diagx-backend-other-purpose = Een opmerking uit de backenduitvoering.
admin-diagx-backend-fix = Controleer de hostnaam, poort en TLS-modus die voor deze backend zijn opgeslagen, of DNS wordt omgezet en of het certificaat geldig is voor de hostnaam. Voer daarna opnieuw uit.
admin-diagx-backend-config-fix = Open de verbindingen van het domein en sla ze opnieuw op; de lijst met opgeslagen backends kon niet worden gelezen.
admin-diagx-backend-results = Backendcontroles
admin-diagx-backends-not-run = De opgeslagen backends zijn nog niet gecontroleerd.
admin-diagx-backends-run-at = Laatst gecontroleerd { $date }: { $count } resultaten.
admin-diagx-failover-title = Failover
admin-diagx-failover-purpose = Of een domein kan overschakelen op een andere mailbackend wanneer de eerste keuze niet bereikbaar is.
admin-diagx-failover-state = Failover { $state ->
    [on] aan
   *[off] uit
} · { $backends } mailbackends{ $enabled ->
    [no] {" "}· domein uitgeschakeld
   *[yes] {""}
 }
admin-diagx-no-domains = Geen opgeslagen domeinen.
admin-diagx-domain-tools = Domeinhulpmiddelen
admin-diagx-domain-tools-purpose = Eenmalige controles voor een opgeslagen domein. Ze melden niet aan en versturen geen e-mail.
admin-diagx-domain = Domein
admin-diagx-tool-dns = DNS-records
admin-diagx-tool-dns-purpose = Zoekt de MX-, SPF-, DMARC- en servicerecords (SRV) van het domein op.
admin-diagx-tool-dns-fix = Voeg de ontbrekende records toe bij uw DNS-provider. Zonder MX ontvangt het domein geen e-mail; zonder SPF en DMARC kunnen andere servers de e-mail ervan weigeren.
admin-diagx-srv-found = { $found } van { $total } servicerecords gevonden
admin-diagx-tool-tls = TLS en handshake
admin-diagx-tool-tls-purpose = Maakt verbinding met elk opgeslagen endpoint van het domein en meldt TLS-versie, vervaldatum van het certificaat en benodigde tijd.
admin-diagx-tool-tls-fix = Controleer voor een onbereikbaar endpoint host, poort en firewall. Vernieuw een certificaat dat binnen 14 dagen verloopt.
admin-diagx-probe-none = Het domein heeft geen opgeslagen endpoints om te testen.
admin-diagx-probe-line = { $host } · bereikbaar: { $reachable ->
    [yes] ja
   *[no] nee
} · TLS { $tls } · certificaat { $days ->
    [none] niet gemeld
   *[other] verloopt over { $days } dagen
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
 }
admin-diagx-tool-detect = Instellingen ontdekken
admin-diagx-tool-detect-purpose = Zoekt gepubliceerde mailinstellingen (autoconfig, autodiscover, SRV) op voor een domeinnaam. Er wordt niets opgeslagen.
admin-diagx-tool-detect-fix = Publiceer autoconfig- of SRV-records voor het domein, of voer de instellingen handmatig in onder Domeinen.
admin-diagx-detect-name = Op te zoeken domeinnaam
admin-diagx-tool-health = Servicestatus
admin-diagx-tool-health-purpose = Mailsynchronisatie, de Postvak UIT, de zoekindex, de weergaveworker en plug-ins, op het Dashboard.
admin-diagx-tool-deployment-purpose = Elke beveiligingsinstelling van de implementatie die deze server kan waarnemen, met de instelling die haar bepaalt.
admin-diagx-tool-ssh = SSH-tunneltests
admin-diagx-tool-ssh-purpose = Test de SSH-tunnels waarmee backends op een ander netwerk worden bereikt.
admin-diagx-tool-geo = Aanmeldgeografie
admin-diagx-tool-geo-purpose = Waar aanmeldingen vandaan komen, en de landendatabase achter het geografiebeleid.
admin-diagx-catalogue-purpose = Wat elke diagnostiekreeks omvat en de limieten waaraan elke uitvoering zich houdt.
admin-diagx-suite-unsupported = Niet beschikbaar: { $reason }
admin-diagx-suite-server = Servercontroles
admin-diagx-suite-backends = Opgeslagen backends
admin-diagx-run-title = { $suite ->
    [backends] Opgeslagen backends
   *[server] Servercontroles
}, { $date }
admin-diagx-changes = Wijzigingen
admin-diagx-changes-first = Geen eerdere uitvoering van dit soort om mee te vergelijken.
admin-diagx-changes-none = Er is niets veranderd sinds de uitvoering van { $date }.
admin-diagx-changes-since = Veranderd sinds de uitvoering van { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Probleemoplossingsrapport
admin-diagx-report-purpose = Een JSON-rapport voor ondersteuning: runtimevlaggen, de nieuwste resultaten en de uitvoering die u kiest. Wachtwoorden, tokens, cookies, sleutels, ruwe configuratie en e-mail worden nooit opgenomen.
admin-diagx-report-run = Op te nemen uitvoering
admin-diagx-report-preview = Voorbeeld
admin-crumb-server = Server
admin-crumb-connectivity = Connectiviteit
admin-crumb-posture = Beveiligingsniveau
admin-crumb-tools = Hulpmiddelen
admin-crumb-history = Geschiedenis
admin-crumb-report = Rapport
admin-diagx-table-actions = Acties
admin-diagx-check-pdfjs-viewer-assets = Bestanden van de pdf-viewer
admin-diagx-check-pdfjs-viewer-assets-purpose = De bestanden van de pdf-viewer zijn allemaal aanwezig, correct getypeerd en komen overeen met deze serverbuild.
admin-diagx-check-pdfjs-viewer-assets-fix = Implementeer de webbuild opnieuw, of verwijder de map pdfjs uit MW_WEB_DIR zodat de in de server ingebouwde kopie wordt gebruikt. Als dit mislukt in plaats van waarschuwt, bouw dan de serverimage opnieuw.
