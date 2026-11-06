# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = السرعة
privacy-speed-unavailable = أرقام السرعة غير متاحة من هذا الخادم.
privacy-speed-empty = لم تُقدَّم أي صور في هذا النطاق بعد.
privacy-speed-off = تتبع السرعة متوقف (الإعداد: تتبع سرعة وكيل الصور). لا يُسجَّل أي شيء جديد.
privacy-speed-verdict-healthy = سليم
privacy-speed-verdict-slow = بطيء
privacy-speed-verdict-degraded = متدهور
privacy-speed-verdict-noData = لا توجد بيانات كافية
privacy-speed-reason-ok = الصور المجلوبة وإصابات ذاكرة التخزين المؤقت ضمن الحدود أدناه، وفشل عدد قليل من الطلبات.
privacy-speed-reason-failureRate = انتهى { $failurePct } % من الطلبات بانشغال أو انتهاء مهلة أو فشل من المصدر.
privacy-speed-reason-missP95 = الصور التي توجّب جلبها بطيئة: المئين 95 لها أعلى من الحد أدناه.
privacy-speed-reason-hitP95 = الصور المخزَّنة مؤقتًا بطيئة الاستجابة: المئين 95 لها أعلى من الحد أدناه.
privacy-speed-reason-fewRequests = أقل من { $min } طلب في هذا النطاق، وهو عدد قليل جدًا للحكم.
privacy-speed-thresholds = بطيء عند: المئين 95 للصورة المجلوبة { $slowMiss }، أو المئين 95 للصورة المخزَّنة مؤقتًا { $slowHit }، أو { $slowFail } % انشغال أو انتهاء مهلة أو فشل. متدهور عند: المئين 95 للصورة المجلوبة { $degradedMiss } أو فشل { $degradedFail } %. حدود ثابتة وليست إعدادات.
privacy-speed-median = الوسيط
privacy-speed-p95 = المئين 95
privacy-speed-p99 = المئين 99
privacy-speed-max = الأبطأ
privacy-speed-requests = الطلبات المقيسة
privacy-speed-busy = ردود الانشغال
privacy-speed-timeouts = انتهاءات المهلة
privacy-speed-upstream-errors = أخطاء المصدر
privacy-speed-hit = إصابات ذاكرة التخزين المؤقت
privacy-speed-miss = الصور المجلوبة (إخفاقات ذاكرة التخزين المؤقت)
privacy-speed-split-count = { $count } طلب
privacy-speed-split-times = الوسيط { $p50 }، المئين 95 { $p95 }، الأبطأ { $max }.
privacy-speed-chart = وقت تقديم صورة
privacy-speed-chart-miss = وقت تقديم صورة مجلوبة
privacy-speed-chart-unit = من استلام الطلب إلى جاهزية الرد
privacy-speed-phases = أين يذهب الوقت
privacy-speed-phases-help = للطلبات التي لم تكن إصابات في ذاكرة التخزين المؤقت: المدة التي استغرقتها كل خطوة ونسبتها من إجمالي وقتها.
privacy-speed-phase = الخطوة
privacy-speed-share = نسبة الوقت
privacy-speed-phase-queue = انتظار السعة
privacy-speed-phase-cache = البحث في ذاكرة التخزين المؤقت
privacy-speed-phase-upstream = الجلب من المصدر
privacy-speed-phase-decode = فك الترميز وإعادة الترميز
privacy-speed-phase-other = كل ما عدا ذلك
privacy-speed-note = تُقاس على عملية هذا الخادم فقط، وتُحفظ في الذاكرة وتكون فارغة بعد إعادة التشغيل. المئينات دقيقة في حدود { $error } %. لا تُخزَّن أي عناوين URL أو مضيفين أو حسابات.
