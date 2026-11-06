# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Eingehender Zugriff
admin-inbound-help = Beschränken Sie vertrauenswürdige Quell-IPs für die Admin-Konsole und für Webmail getrennt. Beide Richtlinien sind zunächst deaktiviert. Diese Regeln ändern weder vertrauenswürdige Proxys noch API-Scopes oder ausgehenden Zugriff.
admin-inbound-unscoped = Diese bereitstellungsweite Richtlinie erfordert einen Administrator mit uneingeschränktem Sicherheitszugriff.
admin-inbound-admin = Admin-Konsole und API
admin-inbound-admin-help = Umfasst die Anmeldung und diesen Editor. Eine bestehende Administrator-Authentifizierung umgeht die Netzwerkrichtlinie nicht.
admin-inbound-webmail = Webmail und Konto-APIs
admin-inbound-webmail-help = Umfasst E-Mail, DAV, signierte Inhalte, SSO, OIDC und API-Routen. Bereits hochgestufte Verbindungen behalten ihre Handshake-Entscheidung.
admin-inbound-enabled = Diesen Bereich auf die Zulassungsliste beschränken
admin-inbound-cidrs = Notfall-IPs und CIDRs
admin-inbound-cidrs-help = Eine konkrete IP oder ein CIDR pro Zeile, bis zu 128. Bei aktivierter Richtlinie ist mindestens eine erforderlich. Nehmen Sie eine vertrauenswürdige Quelle auf, die Sie für die Wiederherstellung nutzen können.
admin-inbound-hostnames = Erlaubte DNS-Hostnamen
admin-inbound-hostnames-help = Ein DNS-Name in Kleinbuchstaben und ASCII pro Zeile. Verwenden Sie für internationale Namen IDNA-ASCII. Keine URLs, Ports, Platzhalter, IP-Literale oder abschließenden Punkte. Namen gewähren nur die aktuell aufgelösten Adressen.
admin-inbound-dns = Grenzen der DNS-Auflösung
admin-inbound-dns-help = Neue, abgelaufene und fehlgeschlagene Namen gewähren keinen Zugriff. Erfolgreiche Antworten beachten ihre DNS-TTL und die konfigurierte Cache-Obergrenze. Anfragen warten nie auf die DNS-Auflösung.
admin-inbound-servers = Rekursive DNS-Server
admin-inbound-servers-help = Leer lassen für die System-Resolver oder bis zu vier IP:Port-Sockets eingeben, einen pro Zeile (zum Beispiel 10.0.0.53:53 oder [fd00::53]:53). Ausdrücklich angegebene Server ersetzen die System-Resolver ohne öffentlichen Fallback. Das Vertrauen in den Resolver liegt in Ihrer Verantwortung; eine unabhängige DNSSEC-Validierung wird nicht zugesichert.
admin-inbound-dns-timeoutMs = Zeitlimit der Abfrage (Millisekunden)
admin-inbound-dns-cacheSeconds = Maximale Cache-Lebensdauer (Sekunden)
admin-inbound-dns-maxHosts = Maximale Anzahl unterschiedlicher Hostnamen
admin-inbound-dns-maxAddresses = Maximale Anzahl Adressen pro Hostname
admin-inbound-dns-concurrency = Gleichzeitige Abfragen
admin-inbound-bounds = Zulässiger Bereich: { $min }–{ $max }.
admin-inbound-lockout = Diese Richtlinie kann Ihren eigenen Zugriff sofort blockieren, einschließlich dieses Editors. Vergewissern Sie sich, dass die Notfall-IPs Ihre tatsächliche Quelle abdecken, wie sie die konfigurierten vertrauenswürdigen Proxys ermitteln. Es gibt keine automatische Ausnahme für Loopback oder angemeldete Administratoren. Die Wiederherstellung nach einer Aussperrung erfordert eine Wiederherstellung auf Bereitstellungsseite; eine normale Anmeldung kann sie nicht umgehen.
admin-inbound-confirm-title = Richtlinie für eingehenden Zugriff anwenden?
admin-inbound-confirm-word = ANWENDEN
admin-inbound-save = Bestätigte Richtlinie anwenden
admin-inbound-review = Änderungen prüfen
admin-inbound-refresh = Gespeicherte Richtlinie und DNS-Status aktualisieren
admin-inbound-reset = Entwurf auf die geladene Richtlinie zurücksetzen
admin-inbound-saved = Der Server hat die Richtlinie für eingehenden Zugriff akzeptiert.
admin-inbound-conflict = Die gespeicherte Richtlinie hat sich während der Bearbeitung geändert. Setzen Sie den Entwurf zurück und aktualisieren Sie, um die aktuelle Revision zu laden, bevor Sie erneut bearbeiten.
admin-inbound-no-revision = Der Server hat keine Richtlinienrevision zurückgegeben. Aktualisieren Sie vor dem Speichern.
admin-inbound-status = Aktuelle DNS-Freigaben
admin-inbound-status-help = Dies ist eine Momentaufnahme der letzten Statusaktualisierung, kein Countdown und keine Konnektivitätsprüfung. Aktualisieren Sie, um die aktuellen Freigaben zu sehen. Ein Fehler kann neben einer noch gültigen früheren Antwort bis zu deren ursprünglichem Ablauf bestehen.
admin-inbound-refreshing = Eine begrenzte DNS-Aktualisierung läuft.
admin-inbound-idle = Beim letzten Lesen des Status lief keine DNS-Aktualisierung.
admin-inbound-no-hosts = In aktivierten Richtlinien sind keine Hostnamen aktiv.
admin-inbound-hostname = Hostname
admin-inbound-state = Zugriffsfreigabe
admin-inbound-addresses = Nutzbare Adressen
admin-inbound-validity = Gültigkeit bei der letzten Aktualisierung
admin-inbound-error = DNS-Ergebniscode
admin-inbound-usable = Nutzbar
admin-inbound-unavailable = Keine aktuelle Freigabe
admin-inbound-seconds = { $seconds } Sekunden
admin-inbound-on = Aktiviert
admin-inbound-off = Deaktiviert
admin-inbound-invalid-dns = Die DNS-Grenzen oder die Anzahl der Resolver liegen außerhalb der zulässigen Grenzen.
admin-inbound-rescue-required = Jeder aktivierte Bereich erfordert mindestens eine Notfall-IP oder ein CIDR.
admin-inbound-too-many = Die Richtlinie überschreitet die konfigurierte Grenze für Hostnamen oder konkrete Adressen.
admin-inbound-invalid-host = Verwenden Sie DNS-Namen in Kleinbuchstaben und ASCII ohne URLs, Ports, Platzhalter, IP-Literale oder abschließende Punkte.
