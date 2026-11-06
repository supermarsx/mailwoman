# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Zawartość zdalna

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Nie udało się zaktualizować uprawnień dla zdalnych obrazów. { $reason }
remote-banner-blocked = Zdalne obrazy zablokowane
remote-banner-blocked-policy = Zdalne obrazy zablokowane przez Twoje ustawienia prywatności
remote-banner-shown = Obrazy wyświetlone
remote-banner-proxied = Obrazy wczytane przez serwer
remote-banner-direct = Obrazy wczytane z serwerów nadawcy
remote-banner-trackers = { $count ->
    [one] Usunięto { $count } element śledzący
    [few] Usunięto { $count } elementy śledzące
    [many] Usunięto { $count } elementów śledzących
   *[other] Usunięto { $count } elementu śledzącego
  }
remote-banner-trackers-limited = Usunięto co najmniej { $count } elementów śledzących
remote-banner-show = Pokaż obrazy
remote-banner-show-help = Wczytaj obrazy tej wiadomości przez serwer, aby nadawca nie widział Twojego adresu IP
remote-banner-more = Więcej opcji obrazów
remote-banner-always-sender = Zawsze pokazuj obrazy od: { $sender }
remote-banner-always-domain = Zawsze pokazuj obrazy z domeny { $domain }
remote-banner-always-all = Zawsze pokazuj obrazy we wszystkich wiadomościach
remote-banner-settings = Ustawienia prywatności
remote-banner-undo = Cofnij
remote-banner-undo-help-single = Ponownie zablokuj obrazy tej wiadomości
remote-banner-undo-help-per-sender = Przestań automatycznie pokazywać obrazy od: { $sender }
remote-banner-undo-help-per-domain = Przestań automatycznie pokazywać obrazy z domeny { $domain }
remote-banner-undo-help-all = Przestań automatycznie pokazywać obrazy we wszystkich wiadomościach
remote-banner-blocked-help = Obrazy przechowywane na innych serwerach mogą zdradzić nadawcy, kiedy i gdzie otwarto tę wiadomość. Pozostają zablokowane, dopóki nie zdecydujesz się ich pokazać.
remote-banner-help-block = Twoje ustawienia prywatności blokują wszystkie zdalne obrazy. Zmień to w ustawieniach prywatności.
remote-banner-help-ask = Obrazy są wczytywane przez ten serwer, który ukrywa Twój adres IP przed nadawcą.
remote-banner-help-proxy = Obrazy są wczytywane przez ten serwer, który ukrywa Twój adres IP przed nadawcą.
remote-banner-help-direct = Obrazy są wczytywane bezpośrednio z serwerów nadawcy, które widzą Twój adres IP.
remote-banner-hosts = Zablokowane hosty: { $hosts }
remote-banner-trackers-help = Obrazy śledzące są usuwane przed wyświetleniem wiadomości. Obrazy, które serwer rozpoznaje jako śledzące dopiero po ich pobraniu, są zliczane przez ograniczony czas.
remote-images-toast-loading = Wczytywanie obrazów…
remote-images-toast-done = Obrazy wczytane
remote-images-toast-failed = { $count ->
    [one] Nie udało się wczytać { $count } obrazu
    [few] Nie udało się wczytać { $count } obrazów
    [many] Nie udało się wczytać { $count } obrazów
   *[other] Nie udało się wczytać { $count } obrazu
  }
remote-images-toast-retry = Ponów
remote-images-toast-progress = Wczytywanie obrazów… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Wczytywanie obrazów… { $loaded }/{ $total } · Ponowienie { $attempt }/{ $max }
remote-images-toast-cancel = Anuluj
remote-images-toast-cancelled = Anulowano wczytywanie obrazów
remote-images-toast-load-again = Wczytaj ponownie
remote-banner-hosts-manage = Źródła obrazów…
image-hosts-title = Obrazy według źródła
image-hosts-help = Wybierz, co dzieje się z obrazami z każdego źródła. Opcja Nigdy nie pokazuj ma pierwszeństwo przed opcją Zawsze pokazuj i dotyczy każdej wiadomości.
image-hosts-none = Ta wiadomość nie ma zdalnych obrazów.
image-hosts-for = Obrazy z: { $host }
image-hosts-allow = Zawsze pokazuj
image-hosts-ask = Pytaj
image-hosts-block = Nigdy nie pokazuj
image-hosts-failed = Nie udało się zapisać ustawienia źródła obrazów.
image-tip-blocked = Obraz zablokowany. Wybierz Pokaż obrazy, aby go wczytać.
image-tip-loading = Wczytywanie obrazu…
image-tip-error = Obraz niedostępny
