# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: settings. Source: ../en/settings.ftl

settings-title = Instellingen
settings-appearance = Weergave
settings-close = Instellingen sluiten

settings-theme = Thema
settings-density = Dichtheid
settings-accent = Accentkleur
settings-font = Lettertype van de interface
settings-layout = Indeling

# Density options
settings-density-compact = Compact
settings-density-cozy = Comfortabel
settings-density-relaxed = Ruim

# Interface-font options
settings-font-default = Standaard
settings-font-system = Systeem
settings-font-serif = Serif
settings-font-mono = Mono

# Layout options
settings-layout-default = Standaard
settings-layout-ribbon = Lint

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Annuleren
settings-edit = Bewerken
settings-delete = Verwijderen
settings-save = Opslaan
settings-saved = Opgeslagen

# Two-factor authentication (S1)
settings-2fa-title = Tweestapsverificatie
settings-2fa-intro = Voeg een tweede factor toe, zodat bij het inloggen meer dan een wachtwoord nodig is.
settings-2fa-policy-required = Uw organisatie vereist een tweede factor voor dit account.
settings-2fa-enabled = Ingeschakeld
settings-2fa-recovery-title = Herstelcodes
settings-2fa-recovery-once = Bewaar deze nu. Ze worden één keer getoond en kunnen niet opnieuw worden opgevraagd. Elke code werkt één keer.
settings-2fa-recovery-ack = Ik heb deze codes bewaard
settings-2fa-totp-title = Authenticator-app
settings-2fa-totp-enrol = Een authenticator-app instellen
settings-2fa-totp-scan = Voeg dit geheim toe aan uw authenticator-app en voer daarna de code in die de app toont.
settings-2fa-totp-uri-link = Openen in een authenticator-app
settings-2fa-totp-code-label = Authenticatorcode
settings-2fa-totp-confirm = Bevestigen
settings-2fa-totp-disable = Authenticator verwijderen
settings-2fa-passkey-title = Passkeys
settings-2fa-passkey-remove = Verwijderen
settings-2fa-passkey-unsupported = Deze browser ondersteunt geen passkeys.
settings-2fa-passkey-label-placeholder = Naam van passkey (optioneel)
settings-2fa-passkey-add = Passkey toevoegen
settings-2fa-recovery-remaining = Nog { $count } { $count ->
        [one] herstelcode
       *[other] herstelcodes
    } over
settings-2fa-recovery-regenerate = Nieuwe herstelcodes aanmaken
settings-2fa-recovery-code-label = Herstelcode
settings-2fa-error-generic = Dat is niet gelukt. Probeer het opnieuw.
settings-2fa-error-code = Die code kon niet worden geverifieerd.
settings-2fa-error-passkey = De passkey kon niet worden geregistreerd.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Tweestapsverificatie
settings-2fa-challenge-intro = Bevestig uw tweede factor om het inloggen te voltooien.
settings-2fa-challenge-failed = Verificatie mislukt. Probeer het opnieuw.
settings-2fa-challenge-use-passkey = Passkey gebruiken
settings-2fa-challenge-method = Kies een methode
settings-2fa-challenge-totp-tab = Authenticatorcode
settings-2fa-challenge-recovery-tab = Herstelcode
settings-2fa-challenge-verify = Verifiëren

# Active sessions (S11)
settings-sessions-title = Actieve sessies
settings-sessions-intro = Sessies die momenteel op dit account zijn ingelogd.
settings-sessions-last-seen = Laatst actief { $when }
settings-sessions-revoke = Uitloggen
settings-sessions-current = Deze sessie
settings-sessions-revoke-others = { $count } { $count ->
        [one] andere sessie
       *[other] andere sessies
    } uitloggen
settings-sessions-error = De sessie kon niet worden gewijzigd.

# Signatures (W12)
settings-sig-title = Handtekeningen
settings-sig-intro = Tekst die wordt toegevoegd aan berichten die u verstuurt. Eén handtekening kan de standaard zijn.
settings-sig-default = Standaard
settings-sig-new = Nieuwe handtekening
settings-sig-name-label = Naam
settings-sig-body-label = Handtekening
settings-sig-default-label = Als standaard gebruiken
settings-sig-error-name = Geef de handtekening een naam.
settings-sig-error-generic = De handtekening kon niet worden opgeslagen.

# Identities
settings-ident-title = Identiteiten
settings-ident-intro = De naam en het adres waarmee u verzendt. Elke identiteit kan een eigen handtekening gebruiken.
settings-ident-new = Nieuwe identiteit
settings-ident-name-label = Weergavenaam
settings-ident-email-label = E-mailadres
settings-ident-replyto-label = Antwoordadres (optioneel)
settings-ident-signature-label = Standaardhandtekening
settings-ident-signature-none = Geen
settings-ident-error-fields = Voer een naam en een geldig e-mailadres in.
settings-ident-error-generic = De identiteit kon niet worden opgeslagen.

# Notification rules + quiet hours (W15)
settings-notif-title = Meldingen
settings-notif-intro = Kies over welke nieuwe berichten u een melding krijgt, en wanneer het stil moet blijven.
settings-notif-enabled-label = Meldingen tonen voor nieuwe mail
settings-notif-quiet-title = Stille uren
settings-notif-quiet-enabled-label = Meldingen dempen tijdens een ingestelde periode
settings-notif-quiet-start = Van
settings-notif-quiet-end = Tot
settings-notif-rules-title = Regels
settings-notif-rule-match-placeholder = Afzender, map of onderwerp bevat…
settings-notif-rule-action-label = Actie
settings-notif-rule-notify = Melden
settings-notif-rule-mute = Dempen
settings-notif-rule-add = Regel toevoegen
settings-notif-error = De meldingsinstellingen konden niet worden opgeslagen.

# Saved searches → search folders (W13)
settings-search-title = Opgeslagen zoekopdrachten
settings-search-intro = Toon een opgeslagen zoekopdracht als map in uw mappenlijst.
settings-search-empty = U hebt nog geen opgeslagen zoekopdrachten.
settings-search-as-folder = { $name } als map tonen
settings-search-as-folder-label = Als map tonen
settings-search-error = De opgeslagen zoekopdracht kon niet worden gewijzigd.

# Keyboard shortcuts (W14)
settings-kbd-title = Sneltoetsen
settings-kbd-intro = Kies een set sneltoetsen. Wijzigingen gelden op dit apparaat.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Uit (geen sneltoetsen)
settings-kbd-action-compose = Opstellen
settings-kbd-action-archive = Archiveren
settings-kbd-action-reply = Beantwoorden
settings-kbd-action-next = Volgend bericht
settings-kbd-action-previous = Vorig bericht
settings-kbd-action-search = Zoeken

# Offline cache (W16)
settings-offline-title = Offline cache
settings-offline-intro = Hoeveel mail er op dit apparaat wordt bewaard voor offline gebruik, en hoe ruimte wordt vrijgemaakt.
settings-offline-budget-label = Cachelimiet (MB)
settings-offline-retention-label = Bewaren gedurende (dagen)
settings-offline-strategy-label = Wanneer de limiet is bereikt
settings-offline-lru = Minst recent gebruikte verwijderen
settings-offline-oldest = Oudste eerst verwijderen
settings-offline-manual = Alleen als ik de cache leegmaak
settings-offline-purge = Offline cache nu leegmaken

