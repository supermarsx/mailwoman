# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Віддалений вміст

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] Заблоковано { $count } віддалене зображення
    [few] Заблоковано { $count } віддалені зображення
    [many] Заблоковано { $count } віддалених зображень
   *[other] Заблоковано { $count } віддаленого зображення
}
remote-blocked-trackers = { $count ->
    [one] Заблоковано { $count } трекер
    [few] Заблоковано { $count } трекери
    [many] Заблоковано { $count } трекерів
   *[other] Заблоковано { $count } трекера
} серед { $blocked ->
    [one] { $blocked } віддаленого зображення
    [few] { $blocked } віддалених зображень
    [many] { $blocked } віддалених зображень
   *[other] { $blocked } віддаленого зображення
}
remote-blocked-hosts = Заблоковані хости

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Завантажити зображення
remote-grant-sender = Завжди завантажувати від { $sender }
remote-grant-domain = Завжди завантажувати з { $domain }
remote-grant-all = Завжди завантажувати всі віддалені зображення

remote-grant-done-single = Віддалені зображення завантажено для цього повідомлення.
remote-grant-done-per-sender = Віддалені зображення від цього відправника завантажуватимуться.
remote-grant-done-per-domain = Віддалені зображення з цього домену завантажуватимуться.
remote-grant-done-all = Віддалені зображення завантажуватимуться для всієї пошти.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Для цього повідомлення завантажуються віддалені зображення.
remote-revoke = Вимкнути
remote-revoke-done = Віддалені зображення вимкнено. Під час наступного відкриття вони знову блокуватимуться.
