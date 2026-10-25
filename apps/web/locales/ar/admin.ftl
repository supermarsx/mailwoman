# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = إدارة Mailwoman
admin-nav = أقسام الإدارة
admin-sign-out = تسجيل الخروج
admin-nav-domains = النطاقات
admin-nav-users = المستخدمون
admin-nav-integrations = التكاملات
admin-nav-observability = المراقبة
admin-nav-plugins = الإضافات
admin-nav-assist = مساعد الذكاء الاصطناعي
admin-nav-sso = تسجيل الدخول الموحّد
admin-nav-servermeta = بيانات الخادم الوصفية
admin-nav-rethread = الصيانة
admin-nav-2fa = فرض التحقق بخطوتين

admin-delete = حذف
admin-revoke = إبطال
admin-remove = إزالة
admin-saved = تم الحفظ.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = تسجيل دخول المسؤول
admin-login-note = تعمل هذه اللوحة ضمن جلسة مسؤول منفصلة. تنتهي الجلسة بعد 30 دقيقة من عدم النشاط، وبعد 12 ساعة من تسجيل الدخول كحدّ أقصى.
admin-login-session-ended = انتهت جلسة المسؤول. سجّل الدخول مرة أخرى.
admin-login-username = اسم مستخدم المسؤول
admin-login-password = كلمة المرور
admin-login-sign-in = تسجيل الدخول
admin-login-signing-in = جارٍ تسجيل الدخول…
admin-login-invalid = بيانات اعتماد المسؤول غير صالحة
admin-login-unreachable = تعذّر الوصول إلى الخادم

