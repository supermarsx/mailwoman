# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = End-to-end versleuteld
crypto-banner-e2ee-detail = Elke ontvanger heeft een sleutel — de berichttekst wordt op dit apparaat versleuteld.
crypto-banner-tls-label = Transportversleuteling (TLS)
crypto-banner-tls-detail = Er zijn geen versleutelingssleutels van ontvangers gevonden; de bezorging is alleen onderweg beveiligd.
crypto-banner-mixed-label = Gemengde beveiliging
crypto-banner-mixed-detail = Sommige ontvangers kunnen end-to-end versleutelde berichten ontvangen; andere krijgen alleen TLS.
crypto-banner-checking-label = Sleutels van ontvangers controleren
crypto-banner-checking = · controleren…
crypto-banner-tls-only = Alleen TLS voor:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Berichtbeveiliging
crypto-encrypt-label = Versleutelen (end-to-end)
crypto-encrypt-hint-default = Versleutel de berichttekst op dit apparaat voor het verzenden.
crypto-encrypt-drafted = Concept versleuteld op dit apparaat.
crypto-encrypt-no-key = Geen versleutelingssleutel van de ontvanger beschikbaar.
crypto-sign-label = Ondertekenen (bevestigt dat het van jou komt)
crypto-protect-subject = Ook de onderwerpregel versleutelen
crypto-reason-add-recipient = Voeg een ontvanger toe om op versleutelingssleutels te controleren.
crypto-reason-tls = Geen versleutelingssleutel van de ontvanger beschikbaar — verzenden via TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Waarschuwingen voor preventie van gegevensverlies
crypto-dlp-block = Verzenden geblokkeerd
crypto-dlp-require = Versleuteling vereist
crypto-dlp-warn = Let op
crypto-dlp-notify = De beheerder wordt op de hoogte gebracht
crypto-dlp-matched = Gevonden: { $list }
