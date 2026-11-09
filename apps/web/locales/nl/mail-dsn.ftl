# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Bezorgstatusrapport
dsn-unverified = Dit rapport beschrijft de bezorgpoging van de afzender. De inhoud is niet onafhankelijk geverifieerd. Bezorging betekent niet dat het bericht is gelezen.
dsn-reporting-mta = Rapporterende server:
dsn-original-recipient = Oorspronkelijke ontvanger:
dsn-remote-mta = Externe server:
dsn-last-attempt = Laatste poging:
dsn-retry-until = De afzender probeert opnieuw tot:
dsn-action-failed = Mislukking gemeld
dsn-action-delayed = Vertraging gemeld
dsn-action-delivered = Bezorging gemeld
dsn-action-relayed = Doorgifte gemeld
dsn-action-expanded = Lijstuitbreiding gemeld

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Gebruik de serverdiagnose wanneer u uw e-mailprovider om hulp vraagt.

dsn-advice-1 = Controleer zowel de afzender- als de ontvangeradressen. Bevestig indien nodig het huidige adres van de ontvanger.

dsn-advice-2 = Neem op een andere manier contact op met de ontvanger als het mailboxprobleem aanhoudt.

dsn-advice-3 = Deel dit rapport met de beheerder van het ontvangende e-mailsysteem als het probleem aanhoudt.

dsn-advice-4 = Vraag uw e-mailbeheerder om de connectiviteit, DNS en doorstuurroutes te controleren als het probleem aanhoudt.

dsn-advice-5 = Deel de diagnose met uw e-mailprovider, zodat zij de compatibiliteit en limieten van SMTP kunnen controleren.

dsn-advice-6 = Controleer het berichtformaat, de internationale adressen en bijlagen aan de eisen van de ontvanger.

dsn-advice-7 = Deel de diagnose met uw e-mailbeheerder om de exacte beveiligings- of authenticatieregel te identificeren.

dsn-advice-success = Controleer de gerapporteerde bezorgingsactie hierboven om te zien of het bericht is afgeleverd, doorgestuurd of uitgevouwen naar een lijst.

dsn-category-0 = De server heeft een niet-herkende of providerspecifieke detailcode geleverd. De onderstaande diagnose kan de exacte oorzaak verklaren.

dsn-category-1 = Dit is een adresgerelateerde code. De precieze details zijn aanbiederspecifiek of worden niet herkend.

dsn-category-2 = Dit is de mailboxcode van de ontvanger. De precieze details zijn aanbiederspecifiek of worden niet herkend.

dsn-category-3 = Dit is een code voor het ontvangende e-mailsysteem. De precieze details zijn aanbiederspecifiek of worden niet herkend.

dsn-category-4 = Dit is een netwerk- of routeringscode. De precieze details zijn aanbiederspecifiek of worden niet herkend.

dsn-category-5 = Dit is een mailprotocolcode. De precieze details zijn aanbiederspecifiek of worden niet herkend.

dsn-category-6 = Dit is een berichtinhoud- of formaatcode. De precieze details zijn aanbiederspecifiek of worden niet herkend.

dsn-category-7 = Dit is een beveiligings- of beleidscode. Het precieze detail is aanbiederspecifiek of wordt niet herkend; het bewijst op zichzelf niet dat het bericht spam is.

dsn-class-2 = Succes: de server meldt een positieve leveringsstap. Dit bewijst niet dat het bericht is gelezen.

dsn-class-4 = Tijdelijke mislukking: een latere poging kan slagen. Een vertragingsrapport kan nog steeds opnieuw worden geprobeerd door de afzender; een foutrapport betekent dat de poging is beëindigd.

dsn-class-5 = Permanente fout: het is onwaarschijnlijk dat hetzelfde bericht ongewijzigd opnieuw wordt verzonden. Verhelp de oorzaak voordat u opnieuw verzendt.

dsn-meaning-title = Wat deze code betekent

dsn-next-step = Wat te doen

dsn-provider-barracuda = Barracuda

dsn-provider-detected = Interpretatie op basis van de serverdiagnostiek of hostnaam:

dsn-provider-exchange = Microsoft Exchange/Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmail/Google Workspace

dsn-provider-help = Documentatie van de leverancier

