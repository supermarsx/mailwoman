# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = API-sleutels
apikeys-heading = API-sleutels
apikeys-intro = Maak sleutels met een beperkte reikwijdte voor scripts en integraties. Elke sleutel wordt gehasht opgeslagen, één keer getoond en kan afzonderlijk worden ingetrokken. Geef niet meer rechten dan nodig.

apikeys-label = Label
apikeys-label-aria = Label van de sleutel
apikeys-label-placeholder = bijv. back-upscript

apikeys-create = Sleutel maken
apikeys-saved = Ik heb hem bewaard

apikeys-reveal-warning = Kopieer dit geheim nu — het wordt één keer getoond en kan later niet meer worden opgevraagd.
apikeys-copy = Kopiëren
apikeys-copy-aria = Geheim naar het klembord kopiëren
apikeys-copied = Gekopieerd naar het klembord

apikeys-error-need-label = geef de sleutel een label zodat je hem later herkent
apikeys-error-create = de sleutel kon niet worden gemaakt
apikeys-error-revoke = de sleutel kon niet worden ingetrokken

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Bestaande sleutels
apikeys-existing = Bestaande sleutels
apikeys-none = Nog geen sleutels.
apikeys-created = gemaakt op { $date }
apikeys-last-used = laatst gebruikt op { $date }
apikeys-never-used = nooit gebruikt
apikeys-revoked-at = ingetrokken op { $date }
apikeys-revoke = Intrekken
apikeys-status-active = Actief
apikeys-status-revoked = Ingetrokken

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Reikwijdte
apikeys-capabilities = Rechten
apikeys-cap-read = Lezen
apikeys-cap-send = Verzenden
apikeys-cap-delete = Verwijderen
apikeys-surface = Onderdeel
apikeys-surface-mail-label = Mail
apikeys-surface-pim-label = PIM (agenda / taken / notities / contacten)
apikeys-accounts = Accounts
apikeys-all-accounts = Alle accounts
apikeys-folders = Mappen
apikeys-all-folders = Alle mappen
apikeys-folder-ids = Map-id's (kommagescheiden)
apikeys-folder-subset-aria = Selectie van mappen
apikeys-constraints = Beperkingen
apikeys-expiry = Vervaldatum (RFC 3339, leeg = verloopt niet)
apikeys-expiry-aria = Vervaldatum
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Snelheidslimiet (verzoeken/min, leeg = onbeperkt)
apikeys-rate-limit-aria = Snelheidslimiet
apikeys-ip-allowlist = Toegestane IP-adressen (CIDR/IP, kommagescheiden; leeg = alle)
apikeys-ip-allowlist-aria = Toegestane IP-adressen
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = lezen
apikeys-verb-send = verzenden
apikeys-verb-delete = verwijderen
apikeys-summary-mail = mail
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = geen acties
apikeys-summary-nothing = niets
apikeys-summary-none = geen
apikeys-summary-verbs-on = { $verbs } op { $surfaces }
apikeys-summary-all-accounts = alle accounts
apikeys-summary-accounts = accounts: { $ids }
apikeys-summary-all-folders = alle mappen
apikeys-summary-folders = mappen: { $ids }
apikeys-summary-expires = verloopt op { $date }
apikeys-summary-rate = { $n } verz./min
apikeys-summary-ips = IP's: { $ips }
apikeys-summary-mcp-tools = MCP-tools: { $tools }
apikeys-summary-unattended = ONBEHEERD verzenden

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = MCP-sleutels
apikeys-mcp-heading = MCP-sleutels
apikeys-mcp-intro = Geef een AI-agent (via het Model Context Protocol) precies de tools die je kiest. Berichtteksten die naar de agent gaan, worden gemarkeerd als niet-vertrouwde invoer. Elke tool wordt afzonderlijk toegekend.
apikeys-mcp-label-aria = Label van de MCP-sleutel
apikeys-mcp-label-placeholder = bijv. assistent-agent
apikeys-mcp-create = MCP-sleutel maken
apikeys-mcp-reveal-warning = Kopieer dit geheim nu — het wordt één keer getoond.
apikeys-mcp-error-need-label = geef de MCP-sleutel een label
apikeys-mcp-error-need-tool = ken minstens één tool toe
apikeys-mcp-error-create = de MCP-sleutel kon niet worden gemaakt

apikeys-tools = Tools
apikeys-tools-group = MCP-tools
apikeys-outbox-suffix = (via Outbox)

apikeys-unattended-aria = Onbeheerd verzenden
apikeys-unattended-allow = Onbeheerd verzenden toestaan (Outbox overslaan — vereist medeondertekening door een beheerder)
apikeys-unattended-pending = Onbeheerd verzenden blijft inactief totdat een beheerder deze sleutel medeondertekent.
apikeys-unattended-disclosure = Standaard komt een toegestaan verzendverzoek in je Outbox terecht en wacht het tot je het in de app bevestigt — automatisering kan niet zelfstandig verzenden. Onbeheerd verzenden VERWIJDERT die menselijke controlestap, zodat deze sleutel zonder bevestiging mail kan verzenden. Daarnaast moet een beheerder de sleutel medeondertekenen. Geef dit alleen aan automatisering die je volledig vertrouwt; met een gecompromitteerde sleutel voor onbeheerd verzenden kan zonder enige vraag mail namens jou worden verzonden.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Mail doorzoeken
apikeys-tool-mail-search-desc = Berichten doorzoeken. Resultaten worden gemarkeerd als niet-vertrouwde invoer.
apikeys-tool-mail-read-label = Mail lezen
apikeys-tool-mail-read-desc = De tekst van een bericht lezen. Berichtteksten worden gemarkeerd als niet-vertrouwde invoer.
apikeys-tool-folders-list-label = Mappen weergeven
apikeys-tool-folders-list-desc = De mappen van de mailbox weergeven.
apikeys-tool-drafts-create-label = Concepten maken
apikeys-tool-drafts-create-desc = Een conceptbericht maken (wordt nooit automatisch verzonden).
apikeys-tool-mail-send-label = Mail verzenden
apikeys-tool-mail-send-desc = Een bericht in de Outbox plaatsen voor bevestiging in de app.
apikeys-tool-calendar-read-label = Agenda lezen
apikeys-tool-calendar-read-desc = Afspraken in de agenda lezen.
apikeys-tool-calendar-propose-label = Afspraken voorstellen
apikeys-tool-calendar-propose-desc = Afspraken voorstellen (niet vastleggen).
apikeys-tool-tasks-read-label = Taken lezen
apikeys-tool-tasks-read-desc = Taken lezen.
apikeys-tool-tasks-write-label = Taken schrijven
apikeys-tool-tasks-write-desc = Taken maken of bijwerken.
apikeys-tool-contacts-read-label = Contacten lezen
apikeys-tool-contacts-read-desc = Contacten lezen.
apikeys-surface-scripts-label = Sieve-scripts beheren
