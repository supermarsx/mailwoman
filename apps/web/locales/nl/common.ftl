# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Annuleren
common-close = Sluiten
common-save = Opslaan
common-delete = Verwijderen
common-remove = Verwijderen
common-edit = Bewerken
common-add = Toevoegen
common-back = Terug
common-next = Volgende
common-done = Gereed
common-apply = Toepassen
common-confirm = Bevestigen
common-retry = Opnieuw proberen
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = Als dit blijft gebeuren, neem dan contact op met uw beheerder.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Deze app is niet ingesteld voor een server. Neem contact op met uw beheerder.
common-copy = Kopiëren
common-search = Zoeken
common-more = Meer
common-yes = Ja
common-no = Nee

# -- Generic states ----------------------------------------------------------
common-loading = Laden…
common-saving = Opslaan…
common-empty = Hier staat nog niets
common-error = Er is iets misgegaan
common-error-network = Kan de server niet bereiken. Controleer uw verbinding en probeer het opnieuw.
common-offline = U bent offline

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Verbinden…
common-conn-degraded = Realtime-updates gepauzeerd — opnieuw verbinden
common-conn-offline = U bent offline — wijzigingen worden gesynchroniseerd zodra u weer verbonden bent
common-conn-auth-expired = Uw sessie is verlopen — log opnieuw in
common-conn-reconnect = Opnieuw verbinden
common-conn-sign-in-again = Opnieuw inloggen
common-conn-reconnected = Weer verbonden

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Nieuw bericht
common-notify-no-subject = (geen onderwerp)

