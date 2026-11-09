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
mail-inbox-filter = Inboxfilter
mail-inbox-focused = Focus
mail-inbox-other = Overig

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
mail-encrypted-note = Dit bericht is end-to-end versleuteld. Ontgrendel het op dit apparaat om het te lezen.
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
mail-send-not-signed-in = U bent niet ingelogd.
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
mail-compose-sign-unlock-note = Voer de wachtwoordzin van uw sleutel in om dit bericht te ondertekenen. De sleutel blijft alleen voor dit opstelvenster ontgrendeld.
mail-compose-sign-unlock = Ontgrendelen
mail-compose-sign-unlocking = Ontgrendelen…
mail-compose-sign-no-key = Er is geen ondertekeningssleutel beschikbaar op dit apparaat.
mail-compose-sign-unlock-failed = Kan de ondertekeningssleutel niet ontgrendelen.
mail-compose-sign-unlock-required = Ontgrendel uw ondertekeningssleutel om dit ondertekende bericht te verzenden.

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
mail-toast-pinned = Bericht “{ $target }” vastgezet
mail-toast-unpinned = Bericht “{ $target }” losgemaakt
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
mail-toast-send-queued = In wachtrij — wordt verzonden zodra u weer online bent
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
mail-feature-off = Uitgeschakeld door uw instellingen of door uw beheerder
mail-send-disabled = Verzenden is uitgeschakeld door uw instellingen of door uw beheerder. U kunt nog wel een concept opslaan.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Gebeurtenissen
mail-events-upcoming = Alleen aankomende
mail-events-filter = Gebeurtenisfilter
mail-loading-more = Oudere berichten laden…
mail-event-state-invitation = Uitnodiging
mail-event-state-updated = Bijgewerkt
mail-event-state-cancelled = Geannuleerd
mail-event-state-reply = Antwoord
mail-event-state-counter = Nieuw tijdstip voorgesteld
mail-event-state-event = Gebeurtenis
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Meer acties tonen
mail-remote-moved = Dit bericht is vanaf een ander apparaat of een andere app naar { $folder } verplaatst. U leest nog steeds de kopie die u hebt geopend.
mail-remote-moved-unknown = Dit bericht is vanaf een ander apparaat of een andere app verplaatst. U leest nog steeds de kopie die u hebt geopend.
mail-remote-deleted = Dit bericht is vanaf een ander apparaat of een andere app verwijderd. U kunt deze kopie blijven lezen totdat u haar sluit.
mail-remote-dismiss = Sluiten
mail-reply-draft = Antwoord in uitvoering
mail-reply-draft-open = Concept openen
mail-draft-load-failed = Het concept kon niet worden geopend. Probeer het opnieuw.
mail-reply-images-omitted =
    { $count ->
        [one] Eén afbeelding kon niet in het antwoord worden opgenomen.
       *[other] { $count } afbeeldingen konden niet in het antwoord worden opgenomen.
}
mail-export-forbidden = Uw beheerder heeft het exporteren van berichten voor dit account uitgeschakeld.
mail-export-open-failed = De exportopties konden niet worden geopend. Sluit een werkruimtetabblad en probeer het opnieuw.
mail-headers-title = Ruwe berichtkoppen
mail-headers-action-help = Bekijk de oorspronkelijke berichtkoppen, inclusief aflever- en authenticatiegegevens.
mail-headers-help = Oorspronkelijke koppen van het bericht. Gevouwen regels blijven behouden; hier worden alleen koppen getoond.
mail-headers-loading = Berichtkoppen laden…
mail-headers-failed = De berichtkoppen konden niet worden geladen. Controleer uw verbinding en probeer het opnieuw.
mail-headers-copy = Koppen kopiëren
mail-headers-copied = Koppen gekopieerd.
mail-headers-copy-failed = De koppen konden niet worden gekopieerd. Selecteer de tekst om hem handmatig te kopiëren.
mail-attachment-failed = Deze bijlage kon niet worden gedownload. Probeer het opnieuw zodra er verbinding is.
mail-compose-rt-rule = Horizontale lijn
mail-compose-signature-new = Nieuwe berichten
mail-compose-signature-replies = Antwoorden
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = Grootte van de berichttekst
mail-editor-body-size-help = Grootte van de huidige berichttekst als UTF-8, inclusief HTML-opmaak en ingesloten afbeeldingen. Berichtkoppen en afzonderlijke bijlagen zijn niet inbegrepen.
mail-outbox-load-failed = Het Postvak UIT kon niet worden gecontroleerd. Probeer Vernieuwen nadat u uw verbinding hebt gecontroleerd.
mail-outbox-unavailable = Postvak UIT niet beschikbaar. Gebruik Vernieuwen om het opnieuw te proberen.
mail-maxsec-view = Weergave: { $mode }
mail-maxsec-menu = Weergavemodus
mail-toast-tab-pinned = Tabblad “{ $target }” vastgezet
mail-toast-tab-unpinned = Tabblad “{ $target }” losgemaakt
mail-workspace-message = Bericht
mail-opened-announcement = Geopend: { $subject }
mail-workspace-layout = Indeling van de werkruimte
mail-workspace-tabs = Tabbladen
mail-workspace-side-by-side = Naast elkaar
mail-workspace-stack = Gestapeld
mail-workspace-refresh = Map vernieuwen
mail-workspace-load-error = Dit bericht kon niet worden geladen. Sluit het tabblad en probeer het opnieuw.
mail-resize-folders = Grootte van de mappenzijbalk wijzigen
mail-resize-list = Grootte van de berichtenlijst wijzigen
mail-resize-tiles = Grootte van de werkruimtedeelvensters wijzigen
mail-show-folders = Mappenzijbalk tonen
mail-hide-folders = Mappenzijbalk verbergen
mail-compact-folders = Compacte mappictogrammen
mail-toggle-list = Berichtenlijst tonen of verbergen
mail-toggle-reader = Leesvenster tonen of verbergen
mail-folder-counts = { $unread } ongelezen, { $total } berichten
mail-print = Afdrukken / Opslaan als PDF
mail-go-to-top = Naar boven
mail-print-help = Druk dit bericht af of kies Opslaan als PDF in het afdrukvenster. Externe afbeeldingen blijven geblokkeerd.
mail-file-details = Bestandsgegevens en vingerafdruk
mail-file-type = Bestandstype
mail-file-extension = Extensie
mail-file-detected = Gedetecteerde signatuur
mail-file-unknown = Geen herkende signatuur
mail-file-size = Grootte
mail-file-size-bytes = { $count } bytes
mail-file-hash-unavailable = Vingerafdruk niet beschikbaar (vereist een beveiligde browser en een bestand tot 64 MiB).
mail-file-fingerprint-help = Een vingerafdruk identificeert precies deze bytes. Hij bewijst niet dat het bestand veilig is of wie het heeft gemaakt.
mail-file-warning-executable = Dit bestand kan programma's uitvoeren of systeeminstellingen wijzigen. Download het alleen als u de bron vertrouwt.
mail-file-warning-macro = Dit document kan macro's bevatten. Schakel macro's alleen in als u de afzender en het doel hebt gecontroleerd.
mail-file-warning-disguisedName = De bestandsnaam kan de werkelijke extensie verhullen. Controleer de bestandsgegevens voordat u downloadt.
mail-file-warning-typeMismatch = De gedetecteerde bestandssignatuur komt niet overeen met het opgegeven type. De automatische voorvertoning is geblokkeerd.
mail-file-warning-activeDocument = Dit formaat kan actieve inhoud bevatten. De voorvertoning is geïsoleerd; het openen van een gedownloade kopie kan zich anders gedragen.
mail-file-preview-blocked = Automatische voorvertoning is uitgeschakeld voor dit bestandstype of bij een afwijkend type.
mail-quote-header-from = Van:
mail-quote-header-sent = Verzonden:
mail-quote-header-to = Aan:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Onderwerp:
mail-spam-score = Spamanalyse
mail-spam-score-loading = Spamkoppen controleren…
mail-spam-score-unavailable = Spamscore niet beschikbaar
mail-spam-score-unverified = Gemeld door de berichtkoppen, niet onafhankelijk geverifieerd. Door de afzender aangeleverde koppen kunnen vervalst zijn.
mail-spam-score-positive = De kop meldt spam.
mail-spam-score-negative = De kop meldt een resultaat dat geen spam is.
mail-spam-score-unknown = Bij deze score is geen oordeel meegeleverd.
mail-external-recipients = Externe ontvangers
mail-external-count = { $count } ontvanger(s) buiten vertrouwde domeinen
mail-external-help = Controleer de ontvangers in Aan, Cc en Bcc voordat u vertrouwelijke informatie verzendt. Vertrouwen is gebaseerd op de geselecteerde verzendidentiteit en uw effectieve domeinbeleid.
mail-external-reviewed = Ik heb deze externe ontvangers gecontroleerd.
mail-external-confirm-required = Controleer en bevestig de externe ontvangers voordat u verzendt.
mail-quick-reply = Snel antwoord
mail-quick-reply-help = Biedt een compact antwoordtabblad aan waarin de ontvangers al zijn ingevuld. Standaard uit. De normale veiligheidscontroles en verzendopties blijven van kracht.
mail-quick-reply-expand = Ontvangers en onderwerp bewerken
mail-attachment-policy-loading = Bijlagenbeleid controleren…
mail-attachment-policy-unavailable = Het bijlagenbeleid kon niet worden gecontroleerd. Probeer het opnieuw voordat u dit bestand opent.
mail-attachment-policy-blocked = Uw beheerder heeft deze bijlage of de download van het origineel geblokkeerd.
mail-attachment-policy-warning = Uw beheerder adviseert voorzichtigheid met deze bijlage. Ga alleen verder als u de bron vertrouwt.
mail-attachment-policy-acknowledge = Ik begrijp het — doorgaan
mail-attachment-policy-no-preview = Voorvertoningen van bijlagen zijn door uw beheerder uitgeschakeld. Downloads van originelen zijn mogelijk nog beschikbaar.
mail-time-sent = Verzonden
mail-time-received = Ontvangen
mail-time-unknown = Niet opgegeven
mail-time-comparison = Tijd in { $zone }
mail-time-privacy = De vergelijkingstijdzone is uw keuze. Tijdstempels van berichten geven niemands locatie aan.
mail-time-event-comparison = Vergelijking van het tijdstip van de gebeurtenis
mail-time-ambiguous = Deze lokale tijd komt twee keer voor wanneer de klok wordt verzet. Beide mogelijke tijdstippen worden getoond; controleer de bedoelde offset van de gebeurtenis voordat u opslaat.
mail-bulk-add-label = Label toevoegen
mail-select-messages = Berichten selecteren
mail-bulk-remove-label = Label verwijderen
mail-mark-spam = Als spam markeren
mail-bulk-forward = Geselecteerde berichten doorsturen als bijlagen
mail-bulk-download = Geselecteerde berichten downloaden als mbox
mail-bulk-forward-subject = Doorgestuurde berichten ({ $count })
mail-bulk-transfer-failed = De geselecteerde berichten konden niet worden voorbereid. Selecteer maximaal 100 berichten met samen hoogstens 32 MiB en controleer de exportmachtigingen van uw account.
mail-toast-labels-added = Label { $label } toegevoegd aan { $count } berichten
mail-toast-labels-removed = Label { $label } verwijderd van { $count } berichten
mail-action-help-reply = Antwoord de afzender in een nieuw opstel-tabblad.
mail-action-help-reply-all = Antwoord de afzender en de andere ontvangers in een nieuw opstel-tabblad.
mail-action-help-forward = Controleer dit bericht en stuur het door naar ontvangers die u kiest.
mail-action-help-pin = Houd dit bericht vastgezet voor snelle toegang, of maak het los.
mail-action-help-read = Wijzig of dit bericht als gelezen is gemarkeerd. U kunt de wijziging ongedaan maken.