# Interface direction (W20)
settings-dir-title = Tekstrichting van de interface
settings-dir-intro = Volg de taal, of stel een richting vast. Van rechts naar links spiegelt de indeling.
settings-dir-auto = Automatisch
settings-dir-ltr = Van links naar rechts
settings-dir-rtl = Van rechts naar links
settings-dir-preview-title = Voorbeeld
settings-dir-preview-body = Deze tekst wordt gespiegeld als de richting van rechts naar links is.
settings-mail-features = E-mailfuncties
settings-mail-feature-allowSend = Berichten verzenden toestaan
settings-mail-feature-allowSend-help = Als dit uit staat, zijn Verzenden en de sneltoets uitgeschakeld. Concepten kunnen nog steeds worden opgeslagen.
settings-mail-feature-allowReply = Beantwoorden en Allen beantwoorden toestaan
settings-mail-feature-allowReply-help = Als dit uit staat, worden de acties Beantwoorden en Allen beantwoorden en hun sneltoetsen niet aangeboden.
settings-mail-feature-allowForward = Doorsturen toestaan
settings-mail-feature-allowForward-help = Als dit uit staat, worden de acties Doorsturen en de sneltoets niet aangeboden.
settings-mail-feature-allowDownload = Berichten en bijlagen downloaden toestaan
settings-mail-feature-allowDownload-help = Als dit uit staat, worden downloads van berichten en bijlagen niet aangeboden.
settings-mail-feature-allowPrint = Afdrukken toestaan
settings-mail-feature-allowPrint-help = Als dit uit staat, wordt de actie Afdrukken niet aangeboden.
settings-mail-feature-showSpam = De map Spam tonen
settings-mail-feature-showSpam-help = Als dit uit staat, zijn de map Spam en berichten die als spam zijn gemarkeerd verborgen in de maplijst en in zoekresultaten. Er wordt niets verwijderd.
settings-mail-feature-showAuthStatus = Verificatieresultaten van de afzender tonen
settings-mail-feature-showAuthStatus-help = Toont de SPF-, DKIM- en DMARC-resultaten van een bericht in de leesweergave.
settings-mail-feature-closeOnDelete = Een bericht sluiten wanneer het wordt verwijderd
settings-mail-feature-closeOnDelete-help = Sluit het geopende bericht, tabblad of venster wanneer dat bericht wordt verwijderd of naar de prullenbak wordt verplaatst.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Pad in de instellingen
settings-crumb-loading = Naam laden
settings-crumb-new = Nieuw
settings-crumb-edit = Bewerken
settings-badge-color = Kleur van de ongelezen-badge
settings-badge-color-accent = Zelfde als accentkleur
settings-badge-color-accent-lighter = Lichtere accentkleur
settings-badge-color-accent-darker = Donkerdere accentkleur
settings-badge-color-option = { $color } badges
settings-appearance-theme-card = Thema en wisselen
settings-appearance-theme-card-help = Kies een thema en of het het systeem of het tijdstip van de dag volgt.
settings-appearance-colours-card = Kleuren
settings-appearance-colours-card-help = De accentkleur voor acties en selectie, en de kleur van de ongelezen-tellers.
settings-appearance-text-card = Tekst en indeling
settings-appearance-text-card-help = Hoe compact de interface is, welk lettertype wordt gebruikt en hoe mail wordt weergegeven.
settings-sig-create-variant = Variant maken
settings-sig-error-existing = Er bestaat al een handtekening met deze naam. Kies een andere naam.
settings-sig-save-failed = De handtekening kon niet worden opgeslagen. { $reason }
settings-ident-signature-new = Nieuwe berichten
settings-ident-signature-account = Accounthandtekening gebruiken
settings-ident-signature-replies = Antwoorden
settings-ident-signature-inherit = Handtekening voor nieuwe berichten gebruiken
settings-ident-signature-unavailable = Handtekening niet beschikbaar: { $name }
settings-ident-signature-help = Maak handtekeningvarianten bij Handtekeningen en wijs er hier een toe. Dezelfde variant kan voor beide worden gebruikt.
settings-ident-save-failed = De identiteit kon niet worden opgeslagen. { $reason }
settings-search-save-failed = De opgeslagen zoekopdracht kon niet worden opgeslagen. { $reason }
settings-kbd-preset-title = Sneltoetsenset
settings-editor-discard-title = Niet-opgeslagen wijzigingen negeren?
settings-editor-discard-detail = Uw wijzigingen zijn niet opgeslagen. Blijf bewerken of negeer dit concept om de editor te verlaten.
settings-editor-keep = Blijven bewerken
settings-editor-discard = Wijzigingen negeren
settings-editor-missing = Dit item bestaat niet meer. Ga terug naar de lijst en kies een ander item.
settings-editor-back = Terug naar de lijst
settings-sig-edit-title = Handtekening bewerken
settings-ident-edit-title = Identiteit bewerken
settings-sections = Instellingenonderdelen
settings-category-account = Account
settings-category-mail = Mail
settings-category-interface = Interface
settings-category-integrations = Integraties
settings-category-about = Over
settings-nav-preferences = Toetsenbord en weergave
settings-nav-rules = Mailregels
settings-nav-access = API-toegang
settings-nav-metadata = Serverinformatie
settings-folders-title = Mappen en zijbalk
settings-folders-help = Zet mappen boven de andere vast, kies een pictogram en sleep of gebruik de verplaatsknoppen om mappen binnen elke groep te ordenen. Deze keuzes worden met dit account gesynchroniseerd en verplaatsen geen berichten en hernoemen geen mappen.
settings-folders-pin = { $name } vastzetten
settings-folders-icon = Pictogram voor { $name }
settings-folders-automatic = Automatische maprol
settings-folders-drag = Sleep { $name } om te herschikken
settings-folders-up = { $name } omhoog verplaatsen
settings-folders-down = { $name } omlaag verplaatsen
settings-folders-sync-idle = Log in om de mapvoorkeuren te synchroniseren.
settings-folders-sync-loading = Accountvoorkeuren laden…
settings-folders-sync-saving = Accountvoorkeuren opslaan…
settings-folders-sync-synced = Voorkeuren gesynchroniseerd met dit account.
settings-folders-sync-error = De voorkeuren konden niet worden gesynchroniseerd. Uw wijzigingen blijven in deze sessie en worden opnieuw geprobeerd zodra de verbinding terug is.
settings-folder-icon-folder = Map
settings-folder-icon-star = Ster
settings-folder-icon-flag = Vlag
settings-folder-icon-tag = Label
settings-folder-icon-clock = Klok
settings-folder-icon-calendar = Agenda
settings-folder-icon-file = Document
settings-folder-icon-image = Afbeelding
settings-folder-icon-lock = Slot
settings-folder-icon-users = Mensen
settings-folder-icon-archive = Archief
settings-folder-icon-mail = Envelop
settings-folder-icon-attachment = Bijlage
settings-folder-icon-check = Vinkje
settings-folder-icon-alert = Waarschuwing
settings-search-new = Nieuwe zoekmap
settings-search-editor = Zoekmap
settings-search-editor-help = Berichten moeten aan alle voorwaarden voldoen. Laat een voorwaarde leeg om die te negeren. Zoekmappen tonen overeenkomende berichten zonder ze te verplaatsen of te kopiëren.
settings-search-name = Mapnaam
settings-search-name-required = Voer een naam in voor deze zoekmap.
settings-search-invalid = Deze opgeslagen zoekopdracht heeft een ongeldig filter. Ze kan niet veilig worden bewerkt.
settings-search-saved = Zoekmap “{ $name }” opgeslagen.
settings-search-field-text = Alle berichttekst doorzoeken
settings-search-field-from = Afzender bevat
settings-search-field-to = Ontvanger bevat
settings-search-field-subject = Onderwerp bevat
settings-search-field-body = Berichttekst bevat
settings-search-field-filename = Bestandsnaam van bijlage bevat
settings-search-field-inMailbox = Beperken tot mailbox-identificatie (optioneel)
settings-search-field-after = Ontvangen na (UTC-datum en -tijd)
settings-search-field-before = Ontvangen voor (UTC-datum en -tijd)
settings-search-attachment = Bijlagen
settings-search-any = Elk bericht
settings-search-with-attachments = Met bijlagen
settings-search-without-attachments = Zonder bijlagen
settings-search-unread = Alleen ongelezen berichten
settings-search-flagged = Alleen berichten met vlag
settings-sessions-search = Account, apparaat of IP zoeken
settings-sessions-account = Account
settings-sessions-device = Apparaat en client
settings-sessions-location = IP en laatst bekende land
settings-sessions-location-unavailable = Land niet beschikbaar
settings-sessions-activity = Activiteit
settings-sessions-actions = Acties
settings-sessions-revoke-device = { $device } afmelden
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Opstellen en antwoorden
settings-quote-style = Kopstijl voor antwoorden en doorsturen
settings-quote-classic = Klassiek geciteerd antwoord
settings-quote-outlook = Kop in Outlook-stijl
settings-quote-help = De Outlook-stijl voegt Van, Verzonden, Aan, Cc en Onderwerp toe boven het oorspronkelijke bericht, in opgemaakte tekst en platte tekst. Bestaande concepten blijven ongewijzigd.
settings-external-warning = Waarschuwingen voor externe ontvangers
settings-external-off = Uit
settings-external-warn = Een waarschuwing tonen
settings-external-confirm = Bevestiging vereisen vóór het verzenden
settings-external-help = Vergelijkt Aan, Cc en Bcc met het domein van de geselecteerde identiteit en de lijst met vertrouwde domeinen. Dit beschermt verzendingen vanuit deze editor; het is geen afleverregel van de server.
settings-trusted-domains = Vertrouwde ontvangersdomeinen
settings-trusted-domains-help = Voer maximaal 100 exacte domeinen in, één per regel. Subdomeinen moeten apart worden vermeld. Het eigen domein van de verzendende identiteit is altijd vertrouwd. URL's, e-mailadressen en jokertekens worden niet geaccepteerd.
settings-trusted-domains-invalid = Voer maximaal 100 geldige domeinnamen in, zonder URL's, e-mailadressen of jokertekens.
settings-trusted-domains-save = Vertrouwde domeinen opslaan
settings-shortcuts-enabled = Toepassingssneltoetsen inschakelen
settings-shortcuts-action = Actie
settings-shortcuts-binding = Toewijzing
settings-shortcuts-edit = Sneltoets bewerken
settings-shortcuts-edit-action = Sneltoets voor { $action } bewerken
settings-shortcuts-help = Gebruik inherit voor de geselecteerde set, disabled voor geen sneltoets, of een combinatie zoals Ctrl+Shift+K. Ctrl betekent Cmd op macOS. Conflicterende toewijzingen zijn inactief. Invoervelden en dialogen behouden hun gebruikelijke toetsen; door de browser gereserveerde combinaties zijn mogelijk niet beschikbaar.
settings-shortcuts-invalid = Voer inherit, disabled of een geldige enkele toets in, met optioneel de modifiers Ctrl, Alt en Shift.
settings-shortcuts-conflict = Twee acties gebruiken dezelfde sneltoets. Wijzig een van beide toewijzingen; conflicterende sneltoetsen worden niet uitgevoerd.
settings-shortcuts-discard = De niet-opgeslagen sneltoetswijziging negeren?
settings-shortcuts-save = Sneltoets opslaan
settings-shortcuts-back = Terug naar sneltoetsen
settings-shortcuts-off = De set Uit schakelt alle sneltoetsen uit, ook aangepaste toewijzingen. Kies een andere set in Voorkeuren om deze toewijzingen te gebruiken.
settings-kbd-action-replyAll = Allen beantwoorden
settings-kbd-action-forward = Doorsturen
settings-kbd-action-refresh = Berichten vernieuwen
settings-kbd-action-settings = Instellingen openen
settings-kbd-action-trash = Naar de prullenbak verplaatsen
settings-kbd-action-markRead = Als gelezen markeren

