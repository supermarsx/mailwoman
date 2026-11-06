# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Löschung und Bereinigung
compliance-card-erasure-coverage = Die Kennungen einer Person in Audit-Verlauf und Protokollen ersetzen und sehen, welche Backups noch gelöschte Personen enthalten.
admin-erasure-title = Löschung und Bereinigung
admin-erasure-intro = „Person bereinigen“ ersetzt die Adresse und die Konto-IDs einer Person im Audit-Verlauf und in den rotierten Protokolldateien durch ein stabiles, mit einem Schlüssel erzeugtes Pseudonym. Die Audit-Hash-Kette wird neu versiegelt, sodass die Verifizierung weiterhin besteht. Jede Löschung und Bereinigung hinterlässt außerdem einen Tombstone, sodass die Wiederherstellung eines älteren Backups diese Personen erneut löscht.
admin-erasure-docs = Das Register darüber, was pro Tabelle gelöscht, exportiert und aufbewahrt wird, finden Sie in docs/admin/erasure-coverage.md.
admin-erasure-scrub = Person bereinigen
admin-erasure-scrub-note = Führen Sie zuerst die Vorschau aus: Sie zählt die Zeilen und Protokollzeilen, die sich ändern würden, und ändert nichts.
admin-erasure-address = Adresse
admin-erasure-preview = Vorschau
admin-erasure-run = Person bereinigen
admin-erasure-dry-note = Nur Vorschau. Es wurde nichts geändert.
admin-erasure-done-note = Bereinigt. Die Person erscheint jetzt als { $pseudonym }.
admin-erasure-nothing = Kein Verlauf nennt diese Person.
admin-erasure-col-table = Tabelle oder Datei
admin-erasure-col-rows = Zeilen
admin-erasure-confirm-title = Diese Person aus dem Verlauf bereinigen?
admin-erasure-confirm-body = Audit-Einträge und Protokollzeilen, die die Person nennen, werden mit dem Pseudonym neu geschrieben. Dies kann nicht rückgängig gemacht werden; die Adresse lässt sich ohne den Schlüssel nicht aus dem Pseudonym wiederherstellen.
admin-erasure-settings = Einstellungen
admin-erasure-keys = Pseudonymschlüssel
admin-erasure-keys-note = Pseudonyme werden mit der aktuellen Schlüsselgeneration erzeugt. Eine Rotation fügt eine Generation hinzu; ältere bleiben, bis die oben festgelegte Aufbewahrung sie entfernt.
admin-erasure-rotate = Schlüssel rotieren
admin-erasure-keys-none = Es wurde noch kein Schlüssel erstellt. Die erste Bereinigung erstellt einen.
admin-erasure-col-gen = Generation
admin-erasure-col-created = Erstellt
admin-erasure-col-state = Zustand
admin-erasure-key-current = Aktuell
admin-erasure-key-retired = Ausgemustert, aufbewahrt
admin-erasure-key-destroy = Ausgemustert, wird am { $when } gelöscht
admin-erasure-chain = Audit-Kette
admin-erasure-chain-note = Jeder Audit-Eintrag enthält den Hash des vorherigen. Eine Bereinigung schreibt Einträge neu, versiegelt die Kette erneut und zeichnet den alten und den neuen Kopf-Hash auf.
admin-erasure-seal = Neue Einträge versiegeln
admin-erasure-chain-ok = Die Kette ist verifiziert: { $rows } Einträge versiegelt, { $unsealed } warten auf die Versiegelung.
admin-erasure-chain-bad = Die Kette besteht die Verifizierung nicht; erster fehlerhafter Eintrag an Position { $seq }.
admin-erasure-col-time = Zeit
admin-erasure-col-range = Positionen
admin-erasure-backups = Backups
admin-erasure-backups-note = Ein Backup hält beim Erstellen den Tombstone-Wasserstand fest. Der aktuelle Wasserstand ist { $watermark }.
admin-erasure-col-label = Bezeichnung
admin-erasure-col-location = Speicherort
admin-erasure-col-watermark = Wasserstand
admin-erasure-col-marker = Gelöscht seit
admin-erasure-col-actions = Aktionen
admin-erasure-backups-none = Es sind keine Backups registriert.
admin-erasure-contains-erased = Enthält gelöschte Betroffene ({ $count })
admin-erasure-no-marker = Keine
admin-erasure-forget = Vergessen
admin-erasure-register = Backup registrieren