# -- Machine translation (2026-11): added to match en ---------------------
mail-action-help-flag = De berichtvlag voor opvolging instellen of verwijderen.
mail-action-help-archive = Dit bericht naar de ingestelde archiefmap verplaatsen.
mail-action-help-delete = Dit bericht naar de prullenbak verplaatsen. U kunt de verplaatsing ongedaan maken.
mail-action-help-spam = Dit bericht als ongewenst markeren en naar de ingestelde spammap verplaatsen.
mail-action-help-export = De exportpagina openen om dit bericht te downloaden in een formaat naar keuze.
mail-action-help-sweep = Bekijk de andere berichten van deze afzender en kies wat wordt opgeruimd (alles verwijderen, de nieuwste bewaren, oudere verwijderen of de afzender blokkeren) voordat iets wordt toegepast.
mail-empty-trash = Prullenbak legen
mail-trash-help = Bekijk de berichten in de Prullenbak en verwijder ze definitief. Dit kan niet ongedaan worden gemaakt.
mail-trash-preparing = Een momentopname van de berichten in de Prullenbak voorbereiden…
mail-trash-prepare-failed = Het voorbeeld van de Prullenbak kon niet worden voorbereid. Controleer uw verbinding en open het opnieuw. Er zijn geen berichten verwijderd.
mail-trash-confirm = { $count } berichten definitief uit de Prullenbak verwijderen? Dit kan niet ongedaan worden gemaakt. Berichten die na dit voorbeeld binnenkomen, worden uitgesloten; berichten die uit de Prullenbak zijn verplaatst, worden overgeslagen.
mail-trash-limit = Deze bewerking omvat maximaal { $count } berichten. Er blijven meer berichten in de Prullenbak; bekijk daarna een andere bewerking om ze te verwijderen.
mail-trash-permanent = Definitief verwijderen
mail-trash-stop = Stoppen na het huidige verzoek
mail-trash-progress = Verwijderd: { $deleted }; overgeslagen: { $skipped } berichten die niet meer in de Prullenbak staan; nog in dit voorbeeld: { $remaining }.
mail-trash-stopped = Verwijderen gestopt. Bevestigd verwijderd: { $deleted }; overgeslagen: { $skipped }; resterend: { $remaining }; uitkomst onbekend: { $uncertain }. Controleer de Prullenbak voordat u een andere bewerking start.
mail-links-title = Berichtlinks
mail-links-review = Links controleren ({ $count })
mail-links-help = Berichtlinks worden pas geopend nadat u hier de bestemming hebt gecontroleerd. Er wordt niet automatisch een bestemming opgehaald.
mail-links-inert = Gebruik Links controleren boven het bericht om deze bestemming te bekijken en te openen.
mail-links-limit = De eerste 100 verschillende ondersteunde bestemmingen worden getoond. Overige links blijven inactief in het bericht.
mail-links-destination = Te openen bestemming
mail-links-external = Deze bestemming komt van de afzender van het bericht. Het openen ervan verlaat Mailwoman; controleer het adres voordat u doorgaat.
mail-links-open = In een nieuw tabblad openen
mail-links-compose = Een nieuw bericht controleren
mail-links-warning-http = Dit adres gebruikt onversleuteld HTTP.
mail-links-warning-address = Deze bestemming gebruikt een IP-adres in plaats van een domeinnaam.
mail-links-warning-international = Dit domein bevat een geïnternationaliseerde naam. Controleer de ASCII-spelling zorgvuldig.
mail-links-warning-mismatch = Het zichtbare webadres noemt een andere host dan de werkelijke bestemming.
mail-links-warning-direction = De linktekst bevat tekens die de schrijfrichting wijzigen en het adres kunnen verhullen.
mail-outbox-uncertain = Afleverstatus onbekend
mail-outbox-uncertain-note = Het verzenden is begonnen, maar de aflevering kon niet worden bevestigd. Dit bericht wordt niet automatisch opnieuw verzonden. Controleer Verzonden of bevestig het bij de ontvanger voordat u opnieuw verzendt.
mail-conversation-title = Gesprek bekijken
mail-conversation-help = Blader door de beschikbare gespreksberichten van dit account in alle mappen. Het openen van een bericht wijzigt uw mapselectie niet.
mail-conversation-count = { $loaded } van { $total } gespreksberichten worden getoond.
mail-conversation-other-folder = Andere map
mail-conversation-loading = Gesprek laden…
mail-conversation-missing = { $count } bericht(en) zijn niet meer beschikbaar.
mail-conversation-error = Het gesprek kon niet worden geladen. Probeer het opnieuw.
mail-conversation-more = Meer berichten laden
mail-conversation-limit = Deze weergave is beperkt tot { $count } berichten.
mail-compose-request-delivery = Afleveringsmeldingen aanvragen
mail-compose-request-delivery-hint = Vraag de ontvangende mailservers om aflevering, mislukking of vertraging te melden. Dit vraagt niet om, en bewijst niet, dat iemand het bericht heeft gelezen. Teruggestuurde rapporten bevatten koppen, niet de volledige berichttekst.
mail-compose-delivery-unavailable = Afleveringsmeldingen zijn niet beschikbaar of door uw beheerder uitgeschakeld voor dit account. Een uitdrukkelijk aangevraagde melding wordt niet stilzwijgend verwijderd.
mail-compose-account-changed = Dit bericht hoort bij een andere login. Log in bij het juiste account en open het concept opnieuw.
mail-context-open-message = Bericht openen
mail-context-open-folder = Map openen
mail-context-download = Bericht downloaden
mail-context-refresh = Map vernieuwen
mail-context-folder-settings = Mapinstellingen
mail-context-failed = De actie kon niet worden voltooid. Probeer het opnieuw.
mail-sharing-unavailable = Mappen delen is niet beschikbaar of u hebt geen toestemming meer om het te beheren.
mail-trash-cancelling = Stoppen na het huidige verzoek. Bevestigd verwijderd: { $deleted }; overgeslagen: { $skipped }; resterend: { $remaining }. Reeds geaccepteerde verwijderingen kunnen niet ongedaan worden gemaakt.
mail-trash-already-running = Het legen van de Prullenbak loopt al. Gebruik de voortgangsmelding om toekomstige batches te stoppen.
mail-context-search-sender = Deze afzender zoeken
mail-context-search-subject = Hetzelfde onderwerp zoeken
mail-drag-too-many = Sleep maximaal { $count } berichten tegelijk. Selecteer een kleinere groep voordat u verplaatst.
mail-subtab-reorder-help = Sleep om tabbladen te herschikken, of druk op Ctrl+Shift+Pijl-links of Pijl-rechts op het actieve tabblad.
mail-compose-ribbon = Editorlint
mail-compose-ribbon-format = Tekst opmaken
mail-compose-ribbon-insert = Invoegen
mail-compose-font = Lettertype
mail-compose-font-size = Lettergrootte
mail-compose-font-inherit = Standaard
mail-compose-undo = Ongedaan maken
mail-compose-redo = Opnieuw uitvoeren
mail-compose-undo-tip = Ongedaan maken ({ $shortcut })
mail-compose-redo-tip = Opnieuw uitvoeren ({ $shortcut })
mail-workspace-limit = Sluit een tabblad voordat u een ander opent. Maximaal 32 tabbladen kunnen worden behouden.
mail-compose-private-storage-unavailable = Lokaal herstel van concepten is niet beschikbaar voor deze sessie. Houd deze editor open totdat u uw bericht verzendt of kopieert.
mail-offline-storage-unavailable = Offline wijzigingen zijn niet beschikbaar totdat deze sessie de opslagautoriteit heeft geverifieerd. Maak opnieuw verbinding en log opnieuw in.
mail-draft-status-idle = Nog niet in het account opgeslagen
mail-draft-status-pending = Niet-opgeslagen wijzigingen
mail-draft-status-saving = Opslaan in Concepten…
mail-draft-status-saved = Opgeslagen in de Concepten van het account
mail-draft-status-error = Het concept is niet opgeslagen. Uw lokale herstelkopie blijft beschikbaar.
mail-draft-status-uncertain = De opslagbevestiging ontbreekt. Controleer Concepten voordat u nog een kopie maakt; automatisch opslaan is gepauzeerd.
mail-draft-status-conflict = Dit concept is op een ander apparaat gewijzigd. Open de accountkopie opnieuw voordat u doorgaat.
mail-draft-status-unsupported = Dit account kan op dit moment geen opslagbewerkingen in de map Concepten bevestigen.
mail-draft-protected = Versleutelde of ontsleutelde geciteerde inhoud wordt niet als platte tekst opgeslagen. Een eerder accountconcept blijft bestaan totdat u het uitdrukkelijk negeert.
mail-draft-save-now = Opslaan in Concepten
mail-draft-save-failed = Dit concept kon niet in uw account worden opgeslagen. Controleer de editor voordat u hem sluit.
mail-draft-uncertain = Het concept is mogelijk opgeslagen. Automatische nieuwe pogingen zijn gepauzeerd; controleer uw map Concepten.
mail-draft-conflict = Een ander apparaat heeft dit concept gewijzigd. Uw bewerkingen hebben de kopie ervan niet overschreven.
mail-draft-cleanup-pending = Het nieuwe concept is opgeslagen, maar de vorige kopie kon niet worden verwijderd. Controleer Concepten voordat u opnieuw opslaat.
mail-draft-account-list = Accountconcepten
mail-draft-loading = Accountconcepten laden…
mail-draft-close-recovery = Sluiten en alleen lokaal herstel behouden
mail-draft-discard-account = Negeren
mail-draft-discard-pending = Het verwijderen van het concept kon niet worden bevestigd. Het accountrecord blijft bestaan; probeer Negeren opnieuw nadat u de map Concepten hebt bekeken.
mail-draft-limit = Dit account heeft al 100 opgeslagen concepten. Negeer een ongewenst accountconcept voordat u een nieuw opslaat.
mail-draft-quota-help = Maximaal 100 accountconcepten worden bewaard. Negeren verwijdert de bevestigde upstreamkopie voordat de plek wordt vrijgegeven.
mail-draft-discard-confirm = Verwijdering van het concept bevestigen
mail-draft-discard-confirm-detail = “{ $subject }” definitief uit de Concepten van het account verwijderen? Als u de editor in plaats daarvan sluit, blijft het concept behouden.
mail-draft-discard-title = Dit concept negeren?
mail-draft-keep-editing = Blijven bewerken
mail-draft-discard-action = Negeren
mail-draft-close-title = Niet-opgeslagen wijzigingen negeren?
mail-draft-close-protected = Dit bericht bevat versleutelde of ontsleutelde inhoud en wordt daarom niet als concept opgeslagen. Sluiten negeert deze wijzigingen. Een eerder accountconcept blijft, indien aanwezig, in Concepten staan.
mail-draft-close-unstored = Concepten kunnen in deze sessie niet worden opgeslagen, dus sluiten negeert dit bericht.
mail-workspace-view-mode = Weergave-indeling
mail-workspace-tabs-help = Toon één actieve weergave en houd de andere tabbladen open.
mail-workspace-side-by-side-help = Toon twee geselecteerde weergaven naast elkaar. Sleep een tabblad naar een weergave om die te vervangen.
mail-workspace-stack-help = Toon twee geselecteerde weergaven boven elkaar. Sleep een tabblad naar een weergave om die te vervangen.
mail-workspace-masonry-side-by-side = Tegelindeling naast elkaar
mail-workspace-masonry-side-by-side-help = Plaatst alle geopende weergaven in verspringende kolommen. Kleine schermen tonen het actieve tabblad.
mail-workspace-masonry-stack = Tegelindeling gestapeld
mail-workspace-masonry-stack-help = Plaatst alle geopende weergaven in verspringende rijen. Kleine schermen tonen het actieve tabblad.
mail-workspace-view-first = Eerste weergave
mail-workspace-view-second = Tweede weergave
mail-workspace-view-empty = Sleep een geopend tabblad hierheen om deze weergave te kiezen.
mail-popup-blocked = Sta pop-upvensters toe voor Mailwoman om dit tabblad los te maken. Uw tabblad blijft hier.
mail-popup-unavailable = Het losgemaakte venster kon niet worden geladen. Uw tabblad en niet-opgeslagen wijzigingen blijven in het hoofdvenster. Probeer het opnieuw nadat u de app hebt vernieuwd.
mail-popup-reattach = Opnieuw aan het hoofdvenster koppelen
mail-popup-action-reattached = Teruggekeerd naar het hoofdvenster om deze actie te voltooien.
mail-popup-detached = Dit tabblad is geopend in een apart venster. Als u dat venster sluit, keert het hierheen terug.
mail-compose-from-custom = Een ander geautoriseerd adres…
mail-compose-from-alias = Van-adres voor dit bericht
mail-compose-from-alias-help = Gebruik een alias waarmee u mag verzenden. Dit wijzigt alleen dit concept; de autorisatie wordt gecontroleerd vóór het verzenden.
mail-compose-from-refused = Dit adres is niet geautoriseerd om vanuit dit account te verzenden.
mail-compose-from-unavailable = Dit afzenderadres kon niet worden geverifieerd. Uw bericht is niet verzonden.
mail-compose-from-invalid = Voer één geldig e-mailadres in voor de afzender.
mail-compose-from-toggle = De afzender voor dit bericht tonen of verbergen
mail-compose-ribbon-security = Beveiliging
mail-compose-ribbon-delivery = Aflevering
mail-compose-ribbon-recipients = Ontvangers
mail-compose-ribbon-message-type = Berichttype
mail-compose-recall-help = Annuleer berichten die nog worden vastgehouden voor ongedaan maken of geplande aflevering. Reeds afgeleverde berichten kunnen niet worden teruggehaald.
mail-compose-recall-loading = Berichten in de wachtrij controleren…
mail-compose-recall-empty = Er zijn geen berichten in de wachtrij beschikbaar om te annuleren.

