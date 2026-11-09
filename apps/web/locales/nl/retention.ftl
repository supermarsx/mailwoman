# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Bewaartermijnen
retention-intro = Regels die uw eigen mail na een aantal dagen verplaatsen of verwijderen. Een regel kan kijken naar een map, een label, een afzender, een grootte, of het bericht gelezen is of een bijlage heeft. Gemarkeerde mail en mail met het label keep blijft met rust gelaten, tenzij u dat wijzigt. Verwijderde mail gaat eerst naar de prullenbak en wordt na de ongedaanmakingstermijn definitief gewist.
retention-disabled = Bewaarregels zijn uitgeschakeld op deze server. Uw bestaande regels worden niet uitgevoerd.
retention-never = nooit
retention-error = De bewaarinstellingen konden niet worden geladen of opgeslagen. Probeer het opnieuw.
retention-last-run = Laatst uitgevoerd: { $when }
retention-next-run = volgende uitvoering over ongeveer { $days } dag(en)
retention-next-run-unknown = volgende uitvoering nog niet gepland
retention-enabled = Mijn bewaarregels uitvoeren
retention-trash-first = Verwijderde mail eerst naar de prullenbak verplaatsen
retention-trash-forced = Deze server verplaatst mail altijd eerst naar de prullenbak en wist die na de ongedaanmakingstermijn definitief.
retention-undo = Ongedaanmakingstermijn in dagen
retention-undo-help = Mail die een regel naar de prullenbak heeft verplaatst, blijft daar zo lang staan voordat die definitief wordt gewist. 0 tot { $max } dagen.
retention-pause = Alle regels 30 dagen pauzeren
retention-paused-until = Gepauzeerd tot { $when }
retention-resume = Hervatten
retention-policy = Wat deze server toestaat
retention-limits-both = Een regel die mail verwijdert wacht minimaal { $min } dagen en maximaal { $max } dagen.
retention-limits-min = Een regel die mail verwijdert wacht minimaal { $min } dagen.
retention-max-rules = U kunt maximaal { $count } regels bewaren.
retention-permanent-allowed = Een regel mag mail definitief verwijderen als u de stap via de prullenbak uitschakelt.
retention-permanent-denied = Regels kunnen mail niet definitief verwijderen. Mail gaat naar de prullenbak en wordt na { $days } dagen gewist.
retention-protected = Mappen die regels nooit aanraken: { $folders }.
retention-mandatory = Ingesteld door de beheerder: mail in { $folder } die ouder is dan { $days } dagen wordt verwijderd.
retention-rules = Uw regels
retention-add = Nieuwe regel
retention-no-rules = U hebt nog geen regels. Neem hieronder een sjabloon over of voeg er een toe.
retention-rule-paused = gepauzeerd
retention-rule-on = Aan
retention-rule-enabled = Regel { $name } staat aan
retention-edit-short = Bewerken
retention-delete = Verwijderen
retention-new = Nieuwe regel
retention-edit = Regel bewerken
retention-name = Naam
retention-action = Wat te doen
retention-action-keep = Voor altijd bewaren
retention-action-delete = Verwijderen
retention-action-trash = Naar de prullenbak verplaatsen
retention-action-archive = Naar het archief verplaatsen
retention-action-move = Naar een map verplaatsen
retention-action-markRead = Als gelezen markeren
retention-days = Na dagen
retention-target = Map
retention-choose-folder = Kies een map
retention-scope = Welke mail
retention-folder = Map
retention-any-folder = Elke map
retention-subfolders = Submappen opnemen
retention-labels = Labels (gescheiden door komma's)
retention-senders = Afzenderadressen (gescheiden door komma's)
retention-domains = Afzenderdomeinen (gescheiden door komma's)
retention-min-size = Kleinste grootte in KB
retention-max-size = Grootste grootte in KB
retention-read = Leesstatus
retention-read-any = Gelezen of ongelezen
retention-read-read = Alleen gelezen
retention-read-unread = Alleen ongelezen
retention-attachment = Bijlagen
retention-attachment-any = Met of zonder
retention-attachment-with = Met bijlage
retention-attachment-without = Zonder bijlage
retention-exclude = Met rust laten
retention-exclude-flagged = Gemarkeerde mail (met ster)
retention-exclude-unread = Ongelezen mail
retention-exclude-tags = Mail met deze labels (gescheiden door komma's)
retention-exclude-note = Mail onder een juridische bewaarplicht wordt nooit aangeraakt, wat hier ook is ingesteld.
retention-role-inbox = Postvak IN
retention-role-sent = Verzonden
retention-role-archive = Archief
retention-role-junk = Ongewenst
retention-role-trash = Prullenbak
retention-role-drafts = Concepten
retention-btn-preview = Voorbeeld
retention-save = Opslaan
retention-cancel = Annuleren
retention-preview-none = Er zou geen bericht veranderen.
retention-preview-count = Dit zou { $count } bericht(en) wijzigen.
retention-preview-truncated = Voor dit voorbeeld is slechts een deel van de mailbox gelezen.
retention-preview-sample = Voorbeelden
retention-no-subject = (geen onderwerp)
retention-do-trash = naar de prullenbak
retention-do-move = verplaatst
retention-do-delete = verwijderd
retention-do-markRead = als gelezen gemarkeerd
retention-skipped-legalHold = Uw account valt onder een juridische bewaarplicht. Er wordt niets gewijzigd.
retention-skipped-restricted = De verwerking van uw account is beperkt. Er wordt niets gewijzigd.
retention-skipped-noRules = U hebt geen actieve regels en de beheerder heeft er geen ingesteld.
retention-error-name = Geef de regel een naam.
retention-error-days = Dagen moet een geheel getal van 0 tot 3650 zijn.
retention-error-min = Een regel die mail verwijdert moet minimaal { $days } dagen wachten.
retention-error-max = Een regel die mail verwijdert kan maximaal { $days } dagen wachten.
retention-error-target = Kies de map waarnaar de mail moet worden verplaatst.
retention-summary-keep = voor altijd bewaren
retention-summary-delete = na { $days } dagen naar de prullenbak, daarna definitief gewist
retention-summary-delete-permanent = na { $days } dagen definitief verwijderd
retention-summary-trash = na { $days } dagen naar de prullenbak
retention-summary-archive = na { $days } dagen naar het archief
retention-summary-move = na { $days } dagen naar een map
retention-summary-markRead = na { $days } dagen als gelezen gemarkeerd
retention-templates = Sjablonen
retention-templates-help = Een sjabloon voegt een kant-en-klare regel toe. Controleer de labels en dagen voordat u erop vertrouwt.
retention-adopt = Deze regel toevoegen
retention-run-heading = Nu uitvoeren
retention-preview-all = Voorbeeld van alle regels
retention-btn-run = Mijn regels nu uitvoeren
retention-run-result = { $count } bericht(en) gewijzigd.
retention-history = Recente uitvoeringen
retention-history-when = Wanneer
retention-history-matched = Overeenkomsten
retention-history-changed = Gewijzigd
retention-history-errors = Fouten
retention-context-action = Bewaartermijn…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Kalenders

pim-cal-days = Dagen nadat een evenement is geëindigd

pim-cal-enabled = Verwijder beëindigde evenementen

pim-cal-help = Verwijder evenementen in de agenda's die u kiest een bepaald aantal dagen nadat ze zijn geëindigd. Terugkerende evenementen worden nooit verwijderd.

pim-cal-title = Agenda-evenementen

pim-days-range = Kies tussen { $min } en { $max } dagen.

pim-held-title = Verwijderde notities die u kunt herstellen

pim-held-until = Bewaard tot { $when }

pim-limits = Tussen { $min } en { $max } dagen.

pim-notes-days = Dagen sinds een notitie voor het laatst is gewijzigd

pim-notes-enabled = Verwijder onaangeroerde notities

pim-notes-help = Verwijder notities die u een bepaald aantal dagen niet hebt gewijzigd. Vastgezette notities blijven bewaard. Een verwijderde notitie kan gedurende { $days } dagen worden hersteld.

pim-notes-title = Opmerkingen

pim-off = Uitgezet door uw beheerder.

pim-preview = Voorbeeld

pim-restore = Herstellen

pim-result = { $events } gebeurtenis(sen) en { $notes } notitie(s).

pim-run = Ren nu

pim-run-title = Ren nu

pim-save = Opslaan

pim-skipped = { $count } keuze(s) zijn overgeslagen omdat uw beheerder deze heeft uitgeschakeld.

pim-untitled = Naamloze notitie

retention-action-stripAttachments = Verwijder grote bijlagen

retention-do-stripAttachments = bijlagen verwijderd

retention-error-strip-disabled = Het verwijderen van bijlagen is uitgeschakeld door uw beheerder.

retention-error-strip-min = Een regel die bijlagen verwijdert, moet minimaal { $days } dagen wachten.

retention-fd-action = Wat gebeurt er met e-mail in { $folder }

retention-fd-days = Dagen voordat { $folder } wordt verwerkt

retention-fd-help = Stel in hoe lang e-mail in een map blijft staan voordat deze wordt verplaatst of verwijderd. Een map die is ingesteld op "Overnemen" heeft geen eigen standaard; de bovenstaande regels en de beheerderslimieten zijn nog steeds van toepassing. Submappen volgen hun map.

retention-fd-inherit = Erven

retention-fd-mandatory = De beheerder bewaart deze map { $days } dagen

retention-fd-protected = Beschermd door de beheerder

retention-fd-save = Bewaar mapstandaarden

retention-fd-title = Standaardwaarden voor mappen

retention-rule-strip-off = Uitgezet door uw beheerder

retention-strip-allowed = Een regel kan bijlagen van minimaal { $kb } KB verwijderen uit e-mails die minimaal { $days } dagen oud zijn. Een korte notitie vervangt elke bijlage.

retention-strip-denied = Het verwijderen van bijlagen is uitgeschakeld door uw beheerder.

retention-strip-done = Bijlagen verwijderd uit { $count } bericht(en), waardoor { $size } vrijkomt.

retention-strip-inline-kept = Afbeeldingen die in het bericht worden weergegeven, blijven behouden.

retention-strip-inline-removed = Afbeeldingen die in het bericht worden weergegeven, worden ook verwijderd.

retention-strip-limits = Verwijdert bijlagen van minimaal { $kb } KB uit e-mails die minimaal { $days } dagen oud zijn. Het origineel blijft { $undo } dagen in de prullenbak.

retention-strip-preview = Bij { $count } bericht(en) zouden de bijlagen verloren gaan, waardoor { $size } vrijkomt.

retention-strip-preview-estimate = Bij ongeveer { $count } bericht(en) zouden de bijlagen verloren gaan, waardoor er { $size } vrijkomt.

retention-strip-skipped = { $count } regel(s) die bijlagen verwijderen zijn overgeslagen omdat uw beheerder dit heeft uitgeschakeld.

retention-summary-stripAttachments = na { $days } dagen worden grote bijlagen verwijderd
