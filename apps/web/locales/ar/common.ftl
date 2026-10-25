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
