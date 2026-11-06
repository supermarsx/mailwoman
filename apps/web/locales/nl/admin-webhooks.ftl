# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooks
admin-webhooks-intro = Een uitgaande webhook stuurt een ondertekende melding naar een URL wanneer er iets gebeurt in één account. Met een inkomende webhook kan een ander systeem in één account een actie van een mailregel uitvoeren.
admin-webhooks-empty = Nog geen webhooks.
admin-webhooks-unavailable = Deze server biedt geen webhookbeheer aan.
admin-webhooks-load-error = De webhooks konden niet worden geladen.
admin-webhooks-direction-outbound = Uitgaand
admin-webhooks-direction-inbound = Inkomend
admin-webhooks-col-direction = Richting
admin-webhooks-col-label = Naam
admin-webhooks-col-account = Account
admin-webhooks-col-target = Verstuurt naar of staat toe
admin-webhooks-col-created = Aangemaakt
admin-webhooks-no-label = (geen naam)
admin-webhooks-created-by = door { $who }
admin-webhooks-last-used = Laatst gebruikt { $when }
admin-webhooks-never-used = Nog niet gebruikt
admin-webhooks-delete = Verwijderen
admin-webhooks-delete-named = Webhook { $name } verwijderen
admin-webhooks-delete-title = De webhook { $name } verwijderen?
admin-webhooks-delete-body = Leveringen eraan stoppen, of aanroepen die met zijn secret zijn ondertekend worden geweigerd. Dit kan niet ongedaan worden gemaakt; een nieuwe webhook krijgt een nieuw secret.
admin-webhooks-deleted = Webhook verwijderd.
admin-webhooks-add = Een webhook toevoegen
admin-webhooks-add-direction = Richting
admin-webhooks-add-outbound-help = Mailwoman roept uw URL aan.
admin-webhooks-add-inbound-help = Uw systeem roept Mailwoman aan.
admin-webhooks-add-account = Account
admin-webhooks-add-account-help = Het account waarvan de gebeurtenissen worden verstuurd, of het enige account waarop een inkomende aanroep mag werken.
admin-webhooks-add-label = Naam
admin-webhooks-add-label-help = Waar de webhook voor dient, zodat anderen het kunnen zien. Optioneel.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Moet een openbaar internetadres bereiken. Privé-, loopback- en link-local-adressen worden geweigerd.
admin-webhooks-add-events = Gebeurtenissen
admin-webhooks-add-actions = Toegestane acties
admin-webhooks-add-submit = Webhook aanmaken
admin-webhooks-add-saving = Aanmaken…
admin-webhooks-add-check = Controleren
admin-webhooks-add-check-ok = De URL doorstaat de controle. Er is niets opgeslagen.
admin-webhooks-event-message-received = Nieuw bericht
admin-webhooks-event-message-flags = Bericht gelezen, gemarkeerd of van een label voorzien
admin-webhooks-event-message-moved = Bericht verplaatst
admin-webhooks-event-message-deleted = Bericht verwijderd
admin-webhooks-event-submission-sent = Bericht verzonden
admin-webhooks-event-submission-failed = Bericht kon niet worden verzonden
admin-webhooks-event-pim-changed = Agenda, contacten, taken of notities gewijzigd
admin-webhooks-action-run-rules = De mailregels van het account uitvoeren op een berichtbeschrijving
admin-webhooks-action-sieve = Eén regelactie op een bericht toepassen
admin-webhooks-secret-title = Secret voor { $name }
admin-webhooks-secret-once = Kopieer dit secret nu. Het wordt niet opnieuw getoond.
admin-webhooks-secret-outbound = Elke levering bevat X-Mailwoman-Signature: sha256= gevolgd door de HMAC-SHA256 van de aanvraagbody onder dit secret.
admin-webhooks-secret-inbound = Stuur X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (Unix-seconden), X-Mailwoman-Nonce (16 tot 128 letters, cijfers, - of _, nooit hergebruikt) en X-Mailwoman-Signature: sha256= gevolgd door de HMAC-SHA256 van timestamp.nonce.body onder dit secret. Aanroepen die meer dan vijf minuten afwijken, of een nonce hergebruiken, worden geweigerd.
admin-webhooks-secret-done = Ik heb het gekopieerd
admin-webhooks-error-stale = Iemand heeft de webhooks gewijzigd sinds u deze pagina opende. De lijst is opnieuw geladen; probeer het opnieuw.
admin-webhooks-error-account = Voer het account in.
admin-webhooks-error-url = Voer een URL in die begint met https:// of http://.
admin-webhooks-error-events = Kies minstens één gebeurtenis.
admin-webhooks-error-actions = Kies minstens één actie.
