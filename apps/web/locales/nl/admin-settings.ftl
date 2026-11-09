# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Verzenden toestaan
admin-settings-help-mail-allow-send = Laat gebruikers berichten verzenden. Als dit uit staat, schakelt de editor Verzenden en de sneltoets uit en weigert de server verzendingen; concepten kunnen nog steeds worden opgeslagen.
admin-settings-key-mail-allow-reply = Beantwoorden toestaan
admin-settings-help-mail-allow-reply = Laat gebruikers beantwoorden en allen beantwoorden vanuit de leesweergave en met sneltoetsen. Als dit uit staat, worden die acties niet aangeboden.
admin-settings-key-mail-allow-forward = Doorsturen toestaan
admin-settings-help-mail-allow-forward = Laat gebruikers berichten inline of als bijlage doorsturen. Als dit uit staat, worden die acties niet aangeboden.
admin-settings-key-mail-allow-download = Downloads toestaan
admin-settings-help-mail-allow-download = Laat gebruikers berichten en bijlagen downloaden. Als dit uit staat, biedt de webapp geen download aan en weigert de server het exporteren van berichten.
admin-settings-key-mail-allow-print = Afdrukken toestaan
admin-settings-help-mail-allow-print = Laat gebruikers een bericht afdrukken vanuit de leesweergave. Als dit uit staat, wordt de actie Afdrukken niet aangeboden.
admin-settings-key-mail-show-spam = De map Spam tonen
admin-settings-help-mail-show-spam = Toon de map Spam of Junk en berichten die als spam zijn gemarkeerd. Als dit uit staat, zijn ze verborgen in de maplijst en in zoekresultaten; er wordt niets verwijderd.
admin-settings-key-mail-show-auth-status = Verificatieresultaten van de afzender tonen
admin-settings-help-mail-show-auth-status = Toon de SPF-, DKIM- en DMARC-resultaten uit de header Authentication-Results in de leesweergave.
admin-settings-key-mail-close-on-delete = Een bericht sluiten wanneer het wordt verwijderd
admin-settings-help-mail-close-on-delete = Sluit het geopende bericht, tabblad of losgekoppelde venster wanneer dat bericht wordt verwijderd of naar de prullenbak wordt verplaatst. Als dit uit staat, behoudt de leesweergave haar eerdere gedrag.
admin-settings-key-links-trusted-senders = Vertrouwde linkafzenders
admin-settings-help-links-trusted-senders = Exacte afzenderpostvakken waarvan gecontroleerde berichtlinks zonder toestemming per bericht mogen worden geopend. Eén adres per item.
admin-settings-key-appearance-font-size = Lettergrootte van de interface
admin-settings-help-appearance-font-size = Lettergrootte van de interface in pixels, van 12 tot 24.
admin-settings-key-appearance-font-weight = Letterdikte van de interface
admin-settings-help-appearance-font-weight = Basisdikte van de interfacetekst: 300, 400, 500 of 600. Benadrukte tekst behoudt zijn hiërarchie.
admin-settings-key-appearance-font-color = Tekstkleur van de interface
admin-settings-help-appearance-font-color = Tekstkleur van de interface als #RRGGBB. Laat leeg om het thema te volgen.
admin-settings-key-appearance-low-contrast = Tekstcontrast verzachten
admin-settings-help-appearance-low-contrast = Verzacht het contrast van de interfacetekst.
admin-settings-key-mail-message-list-order = Volgorde van de berichtenlijst
admin-settings-help-mail-message-list-order = Volgorde van de berichtenlijst: nieuwste, oudste, afzender of onderwerp.
admin-settings-key-mail-message-list-style = Stijl van de berichtenlijst
admin-settings-help-mail-message-list-style = Toon de berichtenlijst als kaarten of als rijen met kolomkoppen.
admin-settings-key-mail-message-list-preview = Voorbeelden in de berichtenlijst
admin-settings-help-mail-message-list-preview = Toon fragmenten van de berichttekst in de berichtenlijst.
admin-settings-key-mail-after-delete = Na het verwijderen van een bericht
admin-settings-help-mail-after-delete = Welk bericht de leesweergave toont nadat het geopende bericht is verwijderd: het volgende, het vorige of geen.
admin-settings-key-reader-attachments-at-end = Bijlagen na de berichttekst
admin-settings-help-reader-attachments-at-end = Toon bijlagen na de berichttekst in plaats van ervoor.
admin-settings-key-notifications-ask-permission = Toestemming voor meldingen aanbieden
admin-settings-help-notifications-ask-permission = Bied de toestemmingsvraag voor browsermeldingen aan gebruikers op apparaten die meldingen niet hebben toegestaan of geblokkeerd.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Instellingen
admin-settings-description = Waarden voor de hele server, per domein en per gebruiker. Een vergrendelde waarde kan niet worden gewijzigd door gebruikers of door engere bereiken.
admin-settings-list-caption = Instellingen
admin-settings-search = Instellingen zoeken
admin-settings-empty = Geen instelling komt overeen met de zoekopdracht.
admin-settings-unavailable = Deze server biedt het instellingenmodel nog niet aan.
admin-settings-col-setting = Instelling
admin-settings-col-value = Waarde
admin-settings-col-source = Bron
admin-settings-col-overrides = Engere waarden
admin-settings-open = Openen
admin-settings-overrides-none = Geen
admin-settings-overrides = { $domains } domeinen, { $users } gebruikers
admin-settings-not-found = Er is geen instelling met de naam { $key }.
admin-settings-value-on = Aan
admin-settings-value-off = Uit
admin-settings-value-empty = Niet ingesteld
admin-settings-value-list = { $count ->
    [one] { $count } item
   *[other] { $count } items
 }
admin-settings-detail-key = Sleutel: { $key }
admin-settings-global = Voor de hele server
admin-settings-global-help = Deze waarde geldt voor elke gebruiker, tenzij een domein of een gebruiker een engere waarde heeft, of de gebruiker waar dat is toegestaan een eigen waarde kiest.
admin-settings-value-label = Waarde
admin-settings-json-hint = JSON. De server controleert de waarde en wijst het deel aan dat hij weigert.
admin-settings-json-invalid = Dit is geen geldige JSON.
admin-settings-user-override = Gebruikers mogen dit wijzigen
admin-settings-impact-user = { $count ->
    [one] { $count } eigen keuze van een gebruiker wordt genegeerd
   *[other] { $count } eigen keuzes van gebruikers worden genegeerd
 }
admin-settings-impact-admin-user = { $count ->
    [one] { $count } waarde per gebruiker wordt genegeerd
   *[other] { $count } waarden per gebruiker worden genegeerd
 }
admin-settings-impact-profile = { $count ->
    [one] { $count } waarde van een functieprofiel wordt genegeerd
   *[other] { $count } waarden van functieprofielen worden genegeerd
 }
admin-settings-impact-group = { $count ->
    [one] { $count } groepswaarde wordt genegeerd
   *[other] { $count } groepswaarden worden genegeerd
 }
admin-settings-domains = Domeinwaarden
admin-settings-domains-help = Een domeinwaarde geldt voor de gebruikers van dat domein in plaats van de waarde voor de hele server.
admin-settings-domains-caption = Domeinwaarden van { $setting }
admin-settings-domains-empty = Geen enkel domein heeft een eigen waarde.
admin-settings-domain = Domein
admin-settings-domain-value = Waarde voor dit domein
admin-settings-domain-locked = Vergrendeld voor de gebruikers van het domein
admin-settings-domain-add = Domeinwaarde toevoegen
admin-settings-domain-save = Domeinwaarde opslaan
admin-settings-domain-remove = Verwijderen
admin-settings-domain-remove-title = De waarde van { $domain } verwijderen?
admin-settings-domain-remove-body = De gebruikers van { $domain } krijgen weer de waarde voor de hele server.
admin-settings-domain-required = Voer een domeinnaam in.
admin-settings-locked-yes = Vergrendeld
admin-settings-locked-no = Niet vergrendeld
admin-settings-users = Waarden per gebruiker
admin-settings-users-help = Een waarde die een beheerder voor één gebruiker heeft ingesteld. Bij limieten wordt de gebruiker met het adres aangeduid.
admin-settings-users-caption = Waarden per gebruiker van { $setting }
admin-settings-users-empty = Geen enkele gebruiker heeft een door een beheerder ingestelde waarde.
admin-settings-user = Gebruiker
admin-settings-user-hint = Account-id, of bij limieten het adres.
admin-settings-user-value = Waarde voor deze gebruiker
admin-settings-user-add = Waarde per gebruiker toevoegen
admin-settings-user-save = Gebruikerswaarde opslaan
admin-settings-user-required = Voer een gebruiker in.
admin-settings-user-remove-title = De waarde van { $user } verwijderen?
admin-settings-user-remove-body = De gebruiker krijgt de waarde van het domein of de waarde voor de hele server.
admin-settings-source-default = Ingebouwde standaardwaarde
admin-settings-source-envFallback = Omgevingsvariabele (geen opgeslagen waarde)
admin-settings-source-envPin = Omgevingsvariabele
admin-settings-source-global = Opgeslagen voor de hele server
admin-settings-cat-interface = Interface
admin-settings-cat-appearance = Uiterlijk
admin-settings-cat-mail = E-mail
admin-settings-cat-folders = Maplay-out
admin-settings-cat-notifications = Meldingen
admin-settings-cat-offline = Offline gebruik
admin-settings-cat-assistant = AI-assistent
admin-settings-cat-upstream = Mailserver
admin-settings-cat-signin = Inloggen
admin-settings-cat-admin = Beheerconsole
admin-settings-cat-passwords = Wachtwoorden
admin-settings-cat-limits = Limieten en gebruik
admin-settings-cat-plugins = Plug-ins
admin-settings-cat-features = Functierechten
admin-settings-cat-integrations = Integratierechten
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Sta aan het account gekoppelde Nextcloud-bestanden, delen en agenda-, contact- en taakbewerkingen toe.
admin-settings-key-cap-integration-google = Google-brug
admin-settings-help-cap-integration-google = Sta bewerkingen van de Google-brug toe, inclusief het verkrijgen en vernieuwen van tokens.
admin-settings-key-cap-integration-microsoft = Microsoft-brug
admin-settings-help-cap-integration-microsoft = Sta bewerkingen van de Microsoft-brug toe, inclusief het verkrijgen en vernieuwen van tokens.
admin-settings-key-cap-integration-family = Afzonderlijke integratie
admin-settings-help-cap-integration-family = Recht voor de identificatie van een afzonderlijke integratie; de dekking tijdens uitvoering staat in de functiecatalogus.
admin-settings-key-cap-ui-plugin-family = Afzonderlijke UI-plug-in
admin-settings-help-cap-ui-plugin-family = Recht voor één UI-plug-in. Gebruik de exacte beleidssleutel uit de integratiecatalogus.
admin-settings-key-cap-ai-assistant = AI-assistent
admin-settings-help-cap-ai-assistant = Sta configuratie en aanroep van de assistent toe.
admin-settings-key-cap-plugins = Plug-ins
admin-settings-help-cap-plugins = Sta bewerkingen met UI- en engine-plug-ins toe.
admin-settings-key-cap-pim-calendar = Agenda
admin-settings-help-cap-pim-calendar = Sta agendamethoden en abonnementen toe.
admin-settings-key-cap-pim-contacts = Contacten
admin-settings-help-cap-pim-contacts = Sta methoden voor contacten en adresboeken toe.
admin-settings-key-cap-pim-tasks = Taken
admin-settings-help-cap-pim-tasks = Sta taakmethoden toe.
admin-settings-key-cap-pim-notes = Notities
admin-settings-help-cap-pim-notes = Sta notitiemethoden toe.
admin-settings-key-cap-export = Exporteren
admin-settings-help-cap-export = Sta e-mail- en PIM-exports toe.
admin-settings-key-cap-import = Importeren
admin-settings-help-cap-import = Sta e-mail- en PIM-imports toe.
admin-settings-key-cap-api-keys = API-sleutels
admin-settings-help-cap-api-keys = Beleid voor API-sleutels. Raadpleeg de functiecatalogus voor de handhavingsdekking.
admin-settings-key-cap-oauth-apps = OAuth-toepassingen
admin-settings-help-cap-oauth-apps = Beleid voor OAuth-toepassingen. Raadpleeg de functiecatalogus voor de handhavingsdekking.
admin-settings-key-cap-rules = E-mailregels
admin-settings-help-cap-rules = Sta beheerde methoden voor e-mailregels en afwezigheidsberichten toe.
admin-settings-key-cap-rules-forward = Doorsturen via regels
admin-settings-help-cap-rules-forward = Doorstuurbeleid. Raadpleeg de functiecatalogus voor de handhavingsdekking.
admin-settings-key-cap-remote-images = Externe afbeeldingen
admin-settings-help-cap-remote-images = Mogelijkheid voor externe afbeeldingen. Bestaande privacyinstellingen blijven de verzoeken bepalen; controleer de functiedekking.
admin-settings-key-cap-user-background = Eigen achtergrond
admin-settings-help-cap-user-background = Mogelijkheid voor een eigen achtergrond, standaard uitgeschakeld. Controleer de functiedekking.
admin-settings-key-cap-user-css = Eigen CSS
admin-settings-help-cap-user-css = Mogelijkheid voor eigen CSS, standaard uitgeschakeld. Controleer de functiedekking.
admin-settings-key-limit-attachment-bytes = Limiet voor bijlagegrootte
admin-settings-help-limit-attachment-bytes = Bijlagebeleid in bytes, standaard 50000000. Handhaving tijdens uitvoering is nog in behandeling.
admin-settings-key-limit-send-per-hour = Verzendlimiet per uur
admin-settings-help-limit-send-per-hour = Verzendbeleid per uur; nul betekent onbeperkt. Handhaving tijdens uitvoering is nog in behandeling.
admin-settings-key-limit-recipients-per-message = Ontvangers per bericht
admin-settings-help-limit-recipients-per-message = Beleid voor het aantal ontvangers; nul betekent onbeperkt. Handhaving tijdens uitvoering is nog in behandeling.
admin-settings-cat-import = Importeren
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Live-updates
admin-settings-cat-privacy = Privacy
admin-settings-cat-observability = Observeerbaarheid
admin-settings-cat-egress = Uitgaande verbindingen
admin-settings-cat-updates = Updates
admin-settings-cat-backups = Back-ups
admin-settings-cat-about = Over
admin-settings-cat-config = Configuratie
admin-settings-cat-jobs = Taken
admin-settings-cat-maintenance = Onderhoud
admin-settings-cat-health = Gezondheidscontroles
admin-settings-cat-diagnostics = Diagnostiek
admin-settings-cat-users = Gebruikers
admin-settings-cat-server-text = Servertekst
admin-settings-key-ui-language = Taal
admin-settings-help-ui-language = De taal van de mailclient voor gebruikers die er geen hebben gekozen. "Zelfde als de browser" volgt de talen van elke browser.
admin-settings-key-ui-notify-new-version = Melding van nieuwe versie
admin-settings-help-ui-notify-new-version = Aan (standaard) toont ingelogde gebruikers een melding met "Nu herladen" en "Later" wanneer er sinds het laden van hun pagina een nieuwere versie van de webapp is uitgerold. Uit toont dit nooit; gebruikers krijgen dan de nieuwe code bij hun volgende herlaadbeurt.
admin-settings-key-ui-undo-history-size = Grootte van de geschiedenis voor ongedaan maken en opnieuw
admin-settings-help-ui-undo-history-size = Hoeveel recente e-mailacties (archiveren, verwijderen, verplaatsen, als gelezen markeren, markeren met een vlag, labelen en dergelijke) de knoppen Ongedaan maken en Opnieuw onthouden, van 0 tot 50. 0 schakelt de knoppen en de sneltoets Ctrl+Z uit. De standaardwaarde is 50. Gebruikers mogen een eigen grootte kiezen, nooit boven het onderstaande maximum, tenzij een beheerder dit vergrendelt.
admin-settings-key-folders-reorder-enabled = Mappen slepen en neerzetten
admin-settings-help-folders-reorder-enabled = Laat mensen hun mappen herschikken door ze te slepen in de zijbalk en in Instellingen. Uit verbergt de sleepgrepen en de opdrachten Verplaatsen. Gebruikers kunnen dit voor zichzelf uitschakelen, tenzij dit is vergrendeld.
admin-settings-key-folders-reorder-allow-nesting = Mappen nesten
admin-settings-help-folders-reorder-allow-nesting = Laat mensen een map op een andere map neerzetten om hem te nesten, of terug naar het hoogste niveau. Uit houdt elke map onder zijn huidige bovenliggende map en staat alleen herschikken tussen mappen op hetzelfde niveau toe.
admin-settings-key-folders-reorder-allow-server-move = Mappen op de server verplaatsen
admin-settings-help-folders-reorder-allow-server-move = Laat nesten of ontnesten de echte bovenliggende map op de mailserver wijzigen, zodat het overal verandert. Uit houdt die verplaatsingen alleen als indeling in deze app.
admin-settings-key-folders-reorder-allow-system-folders = Systeemmappen verplaatsen
admin-settings-help-folders-reorder-allow-system-folders = Laat mensen Postvak IN, Concepten, Verzonden, Ongewenst, Prullenbak en Archief slepen om ze te herschikken. Uit houdt ze bovenaan vast. Ze worden alleen in deze app herschikt, nooit hernoemd of verplaatst op de server.
admin-settings-key-folders-reorder-max-depth = Nestdiepte van mappen
admin-settings-help-folders-reorder-max-depth = Het diepste nestniveau dat een sleepactie mag maken, van 1 tot 16. Een verplaatsing die een map of een van zijn submappen dieper zou plaatsen, wordt geweigerd.
admin-settings-key-folders-order-locked = De mapvolgorde vastzetten
admin-settings-help-folders-order-locked = Zet de mapvolgorde vast voor iedereen in het bereik. Slepen en de opdrachten Verplaatsen zijn uitgeschakeld en gebruikers zien een vergrendelingshint. Vastgepinde mappen en de door de beheerder ingestelde volgorde blijven.
admin-settings-key-folders-pinned-admin = Afgedwongen vastgepinde mappen
admin-settings-help-folders-pinned-admin = Mappen die in een groep Vastgepind bovenaan de zijbalk van elke gebruiker worden getoond. Eén item per regel, als role:inbox of een mappad zoals Shared/Support. Gebruikers kunnen ze niet losmaken of verplaatsen. Een item zonder overeenkomende map wordt overgeslagen.
admin-settings-key-folders-order-default = Standaard mapvolgorde
admin-settings-help-folders-order-default = De volgorde waarin mappen beginnen voor gebruikers die ze niet hebben geordend, van eerste naar laatste. Eén item per regel, als role:<role> of een mappad. Niet-vermelde mappen volgen in de volgorde van de server. De eigen volgorde van een gebruiker vervangt deze.
admin-settings-key-folders-new-folder-placement = Plaatsing van nieuwe mappen
admin-settings-help-folders-new-folder-placement = Waar een map die op de server verschijnt tussen zijn naaste mappen wordt geplaatst: aan het einde of in alfabetische volgorde.
admin-settings-key-ui-undo-history-max = Maximum van de geschiedenis voor ongedaan maken en opnieuw
admin-settings-help-ui-undo-history-max = Het maximale aantal acties dat een gebruiker in de geschiedenis van Ongedaan maken en Opnieuw mag bewaren, van 0 tot 50. De eigen grootte van een gebruiker wordt tot deze waarde verlaagd. 0 schakelt de geschiedenis voor iedereen uit. De geschiedenis leeft in het browsertabblad en wordt bij uitloggen gewist.
admin-settings-key-ui-bar-mail-top-bar = E-mailwerkbalk
admin-settings-help-ui-bar-mail-top-bar = "pinned" houdt de e-mailwerkbalk met de lay-outbesturing altijd zichtbaar. "auto" verbergt hem achter een smalle greep die hem opent wanneer een gebruiker er naartoe wijst, tikt of tabt.
admin-settings-key-ui-bar-workspace-tabs = Werkruimtetabbladen
admin-settings-help-ui-bar-workspace-tabs = "pinned" houdt de tabbladenbalk van de werkruimte altijd zichtbaar. "auto" toont alleen een smalle greep met de naam van het actieve tabblad tot een gebruiker hem opent.
admin-settings-key-ui-bar-mail-ribbon = E-maillint
admin-settings-help-ui-bar-mail-ribbon = Voor de lintindeling. "pinned" houdt het lint open. "auto" toont alleen de tabnamen; een tabblad opent het lint over de berichtenlijst.
admin-settings-key-ui-bar-composer-ribbon = Editorlint
admin-settings-help-ui-bar-composer-ribbon = Voor elke berichteneditor. "pinned" houdt het editorlint open. "auto" toont alleen de tabnamen; een tabblad openen duwt de berichttekst omlaag in plaats van hem te bedekken.
admin-settings-key-appearance-theme = Thema
admin-settings-help-appearance-theme = Het kleurenthema dat gebruikers zien tot ze een eigen kiezen. Vergrendelen houdt elke gebruiker aan dit thema.
admin-settings-key-appearance-accent = Accentkleur
admin-settings-help-appearance-accent = De accentkleur van het thema. Leeg betekent het eigen accent van het thema.
admin-settings-key-appearance-badge-color = Kleur van ongelezen-badge
admin-settings-help-appearance-badge-color = De kleur van badges met het aantal ongelezen berichten. Default houdt de kleur van het thema; Accent volgt de accentkleur; accent-lighter en accent-darker gebruiken het accent 20% lichter of donkerder, met zwarte of witte tekst. Een vergrendeling heeft voorrang op de keuze van het account.
admin-settings-key-appearance-density = Dichtheid
admin-settings-help-appearance-density = Hoeveel ruimte lijsten en bedieningselementen innemen: compact, cozy of relaxed.
admin-settings-key-appearance-font = Interfacelettertype
admin-settings-key-packages-allowed = Pakketvolging toegestaan
admin-settings-help-packages-allowed = Sta pakketvolging toe voor de betrokken accounts. De provider moet ook zijn ingeschakeld onder Pakketvolging en elke gebruiker moet zich aanmelden. Deze instelling uitschakelen voorkomt direct nieuwe aanroepen naar de provider.
admin-settings-help-appearance-font = Kies een meegeleverd lettertype voor accounts zonder lettertypevoorkeur, of vergrendel de keuze voor alle betrokken accounts. Lettertypen worden van deze server geladen; er wordt geen verzoek aan Google Fonts gedaan. Default gebruikt het lettertype van het thema.
admin-settings-key-appearance-brand-name = Productnaam
admin-settings-help-appearance-brand-name = De naam die op de inlogpagina en in het browsertabblad wordt getoond.
admin-settings-key-login-notice = Inlogmelding
admin-settings-help-login-notice = Een korte tekst op de inlogpagina, bijvoorbeeld een contact voor ondersteuning. Laat leeg voor geen.
admin-settings-key-mail-read-receipts = Leesbevestigingen
admin-settings-key-receipts-dsn-allowed = Verzoeken om bezorgstatus
admin-settings-help-receipts-dsn-allowed = Sta accounts toe om SMTP-bezorgstatusmeldingen aan te vragen. Dit vraagt niet aan en bewijst niet dat een bericht is gelezen.
admin-settings-help-mail-read-receipts = Wat er gebeurt wanneer een geopend bericht om een leesbevestiging vraagt: de gebruiker vragen, nooit verzenden of altijd verzenden.
admin-settings-key-notifications-enabled = Meldingen van nieuwe e-mail
admin-settings-help-notifications-enabled = Of gebruikers meldingen van nieuwe e-mail krijgen. Dit wijzigt hun meldingsregels niet.
admin-settings-key-offline-retention-days = Leeftijd van offlinecache
admin-settings-help-offline-retention-days = Hoeveel dagen een apparaat berichten bewaart voor offline gebruik. 0 betekent geen leeftijdslimiet.
admin-settings-key-offline-budget-mb = Grootte van offlinecache
admin-settings-help-offline-budget-mb = Hoeveel megabyte een apparaat mag gebruiken voor offline e-mail. 0 betekent geen offlinecache.
admin-settings-key-assistant-enabled = AI-assistent
admin-settings-help-assistant-enabled = Of gebruikers de AI-assistent zien. De implementatieschakelaar onder AI-assistent blijft hierboven gelden.
admin-settings-key-assist-profile = AI-providerprofiel
admin-settings-help-assist-profile = De id van het providerprofiel dat deze gebruikers bedient (AI-assistent > Providerprofielen). Leeg gebruikt het eindpunt van de implementatie. Een profiel-id dat niet bestaat schakelt de AI-assistent voor deze gebruikers uit.
admin-settings-key-upstream-default = Standaardmailserver
admin-settings-help-upstream-default = De mailserver waarop een adres inlogt wanneer geen domein er een noemt. De domeintabel gaat voor.
admin-settings-key-upstream-proxy-timeout-secs = Time-out van mailserver
admin-settings-help-upstream-proxy-timeout-secs = Hoe lang één verzoek aan de mailserver mag duren, in seconden.
admin-settings-key-signin-new-users = Nieuwe gebruikers
admin-settings-help-signin-new-users = Wie bij de eerste keer inloggen een account mag aanmaken: iedereen (open), alleen vooraf toegevoegde adressen (preprovisioned), die plus geïmporteerde gebruikers (provisioned), of niemand nieuw (closed). Bestaande accounts worden nooit beïnvloed. Te wijzigen in Inlogbeleid.
admin-settings-key-signin-new-users-mail = Inloggen met wachtwoord: nieuwe gebruikers
admin-settings-help-signin-new-users-mail = Regel voor een nieuwe postvakgebruiker die met een wachtwoord inlogt (IMAP, POP3, Exchange). Inherit gebruikt Nieuwe gebruikers.
admin-settings-key-signin-new-users-sso = Eenmalige aanmelding: nieuwe gebruikers
admin-settings-help-signin-new-users-sso = Regel voor een nieuwe gebruiker die inlogt via eenmalige aanmelding of een vertrouwde header. Inherit gebruikt Nieuwe gebruikers.
admin-settings-key-signin-new-users-linked = Gekoppelde postvakken: nieuwe gebruikers
admin-settings-help-signin-new-users-linked = Regel voor het koppelen van een postvak dat hier nog geen account heeft, beoordeeld op het eigen domein van het gekoppelde postvak. Inherit gebruikt Nieuwe gebruikers.
admin-settings-key-signin-new-users-linked-imap = Gekoppelde IMAP-postvakken: nieuwe gebruikers
admin-settings-help-signin-new-users-linked-imap = Regel alleen voor gekoppelde IMAP-postvakken. Inherit gebruikt Gekoppelde postvakken.
admin-settings-key-signin-new-users-linked-pop3 = Gekoppelde POP3-postvakken: nieuwe gebruikers
admin-settings-help-signin-new-users-linked-pop3 = Regel alleen voor gekoppelde POP3-postvakken. Inherit gebruikt Gekoppelde postvakken.
admin-settings-key-signin-new-users-linked-ews = Gekoppelde Exchange-postvakken: nieuwe gebruikers
admin-settings-help-signin-new-users-linked-ews = Regel alleen voor gekoppelde Exchange-postvakken (EWS). Inherit gebruikt Gekoppelde postvakken.
admin-settings-key-signin-preprovision-ttl-days = Geldigheid van vooraf toegevoegde gebruiker (dagen)
admin-settings-help-signin-preprovision-ttl-days = Hoe lang een vooraf toegevoegde gebruiker in afwachting geldig blijft wanneer geen andere geldigheid is opgegeven.
admin-settings-key-signin-preprovision-max-ttl-days = Langste geldigheid van vooraf toegevoegde gebruiker (dagen)
admin-settings-help-signin-preprovision-max-ttl-days = De langste geldigheid die een beheerder aan een vooraf toegevoegde gebruiker in afwachting mag geven.
admin-settings-key-signin-preprovision-domain-wide = Vooraf toegevoegde items voor een heel domein toestaan
admin-settings-help-signin-preprovision-domain-wide = Sta items toe die iedereen van een heel domein toelaten, tot een gebruikslimiet. Standaard uit.
admin-settings-key-signin-preprovision-retention-days = Verlopen gebruikers in afwachting bewaren (dagen)
admin-settings-help-signin-preprovision-retention-days = Hoe lang een verlopen gebruiker in afwachting wordt bewaard voordat hij wordt verwijderd.
admin-settings-key-signin-domain-mode = Inlogdomeinen
admin-settings-help-signin-domain-mode = Of elk domein mag inloggen of alleen vermelde domeinen. Te wijzigen in Inlogbeleid.
admin-settings-key-signin-refusal-detail = Weigeringsberichten
admin-settings-help-signin-refusal-detail = Of een geweigerde inlogpoging zegt waarom (uitgeschakeld account, adres niet toegestaan) of hetzelfde antwoord geeft als bij een onjuist wachtwoord. Te wijzigen in Inlogbeleid.
admin-settings-key-signin-ldap-bind-precheck = LDAP-controle vóór inloggen
admin-settings-help-signin-ldap-bind-precheck = Controleer het wachtwoord in de directory vóór de mailserver. Te wijzigen in Integraties, na een testbind.
admin-settings-key-twofa-pending-ttl-secs = Tijdslimiet tweede factor
admin-settings-help-twofa-pending-ttl-secs = Hoe lang een gebruiker na het wachtwoord heeft om de tweede factor in te voeren, in seconden.
admin-settings-key-twofa-max-attempts = Pogingen tweede factor
admin-settings-help-twofa-max-attempts = Hoeveel onjuiste codes voor de tweede factor zijn toegestaan voordat het inloggen opnieuw begint.
admin-settings-key-admin-session-idle-secs = Inactiviteitstime-out van console
admin-settings-help-admin-session-idle-secs = Hoe lang een sessie van de beheerconsole duurt zonder verzoek, in seconden.
admin-settings-key-admin-session-absolute-secs = Sessielimiet van console
admin-settings-help-admin-session-absolute-secs = De langste duur van een sessie van de beheerconsole, in seconden, actief of niet.
admin-settings-key-admin-session-rotate-secs = Rotatie van sessie-id van console
admin-settings-help-admin-session-rotate-secs = Hoe vaak, in seconden, de beheerconsole tijdens gebruik de sessie-id vervangt (0 = alleen na een stap-up-bevestiging). Rotatie verlengt de inactiviteits- of absolute limieten nooit.
admin-settings-key-passwd-policy-min-length = Minimale wachtwoordlengte
admin-settings-help-passwd-policy-min-length = Het minimale aantal tekens van een nieuw wachtwoord.
admin-settings-key-passwd-policy-require-upper = Hoofdletter vereisen
admin-settings-help-passwd-policy-require-upper = Een nieuw wachtwoord moet een hoofdletter bevatten.
admin-settings-key-passwd-policy-require-lower = Kleine letter vereisen
admin-settings-help-passwd-policy-require-lower = Een nieuw wachtwoord moet een kleine letter bevatten.
admin-settings-key-passwd-policy-require-digit = Cijfer vereisen
admin-settings-help-passwd-policy-require-digit = Een nieuw wachtwoord moet een cijfer bevatten.
admin-settings-key-passwd-policy-require-symbol = Symbool vereisen
admin-settings-help-passwd-policy-require-symbol = Een nieuw wachtwoord moet een teken bevatten dat geen letter of cijfer is.
admin-settings-key-limit-storage-bytes = Opslaglimiet
admin-settings-help-limit-storage-bytes = Het maximale aantal bytes dat één gebruiker op deze server mag opslaan. 0 betekent geen limiet.
admin-settings-key-limit-send-per-day = Verzonden berichten per dag
admin-settings-help-limit-send-per-day = Het maximale aantal berichten dat één gebruiker per dag mag verzenden. 0 betekent geen limiet.
admin-settings-key-limit-ai-calls-per-day = Aanroepen van AI-assistent per dag
admin-settings-help-limit-ai-calls-per-day = Het maximale aantal verzoeken aan de AI-assistent dat één gebruiker per dag mag doen. 0 betekent geen limiet.
admin-settings-key-quota-warn-thresholds = Waarschuwingsniveaus voor gebruik
admin-settings-help-quota-warn-thresholds = De percentages van een limiet waarbij gebruikers worden gewaarschuwd, laagste eerst.
admin-settings-key-quota-warn-enabled = Gebruikswaarschuwingen
admin-settings-help-quota-warn-enabled = Of gebruikers worden gewaarschuwd wanneer ze een limiet naderen.
admin-settings-key-usage-retention-days = Gebruiksgeschiedenis
admin-settings-help-usage-retention-days = Hoeveel dagen aan dagelijkse gebruikstellingen worden bewaard.
admin-settings-key-usage-meter-api = API-gebruik tellen
admin-settings-help-usage-meter-api = Of API-verzoeken worden meegeteld in de gebruikscijfers.
admin-settings-key-usage-measure-interval-hours = Interval voor opslagmeting
admin-settings-help-usage-measure-interval-hours = Hoe vaak de opgeslagen gegevens van elke gebruiker worden gemeten, in uren.
admin-settings-key-assistant-rate-limit-per-min = Frequentielimiet van AI-assistent
admin-settings-help-assistant-rate-limit-per-min = Het maximale aantal verzoeken aan de AI-assistent per minuut voor de hele server. 0 betekent geen limiet.
admin-settings-key-plugins-kv-max-key-bytes = Sleutelgrootte van plug-in
admin-settings-help-plugins-kv-max-key-bytes = De langste sleutel die een plug-in mag opslaan, in bytes.
admin-settings-key-plugins-kv-max-value-bytes = Waardegrootte van plug-in
admin-settings-help-plugins-kv-max-value-bytes = De grootste waarde die een plug-in onder één sleutel mag opslaan, in bytes.
admin-settings-key-plugins-kv-max-total-bytes = Opslag van plug-in
admin-settings-help-plugins-kv-max-total-bytes = Het maximale aantal bytes dat één plug-in in totaal mag opslaan.
admin-settings-key-plugins-kv-max-keys = Sleutels van plug-in
admin-settings-help-plugins-kv-max-keys = Het maximale aantal sleutels dat één plug-in mag opslaan.
admin-settings-key-import-max-messages = Aantal berichten per import
admin-settings-help-import-max-messages = Het maximale aantal berichten dat één import mag binnenhalen.
admin-settings-key-import-max-message-bytes = Berichtgrootte bij import
admin-settings-help-import-max-message-bytes = Het grootste bericht dat een import accepteert, in bytes.
admin-settings-key-webhooks-max-attempts = Webhookpogingen
admin-settings-help-webhooks-max-attempts = Hoe vaak een webhookbezorging wordt geprobeerd voordat het wordt opgegeven.
admin-settings-key-webhooks-timeout-secs = Webhook-time-out
admin-settings-help-webhooks-timeout-secs = Hoe lang één webhookbezorging mag duren, in seconden.
admin-settings-key-outbox-max-attempts = Verzendpogingen
admin-settings-help-outbox-max-attempts = Hoe vaak een bericht aan de mailserver wordt aangeboden voordat het als niet verzonden wordt gemeld.
admin-settings-key-realtime-recheck-secs = Hercontrole van live-updates
admin-settings-help-realtime-recheck-secs = Hoe vaak een open live-updateverbinding controleert of de sessie nog geldig is, in seconden.
admin-settings-key-privacy-watermark-enabled = Watermerk van lezer
admin-settings-help-privacy-watermark-enabled = Toon een watermerk met het adres van de lezer over geopende berichten.
admin-settings-key-privacy-watermark-opacity = Sterkte van watermerk
admin-settings-help-privacy-watermark-opacity = Hoe zichtbaar het watermerk is, van 0,02 (vaag) tot 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Geschiedenis van openingsregistratie
admin-settings-help-privacy-open-tracking-retention-days = Hoeveel dagen leesregistraties worden bewaard.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Niet-verzonden registratietokens
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Hoeveel uur een registratietoken wacht tot zijn bericht is verzonden voordat het wordt verwijderd.
admin-settings-key-masked-email-domain = Domein van gemaskeerd adres

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-user-override-help = Indien aan kunnen gebruikers in hun Instellingen een eigen waarde kiezen. Indien uit geldt voor hen de hier of voor hun domein ingestelde waarde.
admin-settings-lock-help = Indien vergrendeld worden domeinwaarden en keuzes van gebruikers genegeerd en geldt deze waarde voor iedereen.
admin-settings-save = Opslaan
admin-settings-saved = Opgeslagen. Het wordt binnen 30 seconden van kracht op elk serverproces.
admin-settings-saved-restart = Opgeslagen. Het wordt van kracht na een herstart.
admin-settings-dedicated = Deze instelling wordt gewijzigd in { $section }, dat eerst het effect controleert.
admin-settings-dedicated-link = { $section } openen
admin-settings-pinned = Ingesteld door de omgevingsvariabele { $name }. De hier opgeslagen waarde geldt wanneer de variabele niet is ingesteld.
admin-settings-lock-title = { $setting } vergrendelen?
admin-settings-lock-body = Vergrendelen negeert de hieronder vermelde engere waarden. Ze blijven bewaard en gelden weer wanneer u ontgrendelt.
admin-settings-lock-confirm = Vergrendelen
admin-settings-impact-domain = { $count ->
    [one] { $count } domeinwaarde wordt genegeerd
   *[other] { $count } domeinwaarden worden genegeerd
 }
