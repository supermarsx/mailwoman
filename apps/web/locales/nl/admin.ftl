# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman-beheer
admin-nav = Beheeronderdelen
admin-sign-out = Uitloggen
admin-nav-domains = Domeinen
admin-nav-users = Gebruikers
admin-nav-integrations = Integraties
admin-nav-observability = Monitoring
admin-nav-plugins = Plug-ins
admin-nav-assist = AI-assistent
admin-nav-sso = Single sign-on
admin-nav-servermeta = Servermetadata
admin-nav-rethread = Onderhoud
admin-nav-2fa = Tweestapsverificatie vereisen

# Shared admin actions / states
admin-delete = Verwijderen
admin-revoke = Intrekken
admin-remove = Verwijderen
admin-saved = Opgeslagen.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Inloggen als beheerder
admin-login-note = Dit paneel draait onder een aparte beheersessie. De sessie eindigt na 30 minuten zonder activiteit, en uiterlijk 12 uur na het inloggen.
admin-login-session-ended = Je beheersessie is beëindigd. Log opnieuw in.
admin-login-username = Gebruikersnaam beheerder
admin-login-password = Wachtwoord
admin-login-sign-in = Inloggen
admin-login-signing-in = Inloggen…
admin-login-invalid = Ongeldige inloggegevens voor beheerder
admin-login-unreachable = Kan de server niet bereiken

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domeinen
admin-domains-load-error = Kan de domeinen niet laden
admin-domains-save-error = Kan het domein niet opslaan
admin-domains-delete-error = Kan het domein niet verwijderen
admin-domains-add = Domein toevoegen
admin-domains-name = Domeinnaam
admin-domains-name-placeholder = example.com
admin-domains-note = Een domein wordt op naam geregistreerd. Geregistreerde namen worden aangeboden op het scherm Tweestapsverificatie vereisen wanneer een regel voor één domein geldt. Het registreren van een domein stuurt geen mail door en filtert geen mail.
admin-domains-save = Domein opslaan
admin-domains-empty = Nog geen domeinen.
admin-domains-delete-for = { $name } verwijderen

