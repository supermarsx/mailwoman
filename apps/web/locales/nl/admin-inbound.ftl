# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Inkomende toegang
admin-inbound-help = Beperk vertrouwde bron-IP's voor de beheerconsole en webmail afzonderlijk. Beide beleidsregels staan aanvankelijk uit. Deze regels wijzigen vertrouwde proxy's, API-scopes of uitgaande toegang niet.
admin-inbound-unscoped = Dit beleid voor de hele installatie vereist een beheerder met ongescopete beveiligingstoegang.
admin-inbound-admin = Beheerconsole en API
admin-inbound-admin-help = Omvat inloggen en deze editor. Bestaande beheerdersauthenticatie omzeilt het netwerkbeleid niet.
admin-inbound-webmail = Webmail en account-API's
admin-inbound-webmail-help = Omvat mail, DAV, ondertekende inhoud, SSO, OIDC en API-routes. Bestaande opgewaardeerde verbindingen behouden de beslissing van hun handshake.
admin-inbound-enabled = Dit oppervlak beperken tot de toegestane lijst
admin-inbound-cidrs = Noodtoegangs-IP's en CIDR's
admin-inbound-cidrs-help = Eén letterlijk IP of CIDR per regel, maximaal 128. Er is er minimaal één vereist wanneer ingeschakeld. Neem een vertrouwde bron op die u voor herstel kunt gebruiken.
admin-inbound-hostnames = Toegestane DNS-hostnamen
admin-inbound-hostnames-help = Eén DNS-naam in kleine ASCII-letters per regel. Gebruik IDNA-ASCII voor internationale namen. Geen URL's, poorten, jokertekens, IP-literals of afsluitende punten. Namen geven alleen toegang aan de huidige opgeloste adressen.
admin-inbound-dns = Limieten voor DNS-resolutie
admin-inbound-dns-help = Nieuwe, verlopen en mislukte namen geven geen toegang. Geslaagde antwoorden respecteren hun DNS-TTL en de ingestelde cachelimiet. Verzoeken wachten nooit op DNS-resolutie.
admin-inbound-servers = Recursieve DNS-servers
admin-inbound-servers-help = Laat leeg voor de systeemresolvers, of voer maximaal vier IP:poort-sockets in, één per regel (bijvoorbeeld 10.0.0.53:53 of [fd00::53]:53). Expliciete servers vervangen de systeemresolvers zonder publieke terugval. Het vertrouwen in de resolver is uw verantwoordelijkheid; onafhankelijke DNSSEC-validatie wordt niet beweerd.
admin-inbound-dns-timeoutMs = Time-out voor opzoeken (milliseconden)
admin-inbound-dns-cacheSeconds = Maximale cachelevensduur (seconden)
admin-inbound-dns-maxHosts = Maximaal aantal verschillende hostnamen
admin-inbound-dns-maxAddresses = Maximaal aantal adressen per hostnaam
admin-inbound-dns-concurrency = Gelijktijdige opzoekingen
admin-inbound-bounds = Toegestaan bereik: { $min }–{ $max }.
admin-inbound-lockout = Dit beleid kan uw eigen toegang, inclusief deze editor, onmiddellijk blokkeren. Controleer dat de noodtoegangs-IP's uw werkelijke bron dekken, zoals bepaald door de ingestelde vertrouwde proxy's. Er is geen automatische loopback- of ingelogde-beheerderbypass. Herstel na een buitensluiting vereist herstel aan de kant van de installatie; gewoon inloggen kan dit niet omzeilen.
admin-inbound-confirm-title = Beleid voor inkomende toegang toepassen?
admin-inbound-confirm-word = TOEPASSEN
admin-inbound-save = Bevestigd beleid toepassen
admin-inbound-review = Wijzigingen controleren
admin-inbound-refresh = Opgeslagen beleid en DNS-status vernieuwen
admin-inbound-reset = Concept terugzetten naar het geladen beleid
admin-inbound-saved = De server heeft het beleid voor inkomende toegang geaccepteerd.
admin-inbound-conflict = Het opgeslagen beleid is gewijzigd terwijl u het bewerkte. Zet het concept terug en vernieuw om de huidige revisie te laden voordat u opnieuw bewerkt.
admin-inbound-no-revision = De server gaf geen beleidsrevisie terug. Vernieuw voordat u opslaat.
admin-inbound-status = Huidige DNS-toekenningen
admin-inbound-status-help = Dit is een momentopname van de laatste statusvernieuwing, geen aftelling of connectiviteitstest. Vernieuw om de huidige toekenningen te zien. Een fout kan naast een nog geldig eerder antwoord bestaan tot de oorspronkelijke vervaldatum.
admin-inbound-refreshing = Een begrensde DNS-vernieuwing is bezig.
admin-inbound-idle = Bij de laatste statusuitlezing liep er geen DNS-vernieuwing.
admin-inbound-no-hosts = Er zijn geen hostnamen actief in ingeschakelde beleidsregels.
admin-inbound-hostname = Hostnaam
admin-inbound-state = Toegangstoekenning
admin-inbound-addresses = Bruikbare adressen
admin-inbound-validity = Geldigheid bij laatste vernieuwing
admin-inbound-error = DNS-resultaatcode
admin-inbound-usable = Bruikbaar
admin-inbound-unavailable = Geen huidige toekenning
admin-inbound-seconds = { $seconds } seconden
admin-inbound-on = Ingeschakeld
admin-inbound-off = Uitgeschakeld
admin-inbound-invalid-dns = DNS-limieten of het aantal resolvers valt buiten de toegestane grenzen.
admin-inbound-rescue-required = Elk ingeschakeld oppervlak vereist minstens één noodtoegangs-IP of CIDR.
admin-inbound-too-many = Het beleid overschrijdt de ingestelde limiet voor hostnamen of letterlijke adressen.
admin-inbound-invalid-host = Gebruik DNS-namen in kleine ASCII-letters zonder URL's, poorten, jokertekens, IP-literals of afsluitende punten.
