# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Sicherheitsdetails der Nachricht
security-section-auth = Authentifizierung
security-section-received = Zustellweg
security-section-signature = Signatur
security-section-attachments = Anhänge
security-section-warnings = Warnungen
security-section-controls = Absender-Steuerung
security-received-empty = Keine Received-Kette verfügbar.
security-attachments-empty = Keine Anhänge.
security-hop-unknown = unbekannt
security-unknown = unbekannt

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = bestanden
security-auth-fail = fehlgeschlagen
security-auth-none = nicht vorhanden
security-auth-neutral = neutral
security-auth-temperror = vorübergehender Fehler
security-auth-permerror = dauerhafter Fehler

security-detail-domain = Domain
security-detail-selector = Selektor
security-detail-policy = Richtlinie
security-detail-alignment = Ausrichtung
security-detail-chain-length = Kettenlänge
security-aligned = ausgerichtet
security-not-aligned = nicht ausgerichtet

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Signatur verifiziert
security-sig-unverified-key = Signiert — Schlüssel des Unterzeichners nicht verifiziert
security-sig-invalid = Signatur ist ungültig
security-sig-none = Nicht signiert
security-chain-trusted = Vertrauenswürdige Kette
security-chain-untrusted = Nicht vertrauenswürdige Kette
security-chain-expired = Kette abgelaufen
security-chain-unknown = Kette unbekannt
security-revocation-good = Nicht widerrufen
security-revocation-revoked = Schlüssel widerrufen
security-revocation-unknown = Widerrufsstatus unbekannt
security-key-changed = Schlüssel des Unterzeichners hat sich seit dem letzten Mal geändert
security-fact-signer-key = Schlüssel des Unterzeichners
security-fact-algorithm = Algorithmus
security-fact-key-created = Schlüssel erstellt
security-fact-key-expires = Schlüssel läuft ab
security-fact-chain = Kette
security-fact-revocation = Widerruf
security-fact-key-change = Schlüsselwechsel

