# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Vernieuwen
admin-dash-unavailable = De dashboardcijfers zijn nog niet beschikbaar.
admin-dash-unavailable-detail = Deze server levert de dashboardgegevens niet (GET /admin/dashboard antwoordt met 404). Ze komen met een latere serverupdate. De andere onderdelen werken zoals voorheen.
admin-dash-error-network = Het dashboard kon niet worden geladen: de server antwoordde niet. Probeer het opnieuw.
admin-dash-error-status = Het dashboard kon niet worden geladen: de server antwoordde met { $status }. Probeer het opnieuw; controleer het serverlogboek als het blijft gebeuren.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Vraagt aandacht
admin-dash-warn-fix = Het onderdeel openen waarin je dit oplost
admin-dash-warn-blocked-accounts = { $count ->
    [one] Eén account kan niet inloggen omdat het domein ervan geen configuratie heeft.
   *[other] { $count } accounts kunnen niet inloggen omdat hun domeinen geen configuratie hebben.
}
admin-dash-warn-no-trusted-proxy = Er zijn IP-regels actief maar er is geen vertrouwde proxy ingesteld, dus elke client kan van het adres van de proxy lijken te komen.
admin-dash-warn-allowlist-bypass = Een omgevingsvariabele schakelt de IP-toegangslijst uit.
admin-dash-warn-metrics-token = Voor het metrics-endpoint is geen token ingesteld.
admin-dash-warn-pending-migrations = { $count ->
    [one] Eén databasemigratie is niet toegepast. Start de server opnieuw om hem toe te passen.
   *[other] { $count } databasemigraties zijn niet toegepast. Start de server opnieuw om ze toe te passen.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Eén upstreamcertificaat verloopt binnenkort (volgens de laatste verbindingstest).
   *[other] { $count } upstreamcertificaten verlopen binnenkort (volgens de laatste verbindingstest).
}
admin-dash-warn-unknown = Serverwaarschuwing { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] Eén account geblokkeerd: geen domeinconfiguratie
   *[other] { $count } accounts geblokkeerd: geen domeinconfiguratie
}
admin-dash-blocked-note = Deze accounts logden vóór deze update in, maar hun domein heeft geen configuratie in de console, dus ze kunnen nu niet inloggen. Als je het domein configureert, kunnen ze weer inloggen.
admin-dash-blocked-chart = Geblokkeerde accounts per domein
admin-dash-blocked-row = { $count ->
    [one] één account
   *[other] { $count } accounts
}
admin-dash-blocked-seen = (laatst gebruikte mailhost { $host })
admin-dash-configure-domain = { $domain } configureren
admin-dash-blocked-no-access = Je rol kan geen domeinen configureren.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Installatiechecklist: niets open
    [one] Installatiechecklist: één item open
   *[other] Installatiechecklist: { $open } items open
}
admin-dash-check-done = Gereed
admin-dash-check-open = Open
admin-dash-check-optional = (optioneel)
admin-dash-check-go = Openen
admin-dash-check-hide = Verbergen
admin-dash-check-unhide = Weer tonen
admin-dash-check-hidden = { $count ->
    [one] Eén item is verborgen. Verbergen markeert het niet als gereed.
   *[other] { $count } items zijn verborgen. Verbergen markeert ze niet als gereed.
}
admin-dash-check-show-hidden = Verborgen items tonen
admin-dash-check-hide-hidden = Verborgen items weglaten
admin-dash-check-domain-exists = Een maildomein toevoegen
admin-dash-check-domain-upstream = Het domein een mailserver geven, of een standaard-upstream instellen
admin-dash-check-test-connection = Een geslaagde verbindingstest uitvoeren (in de afgelopen 7 dagen)
admin-dash-check-preferred-domain = Een voorkeursdomein kiezen, zodat gebruikers kunnen inloggen zonder het te typen
admin-dash-check-second-admin = Een tweede beheerdersaccount maken, of het inloggen via de omgeving uitschakelen
admin-dash-check-trusted-proxies = De vertrouwde proxy's vóór deze server configureren
admin-dash-check-security-policy = Het beveiligingsbeleid opslaan
admin-dash-check-metrics-token = Een token instellen voor het metrics-endpoint
admin-dash-check-branding = De huisstijl controleren

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Accounts en sessies
admin-dash-users = Ingerichte gebruikers
admin-dash-signed-in-24h = Ingelogde accounts, afgelopen 24 uur
admin-dash-signed-in-7d = Ingelogde accounts, afgelopen 7 dagen
admin-dash-domains = Domeinen
admin-dash-domains-detail = { $upstream } met een mailserver, { $disabled } uitgeschakeld
admin-dash-sessions = Actieve webmailsessies
admin-dash-admin-sessions = Actieve beheersessies
admin-dash-security = Inlogbeveiliging
admin-dash-failed-webmail = Mislukte webmail-inlogpogingen, afgelopen 24 uur
admin-dash-failed-admin = Mislukte beheerders-inlogpogingen, afgelopen 24 uur
admin-dash-active-blocks = Geblokkeerde adressen
admin-dash-allowlists = IP-toegangslijsten
admin-dash-allowlists-value = Beheer { $admin }, webmail { $webmail }
admin-dash-on = aan
admin-dash-off = uit
admin-dash-trusted-proxies = Vertrouwde proxy's
admin-dash-configured = Geconfigureerd
admin-dash-not-configured = Niet geconfigureerd
admin-dash-untrusted-forwarded = { $count ->
    [one] Eén verzoek in 24 uur bevatte doorstuurheaders van een niet-vertrouwde peer
   *[other] { $count } verzoeken in 24 uur bevatten doorstuurheaders van een niet-vertrouwde peer
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Verkeer, afgelopen { $mins } minuten
admin-dash-traffic-note = Geteld door dit serverproces sinds de start. Andere processen of replica's zijn niet meegeteld.
admin-dash-traffic-empty = In deze periode zijn geen verzoeken geteld.
admin-dash-requests = Verzoeken per minuut
admin-dash-errors = Serverfouten (5xx) per minuut
admin-dash-p95 = Responstijd, 95e percentiel

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Status
admin-dash-h-ok = OK
admin-dash-h-attention = Vraagt aandacht
admin-dash-h-db = Database
admin-dash-h-migrations = Migraties
admin-dash-h-engine = Mailsynchronisatie
admin-dash-h-outbox = Outbox
admin-dash-h-index = Zoekindex
admin-dash-h-sandbox = Weergavesandbox
admin-dash-h-plugins = Plug-in-runtime

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Opslag
admin-dash-db-size = Databasebestand
admin-dash-wal-size = Write-ahead log
admin-dash-index-size = Zoekindex
admin-dash-volume-free = Vrije ruimte op het datavolume
admin-dash-volume-of = van { $total }
admin-dash-na-wal = Deze database gebruikt geen write-ahead-logbestand, of de server heeft dit niet gemeld.
admin-dash-na-volume = De server kan op dit platform de vrije ruimte niet uitlezen.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Recente beheeracties
admin-dash-audit-empty = Er zijn nog geen beheeracties vastgelegd.
admin-dash-audit-when = Wanneer
admin-dash-audit-actor = Wie
admin-dash-audit-action = Actie
admin-dash-audit-target = Doel
admin-dash-audit-all = Het volledige auditlogboek openen
admin-dash-server = Server
admin-dash-version = Versie
admin-dash-build = Build
admin-dash-mode = Modus
admin-dash-mode-engine = Engine: deze server synchroniseert en bewaart mail
admin-dash-mode-proxy = Proxy: deze server geeft verzoeken door aan de mailserver
admin-dash-uptime = Draait al
admin-dash-base-path = Bereikbaar op
admin-dash-base-root = de hoofdmap van de site
admin-dash-duration-dh = { $d } d { $h } u
admin-dash-duration-hm = { $h } u { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
