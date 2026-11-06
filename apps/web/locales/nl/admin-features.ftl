# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Functieprofielen
admin-features-intro = Een profiel is een benoemde set functieschakelaars. Wijs het toe aan een domein, een groep of een gebruiker. Individuele toewijzingen ondersteunen native postvakken en geverifieerde proxy- of SSO-identiteiten; ze regelen de toegang tot Mailwoman-functies.
admin-features-new = Nieuw profiel
admin-features-search = Profielen zoeken
admin-features-empty = Nog geen functieprofielen.
admin-features-no-match = Geen profiel komt overeen met de zoekopdracht.
admin-features-load-error = De functiecatalogus kon niet worden geladen.
admin-features-not-loaded = De functiecatalogus is nog niet geladen. Laad de pagina opnieuw en probeer het nogmaals.
admin-features-stale = Een andere beheerder heeft de functiecatalogus gewijzigd. Deze is opnieuw geladen; controleer de waarden en sla opnieuw op.
admin-features-not-found = Geen profiel met de id { $id }.
admin-features-col-name = Naam
admin-features-col-values = Schakelaars
admin-features-col-assigned = Toegewezen aan
admin-features-values-count = { $count ->
    [one] { $count } schakelaar ingesteld
   *[other] { $count } schakelaars ingesteld
 }
admin-features-assigned-count = { $count ->
    [0] Niet toegewezen
    [one] { $count } plek
   *[other] { $count } plekken
 }
admin-features-editor = Profielinstellingen
admin-features-name = Profielnaam
admin-features-name-help = Alleen zichtbaar voor beheerders, maximaal 128 tekens.
admin-features-name-invalid = Voer een naam van 1 tot 128 tekens in.
admin-features-switches = Functieschakelaars
admin-features-switches-help = "Niet ingesteld" laat de functie over aan het volgende profiel, het domein of de serverstandaard. Een vergrendelde waarde kan niet worden gewijzigd door de gebruiker of door een lager niveau.
admin-features-col-feature = Functie
admin-features-col-value = Waarde
admin-features-col-lock = Vergrendeling
admin-features-value-for = Waarde voor { $feature }
admin-features-unset = Niet ingesteld
admin-features-on = Toegestaan
admin-features-off = Niet toegestaan
admin-features-lock = Vergrendeld
admin-features-save = Profiel opslaan
admin-features-create = Profiel aanmaken
admin-features-delete = Profiel verwijderen
admin-features-delete-title = Het profiel { $name } verwijderen?
admin-features-delete-body = { $count ->
    [0] Het profiel is nergens toegewezen. De schakelaars ervan gelden direct niet meer.
    [one] Het profiel wordt verwijderd van de { $count } plek waaraan het is toegewezen, en de schakelaars ervan gelden direct niet meer.
   *[other] Het profiel wordt verwijderd van de { $count } plekken waaraan het is toegewezen, en de schakelaars ervan gelden direct niet meer.
 }
admin-features-saved = Profiel { $name } opgeslagen.
admin-features-deleted = Profiel { $name } verwijderd.
admin-features-assigned = Profiel { $name } toegewezen.
admin-features-unassigned = Toewijzing van { $name } verwijderd.
admin-features-assignments = Toegewezen aan
admin-features-assignments-help = Profielen gelden in volgorde van prioriteit, hoogste eerst. Bij gelijke prioriteit gaat een toewijzing aan een gebruiker vóór een groep en een groep vóór een domein.
admin-features-assignments-empty = Dit profiel is nog niet toegewezen.
admin-features-assign-form = Dit profiel toewijzen
admin-features-col-scope = Geldt voor
admin-features-col-target = Naam
admin-features-col-priority = Prioriteit
admin-features-col-actions = Acties
admin-features-scope-domain = Domein
admin-features-scope-group = Groep
admin-features-scope-user = Gebruiker
admin-features-assign = Toewijzen
admin-features-unassign = Verwijderen
admin-features-no-domain = Er zijn geen domeinen geconfigureerd.
admin-features-no-group = Er bestaan geen gebruikersgroepen.
admin-features-no-user = Er hebben zich nog geen postvakaccounts aangemeld.
admin-features-tools = Profielhulpmiddelen
admin-features-preset = Beginnen met een voorinstelling
admin-features-preset-choose = Kies een voorinstelling
admin-features-preset-apply = Voorinstelling toepassen
admin-features-preset-applied = Voorinstelling { $name } toegepast. Controleer de waarden en sla op.
admin-features-preset-full = Volledig
admin-features-preset-standard = Standaard
admin-features-preset-mail-only = Alleen mail
admin-features-preset-minimal = Minimaal / kiosk
admin-features-import = JSON importeren
admin-features-export = JSON exporteren
admin-features-exported = Profiel geëxporteerd als JSON-bestand.
admin-features-imported = { $count ->
    [one] { $count } waarde geïmporteerd. Controleer en sla op.
   *[other] { $count } waarden geïmporteerd. Controleer en sla op.
 }