# -- Attachment risk --------------------------------------------------------
security-attach-none = Kein bekanntes Risiko
security-attach-macro = Enthält Makros
security-attach-executable = Ausführbare Datei
security-attach-encrypted-archive = Verschlüsseltes Archiv
security-attach-double-extension = Doppelte Dateiendung
security-attach-mismatch = Typ stimmt nicht überein (angegeben { $declared }, erkannt { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Reply-To-Adresse weicht vom Absender ab
security-anomaly-envelopeFromDivergence = Envelope-Absender weicht von der From-Adresse ab
security-anomaly-messageIdDomainAnomaly = Domain der Message-ID passt nicht zum Absender
security-anomaly-dateSkew = Sendedatum scheint verschoben
security-anomaly-punycodeSender = Absender verwendet Punycode (mögliche Doppelgänger-Domain)

# -- Sender controls --------------------------------------------------------
security-control-block = Absender blockieren
security-control-silence = Absender stummschalten
security-control-ignore-conversation = Unterhaltung ignorieren
security-control-report-phishing = Phishing melden
security-control-report-junk = Junk melden
security-control-done-block = Absender blockiert
security-control-done-silence = Absender stummgeschaltet
security-control-done-ignore-conversation = Unterhaltung ignoriert
security-control-done-report-phishing = Als Phishing gemeldet
security-control-done-report-junk = Als Junk gemeldet

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Zero-Access-Speicherung
security-za-on = An
security-za-off = Aus
security-za-protects = Zero-Access verschlüsselt Ihre gespeicherten E-Mails, Notizen und PIM-Daten, sodass der Hosting-Server nur Chiffretext aufbewahrt, den er nicht lesen kann. Es schützt den Inhalt Ihrer gespeicherten Daten vor einem neugierigen Betreiber oder einem Einbruch beim Speicher-Host: Wer die Datenbank, die Dateien auf der Festplatte oder eine gestohlene Sicherung liest, sieht XChaCha20-Poly1305-Chiffretext und kann weder Nachrichtentexte, Betreffzeilen, Anhänge, Notiztexte noch den Suchindex wiederherstellen.

security-za-server-sees-title = Was der Server weiterhin sieht
security-za-sees-1 = Chiffretext-Blobs — die verschlüsselten Zeilen selbst.
security-za-sees-2 = Undurchsichtige Zeilen-IDs — die Kennungen, mit denen Zeilen gespeichert und abgerufen werden.
security-za-sees-3 = Größen — die Länge jedes Chiffretexts (ungefähre Größe von Nachricht/Anhang).
security-za-sees-4 = Zeitstempel — wann Zeilen geschrieben und aktualisiert werden.
security-za-sees-5 = Envelope-Routing-Metadaten, die zum Weiterleiten von IMAP/SMTP nötig sind — der Server verbindet sich weiterhin in Ihrem Namen mit Ihrem vorgelagerten E-Mail-Anbieter, daher laufen Verbindungsmetadaten und der zum Senden und Empfangen nötige Routing-Envelope über ihn.

security-za-active-server-caveat = Zero-Access schützt gespeicherte Daten (AT REST) vor einem neugierigen oder kompromittierten Host. Ein vollständig bösartiger aktiver Server, der Ihren laufenden IMAP/SMTP-Verkehr weiterleitet, ist ein stärkerer Gegner, und Zero-Access schützt NICHT vor ihm: Ein solcher Server sitzt in der laufenden Verbindung zu Ihrem E-Mail-Anbieter und könnte E-Mails beim Durchlaufen beobachten oder manipulieren, unabhängig davon, wie die gespeicherte Kopie verschlüsselt ist. Wählen Sie Zero-Access, wenn Ihr Anliegen lautet: „Ich möchte nicht, dass derjenige, der den Speicher-Host betreibt (oder in ihn einbricht), meine gespeicherten E-Mails liest.“ Es ist kein Schutz vor einem Betreiber, der den laufenden E-Mail-Weg unterwandert.

security-za-no-search-claim = Es gibt hier keine serverseitige durchsuchbare Verschlüsselung, und es wird auch keine solche behauptet. Die Suche läuft vollständig auf dem Client über Inhalte, die er lokal entschlüsselt hat; der Server hält nie eine durchsuchbare Form Ihres Klartexts.

security-za-recovery-tradeoff = Ihre Schlüssel werden nur auf Ihren Geräten abgeleitet und aufbewahrt — der Server erhält nie einen Klartext-Schlüssel. Wenn Sie Ihre Passphrase (oder Ihren Passkey) UND jedes gekoppelte Gerät UND Ihre Wiederherstellungsphrase verlieren, können die verschlüsselten Daten nicht wiederhergestellt werden. Bewahren Sie die Wiederherstellungsphrase offline auf und behandeln Sie sie wie das Hauptgeheimnis Ihres Kontos.

security-za-setup-key = Schlüssel einrichten
security-za-key-source = Schlüsselquelle
security-za-passphrase = Passphrase
security-za-passkey = Passkey (ohne Passwort)
security-za-passkey-unavailable = Passkeys sind in diesem Browser nicht verfügbar
security-za-passphrase-label = Zero-Access-Passphrase
security-za-enable = Zero-Access aktivieren
security-za-recovery-title = Wiederherstellungsphrase
security-za-recovery-heading = Wiederherstellungsphrase — jetzt offline sichern
security-za-recovery-note = Dies ist die einzige Kopie. Wer sie besitzt, kann Ihre Daten lesen; ohne sie (und ohne gekoppeltes Gerät) können Ihre Daten nicht wiederhergestellt werden.
security-za-disable-section = Zero-Access deaktivieren
security-za-disable-heading = Zero-Access ausschalten
security-za-disable-note = Neue Daten werden wieder unverschlüsselt gespeichert. Vorhandene verschlüsselte Daten bleiben nur lesbar, solange Sie Ihren Schlüssel noch ableiten können.
security-za-disable-btn = Zero-Access deaktivieren
security-za-err-passphrase-len = Verwenden Sie eine Passphrase mit mindestens 8 Zeichen
security-za-err-enable = Zero-Access konnte nicht aktiviert werden
security-za-err-disable = Zero-Access konnte nicht deaktiviert werden

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Gerätekopplung
security-pair-heading = Gerät koppeln
security-pair-intro = Übertragen Sie Ihre Schlüssel auf ein anderes Gerät, ohne sie über den Server zu senden — er leitet nur einen undurchsichtigen versiegelten Umschlag weiter. Vergleichen Sie die sechs Wörter auf beiden Bildschirmen, bevor Sie der Kopplung vertrauen.
security-pair-role = Rolle bei der Kopplung
security-pair-new-role = Dies ist das neue Gerät
security-pair-existing-role = Von hier aus ein anderes Gerät koppeln
security-pair-show-qr = Kopplungs-QR-Code anzeigen
security-pair-scan = Scannen Sie dies auf Ihrem vorhandenen Gerät
security-pair-qr-label = QR-Code zur Gerätekopplung
security-pair-qr-default = Kopplungs-QR-Code
security-pair-copy-code = Oder kopieren Sie diesen Kopplungscode
security-pair-paste-envelope = Fügen Sie den versiegelten Umschlag vom anderen Gerät ein
security-pair-envelope-placeholder = Umschlag…
security-pair-envelope-label = Versiegelter Umschlag
security-pair-complete = Kopplung abschließen
security-pair-code-from-new = Kopplungscode vom neuen Gerät
security-pair-code-placeholder = Kopplungscode…
security-pair-code-label = Kopplungscode
security-pair-seal = Meine Schlüssel für dieses Gerät versiegeln
security-pair-envelope-out = Versiegelter Umschlag — fügen Sie ihn auf dem neuen Gerät ein
security-pair-compare = Vergleichen Sie diese Wörter auf beiden Geräten
security-pair-sas-label = Kurze Authentifizierungszeichenfolge
security-pair-match = Die Wörter stimmen überein
security-pair-abort = Sie unterscheiden sich — abbrechen
security-pair-confirmed = Kopplung bestätigt. Dieser Kanal ist authentifiziert.
security-pair-err-start = Kopplung konnte nicht gestartet werden
security-pair-err-complete = Kopplung konnte nicht abgeschlossen werden
security-pair-err-unlock = Entsperren Sie zuerst Zero-Access, um ein anderes Gerät zu koppeln
security-pair-err-seal = Der Root-Schlüssel konnte nicht versiegelt werden
