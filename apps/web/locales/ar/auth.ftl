# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-email = عنوان البريد الإلكتروني
auth-password = كلمة المرور
auth-sign-in = تسجيل الدخول
auth-signing-in = جارٍ تسجيل الدخول…
auth-invalid-credentials = بيانات الاعتماد غير صحيحة
auth-refused-note = إذا كان عنوان البريد الإلكتروني وكلمة المرور صحيحين، فربما عطّل أحد المسؤولين الحساب. لا يمكن للحساب المعطّل تسجيل الدخول حتى يعيد أحد المسؤولين تفعيله.
auth-unreachable = تعذّر الوصول إلى الخادم
auth-address-not-configured = هذا العنوان غير مُعدّ على هذا الخادم. تواصل مع المسؤول.

# -- Single sign-on (t9) -----------------------------------------------------
auth-sso-heading = تسجيل الدخول الموحّد
auth-sso-divider = أو المتابعة باستخدام
auth-sso-button = تسجيل الدخول باستخدام { $name }
auth-sso-error = لم يكتمل تسجيل الدخول الموحّد. يُرجى المحاولة مرة أخرى أو تسجيل الدخول بكلمة المرور.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = تفويض التطبيق
auth-consent-title = تفويض الوصول
auth-consent-loading = جارٍ تحميل الطلب…
auth-consent-intro = يريد الوصول إلى حسابك.
auth-consent-approved = عميل معتمد من المسؤول
auth-consent-unapproved = عميل غير معروف — غير معتمد من المسؤول
auth-consent-requesting = يطلب
auth-consent-redirects-to = يعيد التوجيه إلى
auth-consent-for-resource = للمورد
auth-consent-deny = رفض
auth-consent-allow = سماح
auth-consent-error = تعذّر تسجيل قرارك

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = محاولات تسجيل الدخول كثيرة جدًا. يُرجى الانتظار قبل المحاولة مجددًا.
auth-retry-countdown = أعد المحاولة بعد { $seconds } ثانية
auth-upstream-protocol = توقّف تسجيل الدخول: أرسل خادم البريد ردًا لم يتمكن هذا الخادم من قراءته. أعد المحاولة لاحقًا أو أبلغ المسؤول.
auth-account-disabled = عطّل المسؤول هذا الحساب، ولا يمكنه تسجيل الدخول حتى يعيد المسؤول تفعيله.
auth-signin-not-allowed = لا يجوز لهذا العنوان تسجيل الدخول إلى هذا الخادم. تواصل مع المسؤول.
auth-signin-not-provisioned = لا يوجد حساب لهذا العنوان بعد، ولا تُنشأ الحسابات الجديدة تلقائيًا. اطلب من المسؤول إضافتك.
auth-reference = المرجع: { $id }
auth-domain-hint = يمكنك تسجيل الدخول باسم المستخدم فقط للنطاق { $domain }.
auth-passkey-sign-in = تسجيل الدخول بمفتاح المرور
auth-passkey-failed = تعذّر تسجيل دخولك بمفتاح المرور. أعد المحاولة أو استخدم كلمة المرور.
auth-remembered = عمليات تسجيل الدخول السابقة
auth-remembered-help = العناوين التي استُخدمت بنجاح على هذا الجهاز. يؤدي اختيار أحدها إلى ملء العنوان فقط.
auth-forget-address = نسيان { $address } على هذا الجهاز
auth-login-timeout = انتهت مهلة تسجيل الدخول. لم تُجرَ أي إعادة محاولة تلقائية. تحقق من الاتصال وأعد المحاولة عندما تكون مستعدًا.
auth-verification-title = التحقق من تسجيل الدخول
auth-verification-loading = جارٍ تحميل التحقق من تسجيل الدخول…
auth-verification-error = تعذّر إكمال التحقق. أعد التحقق قبل تسجيل الدخول.
auth-verification-required = أكمل تحققًا جديدًا من تسجيل الدخول قبل المحاولة مجددًا.
auth-identity-title = مشاركة هويتك
auth-identity-sign-in = سجّل الدخول مجددًا لمتابعة طلب الهوية هذا.
auth-identity-sign-out = تسجيل الخروج من Mailwoman
auth-identity-expired = انتهت صلاحية طلب الهوية هذا أو تغيّر. ارجع إلى التطبيق الطالب وابدأ من جديد.
auth-identity-retry = إعادة المحاولة
auth-identity-disclosure = يتلقى هذا التطبيق معلومات الهوية الواردة أدناه. لا يمنح هذا الإذن إمكانية الوصول إلى بريدك.
auth-identity-scope-openid = معرّف هوية ثابت
auth-identity-scope-profile = معلومات الملف الشخصي المتاحة
auth-identity-scope-email = عنوان بريدك الإلكتروني الذي سجّلت به الدخول، دون ادعاء التحقق منه بصورة مستقلة
auth-identity-scope-offline_access = تجديد الوصول إلى الهوية ما دامت جلسة Mailwoman هذه سارية، لمدة تصل إلى 15 يومًا من تسجيل الدخول
auth-identity-deny = إلغاء
auth-identity-approve = السماح بالوصول إلى الهوية
auth-identity-grants = التطبيقات التي تستخدم هويتك
auth-identity-grants-help = إلغاء الوصول إلى الهوية لأحد التطبيقات. لا يزيل ذلك أذونات واجهة برمجة البريد المنفصلة.
auth-identity-no-grants = لا توجد حاليًا تطبيقات لديها موافقة على الهوية.
auth-identity-revoke = إلغاء الوصول إلى الهوية
auth-maintenance = البريد عبر الويب متوقف مؤقتًا للصيانة. يُرجى المحاولة لاحقًا.