# -- Generic labels ----------------------------------------------------------
common-required = Verplicht
common-optional = Optioneel

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Mappen en apps openen
common-nav-close = Mappen en apps sluiten

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Bijlagen
common-attach-search = Bijlagen zoeken
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Filteren op type
common-attach-loading = Bijlagen laden…
common-attach-empty = Geen bijlagen gevonden.
common-attach-cat-all = alle
common-attach-cat-image = afbeelding
common-attach-cat-pdf = pdf
common-attach-cat-text = tekst
common-attach-cat-audio = audio
common-attach-cat-video = video
common-attach-cat-other = overig

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = Wachtwoord tonen
common-password-hide = Wachtwoord verbergen

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = Mail
common-module-calendar = Agenda
common-module-tasks = Taken
common-module-notes = Notities
common-module-contacts = Contacten
common-module-keys = Sleutels

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = Viewer laden…
common-viewer-loading-audio = Audio laden…
common-viewer-loading-image = Afbeelding laden…
common-viewer-loading-video = Video laden…
common-viewer-rendering-pdf = PDF weergeven…
common-viewer-pdf-failed = Deze PDF kan niet worden weergegeven.
common-viewer-no-preview = Geen voorbeeld beschikbaar voor dit bestandstype.
common-viewer-unknown-type = onbekend type
common-viewer-download = Downloaden
common-viewer-attachments = Bijlagen

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Opnieuw proberen…
common-waiting-online = Wachten op een verbinding…
common-failure-offline = U bent offline. Maak opnieuw verbinding en probeer het nogmaals.
common-failure-network = De server is niet bereikbaar.
common-failure-server = De server reageerde niet op tijd.
common-failure-rate = Te veel verzoeken in korte tijd. Wacht even en probeer het opnieuw.
common-failure-conflict = Dit is tegelijkertijd elders gewijzigd. Laad opnieuw en probeer het nogmaals.
common-failure-account = Het ingelogde account is gewijzigd. Laad de pagina opnieuw.
common-failure-linked = Dit is niet beschikbaar zolang een gekoppelde mailbox geopend is.
common-failure-refused = De server heeft de wijziging geweigerd.
common-new-version = Er is een nieuwe versie van Mailwoman beschikbaar. Als u dit sluit, krijgt u over een uur opnieuw een herinnering.
common-new-version-reload = Nu opnieuw laden
common-toast-close = Melding sluiten
common-toast-seconds = { $seconds } s
common-attach-refresh = Bijlagen vernieuwen
common-attach-open-message = Het bericht met deze bijlage openen
common-attach-page-filter-note = Filters gelden voor deze pagina. Gebruik Volgende om oudere berichten met bijlagen te bekijken.
common-attach-pages = Bijlagenpagina's
common-attach-previous = Vorige
common-attach-next = Volgende
common-attach-page = Pagina { $page }
common-attach-column-name = Bestand
common-attach-column-type = Type
common-attach-column-size = Grootte
common-attach-column-sender = Afzender
common-attach-column-subject = Onderwerp
common-attach-column-date = Ontvangen
common-pdf-preview = PDF-voorbeeld
common-pdf-link-title = Link uit deze PDF openen?
common-pdf-link-help = Hiermee opent u de bestemming in een nieuw tabblad buiten Mailwoman.
common-pdf-link-open = Link openen
common-open-event = Afspraak openen
common-message-preview-too-large = Voorbeelden van berichtbijlagen zijn beperkt tot 4 MiB. U kunt de originele bijlage nog steeds downloaden.
common-message-preview-failed = Van deze berichtbijlage kon geen voorbeeld worden gemaakt. Het bestand wordt mogelijk niet ondersteund, is beschadigd of de voorbeeldservice is niet beschikbaar. U kunt het origineel nog steeds downloaden.
common-message-preview-attachments = Bestanden in dit bericht
common-message-preview-download = Bestand downloaden
common-message-preview-size-bytes = { $count } bytes
common-attach-cat-message = Berichten / Outlook-bijlagen
common-error-retry-help = Dit onderdeel is tijdelijk niet beschikbaar. Probeer het opnieuw om het te laden.
common-error-network-help = Controleer uw verbinding en probeer het opnieuw. De server is mogelijk tijdelijk niet beschikbaar.
common-error-page = Deze pagina kon niet worden geladen
common-error-page-help = Een paginabron kon niet worden gedownload. Probeer het opnieuw. Blijft het probleem bestaan, laad dan de applicatie opnieuw om de nieuwste versie te krijgen.
common-reload-app = Applicatie opnieuw laden
common-new-version-reloading = Er is een nieuwe versie beschikbaar, opnieuw laden…
common-attach-account-changed = Het account is gewijzigd tijdens het laden van bijlagen. Open Bijlagen opnieuw voor het huidige account.
common-attach-search-account = Hele account doorzoeken
common-attach-scanning = Zoeken… { $pages } berichtpagina's gecontroleerd · { $count } bestanden gevonden
common-attach-scan-cancelled = Zoekopdracht geannuleerd · { $count } bestanden gevonden in { $pages } pagina's. De resultaten zijn onvolledig.
common-attach-scan-incomplete = Zoekopdracht onderbroken · { $count } bestanden gevonden. Probeer opnieuw om het hele account te doorzoeken.
common-attach-scan-limited = De eerste 1.000 overeenkomende bestanden worden getoond. Verfijn uw filters om specifiekere bestanden te vinden.
common-attach-scan-complete = Zoekopdracht voltooid · { $count } bestanden gevonden in { $pages } berichtpagina's
common-attach-browse-pages = Pagina's doorbladeren
common-attach-cancel-search = Zoekopdracht annuleren
multiselect-selected = Geselecteerd: { $label }
multiselect-remove = { $label } verwijderen
multiselect-no-match = Geen overeenkomende opties
common-notification-permission-title = Meldingen toestaan?
common-notification-permission-help = Ontvang meldingen over nieuwe berichten op dit apparaat. U kunt dit wijzigen in Instellingen.
common-notification-permission-allow = Meldingen toestaan
common-notification-permission-later = Niet nu
common-notification-permission-error = Meldingen konden niet worden ingeschakeld. Controleer de machtigingen en meldingsinstellingen en probeer het opnieuw.
common-notifications-enabled = Meldingen ingeschakeld op dit apparaat.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [one] 1 geselecteerd
   *[other] { $count } geselecteerd
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = De server is bezet. Wacht een paar seconden en probeer het opnieuw.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = Geen overeenkomsten

combobox-toggle = Opties weergeven voor { $label }

common-attach-cat-archive = Archief

common-attach-cat-document = Documenten

common-attach-cat-spreadsheet = Spreadsheets

common-conn-reconnecting = Opnieuw verbinding maken…

common-viewer-pdf-details = Technische details

common-viewer-pdf-details-help = Voeg deze regels toe wanneer u het probleem rapporteert. Ze bevatten geen documentinhoud.

timezone-group-other = Anders

timezone-use-browser = Gebruik browsertijdzone ({ $zone })
