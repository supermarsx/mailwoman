# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: settings. Source: ../en/settings.ftl

settings-title = Instellingen
settings-appearance = Weergave
settings-close = Instellingen sluiten

settings-theme = Thema
settings-density = Dichtheid
settings-accent = Accentkleur
settings-font = Lettertype van de interface
settings-layout = Indeling

# Density options
settings-density-compact = Compact
settings-density-cozy = Comfortabel
settings-density-relaxed = Ruim

# Interface-font options
settings-font-default = Standaard
settings-font-system = Systeem
settings-font-serif = Serif
settings-font-mono = Mono

# Layout options
settings-layout-default = Standaard
settings-layout-ribbon = Lint

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Annuleren
settings-edit = Bewerken
settings-delete = Verwijderen
settings-save = Opslaan
settings-saved = Opgeslagen

# Two-factor authentication (S1)
settings-2fa-title = Tweestapsverificatie
settings-2fa-intro = Voeg een tweede factor toe, zodat bij het inloggen meer dan een wachtwoord nodig is.
settings-2fa-policy-required = Je organisatie vereist een tweede factor voor dit account.
settings-2fa-enabled = Ingeschakeld
settings-2fa-recovery-title = Herstelcodes
settings-2fa-recovery-once = Bewaar deze nu. Ze worden één keer getoond en kunnen niet opnieuw worden opgevraagd. Elke code werkt één keer.
settings-2fa-recovery-ack = Ik heb deze codes bewaard
settings-2fa-totp-title = Authenticator-app
settings-2fa-totp-enrol = Een authenticator-app instellen
settings-2fa-totp-scan = Voeg dit geheim toe aan je authenticator-app en voer daarna de code in die de app toont.
settings-2fa-totp-uri-link = Openen in een authenticator-app
settings-2fa-totp-code-label = Authenticatorcode
settings-2fa-totp-confirm = Bevestigen
settings-2fa-totp-disable = Authenticator verwijderen
settings-2fa-passkey-title = Passkeys
settings-2fa-passkey-remove = Verwijderen
settings-2fa-passkey-unsupported = Deze browser ondersteunt geen passkeys.
settings-2fa-passkey-label-placeholder = Naam van passkey (optioneel)
settings-2fa-passkey-add = Passkey toevoegen
settings-2fa-recovery-remaining = Nog { $count } { $count ->
        [one] herstelcode
       *[other] herstelcodes
    } over
settings-2fa-recovery-regenerate = Nieuwe herstelcodes aanmaken
settings-2fa-recovery-code-label = Herstelcode
settings-2fa-error-generic = Dat is niet gelukt. Probeer het opnieuw.
settings-2fa-error-code = Die code kon niet worden geverifieerd.
settings-2fa-error-passkey = De passkey kon niet worden geregistreerd.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Tweestapsverificatie
settings-2fa-challenge-intro = Bevestig je tweede factor om het inloggen te voltooien.
settings-2fa-challenge-failed = Verificatie mislukt. Probeer het opnieuw.
settings-2fa-challenge-use-passkey = Passkey gebruiken
settings-2fa-challenge-method = Kies een methode
settings-2fa-challenge-totp-tab = Authenticatorcode
settings-2fa-challenge-recovery-tab = Herstelcode
settings-2fa-challenge-verify = Verifiëren

# Active sessions (S11)
settings-sessions-title = Actieve sessies
settings-sessions-intro = Sessies die momenteel op dit account zijn ingelogd.
settings-sessions-last-seen = Laatst actief { $when }
settings-sessions-revoke = Uitloggen
settings-sessions-current = Deze sessie
settings-sessions-revoke-others = { $count } { $count ->
        [one] andere sessie
       *[other] andere sessies
    } uitloggen
settings-sessions-error = De sessie kon niet worden gewijzigd.