# -- Machine translation (2026-11): added to match en ---------------------
settings-kbd-action-markUnread = Als ongelezen markeren
settings-kbd-action-spam = Als spam markeren
settings-kbd-action-zoomIn = Bericht inzoomen
settings-kbd-action-zoomOut = Bericht uitzoomen
settings-kbd-action-zoomReset = Berichtzoom herstellen
settings-kbd-action-renameFolder = Map hernoemen
settings-kbd-action-openInTab = Bericht in een eigen tabblad openen
settings-notif-device-title = Toestemming op dit apparaat
settings-notif-device-help = Uw meldingsvoorkeuren volgen uw account. De toestemming van de browser hoort bij dit apparaat; Mailwoman vraagt er alleen om wanneer u ervoor kiest die toe te staan.
settings-notif-ask-permission = Toestemming voor meldingen aanbieden op nieuwe apparaten
settings-notif-ask-permission-help = Toon een toestemmingsvraag wanneer dit apparaat meldingen nog niet heeft toegestaan of geblokkeerd. Standaard ingeschakeld; deze voorkeur wordt direct opgeslagen.
settings-notif-permission-default = Er is nog geen browsertoestemming gevraagd.
settings-notif-permission-granted = Browsermeldingen zijn toegestaan op dit apparaat.
settings-notif-permission-denied = Meldingen zijn geblokkeerd. Wijzig de toestemming van deze site in uw browserinstellingen om ze toe te staan.
settings-notif-permission-unsupported = Deze browser ondersteunt hier geen systeemmeldingen.
settings-notif-permission-request = Meldingen toestaan op dit apparaat
settings-notif-permission-error = De browser kon geen toestemming voor meldingen vragen.
settings-notif-timezone = Tijdzone voor stille uren (IANA)
settings-notif-timezone-device = De tijdzone van dit apparaat gebruiken
settings-permissions-background-title = Achtergrondmeldingen in deze browser
settings-permissions-background-help = Sla eerst de meldingsvoorkeuren op en sta browsermeldingen toe. Registreer deze browser om accountactiviteit te ontvangen wanneer de app is gesloten.
settings-permissions-push-unsupported = Web Push op de achtergrond is niet beschikbaar in deze browser.
settings-permissions-push-registered = Deze browser heeft een pushabonnement. Registreer het om het aan het huidige account te koppelen.
settings-permissions-push-unregistered = Deze browser heeft geen pushabonnement.
settings-permissions-push-register = Achtergrondmeldingen registreren
settings-permissions-push-remove = Browserabonnement verwijderen
settings-permissions-push-policy = Uw beheerder staat geen pushmeldingen op de achtergrond toe.
settings-permissions-storage-title = Browseropslag
settings-permissions-storage-help = Permanente opslag beschermt de lokale gegevens van deze site tegen automatische verwijdering wanneer de browser dit toekent. Dit geldt alleen voor dit apparaat en wijzigt de bewaartermijn op de server niet. U kunt sitegegevens wissen in de browserinstellingen; niet-verzonden offline wijzigingen kunnen verloren gaan.
settings-permissions-storage-unsupported = Verzoeken om permanente opslag zijn niet beschikbaar in deze browser.
settings-permissions-storage-persistent = De browser heeft permanente opslag voor deze site toegekend.
settings-permissions-storage-evictable = De browser kan de lokale gegevens van deze site automatisch verwijderen.
settings-permissions-storage-usage = Huidige opslag van de site: { $mib } MiB.
settings-permissions-storage-request = Permanente opslag aanvragen
settings-permissions-storage-policy = Uw beheerder staat geen verzoeken om permanente opslag toe.
settings-permissions-error = De browseractie kon niet worden voltooid. Controleer de opgeslagen accountinstellingen en de browsertoestemmingen van deze site.
settings-notif-background-rules = Inhoudsregels worden beoordeeld terwijl Mailwoman open is. Systeemmeldingen op de achtergrond worden onderdrukt zolang er inhoudsregels zijn ingesteld, omdat een ondoorzichtige achtergrondwake geen berichtdetails bevat.
settings-nav-regional = Regionale instellingen
settings-time-title = Tijdzones
settings-time-help = Kies hoe berichttijden worden weergegeven en eventueel een vergelijkingstijdzone voor het plannen. Locaties worden nooit afgeleid uit e-mailadressen.
settings-time-displayZone = Weergavetijdzone
settings-time-displayZone-help = Gebruik local voor dit apparaat, of een IANA-naam zoals Europe/Lisbon.
settings-time-comparisonZone = Vergelijkingstijdzone
settings-time-comparisonZone-help = Voer een IANA-naam in zoals America/New_York, of laat dit leeg om vergelijkingen te verbergen.
settings-time-helpers = Tijdzonehulpen tonen
settings-time-save = Tijdzone opslaan
settings-time-invalid = Voer een geldige IANA-tijdzone in. Gebruik local alleen voor de weergavetijdzone, of laat de vergelijkingstijdzone leeg.
settings-time-discard = Niet-opgeslagen tijdzonewijzigingen negeren?
settings-folders-name = Map
settings-folders-actions = Acties
settings-folders-edit = Weergave van { $name } bewerken
settings-folders-icon-help = Kies een van vijftig thematische pictogrammen, of laat de automatische keuze op basis van het doel van de map staan. Dit volgt uw account en hernoemt de mailbox niet.
settings-folders-save = Mapweergave opslaan
settings-folders-back = Terug naar mappen
settings-folders-discard = Niet-opgeslagen wijzigingen aan de mapweergave negeren?
settings-folder-labels = Maplabels
settings-folder-labels-help = Orden deze map met maximaal 32 accountlabels. Dit labelt of verplaatst de berichten erin niet. Beheer labelnamen en -kleuren bij Labels.
settings-folders-leave-pending = Deze wijzigingen staan nog in de wachtrij voor synchronisatie. Deze editor verlaten?
settings-folders-retry = Opnieuw proberen op te slaan
settings-folder-role-title = Speciale mappen
settings-folder-role-help = Kies de echte map die voor elke mailrol wordt gebruikt. Automatisch gebruikt serverdetectie. Beheerderstandaard gebruiken verwijdert uw overschrijving. Conflicterende of ontbrekende bestemmingen schakelen die rol uit in plaats van een andere map te kiezen.
settings-folder-role-auto = Automatische detectie
settings-folder-role-inherit = Beheerderstandaard gebruiken
settings-folder-role-path = Ingesteld pad: { $path }
settings-folder-role-missing = De ingestelde map is niet beschikbaar
settings-folder-role-conflict = Deze bestemming ontbreekt of botst met een andere rol. Die rol is niet beschikbaar totdat de toewijzing is hersteld.
settings-folder-role-refresh-error = Opgeslagen, maar de maplijst kon niet worden vernieuwd. Vernieuw de mappen voordat u de nieuwe toewijzing gebruikt.
settings-folder-role-unsupported = Deze overschrijvingen vereisen eigen mailmappen. Upstream JMAP-accounts behouden de maprollen van hun server.
settings-folder-role-inbox = Inbox
settings-folder-role-drafts = Concepten
settings-folder-role-sent = Verzonden mail
settings-folder-role-trash = Prullenbak
settings-folder-role-junk = Ongewenste mail en spam
settings-folder-role-archive = Archief
settings-sessions-select = Selecteren
settings-sessions-select-device = Sessie voor { $device } selecteren
settings-sessions-select-visible = Alle andere zichtbare sessies selecteren
settings-sessions-current-protected = Deze sessie blijft ingelogd en kan hier niet worden geselecteerd. Gebruik Uitloggen om haar te beëindigen.
settings-sessions-selected-title = Geselecteerde sessies afmelden ({ $count })
settings-sessions-selected-confirm = Beëindigt alleen de geselecteerde andere sessies. Deze sessie blijft ingelogd. Mislukte of overgeslagen verzoeken worden gemeld, zonder automatische nieuwe pogingen.
settings-sessions-selected-action = Geselecteerde sessies afmelden
settings-sessions-cancel = Annuleren
settings-sessions-selected-result = Afgemeld: { $revoked }. Al beëindigd: { $unchanged }. Kon niet worden bevestigd: { $failed }. Niet geprobeerd: { $skipped }.
settings-sessions-refresh-failed = De sessielijst kon niet worden vernieuwd. Laad haar opnieuw voordat u meer sessies selecteert.
settings-sig-format = Handtekeningindeling
settings-sig-format-plain = Platte tekst
settings-sig-format-html = Opgemaakte tekst (HTML)
settings-sig-html-help = HTML is beperkt tot 2 MiB en wordt gezuiverd vóór het opslaan en verzenden. Externe afbeeldingen en actieve inhoud worden verwijderd.
settings-sig-html-source = HTML-broncode bewerken
settings-sig-editor-loading = Handtekeningeditor laden…
settings-sig-rich-body = Opgemaakte handtekening
settings-sig-plain-alternative = Alternatief in platte tekst
settings-sig-plain-help = Berichten in platte tekst gebruiken deze tekst. Bewerkingen in opgemaakte tekst werken hem bij totdat u hem aanpast; bewerkingen van de HTML-broncode laten uw alternatief ongemoeid.
settings-sig-account-changed = Het account waarmee u bent ingelogd is gewijzigd. Open deze handtekening opnieuw in het juiste account.
settings-sig-image-type = { $name } is geen PNG-, JPEG-, GIF- of WebP-afbeelding.
settings-sig-image-size = { $name } past niet: de HTML van een handtekening, afbeeldingen inbegrepen, is beperkt tot 2 MiB.
settings-sig-too-large = Deze handtekening is meer dan 2 MiB aan HTML. Verwijder een afbeelding of een deel van de opmaak.
settings-sig-images-help = Afbeeldingen worden in de handtekening opgeslagen en als inlineafbeeldingen met elk bericht meegestuurd.
settings-folder-icon-inbox = Inbox
settings-folder-icon-sent = Verzonden
settings-folder-icon-edit = Bewerken
settings-folder-icon-delete = Verwijderen
settings-folder-icon-spam = Spam
settings-folder-icon-search = Zoeken
settings-folder-icon-pin = Speld
settings-folder-icon-link = Link
settings-folder-icon-shield = Schild
settings-folder-icon-unlock = Open slot
settings-folder-icon-fileAudio = Audio
settings-folder-icon-fileVideo = Video
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Code
settings-folder-icon-print = Afdrukken
settings-folder-icon-download = Download
settings-folder-icon-preferences = Voorkeuren
settings-folder-icon-dashboard = Dashboard
settings-folder-icon-analytics = Analyses
settings-folder-icon-nextcloud = Cloudbestanden
settings-folder-icon-domains = Domeinen
settings-folder-icon-twofactor = Authenticatie
settings-folder-icon-assistant = Assistent
settings-folder-icon-integrations = Integraties
settings-folder-icon-logs = Logboeken
settings-folder-icon-home = Start
settings-folder-icon-briefcase = Aktetas
settings-folder-icon-heart = Hart
settings-folder-icon-shopping = Winkelen
settings-folder-icon-education = Onderwijs
settings-folder-icon-travel = Reizen
settings-folder-icon-coffee = Koffie
settings-folder-icon-gift = Cadeau
settings-folder-icon-bank = Bank
settings-folder-icon-book = Boek
settings-nav-keyboard-display = Toetsenbord en weergave
settings-2fa-passkey-policy-disabled = Uw beheerder heeft inloggen en registreren met passkeys voor dit account uitgeschakeld. Bestaande inloggegevens blijven vermeld, zodat u ze kunt beheren of verwijderen.
settings-mail-interactions = Berichten selecteren en verplaatsen
settings-mail-modifier-selection = Berichten selecteren met modifiertoetsen
settings-mail-modifier-selection-help = Ctrl of Command schakelt afzonderlijke berichten om. Shift selecteert een bereik en Shift met de pijltoetsen breidt het uit. De zichtbare selectieknop blijft beschikbaar wanneer dit uit staat.
settings-mail-drag-folder = Berichten naar mappen slepen
settings-mail-drag-folder-help = Sleep een geselecteerd bericht om de selectie te verplaatsen, of een niet-geselecteerd bericht om alleen dat bericht te verplaatsen. Alleen mappen in dit account worden geaccepteerd. Gebruikers van touch of toetsenbord kunnen de selectiewerkbalk gebruiken.
settings-search-sections = Instellingen doorzoeken
settings-search-no-sections = Geen overeenkomende instellingenonderdelen.
settings-time-local = Lokale tijdzone van het apparaat
settings-time-none = Geen vergelijkingstijdzone
settings-folder-counts = Maptellers
settings-folder-counts-none = Tellers verbergen
settings-folder-counts-unread = Ongelezen berichten
settings-folder-counts-total = Totaal aantal berichten
settings-folder-counts-both = Ongelezen en totaal aantal berichten
settings-folder-counts-unreadIfAny = Ongelezen berichten, alleen als die er zijn
settings-folder-count-scope = Mappen met tellers
settings-folder-count-scope-all = Alle mappen
settings-folder-count-scope-main = Alleen hoofdmappen
settings-folder-count-scope-help = Hoofdmappen zijn Inbox, Concepten, Verzonden, Archief, Ongewenste mail, Prullenbak, Postvak UIT en andere mappen voor speciaal gebruik. Aangepaste mappen en hun submappen tonen geen teller.
settings-account-unread-badge = Aantal ongelezen berichten naast de mailboxnaam tonen
settings-folder-counts-compact = Kleinere badges buiten de Inbox
settings-focused-inbox = Gerichte inbox
settings-focused-inbox-help = Splits de Inbox in de tabbladen Gericht en Overig boven de lijst. Berichten van nieuwsbrieven en geautomatiseerde afzenders gaan naar Overig. Andere mappen en zoekresultaten tonen alle berichten.
settings-focused-senders-title = Afzenders voor Gericht beheren
settings-focused-senders-help = Afzenders die u altijd naar Gericht of Overig wilde laten verplaatsen. Verwijder er een om de automatische sortering weer te laten beslissen.
settings-focused-senders-empty = Nog geen afzenders. Gebruik Altijd naar Gericht verplaatsen of Altijd naar Overig verplaatsen bij een bericht.

