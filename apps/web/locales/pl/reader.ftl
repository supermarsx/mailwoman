# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Wyświetlanie wiadomości
reader-settings-help = Wybierz sposób wyświetlania wiadomości. Domyślne ustawienia administratora obowiązują, dopóki nie wybierzesz własnych. Treść wiadomości pozostaje odizolowana od aplikacji.
reader-colorMode = Kolory wiadomości
reader-colorMode-help = Automatyczny dostosowuje kolory wiadomości w ciemnych motywach. Oryginalny zachowuje kolory nadawcy; adaptacyjny używa aktywnego motywu; odwrócenie odwraca kolory, zachowując kolory obrazów.
reader-fontSize = Rozmiar czcionki do czytania (px)
reader-fontSize-help = Bazowy rozmiar czcionki tekstu wiadomości, od 12 do 24 pikseli. Formatowanie nadawcy nadal może określać rozmiary nagłówków.
reader-lineHeight = Interlinia do czytania
reader-lineHeight-help = Odstęp między wierszami tekstu wiadomości, od 1,2 do 2,2 rozmiaru czcionki.
reader-contentWidth = Szerokość czytania
reader-contentWidth-help = Ogranicz długość wierszy dla wygodnego czytania albo użyj całego okienka wiadomości.
reader-loading = Wygląd ładowania
reader-loading-help = Wybierz spokojne linie zastępcze, wskaźnik tekstowy albo brak wskaźnika wizualnego. Czytniki ekranu nadal otrzymują informację o ładowaniu.
reader-loadingDelayMs = Opóźnienie wskaźnika ładowania (ms)
reader-loadingDelayMs-help = Unikaj migania wskaźnika przy szybkich żądaniach. Opóźnia tylko wskaźnik, nigdy samą wiadomość.
reader-option-auto = Automatycznie wg motywu
reader-option-original = Oryginalne kolory wiadomości
reader-option-adaptive = Dostosuj do motywu
reader-option-invert = Odwróć kolory wiadomości
reader-option-comfortable = Wygodna
reader-option-wide = Szeroka
reader-option-full = Całe okienko
reader-option-skeleton = Linie zastępcze
reader-option-indicator = Wskaźnik tekstowy
reader-option-none = Brak wskaźnika wizualnego
reader-inherit = Użyj ustawienia domyślnego administratora
reader-actionStyle = Wstążka akcji wiadomości
reader-actionStyle-help = Kompaktowe ikony zostawiają więcej miejsca na wiadomość. Każda akcja zachowuje nazwę dla klawiatury i objaśnienie w podpowiedzi. Wybierz widoczne etykiety, jeśli wolisz tekst obok ikon.
reader-option-icons = Kompaktowe ikony
reader-option-labels = Ikony i etykiety
reader-zoom-group = Powiększenie wiadomości
reader-zoom-in = Powiększ
reader-zoom-in-help = Powiększa wiadomość. Zmienia się tylko rozmiar wiadomości.
reader-zoom-out = Pomniejsz
reader-zoom-out-help = Pomniejsza wiadomość. Zmienia się tylko rozmiar wiadomości.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Powiększenie wiadomości { $level }%. Przywróć 100%
reader-zoom-reset-help = Przywróć wiadomość do 100%
reader-zoom-reset-menu = Resetuj powiększenie ({ $level }%)
reader-zoom = Powiększenie wiadomości (%)
reader-zoom-help = Rozmiar, w jakim otwierają się wiadomości, od 50% do 200%. Skalowana jest tylko treść wiadomości, nigdy aplikacja, a drukowanie zawsze używa 100%. Przyciski powiększenia obok akcji wiadomości też to zmieniają.
reader-attachments-summary = { $count ->
    [one] 1 załącznik
    [few] { $count } załączniki
    [many] { $count } załączników
   *[other] { $count } załącznika
  } · { $size }
