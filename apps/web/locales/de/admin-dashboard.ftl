# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = Aktualisieren
admin-dash-unavailable = Die Dashboard-Werte sind noch nicht verfügbar.
admin-dash-unavailable-detail = Dieser Server liefert die Dashboard-Daten nicht (GET /admin/dashboard antwortet mit 404). Sie kommen mit einem späteren Server-Update. Die anderen Bereiche funktionieren wie bisher.
admin-dash-error-network = Das Dashboard konnte nicht geladen werden: Der Server hat nicht geantwortet. Versuchen Sie es erneut.
admin-dash-error-status = Das Dashboard konnte nicht geladen werden: Der Server antwortete mit { $status }. Versuchen Sie es erneut; wenn es sich wiederholt, prüfen Sie das Serverprotokoll.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Handlungsbedarf
admin-dash-warn-fix = Bereich öffnen, in dem sich das beheben lässt
admin-dash-warn-blocked-accounts = { $count ->
    [one] Ein Konto kann sich nicht anmelden, weil seine Domain keine Konfiguration hat.
   *[other] { $count } Konten können sich nicht anmelden, weil ihre Domains keine Konfiguration haben.
}
admin-dash-warn-no-trusted-proxy = IP-Regeln sind aktiv, aber kein vertrauenswürdiger Proxy ist konfiguriert, daher scheint möglicherweise jeder Client von der Adresse des Proxys zu kommen.
admin-dash-warn-allowlist-bypass = Eine Umgebungsvariable schaltet die IP-Freigabeliste ab.
admin-dash-warn-metrics-token = Für den Metrik-Endpunkt ist kein Token gesetzt.
admin-dash-warn-pending-migrations = { $count ->
    [one] Eine Datenbankmigration wurde nicht angewendet. Starten Sie den Server neu, um sie anzuwenden.
   *[other] { $count } Datenbankmigrationen wurden nicht angewendet. Starten Sie den Server neu, um sie anzuwenden.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] Ein Upstream-Zertifikat läuft bald ab (laut dem letzten Verbindungstest).
   *[other] { $count } Upstream-Zertifikate laufen bald ab (laut dem letzten Verbindungstest).
}
admin-dash-warn-unknown = Serverwarnung { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] Ein Konto gesperrt: keine Domain-Konfiguration
   *[other] { $count } Konten gesperrt: keine Domain-Konfiguration
}
admin-dash-blocked-note = Diese Konten haben sich vor diesem Update angemeldet, aber ihre Domain hat keine Konfiguration in der Konsole, daher können sie sich jetzt nicht anmelden. Wird die Domain konfiguriert, können sie sich wieder anmelden.
admin-dash-blocked-chart = Gesperrte Konten pro Domain
admin-dash-blocked-row = { $count ->
    [one] ein Konto
   *[other] { $count } Konten
}
admin-dash-blocked-seen = (zuletzt verwendeter Mailserver { $host })
admin-dash-configure-domain = { $domain } konfigurieren
admin-dash-blocked-no-access = Ihre Rolle darf keine Domains konfigurieren.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Einrichtungs-Checkliste: nichts offen
    [one] Einrichtungs-Checkliste: ein Punkt offen
   *[other] Einrichtungs-Checkliste: { $open } Punkte offen
}
admin-dash-check-done = Erledigt
admin-dash-check-open = Offen
admin-dash-check-optional = (optional)
admin-dash-check-go = Öffnen
admin-dash-check-hide = Ausblenden
admin-dash-check-unhide = Wieder anzeigen
admin-dash-check-hidden = { $count ->
    [one] Ein Punkt ist ausgeblendet. Ausblenden markiert ihn nicht als erledigt.
   *[other] { $count } Punkte sind ausgeblendet. Ausblenden markiert sie nicht als erledigt.
}
admin-dash-check-show-hidden = Ausgeblendete Punkte anzeigen
admin-dash-check-hide-hidden = Ausgeblendete Punkte weglassen
admin-dash-check-domain-exists = Eine Mail-Domain hinzufügen
admin-dash-check-domain-upstream = Der Domain einen Mailserver zuweisen oder einen Standard-Upstream festlegen
admin-dash-check-test-connection = Einen erfolgreichen Verbindungstest ausführen (innerhalb der letzten 7 Tage)
admin-dash-check-preferred-domain = Eine bevorzugte Domain wählen, damit Benutzer sich ohne Eingabe der Domain anmelden können
admin-dash-check-second-admin = Ein zweites Admin-Konto anlegen oder die Anmeldung über die Umgebung abschalten
admin-dash-check-trusted-proxies = Die vertrauenswürdigen Proxys vor diesem Server konfigurieren
admin-dash-check-security-policy = Die Sicherheitsrichtlinie speichern
admin-dash-check-metrics-token = Ein Token für den Metrik-Endpunkt festlegen
admin-dash-check-branding = Das Branding prüfen

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Konten und Sitzungen
admin-dash-users = Angelegte Benutzer
admin-dash-signed-in-24h = Angemeldete Konten, letzte 24 Stunden
admin-dash-signed-in-7d = Angemeldete Konten, letzte 7 Tage
admin-dash-domains = Domains
admin-dash-domains-detail = { $upstream } mit Mailserver, { $disabled } deaktiviert
admin-dash-sessions = Aktive Webmail-Sitzungen
admin-dash-admin-sessions = Aktive Admin-Sitzungen
admin-dash-security = Anmeldesicherheit
admin-dash-failed-webmail = Fehlgeschlagene Webmail-Anmeldungen, letzte 24 Stunden
admin-dash-failed-admin = Fehlgeschlagene Admin-Anmeldungen, letzte 24 Stunden
admin-dash-active-blocks = Gesperrte Adressen
admin-dash-allowlists = IP-Freigabelisten
admin-dash-allowlists-value = Admin { $admin }, Webmail { $webmail }
admin-dash-on = an
admin-dash-off = aus
admin-dash-trusted-proxies = Vertrauenswürdige Proxys
admin-dash-configured = Konfiguriert
admin-dash-not-configured = Nicht konfiguriert
admin-dash-untrusted-forwarded = { $count ->
    [one] Eine Anfrage in 24 Stunden enthielt Weiterleitungs-Header von einer nicht vertrauenswürdigen Gegenstelle
   *[other] { $count } Anfragen in 24 Stunden enthielten Weiterleitungs-Header von einer nicht vertrauenswürdigen Gegenstelle
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Verkehr, letzte { $mins } Minuten
admin-dash-traffic-note = Gezählt von diesem Serverprozess seit seinem Start. Andere Prozesse oder Replikate sind nicht enthalten.
admin-dash-traffic-empty = In diesem Zeitraum wurden keine Anfragen gezählt.
admin-dash-requests = Anfragen pro Minute
admin-dash-errors = Serverfehler (5xx) pro Minute
admin-dash-p95 = Antwortzeit, 95. Perzentil

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Zustand
admin-dash-h-ok = OK
admin-dash-h-attention = Handlungsbedarf
admin-dash-h-db = Datenbank
admin-dash-h-migrations = Migrationen
admin-dash-h-engine = E-Mail-Synchronisierung
admin-dash-h-outbox = Postausgang
admin-dash-h-index = Suchindex
admin-dash-h-sandbox = Render-Sandbox
admin-dash-h-plugins = Plugin-Laufzeit

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Speicher
admin-dash-db-size = Datenbankdatei
admin-dash-wal-size = Write-Ahead-Log
admin-dash-index-size = Suchindex
admin-dash-volume-free = Freier Speicher auf dem Datenvolume
admin-dash-volume-of = von { $total }
admin-dash-na-wal = Diese Datenbank verwendet keine Write-Ahead-Log-Datei, oder der Server hat sie nicht gemeldet.
admin-dash-na-volume = Der Server kann den freien Speicher auf dieser Plattform nicht lesen.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Letzte Admin-Aktionen
admin-dash-audit-empty = Noch keine Admin-Aktionen erfasst.
admin-dash-audit-when = Wann
admin-dash-audit-actor = Wer
admin-dash-audit-action = Aktion
admin-dash-audit-target = Ziel
admin-dash-audit-all = Vollständiges Audit-Protokoll öffnen
admin-dash-server = Server
admin-dash-version = Version
admin-dash-build = Build
admin-dash-mode = Modus
admin-dash-mode-engine = Engine: Dieser Server synchronisiert und speichert E-Mails
admin-dash-mode-proxy = Proxy: Dieser Server leitet Anfragen an den Mailserver weiter
admin-dash-uptime = Läuft seit
admin-dash-base-path = Bereitgestellt unter
admin-dash-base-root = dem Stammverzeichnis der Website
admin-dash-duration-dh = { $d } T { $h } Std.
admin-dash-duration-hm = { $h } Std. { $m } Min.
admin-dash-duration-m = { $m } Min.
admin-dash-duration-s = { $s } s
