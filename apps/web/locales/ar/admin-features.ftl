# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = ملفات تعريف الميزات
admin-features-intro = ملف التعريف مجموعة مسمّاة من مفاتيح الميزات. يمكنك تعيينه لنطاق أو مجموعة أو مستخدم. تدعم التعيينات الفردية صناديق البريد الأصلية وهويات الوكيل أو SSO الموثَّقة؛ وهي تتحكم في الوصول إلى ميزات Mailwoman.
admin-features-new = ملف تعريف جديد
admin-features-search = بحث في ملفات التعريف
admin-features-empty = لا توجد ملفات تعريف ميزات بعد.
admin-features-no-match = لا يوجد ملف تعريف يطابق البحث.
admin-features-load-error = تعذّر تحميل كتالوج الميزات.
admin-features-not-loaded = لم يُحمَّل كتالوج الميزات بعد. أعد تحميل الصفحة وحاول مجددًا.
admin-features-stale = غيّر مسؤول آخر كتالوج الميزات. أُعيد تحميله؛ تحقق من القيم واحفظ مرة أخرى.
admin-features-not-found = لا يوجد ملف تعريف بالمعرّف { $id }.
admin-features-col-name = الاسم
admin-features-col-values = المفاتيح
admin-features-col-assigned = معيّن إلى
admin-features-values-count = { $count ->
    [zero] لا مفاتيح معيّنة
    [one] مفتاح واحد معيّن
    [two] مفتاحان معيّنان
    [few] { $count } مفاتيح معيّنة
    [many] { $count } مفتاحًا معيّنًا
   *[other] { $count } مفتاح معيّن
}
admin-features-assigned-count = { $count ->
    [0] غير معيّن
    [zero] غير معيّن
    [one] موضع واحد
    [two] موضعان
    [few] { $count } مواضع
    [many] { $count } موضعًا
   *[other] { $count } موضع
}
admin-features-editor = إعدادات ملف التعريف
admin-features-name = اسم ملف التعريف
admin-features-name-help = يظهر للمسؤولين فقط، حتى 128 حرفًا.
admin-features-name-invalid = أدخل اسمًا من 1 إلى 128 حرفًا.
admin-features-switches = مفاتيح الميزات
admin-features-switches-help = تترك «غير معيّن» الميزة لملف التعريف التالي أو النطاق أو الإعداد الافتراضي للخادم. لا يمكن للمستخدم أو لمستوى أدنى تغيير قيمة مقفلة.
admin-features-col-feature = الميزة
admin-features-col-value = القيمة
admin-features-col-lock = القفل
admin-features-value-for = قيمة { $feature }
admin-features-unset = غير معيّن
admin-features-on = مسموح
admin-features-off = غير مسموح
admin-features-lock = مقفل
admin-features-save = حفظ ملف التعريف
admin-features-create = إنشاء ملف تعريف
admin-features-delete = حذف ملف التعريف
admin-features-delete-title = حذف ملف التعريف { $name }؟
admin-features-delete-body = { $count ->
    [0] ملف التعريف غير معيّن في أي مكان. تتوقف مفاتيحه عن السريان فورًا.
    [zero] ملف التعريف غير معيّن في أي مكان. تتوقف مفاتيحه عن السريان فورًا.
    [one] يُزال ملف التعريف من الموضع الواحد المعيّن إليه، وتتوقف مفاتيحه عن السريان فورًا.
    [two] يُزال ملف التعريف من الموضعين المعيّن إليهما، وتتوقف مفاتيحه عن السريان فورًا.
    [few] يُزال ملف التعريف من { $count } مواضع معيّن إليها، وتتوقف مفاتيحه عن السريان فورًا.
    [many] يُزال ملف التعريف من { $count } موضعًا معيّنًا إليها، وتتوقف مفاتيحه عن السريان فورًا.
   *[other] يُزال ملف التعريف من { $count } موضع معيّن إليها، وتتوقف مفاتيحه عن السريان فورًا.
}
admin-features-saved = حُفظ ملف التعريف { $name }.
admin-features-deleted = حُذف ملف التعريف { $name }.
admin-features-assigned = عُيّن ملف التعريف { $name }.
admin-features-unassigned = أُزيل تعيين { $name }.
admin-features-assignments = معيّن إلى
admin-features-assignments-help = تُطبَّق ملفات التعريف بترتيب الأولوية، الأعلى أولًا. عند تساوي الأولوية يُطبَّق تعيين المستخدم قبل المجموعة، والمجموعة قبل النطاق.
admin-features-assignments-empty = لم يُعيَّن ملف التعريف هذا بعد.
admin-features-assign-form = تعيين ملف التعريف هذا
admin-features-col-scope = ينطبق على
admin-features-col-target = الاسم
admin-features-col-priority = الأولوية
admin-features-col-actions = الإجراءات
admin-features-scope-domain = النطاق
admin-features-scope-group = المجموعة
admin-features-scope-user = المستخدم
admin-features-assign = تعيين
admin-features-unassign = إزالة
admin-features-no-domain = لم يُضبط أي نطاق.
admin-features-no-group = لا توجد مجموعات مستخدمين.
admin-features-no-user = لم يسجّل أي حساب صندوق بريد الدخول بعد.
admin-features-tools = أدوات ملف التعريف
admin-features-preset = البدء من إعداد مسبق
admin-features-preset-choose = اختر إعدادًا مسبقًا
admin-features-preset-apply = تطبيق الإعداد المسبق
admin-features-preset-applied = طُبّق الإعداد المسبق { $name }. راجع القيم واحفظ.
admin-features-preset-full = كامل
admin-features-preset-standard = قياسي
admin-features-preset-mail-only = البريد فقط
admin-features-preset-minimal = أدنى / كشك
admin-features-import = استيراد JSON
admin-features-export = تصدير JSON
admin-features-exported = صُدِّر ملف التعريف كملف JSON.
admin-features-imported = { $count ->
    [zero] لم تُستورد قيم. راجعها واحفظ.
    [one] استُوردت قيمة واحدة. راجعها واحفظ.
    [two] استُوردت قيمتان. راجعهما واحفظ.
    [few] استُوردت { $count } قيم. راجعها واحفظ.
    [many] استُوردت { $count } قيمة. راجعها واحفظ.
   *[other] استُوردت { $count } قيمة. راجعها واحفظ.
}
admin-features-import-json = الملف ليس JSON صالحًا.
admin-features-import-format = الملف ليس ملف تعريف ميزات لـ Mailwoman.
admin-features-import-version = ملف تعريف الميزات هذا من إصدار أحدث ولا يمكن استيراده.
admin-features-clone = استنساخ
admin-features-clone-name = نسخة من { $name }
admin-features-filter = البحث عن ميزة
admin-features-filter-placeholder = الاسم أو الإعداد أو المفتاح
admin-features-filter-none = لا توجد ميزة تطابق البحث.
admin-features-diff-summary = { $differs ->
    [zero] لا قيم تختلف عن الإعداد الافتراضي للخادم
    [one] قيمة واحدة تختلف عن الإعداد الافتراضي للخادم
    [two] قيمتان تختلفان عن الإعداد الافتراضي للخادم
    [few] { $differs } قيم تختلف عن الإعداد الافتراضي للخادم
    [many] { $differs } قيمة تختلف عن الإعداد الافتراضي للخادم
   *[other] { $differs } قيمة تختلف عن الإعداد الافتراضي للخادم
} · { $same } تكرّره
admin-features-differs = تختلف عن الافتراضي
admin-features-default-is = الافتراضي: { $value }
admin-features-fix-values = صحّح القيم المعلَّمة بأنها غير صالحة قبل الحفظ.
admin-features-allow-all = السماح بالكل
admin-features-deny-all = عدم السماح بالكل
admin-features-clear-all = مسح
admin-features-group-allow = السماح بكل ميزة في { $group }
admin-features-group-deny = عدم السماح بأي ميزة في { $group }
admin-features-group-clear = مسح كل قيمة في { $group }
admin-features-enforcement-ui = مخفي في التطبيق فقط
admin-features-enforcement-partial = بعض الحدود لم تُحتسب بعد
admin-features-other = إعدادات أخرى يمكن أن يتضمنها ملف التعريف
admin-features-other-help = كل إعداد آخر يجوز للمسؤول ضبطه لكل ملف تعريف. اترك الحقل فارغًا للإبقاء على قيمة الخادم.
admin-features-cat-compose = البريد والإنشاء
admin-features-cat-reading = القراءة والخصوصية
admin-features-cat-views = العروض والبحث
admin-features-cat-organise = التنظيم
admin-features-cat-pim = التقويم وجهات الاتصال والمهام والملاحظات والمفاتيح
admin-features-cat-data = البيانات
admin-features-cat-accounts = الحسابات والوصول
admin-features-cat-integrations = عمليات التكامل
admin-features-cat-developer = وصول المطوّرين
admin-features-cat-personal = التخصيص
admin-features-feature-compose-sending = حدود الإرسال
admin-features-feature-compose-sending-help = يرفض الخادم عدد المستلمين لكل رسالة إذا تجاوز الحد. يُخزَّن عدد الرسائل في الساعة وفي اليوم لملف التعريف لكنه لا يُحتسب بعد.
admin-features-feature-storage = حصة التخزين
admin-features-feature-storage-help = يُخزَّن حد التخزين لملف التعريف؛ وتظل حصة خادم البريد نفسه سارية.
admin-features-feature-access-protocols = بروتوكولات الوصول إلى البريد
admin-features-feature-access-protocols-help = بروتوكولات خادم البريد التي يجوز لهؤلاء المستخدمين تسجيل الدخول بها، مفصولة بفواصل: imap, pop3, ews.
admin-features-feature-accounts-linked = صناديق البريد المرتبطة
admin-features-feature-calendar = التقويم
admin-features-feature-contacts = جهات الاتصال
admin-features-feature-tasks = المهام
admin-features-feature-notes = الملاحظات
admin-features-preview = القيم الفعّالة
admin-features-preview-help = ما تؤول إليه كل ميزة لمستخدم واحد أو لمستخدم في نطاق واحد، بعد كل ملف تعريف ومجموعة وقيمة نطاق وقفل.
admin-features-preview-kind = معاينة لـ
admin-features-preview-target = من
admin-features-preview-show = عرض
admin-features-preview-loading = جارٍ التحميل…
admin-features-preview-state = الفعّال
admin-features-preview-from = حُسم بواسطة
admin-features-preview-profiles = ملفات التعريف المنطبقة، الأول يغلب: { $names }
admin-features-preview-no-profiles = لا ينطبق أي ملف تعريف؛ تحسم قيم النطاق والخادم.
admin-features-source-default = الافتراضي المدمج
admin-features-source-env-fallback = افتراضي البيئة
admin-features-source-env-pin = البيئة (مثبّتة)
admin-features-source-global = قيمة الخادم
admin-features-source-domain = قيمة النطاق
admin-features-source-profile = ملف التعريف
admin-features-source-profile-named = ملف التعريف { $name }
admin-features-source-group = قيمة المجموعة
admin-features-source-admin-user = قيمة معيّنة لهذا المستخدم
admin-features-source-user = اختيار المستخدم نفسه
admin-features-source-locked = { $source } (مقفل)
admin-audit-retention-title = الاحتفاظ بسجل التدقيق
admin-audit-retention-intro = المدة التي تُحفظ فيها إدخالات التدقيق، لكل فئة. تتبع الفئة التي ليست لها قيمة خاصة القيمة الافتراضية؛ وبلا قيمة افتراضية تُحفظ كل الإدخالات. يوقف الحجز القانوني التنظيف لفئته، ويوقفه الحجز على القيمة الافتراضية لكل الفئات. تُسجَّل التغييرات وعمليات التنظيف في سجل التدقيق.
admin-audit-retention-loading = جارٍ تحميل سياسة الاحتفاظ…
admin-audit-retention-unavailable = الاحتفاظ بالتدقيق غير متاح على هذا الخادم.
admin-retention-col-category = الفئة
admin-retention-col-kept = المحفوظ
admin-retention-col-change = التغيير
admin-retention-cat-default = الافتراضي
admin-retention-cat-admin = إجراءات المسؤولين
admin-retention-cat-security = أحداث الأمان
admin-retention-cat-alert = تنبيهات المستخدمين
admin-retention-forever = كل الإدخالات
admin-retention-days = { $days ->
    [zero] لا أيام
    [one] يوم واحد
    [two] يومان
    [few] { $days } أيام
    [many] { $days } يومًا
   *[other] { $days } يوم
}
admin-retention-follows-default = الافتراضي: { $kept }
admin-retention-hold-on = حجز قانوني
admin-retention-read-only = يتطلب تغيير الاحتفاظ إذن settings.write.
admin-retention-edit-for = الاحتفاظ لـ { $category }
admin-retention-keep = الاحتفاظ
admin-retention-mode-inherit = اتباع الافتراضي
admin-retention-mode-forever = كل الإدخالات
admin-retention-mode-days = لعدد من الأيام
admin-retention-days-label = الأيام
admin-retention-hold = حجز قانوني
admin-retention-save = حفظ
admin-retention-bounds = أدخل عددًا صحيحًا من { $min } إلى { $max }.
admin-retention-saved = حُفظ الاحتفاظ لـ { $category }.
admin-retention-task-none = لا توجد مهمة تنظيف بعد. تُنشأ يوميًا في 03:15 بالتوقيت العالمي المنسّق عند حفظ عدد الأيام لأول مرة.
admin-retention-task-disabled = مهمة التنظيف المجدولة معطّلة؛ لا تُنظَّف الإدخالات إلا بـ «تنظيف الآن».
admin-retention-task-next = التنظيف المجدول التالي: { $when }.
admin-retention-task-daily = يُنظَّف يوميًا في 03:15 بالتوقيت العالمي المنسّق.
admin-retention-last = آخر تنظيف أزال إدخالات: { $when }.
admin-retention-prune = تنظيف الآن
admin-retention-prune-title = تنظيف سجل التدقيق الآن؟
admin-retention-prune-body = تُحذف الإدخالات الأقدم من مدة احتفاظ فئتها. لا تتغير الفئات التي تحتفظ بكل الإدخالات أو الخاضعة لحجز قانوني. لا يمكن التراجع عن هذا.
admin-retention-pruned = { $count ->
    [0] لم يكن أي إدخال قديمًا بما يكفي للإزالة.
    [zero] لم يكن أي إدخال قديمًا بما يكفي للإزالة.
    [one] أُزيل إدخال واحد.
    [two] أُزيل إدخالان.
    [few] أُزيلت { $count } إدخالات.
    [many] أُزيل { $count } إدخالًا.
   *[other] أُزيل { $count } إدخال.
}
admin-auth-attempted = حُوولت المحاولة باسم { $name }
admin-features-feature-compose-open-tracking = تتبع فتح الرسائل
admin-features-feature-compose-open-tracking-help = السماح بتسجيل فتح الرسائل المنسّقة المؤهلة باستخدام صورة التتبع الخاصة بالخادم. يتطلب عنوان URL عامًا مضبوطًا.
admin-features-feature-reading-original-html = عرض HTML الأصلي
admin-features-feature-reading-original-html-help = السماح للمستخدمين باختيار HTML الأصلي لرسائل فردية، داخل إطار القراءة المعزول. يبقى HTML المنقّى هو الافتراضي.
admin-features-inherit = وراثة
admin-features-override = تعيين قيمة
admin-features-setting-mode = وضع القيمة لـ { $setting }
admin-features-inherit-help = تزيل الوراثة قيمة ملف التعريف هذا وقفله. يحسم ملف التعريف التالي المنطبق أو النطاق أو قيمة الخادم؛ وتعرض المعاينة الفعّالة النتيجة ومصدرها.
admin-features-domain-alias = { $alias } (اسم مستعار لـ { $domain })
admin-features-domain-aliases = { $domain } (الأسماء المستعارة: { $aliases })
