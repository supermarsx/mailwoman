# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Map verplaatst: { $name }
folder-dnd-moved-many = { $count } mappen verplaatst
folder-dnd-undo = Ongedaan maken
folder-dnd-cancel = Annuleren
folder-dnd-handle = Sleep { $name } om de map te verplaatsen
folder-dnd-locked-hint = De mapvolgorde is vastgelegd door uw beheerder.
folder-dnd-forced-hint = Vastgezet door uw beheerder. Kan niet worden verplaatst of losgemaakt.
folder-dnd-disabled-hint = Slepen en neerzetten van mappen is uitgeschakeld.
folder-dnd-system-hint = Systeemmappen kunnen niet worden verplaatst.
folder-dnd-pinned-group = Vastgezet
folder-dnd-top-level = Naar het hoogste niveau verplaatsen
folder-dnd-problem-disabled = Mappen verplaatsen is uitgeschakeld.
folder-dnd-problem-locked = De mapvolgorde is vastgelegd door uw beheerder.
folder-dnd-problem-self = Een map kan niet in zichzelf worden verplaatst.
folder-dnd-problem-descendant = Een map kan niet in een van haar eigen submappen worden verplaatst.
folder-dnd-problem-system = Systeemmappen kunnen hier niet naartoe worden verplaatst.
folder-dnd-problem-no-children = Die map kan geen submappen bevatten.
folder-dnd-problem-nesting-off = Uw beheerder staat het nesten van mappen niet toe.
folder-dnd-problem-depth = Dat zou mappen te diep nesten.
folder-dnd-problem-forced = Deze map is vastgezet door uw beheerder.
folder-dnd-problem-unknown = Die map bestaat niet meer.
folder-dnd-problem-noop = De map staat daar al.
folder-dnd-problem-collision = Daar bestaat al een map met de naam { $name }. Hernoem die eerst.
folder-dnd-problem-server = De server heeft { $name } niet verplaatst. Er is niets gewijzigd.
folder-dnd-announce-up = { $name } omhoog verplaatst
folder-dnd-announce-down = { $name } omlaag verplaatst
folder-dnd-announce-indent = { $name } verplaatst naar de map erboven
folder-dnd-announce-outdent = { $name } een niveau omhoog verplaatst
folder-dnd-nest-title = { $name } hierheen verplaatsen?
folder-dnd-nest-title-many = { $count } mappen hierheen verplaatsen?
folder-dnd-nest-body = Kies of dit de map op de mailserver wijzigt of alleen hoe deze app haar toont.
folder-dnd-nest-legend = Hoe te verplaatsen
folder-dnd-nest-local = Alleen in deze app schikken
folder-dnd-nest-local-help = Alleen een weergave. De map blijft op de server en in andere mailapps staan waar ze staat.
folder-dnd-nest-server = Op de server verplaatsen
folder-dnd-nest-server-help = Wijzigt de map overal, in elke mailapp die dit account gebruikt.
folder-dnd-nest-remember = Mijn keuze onthouden
folder-dnd-nest-confirm = Verplaatsen
folder-dnd-arranged-only = Alleen in deze app geschikt
folder-dnd-arranged-only-help = De plek op de server is anders: { $path }
folder-dnd-apply-server = Op de server toepassen
folder-dnd-reset = Volgorde terugzetten naar standaard
folder-dnd-reset-done = Mapvolgorde teruggezet
folder-dnd-move-to = { $name } verplaatsen naar...
folder-dnd-layout-link = Maplay-out
folder-dnd-reorder-enable = Mappen herschikken door te slepen
folder-dnd-nest-mode = Wanneer ik een map nest
folder-dnd-nest-mode-ask = Mij vragen
folder-dnd-nest-mode-server = Op de server verplaatsen
folder-dnd-nest-mode-local = Alleen in deze app schikken
folder-dnd-new-placement = Nieuwe mappen verschijnen
folder-dnd-new-placement-end = Na hun mappen op hetzelfde niveau
folder-dnd-new-placement-alphabetical = In alfabetische volgorde
folder-dnd-handles = Sleepgrepen tonen
folder-dnd-handles-hover = Bij aanwijzen
folder-dnd-handles-always = Altijd
folder-dnd-arrange-title = Mappen schikken
folder-dnd-arrange-help = Sleep een map om haar te herschikken, zet haar neer op een andere map om haar te nesten, of gebruik Alt en de pijltoetsen op een map met focus.
folder-dnd-admin-locked = Ingesteld door uw beheerder
folder-dnd-arrange-menu = Verplaatsen naar…
folder-star-add = Map met ster markeren
folder-star-remove = Ster van map verwijderen
folder-star-starred = Met ster
folder-star-toggle = { $name } met ster markeren
folder-star-only = Alleen mappen met ster tonen
folder-star-only-help = Toont alleen mappen met ster en de mappen die ze bevatten.
folder-dnd-indent = { $name } naar de map erboven verplaatsen
folder-dnd-outdent = { $name } een niveau omhoog verplaatsen
folder-dnd-more-actions = Schikken
folder-dnd-selected-move = Geselecteerde verplaatsen naar…
folder-dnd-locked-row = Vergrendeld
folder-dnd-problem-pin = Vastgezette mappen blijven boven de andere. Maak ze eerst los om ze daarheen te verplaatsen.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Leeg…

folder-bar-label = Acties voor de geselecteerde mappen