# -- Machine translation (2026-11): added to match en ---------------------
mail-compose-recall-failed = De wachtrijstatus kon niet worden bevestigd. Sluit dit paneel en open het opnieuw om het te vernieuwen voordat u nog een actie uitvoert.
mail-compose-importance = Belangrijkheid
mail-compose-importance-high = Hoog
mail-compose-importance-normal = Normaal
mail-compose-importance-low = Laag
mail-insert-account-title = Accountitem invoegen
mail-insert-account-help = Voeg een agenda-export, visitekaartje van een contact of oorspronkelijk bericht uit dit account als bijlage toe.
mail-insert-account-close = Kiezer voor accountitems sluiten
mail-insert-account-kind = Itemtype
mail-insert-account-event = Agenda-afspraak (.ics)
mail-insert-account-calendar = Agenda (.ics)
mail-insert-account-contact = Visitekaartje van contact (.vcf)
mail-insert-account-message = Oorspronkelijk bericht (.eml)
mail-insert-account-search = Vermelde items filteren
mail-insert-account-selection = Selecteer een item
mail-insert-account-scope = Alleen items uit het huidige account worden vermeld. Oorspronkelijke berichten volgen het exportbeleid van de server.
mail-insert-account-limited = Maximaal 100 items worden getoond. Filteren doorzoekt alleen deze lijst.
mail-insert-account-unavailable = Er zijn geen ondersteunde accountexports beschikbaar.
mail-insert-account-empty = Geen overeenkomende items.
mail-insert-account-attach = Geselecteerd item bijvoegen
mail-insert-account-loading = Accountitem laden…
mail-insert-account-error = Het item kon niet worden bijgevoegd. Controleer uw toegang en het exportbeleid en probeer het opnieuw. Grote agenda's vereisen mogelijk afzonderlijke exports van afspraken.
mail-compose-html-source = HTML-broncode
mail-editor-clear-format = Opmaak wissen
mail-compose-format-html = HTML
mail-compose-html-visual = Visuele editor
mail-compose-html-help = Bewerk de HTML rechtstreeks. Alleen ondersteunde e-mailopmaak wordt opgeslagen en verzonden; scripts, externe afbeeldingen en onveilige opmaak worden verwijderd.
mail-compose-html-processing = Veilige HTML voorbereiden…
mail-compose-html-failed = De HTML kon niet worden voorbereid. Bewerk haar of probeer het opnieuw voordat u opslaat of verzendt.
mail-compose-check-names = Namen controleren
mail-compose-check-names-help = Zet exacte namen om met uw geladen contacten en controleert de opmaak van adressen. Dit verifieert niet of er in de mailbox kan worden afgeleverd.
mail-compose-names-unresolved = Kies een volledig e-mailadres voor deze niet-herkende of dubbelzinnige namen: { $names }
mail-compose-names-checked = Opmaak van ontvangers gecontroleerd; { $count } contactna(a)m(en) herkend. Aflevering in de mailbox is niet geverifieerd.
mail-compose-ribbon-table = Tabel
mail-compose-group-clipboard = Klembord
mail-compose-group-paragraph = Alinea
mail-compose-group-editing = Bewerken
mail-compose-group-view = Weergave
mail-compose-dark-background = Donkere achtergrond
mail-compose-dark-background-help = Toont het bericht op een donkere achtergrond terwijl u schrijft. Alleen uw scherm verandert: het bericht wordt zonder achtergrondkleur verzonden.
mail-compose-group-assist = Schrijfassistent
mail-editor-counts = Editorstatistieken
mail-editor-words = { $count ->
    [one] { $count } woord
   *[other] { $count } woorden
}
mail-editor-paragraphs = { $count ->
    [one] { $count } alinea
   *[other] { $count } alinea's
}
mail-editor-characters = { $count ->
    [one] { $count } teken
   *[other] { $count } tekens
}
mail-editor-selection-required = Selecteer geschikte tekst of tabelcellen voordat u deze actie gebruikt.
mail-editor-clipboard-unavailable = Toegang tot het klembord was niet beschikbaar. Gebruik de sneltoets Kopiëren, Knippen of Plakken van uw browser.
mail-editor-selection-changed = De selectie is gewijzigd. Selecteer de bedoelde tekst en probeer het opnieuw.
mail-editor-clipboard = Klembord en opmaak
mail-editor-copy = Kopiëren
mail-editor-cut = Knippen
mail-editor-paste = Plakken
mail-editor-copy-format = Opmaak kopiëren
mail-editor-apply-format = Opmaak toepassen
mail-editor-style = Alinea-opmaakprofiel
mail-editor-text-color = Letterkleur
mail-editor-clear-color = Standaardkleur
mail-editor-highlight = Markeren
mail-editor-clear-highlight = Markering wissen
mail-editor-paragraph = Alinea-opties
mail-editor-alignment = Uitlijning
mail-editor-line-height = Regelhoogte
mail-editor-space-before = Ruimte voor
mail-editor-space-after = Ruimte na
mail-editor-indent = Inspringing
mail-editor-shading = Alineaschaduw
mail-editor-clear-shading = Schaduw wissen
mail-editor-case = Hoofdletters wijzigen
mail-editor-sort = Geselecteerde alinea's sorteren
mail-editor-sort-alpha = Alfabetisch
mail-editor-sort-numeric = Numerieke volgorde
mail-editor-table-rows = Tabelrijen
mail-editor-table-columns = Tabelkolommen
mail-editor-table-insert = Tabel invoegen
mail-editor-table-limits = Maximaal 20 rijen, 10 kolommen en 200 cellen per tabel; hoogstens 10 tabellen. Geneste tabellen worden niet ondersteund.
mail-editor-table-row-before = Rij erboven
mail-editor-table-row-after = Rij eronder
mail-editor-table-row-delete = Rij verwijderen
mail-editor-table-column-before = Kolom ervoor
mail-editor-table-column-after = Kolom erna
mail-editor-table-column-delete = Kolom verwijderen
mail-editor-table-merge = Cellen samenvoegen
mail-editor-table-split = Cel splitsen
mail-editor-table-header = Koprij in-/uitschakelen
mail-editor-table-delete = Tabel verwijderen
mail-editor-table-shading = Celschaduw
mail-editor-find-replace = Zoeken en vervangen
mail-editor-find = Tekst zoeken
mail-editor-replacement = Vervangen door
mail-editor-match-case = Hoofdlettergevoelig
mail-editor-previous = Vorige overeenkomst
mail-editor-next = Volgende overeenkomst
mail-editor-replace = Geselecteerde overeenkomst vervangen
mail-editor-replace-all = Alle overeenkomsten vervangen
mail-editor-matches = { $count } overeenkomsten (maximaal 1.000)
mail-editor-select-all = Alles selecteren
mail-editor-clear-selection = Selectie wissen
mail-editor-history = Geschiedenis van ongedaan maken
mail-editor-undo-groups = De laatste { $count } wijzigingsgroepen ongedaan maken
mail-editor-zoom = Editorzoom
mail-editor-markers = Alineatekens
mail-editor-style-normal = Normaal
mail-editor-style-no-spacing = Geen tussenruimte
mail-editor-style-title = Titel
mail-editor-style-subtitle = Ondertitel
mail-editor-style-heading1 = Kop 1
mail-editor-style-heading2 = Kop 2
mail-editor-style-heading3 = Kop 3
mail-editor-style-heading4 = Kop 4
mail-editor-style-heading5 = Kop 5
mail-editor-style-heading6 = Kop 6
mail-editor-align-left = Links
mail-editor-align-center = Gecentreerd
mail-editor-align-right = Rechts
mail-editor-align-justify = Uitvullen
mail-editor-case-lower = kleine letters
mail-editor-case-upper = HOOFDLETTERS
mail-editor-case-title = Elk Woord Met Hoofdletter
mail-editor-case-sentence = Hoofdletter aan het begin van de zin
mail-editor-action-history = Geschiedenis van { $action }
mail-editor-history-empty = Geen acties beschikbaar
mail-editor-redo-groups = { $count } wijzigingen opnieuw uitvoeren
mail-compose-identities-failed = De verzendidentiteiten konden niet worden geladen. Sluit dit concept en open het opnieuw om het nogmaals te proberen.
mail-folders-loading = Mappen laden…
mail-initial-sync-error = Uw berichten konden nog niet worden gesynchroniseerd.
mail-initial-sync-error-help = De verbinding met uw mailserver is verbroken. Probeer het opnieuw, of vraag uw beheerder de verbindingslogboeken te controleren als dit blijft gebeuren.
mail-links-trusted = Vertrouwd domein: opent zonder controle
mail-links-trust-domain = { $domain } vertrouwen
mail-links-trust-failed = Het vertrouwde domein kon niet worden opgeslagen. Probeer het opnieuw.
mail-focus-always-focused = Altijd naar Gericht verplaatsen
mail-focus-always-other = Altijd naar Overig verplaatsen

