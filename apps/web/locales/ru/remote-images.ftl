# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Внешнее содержимое

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Не удалось обновить разрешения для удалённых изображений. { $reason }
remote-banner-blocked = Удалённые изображения заблокированы
remote-banner-blocked-policy = Удалённые изображения заблокированы Вашими настройками конфиденциальности
remote-banner-shown = Изображения показаны
remote-banner-proxied = Изображения загружены через сервер
remote-banner-direct = Изображения загружены с серверов отправителя
remote-banner-trackers = { $count ->
    [one] Удалён { $count } трекер
    [few] Удалено { $count } трекера
    [many] Удалено { $count } трекеров
   *[other] Удалено { $count } трекера
}
remote-banner-trackers-limited = Удалено не менее { $count } трекеров
remote-banner-show = Показать изображения
remote-banner-show-help = Загрузить изображения этого сообщения через сервер, чтобы отправитель не видел Ваш IP-адрес
remote-banner-more = Другие параметры изображений
remote-banner-always-sender = Всегда показывать изображения от: { $sender }
remote-banner-always-domain = Всегда показывать изображения с домена { $domain }
remote-banner-always-all = Всегда показывать изображения во всей почте
remote-banner-settings = Настройки конфиденциальности
remote-banner-undo = Отменить
remote-banner-undo-help-single = Снова заблокировать изображения этого сообщения
remote-banner-undo-help-per-sender = Больше не показывать изображения от { $sender } автоматически
remote-banner-undo-help-per-domain = Больше не показывать изображения с домена { $domain } автоматически
remote-banner-undo-help-all = Больше не показывать изображения во всей почте автоматически
remote-banner-blocked-help = Изображения, хранящиеся на других серверах, могут сообщить отправителю, когда и где было открыто это сообщение. Они остаются заблокированными, пока Вы не решите их показать.
remote-banner-help-block = Ваши настройки конфиденциальности блокируют все удалённые изображения. Изменить это можно в настройках конфиденциальности.
remote-banner-help-ask = Изображения загружаются через этот сервер, который скрывает Ваш IP-адрес от отправителя.
remote-banner-help-proxy = Изображения загружаются через этот сервер, который скрывает Ваш IP-адрес от отправителя.
remote-banner-help-direct = Изображения загружаются прямо с серверов отправителя, которые видят Ваш IP-адрес.
remote-banner-hosts = Заблокированные узлы: { $hosts }
remote-banner-trackers-help = Отслеживающие изображения удаляются до показа сообщения. Изображения, которые сервер распознаёт как трекеры только после загрузки, учитываются ограниченное время.
remote-images-toast-loading = Загрузка изображений…
remote-images-toast-done = Изображения загружены
remote-images-toast-failed = { $count ->
    [one] Не удалось загрузить { $count } изображение
    [few] Не удалось загрузить { $count } изображения
    [many] Не удалось загрузить { $count } изображений
   *[other] Не удалось загрузить { $count } изображения
}
remote-images-toast-retry = Повторить
remote-images-toast-progress = Загрузка изображений… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Загрузка изображений… { $loaded }/{ $total } · Попытка { $attempt }/{ $max }
remote-images-toast-cancel = Отмена
remote-images-toast-cancelled = Загрузка изображений отменена
remote-images-toast-load-again = Загрузить снова
remote-banner-hosts-manage = Источники изображений…
image-hosts-title = Изображения по источникам
image-hosts-help = Выберите, что делать с изображениями из каждого источника. «Никогда не показывать» имеет приоритет над «Всегда показывать» и действует для каждого сообщения.
image-hosts-none = В этом сообщении нет удалённых изображений.
image-hosts-for = Изображения с: { $host }
image-hosts-allow = Всегда показывать
image-hosts-ask = Спрашивать
image-hosts-block = Никогда не показывать
image-hosts-failed = Не удалось сохранить настройку источника изображений.
image-tip-blocked = Изображение заблокировано. Выберите «Показать изображения», чтобы загрузить его.
image-tip-loading = Загрузка изображения…
image-tip-error = Изображение недоступно
