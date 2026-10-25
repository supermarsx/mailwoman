# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = تغيير كلمة المرور
passwd-heading = تغيير كلمة المرور

# -- Forced change -----------------------------------------------------------
passwd-force-change = يطلب منك المسؤول تغيير كلمة المرور قبل المتابعة.

# -- Fields ------------------------------------------------------------------
passwd-current-label = كلمة المرور الحالية
passwd-new-label = كلمة المرور الجديدة
passwd-confirm-label = تأكيد كلمة المرور الجديدة

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = كلمتا المرور الجديدتان متطابقتان.
passwd-match-no = كلمتا المرور الجديدتان غير متطابقتين بعد.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = { $count ->
    [zero] { $count } حرف على الأقل
    [one] حرف واحد على الأقل
    [two] حرفين على الأقل
    [few] { $count } أحرف على الأقل
    [many] { $count } حرفًا على الأقل
   *[other] { $count } حرف على الأقل
}
passwd-rule-uppercase = حرف كبير
passwd-rule-lowercase = حرف صغير
passwd-rule-digit = رقم
passwd-rule-symbol = رمز

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = هذا الحساب بنمط عدم الوصول (zero-access). قبل تطبيق التغيير ستُعرض عليك عبارة استرداد — احفظها لتتمكن من الوصول إلى بياناتك إذا حدث أي خطأ.

# -- Actions -----------------------------------------------------------------
passwd-continue = متابعة
passwd-submit = تغيير كلمة المرور

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = احفظ عبارة الاسترداد
passwd-recovery-prose = اكتب هذه العبارة واحتفظ بها في مكان آمن. تُعرض قبل تغيير كلمة المرور لتتمكن من استرداد بياناتك حتى لو فُقدت كلمة المرور الجديدة. لا تُخزَّن على الخادم.
passwd-recovery-ack-label = لقد حفظت عبارة الاسترداد
passwd-recovery-ack-text = لقد حفظت عبارة الاسترداد في مكان آمن.

# -- Done phase --------------------------------------------------------------
passwd-done = تم تغيير كلمة المرور.
passwd-done-reencrypt = أُعيد تشفير بيانات اعتماد الخادم المخزّنة باستخدام كلمة المرور الجديدة.
passwd-done-rewrap = أُعيد تغليف مفاتيح عدم الوصول (zero-access) باستخدام كلمة المرور الجديدة.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = أدخل كلمة المرور الحالية
passwd-error-mismatch = كلمة المرور الجديدة وتأكيدها غير متطابقين
passwd-error-policy = تحتاج كلمة المرور الجديدة إلى { $rules }
passwd-error-ack-first = أكّد أولًا أنك حفظت عبارة الاسترداد
passwd-error-prepare-recovery = تعذّر تجهيز عبارة الاسترداد
passwd-error-change = تعذّر تغيير كلمة المرور
passwd-error-session-ended = انتهت جلستك. سجّل الدخول مرة أخرى.
passwd-error-not-configured = هذا الخادم غير مُعدّ لتغيير كلمات المرور. تواصل مع المسؤول.
passwd-error-contact-admin = تعذّر على الخادم إتمام التغيير. تواصل مع المسؤول.

# -- Forced change screen ----------------------------------------------------
passwd-forced-title = غيّر كلمة المرور للمتابعة
passwd-forced-signed-in-as = مسجّل الدخول باسم { $username }.
passwd-forced-explain = طلب أحد المسؤولين تغيير كلمة المرور لهذا الحساب. البريد والتقويم وجهات الاتصال غير متاحة حتى ينجح التغيير.
passwd-forced-backend-note = إذا رُفض التغيير رغم أن كلمة المرور الحالية صحيحة، فقد لا يتمكن هذا الخادم من تغيير كلمة المرور. تواصل مع المسؤول: يمكنه إعداد تغيير كلمات المرور أو إزالة هذا الشرط.
passwd-forced-sign-out = تسجيل الخروج
passwd-forced-still-required = قبل الخادم التغيير لكنه لا يزال يطلب تغيير كلمة المرور لهذا الحساب. تواصل مع المسؤول.
passwd-forced-reload-failed = تم تغيير كلمة المرور، لكن تعذّر تحميل حسابك. تحقق من اتصالك وحاول مرة أخرى.
passwd-forced-retry = حاول مرة أخرى
