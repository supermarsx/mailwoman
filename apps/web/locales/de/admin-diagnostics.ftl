# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Diagnoseseiten
admin-diagx-tab-overview = Übersicht
admin-diagx-tab-server = Server
admin-diagx-tab-connectivity = Konnektivität
admin-diagx-tab-backends = E-Mail-Backends
admin-diagx-tab-posture = Sicherheitslage
admin-diagx-tab-tools = Werkzeuge
admin-diagx-tab-history = Verlauf
admin-diagx-tab-report = Bericht
admin-diagx-tab-server-intro = Prüfungen der eigenen Komponenten dieses Servers: die Datenbank, ihre Migrationen und der Serverschlüssel. Sie laufen mit den Serverprüfungen.
admin-diagx-tab-connectivity-intro = Ob dieser Server unter seinem öffentlichen Namen erreichbar ist, die gespeicherten Backends erreichen kann und private Adressen unerreichbar hält. Die Domain-Werkzeuge unten suchen DNS-Einträge, testen TLS und ermitteln Einstellungen für eine Domain.
admin-diagx-tab-backends-intro = Ein Verbindungs- und Sicherheits-Handshake mit jedem gespeicherten E-Mail-, Versand-, Regel- und DAV-Endpunkt. Es werden keine Zugangsdaten gesendet und keine E-Mails zugestellt.
admin-diagx-tab-posture-intro = Einstellungen, die Anmeldungen und Sitzungen schützen, und wie fail2ban die Adresse einer Anfrage sehen würde.
admin-diagx-tab-tools-intro = Weitere Stellen zur Prüfung des Dienstes und die Grenzen, die für Diagnoseläufe gelten.
admin-diagx-tab-history-intro = Gespeicherte Läufe, neueste zuerst. Öffnen Sie einen Lauf, um seine Ergebnisse und die Änderungen gegenüber dem vorherigen Lauf zu sehen.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Warnung
admin-diagx-status-fail = Fehlgeschlagen
admin-diagx-status-skipped = Übersprungen
admin-diagx-status-not-run = Nicht ausgeführt
admin-diagx-status-running = Läuft
admin-diagx-run = Ausführen
admin-diagx-run-again = Erneut ausführen
admin-diagx-running = Läuft…
admin-diagx-run-server-note = Führt die Serverprüfungen erneut aus. Alle Karten dieses Laufs werden aktualisiert.
admin-diagx-run-backends-note = Prüft alle gespeicherten Backends erneut. Alle Backend-Karten werden aktualisiert.
admin-diagx-how-to-fix = So beheben Sie es
admin-diagx-details = Details
admin-diagx-copy-details = Details kopieren
admin-diagx-copy-value = { $label } kopieren
admin-diagx-copied = Kopiert.
admin-diagx-copy-failed = Der Browser hat das Kopieren nicht erlaubt. Markieren Sie den Text und kopieren Sie ihn stattdessen.
admin-diagx-not-run-yet = Noch nicht ausgeführt.
admin-diagx-not-reported = Der letzte Serverlauf hat diese Prüfung nicht gemeldet; sie gilt für diese Bereitstellung nicht.
admin-diagx-posture-met = Erfüllt
admin-diagx-posture-not-met = Nicht erfüllt
admin-diagx-loading = Wird geladen…
admin-diagx-none = Keine
admin-diagx-yes = Ja
admin-diagx-no = Nein
admin-diagx-open = Öffnen
admin-diagx-row-result = Ergebnis
admin-diagx-row-duration = Dauer
admin-diagx-row-checked = Geprüft
admin-diagx-row-id = Prüfungs-ID
admin-diagx-row-domain = Domain
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protokoll
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Fähigkeiten
admin-diagx-row-credentials = Zugangsdaten gesendet
admin-diagx-row-source = Gefunden in
admin-diagx-row-oauth = Benötigt OAuth
admin-diagx-overview-intro = Das neueste Ergebnis jeder Prüfung, nach Gruppen. „Alle ausführen“ führt die Serverprüfungen aus und prüft dann jedes gespeicherte Backend.
admin-diagx-run-all = Alle Prüfungen ausführen
admin-diagx-run-all-step = Schritt { $step } von { $steps }: { $suite ->
    [backends] gespeicherte Backends
   *[server] Serverprüfungen
}.
admin-diagx-run-all-done = Alle Prüfungen abgeschlossen. Die Ergebnisse sind im Verlauf gespeichert.
admin-diagx-backends-progress = { $completed } von { $total } Backends geprüft.
admin-diagx-counts = { $ok } OK · { $warn } Warnungen · { $fail } fehlgeschlagen · { $notRun } nicht ausgeführt
admin-diagx-attention = Erfordert Aufmerksamkeit
admin-diagx-attention-none = Keine Prüfung hat in ihrem letzten Lauf eine Warnung oder einen Fehler gemeldet.
admin-diagx-never-run = Es wurden noch keine Prüfungen ausgeführt. Führen Sie alle Prüfungen aus, um ein erstes Ergebnis zu erhalten.
admin-diagx-last-server = Letzte Serverprüfungen
admin-diagx-last-backends = Letzte Backend-Prüfungen
admin-diagx-never = Nie
admin-diagx-open-health = Dienstzustand
admin-diagx-check-database-connectivity = Datenbankverbindung
admin-diagx-check-database-connectivity-purpose = Die Datenbank beantwortet eine einfache Abfrage.
admin-diagx-check-database-connectivity-fix = Prüfen Sie, ob die Datenbank läuft und ob MW_DB_PATH oder MW_DATABASE_URL auf sie verweist. Das Serverprotokoll nennt den Verbindungsfehler.
admin-diagx-check-database-write = Datenbankschreibzugriffe
admin-diagx-check-database-write-purpose = Eine Testzeile kann geschrieben und anschließend zurückgerollt werden.
admin-diagx-check-database-write-fix = Prüfen Sie den freien Speicherplatz und ob die Datenbankdatei oder der Benutzer schreiben darf. Ein schreibgeschütztes Volume oder ein volles Laufwerk verhindert Schreibzugriffe.
admin-diagx-check-database-migrations = Datenbankmigrationen
admin-diagx-check-database-migrations-purpose = Die Datenbank enthält alle Migrationen, die diese Serverversion erwartet.
admin-diagx-check-database-migrations-fix = Starten Sie den Server neu, damit er ausstehende Migrationen anwendet, und prüfen Sie sein Protokoll auf eine fehlgeschlagene Migration. Betreiben Sie keinen älteren Server mit einer neueren Datenbank.
admin-diagx-check-server-key-roundtrip = Serverschlüssel
admin-diagx-check-server-key-roundtrip-purpose = Der Serverschlüssel versiegelt und öffnet einen Testwert.
admin-diagx-check-server-key-roundtrip-fix = Prüfen Sie, ob MW_SERVER_KEY (oder die Schlüsseldatei) der Schlüssel ist, mit dem diese Datenbank erstellt wurde. Gespeicherte Geheimnisse lassen sich mit einem anderen Schlüssel nicht öffnen.
admin-diagx-check-other-purpose = Eine von diesem Server gemeldete Prüfung.
admin-diagx-check-other-fix = Öffnen Sie die Details für das Rohergebnis und prüfen Sie das Serverprotokoll zum Zeitpunkt des Laufs.
admin-diagx-check-network-public-dns = Öffentlicher Name
admin-diagx-check-network-public-dns-purpose = Der Hostname in MW_PUBLIC_URL lässt sich von diesem Server aus auflösen.
admin-diagx-check-network-public-dns-fix = Prüfen Sie den DNS-Eintrag des öffentlichen Namens oder setzen Sie MW_PUBLIC_URL auf die Adresse, unter der Benutzer diesen Server erreichen.
admin-diagx-check-network-public-port = Öffentlicher Port
admin-diagx-check-network-public-port-purpose = Der Port der öffentlichen URL nimmt eine Verbindung von diesem Server an.
admin-diagx-check-network-public-port-fix = Prüfen Sie Firewall, Reverse-Proxy und Portweiterleitung. Manche Router können sich nicht mit ihrer eigenen öffentlichen Adresse verbinden (NAT-Hairpinning); testen Sie auch von außerhalb Ihres Netzwerks.
admin-diagx-check-network-backend-ports = Backend-Ports
admin-diagx-check-network-backend-ports-purpose = Host und Port jedes gespeicherten Backends nehmen eine direkte Verbindung an.
admin-diagx-check-network-backend-ports-fix = Prüfen Sie, ob die Backend-Hosts laufen und ob eine Firewall diesem Server den Zugriff erlaubt. „E-Mail-Backends“ zeigt, welcher Endpunkt den Handshake nicht besteht.
admin-diagx-check-egress-private-policy = Schutz privater Adressen
admin-diagx-check-egress-private-policy-purpose = Ausgehende Verbindungen zu Loopback- und privaten Adressen werden standardmäßig abgelehnt.
admin-diagx-check-egress-private-policy-fix = Prüfen Sie die Egress-Einstellungen unter „Datenschutz“. Erlauben Sie private Adressen nur für benannte Backends, die sie benötigen.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Anfragen erreichen diesen Server über HTTPS.
admin-diagx-check-security-https-fix = Stellen Sie die Konsole über HTTPS bereit oder setzen Sie MW_PUBLIC_URL auf https und MW_TRUSTED_PROXIES, wenn ein Reverse-Proxy TLS terminiert.
admin-diagx-check-security-secure-cookies = Sichere Cookies
admin-diagx-check-security-secure-cookies-purpose = Sitzungs-Cookies werden nur über HTTPS gesendet.
admin-diagx-check-security-secure-cookies-fix = Setzen Sie MW_COOKIE_SECURE auf true, sobald der Server über HTTPS erreicht wird.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Browser werden angewiesen, für diese Website HTTPS zu verwenden.
admin-diagx-check-security-hsts-fix = Aktivieren Sie Strict-Transport-Security in den Sicherheitseinstellungen oder senden Sie den Header vom Reverse-Proxy.
admin-diagx-check-security-csrf = CSRF-Schutz
admin-diagx-check-security-csrf-purpose = Anfragen, die Daten ändern, müssen das CSRF-Token enthalten.
admin-diagx-check-security-csrf-fix = Setzen Sie MW_CSRF_STRICT auf true.
admin-diagx-check-security-admin-factors = Zweiter Faktor für Administratoren
admin-diagx-check-security-admin-factors-purpose = Jeder Administrator hat einen zweiten Anmeldefaktor.
admin-diagx-check-security-admin-factors-fix = Bitten Sie jeden Administrator ohne zweiten Faktor, in „Meine Einstellungen“ einen Passkey oder eine Authenticator-App hinzuzufügen, oder verlangen Sie ihn im Sicherheitszentrum.
admin-diagx-check-security-signin-pacing = Wartezeit bei Anmeldewiederholung
admin-diagx-check-security-signin-pacing-purpose = Auf eine fehlgeschlagene Anmeldung folgt eine kurze Wartezeit.
admin-diagx-check-security-signin-pacing-fix = Aktivieren Sie die Anmeldedrosselung in der Sicherheitsrichtlinie.
admin-diagx-check-security-credential-locks = Zugangsdatensperren
admin-diagx-check-security-credential-locks-purpose = Wiederholte fehlgeschlagene Anmeldungen sperren die Zugangsdaten für eine Weile.
admin-diagx-check-security-credential-locks-fix = Aktivieren Sie automatische Sperren im Sicherheitszentrum.
admin-diagx-check-security-geography = Daten zur Anmelde-Geografie
admin-diagx-check-security-geography-purpose = Die Richtlinie zur Anmelde-Geografie verfügt über die benötigten Länderdaten.
admin-diagx-check-security-geography-fix = Installieren oder aktualisieren Sie die Länderdatenbank oder deaktivieren Sie die Geografie-Richtlinie.
admin-diagx-check-security-render-isolation = Isolierte Nachrichtendarstellung
admin-diagx-check-security-render-isolation-purpose = Nachrichten werden im separaten, isolierten Worker dargestellt.
admin-diagx-check-security-render-isolation-fix = Prüfen Sie, ob MW_RENDER_BIN auf das Programm mw-render verweist und ob es startet. Der Dienstzustand im Dashboard zeigt den Status des Workers.
admin-diagx-check-security-deployment = Bereitstellungseinstellungen
admin-diagx-check-security-deployment-purpose = Die Sicherheitseinstellungen der Bereitstellung können gelesen werden.
admin-diagx-check-security-deployment-fix = Prüfen Sie die Datenbankverbindung und öffnen Sie dann die Sicherheitsbewertung der Bereitstellung.
admin-diagx-check-security-plaintext-backends = Passwörter über unverschlüsselte Verbindungen
admin-diagx-check-security-plaintext-backends-purpose = Kein gespeichertes Backend akzeptiert Passwörter ohne TLS.
admin-diagx-check-security-plaintext-backends-fix = Bearbeiten Sie die Verbindungen der Domain und wählen Sie TLS oder STARTTLS. Erlauben Sie unverschlüsselte Verbindungen nur in einem privaten Netzwerk unter Ihrer Kontrolle.
admin-diagx-check-fail2ban-client-address = fail2ban-Client-Adresse
admin-diagx-check-fail2ban-client-address-purpose = Die Adresse, die eine fail2ban-Protokollzeile nennen würde, ist die tatsächliche Adresse des Clients.
admin-diagx-check-fail2ban-client-address-fix = Setzen Sie hinter einem Reverse-Proxy MW_TRUSTED_PROXIES auf die Adresse des Proxys, sonst sperrt fail2ban den Proxy statt des Clients. Eine Loopback-Adresse ist zu erwarten, wenn Sie die Konsole auf dem Server selbst öffnen.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = { $protocol }-Endpunkt: Verbindungs- und Sicherheits-Handshake, ohne Anmeldung.
admin-diagx-backend-other-purpose = Ein Hinweis aus dem Backend-Lauf.
admin-diagx-backend-fix = Prüfen Sie Hostname, Port und TLS-Modus, die für dieses Backend gespeichert sind, ob DNS auflöst und ob das Zertifikat für den Hostnamen gültig ist. Führen Sie die Prüfung dann erneut aus.
admin-diagx-backend-config-fix = Öffnen Sie die Verbindungen der Domain und speichern Sie sie erneut; die gespeicherte Backend-Liste konnte nicht gelesen werden.
admin-diagx-backend-results = Backend-Prüfungen
admin-diagx-backends-not-run = Die gespeicherten Backends wurden noch nicht geprüft.
admin-diagx-backends-run-at = Zuletzt geprüft { $date }: { $count } Ergebnisse.
admin-diagx-failover-title = Failover
admin-diagx-failover-purpose = Ob eine Domain auf ein anderes E-Mail-Backend wechseln darf, wenn die erste Wahl nicht erreichbar ist.
admin-diagx-failover-state = Failover { $state ->
    [on] ein
   *[off] aus
} · { $backends } E-Mail-Backends{ $enabled ->
    [no] {" "}· Domain deaktiviert
   *[yes] {""}
}
admin-diagx-no-domains = Keine gespeicherten Domains.
admin-diagx-domain-tools = Domain-Werkzeuge
admin-diagx-domain-tools-purpose = Einmalige Prüfungen für eine gespeicherte Domain. Sie melden sich nicht an und senden keine E-Mails.
admin-diagx-domain = Domain
admin-diagx-tool-dns = DNS-Einträge
admin-diagx-tool-dns-purpose = Sucht die MX-, SPF-, DMARC- und Dienst-Einträge (SRV) der Domain.
admin-diagx-tool-dns-fix = Fügen Sie die fehlenden Einträge bei Ihrem DNS-Anbieter hinzu. Ohne MX empfängt die Domain keine E-Mails; ohne SPF und DMARC können andere Server ihre E-Mails ablehnen.
admin-diagx-srv-found = { $found } von { $total } Diensteinträgen gefunden
admin-diagx-tool-tls = TLS und Handshake
admin-diagx-tool-tls-purpose = Verbindet sich mit jedem gespeicherten Endpunkt der Domain und meldet TLS-Version, Zertifikatsablauf und benötigte Zeit.
admin-diagx-tool-tls-fix = Prüfen Sie bei einem nicht erreichbaren Endpunkt Host, Port und Firewall. Erneuern Sie ein Zertifikat, das innerhalb von 14 Tagen abläuft.
admin-diagx-probe-none = Die Domain hat keine gespeicherten Endpunkte zum Testen.
admin-diagx-probe-line = { $host } · erreichbar: { $reachable ->
    [yes] ja
   *[no] nein
} · TLS { $tls } · Zertifikat { $days ->
    [none] nicht gemeldet
   *[other] läuft in { $days } Tagen ab
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = Einstellungen ermitteln
admin-diagx-tool-detect-purpose = Sucht veröffentlichte E-Mail-Einstellungen (Autoconfig, Autodiscover, SRV) für einen Domainnamen. Es wird nichts gespeichert.
admin-diagx-tool-detect-fix = Veröffentlichen Sie Autoconfig- oder SRV-Einträge für die Domain oder geben Sie die Einstellungen unter „Domains“ manuell ein.
admin-diagx-detect-name = Nachzuschlagender Domainname
admin-diagx-tool-health = Dienstzustand
admin-diagx-tool-health-purpose = E-Mail-Synchronisierung, Postausgang, Suchindex, Darstellungs-Worker und Plugins im Dashboard.
admin-diagx-tool-deployment-purpose = Jede Sicherheitseinstellung der Bereitstellung, die dieser Server beobachten kann, mit der Einstellung, die sie steuert.
admin-diagx-tool-ssh = SSH-Tunnel-Tests
admin-diagx-tool-ssh-purpose = Testen Sie die SSH-Tunnel, über die Backends in einem anderen Netzwerk erreicht werden.
admin-diagx-tool-geo = Anmelde-Geografie
admin-diagx-tool-geo-purpose = Woher Anmeldungen kommen und die Länderdatenbank hinter der Geografie-Richtlinie.
admin-diagx-catalogue-purpose = Was jede Diagnose-Suite abdeckt und welche Grenzen jeder Lauf einhält.
admin-diagx-suite-unsupported = Nicht verfügbar: { $reason }
admin-diagx-suite-server = Serverprüfungen
admin-diagx-suite-backends = Gespeicherte Backends
admin-diagx-run-title = { $suite ->
    [backends] Gespeicherte Backends
   *[server] Serverprüfungen
}, { $date }
admin-diagx-changes = Änderungen
admin-diagx-changes-first = Kein früherer Lauf dieser Art zum Vergleich.
admin-diagx-changes-none = Seit dem Lauf vom { $date } hat sich nichts geändert.
admin-diagx-changes-since = Geändert seit dem Lauf vom { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Fehlerbehebungsbericht
admin-diagx-report-purpose = Ein JSON-Bericht für den Support: Laufzeit-Flags, die neuesten Ergebnisse und der von Ihnen gewählte Lauf. Passwörter, Tokens, Cookies, Schlüssel, Roh-Konfiguration und E-Mails sind nie enthalten.
admin-diagx-report-run = Einzubeziehender Lauf
admin-diagx-report-preview = Vorschau
admin-crumb-server = Server
admin-crumb-connectivity = Konnektivität
admin-crumb-posture = Sicherheitslage
admin-crumb-tools = Werkzeuge
admin-crumb-history = Verlauf
admin-crumb-report = Bericht
admin-diagx-table-actions = Aktionen
admin-diagx-check-pdfjs-viewer-assets = PDF-Viewer-Dateien
admin-diagx-check-pdfjs-viewer-assets-purpose = Die Dateien des PDF-Viewers sind vollständig vorhanden, korrekt typisiert und passen zu diesem Server-Build.
admin-diagx-check-pdfjs-viewer-assets-fix = Stellen Sie den Web-Build erneut bereit oder entfernen Sie den Ordner pdfjs aus MW_WEB_DIR, damit die im Server integrierte Kopie verwendet wird. Schlägt die Prüfung fehl, statt nur zu warnen, bauen Sie das Server-Image neu.