# -- Machine translation (2026-11): added to match en ---------------------
settings-focused-senders-to-focused = Altijd Gericht
settings-focused-senders-to-other = Altijd Overig
settings-focused-senders-remove = Verwijderen
settings-focused-senders-remove-label = { $sender } verwijderen
settings-focused-senders-remove-failed = De afzender kon niet worden verwijderd. Probeer het opnieuw.
settings-folder-counts-reset = Maptellers overnemen
settings-folder-counts-reset-size = Badgegrootte overnemen
settings-mail-resume-tabs = Mijn geopende tabbladen hervatten
settings-mail-resume-tabs-help = Herstelt de tabbladverwijzingen van het account nadat Mailwoman opnieuw is geopend. Berichtteksten en URL's van bijlagen worden niet opgeslagen in het werkruimtebestand.
settings-row-actions-mode = Snelle acties in de berichtenlijst
settings-row-actions-mode-open = Altijd geopend
settings-row-actions-mode-hover = Tonen bij aanwijzen
settings-row-actions-mode-click = Tonen bij klikken
settings-row-actions-mode-off = Uit
settings-row-actions-mode-help = Hoe de snelle acties op elke berichtrij verschijnen. Bij aanwijzen ziet u drie knoppen en een knop Meer; bij klikken ziet u één knop die ze opent; op touchscreens werkt aanwijzen als klikken. Uit verwijdert ze uit de rijen (het contextmenu en de werkbalk hebben nog steeds alle acties).
settings-mail-feature-showGoToTop = Een knop Naar boven tonen in de berichtenlijst
settings-mail-feature-showGoToTop-help = Voegt een knop toe die de berichtenlijst terug naar het eerste bericht scrolt. Standaard uit.
settings-mail-feature-showCreateFolderButton = Een knop Map maken tonen naast Opstellen
settings-mail-feature-showCreateFolderButton-help = Voegt in de zijbalk een knop met alleen een pictogram toe waarmee een map wordt gemaakt. Standaard uit.
settings-draft-save = Accountconcepten
settings-draft-save-help = Sla op in de map Concepten van uw account om ze op een ander apparaat te openen. Opslaan gebeurt stil; alleen fouten tonen een melding. Niet-ondersteunde accounts en beschermde ontsleutelde inhoud worden nooit stilzwijgend geüpload.
settings-draft-save-mode = Automatisch opslaan
settings-draft-realtime = Na een pauze in het typen
settings-draft-periodic = Met regelmatige tussenpozen
settings-draft-manual = Alleen wanneer ik Opslaan kies
settings-draft-debounce = Typpauze (milliseconden)
settings-draft-interval = Opslaginterval (seconden)
settings-permissions-push-other-registration = Deze browser heeft een abonnement dat niet op dit account is geregistreerd. Verwijder het voordat u opnieuw registreert.
settings-show-from = Van tonen bij het opstellen
settings-show-from-help = Begin met het veld Van zichtbaar. U kunt het ook voor één bericht tonen via het lint. Alleen geautoriseerde verzendadressen worden geaccepteerd.
settings-sessions-table = Tabel met actieve sessies
settings-folders-table = Maplijst
settings-kbd-reference = Toewijzingen van de set bekijken
settings-kbd-reference-help = Alleen de referentie van de set. Aangepaste toewijzingen en het beheerdersbeleid kunnen de actieve sneltoetsen wijzigen.
settings-sessions-client-details = Clientgegevens
settings-2fa-passkey-rules = Uw organisatie accepteert alleen passkeys die:
settings-2fa-passkey-rule-platform = op dit apparaat zijn opgeslagen (Windows Hello, Touch ID, schermvergrendeling van Android)
settings-2fa-passkey-rule-cross-platform = op een beveiligingssleutel of een telefoon staan
settings-2fa-passkey-rule-user-verification = uw pincode, vingerafdruk of gezicht controleren
settings-2fa-passkey-rule-resident-key = op de authenticator zijn opgeslagen (vindbaar)
settings-2fa-passkey-gap-allowed = Toegevoegd vóór het huidige beleid en voldoet er niet aan. Werkt nog steeds; voeg een nieuwe passkey toe en verwijder deze.
settings-2fa-passkey-gap-refused = Voldoet niet aan het huidige beleid en wordt niet meer geaccepteerd om in te loggen. Voeg een nieuwe passkey toe en verwijder deze.
settings-2fa-passkey-browser-refused-policy = Uw browser vond geen authenticator die voldoet aan het passkeybeleid van uw organisatie, of het verzoek is geannuleerd.
settings-2fa-policy-user-verification = Uw organisatie vereist een passkey die uw pincode, vingerafdruk of gezicht controleert. Deze authenticator heeft u niet geverifieerd.
settings-2fa-policy-platform = Uw organisatie accepteert alleen passkeys die op dit apparaat zijn opgeslagen (zoals Windows Hello, Touch ID of schermvergrendeling van Android). Beveiligingssleutels en telefoons worden niet geaccepteerd.
settings-2fa-policy-cross-platform = Uw organisatie accepteert alleen beveiligingssleutels of passkeys op een telefoon. Passkeys die op dit apparaat zijn opgeslagen, worden niet geaccepteerd.
settings-2fa-policy-attachment-unknown = Uw browser heeft niet aangegeven wat voor soort authenticator dit is, dus die kan niet worden getoetst aan het passkeybeleid van uw organisatie. Probeer een andere browser.
settings-2fa-policy-resident-key = Uw organisatie vereist een passkey die op de authenticator is opgeslagen. Deze authenticator heeft er geen opgeslagen.
settings-2fa-policy-legacy = Deze passkey is toegevoegd vóór het huidige passkeybeleid van uw organisatie en voldoet er niet meer aan. Log in met een andere factor of een herstelcode en voeg daarna een nieuwe passkey toe.
settings-2fa-policy-last-factor-blocked = Uw organisatie staat niet toe dat u uw laatste tweede factor verwijdert. Voeg eerst een andere inlogmethode toe.
settings-2fa-policy-last-factor-required = Uw organisatie vereist tweefactorauthenticatie. Voeg een andere inlogmethode toe voordat u deze verwijdert.
settings-2fa-last-factor-hint = Voeg eerst een andere inlogmethode toe.
settings-2fa-last-factor-tooltip = Voeg eerst een andere inlogmethode toe. Uw organisatie vereist ten minste één passkey of authenticator-app op uw account.
settings-2fa-challenge-passkey-not-used = De passkey is niet gebruikt: het verzoek is geannuleerd of verlopen, of deze authenticator kan niet doen wat uw organisatie vereist (zoals een controle van pincode, vingerafdruk of gezicht). Probeer het opnieuw of gebruik een andere methode.
settings-folders-hidden-help = Verborgen mappen en hun submappen blijven buiten de zijbalk. Ze worden niet verwijderd en de toegang wordt niet beperkt. Toon hier een submap om ook de verborgen bovenliggende mappen te tonen. De Inbox blijft altijd beschikbaar.
settings-folders-collapse-user = Gebruikersmappen in de zijbalk samenvouwen
settings-folders-show = { $name } in de zijbalk tonen
settings-folders-hide = { $name } in de zijbalk verbergen
settings-folders-show-short = Tonen
settings-folders-hide-short = Verbergen
settings-folders-search = Mappen zoeken
settings-folders-select = Map selecteren: { $name }
settings-folders-selected = Geselecteerde mappen: { $count }
settings-folders-select-visible = Zichtbare selecteren
settings-folders-select-all = Alle mappen selecteren
settings-folders-clear = Selectie wissen
settings-folders-pin-selected = Geselecteerde vastzetten
settings-folders-unpin-selected = Geselecteerde losmaken
settings-folders-show-selected = Geselecteerde tonen
settings-folders-hide-selected = Geselecteerde verbergen
settings-folders-visible = Getoond
settings-folders-hidden = Verborgen
settings-folders-pinned = Vastgezet
settings-folders-location = Locatie
settings-folders-counts = { $unread } ongelezen / { $total } totaal
settings-folders-more = Meer acties voor { $name }
settings-folders-empty = Geen mappen komen overeen met uw zoekopdracht.
settings-folders-root = Hoogste niveau
settings-folders-order-help = Sleep mappen om ze in de zijbalk te herschikken. Vastgezette mappen blijven bij elkaar.
cache-title = Mailcache en vooraf laden
cache-help = Bewaar recent bezochte mappen en berichten in het geheugen van dit tabblad. Gecachte inhoud wordt op de achtergrond vernieuwd. Het tabblad sluiten of uitloggen wist de cache. Vooraf laden markeert mail nooit als gelezen en laadt geen externe afbeeldingen.
cache-enabled = Sessiecache inschakelen
cache-mailboxes = Berichtenlijsten van mappen cachen
cache-messages = Geopende berichten cachen
cache-freshSeconds = Versheidsinterval (seconden)
cache-maxAgeSeconds = Maximale leeftijd in de cache (seconden)
cache-maxEntries = Maximaal aantal items in de cache
cache-maxBytes = Geheugenbudget (bytes)
cache-prefetchEnabled = Speculatief vooraf laden inschakelen
cache-hoverEnabled = Vooraf laden bij het aanwijzen van een map
cache-focusEnabled = Vooraf laden wanneer een map toetsenbordfocus krijgt
cache-dwellMs = Vertraging voor aanwijzer en focus (milliseconden)
cache-idleEnabled = Mappen vooraf laden bij inactiviteit
cache-idleDelayMs = Vertraging bij inactiviteit (milliseconden)
cache-maxConcurrency = Maximaal aantal gelijktijdige voorlaadacties
cache-maxQueued = Maximaal aantal voorlaadacties in de wachtrij
cache-preloadMessages = Speculatieve berichtteksten toestaan
cache-excludedFolders = Mappen die zijn uitgesloten van vooraf laden
cache-exclusions-help = Eén regel per regel: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive, of id: gevolgd door een exact map-ID.
cache-refreshOnOpen = Gecachte inhoud vernieuwen bij het openen
cache-clear = Gecachte mail nu wissen
settings-compose-window = Nieuwe berichten openen in
settings-compose-window-tab = Een werkruimtetabblad
settings-compose-window-new = Een nieuw venster
settings-compose-window-help = Als uw browser een nieuw venster blokkeert, wordt de editor geopend in een werkruimtetabblad.
settings-compose-read-receipt = Altijd leesbevestigingen aanvragen
settings-compose-delivery-receipt = Altijd afleverbevestigingen aanvragen
settings-compose-reply-copy = Een kopie van antwoorden in de oorspronkelijke map opslaan
settings-compose-receipts-help = Bevestigingen zijn verzoeken. De ontvanger of diens mailserver kan ze weigeren of negeren.
settings-compose-forward = Standaardindeling voor doorsturen
settings-compose-forward-inline = Berichttekst in het bericht
settings-compose-forward-attachment = Oorspronkelijk bericht als .eml-bijlage
settings-compose-saving = Opstelvoorkeur opslaan…
settings-compose-saved = Opstelvoorkeur opgeslagen
settings-ident-organization = Organisatie
settings-ident-auto-bcc = Automatisch Bcc
settings-ident-auto-bcc-help = Voeg maximaal 20 adressen toe aan Bcc bij het verzenden vanuit deze identiteit. Scheid adressen met komma's.
settings-ident-pgp-key = OpenPGP-sleutel
settings-ident-pgp-none = De standaardsleutel gebruiken
settings-ident-pgp-unavailable = Opgeslagen sleutel niet beschikbaar — kies een andere sleutel
settings-ident-pgp-help = Kies een van uw bestaande OpenPGP-ondertekeningssleutels. Het privésleutelmateriaal blijft in uw sleutelkluis.
settings-ident-error-optional = Controleer de adressen voor Antwoorden naar en automatisch Bcc, de organisatie en de geselecteerde sleutel.
settings-compose-open-tracking = Het openen van berichten standaard registreren
settings-compose-open-tracking-help = Voegt een trackingafbeelding toe aan in aanmerking komende opgemaakte berichten. Openen hangt ervan af of de ontvanger afbeeldingen laadt en bewijst niet dat het bericht is gelezen. Uw beheerder kan tracking uitschakelen of deze standaard vergrendelen.
settings-compose-open-tracking-inherit = Overgenomen trackingstandaard gebruiken
settings-reply-images = Afbeeldingen in antwoorden en doorgestuurde berichten
settings-reply-images-ask = Elke keer vragen
settings-reply-images-allow = Altijd opnemen
settings-reply-images-trusted = Opnemen voor vertrouwde afzenderdomeinen, anders vragen
settings-reply-images-deny = Nooit opnemen
settings-reply-images-help = Kies of een antwoord of doorgestuurd bericht de afbeeldingen van het oorspronkelijke bericht meeneemt. Opgenomen afbeeldingen worden als bijlagen in het concept gekopieerd.
settings-reply-image-domains = Vertrouwde afzenderdomeinen voor afbeeldingen
settings-reply-image-domains-help = Voer maximaal 200 exacte afzenderdomeinen in, één per regel. Wordt gebruikt wanneer de keuze hierboven vertrouwde domeinen is.
settings-reply-image-domains-save = Vertrouwde afbeeldingsdomeinen opslaan
settings-ident-group-basics = Basis
settings-ident-group-replies = Afhandeling van antwoorden
settings-ident-group-signature = Handtekening
settings-ident-group-sending = Verzendopties
settings-ident-name-required = Voer een weergavenaam in.
settings-ident-email-invalid = Voer een geldig e-mailadres in, zoals naam@voorbeeld.nl.
settings-ident-email-duplicate = U hebt al een identiteit met dit adres.
settings-ident-replyto-invalid = Voer een geldig e-mailadres in of laat dit leeg.
settings-ident-replyto-help = Antwoorden op berichten die vanuit deze identiteit zijn verzonden, gaan naar dit adres in plaats van naar het adres van de afzender.
settings-ident-bcc-invalid = Controleer de Bcc-lijst: gebruik geldige adressen gescheiden door komma's, maximaal 20.
settings-ident-delete-title = De identiteit { $email } verwijderen?
settings-ident-delete-body = U kunt dan niet meer vanuit deze identiteit verzenden. Concepten en verzonden berichten blijven ongewijzigd.
settings-ident-empty = U hebt nog geen identiteiten. Voeg er een toe om bij het schrijven de naam, het adres en de handtekening te kiezen.
settings-sig-delete-title = De handtekening { $name } verwijderen?
settings-sig-delete-body = Identiteiten die haar gebruiken, vallen terug op de accounthandtekening. Reeds verzonden berichten blijven ongewijzigd.
settings-sig-deleted = Handtekening verwijderd.
settings-reply-from = Antwoorden vanaf
settings-reply-from-match = Het adres waarnaar het bericht is verzonden
settings-reply-from-default = Altijd mijn standaardidentiteit
settings-reply-from-ask = Mij vragen wanneer meerdere identiteiten overeenkomen
settings-reply-from-keep = De al geselecteerde identiteit behouden
settings-reply-from-help = Kies bij het beantwoorden of doorsturen welke identiteit wordt gebruikt. Als geen enkele identiteit overeenkomt met het adres waarnaar het bericht is verzonden, wordt uw standaardidentiteit gebruikt. Plusadressen zoals naam+tag@voorbeeld.nl komen overeen met naam@voorbeeld.nl.
settings-ident-default-badge = Standaard
settings-ident-make-default = Standaard maken
settings-ident-default-set = { $email } is nu uw standaardidentiteit.
settings-ident-create-variant = Variant maken