admin-features-import-json = Het bestand is geen geldige JSON.
admin-features-import-format = Het bestand is geen Mailwoman-functieprofiel.
admin-features-import-version = Dit bestand met een functieprofiel komt uit een nieuwere versie en kan niet worden geïmporteerd.
admin-features-clone = Klonen
admin-features-clone-name = Kopie van { $name }
admin-features-filter = Een functie zoeken
admin-features-filter-placeholder = Naam, instelling of sleutel
admin-features-filter-none = Geen functie komt overeen met de zoekopdracht.
admin-features-diff-summary = { $differs ->
    [one] { $differs } waarde wijkt af van de serverstandaard
   *[other] { $differs } waarden wijken af van de serverstandaard
} · { $same } herhalen die
admin-features-differs = Wijkt af van de standaard
admin-features-default-is = Standaard: { $value }
admin-features-fix-values = Corrigeer de als ongeldig gemarkeerde waarden voordat u opslaat.
admin-features-allow-all = Alles toestaan
admin-features-deny-all = Niets toestaan
admin-features-clear-all = Wissen
admin-features-group-allow = Elke functie in { $group } toestaan
admin-features-group-deny = Geen enkele functie in { $group } toestaan
admin-features-group-clear = Elke waarde in { $group } wissen
admin-features-enforcement-ui = Alleen verborgen in de app
admin-features-enforcement-partial = Sommige limieten worden nog niet geteld
admin-features-other = Andere instellingen die een profiel kan bevatten
admin-features-other-help = Elke andere instelling die een beheerder per profiel mag instellen. Laat een veld leeg om de serverwaarde te behouden.
admin-features-cat-compose = Mail en opstellen
admin-features-cat-reading = Lezen en privacy
admin-features-cat-views = Weergaven en zoeken
admin-features-cat-organise = Ordenen
admin-features-cat-pim = Agenda, contacten, taken, notities en sleutels
admin-features-cat-data = Gegevens
admin-features-cat-accounts = Accounts en toegang
admin-features-cat-integrations = Integraties
admin-features-cat-developer = Toegang voor ontwikkelaars
admin-features-cat-personal = Personalisatie
admin-features-feature-compose-sending = Verzendlimieten
admin-features-feature-compose-sending-help = Ontvangers per bericht worden door de server boven de limiet geweigerd. Berichten per uur en per dag worden voor het profiel opgeslagen maar nog niet geteld.
admin-features-feature-storage = Opslagquotum
admin-features-feature-storage-help = De opslaglimiet wordt voor het profiel opgeslagen; het eigen quotum van de mailserver blijft gelden.
admin-features-feature-access-protocols = Protocollen voor mailtoegang
admin-features-feature-access-protocols-help = Met welke mailserverprotocollen deze gebruikers zich mogen aanmelden, gescheiden door komma's: imap, pop3, ews.
admin-features-feature-accounts-linked = Gekoppelde postvakken
admin-features-feature-calendar = Agenda
admin-features-feature-contacts = Contacten
admin-features-feature-tasks = Taken
admin-features-feature-notes = Notities
admin-features-preview = Effectieve waarden
admin-features-preview-help = Waartoe elke functie wordt vertaald voor één gebruiker of voor een gebruiker van één domein, na elk profiel, elke groep, elke domeinwaarde en elke vergrendeling.
admin-features-preview-kind = Voorbeeld voor
admin-features-preview-target = Wie
admin-features-preview-show = Tonen
admin-features-preview-loading = Laden…
admin-features-preview-state = Effectief
admin-features-preview-from = Bepaald door
admin-features-preview-profiles = Profielen die gelden, eerste wint: { $names }
admin-features-preview-no-profiles = Er geldt geen profiel; het domein en de serverwaarden beslissen.
admin-features-source-default = Ingebouwde standaard
admin-features-source-env-fallback = Standaard uit de omgeving
admin-features-source-env-pin = Omgeving (vastgezet)
admin-features-source-global = Serverwaarde
admin-features-source-domain = Domeinwaarde
admin-features-source-profile = Profiel
admin-features-source-profile-named = Profiel { $name }
admin-features-source-group = Groepswaarde
admin-features-source-admin-user = Voor deze gebruiker ingestelde waarde
admin-features-source-user = De eigen keuze van de gebruiker
admin-features-source-locked = { $source } (vergrendeld)
admin-features-feature-compose-open-tracking = Openingen van berichten bijhouden
admin-features-feature-compose-open-tracking-help = Toestaan dat openingen van geschikte opgemaakte berichten worden vastgelegd met de trackingafbeelding van de server. Vereist een geconfigureerde openbare URL.
admin-features-feature-reading-original-html = Weergave van originele HTML
admin-features-feature-reading-original-html-help = Gebruikers toestaan om voor afzonderlijke berichten te kiezen voor originele HTML, binnen het geïsoleerde leesframe. Opgeschoonde HTML blijft de standaard.
admin-features-inherit = Overnemen
admin-features-override = Een waarde instellen
admin-features-setting-mode = Waardemodus voor { $setting }
admin-features-inherit-help = Overnemen verwijdert de waarde en vergrendeling van dit profiel. Het volgende toepasselijke profiel, domein of de serverwaarde beslist; het effectieve voorbeeld toont het resultaat en de bron ervan.
admin-features-domain-alias = { $alias } (alias van { $domain })
admin-features-domain-aliases = { $domain } (aliassen: { $aliases })