# Signatures (W12)
settings-sig-title = Handtekeningen
settings-sig-intro = Tekst die wordt toegevoegd aan berichten die je verstuurt. Eén handtekening kan de standaard zijn.
settings-sig-default = Standaard
settings-sig-new = Nieuwe handtekening
settings-sig-name-label = Naam
settings-sig-body-label = Handtekening
settings-sig-default-label = Als standaard gebruiken
settings-sig-error-name = Geef de handtekening een naam.
settings-sig-error-generic = De handtekening kon niet worden opgeslagen.

# Identities
settings-ident-title = Identiteiten
settings-ident-intro = De naam en het adres waarmee je verzendt. Elke identiteit kan een eigen handtekening gebruiken.
settings-ident-new = Nieuwe identiteit
settings-ident-name-label = Weergavenaam
settings-ident-email-label = E-mailadres
settings-ident-replyto-label = Antwoordadres (optioneel)
settings-ident-signature-label = Standaardhandtekening
settings-ident-signature-none = Geen
settings-ident-error-fields = Voer een naam en een geldig e-mailadres in.
settings-ident-error-generic = De identiteit kon niet worden opgeslagen.

# Notification rules + quiet hours (W15)
settings-notif-title = Meldingen
settings-notif-intro = Kies over welke nieuwe berichten je een melding krijgt, en wanneer het stil moet blijven.
settings-notif-enabled-label = Meldingen tonen voor nieuwe mail
settings-notif-quiet-title = Stille uren
settings-notif-quiet-enabled-label = Meldingen dempen tijdens een ingestelde periode
settings-notif-quiet-start = Van
settings-notif-quiet-end = Tot
settings-notif-rules-title = Regels
settings-notif-rule-match-placeholder = Afzender, map of onderwerp bevat…
settings-notif-rule-action-label = Actie
settings-notif-rule-notify = Melden
settings-notif-rule-mute = Dempen
settings-notif-rule-add = Regel toevoegen
settings-notif-error = De meldingsinstellingen konden niet worden opgeslagen.

# Saved searches → search folders (W13)
settings-search-title = Opgeslagen zoekopdrachten
settings-search-intro = Toon een opgeslagen zoekopdracht als map in je mappenlijst.
settings-search-empty = Je hebt nog geen opgeslagen zoekopdrachten.
settings-search-as-folder = { $name } als map tonen
settings-search-as-folder-label = Als map tonen
settings-search-error = De opgeslagen zoekopdracht kon niet worden gewijzigd.

# Keyboard shortcuts (W14)
settings-kbd-title = Sneltoetsen
settings-kbd-intro = Kies een set sneltoetsen. Wijzigingen gelden op dit apparaat.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Uit (geen sneltoetsen)
settings-kbd-action-compose = Opstellen
settings-kbd-action-archive = Archiveren
settings-kbd-action-reply = Beantwoorden
settings-kbd-action-next = Volgend bericht
settings-kbd-action-previous = Vorig bericht
settings-kbd-action-search = Zoeken

# Offline cache (W16)
settings-offline-title = Offline cache
settings-offline-intro = Hoeveel mail er op dit apparaat wordt bewaard voor offline gebruik, en hoe ruimte wordt vrijgemaakt.
settings-offline-budget-label = Cachelimiet (MB)
settings-offline-retention-label = Bewaren gedurende (dagen)
settings-offline-strategy-label = Wanneer de limiet is bereikt
settings-offline-lru = Minst recent gebruikte verwijderen
settings-offline-oldest = Oudste eerst verwijderen
settings-offline-manual = Alleen als ik de cache leegmaak
settings-offline-purge = Offline cache nu leegmaken

# Interface direction (W20)
settings-dir-title = Tekstrichting van de interface
settings-dir-intro = Volg de taal, of stel een richting vast. Van rechts naar links spiegelt de indeling.
settings-dir-auto = Automatisch
settings-dir-ltr = Van links naar rechts
settings-dir-rtl = Van rechts naar links
settings-dir-preview-title = Voorbeeld
settings-dir-preview-body = Deze tekst wordt gespiegeld als de richting van rechts naar links is.
