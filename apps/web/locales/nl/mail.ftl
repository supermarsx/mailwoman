# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Instellingen
mail-compose = Opstellen
mail-nav-mailboxes = Mappen
mail-nav-attachments = Bijlagen
mail-nav-outbox = Postvak UIT
mail-nav-sharing = Map delen
mail-nav-apps = Apps
mail-logout = Uitloggen
mail-offline = Offline
mail-module-loading = { $module } laden…

# -- Search ------------------------------------------------------------------
mail-search = Zoeken
mail-search-label = Mail zoeken
mail-search-placeholder = Mail zoeken — from:alice subject:invoice larger:1mb
mail-search-clear = Wissen

# -- Message list ------------------------------------------------------------
mail-list-label = Berichten
mail-loading = Berichten laden…
mail-empty = Geen berichten
mail-unknown-sender = (onbekende afzender)
mail-no-subject = (geen onderwerp)
mail-pinned = Vastgezet
mail-has-attachment = Heeft bijlage
mail-answered = Beantwoord
mail-forwarded = Doorgestuurd
mail-unread = Ongelezen
mail-flagged = Gemarkeerd
# Announced on a selected row; the row also shows a check mark.
mail-selected = Geselecteerd
# Announced row summary for screen readers (position within the whole list).
mail-row-position = Bericht { $pos } van { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } bericht
   *[other] { $count } berichten
}
mail-thread-expand = Gesprek uitvouwen, { $count ->
    [one] { $count } bericht
   *[other] { $count } berichten
}
mail-thread-collapse = Gesprek samenvouwen, { $count ->
    [one] { $count } bericht
   *[other] { $count } berichten
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Weergaveopties
mail-reading-pane = Leesvenster
mail-reading-pane-right = Rechts
mail-reading-pane-bottom = Onder
mail-reading-pane-off = Uit

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Geselecteerde berichten
mail-bulk-count = { $count } geselecteerd
mail-mark-read = Markeren als gelezen
mail-mark-unread = Markeren als ongelezen
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Markeren
mail-unflag-message = Markering verwijderen
mail-move-to = Verplaatsen naar map
mail-move-to-placeholder = Verplaatsen naar…
mail-select-all = Alles selecteren
mail-clear-selection = Selectie opheffen

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = Inbox met focus
mail-inbox-filter = Inboxfilter
mail-inbox-focused = Focus
mail-inbox-other = Overig
mail-inbox-turn-off = Uitzetten

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Geopende tabbladen
mail-subtab-pin = { $title } vastzetten
mail-subtab-unpin = { $title } losmaken
mail-subtab-tearoff = { $title } openen in een nieuw venster
mail-subtab-close = { $title } sluiten

# -- Per-row actions ---------------------------------------------------------
mail-pin = Vastzetten
mail-unpin = Losmaken
mail-snooze = Uitstellen
mail-snooze-menu = Uitstellen tot
mail-snooze-later = Later vandaag
mail-snooze-tomorrow = Morgen
mail-snooze-next-week = Volgende week
mail-unsnooze = Uitstel opheffen
mail-label = Label
mail-labels-menu = Labels
mail-flag = Markeren voor opvolging
mail-clear-flag = Opvolging wissen
mail-archive = Archiveren
mail-delete = Verwijderen
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = Meer acties

# -- Reader ------------------------------------------------------------------
mail-reader-label = Bericht
mail-reader-empty = Selecteer een bericht om te lezen
mail-back = Terug
mail-reader-from = Van: { $addr }
mail-reader-to = Aan: { $addr }
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Berichtacties
mail-reply = Beantwoorden
mail-reply-all = Allen beantwoorden
mail-forward = Doorsturen
mail-reply-failed = Het bericht kon niet worden voorbereid. Probeer het opnieuw.
mail-reply-unthreaded = De ID van het oorspronkelijke bericht kon niet worden gelezen. Dit antwoord wordt niet in hetzelfde gesprek geplaatst.
mail-spam = Spam
mail-export = Exporteren
mail-sweep-sender = Afzender opruimen
mail-message-body = Berichttekst
mail-no-content = Geen inhoud
mail-sanitizing = Opschonen…
mail-attachments = Bijlagen
mail-attachment-unnamed = (naamloos)
mail-attachment-open = Bijlage { $name }
mail-attachment-close = Bijlage sluiten
mail-attachment-loading = Bijlage laden…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Versleuteld bericht
mail-encrypted-note = 🔒 Dit bericht is end-to-end versleuteld. Ontgrendel het op dit apparaat om het te lezen.
mail-key-passphrase = Wachtwoordzin voor de sleutel
mail-decrypt = Ontsleutelen
mail-decrypting = Ontsleutelen…
mail-decrypt-no-key = Er is geen privésleutel beschikbaar om dit bericht te ontsleutelen.
mail-decrypt-failed = Ontsleutelen mislukt

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = Op { $date } schreef { $name }:
mail-quote-attribution-undated = { $name } schreef:
# The header block above a forwarded original.
mail-forward-heading = ---------- Doorgestuurd bericht ----------
mail-forward-from = Van: { $addr }
mail-forward-date = Datum: { $date }
mail-forward-subject = Onderwerp: { $subject }
mail-forward-to = Aan: { $addr }
mail-forward-cc = Cc: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Bericht opstellen
mail-compose-title = Nieuw bericht
mail-compose-title-reply = Beantwoorden
mail-compose-title-reply-all = Allen beantwoorden
mail-compose-title-forward = Doorsturen
mail-compose-close = Sluiten
mail-compose-from = Van
mail-compose-from-default = Standaard
mail-compose-to = Aan
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Contactsuggesties
mail-compose-cc = Cc
mail-compose-bcc = Bcc
mail-compose-show-cc-bcc = Cc of Bcc toevoegen
mail-compose-bcc-encrypted = Een versleuteld bericht vermeldt de sleutel van elke ontvanger, dus een Bcc-ontvanger zou zichtbaar zijn voor de anderen. Verwijder de Bcc-ontvangers of zet versleuteling uit.
mail-compose-plaintext-confirm-title = Ontsleutelde tekst zonder versleuteling verzenden?
mail-compose-plaintext-confirm-body = Dit bericht citeert tekst uit een versleuteld bericht. Het is niet ingesteld om te worden versleuteld, dus de geciteerde tekst zou onbeveiligd worden verzonden.
mail-compose-plaintext-confirm-send = Onversleuteld verzenden
mail-compose-plaintext-confirm-cancel = Teruggaan
mail-compose-subject = Onderwerp
mail-compose-body = Berichttekst
mail-compose-attachments = Bijlagen
mail-compose-remove-attachment = { $name } verwijderen
mail-compose-attach-nextcloud = Bijvoegen uit Nextcloud
mail-compose-close-nextcloud = Nextcloud sluiten
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Bestand bijvoegen
mail-compose-uploading = Uploaden…
mail-compose-upload-unavailable = Bestanden uploaden is op dit moment niet mogelijk.
mail-compose-upload-failed = Kan { $name } niet uploaden.
mail-compose-upload-too-large = { $name } is { $size } MB. De maximale uploadgrootte is { $max } MB.
mail-compose-send-later = Later verzenden
mail-compose-cancel = Annuleren
mail-compose-send = Verzenden
mail-compose-schedule = Inplannen
mail-compose-sending = Verzenden…
mail-compose-encrypted-subject = Versleuteld bericht
mail-compose-dlp-blocked = Verzenden is geblokkeerd door een regel voor preventie van gegevensverlies (zie de waarschuwing hierboven).
mail-compose-send-failed = Verzenden mislukt
mail-compose-send-later-past = Kies een verzendtijd in de toekomst, of maak het veld leeg om nu te verzenden.
mail-compose-options = Verzendopties
mail-compose-request-receipt = Leesbevestiging vragen
mail-compose-request-receipt-hint = Het mailprogramma van de ontvanger kan het hun vragen, er zonder te vragen een sturen of het verzoek negeren.
mail-compose-track-opens = Vastleggen wanneer dit bericht wordt geopend
mail-compose-track-opens-hint = Er wordt een afbeelding van 1×1 aan het bericht toegevoegd die van deze server laadt. Veel mailprogramma's blokkeren afbeeldingen of laden ze vooraf, dus een vastgelegde opening bewijst niet dat het bericht is gelezen, en het ontbreken ervan bewijst niet dat het niet is gelezen. De ontvanger krijgt dit niet te horen.
mail-compose-image-type = { $name } kan niet in de tekst worden geplaatst. Gebruik een PNG-, JPEG-, GIF- of WebP-afbeelding.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = Je bent niet ingelogd.
mail-send-no-draft-folder = Dit account heeft geen map om het concept in op te slaan.
mail-send-no-recipient = Voeg minstens één ontvanger toe.
mail-send-bad-recipient = { $count ->
    [one] Dit is geen e-mailadres: { $addresses }
   *[other] Dit zijn geen e-mailadressen: { $addresses }
}
mail-send-later-past = De verzendtijd ligt niet in de toekomst. Er is niets verzonden.
mail-send-tracking-failed = Openingen kunnen nu niet worden vastgelegd, dus er is niets verzonden. Schakel "Vastleggen wanneer dit bericht wordt geopend" uit om zonder te verzenden.
mail-send-draft-refused = De server heeft dit bericht geweigerd: { $reason }. Er is niets verzonden.
mail-send-submission-refused = De server heeft geweigerd dit bericht te verzenden: { $reason }. Er is niets verzonden.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Ondertekeningssleutel ontgrendelen
mail-compose-sign-unlock-note = Voer de wachtwoordzin van je sleutel in om dit bericht te ondertekenen. De sleutel blijft alleen voor dit opstelvenster ontgrendeld.
mail-compose-sign-unlock = Ontgrendelen
mail-compose-sign-unlocking = Ontgrendelen…
mail-compose-sign-no-key = Er is geen ondertekeningssleutel beschikbaar op dit apparaat.
mail-compose-sign-unlock-failed = Kan de ondertekeningssleutel niet ontgrendelen.
mail-compose-sign-unlock-required = Ontgrendel je ondertekeningssleutel om dit ondertekende bericht te verzenden.

# Rich-text editor (W1)
mail-compose-format-plain = Platte tekst
mail-compose-format-rich = Opgemaakte tekst
mail-compose-rt-toolbar = Opmaak
mail-compose-rt-bold = Vet
mail-compose-rt-italic = Cursief
mail-compose-rt-underline = Onderstrepen
mail-compose-rt-strike = Doorhalen
mail-compose-rt-bullet = Lijst met opsommingstekens
mail-compose-rt-ordered = Genummerde lijst
mail-compose-rt-quote = Citaat
mail-compose-rt-link = Link
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Toepassen
mail-compose-rt-image = Afbeelding invoegen

# Signature picker (W12)
mail-compose-signature = Handtekening
mail-compose-signature-none = Handtekening toevoegen…

# Recall (W10)
mail-compose-recall = Terughalen
mail-compose-recall-action = Terughalen
mail-compose-recall-holding = Vastgehouden, wacht op verzending
mail-compose-recall-scheduled = Ingepland voor { $when }

# Drafts drawer (W9)
mail-compose-drafts = Concepten
mail-compose-drafts-empty = Geen opgeslagen concepten
mail-compose-drafts-delete = Concept verwijderen
mail-compose-drafts-no-recipient = (geen ontvanger)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Postvak UIT
mail-outbox-refresh = Vernieuwen
mail-outbox-empty = Niets dat wacht op verzending
mail-outbox-send-now = Nu verzenden
mail-outbox-cancel = Annuleren
mail-outbox-scheduled = Ingepland
mail-outbox-holding = Wordt binnenkort verzonden
mail-outbox-sent = Verzonden
mail-outbox-canceled = Geannuleerd
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = Vastgehouden
mail-outbox-failed = Niet verzonden
mail-outbox-release = Vrijgeven
mail-outbox-discard = Weggooien
mail-outbox-held-note = Vastgehouden. Wordt pas verzonden na vrijgave.
mail-outbox-held-note-api-key = Vastgehouden — aangemaakt met API-sleutel { $name }. Wordt pas verzonden na vrijgave.
mail-outbox-held-note-oauth = Vastgehouden — aangemaakt door de gekoppelde app { $name }. Wordt pas verzonden na vrijgave.
mail-outbox-origin-api-key = Aangemaakt met API-sleutel { $name }
mail-outbox-origin-oauth = Aangemaakt door de gekoppelde app { $name }
mail-outbox-to = Aan { $recipients }
mail-outbox-no-subject = (geen onderwerp)
mail-outbox-attempts = { $count } keer geprobeerd. Laatste fout: { $error }
mail-outbox-error = Fout: { $error }
mail-outbox-toast-canceled = Verzenden geannuleerd
mail-outbox-toast-discarded = Weggegooid. Het bericht is niet verzonden.
mail-outbox-toast-sent = Verzonden
mail-outbox-toast-retrying = Nog niet verzonden: { $error }. Er wordt opnieuw geprobeerd.
mail-outbox-toast-failed = Niet verzonden: { $error }
mail-outbox-toast-not-released = Kan niet verzenden: { $error }
mail-outbox-toast-not-canceled = Kan niet annuleren: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = Verbinding verbroken — opnieuw proberen…
mail-net-back = Weer online
mail-queue-sent = { $count ->
    [one] { $count } wijziging uit de wachtrij verzonden
   *[other] { $count } wijzigingen uit de wachtrij verzonden
}
mail-queue-failed = { $count ->
    [one] { $count } wijziging in de wachtrij mislukt
   *[other] { $count } wijzigingen in de wachtrij mislukt
}
mail-queue-notice-label = Wijzigingen die niet konden worden toegepast
mail-queue-notice-title = { $count ->
    [one] { $count } offline gemaakte wijziging kon niet worden toegepast
   *[other] { $count } offline gemaakte wijzigingen konden niet worden toegepast
}
mail-queue-kind-send = “{ $subject }” verzenden
mail-queue-kind-send-untitled = Een bericht zonder onderwerp verzenden
mail-queue-kind-move = Een bericht verplaatsen
mail-queue-kind-flag = Een label of markering wijzigen
mail-queue-kind-draft = Een concept opslaan
mail-queue-kind-pim = Wijziging in agenda, taak, notitie of contact
mail-queue-reason = Reden: { $reason }
mail-queue-reason-unknown = De server heeft geen reden opgegeven.
mail-queue-retry = Opnieuw proberen
mail-queue-discard = Weggooien
mail-queue-retry-item = Opnieuw proberen: { $what }
mail-queue-discard-item = Weggooien: { $what }
mail-queue-discarded = Weggegooid. De wijziging is niet toegepast.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Berichten opruimen
mail-sweep-title = { $sender } opruimen
mail-sweep-all = Alles van deze afzender verwijderen
mail-sweep-keep-latest = Nieuwste behouden, de rest verwijderen
mail-sweep-older-than = Ouder dan N dagen verwijderen
mail-sweep-block = Alles verwijderen en deze afzender blokkeren
mail-sweep-days = Dagen
mail-sweep-count = { $count ->
    [one] { $count } bericht wordt naar de prullenbak verplaatst
   *[other] { $count } berichten worden naar de prullenbak verplaatst
}
mail-sweep-cancel = Annuleren
mail-sweep-run = { $count } opruimen
mail-sweeping = Opruimen…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Sluiten
mail-undo-action = Ongedaan maken
mail-undo-send-cancel = Annuleren
mail-undo-failed = Kan niet ongedaan maken

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Label toegevoegd
mail-toast-label-removed = Label verwijderd
mail-toast-pinned = Vastgezet
mail-toast-unpinned = Losgemaakt
mail-toast-snoozed = Uitgesteld
mail-toast-follow-up-set = Opvolging ingesteld
mail-toast-follow-up-cleared = Opvolging gewist
mail-toast-archived = Gearchiveerd
mail-toast-trashed = Naar prullenbak verplaatst
mail-toast-marked-spam = Gemarkeerd als spam
mail-toast-moved-to = Verplaatst naar { $folder }
mail-toast-moved = Verplaatst naar map
mail-toast-archived-count = { $count } berichten gearchiveerd
mail-toast-trashed-count = { $count } berichten naar prullenbak verplaatst
mail-toast-marked-spam-count = { $count } berichten gemarkeerd als spam
mail-toast-moved-count = { $count } berichten verplaatst
mail-toast-moved-to-count = { $count } berichten verplaatst naar { $folder }
mail-toast-marked-read = { $count ->
    [one] Gemarkeerd als gelezen
   *[other] { $count } berichten gemarkeerd als gelezen
}
mail-toast-marked-unread = { $count ->
    [one] Gemarkeerd als ongelezen
   *[other] { $count } berichten gemarkeerd als ongelezen
}
mail-toast-flagged = { $count ->
    [one] Gemarkeerd
   *[other] { $count } berichten gemarkeerd
}
mail-toast-unflagged = { $count ->
    [one] Markering verwijderd
   *[other] Markering verwijderd van { $count } berichten
}
mail-toast-move-failed = Kan de berichten niet verplaatsen. Er is niets verplaatst.
mail-toast-update-failed = Kan de berichten niet wijzigen. Er is niets gewijzigd.
mail-toast-update-refused = { $count ->
    [one] De server heeft de wijziging voor één bericht geweigerd
   *[other] De server heeft de wijziging voor { $count } berichten geweigerd
}
mail-toast-no-archive-folder = Geen archiefmap
mail-toast-no-trash-folder = Geen prullenbakmap
mail-toast-no-spam-folder = Geen spammap
mail-toast-sweep-nothing = Niets om op te ruimen
mail-toast-swept = { $count ->
    [one] { $count } bericht opgeruimd
   *[other] { $count } berichten opgeruimd
}
mail-toast-load-more-failed = Kan niet meer berichten laden
mail-toast-export-nothing = Niets om te exporteren
mail-toast-exported = .eml geëxporteerd
mail-toast-export-failed = Exporteren mislukt
mail-toast-send-queued = In wachtrij — wordt verzonden zodra je weer online bent
mail-toast-tracking-not-bound = Het bericht is verzonden, maar het openen ervan wordt niet vastgelegd.
mail-toast-send-scheduled = Ingepland voor verzending
mail-toast-sent = Bericht verzonden
mail-toast-send-canceled = Verzenden geannuleerd

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Lint
mail-ribbon-tab-home = Start
mail-ribbon-tab-view = Beeld
mail-ribbon-tab-folder = Map
mail-ribbon-expand = Uitvouwen
mail-ribbon-collapse = Samenvouwen
mail-ribbon-group-new = Nieuw
mail-ribbon-compose = Opstellen
mail-ribbon-group-session = Sessie
mail-ribbon-logout = Uitloggen
mail-ribbon-group-theme = Thema
mail-ribbon-group-density = Dichtheid
mail-ribbon-group-settings = Instellingen
mail-ribbon-more = Meer…
mail-ribbon-group-folders = Mappen
mail-density-compact = compact
mail-density-cozy = comfortabel
mail-density-relaxed = ruim

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Beveiligingsniveau van bericht
mail-maxsec-full = Volledig
mail-maxsec-full-hint = Opgeschoonde HTML met afbeeldingen en opmaak.
mail-maxsec-no-media = Geen media
mail-maxsec-no-media-hint = Opgeschoonde HTML waaruit alle afbeeldingen en media zijn verwijderd.
mail-maxsec-plain-text = Platte tekst
mail-maxsec-plain-text-hint = Geen HTML weergegeven — bericht alleen als platte tekst getoond.
