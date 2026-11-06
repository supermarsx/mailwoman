# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Omhoog
nextcloud-loading = Laden…
nextcloud-list-error = Kan de inhoud van deze map niet weergeven.
nextcloud-empty = Deze map is leeg.
nextcloud-file-list = Nextcloud-bestanden
nextcloud-open-folder = Map { $name } openen
nextcloud-select-file = { $name } selecteren

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Bijvoegen uit Nextcloud
nextcloud-attach-action = { $count ->
    [0] Bijvoegen
    [one] { $count } bestand bijvoegen
   *[other] { $count } bestanden bijvoegen
  }
nextcloud-error-select-file = selecteer minstens één bestand om bij te voegen
nextcloud-error-attach-failed = kan de geselecteerde bestanden niet bijvoegen

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Opslaan in Nextcloud
nextcloud-save-title = “{ $name }” opslaan in Nextcloud
nextcloud-save-action = Hier opslaan ({ $dir })
nextcloud-saved = Opgeslagen in { $path }
nextcloud-error-save-failed = kan niet opslaan in Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Deellink maken
nextcloud-share-title = Deellink
nextcloud-share-intro = Maak een openbare link naar { $path } in plaats van het bestand bij te voegen.
nextcloud-protect-password = Beveiligen met een wachtwoord
nextcloud-password-label = Wachtwoord
nextcloud-share-password = Wachtwoord voor de deellink
nextcloud-set-expiry = Vervaldatum instellen
nextcloud-expires-on = Verloopt op
nextcloud-expiry-date = Vervaldatum
nextcloud-create-link = Link maken
nextcloud-password-protected = Met wachtwoord beveiligd
nextcloud-no-password = Geen wachtwoord
nextcloud-expires = verloopt { $date }
nextcloud-no-expiry = verloopt niet
nextcloud-error-need-password = voer een wachtwoord in of zet de wachtwoordbeveiliging uit
nextcloud-error-need-expiry = kies een vervaldatum of zet de vervaldatum uit
nextcloud-error-share-failed = kan de deellink niet maken

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } is groter dan de bijlagelimiet. Upload het naar de map “Mail attachments” van je Nextcloud en voeg in plaats daarvan een deellink in het bericht in.
nextcloud-upload-and-link = Uploaden en link invoegen
nextcloud-share-not-now = Niet nu
nextcloud-upload-progress = Uploaden: { $percent }%
nextcloud-upload-progress-label = Uploadvoortgang
nextcloud-share-inserted = Deellink voor { $name } in het bericht ingevoegd.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = De link werkt tot { $date }.
nextcloud-share-body-password = De link is met een wachtwoord beveiligd.
nextcloud-account-title = Nextcloud-accounts
nextcloud-account-help = Koppel je eigen account aan een instantie die je beheerder heeft goedgekeurd. Inloggegevens worden versleuteld opgeslagen, alleen voor dit mailboxaccount.
nextcloud-account-load-error = De Nextcloud-koppelingen konden niet worden geladen. Laad opnieuw om het nog eens te proberen.
nextcloud-account-unlink-error = De koppeling kon niet worden verwijderd. Laad opnieuw voordat je het nog eens probeert.
nextcloud-account-reload = Opnieuw laden
nextcloud-account-no-instances = Er zijn geen ingeschakelde Nextcloud-instanties beschikbaar. Vraag je beheerder er een in te stellen.
nextcloud-account-ready = Gekoppeld en klaar
nextcloud-account-stale = De instantieconfiguratie is gewijzigd. Verifieer de koppeling opnieuw.
nextcloud-account-unlinked = Niet gekoppeld
nextcloud-account-relink = Koppeling opnieuw verifiëren
nextcloud-account-link = Account koppelen
nextcloud-account-unlink = Account ontkoppelen
nextcloud-account-password-help = Maak in de persoonlijke beveiligingsinstellingen van Nextcloud een app-wachtwoord aan en kopieer de gebruikersnaam en het wachtwoord hierheen. De verificatie doet één begrensd verzoek; geweigerde inloggegevens worden niet opnieuw geprobeerd.
nextcloud-account-username = Gebruikersnaam van het Nextcloud-app-wachtwoord
nextcloud-account-password = Nextcloud-app-wachtwoord
nextcloud-account-verify = Verifiëren en koppelen
nextcloud-account-verify-error = Verifiëren of opslaan is mislukt. Controleer de instantie, de gebruikersnaam en het wachtwoord van het app-wachtwoord. Het wachtwoordveld is gewist; een bestaande werkende koppeling is niet vervangen.
nextcloud-admin-title = Nextcloud-instanties
nextcloud-admin-help = Keur HTTPS-instanties goed die mailboxgebruikers afzonderlijk mogen koppelen. Aan mailboxaccounts wordt geen gedeeld servicewachtwoord gegeven. Bestandsbewerkingen zijn beperkt tot 8 MiB. Gebruikers van de native engine zoeken en verbinden na het koppelen expliciet agenda's, takenlijsten en adresboeken.
nextcloud-admin-add = Nextcloud-instantie toevoegen
nextcloud-admin-edit = Nextcloud-instantie bewerken
nextcloud-admin-enabled = Ingeschakeld
nextcloud-admin-disabled = Uitgeschakeld
nextcloud-admin-missing = Deze instantie bestaat niet meer. Ga terug naar de instantielijst.
nextcloud-admin-private-help = Privéadressen vereisen een expliciete Nextcloud-egresstoestemming. Ingestelde proxyroutes blijven leidend; een door de proxy geweigerd verzoek valt nooit terug op een directe verbinding.
nextcloud-admin-revalidation = Bij het opslaan van een wijziging aan een instantie moeten gebruikers hun koppelingen opnieuw verifiëren. Het uitschakelen van een instantie stopt nieuwe bewerkingen onmiddellijk.
nextcloud-admin-authority = Autoriteit van de instantie
nextcloud-admin-name = Weergavenaam
nextcloud-admin-name-help = Een korte naam die aan gebruikers wordt getoond bij het kiezen van een instantie.
nextcloud-admin-url = HTTPS-installatie-URL
nextcloud-admin-url-help = Gebruik de installatieroot, zoals https://cloud.example.org/ of https://example.org/nextcloud/. Het certificaat moet vertrouwd zijn en overeenkomen met de hostnaam. Inloggegevens, queryparameters en fragmenten worden geweigerd.
nextcloud-admin-enable = Gebruikers toestaan deze instantie te koppelen
nextcloud-admin-ca = Aanvullende certificeringsinstanties (PEM)
nextcloud-admin-ca-help = Optionele privé-CA-certificaten voor deze Nextcloud-server. Publiek vertrouwen blijft beschikbaar en hostnaamverificatie blijft vereist. Wijzigingen vereisen dat gebruikers hun koppelingen opnieuw verifiëren; het vertrouwen in proxycertificaten staat los hiervan.
nextcloud-instance-select = Nextcloud-instantie
nextcloud-instance-placeholder = Kies een gekoppelde instantie
nextcloud-instances-empty = Geen geverifieerde Nextcloud-koppelingen. Voeg een koppeling toe of verifieer er een via Gebruikersinstellingen → Nextcloud.
nextcloud-instances-error = De gekoppelde instanties konden niet worden geladen. Open de kiezer opnieuw om het nog eens te proberen.
nextcloud-save-warning = Je beheerder vereist een bevestiging voordat deze bijlage de mailbox mag verlaten.
nextcloud-save-acknowledge = Ik begrijp de waarschuwing over de bijlage en wil dit bestand opslaan in de geselecteerde Nextcloud-instantie.
nextcloud-save-attachment = { $name } opslaan in Nextcloud
nextcloud-save-close = Nextcloud-bestemming sluiten
nextcloud-save-options = Bijlagen opslaan in Nextcloud
nextcloud-pim-title = Contacten en agenda's
nextcloud-pim-help = Zoek collecties van dit geverifieerde Nextcloud-account en verbind daarna expliciet degene die je wilt. Synchroniseer om actuele gegevens op te halen; verbonden bewerkingen gebruiken de externe collectie en mislukken als de koppeling of de rechten zijn ingetrokken.
nextcloud-pim-category = Collectiecategorie
nextcloud-pim-calendar = Agenda's
nextcloud-pim-contacts = Adresboeken
nextcloud-pim-tasks = Takenlijsten
nextcloud-pim-discover = Collecties zoeken
nextcloud-pim-discovered = Beschikbare collecties
nextcloud-pim-empty = Er zijn geen overeenkomende collecties teruggegeven.
nextcloud-pim-connect = Verbinden
nextcloud-pim-connected = Verbonden collecties
nextcloud-pim-sync = Nu synchroniseren
nextcloud-pim-sync-needed = Collectie verbonden. Synchroniseer nu om de inhoud te laden.
nextcloud-pim-synced = Synchronisatie voltooid.
nextcloud-pim-error = De bewerking is mislukt. Controleer de accountkoppeling en de rechten en probeer het daarna expliciet opnieuw. Bestaande gecachete gegevens zijn niet weggegooid.
nextcloud-pim-native-required = Synchronisatie van contacten en agenda vereist de native mailengine. Bestandstoegang blijft beschikbaar via je accountkoppeling.
nextcloud-pim-disconnect = Verbinding verbreken
nextcloud-pim-disconnect-help = Bij het verbreken van de verbinding blijven gecachete gegevens behouden en stoppen externe schrijfacties. Opnieuw gekoppelde accounts vereisen opnieuw zoeken en expliciet verbinden.
nextcloud-pim-ready = Gereed
nextcloud-pim-disconnected = Verbroken of koppeling gewijzigd
nextcloud-pim-unnamed = Collectie zonder naam
nextcloud-pim-back = Terug naar Nextcloud-accounts
nextcloud-pim-create-title = Nieuwe collectie in Nextcloud
nextcloud-pim-create-help = Maakt een lege agenda of takenlijst aan in dit Nextcloud-account en verbindt die hier.
nextcloud-pim-create-name = Naam van de nieuwe collectie
nextcloud-pim-create = Aanmaken in Nextcloud
nextcloud-pim-created = { $name } is aangemaakt in Nextcloud en verbonden. Synchroniseer nu om het te laden.
nextcloud-pim-interval = Interval voor automatisch vernieuwen (seconden)
nextcloud-pim-interval-help = Nul gebruikt alleen handmatig vernieuwen. Positieve intervallen hebben een minimum van 60 seconden. Verbonden accounts worden op de achtergrond vernieuwd; bij fouten wordt langer gewacht en geweigerde inloggegevens stoppen verdere verzoeken totdat je opnieuw koppelt.
nextcloud-pim-interval-reset = Standaard van de beheerder gebruiken
nextcloud-admin-empty = Nog geen goedgekeurde instanties. Voeg een instantie toe om die beschikbaar te maken voor het koppelen van accounts.
nextcloud-admin-status = Status
nextcloud-admin-open = { $name } openen
