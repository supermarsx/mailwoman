# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Identitätsanbieter
admin-identity-description = Genehmigten Anwendungen die Anmeldung mit Mailwoman erlauben. Dies ist getrennt von externen Anmeldeanbietern und dem Zugriff auf die Mail-API. Der Autorisierungscode mit PKCE ist erforderlich; andere Grant-Typen sind nicht verfügbar.
admin-identity-configure = Aussteller konfigurieren
admin-identity-new = Identitäts-Client hinzufügen
admin-identity-edit = Client bearbeiten
admin-identity-enabled = Aktiviert
admin-identity-disabled = Deaktiviert
admin-identity-all = Alle
admin-identity-discovery = Discovery-Dokument öffnen
admin-identity-search = Clients suchen
admin-identity-status = Status
admin-identity-clients = Identitäts-Clients
admin-identity-client = Client-Identität
admin-identity-client-id = Client-ID
admin-identity-name = Name
admin-identity-method = Client-Authentifizierung
admin-identity-public = Öffentlicher Client (PKCE, kein Geheimnis)
admin-identity-pkce = Jeder Client muss PKCE S256 verwenden. Geheimnisse vertraulicher Clients werden hier erzeugt und nur einmal angezeigt.
admin-identity-rotate-secret = Das vorhandene Client-Geheimnis ersetzen
admin-identity-redirects = Registrierte Ziele
admin-identity-redirect-help = Ein exakter HTTPS-URI pro Zeile; numerisches Loopback-HTTP ist für lokale Anwendungen erlaubt. Platzhalter und Fragmente werden abgelehnt.
admin-identity-callbacks = Autorisierungs-Callback-URIs
admin-identity-logout-uris = Weiterleitungs-URIs nach der Abmeldung
admin-identity-scopes = Erlaubte Identitäts-Scopes
admin-identity-actions = Aktionen
admin-identity-delete = Client löschen
admin-identity-delete-confirm = Identitätszugriff für { $name } löschen? Bestehende Identitäts-Grants funktionieren nicht mehr, und die reine Identitäts-Client-Registrierung wird entfernt.
admin-identity-empty = Keine passenden Identitäts-Clients.
admin-identity-cancel = Abbrechen
admin-identity-save = Speichern
admin-identity-done = Fertig
admin-identity-secret = Erzeugtes Client-Geheimnis
admin-identity-secret-once = Kopieren Sie dieses Geheimnis jetzt in die abhängige Anwendung. Es kann nicht erneut angezeigt werden. Legen Sie ein vertrauliches Geheimnis niemals in Browsercode ab.
admin-identity-issuer = Öffentliche HTTPS-Aussteller-URL
admin-identity-issuer-help = Verwenden Sie den exakten HTTPS-Ursprung dieser Bereitstellung und den Pfad /oidc, einschließlich eines etwaigen Präfixes. Eine Änderung des Ausstellers oder der Konfiguration widerruft bestehende Identitäts-Grants.
admin-identity-rotate-key = Signaturschlüssel rotieren
admin-identity-key-help = Die Rotation erfordert eine frische Administratorverifizierung. Frühere öffentliche Schlüssel bleiben 15 Minuten lang verfügbar, damit kürzlich ausgestellte Token ihre kurze Lebensdauer beenden können.
