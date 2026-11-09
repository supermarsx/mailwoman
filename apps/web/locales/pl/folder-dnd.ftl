# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Przeniesiono folder: { $name }
folder-dnd-moved-many = Przeniesiono foldery: { $count }
folder-dnd-undo = Cofnij
folder-dnd-cancel = Anuluj
folder-dnd-handle = Przeciągnij { $name }, aby przenieść
folder-dnd-locked-hint = Kolejność folderów jest ustalona przez administratora.
folder-dnd-forced-hint = Przypięty przez administratora. Nie można go przenieść ani odpiąć.
folder-dnd-disabled-hint = Przeciąganie folderów jest wyłączone.
folder-dnd-system-hint = Folderów systemowych nie można przenosić.
folder-dnd-pinned-group = Przypięte
folder-dnd-top-level = Przenieś na najwyższy poziom
folder-dnd-problem-disabled = Przenoszenie folderów jest wyłączone.
folder-dnd-problem-locked = Kolejność folderów jest ustalona przez administratora.
folder-dnd-problem-self = Folderu nie można przenieść do niego samego.
folder-dnd-problem-descendant = Folderu nie można przenieść do jego własnego podfolderu.
folder-dnd-problem-system = Folderów systemowych nie można tu przenieść.
folder-dnd-problem-no-children = Ten folder nie może zawierać podfolderów.
folder-dnd-problem-nesting-off = Administrator nie zezwala na zagnieżdżanie folderów.
folder-dnd-problem-depth = Folderów byłoby zagnieżdżonych zbyt głęboko.
folder-dnd-problem-forced = Ten folder jest przypięty przez administratora.
folder-dnd-problem-unknown = Tego folderu już nie ma.
folder-dnd-problem-noop = Folder już tam jest.
folder-dnd-problem-collision = Folder o nazwie { $name } już tam istnieje. Najpierw zmień jego nazwę.
folder-dnd-problem-server = Serwer nie przeniósł folderu { $name }. Nic nie zostało zmienione.
folder-dnd-announce-up = { $name } przesunięto w górę
folder-dnd-announce-down = { $name } przesunięto w dół
folder-dnd-announce-indent = { $name } przeniesiono do folderu powyżej
folder-dnd-announce-outdent = { $name } przeniesiono o jeden poziom wyżej
folder-dnd-nest-title = Przenieść { $name } tutaj?
folder-dnd-nest-title-many = Przenieść tutaj foldery: { $count }?
folder-dnd-nest-body = Wybierz, czy zmiana dotyczy folderu na serwerze poczty, czy tylko sposobu jego wyświetlania w tej aplikacji.
folder-dnd-nest-legend = Sposób przeniesienia
folder-dnd-nest-local = Tylko ułóż w tej aplikacji
folder-dnd-nest-local-help = Tylko widok. Folder pozostaje na swoim miejscu na serwerze i w innych aplikacjach pocztowych.
folder-dnd-nest-server = Przenieś na serwerze
folder-dnd-nest-server-help = Zmienia folder wszędzie, w każdej aplikacji pocztowej używającej tego konta.
folder-dnd-nest-remember = Zapamiętaj wybór
folder-dnd-nest-confirm = Przenieś
folder-dnd-arranged-only = Ułożony tylko w tej aplikacji
folder-dnd-arranged-only-help = Jego miejsce na serwerze jest inne: { $path }
folder-dnd-apply-server = Zastosuj na serwerze
folder-dnd-reset = Przywróć domyślną kolejność
folder-dnd-reset-done = Przywrócono kolejność folderów
folder-dnd-move-to = Przenieś { $name } do...
folder-dnd-layout-link = Układ folderów
folder-dnd-reorder-enable = Zmieniaj układ folderów przez przeciąganie
folder-dnd-nest-mode = Przy zagnieżdżaniu folderu
folder-dnd-nest-mode-ask = Pytaj
folder-dnd-nest-mode-server = Przenieś na serwerze
folder-dnd-nest-mode-local = Tylko ułóż w tej aplikacji
folder-dnd-new-placement = Nowe foldery pojawiają się
folder-dnd-new-placement-end = Po folderach tego samego poziomu
folder-dnd-new-placement-alphabetical = W kolejności alfabetycznej
folder-dnd-handles = Pokaż uchwyty przeciągania
folder-dnd-handles-hover = Po najechaniu
folder-dnd-handles-always = Zawsze
folder-dnd-arrange-title = Ułóż foldery
folder-dnd-arrange-help = Przeciągnij folder, aby zmienić jego kolejność, upuść go na inny folder, aby go zagnieździć, albo użyj Alt i klawiszy strzałek na wybranym folderze.
folder-dnd-admin-locked = Ustawione przez administratora
folder-dnd-arrange-menu = Przenieś do…
folder-star-add = Oznacz folder gwiazdką
folder-star-remove = Usuń gwiazdkę z folderu
folder-star-starred = Z gwiazdką
folder-star-toggle = Oznacz gwiazdką: { $name }
folder-star-only = Pokaż tylko oznaczone gwiazdką
folder-star-only-help = Pokazuje tylko foldery oznaczone gwiazdką i foldery, które je zawierają.
folder-dnd-indent = Przenieś { $name } do folderu powyżej
folder-dnd-outdent = Przenieś { $name } o jeden poziom wyżej
folder-dnd-more-actions = Układ
folder-dnd-selected-move = Przenieś zaznaczone do…
folder-dnd-locked-row = Zablokowane
folder-dnd-problem-pin = Przypięte foldery pozostają nad pozostałymi. Najpierw go odepnij, aby go tam przenieść.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Puste…

