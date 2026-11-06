# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Wissen en opschonen
compliance-card-erasure-coverage = Vervang de identificatoren van één persoon in de auditgeschiedenis en logboeken, en zie welke back-ups nog mensen bevatten die zijn gewist.
admin-erasure-title = Wissen en opschonen
admin-erasure-intro = Persoon opschonen vervangt het adres en de account-id's van een persoon in de auditgeschiedenis en in de geroteerde logbestanden door een stabiel pseudoniem met sleutel. De hashketen van de audit wordt opnieuw verzegeld zodat de verificatie nog slaagt. Elke wissing en elke opschoning laat ook een tombstone achter, zodat het terugzetten van een oudere back-up die mensen opnieuw wist.
admin-erasure-docs = Het register van wat per tabel wordt gewist, geëxporteerd en bewaard staat in docs/admin/erasure-coverage.md.
admin-erasure-scrub = Persoon opschonen
admin-erasure-scrub-note = Voer eerst het voorbeeld uit: dat telt de rijen en logregels die zouden veranderen en wijzigt niets.
admin-erasure-address = Adres
admin-erasure-preview = Voorbeeld
admin-erasure-run = Persoon opschonen
admin-erasure-dry-note = Alleen voorbeeld. Er is niets gewijzigd.
admin-erasure-done-note = Opgeschoond. De persoon verschijnt nu als { $pseudonym }.
admin-erasure-nothing = Geen enkele geschiedenis noemt deze persoon.
admin-erasure-col-table = Tabel of bestand
admin-erasure-col-rows = Rijen
admin-erasure-confirm-title = Deze persoon uit de geschiedenis opschonen?
admin-erasure-confirm-body = Auditvermeldingen en logregels die de persoon noemen, worden herschreven met het pseudoniem. Dit kan niet ongedaan worden gemaakt; het adres kan zonder de sleutel niet uit het pseudoniem worden hersteld.
admin-erasure-settings = Instellingen
admin-erasure-keys = Pseudoniemsleutels
admin-erasure-keys-note = Pseudoniemen worden gemaakt met de huidige sleutelgeneratie. Roteren voegt een generatie toe; oudere blijven bestaan totdat de bewaartermijn hierboven ze verwijdert.
admin-erasure-rotate = Sleutel roteren
admin-erasure-keys-none = Er is nog geen sleutel aangemaakt. De eerste opschoning maakt er een.
admin-erasure-col-gen = Generatie
admin-erasure-col-created = Aangemaakt
admin-erasure-col-state = Status
admin-erasure-key-current = Huidig
admin-erasure-key-retired = Buiten gebruik, bewaard
admin-erasure-key-destroy = Buiten gebruik, verwijderd op { $when }
admin-erasure-chain = Auditketen
admin-erasure-chain-note = Elke auditvermelding bevat een hash van de vorige. Een opschoning herschrijft vermeldingen, verzegelt de keten opnieuw en legt de oude en nieuwe kophash vast.
admin-erasure-seal = Nieuwe vermeldingen verzegelen
admin-erasure-chain-ok = De keten wordt geverifieerd: { $rows } vermeldingen verzegeld, { $unsealed } wachten op verzegeling.
admin-erasure-chain-bad = De keten wordt niet geverifieerd; eerste foute vermelding op positie { $seq }.
admin-erasure-col-time = Tijd
admin-erasure-col-range = Posities
admin-erasure-backups = Back-ups
admin-erasure-backups-note = Een back-up legt het tombstone-watermerk vast op het moment dat die wordt gemaakt. Het huidige watermerk is { $watermark }.
admin-erasure-col-label = Label
admin-erasure-col-location = Locatie
admin-erasure-col-watermark = Watermerk
admin-erasure-col-marker = Gewist sinds
admin-erasure-col-actions = Acties
admin-erasure-backups-none = Er zijn geen back-ups geregistreerd.
admin-erasure-contains-erased = Bevat gewiste personen ({ $count })
admin-erasure-no-marker = Geen
admin-erasure-forget = Vergeten
admin-erasure-register = Back-up registreren
