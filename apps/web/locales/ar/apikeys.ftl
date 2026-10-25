# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = مفاتيح API
apikeys-heading = مفاتيح API
apikeys-intro = أنشئ مفاتيح محدودة النطاق للبرامج النصية والتكاملات. يُخزَّن كل مفتاح مُجزّأً (hashed)، ويُعرض مرة واحدة، ويمكن إلغاؤه بشكل منفرد. امنح أقل نطاق يكفي للعمل.

apikeys-label = التسمية
apikeys-label-aria = تسمية المفتاح
apikeys-label-placeholder = مثلًا: برنامج النسخ الاحتياطي

apikeys-create = إنشاء مفتاح
apikeys-saved = لقد حفظته

apikeys-reveal-warning = انسخ هذا السر الآن — يُعرض مرة واحدة ولا يمكن استرجاعه مرة أخرى.
apikeys-copy = نسخ
apikeys-copy-aria = نسخ السر إلى الحافظة
apikeys-copied = نُسخ إلى الحافظة

apikeys-error-need-label = أعطِ المفتاح تسمية لتتعرّف عليه لاحقًا
apikeys-error-create = تعذّر إنشاء المفتاح
apikeys-error-revoke = تعذّر إلغاء المفتاح

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = المفاتيح الحالية
apikeys-existing = المفاتيح الحالية
apikeys-none = لا توجد مفاتيح بعد.
apikeys-created = أُنشئ في { $date }
apikeys-last-used = آخر استخدام { $date }
apikeys-never-used = لم يُستخدم قط
apikeys-revoked-at = أُلغي في { $date }
apikeys-revoke = إلغاء
apikeys-status-active = نشط
apikeys-status-revoked = ملغى

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = النطاق
apikeys-capabilities = الصلاحيات
apikeys-cap-read = قراءة
apikeys-cap-send = إرسال
apikeys-cap-delete = حذف
apikeys-surface = المجال
apikeys-surface-mail-label = البريد
apikeys-surface-pim-label = PIM (التقويم / المهام / الملاحظات / جهات الاتصال)
apikeys-accounts = الحسابات
apikeys-all-accounts = جميع الحسابات
apikeys-folders = المجلدات
apikeys-all-folders = جميع المجلدات
apikeys-folder-ids = معرّفات المجلدات (مفصولة بفواصل)
apikeys-folder-subset-aria = مجموعة فرعية من المجلدات
apikeys-constraints = القيود
apikeys-expiry = انتهاء الصلاحية (RFC 3339، فارغ = بلا انتهاء)
apikeys-expiry-aria = انتهاء الصلاحية
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = حد المعدل (طلبات/دقيقة، فارغ = بلا حد)
apikeys-rate-limit-aria = حد المعدل
apikeys-ip-allowlist = قائمة عناوين IP المسموح بها (CIDR/IP، مفصولة بفواصل؛ فارغ = أي عنوان)
apikeys-ip-allowlist-aria = قائمة عناوين IP المسموح بها
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = قراءة
apikeys-verb-send = إرسال
apikeys-verb-delete = حذف
apikeys-summary-mail = البريد
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = بلا أفعال
apikeys-summary-nothing = لا شيء
apikeys-summary-none = لا يوجد
apikeys-summary-verbs-on = { $verbs } على { $surfaces }
apikeys-summary-all-accounts = جميع الحسابات
apikeys-summary-accounts = الحسابات: { $ids }
apikeys-summary-all-folders = جميع المجلدات
apikeys-summary-folders = المجلدات: { $ids }
apikeys-summary-expires = ينتهي في { $date }
apikeys-summary-rate = { $n } طلب/دقيقة
apikeys-summary-ips = عناوين IP: { $ips }
apikeys-summary-mcp-tools = أدوات MCP: { $tools }
apikeys-summary-unattended = إرسال بلا إشراف

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = مفاتيح MCP
apikeys-mcp-heading = مفاتيح MCP
apikeys-mcp-intro = امنح وكيل ذكاء اصطناعي (عبر Model Context Protocol) الأدوات التي تختارها بالضبط. تُوسَم نصوص الرسائل المُعادة إلى الوكيل بأنها مدخلات غير موثوقة. تُمنح كل أداة بشكل منفرد.
apikeys-mcp-label-aria = تسمية مفتاح MCP
apikeys-mcp-label-placeholder = مثلًا: وكيل المساعد
apikeys-mcp-create = إنشاء مفتاح MCP
apikeys-mcp-reveal-warning = انسخ هذا السر الآن — يُعرض مرة واحدة.
apikeys-mcp-error-need-label = أعطِ مفتاح MCP تسمية
apikeys-mcp-error-need-tool = امنح أداة واحدة على الأقل
apikeys-mcp-error-create = تعذّر إنشاء مفتاح MCP

apikeys-tools = الأدوات
apikeys-tools-group = أدوات MCP
apikeys-outbox-suffix = (عبر صندوق الصادر)

apikeys-unattended-aria = إرسال بلا إشراف
apikeys-unattended-allow = السماح بالإرسال بلا إشراف (تجاوز صندوق الصادر — يتطلب توقيعًا مقابلًا من المسؤول)
apikeys-unattended-pending = يبقى الإرسال بلا إشراف غير نشط حتى يوقّع أحد المسؤولين على هذا المفتاح توقيعًا مقابلًا.
apikeys-unattended-disclosure = افتراضيًا، يصل الإرسال الممنوح إلى صندوق الصادر وينتظر تأكيدك في التطبيق — لا يمكن للأتمتة الإرسال من تلقاء نفسها. الإرسال بلا إشراف يُزيل خطوة التأكيد البشري هذه، فيتمكن هذا المفتاح من إرسال البريد دون تأكيد. ويتطلب أيضًا توقيعًا مقابلًا من أحد المسؤولين على المفتاح. لا تمنحه إلا لأتمتة تثق بها تمامًا؛ فمفتاح إرسال بلا إشراف مُخترَق يمكنه إرسال البريد باسمك دون أي مطالبة.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = البحث في البريد
apikeys-tool-mail-search-desc = البحث في الرسائل. تُوسَم النتائج بأنها مدخلات غير موثوقة.
apikeys-tool-mail-read-label = قراءة البريد
apikeys-tool-mail-read-desc = قراءة نص رسالة. تُوسَم النصوص بأنها مدخلات غير موثوقة.
apikeys-tool-folders-list-label = عرض المجلدات
apikeys-tool-folders-list-desc = عرض مجلدات صندوق البريد.
apikeys-tool-drafts-create-label = إنشاء مسودات
apikeys-tool-drafts-create-desc = إنشاء مسودة رسالة (لا تُرسل تلقائيًا أبدًا).
apikeys-tool-mail-send-label = إرسال البريد
apikeys-tool-mail-send-desc = وضع رسالة في صندوق الصادر لتأكيدها داخل التطبيق.
apikeys-tool-calendar-read-label = قراءة التقويم
apikeys-tool-calendar-read-desc = قراءة أحداث التقويم.
apikeys-tool-calendar-propose-label = اقتراح أحداث
apikeys-tool-calendar-propose-desc = اقتراح أحداث التقويم (دون اعتمادها).
apikeys-tool-tasks-read-label = قراءة المهام
apikeys-tool-tasks-read-desc = قراءة المهام.
apikeys-tool-tasks-write-label = كتابة المهام
apikeys-tool-tasks-write-desc = إنشاء المهام أو تحديثها.
apikeys-tool-contacts-read-label = قراءة جهات الاتصال
apikeys-tool-contacts-read-desc = قراءة جهات الاتصال.
