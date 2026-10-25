# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = API-Schlüssel
apikeys-heading = API-Schlüssel
apikeys-intro = Erstellen Sie eingeschränkte Schlüssel für Skripte und Integrationen. Jeder Schlüssel wird gehasht gespeichert, nur einmal angezeigt und kann einzeln widerrufen werden. Vergeben Sie den kleinsten Umfang, der ausreicht.

apikeys-label = Bezeichnung
apikeys-label-aria = Schlüsselbezeichnung
apikeys-label-placeholder = z. B. Backup-Skript

apikeys-create = Schlüssel erstellen
apikeys-saved = Ich habe ihn gespeichert

apikeys-reveal-warning = Kopieren Sie dieses Geheimnis jetzt — es wird nur einmal angezeigt und kann nicht erneut abgerufen werden.
apikeys-copy = Kopieren
apikeys-copy-aria = Geheimnis in die Zwischenablage kopieren
apikeys-copied = In die Zwischenablage kopiert

apikeys-error-need-label = Geben Sie dem Schlüssel eine Bezeichnung, damit Sie ihn später wiedererkennen
apikeys-error-create = Der Schlüssel konnte nicht erstellt werden
apikeys-error-revoke = Der Schlüssel konnte nicht widerrufen werden

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Vorhandene Schlüssel
apikeys-existing = Vorhandene Schlüssel
apikeys-none = Noch keine Schlüssel.
apikeys-created = erstellt am { $date }
apikeys-last-used = zuletzt verwendet am { $date }
apikeys-never-used = nie verwendet
apikeys-revoked-at = widerrufen am { $date }
apikeys-revoke = Widerrufen
apikeys-status-active = Aktiv
apikeys-status-revoked = Widerrufen

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Umfang
apikeys-capabilities = Berechtigungen
apikeys-cap-read = Lesen
apikeys-cap-send = Senden
apikeys-cap-delete = Löschen
apikeys-surface = Bereich
apikeys-surface-mail-label = E-Mail
apikeys-surface-pim-label = PIM (Kalender / Aufgaben / Notizen / Kontakte)
apikeys-accounts = Konten
apikeys-all-accounts = Alle Konten
apikeys-folders = Ordner
apikeys-all-folders = Alle Ordner
apikeys-folder-ids = Ordner-IDs (durch Kommas getrennt)
apikeys-folder-subset-aria = Ordnerauswahl
apikeys-constraints = Einschränkungen
apikeys-expiry = Ablauf (RFC 3339, leer = kein Ablauf)
apikeys-expiry-aria = Ablauf
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Ratenbegrenzung (Anfragen/Min., leer = unbegrenzt)
apikeys-rate-limit-aria = Ratenbegrenzung
apikeys-ip-allowlist = IP-Zulassungsliste (CIDR/IP, durch Kommas getrennt; leer = alle)
apikeys-ip-allowlist-aria = IP-Zulassungsliste
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = lesen
apikeys-verb-send = senden
apikeys-verb-delete = löschen
apikeys-summary-mail = E-Mail
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = keine Aktionen
apikeys-summary-nothing = nichts
apikeys-summary-none = keine
apikeys-summary-verbs-on = { $verbs } für { $surfaces }
apikeys-summary-all-accounts = alle Konten
apikeys-summary-accounts = Konten: { $ids }
apikeys-summary-all-folders = alle Ordner
apikeys-summary-folders = Ordner: { $ids }
apikeys-summary-expires = läuft ab am { $date }
apikeys-summary-rate = { $n } Anfr./Min.
apikeys-summary-ips = IPs: { $ips }
apikeys-summary-mcp-tools = MCP-Werkzeuge: { $tools }
apikeys-summary-unattended = UNBEAUFSICHTIGTES Senden

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = MCP-Schlüssel
apikeys-mcp-heading = MCP-Schlüssel
apikeys-mcp-intro = Gewähren Sie einem KI-Agenten (über das Model Context Protocol) genau die Werkzeuge, die Sie auswählen. An den Agenten zurückgegebene E-Mail-Texte werden als nicht vertrauenswürdige Eingabe gekennzeichnet. Jedes Werkzeug wird einzeln gewährt.
apikeys-mcp-label-aria = Bezeichnung des MCP-Schlüssels
apikeys-mcp-label-placeholder = z. B. Assistenz-Agent
apikeys-mcp-create = MCP-Schlüssel erstellen
apikeys-mcp-reveal-warning = Kopieren Sie dieses Geheimnis jetzt — es wird nur einmal angezeigt.
apikeys-mcp-error-need-label = Geben Sie dem MCP-Schlüssel eine Bezeichnung
apikeys-mcp-error-need-tool = Gewähren Sie mindestens ein Werkzeug
apikeys-mcp-error-create = Der MCP-Schlüssel konnte nicht erstellt werden

