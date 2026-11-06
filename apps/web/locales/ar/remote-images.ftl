# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = المحتوى الخارجي

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = تعذّر تحديث أذونات الصور البعيدة. { $reason }
remote-banner-blocked = تم حظر الصور البعيدة
remote-banner-blocked-policy = تم حظر الصور البعيدة بواسطة إعدادات الخصوصية لديك
remote-banner-shown = تم عرض الصور
remote-banner-proxied = تم تحميل الصور عبر الخادم
remote-banner-direct = تم تحميل الصور من خوادم المرسل
remote-banner-trackers = { $count ->
    [zero] لم تُزل أي أداة تتبع
    [one] تمت إزالة أداة تتبع واحدة
    [two] تمت إزالة أداتي تتبع
    [few] تمت إزالة { $count } أدوات تتبع
    [many] تمت إزالة { $count } أداة تتبع
   *[other] تمت إزالة { $count } أداة تتبع
}
remote-banner-trackers-limited = تمت إزالة { $count } أداة تتبع على الأقل
remote-banner-show = عرض الصور
remote-banner-show-help = حمّل صور هذه الرسالة عبر الخادم حتى لا يرى المرسل عنوان IP الخاص بك
remote-banner-more = المزيد من خيارات الصور
remote-banner-always-sender = عرض الصور من { $sender } دائمًا
remote-banner-always-domain = عرض الصور من { $domain } دائمًا
remote-banner-always-all = عرض الصور في كل البريد دائمًا
remote-banner-settings = إعدادات الخصوصية
remote-banner-undo = تراجع
remote-banner-undo-help-single = حظر صور هذه الرسالة مرة أخرى
remote-banner-undo-help-per-sender = التوقف عن عرض الصور من { $sender } تلقائيًا
remote-banner-undo-help-per-domain = التوقف عن عرض الصور من { $domain } تلقائيًا
remote-banner-undo-help-all = التوقف عن عرض الصور في كل البريد تلقائيًا
remote-banner-blocked-help = يمكن للصور المخزّنة على خوادم أخرى أن تخبر المرسل بوقت ومكان فتحك لهذه الرسالة. تبقى محظورة إلى أن تختار عرضها.
remote-banner-help-block = تحظر إعدادات الخصوصية لديك جميع الصور البعيدة. غيّر هذا من إعدادات الخصوصية.
remote-banner-help-ask = تُحمَّل الصور عبر هذا الخادم، مما يخفي عنوان IP الخاص بك عن المرسل.
remote-banner-help-proxy = تُحمَّل الصور عبر هذا الخادم، مما يخفي عنوان IP الخاص بك عن المرسل.
remote-banner-help-direct = تُحمَّل الصور مباشرة من خوادم المرسل، التي يمكنها رؤية عنوان IP الخاص بك.
remote-banner-hosts = المضيفون المحظورون: { $hosts }
remote-banner-trackers-help = تُزال صور التتبع قبل عرض الرسالة. أما الصور التي يتعرف الخادم عليها كأدوات تتبع بعد تنزيلها فقط فتُحتسب لفترة محدودة.
remote-images-toast-loading = جارٍ تحميل الصور…
remote-images-toast-done = تم تحميل الصور
remote-images-toast-failed = { $count ->
    [zero] تعذّر تحميل أي صورة
    [one] تعذّر تحميل صورة واحدة
    [two] تعذّر تحميل صورتين
    [few] تعذّر تحميل { $count } صور
    [many] تعذّر تحميل { $count } صورة
   *[other] تعذّر تحميل { $count } صورة
}
remote-images-toast-retry = إعادة المحاولة
remote-images-toast-progress = جارٍ تحميل الصور… { $loaded }/{ $total }
remote-images-toast-retrying-progress = جارٍ تحميل الصور… { $loaded }/{ $total } · المحاولة { $attempt }/{ $max }
remote-images-toast-cancel = إلغاء
remote-images-toast-cancelled = تم إلغاء تحميل الصور
remote-images-toast-load-again = تحميل مرة أخرى
remote-banner-hosts-manage = مصادر الصور…
image-hosts-title = الصور حسب المصدر
image-hosts-help = اختر ما يحدث للصور من كل مصدر. يتفوق خيار "عدم العرض أبدًا" على "العرض دائمًا" ويسري على كل الرسائل.
image-hosts-none = لا تحتوي هذه الرسالة على صور بعيدة.
image-hosts-for = الصور من { $host }
image-hosts-allow = العرض دائمًا
image-hosts-ask = السؤال
image-hosts-block = عدم العرض أبدًا
image-hosts-failed = تعذّر حفظ إعداد مصدر الصور.
image-tip-blocked = تم حظر الصورة. اختر عرض الصور لتحميلها.
image-tip-loading = جارٍ تحميل الصورة…
image-tip-error = الصورة غير متاحة