# -- Machine translation (2026-11): added to match en ---------------------
mail-focus-moved-focused = Berichten van { $sender } gaan nu naar Gericht.
mail-focus-moved-other = Berichten van { $sender } gaan nu naar Overig.
mail-focus-save-failed = De keuze voor Gerichte inbox voor deze afzender kon niet worden opgeslagen. Probeer het opnieuw.
mail-context-search-domain = Berichten van dit domein zoeken
mail-loading-mailbox = { $name } laden…
mail-user-folders = Gebruikersmappen
mail-context-hide-folder = Verbergen in zijbalk
mail-folder-fallback = map
mail-toast-moving = { $count ->
    [one] Eén bericht naar { $folder } verplaatsen…
   *[other] { $count } berichten naar { $folder } verplaatsen…
}
mail-toast-move-queueing = { $count ->
    [one] Eén bericht in de wachtrij zetten om naar { $folder } te verplaatsen…
   *[other] { $count } berichten in de wachtrij zetten om naar { $folder } te verplaatsen…
}
mail-toast-move-queued = { $count ->
    [one] Eén bericht in de wachtrij gezet om naar { $folder } te verplaatsen
   *[other] { $count } berichten in de wachtrij gezet om naar { $folder } te verplaatsen
}
mail-toast-move-partial = { $count } van { $total } berichten naar { $folder } verplaatst; { $failed } konden niet worden verplaatst
mail-send-copy-folder-unavailable = De geselecteerde map voor de verzonden kopie is niet beschikbaar. Kies een andere map voordat u verzendt.
mail-send-copy-failed = Het bericht is geaccepteerd, maar de verzonden kopie kon niet worden opgeslagen. Verzend het niet opnieuw.
mail-compose-attach-public-key = Mijn openbare sleutel of certificaat bijvoegen
mail-compose-include-hashes = SHA-256-hashes van bijgevoegde bestanden opnemen
mail-compose-hashes-pending = Hashes van bijlagen worden berekend. Verzend wanneer het klaar is.
mail-compose-hashes-failed = De bytes van de bijlagen konden niet worden geverifieerd. Schakel hashes uit of probeer het opnieuw door ze weer in te schakelen.
mail-compose-crypto-pending = De versleuteling wordt bijgewerkt. Wacht even voordat u verzendt.
mail-compose-public-key-unavailable = Geen huidige openbare sleutel of certificaat komt overeen met het verzendadres.
mail-compose-sent-copy = Verzonden bericht opslaan in
mail-compose-sent-copy-default = Standaard van het account
folder-create = Map maken
folder-name = Mapnaam
folder-parent = Bovenliggende map
folder-top-level = Hoogste niveau
folder-created = Map gemaakt.
folder-create-unavailable = Mappen maken is niet beschikbaar voor dit account of deze bovenliggende map.
folder-create-invalid = Kies een geldige, unieke mapnaam zonder hiërarchiescheidingstekens of stuurtekens.
folder-create-failed = De map kon niet worden gemaakt. Vernieuw en probeer het opnieuw.
folder-cleanup = Map opschonen…
folder-cleanup-title = { $name } opschonen
folder-cleanup-help = Zoek berichten die ouder zijn dan de gekozen leeftijd. Controleer het aantal voordat u verwijdert. Submappen worden uitgesloten.
folder-cleanup-age = Ouder dan
folder-age-days = Dagen
folder-age-weeks = Weken
folder-age-months = Maanden
folder-age-years = Jaren
folder-cleanup-preview = Voorbeeld van opschonen
folder-cleanup-confirm = Overeenkomende berichten verwijderen
folder-cleanup-permanent = { $count } overeenkomende berichten definitief verwijderen. Dit kan niet ongedaan worden gemaakt.
folder-cleanup-trash = { $count } overeenkomende berichten naar de Prullenbak verplaatsen.
folder-cleanup-before = Ontvangen vóór { $date }
folder-cleanup-done = { $count } berichten opgeschoond.
mail-from = Afzender
mail-subject = Onderwerp
mail-date = Datum
mail-cancel = Annuleren
mail-bulk-selected = { $count } gesprekken geselecteerd
mail-bulk-select-folder = Alles in { $folder } selecteren
mail-list-order = Berichtvolgorde
mail-list-order-newest = Nieuwste eerst
mail-list-order-oldest = Oudste eerst
mail-list-order-sender = Afzender
mail-list-order-subject = Onderwerp
mail-list-order-error = De berichtvolgorde kon niet worden opgeslagen.
mail-bulk-selection-load-failed = De volledige map kon niet worden geselecteerd. Controleer uw verbinding en probeer het opnieuw.
mail-privacy-bar = Privacy van het bericht
mail-privacy-trackers-allow = Trackers voor dit bericht toestaan
mail-privacy-trackers-block = Trackers weer blokkeren
mail-privacy-images-loading = { $loaded }/{ $total } afbeeldingen getoond · { $pending } worden geladen
mail-privacy-images-failed = { $loaded }/{ $total } afbeeldingen getoond · { $failed } mislukt
mail-privacy-images-loaded = { $loaded }/{ $total } afbeeldingen getoond
mail-privacy-images-loading-failed = { $loaded }/{ $total } afbeeldingen getoond · { $pending } worden geladen · { $failed } mislukt
mail-reader-folder-title = Map: { $folder }
mail-reader-move-to = Verplaatsen naar…
mail-action-help-move = Dit bericht naar een andere map verplaatsen.
folder-ops-new-subfolder = Nieuwe submap
folder-ops-mark-read = Alles als gelezen markeren
folder-ops-this-folder = Deze map
folder-ops-with-subfolders = Submappen opnemen
folder-ops-no-unread = Geen ongelezen berichten.
folder-ops-read-unavailable = U kunt berichten in deze map niet wijzigen.
folder-ops-more = Meer
folder-ops-system-folder = Een systeemmap kan niet op deze manier worden gewijzigd.
folder-ops-sort-subfolders = Submappen sorteren op
folder-ops-sort-folders = Mappen sorteren op
folder-ops-sort-name = Naam
folder-ops-sort-unread = Aantal ongelezen
folder-ops-sort-total = Totaal aantal
folder-ops-sort-date = Laatste activiteit
folder-ops-sort-custom = Aangepaste volgorde
folder-ops-sort-asc = Oplopend
folder-ops-sort-desc = Aflopend
folder-ops-delete-with-subfolders = Map en submappen verwijderen
folder-ops-delete-unavailable = Deze map kan niet worden verwijderd.
folder-ops-delete-tree-body = Dit verwijdert { $folders } mappen en de { $count } berichten daarin definitief. Dit kan niet ongedaan worden gemaakt.
folder-ops-copy-all-menu = Alle items kopiëren naar…
folder-ops-move-all-menu = Alle items verplaatsen naar…
folder-ops-move-folder-menu = Map verplaatsen naar…
folder-ops-move-up = Omhoog verplaatsen
folder-ops-move-down = Omlaag verplaatsen
folder-ops-move-unavailable = U kunt deze map of de items erin niet verplaatsen.
folder-ops-move-duplicate = De bestemming heeft al een map met deze naam.
folder-ops-move-folder-title = { $name } verplaatsen
folder-ops-move-folder-body = De map wordt samen met haar submappen en berichten verplaatst. Kies de map waarin zij komt te staan, of het hoogste niveau.
folder-ops-move-here = Hierheen verplaatsen
folder-ops-moved-folder = { $name } verplaatst.
folder-ops-choose-folder = Kies een bestemmingsmap.
folder-ops-destination = Bestemmingsmap
folder-ops-include-subfolders = De berichten van { $count } submappen opnemen
folder-ops-copy-all-title = Alle items van { $name } kopiëren
folder-ops-move-all-title = Alle items van { $name } verplaatsen
folder-ops-copy-all-body = Voegt { $count } berichten toe aan de bestemmingsmap. Ze blijven waar ze zijn.
folder-ops-move-all-body = Verplaatst { $count } berichten naar de bestemmingsmap. U kunt dit achteraf ongedaan maken.
folder-ops-copy-all = Items kopiëren
folder-ops-move-all = Items verplaatsen
folder-ops-undo = Ongedaan maken
folder-ops-stop = Stoppen
folder-ops-marking = { $name } als gelezen markeren…
folder-ops-marking-progress = { $name } als gelezen markeren: { $done } van { $total }
folder-ops-marked = { $count } berichten in { $name } als gelezen gemarkeerd.
folder-ops-copying = { $name } kopiëren…
folder-ops-copying-progress = { $name } kopiëren: { $done } van { $total }
folder-ops-copied = { $count } berichten uit { $name } gekopieerd.
folder-ops-moving = { $name } verplaatsen…
folder-ops-moving-progress = { $name } verplaatsen: { $done } van { $total }
folder-ops-moved = { $count } berichten uit { $name } verplaatst.
folder-ops-stopped = { $name } gestopt na { $count } van { $total } berichten.
folder-ops-partial = { $count } van { $total } berichten in { $name } zijn verwerkt; { $refused } zijn geweigerd.
folder-ops-failed = { $name } kon niet worden voltooid. Vernieuw en probeer het opnieuw.
folder-ops-error-forbidden = U hebt niet de rechten voor deze wijziging.
folder-ops-error-exists = Daar bestaat al een map met deze naam.
folder-ops-error-mailboxHasEmail = De map bevat nog berichten.
folder-ops-error-notFound = De map bestaat niet meer.
folder-ops-error-failed = De wijziging kon niet worden doorgevoerd. Vernieuw en probeer het opnieuw.
folder-ops-expand = Submappen van { $name } tonen
folder-ops-collapse = Submappen van { $name } verbergen
mail-context-reply = Beantwoorden
mail-context-reply-all = Allen beantwoorden
mail-context-forward = Doorsturen