# -- Machine translation (2026-11): added to match en ---------------------
settings-ident-variant-count = { $count } { $count ->
        [one] variant
       *[other] varianten
}
settings-ident-inherits = Gebruikt { $base } voor: { $fields }
settings-ident-field-replyTo = antwoordadres
settings-ident-field-autoBcc = automatische Bcc
settings-ident-field-organization = organisatie
settings-ident-field-pgpKeyId = OpenPGP-sleutel
settings-ident-field-signatureName = handtekening
settings-ident-delete-default-body = Dit is uw standaardidentiteit. Uw oudste overgebleven identiteit wordt de standaard. U kunt dan niet meer vanuit deze identiteit verzenden. Concepten en verzonden berichten blijven ongewijzigd.
settings-ident-variants-choice-title = { $email } verwijderen?
settings-ident-variants-choice-body = Deze identiteit heeft { $count } { $count ->
        [one] variant
       *[other] varianten
}. Verwijder { $count ->
        [one] die
       *[other] die
} ook, of bewaar { $count ->
        [one] die
       *[other] die
} als afzonderlijke identiteiten. Een bewaarde variant kopieert de instellingen die zij van deze identiteit had overgenomen.
settings-ident-delete-with-variants = Identiteit en varianten verwijderen
settings-ident-delete-keep-variants = Identiteit verwijderen, varianten bewaren
settings-ident-new-variant = Nieuwe variant
settings-ident-edit-variant-title = Variant bewerken
settings-ident-variant-intro = Een variant van { $base }. Laat een veld leeg om de waarde van die identiteit te gebruiken.
settings-ident-label-label = Variantlabel
settings-ident-label-required = Voer een label in voor deze variant, zoals Nieuwsbrieven.
settings-ident-inherit-hint = Leeg: gebruikt { $value } van de basisidentiteit.
settings-ident-inherit-none = niets
settings-check-policy-allow = Verzenden zonder te vragen
settings-check-policy-warn = Elke keer vragen
settings-check-policy-warn-remember = Vragen, en mij laten stoppen met vragen
settings-check-policy-deny = Verzenden blokkeren
settings-check-invalid = Ongeldige ontvangersadressen
settings-check-attachment = Bijlage genoemd maar ontbreekt
settings-check-subject = Leeg onderwerp
settings-check-help = Wat te doen wanneer de controle een probleem vindt. Vragen biedt altijd Toch verzenden aan; alleen Verzenden blokkeren houdt een bericht tegen.
settings-external-warn-remember = Vragen, en mij laten stoppen met vragen
settings-external-deny = Verzenden blokkeren
settings-auto-chip = Een ontvanger voltooien wanneer ik een spatie, komma of puntkomma typ
settings-auto-chip-help = Een geldig adres wordt een voltooide ontvanger en het scheidingsteken wordt automatisch toegevoegd.
settings-recipient-separator = Scheidingsteken voor ontvangers
settings-recipient-separator-comma = Komma
settings-recipient-separator-semicolon = Puntkomma
settings-header-collapse = Berichtkop (Aan, Cc, Onderwerp)
settings-header-collapse-expanded = Altijd tonen
settings-header-collapse-collapsed = Samenvouwen tot een samenvattende regel
settings-header-collapse-auto = Samenvouwen terwijl ik de berichttekst schrijf
settings-header-collapse-help = Samengevouwen toont de kop één regel zoals Aan: Ana +2 en wordt geopend wanneer u erop klikt of een veld aandacht nodig heeft.
settings-retention-title = Bewaartermijn
settings-folder-icon-camera = Camera
settings-folder-icon-music = Muziek
settings-folder-icon-film = Film
settings-folder-icon-lightbulb = Idee
settings-folder-icon-rocket = Raket
settings-folder-icon-trophy = Trofee
settings-folder-icon-leaf = Blad
settings-folder-icon-globe = Wereldbol
settings-folder-icon-cloud = Wolk
settings-folder-icon-bookmark = Bladwijzer
settings-folder-icon-wallet = Portemonnee
settings-folder-icon-medical = Medisch
settings-folder-icon-bell = Bel
settings-folder-icon-key = Sleutel
settings-folder-icon-receipt = Bon
settings-folder-icon-cart = Winkelwagen
settings-folder-icon-package = Pakket
settings-folder-icon-wrench = Gereedschap
settings-folder-icon-chart = Grafiek
settings-folder-icon-team = Team
settings-folder-icon-store = Winkel
settings-folder-icon-office = Kantoor

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-icon-laptop = Laptop
settings-folder-icon-crown = Kroon
settings-folder-icon-family = Familie
settings-folder-icon-group-general = Algemeen
settings-folder-icon-group-mail = Mail
settings-folder-icon-group-work = Werk
settings-folder-icon-group-finance = Financiën en winkelen
settings-folder-icon-group-travel = Reizen en thuis
settings-folder-icon-group-personal = Persoonlijk en plezier
settings-folder-icon-group-tech = Techniek en gereedschap
settings-folder-icon-search-label = Pictogrammen zoeken
settings-folder-icon-no-match = Geen pictogrammen gevonden

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Authenticatie (DMARC)

