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

# -- Health ----------------------------------------------------------------------------
admin-dash-health = الحالة الصحية
admin-dash-h-ok = سليم
admin-dash-h-attention = يحتاج إلى انتباه
admin-dash-h-db = قاعدة البيانات
admin-dash-h-migrations = الترحيلات
admin-dash-h-engine = مزامنة البريد
admin-dash-h-outbox = صندوق الصادر
admin-dash-h-index = فهرس البحث
admin-dash-h-sandbox = بيئة العرض المعزولة
admin-dash-h-plugins = بيئة تشغيل الإضافات

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = التخزين
admin-dash-db-size = ملف قاعدة البيانات
admin-dash-wal-size = سجل الكتابة المسبقة
admin-dash-index-size = فهرس البحث
admin-dash-volume-free = المساحة الحرة على وحدة تخزين البيانات
admin-dash-volume-of = من { $total }
admin-dash-na-wal = لا تستخدم قاعدة البيانات هذه ملف سجل كتابة مسبقة، أو لم يُبلغ الخادم عنه.
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

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = سياسة الدولة مفعّلة لكن لم تُحمَّل قاعدة بيانات دول صالحة للاستخدام، لذا تُرفض عمليات الدخول التي تشملها.
admin-dash-warn-geoip-stale = سياسة الدولة مفعّلة وقاعدة بيانات الدول قديمة.
admin-dash-h-status-ok = سليم
admin-dash-h-status-warn = تحذير
admin-dash-h-status-fail = فاشل
admin-dash-h-status-disabled = معطّل
admin-dash-h-checked = وقت الفحص: { $time }
admin-dash-h-refresh = فحص مجددًا
admin-dash-h-refresh-failed = تعذّرت إعادة فحص السلامة (الحالة { $status }). تُعرض النتيجة السابقة.
admin-dash-h-refresh-network = تعذّرت إعادة فحص السلامة: لم يستجب الخادم. تُعرض النتيجة السابقة.
admin-dash-h-enable = لتفعيله
admin-dash-h-probe-time = استغرق الفحص { $time }
admin-dash-h-outbox-probe-time = استغرق تجميع الحالة { $time }؛ ويُعرض توقيت مصافحة الإرسال بشكل منفصل.
admin-dash-h-history = آخر { $n } فحوصات
admin-dash-h-open-diagnostics = التشخيص
admin-dash-h-open-logs = السجلات
admin-dash-h-open-plugins = الإضافات
admin-dash-h-open-maintenance = الصيانة
admin-dash-h-not-tracked = غير متتبَّع
admin-dash-h-unlimited = بلا حد
admin-dash-h-m-none = لا شيء
admin-dash-h-m-ago = منذ { $age }
admin-dash-h-m-ping = زمن الذهاب والإياب
admin-dash-h-m-pool = مجمّع الاتصالات
admin-dash-h-m-pool-value = { $inUse } قيد الاستخدام، { $idle } خاملة ({ $size } مفتوحة)
admin-dash-h-m-engine = خادم قاعدة البيانات
admin-dash-h-m-location = الموقع
admin-dash-h-m-db-size = حجم قاعدة البيانات
admin-dash-h-m-wal-size = سجل الكتابة المسبقة
admin-dash-h-m-journal = وضع السجل
admin-dash-h-m-slow = استعلامات بطيئة (آخر ساعة)
admin-dash-h-m-applied = المطبّقة
admin-dash-h-m-applied-value = { $applied } من { $expected } في هذا الإصدار
admin-dash-h-m-latest = أحدث ما طُبّق
admin-dash-h-m-pending = لم تُطبَّق بعد
admin-dash-h-m-failed = بدأت ولم تكتمل
admin-dash-h-m-unknown = من إصدار أحدث
admin-dash-h-m-accounts = صناديق البريد
admin-dash-h-m-accounts-value = { $total } مسجّلة: { $syncing } قيد المزامنة، { $idle } خاملة، { $erroring } فاشلة
admin-dash-h-m-last-sync = منذ آخر مزامنة مكتملة
admin-dash-h-m-last-sync-value = الوسيط { $p50 }، الأطول { $max }
admin-dash-h-m-never-synced = لم تتم مزامنة منذ بدء الخادم
admin-dash-h-m-queue = عمليات المزامنة الجارية
admin-dash-h-m-errors-hour = أخطاء المزامنة (آخر ساعة)
admin-dash-h-m-by-kind = الاتصالات حسب الخلفية
admin-dash-h-m-kind-value = { $kind }: { $accounts } صناديق بريد، { $watch } اتصالات مراقبة
admin-dash-h-m-recent-errors = الأخطاء الأخيرة
admin-dash-h-m-queued = بانتظار الإرسال
admin-dash-h-m-queued-value = { $queued } مستحقة، { $deferred } قيد إعادة المحاولة، { $sending } قيد الإرسال
admin-dash-h-m-oldest = الأقدم انتظارًا
admin-dash-h-m-scheduled = مجدولة أو معلّقة
admin-dash-h-m-scheduled-value = { $scheduled } مجدولة، { $held } معلّقة
admin-dash-h-m-failed-total = فاشلة (لا تزال في صندوق الصادر)
admin-dash-h-m-failed-total-value = { $failed } فاشلة، { $uncertain } بتسليم غير معروف
admin-dash-h-m-sends-hour = الإرسال (آخر ساعة)
admin-dash-h-m-sends-hour-value = { $sent } أُرسلت، { $deferred } مؤجلة، { $failed } فاشلة
admin-dash-h-m-last-send = آخر إرسال
admin-dash-h-m-last-send-value = { $outcome }، منذ { $age }
admin-dash-h-m-last-send-none = لا يوجد منذ بدء الخادم
admin-dash-h-outcome-sent = أُرسلت
admin-dash-h-outcome-deferred = مؤجلة
admin-dash-h-outcome-failed = فاشلة
admin-dash-h-outcome-uncertain = تسليم غير معروف
admin-dash-h-m-submission = خوادم الإرسال
admin-dash-h-m-submission-value = استجاب { $reachable } من { $checked } (مصافحة فقط دون تسجيل دخول؛ الأبطأ { $slowest })، فُحصت منذ { $age }
admin-dash-h-m-submission-none = لم يُعدّ أي خادم إرسال.
admin-dash-h-m-submission-pending = فحص الاتصال جارٍ.
admin-dash-h-m-submission-unmeasured = يتطلب { $count } خادم/خوادم فحصًا موثّقًا لدى المزوّد؛ لم تُرسل أي بيانات اعتماد.
admin-dash-h-m-documents = المستندات
admin-dash-h-m-documents-value = { $documents } من { $messages } رسالة مخزّنة
admin-dash-h-m-index-pending = لم تُفهرس بعد
admin-dash-h-m-index-size = الحجم على القرص
admin-dash-h-m-index-memory = في الذاكرة
admin-dash-h-m-last-commit = آخر إيداع
admin-dash-h-m-rebuild = إعادة البناء
admin-dash-h-m-rebuild-running = قيد التشغيل: { $pct } ({ $done } من { $total })
admin-dash-h-m-rebuild-idle = غير قيد التشغيل
admin-dash-h-m-version = إصدار العامل
admin-dash-h-m-worker-missing = لا يوجد ملف تنفيذي للعامل
admin-dash-h-m-render-ping = Ping (البدء، jail، الاستجابة)
admin-dash-h-m-render-jobs = المهام الجارية الآن
admin-dash-h-m-render-failures = المشكلات (آخر ساعة)
admin-dash-h-m-render-failures-value = { $failed } فاشلة، { $timeouts } انتهت مهلتها، { $busy } رُفضت أثناء الانشغال
admin-dash-h-m-jail = jail النواة
admin-dash-h-m-jail-required = مطلوب
admin-dash-h-m-jail-optional = غير مطلوب على هذه المنصة
admin-dash-h-m-jail-degraded = متدهور: { $why }
admin-dash-h-m-plugins = الإضافات
admin-dash-h-m-plugins-value = { $installed } مثبّتة، { $enabled } مفعّلة، { $loaded } محمّلة
admin-dash-h-m-plugin-list = الإضافات المفعّلة
admin-dash-h-m-plugin-loaded = محمّلة
admin-dash-h-m-plugin-not-loaded = غير محمّلة ({ $reason })
admin-dash-h-m-plugin-limits = الذاكرة { $memory }، المهلة { $deadline }، الوقود { $fuel }
admin-dash-h-m-plugin-calls = إخفاقات الاستدعاء (آخر ساعة)
admin-dash-h-m-plugin-calls-value = { $traps } فخاخ، { $limits } تجاوزات للحدود، { $errors } أخطاء أخرى
admin-dash-h-m-host-ping = قفل مضيف الإضافات
admin-dash-h-t-ping = تحذير فوق { $warn }، وفشل فوق { $fail }
admin-dash-h-t-oldest = تحذير عندما تنتظر رسالة أكثر من { $warn }، وفشل عند أكثر من { $fail }
admin-dash-h-t-pending = تحذير عند وجود أكثر من { $n } رسالة غير مفهرسة
admin-dash-h-t-render = تحذير فوق { $warn }؛ وفشل عند عدم وجود استجابة خلال { $timeout }
admin-dash-h-t-sync = فشل عندما تخفق مزامنة { $pct } أو أكثر من صناديق البريد
admin-dash-h-chart-db = زمن ذهاب وإياب قاعدة البيانات
admin-dash-h-chart-render = Ping عامل العرض
admin-dash-h-chart-outbox = رسائل بانتظار الإرسال
admin-dash-h-chart-search = رسائل غير مفهرسة
admin-dash-na-index-proxy = وضع الوكيل: لا يحتفظ هذا الخادم بفهرس بحث.
admin-dash-na-index-memory = فهرس البحث محفوظ في الذاكرة، لذا ليس له حجم على القرص.
admin-dash-customize = تخصيص لوحة المعلومات
admin-dash-limitations = توفر البيانات