# -- Machine translation (2026-11): added to match en ---------------------
mail-context-quick-print = Snel afdrukken
mail-context-copy = Kopiëren
mail-context-copy-to-folder = Bericht naar map kopiëren…
mail-context-copy-sender = Afzenderadres kopiëren
mail-context-copy-subject = Onderwerp kopiëren
mail-context-copy-link = Berichtlink kopiëren
mail-context-copied = Gekopieerd.
mail-context-copy-failed = Kopiëren naar het klembord is niet gelukt.
mail-context-print-failed = Het bericht kon niet worden afgedrukt.
mail-reply-images-title = Afbeeldingen uit het oorspronkelijke bericht
mail-reply-images-ask = De afbeeldingen van het oorspronkelijke bericht in dit antwoord opnemen?
mail-reply-images-include = Afbeeldingen opnemen
mail-reply-images-not-now = Nu niet
mail-reply-images-trust = { $domain } altijd vertrouwen
mail-reply-images-loading = Afbeeldingen toevoegen…
mail-reply-images-cancelled = Laden geannuleerd. Er zijn geen afbeeldingen toegevoegd.
mail-reply-images-failed = De afbeeldingen konden niet worden toegevoegd.
mail-reply-images-retry = Opnieuw laden
mail-image-cancel = Annuleren
mail-compose-from-why-match = Verzenden vanaf { $identity } omdat het bericht naar { $address } is verzonden.
mail-compose-from-why-match-plus = Verzenden vanaf { $identity } omdat het bericht naar { $address } is verzonden, een plusadres ervan.
mail-compose-from-why-ambiguous = Meerdere identiteiten komen overeen met { $address }. Verzenden vanaf { $identity }; kies een andere:
mail-compose-from-why-default = Verzenden vanaf uw standaardidentiteit, { $identity }, omdat geen enkele overeenkomt met de oorspronkelijke ontvangers.
mail-compose-from-why-forced-default = Verzenden vanaf uw standaardidentiteit, { $identity }, zoals ingesteld in uw instellingen.
mail-compose-from-why-keep = Verzenden vanaf { $identity }.
mail-compose-from-change = Wijzigen
reader-attachments-reason-encrypted = Dit bericht is versleuteld. De bijlagen verschijnen nadat u het hebt ontgrendeld.
reader-attachments-reason-list-missing = De bijlagenlijst van dit bericht kon niet worden geladen, daarom is de bijlagenbalk verborgen.
reader-attachments-reason-inline-only = De bijlagen van dit bericht ({ $count }) zijn inlineafbeeldingen die in het bericht zelf worden getoond.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] Eén bijlage van dit bericht heeft geen downloadbare inhoud, daarom is de bijlagenbalk verborgen.
   *[other] { $count } bijlagen van dit bericht hebben geen downloadbare inhoud, daarom is de bijlagenbalk verborgen.
}
reader-attachments-reason-policy-blocked = Uw bijlagenbeleid blokkeert het downloaden van { $count ->
    [one] deze bijlage.
   *[other] deze bijlagen.
}
reader-attachments-reason-policy-unavailable = Het bijlagenbeleid kon niet worden gecontroleerd, daarom kunnen bijlagen nog niet worden geopend.
mail-drafts-popup-count = { $count ->
    [one] 1 concept
   *[other] { $count } concepten
}
mail-drafts-popup-search = Zoeken op onderwerp of ontvanger
mail-drafts-popup-tab-local = Lokale concepten
mail-drafts-popup-local-hint = Lokale concepten worden alleen in deze browser bewaard.
mail-drafts-popup-badge-account = Account
mail-drafts-popup-badge-local = Lokaal
mail-drafts-popup-current = Nu geopend
mail-drafts-popup-select = Selecteren
mail-drafts-popup-select-all = Alles selecteren
mail-drafts-popup-clear = Wissen
mail-drafts-popup-done = Gereed
mail-drafts-popup-selected = { $count } geselecteerd
mail-drafts-popup-select-row = { $subject } selecteren
mail-drafts-popup-open = Openen
mail-drafts-popup-discard = Negeren
mail-drafts-popup-discard-selected = Geselecteerde negeren
mail-drafts-popup-more = Meer acties voor { $subject }
mail-drafts-popup-retry = Opnieuw proberen
mail-drafts-popup-load-failed = De accountconcepten konden niet worden geladen.
mail-drafts-popup-empty-account = Geen accountconcepten
mail-drafts-popup-no-match = Geen concepten komen overeen met “{ $query }”
mail-drafts-popup-discarding = Negeren…
mail-drafts-popup-row-failed = Kon niet worden genegeerd. De accountkopie blijft bestaan.
mail-drafts-popup-confirm-title = { $count ->
    [one] Dit concept negeren?
   *[other] { $count } concepten negeren?
}
mail-drafts-popup-confirm-account = Accountconcepten worden eerst uit de map Concepten van uw account verwijderd en hun plek wordt pas vrijgegeven wanneer die verwijdering is bevestigd.
mail-drafts-popup-confirm-local = Lokale concepten worden uit deze browser verwijderd.
mail-drafts-popup-announce-done = { $count ->
    [one] 1 concept genegeerd
   *[other] { $count } concepten genegeerd
}
mail-drafts-popup-announce-failed = { $done } genegeerd, { $failed } konden niet worden genegeerd
mail-check-title = Controleren vóór het verzenden
mail-check-intro = Hier kijkt u misschien nog eens naar:
mail-check-invalid = Sommige ontvangersadressen lijken ongeldig: { $list }
mail-check-subject = Het onderwerp is leeg.
mail-check-attachment = Het bericht noemt een bijlage, maar er is niets bijgevoegd.
mail-check-external = Ontvangers buiten vertrouwde domeinen: { $list }
mail-check-send-anyway = Toch verzenden
mail-check-review = Controleren
mail-check-remember = Hier niet meer naar vragen
mail-check-denied-invalid = Verzenden wordt geblokkeerd door uw instellingen. Corrigeer de ongeldige ontvangersadressen: { $list }
mail-check-denied-subject = Verzenden wordt geblokkeerd door uw instellingen. Voeg een onderwerp toe.
mail-check-denied-attachment = Verzenden wordt geblokkeerd door uw instellingen. Voeg het genoemde bestand als bijlage toe of formuleer het bericht anders.
mail-check-denied-external = Verzenden wordt geblokkeerd door uw instellingen. Buiten vertrouwde domeinen: { $list }
mail-alerts-label = Problemen met het bericht
mail-alert-invalid = Ongeldig adres: { $address }
mail-alert-subject = Geen onderwerp
mail-recipient-chip-remove = { $address } verwijderen
mail-recipient-chips = Ontvangers in { $field }
mail-recipient-invalid-title = Geen geldig e-mailadres
mail-recipient-chip-name = { $name }, { $address }. Enter voor contactgegevens, Backspace of F2 om te bewerken, Delete om te verwijderen
mail-recipient-chip-name-invalid = { $address }, geen geldig adres. Enter om te bewerken, Delete om te verwijderen
mail-recipient-added = { $name } toegevoegd
mail-recipient-removed = { $name } verwijderd
mail-recipient-removed-many = { $count ->
    [one] { $count } ontvanger verwijderd
   *[other] { $count } ontvangers verwijderd
}
mail-recipient-editing = { $name } bewerken
mail-recipient-all-selected = { $count ->
    [one] { $count } ontvanger geselecteerd
   *[other] { $count } ontvangers geselecteerd
}
mail-external-line = { $count } buiten vertrouwde domeinen: { $domains }
mail-external-dismiss = Sluiten
mail-status-label = Status van de editor
mail-status-more = { $count } meer meldingen
mail-status-more-label = Meer meldingen
mail-status-no-recipients = Geen ontvangers
mail-status-no-attachment = Geen bijlage
mail-status-external = Extern: { $count }
mail-status-external-reviewed = Externe ontvangers gecontroleerd
mail-status-external-review = Als gecontroleerd markeren
mail-status-from-match = Van { $identity }
mail-status-from-match-plus = Van { $identity }
mail-status-from-ambiguous = Van { $identity } (meerdere komen overeen)
mail-status-from-default = Van standaardidentiteit
mail-status-from-forced-default = Van standaardidentiteit
mail-status-from-keep = Van { $identity }
mail-status-held = { $count } in de wachtrij
mail-status-storage = Geen lokaal herstel
mail-status-protected = Niet als platte tekst opgeslagen
mail-status-saving = Opslaan…
mail-status-saved = Opgeslagen { $time }
mail-status-saved-full = Opgeslagen in Concepten van het account · { $time }
mail-status-not-saved = Concept niet opgeslagen
mail-status-retry = Opnieuw proberen
mail-status-conflict = Op een ander apparaat gewijzigd
mail-status-uncertain = Opslaan niet bevestigd
mail-status-close-recovery = Sluiten, herstel behouden
mail-status-unsupported = Opslaan van concept niet bevestigd
mail-compose-header = Berichtkop
mail-header-label = Berichtkop
mail-header-collapse = Berichtkop samenvouwen
mail-header-expand = Berichtkop uitvouwen: { $summary }
mail-header-from-chip = Van { $address }. Afzender wijzigen
mail-compose-group-timing = Verzendtijdstip
mail-compose-group-priority = Prioriteit
mail-compose-group-receipts = Bevestigingen en tracking
mail-compose-group-keys-files = Sleutels en bestanden
mail-compose-group-sent-copy = Verzonden kopie
mail-editor-style-quote = Citaat
mail-editor-style-code = Code
mail-compose-ribbon-table-layout = Tabelindeling
mail-compose-ribbon-picture = Afbeelding
mail-editor-table-group-rows = Rijen
mail-editor-table-group-columns = Kolommen
mail-editor-table-group-cells = Cellen
mail-editor-table-row-height = Rijhoogte
mail-editor-table-column-width = Kolombreedte
mail-editor-table-size-auto = Automatisch
mail-editor-table-padding = Celopvulling
mail-editor-table-padding-compact = Compact
mail-editor-table-padding-normal = Normaal
mail-editor-table-padding-roomy = Ruim
mail-editor-table-align-left = Cel links uitlijnen