settings-badge-auth-help = Het DMARC resultaat; het opent de volledige beveiligingsdetails.

settings-badge-density = Badge-grootte

settings-badge-density-comfortable = Comfortabel

settings-badge-density-compact = Compact

settings-badge-density-help = Compacte of comfortabele hoogte en afstand voor elke badge.

settings-badge-folder = Map

settings-badge-folder-help = Waar het bericht wordt opgeslagen. Alleen als het opmerkelijk is, wordt het weergegeven voor Junk en Trash.

settings-badge-labels = Badge-etiketten

settings-badge-labels-help = Pictogram met tekst, of het pictogram alleen (de volledige tekst blijft in de tooltip en voor schermlezers).

settings-badge-labels-icon = Alleen icoon

settings-badge-labels-text = Icoon en tekst

settings-badge-list = Mailinglijst

settings-badge-list-help = De mailinglijst waar het bericht vandaan kwam; het opent de abonnementsgegevens en acties. Alleen wanneer opmerkelijk wordt weergegeven wanneer er e-mail blijft aankomen nadat u zich heeft afgemeld.

settings-badge-mode-always = Altijd

settings-badge-mode-never = Nooit

settings-badge-mode-notable = Alleen als het opmerkelijk is

settings-badge-sanitized = Gezuiverd HTML