folder-bar-mark-read = Markeer alles als gelezen

folder-bar-pin = Vastzetten

folder-bar-star = Ster

folder-bar-unpin = Losmaken

folder-bar-unstar = Ster verwijderen

folder-dnd-handle-hint = Sleep om deze map te verplaatsen

folder-pinned-group = Vastgezet

folder-pinned-unpin-zone = Kom hierheen om los te maken

settings-folders-bulk-delete-body = { $folders ->
        [one] { $folders } map en de { $messages } berichten daarin worden verwijderd. Dit kan niet ongedaan worden gemaakt.
       *[other] { $folders } mappen (met hun submappen) en de { $messages } berichten daarin worden verwijderd. Dit kan niet ongedaan worden gemaakt.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [one] Verwijder { $count } map
       *[other] Verwijder { $count } mappen
    }

settings-folders-bulk-delete-done = { $count ->
        [one] { $count } map verwijderd
       *[other] { $count } mappen verwijderd
    }

settings-folders-bulk-delete-failed = Er is niets verwijderd. Niet verwijderd: { $names }

settings-folders-bulk-delete-list = Mappen om te verwijderen

settings-folders-bulk-delete-none = Geen van de geselecteerde mappen kan worden verwijderd (systeemmappen en mappen die u niet mag verwijderen worden overgeslagen).

settings-folders-bulk-delete-others = Ook niet verwijderd: { $names }

settings-folders-bulk-delete-partial = { $done } mappen verwijderd. Niet verwijderd: { $names }

settings-folders-bulk-delete-partial-done = { $done } mappen verwijderd.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [one] Niet verwijderd omdat er een wettelijke bewaarplicht op het account rust: { $names }
       *[other] Niet verwijderd omdat het account onder een wettelijke bewaarplicht valt ({ $count } mappen): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [one] Niet verwijderd omdat de verwerking van het account beperkt is: { $names }
       *[other] Niet verwijderd omdat de verwerking van het account beperkt is ({ $count } mappen): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } geselecteerde mappen worden overgeslagen omdat ze niet kunnen worden verwijderd: { $names }

settings-folders-bulk-delete-subfolders = Verwijder ook de submappen van elke map

settings-folders-bulk-delete-subfolders-off = Alleen de geselecteerde mappen worden verwijderd. Een map die nog een submap buiten de selectie heeft, wordt behouden.

settings-folders-bulk-delete-title = { $count ->
        [one] { $count } map verwijderen?
       *[other] { $count } mappen verwijderen?
    }

settings-folders-bulk-empty-body = { $total } berichten in { $folders } mappen worden verwijderd: { $toTrash } verplaatsen naar de prullenbak en kunnen ongedaan worden gemaakt, { $permanent } worden voorgoed verwijderd en kunnen niet meer worden teruggehaald. Submappen worden niet aangeraakt.

settings-folders-bulk-empty-confirm = Maak de mappen leeg

settings-folders-bulk-empty-counting = De berichten in elke map tellen...

settings-folders-bulk-empty-done = { $count } mappen geleegd, { $messages } berichten

settings-folders-bulk-empty-fail-refused = de server weigerde een deel ervan; { $remaining } berichten blijven bestaan

settings-folders-bulk-empty-fail-stopped = gestopt; { $remaining } berichten blijven bestaan

settings-folders-bulk-empty-fail-uncertain = het resultaat van de laatste batch is onbekend; controleer de map

settings-folders-bulk-empty-list = Mappen om te legen

settings-folders-bulk-empty-partial = { $done } mappen geleegd; { $failed } is mislukt. Zie de lijst.

settings-folders-bulk-empty-permanent = { $count } berichten worden voorgoed verwijderd

settings-folders-bulk-empty-progress = { $done } van { $of } mappen klaar, { $messages } berichten verwerkt. Nu: { $name }

settings-folders-bulk-empty-skip-count-failed = kon niet worden geteld, overgeslagen

settings-folders-bulk-empty-skip-empty = al leeg, overgeslagen

settings-folders-bulk-empty-skip-permanent-denied = voorgoed zou worden verwijderd, wat de beheerder heeft uitgeschakeld en overgeslagen

settings-folders-bulk-empty-skip-search = een opgeslagen zoekopdracht, overgeslagen

settings-folders-bulk-empty-skip-unsupported = dit account kan het niet legen, overgeslagen

settings-folders-bulk-empty-title = { $count ->
        [one] { $count } map leegmaken?
       *[other] { $count } mappen leegmaken?
    }

settings-folders-bulk-empty-to-trash = { $count } berichten worden naar de prullenbak verplaatst

settings-folders-bulk-empty-undo-label = { $count } mappen geleegd ({ $messages } berichten)

settings-folders-delete-selected = Verwijderen…

settings-folders-select-user = Selecteer alle gebruikersmappen

settings-folders-subscribe-failed = De abonnementen konden niet worden gewijzigd.

settings-folders-subscribe-partial = { $done } mappen gewijzigd; { $failed } kon niet worden gewijzigd.

settings-folders-subscribe-selected = Abonneer je

settings-folders-subscribed = { $count ->
        [one] Geabonneerd op { $count } map
       *[other] Geabonneerd op { $count } mappen
    }

settings-folders-unsubscribe-selected = Afmelden

settings-folders-unsubscribed = { $count ->
        [one] Afgemeld voor { $count } map
       *[other] Afgemeld voor { $count } mappen
    }
