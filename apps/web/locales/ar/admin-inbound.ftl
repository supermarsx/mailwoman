# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = الوصول الوارد
admin-inbound-help = قيّد عناوين IP المصدر الموثوقة لوحدة تحكم المسؤول والبريد عبر الويب كلًّا على حدة. تبدأ السياستان معطّلتين. لا تغيّر هذه القواعد الوكلاء الموثوقين أو نطاقات API أو الوصول الصادر.
admin-inbound-unscoped = تتطلب هذه السياسة على مستوى النشر مسؤولًا لديه وصول أمني غير مقيّد بنطاق.
admin-inbound-admin = وحدة تحكم المسؤول وAPI
admin-inbound-admin-help = تشمل تسجيل الدخول وهذا المحرر. لا تتجاوز مصادقة المسؤول الحالية سياسة الشبكة.
admin-inbound-webmail = البريد عبر الويب وواجهات API للحساب
admin-inbound-webmail-help = تغطي مسارات البريد وDAV والمحتوى الموقَّع وSSO وOIDC وAPI. تحتفظ الاتصالات المرقّاة الحالية بقرار المصافحة الخاص بها.
admin-inbound-enabled = حصر هذه الواجهة في قائمة السماح
admin-inbound-cidrs = عناوين IP وCIDR للإنقاذ
admin-inbound-cidrs-help = عنوان IP حرفي واحد أو CIDR واحد في كل سطر، حتى 128. يلزم واحد على الأقل عند التفعيل. ضمّن مصدرًا موثوقًا يمكنك استخدامه للاسترداد.
admin-inbound-hostnames = أسماء مضيفي DNS المسموح بها
admin-inbound-hostnames-help = اسم DNS واحد بأحرف ASCII صغيرة في كل سطر. استخدم IDNA ASCII للأسماء الدولية. لا عناوين URL ولا منافذ ولا أحرف بدل ولا عناوين IP حرفية ولا نقاط ختامية. لا تمنح الأسماء سوى العناوين المحلولة حاليًا.
admin-inbound-dns = حدود حل DNS
admin-inbound-dns-help = الأسماء الجديدة والمنتهية والفاشلة لا تمنح أي وصول. تلتزم الإجابات الناجحة بـ TTL الخاص بـ DNS وبحد ذاكرة التخزين المؤقت المضبوط. لا تنتظر الطلبات حل DNS أبدًا.
admin-inbound-servers = خوادم DNS التكرارية
admin-inbound-servers-help = اتركه فارغًا لاستخدام محللات النظام، أو أدخل حتى أربعة مقابس IP:المنفذ، واحدًا في كل سطر (مثل 10.0.0.53:53 أو [fd00::53]:53). تحل الخوادم الصريحة محل محللات النظام دون بديل عام. الثقة بالمحلل مسؤوليتك؛ ولا يُدّعى تحقق DNSSEC مستقل.
admin-inbound-dns-timeoutMs = مهلة البحث (مللي ثانية)
admin-inbound-dns-cacheSeconds = أقصى مدة للتخزين المؤقت (ثوانٍ)
admin-inbound-dns-maxHosts = أقصى عدد من أسماء المضيفين المختلفة
admin-inbound-dns-maxAddresses = أقصى عدد من العناوين لكل اسم مضيف
admin-inbound-dns-concurrency = عمليات البحث المتزامنة
admin-inbound-bounds = النطاق المسموح: { $min }–{ $max }.
admin-inbound-lockout = قد تحجب هذه السياسة وصولك أنت فورًا، بما في ذلك هذا المحرر. تأكد من أن عناوين IP للإنقاذ تغطي مصدرك الفعلي كما تحدده الوكلاء الموثوقون المضبوطون. لا يوجد تجاوز تلقائي للحلقة الراجعة أو للمسؤول المسجَّل دخوله. يتطلب الاسترداد من الإقفال استعادة من جانب النشر؛ ولا يتجاوزه تسجيل الدخول العادي.
admin-inbound-confirm-title = تطبيق سياسة الوصول الوارد؟
admin-inbound-confirm-word = APPLY
admin-inbound-save = تطبيق السياسة المؤكَّدة
admin-inbound-review = مراجعة التغييرات
admin-inbound-refresh = تحديث السياسة المحفوظة وحالة DNS
admin-inbound-reset = إعادة المسودة إلى السياسة المحمَّلة
admin-inbound-saved = قبل الخادم سياسة الوصول الوارد.
admin-inbound-conflict = تغيّرت السياسة المحفوظة أثناء التحرير. أعد ضبط المسودة وحدّث لتحميل مراجعتها الحالية قبل التحرير مرة أخرى.
admin-inbound-no-revision = لم يُرجع الخادم مراجعة للسياسة. حدّث قبل الحفظ.
admin-inbound-status = منح DNS الحالية
admin-inbound-status-help = هذه لقطة من آخر تحديث للحالة، وليست عدًّا تنازليًا ولا فحصًا للاتصال. حدّث لرؤية المنح الحالية. قد يتزامن خطأ مع إجابة سابقة لا تزال صالحة إلى حين انتهاء صلاحيتها الأصلية.
admin-inbound-refreshing = تحديث DNS محدود قيد التنفيذ.
admin-inbound-idle = لم يكن أي تحديث DNS قيد التشغيل عند آخر قراءة للحالة.
admin-inbound-no-hosts = لا توجد أسماء مضيفين نشطة في السياسات المفعّلة.
admin-inbound-hostname = اسم المضيف
admin-inbound-state = منح الوصول
admin-inbound-addresses = العناوين القابلة للاستخدام
admin-inbound-validity = الصلاحية عند آخر تحديث
admin-inbound-error = رمز نتيجة DNS
admin-inbound-usable = قابل للاستخدام
admin-inbound-unavailable = لا يوجد منح حالي
admin-inbound-seconds = { $seconds } ثانية
admin-inbound-on = مفعّل
admin-inbound-off = معطّل
admin-inbound-invalid-dns = حدود DNS أو عدد المحللات خارج الحدود المسموح بها.
admin-inbound-rescue-required = تتطلب كل واجهة مفعّلة عنوان IP أو CIDR واحدًا على الأقل للإنقاذ.
admin-inbound-too-many = تتجاوز السياسة حد أسماء المضيفين أو العناوين الحرفية المضبوط.
admin-inbound-invalid-host = استخدم أسماء DNS بأحرف ASCII صغيرة دون عناوين URL أو منافذ أو أحرف بدل أو عناوين IP حرفية أو نقاط ختامية.
