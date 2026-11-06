# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = موافق
common-cancel = إلغاء
common-close = إغلاق
common-save = حفظ
common-delete = حذف
common-remove = إزالة
common-edit = تعديل
common-add = إضافة
common-back = رجوع
common-next = التالي
common-done = تم
common-apply = تطبيق
common-confirm = تأكيد
common-retry = إعادة المحاولة
common-contact-administrator = إذا استمر حدوث ذلك، فتواصل مع المسؤول.
common-shell-not-configured = لم يُعَدّ هذا التطبيق للاتصال بخادم. تواصل مع المسؤول.
common-copy = نسخ
common-search = بحث
common-more = المزيد
common-yes = نعم
common-no = لا

# -- Generic states ----------------------------------------------------------
common-loading = جارٍ التحميل…
common-saving = جارٍ الحفظ…
common-empty = لا يوجد شيء هنا بعد
common-error = حدث خطأ ما
common-error-network = تعذّر الوصول إلى الخادم. تحقّق من اتصالك وحاول مرة أخرى.
common-offline = أنت غير متصل

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = جارٍ الاتصال…
common-conn-degraded = التحديثات الفورية متوقفة مؤقتًا — جارٍ إعادة الاتصال
common-conn-offline = أنت غير متصل — تُزامَن التغييرات عند إعادة الاتصال
common-conn-auth-expired = انتهت صلاحية جلستك — يُرجى تسجيل الدخول مرة أخرى
common-conn-reconnect = إعادة الاتصال
common-conn-sign-in-again = تسجيل الدخول مرة أخرى
common-conn-reconnected = أُعيد الاتصال

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = رسالة جديدة
common-notify-no-subject = (بلا موضوع)

# -- Generic labels ----------------------------------------------------------
common-required = مطلوب
common-optional = اختياري

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = فتح المجلدات والتطبيقات
common-nav-close = إغلاق المجلدات والتطبيقات

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = المرفقات
common-attach-search = البحث في المرفقات
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = التصفية حسب النوع
common-attach-loading = جارٍ تحميل المرفقات…
common-attach-empty = لا توجد مرفقات مطابقة.
common-attach-cat-all = الكل
common-attach-cat-image = صورة
common-attach-cat-pdf = pdf
common-attach-cat-text = نص
common-attach-cat-audio = صوت
common-attach-cat-video = فيديو
common-attach-cat-other = أخرى