# -- Machine translation (2026-11): added to match en ---------------------
mail-editor-table-align-center = Cel centreren
mail-editor-table-align-right = Cel rechts uitlijnen
mail-picture-group-size = Grootte
mail-picture-group-arrange = Rangschikken
mail-picture-group-text = Tekst
mail-picture-group-style = Stijl
mail-picture-group-edit = Bewerken
mail-picture-width = Breedte
mail-picture-height = Hoogte
mail-picture-lock = Verhoudingen behouden
mail-picture-reset-size = Oorspronkelijke grootte
mail-picture-align-inline = In lijn met de tekst
mail-picture-align-left = Links uitlijnen
mail-picture-align-center = Centreren
mail-picture-align-right = Rechts uitlijnen
mail-picture-alt = Alternatieve tekst
mail-picture-title = Titel
mail-picture-border = Rand
mail-picture-radius = Hoeken
mail-picture-none = Geen
mail-picture-round = Afgerond
mail-picture-rotate-ccw = Naar links draaien
mail-picture-rotate-cw = Naar rechts draaien
mail-picture-flip-h = Horizontaal spiegelen
mail-picture-flip-v = Verticaal spiegelen
mail-picture-crop = Bijsnijden
mail-picture-crop-left = Links (%)
mail-picture-crop-top = Boven (%)
mail-picture-crop-right = Rechts (%)
mail-picture-crop-bottom = Onder (%)
mail-picture-crop-apply = Bijsnijden
mail-picture-replace = Vervangen
mail-picture-remove = Verwijderen
mail-picture-failed-no-image = Selecteer eerst een afbeelding.
mail-picture-failed-no-source = Deze afbeelding heeft geen lokale kopie om te bewerken. Vervang haar in plaats daarvan.
mail-picture-failed-failed = De afbeelding kon niet worden gewijzigd.
mail-context-download-attachments = Bijlagen downloaden
mail-context-download-named = { $name } downloaden
mail-context-download-all = Alles als ZIP downloaden
mail-context-loading-attachments = Bijlagen laden…
mail-context-attachments-failed = De bijlagen konden niet worden geladen.
mail-context-searching = Het bericht zoeken…
mail-context-go-reply = Naar het antwoord gaan
mail-context-go-forward = Naar het doorgestuurde bericht gaan
mail-context-reply-not-found = Antwoord niet gevonden
mail-context-forward-not-found = Doorgestuurd bericht niet gevonden
mail-no-recipients = (geen ontvangers)
mail-attachment-downloaded = { $name } gedownload
mail-attachment-download-failed = Het downloaden is mislukt.
mail-attachments-zip-done = { $count } bijlagen als ZIP gedownload.
mail-attachments-zip-capped = De ZIP bevat de eerste { $count } bijlagen; de limiet is { $max } bestanden of { $size }.
mail-focus-not-found = Bericht niet gevonden in { $folder }
mail-focus-go-to = Naar { $folder } gaan
mail-nav-views-queues = Weergaven en wachtrijen
mail-badge-unread = { $unread ->
    [one] { $unread } ongelezen bericht
   *[other] { $unread } ongelezen berichten
}
mail-badge-total = { $total ->
    [one] { $total } bericht in deze map
   *[other] { $total } berichten in deze map
}
mail-badge-both = { $unread ->
    [one] { $unread } ongelezen bericht
   *[other] { $unread } ongelezen berichten
} van { $total ->
    [one] { $total } in deze map
   *[other] { $total } in deze map
}
mail-badge-drafts = { $total ->
    [one] Concepten: { $total } bericht
   *[other] Concepten: { $total } berichten
}
mail-badge-compact-hint = Andere mappen dan de Inbox gebruiken een kleinere badge. Wijzig dit onder Instellingen, Mappen en zijbalk.
mail-badge-outbox = { $count ->
    [one] Postvak UIT: { $count } bericht wacht om te worden verzonden
   *[other] Postvak UIT: { $count } berichten wachten om te worden verzonden
}
mail-color-menu = Berichtkleuren
mail-color-button = Berichtkleuren: { $mode }
mail-color-use-default = Mijn standaard gebruiken ({ $mode })
mail-color-make-default = Dit mijn standaard maken
mail-color-locked = Vergrendeld door uw beheerder
mail-color-confirm-title = Uw standaardberichtkleuren wijzigen?
mail-color-confirm-body = Elk bericht gebruikt "{ $mode }", tenzij u dit voor één bericht wijzigt.
mail-color-confirm = Standaard maken
mail-color-default-failed = Uw standaardberichtkleuren konden niet worden opgeslagen.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mail-attachments-none = De geselecteerde berichten hebben geen bijlagen om te downloaden.

