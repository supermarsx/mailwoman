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

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = Eine Länderrichtlinie ist aktiv, aber es ist keine nutzbare Länderdatenbank geladen, daher werden die davon erfassten Anmeldungen abgelehnt.
admin-dash-warn-geoip-stale = Eine Länderrichtlinie ist aktiv, und die Länderdatenbank ist veraltet.
admin-dash-h-status-ok = In Ordnung
admin-dash-h-status-warn = Warnung
admin-dash-h-status-fail = Fehlerhaft
admin-dash-h-status-disabled = Deaktiviert
admin-dash-h-checked = Geprüft um { $time }
admin-dash-h-refresh = Erneut prüfen
admin-dash-h-refresh-failed = Die Zustandsprüfung konnte nicht wiederholt werden (Status { $status }). Das vorherige Ergebnis wird angezeigt.
admin-dash-h-refresh-network = Die Zustandsprüfung konnte nicht wiederholt werden: Der Server hat nicht geantwortet. Das vorherige Ergebnis wird angezeigt.
admin-dash-h-enable = So schalten Sie es ein
admin-dash-h-probe-time = Die Prüfung dauerte { $time }
admin-dash-h-outbox-probe-time = Die Statusaggregation dauerte { $time }; die Dauer des Übermittlungs-Handshakes wird separat angezeigt.
admin-dash-h-history = Letzte { $n } Prüfungen
admin-dash-h-open-diagnostics = Diagnose
admin-dash-h-open-logs = Protokolle
admin-dash-h-open-plugins = Plugins
admin-dash-h-open-maintenance = Wartung
admin-dash-h-not-tracked = Nicht erfasst
admin-dash-h-unlimited = keine Begrenzung
admin-dash-h-m-none = keine
admin-dash-h-m-ago = vor { $age }
admin-dash-h-m-ping = Umlaufzeit
admin-dash-h-m-pool = Verbindungspool
admin-dash-h-m-pool-value = { $inUse } in Verwendung, { $idle } im Leerlauf ({ $size } offen)
admin-dash-h-m-engine = Datenbankserver
admin-dash-h-m-location = Speicherort
admin-dash-h-m-db-size = Datenbankgröße
admin-dash-h-m-wal-size = Write-Ahead-Log
admin-dash-h-m-journal = Journalmodus
admin-dash-h-m-slow = Langsame Abfragen (letzte Stunde)
admin-dash-h-m-applied = Angewendet
admin-dash-h-m-applied-value = { $applied } von { $expected } in diesem Build
admin-dash-h-m-latest = Zuletzt angewendet
admin-dash-h-m-pending = Noch nicht angewendet
admin-dash-h-m-failed = Gestartet, nicht abgeschlossen
admin-dash-h-m-unknown = Aus einem neueren Build
admin-dash-h-m-accounts = Postfächer
admin-dash-h-m-accounts-value = { $total } registriert: { $syncing } synchronisieren, { $idle } im Leerlauf, { $erroring } fehlerhaft
admin-dash-h-m-last-sync = Seit der letzten abgeschlossenen Synchronisierung
admin-dash-h-m-last-sync-value = Median { $p50 }, längste { $max }
admin-dash-h-m-never-synced = Seit dem Serverstart nicht synchronisiert
admin-dash-h-m-queue = Laufende Synchronisierungen
admin-dash-h-m-errors-hour = Synchronisierungsfehler (letzte Stunde)
admin-dash-h-m-by-kind = Verbindungen nach Backend
admin-dash-h-m-kind-value = { $kind }: { $accounts } Postfächer, { $watch } Überwachungsverbindungen
admin-dash-h-m-recent-errors = Letzte Fehler
admin-dash-h-m-queued = Warten auf den Versand
admin-dash-h-m-queued-value = { $queued } fällig, { $deferred } werden erneut versucht, { $sending } werden gesendet
admin-dash-h-m-oldest = Am längsten wartend
admin-dash-h-m-scheduled = Geplant oder zurückgehalten
admin-dash-h-m-scheduled-value = { $scheduled } geplant, { $held } zurückgehalten
admin-dash-h-m-failed-total = Fehlgeschlagen (noch in einem Postausgang)
admin-dash-h-m-failed-total-value = { $failed } fehlgeschlagen, { $uncertain } mit unbekanntem Zustellstatus
admin-dash-h-m-sends-hour = Sendungen (letzte Stunde)
admin-dash-h-m-sends-hour-value = { $sent } gesendet, { $deferred } zurückgestellt, { $failed } fehlgeschlagen
admin-dash-h-m-last-send = Letzter Versand
admin-dash-h-m-last-send-value = { $outcome }, vor { $age }
admin-dash-h-m-last-send-none = Keiner seit dem Serverstart
admin-dash-h-outcome-sent = Gesendet
admin-dash-h-outcome-deferred = Zurückgestellt
admin-dash-h-outcome-failed = Fehlgeschlagen
admin-dash-h-outcome-uncertain = Zustellstatus unbekannt
admin-dash-h-m-submission = Übermittlungsserver
admin-dash-h-m-submission-value = { $reachable } von { $checked } haben geantwortet (nur Handshake, keine Anmeldung; langsamster { $slowest }), geprüft vor { $age }
admin-dash-h-m-submission-none = Es ist kein Übermittlungsserver konfiguriert.
admin-dash-h-m-submission-pending = Verbindungsprüfung läuft.
admin-dash-h-m-submission-unmeasured = { $count } Server erfordern eine authentifizierte Prüfung beim Anbieter; es wurden keine Zugangsdaten gesendet.
admin-dash-h-m-documents = Dokumente
admin-dash-h-m-documents-value = { $documents } von { $messages } gespeicherten Nachrichten
admin-dash-h-m-index-pending = Noch nicht indiziert
admin-dash-h-m-index-size = Größe auf dem Datenträger
admin-dash-h-m-index-memory = Im Arbeitsspeicher
admin-dash-h-m-last-commit = Letzter Commit
admin-dash-h-m-rebuild = Neuaufbau
admin-dash-h-m-rebuild-running = Läuft: { $pct } ({ $done } von { $total })
admin-dash-h-m-rebuild-idle = Läuft nicht
admin-dash-h-m-version = Worker-Version
admin-dash-h-m-worker-missing = Keine Worker-Binärdatei
admin-dash-h-m-render-ping = Ping (Start, Jail, Antwort)
admin-dash-h-m-render-jobs = Aktuell laufende Aufträge
admin-dash-h-m-render-failures = Probleme (letzte Stunde)
admin-dash-h-m-render-failures-value = { $failed } fehlgeschlagen, { $timeouts } mit Zeitüberschreitung, { $busy } wegen Auslastung abgewiesen
admin-dash-h-m-jail = Kernel-Jail
admin-dash-h-m-jail-required = Erforderlich
admin-dash-h-m-jail-optional = Auf dieser Plattform nicht erforderlich
admin-dash-h-m-jail-degraded = Eingeschränkt: { $why }
admin-dash-h-m-plugins = Plugins
admin-dash-h-m-plugins-value = { $installed } installiert, { $enabled } aktiviert, { $loaded } geladen
admin-dash-h-m-plugin-list = Aktivierte Plugins
admin-dash-h-m-plugin-loaded = geladen
admin-dash-h-m-plugin-not-loaded = nicht geladen ({ $reason })
admin-dash-h-m-plugin-limits = Speicher { $memory }, Frist { $deadline }, Fuel { $fuel }
admin-dash-h-m-plugin-calls = Aufruffehler (letzte Stunde)
admin-dash-h-m-plugin-calls-value = { $traps } Traps, { $limits } Grenzüberschreitungen, { $errors } sonstige Fehler
admin-dash-h-m-host-ping = Sperre des Plugin-Hosts
admin-dash-h-t-ping = Warnung über { $warn }, Fehler über { $fail }
admin-dash-h-t-oldest = Warnung, wenn eine Nachricht länger als { $warn } wartet, Fehler ab { $fail }
admin-dash-h-t-pending = Warnung bei mehr als { $n } nicht indizierten Nachrichten
admin-dash-h-t-render = Warnung über { $warn }; Fehler, wenn innerhalb von { $timeout } keine Antwort kommt
admin-dash-h-t-sync = Fehler, wenn { $pct } oder mehr der Postfächer nicht synchronisiert werden können
admin-dash-h-chart-db = Datenbank-Umlaufzeit
admin-dash-h-chart-render = Ping des Render-Workers
admin-dash-h-chart-outbox = Nachrichten, die auf den Versand warten
admin-dash-h-chart-search = Nicht indizierte Nachrichten
admin-dash-na-index-proxy = Proxy-Modus: Dieser Server führt keinen Suchindex.
admin-dash-na-index-memory = Der Suchindex wird im Arbeitsspeicher gehalten und hat daher keine Größe auf dem Datenträger.
admin-dash-customize = Dashboard anpassen
admin-dash-limitations = Datenverfügbarkeit