dsn-provider-mimecast = Mimecast

dsn-provider-proofpoint = Bewijspunt

dsn-provider-relay = SMTP relais

dsn-provider-yahoo = Yahoo/AOL

dsn-server-diagnostic = Diagnostische server

dsn-status-0-0 = Een specifiekere reden gaf de server niet.

dsn-status-1-0 = Er is een probleem met een e-mailadres.

dsn-status-1-1 = Het postvak van de ontvanger bestaat niet. Bevestig het adres bij de ontvanger.

dsn-status-1-10 = Het ontvangende domein verklaart expliciet dat het geen e-mail accepteert (null MX). Gebruik een ander adres.

dsn-status-1-2 = Het ontvangende domein kan deze e-mail niet ontvangen. Controleer het onderdeel na @.

dsn-status-1-3 = Het adres van de ontvanger heeft een ongeldig formaat. Controleer op typefouten of extra spaties.

dsn-status-1-4 = De server kan geen unieke ontvanger voor dit adres identificeren.

dsn-status-1-5 = Het adres van de ontvanger werd als geldig geaccepteerd.

dsn-status-1-6 = Deze mailbox is verhuisd en er is geen doorstuuradres beschikbaar. Vraag naar het nieuwe adres.

dsn-status-1-7 = Het afzenderadres heeft een ongeldig formaat. Controleer het verzendadres van het account.

dsn-status-1-8 = Het afzenderdomein is ongeldig of kan geen retourmail ontvangen.

dsn-status-1-9 = Het bericht is doorgegeven aan een systeem dat geen verdere bezorgingsrapporten kan leveren.

dsn-status-2-0 = Het postvak van de ontvanger heeft een onbekend probleem.

dsn-status-2-1 = De mailbox van de ontvanger is uitgeschakeld of accepteert geen berichten. Neem op een andere manier contact op met de ontvanger.

dsn-status-2-2 = De mailbox van de ontvanger is vol. De ontvanger moet ruimte vrijmaken of zijn/haar quotum verhogen.

dsn-status-2-3 = Het bericht overschrijdt de maximale grootte van deze ontvanger. Verminder bijlagen of gebruik een deellink.

dsn-status-2-4 = De server kan de leden van een mailinglijst niet identificeren. Neem contact op met de lijsteigenaar.

dsn-status-3-0 = Het ontvangende e-mailsysteem meldde een niet-gespecificeerd probleem.

dsn-status-3-1 = De ontvangende mailserver heeft geen opslagruimte meer. De beheerder moet ruimte vrijmaken.

dsn-status-3-2 = De ontvangende server accepteert geen mail, bijvoorbeeld tijdens onderhoud of overbelasting.

dsn-status-3-3 = Het ontvangende systeem ondersteunt een functie die door dit bericht wordt vereist niet.

dsn-status-3-4 = Het bericht overschrijdt de maximale grootte van de ontvangende server. Verminder bijlagen of gebruik een deellink.

dsn-status-3-5 = Een configuratieprobleem met de mailserver verhindert de bezorging. Vraag de e-mailbeheerder om onderzoek te doen.

dsn-status-3-6 = De server heeft het bericht geaccepteerd, maar de gevraagde bezorgingsprioriteit gewijzigd.

dsn-status-4-0 = Er is zonder verdere details een netwerk- of e-mailrouteringsprobleem gemeld.

dsn-status-4-1 = De ontvangende server heeft de verbindingspoging niet beantwoord.

dsn-status-4-2 = De verbinding werd tot stand gebracht, maar werd onderbroken voordat de e-mailoverdracht was voltooid.

dsn-status-4-3 = Het opzoeken van DNS of een directory is mislukt. Een e-mailbeheerder moet mogelijk de naamomzetting controleren.

dsn-status-4-4 = De server kon geen route vinden naar het mailsysteem van de ontvanger.

dsn-status-4-5 = Door files op het postverkeer werd de bezorging verhinderd.

dsn-status-4-6 = Er is een doorstuurlus gedetecteerd. Vraag de beheerder om de doorstuur- en routeringsregels te controleren.

dsn-status-4-7 = De levertijd is verstreken. Controleer de diagnose van het onderliggende probleem voordat u het opnieuw verzendt.