# -- Domains -----------------------------------------------------------------
admin-domains-title = النطاقات
admin-domains-load-error = تعذّر تحميل النطاقات
admin-domains-save-error = تعذّر حفظ النطاق
admin-domains-delete-error = تعذّر حذف النطاق
admin-domains-add = إضافة نطاق
admin-domains-name = اسم النطاق
admin-domains-name-placeholder = example.com
admin-domains-note = يُسجَّل النطاق باسمه. تُعرض الأسماء المسجّلة في شاشة فرض التحقق بخطوتين عندما تنطبق قاعدة على نطاق واحد. تسجيل نطاق لا يوجّه البريد ولا يصفّيه.
admin-domains-save = حفظ النطاق
admin-domains-empty = لا توجد نطاقات بعد.
admin-domains-delete-for = حذف { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = المستخدمون
admin-users-load-error = تعذّر تحميل المستخدمين
admin-users-provision-error = تعذّر تجهيز المستخدم
admin-users-flag-error = تعذّر تحديث العلامة
admin-users-revoke-error = تعذّر إبطال الجلسات
admin-users-provision = تجهيز مستخدم
admin-users-username = اسم المستخدم
admin-users-domain = النطاق
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = حصة البايتات (0 = غير محدودة)
admin-users-quota-msgs = حصة الرسائل (0 = غير محدودة)
admin-users-empty = لا يوجد مستخدمون بعد.
admin-users-col-account = الحساب
admin-users-col-quota = الحصة (بايت/رسائل)
admin-users-col-zeroaccess = عدم الوصول
admin-users-col-flags = العلامات
admin-users-col-sessions = الجلسات
admin-users-zeroaccess-for = عدم الوصول لـ { $account }
admin-users-zeroaccess-help = هذا سجل فقط. يشغّل المستخدم تخزين عدم الوصول ويوقفه بنفسه، في إعداداته، وبمفتاحه الخاص. تحديد هذا المربع لا يشفّر البريد المخزّن للحساب، وإلغاء تحديده لا يفك تشفيره.
admin-users-disable-for = تعطيل { $account }
admin-users-disabled = معطّل
admin-users-disabled-help = يمنع تسجيل الدخول وينهي كل جلسة مفتوحة لهذا الحساب. تُرفض مفاتيح API والرموز الخاصة به. لا يعطّل هذا صندوق البريد على خادم البريد: يظل بإمكان برامج البريد الأخرى الوصول إليه حتى تعطّله هناك أيضًا.
admin-users-force-change-for = فرض تغيير كلمة المرور لـ { $account }
admin-users-force-change = فرض التغيير
admin-users-force-change-help = يُوقَف المستخدم عند شاشة تغيير كلمة المرور، عند تسجيل الدخول التالي وفي الجلسات المفتوحة بالفعل، ولا يمكنه الوصول إلى البريد حتى ينجح التغيير. يتطلب هذا واجهة خلفية عاملة لتغيير كلمة المرور (MW_PASSWD_BACKEND). بدونها لا يمكن أن ينجح التغيير ويبقى المستخدم عند تلك الشاشة حتى تلغي تحديد هذا المربع.
admin-users-revoke-for = إبطال جلسات { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = التكاملات
admin-integrations-load-error = تعذّر تحميل التكاملات
admin-integrations-revoke-error = تعذّر إبطال المفتاح
admin-integrations-ldap = دليل LDAP / GAL
admin-integrations-nextcloud = جسر Nextcloud
admin-integrations-active = نشط
admin-integrations-configured = مُعدّ
admin-integrations-not-configured = غير مُعدّ
admin-integrations-unknown = الحالة غير معروفة
admin-integrations-config-note = تعني "مُعدّ" أن لدى هذا النشر إعدادات للتكامل: إدخال دليل LDAP مفعّل في قاعدة البيانات، أو متغيرات البيئة الثلاثة MW_NEXTCLOUD_*. يُقرأ كلاهما عند بدء تشغيل الخادم. لا تعني أنه تم الاتصال بالخدمة البعيدة أو أنها قابلة للوصول.
admin-integrations-webhooks = خطافات الويب
admin-integrations-webhooks-empty = لا توجد خطافات ويب مسجّلة.
admin-integrations-keys = مفاتيح API وMCP
admin-integrations-keys-empty = لم يُصدر أي مفتاح.
admin-integrations-col-prefix = البادئة
admin-integrations-col-account = الحساب
admin-integrations-col-scopes = النطاقات
admin-integrations-col-status = الحالة
admin-integrations-status-revoked = مُبطَل
admin-integrations-status-active = نشط
admin-integrations-revoke-key = إبطال المفتاح { $prefix }
admin-integrations-col-unattended = الإرسال غير المراقَب
admin-integrations-unattended-not-requested = لم يُطلب
admin-integrations-unattended-requested = مطلوب، لم تتم الموافقة
admin-integrations-unattended-approved = تمت الموافقة
admin-integrations-unattended-approve = موافقة
admin-integrations-unattended-withdraw = سحب
admin-integrations-unattended-approve-key = الموافقة على الإرسال غير المراقَب للمفتاح { $prefix }
admin-integrations-unattended-withdraw-key = سحب الموافقة على الإرسال غير المراقَب للمفتاح { $prefix }
admin-integrations-unattended-approve-title = الموافقة على الإرسال غير المراقَب لهذا المفتاح؟
admin-integrations-unattended-withdraw-title = سحب الموافقة لهذا المفتاح؟
admin-integrations-unattended-owner = المالك: { $account }
admin-integrations-unattended-key = المفتاح: { $prefix }
admin-integrations-unattended-scope = الأذونات: { $permissions }. أدوات MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = تعذّرت قراءة نطاق هذا المفتاح.
admin-integrations-scope-read = قراءة
admin-integrations-scope-send = إرسال
admin-integrations-scope-delete = حذف
admin-integrations-scope-mail = البريد
admin-integrations-scope-pim = التقويم والمهام والملاحظات وجهات الاتصال
admin-integrations-scope-none = لا شيء
admin-integrations-unattended-approve-meaning = الرسائل المرسلة بهذا المفتاح عبر MCP تُنقل دون أن يُطلقها شخص.
admin-integrations-unattended-withdraw-meaning = يُرفض أي إرسال بهذا المفتاح عبر MCP حتى تتم الموافقة على المفتاح مجددًا. لا تُنقل الرسالة ولا توضع في صندوق صادر المالك.
admin-integrations-unattended-approve-effect = تسري الموافقة فورًا، بدءًا من الإرسال التالي للمفتاح. لا حاجة لإعادة التشغيل.
admin-integrations-unattended-withdraw-effect = يسري السحب فورًا: يتوقف المفتاح عن الإرسال غير المراقَب بدءًا من إرساله التالي. لا حاجة لإعادة التشغيل.
admin-integrations-unattended-cancel = إلغاء
admin-integrations-unattended-saving = جارٍ الحفظ…
admin-integrations-unattended-approved-done = تمت الموافقة على الإرسال غير المراقَب للمفتاح { $prefix }. وهي سارية الآن.
admin-integrations-unattended-withdrawn-done = سُحبت الموافقة للمفتاح { $prefix }. لم يعد هذا المفتاح يرسل دون مراقبة.
admin-integrations-unattended-error-gone = المفتاح { $prefix } لم يعد موجودًا أو تم إبطاله. لم يتغير شيء.
admin-integrations-unattended-error-conflict = المفتاح { $prefix } مُبطَل أو أن مالكه لم يطلب الإرسال غير المراقَب. لم تتم الموافقة على شيء.
admin-integrations-unattended-error = تعذّر تغيير الموافقة للمفتاح { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = المراقبة
admin-obs-load-error = تعذّر تحميل بيانات المراقبة
admin-obs-export-error = تعذّر تصدير سجل التدقيق
admin-obs-ban-add-error = تعذّرت إضافة الحظر
admin-obs-unban-error = تعذّرت إزالة الحظر
admin-obs-telemetry-note = لا يُضبط التسجيل والقياس عن بُعد هنا. يُقرأ مرشّح السجل (MW_LOG) ومُجمِّع OTLP (MW_OTLP_ENDPOINT) ونقطة نهاية المقاييس (MW_METRICS_TOKEN) من بيئة الخادم عند بدء تشغيله.
admin-obs-audit = سجل التدقيق
admin-obs-export = تصدير JSONL
admin-obs-audit-empty = لا توجد إدخالات تدقيق.
admin-obs-col-time = الوقت
admin-obs-col-actor = المنفّذ
admin-obs-col-action = الإجراء
admin-obs-col-target = الهدف
admin-obs-bans = مراقب تسجيل الدخول / قائمة الحظر
admin-obs-bans-note = هذه القائمة سجل فقط. لا يرفض Mailwoman الاتصالات من العناوين الموجودة فيها، ولا أثر لأي إدخال على تسجيل الدخول. كل تسجيل دخول فاشل للمسؤول يكتب سطرًا في سجل الخادم يمكن لسجن fail2ban التصرف بناءً عليه، وخمس محاولات فاشلة من عنوان واحد تضيف ذلك العنوان هنا. يتم الحظر الفعلي بواسطة ذلك السجن أو بواسطة جدار الحماية لديك.
admin-obs-ban-add = إضافة حظر
admin-obs-ban-ip = عنوان IP
admin-obs-ban-reason = السبب
admin-obs-ban-btn = حظر IP
admin-obs-bans-empty = لا توجد حظورات نشطة.
admin-obs-unban-for = إلغاء حظر { $ip }
admin-obs-unban-btn = إلغاء الحظر

# -- Plugins (§22) -----------------------------------------------------------
admin-plugins-title = الإضافات
admin-plugins-intro = تعمل إضافات المحرك في بيئة WebAssembly معزولة مقيّدة بالقدرات. وافق على الإضافة قبل أن يمكن تفعيلها، وامنحها فقط القدرات التي تحتاجها.
admin-plugins-empty = لا توجد إضافات مسجّلة.
admin-plugins-signed = موقّعة
admin-plugins-unsigned = غير موقّعة
admin-plugins-approved = موافق عليها
admin-plugins-enabled = مفعّلة
admin-plugins-approve = موافقة
admin-plugins-enable = تفعيل
admin-plugins-disable = تعطيل
admin-plugins-net = الشبكة: { $hosts }
admin-plugins-limits = الحدود: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = الحدود: { $memory } MiB · { $deadline } ms · { $fuel } وقود
admin-plugins-allow-unsigned-for = السماح بالإضافة غير الموقّعة { $name }
admin-plugins-allow-unsigned = السماح بتشغيل هذه الإضافة غير الموقّعة
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
admin-allowlist-title = قائمة السماح لإضافات الجهات الخارجية
admin-allowlist-intro = لا يُحمَّل مكوّن من جهة خارجية (غير أساسي) إلا بعد أن يوافق المسؤول على بصمة SHA-256 الدقيقة الخاصة به. يحسب الخادم البصمة أدناه من بايتات المكوّن على القرص؛ والموافقة عليها تثبّت تلك البايتات بعينها. المكوّنات الأساسية مثبّتة في البناء ولا تُدار هنا.
admin-allowlist-load-error = تعذّر تحميل قائمة السماح للإضافات.
admin-allowlist-present-heading = المكوّنات على القرص
admin-allowlist-present-empty = لا توجد مكوّنات من جهات خارجية. ضع مكوّنًا في دليل إضافات الجهات الخارجية ليظهر هنا.
admin-allowlist-digest-label = بصمة SHA-256 المحسوبة
admin-allowlist-status-approved = موافق عليه
admin-allowlist-status-pending = غير موافق عليه
admin-allowlist-status-firstparty = أساسي
admin-allowlist-unsigned-note = مقبول بتثبيت البصمة. لا يحمل هذا المكوّن توقيعًا؛ الموافقة تثق بالبايتات الدقيقة التي تظهر بصمتها، وهذا هو الوضع المتوقع لمكوّن تمت الموافقة عليه بهذه الطريقة.
admin-allowlist-highpower-note = لا يمكن منح القدرات عالية الصلاحية ({ $caps }) لإضافة من جهة خارجية. يرفضها الخادم بغض النظر عن الموافقة؛ فهي متاحة للمكوّنات الأساسية فقط.
admin-allowlist-firstparty-note = يطابق هذا المعرّف مكوّنًا أساسيًا. التثبيت الأساسي له الأولوية دائمًا، لذا لا يمكن الموافقة على هذا المعرّف كجهة خارجية.
admin-allowlist-approve = الموافقة على البصمة
admin-allowlist-approve-for = الموافقة على بصمة { $id }
admin-allowlist-revoke = إبطال
admin-allowlist-revoke-for = إبطال تثبيت { $id }
admin-allowlist-uninstall = إلغاء التثبيت
admin-allowlist-uninstall-for = إلغاء تثبيت { $id }
admin-allowlist-pins-heading = التثبيتات الموافق عليها والمُبطَلة
admin-allowlist-pins-empty = لا توجد تثبيتات مسجّلة.
admin-allowlist-pin-approved-by = وافق عليه { $by } في { $at }
admin-allowlist-pin-revoked = مُبطَل
admin-allowlist-cancel = إلغاء

admin-allowlist-approve-title = الموافقة على تحميل هذا المكوّن؟
admin-allowlist-approve-detail = الموافقة تثبّت البايتات الدقيقة التي تظهر بصمة SHA-256 الخاصة بها أدناه. بعد الموافقة، لن تُحمَّل لهذا المعرّف إلا البايتات المطابقة لهذه البصمة؛ وأي تغيير في المكوّن ينتج بصمة مختلفة ولن يُحمَّل حتى تتم الموافقة عليه مجددًا. الموافقة وحدها لا تمنح أي قدرات.
admin-allowlist-approve-confirm = الموافقة على البصمة

admin-allowlist-revoke-title = إبطال هذا التثبيت؟
admin-allowlist-revoke-detail = الإبطال يزيل الموافقة على هذه البصمة ويعطّل الإضافة. يسري عند التحميل التالي؛ ولا تتوقف النسخة العاملة حاليًا حتى ذلك الحين.
admin-allowlist-revoke-confirm = إبطال التثبيت

admin-allowlist-uninstall-title = إلغاء تثبيت هذه الإضافة؟
admin-allowlist-uninstall-detail = إلغاء التثبيت يحذف بيانات المفتاح/القيمة المخزّنة للإضافة لكل الحسابات، ويزيل تثبيتاتها في قائمة السماح، ويعطّلها. لا يُحذف ملف المكوّن على القرص؛ ويمكن الموافقة عليه مجددًا لاحقًا.
admin-allowlist-uninstall-confirm = إلغاء تثبيت الإضافة

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = تسجيل الدخول الموحّد
admin-sso-intro = أعدّ واجهات تسجيل الدخول الخلفية OIDC وSAML 2.0. تظهر الواجهات المفعّلة كأزرار "تسجيل الدخول باستخدام…" في صفحة دخول صندوق البريد، على مستوى النشر كله أو لنطاق واحد.
admin-sso-add = إضافة واجهة تسجيل دخول خلفية
admin-sso-edit = تعديل
admin-sso-create = إضافة واجهة خلفية
admin-sso-update = حفظ التغييرات
admin-sso-cancel = إلغاء
admin-sso-empty = لا توجد واجهات SSO خلفية مُعدّة.
admin-sso-load-error = تعذّر تحميل واجهات SSO الخلفية.
admin-sso-save-error = تعذّر حفظ الواجهة الخلفية.
admin-sso-delete-error = تعذّر حذف الواجهة الخلفية.

admin-sso-id = معرّف الواجهة الخلفية
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = الاسم المعروض
admin-sso-display-name-placeholder = تسجيل الدخول باستخدام Acme SSO
admin-sso-kind = البروتوكول
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = النطاق
admin-sso-scope-deployment = النشر بالكامل
admin-sso-scope-domain = نطاق واحد
admin-sso-domain = النطاق
admin-sso-domain-placeholder = example.org
admin-sso-enabled = مفعّلة
admin-sso-first-login = سياسة أول تسجيل دخول
admin-sso-policy-allowlist = قائمة السماح (رفض المستخدمين غير المعروفين)
admin-sso-policy-autocreate = إنشاء الحسابات تلقائيًا عند أول تسجيل دخول

admin-sso-issuer = عنوان URL للجهة المُصدِرة
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = معرّف العميل
admin-sso-client-secret = سر العميل
admin-sso-secret-unchanged = اتركه فارغًا للاحتفاظ بالسر المخزّن
admin-sso-redirect = عنوان URL لإعادة التوجيه
admin-sso-scopes = النطاقات
admin-sso-metadata = بيانات SP الوصفية

admin-sso-sp-entity-id = معرّف كيان SP
admin-sso-acs-url = عنوان URL لـ ACS
admin-sso-idp-metadata-url = عنوان URL لبيانات IdP الوصفية
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = بيانات IdP الوصفية بصيغة XML
admin-sso-idp-metadata-xml-placeholder = الصق بيانات IdP الوصفية بصيغة XML، أو استخدم عنوان URL أعلاه
admin-sso-idp-sso-url = عنوان URL لـ SSO لدى IdP
admin-sso-idp-slo-url = عنوان URL لتسجيل الخروج (SLO) لدى IdP
admin-sso-idp-certs = شهادات توقيع IdP (PEM)
admin-sso-idp-certs-placeholder = شهادة PEM واحدة لكل كتلة، مفصولة بسطر فارغ
admin-sso-nameid-format = تنسيق NameID
admin-sso-want-signed = طلب تأكيدات موقّعة
admin-sso-want-encrypted = طلب تأكيدات مشفّرة

admin-sso-claims = تعيين المطالبات
admin-sso-claim-email = مطالبة البريد الإلكتروني
admin-sso-claim-username = مطالبة اسم المستخدم
admin-sso-claim-display = مطالبة الاسم المعروض
admin-sso-claim-groups = مطالبة المجموعات

admin-sso-badge-enabled = مفعّلة
admin-sso-badge-disabled = معطّلة
admin-sso-enable = تفعيل
admin-sso-disable = تعطيل
admin-sso-delete = حذف
admin-sso-enable-for = تفعيل { $name }
admin-sso-disable-for = تعطيل { $name }
admin-sso-delete-for = حذف { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = بيانات الخادم الوصفية
admin-servermeta-intro = عرض وتعديل تعليقات الخادم وفق RFC 5464 لحساب مُجهَّز. تُكتب التغييرات مباشرة إلى خادم البريد، وهو الذي يقرر ما إذا كان يحق للحساب تعيينها.
admin-servermeta-account = الحساب
admin-servermeta-select-option = اختر حسابًا…
admin-servermeta-select-prompt = اختر حسابًا لعرض تعليقات الخادم الخاصة به وتعديلها.
admin-servermeta-load-error = تعذّر تحميل قائمة الحسابات.
admin-servermeta-no-accounts = لا توجد حسابات مُجهَّزة.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = إعادة ترتيب سلاسل صندوق البريد
admin-rethread-intro = يعيد تشغيل ترتيب المحادثات في سلاسل (JWZ) على البريد المخزّن لحساب مُجهَّز ويعيد تعيين مفاتيح تجميع السلاسل. هذا إجراء صيانة لمرة واحدة، وليس شيئًا يعمل تلقائيًا.
admin-rethread-account = الحساب
admin-rethread-select-option = اختر حسابًا…
admin-rethread-no-accounts = لا توجد حسابات مُجهَّزة.
admin-rethread-load-error = تعذّر تحميل قائمة الحسابات.
admin-rethread-run = إعادة ترتيب السلاسل
admin-rethread-confirm-title = إعادة ترتيب سلاسل صندوق البريد هذا؟
admin-rethread-confirm-warning = إعادة الترتيب تعيد تعيين مفاتيح تجميع المحادثات لهذا الحساب. قد تندمج السلاسل الحالية أو تنقسم، وقد يرى المستخدمون المحادثات مُعاد تجميعها.
admin-rethread-confirm-detail = هذا إجراء صيانة لمرة واحدة. يعمل مرة واحدة الآن؛ ويمكن إعادة تشغيله بأمان ولا يحذف أي بريد.
admin-rethread-confirm = إعادة ترتيب السلاسل
admin-rethread-running = جارٍ إعادة الترتيب…
admin-rethread-cancel = إلغاء
admin-rethread-summary = أُعيد ترتيب { $messages } رسالة في { $threads } سلسلة عبر { $accounts } حساب؛ نُقلت { $reassigned } رسالة إلى سلسلة مختلفة.
admin-rethread-error = فشل إجراء إعادة الترتيب. لم يتغير تجميع السلاسل إذا رفض الخادم الطلب؛ تحقّق من سجلات الخادم وحاول مرة أخرى.

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = فهرس البحث
admin-searchindex-intro = يقرأ البحث فهرسًا مبنيًا من البريد المخزّن. يعيد الخادم بناءه عند البدء إذا لم يطابق البريد المخزّن. لا يُفهرس بريد حسابات عدم الوصول، لذا لا يجد البحث شيئًا لتلك الحسابات.
admin-searchindex-count = { $documents } من أصل { $messages } رسالة مخزّنة مُفهرَسة.
admin-searchindex-on-disk = يُحفظ الفهرس على القرص.
admin-searchindex-in-memory = يُحفظ الفهرس في الذاكرة ويُعاد بناؤه في كل مرة يبدأ فيها الخادم.
admin-searchindex-progress = إعادة بناء قيد التشغيل: اكتمل { $done } من أصل { $total } رسالة.
admin-searchindex-run = إعادة بناء فهرس البحث
admin-searchindex-running = جارٍ إعادة البناء…
admin-searchindex-summary = قُرئت { $messages } رسالة مخزّنة في { $accounts } حساب: { $indexed } مُفهرَسة، { $removed } أُزيلت من الفهرس، { $failed } تعذّرت قراءتها. { $zeroAccess } حساب عدم وصول تُرك دون فهرسة.
admin-searchindex-busy = إعادة بناء قيد التشغيل بالفعل. انتظر حتى تنتهي.
admin-searchindex-error = فشلت إعادة البناء. يحتفظ الفهرس بما كان فيه؛ تحقّق من سجلات الخادم وحاول مرة أخرى.
admin-searchindex-status-error = تعذّرت قراءة حالة فهرس البحث.
admin-searchindex-unavailable = يعمل هذا الخادم في وضع الوكيل ولا يوجد لديه فهرس بحث.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = فرض التحقق بخطوتين
admin-2fa-intro = فرض عامل ثانٍ (مفتاح مرور أو تطبيق مصادقة) لتسجيل الدخول. يمكن لأي مستخدم تسجيل عامل بنفسه؛ وفرضه هنا يُلزم الحسابات المشمولة بالتسجيل عند تسجيل دخولها التالي.
admin-2fa-load-error = تعذّر تحميل سياسة التحقق بخطوتين.
admin-2fa-save-error = تعذّر حفظ سياسة التحقق بخطوتين.
admin-2fa-global = فرض التحقق بخطوتين على النشر بالكامل
admin-2fa-global-label = فرض التحقق بخطوتين على النشر بالكامل
admin-2fa-global-note = عند التفعيل، يجب أن يكون لكل حساب عامل ثانٍ. يمكن لقاعدة خاصة بنطاق أدناه أيضًا فرضه لنطاق واحد دون فرضه في كل مكان.
admin-2fa-domains-heading = المتطلبات لكل نطاق
admin-2fa-domains-empty = لم تُعيَّن متطلبات لأي نطاق.
admin-2fa-col-domain = النطاق
admin-2fa-col-require = مطلوب
admin-2fa-require-for = فرض التحقق بخطوتين لـ { $domain }
admin-2fa-add-domain = النطاق
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = فرض التحقق بخطوتين لهذا النطاق
admin-2fa-add-require-label = فرض التحقق بخطوتين لهذا النطاق
admin-2fa-add-save = إضافة قاعدة نطاق

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = الاتصالات الصادرة
admin-egress-heading = مسارات الوكيل الصادرة
admin-egress-intro = وجّه عمليات الجلب الصادرة من الخادم عبر وكيل HTTP CONNECT أو SOCKS5. يواصل Mailwoman تطبيق سياسة العناوين الخاصة به — ولا يُطلب من الوكيل أبدًا تحليل أي اسم.
admin-egress-empty = لا توجد مسارات مُعدّة. تتم عمليات الجلب الصادرة مباشرة.
admin-egress-none-active = لا يوجد مسار قيد الاستخدام. تتم عمليات الجلب الصادرة مباشرة. لا يحمل المسار المحفوظ حركة مرور إلا بعد اختيار "استخدام هذا المسار".
admin-egress-in-use = تستخدم عمليات الجلب الصادرة المسار "{ $id }".
admin-egress-deactivate = إيقاف استخدام الوكيل
admin-egress-activate = استخدام هذا المسار
admin-egress-activate-for = استخدام المسار { $id }
admin-egress-activate-error = تعذّر التبديل إلى المسار
admin-egress-deactivate-error = تعذّر إيقاف استخدام المسار
admin-egress-col-state = الحالة
admin-egress-state-active = قيد الاستخدام
admin-egress-state-inactive = غير مستخدم
admin-egress-test-scope = يمكن اختبار المسار قيد الاستخدام فقط. يجلب الاختبار عنوان URL ثابتًا واحدًا عبر المسار ويُبلغ عن المدى الذي بلغه؛ ولا يغيّر شيئًا.
admin-egress-probe-url = تم جلب: { $url }
admin-egress-load-error = تعذّر تحميل مسارات الاتصالات الصادرة
admin-egress-save-error = تعذّر حفظ المسار
admin-egress-delete-error = تعذّر حذف المسار
admin-egress-invalid = المعرّف والمضيف ومنفذ أكبر من صفر مطلوبة.
admin-egress-username-required = كلمة المرور تتطلب اسم مستخدم؛ وبدونه سيُصادق المسار باسم لا أحد.
admin-egress-col-id = المعرّف
admin-egress-col-route = المسار
admin-egress-col-username = اسم المستخدم
admin-egress-col-credentials = كلمة المرور
admin-egress-col-plaintext = نص غير مشفّر
admin-egress-col-actions = الإجراءات
admin-egress-cred-set = مُعيَّنة
admin-egress-cred-none = لا شيء
admin-egress-plaintext-on = مسموح
admin-egress-plaintext-off = مرفوض
admin-egress-add-heading = إضافة مسار
admin-egress-edit-heading = تعديل المسار
admin-egress-id = المعرّف
admin-egress-scheme = المخطط
admin-egress-host = المضيف
admin-egress-port = المنفذ
admin-egress-username = اسم المستخدم
admin-egress-password = كلمة المرور
admin-egress-password-unchanged = اتركها فارغة للاحتفاظ بكلمة المرور المخزّنة
admin-egress-allow-plaintext = السماح بمصادر http غير المشفّرة عبر هذا المسار
admin-egress-allow-plaintext-note = معطّل افتراضيًا. مصدر https هو ما يمنع الوكيل من قراءة حركة المرور التي يحملها؛ والسماح بـ http يعني أن الوكيل يراها.
admin-egress-test = اختبار
admin-egress-testing = جارٍ الاختبار…
admin-egress-test-failed = تعذّر تشغيل الاختبار، لذا لا يُعرف شيء عن هذا المسار.
admin-egress-outcome-connected = متصل
admin-egress-outcome-auth-rejected = رُفضت المصادقة
admin-egress-outcome-refused-by-policy = مرفوض بموجب السياسة
admin-egress-outcome-dns-failed = تعذّر تحليل الاسم
admin-egress-outcome-unreachable = يتعذّر الوصول إليه
admin-egress-outcome-origin-tls-failed = فشل TLS لدى المصدر
admin-egress-outcome-route-invalid = المسار غير صالح
admin-egress-outcome-unknown = نتيجة غير معروفة
admin-egress-stage-dns = توقف عند تحليل الاسم
admin-egress-stage-connect = توقف أثناء الاتصال بالوكيل
admin-egress-stage-tunnel = توقف أثناء فتح النفق
admin-egress-stage-origin = توقف عند المصدر
admin-egress-stage-unknown = توقف عند مرحلة غير معروفة
admin-egress-proxied-yes = مرّ عبر الوكيل
admin-egress-proxied-no = لم يمرّ عبر الوكيل
