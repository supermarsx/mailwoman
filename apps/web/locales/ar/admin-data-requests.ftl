# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-data-requests. Source: ../en/admin-data-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-dr-title = طلبات البيانات
admin-dr-intro = يقدّم الأشخاص هذه الطلبات ضمن الإعدادات > الخصوصية والبيانات. أنواع الطلبات المعروضة ومن يوافق عليها ومدتها ومحتوى التصدير هي إعدادات لكل نطاق.
admin-dr-settings-link = فتح الإعدادات
admin-dr-loading = جارٍ التحميل
admin-dr-empty = لا توجد طلبات مطابقة.
admin-dr-stats = قائمة الانتظار
admin-dr-charts = النشاط
admin-dr-tile-open = الطلبات المفتوحة
admin-dr-tile-overdue = المتأخرة
admin-dr-tile-soon = تستحق خلال 7 أيام
admin-dr-tile-ontime = أُجيب عنها بحلول الموعد النهائي
admin-dr-tile-avg = متوسط وقت الرد
admin-dr-tile-none = لا توجد طلبات أُجيب عنها بعد.
admin-dr-hours = ساعات
admin-dr-chart-created = الطلبات المقدَّمة في اليوم
admin-dr-chart-unit = العدد في اليوم، آخر 30 يومًا
admin-dr-chart-summary = قُدّم { $total } طلب في آخر 30 يومًا.
admin-dr-chart-empty = لا توجد طلبات في هذه الفترة.
admin-dr-chart-status = الطلبات حسب الحالة
admin-dr-col-day = اليوم
admin-dr-col-count = الطلبات
admin-dr-filters = عوامل التصفية
admin-dr-any = أي
admin-dr-open = مفتوح (غير منتهٍ)
admin-dr-due-overdue = متأخر
admin-dr-due-soon = يستحق خلال 7 أيام
admin-dr-assignee = مُسنَد إلى
admin-dr-assignee-hint = الاسم، أو لا أحد
admin-dr-search = البحث بالعنوان أو المعرّف
admin-dr-export = تصدير السجل (CSV)
admin-dr-caption = عرض { $shown } من أصل { $total } طلب
admin-dr-col-person = الشخص
admin-dr-col-type = النوع
admin-dr-col-status = الحالة
admin-dr-col-due = الموعد النهائي
admin-dr-col-actions = الإجراءات
admin-dr-unassigned = غير مُسنَد
admin-dr-open-request = فتح
admin-dr-days-left = تبقّى { $days } يوم
admin-dr-overdue-by = متأخر { $days } يوم
admin-dr-answered = أُجيب عنه
admin-dr-detail = تفاصيل الطلب
admin-dr-close = إغلاق
admin-dr-hold = حجز قانوني ({ $source }): { $reason }
admin-dr-hold-policy = من إعداد الحجز القانوني
admin-dr-hold-request = معيَّن على هذا الطلب
admin-dr-hold-badge = حجز قانوني
admin-dr-blocked-note = المحو محجوب بسبب حجز قانوني ولن يُنفَّذ. يستمر عند رفع الحجز.
admin-dr-filed-by = قدّمه
admin-dr-by-self = الشخص نفسه
admin-dr-verified = تم التحقق من الهوية
admin-dr-grace = ينفَّذ المحو بعد
admin-dr-error = آخر خطأ
admin-dr-completion = سجل الإنجاز
admin-dr-assign = إسناد
admin-dr-note = ملاحظة أو سبب
admin-dr-extend-days = التمديد بمقدار (أيام)
admin-dr-verify = تسجيل التحقق من الهوية
admin-dr-approve = موافقة
admin-dr-reject = رفض
admin-dr-extend = تمديد الموعد النهائي
admin-dr-complete = وضع علامة مكتمل
admin-dr-hold-set = وضعه تحت حجز قانوني
admin-dr-hold-release = رفع الحجز
admin-dr-run = تشغيل الآن
admin-dr-trail = السجل
admin-dr-behalf = تقديم طلب نيابةً عن مستخدم
admin-dr-behalf-help = لطلب وصل بالبريد أو شخصيًا. لا يُطلب من الشخص التأكيد برسالة.
admin-dr-behalf-submit = تقديم الطلب

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-dr-bulk = الإجراءات السائبة

admin-dr-bulk-action = العمل

admin-dr-bulk-approve = موافقة

admin-dr-bulk-assign = تعيين

admin-dr-bulk-clear = مسح التحديد

admin-dr-bulk-csv = CSV من الاختيار

admin-dr-bulk-extend = تمديد الموعد المحدد

admin-dr-bulk-failed = رفض

admin-dr-bulk-field-assignee = تعيين إلى (مسح فارغ)

admin-dr-bulk-field-extend = سبب التمديد

admin-dr-bulk-field-note = ملاحظة (اختياري)

admin-dr-bulk-field-reason = السبب

admin-dr-bulk-hold = وضع تحت الحجز القانوني

admin-dr-bulk-ok = تم

admin-dr-bulk-reject = رفض

admin-dr-bulk-release-hold = الافراج عن الحجز القانوني

admin-dr-bulk-run = { $count ->
        [zero] تنطبق على { $count } طلبات
        [one] تنطبق على طلب واحد
        [two] تطبيق على طلبين
        [few] تنطبق على { $count } طلبات
        [many] تنطبق على { $count } طلبا
       *[other] تنطبق على { $count } طلب
    }

admin-dr-bulk-select = اختر طلب { $address }

admin-dr-bulk-select-all = حدد كل طلب مدرج

admin-dr-bulk-selected = { $count ->
        [zero] تم تحديد { $count } طلبات
        [one] تم اختيار طلب واحد
        [two] تم تحديد طلبين
        [few] تم اختيار { $count } طلبات
        [many] تم اختيار { $count } طلب
       *[other] تم اختيار { $count } طلب
    }

admin-dr-bulk-summary = تم لـ { $done }; رفض ل { $failed }. تظل الطلبات المرفوضة محددة.

admin-dr-more = تحميل المزيد من الطلبات