settings-badge-sanitized-help = Wat het HTML-ontsmettingsmiddel met de boodschap deed. Alleen wanneer opmerkelijk wordt aangegeven wanneer er iets is geblokkeerd.

settings-badge-spam = Spamscore

settings-badge-spam-help = De score die de ontvangende server in de berichtkoppen heeft geschreven.

settings-folders-hide-empty = Verberg lege mappen in de zijbalk

settings-folders-hide-empty-help = Mappen verschijnen opnieuw wanneer ze berichten bevatten. Inbox, zoekmappen, mappen met onbekende aantallen en bovenliggende mappen van zichtbare mappen blijven zichtbaar. Overschrijf individuele mappen onder Eigenschappen, Weergave; elke map blijft in deze lijst staan.

settings-folders-list-card = Mappenlijst en volgorde

settings-header-auto-collapse = Vouw de koptekst na het schrijven (seconden)

settings-header-auto-collapse-help = Wanneer de hoofdtekst zoveel seconden focus heeft gehad en de koptekst een ontvanger en een onderwerp bevat, vouwt deze naar de samenvattingsregel. 0 schakelt dit uit.

settings-message-badges = Berichtbadges

settings-message-badges-help = Kies welke badges in de koptekst van een geopend bericht verschijnen. Prioriteitsmarkeringen worden ingesteld onder Mappen en zijbalk. U kunt ook met de rechtermuisknop op een badge klikken om deze te verbergen.

