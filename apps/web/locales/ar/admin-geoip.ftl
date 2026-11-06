# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-geoip. Source: ../en/admin-geoip.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-geoip-title = قاعدة بيانات الدول
admin-geoip-intro = تستخدم سياسات الدول وتسمياتها ملف قاعدة بيانات محليًا واحدًا. ينزّله الخادم من المصدر المختار أدناه؛ ولا تُرسل العناوين أبدًا إلى خدمة تحديد المواقع.
admin-geoip-refresh = تحديث
admin-geoip-loading = جارٍ تحميل حالة قاعدة بيانات الدول…
admin-geoip-status = حالة قاعدة بيانات الدول
admin-geoip-field = الحقل
admin-geoip-value = القيمة
admin-geoip-field-state = قاعدة البيانات
admin-geoip-field-managed = تديرها
admin-geoip-field-source = المصدر
admin-geoip-field-type = نوع قاعدة البيانات
admin-geoip-field-built = تاريخ البناء
admin-geoip-field-size = حجم الملف
admin-geoip-field-networks = الشبكات التي لها دولة
admin-geoip-field-verified = التحقق
admin-geoip-field-installed = تاريخ التثبيت
admin-geoip-field-last-check = آخر فحص
admin-geoip-field-next-check = الفحص التالي
admin-geoip-field-last-error = آخر خطأ
admin-geoip-field-directory = دليل البيانات
admin-geoip-state-loaded = محمّلة
admin-geoip-state-missing = مفقودة
admin-geoip-state-invalid = غير صالحة
admin-geoip-managed-console = هذه الوحدة
admin-geoip-managed-environment = متغير البيئة MW_GEOIP_DB
admin-geoip-source-upload = ملف مرفوع
admin-geoip-source-seed = ملف مثبّت عند النشر
admin-geoip-source-environment = ملف MW_GEOIP_DB
admin-geoip-source-unknown = غير معروف
admin-geoip-verified-checksum = المجموع الاختباري من المزوّد وبنية الملف
admin-geoip-verified-structure = بنية الملف وعملية بحث تجريبية
admin-geoip-never = أبدًا
admin-geoip-next-after-download = بعد أول تنزيل
admin-geoip-auto-off = التحديثات التلقائية متوقفة
admin-geoip-no-directory = لا يوجد
admin-geoip-refusing = لا توجد قاعدة بيانات دول صالحة للاستخدام محمّلة. تمنع سياسة الدول المفعّلة تسجيل الدخول في هذه الحالة، حتى لو كانت العناوين غير المعيّنة مسموحًا بها. نزّل قاعدة بيانات أو ارفعها أدناه.
admin-geoip-missing-inactive = لم تُحمَّل قاعدة بيانات دول. لا توجد سياسة دول مفعّلة، لذا لا يتأثر تسجيل الدخول.
admin-geoip-invalid-reason = السبب: { $reason }
admin-geoip-stale = بُنيت قاعدة البيانات قبل أكثر من { $days } يومًا. ابحث عن تحديثات حتى تعتمد قرارات الدول على بيانات حديثة.
admin-geoip-environment = تم ضبط MW_GEOIP_DB على الخادم، لذا تدير البيئة قاعدة البيانات وهذه الصفحة للقراءة فقط. لإدارة قاعدة البيانات هنا، أزل MW_GEOIP_DB وأعد تشغيل الخادم.
admin-geoip-no-directory-help = ليس للخادم دليل بيانات لقاعدة البيانات. اضبط MW_GEOIP_DIR (مطلوب مع قاعدة بيانات PostgreSQL) وأعد تشغيل الخادم.
admin-geoip-attribution-active = البيانات المستخدمة:
admin-geoip-actions = إجراءات قاعدة البيانات
admin-geoip-download = تنزيل الآن
admin-geoip-downloading = جارٍ التنزيل…
admin-geoip-check = البحث عن تحديثات
admin-geoip-checking = جارٍ الفحص…
admin-geoip-rollback = تراجع
admin-geoip-rolled-back = عادت قاعدة البيانات السابقة إلى العمل.
admin-geoip-installed = ثُبّتت قاعدة البيانات الجديدة وهي قيد الاستخدام.
admin-geoip-up-to-date = قاعدة البيانات المثبّتة هي أحدث إصدار لدى المزوّد.
admin-geoip-previous = نسخة سابقة محفوظة للتراجع: { $source }، بُنيت في { $built }.
admin-geoip-upload-title = رفع قاعدة بيانات
admin-geoip-upload-help = ملف MaxMind DB ‏(.mmdb) أو نسخة gzip منه، يحتوي على رموز الدول. يُفحص الملف قبل أن يحل محل قاعدة البيانات الحالية. يطلب الرفع منك تأكيد هويتك.
admin-geoip-upload-file = ملف قاعدة البيانات
admin-geoip-upload = رفع ملف .mmdb
admin-geoip-uploading = جارٍ الرفع…
admin-geoip-uploaded = ثُبّتت قاعدة البيانات المرفوعة وهي قيد الاستخدام.
admin-geoip-source-title = المصدر والتحديثات
admin-geoip-source-help = مصدر التنزيلات. تستخدم التنزيلات HTTPS عبر سياسة الاتصالات الصادرة للخادم.
admin-geoip-provider = المزوّد
admin-geoip-licence = الترخيص: { $licence }.
admin-geoip-licence-terms = شروط الترخيص
admin-geoip-attribution-required = الإسناد المطلوب:
admin-geoip-custom-licence = تحقق من أن ترخيص مصدرك يسمح بهذا الاستخدام وأضف أي إسناد يتطلبه.
admin-geoip-licence-key = مفتاح ترخيص MaxMind
admin-geoip-token = رمز وصول IPinfo
admin-geoip-key-stored = المفتاح مخزَّن. لا يُعرض أبدًا؛ أدخل مفتاحًا جديدًا لاستبداله.
admin-geoip-key-needed = مطلوب لهذا المزوّد. يُخزَّن مشفّرًا ولا يُعرض مرة أخرى.
admin-geoip-key-clear = إزالة المفتاح المخزَّن
admin-geoip-custom-url = رابط التنزيل
admin-geoip-custom-url-help = عنوان HTTPS لملف .mmdb أو .mmdb.gz أو .tar.gz.
admin-geoip-custom-checksum = رابط المجموع الاختباري (اختياري)
admin-geoip-custom-checksum-help = عنوان HTTPS لملف تكون كلمته الأولى هي SHA-256 للتنزيل.
admin-geoip-custom-hosts = المضيفون المسموح بإعادة التوجيه إليهم (اختياري)
admin-geoip-custom-hosts-help = أسماء المضيفين التي يجوز أن يعيد التنزيل توجيهه إليها إضافةً إلى مضيفه. واحد في كل سطر.
admin-geoip-custom-ca = شهادة CA إضافية (اختياري)
admin-geoip-custom-ca-help = شهادة PEM لجهة CA خاصة وقّعت شهادة HTTPS للمرآة. اتركه فارغًا للإبقاء على الشهادة المخزَّنة.
admin-geoip-custom-ca-stored = شهادة CA مخزَّنة. الصق شهادة جديدة لاستبدالها.
admin-geoip-auto = التحديث تلقائيًا
admin-geoip-interval = الفحص كل
admin-geoip-interval-days = { $count ->
    [zero] { $count } يوم
    [one] يوم واحد
    [two] يومان
    [few] { $count } أيام
    [many] { $count } يومًا
   *[other] { $count } يوم
}
admin-geoip-interval-help = تبدأ الفحوصات المجدولة بعد أول تنزيل من هذا المزوّد. يُعاد الفحص الفاشل بعد ساعة واحدة، ثم ساعتين و4 ساعات وصولًا إلى 24 ساعة.
admin-geoip-step-up = يطلب حفظ تغيير المصدر منك تأكيد هويتك.
admin-geoip-save = حفظ المصدر
admin-geoip-saving = جارٍ الحفظ…
admin-geoip-saved = تم الحفظ.
admin-geoip-lookup-title = بحث تجريبي
admin-geoip-lookup-help = الدولة التي تعيّنها قاعدة البيانات المستخدمة لعنوان ما. لا يُخزَّن العنوان.
admin-geoip-lookup-ip = عنوان IP
admin-geoip-lookup = بحث
admin-geoip-lookup-country = العنوان { $ip } في { $country }.
admin-geoip-lookup-unknown = لا تحتوي قاعدة البيانات على دولة للعنوان { $ip }.
admin-geoip-lookup-unavailable = لم تُحمَّل قاعدة بيانات دول.
admin-geoip-progress-checking = جارٍ فحص مصدر قاعدة بيانات الدول…
admin-geoip-progress-downloading = جارٍ تنزيل قاعدة بيانات الدول…
admin-geoip-progress-verifying = جارٍ التحقق من قاعدة بيانات الدول…
admin-geoip-progress-installing = جارٍ تثبيت قاعدة بيانات الدول…
admin-geoip-progress-total = { $phase } { $received } من { $total }
admin-geoip-progress-bytes = { $phase } تم استلام { $received }
admin-geoip-features-warning-title = ميزات الدول مفعّلة لكن لم تُحمَّل قاعدة بيانات دول
admin-geoip-features-warning-help = لا تستطيع هذه الميزات معرفة مصدر الطلب، لذا لا تفعل شيئًا أو تتبع إعداد "عند تعذّر معرفة الدولة". ثبّت قاعدة بيانات لتعمل.
admin-geoip-features-install = الانتقال إلى قاعدة بيانات الدول
admin-geoip-feature-signinGeography = سياسة دولة تسجيل الدخول على مستوى النسخة
admin-geoip-feature-accountGeography = سياسة دولة تسجيل الدخول لكل حساب
admin-geoip-feature-sessionGeography = فحص الدولة للطلبات المسجَّل دخولها
admin-geoip-feature-adminDiscovery = حظر وحدة المسؤول عند تعذّر معرفة الدولة
admin-geoip-feature-sessionCountryAlerts = تنبيهات تغيّر دولة الجلسة
admin-geoip-features-status-loaded = قاعدة بيانات الدول محمّلة. بُنيت في { $date }، قبل { $age } يومًا.
admin-geoip-features-status-none = لم تُحمَّل قاعدة بيانات دول.
admin-geoip-features-status-stale = قديمة.
