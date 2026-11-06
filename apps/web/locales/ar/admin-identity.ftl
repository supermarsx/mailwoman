# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = مزوّد الهوية
admin-identity-description = اسمح للتطبيقات المعتمدة بتسجيل الدخول عبر Mailwoman. هذا منفصل عن مزوّدي تسجيل الدخول الخارجيين وعن الوصول إلى واجهة API للبريد. يلزم رمز التفويض مع PKCE؛ وأنواع المنح الأخرى غير متاحة.
admin-identity-configure = ضبط المُصدِر
admin-identity-new = إضافة عميل هوية
admin-identity-edit = تعديل العميل
admin-identity-enabled = مفعّل
admin-identity-disabled = معطّل
admin-identity-all = الكل
admin-identity-discovery = فتح مستند الاكتشاف
admin-identity-search = البحث في العملاء
admin-identity-status = الحالة
admin-identity-clients = عملاء الهوية
admin-identity-client = هوية العميل
admin-identity-client-id = معرّف العميل
admin-identity-name = الاسم
admin-identity-method = مصادقة العميل
admin-identity-public = عميل عام (PKCE، بلا سر)
admin-identity-pkce = يجب أن يستخدم كل عميل PKCE S256. تُولَّد أسرار العملاء السرّيين هنا وتُعرض مرة واحدة.
admin-identity-rotate-secret = استبدال سر العميل الحالي
admin-identity-redirects = الوجهات المسجَّلة
admin-identity-redirect-help = عنوان HTTPS URI دقيق واحد في كل سطر؛ ويُسمح بـ HTTP للحلقة الراجعة الرقمية للتطبيقات المحلية. تُرفض أحرف البدل والأجزاء.
admin-identity-callbacks = عناوين URI لاستدعاء التفويض
admin-identity-logout-uris = عناوين URI لإعادة التوجيه بعد تسجيل الخروج
admin-identity-scopes = نطاقات الهوية المسموح بها
admin-identity-actions = الإجراءات
admin-identity-delete = حذف العميل
admin-identity-delete-confirm = حذف وصول الهوية لـ { $name }؟ تتوقف منح الهوية الحالية عن العمل ويُزال تسجيل العميل المخصص للهوية فقط.
admin-identity-empty = لا يوجد عملاء هوية مطابقون.
admin-identity-cancel = إلغاء
admin-identity-save = حفظ
admin-identity-done = تم
admin-identity-secret = سر العميل المُولَّد
admin-identity-secret-once = انسخ هذا السر إلى التطبيق المعتمِد الآن. لا يمكن عرضه مرة أخرى. لا تضع سرًا سريًّا في شيفرة المتصفح أبدًا.
admin-identity-issuer = عنوان URL العام لمُصدِر HTTPS
admin-identity-issuer-help = استخدم أصل HTTPS الدقيق لهذا النشر ومسار /oidc، بما في ذلك أي بادئة للنشر. يؤدي تغيير المُصدِر أو الإعدادات إلى إلغاء منح الهوية الحالية.
admin-identity-rotate-key = تدوير مفتاح التوقيع
admin-identity-key-help = يتطلب التدوير تحققًا جديدًا من المسؤول. تبقى المفاتيح العامة السابقة متاحة 15 دقيقة ليتمكن الرموز الصادرة حديثًا من إتمام عمرها القصير.