dsn-status-5-0 = De mailservers konden de uitwisseling van het bezorgingsprotocol niet voltooien.

dsn-status-5-1 = Een mailserver heeft een niet-ondersteunde opdracht afgewezen of een opdracht die in de verkeerde volgorde is verzonden.

dsn-status-5-2 = Een mailserver kon de syntaxis van een opdracht niet begrijpen.

dsn-status-5-3 = Het bericht heeft te veel ontvangers. Stuur naar kleinere groepen.

dsn-status-5-4 = Een mailserver heeft de parameters van een commando afgewezen.

dsn-status-5-5 = De servers gebruiken incompatibele versies van het mailprotocol.

dsn-status-5-6 = Het authenticatieantwoord was te lang voor de server. Vraag de e-mailbeheerder om de authenticatie-instellingen te controleren.

dsn-status-6-0 = De server heeft een niet-gespecificeerd probleem gemeld met de inhoud of het formaat van het bericht.

dsn-status-6-1 = Het ontvangende systeem ondersteunt het inhoudstype van dit bericht niet.

dsn-status-6-10 = Deze verouderde code betekent dat een internationaal (UTF-8) antwoord vereist is, maar niet wordt ondersteund, zoals bij code x.6.8.

dsn-status-6-2 = Voor de levering is een inhoudsconversie vereist die het afzender- of serverbeleid verbiedt.

dsn-status-6-3 = Voor de levering is een contentconversie vereist die de server niet kan uitvoeren.

dsn-status-6-4 = Bij een inhoudsconversie is informatie verloren gegaan. Controleer het originele bericht en de serverdiagnostiek.

dsn-status-6-5 = De server is mislukt tijdens het converteren van de berichtinhoud.

dsn-status-6-6 = De server kon de berichtinhoud niet ophalen van een ander systeem.

dsn-status-6-7 = Een internationaal e-mailadres wordt niet ondersteund voor deze afzender of ontvanger. Vraag om een ​​alternatief adres.

dsn-status-6-8 = De server heeft een internationaal (UTF-8) antwoord nodig dat het verzendende systeem niet kan accepteren.

dsn-status-6-9 = Het bericht bevat internationale (UTF-8) headers die niet aan ten minste één ontvanger kunnen worden afgeleverd.

dsn-status-7-0 = Een beveiligingsregel of postbeleid verhinderde een normale bezorging. Controleer de serverdiagnose voor de exacte regel.

dsn-status-7-1 = De server heeft geen toestemming voor bezorging geweigerd, mogelijk vanwege filter- of relaybeperkingen. Vraag de e-mailbeheerder om de diagnose te controleren.

dsn-status-7-10 = De server vereist een gecodeerde verbinding of sterkere bescherming vóór authenticatie.

dsn-status-7-11 = Deze authenticatiemethode vereist een gecodeerde SMTP-verbinding (TLS).

dsn-status-7-12 = Het account heeft een authenticatiemethode of wachtwoordmigratie nodig. Neem contact op met de accountaanbieder.

dsn-status-7-13 = Het verzendende account is uitgeschakeld. Neem contact op met de beheerder om de toegang te herstellen.

dsn-status-7-14 = De server heeft een geconfigureerde vertrouwensrelatie nodig om het bericht op te halen. Neem contact op met de beheerder.

dsn-status-7-15 = De berichtprioriteit ligt onder het huidige minimum van de ontvangende server.

dsn-status-7-16 = Het bericht is te groot voor de gevraagde prioriteit. Verklein de grootte of vraag het aan de beheerder.

dsn-status-7-17 = Het postvak van de ontvanger is sinds de vereiste datum van eigenaar veranderd. Bevestig de ontvanger voordat u opnieuw verzendt.

dsn-status-7-18 = Het ontvangende domein is sinds de vereiste datum van eigenaar veranderd. Bevestig de bestemming voordat u opnieuw verzendt.

dsn-status-7-19 = De server kan het voortdurende eigendom van het adres van de ontvanger niet verifiëren. Vraag de beheerder naar de geldigheidscontrole van de ontvanger.

dsn-status-7-2 = Het is de afzender niet toegestaan om op deze mailinglijst te posten. Neem contact op met de lijsteigenaar.

