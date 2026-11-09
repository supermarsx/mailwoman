# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Berichtweergave
reader-settings-help = Kies hoe berichten worden weergegeven. Standaardwaarden van de beheerder gelden totdat u een eigen keuze maakt. De berichtinhoud blijft gescheiden van de applicatie.
reader-colorMode = Berichtkleuren
reader-colorMode-help = Automatisch past berichtkleuren aan in donkere thema's. Origineel behoudt de kleuren van de afzender; aangepast gebruikt het actieve thema; omkeren keert kleuren om en behoudt de kleuren van afbeeldingen.
reader-fontSize = Lettergrootte bij lezen (px)
reader-fontSize-help = Basislettergrootte voor berichttekst, van 12 tot 24 pixels. De opmaak van de afzender kan nog steeds koptekstgroottes bepalen.
reader-lineHeight = Regelafstand bij lezen
reader-lineHeight-help = Ruimte tussen regels berichttekst, van 1,2 tot 2,2 keer de lettergrootte.
reader-contentWidth = Leesbreedte
reader-contentWidth-help = Beperk lange regels voor prettig lezen of gebruik het volledige berichtvenster.
reader-loading = Weergave tijdens laden
reader-loading-help = Kies rustige plaatshouderregels, een tekstindicator of geen visuele laadindicator. Schermlezers ontvangen de laadstatus nog steeds.
reader-loadingDelayMs = Vertraging van laadindicator (ms)
reader-loadingDelayMs-help = Voorkom dat een indicator knippert bij snelle verzoeken. Alleen de indicator wordt vertraagd, nooit het bericht zelf.
reader-option-auto = Automatisch met thema
reader-option-original = Originele berichtkleuren
reader-option-adaptive = Aan thema aanpassen
reader-option-invert = Berichtkleuren omkeren
reader-option-comfortable = Comfortabel
reader-option-wide = Breed
reader-option-full = Volledig venster
reader-option-skeleton = Plaatshouderregels
reader-option-indicator = Tekstindicator
reader-option-none = Geen visuele indicator
reader-inherit = Standaard van de beheerder gebruiken
reader-actionStyle = Actielint voor berichten
reader-actionStyle-help = Compacte pictogrammen laten meer ruimte voor het bericht. Elke actie behoudt zijn toetsenbordnaam en een toelichtende tooltip. Kies zichtbare labels als u liever tekst naast de pictogrammen ziet.
reader-option-icons = Compacte pictogrammen
reader-option-labels = Pictogrammen en labels
reader-zoom-group = Berichtzoom
reader-zoom-in = Inzoomen
reader-zoom-in-help = Maak het bericht groter. Alleen het bericht verandert van grootte.
reader-zoom-out = Uitzoomen
reader-zoom-out-help = Maak het bericht kleiner. Alleen het bericht verandert van grootte.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Berichtzoom { $level }%. Terugzetten op 100%
reader-zoom-reset-help = Het bericht terugzetten op 100%
reader-zoom-reset-menu = Zoom herstellen ({ $level }%)
reader-zoom = Berichtzoom (%)
reader-zoom-help = De grootte waarmee berichten worden geopend, van 50% tot 200%. Alleen de berichttekst wordt geschaald, nooit de app, en afdrukken gebruikt altijd 100%. De zoomknoppen naast de berichtacties wijzigen dit ook.
reader-attachments-summary = { $count ->
    [one] 1 bijlage
   *[other] { $count } bijlagen
 } · { $size }
reader-attachments-show = Bijlagen tonen
reader-attachments-hide = Bijlagen verbergen
reader-attachmentsDensity = Dichtheid van bijlagen
reader-attachmentsDensity-help = Compact toont kleine bestandsknoppen; comfortabel toont grotere voorbeelden wanneer uitgeklapt.
reader-option-compact = Compact
reader-attachmentsCollapsed = Bijlagen standaard samenvouwen
reader-attachmentsCollapsed-help = Begin elk bericht met een compacte samenvatting van de bijlagen. U kunt dat bericht uitklappen zonder deze standaard te wijzigen.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Tekst en kleuren
reader-card-text-help = Grootte, spatiëring, breedte en kleuren van berichttekst.
reader-card-actions = Acties en bijlagen
reader-card-actions-help = De actiebalk van het bericht en hoe bijlagen aanvankelijk worden weergegeven.
reader-card-loading = Laden
reader-card-loading-help = Wat u ziet terwijl een bericht wordt geopend.
settings-mail-card-list = Berichtenlijst
settings-mail-card-list-help = Rij-acties en wat de lijst doet wanneer u een bericht bewerkt.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Downloaden { $name }

reader-attachments-download-all = Alles downloaden

reader-attachments-download-all-blocked = Het bijlagebeleid blokkeert hier elk bestand

reader-attachments-download-all-busy = Het ZIP-bestand voorbereiden

reader-attachments-download-all-label = Download alle bijlagen als ZIP bestand

reader-attachments-download-all-none = Er zijn geen bijlagen om te downloaden

reader-attachments-download-all-warned = { $count ->
        [one] Eén bestand dat een bevestiging nodig heeft, is weggelaten. Download het zelf.
       *[other] Er zijn { $count } bestanden weggelaten die een bevestiging nodig hebben. Download ze één voor één.
    }

reader-attachments-download-blocked = { $name } kan niet worden gedownload: { $reason }

reader-attachments-download-label = Downloaden { $name } ({ $size })

reader-attachments-download-off = Downloads zijn uitgeschakeld

reader-attachments-inline-group = { $count ->
        [one] { $count } inline-afbeelding
       *[other] { $count } inline-afbeeldingen
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } inline-afbeelding
       *[other] { $count } inline-afbeeldingen
    } · { $size }

reader-attachmentsBarClickToToggle = Klik op de balk om uit te vouwen of samen te vouwen

reader-attachmentsBarClickToToggle-help = Als u ergens op de bijlagenbalk klikt, wordt deze omgeschakeld. Wanneer uitgeschakeld, doet alleen de pijlknop dat.

reader-attachmentsBarDownloadAll = Knop Alles downloaden

reader-attachmentsBarDownloadAll-help = Toon een knop op de balk die de bijlagen downloadt als één ZIP bestand.

reader-attachmentsBarShowDownloadButtons = Downloadknoppen op bijlagen

reader-attachmentsBarShowDownloadButtons-help = Toon de downloadknop van elke bijlage door aanwijzen of focussen, altijd of nooit.

reader-attachmentsBarZipInlineImages = Voeg inline afbeeldingen toe in de ZIP

reader-attachmentsBarZipInlineImages-help = Afbeeldingen die in de berichttekst verschijnen, worden buiten Download alles gelaten, tenzij dit is ingeschakeld.

reader-card-attachments-bar = Bijlagenbalk

reader-card-attachments-bar-help = Hoe de balk boven de bijlagen van een bericht reageert en wat deze biedt. Lijnen die als vergrendeld zijn gemarkeerd, worden door uw beheerder ingesteld.

reader-option-always = Altijd

reader-option-hover = Bij hover of focus

reader-option-never = Nooit
