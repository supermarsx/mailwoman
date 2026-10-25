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
common-contact-administrator = Als dit blijft gebeuren, neem dan contact op met je beheerder.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Deze app is niet ingesteld voor een server. Neem contact op met je beheerder.
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
common-error-network = Kan de server niet bereiken. Controleer je verbinding en probeer het opnieuw.
common-offline = Je bent offline

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Verbinden…
common-conn-degraded = Realtime-updates gepauzeerd — opnieuw verbinden
common-conn-offline = Je bent offline — wijzigingen worden gesynchroniseerd zodra je weer verbonden bent
common-conn-auth-expired = Je sessie is verlopen — log opnieuw in
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