dsn-status-7-20 = Geen enkele DKIM-handtekening heeft de verificatie doorstaan. Vraag de e-mailbeheerder van de afzender om de ondertekening en DNS te controleren.

dsn-status-7-21 = Er is een DKIM-handtekening doorgegeven, maar het beleid van de ontvangende server heeft deze niet geaccepteerd.

dsn-status-7-22 = Geen enkele geaccepteerde DKIM-handtekening komt overeen met het Van-adres. Vraag de e-mailbeheerder om de uitlijning van het ondertekeningsdomein te controleren.

dsn-status-7-23 = SPF mislukt: de verzendende server is niet geautoriseerd door het SPF beleid van het afzenderdomein.

dsn-status-7-24 = De server heeft een fout aangetroffen bij het controleren van SPF. Vraag de e-mailbeheerder om de SPF-records en DNS te controleren.

dsn-status-7-25 = De verzendende server slaagde niet voor de omgekeerde DNS-controle van de ontvanger. De beheerder ervan moet de DNS-records van het adres IP controleren.

dsn-status-7-26 = Meerdere verificatiecontroles van berichten zijn mislukt. De diagnose kan SPF, DKIM of DMARC beleidsproblemen identificeren.

dsn-status-7-27 = Het afzenderdomein verklaart dat het geen retour-e-mail accepteert (null MX), wat de ontvanger afwijst.

dsn-status-7-28 = Het ontvangende systeem heeft een mogelijke stroom aan beledigende berichten gedetecteerd.

dsn-status-7-29 = ARC-verificatie is mislukt, dus de ontvanger kon de opgenomen authenticatieketen van het bericht niet accepteren.

dsn-status-7-3 = Voor de levering is een conversie van het beveiligingsformaat vereist die de server niet kan uitvoeren.

dsn-status-7-30 = Het bericht vereist REQUIRETLS, maar de volgende mailserver ondersteunt dit niet. Vraag de beheerder om ondersteuning voor beveiligde bezorging te controleren.

dsn-status-7-4 = Het toedieningssysteem ondersteunt een vereiste beveiligingsfunctie niet.

dsn-status-7-5 = De server kon het bericht niet valideren of decoderen, bijvoorbeeld omdat een sleutel ontbrak of ongeldig was.

dsn-status-7-6 = De server ondersteunt geen cryptografisch algoritme dat vereist is voor het bericht.

dsn-status-7-7 = Een integriteitscontrole geeft aan dat het bericht is gewijzigd of beschadigd.

dsn-status-7-8 = De verzendende server heeft de inloggegevens afgewezen. Controleer de authenticatie-instellingen van het account.

dsn-status-7-9 = De authenticatiemethode is zwakker dan de server toestaat. Er is een sterkere methode nodig.

dsn-vendor-advice-account = Vraag de verzendende beheerder om mogelijk accountmisbruik te onderzoeken, het account te beveiligen en om verwijdering van de beperking aan te vragen.

dsn-vendor-advice-admin = Deel deze diagnose met de e-mailbeheerder van de organisatie om de overeenkomende configuratie of het overeenkomende beleid te bekijken.

dsn-vendor-advice-auth = Controleer de ondersteunde SMTP authenticatiemethode en autorisatie van het account. Voltooi de vereiste aanmelding via de officiële accountpagina van de provider.

dsn-vendor-advice-authentication = Vraag de beheerder van de afzender om SPF, DKIM, DMARC en domeinuitlijning te controleren met behulp van de exacte diagnose.

dsn-vendor-advice-connector = Vraag de beheerder om de relay-autorisatie, connector IP/certificaatmatching, geaccepteerde domeinen en de daadwerkelijke e-mailroute te controleren.

dsn-vendor-advice-content = Bekijk de diagnostische en gatewaylogboeken samen met de beheerder van de ontvanger om de geblokkeerde inhoud en het toepasselijke beleid te identificeren.

dsn-vendor-advice-dns = Vraag de verzendende e-mailbeheerder of IP-provider om DNS te corrigeren, DNS en de SMTP-serveridentiteit om te keren.

dsn-vendor-advice-domain = Vraag de Microsoft 365-beheerder om het verzendende domein te registreren en te verifiëren en de verzendlimieten van de tenant te bekijken.

