# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = تحديث
admin-dash-unavailable = أرقام لوحة المعلومات غير متاحة بعد.
admin-dash-unavailable-detail = لا يوفّر هذا الخادم بيانات لوحة المعلومات (GET /admin/dashboard يُرجع 404). ستصل مع تحديث لاحق للخادم. الأقسام الأخرى تعمل كما كانت.
admin-dash-error-network = تعذّر تحميل لوحة المعلومات: لم يستجب الخادم. حاول مرة أخرى.
admin-dash-error-status = تعذّر تحميل لوحة المعلومات: ردّ الخادم بـ { $status }. حاول مرة أخرى؛ وإذا تكرر ذلك، فتحقّق من سجل الخادم.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = يحتاج إلى انتباه
admin-dash-warn-fix = فتح القسم الذي يعالج هذا
admin-dash-warn-blocked-accounts = { $count ->
    [zero] لا توجد حسابات ممنوعة من تسجيل الدخول بسبب غياب إعداد نطاقها.
    [one] حساب واحد لا يمكنه تسجيل الدخول لأن نطاقه بلا إعداد.
    [two] حسابان لا يمكنهما تسجيل الدخول لأن نطاقيهما بلا إعداد.
    [few] { $count } حسابات لا يمكنها تسجيل الدخول لأن نطاقاتها بلا إعداد.
    [many] { $count } حسابًا لا يمكنها تسجيل الدخول لأن نطاقاتها بلا إعداد.
   *[other] { $count } حساب لا يمكنها تسجيل الدخول لأن نطاقاتها بلا إعداد.
}
admin-dash-warn-no-trusted-proxy = قواعد IP مفعّلة لكن لم يُعدّ أي وكيل موثوق، لذا قد يبدو أن كل العملاء يأتون من عنوان الوكيل.
admin-dash-warn-allowlist-bypass = متغير بيئة يعطّل قائمة السماح لعناوين IP.
admin-dash-warn-metrics-token = لم يُعيَّن رمز لنقطة نهاية المقاييس.
admin-dash-warn-pending-migrations = { $count ->
    [zero] لا توجد ترحيلات قاعدة بيانات غير مطبّقة.
    [one] ترحيل واحد لقاعدة البيانات لم يُطبَّق. أعد تشغيل الخادم لتطبيقه.
    [two] ترحيلان لقاعدة البيانات لم يُطبَّقا. أعد تشغيل الخادم لتطبيقهما.
    [few] { $count } ترحيلات لقاعدة البيانات لم تُطبَّق. أعد تشغيل الخادم لتطبيقها.
    [many] { $count } ترحيلًا لقاعدة البيانات لم تُطبَّق. أعد تشغيل الخادم لتطبيقها.
   *[other] { $count } ترحيل لقاعدة البيانات لم تُطبَّق. أعد تشغيل الخادم لتطبيقها.
}
admin-dash-warn-cert-expiring = { $count ->
    [zero] لا توجد شهادات خوادم أصلية تنتهي صلاحيتها قريبًا (من آخر اختبار اتصال).
    [one] شهادة خادم أصلي واحدة تنتهي صلاحيتها قريبًا (من آخر اختبار اتصال).
    [two] شهادتا خادم أصلي تنتهي صلاحيتهما قريبًا (من آخر اختبار اتصال).
    [few] { $count } شهادات خوادم أصلية تنتهي صلاحيتها قريبًا (من آخر اختبار اتصال).
    [many] { $count } شهادةً لخوادم أصلية تنتهي صلاحيتها قريبًا (من آخر اختبار اتصال).
   *[other] { $count } شهادة لخوادم أصلية تنتهي صلاحيتها قريبًا (من آخر اختبار اتصال).
}
admin-dash-warn-unknown = تحذير من الخادم { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [zero] لا توجد حسابات ممنوعة: لا إعداد للنطاق
    [one] حساب واحد ممنوع: لا إعداد للنطاق
    [two] حسابان ممنوعان: لا إعداد للنطاق
    [few] { $count } حسابات ممنوعة: لا إعداد للنطاق
    [many] { $count } حسابًا ممنوعًا: لا إعداد للنطاق
   *[other] { $count } حساب ممنوع: لا إعداد للنطاق
}
admin-dash-blocked-note = سجّلت هذه الحسابات الدخول قبل هذا التحديث، لكن نطاقها بلا إعداد في وحدة التحكم، لذا لا يمكنها تسجيل الدخول الآن. إعداد النطاق يتيح لها تسجيل الدخول مجددًا.
admin-dash-blocked-chart = الحسابات الممنوعة لكل نطاق
admin-dash-blocked-row = { $count ->
    [zero] لا حسابات
    [one] حساب واحد
    [two] حسابان
    [few] { $count } حسابات
    [many] { $count } حسابًا
   *[other] { $count } حساب
}
admin-dash-blocked-seen = (آخر مضيف بريد مُستخدم { $host })
admin-dash-configure-domain = إعداد { $domain }
admin-dash-blocked-no-access = دورك لا يسمح بإعداد النطاقات.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] قائمة الإعداد: لا شيء مفتوح
    [zero] قائمة الإعداد: لا شيء مفتوح
    [one] قائمة الإعداد: عنصر واحد مفتوح
    [two] قائمة الإعداد: عنصران مفتوحان
    [few] قائمة الإعداد: { $open } عناصر مفتوحة
    [many] قائمة الإعداد: { $open } عنصرًا مفتوحًا
   *[other] قائمة الإعداد: { $open } عنصر مفتوح
}
admin-dash-check-done = تم
admin-dash-check-open = مفتوح
admin-dash-check-optional = (اختياري)
admin-dash-check-go = فتح
admin-dash-check-hide = إخفاء
admin-dash-check-unhide = إظهار مجددًا
admin-dash-check-hidden = { $count ->
    [zero] لا توجد عناصر مخفية.
    [one] عنصر واحد مخفي. الإخفاء لا يعني أنه تم.
    [two] عنصران مخفيان. الإخفاء لا يعني أنهما تمّا.
    [few] { $count } عناصر مخفية. الإخفاء لا يعني أنها تمّت.
    [many] { $count } عنصرًا مخفيًا. الإخفاء لا يعني أنها تمّت.
   *[other] { $count } عنصر مخفي. الإخفاء لا يعني أنها تمّت.
}
admin-dash-check-show-hidden = إظهار العناصر المخفية
admin-dash-check-hide-hidden = استبعاد العناصر المخفية
admin-dash-check-domain-exists = إضافة نطاق بريد
admin-dash-check-domain-upstream = تعيين خادم بريد للنطاق، أو تعيين خادم أصلي افتراضي
admin-dash-check-test-connection = إجراء اختبار اتصال ناجح (خلال آخر 7 أيام)
admin-dash-check-preferred-domain = اختيار نطاق مفضّل، ليتمكن المستخدمون من تسجيل الدخول دون كتابته
admin-dash-check-second-admin = إنشاء حساب مسؤول ثانٍ، أو تعطيل تسجيل الدخول عبر البيئة
admin-dash-check-trusted-proxies = إعداد الوكلاء الموثوقين أمام هذا الخادم
admin-dash-check-security-policy = حفظ سياسة الأمان
admin-dash-check-metrics-token = تعيين رمز لنقطة نهاية المقاييس
admin-dash-check-branding = مراجعة العلامة التجارية

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = الحسابات والجلسات
admin-dash-users = المستخدمون المُجهَّزون
admin-dash-signed-in-24h = الحسابات التي سجّلت الدخول، آخر 24 ساعة
admin-dash-signed-in-7d = الحسابات التي سجّلت الدخول، آخر 7 أيام
admin-dash-domains = النطاقات
admin-dash-domains-detail = { $upstream } بخادم بريد، { $disabled } معطّلة
admin-dash-sessions = جلسات البريد الإلكتروني النشطة
admin-dash-admin-sessions = جلسات المسؤول النشطة
admin-dash-security = أمان تسجيل الدخول
admin-dash-failed-webmail = محاولات دخول فاشلة إلى البريد، آخر 24 ساعة
admin-dash-failed-admin = محاولات دخول فاشلة للمسؤول، آخر 24 ساعة
admin-dash-active-blocks = العناوين المحظورة
admin-dash-allowlists = قوائم السماح لعناوين IP
admin-dash-allowlists-value = المسؤول { $admin }، البريد { $webmail }
admin-dash-on = مفعّل
admin-dash-off = معطّل
admin-dash-trusted-proxies = الوكلاء الموثوقون
admin-dash-configured = مُعدّ
admin-dash-not-configured = غير مُعدّ
admin-dash-untrusted-forwarded = { $count ->
    [zero] لا توجد طلبات خلال 24 ساعة تحمل ترويسات تمرير من نظير غير موثوق
    [one] طلب واحد خلال 24 ساعة حمل ترويسات تمرير من نظير غير موثوق
    [two] طلبان خلال 24 ساعة حملا ترويسات تمرير من نظير غير موثوق
    [few] { $count } طلبات خلال 24 ساعة حملت ترويسات تمرير من نظير غير موثوق
    [many] { $count } طلبًا خلال 24 ساعة حملت ترويسات تمرير من نظير غير موثوق
   *[other] { $count } طلب خلال 24 ساعة حملت ترويسات تمرير من نظير غير موثوق
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = حركة المرور، آخر { $mins } دقيقة
admin-dash-traffic-note = يحسبها عملية الخادم هذه منذ بدء تشغيلها. لا تشمل العمليات أو النسخ المتماثلة الأخرى.
admin-dash-traffic-empty = لم تُحسب أي طلبات في هذه الفترة.
admin-dash-requests = الطلبات في الدقيقة
admin-dash-errors = أخطاء الخادم (5xx) في الدقيقة
admin-dash-p95 = زمن الاستجابة، المئين 95
admin-dash-ms = { $ms } مللي ثانية

# -- Health ----------------------------------------------------------------------------
admin-dash-health = الحالة الصحية
admin-dash-h-ok = سليم
admin-dash-h-attention = يحتاج إلى انتباه
admin-dash-h-db = قاعدة البيانات
admin-dash-h-db-ok = استجابت خلال { $ms }
admin-dash-h-migrations = الترحيلات
admin-dash-h-migrations-ok = محدّثة ({ $n } مطبّقة)
admin-dash-h-migrations-behind = { $applied } من { $expected } مطبّقة
admin-dash-h-engine = مزامنة البريد
admin-dash-h-engine-value = { $accounts } حسابات مسجّلة، { $errors } بها خطأ مزامنة
admin-dash-h-outbox = صندوق الصادر
admin-dash-h-outbox-value = { $pending } في الانتظار (الأقدم { $oldest })، { $failed } فشلت خلال 24 ساعة
admin-dash-h-outbox-none = لا شيء
admin-dash-h-index = فهرس البحث
admin-dash-h-index-value = { $n } مستندات
admin-dash-h-index-rebuilding = جارٍ إعادة البناء ({ $n } مستندات حتى الآن)
admin-dash-h-sandbox = بيئة العرض المعزولة
admin-dash-h-sandbox-ok = متاحة: تُعرض المستندات غير الموثوقة في عملية منفصلة.
admin-dash-h-sandbox-missing = غير موجودة: تُعرض المستندات غير الموثوقة داخل عملية الخادم.
admin-dash-h-sandbox-required = غير موجودة، وهذا الخادم يتطلبها: عرض المستندات غير الموثوقة يفشل.
admin-dash-h-plugins = بيئة تشغيل الإضافات
admin-dash-h-plugins-ok = متاحة
admin-dash-h-plugins-off = معطّلة
admin-dash-na-proxy = يعمل هذا الخادم في وضع الوكيل ولا يزامن البريد بنفسه.
admin-dash-na-unreported = لم يُبلغ الخادم عنها.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = التخزين
admin-dash-db-size = ملف قاعدة البيانات
admin-dash-wal-size = سجل الكتابة المسبقة
admin-dash-index-size = فهرس البحث
admin-dash-volume-free = المساحة الحرة على وحدة تخزين البيانات
admin-dash-volume-of = من { $total }
admin-dash-na-wal = لا تستخدم قاعدة البيانات هذه ملف سجل كتابة مسبقة، أو لم يُبلغ الخادم عنه.
admin-dash-na-index = لا يوجد فهرس بحث على هذا الخادم.
admin-dash-na-volume = لا يستطيع الخادم قراءة المساحة الحرة على هذه المنصة.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = إجراءات المسؤول الأخيرة
admin-dash-audit-empty = لم تُسجَّل أي إجراءات للمسؤول بعد.
admin-dash-audit-when = متى
admin-dash-audit-actor = من
admin-dash-audit-action = الإجراء
admin-dash-audit-target = الهدف
admin-dash-audit-all = فتح سجل التدقيق الكامل
admin-dash-server = الخادم
admin-dash-version = الإصدار
admin-dash-build = البناء
admin-dash-mode = الوضع
admin-dash-mode-engine = المحرك: يزامن هذا الخادم البريد ويخزّنه
admin-dash-mode-proxy = الوكيل: يمرّر هذا الخادم الطلبات إلى خادم البريد
admin-dash-uptime = يعمل منذ
admin-dash-base-path = يُقدَّم على
admin-dash-base-root = جذر الموقع
admin-dash-duration-dh = { $d } ي { $h } س
admin-dash-duration-hm = { $h } س { $m } د
admin-dash-duration-m = { $m } د
admin-dash-duration-s = { $s } ث