# -- Machine translation (2026-11): added to match en ---------------------
admin-retention-col-category = Categorie
admin-retention-col-kept = Bewaard
admin-retention-col-change = Wijzigen
admin-retention-cat-default = Standaard
admin-retention-cat-admin = Acties van beheerders
admin-retention-cat-security = Beveiligingsgebeurtenissen
admin-retention-cat-alert = Waarschuwingen voor gebruikers
admin-retention-forever = Elke vermelding
admin-retention-days = { $days ->
    [one] { $days } dag
   *[other] { $days } dagen
 }
admin-retention-follows-default = Standaard: { $kept }
admin-retention-hold-on = Juridische bewaarplicht
admin-retention-read-only = Voor het wijzigen van bewaartermijnen is het recht settings.write nodig.
admin-retention-edit-for = Bewaartermijn voor { $category }
admin-retention-keep = Bewaren
admin-retention-mode-inherit = De standaard volgen
admin-retention-mode-forever = Elke vermelding
admin-retention-mode-days = Gedurende een aantal dagen
admin-retention-days-label = Dagen
admin-retention-hold = Juridische bewaarplicht
admin-retention-save = Opslaan
admin-retention-bounds = Voer een geheel getal in van { $min } tot { $max }.
admin-retention-saved = Bewaartermijn voor { $category } opgeslagen.
admin-retention-task-none = Nog geen opschoontaak. Ze wordt dagelijks om 03:15 UTC aangemaakt zodra voor het eerst een aantal dagen is opgeslagen.
admin-retention-task-disabled = De geplande opschoontaak is uitgeschakeld; vermeldingen worden alleen met Nu opschonen opgeschoond.
admin-retention-task-next = Volgende geplande opschoning: { $when }.
admin-retention-task-daily = Dagelijks om 03:15 UTC opgeschoond.
admin-retention-last = Laatste opschoning waarbij vermeldingen zijn verwijderd: { $when }.
admin-retention-prune = Nu opschonen
admin-retention-prune-title = Het auditlogboek nu opschonen?
admin-retention-prune-body = Vermeldingen die ouder zijn dan de bewaartermijn van hun categorie worden verwijderd. Categorieën die elke vermelding bewaren of onder juridische bewaarplicht staan, blijven ongewijzigd. Dit kan niet ongedaan worden gemaakt.
admin-retention-pruned = { $count ->
    [0] Er waren geen vermeldingen oud genoeg om te verwijderen.
    [one] { $count } vermelding verwijderd.
   *[other] { $count } vermeldingen verwijderd.
 }
admin-auth-attempted = Geprobeerd als { $name }
admin-audit-retention-title = Bewaartermijn van het auditlogboek
admin-audit-retention-intro = Hoe lang auditvermeldingen worden bewaard, per categorie. Een categorie zonder eigen waarde volgt de standaard; zonder standaard blijft elke vermelding bewaard. Een juridische blokkade stopt het opschonen voor haar categorie, en een blokkade op de standaard stopt het voor alle. Wijzigingen en opschoningen worden vastgelegd in het auditlogboek.
admin-audit-retention-loading = Bewaarbeleid laden…
admin-audit-retention-unavailable = Auditbewaring is niet beschikbaar op deze server.
