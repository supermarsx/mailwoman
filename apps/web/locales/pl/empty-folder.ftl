# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Opróżnij folder…
empty-folder-title = Opróżnić folder „{ $name }”?
empty-folder-body-trash = { $count ->
    [one] Wiadomość z folderu { $name } zostanie przeniesiona do Kosza, skąd można ją przywrócić.
    [few] Wszystkie wiadomości ({ $count }) z folderu { $name } zostaną przeniesione do Kosza, skąd można je przywrócić.
    [many] Wszystkie wiadomości ({ $count }) z folderu { $name } zostaną przeniesione do Kosza, skąd można je przywrócić.
   *[other] Wszystkie wiadomości ({ $count }) z folderu { $name } zostaną przeniesione do Kosza, skąd można je przywrócić.
  } Podfoldery i zawarte w nich wiadomości nie są zmieniane.
empty-folder-body-destroy-trash = { $count ->
    [one] Wiadomość w Koszu zostanie trwale usunięta. Tej operacji nie można cofnąć.
    [few] Wszystkie wiadomości ({ $count }) w Koszu zostaną trwale usunięte. Tej operacji nie można cofnąć.
    [many] Wszystkie wiadomości ({ $count }) w Koszu zostaną trwale usunięte. Tej operacji nie można cofnąć.
   *[other] Wszystkie wiadomości ({ $count }) w Koszu zostaną trwale usunięte. Tej operacji nie można cofnąć.
  } Podfoldery i zawarte w nich wiadomości nie są zmieniane.
empty-folder-body-destroy-junk = { $count ->
    [one] Wiadomość z folderu { $name } zostanie trwale usunięta. Tej operacji nie można cofnąć.
    [few] Wszystkie wiadomości ({ $count }) z folderu { $name } zostaną trwale usunięte. Tej operacji nie można cofnąć.
    [many] Wszystkie wiadomości ({ $count }) z folderu { $name } zostaną trwale usunięte. Tej operacji nie można cofnąć.
   *[other] Wszystkie wiadomości ({ $count }) z folderu { $name } zostaną trwale usunięte. Tej operacji nie można cofnąć.
  } Podfoldery i zawarte w nich wiadomości nie są zmieniane.
empty-folder-body-destroy-no-trash = To konto nie ma folderu Kosz, dlatego { $count ->
    [one] wiadomość z folderu { $name } zostanie trwale usunięta.
    [few] wszystkie wiadomości ({ $count }) z folderu { $name } zostaną trwale usunięte.
    [many] wszystkie wiadomości ({ $count }) z folderu { $name } zostaną trwale usunięte.
   *[other] wszystkie wiadomości ({ $count }) z folderu { $name } zostaną trwale usunięte.
  } Tej operacji nie można cofnąć. Podfoldery i zawarte w nich wiadomości nie są zmieniane.
empty-folder-confirm-trash = Przenieś do Kosza
empty-folder-confirm-destroy = Usuń trwale
empty-folder-cancel = Anuluj
empty-folder-already-empty = Folder „{ $name }” jest już pusty.
empty-folder-prepare-failed = Nie udało się zliczyć wiadomości w folderze „{ $name }”, więc nic nie zmieniono. Sprawdź połączenie i spróbuj ponownie.
empty-folder-syncing = Folder „{ $name }” jest jeszcze synchronizowany, więc nie można go jeszcze zliczyć. Nic nie zmieniono; spróbuj ponownie za chwilę.
empty-folder-running = Inny folder jest już opróżniany. Zatrzymaj to w jego powiadomieniu, zanim rozpoczniesz kolejne.
empty-folder-progress-trash = Przenoszenie wiadomości z folderu { $name } do Kosza: { $done } z { $total }.
empty-folder-progress-destroy = Usuwanie wiadomości z folderu { $name }: { $done } z { $total }.
empty-folder-stop = Zatrzymaj
empty-folder-stopping = Zatrzymywanie po bieżącej partii. Dotąd wykonano: { $done } z { $total }.
empty-folder-done-trash = { $done ->
    [one] Przeniesiono 1 wiadomość z folderu { $name } do Kosza.
    [few] Przeniesiono { $done } wiadomości z folderu { $name } do Kosza.
    [many] Przeniesiono { $done } wiadomości z folderu { $name } do Kosza.
   *[other] Przeniesiono { $done } wiadomości z folderu { $name } do Kosza.
}
empty-folder-done-destroy = { $done ->
    [one] Trwale usunięto 1 wiadomość z folderu { $name }.
    [few] Trwale usunięto { $done } wiadomości z folderu { $name }.
    [many] Trwale usunięto { $done } wiadomości z folderu { $name }.
   *[other] Trwale usunięto { $done } wiadomości z folderu { $name }.
}
empty-folder-stopped = Zatrzymano. Przetworzono wiadomości: { $done } z { $total }; w folderze { $name } pozostało: { $remaining }.
empty-folder-refused = Zatrzymano. Przetworzono wiadomości: { $done } z { $total }; serwer poczty odrzucił: { $refused }. W folderze { $name } pozostało: { $remaining }.
empty-folder-failed = Opróżnianie folderu { $name } zostało zatrzymane z błędem po { $done } wiadomościach. Ostatnia partia mogła zostać ukończona lub nie; sprawdź folder, zanim spróbujesz ponownie.