dsn-vendor-advice-format = Vraag de verzendende toepassing of e-mailbeheerder om het genereren van berichten te corrigeren en maak vervolgens een nieuw bericht.

dsn-vendor-advice-rate = Verlaag het verzendvolume en wacht tot het quotum of de beperking is opgeheven. Controleer de gerapporteerde actie voordat u ervan uitgaat dat deze poging nog steeds in de wachtrij staat.

dsn-vendor-advice-recipient-admin = Neem op een andere manier contact op met de ontvanger of groepseigenaar en vraag de beheerder om de relevante toestemming of het relevante beleid te bekijken.

dsn-vendor-advice-reputation = Vraag de verzendende beheerder om ongewenst verkeer en de reputatie te onderzoeken en volg vervolgens het officiële beoordelings- of verwijderingsproces van de provider.

dsn-vendor-advice-route = Vraag de e-mailbeheerder om de bestemming, doorstuurregels, e-mailservice en firewall/netwerkpad te controleren.

dsn-vendor-advice-tls = Vraag de e-mailbeheerders om TLS, certificaten of DNS beveiligingsrecords te repareren, zodat de vereiste veilige route kan slagen.

dsn-vendor-barracuda-reputation = De reputatieservice van Barracuda vermeldt of blokkeert het verzendende IP-adres.

dsn-vendor-certificate-chain = Het externe TLS-certificaat kon niet worden gevalideerd op basis van het vereiste vertrouwen of MTA-STS-beleid.

dsn-vendor-certificate-expired = De doelmailserver presenteerde een verlopen TLS-certificaat.

dsn-vendor-certificate-host = Het doelcertificaat TLS komt niet overeen met de vereiste hostnaam van de mailserver.

dsn-vendor-connection-refused = De doelserver heeft de netwerkverbinding geweigerd. De mailservice, firewall of netwerkroute moet worden gecontroleerd.

dsn-vendor-content-security = De gateway heeft de berichtinhoud of een bijlage afgewezen op grond van de beveiligingsfilterregels. Raadpleeg de diagnose of logboeken voor het specifieke item.

dsn-vendor-credentials = De service SMTP heeft de opgegeven inloggegevens afgewezen.

dsn-vendor-dane = Het TLS-certificaat of de TLSA-configuratie van de bestemming voldeed niet aan de DANE-validatie.

dsn-vendor-dkim = De ontvanger heeft de DKIM authenticatie- of ondertekeningsconfiguratie van het bericht niet geaccepteerd.

dsn-vendor-dmarc = Er werd niet voldaan aan het DMARC beleid van de ontvanger. Controleer SPF of DKIM authenticatie en afstemming met het zichtbare Van-domein.

dsn-vendor-dnssec = De DNSSEC-records van het doeldomein konden niet worden gevalideerd.

dsn-vendor-exchange-auth = De toepassing of het apparaat is niet correct geverifieerd voor de indiening van Microsoft 365 SMTP.

dsn-vendor-exchange-connector = Microsoft kan de relay niet autoriseren via de geconfigureerde tenantconnector. Het certificaat, de verzendende IP of het geaccepteerde domein komen mogelijk niet meer overeen.

dsn-vendor-exchange-domain-blocked = Microsoft heeft dit verzendende domein beperkt nadat vermoedelijke spamactiviteiten zijn gedetecteerd.

dsn-vendor-exchange-forward-relay = Een stroomafwaarts systeem weigerde doorgestuurde post. De verificatie is mogelijk beïnvloed door doorsturen of een niet-Microsoft-gateway.

dsn-vendor-exchange-forwarding = Microsoft 365-beleid blokkeert automatisch doorsturen naar externe ontvangers. De beheerder van de organisatie moet het doorstuurbeleid beoordelen.

dsn-vendor-exchange-generic = Exchange meldde een algemene afwijzing van de levering. De diagnose identificeert het werkelijke probleem, waarbij mogelijk sprake is van berichtopmaak of bijlagebeleid.

dsn-vendor-exchange-group-allow = De afzender staat niet in de lijst met toegestane afzenders van deze Exchange-groep.

dsn-vendor-exchange-group-internal = Deze Exchange-distributiegroep weigert afzenders buiten de organisatie, tenzij de eigenaar of beheerder hen toestemming geeft.

