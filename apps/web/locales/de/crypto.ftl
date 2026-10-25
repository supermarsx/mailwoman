# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Ende-zu-Ende-verschlüsselt
crypto-banner-e2ee-detail = Jeder Empfänger hat einen Schlüssel — der Nachrichtentext wird auf diesem Gerät verschlüsselt.
crypto-banner-tls-label = Transportverschlüsselung (TLS)
crypto-banner-tls-detail = Es wurden keine Verschlüsselungsschlüssel der Empfänger gefunden; die Zustellung ist nur auf dem Transportweg geschützt.
crypto-banner-mixed-label = Gemischter Schutz
crypto-banner-mixed-detail = Einige Empfänger können Ende-zu-Ende-Verschlüsselung empfangen; andere erhalten nur TLS.
crypto-banner-checking-label = Schlüssel der Empfänger werden geprüft
crypto-banner-checking = · wird geprüft…
crypto-banner-tls-only = Nur TLS für:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Nachrichtensicherheit
crypto-encrypt-label = Verschlüsseln (Ende-zu-Ende)
crypto-encrypt-hint-default = Den Text vor dem Senden auf diesem Gerät verschlüsseln.
crypto-encrypt-drafted = Entwurf auf diesem Gerät verschlüsselt.
crypto-encrypt-no-key = Kein Verschlüsselungsschlüssel der Empfänger verfügbar.
crypto-sign-label = Signieren (bestätigt, dass sie von Ihnen stammt)
crypto-protect-subject = Auch die Betreffzeile verschlüsseln
crypto-reason-add-recipient = Fügen Sie einen Empfänger hinzu, um nach Verschlüsselungsschlüsseln zu suchen.
crypto-reason-tls = Kein Verschlüsselungsschlüssel der Empfänger verfügbar — Versand über TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Warnungen zur Verhinderung von Datenverlust
crypto-dlp-block = Senden blockiert
crypto-dlp-require = Verschlüsselung erforderlich
crypto-dlp-warn = Hinweis
crypto-dlp-notify = Der Administrator wird benachrichtigt
crypto-dlp-matched = Treffer: { $list }