reader-attachments-show = Pokaż załączniki
reader-attachments-hide = Ukryj załączniki
reader-attachmentsDensity = Gęstość załączników
reader-attachmentsDensity-help = Kompaktowa pokazuje małe przyciski plików; wygodna pokazuje większe podglądy po rozwinięciu.
reader-option-compact = Kompaktowa
reader-attachmentsCollapsed = Domyślnie zwijaj załączniki
reader-attachmentsCollapsed-help = Rozpoczynaj każdą wiadomość od kompaktowego podsumowania załączników. Możesz rozwinąć daną wiadomość bez zmiany tej wartości domyślnej.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Tekst i kolory
reader-card-text-help = Rozmiar, odstępy, szerokość i kolory tekstu wiadomości.
reader-card-actions = Akcje i załączniki
reader-card-actions-help = Wstążka akcji wiadomości i sposób rozpoczynania załączników.
reader-card-loading = Wczytywanie
reader-card-loading-help = Co widzisz podczas otwierania wiadomości.
settings-mail-card-list = Lista wiadomości
settings-mail-card-list-help = Akcje w wierszach i zachowanie listy po wykonaniu akcji na wiadomości.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Pobierz { $name }

reader-attachments-download-all = Pobierz wszystko

reader-attachments-download-all-blocked = Polityka załączników blokuje każdy plik tutaj

reader-attachments-download-all-busy = Przygotowanie pliku ZIP

reader-attachments-download-all-label = Pobierz wszystkie załączniki jako plik ZIP

reader-attachments-download-all-none = Brak załączników do pobrania

reader-attachments-download-all-warned = { $count ->
        [one] Pominięto jeden plik wymagający potwierdzenia. Pobierz go samodzielnie.
        [few] Pominięto { $count } pliki wymagające potwierdzenia. Pobierz je jeden po drugim.
        [many] Pominięto { $count } plików wymagających potwierdzenia. Pobierz je jeden po drugim.
       *[other] Pominięto { $count } plików wymagających potwierdzenia. Pobierz je jeden po drugim.
    }

reader-attachments-download-blocked = { $name } nie można pobrać: { $reason }

reader-attachments-download-label = Pobierz { $name } ({ $size })

reader-attachments-download-off = Pobieranie jest wyłączone

reader-attachments-inline-group = { $count ->
        [one] { $count } obraz w tekście
        [few] { $count } obrazy wbudowane
        [many] { $count } obrazów wbudowanych
       *[other] Obrazy wbudowane w formacie { $count }
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } obraz w tekście
        [few] { $count } obrazy wbudowane
        [many] { $count } obrazów wbudowanych
       *[other] Obrazy wbudowane w formacie { $count }
    } · { $size }

reader-attachmentsBarClickToToggle = Kliknij pasek, aby rozwinąć lub zwinąć

reader-attachmentsBarClickToToggle-help = Kliknięcie w dowolnym miejscu paska załączników powoduje jego przełączenie. Gdy jest wyłączony, działa tylko przycisk strzałki.

reader-attachmentsBarDownloadAll = Pobierz wszystko, przycisk

reader-attachmentsBarDownloadAll-help = Pokaż przycisk na pasku, który pobiera załączniki w postaci jednego pliku ZIP.

reader-attachmentsBarShowDownloadButtons = Przyciski pobierania w załącznikach

reader-attachmentsBarShowDownloadButtons-help = Pokaż przycisk pobierania każdego załącznika po najechaniu kursorem lub zaznaczeniu, zawsze lub nigdy.

reader-attachmentsBarZipInlineImages = Dołącz obrazy wbudowane w ZIP

reader-attachmentsBarZipInlineImages-help = Obrazy pojawiające się w tekście wiadomości są pomijane w opcji Pobierz wszystko, chyba że ta opcja jest włączona.

reader-card-attachments-bar = Pasek załączników

reader-card-attachments-bar-help = Jak reaguje pasek nad załącznikami wiadomości i co oferuje. Linie oznaczone jako zablokowane są ustawiane przez administratora.

reader-option-always = Zawsze

reader-option-hover = Po najechaniu kursorem lub skupieniu

reader-option-never = Nigdy

reader-card-header = Nagłówek wiadomości
reader-autoHideHeader = Automatycznie ukrywaj nagłówki wiadomości
reader-autoHideHeader-help = Otwieraj każdą wiadomość ze zwiniętymi danymi nadawcy, odbiorców i metadanymi. Temat i działania pozostają widoczne. Użyj Pokaż nagłówek, aby zobaczyć szczegóły. Domyślnie wyłączone.
reader-header-show = Pokaż nagłówek
reader-header-hide = Ukryj nagłówek
