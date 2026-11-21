admin-cache-title = صيانة التخزين المؤقت
admin-cache-help = افتراضيًا، تنتهي صلاحية محتوى البريد المخزن مؤقتًا بعد 60 يومًا دون استخدام. يؤدي فتح رسالة منتهية الصلاحية إلى تنزيل محتواها مجددًا. تُحفظ الرسائل الأصلية والمسودات والمحتوى المحلي المحمي.
admin-cache-expiry = مدة الاحتفاظ بالرسائل المخزنة مؤقتًا
admin-cache-orphans = الأجسام غير المرجعية واستعادة المساحة
admin-cache-advanced = حدود الدفعات والتوقيت
admin-cache-settings-help = تخضع التغييرات لسياسة إعدادات المسؤول المعتادة والتحقق وسجل التدقيق.
admin-cache-expiry-enabled = إنهاء صلاحية أجسام الرسائل المخزنة مؤقتًا تلقائيًا
admin-cache-retention-days = أيام دون استخدام قبل انتهاء الصلاحية
admin-cache-expiry-status = انتهاء صلاحية الأجسام المخزنة مؤقتًا
admin-cache-expiry-result = تم الفحص: { $scanned } · مراجع التخزين المؤقت المفصولة: { $detached } · المحمية: { $protected }
admin-cache-expiry-help = تُقبل فقط الأجسام التي تأكد إمكان استعادتها من صندوق البريد المصدر. لا يجدد الفحص وتحديث الترويسات مهلة عدم الاستخدام.
admin-cache-inspect = فحص مساحة التخزين المؤقت
admin-cache-inspect-help = افتح مراقبة قاعدة البيانات لقياس المحتوى المخزن مؤقتًا والأجسام المرجعية والنسخ اليتيمة.
admin-cache-no-status = لم تكتمل أي دفعة لانتهاء الصلاحية في عملية الخادم هذه.
admin-cache-settings-unavailable = تعذر تحميل إعدادات التخزين المؤقت.
admin-cache-change-log = سجل المزامنة
admin-cache-change-log-help = افتراضيًا، تنتهي صلاحية سجلات التغييرات القديمة بعد 60 يومًا. تتلقى التطبيقات التي تستخدم رموز مزامنة أقدم استجابة تطلب إعادة مزامنة كاملة.
admin-cache-no-sources = لا يملك أي صندوق بريد متصل حاليًا مسارًا مؤكدًا لإعادة جلب المحتوى. يُحتفظ بالمحتوى المحمي.
admin-cache-wal-recycled = أُعيد تدوير مساحة WAL المخصصة؛ يمكن متابعة التنظيف.
admin-cache-wal-recovery = ينتظر التنظيف استعادة WAL أو إنشاء نقطة تحقق. تُحفظ بيانات السجل المعلقة.

admin-cache-removed-payload = المحتوى المؤقت الذي أزيل
admin-cache-oversized = تأجيل تنظيف محتوى الرسائل الكبير
admin-cache-change-removed = سجلات محفوظات المزامنة التي أزيلت

admin-cache-parsed-copies = نسخ محتوى الرسائل المعالج التي أزيلت
admin-cache-parsed-bytes = بيانات محتوى الرسائل المعالج التي أزيلت
admin-cache-change-chunked = حالات المزامنة التي نُظفت على دفعات

# Public static representations only; process-local counters.
admin-cache-assets-title = ذاكرة RAM للموارد الثابتة العامة
admin-cache-assets-clear = مسح الذاكرة الثابتة
admin-cache-assets-help = تحتفظ بالموارد العامة فقط. تشمل الميزانية البيانات المقيمة المملوكة، ولا تشمل الردود الجارية أو ذاكرة العملية بأكملها. قد تبقى الصفحة الأولية وملفات عامل الخدمة المنشورة حتى انتهاء فترة إعادة التحقق.
admin-cache-assets-clear-help = يحرر المسح الإدخالات المحتفظ بها؛ وتكتمل الردود النشطة كالمعتاد. قد يعيد التحميل المسبق المفعّل ملء الذاكرة. تخص العدادات عملية الخادم هذه.
admin-cache-assets-stale = تعذر تحديث السياسة؛ يبقى آخر إعداد مطبق فعالاً.
admin-cache-assets-applied = وقت تطبيق الإعداد: { $time }
admin-cache-assets-resident = البيانات المملوكة / الميزانية
admin-cache-assets-entries = الإدخالات / الحد
admin-cache-assets-file = الحد الفعلي للملف المملوك
admin-cache-assets-loads = قيد التنفيذ / الحد؛ بالانتظار / الحد
admin-cache-assets-hits = إصابات / تحميلات مقبولة / طلبات انتظار مشتركة
admin-cache-assets-evictions = محذوفة / منتهية / أكبر من الحد
admin-cache-assets-failures = رفض بسبب الانشغال / مهام فاشلة
admin-cache-assets-preload = وضع التحميل المسبق؛ المحتفظ بها / المتخطاة

admin-cache-assets-encoded = البيانات المضغوطة / الجزء المملوك
admin-cache-assets-encoding-loads = الملفات المتحققة / الترميزات المنشأة
admin-cache-assets-encoding-help = تشارك الملفات العامة المضغوطة ميزانية التخزين المؤقت. تُقارن بالبايتات الأصلية، ويمكن ترميز الملفات المعدّلة مرة واحدة أثناء تحميل محدود. يُستثنى البريد وبيانات API وHTML الخاص.
