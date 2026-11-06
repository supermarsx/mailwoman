# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = صفحات التشخيص
admin-diagx-tab-overview = نظرة عامة
admin-diagx-tab-server = الخادم
admin-diagx-tab-connectivity = الاتصال
admin-diagx-tab-backends = الواجهات الخلفية للبريد
admin-diagx-tab-posture = الوضع الأمني
admin-diagx-tab-tools = الأدوات
admin-diagx-tab-history = السجل
admin-diagx-tab-report = التقرير
admin-diagx-tab-server-intro = فحوصات لأجزاء هذا الخادم نفسه: قاعدة البيانات وترحيلاتها ومفتاح الخادم. تعمل مع فحوصات الخادم.
admin-diagx-tab-connectivity-intro = هل يمكن الوصول إلى هذا الخادم باسمه العام، وهل يستطيع الوصول إلى الواجهات الخلفية المحفوظة، وهل يُبقي العناوين الخاصة بعيدة المنال. تبحث أدوات النطاق أدناه عن سجلات DNS وتختبر TLS وتكتشف الإعدادات لنطاق واحد.
admin-diagx-tab-backends-intro = مصافحة اتصال وأمان مع كل نقطة نهاية محفوظة للبريد والإرسال والقواعد وDAV. لا تُرسل بيانات اعتماد ولا يُسلَّم أي بريد.
admin-diagx-tab-posture-intro = الإعدادات التي تحمي تسجيل الدخول والجلسات، وكيف سيرى fail2ban عنوان الطلب.
admin-diagx-tab-tools-intro = أماكن أخرى لفحص الخدمة، والحدود التي تنطبق على عمليات التشخيص.
admin-diagx-tab-history-intro = التشغيلات المحفوظة، الأحدث أولًا. افتح تشغيلًا لرؤية نتائجه وما تغيّر منذ التشغيل الذي قبله.
admin-diagx-status-ok = سليم
admin-diagx-status-warn = تحذير
admin-diagx-status-fail = فشل
admin-diagx-status-skipped = تم تخطيه
admin-diagx-status-not-run = لم يُشغَّل
admin-diagx-status-running = قيد التشغيل
admin-diagx-run = تشغيل
admin-diagx-run-again = تشغيل مجددًا
admin-diagx-running = جارٍ التشغيل…
admin-diagx-run-server-note = يعيد تشغيل فحوصات الخادم. تتحدّث كل بطاقة من ذلك التشغيل.
admin-diagx-run-backends-note = يعيد فحص كل الواجهات الخلفية المحفوظة. تتحدّث كل بطاقة واجهة خلفية.
admin-diagx-how-to-fix = كيفية الإصلاح
admin-diagx-details = التفاصيل
admin-diagx-copy-details = نسخ التفاصيل
admin-diagx-copy-value = نسخ { $label }
admin-diagx-copied = تم النسخ.
admin-diagx-copy-failed = لم يسمح المتصفح بالنسخ. حدّد النص وانسخه يدويًا.
admin-diagx-not-run-yet = لم يُشغَّل بعد.
admin-diagx-not-reported = لم يُبلِّغ آخر تشغيل للخادم عن هذا الفحص؛ فهو لا ينطبق على هذا النشر.
admin-diagx-posture-met = مستوفى
admin-diagx-posture-not-met = غير مستوفى
admin-diagx-loading = جارٍ التحميل…
admin-diagx-none = لا شيء
admin-diagx-yes = نعم
admin-diagx-no = لا
admin-diagx-open = فتح
admin-diagx-row-result = النتيجة
admin-diagx-row-duration = المدة
admin-diagx-row-checked = وقت الفحص
admin-diagx-row-id = معرّف الفحص
admin-diagx-row-domain = النطاق
admin-diagx-row-backend = الواجهة الخلفية
admin-diagx-row-protocol = البروتوكول
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = القدرات
admin-diagx-row-credentials = بيانات الاعتماد المرسَلة
admin-diagx-row-source = وُجدت في
admin-diagx-row-oauth = يتطلب OAuth
admin-diagx-overview-intro = أحدث نتيجة لكل فحص، حسب المجموعة. يشغّل «تشغيل الكل» فحوصات الخادم ثم يفحص كل واجهة خلفية محفوظة.
admin-diagx-run-all = تشغيل كل الفحوصات
admin-diagx-run-all-step = الخطوة { $step } من { $steps }: { $suite ->
    [backends] الواجهات الخلفية المحفوظة
   *[server] فحوصات الخادم
}.
admin-diagx-run-all-done = انتهت كل الفحوصات. حُفظت النتائج في السجل.
admin-diagx-backends-progress = فُحصت { $completed } من { $total } واجهة خلفية.
admin-diagx-counts = { $ok } سليم · { $warn } تحذير · { $fail } فشل · { $notRun } لم يُشغَّل
admin-diagx-attention = يحتاج إلى اهتمام
admin-diagx-attention-none = لم يُبلِّغ أي فحص عن تحذير أو فشل في أحدث تشغيل له.
admin-diagx-never-run = لم يُشغَّل أي فحص بعد. شغّل كل الفحوصات للحصول على أول نتيجة.
admin-diagx-last-server = آخر فحوصات الخادم
admin-diagx-last-backends = آخر فحوصات الواجهات الخلفية
admin-diagx-never = أبدًا
admin-diagx-open-health = سلامة الخدمة
admin-diagx-check-database-connectivity = اتصال قاعدة البيانات
admin-diagx-check-database-connectivity-purpose = تجيب قاعدة البيانات عن استعلام بسيط.
admin-diagx-check-database-connectivity-fix = تحقق من أن قاعدة البيانات تعمل وأن MW_DB_PATH أو MW_DATABASE_URL يشير إليها. يذكر سجل الخادم خطأ الاتصال.
admin-diagx-check-database-write = الكتابة في قاعدة البيانات
admin-diagx-check-database-write-purpose = يمكن كتابة صف مؤقت ثم التراجع عنه.
admin-diagx-check-database-write-fix = تحقق من المساحة الحرة على القرص ومن إمكانية الكتابة في ملف قاعدة البيانات أو للمستخدم. يمنع وجود وحدة تخزين للقراءة فقط أو قرص ممتلئ الكتابة.
admin-diagx-check-database-migrations = ترحيلات قاعدة البيانات
admin-diagx-check-database-migrations-purpose = تحتوي قاعدة البيانات على كل ترحيل يتوقعه إصدار هذا الخادم.
admin-diagx-check-database-migrations-fix = أعد تشغيل الخادم ليطبّق الترحيلات المعلّقة، وراجع سجله بحثًا عن ترحيل فشل. لا تشغّل خادمًا أقدم على قاعدة بيانات أحدث.
admin-diagx-check-server-key-roundtrip = مفتاح الخادم
admin-diagx-check-server-key-roundtrip-purpose = يغلق مفتاح الخادم قيمة اختبار ويفتحها.
admin-diagx-check-server-key-roundtrip-fix = تحقق من أن MW_SERVER_KEY (أو ملف المفتاح) هو المفتاح الذي أُنشئت به قاعدة البيانات هذه. لا يمكن فتح الأسرار المخزّنة بمفتاح آخر.
admin-diagx-check-other-purpose = فحص أبلغ عنه هذا الخادم.
admin-diagx-check-other-fix = افتح «التفاصيل» لرؤية النتيجة الخام وراجع سجل الخادم وقت التشغيل.
admin-diagx-check-network-public-dns = الاسم العام
admin-diagx-check-network-public-dns-purpose = يُحلّ اسم المضيف في MW_PUBLIC_URL من هذا الخادم.
admin-diagx-check-network-public-dns-fix = تحقق من سجل DNS للاسم العام، أو اضبط MW_PUBLIC_URL على العنوان الذي يستخدمه الناس للوصول إلى هذا الخادم.
admin-diagx-check-network-public-port = المنفذ العام
admin-diagx-check-network-public-port-purpose = يقبل منفذ العنوان العام اتصالًا من هذا الخادم.
admin-diagx-check-network-public-port-fix = تحقق من جدار الحماية والوكيل العكسي وإعادة توجيه المنافذ. لا تستطيع بعض أجهزة التوجيه الاتصال بعنوانها العام (NAT hairpinning)؛ اختبر أيضًا من خارج شبكتك.
admin-diagx-check-network-backend-ports = منافذ الواجهات الخلفية
admin-diagx-check-network-backend-ports-purpose = يقبل مضيف ومنفذ كل واجهة خلفية محفوظة اتصالًا مباشرًا.
admin-diagx-check-network-backend-ports-fix = تحقق من أن مضيفي الواجهات الخلفية يعملون وأن جدار الحماية يسمح لهذا الخادم بالوصول إليهم. تعرض «الواجهات الخلفية للبريد» أي نقطة نهاية تفشل مصافحتها.
admin-diagx-check-egress-private-policy = حماية العناوين الخاصة
admin-diagx-check-egress-private-policy-purpose = افتراضيًا، تُرفض الاتصالات الصادرة إلى عناوين الحلقة الراجعة والعناوين الخاصة.
admin-diagx-check-egress-private-policy-fix = راجع إعدادات الاتصالات الصادرة ضمن الخصوصية. اسمح بالعناوين الخاصة فقط للواجهات الخلفية المسمّاة التي تحتاجها.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = تصل الطلبات إلى هذا الخادم عبر HTTPS.
admin-diagx-check-security-https-fix = قدّم وحدة التحكم عبر HTTPS، أو اضبط MW_PUBLIC_URL على https وMW_TRUSTED_PROXIES عندما ينهي وكيل عكسي TLS.
admin-diagx-check-security-secure-cookies = ملفات تعريف الارتباط الآمنة
admin-diagx-check-security-secure-cookies-purpose = تُرسل ملفات تعريف ارتباط الجلسة عبر HTTPS فقط.
admin-diagx-check-security-secure-cookies-fix = اضبط MW_COOKIE_SECURE على true بمجرد الوصول إلى الخادم عبر HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = تُبلَّغ المتصفحات باستخدام HTTPS لهذا الموقع.
admin-diagx-check-security-hsts-fix = فعّل Strict-Transport-Security في إعدادات الأمان، أو أرسله من الوكيل العكسي.
admin-diagx-check-security-csrf = حماية CSRF
admin-diagx-check-security-csrf-purpose = يجب أن تحمل الطلبات التي تغيّر البيانات رمز CSRF.
admin-diagx-check-security-csrf-fix = اضبط MW_CSRF_STRICT على true.
admin-diagx-check-security-admin-factors = العامل الثاني للمسؤولين
admin-diagx-check-security-admin-factors-purpose = لكل مسؤول عامل ثانٍ لتسجيل الدخول.
admin-diagx-check-security-admin-factors-fix = اطلب من كل مسؤول ليس لديه عامل ثانٍ إضافة مفتاح مرور أو تطبيق مصادقة في «تفضيلاتي»، أو اشترط ذلك في مركز الأمان.
admin-diagx-check-security-signin-pacing = تأخير إعادة محاولة تسجيل الدخول
admin-diagx-check-security-signin-pacing-purpose = يعقب فشل تسجيل الدخول انتظار قصير.
admin-diagx-check-security-signin-pacing-fix = فعّل تنظيم وتيرة تسجيل الدخول في سياسة الأمان.
admin-diagx-check-security-credential-locks = أقفال بيانات الاعتماد
admin-diagx-check-security-credential-locks-purpose = يؤدي تكرار فشل تسجيل الدخول إلى قفل بيانات الاعتماد لفترة.
admin-diagx-check-security-credential-locks-fix = فعّل الأقفال التلقائية في مركز الأمان.
admin-diagx-check-security-geography = بيانات جغرافية تسجيل الدخول
admin-diagx-check-security-geography-purpose = لدى سياسة جغرافية تسجيل الدخول بيانات البلدان التي تحتاجها.
admin-diagx-check-security-geography-fix = ثبّت قاعدة بيانات البلدان أو حدّثها، أو عطّل سياسة الجغرافيا.
admin-diagx-check-security-render-isolation = عرض الرسائل المعزول
admin-diagx-check-security-render-isolation-purpose = تُعرض الرسائل في عامل منفصل ومعزول.
admin-diagx-check-security-render-isolation-fix = تحقق من أن MW_RENDER_BIN يشير إلى برنامج mw-render وأنه يبدأ. تعرض سلامة الخدمة في لوحة المعلومات حالة العامل.
admin-diagx-check-security-deployment = إعدادات النشر
admin-diagx-check-security-deployment-purpose = يمكن قراءة إعدادات أمان النشر.
admin-diagx-check-security-deployment-fix = تحقق من اتصال قاعدة البيانات ثم افتح تقييم أمان النشر.
admin-diagx-check-security-plaintext-backends = كلمات المرور عبر اتصالات غير مشفّرة
admin-diagx-check-security-plaintext-backends-purpose = لا توجد واجهة خلفية محفوظة تقبل كلمات المرور بدون TLS.
admin-diagx-check-security-plaintext-backends-fix = حرّر اتصالات النطاق واختر TLS أو STARTTLS. اسمح بالاتصالات العادية فقط على شبكة خاصة تتحكم بها.
admin-diagx-check-fail2ban-client-address = عنوان عميل fail2ban
admin-diagx-check-fail2ban-client-address-purpose = العنوان الذي سيذكره سطر سجل fail2ban هو عنوان العميل الحقيقي.
admin-diagx-check-fail2ban-client-address-fix = خلف وكيل عكسي، اضبط MW_TRUSTED_PROXIES على عنوان الوكيل، وإلا حظر fail2ban الوكيل بدلًا من العميل. يُتوقع عنوان حلقة راجعة عند فتح وحدة التحكم على الخادم نفسه.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = نقطة نهاية { $protocol }: مصافحة اتصال وأمان دون تسجيل الدخول.
admin-diagx-backend-other-purpose = ملاحظة من تشغيل الواجهات الخلفية.
admin-diagx-backend-fix = تحقق من اسم المضيف والمنفذ ووضع TLS المحفوظة لهذه الواجهة الخلفية، ومن أن DNS يُحلّ، وأن الشهادة صالحة لاسم المضيف. ثم شغّل مجددًا.
admin-diagx-backend-config-fix = افتح اتصالات النطاق واحفظها مجددًا؛ تعذّرت قراءة قائمة الواجهات الخلفية المحفوظة.
admin-diagx-backend-results = فحوصات الواجهات الخلفية
admin-diagx-backends-not-run = لم تُفحص الواجهات الخلفية المحفوظة بعد.
admin-diagx-backends-run-at = آخر فحص { $date }: { $count } نتيجة.
admin-diagx-failover-title = التحويل عند الفشل
admin-diagx-failover-purpose = هل يجوز للنطاق التحويل إلى واجهة بريد خلفية أخرى عند تعذّر الوصول إلى خياره الأول.
admin-diagx-failover-state = التحويل عند الفشل { $state ->
    [on] مفعّل
   *[off] معطّل
} · { $backends } واجهة خلفية للبريد{ $enabled ->
    [no] {" "}· النطاق معطّل
   *[yes] {""}
}
admin-diagx-no-domains = لا توجد نطاقات محفوظة.
admin-diagx-domain-tools = أدوات النطاق
admin-diagx-domain-tools-purpose = فحوصات لمرة واحدة لنطاق محفوظ. لا تسجّل الدخول ولا ترسل بريدًا.
admin-diagx-domain = النطاق
admin-diagx-tool-dns = سجلات DNS
admin-diagx-tool-dns-purpose = تبحث عن سجلات MX وSPF وDMARC والخدمة (SRV) للنطاق.
admin-diagx-tool-dns-fix = أضف السجلات الناقصة لدى مزوّد DNS. بدون MX لا يستقبل النطاق بريدًا؛ وبدون SPF وDMARC قد ترفض خوادم أخرى بريده.
admin-diagx-srv-found = وُجد { $found } من { $total } من سجلات الخدمة
admin-diagx-tool-tls = TLS والمصافحة
admin-diagx-tool-tls-purpose = تتصل بكل نقطة نهاية محفوظة للنطاق وتبلغ عن إصدار TLS وانتهاء الشهادة والوقت المستغرق.
admin-diagx-tool-tls-fix = لنقطة نهاية يتعذّر الوصول إليها، تحقق من المضيف والمنفذ وجدار الحماية. جدّد الشهادة التي تنتهي خلال 14 يومًا.
admin-diagx-probe-none = لا توجد للنطاق نقاط نهاية محفوظة للاختبار.
admin-diagx-tool-detect = اكتشاف الإعدادات
admin-diagx-tool-detect-purpose = تبحث عن إعدادات البريد المنشورة (autoconfig وautodiscover وSRV) لاسم نطاق. لا يُحفظ شيء.
admin-diagx-tool-detect-fix = انشر سجلات autoconfig أو SRV للنطاق، أو أدخل الإعدادات يدويًا ضمن النطاقات.
admin-diagx-detect-name = اسم النطاق المراد البحث عنه
admin-diagx-tool-health = سلامة الخدمة
admin-diagx-tool-health-purpose = مزامنة البريد والصادر وفهرس البحث وعامل العرض والإضافات، في لوحة المعلومات.
admin-diagx-tool-deployment-purpose = كل إعداد لأمان النشر يمكن لهذا الخادم ملاحظته، مع الإعداد الذي يتحكم به.
admin-diagx-tool-ssh = اختبارات أنفاق SSH
admin-diagx-tool-ssh-purpose = اختبر أنفاق SSH المستخدمة للوصول إلى واجهات خلفية على شبكة أخرى.
admin-diagx-tool-geo = جغرافيا تسجيل الدخول
admin-diagx-tool-geo-purpose = من أين تأتي عمليات تسجيل الدخول، وقاعدة بيانات البلدان التي تقوم عليها سياسة الجغرافيا.
admin-diagx-catalogue-purpose = ما تغطيه كل مجموعة تشخيص، والحدود التي يلتزم بها كل تشغيل.
admin-diagx-suite-unsupported = غير متاح: { $reason }
admin-diagx-suite-server = فحوصات الخادم
admin-diagx-suite-backends = الواجهات الخلفية المحفوظة
admin-diagx-run-title = { $suite ->
    [backends] الواجهات الخلفية المحفوظة
   *[server] فحوصات الخادم
}، { $date }
admin-diagx-changes = التغييرات
admin-diagx-changes-first = لا يوجد تشغيل سابق من هذا النوع للمقارنة.
admin-diagx-changes-none = لم يتغير شيء منذ تشغيل { $date }.
admin-diagx-changes-since = ما تغيّر منذ تشغيل { $date }:
admin-diagx-change = { $before } ← { $after }
admin-diagx-report-title = تقرير استكشاف الأخطاء
admin-diagx-report-purpose = تقرير JSON للدعم: أعلام وقت التشغيل وأحدث النتائج والتشغيل الذي تختاره. لا تُضمَّن أبدًا كلمات المرور أو الرموز أو ملفات تعريف الارتباط أو المفاتيح أو الإعدادات الخام أو البريد.
admin-diagx-report-run = التشغيل المراد تضمينه
admin-diagx-report-preview = معاينة
admin-crumb-server = الخادم
admin-crumb-connectivity = الاتصال
admin-crumb-posture = الوضع الأمني
admin-crumb-tools = الأدوات
admin-crumb-history = السجل
admin-crumb-report = التقرير
admin-diagx-table-actions = الإجراءات
admin-diagx-check-pdfjs-viewer-assets = ملفات عارض PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = ملفات عارض PDF موجودة كلها وأنواعها صحيحة وتطابق إصدار هذا الخادم.
admin-diagx-check-pdfjs-viewer-assets-fix = أعد نشر بناء الويب، أو احذف مجلد pdfjs من MW_WEB_DIR ليُستخدم النسخة المضمّنة في الخادم. إذا كان الفحص يفشل بدل أن يحذّر، فأعد بناء صورة الخادم.
admin-diagx-probe-line = { $host } · يمكن الوصول إليه: { $reachable ->
    [yes] نعم
   *[no] لا
} · TLS { $tls } · الشهادة { $days ->
    [none] غير مُبلَّغ عنها
   *[other] تنتهي خلال { $days } يومًا
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