mail-attachments-zip-all-blocked = Er is niets gedownload: het bijlagebeleid blokkeert alle { $skipped } bestanden.

mail-attachments-zip-gathering = De bijlagen van { $count } berichten verzamelen…

mail-attachments-zip-skipped = { $skipped ->
        [one] Eén bestand dat werd geblokkeerd door het bijlagebeleid, werd weggelaten.
       *[other] { $skipped } bestanden die door het bijlagebeleid zijn geblokkeerd, zijn weggelaten.
    }

mail-attachments-zip-unreadable = De bijlagen van { $count } berichten konden niet worden gelezen en werden weggelaten.

mail-bulk-download-attachments = Bijlagen downloaden als ZIP

mail-compose-from-via = Het kwam overeen met de koptekst { $header }.

mail-compose-from-why-match-domain = Verzenden vanaf { $identity } omdat het bericht is afgeleverd op { $address }, een adres in hetzelfde domein.

mail-compose-from-why-match-pattern = Verzenden vanaf { $identity } omdat het bericht is verzonden naar { $address }, wat past bij het adrespatroon van die identiteit.

mail-compose-ribbon-autohide = Lint automatisch verbergen (Ctrl+F1)

mail-compose-ribbon-pin = Lint altijd tonen (Ctrl+F1)

