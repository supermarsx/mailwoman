admin-cache-title = Cache-Wartung
admin-cache-help = Zwischengespeicherte E-Mail-Inhalte verfallen standardmäßig nach 60 Tagen ohne Nutzung. Beim Öffnen einer abgelaufenen Nachricht wird ihr Inhalt erneut heruntergeladen. Originalnachrichten, Entwürfe und geschützte lokale Inhalte bleiben erhalten.
admin-cache-expiry = Lebensdauer zwischengespeicherter Nachrichten
admin-cache-orphans = Nicht referenzierte Bodies und Speicherfreigabe
admin-cache-advanced = Batch-Grenzen und Zeitplanung
admin-cache-settings-help = Änderungen unterliegen den üblichen Richtlinien, Prüfungen und Audit-Protokollen der Administratoreinstellungen.
admin-cache-expiry-enabled = Zwischengespeicherte Nachrichteninhalte automatisch verfallen lassen
admin-cache-retention-days = Tage ohne Nutzung bis zum Ablauf
admin-cache-expiry-status = Ablauf zwischengespeicherter Bodies
admin-cache-expiry-result = Geprüft: { $scanned } · Gelöste Cache-Verweise: { $detached } · Geschützt: { $protected }
admin-cache-expiry-help = Nur Bodies, deren Wiederherstellung aus einem vorgelagerten Postfach bestätigt wurde, sind berechtigt. Scans und Kopfzeilenaktualisierungen verlängern die Inaktivitätsfrist nicht.
admin-cache-inspect = Cache-Speicher prüfen
admin-cache-inspect-help = Öffnen Sie die Datenbankübersicht, um zwischengespeicherte Nutzdaten, referenzierte Bodies und verwaiste Kopien zu messen.
admin-cache-no-status = In diesem Serverprozess wurde noch kein Ablauf-Batch abgeschlossen.
admin-cache-settings-unavailable = Die Cache-Einstellungen konnten nicht geladen werden.
admin-cache-change-log = Synchronisierungsverlauf
admin-cache-change-log-help = Alte Änderungsdatensätze verfallen standardmäßig nach 60 Tagen. Clients mit älteren Synchronisierungstokens erhalten eine Antwort zur vollständigen Neusynchronisierung.
admin-cache-no-sources = Für kein verbundenes Postfach ist derzeit ein erneuter Abruf bestätigt. Geschützte Inhalte bleiben erhalten.
admin-cache-wal-recycled = Zugewiesener WAL-Speicher wurde wiederverwendbar gemacht; die Bereinigung kann fortgesetzt werden.
admin-cache-wal-recovery = Die Bereinigung wartet auf WAL-Wiederherstellung oder Checkpointing. Ausstehende Journal-Daten bleiben erhalten.

admin-cache-removed-payload = Entfernter Cache-Inhalt
admin-cache-oversized = Bereinigung großer Nachrichtentexte verschoben
admin-cache-change-removed = Entfernte Einträge im Synchronisierungsverlauf

admin-cache-parsed-copies = Gelöschte Kopien verarbeiteter Nachrichteninhalte
admin-cache-parsed-bytes = Entfernte Daten verarbeiteter Nachrichteninhalte
admin-cache-change-chunked = In Teilmengen bereinigte Synchronisationsstände