settings-reply-from-headers = Headers die de identiteit kiezen

settings-reply-from-headers-help = Eén headernaam per regel, geprobeerd in deze volgorde: afgeleverd-aan, X-origineel-aan, envelop-aan, X-doorgestuurd-naar, opnieuw verzonden-aan, aan, cc. Een weggelaten header wordt niet gelezen.

settings-reply-from-headers-invalid = Gebruik alleen deze namen, één per regel: Bezorgd aan, X-Origineel-aan, Envelop-aan, X-doorgestuurd naar, Opnieuw verzonden naar, Aan, Cc.

settings-reply-from-headers-save = Kopvolgorde opslaan

settings-sessions-linked-created = Koppeling gemaakt

settings-sessions-linked-details = Linkdetails

settings-sessions-linked-kind = Linktype

settings-sessions-linked-kind-linked = Gekoppelde mailbox

settings-sessions-linked-last-used = Laatst gebruikt

settings-sessions-linked-line = Toegang tot gekoppelde mailbox · vanaf { $origin }

settings-sessions-linked-line-hidden = Gekoppelde mailboxtoegang

settings-sessions-linked-origin = Komt uit

settings-sessions-linked-removed = Onbekend (account verwijderd)

settings-sessions-linked-revoke = Linktoegang intrekken

settings-sessions-linked-revoke-label = Linktoegang intrekken voor { $mailbox }

settings-sessions-linked-target = Brievenbus geopend

settings-sessions-origin-full = Volledig adres

settings-sessions-origin-help = Bepaalt hoe deze tabel het account noemt waar een gekoppelde mailboxsessie vandaan komt. Alleen jij ziet het, in je eigen sessies.

settings-sessions-origin-hidden = Verborgen

settings-sessions-origin-masked-local-part = Adres met het eerste deel gemaskeerd

settings-sessions-origin-setting = Laat zien waar gekoppelde mailboxtoegang vandaan komt

settings-shortcuts-search = Zoek snelkoppelingen

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = Houd recent bezochte mappen en berichten gereed om te openen. Kopieën blijven standaard in het geheugen van dit tabblad. Schakel opslag voor onbepaalde tijd in om versleutelde kopieën te bewaren tijdens herlaadbeurten. Als u vooraf laadt, wordt e-mail nooit als gelezen gemarkeerd en worden externe afbeeldingen niet geladen.

cache-keepIndefinitely = Bewaar e-mail in de cache voor onbepaalde tijd

cache-keepIndefinitely-help = Standaard uitgeschakeld. Bewaar gecodeerde kopieën in deze browser tijdens herlaadbeurten zonder dat de leeftijd verstrijkt. Online berichten worden nog steeds vernieuwd. De invoer- en bytelimieten zijn nog steeds van toepassing. Als u dit uitschakelt of in de cache opgeslagen e-mail wist, worden opgeslagen kopieën verwijderd. Het wissen van browsergegevens kan ze ook verwijderen. Bijlagen worden niet gedownload voor offline gebruik.

settings-notif-view-enabled = Meldingenweergave inschakelen
settings-notif-view-sidebar = Meldingen in de zijbalk tonen
settings-notif-view-help = Beheert het meldingencentrum. Beveiligingsmeldingen, e-mailmeldingen en browsermeldingen behouden hun eigen instellingen.