mail-context-download-selected-attachments = { $count ->
        [one] Download bijlagen van het geselecteerde bericht als ZIP
       *[other] Download bijlagen van { $count } geselecteerde berichten als ZIP
    }

mail-context-download-thread-attachments = Download alle { $count } bijlagen van het gesprek als ZIP

mail-context-open-new-tab = Openen in een nieuw tabblad

mail-delete-sender-confirm = Verplaats { $count } berichten naar de prullenbak

mail-delete-sender-detail = { $count } berichten van { $sender } naar de prullenbak verplaatsen? Berichten die al in de prullenbak staan, worden uitgesloten. U kunt deze actie ongedaan maken.

mail-delete-sender-empty = Er zijn geen berichten van deze afzender buiten de Prullenbak.

mail-delete-sender-failed = Kon niet elk overeenkomend bericht en de bijbehorende machtigingen verifiëren. Er werd niets verplaatst; probeer het opnieuw.

mail-delete-sender-help = Zoek de berichten van deze afzender in alle mappen en verplaats ze na bevestiging naar de prullenbak.

mail-delete-sender-title = Verwijder alle berichten van deze afzender

mail-draft-continue = Ga door met bewerken

mail-draft-edit = Concept bewerken

mail-draft-edit-failed = Het concept kon niet worden geopend voor bewerking

mail-draft-edit-help = Open dit concept in de componist om het te blijven bewerken

mail-draft-notice = Dit is een ontwerp

mail-file-copied = Gekopieerd naar het klembord.

mail-file-copy-failed = Kan niet naar het klembord kopiëren.

mail-file-copy-fingerprint = Vingerafdruk kopiëren

mail-file-copy-name = Bestandsnaam kopiëren

mail-file-created = Gemaakt

mail-file-creator = Gemaakt met

mail-file-details-button = Bestandsdetails

mail-file-details-tooltip = Toon bestandsdetails en vingerafdruk

mail-file-encryption = Wachtwoordbeveiliging

mail-file-encryption-none = Er werd niet om een wachtwoord gevraagd

mail-file-encryption-password = Om het te openen was een wachtwoord vereist

mail-file-fingerprint = Vingerafdruk

mail-file-name = Bestandsnaam

mail-file-pages = Pagina's

mail-file-pdf-version = PDF-versie

mail-file-producer = Producent

mail-file-verify-help = Vergelijk deze groepen met een vingerafdruk die de afzender u op een andere manier heeft gegeven, zoals een telefoontje of hun website. Als elke groep overeenkomt, heb je precies dezelfde bytes.

mail-file-verify-title = Controleer de vingerafdruk

mail-follow-up-finished = Vervolgafspraak afgerond

mail-group-by = Groepeer op

mail-header-attachments = { $count ->
        [one] { $count } bijlage
       *[other] { $count } bijlagen
    }

mail-header-expand-short = Berichtkop uitvouwen (Alt+H)

mail-list-group-date-unknown = Onbekende datum

mail-list-group-importance = Belang

mail-list-group-locked = Het groeperen van berichten is vergrendeld door uw beheerder.

mail-list-group-no-subject = (Geen onderwerp)

mail-list-group-none = Geen groepering

mail-list-group-priority-high = Groot belang

mail-list-group-priority-low = Gering belang

mail-list-group-priority-normal = Normaal belang

mail-list-group-received = Datum ontvangen

mail-list-group-sender = Van

mail-list-group-sent = Datum verzonden

mail-list-group-size = Grootte

mail-list-group-size-huge = 10 MB en hoger

mail-list-group-size-large = 1–10 MB

mail-list-group-size-medium = 100 KB–1 MB

mail-list-group-size-small = 25–100 KB

mail-list-group-size-tiny = Onder 25 KB

mail-list-group-subject = Onderwerp

mail-list-order-importance = Belang eerst hoog

mail-list-order-importance-asc = Belang eerst laag

mail-list-order-locked = De berichtvolgorde is vergrendeld door uw beheerder.

mail-list-order-sender-desc = Afzender, Z tot A

mail-list-order-sent-newest = Verzonden, nieuwste eerst

mail-list-order-sent-oldest = Verzonden, oudste eerst

mail-list-order-size = Grootte, grootste eerst

mail-list-order-size-asc = Maat, kleinste eerst

mail-list-order-subject-desc = Onderwerp, Z tot A

mail-mark-answered = Markeer als beantwoord

mail-mark-as = Markeer als

mail-mark-finished = Markeer als voltooid

mail-mark-forwarded = Markeer als doorgestuurd

mail-mark-not-forwarded = Markeer als niet doorgestuurd

mail-mark-unanswered = Markeer als onbeantwoord

mail-mentions-me = Noemt mij

mail-mentions-me-help = Zoek naar berichten waarin een van uw e-mailadressen wordt vermeld

mail-move-search = Zoek in mappen

mail-order-by = Bestel per

mail-reader-folder-go = Opgeslagen in { $folder } · Ga naar map

mail-reply-draft-discard = Antwoord weggooien

mail-reply-draft-discard-detail = Dit antwoord in behandeling definitief verwijderen? Dit kan niet ongedaan worden gemaakt.

mail-reply-draft-discard-title = Dit antwoord verwijderen?

mail-status-from-match-domain = Vanaf { $identity }

mail-status-from-match-pattern = Vanaf { $identity }

mail-status-from-via = Antwoord van { $identity } omdat het overeenkwam met { $header }

mail-toast-follow-up-finished = Vervolg gemarkeerd als voltooid

mail-toast-marked-answered = { $count ->
        [one] { $count } bericht gemarkeerd als beantwoord
       *[other] { $count } berichten gemarkeerd als beantwoord
    }

mail-toast-marked-forwarded = { $count ->
        [one] { $count } bericht gemarkeerd als doorgestuurd
       *[other] { $count } berichten gemarkeerd als doorgestuurd
    }

mail-toast-marked-not-forwarded = { $count ->
        [one] { $count } bericht gemarkeerd als niet doorgestuurd
       *[other] { $count } berichten gemarkeerd als niet doorgestuurd
    }

mail-toast-marked-unanswered = { $count ->
        [one] { $count } bericht gemarkeerd als onbeantwoord
       *[other] { $count } berichten gemarkeerd als onbeantwoord
    }

mail-toast-move-failed-reason = Kon de berichten niet verplaatsen ({ $reason }). Er werd niets verplaatst.

mail-toast-update-refused-reason = { $count ->
        [one] De server weigerde de zet voor één bericht: { $reason }
       *[other] De server heeft de verhuizing geweigerd voor { $count } berichten: { $reason }
    }

reader-badge-hide = Verberg deze badge

reader-badge-locked = Ingesteld door uw beheerder

reader-badge-menu = Berichtbadge

reader-badge-notable = Alleen weergeven als het opmerkelijk is

reader-sanitized-always = Actieve inhoud zoals scripts blijft uitgeschakeld.

reader-sanitized-clean = Bericht HTML had geen wijzigingen nodig

reader-sanitized-clean-detail = Er is niets verwijderd of geblokkeerd uit de HTML van dit bericht.

reader-sanitized-counts = { $blocked } externe items geblokkeerd, { $trackers } trackingitems.

reader-sanitized-detail = Het bericht HTML werd opgeschoond voordat het werd weergegeven. Externe inhoud volgt uw privacy-instellingen.

reader-sanitized-hosts = Externe hosts

reader-sanitized-remote = Externe inhoud geblokkeerd

reader-sanitized-short = Gezuiverd HTML

reader-sanitized-short-blocked = HTML geblokkeerd

reader-sanitized-short-clean = HTML schoon

reader-sanitized-trackers = Artikelen volgen