dsn-vendor-exchange-internal-only = Deze Exchange-ontvanger accepteert geverifieerde afzenders uit de eigen organisatie; dit bericht werd behandeld als extern of niet-geverifieerd.

dsn-vendor-exchange-ipv6 = De bestemming accepteert deze levering boven IPv6 niet. De verzendende beheerder moet de route controleren of een geaccepteerd transportmiddel gebruiken.

dsn-vendor-exchange-journaling = De organisatie probeert on-premises berichten in Microsoft 365 te loggen zonder de vereiste archiefconfiguratie.

dsn-vendor-exchange-onmicrosoft-limit = De organisatie heeft de limiet voor externe ontvangers bereikt voor het onmicrosoft.com verzendende domein. Bij reguliere verzending moet een geverifieerd aangepast domein worden gebruikt.

dsn-vendor-exchange-public-folder = Deze openbare Exchange-map accepteert deze externe of niet-geverifieerde afzender niet.

dsn-vendor-exchange-receive-rate = Deze Exchange-ontvanger heeft de ontvangstlimiet per uur voor alle afzenders bereikt.

dsn-vendor-exchange-recipient = Exchange kon deze ontvanger niet omzetten in zijn directory. Controleer het adres en verwijder verouderde vermeldingen voor automatisch aanvullen voordat u een nieuw bericht maakt.

dsn-vendor-exchange-recipient-access = Exchange heeft de toegang tot dit ontvanger- of bestemmingsdomein geweigerd. Controleer het adres van de ontvanger en de geaccepteerde domeinconfiguratie van de organisatie.

dsn-vendor-exchange-recipient-rate = Exchange heeft binnen het uurvenster te veel berichten van deze afzender naar deze ontvanger ontvangen.

dsn-vendor-exchange-sender-blocked = Microsoft heeft dit verzendende account beperkt nadat vermoedelijke spam of misbruik is gedetecteerd. Hiervoor kan een beheerder nodig zijn om het account te beveiligen en te deblokkeren.

dsn-vendor-exchange-temporary-restriction = Microsoft beperkt dit verzenden van IP of verkeer tijdelijk terwijl verdachte activiteiten worden geëvalueerd.

dsn-vendor-exchange-tenant-blocked = Microsoft heeft de verzending voor deze organisatie beperkt omdat het verkeer ervan als verdacht werd beschouwd.

dsn-vendor-exchange-tenant-blocklist = Het adres of domein van een ontvanger wordt geblokkeerd door de Tenant Allow/Block List van de afzenderorganisatie. Eén geblokkeerde bestemming kan het hele bericht weigeren.

dsn-vendor-exchange-tenant-limit = De Microsoft 365-organisatie heeft de voortschrijdende dagelijkse limiet voor externe ontvangers bereikt.

dsn-vendor-exchange-unregistered = Microsoft heeft verzending geweigerd vanaf een domein dat niet correct is geregistreerd en geverifieerd in de tenant.

dsn-vendor-expired = De verzendservice gaf het op nadat de levertijd was verstreken. Controleer de diagnose voor de verbindings- of routeringsfout die de vertraging heeft veroorzaakt.

dsn-vendor-fortimail-relay = FortiMail heeft deze bezorging geweigerd op grond van haar toegangscontrole- of relaisbeleid. De gatewaybeheerder moet de overeenkomende ontvangstregel controleren.

dsn-vendor-gmail-app-password = Google vereist een app-wachtwoord of een ondersteunde accountautorisatiemethode voor deze SMTP-aanmelding.

dsn-vendor-gmail-auth-method = Google heeft een niet-ondersteunde authenticatiemethode afgewezen, zoals de verouderde XOAUTH-methode.

dsn-vendor-gmail-authentication = Google heeft de authenticatie van de afzender niet geaccepteerd. De diagnose maakt onderscheid tussen ontbrekende SPF/DKIM, SPF harde mislukking en DMARC afwijzing.

dsn-vendor-gmail-browser-login = Google vereist een accountaanmelding of een veiligheidscontrole in de browser voordat deze SMTP-aanmelding wordt toegestaan.

dsn-vendor-gmail-custom-policy = De aangepaste e-mailregel van een Google Workspace-beheerder heeft het bericht afgewezen. Dit betekent niet noodzakelijkerwijs dat het spamfilter van Google dit heeft afgewezen.