folder-bar-label = Akcje dla wybranych folderów

folder-bar-mark-read = Zaznacz wszystkie jako przeczytane

folder-bar-pin = Przypnij

folder-bar-star = Gwiazda

folder-bar-unpin = Odepnij

folder-bar-unstar = Usuń gwiazdkę

folder-dnd-handle-hint = Przeciągnij, aby przenieść ten folder

folder-pinned-group = Przypięty

folder-pinned-unpin-zone = Upuść tutaj, aby odpiąć

settings-folders-bulk-delete-body = { $folders ->
        [one] { $folders } folder i zawarte w nim wiadomości { $messages } zostaną usunięte. Tego nie można cofnąć.
        [few] { $folders } foldery (wraz z podfolderami) i zawarte w nich wiadomości { $messages } zostaną usunięte. Tego nie można cofnąć.
        [many] { $folders } folderów (wraz z ich podfolderami) i zawarte w nich wiadomości { $messages } zostaną usunięte. Tego nie można cofnąć.
       *[other] { $folders } folderów (wraz z ich podfolderami) i zawarte w nich wiadomości { $messages } zostaną usunięte. Tego nie można cofnąć.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [one] Usuń { $count } folder
        [few] Usuń { $count } foldery
        [many] Usuń { $count } folderów
       *[other] Usuń { $count } folderu
    }

settings-folders-bulk-delete-done = { $count ->
        [one] Usunięto { $count } folder
        [few] Usunięto { $count } foldery
        [many] Usunięto { $count } folderów
       *[other] Usunięto { $count } folderu
    }

settings-folders-bulk-delete-failed = Nic nie zostało usunięte. Nie usunięto: { $names }

settings-folders-bulk-delete-list = Foldery do usunięcia

settings-folders-bulk-delete-none = Żadnego z wybranych folderów nie można usunąć (foldery systemowe i foldery, których nie można usunąć, są pomijane).

settings-folders-bulk-delete-others = Nie usunięto również: { $names }

settings-folders-bulk-delete-partial = Usunięto { $done } folderów. Nie usunięto: { $names }

settings-folders-bulk-delete-partial-done = Usunięto { $done } folderów.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [one] Nie usunięte, ponieważ konto jest objęte prawną blokadą: { $names }
        [few] Nie usunięte, ponieważ konto jest objęte prawną blokadą ({ $count } foldery): { $names }
        [many] Nie usunięte, ponieważ konto jest objęte prawną blokadą ({ $count } folderów): { $names }
       *[other] Nie usunięte, ponieważ konto jest objęte prawną blokadą ({ $count } folderu): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [one] Nie usunięte, ponieważ przetwarzanie konta jest ograniczone: { $names }
        [few] Nie usunięte, ponieważ przetwarzanie konta jest ograniczone ({ $count } foldery): { $names }
        [many] Nie usunięte, ponieważ przetwarzanie konta jest ograniczone ({ $count } folderów): { $names }
       *[other] Nie usunięte, ponieważ przetwarzanie konta jest ograniczone ({ $count } folderu): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } wybrane foldery są pomijane, ponieważ nie można ich usunąć: { $names }

