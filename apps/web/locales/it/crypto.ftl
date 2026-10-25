# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Cifratura end-to-end
crypto-banner-e2ee-detail = Ogni destinatario ha una chiave — il corpo del messaggio viene cifrato su questo dispositivo.
crypto-banner-tls-label = Cifratura del trasporto (TLS)
crypto-banner-tls-detail = Non sono state trovate chiavi di cifratura dei destinatari; la consegna è protetta solo durante il transito.
crypto-banner-mixed-label = Protezione mista
crypto-banner-mixed-detail = Alcuni destinatari possono ricevere la cifratura end-to-end; gli altri solo TLS.
crypto-banner-checking-label = Verifica delle chiavi dei destinatari
crypto-banner-checking = · verifica…
crypto-banner-tls-only = Solo TLS per:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Sicurezza del messaggio
crypto-encrypt-label = Cifra (end-to-end)
crypto-encrypt-hint-default = Cifrare il corpo su questo dispositivo prima dell'invio.
crypto-encrypt-drafted = Bozza cifrata su questo dispositivo.
crypto-encrypt-no-key = Nessuna chiave di cifratura del destinatario disponibile.
crypto-sign-label = Firma (conferma l'identità del mittente)
crypto-protect-subject = Cifra anche l'oggetto
crypto-reason-add-recipient = Aggiungere un destinatario per verificare le chiavi di cifratura.
crypto-reason-tls = Nessuna chiave di cifratura del destinatario disponibile — invio tramite TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Avvisi di prevenzione della perdita di dati
crypto-dlp-block = Invio bloccato
crypto-dlp-require = Cifratura obbligatoria
crypto-dlp-warn = Attenzione
crypto-dlp-notify = L'amministratore verrà avvisato
crypto-dlp-matched = Corrispondenze: { $list }
