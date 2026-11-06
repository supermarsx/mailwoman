# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Віддалений вміст

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Не вдалося оновити дозволи для віддалених зображень. { $reason }
remote-banner-blocked = Віддалені зображення заблоковано
remote-banner-blocked-policy = Віддалені зображення заблоковано вашими налаштуваннями конфіденційності
remote-banner-shown = Зображення показано
remote-banner-proxied = Зображення завантажено через сервер
remote-banner-direct = Зображення завантажено із серверів відправника
remote-banner-trackers = { $count ->
    [one] Вилучено { $count } трекер
    [few] Вилучено { $count } трекери
    [many] Вилучено { $count } трекерів
   *[other] Вилучено { $count } трекера
}
remote-banner-trackers-limited = Вилучено щонайменше { $count } трекерів
remote-banner-show = Показати зображення
remote-banner-show-help = Завантажити зображення цього повідомлення через сервер, щоб відправник не бачив вашу IP-адресу
remote-banner-more = Інші параметри зображень
remote-banner-always-sender = Завжди показувати зображення від: { $sender }
remote-banner-always-domain = Завжди показувати зображення з домену { $domain }
remote-banner-always-all = Завжди показувати зображення в усій пошті
remote-banner-settings = Налаштування конфіденційності
remote-banner-undo = Скасувати
remote-banner-undo-help-single = Знову заблокувати зображення цього повідомлення
remote-banner-undo-help-per-sender = Більше не показувати зображення від { $sender } автоматично
remote-banner-undo-help-per-domain = Більше не показувати зображення з домену { $domain } автоматично
remote-banner-undo-help-all = Більше не показувати зображення в усій пошті автоматично
remote-banner-blocked-help = Зображення, що зберігаються на інших серверах, можуть повідомити відправнику, коли й де відкрито це повідомлення. Вони залишаються заблокованими, доки ви не вирішите їх показати.
remote-banner-help-block = Ваші налаштування конфіденційності блокують усі віддалені зображення. Змінити це можна в налаштуваннях конфіденційності.
remote-banner-help-ask = Зображення завантажуються через цей сервер, який приховує вашу IP-адресу від відправника.
remote-banner-help-proxy = Зображення завантажуються через цей сервер, який приховує вашу IP-адресу від відправника.
remote-banner-help-direct = Зображення завантажуються безпосередньо із серверів відправника, які бачать вашу IP-адресу.
remote-banner-hosts = Заблоковані вузли: { $hosts }
remote-banner-trackers-help = Зображення стеження вилучаються до показу повідомлення. Зображення, які сервер розпізнає як трекери лише після завантаження, враховуються обмежений час.
remote-images-toast-loading = Завантаження зображень…
remote-images-toast-done = Зображення завантажено
remote-images-toast-failed = { $count ->
    [one] Не вдалося завантажити { $count } зображення
    [few] Не вдалося завантажити { $count } зображення
    [many] Не вдалося завантажити { $count } зображень
   *[other] Не вдалося завантажити { $count } зображення
}
remote-images-toast-retry = Повторити
remote-images-toast-progress = Завантаження зображень… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Завантаження зображень… { $loaded }/{ $total } · Спроба { $attempt }/{ $max }
remote-images-toast-cancel = Скасувати
remote-images-toast-cancelled = Завантаження зображень скасовано
remote-images-toast-load-again = Завантажити знову
remote-banner-hosts-manage = Джерела зображень…
image-hosts-title = Зображення за джерелами
image-hosts-help = Виберіть, що робити із зображеннями з кожного джерела. «Ніколи не показувати» має пріоритет над «Завжди показувати» і діє для кожного повідомлення.
image-hosts-none = У цьому повідомленні немає віддалених зображень.
image-hosts-for = Зображення з: { $host }
image-hosts-allow = Завжди показувати
image-hosts-ask = Питати
image-hosts-block = Ніколи не показувати
image-hosts-failed = Не вдалося зберегти налаштування джерела зображень.
image-tip-blocked = Зображення заблоковано. Виберіть «Показати зображення», щоб завантажити його.
image-tip-loading = Завантаження зображення…
image-tip-error = Зображення недоступне
