# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: rules. Source: ../en/rules.ftl

rules-title = القواعد وعوامل التصفية
rules-intro = إدارة نصوص تصفية البريد والقواعد المرئية. تعتمد الميزات المتاحة على هذا الحساب وإعدادات المسؤول.
rules-empty = لا توجد قواعد بعد.
rules-unavailable = خادم البريد هذا لا يدعم قواعد البريد.
rules-load-failed = تعذّر تحميل القواعد.
rules-new = قاعدة جديدة
rules-list-label = قواعد البريد
rules-enabled = مفعّلة
rules-delete-named = حذف القاعدة { $name }

rules-builder-title = منشئ القواعد
rules-builder-hint = حدّد قاعدة لتعديلها، أو أنشئ قاعدة جديدة.
rules-name-label = اسم القاعدة
rules-match-legend = متى تُشغَّل
rules-match-all = مطابقة كل الشروط
rules-match-any = مطابقة أي شرط
rules-conditions-legend = الشروط
rules-actions-legend = الإجراءات
rules-add-condition = إضافة شرط
rules-add-action = إضافة إجراء
rules-remove-condition = إزالة الشرط
rules-remove-action = إزالة الإجراء
rules-cond-type-label = الحقل
rules-cond-op-label = العامل
rules-cond-value-label = قيمة المطابقة
rules-act-type-label = الإجراء
rules-act-value-label = قيمة الإجراء
rules-save = حفظ القاعدة
rules-editing = تعديل { $name }
rules-save-refused = لم تُحفظ القاعدة: { $reason }

rules-cond-from = من
rules-cond-to = إلى
rules-cond-subject = الموضوع
rules-cond-thread-unsupported = سلسلة الرسائل { $op } { $value }: شروط سلاسل الرسائل غير مدعومة، لذا لا يمكن حفظ القاعدة حتى يُزال هذا الشرط.
rules-op-contains = يحتوي على
rules-op-is = يساوي

rules-act-move = نقل إلى صندوق بريد
rules-act-tag = إضافة وسم
rules-act-archive = أرشفة
rules-act-suppress = كتم الإشعار
rules-act-stop = إيقاف المعالجة

rules-runs-server = الخادم (Sieve)
rules-runs-server-detail = تُرفع إلى خادم البريد وتُطبَّق هناك.
rules-runs-engine = المحرك
rules-runs-engine-detail = يُطبق عند مزامنة الرسائل الواردة الجديدة؛ تنتظر الردود وعمليات النقل المزامنة التالية.

rules-panes-label = أدوات القواعد
rules-tab-builder = المنشئ
rules-tab-raw = Sieve الخام
rules-tab-dryrun = تشغيل تجريبي

rules-raw-label = مصدر Sieve الخام
rules-raw-reset = إعادة التعيين إلى المُولَّد
rules-lint-clean = لم يُعثر على أي مشكلات.

rules-dryrun-title = تشغيل تجريبي
rules-dryrun-help = أدخل رسالة نموذجية لمعاينة القواعد التي ستُنفَّذ.
rules-dryrun-results = نتائج التشغيل التجريبي
rules-dryrun-none = لا توجد قواعد مفعّلة لتقييمها.
rules-dryrun-nomatch = لا تطابق
rules-dryrun-stopped = لم يُوصل إليها (أوقفت قاعدة سابقة المعالجة)

rules-scripts-label = نصوص Sieve
rules-script-active = نشط
rules-script-new = نص جديد
rules-script-export = تصدير النص
rules-script-import = استيراد النص
rules-script-too-large = يتجاوز النص الحد الأقصى للحجم.
rules-script-read-only = عطّل المسؤول التحرير المباشر للنصوص.
rules-script-validate = التحقق من النص
rules-script-deactivate = إلغاء التنشيط
rules-script-activate = تنشيط
rules-script-saved = تم حفظ النص.

rules-vacation-title = رد الغياب
rules-vacation-enabled = تفعيل الردود التلقائية
rules-vacation-from = البداية (UTC)
rules-vacation-to = النهاية (UTC)
rules-vacation-body = نص الرد
rules-vacation-saved = تم حفظ رد الغياب.
rules-script-template = قالب النص البرمجي
rules-forward-title = إعادة التوجيه
rules-forward-address = إعادة التوجيه إلى
rules-forward-copy = الاحتفاظ بنسخة في هذا الحساب
rules-forward-refused = لا يسمح المسؤول بإعادة التوجيه إلى هذا النطاق.
