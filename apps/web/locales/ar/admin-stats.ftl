# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = الأرقام الرئيسية
admin-stats-charts = المخططات
admin-stats-tables = الجداول
admin-stats-group-traffic = حركة المرور في هذه الفترة
admin-stats-group-storage = التخزين والتسليم، الآن
admin-stats-refresh = تحديث الإحصاءات
admin-stats-loading = جارٍ تحميل الإحصاءات…
admin-stats-requests-total = الطلبات
admin-stats-errors-total = أخطاء الخادم (5xx)
admin-stats-p95-peak = أبطأ مئين 95
admin-stats-requests = الطلبات
admin-stats-errors = أخطاء الخادم (5xx)
admin-stats-not-found = غير موجود (404)
admin-stats-latency = زمن الاستجابة
admin-stats-series-p50 = الوسيط
admin-stats-series-p95 = المئين 95
admin-stats-unit-count = العدد لكل { $bucket }
admin-stats-unit-duration = الوقت لكل { $bucket }، بالوحدة المعروضة
admin-stats-unit-accounts = الحسابات المحظورة لكل نطاق
admin-stats-chart-about = مخطط خطي من { $from } إلى { $to }.
admin-stats-bars-about = { $count ->
    [zero] مخطط أعمدة بلا نطاقات.
    [one] مخطط أعمدة بنطاق واحد.
    [two] مخطط أعمدة بنطاقين.
    [few] مخطط أعمدة بـ { $count } نطاقات.
    [many] مخطط أعمدة بـ { $count } نطاقًا.
   *[other] مخطط أعمدة بـ { $count } نطاق.
}
admin-stats-view-table = عرض كجدول
admin-stats-legend = مفتاح الرسم
admin-stats-empty-chart = لم يُقَس أي شيء في هذه الفترة بعد. اختر فترة أطول، أو عد بعد أن يعالج الخادم بعض الطلبات.
admin-stats-empty-blocked = لا توجد حسابات محظورة.
admin-stats-delta-vs = مقارنة بالفترة السابقة
admin-stats-delta-same = مثل الفترة السابقة
admin-stats-delta-sr-up = ارتفع بمقدار { $change } عن الفترة السابقة.
admin-stats-delta-sr-down = انخفض بمقدار { $change } عن الفترة السابقة.
admin-stats-delta-sr-same = مثل الفترة السابقة.
admin-ct-group = أوقات تحميل العميل
admin-ct-metric-timeToUsable = الوقت حتى الجاهزية للاستخدام
admin-ct-metric-ttfb = أول بايت
admin-ct-metric-dcl = جاهزية DOM
admin-ct-metric-load = تحميل الصفحة
admin-ct-metric-fcp = أول عرض للمحتوى
admin-ct-metric-lcp = أكبر عرض للمحتوى
admin-ct-metric-folderOpen = فتح مجلد
admin-ct-metric-folderLoadCold = تحميل المجلد، من الشبكة
admin-ct-metric-folderLoadCached = تحميل المجلد، من ذاكرة التخزين المؤقت
admin-ct-metric-messageLoadCold = تحميل الرسالة، من الشبكة
admin-ct-metric-messageLoadCached = تحميل الرسالة، من ذاكرة التخزين المؤقت أو الجلب المسبق
admin-ct-device-desktop = سطح المكتب
admin-ct-device-mobile = الجوال
admin-ct-device-unknown = غير معروف
admin-ct-tile = { $metric }، { $stat }
admin-ct-samples =
    { $count ->
        [zero] لا عينات
        [one] عينة واحدة
        [two] عينتان
        [few] { $count } عينات
        [many] { $count } عينة
       *[other] { $count } عينة
}
admin-ct-no-samples = لا توجد عينات في هذه الفترة.
admin-ct-few-samples = { $count } عينات فقط؛ يلزم { $min } على الأقل.
admin-ct-chart-usable = الوقت حتى الجاهزية للاستخدام
admin-ct-group-folder = أوقات تحميل المجلد
admin-ct-group-message = أوقات تحميل الرسالة
admin-ct-chart-folder-cold = تحميل المجلد من الشبكة
admin-ct-chart-folder-cached = تحميل المجلد من ذاكرة التخزين المؤقت
admin-ct-chart-message-cold = تحميل الرسالة من الشبكة
admin-ct-chart-message-cached = تحميل الرسالة من ذاكرة التخزين المؤقت أو الجلب المسبق
admin-ct-folder-off = أوقات تحميل المجلد متوقفة افتراضيًا. لا يُقاس إلا من فعّلها.
admin-ct-message-off = أوقات تحميل الرسالة متوقفة افتراضيًا. لا يُقاس إلا من فعّلها.
admin-ct-chart-note = تظهر المئينات فقط للفواصل التي فيها { $min } عينات على الأقل.
admin-ct-empty-chart = لا يحتوي أي فاصل على { $min } عينات اللازمة لعرض الوسيط. اختر فترة أطول.
admin-ct-col-samples = العينات
admin-ct-col-metric = المقياس
admin-ct-col-device = الجهاز
admin-ct-table-title = أوقات تحميل العميل حسب الجهاز
admin-ct-table-caption = العينات والوسيط والمئين 95 لكل مقياس وفئة جهاز.
admin-ct-empty-table = لم تُبلَّغ أي أوقات تحميل للمتصفح في هذه الفترة.
admin-ct-failed = تعذّر تحميل أوقات تحميل العميل.
admin-ct-disabled = جمع أوقات تحميل العميل متوقف. الأرقام أدناه من قبل إيقافه.
admin-ct-note = من المتصفحات المسجَّل دخولها فقط، مع أخذ عينات من { $rate }% من الجلسات، مجهولة الهوية ومجمَّعة. يمتد الوقت حتى الجاهزية من بدء التنقل حتى تُظهر قائمة الرسائل أول صفوفها. الأرقام تقديرية، وضمن نحو { $error }% من القيمة الحقيقية.