admin-settings-help-masked-email-domain = Het domein waaronder gemaskeerde adressen worden aangemaakt. Laat leeg om gemaskeerde adressen uit te schakelen.
admin-settings-key-abuse-report-address = Adres voor misbruikmeldingen
admin-settings-help-abuse-report-address = Waarheen meldingen van spam en misbruik worden gestuurd.
admin-settings-key-dlp-admin-address = Meldingen van gegevenslekken
admin-settings-help-dlp-admin-address = Waarheen meldingen van gegevenslekpreventie worden gestuurd.
admin-settings-key-observability-error-forward-url = Adres voor foutdoorsturing
admin-settings-help-observability-error-forward-url = Een https-adres waarnaar foutrapporten van clients worden doorgestuurd. Laat leeg voor geen.
admin-settings-key-egress-probe-url = Testadres voor uitgaand verkeer
admin-settings-help-egress-probe-url = Het https-adres waarmee een uitgaande proxy wordt getest.
admin-settings-key-push-vapid-contact = Pushcontact
admin-settings-help-push-vapid-contact = Het contact (mailto: of https:) dat bij elke melding aan pushdiensten wordt doorgegeven.
admin-settings-key-push-quiet-hours = Stille uren voor push
admin-settings-help-push-quiet-hours = Uren (UTC) waarin geen pushmelding wordt verzonden. Laat leeg voor geen.
admin-settings-key-updates-check = Op updates controleren
admin-settings-help-updates-check = Of de server eenmaal per dag controleert op een nieuwere release en deze op het dashboard toont.
admin-settings-key-updates-feed-url = Releasefeed
admin-settings-help-updates-feed-url = Het https-adres dat de updatecontrole leest.
admin-settings-key-updates-apply = Updates toepassen
admin-settings-help-updates-apply = Of een update vanuit de console kan worden gestart via de update-agent van de host.
admin-settings-key-backup-retention-count = Bewaarde back-ups
admin-settings-help-backup-retention-count = Hoeveel databaseback-ups worden bewaard.
admin-settings-key-about-security-contact = Beveiligingscontact
admin-settings-help-about-security-contact = Het contact in security.txt (mailto: of https:).
admin-settings-key-about-security-policy-url = Beveiligingsbeleid
admin-settings-help-about-security-policy-url = Het adres van uw beveiligingsbeleid, vermeld in security.txt.
admin-settings-key-about-project-links = Links bij Over
admin-settings-help-about-project-links = Links op de pagina Over, elk met een label en een https-adres.
admin-settings-key-config-snapshots-kept = Configuratiemomentopnamen
admin-settings-help-config-snapshots-kept = Hoeveel configuratiemomentopnamen worden bewaard.
admin-settings-key-jobs-history-days = Taakgeschiedenis
admin-settings-help-jobs-history-days = Hoeveel dagen aan taakuitvoeringen worden bewaard.
admin-settings-key-maintenance-announce-minutes-default = Aanlooptijd van onderhoudsmelding
admin-settings-help-maintenance-announce-minutes-default = Hoeveel minuten vóór een onderhoudsvenster gebruikers erover worden geïnformeerd.
admin-settings-key-maintenance-default-message = Onderhoudsbericht
admin-settings-help-maintenance-default-message = De tekst die gebruikers tijdens onderhoud zien, per taal.
admin-settings-key-health-disk-free-min-percent = Minimale vrije schijfruimte
admin-settings-help-health-disk-free-min-percent = De gereedheidscontrole mislukt wanneer de vrije schijfruimte onder dit percentage komt.
admin-settings-key-observability-otlp-metrics = Metrics verzenden via OTLP
admin-settings-help-observability-otlp-metrics = Stuur servermetrics naar de OTLP-collector die bij het starten is ingesteld.
admin-settings-key-metrics-drilldown-top-n = Detailrijen van metrics
admin-settings-help-metrics-drilldown-top-n = Hoeveel rijen een uitsplitsing van metrics toont.
admin-settings-key-metrics-rollup-days = Metricsgeschiedenis
admin-settings-help-metrics-rollup-days = Hoeveel dagen aan uurlijkse metricsamenvattingen worden bewaard.
admin-settings-key-observability-client-timing-enabled = Laadtijden in de browser
admin-settings-help-observability-client-timing-enabled = Verzamel laadtijden van pagina's uit ingelogde browsers als anonieme geaggregeerde histogrammen. Niets identificeert een persoon, sessie of pagina. Schakel uit om het verzamelen te stoppen.
admin-settings-key-observability-client-timing-sample-rate-percent = Steekproefpercentage voor laadtijden in de browser
admin-settings-help-observability-client-timing-sample-rate-percent = Percentage browsersessies dat laadtijden rapporteert (1 tot 100).
admin-settings-key-observability-client-timing-min-samples = Minimaal aantal metingen voor laadtijden
admin-settings-help-observability-client-timing-min-samples = De mediaan en het 95e percentiel van laadtijden blijven verborgen tot een periode minstens dit aantal metingen heeft.
admin-settings-key-observability-client-timing-folder-loads = Laadtijden van mappen
admin-settings-help-observability-client-timing-folder-loads = Meet hoe lang een map nodig heeft om zijn berichtenlijst te tonen, uitgesplitst naar uit de browsercache of van het netwerk. Standaard aan. Gebruikers kunnen dit voor zichzelf uitschakelen, tenzij deze instelling is vergrendeld. Vereist dat laadtijden in de browser aan staan.
admin-settings-key-observability-client-timing-message-loads = Laadtijden van berichten
admin-settings-help-observability-client-timing-message-loads = Meet hoe lang een bericht nodig heeft om zijn headers en gesaneerde tekst te tonen, uitgesplitst naar uit de cache of vooraf geladen, of van het netwerk. Standaard aan. Gebruikers kunnen dit voor zichzelf uitschakelen, tenzij deze instelling is vergrendeld. Vereist dat laadtijden in de browser aan staan.
admin-settings-key-diagnostics-retention-days = Diagnosegeschiedenis
admin-settings-help-diagnostics-retention-days = Hoeveel dagen aan diagnose-uitvoeringen worden bewaard.
admin-settings-key-data-export-link-hours = Downloadperiode van gegevensexport
admin-settings-help-data-export-link-hours = Hoeveel uur een gegevensexport kan worden gedownload voordat hij wordt verwijderd.
admin-settings-key-dsr-enabled = Gegevensverzoeken
admin-settings-help-dsr-enabled = Of mensen vanuit Instellingen gegevensverzoeken kunnen indienen (inzage, download, correctie, verwijdering, beperking, bezwaar).
admin-settings-key-dsr-type-access = Inzageverzoeken accepteren
admin-settings-help-dsr-type-access = Of het verzoektype inzage aan gebruikers wordt aangeboden.
admin-settings-key-dsr-type-portability = Downloadverzoeken (overdraagbare kopie) accepteren
admin-settings-help-dsr-type-portability = Of het verzoektype download (overdraagbare kopie) aan gebruikers wordt aangeboden.
admin-settings-key-dsr-type-rectification = Correctieverzoeken accepteren
admin-settings-help-dsr-type-rectification = Of het verzoektype correctie aan gebruikers wordt aangeboden.
admin-settings-key-dsr-type-erasure = Verwijderingsverzoeken accepteren
admin-settings-help-dsr-type-erasure = Of het verzoektype verwijdering aan gebruikers wordt aangeboden.
admin-settings-key-dsr-type-restriction = Beperkingsverzoeken accepteren
admin-settings-help-dsr-type-restriction = Of het verzoektype beperking aan gebruikers wordt aangeboden.
admin-settings-key-dsr-type-objection = Bezwaarverzoeken accepteren
admin-settings-help-dsr-type-objection = Of het verzoektype bezwaar aan gebruikers wordt aangeboden.
admin-settings-key-dsr-type-other = Overige verzoeken accepteren
admin-settings-help-dsr-type-other = Of het algemene verzoektype aan gebruikers wordt aangeboden.
admin-settings-key-dsr-approval-export = Goedkeuring van inzage- en downloadverzoeken
admin-settings-help-dsr-approval-export = auto start de export zodra het verzoek is bevestigd. admin wacht tot een beheerder het goedkeurt.
admin-settings-key-dsr-approval-erasure = Goedkeuring van verwijderingsverzoeken
admin-settings-help-dsr-approval-erasure = auto plant de verwijdering zodra het verzoek is bevestigd. admin wacht tot een beheerder het goedkeurt.
admin-settings-key-dsr-approval-other = Afhandeling van overige verzoeken
admin-settings-help-dsr-approval-other = auto markeert het verzoek als in behandeling zodra het is bevestigd. admin wacht tot een beheerder het goedkeurt.
admin-settings-key-dsr-reauth-required = Opnieuw verifiëren voor verzoeken
admin-settings-help-dsr-reauth-required = Vereis een recent wachtwoord of tweede factor om een export- of verwijderingsverzoek in te dienen en om een downloadlink te maken.
admin-settings-key-dsr-confirmation-required = Bevestigingsbericht
admin-settings-help-dsr-confirmation-required = Stuur een bevestigingsbericht met een code naar de inbox van de aanvrager; het verzoek gaat pas verder nadat de code is ingevoerd.
admin-settings-key-dsr-identity-verification-required = Stap voor identiteitsverificatie
admin-settings-help-dsr-identity-verification-required = Een beheerder moet vastleggen dat de identiteit van de aanvrager is geverifieerd voordat het verzoek wordt goedgekeurd.
admin-settings-key-dsr-erasure-grace-days = Uitstelperiode voor verwijdering
admin-settings-help-dsr-erasure-grace-days = Dagen tussen goedkeuring en verwijdering. De aanvrager kan in die tijd annuleren. 0 voert de verwijdering uit bij de volgende worker-ronde.
admin-settings-key-dsr-sla-days = Dagen om een verzoek te beantwoorden
admin-settings-help-dsr-sla-days = Het aantal dagen van indiening tot de uiterste datum die aan gebruikers en beheerders wordt getoond.
admin-settings-key-dsr-sla-max-extension-days = Langste verlenging
admin-settings-help-dsr-sla-max-extension-days = Het maximale aantal dagen dat een beheerder aan de uiterste datum van een verzoek kan toevoegen, met een vastgelegde reden.
admin-settings-key-dsr-sla-reminder-days = Markeringen voor uiterste datum
admin-settings-help-dsr-sla-reminder-days = Dagen vóór de uiterste datum waarop een verzoek in de wachtrij als bijna verlopen wordt gemarkeerd.
admin-settings-key-dsr-export-mail-allowed = E-mail in exports
admin-settings-help-dsr-export-mail-allowed = Sta een aanvrager toe berichten in een export op te nemen. Alleen berichten in de eigen cache van deze server worden opgenomen; e-mail die op de bovenliggende mailserver blijft, niet.
admin-settings-key-dsr-export-mail-format = E-mailformaat in exports
admin-settings-help-dsr-export-mail-format = mbox schrijft één bestand per map; eml schrijft één bestand per bericht.
admin-settings-key-dsr-export-include-settings = Export: instellingen
admin-settings-help-dsr-export-include-settings = Neem instellingen op in een export.
admin-settings-key-dsr-export-include-identities = Export: identiteiten
admin-settings-help-dsr-export-include-identities = Neem identiteiten op in een export.
admin-settings-key-dsr-export-include-contacts = Export: contacten
admin-settings-help-dsr-export-include-contacts = Neem contacten op in een export.
admin-settings-key-dsr-export-include-calendars = Export: agenda's
admin-settings-help-dsr-export-include-calendars = Neem agenda's op in een export.
admin-settings-key-dsr-export-include-notes = Export: notities
admin-settings-help-dsr-export-include-notes = Neem notities op in een export.
admin-settings-key-dsr-export-include-rules = Export: filters en regels
admin-settings-help-dsr-export-include-rules = Neem filters en regels op in een export.
admin-settings-key-dsr-export-include-sessions = Export: sessies, apparaten en inloggeschiedenis
admin-settings-help-dsr-export-include-sessions = Neem sessies, apparaten en inloggeschiedenis op in een export.
admin-settings-key-dsr-export-include-consent = Export: toestemmingsregistraties
admin-settings-help-dsr-export-include-consent = Neem toestemmingsregistraties op in een export.
admin-settings-key-dsr-export-include-audit = Export: auditvermeldingen over de persoon
admin-settings-help-dsr-export-include-audit = Neem auditvermeldingen over de persoon op in een export.
admin-settings-key-dsr-export-max-megabytes = Maximale exportgrootte
admin-settings-help-dsr-export-max-megabytes = Grootste export in megabyte. Secties die dit zouden overschrijden worden weggelaten en in het manifest vermeld.
admin-settings-key-dsr-export-format = Exportformaten
admin-settings-help-dsr-export-format = json schrijft JSON-bestanden. jsonCsv schrijft ook CSV-bestanden voor lijstgegevens.
admin-settings-key-dsr-export-encrypt = Exports in rust verzegelen
admin-settings-help-dsr-export-encrypt = Sla gemaakte exports versleuteld met de serversleutel op tot ze worden gedownload of verlopen.
admin-settings-key-dsr-download-link-minutes = Geldigheid van downloadlink
admin-settings-help-dsr-download-link-minutes = Minuten dat een downloadlink werkt nadat hij is gemaakt. De export zelf wordt bewaard gedurende de downloadperiode van gegevensexport.
admin-settings-key-dsr-download-single-use = Downloadlink voor eenmalig gebruik
admin-settings-help-dsr-download-single-use = Een downloadlink werkt niet meer nadat hij één keer is gebruikt.
admin-settings-key-dsr-retention-completed-days = Bewaartermijn van afgeronde verzoeken
admin-settings-help-dsr-retention-completed-days = Dagen dat een afgerond verzoek en het bijbehorende spoor worden bewaard voordat ze worden verwijderd.
admin-settings-key-dsr-notify-admins = Nieuwe verzoeken voor beheerders vastleggen
admin-settings-help-dsr-notify-admins = Leg elk nieuw verzoek vast als beveiligingsgebeurtenis in het auditlogboek, waar beheerders en waarschuwingsontvangers het zien.
admin-settings-key-dsr-notify-requester = Voortgangsberichten aan de aanvrager
admin-settings-help-dsr-notify-requester = Plaats een bericht in de inbox van de aanvrager wanneer een verzoek van status verandert.
admin-settings-key-dsr-contact-name = Naam van contact voor gegevensbescherming
admin-settings-help-dsr-contact-name = Naam die aan gebruikers wordt getoond op de pagina met gegevensverzoeken.
admin-settings-key-dsr-contact-email = Adres van contact voor gegevensbescherming
admin-settings-help-dsr-contact-email = E-mailadres dat aan gebruikers wordt getoond op de pagina met gegevensverzoeken.
admin-settings-key-dsr-contact-url = Contactpagina voor gegevensbescherming
admin-settings-help-dsr-contact-url = Link die aan gebruikers wordt getoond op de pagina met gegevensverzoeken.
admin-settings-key-dsr-on-behalf-allowed = Verzoeken namens een gebruiker
admin-settings-help-dsr-on-behalf-allowed = Sta een beheerder toe een verzoek voor een gebruiker in te dienen, bijvoorbeeld een per post ontvangen verzoek.
admin-settings-key-dsr-rate-limit-per-day = Verzoeken per persoon per dag
admin-settings-help-dsr-rate-limit-per-day = Het maximale aantal verzoeken dat één persoon in 24 uur kan indienen.
admin-settings-key-dsr-legal-hold-subjects = Juridische bewaarplicht
admin-settings-help-dsr-legal-hold-subjects = Adressen, of @domein-items, waarvan de verwijdering is geblokkeerd. Een verwijderingsverzoek voor hen blijft als geblokkeerd in de wachtrij staan tot het item wordt verwijderd.
admin-settings-key-scrub-enabled = Persoon wissen uit sporen
admin-settings-help-scrub-enabled = Staat de actie Persoon wissen uit sporen toe en het wissen dat op een verwijdering kan volgen. Uitgeschakeld wordt geen van beide uitgevoerd.
admin-settings-key-scrub-pseudonym-algorithm = Lengte van pseudoniem
admin-settings-help-scrub-pseudonym-algorithm = Hoe het pseudoniem met sleutel dat een identificator vervangt wordt gemaakt: HMAC-SHA256 onder de huidige sleutelgeneratie, afgekapt tot 16 of 32 hexadecimale tekens.
admin-settings-key-scrub-include-logs = Logbestanden wissen
admin-settings-help-scrub-include-logs = Bij wissen wordt de identificator ook herschreven in de geroteerde logbestanden, ook de gecomprimeerde.
admin-settings-key-scrub-auto-run-on-erasure = Wissen na verwijdering
admin-settings-help-scrub-auto-run-on-erasure = Een voltooide verwijdering vervangt ook de identificatoren van de persoon in de auditgeschiedenis en logs door het pseudoniem. Standaard uit.
admin-settings-key-scrub-key-retention-days = Bewaartermijn van pseudoniemsleutel
admin-settings-help-scrub-key-retention-days = Dagen dat een pseudoniemsleutel wordt bewaard nadat hij is geroteerd. 0 bewaart hem. Na verwijdering kunnen oude pseudoniemen niet meer opnieuw worden berekend en kunnen met de sleutel gemaakte grafstenen niet meer worden herhaald.
admin-settings-key-backup-propagate-erasure = Verwijderingen herhalen bij herstel
admin-settings-help-backup-propagate-erasure = Een herstel herhaalt de verwijderingsgrafstenen die nieuwer zijn dan de back-up, zodat sindsdien verwijderde personen opnieuw worden verwijderd.
admin-settings-key-backup-tombstone-retention-days = Bewaartermijn van verwijderingsgrafstenen
admin-settings-help-backup-tombstone-retention-days = Dagen dat een verwijderingsgrafsteen wordt bewaard. 0 bewaart ze voor altijd.
admin-settings-key-dsr-text-intro = Inleiding gegevensverzoeken
admin-settings-help-dsr-text-intro = Tekst boven het verzoekformulier. Eén waarde per taal.
admin-settings-key-dsr-text-confirmation = Tekst van bevestigingsbericht
admin-settings-help-dsr-text-confirmation = Tekst die aan het bevestigingsbericht wordt toegevoegd. Eén waarde per taal.
admin-settings-key-dsr-text-completion = Tekst van afrondingsbericht
admin-settings-help-dsr-text-completion = Tekst die wordt toegevoegd aan het bericht dat wordt verzonden wanneer een verzoek is afgerond. Eén waarde per taal.
admin-settings-key-server-text-family = Servertekst
admin-settings-help-server-text-family = Tekst die de server in e-mails en meldingen schrijft, per taal en domein.
admin-settings-cat-sieve = E-mailfilters
admin-settings-key-sieve-max-script-bytes = Maximale grootte van filterscript
admin-settings-help-sieve-max-script-bytes = Maximaal aantal bytes in één Sieve-filterscript.
admin-settings-key-sieve-max-scripts = Maximaal aantal filterscripts
admin-settings-help-sieve-max-scripts = Maximaal aantal opgeslagen scripts per account.
admin-settings-key-sieve-max-redirects = Maximaal aantal omleidingen
admin-settings-help-sieve-max-redirects = Maximaal aantal omleidingsopdrachten in één script.
admin-settings-key-sieve-vacation-allowed = Filteracties voor afwezigheid
admin-settings-help-sieve-vacation-allowed = Sta scripts toe afwezigheidsberichten te verzenden wanneer de backend dit ondersteunt.
admin-settings-key-sieve-allowed-extensions = Toegestane filterextensies
admin-settings-help-sieve-allowed-extensions = Een optionele lijst met Sieve-extensienamen in kleine letters. De backend moet elke extensie ook ondersteunen.
admin-settings-key-cap-sieve-raw-edit = Bron van filterscript bewerken
admin-settings-help-cap-sieve-raw-edit = Sta directe bewerking van Sieve-scripts toe voor de geselecteerde accounts.
admin-settings-key-cap-sieve-forward-external = Doorsturen buiten het accountdomein
admin-settings-help-cap-sieve-forward-external = Sta extern doorsturen in scripts toe. Voor het opslaan van deze scripts is ook recente verificatie vereist.
admin-settings-key-session-max-concurrent = Gelijktijdige sessies
admin-settings-help-session-max-concurrent = Maximaal aantal sessies per account. De oudste sessies worden ingetrokken wanneer een nieuwe login deze limiet overschrijdt.
admin-settings-key-session-idle-secs = Inactiviteitstime-out van sessie
admin-settings-help-session-idle-secs = Beëindig inactieve sessies na dit aantal seconden, tot de limiet van de implementatie.
admin-settings-key-session-absolute-secs = Levensduur van sessie
admin-settings-help-session-absolute-secs = Beëindig sessies na dit aantal seconden, tot de limiet van de implementatie.
admin-settings-key-twofa-remember-days = Levensduur van onthouden apparaat
admin-settings-help-twofa-remember-days = Dagen voordat een onthouden apparaat opnieuw een tweede factor moet verifiëren. Nul schakelt deze functie uit.
admin-settings-key-twofa-passkeys-enabled = Passkeys toestaan
admin-settings-help-twofa-passkeys-enabled = Sta passkeys toe als authenticatiefactor.
admin-settings-key-twofa-required = Tweefactorauthenticatie vereisen
admin-settings-help-twofa-required = Vereis een geregistreerde tweede factor voordat een sessie wordt uitgegeven.
admin-settings-key-twofa-passkey-attachment = Type authenticator voor passkey
admin-settings-help-twofa-passkey-attachment = any (standaard): elke authenticator. platform: passkeys opgeslagen op het apparaat (Windows Hello, Touch ID, schermvergrendeling van Android). cross-platform: beveiligingssleutels en telefoons. De browser meldt dit; het is niet ondertekend door de authenticator.
admin-settings-key-twofa-passkey-user-verification = Gebruikersverificatie voor passkey
admin-settings-help-twofa-passkey-user-verification = required: de passkey moet een pincode of biometrie controleren bij het toevoegen en bij elke login; een ceremonie zonder dat wordt geweigerd. preferred (standaard) en discouraged veranderen alleen wat de browser wordt gevraagd.
admin-settings-key-twofa-passkey-resident-key = Vindbare passkeys
admin-settings-help-twofa-passkey-resident-key = required: de inloggegevens moeten op de authenticator worden opgeslagen. discouraged (standaard) of preferred. In engine-modus zijn passkeys altijd vindbaar, omdat ze ook zonder wachtwoord inloggen.
admin-settings-key-twofa-passkey-legacy = Passkeys toegevoegd vóór het huidige beleid
admin-settings-help-twofa-passkey-legacy = allow (standaard): een oudere passkey die niet aan het beleid voldoet blijft werken en de gebruiker wordt gevraagd hem te vervangen. refuse: hij wordt bij het inloggen met een bericht geweigerd; de gebruiker logt in met een andere factor of een herstelcode.
admin-settings-key-twofa-last-factor-removal = Laatste tweede factor verwijderen
admin-settings-help-twofa-last-factor-removal = Wanneer het verwijderen van een passkey of authenticator-app het account zonder zou laten. allow (standaard): hij wordt verwijderd en inloggen vereist alleen het wachtwoord. block: de gebruiker moet eerst een andere inlogmethode toevoegen (herstelcodes tellen niet mee). Waar tweefactorauthenticatie vereist is, blijft de laatste altijd behouden.
admin-settings-key-privacy-remote-content = Externe inhoud
admin-settings-help-privacy-remote-content = Afbeeldingen blokkeren, eerst vragen, de proxy gebruiken of rechtstreeks laden wanneer toegestaan.
admin-settings-key-privacy-trackers = Trackers blokkeren
admin-settings-help-privacy-trackers = Blokkeer bekende trackingafbeeldingen in e-mail.
admin-settings-key-privacy-clean-links = Links opschonen
admin-settings-help-privacy-clean-links = Verwijder trackingparameters uit links in berichten.
admin-settings-key-privacy-max-bytes = Limiet voor externe afbeeldingen
admin-settings-help-privacy-max-bytes = Maximaal aantal bytes dat voor één externe afbeelding wordt opgehaald.
admin-settings-key-privacy-cache-bytes = Cache voor externe afbeeldingen
admin-settings-help-privacy-cache-bytes = Maximaal aantal bytes dat in de cache voor externe afbeeldingen wordt bewaard.
admin-settings-key-privacy-rate-per-min = Verzoekfrequentie voor externe afbeeldingen
admin-settings-help-privacy-rate-per-min = Maximaal aantal ophalingen van externe afbeeldingen per account per minuut.
admin-settings-privacy-fetch-title = Externe afbeeldingen ophalen
admin-settings-privacy-fetch-help = Tijdslimieten, omleidingen en parallelle downloads voor afbeeldingen die de server voor de lezer ophaalt. Een afbeelding boven een limiet toont een plaatsaanduiding.
admin-settings-key-privacy-fetch-connect-timeout-ms = Verbindingstime-out voor externe afbeelding
admin-settings-help-privacy-fetch-connect-timeout-ms = Milliseconden die zijn toegestaan om de verbinding met een afbeeldingshost te openen, inclusief de TLS-handshake. Standaard 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Antwoordtime-out voor externe afbeelding
admin-settings-help-privacy-fetch-headers-timeout-ms = Milliseconden die zijn toegestaan vanaf het begin van een verzoek tot de afbeeldingshost met headers antwoordt. Standaard 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Time-out bij stilvallen van externe afbeelding
admin-settings-help-privacy-fetch-idle-timeout-ms = Langste pauze, in milliseconden, tussen twee delen van een afbeeldingsdownload voordat deze wordt gestopt. Standaard 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Totale time-out voor externe afbeelding
admin-settings-help-privacy-fetch-total-timeout-ms = Milliseconden die zijn toegestaan voor het ophalen van een afbeelding, inclusief omleidingen. Wachten in de wachtrij heeft een apart budget, zodat een afbeelding die moet wachten zijn netwerktijd niet opgebruikt voordat hij begint. Standaard 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Wachtrijtime-out voor externe afbeelding
admin-settings-help-privacy-fetch-queue-timeout-ms = Maximaal aantal milliseconden dat afbeeldingen op capaciteit mogen wachten bovenop hun ophaalbudget. De begrensde wachtrij vangt grote geauthenticeerde pieken op terwijl de limieten voor actief ophalen en decoderen van kracht blijven. Standaard 60000.
admin-settings-key-privacy-max-redirects = Omleidingen voor externe afbeelding
admin-settings-help-privacy-max-redirects = Omleidingen die voor één afbeelding worden gevolgd. Elke wordt gecontroleerd met dezelfde adresregels als het eerste verzoek. Standaard 3.
admin-settings-key-privacy-max-concurrent-per-message = Downloads van externe afbeeldingen per bericht
admin-settings-help-privacy-max-concurrent-per-message = Afbeeldingen van één bericht die tegelijk worden opgehaald. Andere wachten op een vrije plek. Bereik 1–128; standaard 30.
admin-settings-key-privacy-max-concurrent-per-account = Downloads van externe afbeeldingen per account
admin-settings-help-privacy-max-concurrent-per-account = Afbeeldingen die tegelijk worden opgehaald voor één account over al zijn geopende berichten. Bereik 1–256; standaard 128.
admin-settings-key-privacy-allow-direct = Directe afbeeldingen toestaan
admin-settings-key-cap-remote-content-direct = Recht voor directe afbeeldingen
admin-settings-help-cap-remote-content-direct = Sta dit account toe directe afbeeldingen te selecteren wanneer de server dit toestaat.
admin-settings-help-privacy-allow-direct = Sta het rechtstreeks laden van afbeeldingen toe voor accounts met de vereiste mogelijkheid.
admin-settings-key-logs-auth-retention-days = Bewaartermijn van authenticatiegeschiedenis
admin-settings-help-logs-auth-retention-days = Dagen dat authenticatiemetagegevens worden bewaard, onafhankelijk van de duurzame accountidentiteit.
admin-settings-key-logs-auth-max-entries = Limiet voor items in authenticatiegeschiedenis
admin-settings-help-logs-auth-max-entries = Maximaal aantal bewaarde items. Overeenkomende herhaalde mislukkingen binnen één minuut delen één item.
admin-settings-key-reader-color-mode = Berichtkleuren
admin-settings-cat-reader = Berichtweergave
admin-settings-key-mail-row-actions-mode = Snelle acties in berichtenlijst
admin-settings-help-mail-row-actions-mode = Hoe de snelle acties op elke berichtrij verschijnen: open (altijd beschikbaar, zichtbaar bij aanwijzen of focus), hover (drie bedieningselementen en een knop Meer), click (één knop die ze opent) of off (van de rijen verwijderd; het contextmenu en de werkbalk behouden elke actie). Hover gedraagt zich als click op aanraakschermen. Gebruikers mogen een eigen keuze maken, tenzij een beheerder dit vergrendelt.
admin-settings-key-mail-quick-reply = Snel antwoorden

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-quick-reply = Biedt een compact antwoordtabblad aan met de ontvangers al ingevuld. Standaard uit; gebruikers mogen het inschakelen, tenzij een beheerder het vergrendelt. De normale verzendbeveiligingen blijven van kracht.
admin-settings-key-mail-request-read-receipt = Standaard leesbevestigingen aanvragen
admin-settings-help-mail-request-read-receipt = Begin ondersteunde uitgaande berichten met een geselecteerd verzoek om bevestiging. Dit is een standaard bij het opstellen, geen garantie dat de ontvanger een bevestiging verstuurt.
admin-settings-help-reader-color-mode = Automatisch volgt donkere thema's. Kies originele, adaptieve of omgekeerde weergave binnen het geïsoleerde berichtkader.
admin-settings-key-reader-font-size = Lettergrootte van bericht
admin-settings-help-reader-font-size = Standaardleesgrootte in pixels, van 12 tot 24.
admin-settings-key-reader-line-height = Regelhoogte van bericht
admin-settings-help-reader-line-height = Regelafstand bij het lezen van 1,2 tot 2,2.
admin-settings-key-reader-content-width = Leesbreedte van bericht
admin-settings-help-reader-content-width = Berichtinhoud in comfortabele, brede of paneelvullende breedte.
admin-settings-key-reader-loading = Uiterlijk van berichtlaadindicator
admin-settings-help-reader-loading = Plaatsaanduidingsregels, een tekstindicator of geen visuele indicator. Toegankelijkheidsaankondigingen blijven beschikbaar.
admin-settings-key-reader-loading-delay-ms = Vertraging van berichtlaadindicator
admin-settings-help-reader-loading-delay-ms = Vertraag alleen de laadindicator, tussen 0 en 1000 milliseconden. De bezorging van berichten wordt nooit vertraagd.
admin-settings-key-reader-zoom = Berichtzoom
admin-settings-help-reader-zoom = Standaardzoom van de berichttekst in procenten, van 50 tot 200; de lezer gebruikt de dichtstbijzijnde zoomstap. Alleen het bericht schaalt en afdrukken gebruikt 100%.
admin-settings-key-reader-attachments-collapsed = Bijlagenbalk ingeklapt
admin-settings-help-reader-attachments-collapsed = Of de bijlagenbalk van de lezer ingeklapt begint, met alleen het aantal en de totale grootte. De laatste keer openen of sluiten door de gebruiker wordt hier opgeslagen. Standaard uit: de bijlagen worden getoond.
admin-settings-key-login-show-domain-hint = Voorkeursdomein tonen bij inloggen
admin-settings-help-login-show-domain-hint = Toon welk domein een gebruikersnaam zonder e-mailachtervoegsel accepteert. Standaard verborgen. Dit wijzigt alleen de hint; inloggen met een gebruikersnaam blijft werken.
admin-settings-key-signin-retry-cooldown-secs = Wachttijd tussen inlogpogingen
admin-settings-help-signin-retry-cooldown-secs = Minimaal aantal seconden tussen inlogpogingen met wachtwoord vanaf één clientnetwerk. Standaard 3; 0 schakelt de wachttijd uit. Bestaande blokkades na mislukkingen blijven gelden.
admin-settings-key-mail-quote-style = Headerstijl bij antwoorden en doorsturen
admin-settings-key-mail-external-recipient-warning = Waarschuwing voor externe ontvanger
admin-settings-help-mail-external-recipient-warning = Uit, waarschuwen of bevestiging vereisen in de editor. Geldt voor Aan, Cc en Bcc. Dit is een beveiliging in de editor, geen bezorgbeperking van de server.
admin-settings-key-mail-trusted-recipient-domains = Vertrouwde ontvangerdomeinen
admin-settings-help-mail-trusted-recipient-domains = Maximaal 100 exacte ontvangerdomeinen. Het domein van de geselecteerde verzendidentiteit wordt ook vertrouwd. Ongeldige items verlenen nooit vertrouwen. Vergrendel deze instelling om wijzigingen door gebruikers te voorkomen.
admin-settings-help-mail-quote-style = Gebruik klassieke geciteerde antwoorden of berichtheaders in Outlook-stijl. Dit stelt de standaard van de gebruiker in; vergrendel het om de stijl af te dwingen. Bestaande concepten blijven ongewijzigd.
admin-settings-key-signin-anti-bot-mode = Botbescherming bij inloggen
admin-settings-help-signin-anti-bot-mode = Zelfgehoste proof-of-work-verificatie: off, mailbox, admin of all. Er wordt geen externe verificatiedienst benaderd.
admin-settings-key-signin-anti-bot-difficulty = Verificatie-inspanning bij inloggen
admin-settings-help-signin-anti-bot-difficulty = Maximale teller van de oplosser, 100–20000. Hogere waarden vragen meer werk van de browser; standaard 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Geldigheid van inlogverificatie
admin-settings-help-signin-anti-bot-expiry-secs = Seconden dat een verificatie voor eenmalig gebruik geldig blijft, 60–600; standaard 180. Na elke inlogpoging is een nieuwe verificatie nodig.
admin-settings-key-attachments-preview-allowed = Voorbeelden van bijlagen toestaan
admin-settings-help-attachments-preview-allowed = Schakel uit om het weergeven van bijlagevoorbeelden te voorkomen; originele downloads hebben een eigen instelling.
admin-settings-key-attachments-download-allowed = Levering van originele bijlage toestaan
admin-settings-help-attachments-download-allowed = Uitschakelen voorkomt ook voorbeelden waarvoor de originele bytes nodig zijn. Browservoorbeelden zijn geen kopieerbeveiliging.
admin-settings-key-attachments-max-bytes = Maximaal aantal bytes van bijlage
admin-settings-help-attachments-max-bytes = Nul is onbeperkt. Geldt voor de geleverde originele bytes, inclusief bereikverzoeken.
admin-settings-key-attachments-allowed-extensions = Toegestane bestandsnaamextensies
admin-settings-help-attachments-allowed-extensions = JSON-array met extensies in kleine letters zonder punt, zoals ["pdf","tar.gz"]. Leeg staat elke extensie toe; blokkeerregels gaan voor.
admin-settings-key-attachments-blocked-extensions = Geblokkeerde bestandsnaamextensies
admin-settings-help-attachments-blocked-extensions = JSON-array met extensies in kleine letters. Beoordeeld aan de hand van opgeslagen MIME-bestandsnamen, nooit de naam in de download-URL.
admin-settings-key-attachments-warn-extensions = Waarschuwen voor bestandsnaamextensies
admin-settings-help-attachments-warn-extensions = JSON-array met extensies die uitdrukkelijke bevestiging vereisen vóór levering van het origineel.
admin-settings-key-attachments-allowed-mime-types = Toegestane MIME-typen
admin-settings-help-attachments-allowed-mime-types = JSON-array met typen in kleine letters of jokertekens op hoofdniveau, zoals ["image/*","application/pdf"]. Leeg staat elk type toe.
admin-settings-key-attachments-blocked-mime-types = Geblokkeerde MIME-typen
admin-settings-help-attachments-blocked-mime-types = JSON-array met typen in kleine letters of jokertekens op hoofdniveau. Blokkeerregels gaan voor.
admin-settings-key-attachments-warn-mime-types = Waarschuwen voor MIME-typen
admin-settings-help-attachments-warn-mime-types = JSON-array met typen in kleine letters of jokertekens op hoofdniveau die bevestiging vereisen. MIME-metagegevens zijn geen detectie van malware.
admin-settings-cat-shortcuts = Sneltoetsen
admin-settings-cat-time = Tijdzones
admin-settings-key-time-display-zone = Weergavetijdzone
admin-settings-help-time-display-zone = IANA-tijdzone zoals Europe/Lisbon, of local om het huidige apparaat te volgen. Tijdstempels van berichten blijven opgeslagen als hun oorspronkelijke tijdstip.
admin-settings-key-time-comparison-zone = Vergelijkingstijdzone
admin-settings-help-time-comparison-zone = Optionele tweede IANA-tijdzone voor bericht- en agendahulpmiddelen. Laat leeg om de vergelijking te verbergen; er wordt geen locatie afgeleid uit een e-mailadres.
admin-settings-key-time-show-helpers = Tijdzonehulpmiddelen tonen
admin-settings-help-time-show-helpers = Toon vergelijkingen tussen lokale tijd en de geselecteerde zone voor tijdstempels van berichten en agenda-afspraken. Vergrendelingen door beheerders kunnen deze keuze afdwingen.
admin-settings-key-shortcuts-enabled = Sneltoetsen inschakelen
admin-settings-help-shortcuts-enabled = Sta sneltoetsen van de toepassing toe. Vergrendelingen kunnen ze globaal of per domein uitschakelen. Invoervelden en modale dialogen behouden hun normale toetsenbordafhandeling.
admin-settings-key-shortcuts-compose = Sneltoets voor opstellen
admin-settings-key-shortcuts-archive = Sneltoets voor archiveren
admin-settings-key-shortcuts-reply = Sneltoets voor antwoorden
admin-settings-key-shortcuts-reply-all = Sneltoets voor allen antwoorden
admin-settings-key-shortcuts-forward = Sneltoets voor doorsturen
admin-settings-key-shortcuts-next = Sneltoets voor volgend bericht
admin-settings-key-shortcuts-previous = Sneltoets voor vorig bericht
admin-settings-key-shortcuts-search = Sneltoets om zoeken te focussen
admin-settings-key-shortcuts-refresh = Sneltoets voor berichten vernieuwen
admin-settings-key-shortcuts-settings = Sneltoets voor instellingen openen
admin-settings-key-shortcuts-trash = Sneltoets voor verplaatsen naar de prullenbak
admin-settings-key-shortcuts-mark-read = Sneltoets voor als gelezen markeren
admin-settings-key-shortcuts-mark-unread = Sneltoets voor als ongelezen markeren
admin-settings-key-shortcuts-spam = Sneltoets voor als spam markeren
admin-settings-key-shortcuts-zoom-in = Sneltoets om bericht in te zoomen
admin-settings-key-shortcuts-zoom-out = Sneltoets om bericht uit te zoomen
admin-settings-key-shortcuts-zoom-reset = Sneltoets om berichtzoom te herstellen
admin-settings-key-shortcuts-open-in-tab = Sneltoets om bericht in een eigen tabblad te openen
admin-settings-key-notifications-push-allowed = Pushmeldingen op de achtergrond toegestaan
admin-settings-help-notifications-push-allowed = Sta toe dat de ingeschakelde meldingen van dit account zich registreren en pushberichten op de achtergrond ontvangen. Toestemming van de browser is nog steeds vereist.
admin-settings-key-offline-persistent-storage-allowed = Verzoeken om permanente browseropslag toegestaan
admin-settings-help-offline-persistent-storage-allowed = Sta een expliciet verzoek toe om lokaal gecachte gegevens te beschermen tegen automatische verwijdering door de browser. De browser kan weigeren; dit verleent niet automatisch toestemming.
admin-settings-key-security-signin-mailbox-failure-limit = Limiet voor mislukte inloggegevens van postvak
admin-settings-help-security-signin-mailbox-failure-limit = Afgewezen inloggegevens van één bron-IP binnen het telvenster leiden tot een tijdelijke inlogblokkade voor postvakken. Netwerk- en configuratiefouten tellen niet mee.
admin-settings-key-security-signin-admin-failure-limit = Limiet voor mislukte inloggegevens van beheerder
admin-settings-help-security-signin-admin-failure-limit = Afgewezen beheerdersgegevens van één bron-IP leiden tot een aparte tijdelijke inlogblokkade voor beheerders. Korte beheerderssessies en eisen voor recent bewijs blijven ongewijzigd.
admin-settings-key-security-signin-failure-window-secs = Telvenster voor mislukte inloggegevens
admin-settings-help-security-signin-failure-window-secs = Voortschrijdend venster in seconden voor afwijzingen van inloggegevens en MFA. Een succesvol voltooide login zet die bron en dat oppervlak terug.
admin-settings-key-security-signin-lock-secs = Duur van tijdelijke inlogblokkade
admin-settings-help-security-signin-lock-secs = Duur van de blokkade in seconden. Herhaalde pogingen tijdens de blokkade verlengen de oorspronkelijke vervaldatum niet. Handmatige globale IP-blokkades zijn los hiervan.
admin-settings-key-security-signin-geo-scope = Bereik van landenbeleid bij inloggen
admin-settings-help-security-signin-geo-scope = Standaard uit. Geldt alleen voor nieuwe logins van postvakken, beheerders of alle. Bestaande sessies worden niet ingetrokken. Controleer de diagnose van de huidige bron voordat u dit inschakelt; een niet-beschikbare lokale GeoIP-database weigert het inloggen.
admin-settings-key-security-signin-geo-mode = Toegestane of geweigerde landenlijst
admin-settings-help-security-signin-geo-mode = Allow staat alleen vermelde landen toe; deny weigert vermelde landen. Onbekende adressen volgen hun eigen expliciete beleid.
admin-settings-key-security-signin-geo-countries = Landcodes voor inloggen
admin-settings-help-security-signin-geo-countries = JSON-array met tweeletterige landcodes in hoofdletters, bijvoorbeeld ["PT", "GB"]. Gebruikt alleen de lokale landendatabase (Beveiliging > Landendatabase, of MW_GEOIP_DB); adressen worden nooit naar een locatiedienst gestuurd.
admin-settings-key-security-signin-geo-unknown = Beleid voor niet-toegewezen bronadressen
admin-settings-help-security-signin-geo-unknown = Sta adressen toe of weiger ze die ontbreken in een leesbare lokale database, inclusief privéadressen. Ontbrekende of onleesbare databases weigeren altijd logins onder actief beleid, ook wanneer onbekende adressen zijn toegestaan.
admin-settings-key-contacts-avatar-mode = Contactfoto's en gegenereerde avatars
admin-settings-help-contacts-avatar-mode = Genereer initialen lokaal, verberg avatars of gebruik ingebedde foto's uit de eigen adresboeken van elk account. Er worden geen externe foto-URL's of externe avatardiensten benaderd. Gebruikers- en domeinbeleid kan door een beheerder worden vastgehouden.
admin-settings-cat-contacts = Contactfoto's
admin-settings-key-reader-action-style = Actielint van bericht
admin-settings-help-reader-action-style = Compacte pictogrammen zijn de standaard. Kies labels voor pictogrammen met zichtbare tekst; beide behouden actienamen en thematische knopinfo. Ondersteunt globale, domein- en gebruikersvergrendelingen.
admin-settings-key-nextcloud-sync-interval-secs = Interval voor automatisch vernieuwen van Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Seconden tussen het vernieuwen van collecties op de achtergrond voor verbonden accounts met de native engine. Nul is alleen handmatig; positieve waarden hebben een minimum van zestig seconden. Bij mislukkingen wordt het interval verlengd. Er wordt geen achtergrondlogin geprobeerd.
admin-settings-key-mail-folder-role-inbox = Toewijzing van map Postvak IN
admin-settings-key-mail-folder-role-drafts = Toewijzing van map Concepten
admin-settings-key-mail-folder-role-sent = Toewijzing van map Verzonden
admin-settings-key-mail-folder-role-trash = Toewijzing van map Prullenbak
admin-settings-key-mail-folder-role-junk = Toewijzing van map Ongewenst
admin-settings-key-mail-folder-role-archive = Toewijzing van map Archief
admin-settings-key-mail-request-delivery-receipt = Standaard bezorgmeldingen aanvragen
admin-settings-help-mail-request-delivery-receipt = Selecteer bezorgmeldingen voor nieuwe berichten wanneer het account SMTP-DSN ondersteunt. Vergrendelen bepaalt deze opgeslagen standaard; receipts.dsnAllowed bepaalt afzonderlijk of verzoeken zijn toegestaan. Bezorging bewijst niet dat een bericht is gelezen.
admin-settings-key-security-signin-geo-enforce-sessions = Landenbeleid afdwingen op bestaande sessies
admin-settings-help-security-signin-geo-enforce-sessions = Standaard uit. Pas het ingestelde landenbereik voor inloggen toe op geauthenticeerde verzoeken, native bearer-toegang en ondertekende lezerinhoud. Geblokkeerde clients kunnen nog uitloggen. Reeds geopende streams worden gecontroleerd wanneer ze opnieuw verbinden; dit beëindigt bestaande verbindingen niet.
admin-settings-key-security-protection-rate-mode = Geaggregeerde verzoekbudgetten
admin-settings-help-security-protection-rate-mode = Standaard uit. Observe registreert uitputting van het budget zonder te weigeren; enforce geeft een tijdelijke 429. Bestaande protocolgelijktijdigheidslimieten blijven onafhankelijk.
admin-settings-key-security-protection-adaptive-mode = Adaptieve verzoekbescherming
admin-settings-help-security-protection-adaptive-mode = Standaard uit. Observe registreert verklaarbare getypeerde afwijkingen en voorgestelde acties; enforce past begrensde tijdelijke afremming of blokkades toe. Dit is een deterministisch tellerbeleid, geen AI-classificator.
admin-settings-key-security-protection-client-requests = Clientverzoeken per venster
admin-settings-help-security-protection-client-requests = Verzoeken per venster van één niet-ingelogd adres, en elk inlogverzoek. Een verzoek met een geldige postvak- of beheerderssessie gebruikt in plaats daarvan een eigen, ruimere limiet. Nul schakelt dit budget uit.
admin-settings-key-security-protection-account-requests = Accountverzoeken per venster
admin-settings-help-security-protection-account-requests = Verzoeken per venster voor één ingelogd postvak, over al zijn sessies, apparaten en adressen. Hoger gehouden dan de anonieme limiet zodat ingelogd gebruik meer ruimte heeft. Nul schakelt dit budget uit.
admin-settings-key-rate-limit-diagnostics-burst = Diagnose-uitvoeringen tegelijk
admin-settings-help-rate-limit-diagnostics-burst = Diagnose-uitvoeringen en backendcontroles die een beheerderssessie achter elkaar mag starten. Een controle starten die al loopt geeft die controle terug in plaats van opnieuw te tellen.
admin-settings-key-rate-limit-diagnostics-per-minute = Diagnose-uitvoeringen per minuut
admin-settings-help-rate-limit-diagnostics-per-minute = Diagnose-uitvoeringen en backendcontroles per minuut die een beheerderssessie terugkrijgt.
admin-settings-key-security-protection-admin-requests = Beheerdersverzoeken per venster
admin-settings-help-security-protection-admin-requests = Verzoeken per venster voor één ingelogde beheerder. Hoger gehouden dan de anonieme limiet. Nul schakelt dit budget uit.
admin-settings-key-security-protection-rate-window-secs = Venster van verzoekbudget (seconden)
admin-settings-help-security-protection-rate-window-secs = Vast interval van het verzoekbudget. Belastingafwerping door capaciteit en gewone not-found-antwoorden zijn geen adaptief bewijs.
admin-settings-key-security-protection-adaptive-clients = Clients observeren voor adaptieve bescherming
admin-settings-key-security-protection-adaptive-accounts = Accounts observeren voor adaptieve bescherming
admin-settings-key-security-protection-adaptive-admins = Beheerders observeren voor adaptieve bescherming
admin-settings-key-security-protection-short-window-secs = Kort afwijkingsvenster (seconden)
admin-settings-help-security-protection-short-window-secs = Voortschrijdend observatievenster. Lange vensters ondersteunen uren tot dertig dagen; afronding van buckets kan hooguit een extra vierenzestigste van een venster aanhouden.
admin-settings-key-security-protection-short-minimum-samples = Minimaal aantal metingen voor korte verhouding
admin-settings-help-security-protection-short-minimum-samples = Minimaal aantal waargenomen normale plus afwijkende verzoeken voordat de verhoudingstrigger geldt. De absolute afwijkingsdrempel is onafhankelijk van dit minimum.
admin-settings-key-security-protection-short-absolute-anomalies = Absoluut afwijkingsbudget (kort)
admin-settings-help-security-protection-short-absolute-anomalies = Triggert wanneer zoveel geselecteerde afwijkingen zich ophopen, ook als normale verzoeken de verhouding verdunnen. Nul schakelt deze trigger uit.
admin-settings-key-security-protection-short-ratio-percent = Korte afwijkingsverhouding (procent)
admin-settings-help-security-protection-short-ratio-percent = Triggert wanneer geselecteerde afwijkingen dit percentage bereiken na het minimale aantal metingen. De absolute of de verhoudingstrigger volstaat. Nul schakelt deze trigger uit.
admin-settings-key-security-protection-long-window-secs = Lang afwijkingsvenster (seconden)
admin-settings-help-security-protection-long-window-secs = Voortschrijdend observatievenster. Lange vensters ondersteunen uren tot dertig dagen; afronding van buckets kan hooguit een extra vierenzestigste van een venster aanhouden.
admin-settings-key-security-protection-long-minimum-samples = Minimaal aantal metingen voor lange verhouding
admin-settings-help-security-protection-long-minimum-samples = Minimaal aantal waargenomen normale plus afwijkende verzoeken voordat de verhoudingstrigger geldt. De absolute afwijkingsdrempel is onafhankelijk van dit minimum.
admin-settings-key-security-protection-long-absolute-anomalies = Absoluut afwijkingsbudget (lang)
admin-settings-help-security-protection-long-absolute-anomalies = Triggert wanneer zoveel geselecteerde afwijkingen zich ophopen, ook als normale verzoeken de verhouding verdunnen. Nul schakelt deze trigger uit.
admin-settings-key-security-protection-long-ratio-percent = Lange afwijkingsverhouding (procent)
admin-settings-help-security-protection-long-ratio-percent = Triggert wanneer geselecteerde afwijkingen dit percentage bereiken na het minimale aantal metingen. De absolute of de verhoudingstrigger volstaat. Nul schakelt deze trigger uit.
admin-settings-key-security-protection-normal-cap-per-bucket = Plafond van normale basislijn per bucket
admin-settings-help-security-protection-normal-cap-per-bucket = Begrens de normale basislijn die in elke bucket van het voortschrijdende venster wordt geleerd. Onafhankelijke absolute afwijkingsbudgetten voorkomen dat onbeperkte aanvulling met geldige verzoeken misbruik verbergt.
admin-settings-key-security-protection-signals = Signalen voor adaptief bewijs
admin-settings-help-security-protection-signals = Selecteer gezaghebbende waarnemingen van misvormd, te groot, afgewezen inloggegevens, geweigerde toestemming, overschreden tijdslimiet voor de body of, op aanvraag, misser op een beheerroute. Het aftasten van beheerroutes sluit not-found-antwoorden voor entiteiten en statische bestanden uit; algemene statuscodes, capaciteitsweigeringen en user-agentstrings bewijzen geen misbruik.
admin-settings-key-security-protection-throttle-secs = Eerste tijdelijke afremming (seconden)
admin-settings-help-security-protection-throttle-secs = De eerste drempeloverschrijding start een vaste afremtermijn. Verzoeken tijdens die termijn verlengen deze niet.
admin-settings-key-security-protection-block-secs = Opgeschaalde tijdelijke blokkade (seconden)
admin-settings-help-security-protection-block-secs = Een latere drempeloverschrijding vóór herstel kan opschalen naar deze begrensde blokkade. Er wordt geen permanente accountblokkade aangemaakt.
admin-settings-key-security-protection-decay-secs = Herstelinterval zonder afwijkingen (seconden)
admin-settings-help-security-protection-decay-secs = Na dit afwijkingsvrije interval en het verlopen van de huidige actie keert de opschaling terug naar de beginfase. Voortschrijdend bewijs verloopt volgens de ingestelde vensters.
admin-settings-key-security-protection-exempt-clients = Vrijgestelde clientadressen en -netwerken
admin-settings-help-security-protection-exempt-clients = Exacte IP-adressen of CIDR-netwerken stellen alleen de clientdimensie vrij. Limieten voor geverifieerde accounts blijven gelden. Gebruik dit bewust voor gedeelde NAT-netwerken; doorgestuurde adressen worden alleen vertrouwd via ingestelde proxy's.
admin-settings-key-security-protection-exempt-accounts = Vrijgestelde geverifieerde account-id's
admin-settings-help-security-protection-exempt-accounts = Exacte door de server geverifieerde postvakaccount-id's stellen alleen hun accountdimensie vrij, nooit de dimensie van de bronclient. E-mailadressen en door de aanroeper opgegeven accountparameters zijn geen identiteitsbewijs.
admin-settings-key-security-protection-exempt-admins = Vrijgestelde beheerder-id's
admin-settings-help-security-protection-exempt-admins = Exacte geauthenticeerde beheerder-id's stellen alleen hun beheerdersdimensie vrij. Bestaande blokkades na mislukte logins en strikte termijnen van beheerderssessies blijven ongewijzigd.
admin-settings-key-security-protection-storage-failure = Beleid bij opslagfout van handhaving
admin-settings-help-security-protection-storage-failure = Closed weigert beschermd werk wanneer een ingeschakeld handhavingsbudget niet kan worden gecontroleerd. Open staat het tijdelijk toe met een begrensde operationele waarschuwing. Fouten in alleen-observatiemodus leggen nooit een handhavingsactie op.
admin-settings-key-security-admin-probe-enabled = Bescherming tegen aftasten van beheer
admin-settings-help-security-admin-probe-enabled = Tel verzoeken die naar de beheerconsole zoeken per clientnetwerk (IPv4-adres, IPv6 /64), vertraag een bron die doorgaat en blokkeer hem dan. Typefouten en verouderde bladwijzers blijven binnen de speelruimte. Te wijzigen in Beveiliging > Adaptieve bescherming.
admin-settings-key-security-admin-probe-grace-allowance = Speelruimte (missers per venster)
admin-settings-help-security-admin-probe-grace-allowance = Missers die per venster worden genegeerd. De score daalt continu met zoveel punten per venster.
admin-settings-key-security-admin-probe-window-secs = Speelruimtevenster (seconden)
admin-settings-help-security-admin-probe-window-secs = Het venster waarop de speelruimte van toepassing is.
admin-settings-key-security-admin-probe-block-after = Missers boven de speelruimte vóór een blokkade
admin-settings-help-security-admin-probe-block-after = Missers boven de speelruimte die een blokkade starten. Missers daartussen worden beantwoord na de tarpit-vertraging.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Tarpit-vertraging (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Vertraging die wordt toegevoegd aan elke getelde misser boven de speelruimte. 0 schakelt de tarpit uit. Andere verzoeken worden nooit vertraagd.
admin-settings-key-security-admin-probe-block-secs = Eerste blokkade (seconden)
admin-settings-help-security-admin-probe-block-secs = Duur van de eerste blokkade. Een geblokkeerde bron krijgt voor elk pad hetzelfde antwoord.
admin-settings-key-security-admin-probe-max-block-secs = Langste blokkade (seconden)
admin-settings-help-security-admin-probe-max-block-secs = Een herhaalde blokkade verdubbelt de vorige duur, tot dit plafond.
admin-settings-key-security-admin-probe-repeat-window-secs = Herhalingsvenster (seconden)
admin-settings-help-security-admin-probe-repeat-window-secs = Een blokkade die binnen zoveel tijd na het einde van de vorige begint, telt als herhaling.
admin-settings-key-security-admin-probe-block-status = Status van blokkadeantwoord
admin-settings-help-security-admin-probe-block-status = HTTP-status die naar een geblokkeerde bron wordt gestuurd, met Retry-After: 429 of 403.
admin-settings-key-security-admin-probe-signals = Getelde missers
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (een 404 op het padniveau van de console), admin-signin-failed en admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Aftastpaden
admin-settings-help-security-admin-probe-probe-paths = Paden waar scanners om vragen, zoals /wp-admin of /.env. Een pad dat er een als hele segmenten bevat, telt als misser. Het eigen pad van de console nooit.
admin-settings-key-security-admin-probe-allowlist = Nooit geblokkeerde netwerken
admin-settings-help-security-admin-probe-allowlist = IP-adressen of CIDR-netwerken (kantoor, VPN) die nooit worden geteld of geblokkeerd.
admin-settings-key-security-admin-probe-audit-ip = Bronadres in het auditlogboek
admin-settings-help-security-admin-probe-audit-ip = masked toont IPv4 /24 en IPv6 /48; full toont het adres. De fail2ban-regel bevat altijd het volledige adres.
admin-settings-key-security-admin-probe-fail2ban-log = fail2ban-logregel bij blokkade
admin-settings-help-security-admin-probe-fail2ban-log = Schrijf één mailwoman[admin-probe]-regel met rhost=<address> wanneer een bron wordt geblokkeerd.
admin-settings-key-appearance-custom-css-enabled = Persoonlijke uiterlijk-CSS toestaan
admin-settings-help-appearance-custom-css-enabled = Sta beperkte visuele stijlen toe in de werkruimte van de eigenaar. Standaard uitgeschakeld; afgedwongen vergrendelingen van het postvakuiterlijk pauzeren deze stijlen.
admin-settings-key-appearance-custom-js-enabled = Persoonlijke uiterlijkscripts toestaan
admin-settings-help-appearance-custom-js-enabled = Sta geïsoleerde scripts toe die alleen uiterlijkwijzigingen kunnen aanvragen. Standaard uitgeschakeld; scripts hebben geen toegang tot e-mail, accountgegevens of de DOM van de toepassing.
admin-settings-key-mail-modifier-selection = Berichtselectie met modificatortoetsen
admin-settings-help-shortcuts-compose = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-archive = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-reply = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-reply-all = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-forward = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-next = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-previous = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-search = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-refresh = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-settings = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-trash = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-mark-read = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-mark-unread = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-spam = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-zoom-in = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-zoom-out = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-zoom-reset = Gebruik inherit voor de geselecteerde voorinstelling, disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-shortcuts-open-in-tab = Gebruik inherit voor de geselecteerde voorinstelling (Shift+Enter), disabled om deze actie te verwijderen, of een toetscombinatie zoals Ctrl+Shift+K. Conflicterende of ongeldige combinaties worden niet uitgevoerd.
admin-settings-help-mail-folder-role-inbox = Null gebruikt automatische detectie. Gebruik een object met een exact pad voor gedeelde standaarden of een object met de id van een eigen postvak voor één account. Ontbrekende of conflicterende doelen schakelen deze rol uit; beheerdersvergrendelingen gelden.
admin-settings-help-mail-folder-role-drafts = Null gebruikt automatische detectie. Gebruik een object met een exact pad voor gedeelde standaarden of een object met de id van een eigen postvak voor één account. Ontbrekende of conflicterende doelen schakelen deze rol uit; beheerdersvergrendelingen gelden.
admin-settings-help-mail-folder-role-sent = Null gebruikt automatische detectie. Gebruik een object met een exact pad voor gedeelde standaarden of een object met de id van een eigen postvak voor één account. Ontbrekende of conflicterende doelen schakelen deze rol uit; beheerdersvergrendelingen gelden.
admin-settings-help-mail-folder-role-trash = Null gebruikt automatische detectie. Gebruik een object met een exact pad voor gedeelde standaarden of een object met de id van een eigen postvak voor één account. Ontbrekende of conflicterende doelen schakelen deze rol uit; beheerdersvergrendelingen gelden.
admin-settings-help-mail-folder-role-junk = Null gebruikt automatische detectie. Gebruik een object met een exact pad voor gedeelde standaarden of een object met de id van een eigen postvak voor één account. Ontbrekende of conflicterende doelen schakelen deze rol uit; beheerdersvergrendelingen gelden.
admin-settings-help-mail-folder-role-archive = Null gebruikt automatische detectie. Gebruik een object met een exact pad voor gedeelde standaarden of een object met de id van een eigen postvak voor één account. Ontbrekende of conflicterende doelen schakelen deze rol uit; beheerdersvergrendelingen gelden.
admin-settings-help-security-protection-adaptive-clients = Schakel deze onderwerpdimensie in wanneer de adaptieve modus actief is. Accounts en beheerders worden pas geteld nadat hun identiteit is geverifieerd.
admin-settings-help-security-protection-adaptive-accounts = Schakel deze onderwerpdimensie in wanneer de adaptieve modus actief is. Accounts en beheerders worden pas geteld nadat hun identiteit is geverifieerd.
admin-settings-help-security-protection-adaptive-admins = Schakel deze onderwerpdimensie in wanneer de adaptieve modus actief is. Accounts en beheerders worden pas geteld nadat hun identiteit is geverifieerd.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-modifier-selection = Sta bereikselectie met Ctrl, Command en Shift toe. Vergrendel deze waarde om haar af te dwingen; zichtbare selectiebesturing blijft beschikbaar.
admin-settings-key-mail-drag-to-folder = Berichten naar mappen slepen
admin-settings-help-mail-drag-to-folder = Sta het slepen van aan het account gebonden berichten naar echte mappen toe. Dit verleent geen schrijfrechten op het postvak; de server autoriseert elke verplaatsing nog steeds.
admin-settings-key-admin-allow-last-superadmin-loss = Verlies van de laatste Super admin toestaan
admin-settings-help-admin-allow-last-superadmin-loss = Standaard uit. Inschakelen staat toe dat latere wijzigingen de laatste Super admin-toegang verwijderen. Een volledige Super admin moet het gevolg bevestigen met recente authenticatie.
admin-settings-last-superadmin-title = Verlies van alle Super admin-toegang toestaan?
admin-settings-last-superadmin-body = Latere accountwijzigingen kunnen ertoe leiden dat niemand deze implementatie nog kan beheren. Herstel vereist lokaal beheer van de implementatie. Typ ALLOW LAST SUPERADMIN LOSS om deze uitzondering in te schakelen.
admin-settings-last-superadmin-confirm = Verlies van toegang toestaan
admin-settings-key-mail-folder-counts = Maptellers
admin-settings-help-mail-folder-counts = Kies verborgen, ongelezen, ongelezen alleen zolang er ongelezen e-mail is, totaal, of beide tellers naast mappen. Stel globale, domein- of gebruikersstandaarden in en vergrendel de keuze indien nodig. Geldt voor elk gekoppeld postvak van een account.
admin-settings-key-mail-folder-count-scope = Mappen met tellers
admin-settings-help-mail-folder-count-scope = Alle mappen, of alleen hoofdmappen (Postvak IN, Concepten, Verzonden, Archief, Ongewenst, Prullenbak, Postvak UIT en andere mappen voor speciaal gebruik). Bij alleen hoofdmappen tonen eigen mappen en hun submappen geen teller. Geldt voor elk gekoppeld postvak van een account.
admin-settings-key-mail-compact-folder-badges = Compacte secundaire mapbadges
admin-settings-help-mail-compact-folder-badges = Toon kleinere badges buiten Postvak IN. Gebruikers kunnen een eigen grootte kiezen, tenzij deze instelling is vergrendeld.
admin-settings-key-mail-focused-inbox = Gerichte inbox
admin-settings-help-mail-focused-inbox = Splits Postvak IN in de tabbladen Gericht en Overig; nieuwsbrieven en geautomatiseerde afzenders gaan naar Overig. Standaard uit. Gebruikers kunnen dit in- of uitschakelen in Instellingen > Mappen en zijbalk, tenzij deze instelling is vergrendeld.
admin-settings-key-mail-account-unread-badge = Aantal ongelezen naast de postvaknaam
admin-settings-help-mail-account-unread-badge = Toon het aantal ongelezen berichten van Postvak IN naast het ingelogde adres of de bijnaam in de zijbalk. Standaard uit. De tellers van andere postvakken in het postvakmenu worden altijd getoond. Geldt voor elk gekoppeld postvak van een account.
admin-settings-key-privacy-remote-image-downloads = Handmatige downloads van externe afbeeldingen
admin-settings-key-privacy-remote-images-toast = Melding bij het laden van afbeeldingen
admin-settings-help-privacy-remote-images-toast = Toon één melding in de lezer terwijl de externe afbeeldingen van een bericht via de privacyproxy worden geladen, en meld wanneer ze zijn geladen of hoeveel er niet konden worden geladen. Standaard aan. Wijzigt alleen wat de lezer toont; welke afbeeldingen worden geladen, wordt nog steeds bepaald door het beleid voor externe inhoud.
admin-settings-help-privacy-remote-image-downloads = Sta een expliciete download toe van een afbeelding waarnaar het eigen bericht verwijst, via de bewaakte privacyproxy. Standaard uitgeschakeld; dit opent de externe URL nooit rechtstreeks.
admin-settings-key-mail-resume-tabs = Werkruimtetabbladen hervatten
admin-settings-help-mail-resume-tabs = Herstel de tabverwijzingen van het account na herladen. Uitschakelen wist het lokale werkruimtebestand zonder concepten te verwijderen.
admin-settings-key-metrics-default-range-seconds = Standaardbereik van verzoekgeschiedenis
admin-settings-help-metrics-default-range-seconds = Seconden die worden getoond wanneer een geschiedenisverzoek geen bereik opgeeft. Begrensd door de bewaartermijn en het maximumbereik.
admin-settings-key-metrics-max-range-seconds = Maximumbereik van verzoekgeschiedenis
admin-settings-help-metrics-max-range-seconds = Maximaal aantal seconden per geschiedenisquery. De bewaartermijn blijft een extra limiet.
admin-settings-key-contacts-favorites-first = Favorieten eerst in suggesties
admin-settings-help-contacts-favorites-first = Toon favoriete contacten vóór andere treffers in ontvangersuggesties. Alleen de volgorde verandert; geen suggestie wordt verborgen. Overschrijvingen per gebruiker en domein kunnen worden vergrendeld.
admin-settings-key-contacts-sender-icons = Meegeleverde afzenderpictogrammen
admin-settings-help-contacts-sender-icons = Toon lokaal meegeleverde pictogrammen voor herkende afzenderdomeinen, met gegenereerde initialen voor andere. Contactfoto's hebben voorrang. Pictogrammen verifiëren de identiteit van een afzender niet. Er wordt geen externe logodienst benaderd; overschrijvingen per gebruiker en domein kunnen worden vergrendeld.
admin-settings-key-mail-brand-avatars = Merklogo's naast geverifieerde afzenders
admin-settings-help-mail-brand-avatars = Toon een lokaal meegeleverd merklogo naast de afzender in de lezer, alleen wanneer het bericht DKIM en een uitgelijnde DMARC-controle voor het eigen domein van dat merk heeft doorstaan. Berichten die deze controles niet doorstaan of missen, behouden de normale avatar. Er wordt geen logo opgehaald bij de afzender of een logodienst.
admin-settings-key-mail-draft-save-mode = Opslagmodus voor conceptaccount
admin-settings-help-mail-draft-save-mode = Opslaan na een typpauze, periodiek of alleen op verzoek. Opslaan vereist een ondersteunde bovenliggende map Concepten; onzekere schrijfacties worden nooit automatisch herhaald.
admin-settings-key-mail-draft-debounce-ms = Typpauze voor concept (milliseconden)
admin-settings-help-mail-draft-debounce-ms = Wacht 500 tot 10000 milliseconden na bewerken voordat wordt opgeslagen in realtimemodus.
admin-settings-key-mail-draft-interval-secs = Opslaginterval voor concept (seconden)
admin-settings-help-mail-draft-interval-secs = Sla bewerkte concepten elke 5 tot 3600 seconden op in periodieke modus. Ongewijzigde concepten worden niet opnieuw geschreven.
admin-settings-key-login-remember-signins = Adressen van geslaagde logins onthouden
admin-settings-help-login-remember-signins = Bewaar tot tien succesvol gebruikte gebruikersnamen of e-mailadressen op dit apparaat voor de vervolgkeuzelijst bij inloggen. Slaat nooit wachtwoorden of sessies op. Uitschakelen verwijdert de opgeslagen adressen van deze server wanneer een apparaat de volgende keer de inlogpagina laadt of een login voltooit; het beheert niet de eigen automatische invulling van de browser.
admin-settings-key-login-show-remembered-hint = Uitleg over opgeslagen adressen tonen
admin-settings-help-login-show-remembered-hint = Toon verklarende tekst in de vervolgkeuzelijst met eerdere logins. Standaard verborgen. Geldt alleen wanneer het onthouden van geslaagde logins is ingeschakeld.
admin-settings-key-mail-show-from = Het veld Van in de editor tonen
admin-settings-help-mail-show-from = Toon Van standaard in nieuwe editors. Gebruikers kunnen het voor één bericht tonen; alleen geautoriseerde adressen kunnen verzenden.
admin-settings-key-accounts-linking-enabled = Gekoppelde postvakaccounts toestaan
admin-settings-help-accounts-linking-enabled = Standaard uitgeschakeld. Globaal uitschakelen blokkeert alle gekoppelde toegang; beide accounts moeten ook voldoen aan hun domein- en accountbeleid. Koppelen vereist daadwerkelijke authenticatie van het onderliggende account en recent bewijs van het primaire.
admin-settings-key-accounts-use-linked = Gekoppelde postvakken gebruiken
admin-settings-help-accounts-use-linked = Afmelding door de gebruiker voor gekoppelde toegang. Kan een uitschakeling door de beheerder niet overschrijven. Postvakgegevens blijven behouden wanneer de toegang wordt uitgeschakeld.
admin-settings-key-accounts-max-linked = Maximaal aantal gekoppelde postvakken
admin-settings-help-accounts-max-linked = Maximaal aantal geverifieerde onderliggende postvakken per primaire identiteit, van 1 tot 16. Standaard 5.
admin-settings-key-accounts-store-linked-credentials = Toegang tot gekoppelde postvakken behouden
admin-settings-help-accounts-store-linked-credentials = Bewaar het wachtwoord van elk gekoppeld postvak verzegeld op de server zodat de toegang zichzelf vernieuwt, over sessies en herstarts heen. Wordt nooit bewaard voor accounts met nultoegang of postvakken met tweestapsverificatie. Uitschakelen wist opgeslagen wachtwoorden; koppelingen worden dan vernieuwd door opnieuw in te loggen. Standaard aan.
admin-settings-key-privacy-strip-mailer-headers = Mailerheaders van uitgaande berichten verwijderen
admin-settings-help-privacy-strip-mailer-headers = Verwijder X-Mailer, User-Agent en X-MimeOLE op het hoogste niveau uit native SMTP- en EWS-e-mail. Bijgevoegde berichten en rapportpayloads blijven intact. Verzendingen via de bovenliggende JMAP-proxy vallen hier niet onder.
admin-settings-key-images-trusted-hosts = Vertrouwde afbeeldingshosts
admin-settings-help-images-trusted-hosts = Maximaal 100 afbeeldingshosts, en hun subdomeinen, waarvan externe afbeeldingen zonder toestemming per bericht worden geladen wanneer externe inhoud op vragen staat. Overschrijft nooit de blokkeermodus; een geblokkeerde host blijft geblokkeerd.
admin-settings-key-images-blocked-hosts = Geblokkeerde afbeeldingshosts
admin-settings-help-images-blocked-hosts = Maximaal 100 afbeeldingshosts, en hun subdomeinen, waarvan externe afbeeldingen in geen enkele modus worden geladen. Gaat voor op vertrouwde hosts en op elke toestemming.
admin-settings-key-links-trusted-domains = Vertrouwde linkdomeinen
admin-settings-help-links-trusted-domains = Maximaal 100 exacte domeinen waarvan links worden geopend vanuit Links controleren zonder de bestemmingsstap. Geen jokertekens, schema's, poorten of IP-adressen. Gewone HTTP-links worden altijd gecontroleerd. Vergrendel deze instelling om te voorkomen dat gebruikers de lijst wijzigen.
admin-settings-key-links-trusted-subdomains = Subdomeinen van vertrouwde linkdomeinen vertrouwen
admin-settings-help-links-trusted-subdomains = Uit: alleen het exacte domein wordt vertrouwd. Aan: namen die eindigen op .<domain> worden ook vertrouwd. Er wordt op hele labels vergeleken, dus example.com vertrouwt evilexample.com nooit.
admin-settings-key-cap-compose-rich-text = Opstellen met opgemaakte tekst
admin-settings-help-cap-compose-rich-text = Of gebruikers opgemaakte (HTML-)berichten mogen schrijven. Uit verstuurt alleen platte tekst.
admin-settings-key-cap-compose-attachments = Bijlagen in nieuwe berichten
admin-settings-help-cap-compose-attachments = Of gebruikers bestanden mogen bijvoegen aan berichten die ze schrijven. Inlineafbeeldingen in de tekst tellen niet mee.
admin-settings-key-cap-compose-share-links = Deellinks voor grote bestanden
admin-settings-help-cap-compose-share-links = Of gebruikers een groot bestand als Nextcloud-deellink mogen verzenden in plaats van als bijlage.
admin-settings-key-cap-compose-scheduled-send = Gepland verzenden
admin-settings-help-cap-compose-scheduled-send = Of gebruikers een bericht mogen plannen om later te worden verzonden.
admin-settings-key-cap-compose-read-receipts = Leesbevestigingen aanvragen
admin-settings-help-cap-compose-read-receipts = Of gebruikers ontvangers om een leesbevestiging mogen vragen.
admin-settings-key-cap-signatures = Handtekeningen
admin-settings-help-cap-signatures = Of gebruikers hun eigen handtekeningen mogen maken en bewerken.
admin-settings-key-cap-identities = Verzendidentiteiten
admin-settings-help-cap-identities = Of gebruikers de identiteiten waarmee ze verzenden mogen toevoegen en bewerken.
admin-settings-key-cap-keys = OpenPGP- en S/MIME-sleutels
admin-settings-help-cap-keys = Of gebruikers de module Sleutels hebben en berichten mogen ondertekenen en versleutelen.
admin-settings-key-cap-links-trusted = Vertrouwde linkdomeinen bewerken
admin-settings-help-cap-links-trusted = Of gebruikers hun eigen lijst met vertrouwde linkdomeinen mogen wijzigen. De lijst van de beheerder blijft gelden.
admin-settings-key-cap-zoom = Zoom van lezer
admin-settings-help-cap-zoom = Of gebruikers de berichtlezer mogen zoomen.
admin-settings-key-cap-mail-focused-inbox = Gerichte inbox
admin-settings-help-cap-mail-focused-inbox = Of gebruikers de inboxtabbladen Gericht en Overig mogen inschakelen.
admin-settings-key-cap-views-events = Weergave Gebeurtenissen
admin-settings-help-cap-views-events = Of gebruikers de weergave Gebeurtenissen van agenda-e-mail hebben.
admin-settings-key-cap-views-attachments = Weergave Bijlagen
admin-settings-help-cap-views-attachments = Of gebruikers de weergave Bijlagen hebben. De berichten blijven bereikbaar via zoeken.
admin-settings-key-cap-search-operators = Zoekoperatoren
admin-settings-help-cap-search-operators = Of gebruikers mogen zoeken met operatoren zoals from:, subject: of has:attachment.
admin-settings-key-cap-saved-searches = Opgeslagen zoekopdrachten
admin-settings-help-cap-saved-searches = Of gebruikers zoekopdrachten mogen opslaan.
admin-settings-key-cap-vacation = Automatisch antwoord bij afwezigheid
admin-settings-help-cap-vacation = Of gebruikers een automatisch afwezigheidsantwoord mogen instellen.
admin-settings-key-cap-labels = Labels
admin-settings-help-cap-labels = Of gebruikers labels op berichten mogen zetten.
admin-settings-key-cap-snooze = Uitstellen
admin-settings-help-cap-snooze = Of gebruikers berichten mogen uitstellen. Een uitstel beëindigen blijft mogelijk.
admin-settings-key-cap-follow-up = Herinneringen voor opvolging
admin-settings-help-cap-follow-up = Of gebruikers herinneringen voor opvolging op berichten mogen instellen. Er een wissen blijft mogelijk.
admin-settings-key-cap-masked = Gemaskeerde adressen
admin-settings-help-cap-masked = Of gebruikers gemaskeerde adressen mogen maken en beheren.
admin-settings-key-cap-directory = Zoeken in de directory
admin-settings-help-cap-directory = Of gebruikers in de directory van de organisatie mogen zoeken.
admin-settings-key-cap-mcp = MCP-toegang
admin-settings-help-cap-mcp = Of de sleutels en tokens van de gebruiker het MCP-eindpunt mogen gebruiken.
admin-settings-key-cap-appearance-custom = Uiterlijk aanpassen
admin-settings-help-cap-appearance-custom = Of gebruikers hun thema, accent, aangepaste kleuren en badgekleuren, dichtheid en lettertype mogen wijzigen. Uit behoudt het uiterlijk van de beheerder.
admin-settings-key-cap-appearance-custom-code = Eigen CSS en scripts
admin-settings-help-cap-appearance-custom-code = Of gebruikers hun eigen CSS en uiterlijkscripts mogen opslaan. De implementatieschakelaars onder Uiterlijk blijven hierboven gelden.
admin-settings-key-cap-avatars = Postvakavatars
admin-settings-help-cap-avatars = Of gebruikers postvakavatars mogen wijzigen.
admin-settings-key-cap-shortcuts = Eigen sneltoetsen
admin-settings-help-cap-shortcuts = Of gebruikers sneltoetsen mogen wijzigen. De standaardsneltoetsen blijven werken.
admin-settings-key-cap-analytics = Productanalyse
admin-settings-help-cap-analytics = Of gebruikers om toestemming voor analyse wordt gevraagd en gebruiksgebeurtenissen mogen verzenden.
admin-settings-key-cap-apps-native = Desktop- en mobiele apps
admin-settings-help-cap-apps-native = Of gebruikers de desktop- en mobiele apps mogen gebruiken.
admin-settings-key-access-protocols = Protocollen voor e-mailtoegang
admin-settings-help-access-protocols = Met welke mailserverprotocollen deze gebruikers mogen inloggen: imap, pop3, ews. Logins via een JMAP-server volgen die server.
admin-settings-key-limit-linked-mailboxes = Gekoppelde postvakken per gebruiker
admin-settings-help-limit-linked-mailboxes = Hoeveel postvakken een gebruiker mag koppelen, onder het servermaximum. 0 betekent dat alleen het servermaximum geldt.
admin-settings-key-login-show-account-help-notice = Accounthulpmelding tonen bij inloggen
admin-settings-help-login-show-account-help-notice = Toon een korte uitleg na een mislukte login. Standaard verborgen; de melding onthult niet of een account bestaat of is uitgeschakeld.
admin-settings-key-appearance-leetspeak = Leetspeak-tekststijl
admin-settings-help-appearance-leetspeak = Toon interfacelabels en alleen-lezen berichttekst in leetspeak. Het originele bericht en bewerkbare tekst blijven ongewijzigd. Gebruikers kunnen normale tekst kiezen, tenzij deze instelling is vergrendeld.
admin-settings-key-mail-cache-enabled = Sessiecache inschakelen
admin-settings-key-mail-cache-mailboxes = Berichtenlijsten van mappen cachen
admin-settings-key-mail-cache-messages = Geopende berichten cachen
admin-settings-key-mail-cache-fresh-seconds = Actualiteitsinterval (seconden)
admin-settings-key-mail-cache-max-age-seconds = Maximale leeftijd in cache (seconden)
admin-settings-key-mail-cache-max-entries = Maximaal aantal items in cache
admin-settings-key-mail-cache-max-bytes = Geheugenbudget (bytes)
admin-settings-key-mail-cache-prefetch-enabled = Speculatief vooraf laden inschakelen
admin-settings-key-mail-cache-hover-enabled = Vooraf laden bij aanwijzen van een map
admin-settings-key-mail-cache-focus-enabled = Vooraf laden wanneer een map toetsenbordfocus krijgt
admin-settings-key-mail-cache-dwell-ms = Vertraging bij aanwijzen en focus (milliseconden)
admin-settings-key-mail-cache-idle-enabled = Mappen vooraf laden tijdens inactiviteit
admin-settings-key-mail-cache-idle-delay-ms = Inactiviteitsvertraging (milliseconden)
admin-settings-key-mail-cache-max-concurrency = Maximaal aantal gelijktijdige voorafgaande laadacties
admin-settings-key-mail-cache-max-queued = Maximaal aantal voorafgaande laadacties in de wachtrij
admin-settings-key-mail-cache-preload-messages = Speculatieve berichtteksten toestaan
admin-settings-key-mail-cache-excluded-folders = Mappen uitgesloten van vooraf laden
admin-settings-key-mail-cache-refresh-on-open = Gecachte inhoud vernieuwen bij openen
admin-settings-help-mail-cache-refresh-on-open = Gecachte inhoud vernieuwen bij openen
admin-settings-key-mail-templates-enabled = Antwoordsjablonen
admin-settings-help-mail-templates-enabled = Schakel persoonlijke en gedeelde antwoordsjablonen in platte tekst in de editor in. Gebruikers kunnen zich afmelden, tenzij deze instelling is vergrendeld.
admin-settings-key-mail-templates-allow-personal = Persoonlijke sjablonen toestaan
admin-settings-help-mail-templates-allow-personal = Sta gebruikers toe hun eigen antwoordsjablonen te maken. Gedeelde sjablonen worden afzonderlijk door beheerders beheerd.
admin-settings-key-mail-templates-shared = Gedeelde antwoordsjablonen
admin-settings-help-mail-templates-shared = Beheer herbruikbare antwoorden in platte tekst voor dit bereik. Sjablonen stellen nooit ontvangers in en verzenden geen berichten.
admin-settings-key-mail-templates-personal = Persoonlijke antwoordsjablonen
admin-settings-help-mail-templates-personal = Persoonlijke herbruikbare antwoorden in platte tekst. Maken of bewerken vereist dat antwoordsjablonen en persoonlijke sjablonen zijn ingeschakeld.
admin-settings-key-mail-compose-window = Editorvenster
admin-settings-help-mail-compose-window = Open nieuwe concepten in een werkruimtetabblad of een apart venster. Geblokkeerde pop-ups vallen terug op een tabblad.
admin-settings-key-mail-reply-copy-to-original = Antwoorden in hun oorspronkelijke map houden
admin-settings-help-mail-reply-copy-to-original = Sla een kopie van een verzonden antwoord op in de map van het oorspronkelijke bericht, waar dat wordt ondersteund.
admin-settings-key-mail-compose-check-invalid-recipients = Controleren op ongeldige ontvangers vóór verzenden
admin-settings-help-mail-compose-check-invalid-recipients = Wat er gebeurt wanneer een bericht naar een ongeldig adres wordt verzonden: toestaan, waarschuwen, waarschuwen en de keuze onthouden, of weigeren.
admin-settings-key-mail-compose-check-missing-attachment = Controleren op ontbrekende bijlage vóór verzenden
admin-settings-help-mail-compose-check-missing-attachment = Wat er gebeurt wanneer de tekst een bijlage noemt maar er geen is bijgevoegd: toestaan, waarschuwen, waarschuwen en de keuze onthouden, of weigeren.
admin-settings-key-mail-compose-check-empty-subject = Controleren op leeg onderwerp vóór verzenden
admin-settings-help-mail-compose-check-empty-subject = Wat er gebeurt wanneer een bericht geen onderwerp heeft: toestaan, waarschuwen, waarschuwen en de keuze onthouden, of weigeren.
admin-settings-key-mail-compose-auto-chip-recipients = Getypte ontvangers automatisch afronden
admin-settings-help-mail-compose-auto-chip-recipients = Maak van een getypt adres een ontvangerschip wanneer er een spatie, komma of puntkomma op volgt.
admin-settings-key-mail-compose-header-collapse = Editorheader
admin-settings-help-mail-compose-header-collapse = Hoe de rijen Aan, Cc, Bcc en Onderwerp van de editor zich gedragen: uitgevouwen, ingeklapt tot een samenvatting van één regel, of auto (ingeklapt tijdens het schrijven van de tekst, open wanneer een veld focus heeft). Gebruikers kunnen hem zelf in- of uitklappen, tenzij dit is vergrendeld.
admin-settings-key-mail-compose-recipient-separator = Scheidingsteken voor ontvangers
admin-settings-help-mail-compose-recipient-separator = Het scheidingsteken dat na een afgeronde ontvanger wordt ingevoegd: een komma of een puntkomma.
admin-settings-key-mail-reply-from-mode = Identiteit voor antwoorden en doorsturen
admin-settings-help-mail-reply-from-mode = Vanuit welke identiteit een antwoord of doorgestuurd bericht wordt verzonden: die waaraan het origineel was gericht, de standaard, een keuzelijst of de momenteel geselecteerde.
admin-settings-key-mail-forward-mode = Standaardformaat voor doorsturen
admin-settings-help-mail-forward-mode = Stuur tekst inline door of voeg het originele bericht toe als .eml-bestand.
admin-settings-key-mail-show-go-to-top = Knop Naar boven tonen
admin-settings-help-mail-show-go-to-top = Toon in de berichtenlijst een knop Naar boven die terugscrolt naar het eerste bericht. Standaard uit.
admin-settings-key-mail-show-create-folder-button = Knop Map maken tonen
admin-settings-help-mail-show-create-folder-button = Toon in de zijbalk een knop Map maken met alleen een pictogram naast Opstellen. Standaard uit.
admin-settings-key-security-protection-user-agent-mode = Observatiemodus voor user-agent
admin-settings-help-security-protection-user-agent-mode = Standaard enforce: geselecteerde herkende bot-headers worden direct geblokkeerd wanneer botblokkering bij het eerste verzoek is ingeschakeld. Observe registreert beslissingen zonder te weigeren; off schakelt deze dimensie uit. Header-claims zijn geen geverifieerde identiteiten.
admin-settings-key-security-protection-user-agent-block-bots = Herkende bots blokkeren bij het eerste verzoek
admin-settings-help-security-protection-user-agent-block-bots = Standaard waar. Geselecteerde zoek-, AI-, automatiserings- en aangepaste bot-headers leiden in Enforce-modus bij het eerste verzoek tot een tijdelijke HTTP 429-blokkade. Observe registreert alleen; onwaar herstelt de eerdere verzoekbudgetten. Ontbrekende/ongebruikelijke headers, bereik en uitzonderingen behouden hun bestaande gedrag.
admin-settings-key-security-protection-user-agent-categories = User-agentcategorieën
admin-settings-key-security-protection-user-agent-weak-requests = Drempel voor verzoeken met zwak signaal
admin-settings-key-security-protection-user-agent-crawler-requests = Drempel voor crawlerverzoeken
admin-settings-key-security-protection-user-agent-window-secs = Observatievenster voor user-agent (seconden)
admin-settings-key-security-protection-user-agent-max-action = Maximale user-agentreactie
admin-settings-help-security-protection-user-agent-max-action = Maximale opschaling voor zwak bewijs of bewijs van herhaalde verzoeken, standaard afremmen. Deze limiet beperkt de blokkades bij het eerste verzoek van geselecteerde herkende bot-headers niet.
admin-settings-key-security-protection-user-agent-scope = Bereik van user-agentbeleid
admin-settings-key-security-protection-user-agent-adaptive = Adaptieve user-agentobservaties
admin-settings-key-security-protection-user-agent-patterns = User-agentpatronen
admin-settings-key-security-protection-user-agent-exceptions = User-agentuitzonderingen
admin-settings-key-mail-reader-unsafe-html-mode = Mogelijk onveilige HTML in berichten
admin-settings-help-mail-reader-unsafe-html-mode = Ask biedt standaard een knop voor gesaneerde HTML per bericht. Strict toont alleen veilige tekst; sanitized toont automatisch opgeschoonde HTML. Actieve inhoud en automatische externe bronnen blijven geblokkeerd. Vergrendel om dit beleid af te dwingen.
admin-settings-key-reader-attachments-density = Dichtheid van bijlagen
admin-settings-help-reader-attachments-density = Standaard compacte bestandsknoppen, of comfortabele voorbeelden. Bepaalt de weergave wanneer de bijlagen van een bericht zijn uitgevouwen.
admin-settings-key-cap-compose-open-tracking = Registratie van berichtopeningen toestaan
admin-settings-help-cap-compose-open-tracking = Of dit account berichtopeningen mag registreren. Vereist een ingestelde openbare URL; uitgeschakelde accounts kunnen geen registratietokens maken of koppelen.
admin-settings-key-mail-open-tracking = Openingen standaard registreren
admin-settings-help-mail-open-tracking = Standaardkeuze voor registratie per bericht voor in aanmerking komende opgemaakte berichten. Gebruikers kunnen deze overschrijven, tenzij vergrendeld; het uitschakelen van de registratiemogelijkheid gaat altijd voor.
admin-settings-key-cap-reading-original-html = Weergave van originele HTML toestaan
admin-settings-help-cap-reading-original-html = Of gebruikers voor afzonderlijke berichten kunnen kiezen voor de weergave van originele HTML. De standaard gesaneerde renderer en de beveiliging van het geïsoleerde kader blijven gelden.
admin-settings-key-mail-reader-allow-original-html = Weergave van originele HTML aanbieden
admin-settings-help-mail-reader-allow-original-html = Toon de optie per bericht om originele HTML te gebruiken wanneer de leesmogelijkheid dit toestaat. Standaard uit; dit wijzigt de standaardberichtrenderer niet.
admin-settings-key-privacy-tracker-override-allowed = Toestemming voor trackers per bericht aanbieden
admin-settings-help-privacy-tracker-override-allowed = Toon de actie Trackers toestaan in de privacybalk van het bericht. Standaard beschikbaar, maar nooit actief zonder expliciete toestemming. Een vergrendeld beleid voor trackerblokkering verbiedt de overschrijving nog steeds.
admin-settings-key-privacy-max-concurrent-global = Globale parallelle afbeeldingsophalingen
admin-settings-help-privacy-max-concurrent-global = Maximaal aantal actieve netwerkverzoeken voor afbeeldingen in dit serverproces. Bereik 1–256; standaard 128. Limieten per bericht en per account blijven gelden.
admin-settings-key-privacy-max-concurrent-pipeline = Gelijktijdige verwerkingspijplijnen voor afbeeldingen
admin-settings-help-privacy-max-concurrent-pipeline = Maximaal aantal afbeeldingen dat samen wordt toegelaten voor ophalen, decoderen en cachepublicatie. Bereik 1–256; standaard 128. Netwerk- en decoderplafonds worden afzonderlijk afgedwongen.
admin-settings-key-privacy-max-concurrent-decodes = Parallelle afbeeldingsdecoders
admin-settings-help-privacy-max-concurrent-decodes = Maximaal aantal afbeeldingsdecoderingen dat tegelijk loopt. Bereik 1–16; standaard 10. Hogere waarden gebruiken meer CPU en geheugen.
admin-settings-key-privacy-max-queued-requests = Wachtende afbeeldingsverzoeken
admin-settings-help-privacy-max-queued-requests = Maximaal aantal verzoeken dat wacht op netwerk- of verwerkingscapaciteit voor afbeeldingen. Bereik 16–16384; standaard 2048. Overloop of verlopen van de wachtrij blijft opnieuw te proberen.
admin-settings-key-privacy-rate-burst = Burstruimte voor afbeeldingsverzoeken
admin-settings-help-privacy-rate-burst = Begin- en maximumaantal ophaaltokens per account, aangevuld met de ingestelde frequentie van afbeeldingsverzoeken. Bereik 1–12000; standaard 1024. Gecachte successen verbruiken geen tokens.
admin-settings-key-privacy-image-rate-mode = Dimensionering van frequentie voor afbeeldingsverzoeken
admin-settings-help-privacy-image-rate-mode = Automatic schaalt burst en aanvulling per account op basis van replicacapaciteit, wachtrijbelasting en het aantal afbeeldingen van het bericht, binnen de onderstaande vloeren en plafonds. Manual gebruikt de instellingen voor burstruimte en verzoekfrequentie exact. Standaard automatic. Verzoeken boven de ruimte wachten op een token tot de deadline van de ophaalwachtrij; authenticatie en toestemmingen worden eerst gecontroleerd.
admin-settings-help-mail-cache-enabled = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-mailboxes = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-messages = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-fresh-seconds = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-max-age-seconds = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-max-entries = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-max-bytes = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-prefetch-enabled = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-hover-enabled = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-focus-enabled = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-dwell-ms = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-idle-enabled = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-idle-delay-ms = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-max-concurrency = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-max-queued = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-preload-messages = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-mail-cache-excluded-folders = Bewaart recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist het. Vooraf laden markeert e-mail nooit als gelezen en laadt nooit externe afbeeldingen.
admin-settings-help-security-protection-user-agent-categories = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-weak-requests = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-crawler-requests = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-window-secs = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-scope = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-adaptive = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-patterns = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.
admin-settings-help-security-protection-user-agent-exceptions = Configureert dit onderdeel van de optionele observatie van user-agents. Door de client opgegeven user-agent-tekst is een zwak signaal, geen bewijs van identiteit of misbruik.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-rate-burst-floor = Ondergrens automatische burstruimte
admin-settings-help-privacy-image-rate-burst-floor = Laagste burstruimte per account in de automatische snelheidsmodus. Bereik 1–12000; standaard 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Bovengrens automatische burstruimte
admin-settings-help-privacy-image-rate-burst-ceiling = Hoogste burstruimte per account in de automatische snelheidsmodus. Bereik 1–12000; standaard 2048.
admin-settings-key-privacy-image-rate-refill-floor = Ondergrens automatische aanvulling
admin-settings-help-privacy-image-rate-refill-floor = Laagste aanvulling per account, in ophaaltokens per minuut, in de automatische snelheidsmodus. Bereik 10–60000; standaard 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Bovengrens automatische aanvulling
admin-settings-help-privacy-image-rate-refill-ceiling = Hoogste aanvulling per account, in ophaaltokens per minuut, in de automatische snelheidsmodus. Bereik 10–60000; standaard 6000.
admin-settings-key-privacy-image-cache-mode = Grootte van de afbeeldingscache
admin-settings-help-privacy-image-cache-mode = Automatisch bepaalt de cachegrootte aan de hand van het geheugen dat overblijft na de reserveringen voor decoder en pipeline en verlengt de levensduur zolang de hitratio hoog is en er geheugen vrij is. Handmatig gebruikt de instellingen voor cachegrootte, aantal items en levensduur precies zoals ingevoerd. Standaard automatisch.
admin-settings-key-privacy-image-cache-bytes-floor = Ondergrens automatische cachegrootte
admin-settings-help-privacy-image-cache-bytes-floor = Kleinste afbeeldingscache in bytes in de automatische modus. Bereik 0–1073741824; standaard 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Bovengrens automatische cachegrootte
admin-settings-help-privacy-image-cache-bytes-ceiling = Grootste afbeeldingscache in bytes in de automatische modus. Bereik 0–1073741824; standaard 268435456.
admin-settings-key-privacy-cache-entries = Limiet aantal items in de afbeeldingscache
admin-settings-help-privacy-cache-entries = Maximaal aantal gecachete afbeeldingen; het vaste aantal in de handmatige modus en de bovengrens in de automatische modus. Bereik 16–65536; standaard 4096.
admin-settings-key-privacy-cache-entry-bytes = Grootste gecachete afbeelding
admin-settings-help-privacy-cache-entry-bytes = Grootste afzonderlijke opnieuw gecodeerde afbeelding die in de cache wordt bewaard, in bytes. Grotere afbeeldingen worden wel geleverd maar niet bewaard. Bereik 65536–16777216; standaard 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Ondergrens automatische cacheleven
admin-settings-help-privacy-image-cache-ttl-floor-secs = Kortste cachelevensduur in seconden in de automatische modus, gebruikt bij een lage hitratio of onder geheugendruk. Bereik 0–86400; standaard 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Bovengrens automatische cachelevensduur
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Langste cachelevensduur in seconden in de automatische modus, gebruikt bij een hoge hitratio en vrij geheugen. Bereik 0–86400; standaard 21600.
admin-settings-key-privacy-cache-ttl-secs = Levensduur cache voor geslaagde afbeeldingen
admin-settings-help-privacy-cache-ttl-secs = Aantal seconden dat gedecodeerde afbeeldingen in dit proces herbruikbaar blijven. Bereik 0–86400; standaard 3600. Nul schakelt hergebruik van geslaagde afbeeldingen uit; autorisatie wordt altijd eerst gecontroleerd.
admin-settings-key-privacy-image-latency-tracking = Snelheidsregistratie van de afbeeldingsproxy
admin-settings-help-privacy-image-latency-tracking = Legt vast hoe lang elke via de proxy geleverde afbeelding duurt (mediaan, p95, p99 en de tijd voor wachten, ophalen en decoderen) als tellingen in vaste tijdsvakken in het geheugen. Er worden geen URL's, hosts of accounts opgeslagen. Standaard aan; de geschiedenis wordt bij een herstart gewist en geldt per serverproces. Uitschakelen stopt nieuwe registratie; de sectie Snelheid blijft tonen wat is vastgelegd.
admin-settings-key-privacy-negative-cache-ttl-secs = Levensduur cache voor mislukte afbeeldingen
admin-settings-help-privacy-negative-cache-ttl-secs = Aantal seconden dat in aanmerking komende fouten van de bronserver worden onthouden om herhaalde verzoeken te voorkomen. Bereik 0–300; standaard 10. Nul schakelt hergebruik van fouten uit. Een expliciete nieuwe poging wist de betrokken fouten.
admin-settings-key-privacy-image-retry-attempts = Automatische nieuwe pogingen voor afbeeldingen
admin-settings-help-privacy-image-retry-attempts = Maximaal aantal nieuwe pogingen bij tijdelijke afbeeldingsfouten tijdens het laden van één zichtbaar bericht. Bereik 0–8; standaard 3. Nul schakelt automatische nieuwe pogingen uit; handmatig opnieuw proberen blijft mogelijk.
admin-settings-key-privacy-image-retry-delay-ms = Vertraging automatische nieuwe poging voor afbeeldingen
admin-settings-help-privacy-image-retry-delay-ms = Milliseconden tussen automatische nieuwe pogingen, nadat het huidige frame is afgerond. Bereik 500–60000; standaard 5000. Definitieve weigeringen worden niet automatisch opnieuw geprobeerd.
admin-settings-key-privacy-image-decode-timeout-ms = Time-out voor afbeeldingsdecodering
admin-settings-help-privacy-image-decode-timeout-ms = Aantal milliseconden (kloktijd) dat één gesandboxte afbeeldingsdecodering na het ophalen mag duren. Bereik 1000–60000; standaard 10000. Verhoog dit voor grote afbeeldingen; netwerktime-outs staan hier los van.
admin-settings-key-privacy-image-concurrency-mode = Concurrentieprofiel voor afbeeldingen
admin-settings-help-privacy-image-concurrency-mode = Automatisch past zich aan aan de CPU-capaciteit, het containergeheugen en het aantal goedgekeurde afbeeldingen per bericht. Weinig resources, Gebalanceerd en Hoge doorvoer passen zich ook aan, met andere veiligheidsgrenzen. Handmatig gebruikt de numerieke bovengrenzen rechtstreeks. Een profiel toepassen wijzigt alleen deze instelling.
admin-settings-cat-image-proxy = Afbeeldingsproxy
admin-settings-key-privacy-image-ram-max-mi-b = Maximum RAM-laag
admin-settings-help-privacy-image-ram-max-mi-b = Bovengrens in MiB voor de afbeeldingslaag in het geheugen, toegepast bovenop de automatische of handmatige cachegrootte. Nul stelt geen limiet in. Houd deze klein en laat de schijflaag het grootste deel bewaren. Bereik 0–1048576; standaard 0.
admin-settings-key-privacy-image-disk-enabled = Schijflaag voor afbeeldingen
admin-settings-help-privacy-image-disk-enabled = Bewaart reeds opnieuw gecodeerde afbeeldingen op schijf achter de RAM-laag. Standaard uit; als de map ontbreekt, vol is of alleen-lezen, gaat de cache stilzwijgend alleen in RAM door. Alleen uitvoer van de jail wordt opgeslagen, onder gehashte bestandsnamen.
admin-settings-key-privacy-image-disk-path = Schijfmap voor afbeeldingen
admin-settings-help-privacy-image-disk-path = Absolute map voor de schijflaag. Leeg gebruikt een map image-cache naast het databasebestand. Koppel daar een beschrijfbaar volume wanneer het rootbestandssysteem alleen-lezen is. Relatieve paden en .. worden geweigerd en laten de laag uit.
admin-settings-key-privacy-image-disk-mode = Grootte van de afbeeldingsschijf
admin-settings-help-privacy-image-disk-mode = Automatisch gebruikt een tiende van de ruimte die deze cache kan gebruiken (vrije ruimte plus wat al in gebruik is), tussen de ondergrens en bovengrens en nooit meer dan de helft van die ruimte. Handmatig gebruikt precies de onderstaande maximumgrootte. Standaard automatisch.
admin-settings-key-privacy-image-disk-max-mi-b = Grootte afbeeldingsschijf (handmatig)
admin-settings-help-privacy-image-disk-max-mi-b = Grootte van de schijflaag in MiB in de handmatige modus; de uitlezing toont in beide gevallen de effectieve grootte. Bereik 1–1048576; standaard 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Ondergrens automatische schijfgrootte
admin-settings-help-privacy-image-disk-floor-mi-b = Kleinste schijflaag in MiB in de automatische modus; ook gebruikt zolang de vrije ruimte onbekend is. Bereik 0–1048576; standaard 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Bovengrens automatische schijfgrootte
admin-settings-help-privacy-image-disk-ceiling-mi-b = Grootste schijflaag in MiB in de automatische modus. Bereik 0–1048576; standaard 4096.
admin-settings-key-privacy-image-disk-entries = Limiet aantal items op schijf
admin-settings-help-privacy-image-disk-entries = Maximaal aantal op schijf bewaarde afbeeldingen. Elk item kost ook een vaste hoeveelheid RAM in de index, die meetelt voor de RAM-laag. Bereik 16–1048576; standaard 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Grootste afbeelding op schijf
admin-settings-help-privacy-image-disk-entry-bytes = Grootste afzonderlijke opnieuw gecodeerde afbeelding die naar schijf wordt geschreven, in bytes. Afbeeldingen boven de RAM-itemlimiet maar binnen deze limiet gaan rechtstreeks naar schijf. Bereik 65536–67108864; standaard 8388608.
admin-settings-key-privacy-image-disk-min-hits = Hits vóór een schijfschrijfactie
admin-settings-help-privacy-image-disk-min-hits = Aantal hits dat een RAM-item nodig heeft voordat het bij verwijdering naar schijf wordt geschreven in plaats van te vervallen. Nul schrijft elk verwijderd item waarvan de score de degradatiedrempel haalt. Bereik 0–100; standaard 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Schrijfwachtrij voor schijf
admin-settings-help-privacy-image-disk-write-queue-mi-b = Afbeeldingsgegevens in MiB die op de schijfschrijver op de achtergrond mogen wachten. Dit telt mee voor de RAM-laag; daarboven worden verwijderde afbeeldingen weggegooid in plaats van in de wachtrij gezet. Bereik 1–256; standaard 8.
admin-settings-key-privacy-image-ttl-step-secs = Levensduurstap per hit
admin-settings-help-privacy-image-ttl-step-secs = Seconden die bij een hit worden opgeteld bij de resterende levensduur van een afbeelding, vermenigvuldigd met 1 plus de logaritme van het verouderde aantal hits, zodat veelgebruikte afbeeldingen langer leven. Nul houdt elke afbeelding op de basislevensduur. Bereik 0–86400; standaard 600.
admin-settings-key-privacy-image-ttl-max-secs = Langste verlengde levensduur
admin-settings-help-privacy-image-ttl-max-secs = Langste resterende levensduur in seconden die hits kunnen opbouwen. Bereik 60–2592000; standaard 86400.
admin-settings-key-privacy-image-max-age-secs = Maximale leeftijd van afbeeldingen
admin-settings-help-privacy-image-max-age-secs = Leeftijd in seconden waarna een afbeelding opnieuw wordt opgehaald, ongeacht het aantal hits. Het ophaalpad bewaart geen ETag of Last-Modified van de bronserver, dus er is geen voorwaardelijke hervalidatie; de afbeelding wordt gewoon opnieuw opgehaald. Bereik 3600–7776000; standaard 604800.
admin-settings-key-privacy-image-score-half-life-secs = Halfwaardetijd van de score
admin-settings-help-privacy-image-score-half-life-secs = Seconden waarna het aantal hits en de recentheid van een afbeelding gehalveerd zijn in de prioriteitsscore die RAM, schijf en verwijdering bepaalt. Bereik 60–604800; standaard 3600.
admin-settings-key-privacy-image-score-demote-milli = Degradatiescore
admin-settings-help-privacy-image-score-demote-milli = Score, in duizendsten, waaronder een RAM-afbeelding wordt gedegradeerd naar schijf of verwijderd. De promotiedrempel is deze waarde plus de hysterese. Bereik 0–10000; standaard 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Promotiehysterese
admin-settings-help-privacy-image-score-hysteresis-percent = Percentage waarmee de promotiedrempel boven de degradatiedrempel ligt. Een score tussen beide laat de afbeelding waar ze is. Bereik 0–500; standaard 50.
admin-settings-key-privacy-image-context-open-percent = Gewicht geopend bericht
admin-settings-help-privacy-image-context-open-percent = Scoregewicht in procenten voor afbeeldingen die voor een bericht zijn opgevraagd. De afbeeldingen van een bericht tellen na tien minuten inactiviteit als Recent. Bereik 100–1000; standaard 200.
admin-settings-key-privacy-image-context-recent-percent = Gewicht recent bericht
admin-settings-help-privacy-image-context-recent-percent = Scoregewicht in procenten voor onlangs gebruikte afbeeldingen. Verzoeken zonder bericht wegen 100. Bereik 100–1000; standaard 150.
admin-settings-key-privacy-image-account-share-percent = RAM-aandeel per account
admin-settings-help-privacy-image-account-share-percent = Grootste aandeel van de RAM-laag dat de invoegingen van één account mogen vullen terwijl andere accounts de cache gebruiken; daarboven verwijdert het account eerst zijn eigen afbeeldingen met de laagste score. Eén enkel account mag alles gebruiken. Bereik 1–100; standaard 50.
admin-settings-key-privacy-image-sweep-secs = Interval cache-opruiming
admin-settings-help-privacy-image-sweep-secs = Seconden tussen opruimrondes op de achtergrond die afbeeldingen laten verlopen, een beperkte batch opnieuw scoren, koude RAM-afbeeldingen onder druk degraderen en de vrije schijfruimte opnieuw controleren. Bereik 5–3600; standaard 30.
admin-settings-key-privacy-image-snapshot-secs = Interval indexmomentopname
admin-settings-help-privacy-image-snapshot-secs = Seconden tussen atomaire momentopnamen van de schijfindex, die hittellingen en verlengde levensduur over herstarts heen bewaren zonder een schrijfactie per hit. Nul schakelt momentopnamen uit; na een herstart wordt dan de header van elk bestand gelezen. Bereik 0–86400; standaard 300.
admin-settings-key-privacy-image-sweep-batch = Batchgrootte opruiming
admin-settings-help-privacy-image-sweep-batch = Aantal items dat één onderhoudsronde mag laten verlopen, opnieuw scoren of verwijderen. Begrenst het werk onder het cacheslot. Bereik 16–8192; standaard 256.
admin-image-proxy-title = Uitlezing afbeeldingscache
admin-image-proxy-unavailable = De uitlezing van de afbeeldingscache is niet beschikbaar.
admin-image-proxy-unknown = onbekend
admin-image-proxy-ram = RAM-laag: { $used } gebruikt van { $budget } (ingesteld { $configured }), { $entries } afbeeldingen, { $hot } warm, { $cold } koud, hitratio { $hit }.
admin-image-proxy-disk-off = Schijflaag: uit.
admin-image-proxy-disk = Schijflaag: { $bytes } gebruikt van effectief { $budget } ({ $mode }; ondergrens { $floor }, bovengrens { $ceiling }, handmatig { $manual }), { $entries } afbeeldingen, { $hot } warm, { $cold } koud, hitratio { $hit }.
admin-image-proxy-disk-inactive = Schijflaag: aan maar niet actief. { $reason }
admin-image-proxy-moves = Promoties per minuut: { $perMin }. Totaal promoties { $promotions }, degradaties { $demotions }, verwijderingen { $drops }. Indexitems: { $index }.
admin-image-proxy-refresh = Uitlezing vernieuwen
admin-settings-key-security-trust-enabled = Vertrouwensniveaus
admin-settings-help-security-trust-enabled = Geef elk verzoek een niveau: ingelogd, bekende bron (een recente geslaagde aanmelding vanaf dat adres) of onbekend. Alleen de toegestane hoeveelheden voor anonieme verzoeken veranderen; inloggegevens, tweede factoren en vergrendelingen worden nooit versoepeld.
admin-settings-key-security-trust-known-ttl-days = Venster bekende bron (dagen)
admin-settings-help-security-trust-known-ttl-days = Hoe lang een geslaagde aanmelding het adres als bekend laat gelden. Mislukte pogingen tellen nooit mee.
admin-settings-key-security-trust-known-capacity = Aantal bewaarde bekende bronnen
admin-settings-help-security-trust-known-capacity = Maximaal aantal bekende bronnen in het geheugen (elk met een gezouten hash, een land en een accounttelling). De oudste vervalt het eerst.
admin-settings-key-security-trust-persist = Bekende bronnen bewaren over herstarts heen
admin-settings-help-security-trust-persist = Sla de tabel met bekende bronnen (gesleutelde hashes, land, laatst gezien, aantal accounts) op in de instellingenopslag en laad die bij het opstarten. Items die ouder zijn dan het venster vervallen.
admin-settings-key-security-trust-persist-interval-secs = Opslaginterval bekende bronnen (seconden)
admin-settings-help-security-trust-persist-interval-secs = Seconden tussen opslagmomenten, 30 tot 3600. Er wordt alleen opgeslagen als de tabel is gewijzigd, en nog eens bij een nette afsluiting.
admin-settings-key-security-trust-unknown-percent = Toegestane hoeveelheid onbekende bron (%)
admin-settings-help-security-trust-unknown-percent = Toegestane hoeveelheid anonieme verzoeken van een onbekende bron, als percentage van de ingestelde limieten. 100 houdt de basis aan; lager is strenger.
admin-settings-key-security-trust-known-percent = Toegestane hoeveelheid bekende bron (%)
admin-settings-help-security-trust-known-percent = Toegestane hoeveelheid anonieme verzoeken van een bekende bron. 200 verdubbelt de verzoekbudgetten en halveert de wachttijd voor aanmelding en de moeilijkheid van de uitdaging. Omzeilt nooit inloggegevens, tweede factoren of vergrendelingen.
admin-settings-key-security-trust-ua-missing = Bots: lege user-agent
admin-settings-help-security-trust-ua-missing = Actie voor anonieme verzoeken met een lege user-agent: toestaan, beperken of blokkeren. Ingelogde sessies worden nooit beïnvloed.
admin-settings-key-security-trust-ua-uncommon = Bots: onbekende user-agent
admin-settings-help-security-trust-ua-uncommon = Actie voor anonieme verzoeken met een onbekende user-agent: toestaan, beperken of blokkeren.
admin-settings-key-security-trust-ua-automation = Bots: scripts en automatisering
admin-settings-help-security-trust-ua-automation = Actie voor anonieme verzoeken die zich voordoen als script, HTTP-bibliotheek, headless browser of algemene bot: toestaan, beperken of blokkeren.
admin-settings-key-security-trust-ua-search = Bots: zoekcrawlers
admin-settings-help-security-trust-ua-search = Actie voor anonieme verzoeken die zich voordoen als crawler van een zoekmachine: toestaan, beperken of blokkeren.
admin-settings-key-security-trust-ua-ai-training = Bots: crawlers voor AI-training
admin-settings-help-security-trust-ua-ai-training = Actie voor anonieme verzoeken die zich voordoen als GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended of Applebot-Extended: toestaan, beperken of blokkeren. Een claim is een header, geen geverifieerde identiteit.
admin-settings-key-security-trust-ua-ai-search = Bots: crawlers voor AI-zoeken
admin-settings-help-security-trust-ua-ai-search = Actie voor anonieme verzoeken die zich voordoen als OAI-SearchBot, PerplexityBot of Claude-SearchBot: toestaan, beperken of blokkeren.
admin-settings-key-security-trust-ua-ai-user = Bots: AI-ophalers namens gebruikers
admin-settings-help-security-trust-ua-ai-user = Actie voor anonieme verzoeken die zich voordoen als ChatGPT-User of Claude-User: toestaan, beperken of blokkeren.
admin-settings-key-security-trust-ua-custom = Bots: eigen patronen
admin-settings-help-security-trust-ua-custom = Actie voor anonieme verzoeken die overeenkomen met de eigen user-agentpatronen: toestaan, beperken of blokkeren.
admin-settings-key-security-trust-admin-geo-enabled = Beheerontdekking vanuit onbekende landen blokkeren
admin-settings-help-security-trust-admin-geo-enabled = Blokkeer een bron die de beheerconsole aftast vanuit een land zonder recente geslaagde aanmelding. Vereist een leesbare landendatabase en bescherming tegen beheerverkenning. MW_ADMIN_GEO_ENABLED=false schakelt dit uit voor herstel.
admin-settings-key-security-trust-admin-geo-countries = Altijd bekende landen (beheerblokkade)
admin-settings-help-security-trust-admin-geo-countries = ISO-landcodes in hoofdletters die altijd als bekend gelden, naast de landen van recente geslaagde aanmeldingen.
admin-settings-key-security-trust-admin-geo-lookup-failure = Bron zonder land
admin-settings-help-security-trust-admin-geo-lookup-failure = allow blokkeert niet en logt het geval; unknown behandelt een bron zonder gedetecteerd land als een onbekend land.
admin-settings-key-security-trust-admin-geo-threshold = Verkenningen vóór een blokkade
admin-settings-help-security-trust-admin-geo-threshold = Aantal verkenningen van de beheerconsole vanuit een onbekend land binnen het venster dat een blokkade start.
admin-settings-key-security-trust-admin-geo-window-secs = Verkenningsvenster (seconden)
admin-settings-help-security-trust-admin-geo-window-secs = Venster voor de verkenningsdrempel.
admin-settings-key-security-trust-admin-geo-block-on-first = Blokkeren bij de eerste verkenning
admin-settings-help-security-trust-admin-geo-block-on-first = Blokkeer bij de eerste verkenning vanuit een onbekend land. Vergroot de kans dat een reizende beheerder wordt buitengesloten.
admin-settings-key-security-trust-admin-geo-block-secs = Eerste blokkade (seconden)
admin-settings-help-security-trust-admin-geo-block-secs = Duur van de eerste blokkade.
admin-settings-key-security-trust-admin-geo-second-block-secs = Tweede blokkade (seconden)
admin-settings-help-security-trust-admin-geo-second-block-secs = Duur van de tweede blokkade van dezelfde bron binnen de herhalingsperiode.
admin-settings-key-security-trust-admin-geo-max-block-secs = Herhaalblokkade (seconden)
admin-settings-help-security-trust-admin-geo-max-block-secs = Duur van de derde en elke volgende blokkade binnen de herhalingsperiode.
admin-settings-key-security-trust-admin-geo-repeat-days = Herhalingsperiode (dagen)
admin-settings-help-security-trust-admin-geo-repeat-days = Een blokkade binnen dit aantal dagen na de vorige telt als herhaling en gebruikt de volgende, langere duur.
admin-settings-key-security-signin-account-geo-mode = Aanmeldlandbeleid per account
admin-settings-help-security-signin-account-geo-mode = Standaard uit. Blokkeer aanmeldingen vanuit landen die het account nog niet eerder heeft gebruikt, of vanuit elk land buiten de eigen toegestane lijst. IP-adressen en landen op de toelatingslijst komen altijd door. Geldt alleen voor nieuwe webmailaanmeldingen, nooit voor bestaande sessies of protocolclients. Landdetectie gebruikt een IP-geolocatiedatabase die onjuist of verouderd kan zijn; vpn's, mobiele providers, proxy's en reizen kunnen een legitieme aanmelding blokkeren of een buitenlandse doorlaten. Voeg vertrouwde IP-adressen of landen toe aan de toelatingslijst.
admin-settings-key-security-signin-account-geo-allow-countries = Toegestane landen van het account
admin-settings-help-security-signin-account-geo-allow-countries = ISO-landcodes in hoofdletters die voor het account altijd zijn toegestaan. Landdetectie gebruikt een IP-geolocatiedatabase die onjuist of verouderd kan zijn; vpn's, mobiele providers, proxy's en reizen kunnen een legitieme aanmelding blokkeren of een buitenlandse doorlaten. Voeg vertrouwde IP-adressen of landen toe aan de toelatingslijst.
admin-settings-key-security-signin-account-geo-allow-ips = Toegestane IP-adressen van het account
admin-settings-help-security-signin-account-geo-allow-ips = IP-adressen of CIDR-netwerken die voor het account altijd zijn toegestaan. Eén IPv6-adres staat voor zijn /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Toegestane landen voor alle accounts
admin-settings-help-security-signin-account-geo-global-allow-countries = ISO-landcodes in hoofdletters die voor alle accounts zijn toegestaan, opgeteld bij de eigen lijst van elk account.
admin-settings-key-security-signin-account-geo-global-allow-ips = Toegestane IP-adressen voor alle accounts
admin-settings-help-security-signin-account-geo-global-allow-ips = IP-adressen of CIDR-netwerken (een kantoor of vpn) die voor alle accounts zijn toegestaan, opgeteld bij de eigen lijst van elk account.
admin-settings-key-security-signin-account-geo-lookup-failure = Bron zonder land (accountbeleid)
admin-settings-help-security-signin-account-geo-lookup-failure = Wanneer de bron geen gedetecteerd land heeft: de aanmelding toestaan en loggen, of weigeren.
admin-settings-key-security-signin-account-geo-history-days = Landgeschiedenis van het account (dagen)
admin-settings-help-security-signin-account-geo-history-days = Hoeveel dagen een land in de eigen aanmeldgeschiedenis van een account blijft voor het beleid voor ongebruikelijke landen. Dit wordt los van het authenticatielogboek bewaard, dus het logboek inkorten verkort dit niet.
admin-settings-key-security-geo-no-database = Geen landendatabase: standaardgedrag
admin-settings-help-security-geo-no-database = Wat landfuncties doen als er geen database is geladen: toestaan, de bron als onbekend land behandelen of blokkeren (blokkeren geldt alleen voor aanmeldingen). De eigen instelling van een functie geldt ook en de strengste wint.
admin-settings-key-security-geo-not-found = Adres niet in de database: standaardgedrag
admin-settings-help-security-geo-not-found = Wat landfuncties doen als er wel een database is geladen maar geen item voor het adres bestaat. Privé-, loopback- en link-local-adressen zijn lokale netwerken en worden nooit beïnvloed.
admin-settings-key-security-signin-geo-no-database = Aanmeldbeleid van de instantie zonder database
admin-settings-help-security-signin-geo-no-database = Wanneer het landbeleid voor aanmelding van de instantie aan staat en er geen database is geladen: aanmeldingen blokkeren (fail closed) of ongecontroleerd toestaan. Lokale netwerken zijn altijd toegestaan.
admin-settings-key-security-geo-session-lookup-failure = Sessiemeldingen zonder land
admin-settings-help-security-geo-session-lookup-failure = Wanneer het land van een ingelogd verzoek niet kan worden bepaald: negeren, of één sessiemelding per sessie per dag geven. Blokkeert of herroept nooit een sessie.
admin-settings-key-security-protocol-framing-mode = Protocolbewaking: verzoekframing
admin-settings-help-security-protocol-framing-mode = Request-smuggling-klasse: Content-Length samen met Transfer-Encoding, Transfer-Encoding anders dan precies chunked (of bij HTTP/1.0 en HTTP/2), niet-numerieke of tegenstrijdige Content-Length. Afdwingen antwoordt met 400 en sluit de verbinding; alleen detecteren telt slechts. Een uitzondering per routevoorvoegsel geldt nooit voor deze klasse.
admin-settings-key-security-protocol-host-mode = Protocolbewaking: Host-syntaxis
admin-settings-help-security-protocol-host-mode = Syntaxis van de Host-header: ontbrekend bij HTTP/1.1, leeg, herhaald, of met userinfo, witruimte of padtekens. IPv6-literals, poorten en een afsluitende punt worden geaccepteerd. Afdwingen antwoordt met 400 en sluit de verbinding.
admin-settings-key-security-protocol-host-pinning-mode = Protocolbewaking: Host moet overeenkomen met de openbare naam
admin-settings-help-security-protocol-host-pinning-mode = Host die niet overeenkomt met de host van MW_PUBLIC_URL of een van de allowedHosts. Doet niets zolang geen van beide een host noemt. Standaard detecteren, omdat proxy's en gezondheidscontroles een interne naam kunnen sturen.
admin-settings-key-security-protocol-path-control-mode = Protocolbewaking: stuurtekens in het pad
admin-settings-help-security-protocol-path-control-mode = NUL, stuurtekens of gecodeerde CR/LF (%00, %0d, %0a) in het verzoekpad. Afdwingen antwoordt met 400. Wordt nooit uitgezonderd via een routevoorvoegsel.
admin-settings-key-security-protocol-target-mode = Protocolbewaking: trucs met het verzoekdoel
admin-settings-help-security-protocol-target-mode = Dubbele of te lange procentcodering van punten en schuine strepen, gecodeerde punten, puntsegmenten, lege padsegmenten (//), backslashes, ruwe niet-ASCII-bytes en een doel in absolute vorm dat een andere autoriteit noemt dan Host.
admin-settings-key-security-protocol-path-length-mode = Protocolbewaking: lengte van pad en query
admin-settings-help-security-protocol-path-length-mode = Pad of query langer dan maxPathBytes of maxQueryBytes. Afdwingen antwoordt met 414.
admin-settings-key-security-protocol-method-mode = Protocolbewaking: HTTP-methoden
admin-settings-help-security-protocol-method-mode = Methode buiten allowedMethods (standaard TRACE, TRACK, CONNECT en onbekende of kleine-letter-methoden). Afdwingen antwoordt met 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Protocolbewaking: herhaalde headers met één waarde
admin-settings-help-security-protocol-duplicate-headers-mode = Een header die maar één keer hoort voor te komen maar vaker voorkomt (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, voorwaardelijke en bereikheaders), een herhaalde Content-Length met gelijke waarden, of herhaalde Cookie-headers bij HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Protocolbewaking: doorstuurheaders van niet-vertrouwde peers
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* of X-Real-IP verstuurd door een peer buiten MW_TRUSTED_PROXIES. De headers worden hoe dan ook nooit vertrouwd; dit legt de poging vast. Het telt alleen mee voor adaptieve bescherming als vertrouwde proxy's zijn ingesteld.
admin-settings-key-security-protocol-header-hygiene-mode = Protocolbewaking: headerhygiëne
admin-settings-help-security-protocol-header-hygiene-mode = Aantal headers, één enkele waarde of de Cookie-header boven de ingestelde grenzen; niet-ASCII-headerbytes; Upgrade of Connection: upgrade buiten de WebSocket-route; Connection die headers noemt zoals Content-Length of Host. Afdwingen antwoordt met 431 voor bevindingen over grootte en aantal en met 400 voor de rest.
admin-settings-key-security-protocol-body-mode = Protocolbewaking: framing van de verzoekbody
admin-settings-help-security-protocol-body-mode = GET of HEAD met een body, Content-Length boven maxBodyBytes (413), Expect anders dan 100-continue (417) of zonder body.
admin-settings-key-security-protocol-max-path-bytes = Protocolbewaking: langste pad (bytes)
admin-settings-help-security-protocol-max-path-bytes = Langste verzoekpad voordat de klasse voor padlengte het meldt.
admin-settings-key-security-protocol-max-query-bytes = Protocolbewaking: langste query (bytes)
admin-settings-help-security-protocol-max-query-bytes = Langste querystring voordat de klasse voor padlengte die meldt.
admin-settings-key-security-protocol-max-header-count = Protocolbewaking: meeste headers
admin-settings-help-security-protocol-max-header-count = Meeste headervelden in één verzoek voordat de klasse voor headerhygiëne dit meldt. De eigen harde limiet van de listener (MW_HTTP_MAX_HEADERS) blijft gelden.
admin-settings-key-security-protocol-max-header-value-bytes = Protocolbewaking: langste headerwaarde (bytes)
admin-settings-help-security-protocol-max-header-value-bytes = Langste enkele headerwaarde voordat de klasse voor headerhygiëne die meldt.
admin-settings-key-security-protocol-max-cookie-bytes = Protocolbewaking: grootste Cookie-header (bytes)
admin-settings-help-security-protocol-max-cookie-bytes = Gecombineerde grootte van Cookie-headers voordat de klasse voor headerhygiëne die meldt.
admin-settings-key-security-protocol-max-body-bytes = Protocolbewaking: grootste aangegeven body (bytes)
admin-settings-help-security-protocol-max-body-bytes = Grootste Content-Length die de bodyklasse accepteert zonder te melden. Houd deze boven de grootste upload of import die een route accepteert; een uitzondering per routevoorvoegsel kan dit voor één route verhogen.
admin-settings-key-security-protocol-allowed-methods = Protocolbewaking: toegestane methoden
admin-settings-help-security-protocol-allowed-methods = Methoden die de methodeklasse accepteert. De standaardlijst bevat de gewone HTTP-werkwoorden en de WebDAV-, CalDAV- en CardDAV-werkwoorden; de server koppelt nu geen inkomende DAV-routes, dus die leiden bij de router tot een 405.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protocol-allowed-hosts = Protocolbewaking: extra geaccepteerde hosts
admin-settings-help-security-protocol-allowed-hosts = Hostnamen die de hostvastzetklasse accepteert naast de host van MW_PUBLIC_URL, bijvoorbeeld een interne naam die een proxy of gezondheidscontrole gebruikt. Poorten en een afsluitende punt worden genegeerd.
admin-settings-key-security-protocol-exempt-prefixes = Protocolbewaking: uitgezonderde routevoorvoegsels
admin-settings-help-security-protocol-exempt-prefixes = Routevoorvoegsels (elk beginnend met / en langer dan één teken) waarvoor de heuristische klassen niet draaien, voor een legitieme client die dat nodig heeft, bijvoorbeeld een route voor grote uploads. Komt overeen met hele padsegmenten. Bevindingen over framing en padstuurtekens worden nooit uitgezonderd, en / wordt genegeerd.
admin-settings-key-security-health-mode = Toegang tot het gezondheidsendpoint
admin-settings-help-security-health-mode = Wie /healthz en /readyz mag bereiken. internal-only: loopback-, privé-, link-local- en unique-local-clientadressen. allowlist: die plus de onderstaande netwerken. open: iedereen. Wie anders komt, krijgt dezelfde 404 als bij een onbekend pad. Een verzoek met het token MW_HEALTH_TOKEN wordt altijd toegelaten. Het clientadres is het door de vertrouwde proxy bepaalde adres: achter een reverse proxy zonder MW_FORWARDED_MODE en MW_TRUSTED_PROXIES lijkt elk verzoek afkomstig van het privéadres van de proxy, waardoor het hele internet wordt toegelaten.
admin-settings-key-security-health-networks = Netwerken voor het gezondheidsendpoint
admin-settings-help-security-health-networks = IP-adressen of CIDR-netwerken die in de allowlist-modus /healthz en /readyz mogen bereiken, naast de interne bereiken, bijvoorbeeld een externe uptimemonitor.
admin-settings-key-security-spray-enabled = Detectie van password spraying
admin-settings-help-security-spray-enabled = Telt de verschillende gebruikersnamen waarmee één bron mislukt. Van elke gebruikersnaam wordt alleen een gesleutelde hash bewaard, in het geheugen.
admin-settings-key-security-spray-window-secs = Spray-venster (seconden)
admin-settings-help-security-spray-window-secs = Venster waarin mislukte aanmeldingen van één bron per verschillende gebruikersnaam worden geteld. Standaard 900.
admin-settings-key-security-spray-challenge-at = Spray: uitdagingsdrempel
admin-settings-help-security-spray-challenge-at = Aantal verschillende mislukkende gebruikersnamen van één bron binnen het venster dat de proof-of-work-uitdaging voor die bron afdwingt. Geslaagde aanmeldingen tellen nooit mee, dus een gedeeld kantoor- of campusadres wordt niet getroffen. Standaard 10.
admin-settings-key-security-spray-pace-at = Spray: tempodrempel
admin-settings-help-security-spray-pace-at = Aantal verschillende mislukkende gebruikersnamen dat extra wachttijd tussen pogingen van de bron toevoegt. Standaard 15.
admin-settings-key-security-spray-lock-at = Spray: vergrendelingsdrempel
admin-settings-help-security-spray-lock-at = Aantal verschillende mislukkende gebruikersnamen dat de bron vergrendelt voor de vergrendelingsduur, via de bestaande aanmeldvergrendeling. Standaard 25.
admin-settings-key-security-spray-pace-secs = Spray: wachttijd tempo (seconden)
admin-settings-help-security-spray-pace-secs = Seconden tussen pogingen van een bron op of boven de tempodrempel. Standaard 10.
admin-settings-key-security-spray-lock-secs = Spray: vergrendelingsduur (seconden)
admin-settings-help-security-spray-lock-secs = Duur van de tijdelijke vergrendeling. Een beheerder die de bron ontgrendelt, zet ook de telling terug. Standaard 900.
admin-settings-key-security-spray-known-multiplier = Spray: vermenigvuldiger bekende bron
admin-settings-help-security-spray-known-multiplier = Vermenigvuldiger op de drie drempels voor een bekende bron, met een recente geslaagde aanmelding. Standaard 3.
admin-settings-key-security-spray-capacity = Spray: gevolgde bronnen
admin-settings-help-security-spray-capacity = Maximaal aantal bronnen dat in het geheugen wordt gevolgd; de minst recent actieve vervalt eerst. Standaard 20000.
admin-settings-key-security-spike-enabled = Piekdetectie voor de hele instantie
admin-settings-help-security-spike-enabled = Leert een basislijn van mislukte aanmeldingen en gaat naar verhoogde modus wanneer de hele instantie er ver boven zit. Vergrendelt de instantie nooit.
admin-settings-key-security-spike-multiplier-percent = Piek: vermenigvuldiger (%)
admin-settings-help-security-spike-multiplier-percent = Een signaal telt mee wanneer het minstens dit percentage van de geleerde basislijn bedraagt (400 = vier keer) en boven de absolute ondergrens ligt.
admin-settings-key-security-spike-min-failures-per-min = Piek: minimum aantal mislukkingen per minuut
admin-settings-help-security-spike-min-failures-per-min = Ondergrens: mislukte aanmeldingen per minuut waaronder het signaal voor het foutpercentage nooit meetelt. Standaard 20.
admin-settings-key-security-spike-min-sources = Piek: minimum aantal mislukkende bronnen
admin-settings-help-security-spike-min-sources = Ondergrens: verschillende mislukkende bronnen in een vijfminutenvak waaronder dat signaal nooit meetelt. Standaard 10.
admin-settings-key-security-spike-min-usernames = Piek: minimum aantal mislukkende gebruikersnamen
admin-settings-help-security-spike-min-usernames = Ondergrens: verschillende mislukkende gebruikersnamen in een vijfminutenvak waaronder dat signaal nooit meetelt. Standaard 30.
admin-settings-key-security-spike-signals-required = Piek: vereiste signalen
admin-settings-help-security-spike-signals-required = Signalen die tegelijk boven hun basislijn en ondergrens moeten zitten. De handtekening van gedistribueerde spraying telt als twee. Standaard 2.
admin-settings-key-security-spike-warmup-hours = Piek: opwarmtijd (uren)
admin-settings-help-security-spike-warmup-hours = Uren waargenomen verkeer voordat de verhoogde modus kan starten. Tot dan leert de basislijn alleen. Standaard 48.
admin-settings-key-security-spike-adapt-hours = Piek: aanpassing van de basislijn (uren)
admin-settings-help-security-spike-adapt-hours = Tijdconstante van de basislijn. Langer neemt groei trager op. Een piek werkt de basislijn nooit bij. Standaard 168.
admin-settings-key-security-spike-duration-secs = Piek: duur van de verhoogde modus (seconden)
admin-settings-help-security-spike-duration-secs = Hoe lang de verhoogde modus duurt na het laatste bewijs van een piek; hij wordt verlengd zolang signalen boven de helft van hun triggerniveau blijven. Standaard 1800.
admin-settings-key-security-spike-force-challenge = Piek: uitdaging afdwingen
admin-settings-help-security-spike-force-challenge = Vereist in de verhoogde modus de proof-of-work-uitdaging voor anonieme aanmeldingen vanaf onbekende bronnen. Geverifieerde gebruikers en bekende bronnen worden niet beïnvloed.
admin-settings-key-security-spike-pace-factor-percent = Piek: wachttijd aanmelding (%)
admin-settings-help-security-spike-pace-factor-percent = In de verhoogde modus de wachttijd bij aanmelden voor onbekende bronnen, als percentage van de gebruikelijke wachttijd. Standaard 300.
admin-settings-key-security-session-binding-enabled = Sessiebinding
admin-settings-help-security-session-binding-enabled = Bindt elke browsersessie aan gesleutelde hashes van de clientkenmerken. Native clients met een bearer-token, API-sleutels en protocolclients zijn uitgezonderd.
admin-settings-key-security-session-binding-attributes = Sessiebinding: kenmerken
admin-settings-help-security-session-binding-attributes = Gebonden kenmerken: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Network staat standaard uit omdat mobiele netwerken wisselen. Onbekende namen worden genegeerd.
admin-settings-key-security-session-binding-strictness = Sessiebinding: strengheid
admin-settings-help-security-session-binding-strictness = Tolerant accepteert browserupdates en de wijziging van één zacht kenmerk; strikt markeert elk zacht verschil.
admin-settings-key-security-session-binding-webmail-action = Sessiebinding: actie in webmail
admin-settings-help-security-session-binding-webmail-action = Wat een afwijking bij een webmailsessie doet: off, detect (alleen vastleggen), step-up (opnieuw om wachtwoord en tweede factor vragen) of de sessie herroepen. Standaard step-up.
admin-settings-key-security-session-binding-admin-action = Sessiebinding: actie in de beheerconsole
admin-settings-help-security-session-binding-admin-action = Wat een afwijking bij een sessie van de beheerconsole doet: off, detect, step-up of herroepen. Standaard herroepen.
admin-settings-key-security-theft-enabled = Detectie van sessiediefstal
admin-settings-help-security-theft-enabled = Detecteert hergebruik van een geroteerd sessie-id, onmogelijke reizen en gelijktijdig gebruik vanuit verschillende landen.
admin-settings-key-security-theft-rotation-grace-secs = Diefstal: coulanceperiode rotatie (seconden)
admin-settings-help-security-theft-rotation-grace-secs = Nadat een sessie is geroteerd, mag het oude id nog zo lang worden aangeboden (een lopend verzoek) voordat het als diefstal telt. Standaard 30.
admin-settings-key-security-theft-min-travel-secs = Diefstal: minimale reistijd (seconden)
admin-settings-help-security-theft-min-travel-secs = Een landwijziging die sneller volgt dan dit na het vorige verzoek wordt als onmogelijke reis behandeld. Landdetectie kan onnauwkeurig zijn; vpn's en mobiele netwerken kunnen valse wijzigingen veroorzaken. Standaard 3600.
admin-settings-key-security-theft-concurrent-use = Diefstal: gelijktijdig gebruik
admin-settings-help-security-theft-concurrent-use = Markeert één sessie die binnen vijf minuten afwisselend vanuit verschillende landen (of netwerken) wordt gebruikt: off, country of network. Landdetectie kan onnauwkeurig zijn; vpn's en mobiele netwerken kunnen valse wijzigingen veroorzaken.
admin-settings-key-security-theft-webmail-action = Diefstal: reactie in webmail
admin-settings-help-security-theft-webmail-action = Reactie op onmogelijke reizen of gelijktijdig gebruik bij een webmailsessie: alert, step-up of herroepen. Hergebruik van een geroteerde sessie herroept altijd. Standaard step-up.
admin-settings-key-security-theft-admin-action = Diefstal: reactie in de beheerconsole
admin-settings-help-security-theft-admin-action = Reactie op onmogelijke reizen of gelijktijdig gebruik bij een sessie van de beheerconsole: alert, step-up of herroepen. Standaard herroepen.
admin-settings-key-security-theft-retention-days = Diefstal: bewaartermijn van gegevens (dagen)
admin-settings-help-security-theft-retention-days = Hoe lang markeringen van geroteerde sessies en beveiligingsgegevens per sessie worden bewaard; stel dit minstens even lang in als de langste sessieduur. Standaard 15.
admin-settings-key-security-alerts-country-change = Melding bij landwijziging
admin-settings-help-security-alerts-country-change = Waarschuwt wanneer een ingelogde sessie wordt gebruikt vanuit een ander land dan waar ze is gestart. Landdetectie kan onnauwkeurig zijn; vpn's en mobiele netwerken kunnen valse wijzigingen veroorzaken. Gebruikers kunnen dit wijzigen tenzij het is vergrendeld; als een gebruiker het uitschakelt, wordt dat alleen gerespecteerd als dempen is toegestaan.
admin-settings-key-security-alerts-country-change-enabled = Melding bij landwijziging: globale schakelaar
admin-settings-help-security-alerts-country-change-enabled = Uit schakelt de melding bij landwijziging voor iedereen uit. Landdetectie kan onnauwkeurig zijn; vpn's en mobiele netwerken kunnen valse wijzigingen veroorzaken.
admin-settings-key-security-alerts-country-change-cooldown-secs = Melding bij landwijziging: wachttijd (seconden)
admin-settings-help-security-alerts-country-change-cooldown-secs = Minimale tijd tussen meldingen bij landwijziging van één sessie. Elk nieuw land geeft één melding per sessie. Standaard 300.
admin-settings-key-security-alerts-country-change-user-mute = Melding bij landwijziging: gebruikers mogen dempen
admin-settings-help-security-alerts-country-change-user-mute = Of een gebruiker zijn eigen melding bij landwijziging mag uitschakelen. Als dit uit staat, blijft de server melden, ook als de eigen instelling van de gebruiker uit staat.
admin-settings-key-security-alerts-channels = Kanalen voor beveiligingsmeldingen
admin-settings-help-security-alerts-channels = Hoe beveiligingswaarschuwingen over sessies de gebruiker bereiken: in de app, als bericht in de eigen mailbox, of beide. Gebruikers kunnen dit wijzigen tenzij het is vergrendeld.
admin-settings-key-security-alerts-email-enabled = Beveiligingsmeldingen per e-mail
admin-settings-help-security-alerts-email-enabled = Staat toe dat beveiligingswaarschuwingen over sessies in de eigen mailbox van de gebruiker worden bezorgd. Er wordt geen bericht via SMTP verstuurd; zonder bezorgsessie blijft de waarschuwing in de app en het auditlogboek.
admin-settings-key-security-alerts-sensitivity = Gevoeligheid van beveiligingsmeldingen
admin-settings-help-security-alerts-sensitivity = Laag waarschuwt alleen bij locatiebevindingen en verhoogt of herroept nooit; normaal volgt de acties van de beheerder; hoog verdubbelt het reisinterval en controleert ook gelijktijdige netwerken. Gebruikers kunnen dit wijzigen tenzij het is vergrendeld.
admin-settings-key-security-alerts-notice-retention-days = Beveiligingsberichten: bewaartermijn (dagen)
admin-settings-help-security-alerts-notice-retention-days = Hoe lang beveiligingsberichten in de app per account worden bewaard (maximaal 20 per account). Standaard 30.
admin-settings-inline-effective = Van kracht:
admin-settings-inline-reset = Wijzigingen negeren
admin-settings-inline-domains = Waarden per domein
admin-attachments-tab-lists = Toestaan- en blokkeerlijsten
admin-attachments-tab-limits = Limieten
admin-attachments-tab-delivery = Downloaden en voorbeeld
admin-attachments-tab-warnings = Waarschuwingen
admin-attachments-tab-test = Een bestand testen
admin-attachments-tab-denied = U hebt geen toegang tot dit tabblad.
admin-attachments-group-lists = Toestaan- en blokkeerlijsten
admin-attachments-group-limits = Groottelimiet
admin-attachments-group-delivery = Downloaden en voorbeeld
admin-attachments-group-warnings = Waarschuwingen
admin-attachments-add-extension = Extensie, bijvoorbeeld pdf of .tar.gz
admin-attachments-add-mime = MIME-type, bijvoorbeeld image/png of image/*
admin-attachments-add = Toevoegen
admin-attachments-remove = { $rule } verwijderen
admin-attachments-filter = De lijst filteren
admin-attachments-list-label = Regels in deze lijst
admin-attachments-count = { $count } van { $max } regels
admin-attachments-empty-list = Geen regels in deze lijst.
admin-attachments-none-match = Geen enkele regel komt overeen met het filter.
admin-attachments-bulk = Meerdere plakken
admin-attachments-bulk-add = Alles toevoegen
admin-attachments-import = Een tekstbestand importeren
admin-attachments-export = Exporteren als tekst
admin-attachments-err-extension = Geen extensie. Gebruik letters, cijfers, koppeltekens en underscores, maximaal 32 tekens; punten mogen delen scheiden, zoals in tar.gz.
admin-attachments-err-mime = Geen MIME-type. Gebruik type/subtype, of type/* voor alle subtypen.
admin-attachments-err-long = Die regel is te lang.
admin-attachments-invalid = Niet toegevoegd, geen geldige regel: { $entries }
admin-attachments-duplicates = Staat al in de lijst: { $rules }
admin-attachments-overflow = Een lijst bevat maximaal { $max } regels. De rest is niet toegevoegd.
admin-attachments-conflict-blockedOverAllowed = { $rule } overlapt met { $other } in de andere lijst. Blokkeerregels gaan voor, dus het wordt geblokkeerd.
admin-attachments-conflict-blockedOverWarn = { $rule } overlapt met { $other } in de andere lijst. Een geblokkeerd bestand wordt vóór elke waarschuwing geweigerd, dus de waarschuwing verschijnt nooit.
admin-attachments-conflict-allowedAndWarn = { $rule } overlapt met { $other } in de andere lijst. Het bestand is toegestaan en toont een waarschuwing.
admin-attachments-bytes-unit = Eenheid
admin-attachments-unlimited = Geen groottelimiet
admin-attachments-bytes-equals = { $size } ({ $bytes } bytes)
admin-attachments-list-allowedExtensions = toegestane extensies
admin-attachments-list-allowedMimeTypes = toegestane MIME-typen
admin-attachments-list-blockedExtensions = geblokkeerde extensies
admin-attachments-list-blockedMimeTypes = geblokkeerde MIME-typen

# -- Machine translation (2026-11): added to match en ---------------------
admin-attachments-list-warnExtensions = extensies met waarschuwing
admin-attachments-list-warnMimeTypes = MIME-typen met waarschuwing
admin-attachments-sim-title = Een bestand testen
admin-attachments-sim-help = Vraagt de server hoe het opgeslagen beleid een bestand met deze naam, dit type en deze grootte behandelt. Er wordt niets gewijzigd of opgeslagen. Berichten en bijlagen worden nooit gelezen.
admin-attachments-sim-filename = Bestandsnaam
admin-attachments-sim-mime = MIME-type
admin-attachments-sim-size = Grootte
admin-attachments-sim-run = Testen
admin-attachments-sim-invalid = Voer een bestandsnaam en een grootte van nul of meer in.
admin-attachments-sim-allowed = Toegestaan
admin-attachments-sim-warn = Toegestaan met waarschuwing
admin-attachments-sim-blocked = Niet bezorgd
admin-attachments-sim-preview-yes = Voorbeeld is toegestaan.
admin-attachments-sim-preview-no = Voorbeeld is niet toegestaan.
admin-attachments-sim-checked = Grootte gecontroleerd: { $size }.
admin-attachments-sim-matched = Overeenkomende regels in: { $lists }.
admin-attachments-sim-matched-none = Er kwam geen toestaan-, blokkeer- of waarschuwingsregel overeen.
admin-attachments-reason-downloadsDisabled = Downloads staan uit.
admin-attachments-reason-attachmentTooLarge = Het bestand is groter dan de groottelimiet.
admin-attachments-reason-attachmentBlocked = Een blokkeerregel komt overeen.
admin-attachments-reason-attachmentNotAllowed = Er bestaat een toestaanlijst en niets daarin komt overeen.
admin-attachments-reason-attachmentContainerRestricted = Bijgevoegde berichten en TNEF-bestanden worden geweigerd zolang een beperking actief is, omdat hun inhoud niet is opgesomd.
admin-settings-key-mail-reply-images = Afbeeldingen in antwoorden en doorgestuurde berichten
admin-settings-help-mail-reply-images = Of een antwoord of doorgestuurd bericht de afbeeldingen van het oorspronkelijke bericht bevat: vragen (standaard), altijd toestaan, alleen toestaan voor vertrouwde afzenderdomeinen en anders vragen, of altijd weigeren. Vergrendel deze instelling om haar af te dwingen. Afbeeldingen worden alleen gelezen als ze zijn opgenomen.
admin-settings-key-mail-reply-image-trusted-domains = Vertrouwde afzenderdomeinen voor afbeeldingen
admin-settings-help-mail-reply-image-trusted-domains = Maximaal 200 exacte afzenderdomeinen waarvan de afbeeldingen zonder vragen worden opgenomen wanneer het beleid vertrouwde domeinen is. Gebruikers voegen vanuit de citaatprompt toe aan hun eigen lijst, tenzij deze instelling is vergrendeld.
admin-settings-key-mail-send-recall-delay-seconds = Venster voor verzenden ongedaan maken (standaard)
admin-settings-help-mail-send-recall-delay-seconds = Hoeveel seconden een verzonden bericht in de Postvak UIT wacht, waar de afzender het kan terughalen, voordat het wordt bezorgd. 0 verzendt direct. Standaard 10, tot 1800 (30 minuten). Gebruikers kiezen hun eigen waarde tot de onderstaande limiet, tenzij u dit vergrendelt.
admin-settings-key-mail-send-recall-delay-max-seconds = Venster voor verzenden ongedaan maken (limiet)
admin-settings-help-mail-send-recall-delay-max-seconds = Het langste venster voor verzenden ongedaan maken dat een gebruiker mag kiezen, in seconden. De server kort een langer verzoek in tot deze waarde. Standaard en maximum 1800.
admin-settings-key-mail-send-allow-recall = Terughalen van verzonden berichten toestaan
admin-settings-help-mail-send-allow-recall = Als dit uit staat, vertrekken berichten direct en kunnen ze niet uit de Postvak UIT worden teruggehaald. Ingeplande verzendingen worden niet beïnvloed; die kunnen tot hun tijdstip nog worden geannuleerd.
admin-settings-key-mail-send-max-delayed-seconds = Planningshorizon
admin-settings-help-mail-send-max-delayed-seconds = Hoever vooruit een bericht mag worden ingepland, in seconden. Standaard 7776000 (90 dagen), maximum 31536000 (1 jaar). 0 schakelt ingepland verzenden uit.
admin-settings-key-mail-send-max-queued = Berichten in de wachtrij per mailbox
admin-settings-help-mail-send-max-queued = Hoeveel berichten één mailbox tegelijk in de Postvak UIT mag hebben wachten (ongedaanmakingsvenster of ingepland). Standaard 200. Verdere verzendingen worden geweigerd met een duidelijke melding en nooit weggegooid.
admin-attachments-sim-pick = Invullen vanuit een bestand (optioneel)
admin-attachments-mime-toggle = Veelgebruikte MIME-typen tonen
admin-attachments-mime-hint = Kies een veelgebruikt type of typ een ander type/subtype.
admin-attachments-mime-invalid = Dit is geen type/subtype zoals text/plain. Het wordt verzonden zoals getypt.
admin-attachments-mime-group-documents = Documenten
admin-attachments-mime-group-images = Afbeeldingen
admin-attachments-mime-group-archives = Archieven
admin-attachments-mime-group-executables = Uitvoerbare bestanden en scripts
admin-attachments-mime-group-media = Audio en video
admin-attachments-mime-group-email = E-mail en agenda
admin-attachments-mime-group-other = Overig
admin-settings-key-cookies-enabled = Cookietoestemmingsbeheer
admin-settings-help-cookies-enabled = Toon het cookietoestemmingsbeheer aan bezoekers. Uit laat de toepassing zich gedragen zoals zonder dit beheer.
admin-settings-key-cookies-mode = Toestemmingsmodel
admin-settings-help-cookies-mode = optIn: niets optioneels wordt opgeslagen totdat de bezoeker accepteert. optOut: de standaardwaarden van de categorieën gelden totdat de bezoeker weigert. noticeOnly: de bezoeker wordt geïnformeerd, zonder keuze.
admin-settings-key-cookies-region-mode = Toestemmingsmodel in strenge regio's
admin-settings-help-cookies-region-mode = Het model dat voor bezoekers uit de strenge regio's wordt gebruikt. same gebruikt daar ook het bovenstaande toestemmingsmodel.
admin-settings-key-cookies-strict-regions = Strenge regio's
admin-settings-help-cookies-strict-regions = ISO 3166-1-landcodes die het toestemmingsmodel voor strenge regio's gebruiken. Standaard de EU- en EER-staten, het Verenigd Koninkrijk en Zwitserland.
admin-settings-key-cookies-region-header = Landheader
admin-settings-help-cookies-region-header = Naam van de verzoekheader, ingesteld door een proxy of cdn, die de landcode van de bezoeker bevat.
admin-settings-key-cookies-unknown-region = Onbekend land
admin-settings-help-cookies-unknown-region = Welk model geldt als geen land kan worden bepaald: het model voor strenge regio's of het normale.
admin-settings-key-cookies-functional-enabled = Functionele cookies aanbieden
admin-settings-help-cookies-functional-enabled = Bied de categorie voorkeuren en functioneel (thema, taal, onthouden indeling) aan in het toestemmingsbeheer.
admin-settings-key-cookies-functional-default = Standaard voor functioneel
admin-settings-help-cookies-functional-default = Beginstatus van de functionele categorie wanneer het toestemmingsmodel standaardwaarden toestaat.
admin-settings-key-cookies-analytics-enabled = Analysecookies aanbieden
admin-settings-help-cookies-analytics-enabled = Bied de analysecategorie aan in het toestemmingsbeheer.
admin-settings-key-cookies-analytics-default = Standaard voor analyse
admin-settings-help-cookies-analytics-default = Beginstatus van de analysecategorie wanneer het toestemmingsmodel standaardwaarden toestaat.
admin-settings-key-cookies-marketing-enabled = Marketingcookies aanbieden
admin-settings-help-cookies-marketing-enabled = Bied de marketingcategorie aan in het toestemmingsbeheer.
admin-settings-key-cookies-marketing-default = Standaard voor marketing
admin-settings-help-cookies-marketing-default = Beginstatus van de marketingcategorie wanneer het toestemmingsmodel standaardwaarden toestaat.
admin-settings-key-cookies-layout = Indeling
admin-settings-help-cookies-layout = banner is een balk over de pagina, card is een zwevende kaart, modal is een dialoogvenster dat de pagina vasthoudt totdat de bezoeker kiest.
admin-settings-key-cookies-position = Positie
admin-settings-help-cookies-position = Waar het toestemmingsbeheer op de pagina verschijnt.
admin-settings-key-cookies-reject-prominent = Weigeren even prominent als accepteren
admin-settings-help-cookies-reject-prominent = Toon Alles weigeren met dezelfde grootte en stijl als Alles accepteren.
admin-settings-key-cookies-policy-url = URL van het cookiebeleid
admin-settings-help-cookies-policy-url = Link naar uw cookiebeleid, getoond in het toestemmingsbeheer.
admin-settings-key-cookies-privacy-url = URL van het privacybeleid
admin-settings-help-cookies-privacy-url = Link naar uw privacybeleid, getoond in het toestemmingsbeheer.
admin-settings-key-cookies-policy-version = Beleidsversie
admin-settings-help-cookies-policy-version = Verhoog dit getal wanneer uw cookiegebruik verandert, zodat bezoekers opnieuw worden gevraagd.
admin-settings-key-cookies-lifetime-days = Geldigheid van toestemming
admin-settings-help-cookies-lifetime-days = Aantal dagen dat een keuze geldig blijft voordat de bezoeker opnieuw wordt gevraagd.
admin-settings-key-cookies-reprompt-on-version = Opnieuw vragen bij een nieuwe beleidsversie
admin-settings-help-cookies-reprompt-on-version = Vraag bezoekers opnieuw wanneer de beleidsversie hoger is dan die waarop ze hun keuze maakten.
admin-settings-key-cookies-honour-gpc = Global Privacy Control respecteren
admin-settings-help-cookies-honour-gpc = Behandel een Global Privacy Control-signaal van de browser als weigering van analyse en marketing.
admin-settings-key-cookies-honour-dnt = Do Not Track respecteren
admin-settings-help-cookies-honour-dnt = Behandel een Do Not Track-signaal van de browser als weigering van analyse en marketing.
admin-settings-key-cookies-log-consent = Toestemmingslogboek bijhouden
admin-settings-help-cookies-log-consent = Houd aan serverzijde een vastlegging van elke toestemmingskeuze bij, voor statistieken en uw eigen administratie.
admin-settings-key-cookies-log-retention-days = Bewaartermijn toestemmingslogboek
admin-settings-help-cookies-log-retention-days = Aantal dagen dat rijen van het toestemmingslogboek worden bewaard voordat ze worden verwijderd.
admin-settings-key-cookies-scan-interval-hours = Scaninterval voor opslag
admin-settings-help-cookies-scan-interval-hours = Uren tussen automatische scans van de cookies en opslag van de browser. 0 scant alleen op verzoek.
admin-settings-key-cookies-unclassified = Niet-geclassificeerde opslag
admin-settings-help-cookies-unclassified = report laat opslag die niets classificeert met rust en toont die ter controle. De andere waarden behandelen die als die categorie totdat u hem classificeert.
admin-settings-key-cookies-text-title = Titel van het toestemmingsbeheer
admin-settings-help-cookies-text-title = Titel van het toestemmingsbeheer per taal. Leeg gebruikt de ingebouwde tekst.
admin-settings-key-cookies-text-body = Tekst van het toestemmingsbeheer
admin-settings-help-cookies-text-body = Toelichtende tekst van het toestemmingsbeheer per taal. Leeg gebruikt de ingebouwde tekst.
admin-settings-key-cookies-label-necessary = Naam van de categorie noodzakelijk
admin-settings-help-cookies-label-necessary = Naam van de categorie noodzakelijk per taal. Leeg gebruikt de ingebouwde naam.
admin-settings-key-cookies-description-necessary = Beschrijving van de categorie noodzakelijk
admin-settings-help-cookies-description-necessary = Beschrijving van de categorie noodzakelijk per taal. Leeg gebruikt de ingebouwde beschrijving.
admin-settings-key-cookies-label-functional = Naam van de categorie functioneel
admin-settings-help-cookies-label-functional = Naam van de categorie functioneel per taal. Leeg gebruikt de ingebouwde naam.
admin-settings-key-cookies-description-functional = Beschrijving van de categorie functioneel
admin-settings-help-cookies-description-functional = Beschrijving van de categorie functioneel per taal. Leeg gebruikt de ingebouwde beschrijving.
admin-settings-key-cookies-label-analytics = Naam van de categorie analyse
admin-settings-help-cookies-label-analytics = Naam van de categorie analyse per taal. Leeg gebruikt de ingebouwde naam.
admin-settings-key-cookies-description-analytics = Beschrijving van de categorie analyse
admin-settings-help-cookies-description-analytics = Beschrijving van de categorie analyse per taal. Leeg gebruikt de ingebouwde beschrijving.
admin-settings-key-cookies-label-marketing = Naam van de categorie marketing
admin-settings-help-cookies-label-marketing = Naam van de categorie marketing per taal. Leeg gebruikt de ingebouwde naam.
admin-settings-key-cookies-description-marketing = Beschrijving van de categorie marketing
admin-settings-help-cookies-description-marketing = Beschrijving van de categorie marketing per taal. Leeg gebruikt de ingebouwde beschrijving.
admin-settings-key-encryption-rotation-days = Rotatie-interval databasesleutel (dagen)
admin-settings-help-encryption-rotation-days = Aantal dagen na de laatste sleutelwijziging waarna de databasesleutel als toe aan rotatie wordt gemeld. 0 stelt geen vervaldatum in. De sleutel zelf wordt nooit in de instellingen opgeslagen.
admin-settings-key-encryption-warnings-enabled = Versleutelingsbevindingen melden
admin-settings-help-encryption-warnings-enabled = Toon opslag in leesbare tekst en een achterstallige sleutelrotatie als bevinding in de implementatiebeoordeling.
admin-settings-key-retention-sweeper-enabled = Bewaartermijn-opruiming
admin-settings-help-retention-sweeper-enabled = Voer de dagelijkse opruiming uit die rijen verwijdert die ouder zijn dan de limieten op deze pagina. Uitschakelen bewaart alle rijen totdat het weer wordt ingeschakeld.
admin-settings-key-retention-sweeper-batch-rows = Rijen per batch
admin-settings-help-retention-sweeper-batch-rows = Rijen die één verwijderopdracht verwijdert. Kleinere batches houden databaseloks korter vast.
admin-settings-key-retention-sweeper-max-batches = Batches per opslag per uitvoering
admin-settings-help-retention-sweeper-max-batches = Maximaal aantal verwijderopdrachten dat één opslag per uitvoering krijgt. Een opslag met meer oude rijen gaat door in de volgende uitvoering.
admin-settings-key-retention-sweeper-jitter-minutes = Spreiding van de starttijd (minuten)
admin-settings-help-retention-sweeper-jitter-minutes = De volgende uitvoering start een willekeurig aantal minuten, tot deze waarde, na 24 uur, zodat meerdere servers niet tegelijk opruimen.
admin-settings-key-retention-sweeper-vacuum-after-rows = Ruimte terugwinnen na (rijen)
admin-settings-help-retention-sweeper-vacuum-after-rows = Wanneer één uitvoering ten minste dit aantal rijen verwijdert, wordt de database gecomprimeerd (VACUUM). 0 comprimeert nooit na een opruiming; de wekelijkse vacuümtaak staat hier los van.
admin-settings-key-retention-password-change-audit-days = Records van wachtwoordwijzigingen (dagen)
admin-settings-help-retention-password-change-audit-days = Aantal dagen dat een record van een wachtwoordwijziging (account, methode, uitkomst) wordt bewaard. 0 bewaart ze voor altijd.
admin-settings-key-retention-sso-login-audit-days = Records van aanmeldingen via single sign-on (dagen)
admin-settings-help-retention-sso-login-audit-days = Aantal dagen dat een record van een aanmelding via single sign-on (provider, gehasht onderwerp, uitkomst) wordt bewaard. 0 bewaart ze voor altijd.
admin-settings-key-retention-dlp-audit-days = Records van gegevenslekpreventie (dagen)
admin-settings-help-retention-dlp-audit-days = Aantal dagen dat een record van een overeenkomst met een regel voor gegevenslekpreventie wordt bewaard. 0 bewaart ze voor altijd.
admin-settings-key-retention-assist-audit-days = Records van assistenttoegang (dagen)

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-retention-assist-audit-days = Aantal dagen dat een record van het gebruik van een mogelijkheid door een assistent (actor, mogelijkheid, samenvatting van bereik) wordt bewaard. 0 bewaart ze voor altijd.
admin-settings-key-retention-alert-events-days = Records van afgegeven meldingen (dagen)
admin-settings-help-retention-alert-events-days = Aantal dagen dat een record van een afgegeven beheerdersmelding wordt bewaard. 0 bewaart ze voor altijd.
admin-settings-key-retention-job-runs-days = Geschiedenis van achtergrondtaken (dagen)
admin-settings-help-retention-job-runs-days = Aantal dagen dat een voltooide uitvoering van een achtergrondtaak wordt bewaard. Een uitvoering die nog loopt, wordt nooit verwijderd. 0 bewaart de geschiedenis voor altijd.
admin-settings-key-retention-outbox-days = Records van verzonden en geannuleerde verzendingen (dagen)
admin-settings-help-retention-outbox-days = Aantal dagen dat het verzendrecord van een bezorgd of geannuleerd bericht wordt bewaard. Een bericht dat wacht, opnieuw wordt geprobeerd, mislukt of onzeker is, wordt nooit verwijderd. 0 bewaart ze voor altijd.
admin-settings-key-retention-expired-credentials-grace-days = Verlopen inloggegevens (dagen na verloop)
admin-settings-help-retention-expired-credentials-grace-days = Aantal dagen dat een verlopen onthouden apparaat, aanmelduitdaging, aanmeldtoken, OAuth-token, webhook-nonce, foutenteller van een bron of afbeeldingslinkhandle na het verlopen wordt bewaard. 0 verwijdert het bij de volgende uitvoering.
admin-settings-key-retention-ip-addresses-days = Opgeslagen IP-adressen verkleinen na (dagen)
admin-settings-help-retention-ip-addresses-days = Na dit aantal dagen wordt het IP-adres in de authenticatiegeschiedenis en auditvermeldingen afgekapt of gehasht en blijft de rij bewaard. 0 laat adressen zoals vastgelegd.
admin-settings-key-retention-ip-addresses-mode = Methode voor IP-reductie
admin-settings-help-retention-ip-addresses-mode = Afkappen houdt het IPv4 /24- of IPv6 /48-voorvoegsel. Hash vervangt het adres door een korte gesleutelde hash die hetzelfde adres nog steeds groepeert.
admin-settings-key-retention-user-enabled = Gebruikers mogen bewaarregels instellen
admin-settings-help-retention-user-enabled = Gebruikers kunnen hun eigen bewaarregels voor mappen en e-mail instellen. Als dit uit staat, worden bestaande gebruikersregels niet meer uitgevoerd; de organisatiebrede regels hieronder blijven draaien.
admin-settings-key-retention-user-min-days = Minimum aantal dagen voordat een gebruikersregel e-mail verwijdert
admin-settings-help-retention-user-min-days = Een gebruikersregel die e-mail verwijdert, mag niet handelen op e-mail die jonger is dan dit. Een kleinere waarde wordt bij het opslaan van de regel geweigerd en bij het uitvoeren verhoogd.
admin-settings-key-retention-user-max-days = Maximum aantal dagen dat een gebruikersregel mag wachten met het verwijderen van e-mail
admin-settings-help-retention-user-max-days = Een gebruikersregel die e-mail verwijdert, mag niet langer wachten dan dit. 0 stelt geen bovengrens in. Een grotere waarde wordt bij het opslaan van de regel geweigerd en bij het uitvoeren verlaagd.
admin-settings-key-retention-user-max-rules = Maximum aantal bewaarregels per gebruiker
admin-settings-help-retention-user-max-rules = Het maximum aantal bewaarregels dat één gebruiker mag hebben.
admin-settings-key-retention-user-allow-permanent-delete = Gebruikersregels mogen e-mail definitief verwijderen
admin-settings-help-retention-user-allow-permanent-delete = Als dit uit staat, verplaatst een verwijderregel e-mail altijd naar de Prullenbak en geldt het ongedaanmakingsvenster voordat het definitief wordt gewist.
admin-settings-key-retention-user-undo-days = Ongedaanmakingsvenster voor bewaarregels (dagen)
admin-settings-help-retention-user-undo-days = Aantal dagen dat e-mail die door een bewaarregel naar de Prullenbak is verplaatst daar blijft voordat het definitief wordt gewist. Een gebruiker mag een korter venster kiezen. 0 wist bij de volgende uitvoering.
admin-settings-key-retention-user-mandatory = Organisatiebrede e-mailregels
admin-settings-help-retention-user-mandatory = Regels die elk account krijgt, als items maprol:dagen, bijvoorbeeld trash:30 of junk:30. E-mail in die map die ouder is dan het aantal dagen wordt definitief gewist. Een gebruiker kan deze niet wijzigen, pauzeren of e-mail ervan uitsluiten. E-mail van een account onder juridische bewaarplicht wordt overgeslagen.
admin-settings-key-retention-user-protected-folders = Mappen die bewaarregels nooit aanraken
admin-settings-help-retention-user-protected-folders = Mappen waarop geen enkele gebruikersregel werkt, op rol (drafts, sent, inbox, archive) of op mapnaam. Een beschermde map gaat voor een organisatiebrede regel voor dezelfde map.
admin-settings-key-retention-user-templates = Aan gebruikers aangeboden bewaarsjablonen
admin-settings-help-retention-user-templates = Sjablonen die gebruikers in Instellingen kunnen overnemen, op id: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Gebruikersregels uitvoeren voor beperkte accounts
admin-settings-help-retention-user-run-for-restricted = Of gebruikersbewaarregels ook worden uitgevoerd voor accounts onder een beperking van de verwerking. Uit slaat die accounts over.
admin-settings-key-retention-user-interval-hours = Uren tussen uitvoeringen van gebruikersbewaring
admin-settings-help-retention-user-interval-hours = Uren tussen geplande uitvoeringen van de taak voor gebruikersbewaring voor één account.
admin-settings-key-retention-user-batch-messages = Berichten per bewaaropdracht
admin-settings-help-retention-user-batch-messages = Berichten die één verplaats- of verwijderopdracht verwerkt. Er worden er hooguit 200 gebruikt.
admin-settings-key-retention-user-max-per-run = Maximum aantal gewijzigde berichten per account per uitvoering
admin-settings-help-retention-user-max-per-run = Maximum aantal berichten dat de taak voor gebruikersbewaring in één uitvoering voor één account wijzigt. De rest wacht op de volgende uitvoering.
admin-settings-key-retention-user-pause-ms = Pauze tussen bewaarbatches (ms)
admin-settings-help-retention-user-pause-ms = Pauze tussen batches, in milliseconden, zodat een grote opruiming de mailserver niet belast.
admin-settings-key-retention-class-short-days = Bovengrens korte klasse (dagen)
admin-settings-help-retention-class-short-days = Bovengrens voor operationele gegevens: sessies, tokens, snelheidslimieten, taakgeschiedenis en statistieken. Geldt wanneer de klassenlimiet aan staat.
admin-settings-key-retention-class-medium-days = Bovengrens middellange klasse (dagen)
admin-settings-help-retention-class-medium-days = Bovengrens voor aanmeldgeschiedenis, geschiedenis van verzoekbescherming en beveiligingsmeldingen. Mag niet onder de korte klasse liggen; een lagere waarde wordt daartoe verhoogd.
admin-settings-key-retention-class-long-days = Bewaartermijn lange klasse (dagen)
admin-settings-help-retention-class-long-days = Aantal dagen dat de lange klasse (beveiligingsaudit, beheerdersacties, verzoeken, toestemming, tombstones) wordt bewaard, archief inbegrepen. 0 bewaart voor altijd en de console waarschuwt. Mag niet onder de middellange klasse liggen.
admin-settings-key-retention-class-enforce = Elke opslag aan de bovengrens van zijn klasse houden
admin-settings-help-retention-class-enforce = Als dit aan staat, wordt de eigen limiet van elke opslag beperkt tot de bovengrens van zijn klasse en telt 0 (bewaren) als de bovengrens. Als dit uit staat, gelden de limieten per opslag.
admin-settings-key-retention-archive-enabled = Verouderde rijen van de lange klasse archiveren
admin-settings-help-retention-archive-enabled = Verplaats verouderde rijen van de lange klasse naar verzegelde, gecomprimeerde archiefbestanden in plaats van ze te verwijderen.
admin-settings-key-retention-archive-directory = Archiefmap
admin-settings-help-retention-archive-directory = Map voor archiefbestanden, absoluut of relatief aan de werkmap van de server. Segmenten voor de bovenliggende map (..) worden geweigerd.
admin-settings-key-retention-archive-batch-rows = Rijen per archiefbestand
admin-settings-help-retention-archive-batch-rows = Rijen die in één archiefbestand worden geschreven.
admin-settings-key-retention-archive-max-segments = Archiefbestanden per opslag per uitvoering
admin-settings-help-retention-archive-max-segments = Archiefbestanden die één uitvoering voor elke opslag mag schrijven. De rest wacht op de volgende uitvoering.
admin-settings-key-retention-archive-key-generation = Generatie van de archiefverzegelsleutel
admin-settings-help-retention-archive-key-generation = Generatie van de sleutel die nieuwe archiefbestanden verzegelt. Verhoog deze om de sleutel te roteren; eerdere bestanden worden nog steeds met hun eigen generatie geverifieerd.
admin-settings-key-retention-archive-verify-days = Dagen tussen archiefcontroles
admin-settings-help-retention-archive-verify-days = Aantal dagen tussen integriteitscontroles van de archiefbestanden.
admin-settings-key-retention-archive-legal-hold = Juridische bewaarplicht op het archief
admin-settings-help-retention-archive-legal-hold = Bewaar elk archiefbestand, wat de limiet van de lange klasse ook zegt.
admin-settings-key-logging-file-enabled = Logbestanden schrijven
admin-settings-help-logging-file-enabled = Schrijf het serverlogboek ook naar roterende bestanden. Uit houdt het loggen bij de standaarduitvoer, wat een container normaal gebruikt.
admin-settings-key-logging-file-directory = Logmap
admin-settings-help-logging-file-directory = Map voor de logbestanden, absoluut of relatief aan de werkmap van de server. Segmenten voor de bovenliggende map (..) worden geweigerd.
admin-settings-key-logging-file-max-size-mb = Roteren bij grootte (MB)
admin-settings-help-logging-file-max-size-mb = Het huidige logbestand wordt gesloten en een nieuw gestart wanneer het deze grootte bereikt.
admin-settings-key-logging-file-rotate-hours = Roteren bij leeftijd (uren)
admin-settings-help-logging-file-rotate-hours = Het huidige logbestand wordt geroteerd zodra het zo oud is. 0 roteert alleen op grootte.
admin-settings-key-logging-file-max-age-days = Geroteerde bestanden bewaren (dagen)
admin-settings-help-logging-file-max-age-days = Geroteerde logbestanden die ouder zijn dan dit, worden verwijderd.
admin-settings-key-logging-file-max-files = Maximum aantal geroteerde bestanden
admin-settings-help-logging-file-max-files = De oudste geroteerde bestanden boven dit aantal worden verwijderd.
admin-settings-key-logging-file-max-total-mb = Maximale totale grootte (MB)
admin-settings-help-logging-file-max-total-mb = De oudste geroteerde bestanden worden verwijderd totdat de logmap onder deze grootte komt.
admin-settings-key-logging-file-compress = Geroteerde bestanden comprimeren
admin-settings-help-logging-file-compress = Geroteerde logbestanden worden met gzip gecomprimeerd.
admin-settings-key-logging-level = Logniveau
admin-settings-help-logging-level = Het niveau voor onderdelen die hieronder geen eigen niveau hebben. De omgevingsvariabele MW_LOG stelt het niveau bij het opstarten in totdat deze instelling is gelezen.
admin-settings-key-logging-component-levels = Logniveaus per onderdeel
admin-settings-help-logging-component-levels = Eén item onderdeel=niveau per regel, bijvoorbeeld mw_imap=debug. De niveaus zijn error, warn, info, debug en trace.
admin-settings-key-logging-redaction-addresses = E-mailadressen in logs
admin-settings-help-logging-redaction-addresses = Volledig laat adressen zoals gelogd. Afgekapt houdt de eerste letter en het domein. Gehasht vervangt het lokale deel door een korte hash. Geen verwijdert het adres.
admin-settings-key-logging-redaction-ips = IP-adressen in logs
admin-settings-help-logging-redaction-ips = Volledig laat adressen zoals gelogd. Afgekapt houdt het IPv4 /24- of IPv6 /48-voorvoegsel. Gehasht vervangt het adres door een korte hash. Geen verwijdert het.
admin-settings-key-logging-redaction-tokens = Geheimen in logs
admin-settings-help-logging-redaction-tokens = Haalt bearer-tokens, API-sleutels, wachtwoorden in URL's en vergelijkbare geheimen uit logregels.
admin-settings-key-retention-sessions-idle-days = Ongebruikte sessies (dagen)
admin-settings-help-retention-sessions-idle-days = Aantal dagen dat een sessierecord van een browser of native client, met adres en user-agent, ongebruikt mag blijven voordat het wordt verwijderd. De sessietime-outs gelden eerst; dit verwijdert alleen overgebleven rijen. 0 bewaart ze.
admin-settings-key-policy-public-pages = Openbare beleidspagina's
admin-settings-help-policy-public-pages = Lever het gegenereerde privacybeleid op /privacy en het cookiebeleid op /cookies-policy zonder aanmelding.
admin-settings-key-policy-auto-version = Automatische beleidsversies
admin-settings-help-policy-auto-version = Leg een nieuwe beleidsversie vast wanneer een instelling die het beleid beschrijft wijzigt. Uit: wijzigingen wachten op Publiceren.
admin-settings-key-policy-reconsent-on-change = Opnieuw om cookietoestemming vragen bij een nieuwe beleidsversie
admin-settings-help-policy-reconsent-on-change = Verhoog de versie van het cookiebeleid wanneer een nieuwe beleidsversie wordt vastgelegd, zodat bezoekers opnieuw worden gevraagd (vereist cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Naam van de verwerkingsverantwoordelijke
admin-settings-help-policy-controller-name = Naam van de organisatie die beslist waarom en hoe gegevens worden verwerkt, getoond in het privacybeleid.
admin-settings-key-policy-controller-address = Adres van de verwerkingsverantwoordelijke
admin-settings-help-policy-controller-address = Postadres van de verwerkingsverantwoordelijke, getoond in het privacybeleid.
admin-settings-key-policy-legal-bases = Rechtsgronden
admin-settings-help-policy-legal-bases = De rechtsgronden waarop u zich beroept, in uw eigen woorden, getoond in het privacybeleid.
admin-settings-key-policy-hosting-provider = Hostingprovider
admin-settings-help-policy-hosting-provider = Wie de gegevens host en waar, getoond in het privacybeleid.
admin-settings-key-policy-hosting-transfers = Doorgifte buiten het hostingland
admin-settings-help-policy-hosting-transfers = Elke doorgifte van gegevens naar andere landen en de waarborgen die u gebruikt, getoond in het privacybeleid.
admin-settings-key-policy-supervisory-authority = Toezichthoudende autoriteit
admin-settings-help-policy-supervisory-authority = De autoriteit waarbij gebruikers een klacht kunnen indienen, getoond in het privacybeleid.
admin-settings-key-dpia-suggest = Effectbeoordelingen voorstellen
admin-settings-help-dpia-suggest = Toon een voorgestelde effectbeoordeling wanneer de assistent, open-tracking of analyse is ingeschakeld en er geen bestaat.
admin-settings-key-dpia-review-months = Beoordelingsinterval effectbeoordeling
admin-settings-help-dpia-review-months = Maanden tot de beoordelingsdatum die wordt ingesteld wanneer een beoordeling wordt goedgekeurd.
admin-settings-key-dpia-reminder-days = Herinneringsvenster effectbeoordeling
admin-settings-help-dpia-reminder-days = Aantal dagen vóór de beoordelingsdatum waarop een beoordeling als aanstaand wordt getoond.
admin-settings-key-restriction-enabled = Beperkingen van de verwerking
admin-settings-help-restriction-enabled = Of een beheerder of een goedgekeurd verzoek de verwerking van de gegevens van een persoon kan beperken.
admin-settings-key-restriction-auto-apply-on-approval = Beperken wanneer een verzoek wordt goedgekeurd
admin-settings-help-restriction-auto-apply-on-approval = Pas de beperking toe zodra een beperkingsverzoek is goedgekeurd. Uit: een beheerder past haar toe via Naleving > Beperkingen.
admin-settings-key-restriction-mail-delivery = Nieuwe e-mail tijdens beperking

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-restriction-mail-delivery = Wat de beheerder heeft gekozen voor e-mail die binnenkomt terwijl een beperking geldt: bewaren, in de wachtrij zetten of weigeren. Deze server ontvangt geen e-mail; de waarde wordt vastgelegd en getoond, en de mailserver handelt ernaar.
admin-settings-key-restriction-default-expiry-days = Standaardduur van een beperking (dagen)
admin-settings-help-restriction-default-expiry-days = Aantal dagen tot een nieuwe beperking vanzelf eindigt. 0 houdt haar aan totdat iemand haar opheft.
admin-settings-key-restriction-notify-user = De persoon informeren
admin-settings-help-restriction-notify-user = Stuur de persoon een bericht wanneer een beperking wordt toegepast of opgeheven.
admin-settings-key-restriction-scope-indexing = Zoekindexering en embeddings pauzeren
admin-settings-help-restriction-scope-indexing = Of zoekindexering en embeddings standaard worden gepauzeerd bij een nieuwe beperking.
admin-settings-key-restriction-scope-assistant = De assistent pauzeren
admin-settings-help-restriction-scope-assistant = Of de assistent standaard wordt gepauzeerd bij een nieuwe beperking.
admin-settings-key-restriction-scope-analytics = Analyse en open-tracking pauzeren
admin-settings-help-restriction-scope-analytics = Of analyse en open-tracking standaard worden gepauzeerd bij een nieuwe beperking.
admin-settings-key-restriction-scope-classification = Regels en automatisch ordenen pauzeren
admin-settings-help-restriction-scope-classification = Of regels en automatisch ordenen standaard worden gepauzeerd bij een nieuwe beperking.
admin-settings-key-restriction-scope-push = Pushmeldingen pauzeren
admin-settings-help-restriction-scope-push = Of pushmeldingen standaard worden gepauzeerd bij een nieuwe beperking.
admin-settings-key-restriction-scope-third-party = Gekoppelde diensten en plug-ins pauzeren
admin-settings-help-restriction-scope-third-party = Of gekoppelde diensten en plug-ins standaard worden gepauzeerd bij een nieuwe beperking.
admin-settings-key-consent-enabled = Toestemming per doel
admin-settings-help-consent-enabled = Toon gebruikers de doelen en toestemmingskeuzes en dwing ze tijdens uitvoering af.
admin-settings-key-consent-receipts-retention-days = Bewaartermijn toestemmingsbewijzen (dagen)
admin-settings-help-consent-receipts-retention-days = Aantal dagen dat toestemmingsbewijzen worden bewaard. 0 bewaart ze totdat de persoon wordt gewist.
admin-settings-key-consent-art22-notice = Melding over geautomatiseerde verwerking
admin-settings-help-consent-art22-notice = Toon gebruikers de melding in gewone taal over assistent- en zoekfuncties, met de schakelaars per functie.
admin-settings-key-consent-analytics-basis = Rechtsgrond: gebruikstellingen
admin-settings-help-consent-analytics-basis = De rechtsgrond die gebruikers te zien krijgen voor gebruikstellingen. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-analytics-version = Tekstversie: gebruikstellingen
admin-settings-help-consent-analytics-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-analytics-text = Eigen formulering: gebruikstellingen
admin-settings-help-consent-analytics-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-analytics-tag-basis = Rechtsgrond: script van de analyseprovider
admin-settings-help-consent-analytics-tag-basis = De rechtsgrond die gebruikers te zien krijgen voor het script van de analyseprovider. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-analytics-tag-version = Tekstversie: script van de analyseprovider
admin-settings-help-consent-analytics-tag-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-analytics-tag-text = Eigen formulering: script van de analyseprovider
admin-settings-help-consent-analytics-tag-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-assistant-basis = Rechtsgrond: assistent
admin-settings-help-consent-assistant-basis = De rechtsgrond die gebruikers te zien krijgen voor de assistent. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-assistant-version = Tekstversie: assistent
admin-settings-help-consent-assistant-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-assistant-text = Eigen formulering: assistent
admin-settings-help-consent-assistant-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-push-basis = Rechtsgrond: pushmeldingen
admin-settings-help-consent-push-basis = De rechtsgrond die gebruikers te zien krijgen voor pushmeldingen. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-push-version = Tekstversie: pushmeldingen
admin-settings-help-consent-push-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-push-text = Eigen formulering: pushmeldingen
admin-settings-help-consent-push-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-open-tracking-basis = Rechtsgrond: open-tracking
admin-settings-help-consent-open-tracking-basis = De rechtsgrond die gebruikers te zien krijgen voor open-tracking. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-open-tracking-version = Tekstversie: open-tracking
admin-settings-help-consent-open-tracking-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-open-tracking-text = Eigen formulering: open-tracking
admin-settings-help-consent-open-tracking-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-remote-content-basis = Rechtsgrond: externe afbeeldingen
admin-settings-help-consent-remote-content-basis = De rechtsgrond die gebruikers te zien krijgen voor externe afbeeldingen. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-remote-content-version = Tekstversie: externe afbeeldingen
admin-settings-help-consent-remote-content-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-remote-content-text = Eigen formulering: externe afbeeldingen
admin-settings-help-consent-remote-content-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-integrations-basis = Rechtsgrond: gekoppelde diensten
admin-settings-help-consent-integrations-basis = De rechtsgrond die gebruikers te zien krijgen voor gekoppelde diensten. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-integrations-version = Tekstversie: gekoppelde diensten
admin-settings-help-consent-integrations-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-integrations-text = Eigen formulering: gekoppelde diensten
admin-settings-help-consent-integrations-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-consent-telemetry-basis = Rechtsgrond: laadtijdmetingen
admin-settings-help-consent-telemetry-basis = De rechtsgrond die gebruikers te zien krijgen voor laadtijdmetingen. Alleen toestemming laat de functie op een ja wachten; de andere gronden gelden tenzij de persoon intrekt. De beheerder bepaalt welke grond de juiste is.
admin-settings-key-consent-telemetry-version = Tekstversie: laadtijdmetingen
admin-settings-help-consent-telemetry-version = Verhoog dit getal wanneer de formulering of de verwerking verandert; wie met een oudere versie akkoord ging, wordt opnieuw gevraagd.
admin-settings-key-consent-telemetry-text = Eigen formulering: laadtijdmetingen
admin-settings-help-consent-telemetry-text = Uw eigen tekst voor dit doel, per taal. Leeg gebruikt de ingebouwde formulering.
admin-settings-key-breach-enabled = Datalekregister
admin-settings-help-breach-enabled = Schakelt het register en de meldingsklok in.
admin-settings-key-breach-notice-hours = Uren om de autoriteit te melden
admin-settings-help-breach-notice-hours = Uren vanaf het moment van kennisname tot de melding aan de autoriteit. De gebruikelijke termijn is 72.
admin-settings-key-breach-reminders-hours = Herinneringstijden
admin-settings-help-breach-reminders-hours = Uren vóór de deadline waarop een herinnering wordt gegeven, bijvoorbeeld 6, 24 en 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Herhaling na deadline
admin-settings-help-breach-reminders-overdue-repeat-hours = Uren tussen herinneringen nadat de deadline is verstreken. 0 herinnert één keer.
admin-settings-key-breach-authority-name = Toezichthoudende autoriteit
admin-settings-help-breach-authority-name = Naam die op de melding aan de autoriteit wordt afgedrukt.
admin-settings-key-breach-authority-url = Meldpagina van de autoriteit
admin-settings-help-breach-authority-url = Adres van de meldpagina van de autoriteit.
admin-settings-key-breach-authority-contact = Contact van de autoriteit
admin-settings-help-breach-authority-contact = Contactgegevens van de autoriteit.
admin-settings-key-breach-dpo-name = Naam contactpersoon gegevensbescherming
admin-settings-help-breach-dpo-name = Naam die in meldingen als contactpunt wordt vermeld.
admin-settings-key-breach-dpo-email = E-mailadres contactpersoon gegevensbescherming
admin-settings-help-breach-dpo-email = Adres dat in meldingen als contactpunt wordt vermeld.
admin-settings-key-breach-dpo-phone = Telefoonnummer contactpersoon gegevensbescherming
admin-settings-help-breach-dpo-phone = Telefoonnummer dat in meldingen wordt vermeld.
admin-settings-key-breach-template-subject = Onderwerp van het bericht
admin-settings-help-breach-template-subject = Onderwerp van het bericht aan getroffen personen, per taal.
admin-settings-key-breach-template-body = Berichttekst
admin-settings-help-breach-template-body = Tekst van het bericht aan getroffen personen, per taal.
admin-settings-key-breach-notify-audit = Meldingen vastleggen in het auditlogboek
admin-settings-help-breach-notify-audit = Herinneringen en detecties worden beveiligingsgebeurtenissen.
admin-settings-key-breach-notify-webhook-urls = Webhookadressen
admin-settings-help-breach-notify-webhook-urls = Adressen die over herinneringen en berichten worden ingelicht. Behandel ze als geheimen; elke aanroep is ondertekend.
admin-settings-key-breach-notify-operator-addresses = Mailboxen van operators
admin-settings-help-breach-notify-operator-addresses = Lokale mailboxadressen die herinneringen in hun eigen inbox ontvangen.
admin-settings-key-breach-subjects-in-app = Melding in het account van de persoon
admin-settings-help-breach-subjects-in-app = Toon het bericht in het account van de persoon.
admin-settings-key-breach-subjects-inbox = Bericht naar de mailbox van de persoon
admin-settings-help-breach-subjects-inbox = Bezorg het bericht in de eigen mailbox van de persoon.
admin-settings-key-breach-subjects-webhook = Ook naar webhooks sturen
admin-settings-help-breach-subjects-webhook = Stuur de berichttekst ook naar de webhookadressen.
admin-settings-key-breach-retention-years = Bewaartermijn register in jaren
admin-settings-help-breach-retention-years = Aantal jaren dat een gesloten incident in het register blijft.
admin-settings-key-breach-auto-enabled = Automatiseringen
admin-settings-help-breach-auto-enabled = Open conceptincidenten op basis van detecties.
admin-settings-key-breach-auto-start-clock = Klok van het concept
admin-settings-help-breach-auto-start-clock = Start de klok wanneer het concept wordt geopend, of pas wanneer het is bevestigd.
admin-settings-key-breach-auto-assignee = Concepten toewijzen aan
admin-settings-help-breach-auto-assignee = Aan wie een concept wordt toegewezen.
admin-settings-key-breach-auto-escalate-after-hours = Escaleren na uren
admin-settings-help-breach-auto-escalate-after-hours = Aantal uren dat een concept wacht voordat het wordt geëscaleerd. 0 escaleert nooit.
admin-settings-key-breach-auto-escalate-to = Escaleren naar
admin-settings-help-breach-auto-escalate-to = Aan wie een geëscaleerd concept wordt toegewezen.
admin-settings-key-breach-auto-window-minutes = Telvenster in minuten
admin-settings-help-breach-auto-window-minutes = Aantal minuten waarover een regel gebeurtenissen telt.
admin-settings-key-breach-auto-theft-threshold = Meldingen van sessiediefstal
admin-settings-help-breach-auto-theft-threshold = Meldingen binnen het venster die een concept openen. 0 is uit.
admin-settings-key-breach-auto-credential-stuffing-threshold = Aanmeldspray of -piek
admin-settings-help-breach-auto-credential-stuffing-threshold = Gebeurtenissen binnen het venster die een concept openen. 0 is uit.
admin-settings-key-breach-auto-mass-forward-threshold = Doorstuurregels in bulk
admin-settings-help-breach-auto-mass-forward-threshold = Gebeurtenissen binnen het venster die een concept openen. 0 is uit.
admin-settings-key-breach-auto-role-escalation-threshold = Wijzigingen van beheerdersrollen
admin-settings-help-breach-auto-role-escalation-threshold = Wijzigingen binnen het venster die een concept openen. 0 is uit.
admin-settings-key-breach-auto-export-volume-threshold = Ongebruikelijke exports
admin-settings-help-breach-auto-export-volume-threshold = Exports binnen het venster die een concept openen. 0 is uit.
admin-settings-key-breach-auto-audit-chain-threshold = Fouten in de auditketen
admin-settings-help-breach-auto-audit-chain-threshold = Fouten die een concept openen. 0 is uit.
admin-settings-key-breach-auto-backup-anomaly-threshold = Mislukte back-ups of herstelacties
admin-settings-help-breach-auto-backup-anomaly-threshold = Fouten binnen het venster die een concept openen. 0 is uit.
admin-settings-key-breach-auto-dsr-export-threshold = Veel exports van gegevensverzoeken
admin-settings-help-breach-auto-dsr-export-threshold = Exports binnen het venster die een concept openen. 0 is uit.
admin-settings-key-ratelimit-queue-ms = Langste wachttijd voor een vrije plek
admin-settings-help-ratelimit-queue-ms = Langste tijd dat een verzoek wacht op een vrije plek (JMAP-gelijktijdigheid, afbeeldingsdecoders, voorbeeldworkers) voordat het wordt geweigerd met 429 en Retry-After. Bereik 0-30000 ms; standaard 3000. Nul weigert direct.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Gelijktijdige JMAP-verzoeken per account

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-ratelimit-jmap-per-account-concurrent = JMAP-verzoeken die één ingelogd account tegelijk mag uitvoeren, over al zijn tabbladen en apparaten heen. Dit wordt nooit gedeeld met een ander account of een andere gebruiker achter hetzelfde adres. Bereik 4-64; standaard 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Gelijktijdige JMAP-verzoeken, alle accounts
admin-settings-help-ratelimit-jmap-global-concurrent = JMAP-verzoeken die het hele serverproces tegelijk uitvoert, alle accounts samen. Een verzoek daarboven wacht gedurende de wachtrijtijd. Bereik 4-4096; standaard 256.
admin-settings-key-ratelimit-session-burst = Tijdsrapporten per sessie achter elkaar
admin-settings-help-ratelimit-session-burst = Tijdsrapporten van de client die één sessie achter elkaar mag sturen. Het endpoint gebruikt de sessie als sleutel, nooit het adres. Bereik 1-1000; standaard 60.
admin-settings-key-ratelimit-session-per-minute = Tijdsrapporten die per minuut terugkomen
admin-settings-help-ratelimit-session-per-minute = Tijdsrapporten van de client per minuut die een sessie na zijn burst terugkrijgt. Bereik 1-600; standaard 30.
admin-settings-key-ratelimit-anon-burst = Anonieme toestemmingsverzoeken achter elkaar
admin-settings-help-ratelimit-anon-burst = Verzoeken die één adres achter elkaar mag doen naar de anonieme routes voor cookietoestemming. Bemeten op een kantoor achter één adres. Bereik 1-1000; standaard 60.
admin-settings-key-ratelimit-anon-per-minute = Anonieme toestemmingsverzoeken die per minuut terugkomen
admin-settings-help-ratelimit-anon-per-minute = Verzoeken per minuut die een adres terugkrijgt op de anonieme routes voor cookietoestemming. Bereik 1-600; standaard 30.
admin-settings-key-ratelimit-trusted-multiplier = Accounthoeveelheid vanuit een bekende bron (%)
admin-settings-help-ratelimit-trusted-multiplier = Percentage van de verzoekhoeveelheid van het account dat een ingelogde mailbox krijgt wanneer het verzoek komt van een bron met een geslaagde aanmelding op record. 100 houdt de basishoeveelheid aan. Versoepelt nooit de budgetten voor anoniem, beheerder of user-agent, noch een actieve blokkade. Bereik 100-1000; standaard 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Of u nu ergens op de bijlagenbalk van de lezer klikt, deze wordt uitgevouwen of samengevouwen. Wanneer uitgeschakeld, doet alleen de pijlknop dat.

admin-settings-help-attachments-bar-download-all = Of de bijlagenbalk een knop Alles downloaden biedt waarmee de toegestane bijlagen als één ZIP-bestand worden opgeslagen.

admin-settings-help-attachments-bar-show-download-buttons = Wanneer elke bijlage in de balk zijn downloadknop toont: bij hover of focus (standaard), altijd of nooit. Downloads volgen nog steeds het bijlagebeleid en mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Of afbeeldingen die in de berichttekst worden weergegeven, zijn opgenomen in Alles downloaden ZIP. Standaard uitgeschakeld.

admin-settings-help-cap-reading-delete-sender = Sta gebruikers toe de leesactie in te schakelen die alle overeenkomende afzenderberichten na bevestiging naar de prullenbak verplaatst. Standaard uitgeschakeld. Het kenmerkprofiel moet dit ook toelaten.

admin-settings-help-compose-feature-family = Schakel een componistbediening in of uit op basis van de feature-ID. Standaard ingeschakeld; vergrendel het om te voorkomen dat gebruikers het wijzigen.

admin-settings-help-compose-headers-allow-auto-submitted = Hiermee kunnen gebruikers Automatisch verzonden (RFC 3834) instellen voor een bericht. Standaard uitgeschakeld: het vertelt andere systemen om niet te antwoorden.

admin-settings-help-compose-headers-allow-custom = Hiermee kunnen gebruikers hun eigen headers toevoegen (maximaal 20, namen gecontroleerd, beveiligde headers geweigerd).

admin-settings-help-compose-headers-allow-return-receipt-to = Hiermee kunnen gebruikers ontvangstbewijzen opvragen met de oude Return-Receipt-To-header. Standaard uitgeschakeld.

admin-settings-help-compose-headers-allow-threading-edit = Hiermee kunnen gebruikers In-Reply-To en verwijzingen bewerken, of een nieuwe thread starten vanuit een antwoord.

admin-settings-help-compose-headers-forced = Eén Naam=waarde-paar per regel, toegevoegd aan elk bericht dat vanuit het domein wordt verzonden en ter vervanging van de waarde van de gebruiker. Beschermde headers worden geweigerd.

admin-settings-help-compose-insert-image-resize-default = De grootte van het dialoogvenster Afbeelding uit bestand begint met: origineel, groot (1600 px) of medium (1024 px).

admin-settings-help-compose-insert-max-inline-image-mb = Afbeeldingen die groter zijn dan dit worden niet in de berichttekst geplaatst; ze kunnen nog steeds worden bevestigd. 1 tot 50.

admin-settings-help-compose-layout-default-font = Lettertypefamilie, grootte en kleur waarmee een nieuw bericht begint, als family|size|#colour. Leeg gebruikt het themalettertype.

admin-settings-help-compose-reply-to-allow-override = Als deze optie is uitgeschakeld, wordt de Reply-To van de identiteit gebruikt zoals deze is en kunnen gebruikers deze niet bewerken of verwijderen.

admin-settings-help-compose-reply-to-show = Of de opsteller de Reply-To-rij toont: alleen als een identiteit of het originele bericht er één heeft, altijd of nooit.

admin-settings-help-compose-sent-folder-fixed-id = Maprol of ID gebruikt als de regel 'vaste map' is.

admin-settings-help-compose-sent-folder-mode = Waar de kopie van een verzonden bericht naartoe gaat: de normale map Verzonden, één vaste map, op domein van de ontvanger, op identiteit of op onderwerpregels.

admin-settings-help-compose-sent-folder-per-identity = Eén identiteit=mappaar per regel. Wordt gebruikt als de regel 'op identiteit' is.

admin-settings-help-compose-sent-folder-rules = Eén regel per regel, domain:example.com=map of subject:text=map. De eerste wedstrijd wint.

admin-settings-help-compose-sent-folder-skip-copy = Bewaar nooit een kopie van een verzonden bericht. Een beheerder kan dit geforceerd in- of uitschakelen.

admin-settings-help-compose-styles-custom = De eigen componiststijlen van een gebruiker als JSON, geschreven door de Stijlengalerij. Gebruikers stellen het in; er valt hier niets te configureren.

admin-settings-help-compose-styles-default-set = Het uiterlijk van de Stijlengalerij van de componist: standaard, modern, klassiek of compact.

admin-settings-help-compose-styles-max-custom = De meeste aangepaste stijlen die een gebruiker in de componist kan behouden. 0 schakelt Stijl maken uit.

admin-settings-help-contacts-save-from-message-add-note = Voeg de notitie "Opgeslagen uit een bericht op <datum>" toe aan contacten die zijn opgeslagen uit een bericht. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-contacts-save-from-message-default-book = De adresboek-ID die contacten ontvangt die zijn opgeslagen uit een bericht; "standaard" gebruikt het standaardadresboek. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-contacts-save-from-message-default-scope = Welke deelnemers als eerste zijn aangevinkt in het dialoogvenster Contacten opslaan: alleen de afzender, de afzender en Reply-To, of iedereen. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-contacts-save-from-message-enabled = Aanbieding Bewaar contacten uit dit bericht in berichtmenu's. Wanneer uitgeschakeld, is het item verborgen. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-contacts-save-from-message-skip-automated = Laat no-reply-, notificatie- en bounce-adressen buiten het dialoogvenster Contacten opslaan, tenzij u dit handmatig aanvinkt. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-contacts-save-sender-enabled = Bied de mogelijkheid om met één klik de afzender op te slaan als contactitem in berichtmenu's. Wanneer uitgeschakeld, is het item verborgen. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-contacts-save-sender-use-reply-to = Sla het antwoordadres op in plaats van Van als de afzender een mailinglijst- of meldingsadres is. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-dsr-admin-step-up = Aan (standaard): een beheerder die een gegevensverzoek goedkeurt, een verwijdering uitvoert of bulksgewijs goedkeurt, moet eerst bevestigen dat hij/zij het opnieuw is (wachtwoord en tweede factor, of een wachtwoordsleutel). De bevestiging duurt vijf minuten. Uit verwijdert de extra controle; de beslissing wordt nog steeds vastgelegd in het verzoekspoor.

admin-settings-help-export-default-format = Het formaat waarmee het exportdialoogvenster begint.

admin-settings-help-export-default-scope = Het bereik waarmee het exportdialoogvenster begint: bericht, gesprek, selectie of map.

admin-settings-help-export-dialog-enabled = Exporteren in de berichtweergave en de berichtmenu's openen ter plaatse een dialoogvenster. Als deze menu's zijn uitgeschakeld, downloaden ze het ene bericht als .eml.

admin-settings-help-export-folder-enabled = Sta gebruikers toe een hele map te downloaden als een ZIP aan berichten. Elke export wordt gecontroleerd.

admin-settings-help-export-folder-format = De formaten die een mapexport biedt: een ZIP van .eml-bestanden, een ZIP met een mbox-bestand, of beide.

admin-settings-help-export-folder-include-subfolders = Of een mapexport standaard de submappen bevat, bewaard als mapstructuur in de ZIP.

admin-settings-help-export-folder-max-bytes = Het aantal bytes aan berichten dat één mapexport kan bevatten. Een grotere map met het weergegeven formaat wordt geweigerd.

admin-settings-help-export-folder-max-messages = Het aantal berichten dat één mapexport kan bevatten.

admin-settings-help-export-formats = De formaten die het exportdialoogvenster biedt: eml, mbox, txt, md, msg, docx, csv, json, pdf. De server weigert een vermeld, door de server gebouwd formaat.

admin-settings-help-export-message-download-enabled = Sta gebruikers toe een bericht te downloaden als .eml, alleen de headers, of als een mbox-bestand.

admin-settings-help-folders-hide-empty = Verberg lege mappen in de zijbalk. Gebruikers kunnen de standaardinstelling wijzigen of overschrijven voor individuele mappen, tenzij deze instelling is vergrendeld. Er wordt geen e-mail verwijderd.

admin-settings-help-folders-pinned-hide-from-tree = Vastgezette mappen worden weergegeven in een Vastgezette groep boven de mappenboom en behouden ook hun plaats in de boom. Aan verbergt een vastgezette map die geen submappen in de structuur heeft, zodat deze alleen in de groep Vastgezet wordt weergegeven. Een vastgezette map met submappen blijft in de boomstructuur staan.

admin-settings-help-mail-badge-auth = Wanneer de header van de lezer de authenticatiebadge DMARC toont: altijd, alleen wanneer opmerkelijk, of nooit.

admin-settings-help-mail-badge-density = Grootte van de badges in de header van de lezer: compact of comfortabel.

admin-settings-help-mail-badge-folder = Wanneer de header van de lezer de mapbadge toont: altijd, alleen wanneer opmerkelijk of nooit.

admin-settings-help-mail-badge-headers = Toon een badge in de reader voor de headers Expires, Reply-By, Sensitivity en Supersedes van een bericht.

admin-settings-help-mail-badge-labels = Of kopbadges van lezers een pictogram met tekst tonen, of alleen het pictogram.

admin-settings-help-mail-badge-list = Wanneer de header van de lezer de mailinglijstbadge toont: altijd, alleen wanneer opmerkelijk of nooit. Het heeft een List-Id- of List-Unsubscribe-header nodig.

admin-settings-help-mail-badge-sanitized = Wanneer de header van de lezer de opgeschoonde HTML-badge toont: altijd, alleen wanneer opmerkelijk, of nooit.

admin-settings-help-mail-badge-spam = Wanneer de header van de lezer de spamscore-badge toont: altijd, alleen wanneer opmerkelijk of nooit.

admin-settings-help-mail-brand-avatars-require-auth = nooit: toon een logo wanneer het domein overeenkomt. bekende fout (standaard): toon dit tenzij het in de cache opgeslagen resultaat DKIM of DMARC van het bericht een fout is; niet-aangevinkte berichten behouden het logo. altijd: toon het alleen na een afgestemde DMARC pass met DKIM voor het eigen domein van het merk. Gebruikers- en domeinoverschrijvingen kunnen worden vergrendeld.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Enkele seconden moet de hoofdtekst de focus vasthouden voordat een volledige ontvanger en onderwerpkop naar de samenvatting worden gevouwen. Kies 0 om getimed vouwen uit te schakelen, of 1–60 seconden. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-mail-compose-priority-default = De prioriteit waarmee een nieuw bericht begint in de opsteller.

admin-settings-help-mail-compose-priority-enabled = Toon de selector Normale, Hoge en Lage prioriteit in de groep Composeropties. Indien uitgeschakeld, wordt nieuwe e-mail verzonden zonder prioriteitskopteksten.

admin-settings-help-mail-expiry-hide-expired = Wat te doen met een bericht waarvan de vervaldatum is verstreken: niets, grijs weergeven of voorstellen om het te archiveren.

admin-settings-help-mail-list-keep = Afzenders en mailinglijsten worden altijd bijgehouden, één per regel als sender:address of list:List-Id. Ruim deze afzender op, afmeldsuggesties en bewaarregels sla deze over.

admin-settings-help-mail-list-page-size = Hoeveel berichten de lijst ophaalt telkens wanneer een gebruiker naar het einde scrollt. Standaard 50, van 20 tot 200. Gebruikers kiezen hun eigen waarde, tenzij u deze vergrendelt.

admin-settings-help-mail-list-unsubscribe-mode = Hoe Afmelden werkt als een lijst een verzoek met één klik aanbiedt: vraag het eerst, verzend het meteen of open alleen de link of het concept van de lijst.

admin-settings-help-mail-message-list-group = Groepeer gesprekken op ontvangen of verzonden datum, afzender, onderwerp, belang of grootte. Gebruikers kunnen dit overschrijven, tenzij ze zijn vergrendeld.

admin-settings-help-mail-priority-marks = Welke prioriteit de berichtenlijst, gesprekken en lezerweergave markeert: zowel hoog als laag, alleen hoog of geen. Gebruikers kiezen onder Instellingen > Mappen en zijbalk, tenzij vergrendeld.

admin-settings-help-mail-priority-marks-style = Hoe een prioriteitsmarkering eruit ziet: een pictogram, een pictogram met de woorden Hoog of Laag, of een dunne balk. Icoon en tekst behouden de betekenis zonder kleur.

admin-settings-help-mail-priority-send-headers = Welke headers een bericht met hoge of lage prioriteit krijgt wanneer het wordt verzonden: Belang, X-Prioriteit en X-MSMail-Prioriteit.

admin-settings-help-mail-priority-trust-sender-headers = Of de door de afzender ingestelde prioriteit wordt weergegeven: altijd, alleen als het bericht een uitgelijnde DMARC-controle heeft doorstaan, of nooit. Gebruik vertrouwd of nooit tegen afzenders die elk bericht met hoge prioriteit markeren.

admin-settings-help-mail-reply-from-header-priority = De berichtkoppen worden in deze volgorde gelezen om het adres te vinden waarnaar een bericht is verzonden bij het kiezen van de identiteit voor een antwoord. Namen: Bezorgd aan, X-Origineel-aan, Envelop-aan, X-doorgestuurd naar, Opnieuw verzonden aan, Aan, Cc. Een weggelaten header wordt niet gelezen.

admin-settings-help-mail-search-default-scope = Start nieuwe normale e-mailzoekopdrachten in de huidige map of in alle mappen. Dit regelt de initiële standaard, inclusief wanneer deze is vergrendeld; gebruikers kunnen nog steeds een ander bereik kiezen voor een bepaalde zoekopdracht. Opgeslagen en expliciet bereikbare query's blijven behouden.

admin-settings-help-metrics-dimensions-enabled = Sla de opgeslagen statistieken op, opgesplitst per clienttype (web) en apparaatklasse (desktop, mobiel). Schakel uit om één gecombineerde rij per statistiek en minuut op te slaan.

admin-settings-help-metrics-persist-enabled = Sla de prestatiestatistieken (beeldproxysnelheid, browserlaadtijden, snelheidslimiet en aantal verzoekenbescherming) op in de database bij elk spoelinterval en wanneer de server stopt, zodat de grafieken hun geschiedenis behouden na een herstart. Niets identificeert een persoon, adres of bericht. Schakel uit om ze alleen in het geheugen te bewaren.

admin-settings-help-metrics-persist-flush-seconds = Hoe vaak de prestatiestatistieken naar de database worden geschreven (5 tot 3600 seconden). Een server die zonder waarschuwing wordt afgesloten, verliest hoogstens zoveel.

admin-settings-help-observability-client-capture-console = Rapporteer ook console.error-oproepen, maximaal vijf per minuut per pagina. Standaard uitgeschakeld omdat bibliotheken de verwachte omstandigheden registreren.

admin-settings-help-observability-client-capture-csp = Rapporteer schendingen van het inhoudsbeveiligingsbeleid die de browser detecteert, met alleen de geblokkeerde richtlijn en oorsprong. Standaard ingeschakeld.

admin-settings-help-observability-client-enabled = Verzamel foutrapporten van de web-app (foutgrenzen, niet-opgevangen fouten, mislukte chunks, werkerfouten, CSP-overtredingen) met opgeschoonde stapeltraceringen, weergegeven onder Waarneembaarheid > Clientfouten. Standaard ingeschakeld. Rapporten bevatten geen berichtinhoud, adressen, tokens, namen van bijlagen of URL-query's. Schakel uit om het verzamelen te stoppen; bestaande rapporten vervallen met hun bewaring.

admin-settings-help-observability-client-forward = Stuur elk opgeschoond rapport ook naar de fout-sink die is geconfigureerd met MW_ERROR_FORWARD_URL. Standaard uitgeschakeld. Zonder geconfigureerde sink wordt er niets verzonden.

admin-settings-help-observability-client-include-account-hash = Voeg een ingetoetste eenrichtingshash van het account toe aan rapporten die worden verzonden vanuit een ingelogde sessie, zodat rapporten van één account van elkaar kunnen worden onderscheiden zonder deze een naam te geven. Standaard uitgeschakeld. De hash verandert wanneer de privacysleutel draait.

admin-settings-help-observability-client-max-per-session = De meeste foutrapporten kunnen worden verzonden na het laden van één pagina (1 tot 1000). Herhalingen van dezelfde fout binnen 60 seconden tellen één keer. Standaard 20.

admin-settings-help-observability-client-sample-rate = Fractie van foutrapporten die een browser verzendt, van 0 tot 1. Standaard 1 (alles). Het laden van elke pagina stopt nog steeds bij de limiet per sessie.

admin-settings-help-observability-client-source-maps = Los bundelposities in stapeltraceringen op naar bronbestanden wanneer de build verborgen bronkaarten heeft verzonden (gebouwd met MW_SOURCEMAP=hidden). Kaarten worden alleen door de server gelezen en nooit aan browsers aangeboden. Standaard ingeschakeld; zonder kaarten worden de onbewerkte frames getoond.

admin-settings-help-privacy-error-reports = Of een browser een foutmelding stuurt als een deel van de app uitvalt. Standaard ingeschakeld. Globaal, domein en gebruikersbereik: stel hier de standaard in of vergrendel deze; mensen kunnen het zelf uitschakelen via Instellingen > Privacy, tenzij het vergrendeld is.

admin-settings-help-privacy-image-cache-max-stale-secs = Seconden voorbij zijn versheid wordt een in de cache opgeslagen afbeelding bewaard om te antwoorden wanneer de oorsprong niet kan worden bereikt. Nooit onder het revalidatievenster. Bereik 0–604800; standaard 86400.

admin-settings-help-privacy-image-cache-stale-secs = Seconden voorbij de versheid wordt een in het cachegeheugen opgeslagen afbeelding nog steeds onmiddellijk weergegeven terwijl deze op de achtergrond opnieuw wordt gevalideerd. Nul schakelt het uit. Bereik 0–86400; standaard 3600.

admin-settings-help-privacy-image-resize-enabled = Schaal extra grote afbeeldingen op afstand terug naar de breedte waar de lezer om vraagt (320, 640, 960, 1280 of 1600 pixels). Geanimeerde afbeeldingen worden ongeschaald weergegeven. Standaard uitgeschakeld.

admin-settings-help-privacy-image-resize-format = Automatisch kiest de beste encoder in de build, namelijk PNG. Standaard automatisch.

admin-settings-help-privacy-image-resize-max-pixels = Afbeeldingen met meer gedecodeerde pixels dan dit worden ongeschaald weergegeven; de grootte wordt vóór elke decodering uit de header gelezen. Bereik 100.000–64.000.000; standaard 4000000.

admin-settings-help-privacy-image-resize-max-width = De breedste afbeelding die het formaat wijzigt. Bereik 320–1600; standaard 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Hoe Instellingen > Beveiliging > Sessies de account een naam geeft waar een toegangssessie voor een gekoppelde mailbox vandaan komt: het volledige adres, het adres waarvan het eerste deel van de naam is gemaskeerd, of niets. De waarde wordt alleen getoond aan de eigenaar van de sessies, nooit aan andere gebruikers. Standaard: volledig adres.

admin-settings-help-properties-folder-enabled = Bied eigenschappen aan in de mapmenu's en op de snelkoppeling. Indien uitgeschakeld, is het dialoogvenster niet beschikbaar.

admin-settings-help-properties-folder-notes-enabled = Sta een beschrijving of notitie toe voor elke map.

admin-settings-help-properties-folder-notes-max-length = Het maximale aantal tekens dat een mapnotitie kan bevatten.

admin-settings-help-properties-folder-overrides-allowed = Welke instellingen een gebruiker voor een enkele map mag overschrijven. Een overschrijving die niet in de lijst staat, wordt verborgen en elke opgeslagen waarde wordt genegeerd.

admin-settings-help-properties-folder-sections = De secties van het dialoogvenster Eigenschappen van de map: algemeen, extern, grootte, offline, overschrijvingen, notities, acties. Een sectie die niet in de lijst staat, is verborgen.

admin-settings-help-properties-folder-size-compute = Wanneer de grootte van een map op de server wordt bepaald: nooit, wanneer het dialoogvenster wordt geopend, of op de achtergrond. Het tellen van een grote mailbox kost de server een scan.

admin-settings-help-properties-headers-raw = Sta toe dat het dialoogvenster Eigenschappen van het bericht de onbewerkte headers weergeeft. Wanneer uitgeschakeld, wordt alleen de geparseerde samenvatting weergegeven.

admin-settings-help-properties-message-enabled = Bied eigenschappen aan in de berichtmenu's en op de snelkoppeling. Indien uitgeschakeld, is het dialoogvenster niet beschikbaar.

admin-settings-help-properties-message-permanent-delete = Of het berichtdialoogvenster een bericht voorgoed mag verwijderen: toestaan, twee keer vragen of weigeren. Een bericht onder wettelijke bewaarplicht wordt nooit verwijderd. Het op deze manier verwijderen kan niet ongedaan worden gemaakt.

admin-settings-help-properties-message-sections = De secties van het dialoogvenster Eigenschappen van het bericht: samenvatting, score, kopteksten, bijlagen, acties. Een sectie die niet in de lijst staat, is verborgen.

admin-settings-help-properties-offline-clear-enabled = Sta gebruikers toe de offline cache van een enkele map te wissen via het dialoogvenster Eigenschappen.

admin-settings-help-properties-remote-location-visibility = Hoeveel van de locatie aan de serverzijde zien gebruikers: volledig (host en pad), gemaskeerde host (host gedeeltelijk verborgen), alleen pad (postbuspad zonder de host) of verborgen.

admin-settings-help-properties-scoring-show = Toon spamscore, authenticatieresultaten en beveiligingssignalen in het berichteigenschappendialoogvenster.

admin-settings-help-properties-shortcut-enabled = Open het dialoogvenster Eigenschappen vanaf het toetsenbord in een specifieke map of geselecteerd bericht.

admin-settings-help-properties-shortcut-key = De toetsencombinatie die Eigenschappen opent; Standaard Alt+Enter.

admin-settings-help-reader-actions-archive = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-block-sender = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-colors = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-conversation = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-create-filter = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-delete = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-delete-sender = Standaard verborgen. Kies de werkbalk of het menu Meer om dit in te schakelen wanneer de mogelijkheid voor het verwijderen van de afzender en het functieprofiel dit toestaan. Gebruikers kunnen de plaatsing overschrijven, tenzij deze is vergrendeld.

admin-settings-help-reader-actions-edit-draft = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-export = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-finish-followup = Kies werkbalk, menu Meer of verborgen voor het voltooien van gemarkeerde berichten en vervolgherinneringen.

admin-settings-help-reader-actions-flag = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-focus-focused = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-focus-other = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-forward = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-headers = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-links = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-move = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-not-spam = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-open-draft = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-pin = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-print = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-properties = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-read = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-reply = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-reply-all = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-save-contacts = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-save-sender = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-search = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-security = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-spam = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-sweep = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-view-source = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-reader-actions-zoom = Kies werkbalk, menu Meer of verborgen. Functiebeperkingen, machtigingen en berichtcontext zijn nog steeds van toepassing. Gebruikers kunnen kiezen tenzij dit is vergrendeld.

admin-settings-help-retention-client-errors-event-days = Dagen waarop een individuele browserfout optreedt (de stacktracering en browserklasse) wordt bijgehouden. Standaard 30. Een groep wordt langer bewaard, zie de volgende instelling.

admin-settings-help-retention-client-errors-group-days = Dagen waarop een browserfoutgroep (naam, bericht, aantal) wordt bewaard nadat deze voor het laatst is gezien. Standaard 180. Nooit korter dan de retentie van het voorkomen.

admin-settings-help-retention-metrics-day-buckets-days = Dagen van dagelijkse prestatiestatistieken die worden bewaard voordat ze worden verwijderd. 0 houdt ze zonder limiet, waardoor de tafel langzaam groeit zolang de server draait.

admin-settings-help-retention-metrics-hour-buckets-days = Dagen met prestatiestatistieken per uur die worden bewaard voordat ze worden samengevoegd in dagelijkse rijen (ten minste 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Uren met prestatiestatistieken van één minuut die worden bijgehouden voordat ze worden samengevoegd in rijen per uur (ten minste 1).

admin-settings-help-retention-user-allow-strip-attachments = Of gebruikers regels mogen maken die grote bijlagen uit oude e-mail verwijderen. Standaard uitgeschakeld. Als deze actie is uitgeschakeld, wordt deze verborgen in de regeleditor. Een regel die er gebruik van maakt, wordt geweigerd wanneer deze wordt opgeslagen, en eerder opgeslagen regels worden overgeslagen wanneer de taak wordt uitgevoerd.

admin-settings-help-retention-user-calendar-enabled = Of gebruikers agenda's mogen kiezen waarvan de afspraken een bepaald aantal dagen nadat ze zijn beëindigd, worden verwijderd. Standaard uitgeschakeld. Als deze optie is uitgeschakeld, is deze verborgen in Instellingen, wordt een keuze geweigerd wanneer deze wordt opgeslagen en worden eerder opgeslagen keuzes niet uitgevoerd.

admin-settings-help-retention-user-calendar-max-days = De meeste dagen kan een gebruiker wachten voordat beëindigde gebeurtenissen worden verwijderd. Een grotere waarde wordt geweigerd bij het opslaan en verlaagd als deze wordt uitgevoerd.

admin-settings-help-retention-user-calendar-min-days = Minste dagen nadat een gebeurtenis is geëindigd voordat een gebruikersregel deze kan verwijderen. Een kleinere waarde wordt geweigerd wanneer deze wordt opgeslagen en verhoogd wanneer deze wordt uitgevoerd.

admin-settings-help-retention-user-notes-enabled = Of gebruikers ervoor kunnen kiezen notities te verwijderen die gedurende een bepaald aantal dagen niet zijn gewijzigd. Standaard uitgeschakeld. Verwijderde notities kunnen worden hersteld totdat het venster voor ongedaan maken voor bewaarregels eindigt.

admin-settings-help-retention-user-notes-max-days = De meeste dagen kan een gebruiker wachten voordat onaangeroerde notities worden verwijderd. Een grotere waarde wordt geweigerd bij het opslaan en verlaagd als deze wordt uitgevoerd.

admin-settings-help-retention-user-notes-min-days = Minimaal aantal dagen moet een notitie onaangeroerd blijven voordat een gebruikersregel deze kan verwijderen. Een kleinere waarde wordt geweigerd wanneer deze wordt opgeslagen en verhoogd wanneer deze wordt uitgevoerd.

admin-settings-help-retention-user-strip-keep-inline-images = Of afbeeldingen die in de berichttekst worden weergegeven, behouden blijven wanneer bijlagen worden verwijderd.

admin-settings-help-retention-user-strip-min-age-days = Het minimum aantal dagen dat een bericht oud moet zijn voordat een regel de bijlagen mag verwijderen. Een kleinere waarde wordt geweigerd wanneer de regel wordt opgeslagen en verhoogd wanneer deze wordt uitgevoerd.

admin-settings-help-retention-user-strip-min-attachment-kb = Alleen bijlagen van minimaal dit formaat worden verwijderd. Kleinere bijlagen blijven bij het bericht.

admin-settings-help-retention-user-strip-undo-days = Dagen dat het oorspronkelijke bericht in de prullenbak blijft staan nadat de bijlagen zijn verwijderd. -1 volgt het ongedaan maken-venster voor bewaarregels. 0 verwijdert het origineel in één keer.

admin-settings-help-server-shutdown-grace-seconds = Hoe lang een stoppende server verzoeken tijdens de vlucht laat voltooien voordat de verbindingen worden gesloten, in seconden. De variabele MW_HTTP_SHUTDOWN_TIMEOUT_MS overschrijft deze. Geldt bij de volgende start.

admin-settings-help-session-last-activity-flush-seconds = De laatst geziene tijd van een sessie wordt hoogstens zo vaak weergegeven, in seconden. Zero schrijft het bij elk verzoek. Het verandert niets aan de inactieve of absolute limieten. Geldt bij de volgende start.

admin-settings-help-session-persist-across-restarts = Indien ingeschakeld, blijven mailbox-, app- en consolesessies geldig nadat de server opnieuw is opgestart. Indien uitgeschakeld, eindigt elke sessie wanneer de server start. Geldt bij de volgende start.

admin-settings-key-attachments-bar-click-to-toggle = Klik op de bijlagenbalk om te schakelen

admin-settings-key-attachments-bar-download-all = Knop Alles downloaden

admin-settings-key-attachments-bar-show-download-buttons = Downloadknoppen voor bijlagen

admin-settings-key-attachments-bar-zip-inline-images = Inline afbeeldingen in de Download alles ZIP

admin-settings-key-cap-reading-delete-sender = Sta het verwijderen van alle e-mail van een afzender toe

admin-settings-key-compose-feature-family = Composerfuncties

admin-settings-key-compose-headers-allow-auto-submitted = Sta automatisch verzonden toe

admin-settings-key-compose-headers-allow-custom = Aangepaste kopteksten toestaan

admin-settings-key-compose-headers-allow-return-receipt-to = Retour-ontvangst-aan toestaan

admin-settings-key-compose-headers-allow-threading-edit = Bewerken van threadingkoppen toestaan

admin-settings-key-compose-headers-forced = Geforceerde kopteksten

admin-settings-key-compose-insert-image-resize-default = Vooraf ingesteld beeldformaat

admin-settings-key-compose-insert-max-inline-image-mb = Grootste inline-afbeelding (MB)

admin-settings-key-compose-layout-default-font = Standaard berichtlettertype

admin-settings-key-compose-reply-to-allow-override = Wijzigen van antwoord aan toestaan

admin-settings-key-compose-reply-to-show = Antwoord-aan-rij

admin-settings-key-compose-sent-folder-fixed-id = Verzonden kopie: vaste map

admin-settings-key-compose-sent-folder-mode = Verzonden kopie: mapregel

admin-settings-key-compose-sent-folder-per-identity = Verzonden kopie: map per identiteit

admin-settings-key-compose-sent-folder-rules = Verzonden kopie: regels

admin-settings-key-compose-sent-folder-skip-copy = Verzonden kopie: niet opslaan

admin-settings-key-compose-styles-custom = Aangepaste stijlen

admin-settings-key-compose-styles-default-set = Stijl ingesteld

admin-settings-key-compose-styles-max-custom = Aangepaste stijlen per gebruiker

admin-settings-key-contacts-save-from-message-add-note = Opmerking over contacten die zijn opgeslagen vanuit een bericht

admin-settings-key-contacts-save-from-message-default-book = Adresboek voor contacten opgeslagen uit een bericht

admin-settings-key-contacts-save-from-message-default-scope = Contacten zijn als eerste aangevinkt bij het opslaan van een bericht

admin-settings-key-contacts-save-from-message-enabled = Bewaar contacten uit een bericht

admin-settings-key-contacts-save-from-message-skip-automated = Sla geautomatiseerde afzenders over bij het opslaan van contacten

admin-settings-key-contacts-save-sender-enabled = Afzender opslaan als contactpersoon

admin-settings-key-contacts-save-sender-use-reply-to = Bewaar Reply-To voor lijstafzenders

admin-settings-key-dsr-admin-step-up = Bevestig opnieuw voordat u goedkeurt

admin-settings-key-export-default-format = Standaardformaat van dialoogvenster exporteren

admin-settings-key-export-default-scope = Standaardbereik van dialoogvenster exporteren

admin-settings-key-export-dialog-enabled = Dialoogvenster exporteren

admin-settings-key-export-folder-enabled = Map exporteren

admin-settings-key-export-folder-format = Exportformaat voor mappen

admin-settings-key-export-folder-include-subfolders = Neem submappen op in de export

admin-settings-key-export-folder-max-bytes = Limiet voor exportgrootte van map

admin-settings-key-export-folder-max-messages = Berichtlimiet voor mapexport

admin-settings-key-export-formats = Dialoogformaten exporteren

admin-settings-key-export-message-download-enabled = Bericht downloaden

admin-settings-key-folders-hide-empty = Verberg lege mappen

admin-settings-key-folders-pinned-hide-from-tree = Verberg vastgezette mappen in de boomstructuur

admin-settings-key-mail-badge-auth = Authenticatiebadge

admin-settings-key-mail-badge-density = Grootte van lezersbadge

admin-settings-key-mail-badge-folder = Mapbadge

admin-settings-key-mail-badge-headers = Kopbadges

admin-settings-key-mail-badge-labels = Etiketten voor lezersbadges

admin-settings-key-mail-badge-list = Mailinglijst-badge

admin-settings-key-mail-badge-sanitized = Gezuiverde HTML-badge

admin-settings-key-mail-badge-spam = Spamscore-badge

admin-settings-key-mail-brand-avatars-require-auth = Authenticatie vereist voor merklogo's

admin-settings-key-mail-compose-header-auto-collapse-seconds = Vertraging voor automatisch samenvouwen van koptekst van componist

admin-settings-key-mail-compose-priority-default = Standaard berichtprioriteit

admin-settings-key-mail-compose-priority-enabled = Prioriteitskiezer in de componist

admin-settings-key-mail-expiry-hide-expired = Verlopen berichten

admin-settings-key-mail-list-keep = Altijd bewaren

admin-settings-key-mail-list-page-size = Paginagrootte berichtenlijst

admin-settings-key-mail-list-unsubscribe-mode = Mailinglijst Afmelden

admin-settings-key-mail-message-list-group = Groepering van berichten

admin-settings-key-mail-priority-marks = Prioriteitsmerken

admin-settings-key-mail-priority-marks-style = Prioriteitsmarkeringsstijl

admin-settings-key-mail-priority-send-headers = Prioriteitsheaders geschreven bij verzending

admin-settings-key-mail-priority-trust-sender-headers = Vertrouw op de door de afzender ingestelde prioriteit

admin-settings-key-mail-reply-from-header-priority = Headers die de antwoordidentiteit kiezen

admin-settings-key-mail-search-default-scope = Standaard zoekbereik voor e-mail

admin-settings-key-metrics-dimensions-enabled = Statistieken opsplitsen per clienttype en apparaatklasse

admin-settings-key-metrics-persist-enabled = Houd statistieken bij over herstarts

admin-settings-key-metrics-persist-flush-seconds = Interval voor het opslaan van statistieken (seconden)

admin-settings-key-observability-client-capture-console = Consolefouten melden

admin-settings-key-observability-client-capture-csp = Rapporteer CSP overtredingen

admin-settings-key-observability-client-enabled = Browserfoutrapporten

admin-settings-key-observability-client-forward = Foutrapporten doorsturen

admin-settings-key-observability-client-include-account-hash = Voeg account-hash toe aan foutrapporten

admin-settings-key-observability-client-max-per-session = Foutrapporten per geladen pagina

admin-settings-key-observability-client-sample-rate = Bemonsteringsfrequentie foutrapport

admin-settings-key-observability-client-source-maps = Los stacktraces op met bronkaarten

admin-settings-key-privacy-error-reports = Foutrapporten verzenden

admin-settings-key-privacy-image-cache-max-stale-secs = Blijf oud als de oorsprong faalt (seconden)

admin-settings-key-privacy-image-cache-stale-secs = Serveer oud tijdens het revalideren (seconden)

admin-settings-key-privacy-image-resize-enabled = Formaat van grote afbeeldingen wijzigen

admin-settings-key-privacy-image-resize-format = Formaat van afbeelding aangepast

admin-settings-key-privacy-image-resize-max-pixels = Grootste afbeelding waarvan het formaat kan worden gewijzigd (pixels)

admin-settings-key-privacy-image-resize-max-width = Grootste afbeelding met gewijzigd formaat (pixels)

admin-settings-key-privacy-sessions-show-linked-origin = Gekoppelde mailboxoorsprong in sessies

admin-settings-key-properties-folder-enabled = Dialoogvenster Mapeigenschappen

admin-settings-key-properties-folder-notes-enabled = Mapnotities

admin-settings-key-properties-folder-notes-max-length = Lengte mapnotitie

admin-settings-key-properties-folder-overrides-allowed = Overschrijvingen per map toegestaan

admin-settings-key-properties-folder-sections = Mapdialoogsecties

admin-settings-key-properties-folder-size-compute = Berekening van de mapgrootte

admin-settings-key-properties-headers-raw = Ruwe kopteksten

admin-settings-key-properties-message-enabled = Dialoogvenster Berichteigenschappen

admin-settings-key-properties-message-permanent-delete = Permanente verwijdering uit eigenschappen

admin-settings-key-properties-message-sections = Berichtdialoogsecties

admin-settings-key-properties-offline-clear-enabled = Offline cache per map wissen

admin-settings-key-properties-remote-location-visibility = Detail van afgelegen locatie

admin-settings-key-properties-scoring-show = Score- en authenticatiedetails

admin-settings-key-properties-shortcut-enabled = Snelkoppeling Eigenschappen

admin-settings-key-properties-shortcut-key = Sneltoets Eigenschappen

admin-settings-key-reader-actions-archive = Plaatsing van archiefactie

admin-settings-key-reader-actions-block-sender = Blokkeer de plaatsing van afzenderacties

admin-settings-key-reader-actions-colors = Berichtkleuren actieplaatsing

admin-settings-key-reader-actions-conversation = Bekijk de plaatsing van gespreksacties

admin-settings-key-reader-actions-create-filter = Filteractieplaatsing maken

admin-settings-key-reader-actions-delete = Actieplaatsing verwijderen

admin-settings-key-reader-actions-delete-sender = Verwijder alle berichten uit de afzenderactie

admin-settings-key-reader-actions-edit-draft = Plaatsing van conceptactie bewerken

admin-settings-key-reader-actions-export = Actieplaatsing exporteren

admin-settings-key-reader-actions-finish-followup = Markeer de vervolgactie als voltooide actie

admin-settings-key-reader-actions-flag = Markeer/demarkeer actieplaatsing

admin-settings-key-reader-actions-focus-focused = Ga naar gerichte actieplaatsing

admin-settings-key-reader-actions-focus-other = Ga naar Andere actieplaatsing

admin-settings-key-reader-actions-forward = Plaatsing van voorwaartse actie

admin-settings-key-reader-actions-headers = Plaatsing van onbewerkte koptekstacties

admin-settings-key-reader-actions-links = Bekijk de plaatsing van de linksactie

admin-settings-key-reader-actions-move = Verplaats actieplaatsing

admin-settings-key-reader-actions-not-spam = Geen plaatsing van spamacties

admin-settings-key-reader-actions-open-draft = Open antwoordconceptactieplaatsing

admin-settings-key-reader-actions-pin = Actieplaatsing vastzetten/losmaken

admin-settings-key-reader-actions-print = Plaatsing van afdrukactie

admin-settings-key-reader-actions-properties = Berichteigenschappen actieplaatsing

admin-settings-key-reader-actions-read = Plaatsing van gelezen/ongelezen actie

admin-settings-key-reader-actions-reply = Plaatsing van antwoordactie

admin-settings-key-reader-actions-reply-all = Beantwoord alle actieplaatsingen

admin-settings-key-reader-actions-save-contacts = Bewaar de actieplaatsing van berichtcontacten

admin-settings-key-reader-actions-save-sender = Plaatsing van afzenderactie opslaan

admin-settings-key-reader-actions-search = Gerelateerde plaatsing van zoekacties

admin-settings-key-reader-actions-security = Plaatsing van actie in beveiligingsmodus

admin-settings-key-reader-actions-spam = Markeer de actieplaatsing als spam

admin-settings-key-reader-actions-sweep = Plaatsing van actie voor sweep-afzender

admin-settings-key-reader-actions-view-source = Bekijk de plaatsing van berichtbronacties

admin-settings-key-reader-actions-zoom = Plaatsing van zoomacties

admin-settings-key-retention-client-errors-event-days = Bijgehouden foutvoorvallen (dagen)

admin-settings-key-retention-client-errors-group-days = Foutgroepen bewaard (dagen)

admin-settings-key-retention-metrics-day-buckets-days = Dagelijkse statistieken bijgehouden (dagen)

admin-settings-key-retention-metrics-hour-buckets-days = Bijgehouden statistieken per uur (dagen)

admin-settings-key-retention-metrics-minute-buckets-hours = Bijgehouden minuutstatistieken (uren)

admin-settings-key-retention-user-allow-strip-attachments = Gebruikers kunnen bijlagen uit oude e-mail verwijderen

admin-settings-key-retention-user-calendar-enabled = Gebruikers kunnen beëindigde agenda-evenementen verwijderen

admin-settings-key-retention-user-calendar-max-days = De meeste dagen vóór het einde van de evenementen worden verwijderd

admin-settings-key-retention-user-calendar-min-days = De minste dagen vóór het einde van de evenementen worden verwijderd

admin-settings-key-retention-user-notes-enabled = Gebruikers kunnen onaangeroerde notities verwijderen

admin-settings-key-retention-user-notes-max-days = De meeste dagen voordat onaangeroerde aantekeningen worden verwijderd

admin-settings-key-retention-user-notes-min-days = Minste dagen voordat onaangeroerde bankbiljetten worden verwijderd

admin-settings-key-retention-user-strip-keep-inline-images = Bewaar inline afbeeldingen

admin-settings-key-retention-user-strip-min-age-days = Minste dagen voordat bijlagen worden verwijderd

admin-settings-key-retention-user-strip-min-attachment-kb = Kleinste verwijderde bijlage (KB)

admin-settings-key-retention-user-strip-undo-days = Venster voor ongedaan maken nadat bijlagen zijn verwijderd (dagen)

admin-settings-key-server-shutdown-grace-seconds = Uitstelperiode voor afsluiten

admin-settings-key-session-last-activity-flush-seconds = Laatst gezien schrijfinterval

admin-settings-key-session-persist-across-restarts = Behoud sessies tijdens herstarts