# -- Users -------------------------------------------------------------------
admin-users-title = Gebruikers
admin-users-load-error = Kan de gebruikers niet laden
admin-users-provision-error = Kan de gebruiker niet inrichten
admin-users-flag-error = Kan de markering niet bijwerken
admin-users-revoke-error = Kan de sessies niet intrekken
admin-users-provision = Gebruiker inrichten
admin-users-username = Gebruikersnaam
admin-users-domain = Domein
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Quotum in bytes (0 = onbeperkt)
admin-users-quota-msgs = Quotum in berichten (0 = onbeperkt)
admin-users-empty = Nog geen gebruikers.
admin-users-col-account = Account
admin-users-col-quota = Quotum (bytes/berichten)
admin-users-col-zeroaccess = Zero-access
admin-users-col-flags = Markeringen
admin-users-col-sessions = Sessies
admin-users-zeroaccess-for = Zero-access voor { $account }
admin-users-zeroaccess-help = Alleen een registratie. Zero-access-opslag wordt door de gebruiker zelf aan- en uitgezet, in de eigen instellingen, met de eigen sleutel. Dit vakje aanvinken versleutelt de opgeslagen mail van het account niet, en het uitvinken ontsleutelt die niet.
admin-users-disable-for = { $account } uitschakelen
admin-users-disabled = uitgeschakeld
admin-users-disabled-help = Blokkeert het inloggen en beëindigt elke open sessie van dit account. De API-sleutels en tokens ervan worden geweigerd. Dit schakelt de mailbox op de mailserver niet uit: andere mailprogramma's kunnen er nog bij totdat je hem daar ook uitschakelt.
admin-users-force-change-for = Wachtwoordwijziging afdwingen voor { $account }
admin-users-force-change = wijziging afdwingen
admin-users-force-change-help = De gebruiker wordt vastgehouden op een scherm om het wachtwoord te wijzigen, bij de volgende keer inloggen en in sessies die al open zijn, en kan pas bij de mail als de wijziging lukt. Hiervoor is een werkende backend voor wachtwoordwijziging nodig (MW_PASSWD_BACKEND). Zonder backend kan de wijziging niet lukken en blijft de gebruiker op dat scherm totdat je dit vakje uitvinkt.
admin-users-revoke-for = Sessies intrekken voor { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integraties
admin-integrations-load-error = Kan de integraties niet laden
admin-integrations-revoke-error = Kan de sleutel niet intrekken
admin-integrations-ldap = LDAP-/GAL-directory
admin-integrations-nextcloud = Nextcloud-koppeling
admin-integrations-active = Actief
admin-integrations-configured = Geconfigureerd
admin-integrations-not-configured = Niet geconfigureerd
admin-integrations-unknown = Status onbekend
admin-integrations-config-note = “Geconfigureerd” betekent dat deze installatie instellingen voor de integratie heeft: een ingeschakelde LDAP-directoryvermelding in de database, of de drie omgevingsvariabelen MW_NEXTCLOUD_*. Beide worden gelezen wanneer de server start. Het betekent niet dat er contact met de externe dienst is gemaakt of dat die bereikbaar is.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = Geen webhooks geregistreerd.
admin-integrations-keys = API- en MCP-sleutels
admin-integrations-keys-empty = Geen sleutels uitgegeven.
admin-integrations-col-prefix = Prefix
admin-integrations-col-account = Account
admin-integrations-col-scopes = Reikwijdte
admin-integrations-col-status = Status
admin-integrations-status-revoked = ingetrokken
admin-integrations-status-active = actief
admin-integrations-revoke-key = Sleutel { $prefix } intrekken
admin-integrations-col-unattended = Onbeheerd verzenden
admin-integrations-unattended-not-requested = Niet aangevraagd
admin-integrations-unattended-requested = Aangevraagd, niet goedgekeurd
admin-integrations-unattended-approved = Goedgekeurd
admin-integrations-unattended-approve = Goedkeuren
admin-integrations-unattended-withdraw = Intrekken
admin-integrations-unattended-approve-key = Onbeheerd verzenden goedkeuren voor sleutel { $prefix }
admin-integrations-unattended-withdraw-key = Goedkeuring van onbeheerd verzenden intrekken voor sleutel { $prefix }
admin-integrations-unattended-approve-title = Onbeheerd verzenden goedkeuren voor deze sleutel?
admin-integrations-unattended-withdraw-title = De goedkeuring voor deze sleutel intrekken?
admin-integrations-unattended-owner = Eigenaar: { $account }
admin-integrations-unattended-key = Sleutel: { $prefix }
admin-integrations-unattended-scope = Rechten: { $permissions }. MCP-tools: { $tools }.
admin-integrations-unattended-scope-unreadable = De reikwijdte van deze sleutel kon niet worden gelezen.
admin-integrations-scope-read = lezen
admin-integrations-scope-send = verzenden
admin-integrations-scope-delete = verwijderen
admin-integrations-scope-mail = mail
admin-integrations-scope-pim = agenda, taken, notities en contacten
admin-integrations-scope-none = geen
admin-integrations-unattended-approve-meaning = Berichten die met deze sleutel via MCP worden verzonden, worden verstuurd zonder dat een persoon ze vrijgeeft.
admin-integrations-unattended-withdraw-meaning = Een verzending met deze sleutel via MCP wordt geweigerd totdat de sleutel opnieuw is goedgekeurd. Het bericht wordt niet verstuurd en niet in de Outbox van de eigenaar geplaatst.
admin-integrations-unattended-approve-effect = De goedkeuring geldt direct, vanaf de volgende verzending met de sleutel. Een herstart is niet nodig.
admin-integrations-unattended-withdraw-effect = De intrekking geldt direct: vanaf de volgende verzending verzendt de sleutel niet meer onbeheerd. Een herstart is niet nodig.
admin-integrations-unattended-cancel = Annuleren
admin-integrations-unattended-saving = Opslaan…
admin-integrations-unattended-approved-done = Onbeheerd verzenden goedgekeurd voor sleutel { $prefix }. Dit geldt nu.
admin-integrations-unattended-withdrawn-done = Goedkeuring ingetrokken voor sleutel { $prefix }. Deze sleutel verzendt niet meer onbeheerd.
admin-integrations-unattended-error-gone = Sleutel { $prefix } bestaat niet meer of is ingetrokken. Er is niets gewijzigd.
admin-integrations-unattended-error-conflict = Sleutel { $prefix } is ingetrokken of de eigenaar heeft geen onbeheerd verzenden aangevraagd. Er is niets goedgekeurd.
admin-integrations-unattended-error = Kan de goedkeuring voor sleutel { $prefix } niet wijzigen.

# -- Observability -----------------------------------------------------------
admin-obs-title = Monitoring
admin-obs-load-error = Kan de monitoringgegevens niet laden
admin-obs-export-error = Kan het auditlogboek niet exporteren
admin-obs-ban-add-error = Kan de blokkade niet toevoegen
admin-obs-unban-error = Kan de blokkade niet verwijderen
admin-obs-telemetry-note = Logging en telemetrie worden hier niet ingesteld. Het logfilter (MW_LOG), de OTLP-collector (MW_OTLP_ENDPOINT) en het metrics-endpoint (MW_METRICS_TOKEN) worden bij het starten uit de omgeving van de server gelezen.
admin-obs-audit = Auditlogboek
admin-obs-export = JSONL exporteren
admin-obs-audit-empty = Geen auditvermeldingen.
admin-obs-col-time = Tijd
admin-obs-col-actor = Uitvoerder
admin-obs-col-action = Actie
admin-obs-col-target = Doel
admin-obs-bans = Inlogmonitor / blokkeerlijst
admin-obs-bans-note = Deze lijst is een registratie. Mailwoman weigert geen verbindingen van de adressen erop, en een vermelding heeft geen effect op het inloggen. Elke mislukte beheerders-inlogpoging schrijft een regel naar het serverlogboek waarop een fail2ban-jail kan reageren, en vijf mislukte pogingen vanaf één adres voegen dat adres hier toe. Het blokkeren gebeurt door die jail of door je firewall.
admin-obs-ban-add = Blokkade toevoegen
admin-obs-ban-ip = IP-adres
admin-obs-ban-reason = Reden
admin-obs-ban-btn = IP blokkeren
admin-obs-bans-empty = Geen actieve blokkades.
admin-obs-unban-for = Blokkade van { $ip } opheffen
admin-obs-unban-btn = Blokkade opheffen

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Plug-ins
admin-plugins-intro = Engine-plug-ins draaien in een WebAssembly-sandbox met rechtencontrole. Keur een plug-in goed voordat hij kan worden ingeschakeld, en geef alleen de rechten die hij nodig heeft.
admin-plugins-empty = Er zijn geen plug-ins geregistreerd.
admin-plugins-signed = Ondertekend
admin-plugins-unsigned = Niet ondertekend
admin-plugins-approved = Goedgekeurd
admin-plugins-enabled = Ingeschakeld
admin-plugins-approve = Goedkeuren
admin-plugins-enable = Inschakelen
admin-plugins-disable = Uitschakelen
admin-plugins-net = net: { $hosts }
admin-plugins-limits = limieten: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = limieten: { $memory } MiB · { $deadline } ms · { $fuel } fuel
admin-plugins-allow-unsigned-for = Niet-ondertekende plug-in { $name } toestaan
admin-plugins-allow-unsigned = Deze niet-ondertekende plug-in laten draaien
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = Toegangslijst voor plug-ins van derden
admin-allowlist-intro = Een component van derden (niet van het project zelf) wordt pas geladen nadat een beheerder de exacte SHA-256-digest ervan heeft goedgekeurd. De digest hieronder berekent de server over de bytes van de component op schijf; goedkeuren legt precies die bytes vast. Eigen componenten zijn in de build vastgelegd en worden hier niet beheerd.
admin-allowlist-load-error = Kan de toegangslijst voor plug-ins niet laden.
admin-allowlist-present-heading = Componenten op schijf
admin-allowlist-present-empty = Er zijn geen componenten van derden aanwezig. Plaats een component in de map voor plug-ins van derden zodat hij hier verschijnt.
admin-allowlist-digest-label = Berekende SHA-256
admin-allowlist-status-approved = Goedgekeurd
admin-allowlist-status-pending = Niet goedgekeurd
admin-allowlist-status-firstparty = Eigen component
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Toegelaten via vastgelegde digest. Deze component heeft geen handtekening; de goedkeuring vertrouwt precies de bytes waarvan de digest wordt getoond, wat de verwachte situatie is voor een component die op deze manier is goedgekeurd.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = Krachtige rechten ({ $caps }) kunnen niet aan een plug-in van derden worden toegekend. De server weigert ze ongeacht de goedkeuring; ze zijn alleen beschikbaar voor eigen componenten.
admin-allowlist-firstparty-note = Deze id komt overeen met een eigen component. De vastlegging van de eigen component gaat altijd voor, dus deze id kan niet als component van derden worden goedgekeurd.
admin-allowlist-approve = Digest goedkeuren
admin-allowlist-approve-for = Digest goedkeuren voor { $id }
admin-allowlist-revoke = Intrekken
admin-allowlist-revoke-for = Vastlegging intrekken voor { $id }
admin-allowlist-uninstall = Verwijderen
admin-allowlist-uninstall-for = { $id } verwijderen
admin-allowlist-pins-heading = Goedgekeurde en ingetrokken vastleggingen
admin-allowlist-pins-empty = Geen vastleggingen geregistreerd.
admin-allowlist-pin-approved-by = Goedgekeurd door { $by } op { $at }
admin-allowlist-pin-revoked = Ingetrokken
admin-allowlist-cancel = Annuleren

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Deze component goedkeuren om te laden?
admin-allowlist-approve-detail = Goedkeuren legt precies de bytes vast waarvan de SHA-256 hieronder wordt getoond. Na goedkeuring worden voor deze id alleen bytes geladen die met deze digest overeenkomen; elke wijziging aan de component levert een andere digest op en wordt pas weer geladen na een nieuwe goedkeuring. Goedkeuring kent op zichzelf geen rechten toe.
admin-allowlist-approve-confirm = Digest goedkeuren

# Revoke confirmation.
admin-allowlist-revoke-title = Deze vastlegging intrekken?
admin-allowlist-revoke-detail = Intrekken verwijdert de goedkeuring voor deze digest en schakelt de plug-in uit. Dit geldt bij de volgende keer laden; een instantie die al draait, wordt tot dan niet gestopt.
admin-allowlist-revoke-confirm = Vastlegging intrekken

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Deze plug-in verwijderen?
admin-allowlist-uninstall-detail = Verwijderen wist de opgeslagen sleutel/waarde-gegevens van de plug-in voor elk account, verwijdert de vastleggingen in de toegangslijst en schakelt de plug-in uit. Het componentbestand op schijf wordt niet gewist; het kan later opnieuw worden goedgekeurd.
admin-allowlist-uninstall-confirm = Plug-in verwijderen

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Single sign-on
admin-sso-intro = Configureer OIDC- en SAML 2.0-inlogbackends. Ingeschakelde backends verschijnen als knoppen “Inloggen met…” op het inlogscherm van de mailbox, voor de hele installatie of voor één domein.
admin-sso-add = Een inlogbackend toevoegen
admin-sso-edit = Bewerken
admin-sso-create = Backend toevoegen
admin-sso-update = Wijzigingen opslaan
admin-sso-cancel = Annuleren
admin-sso-empty = Geen SSO-backends geconfigureerd.
admin-sso-load-error = Kan de SSO-backends niet laden.
admin-sso-save-error = Kan de backend niet opslaan.
admin-sso-delete-error = Kan de backend niet verwijderen.

# Common fields
admin-sso-id = Backend-id
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Weergavenaam
admin-sso-display-name-placeholder = Inloggen met Acme SSO
admin-sso-kind = Protocol
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Bereik
admin-sso-scope-deployment = Hele installatie
admin-sso-scope-domain = Eén domein
admin-sso-domain = Domein
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Ingeschakeld
admin-sso-first-login = Beleid bij eerste keer inloggen
admin-sso-policy-allowlist = Toegangslijst (onbekende gebruikers weigeren)
admin-sso-policy-autocreate = Accounts automatisch aanmaken bij eerste keer inloggen

# OIDC fields
admin-sso-issuer = Issuer-URL
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = Client-id
admin-sso-client-secret = Clientgeheim
admin-sso-secret-unchanged = Leeg laten om het opgeslagen geheim te behouden
admin-sso-redirect = Redirect-URL
admin-sso-scopes = Scopes
admin-sso-metadata = SP-metadata

# SAML fields
admin-sso-sp-entity-id = SP-entity-id
admin-sso-acs-url = ACS-URL
admin-sso-idp-metadata-url = URL van IdP-metadata
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML van IdP-metadata
admin-sso-idp-metadata-xml-placeholder = Plak de XML van de IdP-metadata, of gebruik de URL hierboven
admin-sso-idp-sso-url = SSO-URL van de IdP
admin-sso-idp-slo-url = Uitlog-URL (SLO) van de IdP
admin-sso-idp-certs = Ondertekeningscertificaten van de IdP (PEM)
admin-sso-idp-certs-placeholder = Eén PEM-certificaat per blok, gescheiden door een lege regel
admin-sso-nameid-format = NameID-indeling
admin-sso-want-signed = Ondertekende assertions vereisen
admin-sso-want-encrypted = Versleutelde assertions vereisen

# Claim map
admin-sso-claims = Claimtoewijzing
admin-sso-claim-email = E-mailclaim
admin-sso-claim-username = Gebruikersnaamclaim
admin-sso-claim-display = Weergavenaamclaim
admin-sso-claim-groups = Groepenclaim

# List row
admin-sso-badge-enabled = Ingeschakeld
admin-sso-badge-disabled = Uitgeschakeld
admin-sso-enable = Inschakelen
admin-sso-disable = Uitschakelen
admin-sso-delete = Verwijderen
# `name` is the backend's admin-set display name.
admin-sso-enable-for = { $name } inschakelen
admin-sso-disable-for = { $name } uitschakelen
admin-sso-delete-for = { $name } verwijderen

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = Servermetadata
admin-servermeta-intro = Bekijk en bewerk RFC 5464-serverannotaties voor een ingericht account. Wijzigingen worden rechtstreeks naar de mailserver geschreven, die bepaalt of het account ze mag instellen.
admin-servermeta-account = Account
admin-servermeta-select-option = Selecteer een account…
admin-servermeta-select-prompt = Selecteer een account om de serverannotaties ervan te bekijken en te bewerken.
admin-servermeta-load-error = Kan de lijst met accounts niet laden.
admin-servermeta-no-accounts = Er zijn geen accounts ingericht.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = Gesprekken in mailbox opnieuw groeperen
admin-rethread-intro = Voert de gespreksgroepering (JWZ) opnieuw uit over de opgeslagen mail van een ingericht account en bepaalt de groepering van gesprekken opnieuw. Dit is een eenmalige onderhoudsactie, niet iets dat automatisch draait.
admin-rethread-account = Account
admin-rethread-select-option = Selecteer een account…
admin-rethread-no-accounts = Er zijn geen accounts ingericht.
admin-rethread-load-error = Kan de lijst met accounts niet laden.
admin-rethread-run = Gesprekken opnieuw groeperen
admin-rethread-confirm-title = Gesprekken in deze mailbox opnieuw groeperen?
admin-rethread-confirm-warning = Opnieuw groeperen bepaalt de gespreksgroepering voor dit account opnieuw. Bestaande gesprekken kunnen worden samengevoegd of gesplitst, en gebruikers kunnen gesprekken anders gegroepeerd zien.
admin-rethread-confirm-detail = Dit is een eenmalige onderhoudsactie. Ze wordt nu één keer uitgevoerd; opnieuw uitvoeren is veilig en er wordt geen mail verwijderd.
admin-rethread-confirm = Gesprekken opnieuw groeperen
admin-rethread-running = Opnieuw groeperen…
admin-rethread-cancel = Annuleren
admin-rethread-summary = { $messages } bericht(en) opnieuw gegroepeerd in { $threads } gesprek(ken) over { $accounts } account(s); { $reassigned } bericht(en) naar een ander gesprek verplaatst.
admin-rethread-error = Het opnieuw groeperen is mislukt. Als de server het verzoek heeft geweigerd, is er geen gespreksgroepering gewijzigd; controleer de serverlogboeken en probeer het opnieuw.

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = Zoekindex
admin-searchindex-intro = Zoeken gebruikt een index die uit de opgeslagen mail is opgebouwd. De server bouwt die bij het starten opnieuw op als ze niet overeenkomt met de opgeslagen mail. Mail van zero-access-accounts wordt niet geïndexeerd, dus zoeken vindt niets voor die accounts.
admin-searchindex-count = { $documents } van { $messages } opgeslagen bericht(en) zijn geïndexeerd.
admin-searchindex-on-disk = De index wordt op schijf bewaard.
admin-searchindex-in-memory = De index wordt in het geheugen bewaard en bij elke start van de server opnieuw opgebouwd.
admin-searchindex-progress = Er loopt een herbouw: { $done } van { $total } bericht(en) klaar.
admin-searchindex-run = Zoekindex opnieuw opbouwen
admin-searchindex-running = Opnieuw opbouwen…
admin-searchindex-summary = { $messages } opgeslagen bericht(en) gelezen in { $accounts } account(s): { $indexed } geïndexeerd, { $removed } uit de index verwijderd, { $failed } konden niet worden gelezen. { $zeroAccess } zero-access-account(s) niet geïndexeerd.
admin-searchindex-busy = Er loopt al een herbouw. Wacht tot die klaar is.
admin-searchindex-error = De herbouw is mislukt. De index behoudt wat erin stond; controleer de serverlogboeken en probeer het opnieuw.
admin-searchindex-status-error = Kan de status van de zoekindex niet lezen.
admin-searchindex-unavailable = Deze server draait in proxymodus en heeft geen zoekindex.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
admin-2fa-title = Tweestapsverificatie vereisen
admin-2fa-intro = Vereis een tweede factor (passkey of authenticator-app) bij het inloggen. Elke gebruiker kan zelf een factor instellen; als je het hier vereist, moeten accounts binnen het bereik bij de volgende keer inloggen een factor instellen.
admin-2fa-load-error = Kan het beleid voor tweestapsverificatie niet laden.
admin-2fa-save-error = Kan het beleid voor tweestapsverificatie niet opslaan.
admin-2fa-global = Tweestapsverificatie vereisen voor de hele installatie
admin-2fa-global-label = Tweestapsverificatie vereisen voor de hele installatie
admin-2fa-global-note = Als dit aan staat, moet elk account een tweede factor hebben. Met een regel per domein hieronder kun je het ook voor één domein vereisen zonder het overal te vereisen.
admin-2fa-domains-heading = Vereisten per domein
admin-2fa-domains-empty = Geen vereisten per domein ingesteld.
admin-2fa-col-domain = Domein
admin-2fa-col-require = Vereist
admin-2fa-require-for = Tweestapsverificatie vereisen voor { $domain }
admin-2fa-add-domain = Domein
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Tweestapsverificatie vereisen voor dit domein
admin-2fa-add-require-label = Tweestapsverificatie vereisen voor dit domein
admin-2fa-add-save = Domeinregel toevoegen

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Uitgaand verkeer
admin-egress-heading = Uitgaande proxyroutes
admin-egress-intro = Leid de uitgaande verzoeken van de server via een HTTP CONNECT- of SOCKS5-proxy. Mailwoman past nog steeds zijn eigen adresbeleid toe — de proxy wordt nooit gevraagd een naam om te zetten.
admin-egress-empty = Geen routes geconfigureerd. Uitgaande verzoeken gaan rechtstreeks.
admin-egress-none-active = Er is geen route in gebruik. Uitgaande verzoeken gaan rechtstreeks. Een opgeslagen route draagt pas verkeer nadat je “Deze route gebruiken” kiest.
admin-egress-in-use = Uitgaande verzoeken gebruiken de route “{ $id }”.
admin-egress-deactivate = Geen proxy meer gebruiken
admin-egress-activate = Deze route gebruiken
admin-egress-activate-for = Route { $id } gebruiken
admin-egress-activate-error = Kan niet overschakelen naar de route
admin-egress-deactivate-error = Kan het gebruik van de route niet stoppen
admin-egress-col-state = Status
admin-egress-state-active = In gebruik
admin-egress-state-inactive = Niet in gebruik
admin-egress-test-scope = Alleen de route die in gebruik is, kan worden getest. Een test haalt één vaste URL op via de route en meldt hoe ver die kwam; er wordt niets gewijzigd.
admin-egress-probe-url = Opgehaald: { $url }
admin-egress-load-error = Kan de uitgaande routes niet laden
admin-egress-save-error = Kan de route niet opslaan
admin-egress-delete-error = Kan de route niet verwijderen
admin-egress-invalid = Een id, een host en een poort groter dan nul zijn verplicht.
admin-egress-username-required = Een wachtwoord vereist een gebruikersnaam; zonder gebruikersnaam zou de route zich als niemand aanmelden.
admin-egress-col-id = Id
admin-egress-col-route = Route
admin-egress-col-username = Gebruikersnaam
admin-egress-col-credentials = Wachtwoord
admin-egress-col-plaintext = Onversleuteld
admin-egress-col-actions = Acties
admin-egress-cred-set = Ingesteld
admin-egress-cred-none = Geen
admin-egress-plaintext-on = Toegestaan
admin-egress-plaintext-off = Geweigerd
admin-egress-add-heading = Een route toevoegen
admin-egress-edit-heading = Route bewerken
admin-egress-id = Id
admin-egress-scheme = Schema
admin-egress-host = Host
admin-egress-port = Poort
admin-egress-username = Gebruikersnaam
admin-egress-password = Wachtwoord
admin-egress-password-unchanged = Leeg laten om het opgeslagen wachtwoord te behouden
admin-egress-allow-plaintext = Onversleutelde http-bestemmingen via deze route toestaan
admin-egress-allow-plaintext-note = Standaard uit. Een https-bestemming zorgt ervoor dat een proxy het verkeer dat hij doorgeeft niet kan lezen; als je http toestaat, ziet de proxy het wel.
admin-egress-test = Testen
admin-egress-testing = Testen…
admin-egress-test-failed = De test kon niet worden uitgevoerd, dus over deze route is niets bekend.
admin-egress-outcome-connected = Verbonden
admin-egress-outcome-auth-rejected = Aanmelding geweigerd
admin-egress-outcome-refused-by-policy = Geweigerd door beleid
admin-egress-outcome-dns-failed = Naam kon niet worden omgezet
admin-egress-outcome-unreachable = Onbereikbaar
admin-egress-outcome-origin-tls-failed = TLS naar bestemming mislukt
admin-egress-outcome-route-invalid = Route is niet geldig
admin-egress-outcome-unknown = Onbekend resultaat
admin-egress-stage-dns = gestopt bij het omzetten van de naam
admin-egress-stage-connect = gestopt bij het verbinden met de proxy
admin-egress-stage-tunnel = gestopt bij het openen van de tunnel
admin-egress-stage-origin = gestopt bij de bestemming
admin-egress-stage-unknown = gestopt in een onbekende fase
admin-egress-proxied-yes = Via de proxy gegaan
admin-egress-proxied-no = Niet via de proxy gegaan
