# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: rules. Source: ../en/rules.ftl

rules-title = Regels en filters
rules-intro = Beheer filterscripts en visuele regels. Beschikbare functies hangen af van dit account en de beheerdersinstellingen.
rules-empty = Nog geen regels.
rules-unavailable = Deze mailserver ondersteunt geen mailregels.
rules-load-failed = De regels konden niet worden geladen.
rules-new = Nieuwe regel
rules-list-label = Mailregels
rules-enabled = Ingeschakeld
rules-delete-named = Regel { $name } verwijderen

# Builder
rules-builder-title = Regels opstellen
rules-builder-hint = Selecteer een regel om te bewerken, of maak een nieuwe.
rules-name-label = Naam van regel
rules-match-legend = Wanneer uitvoeren
rules-match-all = Aan alle voorwaarden voldoen
rules-match-any = Aan een van de voorwaarden voldoen
rules-conditions-legend = Voorwaarden
rules-actions-legend = Acties
rules-add-condition = Voorwaarde toevoegen
rules-add-action = Actie toevoegen
rules-remove-condition = Voorwaarde verwijderen
rules-remove-action = Actie verwijderen
rules-cond-type-label = Veld
rules-cond-op-label = Operator
rules-cond-value-label = Te vergelijken waarde
rules-act-type-label = Actie
rules-act-value-label = Waarde van actie
rules-save = Regel opslaan
rules-editing = { $name } bewerken
rules-save-refused = De regel is niet opgeslagen: { $reason }

# Condition fields and operators
rules-cond-from = Van
rules-cond-to = Aan
rules-cond-subject = Onderwerp
rules-cond-thread-unsupported = Gesprek { $op } { $value }: voorwaarden op gesprekken worden niet ondersteund, dus de regel kan pas worden opgeslagen als deze voorwaarde is verwijderd.
rules-op-contains = bevat
rules-op-is = is

# Actions
rules-act-move = Verplaatsen naar map
rules-act-tag = Tag toevoegen
rules-act-archive = Archiveren
rules-act-suppress = Melding onderdrukken
rules-act-stop = Verwerking stoppen

# Where it runs
rules-runs-server = Server (Sieve)
rules-runs-server-detail = Geüpload naar de mailserver en daar toegepast.
rules-runs-engine = Engine
rules-runs-engine-detail = Toegepast bij het synchroniseren van nieuwe inboxberichten; antwoorden en verplaatsingen wachten op de volgende synchronisatie.

# Panes
rules-panes-label = Regelhulpmiddelen
rules-tab-builder = Opstellen
rules-tab-raw = Sieve-broncode
rules-tab-dryrun = Proefdraaien

# Raw editor
rules-raw-label = Sieve-broncode
rules-raw-reset = Terugzetten naar gegenereerde code
rules-lint-clean = Geen problemen gevonden.

# Dry run
rules-dryrun-title = Proefdraaien
rules-dryrun-help = Voer een voorbeeldbericht in om te zien welke regels zouden worden uitgevoerd.
rules-dryrun-results = Resultaten van proefdraaien
rules-dryrun-none = Geen ingeschakelde regels om te evalueren.
rules-dryrun-nomatch = Geen overeenkomst
rules-dryrun-stopped = Niet bereikt (een eerdere regel heeft de verwerking gestopt)

rules-scripts-label = Sieve-scripts
rules-script-active = Actief
rules-script-new = Nieuw script
rules-script-export = Script exporteren
rules-script-import = Script importeren
rules-script-too-large = Het script overschrijdt de maximale grootte.
rules-script-read-only = De beheerder heeft het rechtstreeks bewerken van scripts uitgeschakeld.
rules-script-validate = Script valideren
rules-script-deactivate = Deactiveren
rules-script-activate = Activeren
rules-script-saved = Script opgeslagen.

rules-vacation-title = Afwezigheidsbericht
rules-vacation-enabled = Automatische antwoorden inschakelen
rules-vacation-from = Begin (UTC)
rules-vacation-to = Einde (UTC)
rules-vacation-body = Antwoordtekst
rules-vacation-saved = Afwezigheidsbericht opgeslagen.
rules-script-template = Scriptsjabloon
rules-forward-title = Doorsturen
rules-forward-address = Doorsturen naar
rules-forward-copy = Een kopie in dit account bewaren
rules-forward-refused = Uw beheerder staat doorsturen naar dit domein niet toe.