apikeys-tools = Werkzeuge
apikeys-tools-group = MCP-Werkzeuge
apikeys-outbox-suffix = (über den Postausgang)

apikeys-unattended-aria = Unbeaufsichtigtes Senden
apikeys-unattended-allow = Unbeaufsichtigtes Senden erlauben (umgeht den Postausgang — erfordert Gegenzeichnung durch einen Administrator)
apikeys-unattended-pending = Unbeaufsichtigtes Senden bleibt inaktiv, bis ein Administrator diesen Schlüssel gegenzeichnet.
apikeys-unattended-disclosure = Standardmäßig landet ein gewährter Sendevorgang in Ihrem Postausgang und wartet, bis Sie ihn in der App bestätigen — eine Automatisierung kann nicht selbstständig senden. Unbeaufsichtigtes Senden ENTFERNT diesen Schritt der menschlichen Kontrolle, sodass dieser Schlüssel E-Mails ohne Bestätigung senden kann. Zusätzlich muss ein Administrator den Schlüssel gegenzeichnen. Gewähren Sie es nur einer Automatisierung, der Sie voll vertrauen; ein kompromittierter Schlüssel mit unbeaufsichtigtem Senden kann ohne Rückfrage E-Mails in Ihrem Namen senden.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = E-Mails durchsuchen
apikeys-tool-mail-search-desc = Nachrichten durchsuchen. Ergebnisse werden als nicht vertrauenswürdige Eingabe gekennzeichnet.
apikeys-tool-mail-read-label = E-Mails lesen
apikeys-tool-mail-read-desc = Einen Nachrichtentext lesen. Texte werden als nicht vertrauenswürdige Eingabe gekennzeichnet.
apikeys-tool-folders-list-label = Ordner auflisten
apikeys-tool-folders-list-desc = Postfachordner auflisten.
apikeys-tool-drafts-create-label = Entwürfe erstellen
apikeys-tool-drafts-create-desc = Einen Nachrichtenentwurf erstellen (wird nie automatisch gesendet).
apikeys-tool-mail-send-label = E-Mails senden
apikeys-tool-mail-send-desc = Eine Nachricht zur Bestätigung in der App in den Postausgang stellen.
apikeys-tool-calendar-read-label = Kalender lesen
apikeys-tool-calendar-read-desc = Kalendertermine lesen.
apikeys-tool-calendar-propose-label = Termine vorschlagen
apikeys-tool-calendar-propose-desc = Kalendertermine vorschlagen (nicht festlegen).
apikeys-tool-tasks-read-label = Aufgaben lesen
apikeys-tool-tasks-read-desc = Aufgaben lesen.
apikeys-tool-tasks-write-label = Aufgaben schreiben
apikeys-tool-tasks-write-desc = Aufgaben erstellen oder aktualisieren.
apikeys-tool-contacts-read-label = Kontakte lesen
apikeys-tool-contacts-read-desc = Kontakte lesen.
