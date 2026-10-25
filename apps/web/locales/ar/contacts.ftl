# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: contacts. Source: ../en/contacts.ftl

contacts-title = جهات الاتصال
contacts-search = البحث في جهات الاتصال
contacts-new = جهة اتصال جديدة
contacts-find-duplicates = البحث عن التكرارات
contacts-loading = جارٍ التحميل…
contacts-none = لا توجد جهات اتصال.
contacts-list = قائمة جهات الاتصال
contacts-favorite = تفضيل { $name }
contacts-export-vcard = تصدير vCard
contacts-export-csv = تصدير CSV
contacts-select-hint = اختر جهة اتصال لعرض بطاقتها.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = دفاتر العناوين والمجموعات
contacts-address-books = دفاتر العناوين
contacts-all = كل جهات الاتصال
contacts-favorites = ★ المفضلة
contacts-groups = المجموعات
contacts-new-group = مجموعة جديدة
contacts-new-group-name = اسم المجموعة الجديدة
contacts-group-name-placeholder = اسم المجموعة
contacts-create = إنشاء
contacts-data = البيانات
contacts-import = استيراد…

# -- Business card -----------------------------------------------------------
contacts-card = جهة الاتصال { $name }
contacts-favorited = ★ في المفضلة
contacts-favorite-action = ☆ تفضيل
contacts-email = البريد الإلكتروني
contacts-phone = الهاتف
contacts-dates = التواريخ
contacts-groups-label = المجموعات
contacts-notes = ملاحظات
contacts-security-key = مفتاح الأمان
contacts-key-on-file = يوجد مفتاح/شهادة محفوظة (للعرض فقط حتى يتوفر دعم PGP/S-MIME).
contacts-no-key = لا يوجد مفتاح محفوظ.
contacts-directory-security = أمان الدليل
contacts-member-of = عضو في
contacts-group-membership = عضوية { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = جهة اتصال جديدة
contacts-edit-contact = تعديل جهة الاتصال
contacts-full-name = الاسم الكامل
contacts-given = الاسم الأول
contacts-given-placeholder = الاسم الأول
contacts-surname = اسم العائلة
contacts-surname-placeholder = اسم العائلة
contacts-organization = المؤسسة
contacts-job-title = المسمى الوظيفي
contacts-job-title-placeholder = المسمى
contacts-email-n = البريد الإلكتروني { $n }
contacts-email-label = تسمية البريد الإلكتروني { $n }
contacts-email-label-placeholder = العمل
contacts-remove-email = إزالة البريد الإلكتروني { $n }
contacts-add-email = إضافة بريد إلكتروني
contacts-phone-n = الهاتف { $n }
contacts-remove-phone = إزالة الهاتف { $n }
contacts-add-phone = إضافة هاتف
contacts-security-key-label = مفتاح الأمان (عنصر نائب غير مقروء)
contacts-security-key-placeholder = مفتاح PGP / شهادة S-MIME (للعرض فقط)
contacts-favorite-word = مفضلة

# -- Import dialog -----------------------------------------------------------
contacts-import-title = استيراد جهات الاتصال
contacts-imported = استُورد { $count ->
    [zero] لا شيء من جهات الاتصال
    [one] جهة اتصال واحدة
    [two] جهتا اتصال
    [few] { $count } جهات اتصال
    [many] { $count } جهة اتصال
   *[other] { $count } جهة اتصال
}.
contacts-import-hint = الصق vCard (.vcf) أو CSV، أو اختر ملفًا. يُكتشف التنسيق تلقائيًا.
contacts-import-file = استيراد ملف
contacts-paste = لصق vCard أو CSV
contacts-preview = معاينة
contacts-map-columns = ربط الأعمدة
contacts-col = العمود
contacts-maps-to = يُربط بـ
contacts-sample = عيّنة
contacts-map-column = ربط العمود { $header }
contacts-preview-count = معاينة ({ $count })
contacts-import-preview = معاينة الاستيراد
contacts-no-name = (بلا اسم)
contacts-no-email = بلا بريد إلكتروني
contacts-import-n = استيراد { $count ->
    [zero] لا شيء من جهات الاتصال
    [one] جهة اتصال واحدة
    [two] جهتَي اتصال
    [few] { $count } جهات اتصال
    [many] { $count } جهة اتصال
   *[other] { $count } جهة اتصال
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = دمج التكرارات
contacts-merge-review-hint = راجع البطاقة المدمجة. تصبح البطاقات الأخرى محذوفة مع إمكانية الاستعادة.
contacts-merged-preview = معاينة الدمج
contacts-emails = عناوين البريد الإلكتروني
contacts-phones = الهواتف
contacts-merge-confirm = دمج جهات الاتصال
contacts-no-duplicates = لم يُعثر على تكرارات.
contacts-possible-duplicates = تكرارات محتملة: { $count }
contacts-review-merge = مراجعة الدمج
