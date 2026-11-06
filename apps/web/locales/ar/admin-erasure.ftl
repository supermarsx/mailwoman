# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = المحو والتنقية
compliance-card-erasure-coverage = استبدل معرّفات شخص واحد في سجل التدقيق والسجلات، واعرف أي النسخ الاحتياطية ما زالت تحتفظ بأشخاص مُحوا.
admin-erasure-title = المحو والتنقية
admin-erasure-intro = تستبدل «تنقية شخص» عنوان الشخص ومعرّفات حسابه في سجل التدقيق وملفات السجل المُدوَّرة باسم مستعار ثابت مفتاحي. تُختم سلسلة تجزئة التدقيق من جديد فيظل التحقق ناجحًا. كما يترك كل محو وتنقية شاهدًا، فتؤدي استعادة نسخة احتياطية أقدم إلى محو هؤلاء الأشخاص مرة أخرى.
admin-erasure-docs = يوجد سجل ما يُمحى ويُصدَّر ويُحتفظ به لكل جدول في docs/admin/erasure-coverage.md.
admin-erasure-scrub = تنقية شخص
admin-erasure-scrub-note = شغّل المعاينة أولًا: تعدّ الصفوف وأسطر السجل التي ستتغير ولا تغيّر شيئًا.
admin-erasure-address = العنوان
admin-erasure-preview = معاينة
admin-erasure-run = تنقية شخص
admin-erasure-dry-note = معاينة فقط. لم يتغير شيء.
admin-erasure-done-note = تمت التنقية. يظهر الشخص الآن باسم { $pseudonym }.
admin-erasure-nothing = لا يذكر أي سجل هذا الشخص.
admin-erasure-col-table = الجدول أو الملف
admin-erasure-col-rows = الصفوف
admin-erasure-confirm-title = تنقية هذا الشخص من السجل؟
admin-erasure-confirm-body = تُعاد كتابة مدخلات التدقيق وأسطر السجل التي تذكر الشخص بالاسم المستعار. لا يمكن التراجع عن هذا؛ ولا يمكن استرجاع العنوان من الاسم المستعار بدون المفتاح.
admin-erasure-settings = الإعدادات
admin-erasure-keys = مفاتيح الأسماء المستعارة
admin-erasure-keys-note = تُنشأ الأسماء المستعارة بجيل المفتاح الحالي. يضيف التدوير جيلًا؛ وتبقى الأجيال الأقدم إلى أن تزيلها مدة الاحتفاظ المضبوطة أعلاه.
admin-erasure-rotate = تدوير المفتاح
admin-erasure-keys-none = لم يُنشأ أي مفتاح بعد. تنشئ أول تنقية واحدًا.
admin-erasure-col-gen = الجيل
admin-erasure-col-created = تاريخ الإنشاء
admin-erasure-col-state = الحالة
admin-erasure-key-current = الحالي
admin-erasure-key-retired = متقاعد، محفوظ
admin-erasure-key-destroy = متقاعد، يُحذف في { $when }
admin-erasure-chain = سلسلة التدقيق
admin-erasure-chain-note = يحتوي كل مدخل تدقيق على تجزئة المدخل السابق. تعيد التنقية كتابة المدخلات وتختم السلسلة من جديد، وتسجّل تجزئة الرأس القديمة والجديدة.
admin-erasure-seal = ختم المدخلات الجديدة
admin-erasure-chain-ok = تم التحقق من السلسلة: { $rows } مدخلًا مختومًا، و{ $unsealed } بانتظار الختم.
admin-erasure-chain-bad = لم يتم التحقق من السلسلة؛ أول مدخل سيئ عند الموضع { $seq }.
admin-erasure-col-time = الوقت
admin-erasure-col-range = المواضع
admin-erasure-backups = النسخ الاحتياطية
admin-erasure-backups-note = تسجّل النسخة الاحتياطية علامة مستوى الشواهد عند إنشائها. علامة المستوى الحالية هي { $watermark }.
admin-erasure-col-label = التسمية
admin-erasure-col-location = الموقع
admin-erasure-col-watermark = علامة المستوى
admin-erasure-col-marker = ممحو منذ
admin-erasure-col-actions = الإجراءات
admin-erasure-backups-none = لا توجد نسخ احتياطية مسجَّلة.
admin-erasure-contains-erased = تحتوي على أشخاص ممحوين ({ $count })
admin-erasure-no-marker = لا شيء
admin-erasure-forget = نسيان
admin-erasure-register = تسجيل نسخة احتياطية
