# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Внешнее содержимое

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] Заблокировано { $count } внешнее изображение
    [few] Заблокировано { $count } внешних изображения
    [many] Заблокировано { $count } внешних изображений
   *[other] Заблокировано { $count } внешнего изображения
}
remote-blocked-trackers = { $count ->
    [one] Заблокирован { $count } трекер
    [few] Заблокировано { $count } трекера
    [many] Заблокировано { $count } трекеров
   *[other] Заблокировано { $count } трекера
} из { $blocked ->
    [one] { $blocked } внешнего изображения
    [few] { $blocked } внешних изображений
    [many] { $blocked } внешних изображений
   *[other] { $blocked } внешнего изображения
}
remote-blocked-hosts = Заблокированные узлы

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Загрузить изображения
remote-grant-sender = Всегда загружать от { $sender }
remote-grant-domain = Всегда загружать с { $domain }
remote-grant-all = Всегда загружать все внешние изображения

remote-grant-done-single = Внешние изображения загружены для этого сообщения.
remote-grant-done-per-sender = Внешние изображения от этого отправителя будут загружаться.
remote-grant-done-per-domain = Внешние изображения с этого домена будут загружаться.
remote-grant-done-all = Внешние изображения будут загружаться для всей почты.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Для этого сообщения загружаются внешние изображения.
remote-revoke = Отключить
remote-revoke-done = Загрузка внешних изображений отключена. При следующем открытии они снова будут заблокированы.
