# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: mailing-list. Source: ../en/mailing-list.ftl


# -- Machine translation (2026-11): added to match en ---------------------
mailing-list-title = Дії зі списком розсилки
mailing-list-unverified = Ці команди отримано від відправника повідомлення, їх не перевірено незалежно. Виберіть адресу призначення, щоб спершу перевірити її.
mailing-list-no-post = Цей список не приймає повідомлення електронною поштою.
mailing-list-help = Довідка
mailing-list-unsubscribe = Відписатися
mailing-list-post = Написати до списку
mailing-list-owner = Зв’язатися з власником списку
mailing-list-archive = Архів списку
mailing-list-bar = Список розсилки

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mailing-list-action-block = Заблокувати відправника

mailing-list-action-clean = Очистіть цього відправника

mailing-list-action-clean-help = Перегляньте, що робити з іншими повідомленнями від цього відправника, перш ніж щось застосовувати.

mailing-list-action-failed = Це не спрацювало. Спробуйте знову.

mailing-list-action-filter = Створити фільтр із цього списку…

mailing-list-action-folder = Перемістити майбутні повідомлення до папки…

mailing-list-action-keep = Завжди тримати

mailing-list-action-mute = Вимкнути цей список

mailing-list-action-spam = Повідомити про спам

mailing-list-action-unkeep = Перестаньте зберігати

mailing-list-action-unsubscribe-clean = Відпишіться і приберіть

mailing-list-address = Надіслано з

mailing-list-badge-named = Список розсилки · { $name }

mailing-list-badge-tooltip = Список розсилки: { $name } · Керувати підпискою

mailing-list-badge-tooltip-ignored = Список розсилки: { $name } · Скасовано підписку на { $date }, але пошта все ще надходить

mailing-list-badge-tooltip-unsubscribed = Список розсилки: { $name } · Скасовано підписку на { $date }

mailing-list-badge-unsubscribed = Скасував підписку

mailing-list-confirm = Надіслати запит на скасування підписки на { $host }? Mailwoman надсилає його за вас, тому хост списку не бачить вашу адресу IP.

mailing-list-confirm-cancel = Скасувати

mailing-list-confirm-send = Надіслати запит

mailing-list-confirm-title = Підтвердити відмову від підписки

mailing-list-copied = Скопійовано

mailing-list-copy = Копія

mailing-list-copy-failed = Не вдалося скопіювати

mailing-list-done = Запит на скасування підписки надіслано на номер { $host }. Пошта може припинитися через кілька днів.

mailing-list-failed = Запит на скасування підписки не виконано ({ $reason }). Замість цього можна використовувати сторінку самого списку.

mailing-list-forget = Забути

mailing-list-id = Ідентифікатор списку

mailing-list-keep-help = Очищення в цьому вікні пропускає відправника, якого ви зберегли.

mailing-list-keep-not-saved = Лише на цій сторінці: вибір не вдалося зберегти у ваших налаштуваннях.

mailing-list-kept-note = Ви зберігаєте цього відправника, тому очищення для нього тут вимкнено.

mailing-list-leaves-app = Це залишає Mailwoman: сторінку списку або чернетку, яку ви надсилаєте самі. Нічого не записується, доки ви не скажете, що закінчили.

mailing-list-links = Список посилань

mailing-list-list-count = { $count ->
        [one] { $count } повідомлення з цього списку у вашій пошті. Востаннє отримано { $date }.
        [few] { $count } повідомлення з цього списку знаходяться у вашій пошті. Востаннє отримано { $date }.
        [many] У вашій пошті { $count } повідомлень із цього списку. Востаннє отримано { $date }.
       *[other] { $count } повідомлення з цього списку знаходяться у вашій пошті. Востаннє отримано { $date }.
    }

mailing-list-list-count-none = У вашій пошті немає повідомлень із цього списку.

mailing-list-mark-done = Я відписався, запам'ятайте

mailing-list-method-link = Веб-сторінка на { $host }, відкрита в новій вкладці після перегляду

mailing-list-method-mail = Надішліть електронний лист на адресу { $address }, написаний як чернетка, яку ви переглядаєте

mailing-list-method-none = Це повідомлення не пропонує спосіб скасувати підписку.

mailing-list-method-one-click = Запит в один клік на { $host }, надісланий Mailwoman для вас

mailing-list-methods = Способи відписки

mailing-list-name = Список

mailing-list-open-new-tab = Відкрити в новій вкладці

mailing-list-remembered = Скасування підписки запам'ятовується для цього списку.

mailing-list-rule-hint = Додайте правило до заголовка List-Id ({ $id }) у правилах.

mailing-list-stats = { $count ->
        [one] У поштовій скриньці { $count } повідомлення від цього відправника. Востаннє отримано { $date }.
        [few] У поштовій скриньці { $count } повідомлення від цього відправника. Востаннє отримано { $date }.
        [many] У поштовій скриньці { $count } повідомлень від цього відправника. Востаннє отримано { $date }.
       *[other] У поштовій скриньці { $count } повідомлення від цього відправника. Востаннє отримано { $date }.
    }

mailing-list-stats-none = Жодні інші повідомлення від цього відправника не завантажуються в поштову скриньку.

mailing-list-status-ignored = Відтоді надійшла пошта, тому список може ігнорувати ваш запит.

mailing-list-status-subscribed = У цьому браузері для цього списку не зареєстровано скасування підписки.

mailing-list-status-unsubscribed = Скасував підписку на { $date }.

mailing-list-toast-blocked = Відправника заблоковано.

mailing-list-toast-kept = Цей відправник зберігається.

mailing-list-toast-muted = Цей список ігнорується.

mailing-list-toast-reported = Повідомлено як спам.

mailing-list-toast-unkept = Цей відправник більше не зберігається.

mailing-list-write = Напишіть