dsn-vendor-gmail-dmarc-record = Google vereist een DMARC-record met een gedeclareerd beleid voor dit verzendende domein.

dsn-vendor-gmail-rate = Google heeft een verzend- of doorstuurlimiet toegepast of een ongebruikelijke berichtsnelheid gedetecteerd. De diagnose identificeert het betreffende account, IP of domein.

dsn-vendor-gmail-relay = Google Workspace heeft de verzendende IP, het domein of het account niet geautoriseerd voor de SMTP relay-service.

dsn-vendor-gmail-reputation = De filtering van Google heeft het bericht afgewezen omdat de e-mail ongevraagd was of de reputatie van de verzendende IP/domein te laag was.

dsn-vendor-gmail-spf-record = Google beschouwt de SPF-configuratie van het afzenderdomein als ongeldig of verdacht.

dsn-vendor-greylist = De gateway heeft deze levering tijdelijk uitgesteld met behulp van grijze lijsten. Een correct geconfigureerde verzendende server probeert normaal gesproken later opnieuw.

dsn-vendor-headers = De ontvangende service heeft ongeldige of ontbrekende berichtkoppen afgewezen. De verzendende applicatie moet correct opgemaakte e-mail genereren.

dsn-vendor-helo = De verzendende server gebruikte een onaanvaardbare identiteit HELO/EHLO. De begroeting moet de verzendende server correct identificeren.

dsn-vendor-ip-block = De ontvangende service heeft de verzendende IP afgewezen vanwege zijn reputatie of een vermelding op de blokkeerlijst.

dsn-vendor-mimecast-block = Mimecast heeft de afzender of het domein afgewezen op grond van een geblokkeerde afzender of administratief beleid.

dsn-vendor-mimecast-content = Een inhoudonderzoeksbeleid van Mimecast verwierp het bericht.

dsn-vendor-mimecast-spoof = Het anti-spoofingbeleid van Mimecast verwierp de identiteit van de afzender.

dsn-vendor-mta-sts-host = De bestemmingsmailserver is niet toegestaan volgens het MTA-STS-beleid van het domein.

dsn-vendor-proofpoint-block = Proofpoint blokkeerde de verzendende IP of het domein na het detecteren van spam of kwaadwillige activiteiten.

dsn-vendor-proofpoint-throttle = Proofpoint heeft deze verzending IP tijdelijk uitgesteld omdat het verzendpatroon als ongebruikelijk werd beschouwd.

dsn-vendor-ptr = De verzendende IP heeft geen acceptabele omgekeerde DNS, of de PTR naam wordt niet terugveranderd naar dezelfde IP.

dsn-vendor-recipient-ip-block = De ontvangende organisatie blokkeert expliciet het verzendende IP-adres. De beheerder beheert dit lokale blok.

dsn-vendor-relay = De mailserver stuurt dit bericht niet door naar de bestemming. Controleer SMTP authenticatie, vertrouwde verzendende IP's, connectoren en geaccepteerde domeinen.

dsn-vendor-routing-loop = Het bericht is in een doorstuur- of routeringslus terechtgekomen en heeft het toegestane aantal hops overschreden.

dsn-vendor-sending-limit = Het verzendende account heeft een bericht- of ontvangerquotum overschreden. Herhaaldelijke, onmiddellijke herverzendingen zullen de limiet niet verwijderen.

dsn-vendor-spf = De ontvanger heeft geen SPF-authenticatie geaccepteerd voor het verzendende IP- en envelop-afzenderdomein.

dsn-vendor-starttls = De vereiste gecodeerde SMTP-verbinding kon niet tot stand worden gebracht of werd niet ondersteund door de volgende server.

dsn-vendor-yahoo-deferred = Yahoo heeft dit bericht tijdelijk uitgesteld, meestal tijdens het evalueren van klachten over volume, reputatie of ontvangers. Controleer de diagnose voor de aangegeven oorzaak.

dsn-vendor-yahoo-policy = Yahoo heeft het bericht afgewezen op grond van zijn e-mailbeleid. Er kan sprake zijn van authenticatie, DMARC-uitlijning of berichtinhoud; de diagnose geeft de specifieke reden aan.