# -- Password fields (components/PasswordInput.tsx): the button at the end of
common-password-show = إظهار كلمة المرور
common-password-hide = إخفاء كلمة المرور

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
common-module-mail = البريد
common-module-calendar = التقويم
common-module-tasks = المهام
common-module-notes = الملاحظات
common-module-contacts = جهات الاتصال
common-module-keys = المفاتيح

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
common-viewer-loading = جارٍ تحميل العارض…
common-viewer-loading-audio = جارٍ تحميل الصوت…
common-viewer-loading-image = جارٍ تحميل الصورة…
common-viewer-loading-video = جارٍ تحميل الفيديو…
common-viewer-rendering-pdf = جارٍ عرض ملف PDF…
common-viewer-pdf-failed = تعذّر عرض ملف PDF هذا.
common-viewer-no-preview = لا تتوفر معاينة مضمّنة لهذا النوع من الملفات.
common-viewer-unknown-type = نوع غير معروف
common-viewer-download = تنزيل
common-viewer-attachments = المرفقات

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = جارٍ إعادة المحاولة…
common-waiting-online = في انتظار الاتصال…
common-failure-offline = أنت غير متصل. أعد الاتصال ثم أعد المحاولة.
common-failure-network = تعذّر الوصول إلى الخادم.
common-failure-server = لم يستجب الخادم في الوقت المناسب.
common-failure-rate = طلبات كثيرة جدًا في وقت قصير. انتظر قليلًا ثم أعد المحاولة.
common-failure-conflict = تغيّر هذا في مكان آخر في الوقت نفسه. أعد التحميل ثم أعد المحاولة.
common-failure-account = تغيّر الحساب الذي سجّلت الدخول به. أعد تحميل الصفحة.
common-failure-linked = هذا غير متاح أثناء فتح صندوق بريد مرتبط.
common-failure-refused = رفض الخادم التغيير.
common-new-version = يتوفر إصدار جديد من Mailwoman. سيذكّرك إغلاق هذا التنبيه مرة أخرى بعد ساعة.
common-new-version-reload = إعادة التحميل الآن
common-toast-close = إغلاق الإشعار
common-toast-seconds = { $seconds } ث
common-attach-refresh = تحديث المرفقات
common-attach-open-message = فتح الرسالة التي تحتوي على هذا المرفق
common-attach-page-filter-note = تنطبق عوامل التصفية على هذه الصفحة. استخدم «التالي» لتصفح الرسائل الأقدم التي تحتوي على مرفقات.
common-attach-pages = صفحات المرفقات
common-attach-previous = السابق
common-attach-next = التالي
common-attach-page = الصفحة { $page }
common-attach-column-name = الملف
common-attach-column-type = النوع
common-attach-column-size = الحجم
common-attach-column-sender = المرسِل
common-attach-column-subject = الموضوع
common-attach-column-date = تاريخ الاستلام
common-pdf-preview = معاينة PDF
common-pdf-link-title = فتح الرابط من ملف PDF هذا؟
common-pdf-link-help = سيفتح هذا الوجهة في علامة تبويب جديدة خارج Mailwoman.
common-pdf-link-open = فتح الرابط
common-open-event = فتح الحدث
common-message-preview-too-large = معاينة مرفقات الرسائل محدودة بـ 4 MiB. ما زال بإمكانك تنزيل المرفق الأصلي.
common-message-preview-failed = تعذّرت معاينة مرفق الرسالة هذا. قد يكون الملف غير مدعوم أو تالفًا، أو قد تكون خدمة المعاينة غير متاحة. ما زال بإمكانك تنزيل الملف الأصلي.
common-message-preview-attachments = الملفات داخل هذه الرسالة
common-message-preview-download = تنزيل الملف
common-message-preview-size-bytes = { $count } بايت
common-attach-cat-message = الرسائل / مرفقات Outlook
common-error-retry-help = هذا القسم غير متاح مؤقتًا. أعد المحاولة لإعادة تحميله.
common-error-network-help = تحقق من اتصالك ثم أعد المحاولة. قد يكون الخادم غير متاح مؤقتًا.
common-error-page = تعذّر تحميل هذه الصفحة
common-error-page-help = تعذّر تنزيل أحد موارد الصفحة. أعد المحاولة. إذا استمرت المشكلة فأعد تحميل التطبيق للحصول على أحدث إصدار.
common-reload-app = إعادة تحميل التطبيق
common-new-version-reloading = يتوفر إصدار جديد، جارٍ إعادة التحميل…
common-attach-account-changed = تغيّر الحساب أثناء تحميل المرفقات. أعد فتح المرفقات للحساب الحالي.
common-attach-search-account = البحث في الحساب بأكمله
common-attach-scanning = جارٍ البحث… تم فحص { $pages } من صفحات الرسائل · عُثر على { $count } من الملفات
common-attach-scan-cancelled = أُلغي البحث · عُثر على { $count } من الملفات في { $pages } من الصفحات. النتائج غير مكتملة.
common-attach-scan-incomplete = انقطع البحث · عُثر على { $count } من الملفات. أعد المحاولة للبحث في الحساب بأكمله.
common-attach-scan-limited = تُعرض أول 1,000 ملف مطابق. ضيّق عوامل التصفية للعثور على ملفات أكثر تحديدًا.
common-attach-scan-complete = اكتمل البحث · عُثر على { $count } من الملفات عبر { $pages } من صفحات الرسائل
common-attach-browse-pages = تصفح الصفحات
common-attach-cancel-search = إلغاء البحث
multiselect-selected = المحدد: { $label }
multiselect-remove = إزالة { $label }
multiselect-no-match = لا توجد خيارات مطابقة
common-notification-permission-title = السماح بالإشعارات؟
common-notification-permission-help = احصل على إشعارات بالرسائل الجديدة على هذا الجهاز. يمكنك تغيير ذلك من الإعدادات.
common-notification-permission-allow = السماح بالإشعارات
common-notification-permission-later = ليس الآن
common-notification-permission-error = تعذّر تفعيل الإشعارات. تحقق من إعدادات الإذن والإشعارات ثم أعد المحاولة.
common-notifications-enabled = تم تفعيل الإشعارات على هذا الجهاز.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [zero] لا شيء محدد
    [one] تحديد واحد
    [two] تحديدان
    [few] { $count } تحديدات
    [many] { $count } تحديدًا
   *[other] { $count } تحديد
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = الخادم مشغول. انتظر بضع ثوانٍ ثم حاول مرة أخرى.