settings-folders-bulk-delete-subfolders = Usuń także podfoldery każdego folderu

settings-folders-bulk-delete-subfolders-off = Usuwane są tylko wybrane foldery. Folder, który nadal ma podfolder poza zaznaczeniem, zostanie zachowany.

settings-folders-bulk-delete-title = { $count ->
        [one] Usunąć { $count } folder?
        [few] Usunąć { $count } foldery?
        [many] Usunąć { $count } folderów?
       *[other] Usunąć { $count } folderu?
    }

settings-folders-bulk-empty-body = { $total } wiadomości w folderach { $folders } zostaną usunięte: { $toTrash } zostaną przeniesione do Kosza i można je cofnąć, { $permanent } zostaną usunięte na stałe i nie będzie można ich przywrócić. Podfoldery nie są dotykane.

settings-folders-bulk-empty-confirm = Opróżnij foldery

settings-folders-bulk-empty-counting = Liczenie wiadomości w każdym folderze…

settings-folders-bulk-empty-done = Opróżniono { $count } foldery, { $messages } wiadomości

settings-folders-bulk-empty-fail-refused = serwer odmówił jego części; Pozostało { $remaining } wiadomości

settings-folders-bulk-empty-fail-stopped = zatrzymany; Pozostało { $remaining } wiadomości

settings-folders-bulk-empty-fail-uncertain = wynik ostatniej partii jest nieznany; sprawdź folder

settings-folders-bulk-empty-list = Foldery do opróżnienia

settings-folders-bulk-empty-partial = Opróżniono foldery { $done }; { $failed } nie powiodło się. Zobacz listę.

settings-folders-bulk-empty-permanent = { $count } wiadomości są usuwane na zawsze

settings-folders-bulk-empty-progress = Wykonano { $done } z { $of } folderów, przetworzono { $messages } wiadomości. Teraz: { $name }

settings-folders-bulk-empty-skip-count-failed = nie dało się policzyć, pominąć

settings-folders-bulk-empty-skip-empty = już pusty, pominięty

settings-folders-bulk-empty-skip-permanent-denied = zostałyby usunięte na dobre, co administrator wyłączył, pominął

settings-folders-bulk-empty-skip-search = zapisane wyszukiwanie, pominięte

settings-folders-bulk-empty-skip-unsupported = to konto nie może go opróżnić, pominięte

settings-folders-bulk-empty-title = { $count ->
        [one] Opróżnić { $count } folder?
        [few] Opróżnić { $count } foldery?
        [many] Opróżnić { $count } folderów?
       *[other] Opróżnić wybrane foldery ({ $count })?
    }

settings-folders-bulk-empty-to-trash = { $count } wiadomości są przenoszone do Kosza

settings-folders-bulk-empty-undo-label = Opróżniono foldery { $count } ({ $messages } wiadomości)

settings-folders-delete-selected = Usuń…

settings-folders-select-user = Wybierz wszystkie foldery użytkownika

settings-folders-subscribe-failed = Nie można zmienić subskrypcji.

settings-folders-subscribe-partial = Zmieniono foldery { $done }; Nie można zmienić { $failed }.

settings-folders-subscribe-selected = Subskrybuj

settings-folders-subscribed = { $count ->
        [one] Subskrybowano { $count } folder
        [few] Subskrybowano { $count } foldery
        [many] Subskrybowano { $count } folderów
       *[other] Subskrybowano { $count } folderu
    }

settings-folders-unsubscribe-selected = Zrezygnuj z subskrypcji

settings-folders-unsubscribed = { $count ->
        [one] Anulowano subskrypcję { $count } folderu
        [few] Anulowano subskrypcję { $count } folderów
        [many] Anulowano subskrypcję z { $count } folderów
       *[other] Anulowano subskrypcję { $count } folderów
    }
